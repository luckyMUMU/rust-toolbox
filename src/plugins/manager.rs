//! Plugin manager for loading and managing plugins

use crate::error::Result;
use crate::plugins::PluginType;
use std::collections::HashMap;

/// Plugin manager
pub struct PluginManager {
    plugins: HashMap<String, PluginType>,
    plugin_configs: HashMap<String, serde_json::Value>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_configs: HashMap::new(),
        }
    }
    
    pub fn load_plugin(&mut self, _plugin: PluginType) -> Result<()> {
        // Implementation will be added later
        Ok(())
    }
    
    pub fn get_plugin(&self, name: &str) -> Option<&PluginType> {
        self.plugins.get(name)
    }
    
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}