//! DI 具体模块实现
//!
//! 提供各业务领域的服务注册模块

pub mod workflow;
pub mod plugin;
pub mod storage;

pub use workflow::WorkflowModule;
pub use plugin::PluginModule;
pub use storage::StorageModule;
