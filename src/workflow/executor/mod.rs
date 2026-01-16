//! Executor layer for component execution.
//!
//! This module provides the executor abstraction using the Chain of Responsibility pattern.
//! Executors wrap component execution with cross-cutting concerns like:
//!
//! - Retry logic
//! - Caching
//! - Audit logging
//! - Metrics collection
//!
//! # Architecture
//!
//! Executors form a chain where each executor can:
//! 1. Perform pre-execution logic
//! 2. Delegate to the next executor in the chain
//! 3. Perform post-execution logic
//!
//! ```text
//! AuditExecutor -> CacheExecutor -> RetryExecutor -> BasicExecutor -> Component
//! ```

pub mod audit;
pub mod basic;
pub mod cache;
pub mod retry;

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::workflow::component::{Component, ComponentOutput};
use crate::workflow::context::DataContext;
use async_trait::async_trait;
use std::sync::Arc;

/// Executor trait for component execution.
///
/// Executors wrap the execution of components, allowing cross-cutting concerns
/// to be applied in a composable manner.
///
/// # Example
///
/// ```ignore
/// // Create a basic executor
/// let basic = Arc::new(BasicExecutor);
///
/// // Wrap with retry logic
/// let retry = Arc::new(RetryExecutor::new(basic, 3, Duration::from_secs(1)));
///
/// // Wrap with caching
/// let cached = Arc::new(CacheExecutor::new(retry, cache));
///
/// // Execute a component
/// let output = cached.execute(component, &mut context, &exec_ctx).await?;
/// ```
#[async_trait]
pub trait Executor: Send + Sync {
    /// Execute a component through this executor.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to execute
    /// * `context` - Mutable data context for the component
    /// * `execution_ctx` - Execution context with workflow metadata
    ///
    /// # Returns
    ///
    /// The component's output wrapped in a Result
    async fn execute(
        &self,
        component: &dyn Component,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput>;

    /// Get a human-readable name for this executor (for logging/debugging)
    fn name(&self) -> &str {
        "Executor"
    }
}

/// A boxed executor for dynamic dispatch
pub type BoxedExecutor = Arc<dyn Executor>;

/// Builder for constructing executor chains.
///
/// # Example
///
/// ```ignore
/// let executor = ExecutorChainBuilder::new()
///     .with_audit(audit_logger)
///     .with_cache(cache)
///     .with_retry(3, Duration::from_secs(1))
///     .build();
/// ```
pub struct ExecutorChainBuilder {
    executor: BoxedExecutor,
}

impl ExecutorChainBuilder {
    /// Create a new builder with a basic executor at the base
    pub fn new() -> Self {
        Self {
            executor: Arc::new(basic::BasicExecutor),
        }
    }

    /// Add retry capability to the chain
    pub fn with_retry(self, max_retries: u32, base_delay: std::time::Duration) -> Self {
        Self {
            executor: Arc::new(retry::RetryExecutor::new(
                self.executor,
                max_retries,
                base_delay,
            )),
        }
    }

    /// Add caching capability to the chain
    pub fn with_cache(self, cache: Arc<moka::future::Cache<String, ComponentOutput>>) -> Self {
        Self {
            executor: Arc::new(cache::CacheExecutor::new(self.executor, cache)),
        }
    }

    /// Add audit logging capability to the chain
    pub fn with_audit(self, audit_logger: Arc<crate::workflow::AuditLogger>) -> Self {
        Self {
            executor: Arc::new(audit::AuditExecutor::new(self.executor, audit_logger)),
        }
    }

    /// Build the final executor chain
    pub fn build(self) -> BoxedExecutor {
        self.executor
    }
}

impl Default for ExecutorChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Re-exports
pub use audit::AuditExecutor;
pub use basic::BasicExecutor;
pub use cache::CacheExecutor;
pub use retry::RetryExecutor;
