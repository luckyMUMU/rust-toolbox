//! Workflow execution state and management

use crate::core::{ExecutionStatus, WorkflowId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Workflow execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: WorkflowId,
    pub workflow_name: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub current_node: Option<String>,
    pub node_states: HashMap<String, NodeExecutionState>,
    pub global_context: Value,
}

/// Node execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionState {
    pub status: ExecutionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub retry_count: u32,
}

/// Workflow state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub execution: WorkflowExecution,
    pub checkpoints: Vec<Checkpoint>,
    pub metadata: HashMap<String, Value>,
}

/// Checkpoint for workflow recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub node_id: String,
    pub state_snapshot: Value,
}

/// Execution record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub workflow_id: WorkflowId,
    pub execution_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: ExecutionStatus,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub duration: Option<chrono::Duration>,
}
