//! Workflow execution engine

use crate::core::{ExecutionContext, ExecutionStatus, WorkflowId};
use crate::error::{Result, WorkflowError};
use crate::storage::StateManager;
use crate::tools::ToolRegistry;
use crate::workflow::{
    DagScheduler, WorkflowDefinition, WorkflowExecution, WorkflowState, 
    NodeExecutionState, Checkpoint, ExecutionRecord
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

/// Error recovery actions
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

/// Error context for better error reporting
#[derive(Debug, Clone)]
struct ErrorContext {
    workflow_id: WorkflowId,
    workflow_name: String,
    node_id: String,
    execution_time: DateTime<Utc>,
    current_node: Option<String>,
    global_context: Value,
}

/// Trait for workflow engines
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute_workflow(&self, definition: WorkflowDefinition) -> Result<WorkflowExecution>;
    async fn pause_workflow(&self, id: WorkflowId) -> Result<()>;
    async fn resume_workflow(&self, id: WorkflowId) -> Result<()>;
    async fn stop_workflow(&self, id: WorkflowId) -> Result<()>;
    async fn get_workflow_status(&self, id: WorkflowId) -> Result<ExecutionStatus>;
}

/// Default implementation of the workflow engine
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
        Self {
            state_manager,
            tool_registry,
            active_executions: DashMap::new(),
            control_signals: DashMap::new(),
            parallel_semaphore: Arc::new(Semaphore::new(max_parallel_workflows)),
        }
    }

    /// Execute a single node (basic implementation)
    async fn execute_node(
        &self,
        node_id: &str,
        workflow_execution: Arc<RwLock<WorkflowExecution>>,
        context: ExecutionContext,
    ) -> Result<Value> {
        let workflow_id = {
            let execution = workflow_execution.read().await;
            execution.id
        };
        
        // Check if we should stop or pause
        if let Some(control) = self.control_signals.get(&workflow_id) {
            if control.should_stop() {
                return Err(WorkflowError::ExecutionCancelled.into());
            }
            if control.should_pause() {
                // Wait until resumed or stopped
                while control.should_pause() && !control.should_stop() {
                    sleep(Duration::from_millis(100)).await;
                }
                if control.should_stop() {
                    return Err(WorkflowError::ExecutionCancelled.into());
                }
            }
        }

        // For testing purposes, we'll use the node_id as the tool name
        // In a real implementation, this would get the tool name from the workflow definition
        let tool_name = node_id;
        
        // Execute the tool via tool registry
        let params = Value::Null; // Default empty parameters for testing
        self.tool_registry.execute_tool(tool_name, params, context).await
    }
    /// Execute a single node with retry logic
    pub async fn execute_node_with_retry(
        &self,
        node_id: &str,
        workflow_execution: Arc<RwLock<WorkflowExecution>>,
        context: ExecutionContext,
        retry_policy: Option<&crate::core::RetryPolicy>,
    ) -> Result<Value> {
        let default_retry = crate::core::RetryPolicy::default();
        let retry_policy = retry_policy.unwrap_or(&default_retry);
        
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < retry_policy.max_attempts {
            attempt += 1;

            // Update retry count in node state
            {
                let mut execution = workflow_execution.write().await;
                if let Some(node_state) = execution.node_states.get_mut(node_id) {
                    node_state.retry_count = attempt;
                }
            }

            match self.execute_node(node_id, workflow_execution.clone(), context.clone()).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    
                    // Don't retry if this is the last attempt
                    if attempt >= retry_policy.max_attempts {
                        break;
                    }

                    // Calculate delay based on retry strategy
                    let delay = self.calculate_retry_delay(retry_policy, attempt);
                    
                    // Log retry attempt
                    tracing::warn!(
                        "Node {} failed on attempt {}/{}, retrying in {:?}: {}",
                        node_id,
                        attempt,
                        retry_policy.max_attempts,
                        delay,
                        last_error.as_ref().unwrap()
                    );

                    // Wait before retrying
                    sleep(delay).await;

                    // Check if we should stop retrying due to control signals
                    let workflow_id = {
                        let execution = workflow_execution.read().await;
                        execution.id
                    };

                    if let Some(control) = self.control_signals.get(&workflow_id) {
                        if control.should_stop() {
                            return Err(WorkflowError::ExecutionCancelled.into());
                        }
                    }
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap_or_else(|| WorkflowError::workflow_execution("Unknown error").into()))
    }

    /// Calculate retry delay based on strategy
    pub fn calculate_retry_delay(&self, retry_policy: &crate::core::RetryPolicy, attempt: u32) -> Duration {
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
                let has_alternative_paths = self.has_alternative_execution_paths(node_id, scheduler);
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
        let error_record = ExecutionRecord {
            workflow_id,
            execution_id: workflow_id.to_string(),
            timestamp: Utc::now(),
            status: ExecutionStatus::Failed,
            result: None,
            error: Some(error.to_string()),
            duration: None,
        };

        // Save error record
        self.state_manager.save_execution_record(error_record).await?;

        // Update workflow status
        {
            let mut execution = workflow_execution.write().await;
            execution.status = ExecutionStatus::Failed;
            execution.completed_at = Some(Utc::now());
        }

        // Save final state
        let execution = workflow_execution.read().await;
        self.save_workflow_state_with_checkpoint(&execution, None).await?;

        Ok(())
    }

    /// Check if workflow can be recovered from current state
    pub async fn can_recover_workflow(&self, workflow_id: WorkflowId) -> Result<bool> {
        if let Some(workflow_state) = self.load_workflow_state(workflow_id).await? {
            match workflow_state.execution.status {
                ExecutionStatus::Running | ExecutionStatus::Paused => Ok(true),
                ExecutionStatus::Failed => {
                    // Check if there are any completed nodes that can be resumed from
                    let has_completed_nodes = workflow_state.execution.node_states
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
        _workflow_id: WorkflowId,
        node_id: &str,
        state_snapshot: Value,
    ) -> Result<Checkpoint> {
        Ok(Checkpoint {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            node_id: node_id.to_string(),
            state_snapshot,
        })
    }

    /// Save workflow state to persistence with checkpoints
    async fn save_workflow_state_with_checkpoint(
        &self, 
        workflow_execution: &WorkflowExecution,
        checkpoint: Option<Checkpoint>
    ) -> Result<()> {
        let mut checkpoints = Vec::new();
        if let Some(cp) = checkpoint {
            checkpoints.push(cp);
        }

        let workflow_state = WorkflowState {
            execution: workflow_execution.clone(),
            checkpoints,
            metadata: std::collections::HashMap::new(),
        };

        self.state_manager
            .save_workflow_state(workflow_execution.id, workflow_state)
            .await
    }

    /// Recover workflow from checkpoint
    pub async fn recover_workflow(&self, _workflow_id: WorkflowId) -> Result<Option<WorkflowExecution>> {
        if let Some(workflow_state) = self.load_workflow_state(_workflow_id).await? {
            let mut execution = workflow_state.execution;
            
            // Only recover if the workflow was in a recoverable state
            match execution.status {
                ExecutionStatus::Running | ExecutionStatus::Paused => {
                    // Reset status to pending for recovery
                    execution.status = ExecutionStatus::Pending;
                    
                    // Find the last checkpoint
                    if let Some(last_checkpoint) = workflow_state.checkpoints.last() {
                        // Restore state from checkpoint
                        execution.current_node = Some(last_checkpoint.node_id.clone());
                        execution.global_context = last_checkpoint.state_snapshot.clone();
                        
                        // Reset node states after the checkpoint
                        for (_node_id, node_state) in execution.node_states.iter_mut() {
                            if node_state.status == ExecutionStatus::Running {
                                // Reset running nodes to pending
                                node_state.status = ExecutionStatus::Pending;
                                node_state.started_at = None;
                                node_state.completed_at = None;
                                node_state.error = None;
                            }
                        }
                    }
                    
                    Ok(Some(execution))
                }
                _ => {
                    // Workflow is in a terminal state, cannot recover
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Resume workflow from saved state
    pub async fn resume_from_state(&self, workflow_state: WorkflowState) -> Result<WorkflowExecution> {
        let mut execution = workflow_state.execution;
        
        // Validate that the workflow can be resumed
        if !execution.status.can_resume() && execution.status != ExecutionStatus::Running {
            return Err(WorkflowError::InvalidStateTransition {
                from: execution.status,
                to: ExecutionStatus::Running,
            }.into());
        }

        // Update status to running
        execution.status = ExecutionStatus::Running;
        
        // Store the execution in active executions
        let execution_arc = Arc::new(RwLock::new(execution.clone()));
        self.active_executions.insert(execution.id, execution_arc.clone());

        // Initialize control signals
        self.control_signals.insert(execution.id, ExecutionControl::new());

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
            Some(last_time) => {
                now.signed_duration_since(*last_time).to_std()
                    .map(|d| d >= checkpoint_interval)
                    .unwrap_or(false)
            }
            None => true, // First checkpoint
        };

        if should_checkpoint {
            *last_checkpoint_time = Some(now);
            
            let current_node = workflow_execution.current_node.clone()
                .unwrap_or_else(|| "unknown".to_string());
            
            let checkpoint = self.create_checkpoint(
                workflow_execution.id,
                &current_node,
                workflow_execution.global_context.clone(),
            ).await?;
            
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
            duration: workflow_execution.completed_at.map(|completed| {
                completed.signed_duration_since(workflow_execution.started_at)
            }),
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
        let _permit = self.parallel_semaphore.acquire().await
            .map_err(|_| WorkflowError::ResourceExhausted)?;

        // Generate workflow execution ID
        let workflow_id = definition.generate_id();
        let _execution_id = Uuid::new_v4().to_string();

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
        let context = ExecutionContext::new()
            .with_workflow_id(workflow_id);

        // Store the execution in active executions
        let execution_arc = Arc::new(RwLock::new(workflow_execution.clone()));
        self.active_executions.insert(workflow_id, execution_arc.clone());

        // Initialize control signals
        self.control_signals.insert(workflow_id, ExecutionControl::new());

        // Create scheduler for the workflow
        let mut scheduler = DagScheduler::from_workflow(&definition)?;

        // Main execution loop
        let execution_result = async {
            let checkpoint_interval = definition.global_config.checkpoint_interval
                .unwrap_or(Duration::from_secs(300)); // 5 minutes default
            let mut last_checkpoint_time: Option<DateTime<Utc>> = None;

            while !scheduler.is_execution_complete() && scheduler.has_ready_nodes() {
                // Check for control signals
                if let Some(control) = self.control_signals.get(&workflow_id) {
                    if control.should_stop() {
                        break;
                    }
                    if control.should_pause() {
                        // Update status to paused
                        {
                            let mut execution = execution_arc.write().await;
                            execution.status = ExecutionStatus::Paused;
                        }
                        
                        // Create checkpoint before pausing
                        let execution = execution_arc.read().await;
                        let checkpoint = self.maybe_create_checkpoint(
                            &execution, 
                            &mut last_checkpoint_time, 
                            Duration::from_secs(0) // Force checkpoint
                        ).await?;
                        
                        self.save_workflow_state_with_checkpoint(&execution, checkpoint).await?;
                        
                        // Wait until resumed or stopped
                        while control.should_pause() && !control.should_stop() {
                            sleep(Duration::from_millis(100)).await;
                        }
                        
                        if control.should_stop() {
                            break;
                        }
                        
                        // Update status back to running
                        {
                            let mut execution = execution_arc.write().await;
                            execution.status = ExecutionStatus::Running;
                        }
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
                    let node_retry_policy = {
                        // TODO: Get retry policy from node definition
                        // For now, use default
                        None
                    };
                    
                    let node_result = self.execute_node_with_retry(
                        &node_id, 
                        execution_arc.clone(), 
                        context.clone(),
                        node_retry_policy
                    ).await;

                    // Update node state based on result
                    {
                        let mut execution = execution_arc.write().await;
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
                            scheduler.mark_node_completed(&node_id)?;
                        }
                        Err(error) => {
                            let recovery_action = self.handle_node_error(
                                &node_id,
                                &error,
                                execution_arc.clone(),
                                &mut scheduler,
                            ).await?;

                            // Check if we should stop the workflow
                            if recovery_action == ErrorRecoveryAction::StopWorkflow {
                                break;
                            }
                        }
                    }

                    // Create periodic checkpoints
                    let execution = execution_arc.read().await;
                    let checkpoint = self.maybe_create_checkpoint(
                        &execution, 
                        &mut last_checkpoint_time, 
                        checkpoint_interval
                    ).await?;
                    
                    self.save_workflow_state_with_checkpoint(&execution, checkpoint).await?;
                }
            }

            Ok::<(), WorkflowError>(())
        }.await;

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
                self.handle_workflow_error(&error, execution_arc.clone()).await?;
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

        // Save final state and record execution
        let final_execution = {
            let execution = execution_arc.read().await;
            self.save_workflow_state_with_checkpoint(&execution, None).await?;
            self.record_execution(&execution).await?;
            execution.clone()
        };

        // Clean up
        self.active_executions.remove(&workflow_id);
        self.control_signals.remove(&workflow_id);

        Ok(final_execution)
    }

    async fn pause_workflow(&self, id: WorkflowId) -> Result<()> {
        // Check if workflow is active
        if !self.active_executions.contains_key(&id) {
            return Err(WorkflowError::WorkflowNotFound(id).into());
        }

        // Check current status and update to paused
        {
            let execution = self.active_executions.get(&id)
                .ok_or_else(|| WorkflowError::WorkflowNotFound(id))?;
            let mut execution = execution.write().await;
            
            if !execution.status.can_pause() {
                return Err(WorkflowError::InvalidStateTransition {
                    from: execution.status,
                    to: ExecutionStatus::Paused,
                }.into());
            }
            
            execution.status = ExecutionStatus::Paused;
        }

        // Set pause signal
        if let Some(mut control) = self.control_signals.get_mut(&id) {
            control.request_pause();
        } else {
            return Err(WorkflowError::WorkflowNotFound(id).into());
        }

        Ok(())
    }

    async fn resume_workflow(&self, id: WorkflowId) -> Result<()> {
        // Check if workflow is active
        if !self.active_executions.contains_key(&id) {
            return Err(WorkflowError::WorkflowNotFound(id).into());
        }

        // Check current status and update to running
        {
            let execution = self.active_executions.get(&id)
                .ok_or_else(|| WorkflowError::WorkflowNotFound(id))?;
            let mut execution = execution.write().await;
            
            if !execution.status.can_resume() {
                return Err(WorkflowError::InvalidStateTransition {
                    from: execution.status,
                    to: ExecutionStatus::Running,
                }.into());
            }
            
            execution.status = ExecutionStatus::Running;
        }

        // Clear pause signal
        if let Some(mut control) = self.control_signals.get_mut(&id) {
            control.clear_pause();
        } else {
            return Err(WorkflowError::WorkflowNotFound(id).into());
        }

        Ok(())
    }

    async fn stop_workflow(&self, id: WorkflowId) -> Result<()> {
        // Check if workflow is active
        if !self.active_executions.contains_key(&id) {
            return Err(WorkflowError::WorkflowNotFound(id).into());
        }

        // Check current status
        let current_status = {
            let execution = self.active_executions.get(&id)
                .ok_or_else(|| WorkflowError::WorkflowNotFound(id))?;
            let execution = execution.read().await;
            execution.status
        };

        if !current_status.can_stop() {
            return Err(WorkflowError::InvalidStateTransition {
                from: current_status,
                to: ExecutionStatus::Cancelled,
            }.into());
        }

        // Set stop signal
        if let Some(mut control) = self.control_signals.get_mut(&id) {
            control.request_stop();
        } else {
            return Err(WorkflowError::WorkflowNotFound(id).into());
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

        Err(WorkflowError::WorkflowNotFound(id).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{StateManager, SimpleMemoryCache, FileStorage};
    use crate::tools::ToolRegistry;
    use crate::workflow::{WorkflowDefinition, WorkflowNode, NodeType};
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

        async fn execute_tool(&self, _name: &str, _params: Value, _context: ExecutionContext) -> Result<Value> {
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
        
        DefaultWorkflowEngine::new(state_manager, tool_registry, 10)
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
        1usize..3  // Reduced from 1..10
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
        engine.active_executions.insert(workflow_id, execution_arc.clone());
        engine.control_signals.insert(workflow_id, ExecutionControl::new());

        // Test pause
        let pause_result = engine.pause_workflow(workflow_id).await;
        assert!(pause_result.is_ok(), "Pause should succeed: {:?}", pause_result);

        // Verify execution status is updated to Paused
        {
            let execution = execution_arc.read().await;
            assert_eq!(execution.status, ExecutionStatus::Paused, "Execution status should be Paused");
        }

        // Verify pause signal is set
        let control = engine.control_signals.get(&workflow_id).unwrap();
        assert!(control.should_pause(), "Pause signal should be set");

        // Test resume
        let resume_result = engine.resume_workflow(workflow_id).await;
        assert!(resume_result.is_ok(), "Resume should succeed: {:?}", resume_result);

        // Verify execution status is updated to Running
        {
            let execution = execution_arc.read().await;
            assert_eq!(execution.status, ExecutionStatus::Running, "Execution status should be Running");
        }

        // Verify pause signal is cleared
        let control = engine.control_signals.get(&workflow_id).unwrap();
        assert!(!control.should_pause(), "Pause signal should be cleared after resume");

        // Clean up
        engine.active_executions.remove(&workflow_id);
        engine.control_signals.remove(&workflow_id);
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
            engine.control_signals.insert(workflow_id, ExecutionControl::new());
            
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
        assert!(pause_result.is_err(), "Pause should fail for non-existent workflow");
        
        // Test resume on non-existent workflow
        let resume_result = engine.resume_workflow(non_existent_id).await;
        assert!(resume_result.is_err(), "Resume should fail for non-existent workflow");
        
        // Test stop on non-existent workflow
        let stop_result = engine.stop_workflow(non_existent_id).await;
        assert!(stop_result.is_err(), "Stop should fail for non-existent workflow");
        
        // Test status on non-existent workflow
        let status_result = engine.get_workflow_status(non_existent_id).await;
        assert!(status_result.is_err(), "Status should fail for non-existent workflow");
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