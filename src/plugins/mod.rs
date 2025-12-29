//! Plugin system for extending functionality

pub mod docker;
pub mod manager;
pub mod native;
pub mod nodejs;
pub mod python;
pub mod types;
// pub mod wasm;  // Temporarily disabled due to wasmtime/extism dependency issues

pub use docker::{
    DockerPlugin as DockerPluginImpl, DockerPluginBuilder, DockerEnvironment, 
    DockerRuntimeConfig, DockerToolExecutor, DockerToolNode, DockerToolConfig,
    DockerMount, DockerMountType, DockerResourceLimits, DockerNetworkConfig,
};
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
    NativePlugin, PythonPlugin, NodeJsPlugin, DockerPlugin, // WasmPlugin,  // Temporarily disabled
};
// pub use wasm::{  // Temporarily disabled
//     WasmPlugin as WasmPluginImpl, WasmPluginBuilder, WasmRuntimeConfig,
//     WasmToolExecutor, WasmToolNode, ExtismToolNode, WasmRuntimeType, ExtismConfig,
// };