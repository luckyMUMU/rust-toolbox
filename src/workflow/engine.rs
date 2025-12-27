//! Workflow execution engine

use crate::core::{ExecutionStatus, WorkflowId};
use crate::error::Result;
use crate::workflow::{WorkflowDefinition, WorkflowExecution};
use async_trait::async_trait;

/// Trait for workflow engines
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute_workflow(&self, definition: WorkflowDefinition) -> Result<WorkflowExecution>;
    async fn pause_workflow(&self, id: WorkflowId) -> Result<()>;
    async fn resume_workflow(&self, id: WorkflowId) -> Result<()>;
    async fn stop_workflow(&self, id: WorkflowId) -> Result<()>;
    async fn get_workflow_status(&self, id: WorkflowId) -> Result<ExecutionStatus>;
}