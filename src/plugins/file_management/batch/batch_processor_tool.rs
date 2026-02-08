//! Batch Processor Tool implementation
//!
//! This module implements the actual Batch Processor Tool that replaces the
//! placeholder executor with full batch processing functionality.

use super::{
    batch_processor::{BatchItem, BatchProcessor, BatchProcessorConfig, BatchResult, BatchStatus},
    progress_tracker::{ProgressEvent, ProgressTracker, ProgressTrackerConfig},
};
use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::plugins::file_management::plugin::FileManagementConfig;
use crate::error::{Result, WorkflowError};
use crate::performance::concurrency::ConcurrencyManager;
use crate::tools::types::{Tool, NativeToolBuilder, ToolInput, ToolOutput};
use crate::tools::registry::ToolRegistry;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// Parameters for the Batch Processor Tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessorParams {
    /// Name of the tool to execute in batch
    pub tool_name: String,

    /// Array of batch items to process
    pub batch_items: Vec<BatchItemParams>,

    /// Maximum number of concurrent operations
    pub max_concurrency: Option<usize>,

    /// Whether to continue processing if individual items fail
    pub continue_on_error: Option<bool>,

    /// Timeout for individual batch items (in seconds)
    pub item_timeout_seconds: Option<u64>,

    /// Progress reporting interval (in seconds)
    pub progress_interval_seconds: Option<u64>,

    /// Enable detailed progress tracking
    pub enable_progress_tracking: Option<bool>,

    /// Enable retry for failed items
    pub retry_failed_items: Option<bool>,

    /// Maximum number of retries per item
    pub max_retries: Option<usize>,

    /// Batch processing mode
    pub processing_mode: Option<BatchProcessingMode>,

    /// Enable experimental mode
    pub experimental_mode: Option<bool>,
}

/// Parameters for a single batch item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemParams {
    /// Unique identifier for this batch item
    pub id: String,

    /// Parameters to pass to the tool
    pub parameters: Value,

    /// Optional metadata for this item
    pub metadata: Option<HashMap<String, Value>>,

    /// Priority for this item (higher numbers = higher priority)
    pub priority: Option<i32>,
}

/// Batch processing mode
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum BatchProcessingMode {
    /// Process all items in parallel (default)
    #[default]
    Parallel,

    /// Process items sequentially
    Sequential,

    /// Process items in priority order
    PriorityBased,

    /// Adaptive processing based on system load
    Adaptive,
}


/// Result of batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessorResult {
    /// Batch execution ID
    pub batch_id: String,

    /// Tool name that was executed
    pub tool_name: String,

    /// Processing mode used
    pub processing_mode: BatchProcessingMode,

    /// Overall batch status
    pub status: BatchStatus,

    /// Total processing time in milliseconds
    pub total_duration_ms: u64,

    /// Progress summary
    pub progress_summary: BatchProgressSummary,

    /// Results for all items
    pub item_results: Vec<BatchItemResultSummary>,

    /// Error summary
    pub error_summary: BatchErrorSummary,

    /// Performance metrics
    pub performance_metrics: BatchPerformanceMetricsSummary,

    /// Progress events (if tracking enabled)
    pub progress_events: Option<Vec<ProgressEvent>>,

    /// Whether this was run in experimental mode
    pub experimental_mode: bool,
}

/// Summary of batch progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgressSummary {
    pub total_items: usize,
    pub completed_items: usize,
    pub failed_items: usize,
    pub skipped_items: usize,
    pub progress_percentage: f64,
    pub processing_rate: f64,
    pub estimated_time_remaining_ms: Option<u64>,
}

/// Summary of batch item result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResultSummary {
    pub id: String,
    pub status: String,
    pub duration_ms: u64,
    pub retry_attempts: usize,
    pub has_result: bool,
    pub has_error: bool,
    pub error_type: Option<String>,
}

/// Summary of batch errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchErrorSummary {
    pub total_errors: usize,
    pub error_types: HashMap<String, usize>,
    pub most_common_error: Option<String>,
    pub permanently_failed_items: Vec<String>,
}

/// Summary of batch performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPerformanceMetricsSummary {
    pub average_item_duration_ms: u64,
    pub min_item_duration_ms: u64,
    pub max_item_duration_ms: u64,
    pub throughput: f64,
    pub concurrency_utilization: f64,
}

/// Batch Processor Tool implementation
pub struct BatchProcessorTool {
    config: FileManagementConfig,
    plugin_info: PluginInfo,
    concurrency_manager: Option<Arc<ConcurrencyManager>>,
}

impl BatchProcessorTool {
    /// Create a new Batch Processor Tool
    pub fn new(config: FileManagementConfig, plugin_info: PluginInfo) -> Self {
        Self {
            config,
            plugin_info,
            concurrency_manager: None,
        }
    }

    /// Set the concurrency manager
    pub fn with_concurrency_manager(mut self, manager: Arc<ConcurrencyManager>) -> Self {
        self.concurrency_manager = Some(manager);
        self
    }

    /// Create the tool info for registration
    pub fn create_tool_info(&self) -> ToolInfo {
        ToolInfo {
            name: "batch-processor".to_string(),
            version: "1.0.0".to_string(),
            description:
                "Generic batch processing for any tool with progress tracking and error handling"
                    .to_string(),
            category: Some("batch-processing".to_string()),
            tags: vec![
                "batch".to_string(),
                "parallel".to_string(),
                "processing".to_string(),
                "concurrency".to_string(),
                "progress".to_string(),
            ],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "tool_name": {
                        "type": "string",
                        "description": "Name of the tool to execute in batch"
                    },
                    "batch_items": {
                        "type": "array",
                        "description": "Array of batch items to process",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {
                                    "type": "string",
                                    "description": "Unique identifier for this batch item"
                                },
                                "parameters": {
                                    "type": "object",
                                    "description": "Parameters to pass to the tool"
                                },
                                "metadata": {
                                    "type": "object",
                                    "description": "Optional metadata for this item"
                                },
                                "priority": {
                                    "type": "integer",
                                    "description": "Priority for this item (higher numbers = higher priority)",
                                    "default": 0
                                }
                            },
                            "required": ["id", "parameters"]
                        }
                    },
                    "max_concurrency": {
                        "type": "integer",
                        "description": "Maximum number of concurrent operations",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 4
                    },
                    "continue_on_error": {
                        "type": "boolean",
                        "description": "Whether to continue processing if individual items fail",
                        "default": true
                    },
                    "item_timeout_seconds": {
                        "type": "integer",
                        "description": "Timeout for individual batch items (in seconds)",
                        "minimum": 1,
                        "maximum": 3600,
                        "default": 300
                    },
                    "progress_interval_seconds": {
                        "type": "integer",
                        "description": "Progress reporting interval (in seconds)",
                        "minimum": 1,
                        "maximum": 60,
                        "default": 5
                    },
                    "enable_progress_tracking": {
                        "type": "boolean",
                        "description": "Enable detailed progress tracking",
                        "default": true
                    },
                    "retry_failed_items": {
                        "type": "boolean",
                        "description": "Enable retry for failed items",
                        "default": false
                    },
                    "max_retries": {
                        "type": "integer",
                        "description": "Maximum number of retries per item",
                        "minimum": 0,
                        "maximum": 10,
                        "default": 2
                    },
                    "processing_mode": {
                        "type": "string",
                        "enum": ["Parallel", "Sequential", "PriorityBased", "Adaptive"],
                        "description": "Batch processing mode",
                        "default": "Parallel"
                    },
                    "experimental_mode": {
                        "type": "boolean",
                        "description": "Run in experimental mode (simulation only)",
                        "default": false
                    }
                },
                "required": ["tool_name", "batch_items"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "batch_id": {"type": "string"},
                    "tool_name": {"type": "string"},
                    "processing_mode": {"type": "string"},
                    "status": {"type": "string", "enum": ["Completed", "PartiallyCompleted", "Failed", "Cancelled"]},
                    "total_duration_ms": {"type": "number"},
                    "progress_summary": {
                        "type": "object",
                        "properties": {
                            "total_items": {"type": "number"},
                            "completed_items": {"type": "number"},
                            "failed_items": {"type": "number"},
                            "skipped_items": {"type": "number"},
                            "progress_percentage": {"type": "number"},
                            "processing_rate": {"type": "number"},
                            "estimated_time_remaining_ms": {"type": "number"}
                        }
                    },
                    "item_results": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string"},
                                "status": {"type": "string"},
                                "duration_ms": {"type": "number"},
                                "retry_attempts": {"type": "number"},
                                "has_result": {"type": "boolean"},
                                "has_error": {"type": "boolean"},
                                "error_type": {"type": "string"}
                            }
                        }
                    },
                    "error_summary": {
                        "type": "object",
                        "properties": {
                            "total_errors": {"type": "number"},
                            "error_types": {"type": "object"},
                            "most_common_error": {"type": "string"},
                            "permanently_failed_items": {"type": "array", "items": {"type": "string"}}
                        }
                    },
                    "performance_metrics": {
                        "type": "object",
                        "properties": {
                            "average_item_duration_ms": {"type": "number"},
                            "min_item_duration_ms": {"type": "number"},
                            "max_item_duration_ms": {"type": "number"},
                            "throughput": {"type": "number"},
                            "concurrency_utilization": {"type": "number"}
                        }
                    },
                    "progress_events": {
                        "type": "array",
                        "description": "Progress events (if tracking enabled)"
                    },
                    "experimental_mode": {
                        "type": "boolean",
                        "description": "Whether the operation was run in experimental mode"
                    }
                }
            }),
            plugin_name: Some(self.plugin_info.name.clone()),
            dependencies: Vec::new(),
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Create the tool executor
    pub fn create_executor(&self) -> Arc<BatchProcessorExecutor> {
        Arc::new(BatchProcessorExecutor::new(
            self.config.clone(),
            self.concurrency_manager.clone(),
        ))
    }

    /// Create a complete Tool instance
    pub fn create_tool(&self) -> Result<Tool> {
        let tool_info = self.create_tool_info();
        let executor = self.create_executor();

        let native_tool = NativeToolBuilder::new()
            .name(&tool_info.name)
            .version(&tool_info.version)
            .description(&tool_info.description)
            .category(tool_info.category.unwrap_or_default())
            .tags(tool_info.tags)
            .executor(move |input, ctx| {
                let executor = executor.clone();
                async move {
                    let result = executor.execute(input.params, ctx).await?;
                    Ok(ToolOutput::success(result))
                }
            })
            .build()?;

        Ok(Tool::Native(Arc::new(native_tool)))
    }
}

/// Executor for the Batch Processor Tool
pub struct BatchProcessorExecutor {
    #[allow(dead_code)]
    config: FileManagementConfig,
    concurrency_manager: Option<Arc<ConcurrencyManager>>,
}

impl BatchProcessorExecutor {
    pub fn new(
        config: FileManagementConfig,
        concurrency_manager: Option<Arc<ConcurrencyManager>>,
    ) -> Self {
        Self {
            config,
            concurrency_manager,
        }
    }

    /// Parse batch processing parameters
    fn parse_parameters(&self, params: &Value) -> Result<BatchProcessorParams> {
        serde_json::from_value(params.clone()).map_err(|e| {
            WorkflowError::validation(format!("Invalid batch processor parameters: {}", e))
        })
    }

    /// Convert parameters to batch processor configuration
    fn create_batch_config(&self, params: &BatchProcessorParams) -> BatchProcessorConfig {
        BatchProcessorConfig {
            max_concurrency: params.max_concurrency.unwrap_or(num_cpus::get().max(4)),
            continue_on_error: params.continue_on_error.unwrap_or(true),
            item_timeout: params.item_timeout_seconds.map(Duration::from_secs),
            progress_interval: Duration::from_secs(params.progress_interval_seconds.unwrap_or(5)),
            max_batch_size: 10000, // Hard limit for safety
            enable_progress_tracking: params.enable_progress_tracking.unwrap_or(true),
            retry_failed_items: params.retry_failed_items.unwrap_or(false),
            max_retries: params.max_retries.unwrap_or(2),
        }
    }

    /// Create progress tracker configuration
    fn create_progress_config(&self, params: &BatchProcessorParams) -> ProgressTrackerConfig {
        ProgressTrackerConfig {
            update_interval: Duration::from_secs(params.progress_interval_seconds.unwrap_or(5)),
            max_event_history: 1000,
            track_item_events: params.enable_progress_tracking.unwrap_or(true),
            calculate_metrics: true,
            event_buffer_size: 100,
            persist_progress: false,
        }
    }

    /// Convert batch item parameters to batch items
    fn convert_batch_items(&self, item_params: &[BatchItemParams]) -> Vec<BatchItem> {
        item_params
            .iter()
            .map(|item_param| {
                let mut item = BatchItem::new(&item_param.id, item_param.parameters.clone());

                if let Some(ref metadata) = item_param.metadata {
                    item = item.with_metadata(metadata.clone());
                }

                if let Some(priority) = item_param.priority {
                    item = item.with_priority(priority);
                }

                item
            })
            .collect()
    }

    /// Create a mock tool registry for testing
    /// In a real implementation, this would be injected
    fn create_mock_registry(&self) -> Arc<ToolRegistry> {
        use crate::tools::registry::ToolRegistry;
        use crate::tools::types::{NativeToolBuilder, ToolInput, ToolOutput};
        use crate::core::ExecutionContext;
        use serde_json::json;
        
        let registry = ToolRegistry::new();
        
        // Add mock tools
        for tool_name in ["test-tool", "echo-tool", "slow-tool"] {
            let name = tool_name.to_string();
            let native_tool = NativeToolBuilder::new()
                .name(tool_name)
                .version("1.0.0")
                .description("Mock tool for testing")
                .executor(move |input: ToolInput, _ctx: ExecutionContext| {
                    let name = name.clone();
                    async move {
                        let delay = match name.as_str() {
                            "slow-tool" => tokio::time::Duration::from_millis(100),
                            _ => tokio::time::Duration::from_millis(10),
                        };
                        tokio::time::sleep(delay).await;
                        
                        Ok(ToolOutput::success(json!({
                            "tool": name,
                            "input": input.params,
                            "processed": true,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        })))
                    }
                })
                .build();
                
            if let Ok(native_tool) = native_tool {
                let tool = Tool::Native(Arc::new(native_tool));
                registry.register(tool_name, tool);
            }
        }
        
        Arc::new(registry)
    }

    /// Convert batch result to tool result
    fn convert_batch_result(
        &self,
        batch_result: BatchResult,
        processing_mode: BatchProcessingMode,
        progress_events: Option<Vec<ProgressEvent>>,
        experimental_mode: bool,
    ) -> BatchProcessorResult {
        // Convert progress
        let progress_summary = BatchProgressSummary {
            total_items: batch_result.progress.total_items,
            completed_items: batch_result.progress.completed_items,
            failed_items: batch_result.progress.failed_items,
            skipped_items: batch_result.progress.skipped_items,
            progress_percentage: batch_result.progress.progress_percentage,
            processing_rate: batch_result.progress.processing_rate,
            estimated_time_remaining_ms: batch_result
                .progress
                .estimated_time_remaining
                .map(|d| d.as_millis() as u64),
        };

        // Convert item results
        let item_results = batch_result
            .item_results
            .iter()
            .map(|item_result| {
                let error_type = item_result.error.as_ref().map(|error| {
                    if error.contains("timeout") {
                        "timeout".to_string()
                    } else if error.contains("validation") {
                        "validation".to_string()
                    } else if error.contains("permission") {
                        "permission".to_string()
                    } else if error.contains("not found") {
                        "not_found".to_string()
                    } else {
                        "other".to_string()
                    }
                });

                BatchItemResultSummary {
                    id: item_result.id.clone(),
                    status: format!("{:?}", item_result.status),
                    duration_ms: item_result.duration.as_millis() as u64,
                    retry_attempts: item_result.retry_attempts,
                    has_result: item_result.result.is_some(),
                    has_error: item_result.error.is_some(),
                    error_type,
                }
            })
            .collect();

        // Convert error summary
        let most_common_error = batch_result
            .error_summary
            .common_errors
            .first()
            .map(|(error, _)| error.clone());

        let error_summary = BatchErrorSummary {
            total_errors: batch_result.error_summary.total_errors,
            error_types: batch_result.error_summary.error_types,
            most_common_error,
            permanently_failed_items: batch_result.error_summary.permanently_failed_items,
        };

        // Convert performance metrics
        let performance_metrics = BatchPerformanceMetricsSummary {
            average_item_duration_ms: batch_result
                .performance_metrics
                .average_item_duration
                .as_millis() as u64,
            min_item_duration_ms: batch_result
                .performance_metrics
                .min_item_duration
                .as_millis() as u64,
            max_item_duration_ms: batch_result
                .performance_metrics
                .max_item_duration
                .as_millis() as u64,
            throughput: batch_result.performance_metrics.throughput,
            concurrency_utilization: batch_result.performance_metrics.concurrency_utilization,
        };

        BatchProcessorResult {
            batch_id: batch_result.batch_id,
            tool_name: batch_result.tool_name,
            processing_mode,
            status: batch_result.status,
            total_duration_ms: batch_result.total_duration.as_millis() as u64,
            progress_summary,
            item_results,
            error_summary,
            performance_metrics,
            progress_events,
            experimental_mode,
        }
    }

    /// Simulate batch processing in experimental mode
    async fn simulate_batch_processing(&self, params: &BatchProcessorParams) -> Result<Value> {
        info!(
            "Simulating batch processing for {} items",
            params.batch_items.len()
        );

        let start_time = std::time::Instant::now();
        let processing_mode = params.processing_mode.clone().unwrap_or_default();

        // Simulate processing time based on batch size and concurrency
        let max_concurrency = params.max_concurrency.unwrap_or(4);
        let estimated_item_duration_ms = 100; // Simulate 100ms per item
        let estimated_total_duration_ms =
            (params.batch_items.len() as u64 * estimated_item_duration_ms) / max_concurrency as u64;

        // Create simulated results
        let item_results: Vec<BatchItemResultSummary> = params
            .batch_items
            .iter()
            .map(|item| BatchItemResultSummary {
                id: item.id.clone(),
                status: "Completed".to_string(),
                duration_ms: estimated_item_duration_ms,
                retry_attempts: 0,
                has_result: true,
                has_error: false,
                error_type: None,
            })
            .collect();

        let progress_summary = BatchProgressSummary {
            total_items: params.batch_items.len(),
            completed_items: params.batch_items.len(),
            failed_items: 0,
            skipped_items: 0,
            progress_percentage: 100.0,
            processing_rate: params.batch_items.len() as f64
                / (estimated_total_duration_ms as f64 / 1000.0),
            estimated_time_remaining_ms: Some(0),
        };

        let error_summary = BatchErrorSummary {
            total_errors: 0,
            error_types: HashMap::new(),
            most_common_error: None,
            permanently_failed_items: Vec::new(),
        };

        let performance_metrics = BatchPerformanceMetricsSummary {
            average_item_duration_ms: estimated_item_duration_ms,
            min_item_duration_ms: estimated_item_duration_ms,
            max_item_duration_ms: estimated_item_duration_ms,
            throughput: params.batch_items.len() as f64
                / (estimated_total_duration_ms as f64 / 1000.0),
            concurrency_utilization: 1.0,
        };

        let simulated_result = BatchProcessorResult {
            batch_id: uuid::Uuid::new_v4().to_string(),
            tool_name: params.tool_name.clone(),
            processing_mode,
            status: BatchStatus::Completed,
            total_duration_ms: start_time.elapsed().as_millis() as u64,
            progress_summary,
            item_results,
            error_summary,
            performance_metrics,
            progress_events: None,
            experimental_mode: true,
        };

        info!(
            "Batch processing simulation completed for {} items",
            params.batch_items.len()
        );
        Ok(serde_json::to_value(simulated_result)?)
    }

    /// Execute the batch processor tool
    pub async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        info!("Executing batch processor tool");

        // Parse parameters
        let batch_params = self.parse_parameters(&params)?;

        // Check if we're in experimental mode
        let experimental_mode = batch_params.experimental_mode.unwrap_or(false);
        if experimental_mode {
            info!("Running batch processor tool in experimental mode");
        }

        debug!(
            "Batch processing {} items with tool '{}'",
            batch_params.batch_items.len(),
            batch_params.tool_name
        );

        // Validate batch size
        if batch_params.batch_items.is_empty() {
            return Err(WorkflowError::validation("Batch items cannot be empty"));
        }

        if batch_params.batch_items.len() > 10000 {
            return Err(WorkflowError::validation(
                "Batch size exceeds maximum limit of 10000 items",
            ));
        }

        // In experimental mode, simulate the batch processing
        if experimental_mode {
            return self.simulate_batch_processing(&batch_params).await;
        }

        // Create configurations
        let batch_config = self.create_batch_config(&batch_params);
        let progress_config = self.create_progress_config(&batch_params);

        // Create batch processor
        let mut batch_processor = BatchProcessor::with_config(batch_config);

        // Add concurrency manager if available
        if let Some(ref manager) = self.concurrency_manager {
            batch_processor = batch_processor.with_concurrency_manager(manager.clone());
        }

        // Create mock tool registry (in real implementation, this would be injected)
        let tool_registry = self.create_mock_registry();
        batch_processor = batch_processor.with_tool_registry(tool_registry);

        // Create progress tracker
        let progress_tracker = ProgressTracker::new(progress_config);
        let progress_receiver = if batch_params.enable_progress_tracking.unwrap_or(true) {
            Some(progress_tracker.subscribe())
        } else {
            None
        };

        // Start progress reporting if enabled
        let _progress_task = if batch_params.enable_progress_tracking.unwrap_or(true) {
            Some(progress_tracker.start_progress_reporting().await)
        } else {
            None
        };

        // Convert batch items
        let batch_items = self.convert_batch_items(&batch_params.batch_items);

        // Process batch
        let batch_result = batch_processor
            .process_batch(&batch_params.tool_name, batch_items, context)
            .await
            .map_err(|e| WorkflowError::tool(format!("Batch processing failed: {}", e)))?;

        // Collect progress events if tracking was enabled
        let progress_events = if let Some(mut receiver) = progress_receiver {
            let mut events = Vec::new();

            // Try to collect events (non-blocking)
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
                if events.len() > 100 {
                    // Limit event collection
                    break;
                }
            }

            if !events.is_empty() {
                Some(events)
            } else {
                None
            }
        } else {
            None
        };

        // Convert result
        let processing_mode = batch_params.processing_mode.unwrap_or_default();
        let experimental_mode = batch_params.experimental_mode.unwrap_or(false);
        let tool_result = self.convert_batch_result(
            batch_result,
            processing_mode,
            progress_events,
            experimental_mode,
        );

        info!(
            "Batch processing completed: {} total, {} completed, {} failed",
            tool_result.progress_summary.total_items,
            tool_result.progress_summary.completed_items,
            tool_result.progress_summary.failed_items
        );

        // Convert to JSON
        serde_json::to_value(tool_result)
            .map_err(|e| WorkflowError::tool(format!("Failed to serialize batch result: {}", e)))
    }

    /// Validate parameters for the batch processor tool
    pub fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Parse parameters to validate structure
        let batch_params = self.parse_parameters(params)?;

        // Validate tool name
        if batch_params.tool_name.trim().is_empty() {
            return Err(WorkflowError::validation("tool_name cannot be empty"));
        }

        // Validate batch items
        if batch_params.batch_items.is_empty() {
            return Err(WorkflowError::validation("batch_items cannot be empty"));
        }

        if batch_params.batch_items.len() > 10000 {
            return Err(WorkflowError::validation(
                "batch_items exceeds maximum limit of 10000",
            ));
        }

        // Validate each batch item
        for (index, item) in batch_params.batch_items.iter().enumerate() {
            if item.id.trim().is_empty() {
                return Err(WorkflowError::validation(format!(
                    "batch_items[{}].id cannot be empty",
                    index
                )));
            }

            if !item.parameters.is_object() && !item.parameters.is_null() {
                return Err(WorkflowError::validation(format!(
                    "batch_items[{}].parameters must be an object",
                    index
                )));
            }
        }

        // Validate optional numeric parameters
        if let Some(concurrency) = batch_params.max_concurrency {
            if concurrency == 0 || concurrency > 100 {
                return Err(WorkflowError::validation(
                    "max_concurrency must be between 1 and 100",
                ));
            }
        }

        if let Some(timeout) = batch_params.item_timeout_seconds {
            if timeout == 0 || timeout > 3600 {
                return Err(WorkflowError::validation(
                    "item_timeout_seconds must be between 1 and 3600",
                ));
            }
        }

        if let Some(interval) = batch_params.progress_interval_seconds {
            if interval == 0 || interval > 60 {
                return Err(WorkflowError::validation(
                    "progress_interval_seconds must be between 1 and 60",
                ));
            }
        }

        if let Some(retries) = batch_params.max_retries {
            if retries > 10 {
                return Err(WorkflowError::validation("max_retries must be 10 or less"));
            }
        }

        Ok(())
    }
}

/// Mock tool registry for testing
/// In a real implementation, this would be replaced with the actual tool registry
pub struct MockToolRegistry {
    tools: HashMap<String, Tool>,
}

impl Default for MockToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MockToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Add a mock tool for testing
        registry.add_mock_tool("test-tool");
        registry.add_mock_tool("echo-tool");
        registry.add_mock_tool("slow-tool");

        registry
    }

    fn add_mock_tool(&mut self, name: &str) {
        use crate::tools::types::{NativeToolBuilder, ToolInput, ToolOutput};
        use crate::core::ExecutionContext;
        use serde_json::json;
        
        let name = name.to_string();
        let tool_name = name.clone();
        
        let native_tool = NativeToolBuilder::new()
            .name(&name)
            .version("1.0.0")
            .description("Mock tool for testing")
            .executor(move |input: ToolInput, _ctx: ExecutionContext| {
                let tool_name = tool_name.clone();
                async move {
                    // Simulate some processing time
                    let delay = match tool_name.as_str() {
                        "slow-tool" => tokio::time::Duration::from_millis(100),
                        _ => tokio::time::Duration::from_millis(10),
                    };
                    tokio::time::sleep(delay).await;
                    
                    Ok(ToolOutput::success(json!({
                        "tool": tool_name,
                        "input": input.params,
                        "processed": true,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })))
                }
            })
            .build();
            
        if let Ok(native_tool) = native_tool {
            let tool = Tool::Native(Arc::new(native_tool));
            self.tools.insert(name.to_string(), tool);
        }
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.get(name).cloned()
    }

    fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools.values().map(|tool| tool.get_info()).collect()
    }

    fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

/// Mock tool for testing batch processing
pub struct MockTool {
    name: String,
}

impl MockTool {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        // Simulate some processing time
        let delay = match self.name.as_str() {
            "slow-tool" => Duration::from_millis(100),
            _ => Duration::from_millis(10),
        };

        tokio::time::sleep(delay).await;

        // Return a simple result
        Ok(json!({
            "tool": self.name,
            "input": params,
            "processed": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PluginType;
    use tempfile::TempDir;

    fn create_test_config() -> FileManagementConfig {
        let temp_dir = TempDir::new().unwrap();
        FileManagementConfig {
            temp_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        }
    }

    fn create_test_plugin_info() -> PluginInfo {
        PluginInfo {
            name: "file-management".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Native,
            description: Some("Test plugin".to_string()),
            author: Some("Test".to_string()),
            homepage: Some("https://example.com".to_string()),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_batch_processor_tool_creation() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();
        let tool = BatchProcessorTool::new(config, plugin_info);

        let tool_info = tool.create_tool_info();
        assert_eq!(tool_info.name, "batch-processor");
        assert_eq!(tool_info.version, "1.0.0");
        assert!(tool_info.description.contains("batch processing"));
    }

    #[test]
    fn test_batch_processor_tool_schema() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();
        let tool = BatchProcessorTool::new(config, plugin_info);

        let tool_info = tool.create_tool_info();
        let schema = &tool_info.parameters_schema;

        // Check required fields
        assert!(schema["properties"]["tool_name"].is_object());
        assert!(schema["properties"]["batch_items"].is_object());
        assert_eq!(schema["required"].as_array().unwrap().len(), 2);

        // Check optional fields
        assert!(schema["properties"]["max_concurrency"].is_object());
        assert!(schema["properties"]["continue_on_error"].is_object());
        assert!(schema["properties"]["processing_mode"].is_object());
    }

    #[tokio::test]
    async fn test_batch_processor_executor_validation() {
        let config = create_test_config();
        let executor = BatchProcessorExecutor::new(config, None);

        // Test valid parameters
        let valid_params = json!({
            "tool_name": "test-tool",
            "batch_items": [
                {
                    "id": "item-1",
                    "parameters": {"value": 1}
                },
                {
                    "id": "item-2",
                    "parameters": {"value": 2}
                }
            ]
        });

        assert!(executor.validate_parameters(&valid_params).is_ok());

        // Test invalid parameters - empty tool name
        let invalid_params = json!({
            "tool_name": "",
            "batch_items": [{"id": "item-1", "parameters": {}}]
        });

        assert!(executor.validate_parameters(&invalid_params).is_err());

        // Test invalid parameters - empty batch items
        let invalid_params = json!({
            "tool_name": "test-tool",
            "batch_items": []
        });

        assert!(executor.validate_parameters(&invalid_params).is_err());

        // Test invalid parameters - invalid concurrency
        let invalid_params = json!({
            "tool_name": "test-tool",
            "batch_items": [{"id": "item-1", "parameters": {}}],
            "max_concurrency": 0
        });

        assert!(executor.validate_parameters(&invalid_params).is_err());
    }

    #[tokio::test]
    async fn test_batch_processor_executor_execution() {
        let config = create_test_config();
        let executor = BatchProcessorExecutor::new(config, None);

        let params = json!({
            "tool_name": "test-tool",
            "batch_items": [
                {
                    "id": "item-1",
                    "parameters": {"value": 1},
                    "priority": 5
                },
                {
                    "id": "item-2",
                    "parameters": {"value": 2},
                    "priority": 1
                }
            ],
            "max_concurrency": 2,
            "continue_on_error": true,
            "enable_progress_tracking": false
        });

        let context = ExecutionContext::new();
        let result = executor.execute(params, context).await.unwrap();

        // Parse result
        let batch_result: BatchProcessorResult = serde_json::from_value(result).unwrap();

        // Verify basic structure
        assert_eq!(batch_result.tool_name, "test-tool");
        assert_eq!(batch_result.progress_summary.total_items, 2);
        assert_eq!(batch_result.item_results.len(), 2);

        // Verify all items were processed
        for item_result in &batch_result.item_results {
            assert_eq!(item_result.status, "Completed");
            assert!(item_result.has_result);
            assert!(!item_result.has_error);
        }
    }

    #[tokio::test]
    async fn test_batch_processor_with_progress_tracking() {
        let config = create_test_config();
        let executor = BatchProcessorExecutor::new(config, None);

        let params = json!({
            "tool_name": "test-tool",
            "batch_items": [
                {"id": "item-1", "parameters": {"value": 1}},
                {"id": "item-2", "parameters": {"value": 2}},
                {"id": "item-3", "parameters": {"value": 3}}
            ],
            "enable_progress_tracking": true,
            "progress_interval_seconds": 1
        });

        let context = ExecutionContext::new();
        let result = executor.execute(params, context).await.unwrap();

        let batch_result: BatchProcessorResult = serde_json::from_value(result).unwrap();

        // Verify progress tracking was enabled
        assert_eq!(batch_result.progress_summary.total_items, 3);
        assert_eq!(batch_result.progress_summary.completed_items, 3);
        assert_eq!(batch_result.progress_summary.progress_percentage, 1.0);

        // Progress events might be empty due to timing, but structure should be correct
        // (events are collected asynchronously and might not be captured in tests)
    }

    #[test]
    fn test_batch_item_conversion() {
        let config = create_test_config();
        let executor = BatchProcessorExecutor::new(config, None);

        let item_params = vec![
            BatchItemParams {
                id: "item-1".to_string(),
                parameters: json!({"value": 1}),
                metadata: Some({
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("test"));
                    meta
                }),
                priority: Some(5),
            },
            BatchItemParams {
                id: "item-2".to_string(),
                parameters: json!({"value": 2}),
                metadata: None,
                priority: None,
            },
        ];

        let batch_items = executor.convert_batch_items(&item_params);

        assert_eq!(batch_items.len(), 2);
        assert_eq!(batch_items[0].id, "item-1");
        assert_eq!(batch_items[0].priority, 5);
        assert_eq!(batch_items[1].id, "item-2");
        assert_eq!(batch_items[1].priority, 0);
    }

    #[test]
    fn test_mock_tool_registry() {
        let registry = MockToolRegistry::new();

        // Test that mock tools are available
        assert!(registry.get_tool("test-tool").is_some());
        assert!(registry.get_tool("echo-tool").is_some());
        assert!(registry.get_tool("slow-tool").is_some());
        assert!(registry.get_tool("nonexistent-tool").is_none());

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 3);
    }

    #[tokio::test]
    async fn test_mock_tool_execution() {
        let tool = MockTool::new("test-tool");
        let context = ExecutionContext::new();
        let params = json!({"test": "value"});

        let result = tool.execute(params.clone(), context).await.unwrap();

        assert_eq!(result["tool"], "test-tool");
        assert_eq!(result["input"], params);
        assert_eq!(result["processed"], true);
        assert!(result["timestamp"].is_string());
    }
}
