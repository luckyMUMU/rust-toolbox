//! Plugin manager for loading and managing plugins

use crate::core::PluginInfo;
use crate::error::{Result, WorkflowError};
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus};
use crate::tools::{ToolNode, ToolRegistry};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

/// Runtime manager for different plugin types
pub struct RuntimeManager {
    // Runtime management will be expanded in later tasks
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn create_runtime(&self, _plugin_type: crate::core::PluginType) -> Result<()> {
        // Runtime creation will be implemented in later tasks
        Ok(())
    }
    
    pub fn cleanup_runtime(&self, _plugin_name: &str) -> Result<()> {
        // Runtime cleanup will be implemented in later tasks
        Ok(())
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    plugin_configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
    runtime_manager: RuntimeManager,
    tool_registry: Option<Arc<RwLock<dyn ToolRegistry>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_configs: Arc::new(RwLock::new(HashMap::new())),
            runtime_manager: RuntimeManager::new(),
            tool_registry: None,
        }
    }

    /// Create a new plugin manager with a tool registry
    pub fn with_tool_registry(tool_registry: Arc<RwLock<dyn ToolRegistry>>) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_configs: Arc::new(RwLock::new(HashMap::new())),
            runtime_manager: RuntimeManager::new(),
            tool_registry: Some(tool_registry),
        }
    }

    /// Set the tool registry for plugin integration
    pub fn set_tool_registry(&mut self, tool_registry: Arc<RwLock<dyn ToolRegistry>>) {
        self.tool_registry = Some(tool_registry);
        info!("Tool registry set for plugin manager");
    }
    
    /// Load a plugin with configuration
    pub fn load_plugin(&self, mut plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<()> {
        let plugin_name = config.name.clone();
        
        info!("Loading plugin: {}", plugin_name);
        
        // Validate plugin configuration
        self.validate_plugin_config(&config)?;
        
        // Create runtime environment if needed
        self.runtime_manager.create_runtime(config.plugin_type.clone())?;
        
        // Initialize the plugin
        plugin.initialize(config.clone()).map_err(|e| {
            error!("Failed to initialize plugin {}: {}", plugin_name, e);
            WorkflowError::plugin(format!("Failed to initialize plugin {}: {}", plugin_name, e))
        })?;

        // Register plugin tools with the main tool registry
        self.register_plugin_tools(&plugin_name, &*plugin)?;
        
        // Store plugin and configuration
        {
            let mut plugins = self.plugins.write().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on plugins".to_string(),
                }
            })?;
            
            let mut configs = self.plugin_configs.write().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on plugin configs".to_string(),
                }
            })?;
            
            plugins.insert(plugin_name.clone(), plugin);
            configs.insert(plugin_name.clone(), config);
        }
        
        info!("Successfully loaded plugin: {}", plugin_name);
        Ok(())
    }
    
    /// Unload a plugin
    pub fn unload_plugin(&self, name: &str) -> Result<()> {
        info!("Unloading plugin: {}", name);

        // Unregister plugin tools from the main tool registry
        self.unregister_plugin_tools(name)?;
        
        let mut plugin = {
            let mut plugins = self.plugins.write().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on plugins".to_string(),
                }
            })?;
            
            plugins.remove(name).ok_or_else(|| {
                WorkflowError::not_found(format!("Plugin not found: {}", name))
            })?
        };
        
        // Shutdown the plugin
        plugin.shutdown().map_err(|e| {
            error!("Failed to shutdown plugin {}: {}", name, e);
            WorkflowError::plugin(format!("Failed to shutdown plugin {}: {}", name, e))
        })?;
        
        // Cleanup runtime
        self.runtime_manager.cleanup_runtime(name)?;
        
        // Remove configuration
        {
            let mut configs = self.plugin_configs.write().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on plugin configs".to_string(),
                }
            })?;
            configs.remove(name);
        }
        
        info!("Successfully unloaded plugin: {}", name);
        Ok(())
    }
    
    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Result<Option<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on plugins".to_string(),
            }
        })?;
        
        // Note: This is a simplified implementation. In a real scenario,
        // we would need to handle the conversion from Box<dyn Plugin> to Arc<dyn Plugin>
        // more carefully, possibly by storing Arc<dyn Plugin> directly.
        Ok(None)
    }
    
    /// List all loaded plugins
    pub fn list_plugins(&self) -> Result<Vec<PluginInfo>> {
        let plugins = self.plugins.read().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on plugins".to_string(),
            }
        })?;
        
        Ok(plugins.values().map(|plugin| plugin.info().clone()).collect())
    }
    
    /// Get plugin status
    pub fn get_plugin_status(&self, name: &str) -> Result<Option<PluginStatus>> {
        let plugins = self.plugins.read().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on plugins".to_string(),
            }
        })?;
        
        Ok(plugins.get(name).map(|plugin| plugin.status()))
    }
    
    /// Reload a plugin
    pub fn reload_plugin(&self, name: &str) -> Result<()> {
        info!("Reloading plugin: {}", name);
        
        // Get the current configuration
        let config = {
            let configs = self.plugin_configs.read().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire read lock on plugin configs".to_string(),
                }
            })?;
            
            configs.get(name).cloned().ok_or_else(|| {
                WorkflowError::not_found(format!("Plugin configuration not found: {}", name))
            })?
        };
        
        // Unload the plugin
        if let Err(e) = self.unload_plugin(name) {
            warn!("Failed to unload plugin {} during reload: {}", name, e);
        }
        
        // Recreate and load the plugin
        // Note: This is a simplified implementation. In practice, we would need
        // to recreate the plugin instance based on its type and configuration.
        warn!("Plugin reload not fully implemented yet for: {}", name);
        
        Ok(())
    }
    
    /// Get all tools from all loaded plugins
    pub fn get_all_tools(&self) -> Result<Vec<Arc<dyn ToolNode>>> {
        let plugins = self.plugins.read().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on plugins".to_string(),
            }
        })?;
        
        let mut all_tools = Vec::new();
        for plugin in plugins.values() {
            all_tools.extend(plugin.get_tools());
        }
        
        Ok(all_tools)
    }
    
    /// Get tools from a specific plugin
    pub fn get_plugin_tools(&self, plugin_name: &str) -> Result<Vec<Arc<dyn ToolNode>>> {
        let plugins = self.plugins.read().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on plugins".to_string(),
            }
        })?;
        
        plugins
            .get(plugin_name)
            .map(|plugin| plugin.get_tools())
            .ok_or_else(|| WorkflowError::not_found(format!("Plugin not found: {}", plugin_name)))
    }

    /// Register plugin tools with the main tool registry
    fn register_plugin_tools(&self, plugin_name: &str, plugin: &dyn Plugin) -> Result<()> {
        if let Some(tool_registry) = &self.tool_registry {
            let tools = plugin.get_tools();
            info!("Registering {} tools from plugin '{}' with main tool registry", tools.len(), plugin_name);
            
            let mut registry = tool_registry.write().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on tool registry".to_string(),
                }
            })?;
            
            for tool in tools {
                if let Err(e) = registry.register_tool(tool.clone()) {
                    error!("Failed to register tool '{}' from plugin '{}': {}", tool.name(), plugin_name, e);
                    // Continue registering other tools even if one fails
                } else {
                    debug!("Registered tool '{}' from plugin '{}'", tool.name(), plugin_name);
                }
            }
            
            info!("Completed tool registration for plugin '{}'", plugin_name);
        } else {
            debug!("No tool registry available - plugin tools will only be accessible through plugin");
        }
        
        Ok(())
    }

    /// Unregister plugin tools from the main tool registry
    fn unregister_plugin_tools(&self, plugin_name: &str) -> Result<()> {
        if let Some(tool_registry) = &self.tool_registry {
            // Get the plugin to access its tools
            let tools = {
                let plugins = self.plugins.read().map_err(|_| {
                    WorkflowError::ConcurrentAccess {
                        message: "Failed to acquire read lock on plugins".to_string(),
                    }
                })?;
                
                if let Some(plugin) = plugins.get(plugin_name) {
                    plugin.get_tools()
                } else {
                    Vec::new()
                }
            };
            
            if !tools.is_empty() {
                info!("Unregistering {} tools from plugin '{}' from main tool registry", tools.len(), plugin_name);
                
                let mut registry = tool_registry.write().map_err(|_| {
                    WorkflowError::ConcurrentAccess {
                        message: "Failed to acquire write lock on tool registry".to_string(),
                    }
                })?;
                
                for tool in tools {
                    if let Err(e) = registry.unregister_tool(tool.name()) {
                        warn!("Failed to unregister tool '{}' from plugin '{}': {}", tool.name(), plugin_name, e);
                        // Continue unregistering other tools even if one fails
                    } else {
                        debug!("Unregistered tool '{}' from plugin '{}'", tool.name(), plugin_name);
                    }
                }
                
                info!("Completed tool unregistration for plugin '{}'", plugin_name);
            }
        }
        
        Ok(())
    }
    
    /// Validate plugin configuration
    fn validate_plugin_config(&self, config: &PluginConfig) -> Result<()> {
        if config.name.is_empty() {
            return Err(WorkflowError::ValidationError(
                "Plugin name cannot be empty".to_string(),
            ));
        }
        
        // Check if plugin with same name already exists
        let plugins = self.plugins.read().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on plugins".to_string(),
            }
        })?;
        
        if plugins.contains_key(&config.name) {
            return Err(WorkflowError::ValidationError(format!(
                "Plugin with name '{}' already exists",
                config.name
            )));
        }
        
        // Validate security policy
        if config.security_policy.allow_file_system_access && config.security_policy.allowed_paths.is_empty() {
            warn!(
                "Plugin '{}' has file system access enabled but no allowed paths specified",
                config.name
            );
        }
        
        // Validate resource limits
        if let Some(max_memory) = config.resource_limits.max_memory {
            if max_memory == 0 {
                return Err(WorkflowError::ValidationError(
                    "Maximum memory limit cannot be zero".to_string(),
                ));
            }
        }
        
        debug!("Plugin configuration validation passed for: {}", config.name);
        Ok(())
    }
    
    /// Shutdown all plugins
    pub fn shutdown_all(&self) -> Result<()> {
        info!("Shutting down all plugins");
        
        let plugin_names: Vec<String> = {
            let plugins = self.plugins.read().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire read lock on plugins".to_string(),
                }
            })?;
            plugins.keys().cloned().collect()
        };
        
        let mut errors = Vec::new();
        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name) {
                error!("Failed to unload plugin {}: {}", name, e);
                errors.push(format!("Failed to unload plugin {}: {}", name, e));
            }
        }
        
        if !errors.is_empty() {
            return Err(WorkflowError::plugin(format!(
                "Failed to shutdown some plugins: {}",
                errors.join(", ")
            )));
        }
        
        info!("All plugins shut down successfully");
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        if let Err(e) = self.shutdown_all() {
            error!("Failed to shutdown plugins during drop: {}", e);
        }
    }
}