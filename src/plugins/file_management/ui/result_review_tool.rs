//! Result Review and Confirmation Tools
//!
//! This module provides tools for reviewing experimental results before execution
//! and batch confirmation for multiple operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

use crate::core::ExecutionContext;
use crate::plugins::file_management::core::error::FileManagementResult;
use crate::plugins::file_management::utils::utils::ExperimentalOperation;
use crate::error::WorkflowError;
use crate::tools::types::Tool;

/// Tool for reviewing experimental results before execution
#[derive(Debug, Clone)]
pub struct ResultReviewTool {
    config: ResultReviewConfig,
}

/// Configuration for result review tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultReviewConfig {
    pub auto_approve_safe_operations: bool,
    pub require_confirmation_threshold: usize,
    pub show_detailed_impact: bool,
    pub enable_batch_review: bool,
}

impl Default for ResultReviewConfig {
    fn default() -> Self {
        Self {
            auto_approve_safe_operations: false,
            require_confirmation_threshold: 5,
            show_detailed_impact: true,
            enable_batch_review: true,
        }
    }
}

/// Parameters for result review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultReviewParams {
    pub experimental_results: Vec<ExperimentalResult>,
    pub review_mode: ReviewMode,
    pub confirmation_options: Option<ConfirmationOptions>,
    pub batch_size: Option<usize>,
}

/// Mode for reviewing results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewMode {
    /// Review all operations individually
    Individual,
    /// Review operations in batches
    Batch,
    /// Review only operations above risk threshold
    RiskBased,
    /// Auto-approve safe operations, review risky ones
    Smart,
}

/// Options for confirmation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationOptions {
    pub timeout_seconds: Option<u64>,
    pub default_action: DefaultAction,
    pub allow_partial_approval: bool,
    pub require_explicit_confirmation: bool,
}

/// Default action when timeout occurs or no explicit choice is made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefaultAction {
    Approve,
    Reject,
    Defer,
}

/// Represents an experimental result that can be reviewed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalResult {
    pub operation_id: String,
    pub operation_type: String,
    pub source_path: Option<PathBuf>,
    pub target_path: Option<PathBuf>,
    pub description: String,
    pub estimated_impact: OperationImpact,
    pub risk_level: RiskLevel,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, Value>,
    pub timestamp: DateTime<Utc>,
}

/// Impact assessment for an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationImpact {
    pub files_affected: usize,
    pub directories_affected: usize,
    pub estimated_size_bytes: u64,
    pub estimated_duration_ms: u64,
    pub reversible: bool,
    pub backup_required: bool,
}

/// Risk level assessment
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Result of the review process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultReviewResult {
    pub review_id: String,
    pub approved_operations: Vec<String>,
    pub rejected_operations: Vec<String>,
    pub deferred_operations: Vec<String>,
    pub review_summary: ReviewSummary,
    pub confirmation_details: Vec<ConfirmationDetail>,
    pub processing_time_ms: u64,
}

/// Summary of the review process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_operations: usize,
    pub approved_count: usize,
    pub rejected_count: usize,
    pub deferred_count: usize,
    pub auto_approved_count: usize,
    pub manual_review_count: usize,
    pub total_estimated_impact: OperationImpact,
}

/// Details about a specific confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationDetail {
    pub operation_id: String,
    pub decision: ConfirmationDecision,
    pub decision_time_ms: u64,
    pub user_input: Option<String>,
    pub auto_decided: bool,
    pub reason: String,
}

/// Decision made during confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationDecision {
    Approved,
    Rejected,
    Deferred,
    Modified(Value), // Contains modified parameters
}

impl ResultReviewTool {
    pub fn new(config: ResultReviewConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(ResultReviewConfig::default())
    }

    /// Convert experimental operations to reviewable results
    pub fn prepare_results_for_review(
        &self,
        operations: &[ExperimentalOperation],
    ) -> FileManagementResult<Vec<ExperimentalResult>> {
        let mut results = Vec::new();

        for (index, operation) in operations.iter().enumerate() {
            let impact = self.assess_operation_impact(operation)?;
            let risk_level = self.assess_risk_level(operation, &impact);

            let result = ExperimentalResult {
                operation_id: format!("op_{}", index),
                operation_type: operation.operation_type.clone(),
                source_path: operation.source_path.clone(),
                target_path: operation.target_path.clone(),
                description: operation.description.clone(),
                estimated_impact: impact,
                risk_level,
                dependencies: Vec::new(), // TODO: Implement dependency analysis
                metadata: HashMap::new(),
                timestamp: operation.timestamp,
            };

            results.push(result);
        }

        Ok(results)
    }

    /// Assess the impact of an operation
    fn assess_operation_impact(
        &self,
        operation: &ExperimentalOperation,
    ) -> FileManagementResult<OperationImpact> {
        let mut impact = OperationImpact {
            files_affected: 0,
            directories_affected: 0,
            estimated_size_bytes: operation.estimated_size.unwrap_or(0),
            estimated_duration_ms: 100, // Default estimate
            reversible: true,
            backup_required: false,
        };

        // Analyze operation type to determine impact
        match operation.operation_type.as_str() {
            "move" | "rename" => {
                impact.reversible = true;
                impact.backup_required = false;
                impact.files_affected = 1;
                impact.estimated_duration_ms = 50;
            }
            "copy" => {
                impact.reversible = true;
                impact.backup_required = false;
                impact.files_affected = 1;
                impact.estimated_duration_ms = 200;
            }
            "delete" => {
                impact.reversible = false;
                impact.backup_required = true;
                impact.files_affected = 1;
                impact.estimated_duration_ms = 10;
            }
            "merge" => {
                impact.reversible = false;
                impact.backup_required = true;
                impact.directories_affected = 2;
                impact.estimated_duration_ms = 1000;
            }
            _ => {
                // Conservative defaults for unknown operations
                impact.reversible = false;
                impact.backup_required = true;
                impact.estimated_duration_ms = 500;
            }
        }

        // Adjust based on file size
        if let Some(size) = operation.estimated_size {
            impact.estimated_duration_ms += size / 1_000_000; // 1ms per MB
        }

        Ok(impact)
    }

    /// Assess the risk level of an operation
    fn assess_risk_level(
        &self,
        operation: &ExperimentalOperation,
        impact: &OperationImpact,
    ) -> RiskLevel {
        let mut risk_score = 0;

        // Risk factors
        if !impact.reversible {
            risk_score += 3;
        }
        if impact.backup_required {
            risk_score += 2;
        }
        if impact.files_affected > 10 {
            risk_score += 2;
        }
        if impact.directories_affected > 5 {
            risk_score += 2;
        }
        if impact.estimated_size_bytes > 100_000_000 {
            // 100MB
            risk_score += 1;
        }

        // Operation-specific risks
        match operation.operation_type.as_str() {
            "delete" => risk_score += 3,
            "merge" => risk_score += 2,
            "move" => risk_score += 1,
            "copy" => risk_score += 0,
            _ => risk_score += 1,
        }

        // Convert score to risk level
        match risk_score {
            0..=2 => RiskLevel::Low,
            3..=5 => RiskLevel::Medium,
            6..=8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    /// Process review based on mode and configuration
    pub fn process_review(
        &self,
        params: &ResultReviewParams,
        context: &ExecutionContext,
    ) -> FileManagementResult<ResultReviewResult> {
        let start_time = std::time::Instant::now();
        let review_id = uuid::Uuid::new_v4().to_string();

        let mut approved_operations = Vec::new();
        let mut rejected_operations = Vec::new();
        let mut deferred_operations = Vec::new();
        let mut confirmation_details = Vec::new();
        let mut auto_approved_count = 0;
        let mut manual_review_count = 0;

        match params.review_mode {
            ReviewMode::Individual => {
                for result in &params.experimental_results {
                    let decision = self.review_individual_operation(result, context)?;
                    self.apply_decision(
                        &decision,
                        result,
                        &mut approved_operations,
                        &mut rejected_operations,
                        &mut deferred_operations,
                    );

                    if decision.auto_decided {
                        auto_approved_count += 1;
                    } else {
                        manual_review_count += 1;
                    }

                    confirmation_details.push(decision);
                }
            }
            ReviewMode::Batch => {
                let batch_size = params.batch_size.unwrap_or(10);
                for batch in params.experimental_results.chunks(batch_size) {
                    let decisions = self.review_batch_operations(batch, context)?;
                    for decision in decisions {
                        let result = params
                            .experimental_results
                            .iter()
                            .find(|r| r.operation_id == decision.operation_id)
                            .unwrap();

                        self.apply_decision(
                            &decision,
                            result,
                            &mut approved_operations,
                            &mut rejected_operations,
                            &mut deferred_operations,
                        );

                        if decision.auto_decided {
                            auto_approved_count += 1;
                        } else {
                            manual_review_count += 1;
                        }

                        confirmation_details.push(decision);
                    }
                }
            }
            ReviewMode::RiskBased => {
                for result in &params.experimental_results {
                    let decision = if result.risk_level <= RiskLevel::Low
                        && self.config.auto_approve_safe_operations
                    {
                        ConfirmationDetail {
                            operation_id: result.operation_id.clone(),
                            decision: ConfirmationDecision::Approved,
                            decision_time_ms: 0,
                            user_input: None,
                            auto_decided: true,
                            reason: "Auto-approved: Low risk operation".to_string(),
                        }
                    } else {
                        self.review_individual_operation(result, context)?
                    };

                    self.apply_decision(
                        &decision,
                        result,
                        &mut approved_operations,
                        &mut rejected_operations,
                        &mut deferred_operations,
                    );

                    if decision.auto_decided {
                        auto_approved_count += 1;
                    } else {
                        manual_review_count += 1;
                    }

                    confirmation_details.push(decision);
                }
            }
            ReviewMode::Smart => {
                // Combine risk-based and batch processing
                let (safe_ops, risky_ops): (Vec<_>, Vec<_>) =
                    params.experimental_results.iter().partition(|r| {
                        r.risk_level <= RiskLevel::Low && self.config.auto_approve_safe_operations
                    });

                // Auto-approve safe operations
                for result in safe_ops {
                    let decision = ConfirmationDetail {
                        operation_id: result.operation_id.clone(),
                        decision: ConfirmationDecision::Approved,
                        decision_time_ms: 0,
                        user_input: None,
                        auto_decided: true,
                        reason: "Auto-approved: Safe operation".to_string(),
                    };

                    approved_operations.push(result.operation_id.clone());
                    auto_approved_count += 1;
                    confirmation_details.push(decision);
                }

                // Review risky operations
                for result in risky_ops {
                    let decision = self.review_individual_operation(result, context)?;
                    self.apply_decision(
                        &decision,
                        result,
                        &mut approved_operations,
                        &mut rejected_operations,
                        &mut deferred_operations,
                    );
                    manual_review_count += 1;
                    confirmation_details.push(decision);
                }
            }
        }

        let total_estimated_impact = self.calculate_total_impact(&params.experimental_results);

        let review_summary = ReviewSummary {
            total_operations: params.experimental_results.len(),
            approved_count: approved_operations.len(),
            rejected_count: rejected_operations.len(),
            deferred_count: deferred_operations.len(),
            auto_approved_count,
            manual_review_count,
            total_estimated_impact,
        };

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ResultReviewResult {
            review_id,
            approved_operations,
            rejected_operations,
            deferred_operations,
            review_summary,
            confirmation_details,
            processing_time_ms,
        })
    }

    /// Review a single operation
    fn review_individual_operation(
        &self,
        result: &ExperimentalResult,
        _context: &ExecutionContext,
    ) -> FileManagementResult<ConfirmationDetail> {
        let start_time = std::time::Instant::now();

        // Check if this should be auto-approved
        if self.should_auto_approve(result) {
            return Ok(ConfirmationDetail {
                operation_id: result.operation_id.clone(),
                decision: ConfirmationDecision::Approved,
                decision_time_ms: 0,
                user_input: None,
                auto_decided: true,
                reason: "Auto-approved based on configuration".to_string(),
            });
        }

        // Present operation for manual review
        info!("=== Operation Review Required ===");
        info!(
            "Operation: {} ({})",
            result.description, result.operation_type
        );
        info!("Risk Level: {:?}", result.risk_level);

        if let Some(source) = &result.source_path {
            info!("Source: {}", source.display());
        }
        if let Some(target) = &result.target_path {
            info!("Target: {}", target.display());
        }

        info!(
            "Impact: {} files, {} directories, {} bytes",
            result.estimated_impact.files_affected,
            result.estimated_impact.directories_affected,
            result.estimated_impact.estimated_size_bytes
        );

        info!(
            "Reversible: {}, Backup Required: {}",
            result.estimated_impact.reversible, result.estimated_impact.backup_required
        );

        // In a real implementation, this would present a UI for user decision
        // For now, we'll simulate based on risk level
        let decision = if result.risk_level >= RiskLevel::High {
            ConfirmationDecision::Deferred
        } else {
            ConfirmationDecision::Approved
        };

        let decision_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ConfirmationDetail {
            operation_id: result.operation_id.clone(),
            decision,
            decision_time_ms,
            user_input: Some("Simulated user decision".to_string()),
            auto_decided: false,
            reason: format!("Manual review for {:?} risk operation", result.risk_level),
        })
    }

    /// Review a batch of operations
    fn review_batch_operations(
        &self,
        batch: &[ExperimentalResult],
        _context: &ExecutionContext,
    ) -> FileManagementResult<Vec<ConfirmationDetail>> {
        info!("=== Batch Review: {} operations ===", batch.len());

        let mut decisions = Vec::new();

        // Show batch summary
        let total_impact = self.calculate_batch_impact(batch);
        let risk_distribution = self.analyze_risk_distribution(batch);

        info!(
            "Batch Impact: {} files, {} directories, {} bytes",
            total_impact.files_affected,
            total_impact.directories_affected,
            total_impact.estimated_size_bytes
        );

        info!(
            "Risk Distribution: Low: {}, Medium: {}, High: {}, Critical: {}",
            risk_distribution.0, risk_distribution.1, risk_distribution.2, risk_distribution.3
        );

        // For batch review, we can approve all low-risk operations and defer high-risk ones
        for result in batch {
            let decision = if result.risk_level <= RiskLevel::Medium {
                ConfirmationDecision::Approved
            } else {
                ConfirmationDecision::Deferred
            };

            decisions.push(ConfirmationDetail {
                operation_id: result.operation_id.clone(),
                decision,
                decision_time_ms: 50, // Simulated batch decision time
                user_input: Some("Batch decision".to_string()),
                auto_decided: false,
                reason: format!("Batch review: {:?} risk", result.risk_level),
            });
        }

        Ok(decisions)
    }

    /// Check if an operation should be auto-approved
    fn should_auto_approve(&self, result: &ExperimentalResult) -> bool {
        if !self.config.auto_approve_safe_operations {
            return false;
        }

        result.risk_level == RiskLevel::Low
            && result.estimated_impact.reversible
            && !result.estimated_impact.backup_required
            && result.estimated_impact.files_affected <= 5
    }

    /// Apply a decision to the appropriate list
    fn apply_decision(
        &self,
        decision: &ConfirmationDetail,
        result: &ExperimentalResult,
        approved: &mut Vec<String>,
        rejected: &mut Vec<String>,
        deferred: &mut Vec<String>,
    ) {
        match decision.decision {
            ConfirmationDecision::Approved => approved.push(result.operation_id.clone()),
            ConfirmationDecision::Rejected => rejected.push(result.operation_id.clone()),
            ConfirmationDecision::Deferred => deferred.push(result.operation_id.clone()),
            ConfirmationDecision::Modified(_) => approved.push(result.operation_id.clone()),
        }
    }

    /// Calculate total impact across all operations
    fn calculate_total_impact(&self, results: &[ExperimentalResult]) -> OperationImpact {
        let mut total = OperationImpact {
            files_affected: 0,
            directories_affected: 0,
            estimated_size_bytes: 0,
            estimated_duration_ms: 0,
            reversible: true,
            backup_required: false,
        };

        for result in results {
            total.files_affected += result.estimated_impact.files_affected;
            total.directories_affected += result.estimated_impact.directories_affected;
            total.estimated_size_bytes += result.estimated_impact.estimated_size_bytes;
            total.estimated_duration_ms += result.estimated_impact.estimated_duration_ms;

            if !result.estimated_impact.reversible {
                total.reversible = false;
            }
            if result.estimated_impact.backup_required {
                total.backup_required = true;
            }
        }

        total
    }

    /// Calculate impact for a batch of operations
    fn calculate_batch_impact(&self, batch: &[ExperimentalResult]) -> OperationImpact {
        self.calculate_total_impact(batch)
    }

    /// Analyze risk distribution in a batch
    fn analyze_risk_distribution(
        &self,
        batch: &[ExperimentalResult],
    ) -> (usize, usize, usize, usize) {
        let mut low = 0;
        let mut medium = 0;
        let mut high = 0;
        let mut critical = 0;

        for result in batch {
            match result.risk_level {
                RiskLevel::Low => low += 1,
                RiskLevel::Medium => medium += 1,
                RiskLevel::High => high += 1,
                RiskLevel::Critical => critical += 1,
            }
        }

        (low, medium, high, critical)
    }
}

/// Create a result review tool with default configuration
pub fn create_result_review_tool() -> Tool {
    use crate::tools::types::{NativeToolBuilder, ToolInput, ToolOutput};
    use crate::core::ExecutionContext;
    use std::sync::Arc;
    
    let native_tool = NativeToolBuilder::new()
        .name("result-reviewer")
        .version("1.0.0")
        .description("Review experimental results before execution")
        .category("file_management")
        .tag("review")
        .tag("experimental")
        .executor(|input: ToolInput, ctx: ExecutionContext| async move {
            let tool = ResultReviewTool::with_default_config();
            let params: ResultReviewParams = serde_json::from_value(input.params)
                .map_err(|e| WorkflowError::validation(format!("Invalid parameters: {}", e)))?;
            let result = tool.process_review(&params, &ctx)
                .map_err(|e| WorkflowError::tool(format!("Review failed: {}", e)))?;
            Ok(ToolOutput::success(serde_json::to_value(result).unwrap_or_default()))
        })
        .build()
        .expect("Failed to build result review tool");
    
    Tool::Native(Arc::new(native_tool))
}

/// Create a result review tool with custom configuration
pub fn create_result_review_tool_with_config(config: ResultReviewConfig) -> Tool {
    use crate::tools::types::{NativeToolBuilder, ToolInput, ToolOutput};
    use crate::core::ExecutionContext;
    use std::sync::Arc;
    
    let native_tool = NativeToolBuilder::new()
        .name("result-reviewer")
        .version("1.0.0")
        .description("Review experimental results before execution")
        .category("file_management")
        .tag("review")
        .tag("experimental")
        .executor(move |input: ToolInput, ctx: ExecutionContext| {
            let config = config.clone();
            async move {
                let tool = ResultReviewTool::new(config);
                let params: ResultReviewParams = serde_json::from_value(input.params)
                    .map_err(|e| WorkflowError::validation(format!("Invalid parameters: {}", e)))?;
                let result = tool.process_review(&params, &ctx)
                    .map_err(|e| WorkflowError::tool(format!("Review failed: {}", e)))?;
                Ok(ToolOutput::success(serde_json::to_value(result).unwrap_or_default()))
            }
        })
        .build()
        .expect("Failed to build result review tool");
    
    Tool::Native(Arc::new(native_tool))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_result_review_tool_creation() {
        let tool = ResultReviewTool::with_default_config();
        assert_eq!(tool.name(), "result-reviewer");
        assert_eq!(tool.version(), "1.0.0");
    }

    #[test]
    fn test_risk_assessment() {
        let tool = ResultReviewTool::with_default_config();

        let safe_operation =
            ExperimentalOperation::new("copy", "Copy file A to B").with_estimated_size(1000);

        let impact = tool.assess_operation_impact(&safe_operation).unwrap();
        let risk = tool.assess_risk_level(&safe_operation, &impact);

        assert_eq!(risk, RiskLevel::Low);
        assert!(impact.reversible);
        assert!(!impact.backup_required);
    }

    #[test]
    fn test_dangerous_operation_assessment() {
        let tool = ResultReviewTool::with_default_config();

        let dangerous_operation = ExperimentalOperation::new("delete", "Delete important files")
            .with_estimated_size(100_000_000);

        let impact = tool.assess_operation_impact(&dangerous_operation).unwrap();
        let risk = tool.assess_risk_level(&dangerous_operation, &impact);

        assert!(risk >= RiskLevel::High);
        assert!(!impact.reversible);
        assert!(impact.backup_required);
    }

    #[test]
    fn test_prepare_results_for_review() {
        let tool = ResultReviewTool::with_default_config();

        let operations = vec![
            ExperimentalOperation::new("copy", "Copy file A to B"),
            ExperimentalOperation::new("delete", "Delete file C"),
        ];

        let results = tool.prepare_results_for_review(&operations).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].operation_type, "copy");
        assert_eq!(results[1].operation_type, "delete");
        assert!(results[0].risk_level < results[1].risk_level);
    }
}
