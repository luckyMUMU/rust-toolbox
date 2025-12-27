//! Tool registry for managing available tools

use crate::core::{ExecutionContext, ToolInfo};
use crate::error::Result;
use crate::tools::ToolNode;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Trait for tool registries
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    fn register_tool(&mut self, tool: Arc<dyn ToolNode>) -> Result<()>;
    fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolNode>>;
    fn list_tools(&self) -> Vec<ToolInfo>;
    async fn execute_tool(&self, name: &str, params: Value, context: ExecutionContext) -> Result<Value>;
    fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()>;
}