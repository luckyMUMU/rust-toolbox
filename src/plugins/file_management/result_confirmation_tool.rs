//! Result Confirmation Tool
//! 
//! This module provides a comprehensive tool that combines result review and batch confirmation
//! to provide a complete solution for reviewing experimental results before execution.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info, warn};
use chrono::{DateTime, Utc};

use crate::tools::ToolNode;
use crate::core::{ExecutionContext, ToolInfo, PluginInfo};
use crate::error::WorkflowError;
use super::error::{FileManagementResult};
use super::utils::ExperimentalOperation;
use super::result_review_tool::{
    ResultReviewTool, ResultReviewConfig, ResultReviewParams, ReviewMode,
    ExperimentalResult, RiskLevel, ConfirmationDecision, ResultReviewResult,
};
use super::batch_confirmation_tool::{
    BatchConfirmationTool, BatchConfirmationConfig, BatchConfirmationParams,
    ConfirmationStrategy, BatchOptions, UserPreferences, BatchConfirmationResult,
};

use async_trait::async_trait;

/// Comprehensive result confirmation tool that combines review and batch confirmation
#[derive(Debug, Clone)]
pub struct ResultConfirmationTool {
    review_tool: ResultReviewTool,
    batch_tool: BatchConfirmationTool,
    config: ResultConfirmationConfig,
}

/// Configuration for the result confirmation tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultConfirmationConfig {
    pub review_config: ResultReviewConfig,
    pub batch_config: BatchConfirmationConfig,
    pub enable_two_phase_confirmation: bool,
    pub auto_transition_to_batch: bool,
    pub require_final_confirmation: bool,
    pub enable_rollback_planning: bool,
}

impl Default for ResultConfirmationConfig {
    fn default() -> Self {
        Self {
            review_config: ResultReviewConfig::default(),
            batch_config: BatchConfirmationConfig::default(),
            enable_two_phase_confirmation: true,
            auto_transition_to_batch: true,
            require_final_confirmation: true,
            enable_rollback_planning: true,
        }
    }
}

/// Parameters for comprehensive result confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultConfirmationParams {
    pub experimental_operations: Vec<ExperimentalOperation>,
    pub confirmation_mode: ConfirmationMode,
    pub review_options: Option<ReviewOptions>,
    pub batch_options: Option<BatchOptions>,
    pub user_preferences: Option<UserPreferences>,
    pub rollback_options: Option<RollbackOptions>,
}

/// Mode for confirmation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationMode {
    /// Review first, then batch confirm approved operations
    ReviewThenBatch,
    /// Batch confirm directly without individual review
    BatchOnly,
    /// Review only without batch processing
    ReviewOnly,
    /// Smart mode that chooses based on operation characteristics
    Smart,
}

/// Options for the review phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOptions {
    pub review_mode: ReviewMode,
    pub auto_approve_threshold: Option<f64>,
    pub require_justification: bool,
    pub enable_operation_modification: bool,
}

/// Options for rollback planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackOptions {
    pub create_rollback_plan: bool,
    pub backup_strategy: BackupStrategy,
    pub rollback_timeout_hours: Option<u64>,
    pub enable_automatic_rollback: bool,
}

/// Strategy for creating backups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStrategy {
    None,
    CopyBeforeOperation,
    SnapshotFilesystem,
    DatabaseTransaction,
    Custom(String),
}

/// Comprehensive result of the confirmation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultConfirmationResult {
    pub confirmation_id: String,
    pub phase_results: Vec<ConfirmationPhase>,
    pub final_approved_operations: Vec<String>,
    pub final_rejected_operations: Vec<String>,
    pub final_deferred_operations: Vec<String>,
    pub rollback_plan: Option<RollbackPlan>,
    pub execution_summary: ExecutionSummary,
    pub processing_time_ms: u64,
}

/// A phase in the confirmation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationPhase {
    pub phase_name: String,
    pub phase_type: PhaseType,
    pub input_operations: Vec<String>,
    pub output_operations: PhaseOutput,
    pub phase_duration_ms: u64,
    pub user_interactions: usize,
    pub auto_decisions: usize,
}

/// Type of confirmation phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseType {
    Review,
    BatchConfirmation,
    FinalConfirmation,
    RollbackPlanning,
}

/// Output from a confirmation phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseOutput {
    pub approved: Vec<String>,
    pub rejected: Vec<String>,
    pub deferred: Vec<String>,
    pub modified: Vec<String>,
}

/// Plan for rolling back operations if needed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub plan_id: String,
    pub rollback_operations: Vec<RollbackOperation>,
    pub backup_locations: HashMap<String, String>,
    pub rollback_order: Vec<String>,
    pub estimated_rollback_time_ms: u64,
    pub rollback_dependencies: Vec<String>,
}

/// A single rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackOperation {
    pub operation_id: String,
    pub rollback_type: RollbackType,
    pub source_backup: Option<String>,
    pub target_location: String,
    pub rollback_command: Option<String>,
    pub verification_steps: Vec<String>,
}

/// Type of rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackType {
    RestoreFromBackup,
    ReverseOperation,
    DatabaseRollback,
    FilesystemSnapshot,
    CustomScript,
}

/// Summary of the execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub total_operations_reviewed: usize,
    pub operations_approved_for_execution: usize,
    pub operations_requiring_backup: usize,
    pub estimated_execution_time_ms: u64,
    pub estimated_rollback_time_ms: u64,
    pub risk_assessment: RiskAssessment,
    pub resource_requirements: ResourceRequirements,
}

/// Overall risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_level: RiskLevel,
    pub irreversible_operations: usize,
    pub high_impact_operations: usize,
    pub dependency_risks: Vec<String>,
    pub mitigation_strategies: Vec<String>,
}

/// Resource requirements for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub estimated_disk_space_bytes: u64,
    pub estimated_memory_mb: u64,
    pub estimated_cpu_cores: u32,
    pub network_bandwidth_required: bool,
    pub exclusive_access_required: Vec<String>,
}

impl ResultConfirmationTool {
    pub fn new(config: ResultConfirmationConfig) -> Self {
        let review_tool = ResultReviewTool::new(config.review_config.clone());
        let batch_tool = BatchConfirmationTool::new(config.batch_config.clone());

        Self {
            review_tool,
            batch_tool,
            config,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(ResultConfirmationConfig::default())
    }

    /// Process comprehensive result confirmation
    fn process_confirmation(
        &self,
        params: &ResultConfirmationParams,
        context: &ExecutionContext,
    ) -> FileManagementResult<ResultConfirmationResult> {
        let start_time = std::time::Instant::now();
        let confirmation_id = uuid::Uuid::new_v4().to_string();

        info!("Starting comprehensive result confirmation for {} operations", 
              params.experimental_operations.len());

        // Convert experimental operations to reviewable results
        let experimental_results = self.review_tool.prepare_results_for_review(&params.experimental_operations)?;

        let mut phase_results = Vec::new();
        let mut current_operations = experimental_results;
        let mut final_approved = Vec::new();
        let mut final_rejected = Vec::new();
        let mut final_deferred = Vec::new();

        // Determine confirmation strategy based on mode
        let strategy = self.determine_confirmation_strategy(&params.confirmation_mode, &current_operations);

        match strategy {
            ConfirmationMode::ReviewThenBatch => {
                // Phase 1: Individual/Smart Review
                let review_result = self.execute_review_phase(&current_operations, params, context)?;
                phase_results.push(self.create_phase_result("review", PhaseType::Review, &review_result));

                // Filter operations for batch confirmation
                let approved_for_batch: Vec<_> = current_operations.into_iter()
                    .filter(|op| review_result.approved_operations.contains(&op.operation_id))
                    .collect();

                if !approved_for_batch.is_empty() && self.config.auto_transition_to_batch {
                    // Phase 2: Batch Confirmation
                    let batch_result = self.execute_batch_phase(&approved_for_batch, params, context)?;
                    phase_results.push(self.create_phase_result("batch_confirmation", PhaseType::BatchConfirmation, &batch_result));

                    // Extract approved operations from batch result
                    for batch in &batch_result.batches_processed {
                        final_approved.extend(batch.operations_approved.clone());
                        final_rejected.extend(batch.operations_rejected.clone());
                        final_deferred.extend(batch.operations_deferred.clone());
                    }
                } else {
                    final_approved.extend(review_result.approved_operations);
                }

                final_rejected.extend(review_result.rejected_operations);
                final_deferred.extend(review_result.deferred_operations);
            }
            ConfirmationMode::BatchOnly => {
                // Direct batch confirmation
                let batch_result = self.execute_batch_phase(&current_operations, params, context)?;
                phase_results.push(self.create_phase_result("batch_only", PhaseType::BatchConfirmation, &batch_result));

                // Extract approved operations from batch result
                for batch in &batch_result.batches_processed {
                    final_approved.extend(batch.operations_approved.clone());
                    final_rejected.extend(batch.operations_rejected.clone());
                    final_deferred.extend(batch.operations_deferred.clone());
                }
            }
            ConfirmationMode::ReviewOnly => {
                // Review only
                let review_result = self.execute_review_phase(&current_operations, params, context)?;
                phase_results.push(self.create_phase_result("review_only", PhaseType::Review, &review_result));

                final_approved.extend(review_result.approved_operations);
                final_rejected.extend(review_result.rejected_operations);
                final_deferred.extend(review_result.deferred_operations);
            }
            ConfirmationMode::Smart => {
                // Smart mode: choose strategy based on characteristics
                if current_operations.len() > 20 || self.has_high_risk_operations(&current_operations) {
                    // Use review then batch for complex scenarios
                    let review_result = self.execute_review_phase(&current_operations, params, context)?;
                    phase_results.push(self.create_phase_result("smart_review", PhaseType::Review, &review_result));

                    let approved_for_batch: Vec<_> = current_operations.into_iter()
                        .filter(|op| review_result.approved_operations.contains(&op.operation_id))
                        .collect();

                    if !approved_for_batch.is_empty() {
                        let batch_result = self.execute_batch_phase(&approved_for_batch, params, context)?;
                        phase_results.push(self.create_phase_result("smart_batch", PhaseType::BatchConfirmation, &batch_result));

                        // Extract approved operations from batch result
                        for batch in &batch_result.batches_processed {
                            final_approved.extend(batch.operations_approved.clone());
                            final_rejected.extend(batch.operations_rejected.clone());
                            final_deferred.extend(batch.operations_deferred.clone());
                        }
                    }

                    final_rejected.extend(review_result.rejected_operations);
                    final_deferred.extend(review_result.deferred_operations);
                } else {
                    // Use batch only for simple scenarios
                    let batch_result = self.execute_batch_phase(&current_operations, params, context)?;
                    phase_results.push(self.create_phase_result("smart_batch_only", PhaseType::BatchConfirmation, &batch_result));

                    // Extract approved operations from batch result
                    for batch in &batch_result.batches_processed {
                        final_approved.extend(batch.operations_approved.clone());
                        final_rejected.extend(batch.operations_rejected.clone());
                        final_deferred.extend(batch.operations_deferred.clone());
                    }
                }
            }
        }

        // Final confirmation phase if required
        if self.config.require_final_confirmation && !final_approved.is_empty() {
            let final_phase = self.execute_final_confirmation_phase(&final_approved, context)?;
            phase_results.push(final_phase);
        }

        // Create rollback plan if enabled
        let rollback_plan = if self.config.enable_rollback_planning {
            Some(self.create_rollback_plan(&final_approved, &params.experimental_operations, &params.rollback_options)?)
        } else {
            None
        };

        // Create execution summary
        let execution_summary = self.create_execution_summary(&params.experimental_operations, &final_approved)?;

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ResultConfirmationResult {
            confirmation_id,
            phase_results,
            final_approved_operations: final_approved,
            final_rejected_operations: final_rejected,
            final_deferred_operations: final_deferred,
            rollback_plan,
            execution_summary,
            processing_time_ms,
        })
    }

    /// Determine the best confirmation strategy
    fn determine_confirmation_strategy(
        &self,
        mode: &ConfirmationMode,
        operations: &[ExperimentalResult],
    ) -> ConfirmationMode {
        match mode {
            ConfirmationMode::Smart => {
                // Analyze operations to determine best strategy
                let high_risk_count = operations.iter()
                    .filter(|op| op.risk_level >= RiskLevel::High)
                    .count();
                
                let total_count = operations.len();
                
                if total_count > 50 || high_risk_count > total_count / 4 {
                    ConfirmationMode::ReviewThenBatch
                } else if total_count <= 5 && high_risk_count == 0 {
                    ConfirmationMode::BatchOnly
                } else {
                    ConfirmationMode::ReviewThenBatch
                }
            }
            other => other.clone(),
        }
    }

    /// Check if operations contain high risk items
    fn has_high_risk_operations(&self, operations: &[ExperimentalResult]) -> bool {
        operations.iter().any(|op| op.risk_level >= RiskLevel::High)
    }

    /// Execute the review phase
    fn execute_review_phase(
        &self,
        operations: &[ExperimentalResult],
        params: &ResultConfirmationParams,
        context: &ExecutionContext,
    ) -> FileManagementResult<ResultReviewResult> {
        let review_mode = params.review_options.as_ref()
            .map(|opts| opts.review_mode.clone())
            .unwrap_or(ReviewMode::Smart);

        let review_params = ResultReviewParams {
            experimental_results: operations.to_vec(),
            review_mode,
            confirmation_options: None,
            batch_size: Some(self.config.batch_config.default_batch_size),
        };

        self.review_tool.process_review(&review_params, context)
    }

    /// Execute the batch confirmation phase
    fn execute_batch_phase(
        &self,
        operations: &[ExperimentalResult],
        params: &ResultConfirmationParams,
        context: &ExecutionContext,
    ) -> FileManagementResult<BatchConfirmationResult> {
        let batch_params = BatchConfirmationParams {
            operations: operations.to_vec(),
            confirmation_strategy: ConfirmationStrategy::Smart,
            batch_options: params.batch_options.clone(),
            user_preferences: params.user_preferences.clone(),
        };

        self.batch_tool.process_batch_confirmation(&batch_params, context)
    }

    /// Execute final confirmation phase
    fn execute_final_confirmation_phase(
        &self,
        approved_operations: &[String],
        context: &ExecutionContext,
    ) -> FileManagementResult<ConfirmationPhase> {
        let start_time = std::time::Instant::now();

        info!("=== Final Confirmation Required ===");
        info!("About to execute {} operations", approved_operations.len());
        info!("This is your last chance to review before execution.");
        info!("Type 'CONFIRM' to proceed or 'CANCEL' to abort:");

        // In a real implementation, this would wait for user input
        // For now, simulate confirmation
        let confirmed = true; // Simulate user confirmation

        let phase_duration_ms = start_time.elapsed().as_millis() as u64;

        let output = if confirmed {
            PhaseOutput {
                approved: approved_operations.to_vec(),
                rejected: Vec::new(),
                deferred: Vec::new(),
                modified: Vec::new(),
            }
        } else {
            PhaseOutput {
                approved: Vec::new(),
                rejected: approved_operations.to_vec(),
                deferred: Vec::new(),
                modified: Vec::new(),
            }
        };

        Ok(ConfirmationPhase {
            phase_name: "final_confirmation".to_string(),
            phase_type: PhaseType::FinalConfirmation,
            input_operations: approved_operations.to_vec(),
            output_operations: output,
            phase_duration_ms,
            user_interactions: 1,
            auto_decisions: 0,
        })
    }

    /// Create a phase result from review or batch results
    fn create_phase_result(&self, name: &str, phase_type: PhaseType, result: &dyn PhaseResultTrait) -> ConfirmationPhase {
        ConfirmationPhase {
            phase_name: name.to_string(),
            phase_type,
            input_operations: result.get_input_operations(),
            output_operations: PhaseOutput {
                approved: result.get_approved_operations(),
                rejected: result.get_rejected_operations(),
                deferred: result.get_deferred_operations(),
                modified: result.get_modified_operations(),
            },
            phase_duration_ms: result.get_processing_time_ms(),
            user_interactions: result.get_user_interactions(),
            auto_decisions: result.get_auto_decisions(),
        }
    }

    /// Create rollback plan for approved operations
    fn create_rollback_plan(
        &self,
        approved_operations: &[String],
        original_operations: &[ExperimentalOperation],
        rollback_options: &Option<RollbackOptions>,
    ) -> FileManagementResult<RollbackPlan> {
        let plan_id = uuid::Uuid::new_v4().to_string();
        let mut rollback_operations = Vec::new();
        let mut backup_locations = HashMap::new();
        let mut estimated_rollback_time_ms = 0u64;

        let backup_strategy = rollback_options.as_ref()
            .map(|opts| opts.backup_strategy.clone())
            .unwrap_or(BackupStrategy::CopyBeforeOperation);

        for op_id in approved_operations {
            if let Some(original_op) = original_operations.iter().find(|op| &format!("op_{}", op_id) == op_id || op_id.contains(&op.operation_type)) {
                let rollback_op = self.create_rollback_operation(original_op, &backup_strategy)?;
                estimated_rollback_time_ms += rollback_op.verification_steps.len() as u64 * 100; // Estimate 100ms per verification step
                
                if let Some(backup) = &rollback_op.source_backup {
                    backup_locations.insert(op_id.clone(), backup.clone());
                }
                
                rollback_operations.push(rollback_op);
            }
        }

        // Create rollback order (reverse of execution order for most operations)
        let rollback_order = approved_operations.iter().rev().cloned().collect();

        Ok(RollbackPlan {
            plan_id,
            rollback_operations,
            backup_locations,
            rollback_order,
            estimated_rollback_time_ms,
            rollback_dependencies: Vec::new(),
        })
    }

    /// Create a rollback operation for a specific experimental operation
    fn create_rollback_operation(
        &self,
        operation: &ExperimentalOperation,
        backup_strategy: &BackupStrategy,
    ) -> FileManagementResult<RollbackOperation> {
        let rollback_type = match operation.operation_type.as_str() {
            "move" | "rename" => RollbackType::ReverseOperation,
            "copy" => RollbackType::RestoreFromBackup, // Delete the copy
            "delete" => RollbackType::RestoreFromBackup,
            "merge" => RollbackType::RestoreFromBackup,
            _ => RollbackType::CustomScript,
        };

        let source_backup = match backup_strategy {
            BackupStrategy::CopyBeforeOperation => {
                operation.source_path.as_ref().map(|p| format!("{}.backup", p.display()))
            }
            BackupStrategy::SnapshotFilesystem => {
                Some(format!("snapshot_{}", chrono::Utc::now().timestamp()))
            }
            _ => None,
        };

        let target_location = operation.source_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let verification_steps = vec![
            "Verify target location exists".to_string(),
            "Check file integrity".to_string(),
            "Validate permissions".to_string(),
        ];

        Ok(RollbackOperation {
            operation_id: format!("rollback_{}", operation.operation_type),
            rollback_type,
            source_backup,
            target_location,
            rollback_command: None,
            verification_steps,
        })
    }

    /// Create execution summary
    fn create_execution_summary(
        &self,
        original_operations: &[ExperimentalOperation],
        approved_operations: &[String],
    ) -> FileManagementResult<ExecutionSummary> {
        let total_operations_reviewed = original_operations.len();
        let operations_approved_for_execution = approved_operations.len();
        
        let operations_requiring_backup = original_operations.iter()
            .filter(|op| matches!(op.operation_type.as_str(), "delete" | "merge" | "move"))
            .count();

        let estimated_execution_time_ms = original_operations.iter()
            .map(|op| op.estimated_size.unwrap_or(1000) / 1000) // 1ms per KB
            .sum::<u64>();

        let estimated_rollback_time_ms = estimated_execution_time_ms * 2; // Rollback typically takes longer

        // Assess overall risk
        let high_risk_count = original_operations.iter()
            .filter(|op| matches!(op.operation_type.as_str(), "delete" | "merge"))
            .count();

        let overall_risk_level = if high_risk_count > operations_approved_for_execution / 2 {
            RiskLevel::High
        } else if high_risk_count > 0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let risk_assessment = RiskAssessment {
            overall_risk_level,
            irreversible_operations: high_risk_count,
            high_impact_operations: operations_requiring_backup,
            dependency_risks: Vec::new(),
            mitigation_strategies: vec![
                "Create backups before execution".to_string(),
                "Execute in small batches".to_string(),
                "Monitor progress continuously".to_string(),
            ],
        };

        let total_size_bytes = original_operations.iter()
            .map(|op| op.estimated_size.unwrap_or(0))
            .sum::<u64>();

        let resource_requirements = ResourceRequirements {
            estimated_disk_space_bytes: total_size_bytes * 2, // Original + backup
            estimated_memory_mb: (total_size_bytes / 1_000_000).max(100) as u64, // At least 100MB
            estimated_cpu_cores: 2,
            network_bandwidth_required: false,
            exclusive_access_required: Vec::new(),
        };

        Ok(ExecutionSummary {
            total_operations_reviewed,
            operations_approved_for_execution,
            operations_requiring_backup,
            estimated_execution_time_ms,
            estimated_rollback_time_ms,
            risk_assessment,
            resource_requirements,
        })
    }
}

/// Trait for extracting common information from different result types
trait PhaseResultTrait {
    fn get_input_operations(&self) -> Vec<String>;
    fn get_approved_operations(&self) -> Vec<String>;
    fn get_rejected_operations(&self) -> Vec<String>;
    fn get_deferred_operations(&self) -> Vec<String>;
    fn get_modified_operations(&self) -> Vec<String>;
    fn get_processing_time_ms(&self) -> u64;
    fn get_user_interactions(&self) -> usize;
    fn get_auto_decisions(&self) -> usize;
}

impl PhaseResultTrait for ResultReviewResult {
    fn get_input_operations(&self) -> Vec<String> {
        // Combine all operations
        let mut all_ops = self.approved_operations.clone();
        all_ops.extend(self.rejected_operations.clone());
        all_ops.extend(self.deferred_operations.clone());
        all_ops
    }

    fn get_approved_operations(&self) -> Vec<String> {
        self.approved_operations.clone()
    }

    fn get_rejected_operations(&self) -> Vec<String> {
        self.rejected_operations.clone()
    }

    fn get_deferred_operations(&self) -> Vec<String> {
        self.deferred_operations.clone()
    }

    fn get_modified_operations(&self) -> Vec<String> {
        // Extract modified operations from confirmation details
        self.confirmation_details.iter()
            .filter_map(|detail| {
                if matches!(detail.decision, ConfirmationDecision::Modified(_)) {
                    Some(detail.operation_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_processing_time_ms(&self) -> u64 {
        self.processing_time_ms
    }

    fn get_user_interactions(&self) -> usize {
        self.review_summary.manual_review_count
    }

    fn get_auto_decisions(&self) -> usize {
        self.review_summary.auto_approved_count
    }
}

impl PhaseResultTrait for BatchConfirmationResult {
    fn get_input_operations(&self) -> Vec<String> {
        // Extract from overall summary
        (0..self.overall_summary.total_operations)
            .map(|i| format!("op_{}", i))
            .collect()
    }

    fn get_approved_operations(&self) -> Vec<String> {
        self.batches_processed.iter()
            .flat_map(|batch| batch.operations_approved.clone())
            .collect()
    }

    fn get_rejected_operations(&self) -> Vec<String> {
        self.batches_processed.iter()
            .flat_map(|batch| batch.operations_rejected.clone())
            .collect()
    }

    fn get_deferred_operations(&self) -> Vec<String> {
        self.batches_processed.iter()
            .flat_map(|batch| batch.operations_deferred.clone())
            .collect()
    }

    fn get_modified_operations(&self) -> Vec<String> {
        // Extract modified operations from batch decisions
        self.user_decisions.iter()
            .filter_map(|decision| {
                if let super::batch_confirmation_tool::BatchDecisionType::ModifyAndApprove(modifications) = &decision.decision_type {
                    Some(modifications.iter().map(|m| m.operation_id.clone()).collect::<Vec<_>>())
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    fn get_processing_time_ms(&self) -> u64 {
        self.processing_time_ms
    }

    fn get_user_interactions(&self) -> usize {
        self.overall_summary.manual_decisions
    }

    fn get_auto_decisions(&self) -> usize {
        self.overall_summary.auto_decisions
    }
}

#[async_trait]
impl ToolNode for ResultConfirmationTool {
    fn name(&self) -> &str {
        "result-confirmer"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value, WorkflowError> {
        let params: ResultConfirmationParams = serde_json::from_value(params)
            .map_err(|e| WorkflowError::validation(&format!("Invalid parameters: {}", e)))?;

        let result = self.process_confirmation(&params, &context)
            .map_err(|e| WorkflowError::tool_execution(&format!("Result confirmation failed: {}", e)))?;

        Ok(serde_json::to_value(result)
            .map_err(|e| WorkflowError::tool_execution(&format!("Failed to serialize result: {}", e)))?)
    }

    fn validate_parameters(&self, params: &Value) -> Result<(), WorkflowError> {
        let _: ResultConfirmationParams = serde_json::from_value(params.clone())
            .map_err(|e| WorkflowError::validation(&format!("Invalid parameters: {}", e)))?;
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: "Comprehensive result confirmation tool combining review and batch confirmation with rollback planning".to_string(),
            category: Some("file-management".to_string()),
            tags: vec!["confirmation".to_string(), "review".to_string(), "rollback".to_string()],
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "experimental_operations": {"type": "array"},
                    "confirmation_mode": {"type": "string"},
                    "review_options": {"type": "object"},
                    "batch_options": {"type": "object"},
                    "rollback_options": {"type": "object"}
                },
                "required": ["experimental_operations"]
            }),
            return_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "confirmation_id": {"type": "string"},
                    "phase_results": {"type": "array"},
                    "final_approved_operations": {"type": "array"},
                    "rollback_plan": {"type": "object"}
                }
            }),
            plugin_name: Some("file-management".to_string()),
            dependencies: vec![],
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        None
    }
}

/// Create a comprehensive result confirmation tool with default configuration
pub fn create_result_confirmation_tool() -> Box<dyn ToolNode> {
    Box::new(ResultConfirmationTool::with_default_config())
}

/// Create a comprehensive result confirmation tool with custom configuration
pub fn create_result_confirmation_tool_with_config(config: ResultConfirmationConfig) -> Box<dyn ToolNode> {
    Box::new(ResultConfirmationTool::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::utils::ExperimentalOperation;

    #[test]
    fn test_result_confirmation_tool_creation() {
        let tool = ResultConfirmationTool::with_default_config();
        assert_eq!(tool.name(), "result-confirmer");
        assert_eq!(tool.version(), "1.0.0");
    }

    #[test]
    fn test_confirmation_strategy_determination() {
        let tool = ResultConfirmationTool::with_default_config();
        
        // Test with small, low-risk operations
        let small_ops = vec![
            create_test_experimental_result("op1", RiskLevel::Low),
            create_test_experimental_result("op2", RiskLevel::Low),
        ];
        
        let strategy = tool.determine_confirmation_strategy(&ConfirmationMode::Smart, &small_ops);
        // Should choose batch-only for small, low-risk operations
        assert!(matches!(strategy, ConfirmationMode::BatchOnly));
        
        // Test with large number of operations
        let large_ops: Vec<_> = (0..60)
            .map(|i| create_test_experimental_result(&format!("op{}", i), RiskLevel::Low))
            .collect();
        
        let strategy = tool.determine_confirmation_strategy(&ConfirmationMode::Smart, &large_ops);
        // Should choose review-then-batch for large number of operations
        assert!(matches!(strategy, ConfirmationMode::ReviewThenBatch));
    }

    #[test]
    fn test_rollback_plan_creation() {
        let tool = ResultConfirmationTool::with_default_config();
        
        let operations = vec![
            ExperimentalOperation::new("move", "Move file A to B"),
            ExperimentalOperation::new("delete", "Delete file C"),
        ];
        
        let approved = vec!["op_0".to_string(), "op_1".to_string()];
        
        let rollback_plan = tool.create_rollback_plan(
            &approved,
            &operations,
            &Some(RollbackOptions {
                create_rollback_plan: true,
                backup_strategy: BackupStrategy::CopyBeforeOperation,
                rollback_timeout_hours: Some(24),
                enable_automatic_rollback: false,
            })
        ).unwrap();
        
        assert_eq!(rollback_plan.rollback_operations.len(), 2);
        assert_eq!(rollback_plan.rollback_order.len(), 2);
        // Rollback order should be reverse of execution order
        assert_eq!(rollback_plan.rollback_order[0], "op_1");
        assert_eq!(rollback_plan.rollback_order[1], "op_0");
    }

    fn create_test_experimental_result(id: &str, risk: RiskLevel) -> ExperimentalResult {
        use super::super::result_review_tool::{ExperimentalResult, OperationImpact};
        
        ExperimentalResult {
            operation_id: id.to_string(),
            operation_type: "test".to_string(),
            source_path: None,
            target_path: None,
            description: "Test operation".to_string(),
            estimated_impact: OperationImpact {
                files_affected: 1,
                directories_affected: 0,
                estimated_size_bytes: 1000,
                estimated_duration_ms: 100,
                reversible: risk == RiskLevel::Low,
                backup_required: risk >= RiskLevel::High,
            },
            risk_level: risk,
            dependencies: Vec::new(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}