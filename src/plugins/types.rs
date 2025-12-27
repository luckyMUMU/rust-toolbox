//! Plugin types and traits

use crate::core::PluginInfo;
use crate::error::Result;
use crate::tools::ToolNode;
use std::sync::Arc;

/// Plugin trait
pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;
    fn initialize(&mut self) -> Result<()>;
    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>>;
    fn shutdown(&mut self) -> Result<()>;
}

/// Plugin type enumeration
pub enum PluginType {
    Python(PythonPlugin),
    NodeJs(NodeJsPlugin),
    Docker(DockerPlugin),
    // Native plugins will be added in later tasks when we can properly handle trait objects
    // Native(Box<dyn Plugin>),
    // WASM support will be added in later tasks when wasmtime dependency is available
    // Wasm(WasmPlugin),
}

/// Python plugin wrapper
#[derive(Debug, Clone)]
pub struct PythonPlugin {
    pub info: PluginInfo,
    // Implementation details will be added later
}

/// Node.js plugin wrapper
#[derive(Debug, Clone)]
pub struct NodeJsPlugin {
    pub info: PluginInfo,
    // Implementation details will be added later
}

/// Docker plugin wrapper
#[derive(Debug, Clone)]
pub struct DockerPlugin {
    pub info: PluginInfo,
    // Implementation details will be added later
}