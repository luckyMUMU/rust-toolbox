//! Plugin manager for loading and managing plugins
//!
//! 提供插件生命周期管理，支持异步加载和卸载

use crate::core::PluginInfo;
use crate::error::{Result, WorkflowError};
use crate::plugins::runtime::RuntimeManager;
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus};
use crate::tools::registry::ToolRegistry;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Plugin manager for loading and managing plugins.
///
/// Handles the lifecycle of plugins, including loading, unloading, configuration management,
/// and tool registration.
pub struct PluginManager {
    plugins: DashMap<String, Box<dyn Plugin>>,
    plugin_configs: DashMap<String, PluginConfig>,
    runtime_manager: RuntimeManager,
    tool_registry: Option<Arc<ToolRegistry>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
            plugin_configs: DashMap::new(),
            runtime_manager: RuntimeManager::new(),
            tool_registry: None,
        }
    }

    /// Create a new plugin manager with a tool registry
    pub fn with_tool_registry(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            plugins: DashMap::new(),
            plugin_configs: DashMap::new(),
            runtime_manager: RuntimeManager::new(),
            tool_registry: Some(tool_registry),
        }
    }

    /// Set the tool registry for plugin integration
    pub fn set_tool_registry(&mut self, tool_registry: Arc<ToolRegistry>) {
        self.tool_registry = Some(tool_registry);
        info!("Tool registry set for plugin manager");
    }

    /// Load a plugin asynchronously.
    ///
    /// This process involves:
    /// 1. Validating the configuration.
    /// 2. Creating runtime environment.
    /// 3. Initializing the plugin.
    /// 4. Registering the plugin's tools.
    pub async fn load_plugin_async(
        &self,
        mut plugin: Box<dyn Plugin>,
        config: PluginConfig,
    ) -> Result<()> {
        let plugin_name = config.name.clone();

        info!("Loading plugin asynchronously: {}", plugin_name);

        self.validate_plugin_config(&config)?;

        self.runtime_manager
            .create_runtime(&plugin_name, config.plugin_type.clone())
            .await?;

        plugin.initialize(config.clone()).map_err(|e| {
            error!("Failed to initialize plugin {}: {}", plugin_name, e);
            WorkflowError::plugin(format!(
                "Failed to initialize plugin {}: {}",
                plugin_name, e
            ))
        })?;

        self.register_plugin_tools(&plugin_name, &*plugin)?;

        self.plugins.insert(plugin_name.clone(), plugin);
        self.plugin_configs.insert(plugin_name.clone(), config);

        info!("Successfully loaded plugin: {}", plugin_name);
        Ok(())
    }

    /// Load a plugin synchronously (compatibility layer).
    ///
    /// This method wraps the async implementation for backward compatibility.
    pub fn load_plugin(&self, plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.load_plugin_async(plugin, config))
        })
    }

    /// Unload a plugin asynchronously.
    ///
    /// This removes the plugin from the manager, unregisters its tools, and performs cleanup.
    pub async fn unload_plugin_async(&self, name: &str) -> Result<()> {
        info!("Unloading plugin asynchronously: {}", name);

        self.unregister_plugin_tools(name)?;

        let (_, mut plugin) = self
            .plugins
            .remove(name)
            .ok_or_else(|| WorkflowError::not_found(format!("Plugin not found: {}", name)))?;

        plugin.shutdown().map_err(|e| {
            error!("Failed to shutdown plugin {}: {}", name, e);
            WorkflowError::plugin(format!("Failed to shutdown plugin {}: {}", name, e))
        })?;

        self.runtime_manager.cleanup_runtime(name).await?;

        self.plugin_configs.remove(name);

        info!("Successfully unloaded plugin: {}", name);
        Ok(())
    }

    /// Unload a plugin synchronously (compatibility layer).
    pub fn unload_plugin(&self, name: &str) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.unload_plugin_async(name))
        })
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.get(name).map(|p| p.info().clone())
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| p.info().clone()).collect()
    }

    /// Get plugin status
    pub fn get_plugin_status(&self, name: &str) -> Option<PluginStatus> {
        self.plugins.get(name).map(|p| p.status())
    }

    /// Check if a plugin is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get plugin count
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Reload a plugin asynchronously
    pub async fn reload_plugin_async(&self, name: &str) -> Result<()> {
        info!("Reloading plugin asynchronously: {}", name);

        let config = self
            .plugin_configs
            .get(name)
            .map(|c| c.clone())
            .ok_or_else(|| {
                WorkflowError::not_found(format!("Plugin configuration not found: {}", name))
            })?;

        if let Err(e) = self.unload_plugin_async(name).await {
            warn!("Failed to unload plugin {} during reload: {}", name, e);
        }

        warn!(
            "Plugin reload requires recreating plugin instance - not fully implemented for: {}",
            name
        );

        drop(config);
        Ok(())
    }

    /// Reload a plugin synchronously (compatibility layer)
    pub fn reload_plugin(&self, name: &str) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.reload_plugin_async(name))
        })
    }

    /// Get all tools from all loaded plugins
    pub fn get_all_tools(&self) -> Vec<crate::tools::types::Tool> {
        self.plugins.iter().flat_map(|p| p.get_tools()).collect()
    }

    /// Get tools from a specific plugin
    pub fn get_plugin_tools(&self, plugin_name: &str) -> Result<Vec<crate::tools::types::Tool>> {
        self.plugins
            .get(plugin_name)
            .map(|p| p.get_tools())
            .ok_or_else(|| WorkflowError::not_found(format!("Plugin not found: {}", plugin_name)))
    }

    /// Register tools from a plugin into the main tool registry
    fn register_plugin_tools(&self, plugin_name: &str, plugin: &dyn Plugin) -> Result<()> {
        if let Some(tool_registry) = &self.tool_registry {
            let tools = plugin.get_tools();
            info!(
                "Registering {} tools from plugin '{}' with main tool registry",
                tools.len(),
                plugin_name
            );

            for tool in tools {
                let tool_name = tool.name().to_string();
                tool_registry.register(&tool_name, tool);
                debug!(
                    "Registered tool '{}' from plugin '{}'",
                    tool_name, plugin_name
                );
            }

            info!("Completed tool registration for plugin '{}'", plugin_name);
        } else {
            debug!(
                "No tool registry available - plugin tools will only be accessible through plugin"
            );
        }

        Ok(())
    }

    /// Unregister plugin tools from the main tool registry
    fn unregister_plugin_tools(&self, plugin_name: &str) -> Result<()> {
        if let Some(tool_registry) = &self.tool_registry {
            let tools = self
                .plugins
                .get(plugin_name)
                .map(|p| p.get_tools())
                .unwrap_or_default();

            if !tools.is_empty() {
                info!(
                    "Unregistering {} tools from plugin '{}' from main tool registry",
                    tools.len(),
                    plugin_name
                );

                for tool in tools {
                    let tool_name = tool.name().to_string();
                    tool_registry.remove(&tool_name);
                    debug!(
                        "Unregistered tool '{}' from plugin '{}'",
                        tool_name, plugin_name
                    );
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

        if self.plugins.contains_key(&config.name) {
            return Err(WorkflowError::ValidationError(format!(
                "Plugin with name '{}' already exists",
                config.name
            )));
        }

        if config.security_policy.allow_file_system_access
            && config.security_policy.allowed_paths.is_empty()
        {
            warn!(
                "Plugin '{}' has file system access enabled but no allowed paths specified",
                config.name
            );
        }

        if let Some(max_memory) = config.resource_limits.max_memory {
            if max_memory == 0 {
                return Err(WorkflowError::ValidationError(
                    "Maximum memory limit cannot be zero".to_string(),
                ));
            }
        }

        debug!(
            "Plugin configuration validation passed for: {}",
            config.name
        );
        Ok(())
    }

    /// Shutdown all plugins asynchronously
    pub async fn shutdown_all_async(&self) -> Result<()> {
        info!("Shutting down all plugins asynchronously");

        let plugin_names: Vec<String> = self.plugins.iter().map(|p| p.key().clone()).collect();

        let mut errors = Vec::new();
        for name in plugin_names {
            if let Err(e) = self.unload_plugin_async(&name).await {
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

    /// Shutdown all plugins synchronously (compatibility layer)
    pub fn shutdown_all(&self) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.shutdown_all_async())
        })
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() {
            if let Err(e) = self.shutdown_all() {
                error!("Failed to shutdown plugins during drop: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::types::{ResourceLimits, SecurityPolicy};
    use std::sync::atomic::{AtomicU32, Ordering};

    fn create_test_config(name: &str) -> PluginConfig {
        PluginConfig {
            name: name.to_string(),
            plugin_type: crate::core::PluginType::Native,
            enabled: true,
            config: serde_json::Value::Null,
            security_policy: SecurityPolicy::default(),
            resource_limits: ResourceLimits::default(),
            dependencies: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugin_count(), 0);
    }

    #[test]
    fn test_is_loaded() {
        let manager = PluginManager::new();
        assert!(!manager.is_loaded("test"));
    }

    #[test]
    fn test_list_plugins_empty() {
        let manager = PluginManager::new();
        let plugins = manager.list_plugins();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let manager = PluginManager::new();
        let config = create_test_config("");
        let result = manager.validate_plugin_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_zero_memory() {
        let manager = PluginManager::new();
        let mut config = create_test_config("test");
        config.resource_limits.max_memory = Some(0);
        let result = manager.validate_plugin_config(&config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let manager = Arc::new(PluginManager::new());
        let counter = Arc::new(AtomicU32::new(0));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let manager = manager.clone();
                let counter = counter.clone();
                tokio::spawn(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    manager.plugin_count()
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }
}
