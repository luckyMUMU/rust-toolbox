//! File Management Plugin
//!
//! This crate provides file management capabilities including:
//! - AI-powered folder classification
//! - Batch file processing
//! - Text processing with Chinese support
//! - Human decision integration
//! - Folder merging with conflict resolution

pub mod batch_confirmation_tool;
pub mod batch_processor;
pub mod batch_processor_tool;
pub mod classification_flow;
pub mod classification_tool;
pub mod error;
pub mod error_recovery;
pub mod human_decision_tool;
pub mod monitoring;
pub mod performance;
pub mod plugin;
pub mod progress_tracker;
pub mod registry;
pub mod result_confirmation_tool;
pub mod result_review_tool;
pub mod rule_config;
pub mod text_processor_tool;
pub mod utils;

// Re-export main components for easy access
pub use batch_processor::{
    BatchItem, BatchItemResult, BatchItemStatus, BatchPerformanceMetrics, BatchProcessor,
    BatchProcessorConfig, BatchProgress, BatchResult, BatchStatus,
};
pub use batch_processor_tool::{
    BatchItemParams, BatchProcessingMode, BatchProcessorParams, BatchProcessorResult,
    BatchProcessorTool,
};
pub use classification_flow::{
    AmbiguityDetectorTool, AutomatonBuilderTool, DirectoryScannerTool, ExperimentalCheckTool,
    FolderNamePreprocessorTool, ParallelMatcherTool, ReportGeneratorTool, ResultMergerTool,
    RuleLoaderTool, RulePreprocessorTool, ScoreCalculatorTool,
};
pub use classification_tool::{
    ClassificationCandidate, ClassificationEngine, ClassificationOutputFormat,
    ClassificationParams, ClassificationResult, ClassificationRule, ClassificationRules,
    ClassificationStatus, ClassificationTool,
};
pub use error::{FileManagementError, FileManagementResult};
pub use human_decision_tool::{
    create_human_decision_tool, DecisionContext, DecisionOption, HumanDecisionExecutor,
    HumanDecisionParams, HumanDecisionResult,
};
pub use plugin::{
    FileManagementConfig, FileManagementPlugin, FileManagementPluginBuilder,
};
pub use progress_tracker::{BatchItemProgress, BatchProgressEvent, ProgressTracker};
pub use result_confirmation_tool::{
    BackupStrategy, ConfirmationPhase, ConfirmationStatus, ResourceRequirements,
    ResultConfirmationConfig, ResultConfirmationTool, RollbackCapability,
};
pub use result_review_tool::{
    ImpactAnalysis, OperationPreview, ResourceImpact, ResultReviewConfig, ResultReviewReport,
    ResultReviewTool, RiskAssessment, RiskLevel,
};
pub use text_processor_tool::{
    ChineseProcessingConfig, PinyinStyle, TextOperation, TextProcessorParams,
    TextProcessorResult, TextProcessorTool,
};
pub use utils::{
    create_default_rules, ClassificationResultSummary, DuplicateHandling, FileOperationManager,
    FileSystemUtils, FolderMergeConfig, FolderMerger, MergeConfig, MergeStrategy, PathUtils,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
