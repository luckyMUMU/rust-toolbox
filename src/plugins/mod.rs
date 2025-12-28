//! Plugin system for extending functionality

pub mod manager;
pub mod native;
pub mod nodejs;
pub mod python;
pub mod types;

pub use manager::{PluginManager, RuntimeManager};
pub use native::{NativePlugin as NativePluginImpl, NativePluginBuilder, NativeToolExecutor};
pub use nodejs::{
    NodeJsPlugin as NodeJsPluginImpl, NodeJsPluginBuilder, NodeJsEnvironment, 
    NodeJsRuntimeConfig, NodeJsToolExecutor, NodeJsToolNode, PackageJson,
};
pub use python::{
    PythonPlugin as PythonPluginImpl, PythonPluginBuilder, PythonEnvironment, 
    PythonRuntimeConfig, PythonToolExecutor, PythonToolNode,
};
pub use types::{
    Plugin, PluginConfig, PluginStatus, PluginType, ResourceLimits, SecurityPolicy,
    NativePlugin, PythonPlugin, NodeJsPlugin, DockerPlugin,
};