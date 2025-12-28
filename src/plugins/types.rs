//! Plugin types and traits

use crate::core::PluginInfo;
use crate::error::Result;
use crate::tools::ToolNode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Plugin trait for all plugin types
pub trait Plugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> &PluginInfo;
    
    /// Initialize the plugin with configuration
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    
    /// Get all tools provided by this plugin
    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>>;
    
    /// Shutdown the plugin and cleanup resources
    fn shutdown(&mut self) -> Result<()>;
    
    /// Check if the plugin is initialized
    fn is_initialized(&self) -> bool;
    
    /// Get plugin status
    fn status(&self) -> PluginStatus;
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub plugin_type: crate::core::PluginType,
    pub enabled: bool,
    pub config: Value,
    pub security_policy: SecurityPolicy,
    pub resource_limits: ResourceLimits,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

impl PluginConfig {
    pub fn new(name: String, plugin_type: crate::core::PluginType) -> Self {
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

/// Security policy for plugins
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

/// Resource limits for plugins
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
            max_memory: Some(1024 * 1024 * 1024), // 1GB
            max_cpu_time: Some(Duration::from_secs(300)), // 5 minutes
            max_execution_time: Some(Duration::from_secs(600)), // 10 minutes
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_network_connections: Some(10),
        }
    }
}

/// Plugin status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    Uninitialized,
    Initializing,
    Ready,
    Running,
    Error,
    ShuttingDown,
    Shutdown,
}

/// Plugin type enumeration for concrete implementations
pub enum PluginType {
    Native(NativePlugin),
    Python(PythonPlugin),
    NodeJs(NodeJsPlugin),
    Docker(DockerPlugin),
    // WASM support will be added in later tasks when wasmtime dependency is available
    // Wasm(WasmPlugin),
}

/// Native plugin implementation
pub struct NativePlugin {
    pub info: PluginInfo,
    pub library_path: PathBuf,
    pub tools: Vec<Arc<dyn ToolNode>>,
    pub status: PluginStatus,
    pub config: Option<PluginConfig>,
    // The actual implementation is in the native module
    inner: Option<crate::plugins::native::NativePlugin>,
}

impl NativePlugin {
    pub fn new(info: PluginInfo, library_path: PathBuf) -> Self {
        Self {
            info: info.clone(),
            library_path: library_path.clone(),
            tools: Vec::new(),
            status: PluginStatus::Uninitialized,
            config: None,
            inner: Some(crate::plugins::native::NativePlugin::new(info, library_path)),
        }
    }
}

impl Plugin for NativePlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }
    
    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        if let Some(ref mut inner) = self.inner {
            inner.initialize(config.clone())?;
            self.tools = inner.get_tools();
            self.status = inner.status();
            self.config = Some(config);
        }
        Ok(())
    }
    
    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        if let Some(ref inner) = self.inner {
            inner.get_tools()
        } else {
            self.tools.clone()
        }
    }
    
    fn shutdown(&mut self) -> Result<()> {
        if let Some(ref mut inner) = self.inner {
            inner.shutdown()?;
            self.status = inner.status();
            self.tools.clear();
        }
        Ok(())
    }
    
    fn is_initialized(&self) -> bool {
        if let Some(ref inner) = self.inner {
            inner.is_initialized()
        } else {
            matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
        }
    }
    
    fn status(&self) -> PluginStatus {
        if let Some(ref inner) = self.inner {
            inner.status()
        } else {
            self.status
        }
    }
}

/// Python plugin wrapper
pub struct PythonPlugin {
    pub info: PluginInfo,
    pub status: PluginStatus,
    pub config: Option<PluginConfig>,
    // The actual implementation is in the python module
    inner: Option<crate::plugins::python::PythonPlugin>,
}

impl PythonPlugin {
    pub fn new(info: PluginInfo, runtime_config: crate::plugins::python::PythonRuntimeConfig) -> Self {
        Self {
            info: info.clone(),
            status: PluginStatus::Uninitialized,
            config: None,
            inner: Some(crate::plugins::python::PythonPlugin::new(info, runtime_config)),
        }
    }
}

impl Plugin for PythonPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }
    
    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        if let Some(ref mut inner) = self.inner {
            inner.initialize(config.clone())?;
            self.status = inner.status();
            self.config = Some(config);
        }
        Ok(())
    }
    
    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        if let Some(ref inner) = self.inner {
            inner.get_tools()
        } else {
            Vec::new()
        }
    }
    
    fn shutdown(&mut self) -> Result<()> {
        if let Some(ref mut inner) = self.inner {
            inner.shutdown()?;
            self.status = inner.status();
        }
        Ok(())
    }
    
    fn is_initialized(&self) -> bool {
        if let Some(ref inner) = self.inner {
            inner.is_initialized()
        } else {
            matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
        }
    }
    
    fn status(&self) -> PluginStatus {
        if let Some(ref inner) = self.inner {
            inner.status()
        } else {
            self.status
        }
    }
}

/// Node.js plugin wrapper
pub struct NodeJsPlugin {
    pub info: PluginInfo,
    pub status: PluginStatus,
    pub config: Option<PluginConfig>,
    // The actual implementation is in the nodejs module
    inner: Option<crate::plugins::nodejs::NodeJsPlugin>,
}

impl NodeJsPlugin {
    pub fn new(info: PluginInfo, runtime_config: crate::plugins::nodejs::NodeJsRuntimeConfig) -> Self {
        Self {
            info: info.clone(),
            status: PluginStatus::Uninitialized,
            config: None,
            inner: Some(crate::plugins::nodejs::NodeJsPlugin::new(info, runtime_config)),
        }
    }
}

impl Plugin for NodeJsPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }
    
    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        if let Some(ref mut inner) = self.inner {
            inner.initialize(config.clone())?;
            self.status = inner.status();
            self.config = Some(config);
        }
        Ok(())
    }
    
    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        if let Some(ref inner) = self.inner {
            inner.get_tools()
        } else {
            Vec::new()
        }
    }
    
    fn shutdown(&mut self) -> Result<()> {
        if let Some(ref mut inner) = self.inner {
            inner.shutdown()?;
            self.status = inner.status();
        }
        Ok(())
    }
    
    fn is_initialized(&self) -> bool {
        if let Some(ref inner) = self.inner {
            inner.is_initialized()
        } else {
            matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
        }
    }
    
    fn status(&self) -> PluginStatus {
        if let Some(ref inner) = self.inner {
            inner.status()
        } else {
            self.status
        }
    }
}

/// Docker plugin wrapper
#[derive(Debug, Clone)]
pub struct DockerPlugin {
    pub info: PluginInfo,
    pub status: PluginStatus,
    // Implementation details will be added later
}

impl Plugin for DockerPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }
    
    fn initialize(&mut self, _config: PluginConfig) -> Result<()> {
        // Docker plugin initialization will be implemented later
        self.status = PluginStatus::Ready;
        Ok(())
    }
    
    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        // Implementation will be added later
        Vec::new()
    }
    
    fn shutdown(&mut self) -> Result<()> {
        self.status = PluginStatus::Shutdown;
        Ok(())
    }
    
    fn is_initialized(&self) -> bool {
        matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
}