//! # Plugin System
//!
//! The plugin system allows extending the workflow engine with external functionality.
//! It supports multiple plugin types including Native (Rust dylib), Python, Node.js, and Docker.
//!
//! ## Key Components
//!
//! - **PluginManager**: Manages the lifecycle of plugins (load, unload, reload).
//! - **Plugin**: The core trait that all plugins must implement.
//! - **PluginConfig**: Configuration for plugins including security policies and resource limits.
//! - **RuntimeManager**: Manages language runtimes (e.g., Python interpreter, Node.js process).
//!
//! ## Plugin Types
//!
//! - **Native**: Compiled Rust dynamic libraries (.dll, .so, .dylib).
//! - **Python**: Python scripts running in a managed environment.
//! - **Node.js**: JavaScript/TypeScript modules.
//! - **Docker**: Containerized tools.
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.
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
