//! Plugin system for extending functionality

pub mod docker;
pub mod file_management;
pub mod integration;
pub mod manager;
pub mod native;
pub mod nodejs;
pub mod python;
pub mod types;
// pub mod wasm;  // Temporarily disabled due to wasmtime/extism dependency issues

pub use docker::{
    DockerEnvironment, DockerMount, DockerMountType, DockerNetworkConfig,
    DockerPlugin as DockerPluginImpl, DockerPluginBuilder, DockerResourceLimits,
    DockerRuntimeConfig, DockerToolConfig, DockerToolExecutor, DockerToolNode,
};
pub use file_management::{
    FileManagementConfig, FileManagementPlugin, FileManagementPluginBuilder,
};
pub use integration::{IntegratedPluginSystem, IntegratedPluginSystemBuilder};
pub use manager::{PluginManager, RuntimeManager};
pub use native::{NativePlugin as NativePluginImpl, NativePluginBuilder, NativeToolExecutor};
pub use nodejs::{
    NodeJsEnvironment, NodeJsPlugin as NodeJsPluginImpl, NodeJsPluginBuilder, NodeJsRuntimeConfig,
    NodeJsToolExecutor, NodeJsToolNode, PackageJson,
};
pub use python::{
    PythonEnvironment, PythonPlugin as PythonPluginImpl, PythonPluginBuilder, PythonRuntimeConfig,
    PythonToolExecutor, PythonToolNode,
};
pub use types::{
    DockerPlugin, // WasmPlugin,  // Temporarily disabled
    NativePlugin,
    NodeJsPlugin,
    Plugin,
    PluginConfig,
    PluginStatus,
    PluginType,
    PythonPlugin,
    ResourceLimits,
    SecurityPolicy,
};
// pub use wasm::{  // Temporarily disabled
//     WasmPlugin as WasmPluginImpl, WasmPluginBuilder, WasmRuntimeConfig,
//     WasmToolExecutor, WasmToolNode, ExtismToolNode, WasmRuntimeType, ExtismConfig,
// };
