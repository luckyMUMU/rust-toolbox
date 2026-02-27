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
//! - **RuntimeManager**: Manages language runtimes with process pools and container lifecycle.
//!
//! ## Plugin Types
//!
//! - **Native**: Compiled Rust dynamic libraries (.dll, .so, .dylib).
//! - **Python**: Python scripts running in a managed environment.
//! - **Node.js**: JavaScript/TypeScript modules.
//! - **Docker**: Containerized tools.

// Macros must be defined before use
#[macro_use]
pub mod macros;

pub mod docker;
pub mod error;
pub mod file_management;
pub mod integration;
pub mod manager;
pub mod native;
pub mod nodejs;
pub mod process_pool;
pub mod python;
pub mod runtime;
pub mod types;
pub mod wasm_limits;
pub mod wasm_sandbox;
// pub mod wasm;  // 需要添加 wasmtime/extism 依赖后启用

pub use docker::{
    DockerEnvironment, DockerMount, DockerMountType, DockerNetworkConfig,
    DockerPlugin as DockerPluginImpl, DockerPluginBuilder, DockerResourceLimits,
    DockerRuntimeConfig, DockerToolConfig,
};
pub use error::{IntoPluginError, PluginError, Result as PluginResult};
pub use file_management::{
    FileManagementConfig, FileManagementPlugin, FileManagementPluginBuilder,
};
pub use integration::{IntegratedPluginSystem, IntegratedPluginSystemBuilder};
pub use manager::PluginManager;
pub use native::{NativePlugin as NativePluginImpl, NativePluginBuilder};
pub use nodejs::{
    NodeJsEnvironment, NodeJsPlugin as NodeJsPluginImpl, NodeJsPluginBuilder, NodeJsRuntimeConfig,
    PackageJson,
};
pub use process_pool::{
    PluginProcessPool, PoolStats, ProcessInfo, ProcessPoolConfig, ProcessState,
};
pub use python::{
    PythonEnvironment, PythonPlugin as PythonPluginImpl, PythonPluginBuilder, PythonRuntimeConfig,
};
pub use runtime::{ResourceStats, RuntimeManager, RuntimePoolConfig, RuntimeStats};
pub use types::{
    DockerPlugin, // WasmPlugin,  // 需要添加 wasmtime/extism 依赖后启用
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
pub use wasm_limits::{
    ResourceLimitError, ResourceLimiter, ResourceLimiterBuilder, ResourceReport, ResourceUsage,
    ResourceUsageSnapshot, ResourceUtilization, WasmResourceLimits,
};
pub use wasm_sandbox::{
    AuditEntry, AuditEventType, FileSystemPermissions, NetworkPermissions, SandboxLevel,
    SyscallPermissions, WasmSandbox, WasmSandboxBuilder, WasmSandboxConfig,
};
// pub use wasm::{  // 需要添加 wasmtime/extism 依赖后启用
//     WasmPlugin as WasmPluginImpl, WasmPluginBuilder, WasmRuntimeConfig,
//     WasmToolExecutor, WasmToolNode, ExtismToolNode, WasmRuntimeType, ExtismConfig,
// };
