//! Retry executor implementation.
//!
//! Wraps component execution with retry logic, supporting various
//! backoff strategies.

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::workflow::component::{Component, ComponentOutput, ComponentStatus};
use crate::workflow::context::DataContext;
use crate::workflow::executor::{BoxedExecutor, Executor};
use async_trait::async_trait;
use std::time::Duration;

/// Retry strategy for failed executions.
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Linear backoff: delay = base_delay * attempt
    Linear,
    /// Exponential backoff: delay = base_delay * (multiplier ^ attempt)
    Exponential { multiplier: f64 },
}

impl Default for RetryStrategy {
    fn default() -> Self {
        RetryStrategy::Exponential { multiplier: 2.0 }
    }
}

/// Retry executor that wraps another executor with retry logic.
pub struct RetryExecutor {
    inner: BoxedExecutor,
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
    strategy: RetryStrategy,
}

impl RetryExecutor {
    /// Create a new retry executor with default exponential backoff.
    pub fn new(inner: BoxedExecutor, max_retries: u32, base_delay: Duration) -> Self {
        Self {
            inner,
            max_retries,
            base_delay,
            max_delay: Duration::from_secs(60),
            strategy: RetryStrategy::default(),
        }
    }

    /// Set the retry strategy
    pub fn with_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the maximum delay between retries
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Calculate the delay for a given attempt number
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = match &self.strategy {
            RetryStrategy::Fixed => self.base_delay,
            RetryStrategy::Linear => self.base_delay * attempt,
            RetryStrategy::Exponential { multiplier } => {
                let factor = multiplier.powi(attempt as i32 - 1);
                Duration::from_secs_f64(self.base_delay.as_secs_f64() * factor)
            }
        };
        std::cmp::min(delay, self.max_delay)
    }

    /// Check if the output indicates a retryable failure
    fn is_retryable(&self, output: &ComponentOutput) -> bool {
        matches!(output.status, ComponentStatus::Failure(_))
    }
}

#[async_trait]
impl Executor for RetryExecutor {
    async fn execute(
        &self,
        component: &dyn Component,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput> {
        let mut attempt = 0u32;

        loop {
            attempt += 1;

            tracing::debug!(
                component_id = component.id(),
                attempt = attempt,
                max_retries = self.max_retries,
                "Executing component (attempt {}/{})",
                attempt,
                self.max_retries + 1
            );

            // Execute through the inner executor
            let result = self.inner.execute(component, context, execution_ctx).await;

            match result {
                Ok(output) => {
                    if output.status.is_success() || output.status.is_skip() {
                        return Ok(output);
                    }

                    // Check if we should retry
                    if attempt <= self.max_retries && self.is_retryable(&output) {
                        let delay = self.calculate_delay(attempt);
                        tracing::warn!(
                            component_id = component.id(),
                            attempt = attempt,
                            delay_ms = delay.as_millis() as u64,
                            "Component failed, retrying after {:?}",
                            delay
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return Ok(output);
                }
                Err(e) => {
                    // Check if we should retry on error
                    if attempt <= self.max_retries {
                        let delay = self.calculate_delay(attempt);
                        tracing::warn!(
                            component_id = component.id(),
                            attempt = attempt,
                            error = %e,
                            delay_ms = delay.as_millis() as u64,
                            "Component execution error, retrying after {:?}",
                            delay
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return Err(e);
                }
            }
        }
    }

    fn name(&self) -> &str {
        "RetryExecutor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::component::ComponentType;
    use crate::workflow::executor::BasicExecutor;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    struct FailNTimesComponent {
        id: String,
        fail_count: Arc<AtomicU32>,
        max_failures: u32,
    }

    #[async_trait]
    impl Component for FailNTimesComponent {
        fn id(&self) -> &str {
            &self.id
        }

        fn component_type(&self) -> ComponentType {
            ComponentType::Tool
        }

        async fn execute(
            &self,
            _context: &mut DataContext,
            _execution_ctx: &ExecutionContext,
        ) -> Result<ComponentOutput> {
            let current = self.fail_count.fetch_add(1, Ordering::SeqCst);
            if current < self.max_failures {
                Ok(ComponentOutput::failure(format!("Failure {}", current + 1)))
            } else {
                Ok(ComponentOutput::success())
            }
        }
    }

    #[tokio::test]
    async fn test_retry_executor_success_after_failures() {
        let basic = Arc::new(BasicExecutor) as BoxedExecutor;
        let retry = RetryExecutor::new(basic, 3, Duration::from_millis(10));

        let fail_count = Arc::new(AtomicU32::new(0));
        let component = FailNTimesComponent {
            id: "test".to_string(),
            fail_count: fail_count.clone(),
            max_failures: 2, // Fail twice, then succeed
        };

        let mut context = DataContext::new();
        let exec_ctx = ExecutionContext::new().with_workflow_id(uuid::Uuid::new_v4());

        let result = retry.execute(&component, &mut context, &exec_ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap().status.is_success());
        assert_eq!(fail_count.load(Ordering::SeqCst), 3); // 2 failures + 1 success
    }

    #[tokio::test]
    async fn test_retry_executor_max_retries_exceeded() {
        let basic = Arc::new(BasicExecutor) as BoxedExecutor;
        let retry = RetryExecutor::new(basic, 2, Duration::from_millis(10));

        let fail_count = Arc::new(AtomicU32::new(0));
        let component = FailNTimesComponent {
            id: "test".to_string(),
            fail_count: fail_count.clone(),
            max_failures: 10, // Always fail
        };

        let mut context = DataContext::new();
        let exec_ctx = ExecutionContext::new().with_workflow_id(uuid::Uuid::new_v4());

        let result = retry.execute(&component, &mut context, &exec_ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap().status.is_failure());
        assert_eq!(fail_count.load(Ordering::SeqCst), 3); // 1 initial + 2 retries
    }
}
