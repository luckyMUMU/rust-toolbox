//! Tool registry for managing available tools

use crate::core::{ExecutionContext, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::tools::ToolNode;
use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Trait for tool registries
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// Register a new tool in the registry
    fn register_tool(&mut self, tool: Arc<dyn ToolNode>) -> Result<()>;
    
    /// Get a tool by name
    fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolNode>>;
    
    /// List all available tools
    fn list_tools(&self) -> Vec<ToolInfo>;
    
    /// Execute a tool by name with given parameters
    async fn execute_tool(&self, name: &str, params: Value, context: ExecutionContext) -> Result<Value>;
    
    /// Validate tool parameters without executing
    fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()>;
    
    /// Check if a tool exists in the registry
    fn has_tool(&self, name: &str) -> bool;
    
    /// Remove a tool from the registry
    fn unregister_tool(&mut self, name: &str) -> Result<()>;
    
    /// Get the number of registered tools
    fn tool_count(&self) -> usize;
    
    /// Clear all tools from the registry
    fn clear(&mut self);
}

/// Basic implementation of ToolRegistry using DashMap for concurrent access
pub struct BasicToolRegistry {
    tools: DashMap<String, Arc<dyn ToolNode>>,
    tool_info_cache: DashMap<String, ToolInfo>,
}

impl BasicToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: DashMap::new(),
            tool_info_cache: DashMap::new(),
        }
    }
    
    /// Create a new tool registry with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tools: DashMap::with_capacity(capacity),
            tool_info_cache: DashMap::with_capacity(capacity),
        }
    }
    
    /// Get tool names matching a pattern
    pub fn find_tools_by_pattern(&self, pattern: &str) -> Vec<String> {
        self.tools
            .iter()
            .filter_map(|entry| {
                let name = entry.key();
                if name.contains(pattern) {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get tools by category
    pub fn get_tools_by_category(&self, category: &str) -> Vec<ToolInfo> {
        self.tool_info_cache
            .iter()
            .filter_map(|entry| {
                let info = entry.value();
                if info.category.as_ref().map_or(false, |c| c == category) {
                    Some(info.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get tools by tag
    pub fn get_tools_by_tag(&self, tag: &str) -> Vec<ToolInfo> {
        self.tool_info_cache
            .iter()
            .filter_map(|entry| {
                let info = entry.value();
                if info.tags.contains(&tag.to_string()) {
                    Some(info.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Update tool info cache
    fn update_cache(&self, tool: &Arc<dyn ToolNode>) {
        let info = tool.get_info();
        self.tool_info_cache.insert(info.name.clone(), info);
    }
    
    /// Remove from cache
    fn remove_from_cache(&self, name: &str) {
        self.tool_info_cache.remove(name);
    }
}

impl Default for BasicToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolRegistry for BasicToolRegistry {
    fn register_tool(&mut self, tool: Arc<dyn ToolNode>) -> Result<()> {
        let name = tool.name().to_string();
        
        // Check if tool already exists
        if self.tools.contains_key(&name) {
            warn!("Tool '{}' already exists, replacing", name);
        }
        
        // Validate tool info
        let info = tool.get_info();
        if info.name.is_empty() {
            return Err(WorkflowError::ValidationError(
                "Tool name cannot be empty".to_string(),
            ));
        }
        
        if info.version.is_empty() {
            return Err(WorkflowError::ValidationError(
                "Tool version cannot be empty".to_string(),
            ));
        }
        
        // Register the tool
        self.tools.insert(name.clone(), tool.clone());
        self.update_cache(&tool);
        
        info!("Registered tool: {} v{}", name, info.version);
        Ok(())
    }
    
    fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolNode>> {
        self.tools.get(name).map(|entry| entry.value().clone())
    }
    
    fn list_tools(&self) -> Vec<ToolInfo> {
        self.tool_info_cache
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    async fn execute_tool(&self, name: &str, params: Value, context: ExecutionContext) -> Result<Value> {
        debug!("Executing tool '{}' with params: {}", name, params);
        
        let tool = self.get_tool(name).ok_or_else(|| {
            WorkflowError::NotFound {
                resource: format!("tool '{}'", name),
            }
        })?;
        
        match tool.execute(params, context).await {
            Ok(result) => {
                debug!("Tool '{}' executed successfully", name);
                Ok(result)
            }
            Err(e) => {
                error!("Tool '{}' execution failed: {}", name, e);
                Err(e)
            }
        }
    }
    
    fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()> {
        let tool = self.get_tool(name).ok_or_else(|| {
            WorkflowError::NotFound {
                resource: format!("tool '{}'", name),
            }
        })?;
        
        tool.validate_parameters(params)
    }
    
    fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
    
    fn unregister_tool(&mut self, name: &str) -> Result<()> {
        if self.tools.remove(name).is_some() {
            self.remove_from_cache(name);
            info!("Unregistered tool: {}", name);
            Ok(())
        } else {
            Err(WorkflowError::NotFound {
                resource: format!("tool '{}'", name),
            })
        }
    }
    
    fn tool_count(&self) -> usize {
        self.tools.len()
    }
    
    fn clear(&mut self) {
        let count = self.tools.len();
        self.tools.clear();
        self.tool_info_cache.clear();
        info!("Cleared {} tools from registry", count);
    }
}

/// Builder for BasicToolRegistry with pre-configured tools
pub struct ToolRegistryBuilder {
    registry: BasicToolRegistry,
}

impl ToolRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: BasicToolRegistry::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            registry: BasicToolRegistry::with_capacity(capacity),
        }
    }
    
    pub fn add_tool(mut self, tool: Arc<dyn ToolNode>) -> Result<Self> {
        self.registry.register_tool(tool)?;
        Ok(self)
    }
    
    pub fn build(self) -> BasicToolRegistry {
        self.registry
    }
}

impl Default for ToolRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{AsyncFunctionExecutor, BasicTool};
    use serde_json::json;
    
    #[tokio::test]
    async fn test_basic_tool_registry() {
        let mut registry = BasicToolRegistry::new();
        
        // Create a simple test tool
        let executor = Arc::new(AsyncFunctionExecutor::new(|params, _context| async move {
            Ok(json!({ "result": params }))
        }));
        
        let tool = BasicTool::builder()
            .name("test_tool")
            .version("1.0.0")
            .description("A test tool")
            .executor(executor)
            .build()
            .unwrap();
        
        let tool = Arc::new(tool);
        
        // Test registration
        assert!(registry.register_tool(tool.clone()).is_ok());
        assert_eq!(registry.tool_count(), 1);
        assert!(registry.has_tool("test_tool"));
        
        // Test retrieval
        let retrieved_tool = registry.get_tool("test_tool");
        assert!(retrieved_tool.is_some());
        
        // Test execution
        let context = ExecutionContext::new();
        let params = json!({ "input": "test" });
        let result = registry.execute_tool("test_tool", params.clone(), context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!({ "result": params }));
        
        // Test listing
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");
        
        // Test unregistration
        assert!(registry.unregister_tool("test_tool").is_ok());
        assert_eq!(registry.tool_count(), 0);
        assert!(!registry.has_tool("test_tool"));
    }
    
    #[tokio::test]
    async fn test_tool_not_found() {
        let registry = BasicToolRegistry::new();
        
        // Test getting non-existent tool
        assert!(registry.get_tool("non_existent").is_none());
        
        // Test executing non-existent tool
        let context = ExecutionContext::new();
        let result = registry.execute_tool("non_existent", json!({}), context).await;
        assert!(result.is_err());
        
        // Test validating params for non-existent tool
        let result = registry.validate_tool_params("non_existent", &json!({}));
        assert!(result.is_err());
    }
}