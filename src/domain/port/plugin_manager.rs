//! 插件管理器端口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use crate::domain::model::{PluginInfo, PluginType};
use crate::domain::port::tool_registry::ToolNode;
use crate::error::Result;

/// 插件管理器trait - 管理插件生命周期
/// 
/// 这是领域层端口，具体实现位于基础设施层
#[async_trait]
pub trait PluginManager: Send + Sync {
    /// 加载插件
    async fn load_plugin(&mut self, plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<()>;
    
    /// 卸载插件
    fn unload_plugin(&mut self, name: &str) -> Result<()>;
    
    /// 获取插件
    fn get_plugin(&self, name: &str) -> Option<&dyn Plugin>;
    
    /// 列出所有插件
    fn list_plugins(&self) -> Vec<PluginInfo>;
    
    /// 获取插件数量
    fn plugin_count(&self) -> usize;
    
    /// 检查插件是否存在
    fn has_plugin(&self, name: &str) -> bool;
}

/// 插件trait - 所有插件类型必须实现
/// 
/// 这是领域层核心trait，定义插件的标准接口
pub trait Plugin: Send + Sync {
    /// 获取插件信息
    fn info(&self) -> &PluginInfo;
    
    /// 初始化插件
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    
    /// 获取插件提供的所有工具
    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>>;
    
    /// 关闭插件
    fn shutdown(&mut self) -> Result<()>;
    
    /// 检查是否已初始化
    fn is_initialized(&self) -> bool;
    
    /// 获取插件状态
    fn status(&self) -> PluginStatus;
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub plugin_type: PluginType,
    pub enabled: bool,
    pub config: Value,
    pub security_policy: SecurityPolicy,
    pub resource_limits: ResourceLimits,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

impl PluginConfig {
    pub fn new(name: String, plugin_type: PluginType) -> Self {
        Self {
            name,
            plugin_type,
            enabled: true,
            config: Value::Null,
            security_policy: SecurityPolicy::default(),
            resource_limits: ResourceLimits::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// 安全策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub allow_network_access: bool,
    pub allow_file_system_access: bool,
    pub allowed_paths: Vec<PathBuf>,
    pub environment_variables: HashMap<String, String>,
    pub sandbox_enabled: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allow_network_access: false,
            allow_file_system_access: false,
            allowed_paths: Vec::new(),
            environment_variables: HashMap::new(),
            sandbox_enabled: true,
        }
    }
}

/// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory: Option<u64>,
    pub max_cpu_time: Option<Duration>,
    pub max_execution_time: Option<Duration>,
    pub max_file_size: Option<u64>,
    pub max_network_connections: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: Some(1024 * 1024 * 1024),               // 1GB
            max_cpu_time: Some(Duration::from_secs(300)),       // 5 minutes
            max_execution_time: Some(Duration::from_secs(600)), // 10 minutes
            max_file_size: Some(100 * 1024 * 1024),             // 100MB
            max_network_connections: Some(10),
        }
    }
}

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PluginStatus {
    Uninitialized,
    Initializing,
    Ready,
    Running,
    Error,
    ShuttingDown,
    Shutdown,
}
