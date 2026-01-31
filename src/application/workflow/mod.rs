//! 工作流编排 - 协调领域层完成工作流执行

use crate::domain::port::repository::WorkflowRepository;
use crate::domain::port::tool_registry::ToolRegistry;
use std::sync::Arc;

/// 工作流编排器
pub struct WorkflowOrchestratorImpl {
    workflow_repo: Arc<dyn WorkflowRepository>,
    tool_registry: Arc<dyn ToolRegistry>,
}

impl WorkflowOrchestratorImpl {
    pub fn new(
        workflow_repo: Arc<dyn WorkflowRepository>,
        tool_registry: Arc<dyn ToolRegistry>,
    ) -> Self {
        Self {
            workflow_repo,
            tool_registry,
        }
    }
}

// TODO: 实现 WorkflowOrchestrator trait
