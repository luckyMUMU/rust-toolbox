//! DI 具体模块实现
//!
//! 提供各业务领域的服务注册模块

pub mod plugin;
pub mod storage;
pub mod workflow;

pub use plugin::PluginModule;
pub use storage::StorageModule;
pub use workflow::WorkflowModule;
