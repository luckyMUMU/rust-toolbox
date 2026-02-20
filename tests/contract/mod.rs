//! 契约测试模块
//!
//! 提供核心组件的契约测试框架

pub mod workflow_engine;
pub mod tool_registry;
pub mod plugin_manager;

pub use workflow_engine::WorkflowEngineContract;
pub use tool_registry::ToolRegistryContract;
pub use plugin_manager::PluginManagerContract;
