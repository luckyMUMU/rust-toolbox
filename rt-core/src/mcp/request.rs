use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::mcp::context::McpContext;
use crate::service::{CallerType, PermissionLevel};

/// MCP 请求，包含工具调用信息和上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// 请求 ID
    pub id: String,
    
    /// 请求时间
    pub timestamp: DateTime<Utc>,
    
    /// 调用组件类型
    pub component_type: ComponentType,
    
    /// 调用组件名称
    pub component_name: String,
    
    /// 调用方法
    pub method: String,
    
    /// 请求参数
    pub params: Value,
    
    /// 调用上下文
    pub context: McpContext,
    
    /// 服务调用上下文
    pub service_context: McpServiceContext,
    
    /// 请求超时时间（毫秒）
    pub timeout_ms: Option<u64>,
}

/// 组件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentType {
    /// 工具组件
    Tool,
    
    /// 插件组件
    Plugin,
    
    /// 工作流组件
    Workflow,
    
    /// 服务组件
    Service,
    
    /// 系统组件
    System,
}

/// MCP 服务调用上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServiceContext {
    /// 调用者 ID（如插件名称、工具名称或用户 ID）
    pub caller_id: String,
    
    /// 调用者类型
    pub caller_type: CallerType,
    
    /// 权限级别
    pub permission_level: PermissionLevel,
    
    /// 额外上下文信息
    pub extra: Value,
}

impl McpRequest {
    /// 创建一个新的 MCP 请求
    pub fn new(
        component_type: ComponentType,
        component_name: String,
        method: String,
        params: Value,
        context: McpContext,
        service_context: McpServiceContext,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            component_type,
            component_name,
            method,
            params,
            context,
            service_context,
            timeout_ms: None,
        }
    }
    
    /// 创建一个用于工具调用的 MCP 请求
    pub fn new_tool_call(
        tool_name: String,
        params: Value,
        context: McpContext,
        service_context: McpServiceContext,
    ) -> Self {
        Self::new(
            ComponentType::Tool,
            tool_name,
            "run".to_string(),
            params,
            context,
            service_context,
        )
    }
    
    /// 创建一个用于插件调用的 MCP 请求
    pub fn new_plugin_call(
        plugin_name: String,
        params: Value,
        context: McpContext,
        service_context: McpServiceContext,
    ) -> Self {
        Self::new(
            ComponentType::Plugin,
            plugin_name,
            "run".to_string(),
            params,
            context,
            service_context,
        )
    }
    
    /// 创建一个用于工作流调用的 MCP 请求
    pub fn new_workflow_call(
        workflow_name: String,
        params: Value,
        context: McpContext,
        service_context: McpServiceContext,
    ) -> Self {
        Self::new(
            ComponentType::Workflow,
            workflow_name,
            "execute".to_string(),
            params,
            context,
            service_context,
        )
    }
    
    /// 设置请求超时时间
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}
