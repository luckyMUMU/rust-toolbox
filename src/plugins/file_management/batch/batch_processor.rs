//! Batch processing framework for file management tools
//!
//! This module provides a generic batch processing framework that can execute
//! any tool in parallel across multiple inputs, with progress tracking and
//! error aggregation.

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::performance::concurrency::ConcurrencyManager;
use crate::plugins::file_management::core::error::{FileManagementError, FileManagementResult};
use crate::tools::{ToolNode, ToolRegistry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessorConfig {
    /// Maximum number of concurrent operations
    pub max_concurrency: usize,

    /// Whether to continue processing if individual items fail
    pub continue_on_error: bool,

    /// Timeout for individual batch items
    pub item_timeout: Option<Duration>,

    /// Progress reporting interval
    pub progress_interval: Duration,

    /// Maximum batch size
    pub max_batch_size: usize,

    /// Enable detailed progress tracking
    pub enable_progress_tracking: bool,

    /// Retry policy for failed items
    pub retry_failed_items: bool,

    /// Maximum retries per item
    pub max_retries: usize,
}

impl Default for BatchProcessorConfig {
    fn default() -> Self {
        Self {
            max_concurrency: num_cpus::get().max(4),
            continue_on_error: true,
            item_timeout: Some(Duration::from_secs(300)), // 5 minutes per item
            progress_interval: Duration::from_secs(5),
            max_batch_size: 1000,
            enable_progress_tracking: true,
            retry_failed_items: false,
            max_retries: 2,
        }
    }
}

/// Parameters for a single batch item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    /// Unique identifier for this batch item
    pub id: String,

    /// Parameters to pass to the tool
    pub parameters: Value,

    /// Optional metadata for this item
    pub metadata: HashMap<String, Value>,

    /// Priority for this item (higher numbers = higher priority)
    pub priority: i32,
}

impl BatchItem {
    pub fn new<S: Into<String>>(id: S, parameters: Value) -> Self {
        Self {
            id: id.into(),
            parameters,
            metadata: HashMap::new(),
            priority: 0,
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Result of processing a single batch item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResult {
    /// Item ID
    pub id: String,

    /// Processing status
    pub status: BatchItemStatus,

    /// Result value if successful
    pub result: Option<Value>,

    /// Error message if failed
    pub error: Option<String>,

    /// Processing duration
    pub duration: Duration,

    /// Number of retry attempts
    pub retry_attempts: usize,

    /// Item metadata
    pub metadata: HashMap<String, Value>,
}

/// Status of a batch item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchItemStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Skipped,
    Retrying,
}

/// Progress information for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    /// Total number of items
    pub total_items: usize,

    /// Number of completed items
    pub completed_items: usize,

    /// Number of failed items
    pub failed_items: usize,

    /// Number of skipped items
    pub skipped_items: usize,

    /// Number of items currently being processed
    pub processing_items: usize,

    /// Number of items pending processing
    pub pending_items: usize,

    /// Overall progress percentage (0.0 to 1.0)
    pub progress_percentage: f64,

    /// Estimated time remaining
    pub estimated_time_remaining: Option<Duration>,

    /// Processing rate (items per second)
    pub processing_rate: f64,

    /// Elapsed time since batch started
    pub elapsed_time: Duration,
}

impl BatchProgress {
    pub fn new(total_items: usize) -> Self {
        Self {
            total_items,
            completed_items: 0,
            failed_items: 0,
            skipped_items: 0,
            processing_items: 0,
            pending_items: total_items,
            progress_percentage: 0.0,
            estimated_time_remaining: None,
            processing_rate: 0.0,
            elapsed_time: Duration::from_secs(0),
        }
    }

    pub fn update(&mut self, elapsed_time: Duration) {
        self.elapsed_time = elapsed_time;

        let processed_items = self.completed_items + self.failed_items + self.skipped_items;
        self.progress_percentage = if self.total_items > 0 {
            processed_items as f64 / self.total_items as f64
        } else {
            1.0
        };

        // Calculate processing rate
        if elapsed_time.as_secs() > 0 {
            self.processing_rate = processed_items as f64 / elapsed_time.as_secs_f64();
        }

        // Estimate time remaining
        if self.processing_rate > 0.0 && self.pending_items > 0 {
            let remaining_seconds = self.pending_items as f64 / self.processing_rate;
            self.estimated_time_remaining = Some(Duration::from_secs_f64(remaining_seconds));
        }
    }

    pub fn is_complete(&self) -> bool {
        self.pending_items == 0 && self.processing_items == 0
    }
}

/// Complete result of batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Batch execution ID
    pub batch_id: String,

    /// Tool name that was executed
    pub tool_name: String,

    /// Final progress information
    pub progress: BatchProgress,

    /// Results for all items
    pub item_results: Vec<BatchItemResult>,

    /// Overall batch status
    pub status: BatchStatus,

    /// Total processing time
    pub total_duration: Duration,

    /// Error aggregation summary
    pub error_summary: ErrorSummary,

    /// Performance metrics
    pub performance_metrics: BatchPerformanceMetrics,
}

/// Overall batch processing status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
}

/// Summary of errors encountered during batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    /// Total number of errors
    pub total_errors: usize,

    /// Errors grouped by type
    pub error_types: HashMap<String, usize>,

    /// Most common error messages
    pub common_errors: Vec<(String, usize)>,

    /// Items that failed after all retries
    pub permanently_failed_items: Vec<String>,
}

/// Performance metrics for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPerformanceMetrics {
    /// Average processing time per item
    pub average_item_duration: Duration,

    /// Minimum processing time
    pub min_item_duration: Duration,

    /// Maximum processing time
    pub max_item_duration: Duration,

    /// Throughput (items per second)
    pub throughput: f64,

    /// Concurrency utilization (0.0 to 1.0)
    pub concurrency_utilization: f64,

    /// Memory usage statistics
    pub memory_stats: Option<MemoryStats>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub peak_memory_usage: u64,
    pub average_memory_usage: u64,
    pub memory_efficiency: f64,
}

/// Batch processing framework
pub struct BatchProcessor {
    config: BatchProcessorConfig,
    concurrency_manager: Option<Arc<ConcurrencyManager>>,
    tool_registry: Option<Arc<dyn ToolRegistry>>,
}

impl BatchProcessor {
    /// Create a new batch processor with default configuration
    pub fn new() -> Self {
        Self {
            config: BatchProcessorConfig::default(),
            concurrency_manager: None,
            tool_registry: None,
        }
    }

    /// Create a new batch processor with custom configuration
    pub fn with_config(config: BatchProcessorConfig) -> Self {
        Self {
            config,
            concurrency_manager: None,
            tool_registry: None,
        }
    }

    /// Set the concurrency manager for thread pool integration
    pub fn with_concurrency_manager(mut self, manager: Arc<ConcurrencyManager>) -> Self {
        self.concurrency_manager = Some(manager);
        self
    }

    /// Set the tool registry for tool resolution
    pub fn with_tool_registry(mut self, registry: Arc<dyn ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    /// Process a batch of items using the specified tool
    pub async fn process_batch(
        &self,
        tool_name: &str,
        items: Vec<BatchItem>,
        context: ExecutionContext,
    ) -> FileManagementResult<BatchResult> {
        let batch_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();

        info!(
            "Starting batch processing: {} items with tool '{}'",
            items.len(),
            tool_name
        );

        // Validate batch size
        if items.len() > self.config.max_batch_size {
            return Err(FileManagementError::validation(format!(
                "Batch size {} exceeds maximum allowed size {}",
                items.len(),
                self.config.max_batch_size
            )));
        }

        // Get the tool from registry
        let tool = self.get_tool(tool_name).await?;

        // Initialize progress tracking
        let progress = Arc::new(RwLock::new(BatchProgress::new(items.len())));
        let item_results = Arc::new(RwLock::new(Vec::<BatchItemResult>::new()));

        // Create semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));

        // Sort items by priority (higher priority first)
        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Create progress reporting task if enabled
        let progress_task = if self.config.enable_progress_tracking {
            Some(Self::spawn_progress_reporter(progress.clone(), start_time))
        } else {
            None
        };

        // Process items concurrently
        let mut handles = Vec::new();

        for item in sorted_items {
            let tool_clone = tool.clone();
            let context_clone = context.clone();
            let semaphore_clone = semaphore.clone();
            let progress_clone = progress.clone();
            let item_results_clone = item_results.clone();
            let config = self.config.clone();
            let concurrency_manager = self.concurrency_manager.clone();

            let handle = tokio::spawn(async move {
                Self::process_single_item(
                    tool_clone,
                    item,
                    context_clone,
                    semaphore_clone,
                    progress_clone,
                    item_results_clone,
                    config,
                    concurrency_manager,
                )
                .await
            });

            handles.push(handle);
        }

        // Wait for all items to complete
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Batch processing task failed: {}", e);
            }
        }

        // Stop progress reporting
        if let Some(task) = progress_task {
            task.abort();
        }

        let total_duration = start_time.elapsed();
        let final_progress = progress.read().await.clone();
        let all_results = item_results.read().await.clone();

        // Generate final result
        let batch_result = self
            .create_batch_result(
                batch_id,
                tool_name.to_string(),
                final_progress,
                all_results,
                total_duration,
            )
            .await;

        info!(
            "Batch processing completed: {} total, {} completed, {} failed in {:?}",
            batch_result.progress.total_items,
            batch_result.progress.completed_items,
            batch_result.progress.failed_items,
            total_duration
        );

        Ok(batch_result)
    }

    /// Get a tool from the registry
    async fn get_tool(&self, tool_name: &str) -> FileManagementResult<Arc<dyn ToolNode>> {
        let registry = self
            .tool_registry
            .as_ref()
            .ok_or_else(|| FileManagementError::validation("Tool registry not configured"))?;

        registry.get_tool(tool_name).ok_or_else(|| {
            FileManagementError::validation(format!("Tool '{}' not found", tool_name))
        })
    }

    /// Process a single batch item
    async fn process_single_item(
        tool: Arc<dyn ToolNode>,
        item: BatchItem,
        context: ExecutionContext,
        semaphore: Arc<Semaphore>,
        progress: Arc<RwLock<BatchProgress>>,
        results: Arc<RwLock<Vec<BatchItemResult>>>,
        config: BatchProcessorConfig,
        concurrency_manager: Option<Arc<ConcurrencyManager>>,
    ) {
        // Acquire semaphore permit
        let _permit = semaphore.acquire().await.unwrap();

        // Acquire tool permit from concurrency manager if available
        let _tool_permit = if let Some(ref manager) = concurrency_manager {
            manager.acquire_tool_permit().await.ok()
        } else {
            None
        };

        // Update progress - item is now processing
        {
            let mut progress_guard = progress.write().await;
            progress_guard.processing_items += 1;
            progress_guard.pending_items = progress_guard.pending_items.saturating_sub(1);
        }

        let start_time = Instant::now();
        let mut retry_attempts = 0;
        let mut _last_error = None;

        // Retry loop
        loop {
            let result =
                Self::execute_item_with_timeout(&tool, &item, &context, config.item_timeout).await;

            match result {
                Ok(value) => {
                    // Success - create result and break
                    let item_result = BatchItemResult {
                        id: item.id.clone(),
                        status: BatchItemStatus::Completed,
                        result: Some(value),
                        error: None,
                        duration: start_time.elapsed(),
                        retry_attempts,
                        metadata: item.metadata.clone(),
                    };

                    // Update progress and results
                    {
                        let mut progress_guard = progress.write().await;
                        progress_guard.processing_items =
                            progress_guard.processing_items.saturating_sub(1);
                        progress_guard.completed_items += 1;
                    }

                    results.write().await.push(item_result);
                    break;
                }
                Err(e) => {
                    _last_error = Some(e);
                    retry_attempts += 1;

                    // Check if we should retry
                    if config.retry_failed_items && retry_attempts <= config.max_retries {
                        debug!("Retrying item '{}' (attempt {})", item.id, retry_attempts);
                        continue;
                    } else {
                        // Failed permanently
                        let item_result = BatchItemResult {
                            id: item.id.clone(),
                            status: BatchItemStatus::Failed,
                            result: None,
                            error: Some(_last_error.as_ref().unwrap().to_string()),
                            duration: start_time.elapsed(),
                            retry_attempts,
                            metadata: item.metadata.clone(),
                        };

                        // Update progress and results
                        {
                            let mut progress_guard = progress.write().await;
                            progress_guard.processing_items =
                                progress_guard.processing_items.saturating_sub(1);
                            progress_guard.failed_items += 1;
                        }

                        results.write().await.push(item_result);

                        if !config.continue_on_error {
                            warn!("Item '{}' failed and continue_on_error is false", item.id);
                        }

                        break;
                    }
                }
            }
        }
    }

    /// Execute a single item with timeout
    async fn execute_item_with_timeout(
        tool: &Arc<dyn ToolNode>,
        item: &BatchItem,
        context: &ExecutionContext,
        timeout: Option<Duration>,
    ) -> Result<Value> {
        if let Some(timeout_duration) = timeout {
            tokio::time::timeout(
                timeout_duration,
                tool.execute(item.parameters.clone(), context.clone()),
            )
            .await
            .map_err(|_| WorkflowError::tool("Item execution timed out"))?
        } else {
            tool.execute(item.parameters.clone(), context.clone()).await
        }
    }

    /// Spawn a background task for progress reporting
    fn spawn_progress_reporter(
        progress: Arc<RwLock<BatchProgress>>,
        start_time: Instant,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                let elapsed = start_time.elapsed();
                let mut progress_guard = progress.write().await;
                progress_guard.update(elapsed);

                debug!(
                    "Batch progress: {:.1}% ({}/{}) - Rate: {:.2} items/sec - ETA: {:?}",
                    progress_guard.progress_percentage * 100.0,
                    progress_guard.completed_items
                        + progress_guard.failed_items
                        + progress_guard.skipped_items,
                    progress_guard.total_items,
                    progress_guard.processing_rate,
                    progress_guard.estimated_time_remaining
                );

                if progress_guard.is_complete() {
                    break;
                }
            }
        })
    }

    /// Create the final batch result
    async fn create_batch_result(
        &self,
        batch_id: String,
        tool_name: String,
        progress: BatchProgress,
        item_results: Vec<BatchItemResult>,
        total_duration: Duration,
    ) -> BatchResult {
        // Determine overall status
        let status = if progress.failed_items == 0 {
            BatchStatus::Completed
        } else if progress.completed_items > 0 {
            BatchStatus::PartiallyCompleted
        } else {
            BatchStatus::Failed
        };

        // Create error summary
        let error_summary = self.create_error_summary(&item_results);

        // Create performance metrics
        let performance_metrics = self.create_performance_metrics(&item_results, total_duration);

        BatchResult {
            batch_id,
            tool_name,
            progress,
            item_results,
            status,
            total_duration,
            error_summary,
            performance_metrics,
        }
    }

    /// Create error summary from item results
    fn create_error_summary(&self, item_results: &[BatchItemResult]) -> ErrorSummary {
        let mut error_types = HashMap::new();
        let mut error_messages = HashMap::new();
        let mut permanently_failed_items = Vec::new();

        for result in item_results {
            if result.status == BatchItemStatus::Failed {
                permanently_failed_items.push(result.id.clone());

                if let Some(ref error) = result.error {
                    // Categorize error type (simplified)
                    let error_type = if error.contains("timeout") {
                        "timeout"
                    } else if error.contains("validation") {
                        "validation"
                    } else if error.contains("permission") {
                        "permission"
                    } else if error.contains("not found") {
                        "not_found"
                    } else {
                        "other"
                    };

                    *error_types.entry(error_type.to_string()).or_insert(0) += 1;
                    *error_messages.entry(error.clone()).or_insert(0) += 1;
                }
            }
        }

        // Get most common errors
        let mut common_errors: Vec<(String, usize)> = error_messages.into_iter().collect();
        common_errors.sort_by(|a, b| b.1.cmp(&a.1));
        common_errors.truncate(10); // Keep top 10

        ErrorSummary {
            total_errors: permanently_failed_items.len(),
            error_types,
            common_errors,
            permanently_failed_items,
        }
    }

    /// Create performance metrics from item results
    fn create_performance_metrics(
        &self,
        item_results: &[BatchItemResult],
        total_duration: Duration,
    ) -> BatchPerformanceMetrics {
        if item_results.is_empty() {
            return BatchPerformanceMetrics {
                average_item_duration: Duration::from_secs(0),
                min_item_duration: Duration::from_secs(0),
                max_item_duration: Duration::from_secs(0),
                throughput: 0.0,
                concurrency_utilization: 0.0,
                memory_stats: None,
            };
        }

        let durations: Vec<Duration> = item_results.iter().map(|r| r.duration).collect();
        let total_item_duration: Duration = durations.iter().sum();

        let average_item_duration = total_item_duration / durations.len() as u32;
        let min_item_duration = durations.iter().min().copied().unwrap_or_default();
        let max_item_duration = durations.iter().max().copied().unwrap_or_default();

        let throughput = if total_duration.as_secs() > 0 {
            item_results.len() as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        // Calculate concurrency utilization
        let theoretical_sequential_time = total_item_duration;
        let actual_parallel_time = total_duration;
        let concurrency_utilization = if actual_parallel_time.as_secs() > 0 {
            (theoretical_sequential_time.as_secs_f64() / actual_parallel_time.as_secs_f64())
                / self.config.max_concurrency as f64
        } else {
            0.0
        }
        .min(1.0);

        BatchPerformanceMetrics {
            average_item_duration,
            min_item_duration,
            max_item_duration,
            throughput,
            concurrency_utilization,
            memory_stats: None, // TODO: Implement memory tracking
        }
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{PluginInfo, ToolInfo};
    use crate::tools::BasicTool;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock tool for testing
    struct MockTool {
        name: String,
        call_count: Arc<AtomicUsize>,
        should_fail: bool,
        delay: Duration,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                call_count: Arc::new(AtomicUsize::new(0)),
                should_fail: false,
                delay: Duration::from_millis(10),
            }
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        fn with_delay(mut self, delay: Duration) -> Self {
            self.delay = delay;
            self
        }
    }

    #[async_trait::async_trait]
    impl ToolNode for MockTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn get_info(&self) -> ToolInfo {
            ToolInfo {
                name: self.name.clone(),
                version: "1.0.0".to_string(),
                description: "Mock tool for testing".to_string(),
                category: Some("test".to_string()),
                tags: vec!["mock".to_string(), "test".to_string()],
                parameters_schema: json!({}),
                return_schema: json!({}),
                plugin_name: None,
                dependencies: Vec::new(),
                version_requirements: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        }

        fn get_plugin_info(&self) -> Option<&PluginInfo> {
            None
        }

        async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
            self.call_count.fetch_add(1, Ordering::SeqCst);

            if self.delay > Duration::from_millis(0) {
                tokio::time::sleep(self.delay).await;
            }

            if self.should_fail {
                return Err(WorkflowError::tool("Mock tool failure"));
            }

            Ok(json!({
                "input": params,
                "processed": true,
                "call_count": self.call_count.load(Ordering::SeqCst)
            }))
        }

        fn validate_parameters(&self, _params: &Value) -> Result<()> {
            Ok(())
        }
    }

    // Mock tool registry
    struct MockToolRegistry {
        tools: HashMap<String, Arc<dyn ToolNode>>,
    }

    impl MockToolRegistry {
        fn new() -> Self {
            Self {
                tools: HashMap::new(),
            }
        }

        fn add_tool(&mut self, tool: Arc<dyn ToolNode>) {
            self.tools.insert(tool.name().to_string(), tool);
        }
    }

    #[async_trait::async_trait]
    impl ToolRegistry for MockToolRegistry {
        fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolNode>> {
            self.tools.get(name).cloned()
        }

        fn list_tools(&self) -> Vec<ToolInfo> {
            self.tools.values().map(|tool| tool.get_info()).collect()
        }

        fn register_tool(&mut self, tool: Arc<dyn ToolNode>) -> Result<()> {
            self.tools.insert(tool.name().to_string(), tool);
            Ok(())
        }

        async fn execute_tool(
            &self,
            name: &str,
            params: Value,
            context: ExecutionContext,
        ) -> Result<Value> {
            let tool = self
                .get_tool(name)
                .ok_or_else(|| WorkflowError::tool(&format!("Tool '{}' not found", name)))?;
            tool.execute(params, context).await
        }

        fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()> {
            let tool = self
                .get_tool(name)
                .ok_or_else(|| WorkflowError::tool(&format!("Tool '{}' not found", name)))?;
            tool.validate_parameters(params)
        }

        fn has_tool(&self, name: &str) -> bool {
            self.tools.contains_key(name)
        }

        fn unregister_tool(&mut self, name: &str) -> Result<()> {
            self.tools.remove(name);
            Ok(())
        }

        fn tool_count(&self) -> usize {
            self.tools.len()
        }

        fn clear(&mut self) {
            self.tools.clear();
        }

        fn resolve_dependencies(
            &self,
            _tool_names: Vec<String>,
        ) -> Result<crate::tools::ResolutionResult> {
            Ok(crate::tools::ResolutionResult {
                resolved_versions: HashMap::new(),
                conflicts: Vec::new(),
                warnings: Vec::new(),
            })
        }

        fn check_version_conflicts(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        fn get_dependents(&self, _tool_name: &str) -> Vec<ToolInfo> {
            Vec::new()
        }

        async fn execute_tool_with_templates(
            &self,
            name: &str,
            params: Value,
            _template_context: &crate::tools::TemplateContext,
            execution_context: ExecutionContext,
        ) -> Result<Value> {
            self.execute_tool(name, params, execution_context).await
        }

        fn get_tool_templates(&self, _tool_name: &str) -> Vec<crate::tools::ParameterTemplate> {
            Vec::new()
        }
    }

    #[tokio::test]
    async fn test_batch_processor_creation() {
        let processor = BatchProcessor::new();
        assert_eq!(processor.config.max_concurrency, num_cpus::get().max(4));
        assert!(processor.config.continue_on_error);
    }

    #[tokio::test]
    async fn test_batch_item_creation() {
        let item = BatchItem::new("test-1", json!({"value": 42}))
            .with_priority(5)
            .with_metadata({
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), json!("test"));
                meta
            });

        assert_eq!(item.id, "test-1");
        assert_eq!(item.priority, 5);
        assert_eq!(item.metadata.get("category"), Some(&json!("test")));
    }

    #[tokio::test]
    async fn test_batch_progress_tracking() {
        let mut progress = BatchProgress::new(10);
        assert_eq!(progress.total_items, 10);
        assert_eq!(progress.pending_items, 10);
        assert_eq!(progress.progress_percentage, 0.0);

        progress.completed_items = 5;
        progress.pending_items = 5;
        progress.update(Duration::from_secs(10));

        assert_eq!(progress.progress_percentage, 0.5);
        assert!(progress.processing_rate > 0.0);
    }

    #[tokio::test]
    async fn test_simple_batch_processing() {
        // Create mock tool and registry
        let mock_tool = Arc::new(MockTool::new("test-tool"));
        let mut registry = MockToolRegistry::new();
        registry.add_tool(mock_tool.clone());

        // Create batch processor
        let processor = BatchProcessor::new().with_tool_registry(Arc::new(registry));

        // Create batch items
        let items = vec![
            BatchItem::new("item-1", json!({"value": 1})),
            BatchItem::new("item-2", json!({"value": 2})),
            BatchItem::new("item-3", json!({"value": 3})),
        ];

        // Process batch
        let context = ExecutionContext::new();
        let result = processor
            .process_batch("test-tool", items, context)
            .await
            .unwrap();

        // Verify results
        assert_eq!(result.progress.total_items, 3);
        assert_eq!(result.progress.completed_items, 3);
        assert_eq!(result.progress.failed_items, 0);
        assert_eq!(result.status, BatchStatus::Completed);
        assert_eq!(result.item_results.len(), 3);

        // Verify all items were processed
        for item_result in &result.item_results {
            assert_eq!(item_result.status, BatchItemStatus::Completed);
            assert!(item_result.result.is_some());
        }
    }

    #[tokio::test]
    async fn test_batch_processing_with_failures() {
        // Create mock tool that fails
        let mock_tool = Arc::new(MockTool::new("failing-tool").with_failure());
        let mut registry = MockToolRegistry::new();
        registry.add_tool(mock_tool);

        // Create batch processor with continue_on_error
        let config = BatchProcessorConfig {
            continue_on_error: true,
            ..Default::default()
        };
        let processor = BatchProcessor::with_config(config).with_tool_registry(Arc::new(registry));

        // Create batch items
        let items = vec![
            BatchItem::new("item-1", json!({"value": 1})),
            BatchItem::new("item-2", json!({"value": 2})),
        ];

        // Process batch
        let context = ExecutionContext::new();
        let result = processor
            .process_batch("failing-tool", items, context)
            .await
            .unwrap();

        // Verify results
        assert_eq!(result.progress.total_items, 2);
        assert_eq!(result.progress.completed_items, 0);
        assert_eq!(result.progress.failed_items, 2);
        assert_eq!(result.status, BatchStatus::Failed);

        // Verify error summary
        assert_eq!(result.error_summary.total_errors, 2);
        assert_eq!(result.error_summary.permanently_failed_items.len(), 2);
    }

    #[tokio::test]
    async fn test_batch_processing_concurrency() {
        // Create mock tool with delay
        let mock_tool = Arc::new(MockTool::new("slow-tool").with_delay(Duration::from_millis(100)));
        let mut registry = MockToolRegistry::new();
        registry.add_tool(mock_tool);

        // Create batch processor with limited concurrency
        let config = BatchProcessorConfig {
            max_concurrency: 2,
            ..Default::default()
        };
        let processor = BatchProcessor::with_config(config).with_tool_registry(Arc::new(registry));

        // Create many batch items
        let items: Vec<BatchItem> = (0..10)
            .map(|i| BatchItem::new(format!("item-{}", i), json!({"value": i})))
            .collect();

        let start_time = Instant::now();

        // Process batch
        let context = ExecutionContext::new();
        let result = processor
            .process_batch("slow-tool", items, context)
            .await
            .unwrap();

        let elapsed = start_time.elapsed();

        // Verify results
        assert_eq!(result.progress.completed_items, 10);
        assert_eq!(result.status, BatchStatus::Completed);

        // With concurrency of 2 and 100ms delay per item, 10 items should take roughly 500ms
        // (5 batches of 2 concurrent items each)
        assert!(elapsed >= Duration::from_millis(400)); // Allow some tolerance
        assert!(elapsed < Duration::from_millis(800)); // But not too much

        // Verify concurrency utilization
        assert!(result.performance_metrics.concurrency_utilization > 0.0);
    }

    #[tokio::test]
    async fn test_batch_item_priority_ordering() {
        // Create mock tool
        let mock_tool = Arc::new(MockTool::new("priority-tool"));
        let mut registry = MockToolRegistry::new();
        registry.add_tool(mock_tool);

        // Create batch processor with concurrency of 1 to ensure ordering
        let config = BatchProcessorConfig {
            max_concurrency: 1,
            ..Default::default()
        };
        let processor = BatchProcessor::with_config(config).with_tool_registry(Arc::new(registry));

        // Create batch items with different priorities
        let items = vec![
            BatchItem::new("low-priority", json!({"value": 1})).with_priority(1),
            BatchItem::new("high-priority", json!({"value": 2})).with_priority(10),
            BatchItem::new("medium-priority", json!({"value": 3})).with_priority(5),
        ];

        // Process batch
        let context = ExecutionContext::new();
        let result = processor
            .process_batch("priority-tool", items, context)
            .await
            .unwrap();

        // Verify all items completed
        assert_eq!(result.progress.completed_items, 3);
        assert_eq!(result.status, BatchStatus::Completed);

        // With concurrency of 1, items should be processed in priority order
        // (high -> medium -> low)
        let call_counts: Vec<i64> = result
            .item_results
            .iter()
            .map(|r| r.result.as_ref().unwrap()["call_count"].as_i64().unwrap())
            .collect();

        // Find the call count for each priority item
        let high_priority_call = result
            .item_results
            .iter()
            .find(|r| r.id == "high-priority")
            .unwrap()
            .result
            .as_ref()
            .unwrap()["call_count"]
            .as_i64()
            .unwrap();

        let medium_priority_call = result
            .item_results
            .iter()
            .find(|r| r.id == "medium-priority")
            .unwrap()
            .result
            .as_ref()
            .unwrap()["call_count"]
            .as_i64()
            .unwrap();

        let low_priority_call = result
            .item_results
            .iter()
            .find(|r| r.id == "low-priority")
            .unwrap()
            .result
            .as_ref()
            .unwrap()["call_count"]
            .as_i64()
            .unwrap();

        // High priority should be processed first (call_count = 1)
        // Medium priority should be processed second (call_count = 2)
        // Low priority should be processed last (call_count = 3)
        assert_eq!(high_priority_call, 1);
        assert_eq!(medium_priority_call, 2);
        assert_eq!(low_priority_call, 3);
    }
}
