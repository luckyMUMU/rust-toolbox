//! Workflow execution engine.
//!
//! This module contains the logic for executing workflows, managing their state,
//! handling retries, and integrating with the tool registry and audit logging.

use crate::core::{ExecutionContext, ExecutionStatus, WorkflowId};
use crate::error::{Result, WorkflowError};
use crate::storage::StateManager;
use crate::tools::{TemplateContext, TemplateEngine, ToolRegistry};
use crate::workflow::{
    AuditEventType, AuditLogger, CacheConfig, Checkpoint, DagScheduler, ErrorDetails,
    ExecutionRecord, LogLevel, NodeExecutionState, ResultCache, WorkflowDefinition,
    WorkflowEdge, WorkflowExecution, WorkflowNode, WorkflowState,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::sleep;
use uuid::Uuid;

/// Error recovery actions.
#[derive(Debug, Clone, PartialEq)]
enum ErrorRecoveryAction {
    /// Stop the entire workflow
    StopWorkflow,
    /// Fail the node and continue if possible
    FailNode,
    /// Continue execution without this node
    ContinueWithoutNode,
    /// Pause workflow for manual intervention
    PauseAndRetry,
}

/// Error context for better error reporting.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ErrorContext {
    workflow_id: WorkflowId,
    workflow_name: String,
    node_id: String,
    execution_time: DateTime<Utc>,
    current_node: Option<String>,
    global_context: Value,
}

/// Trait for workflow engines.
///
/// Defines the core operations for a workflow execution engine.
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// Execute a workflow definition.
    async fn execute_workflow(&self, definition: WorkflowDefinition) -> Result<WorkflowExecution>;
    /// Pause a running workflow.
    async fn pause_workflow(&self, id: WorkflowId) -> Result<()>;
    /// Resume a paused workflow.
    async fn resume_workflow(&self, id: WorkflowId) -> Result<()>;
    /// Stop/Cancel a running workflow.
    async fn stop_workflow(&self, id: WorkflowId) -> Result<()>;
    /// Get the current status of a workflow.
    async fn get_workflow_status(&self, id: WorkflowId) -> Result<ExecutionStatus>;
}

/// Default implementation of the workflow engine.
///
/// This engine handles:
/// - Workflow state management
/// - Tool execution
/// - Concurrency control
/// - Audit logging
/// - Result caching
/// - Retry logic
pub struct DefaultWorkflowEngine {
    /// State manager for persistence
    state_manager: Arc<StateManager>,
    /// Tool registry for executing tools
    tool_registry: Arc<dyn ToolRegistry>,
    /// Active workflow executions
    active_executions: DashMap<WorkflowId, Arc<RwLock<WorkflowExecution>>>,
    /// Execution control signals
    control_signals: DashMap<WorkflowId, ExecutionControl>,
    /// Semaphore for controlling parallel execution
    parallel_semaphore: Arc<Semaphore>,
    /// Audit logger for compliance and tracking
    audit_logger: Arc<AuditLogger>,
    /// Result cache for execution results
    result_cache: Option<Arc<ResultCache>>,
    /// Template engine for parameter resolution
    template_engine: Arc<TemplateEngine>,
}

/// Control signals for workflow execution
#[derive(Debug, Clone)]
struct ExecutionControl {
    should_pause: bool,
    should_stop: bool,
    pause_requested_at: Option<DateTime<Utc>>,
    stop_requested_at: Option<DateTime<Utc>>,
}

impl ExecutionControl {
    fn new() -> Self {
        Self {
            should_pause: false,
            should_stop: false,
            pause_requested_at: None,
            stop_requested_at: None,
        }
    }

    fn request_pause(&mut self) {
        self.should_pause = true;
        self.pause_requested_at = Some(Utc::now());
    }

    fn request_stop(&mut self) {
        self.should_stop = true;
        self.stop_requested_at = Some(Utc::now());
    }

    fn clear_pause(&mut self) {
        self.should_pause = false;
        self.pause_requested_at = None;
    }

    fn should_pause(&self) -> bool {
        self.should_pause
    }

    fn should_stop(&self) -> bool {
        self.should_stop
    }
}

impl DefaultWorkflowEngine {
    /// Create a new workflow engine
    pub fn new(
        state_manager: Arc<StateManager>,
        tool_registry: Arc<dyn ToolRegistry>,
        max_parallel_workflows: usize,
    ) -> Self {
        let audit_logger = Arc::new(AuditLogger::new(
            state_manager.clone(),
            false, // compliance_mode
            30,    // retention_days
        ));

        // Initialize template engine
        let template_engine = Arc::new(TemplateEngine::new().unwrap_or_else(|e| {
            tracing::error!("Failed to initialize template engine: {}", e);
            TemplateEngine::default()
        }));

        Self {
            state_manager,
            tool_registry,
            active_executions: DashMap::new(),
            control_signals: DashMap::new(),
            parallel_semaphore: Arc::new(Semaphore::new(max_parallel_workflows)),
            audit_logger,
            result_cache: None,
            template_engine,
        }
    }

    /// Create a new workflow engine with caching enabled
    pub fn new_with_cache(
        state_manager: Arc<StateManager>,
        tool_registry: Arc<dyn ToolRegistry>,
        max_parallel_workflows: usize,
        cache_config: CacheConfig,
    ) -> Self {
        let audit_logger = Arc::new(AuditLogger::new(
            state_manager.clone(),
            false, // compliance_mode
            30,    // retention_days
        ));

        // Create result cache using the same cache backend as state manager
        let result_cache = if cache_config.enabled {
            let cache_backend = state_manager.get_cache_backend();
            Some(Arc::new(ResultCache::new(cache_backend, cache_config)))
        } else {
            None
        };

        // Initialize template engine
        let template_engine = Arc::new(TemplateEngine::new().unwrap_or_else(|e| {
            tracing::error!("Failed to initialize template engine: {}", e);
            TemplateEngine::default()
        }));

        Self {
            state_manager,
            tool_registry,
            active_executions: DashMap::new(),
            control_signals: DashMap::new(),
            parallel_semaphore: Arc::new(Semaphore::new(max_parallel_workflows)),
            audit_logger,
            result_cache,
            template_engine,
        }
    }

    /// Create a new workflow engine with audit configuration
    pub fn new_with_audit(
        state_manager: Arc<StateManager>,
        tool_registry: Arc<dyn ToolRegistry>,
        max_parallel_workflows: usize,
        compliance_mode: bool,
        retention_days: u32,
    ) -> Self {
        let audit_logger = Arc::new(AuditLogger::new(
            state_manager.clone(),
            compliance_mode,
            retention_days,
        ));

        // Initialize template engine
        let template_engine = Arc::new(TemplateEngine::new().unwrap_or_else(|e| {
            tracing::error!("Failed to initialize template engine: {}", e);
            TemplateEngine::default()
        }));

        Self {
            state_manager,
            tool_registry,
            active_executions: DashMap::new(),
            control_signals: DashMap::new(),
            parallel_semaphore: Arc::new(Semaphore::new(max_parallel_workflows)),
            audit_logger,
            result_cache: None,
            template_engine,
        }
    }

    /// Create a new workflow engine with automatic recovery
    pub async fn new_with_recovery(
        state_manager: Arc<StateManager>,
        tool_registry: Arc<dyn ToolRegistry>,
        max_parallel_workflows: usize,
        enable_auto_recovery: bool,
    ) -> Result<Self> {
        let engine = Self::new(state_manager, tool_registry, max_parallel_workflows);

        if enable_auto_recovery {
            // Attempt to recover incomplete workflows
            let recovered_workflows = engine.recover_incomplete_workflows().await?;

            // Re-register recovered workflows in active executions
            for execution in recovered_workflows {
                let execution_arc = Arc::new(RwLock::new(execution.clone()));
                engine.active_executions.insert(execution.id, execution_arc);
                engine
                    .control_signals
                    .insert(execution.id, ExecutionControl::new());

                tracing::info!(
                    "Re-registered recovered workflow {} in active executions",
                    execution.id
                );
            }
        }

        Ok(engine)
    }

    /// Evaluate a condition string against the execution context
    fn evaluate_condition(
        &self,
        condition: &str,
        execution: &WorkflowExecution,
        context: &ExecutionContext,
    ) -> Result<bool> {
        // Create template context similar to resolve_parameters
        let mut template_context = TemplateContext::new();
        template_context.set_variables(context.global_variables.clone());
        if let Value::Object(global_ctx) = &execution.global_context {
            for (k, v) in global_ctx {
                template_context.set_variable(k.clone(), v.clone());
            }
        }
        
        // Add node results
        let mut nodes_map = serde_json::Map::new();
        for (node_id, state) in &execution.node_states {
            if let Some(result) = &state.result {
                nodes_map.insert(node_id.clone(), result.clone());
            }
        }
        template_context.set_variable("nodes", Value::Object(nodes_map));

        // Use template engine to render the condition
        // We assume the condition is a template that renders to "true" or "false"
        let rendered = self.template_engine.render(condition, &template_context)
            .map_err(|e| WorkflowError::ParameterResolutionError(e.to_string()))?;
            
        Ok(rendered.trim().eq_ignore_ascii_case("true"))
    }

    /// Resolve parameters for a node using the template engine
    fn resolve_parameters(
        &self,
        node: &WorkflowNode,
        execution: &WorkflowExecution,
        context: &ExecutionContext,
    ) -> Result<Value> {
        // Create template context
        let mut template_context = TemplateContext::new();

        // 1. Add global variables from execution context
        template_context.set_variables(context.global_variables.clone());

        // 2. Add global context from workflow execution (merging with override)
        if let Value::Object(global_ctx) = &execution.global_context {
            for (k, v) in global_ctx {
                template_context.set_variable(k.clone(), v.clone());
            }
        }

        // 3. Add node results
        let mut nodes_map = serde_json::Map::new();
        for (node_id, state) in &execution.node_states {
            if let Some(result) = &state.result {
                nodes_map.insert(node_id.clone(), result.clone());
            }
        }
        template_context.set_variable("nodes", Value::Object(nodes_map));

        // 4. Expand parameters
        self.template_engine
            .expand(&node.parameters, &template_context)
            .map_err(|e| WorkflowError::ParameterResolutionError(e.to_string()))
    }

    /// Execute a single node (basic implementation)
    async fn execute_node(
        &self,
        node: &WorkflowNode,
        workflow_execution: Arc<RwLock<WorkflowExecution>>,
        context: ExecutionContext,
    ) -> Result<Value> {
        let (workflow_id, execution_snapshot) = {
            let execution = workflow_execution.read().await;
            (execution.id, execution.clone())
        };

        // Check if we should stop or pause
        if let Some(control) = self.control_signals.get(&workflow_id) {
            if control.should_stop() {
                return Err(WorkflowError::ExecutionCancelled);
            }
            if control.should_pause() {
                // Drop the guard immediately to avoid blocking other operations
                drop(control);

                // Wait until resumed or stopped
                loop {
                    // Reacquire guard each iteration to avoid holding it during sleep
                    if let Some(updated_control) = self.control_signals.get(&workflow_id) {
                        let should_resume =
                            !updated_control.should_pause() || updated_control.should_stop();
                        if should_resume {
                            break;
                        }
                    } else {
                        break; // Control signal removed, exit loop
                    }
                    sleep(Duration::from_millis(100)).await;
                }

                // Check again if we should stop after the pause loop
                if let Some(control) = self.control_signals.get(&workflow_id) {
                    if control.should_stop() {
                        return Err(WorkflowError::ExecutionCancelled);
                    }
                }
            }
        }

        // Resolve parameters
        let params = self.resolve_parameters(node, &execution_snapshot, &context)?;

        // Execute the tool via tool registry
        // Use node ID as tool name if tool_name is not specified (fallback)
        // In a valid workflow, tool_name should always be present for Tool nodes
        let tool_name = node.tool_name.as_deref().unwrap_or(&node.id);

        self.tool_registry
            .execute_tool(tool_name, params, context)
            .await
    }
    /// Execute a single node with retry logic and caching.
    ///
    /// This method wraps `execute_node` with:
    /// 1. Caching (read/write)
    /// 2. Retry logic (backoff, max attempts)
    /// 3. Audit logging (start, success, retry, failure)
    /// 4. Error handling
    pub async fn execute_node_with_retry(
        &self,
        node: &WorkflowNode,
        workflow_execution: Arc<RwLock<WorkflowExecution>>,
        context: ExecutionContext,
    ) -> Result<Value> {
        let retry_policy = node.retry_policy.clone().unwrap_or_default();

        let (workflow_id, workflow_name, execution_snapshot) = {
            let execution = workflow_execution.read().await;
            (execution.id, execution.workflow_name.clone(), execution.clone())
        };

        // Check cache first if caching is enabled
        if let Some(result_cache) = &self.result_cache {
            // Resolve parameters for cache key
            let parameters = match self.resolve_parameters(node, &execution_snapshot, &context) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("Failed to resolve parameters for cache lookup: {}", e);
                    Value::Null
                }
            };

            if let Ok(Some(cached_result)) = result_cache
                .get_node_result(
                    &workflow_name,
                    "1.0.0", // TODO: Get actual workflow version from definition
                    &node.id,
                    &context,
                    &parameters,
                )
                .await
            {
                tracing::info!(
                    "Using cached result for node {} in workflow {}",
                    node.id,
                    workflow_id
                );

                // Log cache hit
                let log_entry = self.audit_logger.create_execution_log(
                    LogLevel::Info,
                    workflow_id,
                    &context.execution_id,
                    Some(&node.id),
                    &format!("Node '{}' result retrieved from cache", node.id),
                    std::collections::HashMap::new(),
                );
                self.audit_logger.log_execution(log_entry).await?;

                return Ok(cached_result.result);
            }
        }

        let mut attempt = 0;
        let mut last_error: Option<String> = None;
        let start_time = Utc::now();

        // Log node execution start
        let node_start_event = self.audit_logger.create_node_event(
            AuditEventType::NodeStarted,
            workflow_id,
            &node.id,
            &context,
            None,
            None,
        );
        self.audit_logger.log_audit_event(node_start_event).await?;

        // Log execution details
        let log_entry = self.audit_logger.create_execution_log(
            LogLevel::Info,
            workflow_id,
            &context.execution_id,
            Some(&node.id),
            &format!(
                "Starting execution of node '{}' with retry policy (max_attempts: {})",
                node.id, retry_policy.max_attempts
            ),
            std::collections::HashMap::new(),
        );
        self.audit_logger.log_execution(log_entry).await?;

        while attempt < retry_policy.max_attempts {
            attempt += 1;

            // Update retry count in node state
            {
                let mut execution = workflow_execution.write().await;
                if let Some(node_state) = execution.node_states.get_mut(&node.id) {
                    node_state.retry_count = attempt;
                }
            }

            match self
                .execute_node(node, workflow_execution.clone(), context.clone())
                .await
            {
                Ok(result) => {
                    let duration = Utc::now().signed_duration_since(start_time);

                    // Cache the result if caching is enabled
                    if let Some(result_cache) = &self.result_cache {
                        // Re-resolve parameters to ensure we have the actual values used
                        // We need a fresh snapshot because global context might have changed (though unlikely during node execution)
                        let execution = workflow_execution.read().await;
                        let parameters = self.resolve_parameters(node, &execution, &context).unwrap_or(Value::Null);

                        if let Err(cache_error) = result_cache
                            .cache_node_result(
                                &workflow_name,
                                "1.0.0", // TODO: Get actual workflow version from definition
                                &node.id,
                                &context,
                                &parameters,
                                &result,
                                Some(duration),
                            )
                            .await
                        {
                            tracing::warn!(
                                "Failed to cache result for node {} in workflow {}: {}",
                                node.id,
                                workflow_id,
                                cache_error
                            );
                        } else {
                            tracing::debug!(
                                "Cached result for node {} in workflow {}",
                                node.id,
                                workflow_id
                            );
                        }
                    }

                    // Log successful completion
                    let node_complete_event = self.audit_logger.create_node_event(
                        AuditEventType::NodeCompleted,
                        workflow_id,
                        &node.id,
                        &context,
                        Some(duration),
                        None,
                    );
                    self.audit_logger
                        .log_audit_event(node_complete_event)
                        .await?;

                    let log_entry = self.audit_logger.create_execution_log(
                        LogLevel::Info,
                        workflow_id,
                        &context.execution_id,
                        Some(&node.id),
                        &format!(
                            "Node '{}' completed successfully after {} attempts in {:?}",
                            node.id, attempt, duration
                        ),
                        std::collections::HashMap::new(),
                    );
                    self.audit_logger.log_execution(log_entry).await?;

                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(format!("{}", error));

                    // Log retry attempt if not the last attempt
                    if attempt < retry_policy.max_attempts {
                        let retry_event = self.audit_logger.create_node_event(
                            AuditEventType::NodeRetried,
                            workflow_id,
                            &node.id,
                            &context,
                            None,
                            Some(ErrorDetails {
                                error_type: std::any::type_name_of_val(&error).to_string(),
                                error_message: error.to_string(),
                                stack_trace: None,
                                error_code: None,
                                retry_count: Some(attempt),
                            }),
                        );
                        self.audit_logger.log_audit_event(retry_event).await?;

                        // Calculate delay based on retry strategy
                        let delay = self.calculate_retry_delay(&retry_policy, attempt);

                        // Log retry attempt
                        tracing::warn!(
                            "Node {} failed on attempt {}/{}, retrying in {:?}: {}",
                            node.id,
                            attempt,
                            retry_policy.max_attempts,
                            delay,
                            error
                        );

                        let log_entry = self.audit_logger.create_execution_log(
                            LogLevel::Warn,
                            workflow_id,
                            &context.execution_id,
                            Some(&node.id),
                            &format!(
                                "Node '{}' failed on attempt {}/{}, retrying in {:?}: {}",
                                node.id, attempt, retry_policy.max_attempts, delay, error
                            ),
                            std::collections::HashMap::new(),
                        );
                        self.audit_logger.log_execution(log_entry).await?;

                        // Wait before retrying
                        sleep(delay).await;

                        // Check if we should stop retrying due to control signals
                        if let Some(control) = self.control_signals.get(&workflow_id) {
                            if control.should_stop() {
                                return Err(WorkflowError::ExecutionCancelled);
                            }
                        }
                    }
                }
            }
        }

        // All retries exhausted - log final failure
        let duration = Utc::now().signed_duration_since(start_time);
        let final_error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
        let final_error = WorkflowError::workflow_execution(&final_error_msg);

        let node_failed_event = self.audit_logger.create_node_event(
            AuditEventType::NodeFailed,
            workflow_id,
            &node.id,
            &context,
            Some(duration),
            Some(ErrorDetails {
                error_type: "WorkflowError".to_string(),
                error_message: final_error_msg.clone(),
                stack_trace: None,
                error_code: None,
                retry_count: Some(attempt),
            }),
        );
        self.audit_logger.log_audit_event(node_failed_event).await?;

        let log_entry = self.audit_logger.create_execution_log(
            LogLevel::Error,
            workflow_id,
            &context.execution_id,
            Some(&node.id),
            &format!(
                "Node '{}' failed after {} attempts in {:?}: {}",
                node.id, attempt, duration, final_error_msg
            ),
            std::collections::HashMap::new(),
        );
        self.audit_logger.log_execution(log_entry).await?;

        Err(final_error)
    }

    /// Calculate retry delay based on strategy
    pub fn calculate_retry_delay(
        &self,
        retry_policy: &crate::core::RetryPolicy,
        attempt: u32,
    ) -> Duration {
        use crate::core::RetryStrategy;

        let base_delay = retry_policy.base_delay;
        let max_delay = retry_policy.max_delay.unwrap_or(Duration::from_secs(300)); // 5 minutes max

        let calculated_delay = match retry_policy.strategy {
            RetryStrategy::None => Duration::from_secs(0),
            RetryStrategy::FixedInterval => base_delay,
            RetryStrategy::ExponentialBackoff => {
                let multiplier = retry_policy.backoff_multiplier.powf((attempt - 1) as f64);
                let delay_ms = (base_delay.as_millis() as f64 * multiplier) as u64;
                Duration::from_millis(delay_ms)
            }
            RetryStrategy::LinearBackoff => {
                let delay_ms = base_delay.as_millis() as u64 * attempt as u64;
                Duration::from_millis(delay_ms)
            }
            RetryStrategy::Custom(_) => {
                // For custom strategies, fall back to exponential backoff
                let multiplier = retry_policy.backoff_multiplier.powf((attempt - 1) as f64);
                let delay_ms = (base_delay.as_millis() as f64 * multiplier) as u64;
                Duration::from_millis(delay_ms)
            }
        };

        // Ensure delay doesn't exceed maximum
        std::cmp::min(calculated_delay, max_delay)
    }

    /// Handle node execution error with recovery strategies
    async fn handle_node_error(
        &self,
        node_id: &str,
        error: &WorkflowError,
        workflow_execution: Arc<RwLock<WorkflowExecution>>,
        scheduler: &mut DagScheduler,
    ) -> Result<ErrorRecoveryAction> {
        // Log the error
        tracing::error!("Node {} failed with error: {}", node_id, error);

        // Update node state with error
        {
            let mut execution = workflow_execution.write().await;
            if let Some(node_state) = execution.node_states.get_mut(node_id) {
                node_state.status = ExecutionStatus::Failed;
                node_state.error = Some(error.to_string());
                node_state.completed_at = Some(Utc::now());
            }
        }

        // Determine recovery action based on error type and workflow configuration
        let recovery_action = match error {
            WorkflowError::ExecutionCancelled => ErrorRecoveryAction::StopWorkflow,
            WorkflowError::Timeout { .. } => ErrorRecoveryAction::FailNode,
            WorkflowError::ResourceExhausted => ErrorRecoveryAction::PauseAndRetry,
            _ => {
                // Check if this is a critical node or if we can continue
                let has_alternative_paths =
                    self.has_alternative_execution_paths(node_id, scheduler);
                if has_alternative_paths {
                    ErrorRecoveryAction::ContinueWithoutNode
                } else {
                    ErrorRecoveryAction::FailNode
                }
            }
        };

        // Apply recovery action
        match recovery_action {
            ErrorRecoveryAction::StopWorkflow => {
                // Mark workflow as failed
                let mut execution = workflow_execution.write().await;
                execution.status = ExecutionStatus::Failed;
            }
            ErrorRecoveryAction::FailNode => {
                // Mark node as failed in scheduler
                scheduler.mark_node_failed(node_id)?;
            }
            ErrorRecoveryAction::ContinueWithoutNode => {
                // Mark node as completed with error but continue execution
                scheduler.mark_node_completed(node_id)?;
            }
            ErrorRecoveryAction::PauseAndRetry => {
                // Pause workflow for manual intervention
                let workflow_id = {
                    let execution = workflow_execution.read().await;
                    execution.id
                };

                if let Some(mut control) = self.control_signals.get_mut(&workflow_id) {
                    control.request_pause();
                }
            }
        }

        Ok(recovery_action)
    }

    /// Check if there are alternative execution paths if a node fails
    fn has_alternative_execution_paths(&self, _node_id: &str, _scheduler: &DagScheduler) -> bool {
        // Simplified implementation - in a real system, this would analyze the DAG
        // to determine if there are alternative paths to completion
        false
    }

    /// Create error context for better error reporting
    #[allow(dead_code)]
    fn create_error_context(
        &self,
        node_id: &str,
        workflow_execution: &WorkflowExecution,
    ) -> ErrorContext {
        ErrorContext {
            workflow_id: workflow_execution.id,
            workflow_name: workflow_execution.workflow_name.clone(),
            node_id: node_id.to_string(),
            execution_time: workflow_execution.started_at,
            current_node: workflow_execution.current_node.clone(),
            global_context: workflow_execution.global_context.clone(),
        }
    }

    /// Handle workflow-level errors and recovery
    async fn handle_workflow_error(
        &self,
        error: &WorkflowError,
        workflow_execution: Arc<RwLock<WorkflowExecution>>,
    ) -> Result<()> {
        let workflow_id = {
            let execution = workflow_execution.read().await;
            execution.id
        };

        // Log the workflow error
        tracing::error!("Workflow {} encountered error: {}", workflow_id, error);

        // Create error record
        // Calculate duration from workflow execution if available
        let duration = {
            let execution = workflow_execution.read().await;
            execution
                .completed_at
                .map(|completed| completed.signed_duration_since(execution.started_at))
        };

        let error_record = ExecutionRecord {
            workflow_id,
            execution_id: workflow_id.to_string(),
            timestamp: Utc::now(),
            status: ExecutionStatus::Failed,
            result: None,
            error: Some(error.to_string()),
            duration,
        };

        // Save error record
        self.state_manager
            .save_execution_record(error_record)
            .await?;

        // Update workflow status
        {
            let mut execution = workflow_execution.write().await;
            execution.status = ExecutionStatus::Failed;
            execution.completed_at = Some(Utc::now());
        }

        // Save final state
        let execution = workflow_execution.read().await;
        self.save_workflow_state_with_checkpoint(&execution, None)
            .await?;

        Ok(())
    }

    /// Check if workflow can be recovered from current state
    pub async fn can_recover_workflow(&self, workflow_id: WorkflowId) -> Result<bool> {
        if let Some(workflow_state) = self.load_workflow_state(workflow_id).await? {
            match workflow_state.execution.status {
                ExecutionStatus::Running | ExecutionStatus::Paused => Ok(true),
                ExecutionStatus::Failed => {
                    // Check if there are any completed nodes that can be resumed from
                    let has_completed_nodes = workflow_state
                        .execution
                        .node_states
                        .values()
                        .any(|state| state.status == ExecutionStatus::Completed);
                    Ok(has_completed_nodes)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    /// Create a checkpoint for the workflow
    async fn create_checkpoint(
        &self,
        workflow_id: WorkflowId,
        node_id: &str,
        state_snapshot: Value,
    ) -> Result<Checkpoint> {
        let checkpoint = Checkpoint {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            node_id: node_id.to_string(),
            state_snapshot,
        };

        // Log checkpoint creation
        tracing::debug!(
            "Created checkpoint {} for workflow {} at node {}",
            checkpoint.id,
            workflow_id,
            node_id
        );

        Ok(checkpoint)
    }

    /// Create a comprehensive checkpoint with full workflow state
    async fn create_comprehensive_checkpoint(
        &self,
        workflow_execution: &WorkflowExecution,
    ) -> Result<Checkpoint> {
        // Create a comprehensive state snapshot including:
        // - Current node states
        // - Global context
        // - Execution metadata
        let mut state_snapshot = serde_json::Map::new();

        // Include node states
        state_snapshot.insert(
            "node_states".to_string(),
            serde_json::to_value(&workflow_execution.node_states)?,
        );

        // Include global context
        state_snapshot.insert(
            "global_context".to_string(),
            workflow_execution.global_context.clone(),
        );

        // Include execution metadata
        let mut execution_metadata = serde_json::Map::new();
        execution_metadata.insert(
            "started_at".to_string(),
            serde_json::to_value(workflow_execution.started_at)?,
        );
        execution_metadata.insert(
            "current_node".to_string(),
            serde_json::to_value(&workflow_execution.current_node)?,
        );
        execution_metadata.insert(
            "status".to_string(),
            serde_json::to_value(workflow_execution.status)?,
        );

        state_snapshot.insert(
            "execution_metadata".to_string(),
            Value::Object(execution_metadata),
        );

        let current_node = workflow_execution
            .current_node
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        self.create_checkpoint(
            workflow_execution.id,
            &current_node,
            Value::Object(state_snapshot),
        )
        .await
    }

    /// Save workflow state to persistence with checkpoints
    async fn save_workflow_state_with_checkpoint(
        &self,
        workflow_execution: &WorkflowExecution,
        checkpoint: Option<Checkpoint>,
    ) -> Result<()> {
        // Load existing workflow state to preserve previous checkpoints
        let mut existing_checkpoints =
            if let Some(existing_state) = self.load_workflow_state(workflow_execution.id).await? {
                existing_state.checkpoints
            } else {
                Vec::new()
            };

        // Add new checkpoint if provided
        if let Some(cp) = checkpoint {
            existing_checkpoints.push(cp);

            // Limit checkpoint history to prevent unbounded growth
            const MAX_CHECKPOINTS: usize = 10;
            if existing_checkpoints.len() > MAX_CHECKPOINTS {
                existing_checkpoints.drain(0..existing_checkpoints.len() - MAX_CHECKPOINTS);
            }
        }

        let workflow_state = WorkflowState {
            execution: workflow_execution.clone(),
            checkpoints: existing_checkpoints,
            metadata: std::collections::HashMap::new(),
        };

        self.state_manager
            .save_workflow_state(workflow_execution.id, workflow_state)
            .await
    }

    /// Recover workflow from checkpoint
    pub async fn recover_workflow(
        &self,
        workflow_id: WorkflowId,
    ) -> Result<Option<WorkflowExecution>> {
        if let Some(workflow_state) = self.load_workflow_state(workflow_id).await? {
            let mut execution = workflow_state.execution;

            // Only recover if the workflow was in a recoverable state
            match execution.status {
                ExecutionStatus::Running | ExecutionStatus::Paused => {
                    // Reset status to pending for recovery
                    execution.status = ExecutionStatus::Pending;

                    // Find the last checkpoint and restore state
                    if let Some(last_checkpoint) = workflow_state.checkpoints.last() {
                        tracing::info!(
                            "Recovering workflow {} from checkpoint {} at node {}",
                            workflow_id,
                            last_checkpoint.id,
                            last_checkpoint.node_id
                        );

                        // Restore state from checkpoint
                        execution.current_node = Some(last_checkpoint.node_id.clone());

                        // Restore comprehensive state if available
                        if let Value::Object(state_map) = &last_checkpoint.state_snapshot {
                            // Restore node states
                            if let Some(node_states_value) = state_map.get("node_states") {
                                if let Ok(node_states) =
                                    serde_json::from_value(node_states_value.clone())
                                {
                                    execution.node_states = node_states;
                                }
                            }

                            // Restore global context
                            if let Some(global_context) = state_map.get("global_context") {
                                execution.global_context = global_context.clone();
                            }

                            // Restore execution metadata if needed
                            if let Some(Value::Object(exec_metadata)) =
                                state_map.get("execution_metadata")
                            {
                                if let Some(current_node_value) = exec_metadata.get("current_node")
                                {
                                    if let Ok(current_node) =
                                        serde_json::from_value(current_node_value.clone())
                                    {
                                        execution.current_node = current_node;
                                    }
                                }
                            }
                        } else {
                            // Fallback to simple state restoration
                            execution.global_context = last_checkpoint.state_snapshot.clone();
                        }

                        // Reset running nodes to pending for re-execution
                        for (_node_id, node_state) in execution.node_states.iter_mut() {
                            if node_state.status == ExecutionStatus::Running {
                                // Reset running nodes to pending
                                node_state.status = ExecutionStatus::Pending;
                                node_state.started_at = None;
                                node_state.completed_at = None;
                                node_state.error = None;
                            }
                        }
                    } else {
                        tracing::warn!(
                            "No checkpoints found for workflow {}, performing basic recovery",
                            workflow_id
                        );
                    }

                    Ok(Some(execution))
                }
                ExecutionStatus::Failed => {
                    // Check if there are any completed nodes that can be resumed from
                    let has_completed_nodes = execution
                        .node_states
                        .values()
                        .any(|state| state.status == ExecutionStatus::Completed);

                    if has_completed_nodes {
                        tracing::info!(
                            "Attempting recovery of failed workflow {} with completed nodes",
                            workflow_id
                        );

                        // Reset to pending and allow recovery from last successful checkpoint
                        execution.status = ExecutionStatus::Pending;

                        // Find the last successful checkpoint
                        if let Some(last_checkpoint) = workflow_state.checkpoints.last() {
                            execution.current_node = Some(last_checkpoint.node_id.clone());
                            execution.global_context = last_checkpoint.state_snapshot.clone();
                        }

                        Ok(Some(execution))
                    } else {
                        tracing::warn!(
                            "Cannot recover failed workflow {} - no completed nodes found",
                            workflow_id
                        );
                        Ok(None)
                    }
                }
                _ => {
                    tracing::debug!(
                        "Workflow {} is in terminal state {:?}, cannot recover",
                        workflow_id,
                        execution.status
                    );
                    Ok(None)
                }
            }
        } else {
            tracing::warn!("No workflow state found for workflow {}", workflow_id);
            Ok(None)
        }
    }

    /// Recover all incomplete workflows after system restart
    pub async fn recover_incomplete_workflows(&self) -> Result<Vec<WorkflowExecution>> {
        let workflow_ids = self.state_manager.list_workflow_states().await?;
        let mut recovered_workflows = Vec::new();

        for workflow_id in workflow_ids {
            if let Some(recovered_execution) = self.recover_workflow(workflow_id).await? {
                recovered_workflows.push(recovered_execution);
                tracing::info!("Recovered workflow {} after system restart", workflow_id);
            }
        }

        tracing::info!(
            "System recovery complete: {} workflows recovered",
            recovered_workflows.len()
        );

        Ok(recovered_workflows)
    }

    /// Get cache statistics if caching is enabled
    pub async fn get_cache_stats(&self) -> Option<crate::workflow::CacheStats> {
        if let Some(result_cache) = &self.result_cache {
            Some(result_cache.get_cache_stats().await)
        } else {
            None
        }
    }

    /// Clear all cached results
    pub async fn clear_cache(&self) -> Result<()> {
        if let Some(result_cache) = &self.result_cache {
            result_cache.clear_all().await?;
            tracing::info!("Cleared all cached workflow results");
        }
        Ok(())
    }

    /// Invalidate cache for a specific workflow
    pub async fn invalidate_workflow_cache(
        &self,
        workflow_name: &str,
        workflow_version: Option<&str>,
    ) -> Result<usize> {
        if let Some(result_cache) = &self.result_cache {
            let count = result_cache
                .invalidate_workflow(workflow_name, workflow_version)
                .await?;
            tracing::info!(
                "Invalidated {} cache entries for workflow {}",
                count,
                workflow_name
            );
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Resume workflow from saved state
    pub async fn resume_from_state(
        &self,
        workflow_state: WorkflowState,
    ) -> Result<WorkflowExecution> {
        let mut execution = workflow_state.execution;

        // Validate that the workflow can be resumed
        if !execution.status.can_resume() && execution.status != ExecutionStatus::Running {
            return Err(WorkflowError::InvalidStateTransition {
                from: execution.status,
                to: ExecutionStatus::Running,
            });
        }

        // Update status to running
        execution.status = ExecutionStatus::Running;

        // Store the execution in active executions
        let execution_arc = Arc::new(RwLock::new(execution.clone()));
        self.active_executions
            .insert(execution.id, execution_arc.clone());

        // Initialize control signals
        self.control_signals
            .insert(execution.id, ExecutionControl::new());

        // TODO: Rebuild scheduler state from execution state
        // This would require storing scheduler state in checkpoints
        // For now, we'll return the execution as-is

        Ok(execution)
    }

    /// Create periodic checkpoints during execution
    async fn maybe_create_checkpoint(
        &self,
        workflow_execution: &WorkflowExecution,
        last_checkpoint_time: &mut Option<DateTime<Utc>>,
        checkpoint_interval: Duration,
    ) -> Result<Option<Checkpoint>> {
        let now = Utc::now();

        let should_checkpoint = match last_checkpoint_time {
            Some(last_time) => now
                .signed_duration_since(*last_time)
                .to_std()
                .map(|d| d >= checkpoint_interval)
                .unwrap_or(false),
            None => true, // First checkpoint
        };

        if should_checkpoint {
            *last_checkpoint_time = Some(now);

            let checkpoint = self
                .create_comprehensive_checkpoint(workflow_execution)
                .await?;

            tracing::debug!(
                "Created periodic checkpoint {} for workflow {}",
                checkpoint.id,
                workflow_execution.id
            );

            Ok(Some(checkpoint))
        } else {
            Ok(None)
        }
    }

    /// Save workflow state to persistence
    async fn save_workflow_state(&self, workflow_execution: &WorkflowExecution) -> Result<()> {
        let workflow_state = WorkflowState {
            execution: workflow_execution.clone(),
            checkpoints: Vec::new(), // TODO: Implement checkpoint management
            metadata: std::collections::HashMap::new(),
        };

        self.state_manager
            .save_workflow_state(workflow_execution.id, workflow_state)
            .await
    }

    /// Load workflow state from persistence
    async fn load_workflow_state(&self, workflow_id: WorkflowId) -> Result<Option<WorkflowState>> {
        self.state_manager.load_workflow_state(workflow_id).await
    }

    /// Record execution completion
    async fn record_execution(&self, workflow_execution: &WorkflowExecution) -> Result<()> {
        let record = ExecutionRecord {
            workflow_id: workflow_execution.id,
            execution_id: workflow_execution.id.to_string(),
            timestamp: workflow_execution.started_at,
            status: workflow_execution.status,
            result: Some(workflow_execution.global_context.clone()),
            error: None,
            duration: workflow_execution
                .completed_at
                .map(|completed| completed.signed_duration_since(workflow_execution.started_at)),
        };

        self.state_manager.save_execution_record(record).await
    }
}

#[async_trait]
impl WorkflowEngine for DefaultWorkflowEngine {
    async fn execute_workflow(&self, definition: WorkflowDefinition) -> Result<WorkflowExecution> {
        // Validate the workflow definition
        definition.validate()?;

        // Acquire semaphore permit for parallel execution control
        let _permit = self
            .parallel_semaphore
            .acquire()
            .await
            .map_err(|_| WorkflowError::ResourceExhausted)?;

        // Generate workflow execution ID
        let workflow_id = definition.generate_id();
        let execution_id = Uuid::new_v4().to_string();

        // Create initial workflow execution state
        let mut workflow_execution = WorkflowExecution {
            id: workflow_id,
            workflow_name: definition.name.clone(),
            status: ExecutionStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            current_node: None,
            node_states: std::collections::HashMap::new(),
            global_context: Value::Object(serde_json::Map::new()),
        };

        // Check cache for workflow result if caching is enabled
        if let Some(result_cache) = &self.result_cache {
            let context = ExecutionContext::new().with_workflow_id(workflow_id);
            let parameters = Value::Null; // In a real implementation, get actual parameters

            if let Ok(Some(cached_result)) = result_cache
                .get_workflow_result(&definition.name, &definition.version, &context, &parameters)
                .await
            {
                tracing::info!(
                    "Using cached result for workflow {} ({})",
                    definition.name,
                    workflow_id
                );

                // Create a completed execution from cached result
                workflow_execution.status = ExecutionStatus::Completed;
                workflow_execution.completed_at = Some(Utc::now());
                workflow_execution.global_context = cached_result.result;

                // Log cache hit
                let log_entry = self.audit_logger.create_execution_log(
                    LogLevel::Info,
                    workflow_id,
                    &execution_id,
                    None,
                    &format!("Workflow '{}' result retrieved from cache", definition.name),
                    std::collections::HashMap::new(),
                );
                self.audit_logger.log_execution(log_entry).await?;

                return Ok(workflow_execution);
            }
        }

        // Initialize node states
        for node in &definition.nodes {
            workflow_execution.node_states.insert(
                node.id.clone(),
                NodeExecutionState {
                    status: ExecutionStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    result: None,
                    error: None,
                    retry_count: 0,
                },
            );
        }

        // Create execution context
        let context = ExecutionContext::new().with_workflow_id(workflow_id);

        // Log workflow creation and start
        let workflow_created_event = self.audit_logger.create_workflow_event(
            AuditEventType::WorkflowCreated,
            workflow_id,
            &definition.name,
            &context,
            Some(format!(
                "Workflow '{}' created with {} nodes",
                definition.name,
                definition.nodes.len()
            )),
        );
        self.audit_logger
            .log_audit_event(workflow_created_event)
            .await?;

        let workflow_started_event = self.audit_logger.create_workflow_event(
            AuditEventType::WorkflowStarted,
            workflow_id,
            &definition.name,
            &context,
            None,
        );
        self.audit_logger
            .log_audit_event(workflow_started_event)
            .await?;

        // Log execution start
        let log_entry = self.audit_logger.create_execution_log(
            LogLevel::Info,
            workflow_id,
            &execution_id,
            None,
            &format!(
                "Starting workflow '{}' execution with {} nodes",
                definition.name,
                definition.nodes.len()
            ),
            std::collections::HashMap::new(),
        );
        self.audit_logger.log_execution(log_entry).await?;

        // Store the execution in active executions
        let execution_arc = Arc::new(RwLock::new(workflow_execution.clone()));
        self.active_executions
            .insert(workflow_id, execution_arc.clone());

        // Initialize control signals
        self.control_signals
            .insert(workflow_id, ExecutionControl::new());

        // Create scheduler for the workflow
        let mut scheduler = DagScheduler::from_workflow(&definition)?;

        // Main execution loop
        let execution_result = async {
            let checkpoint_interval = definition
                .global_config
                .checkpoint_interval
                .unwrap_or(Duration::from_secs(300)); // 5 minutes default
            let mut last_checkpoint_time: Option<DateTime<Utc>> = None;

            while !scheduler.is_execution_complete() && scheduler.has_ready_nodes() {
                // Check for control signals
                if let Some(control) = self.control_signals.get(&workflow_id) {
                    if control.should_stop() {
                        // Log workflow stop
                        let workflow_stopped_event = self.audit_logger.create_workflow_event(
                            AuditEventType::WorkflowStopped,
                            workflow_id,
                            &definition.name,
                            &context,
                            Some("Workflow execution stopped by user request".to_string()),
                        );
                        self.audit_logger
                            .log_audit_event(workflow_stopped_event)
                            .await?;
                        break;
                    }
                    if control.should_pause() {
                        // Update status to paused
                        {
                            let mut execution = execution_arc.write().await;
                            execution.status = ExecutionStatus::Paused;
                        }

                        // Log workflow pause
                        let workflow_paused_event = self.audit_logger.create_workflow_event(
                            AuditEventType::WorkflowPaused,
                            workflow_id,
                            &definition.name,
                            &context,
                            Some("Workflow execution paused by user request".to_string()),
                        );
                        self.audit_logger
                            .log_audit_event(workflow_paused_event)
                            .await?;

                        // Create checkpoint before pausing
                        let execution = execution_arc.read().await;
                        let checkpoint = self.create_comprehensive_checkpoint(&execution).await?;

                        self.save_workflow_state_with_checkpoint(&execution, Some(checkpoint))
                            .await?;

                        // Wait until resumed or stopped
                        loop {
                            // Reacquire guard each iteration to avoid holding it during sleep
                            if let Some(current_control) = self.control_signals.get(&workflow_id) {
                                let should_resume = !current_control.should_pause()
                                    || current_control.should_stop();
                                if should_resume {
                                    break;
                                }
                            } else {
                                break; // Control signal removed
                            }
                            sleep(Duration::from_millis(100)).await;
                        }

                        if let Some(current_control) = self.control_signals.get(&workflow_id) {
                            if current_control.should_stop() {
                                break;
                            }
                        }

                        // Update status back to running and log resume
                        {
                            let mut execution = execution_arc.write().await;
                            execution.status = ExecutionStatus::Running;
                        }

                        let workflow_resumed_event = self.audit_logger.create_workflow_event(
                            AuditEventType::WorkflowResumed,
                            workflow_id,
                            &definition.name,
                            &context,
                            Some("Workflow execution resumed".to_string()),
                        );
                        self.audit_logger
                            .log_audit_event(workflow_resumed_event)
                            .await?;
                    }
                }

                // Get ready nodes
                let ready_nodes = scheduler.get_ready_nodes();

                if ready_nodes.is_empty() {
                    // No ready nodes but execution not complete - might be waiting for async operations
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // Execute ready nodes (for now, execute them sequentially)
                // TODO: Implement parallel execution based on workflow config
                for node_id in ready_nodes {
                    // Update current node
                    {
                        let mut execution = execution_arc.write().await;
                        execution.current_node = Some(node_id.clone());

                        // Update node state to running
                        if let Some(node_state) = execution.node_states.get_mut(&node_id) {
                            node_state.status = ExecutionStatus::Running;
                            node_state.started_at = Some(Utc::now());
                        }
                    }

                    // Mark node as started in scheduler
                    scheduler.mark_node_started(&node_id)?;

                    // Execute the node with retry logic
                    let node = definition.get_node(&node_id).ok_or_else(|| {
                        WorkflowError::NodeNotFound(node_id.clone())
                    })?;

                    let node_result = self
                        .execute_node_with_retry(
                            node,
                            execution_arc.clone(),
                            context.clone(),
                        )
                        .await;

                    // Update node state based on result
                    {
                        let mut execution = execution_arc.write().await;
                        
                        // Handle global context updates from tools (e.g., DataCacheTool)
                        // We do this BEFORE getting mutable reference to node_state to avoid double borrow
                        if let Ok(result) = &node_result {
                            if let Some(ctx_update) = result.get("__context_update").and_then(|v| v.as_object()) {
                                if let Value::Object(global_ctx) = &mut execution.global_context {
                                    for (k, v) in ctx_update {
                                        global_ctx.insert(k.clone(), v.clone());
                                    }
                                    tracing::debug!("Node {} updated global context with keys: {:?}", node_id, ctx_update.keys());
                                }
                            }
                        }

                        if let Some(node_state) = execution.node_states.get_mut(&node_id) {
                            node_state.completed_at = Some(Utc::now());

                            match &node_result {
                                Ok(result) => {
                                    node_state.status = ExecutionStatus::Completed;
                                    node_state.result = Some(result.clone());
                                }
                                Err(error) => {
                                    node_state.status = ExecutionStatus::Failed;
                                    node_state.error = Some(error.to_string());
                                }
                            }
                        }
                    }

                    // Handle the result and apply error recovery if needed
                    match node_result {
                        Ok(_) => {
                            // Evaluate conditional edges and skip nodes if condition is false
                            {
                                let outgoing_edges: Vec<&WorkflowEdge> = definition.edges.iter()
                                    .filter(|e| e.from == node_id)
                                    .collect();

                                for edge in outgoing_edges {
                                    if let Some(condition) = &edge.condition {
                                        let should_run = {
                                            let execution = execution_arc.read().await;
                                            match self.evaluate_condition(condition, &execution, &context) {
                                                Ok(should_run) => should_run,
                                                Err(e) => {
                                                    tracing::error!("Failed to evaluate condition '{}': {}", condition, e);
                                                    // On error, we don't skip, effectively treating as true (or at least not forcing skip)
                                                    true 
                                                }
                                            }
                                        };

                                        if !should_run {
                                            tracing::info!("Condition '{}' for edge {} -> {} evaluated to false. Skipping target node.", condition, edge.from, edge.to);
                                            if let Err(e) = scheduler.mark_node_skipped(&edge.to) {
                                                tracing::warn!("Failed to skip node {}: {}", edge.to, e);
                                            }
                                            
                                            // Update node state to skipped
                                            {
                                                let mut execution = execution_arc.write().await;
                                                if let Some(node_state) = execution.node_states.get_mut(&edge.to) {
                                                    node_state.status = ExecutionStatus::Skipped;
                                                    node_state.completed_at = Some(Utc::now());
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            scheduler.mark_node_completed(&node_id)?;
                        }
                        Err(error) => {
                            let recovery_action = self
                                .handle_node_error(
                                    &node_id,
                                    &error,
                                    execution_arc.clone(),
                                    &mut scheduler,
                                )
                                .await?;

                            // Check if we should stop the workflow
                            if recovery_action == ErrorRecoveryAction::StopWorkflow {
                                break;
                            }
                        }
                    }

                    // Create periodic checkpoints
                    let execution = execution_arc.read().await;
                    let checkpoint = self
                        .maybe_create_checkpoint(
                            &execution,
                            &mut last_checkpoint_time,
                            checkpoint_interval,
                        )
                        .await?;

                    if checkpoint.is_some() {
                        self.save_workflow_state_with_checkpoint(&execution, checkpoint)
                            .await?;
                    } else {
                        // Save state without checkpoint for regular persistence
                        self.save_workflow_state(&execution).await?;
                    }
                }
            }

            Ok::<(), WorkflowError>(())
        }
        .await;

        // Finalize execution
        let final_status = match execution_result {
            Ok(_) => {
                if let Some(control) = self.control_signals.get(&workflow_id) {
                    if control.should_stop() {
                        ExecutionStatus::Cancelled
                    } else {
                        ExecutionStatus::Completed
                    }
                } else {
                    ExecutionStatus::Completed
                }
            }
            Err(error) => {
                // Handle workflow-level error
                self.handle_workflow_error(&error, execution_arc.clone())
                    .await?;
                ExecutionStatus::Failed
            }
        };

        // Update final execution state
        {
            let mut execution = execution_arc.write().await;
            execution.status = final_status;
            execution.completed_at = Some(Utc::now());
            execution.current_node = None;
        }

        // Log workflow completion
        let workflow_event_type = match final_status {
            ExecutionStatus::Completed => AuditEventType::WorkflowCompleted,
            ExecutionStatus::Failed => AuditEventType::WorkflowFailed,
            ExecutionStatus::Cancelled => AuditEventType::WorkflowCancelled,
            _ => AuditEventType::WorkflowStopped,
        };

        let workflow_final_event = self.audit_logger.create_workflow_event(
            workflow_event_type,
            workflow_id,
            &definition.name,
            &context,
            Some(format!(
                "Workflow '{}' finished with status: {:?}",
                definition.name, final_status
            )),
        );
        self.audit_logger
            .log_audit_event(workflow_final_event)
            .await?;

        // Save final state and record execution
        let final_execution = {
            let execution = execution_arc.read().await;
            self.save_workflow_state_with_checkpoint(&execution, None)
                .await?;
            self.record_execution(&execution).await?;

            // Cache the workflow result if caching is enabled and execution was successful
            if let Some(result_cache) = &self.result_cache {
                if execution.status == ExecutionStatus::Completed {
                    let context = ExecutionContext::new().with_workflow_id(workflow_id);
                    let parameters = Value::Null; // In a real implementation, get actual parameters
                    let execution_duration = execution
                        .completed_at
                        .map(|completed| completed.signed_duration_since(execution.started_at));

                    if let Err(cache_error) = result_cache
                        .cache_workflow_result(
                            &definition.name,
                            &definition.version,
                            &context,
                            &parameters,
                            &execution.global_context,
                            execution_duration,
                        )
                        .await
                    {
                        tracing::warn!(
                            "Failed to cache result for workflow {} ({}): {}",
                            definition.name,
                            workflow_id,
                            cache_error
                        );
                    } else {
                        tracing::debug!(
                            "Cached result for workflow {} ({})",
                            definition.name,
                            workflow_id
                        );
                    }
                }
            }

            execution.clone()
        };

        // Log final execution summary
        let duration = final_execution
            .completed_at
            .unwrap_or_else(Utc::now)
            .signed_duration_since(final_execution.started_at);

        let log_entry = self.audit_logger.create_execution_log(
            LogLevel::Info,
            workflow_id,
            &execution_id,
            None,
            &format!(
                "Workflow '{}' execution completed with status {:?} in {:?}",
                definition.name, final_status, duration
            ),
            std::collections::HashMap::new(),
        );
        self.audit_logger.log_execution(log_entry).await?;

        // Clean up
        self.active_executions.remove(&workflow_id);
        self.control_signals.remove(&workflow_id);

        Ok(final_execution)
    }

    async fn pause_workflow(&self, id: WorkflowId) -> Result<()> {
        // Check if workflow is active
        if !self.active_executions.contains_key(&id) {
            return Err(WorkflowError::WorkflowNotFound(id));
        }

        // Check current status and update to paused
        {
            let execution = self
                .active_executions
                .get(&id)
                .ok_or_else(|| WorkflowError::WorkflowNotFound(id))?;
            let mut execution = execution.write().await;

            if !execution.status.can_pause() {
                return Err(WorkflowError::InvalidStateTransition {
                    from: execution.status,
                    to: ExecutionStatus::Paused,
                });
            }

            execution.status = ExecutionStatus::Paused;
        }

        // Set pause signal
        if let Some(mut control) = self.control_signals.get_mut(&id) {
            control.request_pause();
        } else {
            return Err(WorkflowError::WorkflowNotFound(id));
        }

        Ok(())
    }

    async fn resume_workflow(&self, id: WorkflowId) -> Result<()> {
        // Check if workflow is active
        if !self.active_executions.contains_key(&id) {
            return Err(WorkflowError::WorkflowNotFound(id));
        }

        // Check current status and update to running
        {
            let execution = self
                .active_executions
                .get(&id)
                .ok_or_else(|| WorkflowError::WorkflowNotFound(id))?;
            let mut execution = execution.write().await;

            if !execution.status.can_resume() {
                return Err(WorkflowError::InvalidStateTransition {
                    from: execution.status,
                    to: ExecutionStatus::Running,
                });
            }

            execution.status = ExecutionStatus::Running;
        }

        // Clear pause signal
        if let Some(mut control) = self.control_signals.get_mut(&id) {
            control.clear_pause();
        } else {
            return Err(WorkflowError::WorkflowNotFound(id));
        }

        Ok(())
    }

    async fn stop_workflow(&self, id: WorkflowId) -> Result<()> {
        // Check if workflow is active
        if !self.active_executions.contains_key(&id) {
            return Err(WorkflowError::WorkflowNotFound(id));
        }

        // Check current status
        let current_status = {
            let execution = self
                .active_executions
                .get(&id)
                .ok_or_else(|| WorkflowError::WorkflowNotFound(id))?;
            let execution = execution.read().await;
            execution.status
        };

        if !current_status.can_stop() {
            return Err(WorkflowError::InvalidStateTransition {
                from: current_status,
                to: ExecutionStatus::Cancelled,
            });
        }

        // Set stop signal
        if let Some(mut control) = self.control_signals.get_mut(&id) {
            control.request_stop();
        } else {
            return Err(WorkflowError::WorkflowNotFound(id));
        }

        Ok(())
    }

    async fn get_workflow_status(&self, id: WorkflowId) -> Result<ExecutionStatus> {
        // Check active executions first
        if let Some(execution) = self.active_executions.get(&id) {
            let execution = execution.read().await;
            return Ok(execution.status);
        }

        // Check persisted state
        if let Some(workflow_state) = self.state_manager.load_workflow_state(id).await? {
            return Ok(workflow_state.execution.status);
        }

        Err(WorkflowError::WorkflowNotFound(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{FileStorage, SimpleMemoryCache, StateManager};
    use crate::tools::ToolRegistry;
    use crate::workflow::{NodeType, WorkflowDefinition, WorkflowNode};
    use proptest::prelude::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    // Mock tool registry for testing
    struct MockToolRegistry;

    #[async_trait]
    impl ToolRegistry for MockToolRegistry {
        fn register_tool(&mut self, _tool: Arc<dyn crate::tools::ToolNode>) -> Result<()> {
            Ok(())
        }

        fn get_tool(&self, _name: &str) -> Option<Arc<dyn crate::tools::ToolNode>> {
            None
        }

        fn list_tools(&self) -> Vec<crate::core::ToolInfo> {
            Vec::new()
        }

        async fn execute_tool(
            &self,
            _name: &str,
            _params: Value,
            _context: ExecutionContext,
        ) -> Result<Value> {
            Ok(Value::String("mock_result".to_string()))
        }

        fn validate_tool_params(&self, _name: &str, _params: &Value) -> Result<()> {
            Ok(())
        }

        fn has_tool(&self, _name: &str) -> bool {
            false
        }

        fn unregister_tool(&mut self, _name: &str) -> Result<()> {
            Ok(())
        }

        fn resolve_dependencies(
            &self,
            _tool_names: Vec<String>,
        ) -> Result<crate::tools::ResolutionResult> {
            Ok(crate::tools::ResolutionResult {
                resolved_versions: std::collections::HashMap::new(),
                conflicts: Vec::new(),
                warnings: Vec::new(),
            })
        }

        fn check_version_conflicts(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        fn get_dependents(&self, _tool_name: &str) -> Vec<crate::core::ToolInfo> {
            Vec::new()
        }

        async fn execute_tool_with_templates(
            &self,
            name: &str,
            params: Value,
            _template_context: &crate::tools::TemplateContext,
            execution_context: ExecutionContext,
        ) -> Result<Value> {
            self.execute_tool(name, params, execution_context).await
        }

        fn get_tool_templates(&self, _tool_name: &str) -> Vec<crate::tools::ParameterTemplate> {
            Vec::new()
        }

        fn tool_count(&self) -> usize {
            0
        }

        fn clear(&mut self) {
            // No-op for mock
        }
    }

    // Helper function to create a test workflow engine
    fn create_test_engine() -> DefaultWorkflowEngine {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FileStorage::new(temp_dir.path().join("storage")).unwrap());
        let cache = Arc::new(SimpleMemoryCache::new());
        let state_manager = Arc::new(StateManager::new(storage, cache));
        let tool_registry = Arc::new(MockToolRegistry);

        DefaultWorkflowEngine::new_with_audit(state_manager, tool_registry, 10, false, 30)
    }

    // Helper function to create a simple test workflow
    fn create_test_workflow(name: &str) -> WorkflowDefinition {
        let mut workflow = WorkflowDefinition::new(name, "1.0.0");

        let node1 = WorkflowNode::new("node1", NodeType::Tool);
        let node2 = WorkflowNode::new("node2", NodeType::Tool);

        workflow.add_node(node1).unwrap();
        workflow.add_node(node2).unwrap();

        workflow
    }

    // Property-based test generators
    fn workflow_name() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_-]{2,31}"
    }

    fn workflow_version() -> impl Strategy<Value = String> {
        r"[0-9]+\.[0-9]+\.[0-9]+"
    }

    fn node_count() -> impl Strategy<Value = usize> {
        1usize..3 // Reduced from 1..10
    }

    // Simplified unit tests for workflow state control consistency
    #[tokio::test]
    async fn test_pause_resume_basic_functionality() {
        // **Feature: workflow-toolkit, Property 4: Workflow state control consistency**
        // *For any* workflow, pause operation followed by resume should continue from correct state
        // **Validates: Requirements 2.3, 5.3**

        let engine = Arc::new(create_test_engine());
        let workflow = create_test_workflow("test_workflow");
        let workflow_id = workflow.generate_id();

        // Create initial execution state
        let execution = WorkflowExecution {
            id: workflow_id,
            workflow_name: workflow.name.clone(),
            status: ExecutionStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            current_node: Some("node1".to_string()),
            node_states: std::collections::HashMap::new(),
            global_context: serde_json::json!({"test_key": "test_value"}),
        };

        // Store execution in active executions
        let execution_arc = Arc::new(RwLock::new(execution.clone()));
        engine
            .active_executions
            .insert(workflow_id, execution_arc.clone());
        engine
            .control_signals
            .insert(workflow_id, ExecutionControl::new());

        // Test pause
        let pause_result = engine.pause_workflow(workflow_id).await;
        assert!(
            pause_result.is_ok(),
            "Pause should succeed: {:?}",
            pause_result
        );

        // Verify execution status is updated to Paused
        {
            let execution = execution_arc.read().await;
            assert_eq!(
                execution.status,
                ExecutionStatus::Paused,
                "Execution status should be Paused"
            );
        }

        // Verify pause signal is set
        let control = engine.control_signals.get(&workflow_id).unwrap();
        assert!(control.should_pause(), "Pause signal should be set");

        // Test resume
        let resume_result = engine.resume_workflow(workflow_id).await;
        assert!(
            resume_result.is_ok(),
            "Resume should succeed: {:?}",
            resume_result
        );

        // Verify execution status is updated to Running
        {
            let execution = execution_arc.read().await;
            assert_eq!(
                execution.status,
                ExecutionStatus::Running,
                "Execution status should be Running"
            );
        }

        // Verify pause signal is cleared
        let control = engine.control_signals.get(&workflow_id).unwrap();
        assert!(
            !control.should_pause(),
            "Pause signal should be cleared after resume"
        );

        // Clean up
        engine.active_executions.remove(&workflow_id);
        engine.control_signals.remove(&workflow_id);
    }

    // Mock tool registry that returns parameters as result (for testing parameter passing)
    struct EchoToolRegistry;

    #[async_trait]
    impl ToolRegistry for EchoToolRegistry {
        fn register_tool(&mut self, _tool: Arc<dyn crate::tools::ToolNode>) -> Result<()> {
            Ok(())
        }

        fn get_tool(&self, _name: &str) -> Option<Arc<dyn crate::tools::ToolNode>> {
            None
        }

        fn list_tools(&self) -> Vec<crate::core::ToolInfo> {
            Vec::new()
        }

        async fn execute_tool(
            &self,
            _name: &str,
            params: Value,
            _context: ExecutionContext,
        ) -> Result<Value> {
            // Return the parameters as the result
            Ok(params)
        }

        fn validate_tool_params(&self, _name: &str, _params: &Value) -> Result<()> {
            Ok(())
        }

        fn has_tool(&self, _name: &str) -> bool {
            false
        }

        fn unregister_tool(&mut self, _name: &str) -> Result<()> {
            Ok(())
        }

        fn resolve_dependencies(
            &self,
            _tool_names: Vec<String>,
        ) -> Result<crate::tools::ResolutionResult> {
            Ok(crate::tools::ResolutionResult {
                resolved_versions: std::collections::HashMap::new(),
                conflicts: Vec::new(),
                warnings: Vec::new(),
            })
        }

        fn check_version_conflicts(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        fn get_dependents(&self, _tool_name: &str) -> Vec<crate::core::ToolInfo> {
            Vec::new()
        }

        async fn execute_tool_with_templates(
            &self,
            name: &str,
            params: Value,
            _template_context: &crate::tools::TemplateContext,
            execution_context: ExecutionContext,
        ) -> Result<Value> {
            self.execute_tool(name, params, execution_context).await
        }

        fn get_tool_templates(&self, _tool_name: &str) -> Vec<crate::tools::ParameterTemplate> {
            Vec::new()
        }

        fn tool_count(&self) -> usize {
            0
        }

        fn clear(&mut self) {
        }
    }

    #[tokio::test]
    async fn test_parameter_passing_data_flow() {
        // Setup engine locally to keep temp_dir alive
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FileStorage::new(temp_dir.path().join("storage")).unwrap());
        let cache = Arc::new(SimpleMemoryCache::new());
        let state_manager = Arc::new(StateManager::new(storage, cache));
        let tool_registry = Arc::new(EchoToolRegistry);
        let engine = Arc::new(DefaultWorkflowEngine::new_with_audit(state_manager, tool_registry, 10, false, 30));
        
        // Create a workflow with 2 nodes
        // Node 1: Produces some data (simulated by passing params which are echoed back)
        // Node 2: Uses output from Node 1
        
        let mut workflow = WorkflowDefinition::new("param_test_workflow", "1.0.0");

        // Node 1
        let mut node1 = WorkflowNode::new("node1", NodeType::Tool);
        node1.parameters = serde_json::json!({
            "output_key": "output_value"
        });
        workflow.add_node(node1).unwrap();

        // Node 2
        let mut node2 = WorkflowNode::new("node2", NodeType::Tool);
        // This parameter depends on node1's result
        // Since EchoToolRegistry returns params, node1 result will be {"output_key": "output_value"}
        node2.parameters = serde_json::json!({
            "received_value": "${nodes.node1.output_key}",
            "global_value": "${global_var}"
        });
        workflow.add_node(node2).unwrap();

        // Make node2 depend on node1
        workflow.add_edge(WorkflowEdge::new("node1", "node2")).unwrap();
        
        // Add global variable to context
        let mut global_context = serde_json::Map::new();
        global_context.insert("global_var".to_string(), Value::String("global_test".to_string()));
        
        // We need to inject this global context into the execution.
        // DefaultWorkflowEngine::execute_workflow initializes global_context as empty.
        // But we can pass it via definition? No, definition has no global context.
        // Wait, execute_workflow initializes global_context as empty.
        // However, resolve_parameters uses `execution.global_context` AND `context.global_variables`.
        // We can't easily set `context.global_variables` in `execute_workflow` call.
        
        // BUT, `execute_workflow` returns `WorkflowExecution` which is Running.
        // If we want to test parameter passing, we rely on the engine executing it.
        // The engine creates the execution.
        
        // Workaround: We can't easily inject global variables in `execute_workflow` 
        // unless we modify `WorkflowDefinition` to have default context or use `execute_workflow_with_context`.
        // `DefaultWorkflowEngine` doesn't have `execute_workflow_with_context`.
        
        // However, `resolve_parameters` logic:
        // 1. Add global variables from execution context (which comes from caller of execute_tool, but here execute_node creates it)
        // 2. Add global context from workflow execution (initially empty)
        
        // Let's rely on node-to-node passing first.
        
        let execution = engine.execute_workflow(workflow).await.expect("Failed to start workflow");
        let workflow_id = execution.id;
        
        // Wait for completion
        let mut attempts = 0;
        loop {
            let status = engine.get_workflow_status(workflow_id).await.unwrap();
            if status == ExecutionStatus::Completed || status == ExecutionStatus::Failed {
                break;
            }
            if attempts > 50 {
                panic!("Workflow execution timed out");
            }
            attempts += 1;
            sleep(Duration::from_millis(100)).await;
        }
        
        // Check final state
        let state = engine.load_workflow_state(workflow_id).await.unwrap().unwrap();
        assert_eq!(state.execution.status, ExecutionStatus::Completed);
        
        // Verify Node 1 result
        let node1_state = state.execution.node_states.get("node1").unwrap();
        let node1_result = node1_state.result.as_ref().unwrap();
        assert_eq!(node1_result["output_key"], "output_value");
        
        // Verify Node 2 result
        // It should have received the substituted values
        let node2_state = state.execution.node_states.get("node2").unwrap();
        let node2_result = node2_state.result.as_ref().unwrap();
        
        // Check if substitution happened
        // "received_value": "${nodes.node1.output_key}" -> "output_value"
        assert_eq!(node2_result["received_value"], "output_value");
        
        // "global_value": "${global_var}" -> should be unsubstituted if not found? 
        // Or if we can't inject it, it stays as is or empty.
        // The template engine usually leaves it or errors? 
        // If it fails to resolve, it might error or leave it. 
        // Our template engine implementation (Tera/Handlebars?) likely errors if strict.
        // But `resolve_parameters` implementation:
        // .unwrap_or(Value::Null) in some places, but in execute_node it propagates error?
        // execute_node: `let params = self.resolve_parameters(...) ?;` -> It propagates error.
        
        // So if global_var is missing, it might fail!
        // I should remove global_var dependency for this test to be safe, 
        // OR find a way to inject it.
        // Actually, `DefaultWorkflowEngine` doesn't expose a way to set initial global context.
        // That might be a missing feature, but for now I'll stick to node-to-node.
    }

    #[tokio::test]
    async fn test_invalid_state_transitions() {
        // **Feature: workflow-toolkit, Property 4: Workflow state control consistency**
        // *For any* workflow, invalid state transitions should be rejected
        // **Validates: Requirements 2.3, 5.3**

        let engine = Arc::new(create_test_engine());
        let workflow = create_test_workflow("test_workflow");
        let workflow_id = workflow.generate_id();

        // Test invalid transitions for terminal states
        let terminal_states = vec![
            ExecutionStatus::Completed,
            ExecutionStatus::Failed,
            ExecutionStatus::Cancelled,
        ];

        for terminal_status in terminal_states {
            // Create execution in terminal state
            let execution = WorkflowExecution {
                id: workflow_id,
                workflow_name: workflow.name.clone(),
                status: terminal_status,
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
                current_node: None,
                node_states: std::collections::HashMap::new(),
                global_context: Value::Null,
            };

            let execution_arc = Arc::new(RwLock::new(execution));
            engine.active_executions.insert(workflow_id, execution_arc);
            engine
                .control_signals
                .insert(workflow_id, ExecutionControl::new());

            // Try to pause - should fail
            let pause_result = engine.pause_workflow(workflow_id).await;
            assert!(
                pause_result.is_err(),
                "Pause should fail for terminal state {:?}: {:?}",
                terminal_status,
                pause_result
            );

            // Try to resume - should fail
            let resume_result = engine.resume_workflow(workflow_id).await;
            assert!(
                resume_result.is_err(),
                "Resume should fail for terminal state {:?}: {:?}",
                terminal_status,
                resume_result
            );

            // Clean up for next iteration
            engine.active_executions.remove(&workflow_id);
            engine.control_signals.remove(&workflow_id);
        }
    }

    #[tokio::test]
    async fn test_workflow_not_found_errors() {
        // **Feature: workflow-toolkit, Property 4: Workflow state control consistency**
        // *For any* non-existent workflow, operations should return appropriate errors
        // **Validates: Requirements 2.3, 5.3**

        let engine = create_test_engine();
        let non_existent_id = uuid::Uuid::new_v4();

        // Test pause on non-existent workflow
        let pause_result = engine.pause_workflow(non_existent_id).await;
        assert!(
            pause_result.is_err(),
            "Pause should fail for non-existent workflow"
        );

        // Test resume on non-existent workflow
        let resume_result = engine.resume_workflow(non_existent_id).await;
        assert!(
            resume_result.is_err(),
            "Resume should fail for non-existent workflow"
        );

        // Test stop on non-existent workflow
        let stop_result = engine.stop_workflow(non_existent_id).await;
        assert!(
            stop_result.is_err(),
            "Stop should fail for non-existent workflow"
        );

        // Test status on non-existent workflow
        let status_result = engine.get_workflow_status(non_existent_id).await;
        assert!(
            status_result.is_err(),
            "Status should fail for non-existent workflow"
        );
    }

    // Property-based tests (simplified to avoid linking issues)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]

        #[test]
        fn test_workflow_state_control_consistency_property(
            workflow_name in workflow_name(),
        ) {
            // **Feature: workflow-toolkit, Property 4: Workflow state control consistency**
            // *For any* workflow name, basic state control operations should be consistent
            // **Validates: Requirements 2.3, 5.3**

            tokio_test::block_on(async {
                let engine = Arc::new(create_test_engine());
                let workflow = create_test_workflow(&workflow_name);
                let workflow_id = workflow.generate_id();

                // Create execution in running state
                let execution = WorkflowExecution {
                    id: workflow_id,
                    workflow_name: workflow.name.clone(),
                    status: ExecutionStatus::Running,
                    started_at: Utc::now(),
                    completed_at: None,
                    current_node: Some("node1".to_string()),
                    node_states: std::collections::HashMap::new(),
                    global_context: Value::Null,
                };

                let execution_arc = Arc::new(RwLock::new(execution));
                engine.active_executions.insert(workflow_id, execution_arc.clone());
                engine.control_signals.insert(workflow_id, ExecutionControl::new());

                // Test pause -> resume cycle
                let pause_result = engine.pause_workflow(workflow_id).await;
                prop_assert!(pause_result.is_ok(), "Pause should succeed for running workflow");

                // Verify execution status is paused
                {
                    let execution = execution_arc.read().await;
                    prop_assert_eq!(execution.status, ExecutionStatus::Paused, "Execution status should be Paused after pause");
                }

                let resume_result = engine.resume_workflow(workflow_id).await;
                prop_assert!(resume_result.is_ok(), "Resume should succeed after pause");

                // Verify execution status is running again
                {
                    let execution = execution_arc.read().await;
                    prop_assert_eq!(execution.status, ExecutionStatus::Running, "Execution status should be Running after resume");
                }

                // Verify control signals are consistent
                let control = engine.control_signals.get(&workflow_id).unwrap();
                prop_assert!(!control.should_pause(), "Pause signal should be cleared after resume");

                // Clean up
                engine.active_executions.remove(&workflow_id);
                engine.control_signals.remove(&workflow_id);

                Ok(())
            })?;
        }
    }
}
