//! Batch processing module for file management plugin
//!
//! Contains tools for batch file operations.

pub mod batch_processor;
pub mod batch_processor_tool;
pub mod progress_tracker;

// Re-export batch components
pub use batch_processor::{
    BatchItem, BatchItemResult, BatchItemStatus, BatchPerformanceMetrics, BatchProcessor,
    BatchProcessorConfig, BatchProgress, BatchResult, BatchStatus,
};
pub use batch_processor_tool::{
    BatchItemParams, BatchProcessingMode, BatchProcessorParams, BatchProcessorResult,
    BatchProcessorTool,
};
pub use progress_tracker::{
    AggregatedStats, PerformanceTrend, ProgressEvent, ProgressTracker, ProgressTrackerConfig,
    ToolUsageStats,
};
