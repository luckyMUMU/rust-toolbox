//! 用例层 - 具体业务用例
//!
//! 用例层封装具体的业务场景，协调应用服务完成用户请求。
//! 每个用例对应一个独立的业务操作。

use crate::application::service::WorkflowService;
use crate::error::Result;
use crate::workflow::{WorkflowDefinition, WorkflowExecution};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// 执行工作流用例
///
/// 负责完整的"执行工作流"业务流程
pub struct ExecuteWorkflowUseCase {
    workflow_service: Arc<WorkflowService>,
}

impl ExecuteWorkflowUseCase {
    /// 创建用例实例
    pub fn new(workflow_service: Arc<WorkflowService>) -> Self {
        Self { workflow_service }
    }

    /// 执行工作流
    #[instrument(skip(self, params), fields(workflow_name = %definition.name))]
    pub async fn execute(
        &self,
        definition: WorkflowDefinition,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowExecution> {
        info!("用例: 执行工作流 {}", definition.name);

        // 调用应用服务执行
        let execution = self
            .workflow_service
            .execute_workflow(definition, params)
            .await?;

        info!("用例完成: 工作流执行完成, execution_id={}", execution.id);
        Ok(execution)
    }
}

/// 管理工作流用例
///
/// 负责工作流的生命周期管理
pub struct ManageWorkflowsUseCase;

impl ManageWorkflowsUseCase {
    /// 创建用例实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for ManageWorkflowsUseCase {
    fn default() -> Self {
        Self::new()
    }
}

/// 管理插件用例
///
/// 负责插件的生命周期管理
pub struct ManagePluginsUseCase;

impl ManagePluginsUseCase {
    /// 创建用例实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for ManagePluginsUseCase {
    fn default() -> Self {
        Self::new()
    }
}

/// 系统监控用例
///
/// 负责系统健康检查和监控
pub struct SystemMonitoringUseCase;

impl SystemMonitoringUseCase {
    /// 创建用例实例
    pub fn new() -> Self {
        Self
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        debug!("用例: 执行健康检查");
        Ok(HealthStatus::Healthy)
    }
}

impl Default for SystemMonitoringUseCase {
    fn default() -> Self {
        Self::new()
    }
}

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

impl HealthStatus {
    /// 是否健康
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
}
