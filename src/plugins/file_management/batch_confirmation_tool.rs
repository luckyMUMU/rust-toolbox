//! Batch Confirmation Tool
//!
//! This module provides tools for batch confirmation of multiple operations
//! with support for different confirmation strategies and user interaction modes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::error::FileManagementResult;
use super::result_review_tool::{ExperimentalResult, OperationImpact, ReviewMode, RiskLevel};
use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::WorkflowError;
use crate::tools::ToolNode;

use async_trait::async_trait;

/// Tool for batch confirmation of operations
#[derive(Debug, Clone)]
pub struct BatchConfirmationTool {
    config: BatchConfirmationConfig,
}

/// Configuration for batch confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfirmationConfig {
    pub default_batch_size: usize,
    pub max_batch_size: usize,
    pub confirmation_timeout_seconds: u64,
    pub enable_smart_batching: bool,
    pub group_by_risk_level: bool,
    pub group_by_operation_type: bool,
    pub auto_confirm_threshold: usize,
}

impl Default for BatchConfirmationConfig {
    fn default() -> Self {
        Self {
            default_batch_size: 10,
            max_batch_size: 50,
            confirmation_timeout_seconds: 300, // 5 minutes
            enable_smart_batching: true,
            group_by_risk_level: true,
            group_by_operation_type: false,
            auto_confirm_threshold: 3,
        }
    }
}

/// Parameters for batch confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfirmationParams {
    pub operations: Vec<ExperimentalResult>,
    pub confirmation_strategy: ConfirmationStrategy,
    pub batch_options: Option<BatchOptions>,
    pub user_preferences: Option<UserPreferences>,
}

/// Strategy for confirming batches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationStrategy {
    /// Confirm all operations at once
    AllAtOnce,
    /// Group by risk level and confirm each group
    ByRiskLevel,
    /// Group by operation type and confirm each group
    ByOperationType,
    /// Use smart batching based on multiple factors
    Smart,
    /// Custom batch size
    CustomBatches(usize),
}

/// Options for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    pub max_operations_per_batch: Option<usize>,
    pub timeout_per_batch_seconds: Option<u64>,
    pub allow_partial_confirmation: bool,
    pub require_unanimous_approval: bool,
    pub enable_preview_mode: bool,
}

/// User preferences for confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub auto_approve_low_risk: bool,
    pub auto_reject_high_risk: bool,
    pub prefer_detailed_summaries: bool,
    pub enable_notifications: bool,
    pub confirmation_sound: bool,
}

/// A batch of operations for confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationBatch {
    pub batch_id: String,
    pub operations: Vec<ExperimentalResult>,
    pub batch_summary: BatchSummary,
    pub recommended_action: RecommendedAction,
    pub created_at: DateTime<Utc>,
}

/// Summary of a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub total_operations: usize,
    pub risk_distribution: RiskDistribution,
    pub operation_types: HashMap<String, usize>,
    pub total_impact: OperationImpact,
    pub estimated_duration: Duration,
    pub reversibility_summary: ReversibilitySummary,
}

/// Distribution of risk levels in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDistribution {
    pub low: usize,
    pub medium: usize,
    pub high: usize,
    pub critical: usize,
}

/// Summary of operation reversibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversibilitySummary {
    pub fully_reversible: usize,
    pub partially_reversible: usize,
    pub irreversible: usize,
    pub backup_required: usize,
}

/// Recommended action for a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    ApproveAll,
    RejectAll,
    ReviewIndividually,
    ApproveWithCaution,
    RequireBackup,
}

/// Result of batch confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfirmationResult {
    pub confirmation_id: String,
    pub batches_processed: Vec<ProcessedBatch>,
    pub overall_summary: OverallSummary,
    pub user_decisions: Vec<BatchDecision>,
    pub processing_time_ms: u64,
}

/// A processed batch with its decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedBatch {
    pub batch_id: String,
    pub decision: BatchDecision,
    pub operations_approved: Vec<String>,
    pub operations_rejected: Vec<String>,
    pub operations_deferred: Vec<String>,
    pub processing_time_ms: u64,
}

/// Decision made for a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDecision {
    pub batch_id: String,
    pub decision_type: BatchDecisionType,
    pub confirmation_method: ConfirmationMethod,
    pub user_input: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub confidence_level: f64,
}

/// Type of decision made for a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchDecisionType {
    ApproveAll,
    RejectAll,
    ApproveSelected(Vec<String>),
    Defer,
    ModifyAndApprove(Vec<OperationModification>),
}

/// Method used for confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationMethod {
    Automatic,
    UserInteraction,
    Timeout,
    PolicyBased,
}

/// Modification to an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationModification {
    pub operation_id: String,
    pub modification_type: ModificationType,
    pub new_parameters: Value,
    pub reason: String,
}

/// Type of modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModificationType {
    ChangeTarget,
    ChangeSource,
    AddBackup,
    ChangeStrategy,
    AddSafeguards,
}

/// Overall summary of batch confirmation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallSummary {
    pub total_operations: usize,
    pub total_batches: usize,
    pub approved_operations: usize,
    pub rejected_operations: usize,
    pub deferred_operations: usize,
    pub modified_operations: usize,
    pub auto_decisions: usize,
    pub manual_decisions: usize,
    pub total_estimated_impact: OperationImpact,
}

impl BatchConfirmationTool {
    pub fn new(config: BatchConfirmationConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(BatchConfirmationConfig::default())
    }

    /// Create batches from operations based on strategy
    pub fn create_batches(
        &self,
        operations: &[ExperimentalResult],
        strategy: &ConfirmationStrategy,
        _options: &Option<BatchOptions>,
    ) -> FileManagementResult<Vec<OperationBatch>> {
        let mut batches = Vec::new();

        match strategy {
            ConfirmationStrategy::AllAtOnce => {
                if operations.len() <= self.config.max_batch_size {
                    batches.push(self.create_single_batch(operations, "all_operations")?);
                } else {
                    // Split into manageable chunks
                    for (i, chunk) in operations.chunks(self.config.max_batch_size).enumerate() {
                        batches.push(self.create_single_batch(chunk, &format!("chunk_{}", i))?);
                    }
                }
            }
            ConfirmationStrategy::ByRiskLevel => {
                batches.extend(self.create_risk_based_batches(operations)?);
            }
            ConfirmationStrategy::ByOperationType => {
                batches.extend(self.create_type_based_batches(operations)?);
            }
            ConfirmationStrategy::Smart => {
                batches.extend(self.create_smart_batches(operations)?);
            }
            ConfirmationStrategy::CustomBatches(size) => {
                let batch_size = (*size).min(self.config.max_batch_size);
                for (i, chunk) in operations.chunks(batch_size).enumerate() {
                    batches.push(self.create_single_batch(chunk, &format!("custom_{}", i))?);
                }
            }
        }

        Ok(batches)
    }

    /// Create a single batch from operations
    fn create_single_batch(
        &self,
        operations: &[ExperimentalResult],
        batch_id: &str,
    ) -> FileManagementResult<OperationBatch> {
        let batch_summary = self.create_batch_summary(operations);
        let recommended_action = self.determine_recommended_action(&batch_summary);

        Ok(OperationBatch {
            batch_id: batch_id.to_string(),
            operations: operations.to_vec(),
            batch_summary,
            recommended_action,
            created_at: Utc::now(),
        })
    }

    /// Create batches grouped by risk level
    fn create_risk_based_batches(
        &self,
        operations: &[ExperimentalResult],
    ) -> FileManagementResult<Vec<OperationBatch>> {
        let mut batches = Vec::new();
        let mut risk_groups: HashMap<RiskLevel, Vec<ExperimentalResult>> = HashMap::new();

        // Group operations by risk level
        for operation in operations {
            risk_groups
                .entry(operation.risk_level.clone())
                .or_insert_with(Vec::new)
                .push(operation.clone());
        }

        // Create batches for each risk level
        for (risk_level, ops) in risk_groups {
            if ops.is_empty() {
                continue;
            }

            let batch_id = format!("risk_{:?}", risk_level).to_lowercase();

            // Split large risk groups into smaller batches
            if ops.len() > self.config.default_batch_size {
                for (i, chunk) in ops.chunks(self.config.default_batch_size).enumerate() {
                    let sub_batch_id = format!("{}_{}", batch_id, i);
                    batches.push(self.create_single_batch(chunk, &sub_batch_id)?);
                }
            } else {
                batches.push(self.create_single_batch(&ops, &batch_id)?);
            }
        }

        Ok(batches)
    }

    /// Create batches grouped by operation type
    fn create_type_based_batches(
        &self,
        operations: &[ExperimentalResult],
    ) -> FileManagementResult<Vec<OperationBatch>> {
        let mut batches = Vec::new();
        let mut type_groups: HashMap<String, Vec<ExperimentalResult>> = HashMap::new();

        // Group operations by type
        for operation in operations {
            type_groups
                .entry(operation.operation_type.clone())
                .or_insert_with(Vec::new)
                .push(operation.clone());
        }

        // Create batches for each operation type
        for (op_type, ops) in type_groups {
            if ops.is_empty() {
                continue;
            }

            let batch_id = format!("type_{}", op_type);

            // Split large type groups into smaller batches
            if ops.len() > self.config.default_batch_size {
                for (i, chunk) in ops.chunks(self.config.default_batch_size).enumerate() {
                    let sub_batch_id = format!("{}_{}", batch_id, i);
                    batches.push(self.create_single_batch(chunk, &sub_batch_id)?);
                }
            } else {
                batches.push(self.create_single_batch(&ops, &batch_id)?);
            }
        }

        Ok(batches)
    }

    /// Create smart batches using multiple factors
    fn create_smart_batches(
        &self,
        operations: &[ExperimentalResult],
    ) -> FileManagementResult<Vec<OperationBatch>> {
        let mut batches = Vec::new();

        // First, separate by risk level
        let (low_risk, higher_risk): (Vec<_>, Vec<_>) = operations
            .iter()
            .partition(|op| op.risk_level == RiskLevel::Low);

        // Low risk operations can be batched together in larger groups
        if !low_risk.is_empty() {
            let batch_size = (self.config.default_batch_size * 2).min(self.config.max_batch_size);
            let low_risk_owned: Vec<ExperimentalResult> = low_risk.into_iter().cloned().collect();
            for (i, chunk) in low_risk_owned.chunks(batch_size).enumerate() {
                batches.push(self.create_single_batch(chunk, &format!("low_risk_{}", i))?);
            }
        }

        // Higher risk operations need smaller, more focused batches
        if !higher_risk.is_empty() {
            // Group by operation type within higher risk
            let mut type_groups: HashMap<String, Vec<ExperimentalResult>> = HashMap::new();
            for operation in higher_risk {
                type_groups
                    .entry(operation.operation_type.clone())
                    .or_insert_with(Vec::new)
                    .push(operation.clone());
            }

            for (op_type, ops) in type_groups {
                let batch_size = (self.config.default_batch_size / 2).max(1);
                for (i, chunk) in ops.chunks(batch_size).enumerate() {
                    let batch_id = format!("high_risk_{}_{}", op_type, i);
                    batches.push(self.create_single_batch(chunk, &batch_id)?);
                }
            }
        }

        Ok(batches)
    }

    /// Create summary for a batch of operations
    fn create_batch_summary(&self, operations: &[ExperimentalResult]) -> BatchSummary {
        let mut risk_distribution = RiskDistribution {
            low: 0,
            medium: 0,
            high: 0,
            critical: 0,
        };

        let mut operation_types = HashMap::new();
        let mut total_impact = OperationImpact {
            files_affected: 0,
            directories_affected: 0,
            estimated_size_bytes: 0,
            estimated_duration_ms: 0,
            reversible: true,
            backup_required: false,
        };

        let mut reversibility_summary = ReversibilitySummary {
            fully_reversible: 0,
            partially_reversible: 0,
            irreversible: 0,
            backup_required: 0,
        };

        for operation in operations {
            // Count risk levels
            match operation.risk_level {
                RiskLevel::Low => risk_distribution.low += 1,
                RiskLevel::Medium => risk_distribution.medium += 1,
                RiskLevel::High => risk_distribution.high += 1,
                RiskLevel::Critical => risk_distribution.critical += 1,
            }

            // Count operation types
            *operation_types
                .entry(operation.operation_type.clone())
                .or_insert(0) += 1;

            // Aggregate impact
            total_impact.files_affected += operation.estimated_impact.files_affected;
            total_impact.directories_affected += operation.estimated_impact.directories_affected;
            total_impact.estimated_size_bytes += operation.estimated_impact.estimated_size_bytes;
            total_impact.estimated_duration_ms += operation.estimated_impact.estimated_duration_ms;

            if !operation.estimated_impact.reversible {
                total_impact.reversible = false;
            }
            if operation.estimated_impact.backup_required {
                total_impact.backup_required = true;
            }

            // Count reversibility
            if operation.estimated_impact.reversible && !operation.estimated_impact.backup_required
            {
                reversibility_summary.fully_reversible += 1;
            } else if operation.estimated_impact.reversible {
                reversibility_summary.partially_reversible += 1;
            } else {
                reversibility_summary.irreversible += 1;
            }

            if operation.estimated_impact.backup_required {
                reversibility_summary.backup_required += 1;
            }
        }

        BatchSummary {
            total_operations: operations.len(),
            risk_distribution,
            operation_types,
            estimated_duration: Duration::from_millis(total_impact.estimated_duration_ms),
            total_impact,
            reversibility_summary,
        }
    }

    /// Determine recommended action for a batch
    fn determine_recommended_action(&self, summary: &BatchSummary) -> RecommendedAction {
        // If all operations are low risk and reversible
        if summary.risk_distribution.high == 0
            && summary.risk_distribution.critical == 0
            && summary.reversibility_summary.irreversible == 0
        {
            return RecommendedAction::ApproveAll;
        }

        // If there are critical operations
        if summary.risk_distribution.critical > 0 {
            return RecommendedAction::ReviewIndividually;
        }

        // If many operations require backup
        if summary.reversibility_summary.backup_required > summary.total_operations / 2 {
            return RecommendedAction::RequireBackup;
        }

        // If mostly high risk operations
        if summary.risk_distribution.high > summary.total_operations / 2 {
            return RecommendedAction::ApproveWithCaution;
        }

        // Default to individual review for mixed batches
        RecommendedAction::ReviewIndividually
    }

    /// Process batch confirmation
    pub fn process_batch_confirmation(
        &self,
        params: &BatchConfirmationParams,
        context: &ExecutionContext,
    ) -> FileManagementResult<BatchConfirmationResult> {
        let start_time = std::time::Instant::now();
        let confirmation_id = uuid::Uuid::new_v4().to_string();

        // Create batches based on strategy
        let batches = self.create_batches(
            &params.operations,
            &params.confirmation_strategy,
            &params.batch_options,
        )?;

        let mut processed_batches = Vec::new();
        let mut user_decisions = Vec::new();
        let mut total_approved = 0;
        let mut total_rejected = 0;
        let mut total_deferred = 0;
        let mut total_modified = 0;
        let mut auto_decisions = 0;
        let mut manual_decisions = 0;

        // Process each batch
        for batch in batches {
            let batch_result =
                self.process_single_batch(&batch, &params.user_preferences, context)?;

            total_approved += batch_result.operations_approved.len();
            total_rejected += batch_result.operations_rejected.len();
            total_deferred += batch_result.operations_deferred.len();

            match &batch_result.decision.confirmation_method {
                ConfirmationMethod::Automatic | ConfirmationMethod::PolicyBased => {
                    auto_decisions += 1
                }
                ConfirmationMethod::UserInteraction => manual_decisions += 1,
                ConfirmationMethod::Timeout => auto_decisions += 1,
            }

            if let BatchDecisionType::ModifyAndApprove(modifications) =
                &batch_result.decision.decision_type
            {
                total_modified += modifications.len();
            }

            user_decisions.push(batch_result.decision.clone());
            processed_batches.push(batch_result);
        }

        let total_estimated_impact = self.calculate_total_impact(&params.operations);

        let overall_summary = OverallSummary {
            total_operations: params.operations.len(),
            total_batches: processed_batches.len(),
            approved_operations: total_approved,
            rejected_operations: total_rejected,
            deferred_operations: total_deferred,
            modified_operations: total_modified,
            auto_decisions,
            manual_decisions,
            total_estimated_impact,
        };

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(BatchConfirmationResult {
            confirmation_id,
            batches_processed: processed_batches,
            overall_summary,
            user_decisions,
            processing_time_ms,
        })
    }

    /// Process a single batch
    fn process_single_batch(
        &self,
        batch: &OperationBatch,
        user_preferences: &Option<UserPreferences>,
        context: &ExecutionContext,
    ) -> FileManagementResult<ProcessedBatch> {
        let start_time = std::time::Instant::now();

        // Check if we can auto-decide based on preferences and batch characteristics
        if let Some(auto_decision) = self.check_auto_decision(batch, user_preferences) {
            let (approved, rejected, deferred) =
                self.apply_batch_decision(&auto_decision, &batch.operations);

            return Ok(ProcessedBatch {
                batch_id: batch.batch_id.clone(),
                decision: auto_decision,
                operations_approved: approved,
                operations_rejected: rejected,
                operations_deferred: deferred,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        // Present batch for user decision
        let decision = self.present_batch_for_confirmation(batch, context)?;
        let (approved, rejected, deferred) =
            self.apply_batch_decision(&decision, &batch.operations);

        Ok(ProcessedBatch {
            batch_id: batch.batch_id.clone(),
            decision,
            operations_approved: approved,
            operations_rejected: rejected,
            operations_deferred: deferred,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Check if batch can be auto-decided
    fn check_auto_decision(
        &self,
        batch: &OperationBatch,
        user_preferences: &Option<UserPreferences>,
    ) -> Option<BatchDecision> {
        if let Some(prefs) = user_preferences {
            // Auto-approve low risk batches if preference is set
            if prefs.auto_approve_low_risk
                && batch.batch_summary.risk_distribution.medium == 0
                && batch.batch_summary.risk_distribution.high == 0
                && batch.batch_summary.risk_distribution.critical == 0
            {
                return Some(BatchDecision {
                    batch_id: batch.batch_id.clone(),
                    decision_type: BatchDecisionType::ApproveAll,
                    confirmation_method: ConfirmationMethod::Automatic,
                    user_input: Some("Auto-approved: All low risk operations".to_string()),
                    timestamp: Utc::now(),
                    confidence_level: 0.9,
                });
            }

            // Auto-reject high risk batches if preference is set
            if prefs.auto_reject_high_risk
                && (batch.batch_summary.risk_distribution.high > 0
                    || batch.batch_summary.risk_distribution.critical > 0)
            {
                return Some(BatchDecision {
                    batch_id: batch.batch_id.clone(),
                    decision_type: BatchDecisionType::RejectAll,
                    confirmation_method: ConfirmationMethod::Automatic,
                    user_input: Some("Auto-rejected: Contains high risk operations".to_string()),
                    timestamp: Utc::now(),
                    confidence_level: 0.8,
                });
            }
        }

        // Check if batch is small enough for auto-approval
        if batch.batch_summary.total_operations <= self.config.auto_confirm_threshold
            && batch.batch_summary.risk_distribution.critical == 0
        {
            return Some(BatchDecision {
                batch_id: batch.batch_id.clone(),
                decision_type: BatchDecisionType::ApproveAll,
                confirmation_method: ConfirmationMethod::PolicyBased,
                user_input: Some("Auto-approved: Small batch with acceptable risk".to_string()),
                timestamp: Utc::now(),
                confidence_level: 0.7,
            });
        }

        None
    }

    /// Present batch for user confirmation
    fn present_batch_for_confirmation(
        &self,
        batch: &OperationBatch,
        _context: &ExecutionContext,
    ) -> FileManagementResult<BatchDecision> {
        info!("=== Batch Confirmation Required ===");
        info!("Batch ID: {}", batch.batch_id);
        info!("Operations: {}", batch.batch_summary.total_operations);
        info!(
            "Risk Distribution: Low: {}, Medium: {}, High: {}, Critical: {}",
            batch.batch_summary.risk_distribution.low,
            batch.batch_summary.risk_distribution.medium,
            batch.batch_summary.risk_distribution.high,
            batch.batch_summary.risk_distribution.critical
        );
        info!("Recommended Action: {:?}", batch.recommended_action);
        info!(
            "Total Impact: {} files, {} directories, {} bytes",
            batch.batch_summary.total_impact.files_affected,
            batch.batch_summary.total_impact.directories_affected,
            batch.batch_summary.total_impact.estimated_size_bytes
        );

        // In a real implementation, this would present a UI for batch confirmation
        // For now, simulate decision based on recommended action
        let decision_type = match batch.recommended_action {
            RecommendedAction::ApproveAll => BatchDecisionType::ApproveAll,
            RecommendedAction::RejectAll => BatchDecisionType::RejectAll,
            RecommendedAction::ReviewIndividually => BatchDecisionType::Defer,
            RecommendedAction::ApproveWithCaution => BatchDecisionType::ApproveAll,
            RecommendedAction::RequireBackup => BatchDecisionType::ApproveAll,
        };

        Ok(BatchDecision {
            batch_id: batch.batch_id.clone(),
            decision_type,
            confirmation_method: ConfirmationMethod::UserInteraction,
            user_input: Some("Simulated user decision".to_string()),
            timestamp: Utc::now(),
            confidence_level: 0.8,
        })
    }

    /// Apply batch decision to operations
    fn apply_batch_decision(
        &self,
        decision: &BatchDecision,
        operations: &[ExperimentalResult],
    ) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut approved = Vec::new();
        let mut rejected = Vec::new();
        let mut deferred = Vec::new();

        match &decision.decision_type {
            BatchDecisionType::ApproveAll => {
                approved.extend(operations.iter().map(|op| op.operation_id.clone()));
            }
            BatchDecisionType::RejectAll => {
                rejected.extend(operations.iter().map(|op| op.operation_id.clone()));
            }
            BatchDecisionType::ApproveSelected(selected_ids) => {
                for operation in operations {
                    if selected_ids.contains(&operation.operation_id) {
                        approved.push(operation.operation_id.clone());
                    } else {
                        deferred.push(operation.operation_id.clone());
                    }
                }
            }
            BatchDecisionType::Defer => {
                deferred.extend(operations.iter().map(|op| op.operation_id.clone()));
            }
            BatchDecisionType::ModifyAndApprove(_modifications) => {
                // For now, approve all modified operations
                approved.extend(operations.iter().map(|op| op.operation_id.clone()));
            }
        }

        (approved, rejected, deferred)
    }

    /// Calculate total impact across all operations
    fn calculate_total_impact(&self, operations: &[ExperimentalResult]) -> OperationImpact {
        let mut total = OperationImpact {
            files_affected: 0,
            directories_affected: 0,
            estimated_size_bytes: 0,
            estimated_duration_ms: 0,
            reversible: true,
            backup_required: false,
        };

        for operation in operations {
            total.files_affected += operation.estimated_impact.files_affected;
            total.directories_affected += operation.estimated_impact.directories_affected;
            total.estimated_size_bytes += operation.estimated_impact.estimated_size_bytes;
            total.estimated_duration_ms += operation.estimated_impact.estimated_duration_ms;

            if !operation.estimated_impact.reversible {
                total.reversible = false;
            }
            if operation.estimated_impact.backup_required {
                total.backup_required = true;
            }
        }

        total
    }
}

#[async_trait]
impl ToolNode for BatchConfirmationTool {
    fn name(&self) -> &str {
        "batch-confirmer"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn execute(
        &self,
        params: Value,
        context: ExecutionContext,
    ) -> Result<Value, WorkflowError> {
        let params: BatchConfirmationParams = serde_json::from_value(params)
            .map_err(|e| WorkflowError::validation(&format!("Invalid parameters: {}", e)))?;

        let result = self
            .process_batch_confirmation(&params, &context)
            .map_err(|e| {
                WorkflowError::tool_execution(&format!("Batch confirmation failed: {}", e))
            })?;

        Ok(serde_json::to_value(result).map_err(|e| {
            WorkflowError::tool_execution(&format!("Failed to serialize result: {}", e))
        })?)
    }

    fn validate_parameters(&self, params: &Value) -> Result<(), WorkflowError> {
        let _: BatchConfirmationParams = serde_json::from_value(params.clone())
            .map_err(|e| WorkflowError::validation(&format!("Invalid parameters: {}", e)))?;
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: "Batch confirmation tool for multiple operations with smart batching and user preferences".to_string(),
            category: Some("file-management".to_string()),
            tags: vec!["batch".to_string(), "confirmation".to_string(), "operations".to_string()],
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "operations": {"type": "array"},
                    "confirmation_strategy": {"type": "string"},
                    "batch_options": {"type": "object"},
                    "user_preferences": {"type": "object"}
                },
                "required": ["operations", "confirmation_strategy"]
            }),
            return_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "confirmation_id": {"type": "string"},
                    "batches_processed": {"type": "array"},
                    "overall_summary": {"type": "object"}
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

/// Create a batch confirmation tool with default configuration
pub fn create_batch_confirmation_tool() -> Box<dyn ToolNode> {
    Box::new(BatchConfirmationTool::with_default_config())
}

/// Create a batch confirmation tool with custom configuration
pub fn create_batch_confirmation_tool_with_config(
    config: BatchConfirmationConfig,
) -> Box<dyn ToolNode> {
    Box::new(BatchConfirmationTool::new(config))
}

#[cfg(test)]
mod tests {
    use super::super::result_review_tool::ExperimentalResult;
    use super::*;

    #[test]
    fn test_batch_confirmation_tool_creation() {
        let tool = BatchConfirmationTool::with_default_config();
        assert_eq!(tool.name(), "batch-confirmer");
        assert_eq!(tool.version(), "1.0.0");
    }

    #[test]
    fn test_create_risk_based_batches() {
        let tool = BatchConfirmationTool::with_default_config();

        let operations = vec![
            create_test_operation("op1", "copy", RiskLevel::Low),
            create_test_operation("op2", "delete", RiskLevel::High),
            create_test_operation("op3", "move", RiskLevel::Low),
        ];

        let batches = tool
            .create_batches(&operations, &ConfirmationStrategy::ByRiskLevel, &None)
            .unwrap();

        assert_eq!(batches.len(), 2); // One for low risk, one for high risk
    }

    #[test]
    fn test_smart_batching() {
        let tool = BatchConfirmationTool::with_default_config();

        let operations = vec![
            create_test_operation("op1", "copy", RiskLevel::Low),
            create_test_operation("op2", "copy", RiskLevel::Low),
            create_test_operation("op3", "delete", RiskLevel::High),
            create_test_operation("op4", "merge", RiskLevel::Critical),
        ];

        let batches = tool
            .create_batches(&operations, &ConfirmationStrategy::Smart, &None)
            .unwrap();

        // Should create separate batches for low risk and high risk operations
        assert!(batches.len() >= 2);
    }

    fn create_test_operation(id: &str, op_type: &str, risk: RiskLevel) -> ExperimentalResult {
        use super::super::result_review_tool::{ExperimentalResult, OperationImpact};

        ExperimentalResult {
            operation_id: id.to_string(),
            operation_type: op_type.to_string(),
            source_path: None,
            target_path: None,
            description: format!("Test {} operation", op_type),
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
            timestamp: Utc::now(),
        }
    }
}
