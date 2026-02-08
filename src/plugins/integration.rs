//! Plugin integration utilities for workflow-toolkit

use crate::core::PluginInfo;
use crate::error::{Result, WorkflowError};
use crate::plugins::manager::PluginManager;
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus};
use crate::tools::registry::ToolRegistry;
use crate::tools::types::{Tool, ToolId};
use std::sync::{Arc, RwLock};
use tracing::info;

/// Integrated plugin and tool management system
///
/// This system ensures that plugin tools are properly registered with the main
/// workflow-toolkit tool registry, satisfying requirements 7.1 and 7.2.
pub struct IntegratedPluginSystem {
    plugin_manager: PluginManager,
    tool_registry: Arc<RwLock<ToolRegistry>>,
}

impl IntegratedPluginSystem {
    /// Create a new integrated plugin system
    pub fn new() -> Self {
        let tool_registry = Arc::new(RwLock::new(ToolRegistry::new()));
        let plugin_manager = PluginManager::with_tool_registry(tool_registry.clone());

        Self {
            plugin_manager,
            tool_registry,
        }
    }

    /// Create a new integrated plugin system with existing tool registry
    pub fn with_tool_registry(tool_registry: Arc<RwLock<ToolRegistry>>) -> Self {
        let plugin_manager = PluginManager::with_tool_registry(tool_registry.clone());

        Self {
            plugin_manager,
            tool_registry,
        }
    }

    /// Load a plugin and register its tools with the main tool registry
    ///
    /// This method ensures requirement 7.1 is satisfied by registering all plugin tools
    /// with the workflow-toolkit tool registry.
    pub fn load_plugin(&mut self, plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<()> {
        info!(
            "Loading plugin '{}' with integrated tool registration",
            config.name
        );

        // Load the plugin through the plugin manager
        // The plugin manager will automatically register tools with the tool registry
        self.plugin_manager.load_plugin(plugin, config)?;

        info!("Plugin loaded and tools registered successfully");
        Ok(())
    }

    /// Unload a plugin and unregister its tools from the main tool registry
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        info!(
            "Unloading plugin '{}' with integrated tool unregistration",
            name
        );

        // Unload the plugin through the plugin manager
        // The plugin manager will automatically unregister tools from the tool registry
        self.plugin_manager.unload_plugin(name)?;

        info!("Plugin unloaded and tools unregistered successfully");
        Ok(())
    }

    /// Get the plugin manager
    pub fn plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }

    /// Get the tool registry
    pub fn tool_registry(&self) -> Arc<RwLock<ToolRegistry>> {
        self.tool_registry.clone()
    }

    /// List all available tools (from all plugins and direct registrations)
    pub fn list_all_tools(&self) -> Result<Vec<crate::core::ToolInfo>> {
        let registry = self
            .tool_registry
            .read()
            .map_err(|_| WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on tool registry".to_string(),
            })?;

        Ok(registry.list_tools())
    }

    /// Execute a tool by name (satisfies requirement 7.2 - standard parameter system)
    pub async fn execute_tool(
        &self,
        name: &str,
        params: serde_json::Value,
        context: crate::core::ExecutionContext,
    ) -> Result<serde_json::Value> {
        // Get tool first to avoid holding lock across await
        let tool = {
            let registry = self
                .tool_registry
                .read()
                .map_err(|_| WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire read lock on tool registry".to_string(),
                })?;
            
            registry.get_tool(name).ok_or_else(|| WorkflowError::NotFound {
                resource: format!("tool '{}'", name),
            })?
        };

        let input = crate::tools::types::ToolInput::new(params);
        let output = tool.execute(input, context).await?;
        Ok(output.result)
    }

    /// Validate tool parameters (satisfies requirement 7.2 - standard parameter system)
    pub fn validate_tool_params(&self, name: &str, params: &serde_json::Value) -> Result<()> {
        let registry = self
            .tool_registry
            .read()
            .map_err(|_| WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on tool registry".to_string(),
            })?;

        registry.validate_tool_params(name, params)
    }

    /// Get plugin status
    pub fn get_plugin_status(&self, name: &str) -> Result<Option<PluginStatus>> {
        self.plugin_manager.get_plugin_status(name)
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Result<Vec<PluginInfo>> {
        self.plugin_manager.list_plugins()
    }

    /// Get tool count from the integrated registry
    pub fn tool_count(&self) -> Result<usize> {
        let registry = self
            .tool_registry
            .read()
            .map_err(|_| WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on tool registry".to_string(),
            })?;

        Ok(registry.tool_count())
    }

    /// Check if a tool exists in the integrated registry
    pub fn has_tool(&self, name: &str) -> Result<bool> {
        let registry = self
            .tool_registry
            .read()
            .map_err(|_| WorkflowError::ConcurrentAccess {
                message: "Failed to acquire read lock on tool registry".to_string(),
            })?;

        Ok(registry.has_tool(name))
    }

    /// Shutdown all plugins and clear the tool registry
    pub fn shutdown_all(&mut self) -> Result<()> {
        info!("Shutting down integrated plugin system");

        // Shutdown all plugins (this will unregister their tools)
        self.plugin_manager.shutdown_all()?;

        // Clear any remaining tools from the registry
        {
            let mut registry =
                self.tool_registry
                    .write()
                    .map_err(|_| WorkflowError::ConcurrentAccess {
                        message: "Failed to acquire write lock on tool registry".to_string(),
                    })?;
            registry.clear();
        }

        info!("Integrated plugin system shut down successfully");
        Ok(())
    }

    /// Register a tool directly with the tool registry (for non-plugin tools)
    pub fn register_tool(&mut self, name: &str, tool: Tool) -> Result<ToolId> {
        let registry =
            self.tool_registry
                .write()
                .map_err(|_| WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on tool registry".to_string(),
                })?;

        let id = registry.register(name, tool);
        Ok(id)
    }

    /// Unregister a tool directly from the tool registry
    pub fn unregister_tool(&mut self, name: &str) -> Result<Option<Tool>> {
        let registry =
            self.tool_registry
                .write()
                .map_err(|_| WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on tool registry".to_string(),
                })?;

        Ok(registry.remove(name))
    }
}

impl Default for IntegratedPluginSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for IntegratedPluginSystem with pre-loaded plugins
pub struct IntegratedPluginSystemBuilder {
    system: IntegratedPluginSystem,
}

impl IntegratedPluginSystemBuilder {
    pub fn new() -> Self {
        Self {
            system: IntegratedPluginSystem::new(),
        }
    }

    pub fn with_tool_registry(tool_registry: Arc<RwLock<ToolRegistry>>) -> Self {
        Self {
            system: IntegratedPluginSystem::with_tool_registry(tool_registry),
        }
    }

    pub fn add_plugin(mut self, plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<Self> {
        self.system.load_plugin(plugin, config)?;
        Ok(self)
    }

    pub fn add_tool(mut self, name: &str, tool: Tool) -> Result<Self> {
        self.system.register_tool(name, tool)?;
        Ok(self)
    }

    pub fn build(self) -> IntegratedPluginSystem {
        self.system
    }
}

impl Default for IntegratedPluginSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PluginType;
    use crate::plugins::file_management::{FileManagementConfig, FileManagementPlugin};
    use crate::plugins::types::{ResourceLimits, SecurityPolicy};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_integrated_plugin_system() {
        let mut system = IntegratedPluginSystem::new();

        // Create a file management plugin
        let temp_dir = TempDir::new().unwrap();
        let plugin = FileManagementPlugin::builder()
            .temp_directory(temp_dir.path())
            .build()
            .unwrap();

        let config = PluginConfig {
            name: "file-management".to_string(),
            plugin_type: PluginType::Native,
            enabled: true,
            config: serde_json::to_value(&FileManagementConfig::default()).unwrap(),
            security_policy: SecurityPolicy {
                allow_file_system_access: true,
                allow_network_access: false,
                allowed_paths: vec![temp_dir.path().to_path_buf()],
                environment_variables: HashMap::new(),
                sandbox_enabled: false,
            },
            resource_limits: ResourceLimits {
                max_memory: Some(1024 * 1024 * 1024), // 1GB
                max_cpu_time: None,
                max_execution_time: Some(std::time::Duration::from_secs(300)),
                max_file_size: Some(100 * 1024 * 1024), // 100MB
                max_network_connections: Some(0),
            },
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        // Load the plugin
        assert!(system.load_plugin(Box::new(plugin), config).is_ok());

        // Check that tools are registered
        let tool_count = system.tool_count().unwrap();
        assert!(
            tool_count > 0,
            "Expected tools to be registered, but got {}",
            tool_count
        );

        // Check that we can list tools
        let tools = system.list_all_tools().unwrap();
        assert!(!tools.is_empty(), "Expected tools to be listed");

        // Check that we can find specific tools
        assert!(system.has_tool("text-processor").unwrap());
        assert!(system.has_tool("folder-classifier").unwrap());

        // Test plugin listing
        let plugins = system.list_plugins().unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "file-management");

        // Test plugin status
        let status = system.get_plugin_status("file-management").unwrap();
        assert!(status.is_some());
        assert_eq!(status.unwrap(), PluginStatus::Ready);

        // Test shutdown
        assert!(system.shutdown_all().is_ok());

        // After shutdown, no tools should be available
        let tool_count_after = system.tool_count().unwrap();
        assert_eq!(tool_count_after, 0);
    }

    #[test]
    fn test_integrated_plugin_system_builder() {
        let temp_dir = TempDir::new().unwrap();
        let plugin = FileManagementPlugin::builder()
            .temp_directory(temp_dir.path())
            .build()
            .unwrap();

        let config = PluginConfig {
            name: "file-management".to_string(),
            plugin_type: PluginType::Native,
            enabled: true,
            config: serde_json::to_value(&FileManagementConfig::default()).unwrap(),
            security_policy: SecurityPolicy {
                allow_file_system_access: true,
                allow_network_access: false,
                allowed_paths: vec![temp_dir.path().to_path_buf()],
                environment_variables: HashMap::new(),
                sandbox_enabled: false,
            },
            resource_limits: ResourceLimits {
                max_memory: Some(1024 * 1024 * 1024),
                max_cpu_time: None,
                max_execution_time: Some(std::time::Duration::from_secs(300)),
                max_file_size: Some(100 * 1024 * 1024),
                max_network_connections: Some(0),
            },
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        let result = IntegratedPluginSystemBuilder::new().add_plugin(Box::new(plugin), config);

        assert!(result.is_ok());
        let system = result.unwrap().build();

        // Verify the system has tools registered
        let tool_count = system.tool_count().unwrap();
        assert!(tool_count > 0);
    }
}
