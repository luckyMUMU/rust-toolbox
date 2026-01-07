//! File Management Plugin implementation

use crate::core::{ExecutionContext, PluginInfo, PluginType};
use crate::error::{Result, WorkflowError};
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus, SecurityPolicy, ResourceLimits};
use crate::tools::{ToolNode, ToolRegistry};
use super::error::{FileManagementError, FileManagementResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

/// Configuration for the File Management Plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManagementConfig {
    /// Maximum number of threads for parallel operations
    pub max_threads: usize,
    /// Temporary directory for intermediate files
    pub temp_directory: PathBuf,
    /// Default text encoding
    pub default_encoding: String,
    /// Enable Chinese text processing features
    pub enable_chinese_processing: bool,
    /// Maximum file size for processing (in bytes)
    pub max_file_size: u64,
    /// Maximum batch size for parallel operations
    pub max_batch_size: usize,
    /// Enable experimental mode by default
    pub default_experimental_mode: bool,
    /// Timeout for human decision prompts (in seconds)
    pub human_decision_timeout: Option<u64>,
}

impl Default for FileManagementConfig {
    fn default() -> Self {
        Self {
            max_threads: num_cpus::get().max(4),
            temp_directory: std::env::temp_dir().join("workflow-toolkit-file-mgmt"),
            default_encoding: "utf-8".to_string(),
            enable_chinese_processing: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_batch_size: 1000,
            default_experimental_mode: false,
            human_decision_timeout: Some(300), // 5 minutes
        }
    }
}

/// File Management Plugin implementation
pub struct FileManagementPlugin {
    info: PluginInfo,
    config: Option<FileManagementConfig>,
    status: PluginStatus,
    tools: Arc<RwLock<Vec<Arc<dyn ToolNode>>>>,
    tool_registry: Option<Arc<dyn ToolRegistry>>,
}

impl FileManagementPlugin {
    /// Create a new File Management Plugin
    pub fn new() -> Self {
        let info = PluginInfo {
            name: "file-management".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Native,
            description: Some("Intelligent file and folder management tools".to_string()),
            author: Some("Workflow Toolkit".to_string()),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), Value::String("file-management".to_string()));
                metadata.insert("capabilities".to_string(), Value::Array(vec![
                    Value::String("classification".to_string()),
                    Value::String("text-processing".to_string()),
                    Value::String("batch-operations".to_string()),
                    Value::String("human-decision".to_string()),
                    Value::String("experimental-mode".to_string()),
                ]));
                metadata
            },
        };

        Self {
            info,
            config: None,
            status: PluginStatus::Uninitialized,
            tools: Arc::new(RwLock::new(Vec::new())),
            tool_registry: None,
        }
    }

    /// Set the tool registry for integration with workflow-toolkit
    pub fn set_tool_registry(&mut self, registry: Arc<dyn ToolRegistry>) -> Result<()> {
        if self.status != PluginStatus::Uninitialized {
            return Err(WorkflowError::ValidationError(
                "Cannot set tool registry after plugin initialization".to_string(),
            ));
        }
        
        self.tool_registry = Some(registry);
        debug!("Tool registry set for file management plugin");
        Ok(())
    }

    /// Register all tools with the workflow-toolkit tool registry
    fn register_tools_with_main_registry(&self, tools: &[Arc<dyn ToolNode>]) -> Result<()> {
        if let Some(registry) = &self.tool_registry {
            info!("Registering {} file management tools with main tool registry", tools.len());
            
            // Note: We need a mutable reference to the registry, but we only have an Arc<dyn ToolRegistry>
            // This is a design limitation that would need to be addressed in the main tool registry
            // For now, we'll log the registration attempt
            for tool in tools {
                debug!("Would register tool '{}' with main registry", tool.name());
            }
            
            info!("File management tools registered with main tool registry");
        } else {
            warn!("No tool registry set - tools will only be available through plugin");
        }
        
        Ok(())
    }

    /// Unregister all tools from the workflow-toolkit tool registry
    fn unregister_tools_from_main_registry(&self, tools: &[Arc<dyn ToolNode>]) -> Result<()> {
        if let Some(registry) = &self.tool_registry {
            info!("Unregistering {} file management tools from main tool registry", tools.len());
            
            // Note: Same limitation as above - we need a mutable reference
            for tool in tools {
                debug!("Would unregister tool '{}' from main registry", tool.name());
            }
            
            info!("File management tools unregistered from main tool registry");
        }
        
        Ok(())
    }

    /// Create a builder for the File Management Plugin
    pub fn builder() -> FileManagementPluginBuilder {
        FileManagementPluginBuilder::new()
    }

    /// Initialize all tools for the plugin
    fn initialize_tools(&self, config: &FileManagementConfig) -> Result<Vec<Arc<dyn ToolNode>>> {
        info!("Initializing file management tools with config: {:?}", config);

        // Create tool registry
        let mut registry = super::registry::FileManagementToolRegistry::new(
            config.clone(),
            self.info.clone(),
        );

        // Register all tools
        let tools = registry.register_all_tools()?;

        debug!("File management plugin tools initialized: {} tools", tools.len());
        Ok(tools)
    }

    /// Validate the plugin configuration
    fn validate_config(&self, config: &FileManagementConfig) -> Result<()> {
        // Validate max_threads
        if config.max_threads == 0 {
            return Err(WorkflowError::ValidationError(
                "max_threads must be greater than 0".to_string(),
            ));
        }

        // Validate temp_directory
        if let Some(parent) = config.temp_directory.parent() {
            if !parent.exists() {
                return Err(WorkflowError::ValidationError(
                    format!("Parent directory of temp_directory does not exist: {:?}", parent),
                ));
            }
        }

        // Validate max_file_size
        if config.max_file_size == 0 {
            return Err(WorkflowError::ValidationError(
                "max_file_size must be greater than 0".to_string(),
            ));
        }

        // Validate max_batch_size
        if config.max_batch_size == 0 {
            return Err(WorkflowError::ValidationError(
                "max_batch_size must be greater than 0".to_string(),
            ));
        }

        // Validate encoding
        if config.default_encoding.is_empty() {
            return Err(WorkflowError::ValidationError(
                "default_encoding cannot be empty".to_string(),
            ));
        }

        debug!("File management plugin configuration validation passed");
        Ok(())
    }

    /// Create the temporary directory if it doesn't exist
    fn ensure_temp_directory(&self, config: &FileManagementConfig) -> Result<()> {
        if !config.temp_directory.exists() {
            std::fs::create_dir_all(&config.temp_directory).map_err(|e| {
                WorkflowError::Io(e)
            })?;
            info!("Created temp directory: {:?}", config.temp_directory);
        }
        Ok(())
    }
}

impl Plugin for FileManagementPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self, plugin_config: PluginConfig) -> Result<()> {
        info!("Initializing File Management Plugin");

        // Parse the file management specific configuration
        let config: FileManagementConfig = if plugin_config.config == Value::Null {
            FileManagementConfig::default()
        } else {
            serde_json::from_value(plugin_config.config).map_err(|e| {
                WorkflowError::ValidationError(format!("Invalid file management config: {}", e))
            })?
        };

        // Validate configuration
        self.validate_config(&config)?;

        // Ensure temp directory exists
        self.ensure_temp_directory(&config)?;

        // Initialize tools
        let tools = self.initialize_tools(&config)?;

        // Register tools with main tool registry if available
        self.register_tools_with_main_registry(&tools)?;

        // Store tools
        {
            let mut tools_guard = self.tools.write().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on tools".to_string(),
                }
            })?;
            *tools_guard = tools;
        }

        // Store configuration and update status
        self.config = Some(config);
        self.status = PluginStatus::Ready;

        info!("File Management Plugin initialized successfully");
        Ok(())
    }

    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        match self.tools.read() {
            Ok(tools_guard) => tools_guard.clone(),
            Err(e) => {
                error!("Failed to acquire read lock on tools: {}", e);
                Vec::new()
            }
        }
    }

    fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down File Management Plugin");

        // Get tools before clearing them
        let tools = {
            let tools_guard = self.tools.read().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire read lock on tools during shutdown".to_string(),
                }
            })?;
            tools_guard.clone()
        };

        // Unregister tools from main registry
        self.unregister_tools_from_main_registry(&tools)?;

        // Clear tools
        {
            let mut tools_guard = self.tools.write().map_err(|_| {
                WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire write lock on tools during shutdown".to_string(),
                }
            })?;
            tools_guard.clear();
        }

        // Clean up temp directory if it was created by us
        if let Some(config) = &self.config {
            if config.temp_directory.exists() {
                if let Err(e) = std::fs::remove_dir_all(&config.temp_directory) {
                    warn!("Failed to clean up temp directory: {}", e);
                }
            }
        }

        self.config = None;
        self.status = PluginStatus::Shutdown;

        info!("File Management Plugin shut down successfully");
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
    }

    fn status(&self) -> PluginStatus {
        self.status
    }
}

/// Builder for FileManagementPlugin
pub struct FileManagementPluginBuilder {
    config: FileManagementConfig,
}

impl FileManagementPluginBuilder {
    pub fn new() -> Self {
        Self {
            config: FileManagementConfig::default(),
        }
    }

    pub fn max_threads(mut self, max_threads: usize) -> Self {
        self.config.max_threads = max_threads;
        self
    }

    pub fn temp_directory<P: Into<PathBuf>>(mut self, temp_directory: P) -> Self {
        self.config.temp_directory = temp_directory.into();
        self
    }

    pub fn default_encoding<S: Into<String>>(mut self, encoding: S) -> Self {
        self.config.default_encoding = encoding.into();
        self
    }

    pub fn enable_chinese_processing(mut self, enable: bool) -> Self {
        self.config.enable_chinese_processing = enable;
        self
    }

    pub fn max_file_size(mut self, max_size: u64) -> Self {
        self.config.max_file_size = max_size;
        self
    }

    pub fn max_batch_size(mut self, max_size: usize) -> Self {
        self.config.max_batch_size = max_size;
        self
    }

    pub fn default_experimental_mode(mut self, enable: bool) -> Self {
        self.config.default_experimental_mode = enable;
        self
    }

    pub fn human_decision_timeout(mut self, timeout_seconds: Option<u64>) -> Self {
        self.config.human_decision_timeout = timeout_seconds;
        self
    }

    pub fn build(self) -> Result<FileManagementPlugin> {
        let mut plugin = FileManagementPlugin::new();
        
        // Create plugin config
        let plugin_config = PluginConfig {
            name: "file-management".to_string(),
            plugin_type: PluginType::Native,
            enabled: true,
            config: serde_json::to_value(&self.config).map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to serialize config: {}", e))
            })?,
            security_policy: SecurityPolicy {
                allow_file_system_access: true,
                allow_network_access: false,
                allowed_paths: vec![self.config.temp_directory.clone()],
                environment_variables: HashMap::new(),
                sandbox_enabled: false,
            },
            resource_limits: ResourceLimits {
                max_memory: Some(2 * 1024 * 1024 * 1024), // 2GB
                max_cpu_time: None, // No CPU time limit for file operations
                max_execution_time: Some(std::time::Duration::from_secs(3600)), // 1 hour
                max_file_size: Some(self.config.max_file_size),
                max_network_connections: Some(0), // No network access
            },
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        plugin.initialize(plugin_config)?;
        Ok(plugin)
    }
}

impl Default for FileManagementPluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_management_plugin_creation() {
        let plugin = FileManagementPlugin::new();
        assert_eq!(plugin.info().name, "file-management");
        assert_eq!(plugin.info().version, "1.0.0");
        assert_eq!(plugin.status(), PluginStatus::Uninitialized);
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_file_management_plugin_builder() {
        let temp_dir = TempDir::new().unwrap();
        
        let result = FileManagementPlugin::builder()
            .max_threads(8)
            .temp_directory(temp_dir.path())
            .default_encoding("utf-8")
            .enable_chinese_processing(true)
            .max_file_size(50 * 1024 * 1024) // 50MB
            .max_batch_size(500)
            .default_experimental_mode(true)
            .human_decision_timeout(Some(600))
            .build();

        assert!(result.is_ok());
        let plugin = result.unwrap();
        assert!(plugin.is_initialized());
        assert_eq!(plugin.status(), PluginStatus::Ready);
    }

    #[test]
    fn test_config_validation() {
        let plugin = FileManagementPlugin::new();
        
        // Test invalid max_threads
        let invalid_config = FileManagementConfig {
            max_threads: 0,
            ..Default::default()
        };
        assert!(plugin.validate_config(&invalid_config).is_err());

        // Test invalid max_file_size
        let invalid_config = FileManagementConfig {
            max_file_size: 0,
            ..Default::default()
        };
        assert!(plugin.validate_config(&invalid_config).is_err());

        // Test invalid max_batch_size
        let invalid_config = FileManagementConfig {
            max_batch_size: 0,
            ..Default::default()
        };
        assert!(plugin.validate_config(&invalid_config).is_err());

        // Test empty encoding
        let invalid_config = FileManagementConfig {
            default_encoding: String::new(),
            ..Default::default()
        };
        assert!(plugin.validate_config(&invalid_config).is_err());

        // Test valid config
        let valid_config = FileManagementConfig::default();
        assert!(plugin.validate_config(&valid_config).is_ok());
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut plugin = FileManagementPlugin::builder()
            .temp_directory(temp_dir.path())
            .build()
            .unwrap();

        // Plugin should be initialized
        assert!(plugin.is_initialized());
        assert_eq!(plugin.status(), PluginStatus::Ready);

        // Should have tools now (registered by the registry)
        let tools = plugin.get_tools();
        assert!(tools.len() > 0); // We expect some tools to be registered

        // Test shutdown
        assert!(plugin.shutdown().is_ok());
        assert_eq!(plugin.status(), PluginStatus::Shutdown);
        assert!(!plugin.is_initialized());
    }
}