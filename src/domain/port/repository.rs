//! 仓储端口 - 数据持久化接口

use crate::error::Result;
use async_trait::async_trait;

/// 工作流仓储
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    /// 保存工作流定义
    async fn save(&self, id: &str, workflow: serde_json::Value) -> Result<()>;

    /// 加载工作流定义
    async fn load(&self, id: &str) -> Result<Option<serde_json::Value>>;

    /// 删除工作流
    async fn delete(&self, id: &str) -> Result<()>;

    /// 列出所有工作流ID
    async fn list(&self) -> Result<Vec<String>>;
}

/// 执行记录仓储
#[async_trait]
pub trait ExecutionRepository: Send + Sync {
    /// 保存执行状态
    async fn save_execution(&self, execution_id: &str, state: serde_json::Value) -> Result<()>;

    /// 加载执行状态
    async fn load_execution(&self, execution_id: &str) -> Result<Option<serde_json::Value>>;

    /// 保存执行历史
    async fn save_history(&self, workflow_id: &str, record: serde_json::Value) -> Result<()>;

    /// 加载执行历史
    async fn load_history(&self, workflow_id: &str) -> Result<Vec<serde_json::Value>>;
}

/// 插件元数据仓储
#[async_trait]
pub trait PluginRepository: Send + Sync {
    /// 保存插件元数据
    async fn save_metadata(&self, plugin_id: &str, metadata: serde_json::Value) -> Result<()>;

    /// 加载插件元数据
    async fn load_metadata(&self, plugin_id: &str) -> Result<Option<serde_json::Value>>;

    /// 列出所有插件元数据
    async fn list_metadata(&self) -> Result<Vec<serde_json::Value>>;
}
