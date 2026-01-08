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
pub mod utils;

// Re-export main plugin components
pub use plugin::{
    FileManagementPlugin, FileManagementPluginBuilder, FileManagementConfig,
};
pub use error::{FileManagementError, FileManagementResult};
pub use registry::FileManagementToolRegistry;
pub use human_decision_tool::{
    HumanDecisionParams, DecisionContext, DecisionOption, HumanDecisionResult,
    HumanDecisionExecutor, create_human_decision_tool,
};
pub use batch_processor::{
    BatchProcessor, BatchProcessorConfig, BatchItem, BatchResult, BatchStatus,
    BatchProgress, BatchItemResult, BatchItemStatus, BatchPerformanceMetrics,
};
pub use batch_processor_tool::{
    BatchProcessorTool, BatchProcessorParams, BatchProcessorResult,
    BatchProcessingMode, BatchItemParams,
};
pub use progress_tracker::{
    ProgressTracker, ProgressTrackerConfig, ProgressEvent, AggregatedStats,
    PerformanceTrend, ToolUsageStats,
};
pub use text_processor_tool::{
    TextProcessorTool, TextProcessorParams, TextProcessorResult,
    TextOperation, ChineseProcessingConfig, TextOutputFormat,
};
pub use classification_tool::{
    ClassificationTool, ClassificationEngine, ClassificationParams, ClassificationResult,
    ClassificationRules, ClassificationRule, ClassificationCandidate, ClassificationStatus,
    ClassificationOutputFormat,
};
// pub use rule_config::{
//     RuleConfigLoader, KeywordCombination, EnhancedClassificationRule,
// };
pub use utils::{
    FileOperationManager, TextProcessor, PathUtils, ValidationUtils,
    ExperimentalMode, HumanDecisionContext, TextNormalizationConfig,
    PinyinStyle, PinyinResult, ChineseTextType, MixedTextResult,
    FolderMerger, FolderMergerConfig, FolderComparisonResult, CommonFolderInfo,
    FolderLocationInfo, UniqueFolderInfo, MergeDirection, MergeRecommendation,
    MergeStrategy, DuplicateHandling, FolderMergeResult, SingleFolderMergeResult,
    FolderMergeError, MergeOperationStats,
};
pub use ac_automaton::{
    Pattern, AutomatonNode, PatternMatch, AutomatonConfig, AutomatonStats,
    AutomatonError, AutomatonResult,
};
pub use result_review_tool::{
    ResultReviewTool, ResultReviewConfig, ResultReviewParams, ReviewMode,
    ConfirmationOptions, DefaultAction, ExperimentalResult, OperationImpact,
    RiskLevel, ResultReviewResult, ReviewSummary, ConfirmationDetail,
    ConfirmationDecision, create_result_review_tool, create_result_review_tool_with_config,
};
pub use batch_confirmation_tool::{
    BatchConfirmationTool, BatchConfirmationConfig, BatchConfirmationParams,
    ConfirmationStrategy, BatchOptions, UserPreferences, OperationBatch,
    BatchSummary, RiskDistribution, ReversibilitySummary, RecommendedAction,
    BatchConfirmationResult, ProcessedBatch, BatchDecision, BatchDecisionType,
    ConfirmationMethod, OperationModification, ModificationType, OverallSummary,
    create_batch_confirmation_tool, create_batch_confirmation_tool_with_config,
};
pub use result_confirmation_tool::{
    ResultConfirmationTool, ResultConfirmationConfig, ResultConfirmationParams,
    ConfirmationMode, ReviewOptions, RollbackOptions, BackupStrategy,
    ResultConfirmationResult, ConfirmationPhase, PhaseType, PhaseOutput,
    RollbackPlan, RollbackOperation, RollbackType, ExecutionSummary,
    RiskAssessment, ResourceRequirements,
    create_result_confirmation_tool, create_result_confirmation_tool_with_config,
};
pub use performance::{
    OptimizedFileOperationManager, PerformanceStats, MemoryPoolStats, CacheStats,
    CachedResult, StreamingUtils, CompressionUtils,
};
pub use error_recovery::{
    ErrorRecoveryManager, RecoveryConfig, RecoveryStats, RecoveryAttempt,
    RecoverySession, RecoveryStrategy,
};
pub use monitoring::{
    FileManagementMonitor, MonitoringConfig, MonitoringStats, AuditEntry,
    AuditResult, ResourceUsage, OperationMetrics, ErrorTracker, Alert,
    AlertType, AlertSeverity,
};