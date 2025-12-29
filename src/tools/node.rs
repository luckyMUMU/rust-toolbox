//! Tool node trait and implementations

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::tools::{ParameterTemplate, TemplateContext, TemplateEngine};
use async_trait::async_trait;
use chrono::Utc;
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for tool nodes
#[async_trait]
pub trait ToolNode: Send + Sync {
    /// Get the tool name
    fn name(&self) -> &str;
    
    /// Get the tool version
    fn version(&self) -> &str;
    
    /// Validate input parameters against the tool's schema
    fn validate_parameters(&self, params: &Value) -> Result<()>;
    
    /// Execute the tool with given parameters and context
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;
    
    /// Get tool information
    fn get_info(&self) -> ToolInfo;
    
    /// Get plugin information if this tool belongs to a plugin
    fn get_plugin_info(&self) -> Option<&PluginInfo>;
    
    /// Get parameter templates for this tool
    fn get_parameter_templates(&self) -> Vec<ParameterTemplate> {
        Vec::new() // Default implementation returns no templates
    }
    
    /// Expand parameters using templates and context
    fn expand_parameters(&self, params: Value, context: &TemplateContext) -> Result<Value> {
        // Default implementation uses template engine for basic expansion
        let engine = TemplateEngine::new()?;
        engine.expand(&params, context)
    }
}

/// Trait for tool executors - separates execution logic from tool metadata
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute the tool logic
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;
    
    /// Validate parameters (optional, can use schema validation)
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        // Default implementation does no validation
        Ok(())
    }
}

/// Basic tool implementation
pub struct BasicTool {
    info: ToolInfo,
    executor: Arc<dyn ToolExecutor>,
    plugin_info: Option<PluginInfo>,
    parameter_schema: Option<JSONSchema>,
    parameter_templates: Vec<ParameterTemplate>,
}

impl BasicTool {
    /// Create a new basic tool
    pub fn new(
        info: ToolInfo,
        executor: Arc<dyn ToolExecutor>,
        plugin_info: Option<PluginInfo>,
    ) -> Result<Self> {
        let parameter_schema = if info.parameters_schema != Value::Null {
            Some(
                JSONSchema::options()
                    .with_draft(Draft::Draft7)
                    .compile(&info.parameters_schema)
                    .map_err(|e| WorkflowError::ValidationError(format!("Invalid parameter schema: {}", e)))?,
            )
        } else {
            None
        };

        Ok(Self {
            info,
            executor,
            plugin_info,
            parameter_schema,
            parameter_templates: Vec::new(),
        })
    }
    
    /// Create a builder for basic tools
    pub fn builder() -> BasicToolBuilder {
        BasicToolBuilder::new()
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
        // First use JSON schema validation if available
        if let Some(schema) = &self.parameter_schema {
            if let Err(errors) = schema.validate(params) {
                let error_messages: Vec<String> = errors
                    .map(|e| format!("{}: {}", e.instance_path, e))
                    .collect();
                return Err(WorkflowError::ValidationError(format!(
                    "Parameter validation failed: {}",
                    error_messages.join(", ")
                )));
            }
        }
        
        // Then use custom validation from executor
        self.executor.validate_parameters(params)
    }
    
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        // Validate parameters before execution
        self.validate_parameters(&params)?;
        
        // Execute the tool
        self.executor.execute(params, context).await
    }
    
    fn get_info(&self) -> ToolInfo {
        self.info.clone()
    }
    
    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        self.plugin_info.as_ref()
    }
    
    fn get_parameter_templates(&self) -> Vec<ParameterTemplate> {
        self.parameter_templates.clone()
    }
    
    fn expand_parameters(&self, params: Value, context: &TemplateContext) -> Result<Value> {
        let engine = TemplateEngine::new()?;
        engine.expand(&params, context)
    }
}

/// Builder for BasicTool
pub struct BasicToolBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    parameters_schema: Value,
    return_schema: Value,
    plugin_name: Option<String>,
    dependencies: Vec<String>,
    version_requirements: HashMap<String, String>,
    parameter_templates: Vec<ParameterTemplate>,
    executor: Option<Arc<dyn ToolExecutor>>,
    plugin_info: Option<PluginInfo>,
}

impl BasicToolBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            category: None,
            tags: Vec::new(),
            parameters_schema: Value::Null,
            return_schema: Value::Null,
            plugin_name: None,
            dependencies: Vec::new(),
            version_requirements: HashMap::new(),
            parameter_templates: Vec::new(),
            executor: None,
            plugin_info: None,
        }
    }
    
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }
    
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    
    pub fn category<S: Into<String>>(mut self, category: S) -> Self {
        self.category = Some(category.into());
        self
    }
    
    pub fn tags<I: IntoIterator<Item = S>, S: Into<String>>(mut self, tags: I) -> Self {
        self.tags = tags.into_iter().map(|s| s.into()).collect();
        self
    }
    
    pub fn parameters_schema(mut self, schema: Value) -> Self {
        self.parameters_schema = schema;
        self
    }
    
    pub fn return_schema(mut self, schema: Value) -> Self {
        self.return_schema = schema;
        self
    }
    
    pub fn plugin_name<S: Into<String>>(mut self, plugin_name: S) -> Self {
        self.plugin_name = Some(plugin_name.into());
        self
    }
    
    pub fn executor(mut self, executor: Arc<dyn ToolExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }
    
    pub fn plugin_info(mut self, plugin_info: PluginInfo) -> Self {
        self.plugin_info = Some(plugin_info);
        self
    }
    
    pub fn dependencies<I: IntoIterator<Item = S>, S: Into<String>>(mut self, dependencies: I) -> Self {
        self.dependencies = dependencies.into_iter().map(|s| s.into()).collect();
        self
    }
    
    pub fn version_requirement<S1: Into<String>, S2: Into<String>>(mut self, tool_name: S1, requirement: S2) -> Self {
        self.version_requirements.insert(tool_name.into(), requirement.into());
        self
    }
    
    pub fn version_requirements<I, S1, S2>(mut self, requirements: I) -> Self 
    where
        I: IntoIterator<Item = (S1, S2)>,
        S1: Into<String>,
        S2: Into<String>,
    {
        self.version_requirements = requirements.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
    
    pub fn parameter_template(mut self, template: ParameterTemplate) -> Self {
        self.parameter_templates.push(template);
        self
    }
    
    pub fn parameter_templates<I: IntoIterator<Item = ParameterTemplate>>(mut self, templates: I) -> Self {
        self.parameter_templates = templates.into_iter().collect();
        self
    }
    
    pub fn build(self) -> Result<BasicTool> {
        let name = self.name.ok_or_else(|| WorkflowError::ValidationError("Tool name is required".to_string()))?;
        let version = self.version.ok_or_else(|| WorkflowError::ValidationError("Tool version is required".to_string()))?;
        let description = self.description.unwrap_or_else(|| format!("Tool: {}", name));
        let executor = self.executor.ok_or_else(|| WorkflowError::ValidationError("Tool executor is required".to_string()))?;
        
        let now = Utc::now();
        let info = ToolInfo {
            name,
            version,
            description,
            category: self.category,
            tags: self.tags,
            parameters_schema: self.parameters_schema,
            return_schema: self.return_schema,
            plugin_name: self.plugin_name,
            dependencies: self.dependencies,
            version_requirements: self.version_requirements,
            created_at: now,
            updated_at: now,
        };
        
        BasicTool::new(info, executor, self.plugin_info)
            .map(|mut tool| {
                tool.parameter_templates = self.parameter_templates;
                tool
            })
    }
}

impl Default for BasicToolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple function-based tool executor
pub struct FunctionExecutor<F>
where
    F: Fn(Value, ExecutionContext) -> Result<Value> + Send + Sync,
{
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
        (self.func)(params, context)
    }
}

/// Async function-based tool executor
pub struct AsyncFunctionExecutor<F, Fut>
where
    F: Fn(Value, ExecutionContext) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Value>> + Send,
{
    func: F,
}

impl<F, Fut> AsyncFunctionExecutor<F, Fut>
where
    F: Fn(Value, ExecutionContext) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Value>> + Send,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F, Fut> ToolExecutor for AsyncFunctionExecutor<F, Fut>
where
    F: Fn(Value, ExecutionContext) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Value>> + Send,
{
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        (self.func)(params, context).await
    }
}