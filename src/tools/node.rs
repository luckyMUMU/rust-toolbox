//! Tool node definitions.
//! 
//! NOTE: This module is being refactored as part of the radical optimization.
//! The old trait-based system is being replaced with an enum-based system.

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::Result;
use crate::tools::types::{NativeTool, NativeToolBuilder, ToolInput, ToolOutput};
use crate::tools::TemplateContext;
use futures::future::BoxFuture;
use serde_json::Value;
use std::sync::Arc;

// MIGRATION: ToolNode trait and ToolExecutor trait have been REMOVED
// Use types::Tool enum and types::NativeTool instead

/// Basic tool implementation using a builder pattern (DEPRECATED)
/// 
/// NOTE: This is kept for backward compatibility during migration.
/// New code should use types::NativeTool and NativeToolBuilder
pub struct BasicTool {
    info: ToolInfo,
    executor: Arc<dyn Fn(ToolInput, ExecutionContext) -> BoxFuture<'static, Result<ToolOutput>> + Send + Sync>,
    plugin_info: Option<PluginInfo>,
}

impl BasicTool {
    /// Create a new builder for BasicTool
    pub fn builder() -> BasicToolBuilder {
        BasicToolBuilder::new()
    }

    /// Create a new BasicTool with a closure-based executor
    pub fn new<F, Fut>(
        info: ToolInfo,
        executor: F,
        plugin_info: Option<PluginInfo>,
    ) -> Result<Self>
    where
        F: Fn(ToolInput, ExecutionContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<ToolOutput>> + Send + 'static,
    {
        Ok(Self {
            info,
            executor: Arc::new(move |input, ctx| Box::pin(executor(input, ctx))),
            plugin_info,
        })
    }

    /// Create a new BasicTool from an Arc<dyn ToolExecutor> (COMPATIBILITY)
    /// 
    /// NOTE: This is for backward compatibility only. New code should use the closure-based new().
    pub fn from_executor(
        info: ToolInfo,
        executor: Arc<dyn crate::tools::compat::ToolExecutor>,
        plugin_info: Option<PluginInfo>,
    ) -> Result<Self> {
        let executor_wrapper = move |input: ToolInput, ctx: ExecutionContext| {
            let executor = executor.clone();
            async move {
                executor.execute(input.params, ctx).await.map(|v| ToolOutput::success(v))
            }
        };
        
        Ok(Self {
            info,
            executor: Arc::new(move |input, ctx| Box::pin(executor_wrapper(input, ctx))),
            plugin_info,
        })
    }

    /// Get tool name
    pub fn name(&self) -> &str {
        &self.info.name
    }

    /// Get tool version
    pub fn version(&self) -> &str {
        &self.info.version
    }

    /// Get tool info
    pub fn get_info(&self) -> ToolInfo {
        self.info.clone()
    }

    /// Get plugin info
    pub fn get_plugin_info(&self) -> Option<&PluginInfo> {
        self.plugin_info.as_ref()
    }

    /// Execute the tool
    pub async fn execute(&self, input: ToolInput, ctx: ExecutionContext) -> Result<ToolOutput> {
        (self.executor)(input, ctx).await
    }
}

// Implement ToolNode trait for backward compatibility
#[async_trait::async_trait]
impl crate::tools::compat::ToolNode for BasicTool {
    fn name(&self) -> &str {
        &self.info.name
    }
    
    fn version(&self) -> &str {
        &self.info.version
    }
    
    fn description(&self) -> String {
        self.info.description.clone()
    }
    
    fn definition(&self) -> ToolInfo {
        self.info.clone()
    }
    
    fn validate_parameters(&self, params: &Value) -> Result<()> {
        let _ = params;
        Ok(())
    }
    
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        let input = ToolInput::new(params);
        let output = self.execute(input, context).await?;
        Ok(output.result)
    }
    
    fn get_info(&self) -> ToolInfo {
        self.info.clone()
    }
    
    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        self.plugin_info.as_ref()
    }
}

/// Builder for BasicTool (DEPRECATED)
/// 
/// NOTE: Use types::NativeToolBuilder instead
pub struct BasicToolBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    executor: Option<Arc<dyn Fn(ToolInput, ExecutionContext) -> BoxFuture<'static, Result<ToolOutput>> + Send + Sync>>,
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

    pub fn executor<F, Fut>(mut self, executor: F) -> Self
    where
        F: Fn(ToolInput, ExecutionContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<ToolOutput>> + Send + 'static,
    {
        self.executor = Some(Arc::new(move |input, ctx| Box::pin(executor(input, ctx))));
        self
    }

    /// Set executor from Arc<dyn ToolExecutor> (COMPATIBILITY)
    ///
    /// NOTE: This is for backward compatibility only. New code should use the closure-based executor().
    pub fn executor_arc(mut self, executor: Arc<dyn crate::tools::compat::ToolExecutor>) -> Self {
        let executor_wrapper = move |input: ToolInput, ctx: ExecutionContext| {
            let executor = executor.clone();
            async move {
                executor.execute(input.params, ctx).await.map(|v| ToolOutput::success(v))
            }
        };
        self.executor = Some(Arc::new(move |input, ctx| Box::pin(executor_wrapper(input, ctx))));
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

/// Helper for creating async function executors (DEPRECATED)
/// 
/// NOTE: Use NativeToolBuilder::executor() instead
pub struct AsyncFunctionExecutor<F> {
    func: F,
}

impl<F> AsyncFunctionExecutor<F>
where
    F: Fn(Value, ExecutionContext) -> BoxFuture<'static, Result<Value>>
        + Send
        + Sync,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }

    pub async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        (self.func)(params, context).await
    }
}

// REMOVED: #[async_trait] impl ToolExecutor for AsyncFunctionExecutor
// Use NativeToolBuilder::executor() instead

/// Helper for synchronous function executors (DEPRECATED)
/// 
/// NOTE: Use NativeToolBuilder::executor() with async block instead
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

    pub async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        (self.func)(params, context)
    }
}

// REMOVED: #[async_trait] impl ToolExecutor for FunctionExecutor
// Use NativeToolBuilder::executor() instead
