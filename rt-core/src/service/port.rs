use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::{error::Result, config::domain::ConfigItem, logger::domain::{LogLevel, LogRecord}};

/// 服务调用上下文，包含调用者信息、权限级别等
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceContext {
    /// 调用者ID（如插件名称、工具名称或用户ID）
    pub caller_id: String,
    
    /// 调用者类型
    pub caller_type: CallerType,
    
    /// 权限级别
    pub permission_level: PermissionLevel,
    
    /// 额外上下文信息
    pub extra: Value,
}

/// 调用者类型
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallerType {
    /// 系统组件
    System,
    /// 核心工具
    CoreTool,
    /// 外部插件
    Plugin,
    /// 用户直接调用
    User,
}

/// 权限级别
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PermissionLevel {
    /// 只读权限，只能读取配置和日志
    ReadOnly,
    /// 标准权限，可以使用大部分服务
    Standard,
    /// 管理员权限，可以修改配置和执行敏感操作
    Admin,
}

/// 服务调用请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// 服务名称
    pub service_name: String,
    
    /// 服务方法
    pub method: String,
    
    /// 请求参数
    pub params: Value,
    
    /// 调用上下文
    pub context: ServiceContext,
}

/// 服务调用响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceResponse {
    /// 响应状态
    pub status: ResponseStatus,
    
    /// 响应数据
    pub data: Option<Value>,
    
    /// 错误信息
    pub error: Option<String>,
    
    /// 跟踪ID，用于日志追踪
    pub trace_id: String,
}

/// 响应状态
#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// 成功
    Success,
    /// 失败
    Failure,
    /// 权限不足
    Unauthorized,
    /// 请求无效
    InvalidRequest,
}

/// 服务端口，定义公共服务调用的标准接口
#[async_trait]
pub trait ServicePort: Send + Sync {
    /// 获取服务名称
    fn name(&self) -> &str;
    
    /// 调用服务方法
    async fn call(&self, method: &str, params: &Value, context: &ServiceContext) -> Result<ServiceResponse>;
}

/// 配置服务端口，定义配置相关的服务接口
pub trait ConfigServicePort: ServicePort {
    /// 获取配置服务名称
    fn name(&self) -> &str {
        "config"
    }
}

/// 日志服务端口，定义日志相关的服务接口
pub trait LogServicePort: ServicePort {
    /// 获取日志服务名称
    fn name(&self) -> &str {
        "logger"
    }
}

/// 工具服务端口，定义工具相关的服务接口
pub trait ToolServicePort: ServicePort {
    /// 获取工具服务名称
    fn name(&self) -> &str {
        "tool"
    }
}
