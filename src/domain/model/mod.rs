//! 领域模型

pub mod execution;
pub mod plugin;
pub mod tool;
pub mod value_object;
pub mod workflow;

// 重新导出常用类型
pub use execution::{ExecutionContext, ExecutionMode, ExecutionStatus, TaskPriority, WorkflowId};
pub use plugin::{PluginInfo, PluginType};
pub use tool::ToolInfo;
pub use value_object::*;
pub use workflow::{
    ConcurrencyConfig, ExecutionMetrics, ResourceLimits, RetryPolicy, RetryStrategy, WorkflowConfig,
};
