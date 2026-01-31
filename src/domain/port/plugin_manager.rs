//! 插件管理器端口

use async_trait::async_trait;
use crate::core::{PluginInfo, PluginType};
use crate::error::Result;

/// 插件管理器trait
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

/// 插件trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    
    /// 插件类型
    fn plugin_type(&self) -> PluginType;
    
    /// 初始化插件
    async fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    
    /// 关闭插件
    async fn shutdown(&mut self) -> Result<()>;
}

/// 插件配置
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub plugin_type: PluginType,
    pub enabled: bool,
    pub config: serde_json::Value,
}
