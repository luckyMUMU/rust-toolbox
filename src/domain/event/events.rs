//! 领域事件定义
//!
//! 提供领域事件的基础 trait 和具体事件类型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use uuid::Uuid;

/// 事件 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    /// 生成新的事件 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 事件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// 事件 ID
    pub event_id: EventId,
    /// 事件类型名称
    pub event_type: String,
    /// 事件发生时间
    pub occurred_at: DateTime<Utc>,
    /// 聚合类型
    pub aggregate_type: String,
    /// 聚合 ID
    pub aggregate_id: String,
    /// 事件版本
    pub version: u64,
    /// 因果关系 ID（触发此事件的事件）
    pub causation_id: Option<EventId>,
    /// 关联 ID（用于追踪整个流程）
    pub correlation_id: Option<String>,
}

impl EventMetadata {
    /// 创建新的元数据
    pub fn new(aggregate_type: &str, aggregate_id: &str, event_type: &str) -> Self {
        Self {
            event_id: EventId::new(),
            event_type: event_type.to_string(),
            occurred_at: Utc::now(),
            aggregate_type: aggregate_type.to_string(),
            aggregate_id: aggregate_id.to_string(),
            version: 1,
            causation_id: None,
            correlation_id: None,
        }
    }

    /// 设置因果关系 ID
    pub fn with_causation(mut self, causation_id: EventId) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    /// 设置关联 ID
    pub fn with_correlation(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }

    /// 设置版本
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }
}

/// 领域事件 trait
pub trait DomainEventTrait: Send + Sync + Debug + 'static {
    /// 获取事件元数据
    fn metadata(&self) -> &EventMetadata;

    /// 获取事件类型名称
    fn event_type() -> &'static str
    where
        Self: Sized;

    /// 转换为 Any 类型
    fn as_any(&self) -> &dyn Any;

    /// 序列化为 JSON
    fn to_json(&self) -> serde_json::Result<String>
    where
        Self: Serialize,
    {
        serde_json::to_string(self)
    }
}

/// 领域事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// 工作流创建事件
    WorkflowCreated(WorkflowCreatedEvent),
    /// 工作流启动事件
    WorkflowStarted(WorkflowStartedEvent),
    /// 工作流完成事件
    WorkflowCompleted(WorkflowCompletedEvent),
    /// 工作流失败事件
    WorkflowFailed(WorkflowFailedEvent),
    /// 节点执行开始事件
    NodeExecutionStarted(NodeExecutionStartedEvent),
    /// 节点执行完成事件
    NodeExecutionCompleted(NodeExecutionCompletedEvent),
    /// 节点执行失败事件
    NodeExecutionFailed(NodeExecutionFailedEvent),
    /// 工具注册事件
    ToolRegistered(ToolRegisteredEvent),
    /// 工具注销事件
    ToolUnregistered(ToolUnregisteredEvent),
    /// 插件加载事件
    PluginLoaded(PluginLoadedEvent),
    /// 插件卸载事件
    PluginUnloaded(PluginUnloadedEvent),
}

impl DomainEvent {
    /// 获取事件元数据
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            DomainEvent::WorkflowCreated(e) => &e.metadata,
            DomainEvent::WorkflowStarted(e) => &e.metadata,
            DomainEvent::WorkflowCompleted(e) => &e.metadata,
            DomainEvent::WorkflowFailed(e) => &e.metadata,
            DomainEvent::NodeExecutionStarted(e) => &e.metadata,
            DomainEvent::NodeExecutionCompleted(e) => &e.metadata,
            DomainEvent::NodeExecutionFailed(e) => &e.metadata,
            DomainEvent::ToolRegistered(e) => &e.metadata,
            DomainEvent::ToolUnregistered(e) => &e.metadata,
            DomainEvent::PluginLoaded(e) => &e.metadata,
            DomainEvent::PluginUnloaded(e) => &e.metadata,
        }
    }

    /// 获取事件类型名称
    pub fn event_type_name(&self) -> &str {
        match self {
            DomainEvent::WorkflowCreated(_) => "WorkflowCreated",
            DomainEvent::WorkflowStarted(_) => "WorkflowStarted",
            DomainEvent::WorkflowCompleted(_) => "WorkflowCompleted",
            DomainEvent::WorkflowFailed(_) => "WorkflowFailed",
            DomainEvent::NodeExecutionStarted(_) => "NodeExecutionStarted",
            DomainEvent::NodeExecutionCompleted(_) => "NodeExecutionCompleted",
            DomainEvent::NodeExecutionFailed(_) => "NodeExecutionFailed",
            DomainEvent::ToolRegistered(_) => "ToolRegistered",
            DomainEvent::ToolUnregistered(_) => "ToolUnregistered",
            DomainEvent::PluginLoaded(_) => "PluginLoaded",
            DomainEvent::PluginUnloaded(_) => "PluginUnloaded",
        }
    }
}

/// 工作流创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCreatedEvent {
    pub metadata: EventMetadata,
    pub workflow_id: String,
    pub workflow_name: String,
    pub workflow_version: String,
}

/// 工作流启动事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStartedEvent {
    pub metadata: EventMetadata,
    pub workflow_id: String,
    pub execution_id: String,
    pub input_params: serde_json::Value,
}

/// 工作流完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCompletedEvent {
    pub metadata: EventMetadata,
    pub workflow_id: String,
    pub execution_id: String,
    pub output: serde_json::Value,
    pub duration_ms: u64,
}

/// 工作流失败事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFailedEvent {
    pub metadata: EventMetadata,
    pub workflow_id: String,
    pub execution_id: String,
    pub error_message: String,
    pub error_code: Option<String>,
    pub failed_node: Option<String>,
}

/// 节点执行开始事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionStartedEvent {
    pub metadata: EventMetadata,
    pub workflow_id: String,
    pub execution_id: String,
    pub node_id: String,
    pub node_type: String,
}

/// 节点执行完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionCompletedEvent {
    pub metadata: EventMetadata,
    pub workflow_id: String,
    pub execution_id: String,
    pub node_id: String,
    pub output: serde_json::Value,
    pub duration_ms: u64,
}

/// 节点执行失败事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionFailedEvent {
    pub metadata: EventMetadata,
    pub workflow_id: String,
    pub execution_id: String,
    pub node_id: String,
    pub error_message: String,
    pub retry_count: u32,
}

/// 工具注册事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegisteredEvent {
    pub metadata: EventMetadata,
    pub tool_name: String,
    pub tool_version: String,
    pub tool_type: String,
}

/// 工具注销事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUnregisteredEvent {
    pub metadata: EventMetadata,
    pub tool_name: String,
}

/// 插件加载事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLoadedEvent {
    pub metadata: EventMetadata,
    pub plugin_name: String,
    pub plugin_type: String,
    pub plugin_version: String,
}

/// 插件卸载事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUnloadedEvent {
    pub metadata: EventMetadata,
    pub plugin_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_creation() {
        let id1 = EventId::new();
        let id2 = EventId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_event_metadata() {
        let metadata = EventMetadata::new("Workflow", "wf-123", "WorkflowCreated")
            .with_version(2)
            .with_correlation("corr-456");
        
        assert_eq!(metadata.aggregate_type, "Workflow");
        assert_eq!(metadata.aggregate_id, "wf-123");
        assert_eq!(metadata.event_type, "WorkflowCreated");
        assert_eq!(metadata.version, 2);
        assert_eq!(metadata.correlation_id, Some("corr-456".to_string()));
    }

    #[test]
    fn test_workflow_created_event() {
        let event = WorkflowCreatedEvent {
            metadata: EventMetadata::new("Workflow", "wf-123", "WorkflowCreated"),
            workflow_id: "wf-123".to_string(),
            workflow_name: "test-workflow".to_string(),
            workflow_version: "1.0.0".to_string(),
        };
        
        assert_eq!(event.workflow_id, "wf-123");
        assert_eq!(event.workflow_name, "test-workflow");
    }
}
