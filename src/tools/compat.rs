//! Compatibility module for old trait-based tool system
//!
//! NOTE: This module provides placeholder types for backward compatibility
//! during the migration to the enum-based system. These will be removed
//! once all dependent code is updated.

use crate::error::Result;
use crate::tools::types::{Tool, ToolInput, ToolOutput};
use crate::tools::version::{Version, VersionRequirement};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// DEPRECATED: ToolNode trait has been replaced by Tool enum
/// 
/// This placeholder is kept for backward compatibility only.
/// New code should use `types::Tool` enum directly.
#[async_trait]
pub trait ToolNode: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    // Default implementations for backward compatibility
    fn description(&self) -> String {
        format!("Tool: {}", self.name())
    }
    
    fn definition(&self) -> crate::core::ToolInfo {
        crate::core::ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: self.description(),
            parameters_schema: Value::Null,
            return_schema: Value::Null,
            category: None,
            tags: Vec::new(),
            dependencies: Vec::new(),
            plugin_name: None,
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
    
    fn validate_parameters(&self, params: &Value) -> Result<()> {
        let _ = params;
        Ok(())
    }
    
    async fn execute(&self, params: Value, context: crate::core::ExecutionContext) -> Result<Value>;
    
    fn get_info(&self) -> crate::core::ToolInfo {
        self.definition()
    }
    
    fn get_plugin_info(&self) -> Option<&crate::core::PluginInfo> {
        None
    }
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
    
    // Additional methods with default implementations for backward compatibility
    async fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()> {
        let _ = (name, params);
        Ok(())
    }
    
    fn has_tool(&self, name: &str) -> bool {
        let _ = name;
        false
    }
    
    async fn unregister_tool(&self, name: &str) -> Result<()> {
        let _ = name;
        Ok(())
    }
    
    fn tool_count(&self) -> usize {
        0
    }
    
    fn clear(&mut self) {}
    
    async fn resolve_dependencies(&self, tool_name: &str) -> Result<Vec<String>> {
        let _ = tool_name;
        Ok(Vec::new())
    }
    
    async fn check_version_conflicts(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
    
    async fn get_dependents(&self, tool_name: &str) -> Result<Vec<String>> {
        let _ = tool_name;
        Ok(Vec::new())
    }
    
    async fn execute_tool_with_templates(&self, name: &str, params: Value, context: crate::core::ExecutionContext) -> Result<Value> {
        self.execute_tool(name, params, context).await
    }
    
    async fn get_tool_templates(&self, name: &str) -> Result<Vec<String>> {
        let _ = name;
        Ok(Vec::new())
    }
}

/// DEPRECATED: ComposableTool trait has been replaced by ComposedTool enum variant
/// 
/// This placeholder is kept for backward compatibility only.
/// New code should use `types::ComposedTool` instead.
#[async_trait]
pub trait ComposableTool: Send + Sync {
    async fn execute(&self, params: Value, context: crate::core::ExecutionContext) -> Result<Value>;
    fn name(&self) -> &str;
    fn description(&self) -> String {
        format!("Composable tool: {}", self.name())
    }
    fn validate_params(&self, params: &Value) -> Result<()> {
        let _ = params;
        Ok(())
    }
}

/// DEPRECATED: BasicToolRegistry has been replaced by registry::ToolRegistry
/// 
/// This placeholder is kept for backward compatibility only.
pub struct BasicToolRegistry;

impl BasicToolRegistry {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BasicToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
