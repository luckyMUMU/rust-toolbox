//! File Management Plugin Module
//!
//! This module contains all components for the file management plugin including
//! tools, utilities, and error handling.
//!
//! ## Module Structure
//!
//! The module is organized into submodules:
//!
//! - `core`: Error handling, recovery, and shared types
//! - `classification`: Folder classification tools
//! - `batch`: Batch processing tools
//! - `text`: Text processing tools
//! - `ui`: Human decision and confirmation tools
//! - `utils`: Utilities, monitoring, performance, and registry
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.

// Core modules
pub mod core;
pub mod classification;
pub mod batch;
pub mod text;
pub mod ui;
pub mod utils;

// Plugin definition stays at root level
pub mod plugin;

// Re-export core types for backward compatibility
pub use core::{
    ErrorContext, ErrorSeverity,
    FileManagementError, FileManagementResult, RecoverySuggestion,
    error::{ErrorContext as _, ErrorSeverity as _, RecoverySuggestion as _},
    error_recovery::{
        ErrorRecoveryManager, RecoveryAttempt, RecoveryConfig, RecoverySession,
        RecoveryStats, RecoveryStrategy,
    },
};

// Re-export classification types for backward compatibility
pub use classification::{
    AmbiguityDetectorTool, AutomatonBuilderTool, ClassificationCandidate,
    ClassificationEngine, ClassificationOutputFormat, ClassificationParams,
    ClassificationResult, ClassificationRule, ClassificationRules, ClassificationStatus,
    ClassificationTool, DirectoryScannerTool, ExperimentalCheckTool,
    FolderNamePreprocessorTool, ParallelMatcherTool, ReportGeneratorTool,
    ResultMergerTool, RuleLoaderTool, RulePreprocessorTool, ScoreCalculatorTool,
};

// Re-export batch types for backward compatibility
pub use batch::{
    AggregatedStats, BatchItem, BatchItemParams, BatchItemResult, BatchItemStatus,
    BatchPerformanceMetrics, BatchProcessingMode, BatchProcessor, BatchProcessorConfig,
    BatchProcessorParams, BatchProcessorResult, BatchProcessorTool, BatchProgress,
    BatchResult, BatchStatus, PerformanceTrend, ProgressEvent, ProgressTracker,
    ProgressTrackerConfig, ToolUsageStats,
};

// Re-export text types for backward compatibility
pub use text::{
    ChineseProcessingConfig, TextOperation, TextOutputFormat, TextProcessorParams,
    TextProcessorResult, TextProcessorTool,
};

// Re-export UI types for backward compatibility
pub use ui::{
    BatchConfirmationConfig, BatchConfirmationParams, BatchConfirmationResult,
    BatchConfirmationTool, BatchDecision, BatchDecisionType, BatchOptions, BatchSummary,
    ConfirmationDecision, ConfirmationDetail, ConfirmationMethod, ConfirmationMode,
    ConfirmationOptions, ConfirmationPhase, ConfirmationStrategy, DecisionContext,
    DecisionOption, DefaultAction, ExecutionSummary, ExperimentalResult, HumanDecisionExecutor,
    HumanDecisionParams, HumanDecisionResult, ModificationType, OperationBatch,
    OperationImpact, OperationModification, OverallSummary, PhaseOutput, PhaseType,
    ProcessedBatch, RecommendedAction, ResourceRequirements, ResultConfirmationConfig,
    ResultConfirmationParams, ResultConfirmationResult, ResultConfirmationTool,
    ResultReviewConfig, ResultReviewParams, ResultReviewResult, ResultReviewTool,
    ReviewMode, ReviewOptions, ReviewSummary, RiskAssessment, RiskDistribution, RiskLevel,
    RollbackOperation, RollbackOptions, RollbackPlan, RollbackType, ReversibilitySummary,
    UserPreferences,
    batch_confirmation_tool::{
        create_batch_confirmation_tool, create_batch_confirmation_tool_with_config,
    },
    human_decision_tool::create_human_decision_tool,
    result_confirmation_tool::{
        create_result_confirmation_tool, create_result_confirmation_tool_with_config,
    },
    result_review_tool::{
        create_result_review_tool, create_result_review_tool_with_config,
    },
};

// Re-export utility types for backward compatibility
pub use utils::{
    Alert, AlertSeverity, AlertType, AuditEntry, AuditResult, CacheStats, CachedResult,
    ChineseTextType, CommonFolderInfo, CompressionUtils, DuplicateHandling, ErrorTracker,
    ExperimentalMode, FileManagementMonitor, FileManagementToolRegistry, FileOperationManager,
    FolderComparisonResult, FolderLocationInfo, FolderMergeError, FolderMergeResult, FolderMerger,
    FolderMergerConfig, HumanDecisionContext, MemoryPoolStats, MergeDirection,
    MergeOperationStats, MergeRecommendation, MergeStrategy, MixedTextResult, MonitoringConfig,
    MonitoringStats, OperationMetrics, OptimizedFileOperationManager, PathUtils, PerformanceStats,
    PinyinResult, PinyinStyle, ResourceUsage, SingleFolderMergeResult, StreamingUtils,
    TextNormalizationConfig, TextProcessor, UniqueFolderInfo, ValidationUtils,
};

// Re-export plugin types
pub use plugin::{
    FileManagementConfig, FileManagementPerformanceConfig, FileManagementPlugin,
    FileManagementPluginBuilder,
};

// Re-export Aho-Corasick types from tools module
pub use crate::tools::algo::ac_automaton::{
    AutomatonConfig, AutomatonError, AutomatonNode, AutomatonResult, AutomatonStats, Pattern,
    PatternMatch,
};
