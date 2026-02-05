//! 工作流编排 - 协调领域层完成工作流执行
//!
//! 工作流编排器负责协调工作流的执行流程，
//! 将应用层的请求转换为领域层的操作。

use crate::error::Result;
use crate::workflow::{WorkflowDefinition, WorkflowEngine, WorkflowExecution};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument};

/// 工作流编排器
///
/// 负责协调工作流执行的核心组件
#[derive(Clone)]
pub struct WorkflowOrchestrator {
    workflow_engine: Arc<dyn WorkflowEngine>,
}

impl WorkflowOrchestrator {
    /// 创建编排器实例
    pub fn new(workflow_engine: Arc<dyn WorkflowEngine>) -> Self {
        Self { workflow_engine }
    }

    /// 编排并执行工作流
    #[instrument(skip(self, params), fields(workflow_name = %definition.name))]
    pub async fn orchestrate(
        &self,
        definition: WorkflowDefinition,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowExecution> {
        info!("开始编排工作流: {}", definition.name);

        // 验证工作流定义
        definition.validate()?;

        // 执行工作流
        let execution = self.workflow_engine.execute(definition, params).await?;

        info!("工作流编排完成: execution_id={}", execution.id);
        Ok(execution)
    }
}
