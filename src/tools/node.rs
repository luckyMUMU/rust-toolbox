//! Tool node trait and implementations

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Trait for tool nodes
#[async_trait]
pub trait ToolNode: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn validate_parameters(&self, params: &Value) -> Result<()>;
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;
    fn get_info(&self) -> ToolInfo;
    fn get_plugin_info(&self) -> Option<&PluginInfo>;
}