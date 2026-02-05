//! Compatibility module for old trait-based tool system
//!
//! NOTE: This module provides placeholder types for backward compatibility
//! during the migration to the enum-based system. These will be removed
//! once all dependent code is updated.

use crate::error::{Result, WorkflowError};
use crate::tools::types::{Tool, ToolId, ToolOutput};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

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
    fn register_tool(&mut self, tool: std::sync::Arc<dyn ToolNode>) -> Result<()>;
    fn get_tool(&self, name: &str) -> Option<std::sync::Arc<dyn ToolNode>>;
    fn list_tools(&self) -> Vec<crate::core::ToolInfo>;
    async fn execute_tool(&self, name: &str, params: Value, context: crate::core::ExecutionContext) -> Result<Value>;
    
    // Additional methods with default implementations for backward compatibility
    fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()> {
        let _ = (name, params);
        Ok(())
    }
    
    fn has_tool(&self, name: &str) -> bool {
        let _ = name;
        false
    }
    
    fn unregister_tool(&mut self, name: &str) -> Result<()> {
        let _ = name;
        Ok(())
    }
    
    fn tool_count(&self) -> usize {
        0
    }
    
    fn clear(&mut self) {}
    
    fn resolve_dependencies(&self, _tool_names: Vec<String>) -> Result<crate::tools::ResolutionResult> {
        Ok(crate::tools::ResolutionResult {
            resolved_versions: HashMap::new(),
            conflicts: Vec::new(),
            warnings: Vec::new(),
        })
    }
    
    fn check_version_conflicts(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
    
    fn get_dependents(&self, _tool_name: &str) -> Vec<crate::core::ToolInfo> {
        Vec::new()
    }
    
    async fn execute_tool_with_templates(&self, name: &str, params: Value, _template_context: &crate::tools::TemplateContext, execution_context: crate::core::ExecutionContext) -> Result<Value> {
        self.execute_tool(name, params, execution_context).await
    }
    
    fn get_tool_templates(&self, _tool_name: &str) -> Vec<crate::tools::ParameterTemplate> {
        Vec::new()
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
/// This placeholder wraps the new registry for backward compatibility.
pub struct BasicToolRegistry {
    inner: crate::tools::registry::ToolRegistry,
}

impl BasicToolRegistry {
    pub fn new() -> Self {
        Self {
            inner: crate::tools::registry::ToolRegistry::new(),
        }
    }
    
    pub fn register(&self, name: &str, tool: Tool) -> ToolId {
        self.inner.register(name, tool)
    }
    
    pub fn get(&self, name: &str) -> Option<Tool> {
        self.inner.get(name)
    }
    
    pub fn list_all_tools(&self) -> Vec<(ToolId, Tool)> {
        self.inner.list_ids().into_iter()
            .filter_map(|id| self.inner.get_by_id(id).map(|tool| (id, tool)))
            .collect()
    }
}

impl Default for BasicToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolRegistry for BasicToolRegistry {
    fn register_tool(&mut self, _tool: std::sync::Arc<dyn ToolNode>) -> Result<()> {
        // Placeholder implementation - new code should use registry::ToolRegistry directly
        Ok(())
    }
    
    fn get_tool(&self, name: &str) -> Option<std::sync::Arc<dyn ToolNode>> {
        let _ = name;
        None
    }
    
    fn list_tools(&self) -> Vec<crate::core::ToolInfo> {
        Vec::new()
    }
    
    async fn execute_tool(&self, _name: &str, _params: Value, _context: crate::core::ExecutionContext) -> Result<Value> {
        Err(WorkflowError::ValidationError("BasicToolRegistry is deprecated, use registry::ToolRegistry".to_string()))
    }
}

/// Extension trait for ToolNode to convert to new enum-based Tool system
///
/// This provides migration support during the transition from trait-based to enum-based tools.
/// TODO: Remove this trait once all plugins migrate to enum-based Tool system (target: 6 months)
pub trait ToolNodeExt: ToolNode {
    /// Convert this ToolNode trait object to the new Tool enum
    ///
    /// During migration, this wraps the trait object in a compatibility shim.
    /// Once plugins are fully migrated, they will create Tool enums directly.
    fn to_tool_enum(self: Arc<Self>) -> Tool;
}

/// Compatibility shim that wraps a ToolNode trait object as a NativeTool
///
/// This allows old trait-based tools to work with the new enum-based system
/// during the migration period.
/// 
/// Note: Currently unused but reserved for backward compatibility during migration.
#[allow(dead_code)]
struct ToolNodeAdapter {
    inner: Arc<dyn ToolNode>,
}

#[allow(dead_code)]
impl ToolNodeAdapter {
    fn new(inner: Arc<dyn ToolNode>) -> Self {
        Self { inner }
    }
}

use crate::tools::types::{NativeTool, ToolMetadata, ToolKind};

impl NativeTool {
    /// Create a NativeTool from a ToolNode trait object (compatibility shim)
    fn from_tool_node(tool_node: Arc<dyn ToolNode>) -> Arc<Self> {
        let metadata = Arc::new(ToolMetadata {
            info: tool_node.definition(),
            kind: ToolKind::Native,
            input_schema: None,
            output_schema: None,
            examples: Vec::new(),
            resource_requirements: crate::tools::types::ResourceRequirements::default(),
            version: tool_node.version().to_string(),
        });
        
        Arc::new(NativeTool {
            id: ToolId::new(),
            metadata,
            executor: Arc::new(move |input, ctx| {
                let tool = tool_node.clone();
                Box::pin(async move {
                    let result = tool.execute(input.params, ctx).await?;
                    Ok(ToolOutput::success(result))
                })
            }),
            middleware_stack: None,
        })
    }
}

impl<T: ToolNode + 'static> ToolNodeExt for T {
    fn to_tool_enum(self: Arc<Self>) -> Tool {
        // During migration: wrap trait object in NativeTool variant
        // This maintains compatibility while allowing gradual migration
        Tool::Native(NativeTool::from_tool_node(self))
    }
}

/// Convert an Arc<dyn ToolNode> trait object to the new Tool enum
/// 
/// This is a helper function for cases where you have a trait object
/// and need to convert it to the enum-based system.
pub fn tool_node_to_enum(tool_node: Arc<dyn ToolNode>) -> Tool {
    Tool::Native(NativeTool::from_tool_node(tool_node))
}

/// Convert a Tool enum back to Arc<dyn ToolNode> for backward compatibility
/// 
/// This is needed during the migration phase when the old registry still expects
/// trait objects but plugins return the new enum-based Tool.
/// 
/// TODO: Remove this once the registry is fully migrated to enum-based tools
pub fn tool_to_trait_object(tool: &Tool) -> Option<Arc<dyn ToolNode>> {
    // Create a ToolNode adapter that wraps any Tool variant
    Some(Arc::new(ToolEnumAdapter::new(tool.clone())))
}

/// Adapter that implements ToolNode trait for a Tool enum
/// 
/// This allows the new Tool enum to work with old code that expects ToolNode trait objects.
/// This is a temporary shim during migration.
struct ToolEnumAdapter {
    tool: Tool,
    name: String,
    version: String,
}

impl ToolEnumAdapter {
    fn new(tool: Tool) -> Self {
        let metadata = tool.metadata();
        Self {
            tool,
            name: metadata.info.name.clone(),
            version: metadata.version.clone(),
        }
    }
}

#[async_trait]
impl ToolNode for ToolEnumAdapter {
    fn name(&self) -> &str {
        // Return reference to stored name
        &self.name
    }

    fn version(&self) -> &str {
        // Return reference to stored version
        &self.version
    }

    fn description(&self) -> String {
        self.tool.metadata().info.description.clone()
    }

    fn definition(&self) -> crate::core::ToolInfo {
        self.tool.metadata().info.clone()
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // For now, just accept all parameters
        // In a full implementation, this would validate against the tool's schema
        let _ = params;
        Ok(())
    }

    async fn execute(&self, params: Value, context: crate::core::ExecutionContext) -> Result<Value> {
        let input = crate::tools::types::ToolInput::new(params);
        let output = self.tool.execute(input, context).await?;
        Ok(output.result)
    }
}
