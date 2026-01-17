//! File Management Plugin Module
//!
//! This module contains all components for the file management plugin including
//! tools, utilities, and error handling.

pub mod ac_automaton;
pub mod batch_confirmation_tool;
pub mod batch_processor;
pub mod batch_processor_tool;
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
pub mod tools;
pub mod utils;

// Re-export main plugin components
pub use batch_processor::{
    BatchItem, BatchItemResult, BatchItemStatus, BatchPerformanceMetrics, BatchProcessor,
    BatchProcessorConfig, BatchProgress, BatchResult, BatchStatus,
};
pub use batch_processor_tool::{
    BatchItemParams, BatchProcessingMode, BatchProcessorParams, BatchProcessorResult,
    BatchProcessorTool,
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
pub use plugin::{FileManagementConfig, FileManagementPlugin, FileManagementPluginBuilder};
pub use progress_tracker::{
    AggregatedStats, PerformanceTrend, ProgressEvent, ProgressTracker, ProgressTrackerConfig,
    ToolUsageStats,
};
pub use registry::FileManagementToolRegistry;
pub use text_processor_tool::{
    ChineseProcessingConfig, TextOperation, TextOutputFormat, TextProcessorParams,
    TextProcessorResult, TextProcessorTool,
};
// pub use rule_config::{
//     RuleConfigLoader, KeywordCombination, EnhancedClassificationRule,
// };
pub use ac_automaton::{
    AutomatonConfig, AutomatonError, AutomatonNode, AutomatonResult, AutomatonStats, Pattern,
    PatternMatch,
};
pub use batch_confirmation_tool::{
    create_batch_confirmation_tool, create_batch_confirmation_tool_with_config,
    BatchConfirmationConfig, BatchConfirmationParams, BatchConfirmationResult,
    BatchConfirmationTool, BatchDecision, BatchDecisionType, BatchOptions, BatchSummary,
    ConfirmationMethod, ConfirmationStrategy, ModificationType, OperationBatch,
    OperationModification, OverallSummary, ProcessedBatch, RecommendedAction, ReversibilitySummary,
    RiskDistribution, UserPreferences,
};
pub use error_recovery::{
    ErrorRecoveryManager, RecoveryAttempt, RecoveryConfig, RecoverySession, RecoveryStats,
    RecoveryStrategy,
};
pub use monitoring::{
    Alert, AlertSeverity, AlertType, AuditEntry, AuditResult, ErrorTracker, FileManagementMonitor,
    MonitoringConfig, MonitoringStats, OperationMetrics, ResourceUsage,
};
pub use performance::{
    CacheStats, CachedResult, CompressionUtils, MemoryPoolStats, OptimizedFileOperationManager,
    PerformanceStats, StreamingUtils,
};
pub use result_confirmation_tool::{
    create_result_confirmation_tool, create_result_confirmation_tool_with_config, BackupStrategy,
    ConfirmationMode, ConfirmationPhase, ExecutionSummary, PhaseOutput, PhaseType,
    ResourceRequirements, ResultConfirmationConfig, ResultConfirmationParams,
    ResultConfirmationResult, ResultConfirmationTool, ReviewOptions, RiskAssessment,
    RollbackOperation, RollbackOptions, RollbackPlan, RollbackType,
};
pub use result_review_tool::{
    create_result_review_tool, create_result_review_tool_with_config, ConfirmationDecision,
    ConfirmationDetail, ConfirmationOptions, DefaultAction, ExperimentalResult, OperationImpact,
    ResultReviewConfig, ResultReviewParams, ResultReviewResult, ResultReviewTool, ReviewMode,
    ReviewSummary, RiskLevel,
};
pub use utils::{
    ChineseTextType, CommonFolderInfo, DuplicateHandling, ExperimentalMode, FileOperationManager,
    FolderComparisonResult, FolderLocationInfo, FolderMergeError, FolderMergeResult, FolderMerger,
    FolderMergerConfig, HumanDecisionContext, MergeDirection, MergeOperationStats,
    MergeRecommendation, MergeStrategy, MixedTextResult, PathUtils, PinyinResult, PinyinStyle,
    SingleFolderMergeResult, TextNormalizationConfig, TextProcessor, UniqueFolderInfo,
    ValidationUtils,
};
