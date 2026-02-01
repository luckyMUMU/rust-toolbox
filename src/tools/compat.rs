//! Compatibility module for old trait-based tool system
//!
//! NOTE: This module provides placeholder types for backward compatibility
//! during the migration to the enum-based system. These will be removed
//! once all dependent code is updated.

use crate::error::Result;
use crate::tools::types::{Tool, ToolInput, ToolOutput};
use async_trait::async_trait;
use serde_json::Value;

/// DEPRECATED: ToolNode trait has been replaced by Tool enum
/// 
/// This placeholder is kept for backward compatibility only.
/// New code should use `types::Tool` enum directly.
#[async_trait]
pub trait ToolNode: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> String;
    fn definition(&self) -> crate::core::ToolInfo;
    fn validate_parameters(&self, params: &Value) -> Result<()>;
    async fn execute(&self, params: Value, context: crate::core::ExecutionContext) -> Result<Value>;
    fn get_info(&self) -> crate::core::ToolInfo;
    fn get_plugin_info(&self) -> Option<&crate::core::PluginInfo>;
}

/// DEPRECATED: ToolExecutor trait has been replaced by closure-based executors
/// 
/// This placeholder is kept for backward compatibility only.
/// New code should use `NativeToolBuilder::executor()` instead.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, params: Value, context: crate::core::ExecutionContext) -> Result<Value>;
    fn validate_parameters(&self, params: &Value) -> Result<()> {
        let _ = params;
        Ok(())
    }
}

/// DEPRECATED: ToolRegistry trait has been replaced by ToolRegistry struct
/// 
/// This placeholder is kept for backward compatibility only.
/// New code should use `registry::ToolRegistry` struct directly.
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    async fn register_tool(&self, tool: std::sync::Arc<dyn ToolNode>) -> Result<()>;
    async fn get_tool(&self, name: &str) -> Option<std::sync::Arc<dyn ToolNode>>;
    async fn list_tools(&self) -> Vec<String>;
    async fn execute_tool(&self, name: &str, params: Value, context: crate::core::ExecutionContext) -> Result<Value>;
}

/// DEPRECATED: ComposableTool trait has been replaced by ComposedTool enum variant
/// 
/// This placeholder is kept for backward compatibility only.
/// New code should use `types::ComposedTool` instead.
#[async_trait]
pub trait ComposableTool: Send + Sync {
    async fn execute(&self, params: Value, context: crate::core::ExecutionContext) -> Result<Value>;
    fn name(&self) -> &str;
    fn description(&self) -> String;
    fn validate_params(&self, params: &Value) -> Result<()> {
        let _ = params;
        Ok(())
    }
}
