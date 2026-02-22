//! Unified state management for workflow execution.
//!
//! This module provides a single source of truth for execution state,
//! replacing the scattered state management across DagScheduler and WorkflowExecution.

pub mod checkpoint;
pub mod checkpoint_recovery;

use crate::core::ExecutionStatus;
use crate::workflow::execution::NodeExecutionState;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Unified execution state tracker.
///
/// This is the single source of truth for all execution state,
/// eliminating the need for manual synchronization between
/// DagScheduler and WorkflowExecution.
#[derive(Debug)]
pub struct ExecutionTracker {
    /// Workflow execution ID
    workflow_id: Uuid,

    /// Workflow name
    workflow_name: String,

    /// Node execution states
    node_states: Arc<DashMap<String, NodeExecutionState>>,

    /// Workflow-level status
    workflow_status: Arc<RwLock<ExecutionStatus>>,

    /// Execution start time
    started_at: DateTime<Utc>,

    /// Execution end time (set when completed)
    completed_at: Arc<RwLock<Option<DateTime<Utc>>>>,

    /// Current node being executed
    current_node: Arc<RwLock<Option<String>>>,

    /// Control signals
    control: Arc<RwLock<ControlSignals>>,
}

/// Control signals for workflow execution.
#[derive(Debug, Clone, Default)]
pub struct ControlSignals {
    /// Whether the workflow should pause
    pub should_pause: bool,
    /// Whether the workflow should stop
    pub should_stop: bool,
    /// When pause was requested
    pub pause_requested_at: Option<DateTime<Utc>>,
    /// When stop was requested
    pub stop_requested_at: Option<DateTime<Utc>>,
}

impl ControlSignals {
    /// Request a pause
    pub fn request_pause(&mut self) {
        self.should_pause = true;
        self.pause_requested_at = Some(Utc::now());
    }

    /// Request a stop
    pub fn request_stop(&mut self) {
        self.should_stop = true;
        self.stop_requested_at = Some(Utc::now());
    }

    /// Clear the pause request
    pub fn clear_pause(&mut self) {
        self.should_pause = false;
        self.pause_requested_at = None;
    }
}

impl ExecutionTracker {
    /// Create a new execution tracker.
    pub fn new(workflow_id: Uuid, workflow_name: impl Into<String>) -> Self {
        Self {
            workflow_id,
            workflow_name: workflow_name.into(),
            node_states: Arc::new(DashMap::new()),
            workflow_status: Arc::new(RwLock::new(ExecutionStatus::Pending)),
            started_at: Utc::now(),
            completed_at: Arc::new(RwLock::new(None)),
            current_node: Arc::new(RwLock::new(None)),
            control: Arc::new(RwLock::new(ControlSignals::default())),
        }
    }

    /// Get the workflow ID
    pub fn workflow_id(&self) -> Uuid {
        self.workflow_id
    }

    /// Get the workflow name
    pub fn workflow_name(&self) -> &str {
        &self.workflow_name
    }

    /// Get the start time
    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    // ==================== Status Management ====================

    /// Get the current workflow status
    pub async fn status(&self) -> ExecutionStatus {
        *self.workflow_status.read().await
    }

    /// Set the workflow status
    pub async fn set_status(&self, status: ExecutionStatus) {
        *self.workflow_status.write().await = status;

        if status.is_terminal() {
            *self.completed_at.write().await = Some(Utc::now());
        }
    }

    /// Mark the workflow as running
    pub async fn mark_running(&self) {
        self.set_status(ExecutionStatus::Running).await;
    }

    /// Mark the workflow as completed
    pub async fn mark_completed(&self) {
        self.set_status(ExecutionStatus::Completed).await;
    }

    /// Mark the workflow as failed
    pub async fn mark_failed(&self) {
        self.set_status(ExecutionStatus::Failed).await;
    }

    /// Mark the workflow as paused
    pub async fn mark_paused(&self) {
        self.set_status(ExecutionStatus::Paused).await;
    }

    // ==================== Node State Management ====================

    /// Initialize node states for all nodes in the workflow
    pub fn initialize_nodes(&self, node_ids: impl IntoIterator<Item = impl Into<String>>) {
        for node_id in node_ids {
            self.node_states.insert(
                node_id.into(),
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
    }

    /// Mark a node as started
    pub async fn mark_node_started(&self, node_id: &str) {
        *self.current_node.write().await = Some(node_id.to_string());

        if let Some(mut state) = self.node_states.get_mut(node_id) {
            state.status = ExecutionStatus::Running;
            state.started_at = Some(Utc::now());
        } else {
            self.node_states.insert(
                node_id.to_string(),
                NodeExecutionState {
                    status: ExecutionStatus::Running,
                    started_at: Some(Utc::now()),
                    completed_at: None,
                    result: None,
                    error: None,
                    retry_count: 0,
                },
            );
        }
    }

    /// Mark a node as completed
    pub fn mark_node_completed(&self, node_id: &str, result: Option<Value>) {
        if let Some(mut state) = self.node_states.get_mut(node_id) {
            state.status = ExecutionStatus::Completed;
            state.completed_at = Some(Utc::now());
            state.result = result;
        }
    }

    /// Mark a node as failed
    pub fn mark_node_failed(&self, node_id: &str, error: impl Into<String>) {
        if let Some(mut state) = self.node_states.get_mut(node_id) {
            state.status = ExecutionStatus::Failed;
            state.completed_at = Some(Utc::now());
            state.error = Some(error.into());
        }
    }

    /// Mark a node as skipped
    pub fn mark_node_skipped(&self, node_id: &str) {
        if let Some(mut state) = self.node_states.get_mut(node_id) {
            state.status = ExecutionStatus::Completed; // Skipped is a form of completion
            state.completed_at = Some(Utc::now());
        }
    }

    /// Increment the retry count for a node
    pub fn increment_retry(&self, node_id: &str) {
        if let Some(mut state) = self.node_states.get_mut(node_id) {
            state.retry_count += 1;
        }
    }

    /// Get the state of a specific node
    pub fn get_node_state(&self, node_id: &str) -> Option<NodeExecutionState> {
        self.node_states.get(node_id).map(|s| s.clone())
    }

    /// Set the state of a specific node
    pub fn set_node_state(&self, node_id: &str, state: NodeExecutionState) {
        self.node_states.insert(node_id.to_string(), state);
    }

    /// Get all node states
    pub fn get_all_node_states(&self) -> std::collections::HashMap<String, NodeExecutionState> {
        self.node_states
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Check if a node has completed (successfully or failed)
    pub fn is_node_completed(&self, node_id: &str) -> bool {
        self.node_states
            .get(node_id)
            .map(|s| s.status.is_terminal())
            .unwrap_or(false)
    }

    /// Check if a node has succeeded
    pub fn is_node_succeeded(&self, node_id: &str) -> bool {
        self.node_states
            .get(node_id)
            .map(|s| s.status == ExecutionStatus::Completed)
            .unwrap_or(false)
    }

    /// Get all completed node IDs
    pub fn get_completed_nodes(&self) -> Vec<String> {
        self.node_states
            .iter()
            .filter(|entry| entry.value().status.is_terminal())
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get all pending node IDs
    pub fn get_pending_nodes(&self) -> Vec<String> {
        self.node_states
            .iter()
            .filter(|entry| entry.value().status == ExecutionStatus::Pending)
            .map(|entry| entry.key().clone())
            .collect()
    }

    // ==================== Control Signals ====================

    /// Request a pause
    pub async fn request_pause(&self) {
        self.control.write().await.request_pause();
    }

    /// Request a stop
    pub async fn request_stop(&self) {
        self.control.write().await.request_stop();
    }

    /// Resume from pause
    pub async fn resume(&self) {
        self.control.write().await.clear_pause();
    }

    /// Check if pause is requested
    pub async fn should_pause(&self) -> bool {
        self.control.read().await.should_pause
    }

    /// Check if stop is requested
    pub async fn should_stop(&self) -> bool {
        self.control.read().await.should_stop
    }

    // ==================== Statistics ====================

    /// Get execution statistics
    pub fn get_stats(&self) -> ExecutionStats {
        let mut total = 0;
        let mut completed = 0;
        let mut failed = 0;
        let mut pending = 0;
        let mut running = 0;

        for entry in self.node_states.iter() {
            total += 1;
            match entry.value().status {
                ExecutionStatus::Completed => completed += 1,
                ExecutionStatus::Failed => failed += 1,
                ExecutionStatus::Pending => pending += 1,
                ExecutionStatus::Running => running += 1,
                _ => {}
            }
        }

        ExecutionStats {
            total_nodes: total,
            completed_nodes: completed,
            failed_nodes: failed,
            pending_nodes: pending,
            running_nodes: running,
        }
    }
}

/// Execution statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_nodes: usize,
    pub completed_nodes: usize,
    pub failed_nodes: usize,
    pub pending_nodes: usize,
    pub running_nodes: usize,
}

impl ExecutionStats {
    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_nodes == 0 {
            100.0
        } else {
            (self.completed_nodes as f64 / self.total_nodes as f64) * 100.0
        }
    }

    /// Check if all nodes are complete
    pub fn is_complete(&self) -> bool {
        self.pending_nodes == 0 && self.running_nodes == 0
    }

    /// Check if any node has failed
    pub fn has_failures(&self) -> bool {
        self.failed_nodes > 0
    }
}

// Re-export checkpoint types
pub use checkpoint::CheckpointManager;
pub use checkpoint_recovery::{
    CheckpointBuilder, CheckpointRecovery, CheckpointStatus, CheckpointType,
    CheckpointValidation, EnhancedCheckpoint, RecoveryConfig, RecoveryContext, RecoveryStrategy,
};
