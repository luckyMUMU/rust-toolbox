//! 应用服务 - 协调用例和领域逻辑
//!
//! 应用服务负责协调领域层完成具体的业务用例，
//! 处理事务边界、安全检查和跨领域逻辑。

use crate::error::Result;
use crate::workflow::{WorkflowDefinition, WorkflowEngine, WorkflowExecution};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument};

/// 工作流应用服务
///
/// 负责协调工作流相关的应用逻辑，包括：
/// - 工作流的创建、执行和管理
/// - 执行状态的监控和查询
/// - 工作流定义的验证和存储
#[derive(Clone)]
pub struct WorkflowService {
    workflow_engine: Arc<dyn WorkflowEngine>,
}

impl WorkflowService {
    /// 创建工作流服务
    pub fn new(workflow_engine: Arc<dyn WorkflowEngine>) -> Self {
        Self { workflow_engine }
    }

    /// 执行工作流
    #[instrument(skip(self, params), fields(workflow_name = %definition.name))]
    pub async fn execute_workflow(
        &self,
        definition: WorkflowDefinition,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowExecution> {
        info!("开始执行工作流: {}", definition.name);

        // 验证工作流定义
        definition.validate()?;

        // 执行工作流
        let execution = self.workflow_engine.execute(definition, params).await?;

        info!("工作流执行完成: execution_id={}", execution.id);
        Ok(execution)
    }
}

/// 插件应用服务
///
/// 负责协调插件相关的应用逻辑
#[derive(Clone)]
pub struct PluginService;

impl PluginService {
    /// 创建插件服务
    pub fn new() -> Self {
        Self
    }
}

impl Default for PluginService {
    fn default() -> Self {
        Self::new()
    }
}
