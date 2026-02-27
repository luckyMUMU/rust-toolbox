//! Middleware system for tool execution.
//!
//! This module provides a middleware pattern for wrapping tool execution,
//! allowing for cross-cutting concerns like logging, caching, retries, etc.
//!
//! # Architecture
//!
//! The middleware system is inspired by tower-rs/tower's Service trait design,
//! adapted for our enum-based tool system. It uses a chain-of-responsibility
//! pattern where each middleware can decide to:
//! - Process the request and pass to next middleware
//! - Short-circuit and return early
//! - Modify the response on the way back
//!
//! # Example
//!
//! ```rust
//! use workflow_toolkit::tools::middleware::{MiddlewareStack, LoggingMiddleware, TimingMiddleware};
//!
//! let mut stack = MiddlewareStack::new();
//! stack.add(Arc::new(LoggingMiddleware::new()));
//! stack.add(Arc::new(TimingMiddleware::new()));
//!
//! // Execute tool through middleware chain
//! let output = stack.execute(input, &tool).await?;
//! ```

use crate::error::{Result, WorkflowError};
use crate::tools::types::{Tool, ToolInput, ToolOutput};
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, warn};

/// Execution metadata passed through middleware chain
#[derive(Clone, Debug, Default)]
pub struct ExecutionMetadata {
    /// Tool name being executed
    pub tool_name: String,
    /// Tool version
    pub tool_version: String,
    /// Start time of execution
    pub start_time: Option<Instant>,
    /// Additional custom metadata
    pub custom: HashMap<String, Value>,
}

impl ExecutionMetadata {
    /// Create new execution metadata
    pub fn new(tool_name: impl Into<String>, tool_version: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            tool_version: tool_version.into(),
            start_time: Some(Instant::now()),
            custom: HashMap::new(),
        }
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// Add custom metadata
    pub fn with_custom(mut self, key: impl Into<String>, value: Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }
}

/// Context passed through middleware chain
///
/// Contains the input data, execution metadata, and a thread-safe
/// storage for custom data that middleware can share.
pub struct MiddlewareContext {
    /// Input to the tool
    pub input: ToolInput,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
    /// Custom data storage for middleware communication
    custom_data: DashMap<String, Box<dyn Any + Send + Sync>>,
}

impl std::fmt::Debug for MiddlewareContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareContext")
            .field("input", &self.input)
            .field("metadata", &self.metadata)
            .field(
                "custom_data",
                &format!("DashMap with {} entries", self.custom_data.len()),
            )
            .finish()
    }
}

impl MiddlewareContext {
    /// Create a new middleware context
    pub fn new(input: ToolInput, metadata: ExecutionMetadata) -> Self {
        Self {
            input,
            metadata,
            custom_data: DashMap::new(),
        }
    }

    /// Insert custom data into the context
    pub fn insert<T: Any + Send + Sync>(&self, key: impl Into<String>, value: T) {
        self.custom_data.insert(key.into(), Box::new(value));
    }

    /// Get custom data from the context
    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        self.custom_data
            .get(key)
            .and_then(|v| v.value().downcast_ref::<T>().cloned())
    }

    /// Check if key exists in custom data
    pub fn contains_key(&self, key: &str) -> bool {
        self.custom_data.contains_key(key)
    }

    /// Remove custom data from the context
    pub fn remove(&self, key: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.custom_data.remove(key).map(|(_, v)| v)
    }
}

/// Represents the next middleware or the actual tool execution
///
/// Uses a lifetime parameter to avoid Box allocation while maintaining
/// the chain of middleware references.
pub struct Next<'a> {
    /// Remaining middleware stack
    stack: &'a [Arc<dyn Middleware>],
    /// The tool to execute at the end of the chain
    tool: &'a Tool,
}

impl<'a> Next<'a> {
    /// Create a new Next instance
    fn new(stack: &'a [Arc<dyn Middleware>], tool: &'a Tool) -> Self {
        Self { stack, tool }
    }

    /// Execute the next middleware or the tool
    ///
    /// If there are more middlewares, calls the next one.
    /// Otherwise, executes the actual tool.
    pub async fn run(&self, ctx: &mut MiddlewareContext) -> Result<ToolOutput> {
        if let Some((current, rest)) = self.stack.split_first() {
            // More middleware to process
            let next = Next {
                stack: rest,
                tool: self.tool,
            };
            current.process(ctx, next).await
        } else {
            // End of chain, execute the tool
            let input = ctx.input.clone();
            let tool = self.tool.clone();
            tool.execute(input, crate::core::ExecutionContext::new())
                .await
        }
    }
}

/// Middleware trait for wrapping tool execution
///
/// Implement this trait to create custom middleware that can intercept,
/// modify, or short-circuit tool execution.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process the request through this middleware
    ///
    /// # Arguments
    /// * `ctx` - The middleware context containing input and metadata
    /// * `next` - The next middleware or tool execution in the chain
    ///
    /// # Returns
    /// The tool output, potentially modified by this middleware
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput>;

    /// Get the name of this middleware for debugging/logging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or("UnknownMiddleware")
    }
}

/// Stack of middleware that processes tools in order
///
/// Middleware are executed in the order they are added (FIFO).
/// The first middleware added is the first to process the request.
#[derive(Clone, Default)]
pub struct MiddlewareStack {
    /// Ordered list of middleware
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareStack {
    /// Create an empty middleware stack
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add a middleware to the stack
    ///
    /// Middleware are executed in the order they are added.
    pub fn add(&mut self, middleware: Arc<dyn Middleware>) {
        self.middlewares.push(middleware);
    }

    /// Add multiple middleware at once
    pub fn add_many(&mut self, middlewares: Vec<Arc<dyn Middleware>>) {
        self.middlewares.extend(middlewares);
    }

    /// Get the number of middleware in the stack
    pub fn len(&self) -> usize {
        self.middlewares.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }

    /// Clear all middleware from the stack
    pub fn clear(&mut self) {
        self.middlewares.clear();
    }

    /// Execute a tool through the middleware chain
    ///
    /// This method creates a context and runs the tool through all
    /// registered middleware in order.
    #[instrument(skip(self, tool), fields(tool_name = %metadata.tool_name))]
    pub async fn execute(
        &self,
        input: ToolInput,
        metadata: ExecutionMetadata,
        tool: &Tool,
    ) -> Result<ToolOutput> {
        if self.middlewares.is_empty() {
            // No middleware, execute directly
            return tool
                .execute(input, crate::core::ExecutionContext::new())
                .await;
        }

        let mut ctx = MiddlewareContext::new(input, metadata);
        let next = Next::new(&self.middlewares, tool);

        debug!(
            "Executing tool {} through {} middleware",
            ctx.metadata.tool_name,
            self.middlewares.len()
        );

        next.run(&mut ctx).await
    }

    /// Execute with simplified API (creates default metadata)
    pub async fn execute_simple(&self, input: ToolInput, tool: &Tool) -> Result<ToolOutput> {
        let metadata = ExecutionMetadata::new("unknown", "0.0.0");
        self.execute(input, metadata, tool).await
    }
}

/// Builder for constructing middleware stacks
pub struct MiddlewareStackBuilder {
    stack: MiddlewareStack,
}

impl MiddlewareStackBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            stack: MiddlewareStack::new(),
        }
    }

    /// Add a middleware
    pub fn with(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.stack.add(middleware);
        self
    }

    /// Add a middleware (boxed)
    pub fn with_boxed<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.stack.add(Arc::new(middleware));
        self
    }

    /// Build the final stack
    pub fn build(self) -> MiddlewareStack {
        self.stack
    }
}

impl Default for MiddlewareStackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Logging middleware that records tool execution
#[derive(Debug, Default)]
pub struct LoggingMiddleware {
    /// Log level for execution start
    log_start: bool,
    /// Log level for execution completion
    log_completion: bool,
}

impl LoggingMiddleware {
    /// Create a new logging middleware
    pub fn new() -> Self {
        Self {
            log_start: true,
            log_completion: true,
        }
    }

    /// Only log completion (not start)
    pub fn log_completion_only(mut self) -> Self {
        self.log_start = false;
        self.log_completion = true;
        self
    }

    /// Disable all logging
    pub fn silent(mut self) -> Self {
        self.log_start = false;
        self.log_completion = false;
        self
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    #[instrument(skip(self, next), fields(middleware = "LoggingMiddleware"))]
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        let tool_name = ctx.metadata.tool_name.clone();

        if self.log_start {
            info!("Starting execution of tool: {}", tool_name);
        }

        let start = Instant::now();
        let result = next.run(ctx).await;
        let elapsed = start.elapsed();

        match &result {
            Ok(output) => {
                if self.log_completion {
                    info!("Tool {} completed successfully in {:?}", tool_name, elapsed);
                    debug!("Output: {:?}", output);
                }
            }
            Err(e) => {
                warn!("Tool {} failed after {:?}: {}", tool_name, elapsed, e);
            }
        }

        result
    }

    fn name(&self) -> &str {
        "LoggingMiddleware"
    }
}

/// Timing middleware that measures execution duration
#[derive(Debug, Default)]
pub struct TimingMiddleware {
    /// Store timing in context
    store_in_context: bool,
}

impl TimingMiddleware {
    /// Create a new timing middleware
    pub fn new() -> Self {
        Self {
            store_in_context: true,
        }
    }

    /// Don't store timing in context (just log)
    pub fn without_context_storage(mut self) -> Self {
        self.store_in_context = false;
        self
    }
}

#[async_trait]
impl Middleware for TimingMiddleware {
    #[instrument(skip(self, next), fields(middleware = "TimingMiddleware"))]
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        let start = Instant::now();
        let tool_name = ctx.metadata.tool_name.clone();

        let result = next.run(ctx).await;

        let elapsed = start.elapsed();
        let millis = elapsed.as_millis() as u64;

        if self.store_in_context {
            ctx.insert(format!("{}_duration_ms", tool_name), millis);
            ctx.insert(format!("{}_duration", tool_name), elapsed);
        }

        debug!("Tool {} executed in {}ms", tool_name, millis);

        result
    }

    fn name(&self) -> &str {
        "TimingMiddleware"
    }
}

/// Retry middleware that automatically retries failed executions
#[derive(Debug)]
pub struct RetryMiddleware {
    /// Maximum number of retry attempts
    max_retries: u32,
    /// Delay between retries
    retry_delay: Duration,
    /// Only retry on specific error types
    retryable_errors: Vec<String>,
}

impl RetryMiddleware {
    /// Create a new retry middleware with default settings
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            retry_delay: Duration::from_millis(100),
            retryable_errors: Vec::new(), // Empty means retry all errors
        }
    }

    /// Set the delay between retries
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Only retry specific error messages
    pub fn retryable_on(mut self, error_patterns: Vec<String>) -> Self {
        self.retryable_errors = error_patterns;
        self
    }

    /// Check if an error is retryable
    fn is_retryable(&self, error: &WorkflowError) -> bool {
        if self.retryable_errors.is_empty() {
            return true;
        }

        let error_str = error.to_string();
        self.retryable_errors
            .iter()
            .any(|pattern| error_str.contains(pattern))
    }
}

#[async_trait]
impl Middleware for RetryMiddleware {
    #[instrument(skip(self, next), fields(middleware = "RetryMiddleware"))]
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        let tool_name = ctx.metadata.tool_name.clone();
        let mut last_error_msg = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                debug!(
                    "Retrying tool {} (attempt {}/{})",
                    tool_name, attempt, self.max_retries
                );
                tokio::time::sleep(self.retry_delay).await;
            }

            match next.run(ctx).await {
                Ok(output) => {
                    if attempt > 0 {
                        info!("Tool {} succeeded after {} retries", tool_name, attempt);
                    }
                    return Ok(output);
                }
                Err(ref e) if attempt < self.max_retries && self.is_retryable(e) => {
                    warn!(
                        "Tool {} failed (attempt {}/{}): {}. Will retry...",
                        tool_name,
                        attempt + 1,
                        self.max_retries + 1,
                        e
                    );
                    last_error_msg = Some(e.to_string());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Err(WorkflowError::WorkflowExecution {
            message: last_error_msg.unwrap_or_else(|| {
                format!(
                    "Tool {} failed after {} retries",
                    tool_name, self.max_retries
                )
            }),
        })
    }

    fn name(&self) -> &str {
        "RetryMiddleware"
    }
}

/// Timeout middleware that enforces execution time limits
#[derive(Debug)]
pub struct TimeoutMiddleware {
    /// Maximum allowed execution time
    timeout: Duration,
}

impl TimeoutMiddleware {
    /// Create a new timeout middleware
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Create with seconds
    pub fn seconds(secs: u64) -> Self {
        Self::new(Duration::from_secs(secs))
    }

    /// Create with milliseconds
    pub fn millis(ms: u64) -> Self {
        Self::new(Duration::from_millis(ms))
    }
}

#[async_trait]
impl Middleware for TimeoutMiddleware {
    #[instrument(skip(self, next), fields(middleware = "TimeoutMiddleware"))]
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        let tool_name = ctx.metadata.tool_name.clone();

        match tokio::time::timeout(self.timeout, next.run(ctx)).await {
            Ok(result) => result,
            Err(_) => Err(WorkflowError::WorkflowExecution {
                message: format!("Tool {} timed out after {:?}", tool_name, self.timeout),
            }),
        }
    }

    fn name(&self) -> &str {
        "TimeoutMiddleware"
    }
}

/// Circuit breaker middleware that stops calling failing tools
#[derive(Debug)]
pub struct CircuitBreakerMiddleware {
    /// Number of failures before opening circuit
    failure_threshold: u32,
    /// Duration to wait before trying again
    reset_timeout: Duration,
    /// Current failure count (shared across clones)
    failure_count: Arc<std::sync::atomic::AtomicU32>,
    /// Last failure time
    last_failure: Arc<std::sync::Mutex<Option<Instant>>>,
}

impl CircuitBreakerMiddleware {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            reset_timeout,
            failure_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            last_failure: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Check if circuit is open
    fn is_open(&self) -> bool {
        let count = self.failure_count.load(std::sync::atomic::Ordering::SeqCst);
        if count < self.failure_threshold {
            return false;
        }

        // Check if enough time has passed to try again
        let last_failure = self.last_failure.lock().unwrap();
        if let Some(last) = *last_failure {
            last.elapsed() < self.reset_timeout
        } else {
            false
        }
    }

    /// Record a success
    fn record_success(&self) {
        self.failure_count
            .store(0, std::sync::atomic::Ordering::SeqCst);
        *self.last_failure.lock().unwrap() = None;
    }

    /// Record a failure
    fn record_failure(&self) {
        self.failure_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        *self.last_failure.lock().unwrap() = Some(Instant::now());
    }
}

#[async_trait]
impl Middleware for CircuitBreakerMiddleware {
    #[instrument(skip(self, next), fields(middleware = "CircuitBreakerMiddleware"))]
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        let tool_name = ctx.metadata.tool_name.clone();

        if self.is_open() {
            return Err(WorkflowError::WorkflowExecution {
                message: format!(
                    "Circuit breaker is OPEN for tool {}. Too many failures.",
                    tool_name
                ),
            });
        }

        let result = next.run(ctx).await;

        match &result {
            Ok(_) => self.record_success(),
            Err(_) => self.record_failure(),
        }

        result
    }

    fn name(&self) -> &str {
        "CircuitBreakerMiddleware"
    }
}

/// Metrics middleware that collects execution statistics
#[derive(Debug, Default)]
pub struct MetricsMiddleware {
    /// Metric prefix
    prefix: String,
}

impl MetricsMiddleware {
    /// Create a new metrics middleware
    pub fn new() -> Self {
        Self {
            prefix: "tool".to_string(),
        }
    }

    /// Set custom prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    #[instrument(skip(self, next), fields(middleware = "MetricsMiddleware"))]
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        let tool_name = ctx.metadata.tool_name.clone();
        let start = Instant::now();

        let result = next.run(ctx).await;

        let elapsed = start.elapsed();
        let status = if result.is_ok() { "success" } else { "failure" };

        // Store metrics in context for later collection
        let metric_key = format!("{}_{}_{}", self.prefix, tool_name, status);
        ctx.insert(metric_key.clone(), elapsed.as_millis() as u64);

        debug!(
            "Metric recorded: {} = {}ms (status: {})",
            metric_key,
            elapsed.as_millis(),
            status
        );

        result
    }

    fn name(&self) -> &str {
        "MetricsMiddleware"
    }
}

/// Caching middleware that caches tool outputs
pub struct CacheMiddleware {
    /// Cache key generator
    key_generator: Arc<dyn Fn(&ToolInput) -> String + Send + Sync>,
    /// TTL for cached entries
    ttl: Duration,
}

impl std::fmt::Debug for CacheMiddleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheMiddleware")
            .field("ttl", &self.ttl)
            .field("key_generator", &"<function>")
            .finish()
    }
}

impl CacheMiddleware {
    /// Create a new cache middleware with default key generation
    pub fn new(ttl: Duration) -> Self {
        Self {
            key_generator: Arc::new(|input: &ToolInput| {
                // Simple hash-based key
                format!("{:?}", input.params)
            }),
            ttl,
        }
    }

    /// Set a custom key generator
    pub fn with_key_generator<F>(mut self, generator: F) -> Self
    where
        F: Fn(&ToolInput) -> String + Send + Sync + 'static,
    {
        self.key_generator = Arc::new(generator);
        self
    }
}

#[async_trait]
impl Middleware for CacheMiddleware {
    #[instrument(skip(self, next), fields(middleware = "CacheMiddleware"))]
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        // For now, just pass through - full cache implementation would
        // require integration with the storage/cache module
        // TODO: Integrate with moka cache from performance module

        debug!("Cache middleware (pass-through - full implementation pending)");
        next.run(ctx).await
    }

    fn name(&self) -> &str {
        "CacheMiddleware"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_tool() -> Tool {
        // Create a simple native tool for testing
        use crate::core::ExecutionContext;
        use crate::tools::types::NativeToolBuilder;
        use std::sync::Arc;

        NativeToolBuilder::new()
            .name("test_tool")
            .version("1.0.0")
            .executor(|input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(input.params))
            })
            .build()
            .map(|tool| Tool::Native(Arc::new(tool)))
            .unwrap()
    }

    #[tokio::test]
    async fn test_middleware_stack_empty() {
        let stack = MiddlewareStack::new();
        let tool = create_test_tool();
        let input = ToolInput::new(json!({"test": "value"}));

        let result = stack.execute_simple(input, &tool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logging_middleware() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(LoggingMiddleware::new()));

        let tool = create_test_tool();
        let input = ToolInput::new(json!({"test": "value"}));
        let metadata = ExecutionMetadata::new("test_tool", "1.0.0");

        let result = stack.execute(input, metadata, &tool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timing_middleware() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(TimingMiddleware::new()));

        let tool = create_test_tool();
        let input = ToolInput::new(json!({"test": "value"}));
        let metadata = ExecutionMetadata::new("test_tool", "1.0.0");

        let result = stack.execute(input.clone(), metadata.clone(), &tool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_data_storage() {
        let ctx = MiddlewareContext::new(
            ToolInput::new(json!({})),
            ExecutionMetadata::new("test", "1.0.0"),
        );

        ctx.insert("key1", 42u64);
        ctx.insert("key2", "hello".to_string());

        assert_eq!(ctx.get::<u64>("key1"), Some(42));
        assert_eq!(ctx.get::<String>("key2"), Some("hello".to_string()));
        assert!(ctx.get::<u64>("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_middleware_chain() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(LoggingMiddleware::new()));
        stack.add(Arc::new(TimingMiddleware::new()));
        stack.add(Arc::new(MetricsMiddleware::new()));

        let tool = create_test_tool();
        let input = ToolInput::new(json!({"test": "value"}));
        let metadata = ExecutionMetadata::new("test_tool", "1.0.0");

        let result = stack.execute(input, metadata, &tool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timeout_middleware_success() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(TimeoutMiddleware::seconds(5)));

        let tool = create_test_tool();
        let input = ToolInput::new(json!({"test": "value"}));
        let metadata = ExecutionMetadata::new("test_tool", "1.0.0");

        let result = stack.execute(input, metadata, &tool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retry_middleware_success() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(RetryMiddleware::new(3)));

        let tool = create_test_tool();
        let input = ToolInput::new(json!({"test": "value"}));
        let metadata = ExecutionMetadata::new("test_tool", "1.0.0");

        let result = stack.execute(input, metadata, &tool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(CircuitBreakerMiddleware::new(
            5,
            Duration::from_secs(60),
        )));

        let tool = create_test_tool();
        let input = ToolInput::new(json!({"test": "value"}));
        let metadata = ExecutionMetadata::new("test_tool", "1.0.0");

        let result = stack.execute(input, metadata, &tool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_middleware_builder() {
        let stack = MiddlewareStackBuilder::new()
            .with(Arc::new(LoggingMiddleware::new()))
            .with(Arc::new(TimingMiddleware::new()))
            .build();

        assert_eq!(stack.len(), 2);
    }
}
