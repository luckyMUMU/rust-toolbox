//! Batch processing framework for file management tools
//!
//! This module provides a generic batch processing framework that can execute
//! any tool in parallel across multiple inputs, with progress tracking and
//! error aggregation.

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::performance::concurrency::ConcurrencyManager;
use crate::plugins::file_management::core::error::{FileManagementError, FileManagementResult};
use crate::tools::registry::ToolRegistry;
use crate::tools::types::Tool;
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
            max_concurrency: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4)
                .max(4),
            continue_on_error: true,
            item_timeout: Some(Duration::from_secs(300)),
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
    tool_registry: Option<Arc<ToolRegistry>>,
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
    pub fn with_tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
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
    async fn get_tool(&self, tool_name: &str) -> FileManagementResult<Tool> {
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
        tool: Tool,
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
        tool: &Tool,
        item: &BatchItem,
        context: &ExecutionContext,
        timeout: Option<Duration>,
    ) -> Result<Value> {
        use crate::tools::types::ToolInput;

        let input = ToolInput::new(item.parameters.clone());
        if let Some(timeout_duration) = timeout {
            tokio::time::timeout(timeout_duration, tool.execute(input, context.clone()))
                .await
                .map_err(|_| WorkflowError::tool("Item execution timed out"))?
                .map(|output| output.result)
        } else {
            tool.execute(input, context.clone())
                .await
                .map(|output| output.result)
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
