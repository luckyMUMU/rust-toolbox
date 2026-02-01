//! Tool component implementation.
//!
//! Wraps tool execution as a workflow component.

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::tools::{TemplateContext, TemplateEngine, ToolRegistry};
use crate::workflow::component::{Component, ComponentOutput, ComponentType};
use crate::workflow::context::DataContext;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Tool component that executes a registered tool.
pub struct ToolComponent {
    /// Component ID
    id: String,

    /// Name of the tool to execute
    tool_name: String,

    /// Tool registry for looking up and executing tools
    tool_registry: Arc<dyn ToolRegistry>,

    /// Default parameters for the tool
    default_params: Value,

    /// Template engine for parameter expansion
    template_engine: Arc<TemplateEngine>,
}

impl ToolComponent {
    /// Create a new tool component.
    pub fn new(
        id: impl Into<String>,
        tool_name: impl Into<String>,
        tool_registry: Arc<dyn ToolRegistry>,
        default_params: Value,
    ) -> Self {
        // Initialize template engine
        // We ignore initialization errors and fallback to basic engine if it fails
        // In a real production system, we might want to propagate this error
        let template_engine = Arc::new(TemplateEngine::new().unwrap_or_else(|e| {
            tracing::error!("Failed to initialize template engine: {}", e);
            TemplateEngine::default()
        }));

        Self {
            id: id.into(),
            tool_name: tool_name.into(),
            tool_registry,
            default_params,
            template_engine,
        }
    }

    /// Get the tool name.
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }

    /// Merge default parameters with runtime parameters.
    ///
    /// Runtime parameters take precedence over default parameters.
    fn merge_params(&self, runtime_params: &Value) -> Value {
        if self.default_params.is_null() {
            return runtime_params.clone();
        }
        if runtime_params.is_null() {
            return self.default_params.clone();
        }

        // Merge objects
        if let (Some(default_obj), Some(runtime_obj)) =
            (self.default_params.as_object(), runtime_params.as_object())
        {
            let mut merged = default_obj.clone();
            for (key, value) in runtime_obj {
                merged.insert(key.clone(), value.clone());
            }
            return Value::Object(merged);
        }

        // If not both objects, prefer runtime params
        runtime_params.clone()
    }

    /// Resolve parameters using template engine
    fn resolve_parameters(&self, params: &Value, context: &DataContext) -> Result<Value> {
        // Create template context
        let mut template_context = TemplateContext::new();

        // Add all global slots to template context
        let global_slots = context.export_global_slots();

        // Special handling for input_params: flatten it into the root context
        if let Some(Value::Object(map)) = global_slots.get("input_params") {
            for (k, v) in map {
                template_context.set_variable(k.clone(), v.clone());
            }
        }

        // Add other global slots (overwriting input_params if name collision, or keeping them as objects)
        template_context.set_variables(global_slots);

        // Expand parameters
        self.template_engine
            .expand(params, &template_context)
            .map_err(|e| crate::error::WorkflowError::ParameterResolutionError(e.to_string()))
    }
}

#[async_trait]
impl Component for ToolComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn component_type(&self) -> ComponentType {
        ComponentType::Tool
    }

    async fn execute(
        &self,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput> {
        tracing::debug!(
            component_id = %self.id,
            tool_name = %self.tool_name,
            "Executing tool component"
        );

        // Get runtime parameters from context or use empty params
        let runtime_params = if context.has_global("input_params") {
            context
                .get_global_raw("input_params")
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        };

        // Merge with default parameters
        let params = self.merge_params(&runtime_params);

        // Resolve templates in parameters
        let resolved_params = self.resolve_parameters(&params, context)?;

        // Execute the tool
        let result = self
            .tool_registry
            .execute_tool(&self.tool_name, resolved_params, execution_ctx.clone())
            .await?;

        // Store the result in context
        context.store_node_output(&self.id, result.clone())?;

        tracing::debug!(
            component_id = %self.id,
            tool_name = %self.tool_name,
            "Tool execution completed"
        );

        Ok(ComponentOutput::success_with_result(result))
    }

    fn validate(&self) -> Result<()> {
        // Check if the tool exists
        if self.tool_registry.get_tool(&self.tool_name).is_none() {
            return Err(crate::error::WorkflowError::tool_not_found(&self.tool_name));
        }
        Ok(())
    }

    fn cacheable(&self) -> bool {
        // Tools are cacheable if they're deterministic
        // This could be configured per-tool in the future
        true
    }

    fn description(&self) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{BasicTool, BasicToolRegistry};
    use serde_json::json;
    use std::sync::Arc;

    fn create_test_registry() -> Arc<dyn crate::tools::ToolRegistry> {
        use crate::tools::registry::ToolRegistry;
        use crate::tools::types::{NativeToolBuilder, Tool};
        
        let registry = ToolRegistry::new();

        // Register a simple echo tool for testing
        let echo_tool = NativeToolBuilder::new()
            .name("echo")
            .version("1.0.0")
            .description("Echo tool for testing")
            .executor(|input, _ctx| async move { Ok(crate::tools::types::ToolOutput::success(input.params)) })
            .build()
            .unwrap();

        registry.register("echo", Tool::Native(Arc::new(echo_tool)));

        Arc::new(registry)
    }

    #[tokio::test]
    async fn test_tool_component_execution() {
        let registry = create_test_registry();

        let component = ToolComponent::new(
            "test-node",
            "echo",
            registry.clone(),
            json!({"key": "default"}),
        );

        let mut context = DataContext::new();
        context
            .set_global("input_params", json!({"input": "test"}))
            .unwrap();

        let exec_ctx = ExecutionContext::new().with_workflow_id(uuid::Uuid::new_v4());

        let result = component.execute(&mut context, &exec_ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.status.is_success());
        assert!(output.result.is_some());
    }

    #[test]
    fn test_param_merging() {
        let registry = create_test_registry();

        let component = ToolComponent::new("test-node", "echo", registry, json!({"a": 1, "b": 2}));

        // Runtime overrides default
        let merged = component.merge_params(&json!({"b": 3, "c": 4}));
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], 3);
        assert_eq!(merged["c"], 4);

        // Null runtime uses default
        let merged = component.merge_params(&Value::Null);
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], 2);
    }
}
