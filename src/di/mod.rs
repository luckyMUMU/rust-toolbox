//! Dependency injection container
//!
//! 提供依赖注入容器和模块系统
//!
//! # 示例
//!
//! ```
//! use std::sync::Arc;
//! use workflow_toolkit::di::{DiContainer, AppModule, ModuleRegistrar};
//!
//! // 定义服务 trait
//! trait Database: Send + Sync {
//!     fn query(&self, sql: &str) -> Vec<String>;
//! }
//!
//! // 注册服务
//! let container = DiContainer::new();
//! container.register_factory::<dyn Database, _>(|| {
//!     Arc::new(MyDatabase::new())
//! });
//!
//! // 解析服务
//! let db = container.resolve::<dyn Database>();
//! ```

pub mod container;
pub mod module;
pub mod modules;
pub mod provider;

pub use container::{DiContainer, DiContainerError};
pub use module::{AppModule, ModuleError, ModuleRegistrar};
pub use modules::{PluginModule, StorageModule, WorkflowModule};
pub use provider::{FnProvider, Provider, SingletonProvider};
