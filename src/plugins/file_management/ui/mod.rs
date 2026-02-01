//! UI module for file management plugin
//!
//! Contains human decision and confirmation tools.

pub mod batch_confirmation_tool;
pub mod human_decision_tool;
pub mod result_confirmation_tool;
pub mod result_review_tool;

// Re-export UI components
pub use batch_confirmation_tool::{
    create_batch_confirmation_tool, create_batch_confirmation_tool_with_config,
    BatchConfirmationConfig, BatchConfirmationParams, BatchConfirmationResult,
    BatchConfirmationTool, BatchDecision, BatchDecisionType, BatchOptions, BatchSummary,
    ConfirmationMethod, ConfirmationStrategy, ModificationType, OperationBatch,
    OperationModification, OverallSummary, ProcessedBatch, RecommendedAction, ReversibilitySummary,
    RiskDistribution, UserPreferences,
};
pub use human_decision_tool::{
    create_human_decision_tool, DecisionContext, DecisionOption, HumanDecisionExecutor,
    HumanDecisionParams, HumanDecisionResult,
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
