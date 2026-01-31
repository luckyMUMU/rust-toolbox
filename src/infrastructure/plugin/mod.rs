//! 插件基础设施实现

use crate::domain::model::{PluginInfo, PluginType};
use crate::domain::port::plugin_manager::{Plugin, PluginConfig, PluginManager, PluginStatus};
use crate::error::{Result, WorkflowError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 插件注册表实现
pub struct PluginRegistryImpl {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
}

impl PluginRegistryImpl {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for PluginRegistryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager for PluginRegistryImpl {
    fn load_plugin(&mut self, plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<()> {
        let name = plugin.info().name.clone();

        // 初始化插件
        let mut plugin = plugin;
        plugin.initialize(config.clone())?;

        // 存储插件
        self.plugins
            .write()
            .map_err(|_| WorkflowError::plugin("Plugin registry lock poisoned"))?
            .insert(name.clone(), plugin);

        // 存储配置
        self.configs
            .write()
            .map_err(|_| WorkflowError::plugin("Plugin config lock poisoned"))?
            .insert(name, config);

        Ok(())
    }

    fn unload_plugin(&mut self, name: &str) -> Result<()> {
        // 获取插件并关闭
        if let Some(mut plugin) = self
            .plugins
            .write()
            .map_err(|_| WorkflowError::plugin("Plugin registry lock poisoned"))?
            .remove(name)
        {
            plugin.shutdown()?;
        }

        // 移除配置
        self.configs
            .write()
            .map_err(|_| WorkflowError::plugin("Plugin config lock poisoned"))?
            .remove(name);

        Ok(())
    }

    fn get_plugin(&self, _name: &str) -> Option<&dyn Plugin> {
        // 由于RwLock的限制，这里返回self的引用
        // 实际实现可能需要Arc<RwLock<dyn Plugin>>
        None
    }

    fn list_plugins(&self) -> Vec<PluginInfo> {
        if let Ok(plugins) = self.plugins.read() {
            plugins.values().map(|p| p.info().clone()).collect()
        } else {
            Vec::new()
        }
    }

    fn plugin_count(&self) -> usize {
        if let Ok(plugins) = self.plugins.read() {
            plugins.len()
        } else {
            0
        }
    }

    fn has_plugin(&self, name: &str) -> bool {
        if let Ok(plugins) = self.plugins.read() {
            plugins.contains_key(name)
        } else {
            false
        }
    }
}

/// 运行时管理器
pub struct RuntimeManager;

impl RuntimeManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}
