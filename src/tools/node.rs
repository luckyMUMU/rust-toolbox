//! Tool node definitions.
//! 
//! NOTE: This module is being refactored as part of the radical optimization.
//! The old trait-based system is being replaced with an enum-based system.

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::Result;
use crate::tools::TemplateContext;
use serde_json::Value;
use std::sync::Arc;

// TODO: Remove old traits and replace with enum-based system
// ToolNode trait - REMOVED (replaced by Tool enum)
// ToolExecutor trait - REMOVED (execution logic moved to specific tool types)

/// Basic tool implementation using a builder pattern
/// 
/// NOTE: This will be replaced with NativeTool in the new system
pub struct BasicTool {
    info: ToolInfo,
    executor: Arc<dyn ToolExecutor>,
    plugin_info: Option<PluginInfo>,
}

impl BasicTool {
    /// Create a new builder for BasicTool
    pub fn builder() -> BasicToolBuilder {
        BasicToolBuilder::new()
    }

    /// Create a new BasicTool
    pub fn new(
        info: ToolInfo,
        executor: Arc<dyn ToolExecutor>,
        plugin_info: Option<PluginInfo>,
    ) -> Result<Self> {
        Ok(Self {
            info,
            executor,
            plugin_info,
        })
    }
}

#[async_trait]
impl ToolNode for BasicTool {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn version(&self) -> &str {
        &self.info.version
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        self.executor.validate_parameters(params)
    }

    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        self.executor.execute(params, context).await
    }

    fn get_info(&self) -> ToolInfo {
        self.info.clone()
    }

    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        self.plugin_info.as_ref()
    }
}

/// Builder for BasicTool
pub struct BasicToolBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    executor: Option<Arc<dyn ToolExecutor>>,
    plugin_info: Option<PluginInfo>,
    category: Option<String>,
    tags: Vec<String>,
    parameters_schema: Option<Value>,
    return_schema: Value,
}

impl BasicToolBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            executor: None,
            plugin_info: None,
            category: None,
            tags: Vec::new(),
            parameters_schema: None,
            return_schema: Value::Null,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn executor(mut self, executor: Arc<dyn ToolExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    pub fn plugin_info(mut self, info: PluginInfo) -> Self {
        self.plugin_info = Some(info);
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn parameters_schema(mut self, schema: Value) -> Self {
        self.parameters_schema = Some(schema);
        self
    }

    pub fn return_schema(mut self, schema: Value) -> Self {
        self.return_schema = schema;
        self
    }

    pub fn build(self) -> Result<BasicTool> {
        let name = self.name.ok_or_else(|| {
            crate::error::WorkflowError::ValidationError("Tool name is required".to_string())
        })?;
        let version = self.version.unwrap_or_else(|| "1.0.0".to_string());
        let executor = self.executor.ok_or_else(|| {
            crate::error::WorkflowError::ValidationError("Tool executor is required".to_string())
        })?;

        let info = ToolInfo {
            name,
            version,
            description: self.description.unwrap_or_default(),
            parameters_schema: self.parameters_schema.unwrap_or(Value::Null),
            return_schema: self.return_schema,
            category: self.category,
            tags: self.tags,
            dependencies: Vec::new(),
            plugin_name: None,
            version_requirements: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Ok(BasicTool {
            info,
            executor,
            plugin_info: self.plugin_info,
        })
    }
}

impl Default for BasicToolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for creating async function executors
pub struct AsyncFunctionExecutor<F> {
    func: F,
}

impl<F> AsyncFunctionExecutor<F>
where
    F: Fn(Value, ExecutionContext) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> ToolExecutor for AsyncFunctionExecutor<F>
where
    F: Fn(Value, ExecutionContext) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
{
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        (self.func)(params, context).await
    }
}

/// Helper for synchronous function executors
pub struct FunctionExecutor<F> {
    func: F,
}

impl<F> FunctionExecutor<F>
where
    F: Fn(Value, ExecutionContext) -> Result<Value> + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> ToolExecutor for FunctionExecutor<F>
where
    F: Fn(Value, ExecutionContext) -> Result<Value> + Send + Sync,
{
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        // Execute synchronous function
        (self.func)(params, context)
    }
}
