use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::mcp::context::McpContext;

/// MCP 响应，包含工具执行结果和更新后的上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// 响应 ID
    pub id: String,
    
    /// 对应的请求 ID
    pub request_id: String,
    
    /// 响应时间
    pub timestamp: DateTime<Utc>,
    
    /// 响应状态
    pub status: ResponseStatus,
    
    /// 响应数据
    pub data: Option<Value>,
    
    /// 错误信息
    pub error: Option<String>,
    
    /// 错误代码
    pub error_code: Option<String>,
    
    /// 更新后的上下文
    pub context: McpContext,
    
    /// 执行耗时（毫秒）
    pub duration_ms: Option<u64>,
    
    /// 跟踪 ID，用于日志追踪
    pub trace_id: String,
}

/// 响应状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStatus {
    /// 成功
    Success,
    
    /// 失败
    Failure,
    
    /// 权限不足
    Unauthorized,
    
    /// 请求无效
    InvalidRequest,
    
    /// 超时
    Timeout,
    
    /// 内部错误
    InternalError,
    
    /// 组件未找到
    ComponentNotFound,
    
    /// 方法未找到
    MethodNotFound,
}

impl McpResponse {
    /// 创建一个新的 MCP 响应
    pub fn new(
        request_id: String,
        status: ResponseStatus,
        data: Option<Value>,
        error: Option<String>,
        error_code: Option<String>,
        context: McpContext,
        duration_ms: Option<u64>,
        trace_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            request_id,
            timestamp: Utc::now(),
            status,
            data,
            error,
            error_code,
            context,
            duration_ms,
            trace_id,
        }
    }
    
    /// 从请求创建成功响应
    pub fn success_from_request(
        request: &crate::mcp::request::McpRequest,
        data: Value,
        context: McpContext,
        duration_ms: Option<u64>,
    ) -> Self {
        Self::new(
            request.id.clone(),
            ResponseStatus::Success,
            Some(data),
            None,
            None,
            context,
            duration_ms,
            Uuid::new_v4().to_string(),
        )
    }
    
    /// 从请求创建失败响应
    pub fn failure_from_request(
        request: &crate::mcp::request::McpRequest,
        error: String,
        error_code: Option<String>,
        context: McpContext,
        duration_ms: Option<u64>,
    ) -> Self {
        Self::new(
            request.id.clone(),
            ResponseStatus::Failure,
            None,
            Some(error),
            error_code,
            context,
            duration_ms,
            Uuid::new_v4().to_string(),
        )
    }
    
    /// 从请求创建内部错误响应
    pub fn internal_error_from_request(
        request: &crate::mcp::request::McpRequest,
        error: String,
        context: McpContext,
    ) -> Self {
        Self::new(
            request.id.clone(),
            ResponseStatus::InternalError,
            None,
            Some(error),
            Some("INTERNAL_ERROR".to_string()),
            context,
            None,
            Uuid::new_v4().to_string(),
        )
    }
    
    /// 从请求创建组件未找到响应
    pub fn component_not_found_from_request(
        request: &crate::mcp::request::McpRequest,
        component_name: &str,
        context: McpContext,
    ) -> Self {
        Self::new(
            request.id.clone(),
            ResponseStatus::ComponentNotFound,
            None,
            Some(format!("Component '{}' not found", component_name)),
            Some("COMPONENT_NOT_FOUND".to_string()),
            context,
            None,
            Uuid::new_v4().to_string(),
        )
    }
    
    /// 从请求创建方法未找到响应
    pub fn method_not_found_from_request(
        request: &crate::mcp::request::McpRequest,
        method_name: &str,
        context: McpContext,
    ) -> Self {
        Self::new(
            request.id.clone(),
            ResponseStatus::MethodNotFound,
            None,
            Some(format!("Method '{}' not found", method_name)),
            Some("METHOD_NOT_FOUND".to_string()),
            context,
            None,
            Uuid::new_v4().to_string(),
        )
    }
    
    /// 检查响应是否成功
    pub fn is_success(&self) -> bool {
        self.status == ResponseStatus::Success
    }
    
    /// 检查响应是否失败
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}
