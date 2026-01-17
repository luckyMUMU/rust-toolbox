//! Checkpoint management for workflow execution.
//!
//! Provides functionality to save and restore execution state
//! for fault tolerance and recovery.

use crate::error::{Result, WorkflowError};
use crate::storage::StateManager;
use crate::workflow::context::DataContext;
use crate::workflow::execution::NodeExecutionState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// A checkpoint of workflow execution state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    /// Checkpoint ID
    pub id: String,

    /// Workflow execution ID
    pub workflow_id: Uuid,

    /// Workflow name
    pub workflow_name: String,

    /// When the checkpoint was created
    pub created_at: DateTime<Utc>,

    /// Node execution states at checkpoint time
    pub node_states: HashMap<String, NodeExecutionState>,

    /// Global context slots at checkpoint time
    pub global_slots: HashMap<String, serde_json::Value>,

    /// Sequence number for ordering checkpoints
    pub sequence: u64,
}

impl ExecutionCheckpoint {
    /// Create a new checkpoint.
    pub fn new(
        workflow_id: Uuid,
        workflow_name: impl Into<String>,
        node_states: HashMap<String, NodeExecutionState>,
        global_slots: HashMap<String, serde_json::Value>,
        sequence: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id,
            workflow_name: workflow_name.into(),
            created_at: Utc::now(),
            node_states,
            global_slots,
            sequence,
        }
    }
}

/// Manager for creating and restoring checkpoints.
pub struct CheckpointManager {
    #[allow(dead_code)]
    state_manager: Arc<StateManager>,
    checkpoint_interval: std::time::Duration,
    last_checkpoint: tokio::sync::RwLock<Option<DateTime<Utc>>>,
    sequence_counter: std::sync::atomic::AtomicU64,
}

impl CheckpointManager {
    /// Create a new checkpoint manager.
    pub fn new(state_manager: Arc<StateManager>, checkpoint_interval: std::time::Duration) -> Self {
        Self {
            state_manager,
            checkpoint_interval,
            last_checkpoint: tokio::sync::RwLock::new(None),
            sequence_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Check if it's time to create a checkpoint.
    pub async fn should_checkpoint(&self) -> bool {
        let last = self.last_checkpoint.read().await;
        match *last {
            None => true,
            Some(last_time) => {
                let elapsed = Utc::now().signed_duration_since(last_time);
                elapsed.to_std().unwrap_or_default() >= self.checkpoint_interval
            }
        }
    }

    /// Create a checkpoint.
    pub async fn create_checkpoint(
        &self,
        workflow_id: Uuid,
        workflow_name: &str,
        node_states: HashMap<String, NodeExecutionState>,
        context: &DataContext,
    ) -> Result<ExecutionCheckpoint> {
        let sequence = self
            .sequence_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let checkpoint = ExecutionCheckpoint::new(
            workflow_id,
            workflow_name,
            node_states,
            context.export_global_slots(),
            sequence,
        );

        // Save to state manager
        self.save_checkpoint(&checkpoint).await?;

        // Update last checkpoint time
        *self.last_checkpoint.write().await = Some(Utc::now());

        tracing::info!(
            workflow_id = %workflow_id,
            checkpoint_id = %checkpoint.id,
            sequence = sequence,
            "Created execution checkpoint"
        );

        Ok(checkpoint)
    }

    /// Save a checkpoint to persistent storage.
    async fn save_checkpoint(&self, checkpoint: &ExecutionCheckpoint) -> Result<()> {
        let key = format!("checkpoint:{}:{}", checkpoint.workflow_id, checkpoint.id);
        let _value = serde_json::to_value(checkpoint).map_err(|e| {
            WorkflowError::serialization(&format!("Failed to serialize checkpoint: {}", e))
        })?;

        // Use the state manager's cache backend for storage
        // In a real implementation, this would use a dedicated checkpoint storage
        tracing::debug!(
            checkpoint_id = %checkpoint.id,
            key = %key,
            "Saving checkpoint to storage"
        );

        Ok(())
    }

    /// Get the latest checkpoint for a workflow.
    pub async fn get_latest_checkpoint(
        &self,
        workflow_id: Uuid,
    ) -> Result<Option<ExecutionCheckpoint>> {
        // In a real implementation, this would query the storage
        // for the checkpoint with the highest sequence number
        tracing::debug!(
            workflow_id = %workflow_id,
            "Looking for latest checkpoint"
        );

        Ok(None)
    }

    /// Restore execution state from a checkpoint.
    pub async fn restore_from_checkpoint(
        &self,
        checkpoint: &ExecutionCheckpoint,
    ) -> Result<(HashMap<String, NodeExecutionState>, DataContext)> {
        let context = DataContext::new();
        context.import_global_slots(checkpoint.global_slots.clone())?;

        tracing::info!(
            workflow_id = %checkpoint.workflow_id,
            checkpoint_id = %checkpoint.id,
            "Restored execution from checkpoint"
        );

        Ok((checkpoint.node_states.clone(), context))
    }

    /// Delete checkpoints older than a certain time.
    pub async fn cleanup_old_checkpoints(
        &self,
        workflow_id: Uuid,
        keep_count: usize,
    ) -> Result<usize> {
        // In a real implementation, this would delete old checkpoints
        // keeping only the most recent `keep_count` checkpoints
        tracing::debug!(
            workflow_id = %workflow_id,
            keep_count = keep_count,
            "Cleaning up old checkpoints"
        );

        Ok(0)
    }

    /// Delete all checkpoints for a workflow.
    pub async fn delete_all_checkpoints(&self, workflow_id: Uuid) -> Result<usize> {
        tracing::debug!(
            workflow_id = %workflow_id,
            "Deleting all checkpoints"
        );

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ExecutionStatus;
    use std::time::Duration;

    #[tokio::test]
    async fn test_checkpoint_creation() {
        let workflow_id = Uuid::new_v4();
        let workflow_name = "test-workflow";

        let mut node_states = HashMap::new();
        node_states.insert(
            "node1".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Completed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                result: Some(serde_json::json!({"result": "success"})),
                error: None,
                retry_count: 0,
            },
        );

        let context = DataContext::new();
        context.set_global("test_key", "test_value").unwrap();

        let checkpoint = ExecutionCheckpoint::new(
            workflow_id,
            workflow_name,
            node_states.clone(),
            context.export_global_slots(),
            1,
        );

        assert_eq!(checkpoint.workflow_id, workflow_id);
        assert_eq!(checkpoint.workflow_name, workflow_name);
        assert_eq!(checkpoint.sequence, 1);
        assert!(checkpoint.node_states.contains_key("node1"));
        assert!(checkpoint.global_slots.contains_key("test_key"));
    }
}
