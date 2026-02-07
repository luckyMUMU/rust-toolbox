//! 错误处理器实现
//!
//! 提供统一的错误处理和恢复机制

use std::future::Future;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// 错误分类
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorClassification {
    /// 业务错误（可预期，可处理）
    BusinessError {
        code: String,
        message: String,
        recoverable: bool,
    },
    /// 系统错误（需要重试或降级）
    SystemError {
        retryable: bool,
        severity: ErrorSeverity,
    },
    /// 网络错误（通常可重试）
    NetworkError {
        timeout: bool,
        retry_after: Option<Duration>,
    },
    /// 资源错误（资源不足）
    ResourceError {
        resource_type: String,
        current_usage: u64,
        limit: u64,
    },
}

/// 错误严重程度
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
    Fatal,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Warning => write!(f, "Warning"),
            ErrorSeverity::Error => write!(f, "Error"),
            ErrorSeverity::Critical => write!(f, "Critical"),
            ErrorSeverity::Fatal => write!(f, "Fatal"),
        }
    }
}

/// 错误处理策略
#[derive(Clone, Debug)]
pub enum ErrorHandlingStrategy {
    /// 立即重试
    ImmediateRetry {
        max_attempts: u32,
        delay: Duration,
    },
    /// 指数退避重试
    ExponentialBackoff {
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    },
    /// 熔断（暂停服务）
    CircuitBreaker {
        threshold: f64,
        recovery_time: Duration,
    },
    /// 降级（使用备用方案）
    Fallback {
        fallback_value: Option<String>,
    },
    /// 快速失败
    FailFast,
    /// 忽略错误继续
    Ignore,
}

impl Default for ErrorHandlingStrategy {
    fn default() -> Self {
        ErrorHandlingStrategy::ExponentialBackoff {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

/// 错误处理器
pub struct ErrorHandler {
    strategy: ErrorHandlingStrategy,
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new(strategy: ErrorHandlingStrategy) -> Self {
        Self { strategy }
    }

    /// 使用默认策略创建
    pub fn with_defaults() -> Self {
        Self::new(ErrorHandlingStrategy::default())
    }

    /// 执行带错误处理的操作
    pub async fn execute<F, Fut, T, E>(
        &self,
        operation: F,
        error_classifier: impl Fn(&E) -> ErrorClassification,
    ) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        match &self.strategy {
            ErrorHandlingStrategy::ImmediateRetry { max_attempts, delay } => {
                self.retry_immediate(operation, *max_attempts, *delay, error_classifier)
                    .await
            }
            ErrorHandlingStrategy::ExponentialBackoff {
                max_attempts,
                base_delay,
                max_delay,
                multiplier,
            } => {
                self.retry_exponential(
                    operation,
                    *max_attempts,
                    *base_delay,
                    *max_delay,
                    *multiplier,
                    error_classifier,
                )
                .await
            }
            ErrorHandlingStrategy::FailFast => operation().await,
            ErrorHandlingStrategy::Ignore => operation().await.or_else(|_| {
                // 返回默认值，这里简化处理
                panic!("Ignore strategy requires default value implementation")
            }),
            _ => operation().await,
        }
    }

    /// 立即重试
    async fn retry_immediate<F, Fut, T, E>(
        &self,
        operation: F,
        max_attempts: u32,
        delay: Duration,
        error_classifier: impl Fn(&E) -> ErrorClassification,
    ) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut last_error = None;

        for attempt in 1..=max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Operation succeeded after {} attempts", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    let classification = error_classifier(&error);
                    last_error = Some(error);

                    if attempt < max_attempts {
                        warn!(
                            "Operation failed (attempt {}/{}), retrying in {:?}: {:?}",
                            attempt, max_attempts, delay, classification
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        error!("Operation failed after {} attempts", max_attempts);
        Err(last_error.unwrap())
    }

    /// 指数退避重试
    async fn retry_exponential<F, Fut, T, E>(
        &self,
        operation: F,
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        error_classifier: impl Fn(&E) -> ErrorClassification,
    ) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut last_error = None;
        let mut current_delay = base_delay;

        for attempt in 1..=max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!(
                            "Operation succeeded after {} attempts with exponential backoff",
                            attempt
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    let classification = error_classifier(&error);
                    last_error = Some(error);

                    if attempt < max_attempts {
                        warn!(
                            "Operation failed (attempt {}/{}), retrying in {:?}: {:?}",
                            attempt, max_attempts, current_delay, classification
                        );
                        tokio::time::sleep(current_delay).await;

                        // 计算下一次延迟
                        let next_delay_secs =
                            current_delay.as_secs_f64() * multiplier;
                        current_delay = Duration::from_secs_f64(
                            next_delay_secs.min(max_delay.as_secs_f64()),
                        );
                    }
                }
            }
        }

        error!(
            "Operation failed after {} attempts with exponential backoff",
            max_attempts
        );
        Err(last_error.unwrap())
    }

    /// 计算退避延迟
    fn calculate_backoff_delay(
        &self,
        attempt: u32,
        base_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    ) -> Duration {
        let delay_secs = base_delay.as_secs_f64() * multiplier.powi(attempt as i32 - 1);
        Duration::from_secs_f64(delay_secs.min(max_delay.as_secs_f64()))
    }
}

/// 工作流错误分类器
pub struct WorkflowErrorClassifier;

impl WorkflowErrorClassifier {
    /// 分类错误
    pub fn classify<E: std::error::Error>(error: &E) -> ErrorClassification {
        let error_msg = error.to_string().to_lowercase();

        // 网络错误
        if error_msg.contains("timeout")
            || error_msg.contains("connection")
            || error_msg.contains("network")
        {
            return ErrorClassification::NetworkError {
                timeout: error_msg.contains("timeout"),
                retry_after: None,
            };
        }

        // 资源错误
        if error_msg.contains("memory")
            || error_msg.contains("disk")
            || error_msg.contains("quota")
            || error_msg.contains("limit")
        {
            return ErrorClassification::ResourceError {
                resource_type: "unknown".to_string(),
                current_usage: 0,
                limit: 0,
            };
        }

        // 业务错误
        if error_msg.contains("validation")
            || error_msg.contains("invalid")
            || error_msg.contains("not found")
        {
            return ErrorClassification::BusinessError {
                code: "BUSINESS_ERROR".to_string(),
                message: error_msg,
                recoverable: false,
            };
        }

        // 默认为系统错误
        ErrorClassification::SystemError {
            retryable: true,
            severity: ErrorSeverity::Error,
        }
    }
}

/// 可恢复操作包装器
pub struct RecoverableOperation<T, E> {
    operation: Box<dyn Fn() -> Result<T, E> + Send + Sync>,
    recovery_strategy: ErrorHandlingStrategy,
}

impl<T, E> RecoverableOperation<T, E> {
    /// 创建可恢复操作
    pub fn new<F>(operation: F, recovery_strategy: ErrorHandlingStrategy) -> Self
    where
        F: Fn() -> Result<T, E> + Send + Sync + 'static,
    {
        Self {
            operation: Box::new(operation),
            recovery_strategy,
        }
    }

    /// 执行操作
    pub fn execute(&self) -> Result<T, E> {
        (self.operation)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_immediate_retry_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let handler = ErrorHandler::new(ErrorHandlingStrategy::ImmediateRetry {
            max_attempts: 3,
            delay: Duration::from_millis(10),
        });

        let result = handler
            .execute(
                || async {
                    let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err::<i32, String>("temporary error".to_string())
                    } else {
                        Ok(42)
                    }
                },
                |e| WorkflowErrorClassifier::classify(&std::io::Error::new(std::io::ErrorKind::Other, e)),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_immediate_retry_exhausted() {
        let handler = ErrorHandler::new(ErrorHandlingStrategy::ImmediateRetry {
            max_attempts: 2,
            delay: Duration::from_millis(10),
        });

        let result = handler
            .execute(
                || async { Err::<i32, String>("persistent error".to_string()) },
                |e| WorkflowErrorClassifier::classify(&std::io::Error::new(std::io::ErrorKind::Other, e)),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let handler = ErrorHandler::new(ErrorHandlingStrategy::ExponentialBackoff {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            multiplier: 2.0,
        });

        let start = std::time::Instant::now();

        let result = handler
            .execute(
                || async {
                    let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err::<i32, String>("temporary error".to_string())
                    } else {
                        Ok(42)
                    }
                },
                |e| WorkflowErrorClassifier::classify(&std::io::Error::new(std::io::ErrorKind::Other, e)),
            )
            .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // 应该有退避延迟：10ms + 20ms = 30ms
        assert!(elapsed >= Duration::from_millis(25));
    }

    #[test]
    fn test_error_classifier() {
        let timeout_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let classification = WorkflowErrorClassifier::classify(&timeout_error);

        assert!(
            matches!(classification, ErrorClassification::NetworkError { timeout: true, .. }),
            "Expected NetworkError with timeout"
        );

        let validation_error = std::io::Error::new(std::io::ErrorKind::InvalidInput, "validation failed");
        let classification = WorkflowErrorClassifier::classify(&validation_error);

        assert!(
            matches!(classification, ErrorClassification::BusinessError { .. }),
            "Expected BusinessError"
        );
    }
}
