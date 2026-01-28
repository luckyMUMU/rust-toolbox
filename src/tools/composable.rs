//! Composable Tool System for Workflow Toolkit
//!
//! This module provides a framework for building complex tools by composing
//! atomic components. It supports tool chaining, conditional execution,
//! and parallel processing.
//!
//! # Example
//!
//! ```rust
//! use workflow_toolkit::tools::composable::{
//!     ToolComposer, ToolChain, ConditionalTool, ParallelTools
//! };
//!
//! // Create a tool chain
//! let chain = ToolChain::new()
//!     .add_step("scan", scan_tool)
//!     .add_step("classify", classify_tool)
//!     .add_step("process", process_tool);
//!
//! // Create a conditional tool
//! let conditional = ConditionalTool::new()
//!     .condition("${score} > 0.8")
//!     .then_branch(high_confidence_tool)
//!     .else_branch(low_confidence_tool);
//!
//! // Compose into a complex tool
//! let composer = ToolComposer::new()
//!     .register("file_pipeline", Box::new(chain))
//!     .register("smart_processor", Box::new(conditional));
//! ```

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::tools::ToolNode;
use crate::workflow::el::{ExpressionContext, ExpressionEngine};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Trait for composable tools that can be combined into complex workflows
#[async_trait]
pub trait ComposableTool: Send + Sync {
    /// Execute the tool with given parameters
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;

    /// Get tool name
    fn name(&self) -> &str;

    /// Get tool description
    fn description(&self) -> String;

    /// Validate parameters
    fn validate_params(&self, _params: &Value) -> Result<()> {
        // Default implementation accepts any params
        Ok(())
    }
}

/// Tool chain for sequential execution
pub struct ToolChain {
    name: String,
    description: String,
    steps: Vec<(String, Arc<dyn ToolNode>)>,
    expression_engine: ExpressionEngine,
}

impl ToolChain {
    /// Create a new tool chain
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            steps: Vec::new(),
            expression_engine: ExpressionEngine::new(),
        }
    }

    /// Add a step to the chain
    pub fn add_step(mut self, name: impl Into<String>, tool: Arc<dyn ToolNode>) -> Self {
        self.steps.push((name.into(), tool));
        self
    }

    /// Get number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

#[async_trait]
impl ComposableTool for ToolChain {
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        info!("Executing tool chain '{}' with {} steps", self.name, self.steps.len());

        let mut current_params = params;
        let mut results = Vec::new();

        for (step_name, tool) in &self.steps {
            debug!("Executing step '{}'", step_name);

            // Execute the tool
            let result = tool.execute(current_params.clone(), context.clone()).await?;

            // Store result
            results.push(json!({
                "step": step_name,
                "result": result.clone()
            }));

            // Pass result to next step
            current_params = result;
        }

        Ok(json!({
            "chain_name": &self.name,
            "steps_executed": results.len(),
            "final_result": current_params,
            "step_results": results
        }))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> String {
        format!("{} (chain of {} tools)", self.description, self.steps.len())
    }
}

/// Conditional tool for branching execution
pub struct ConditionalTool {
    name: String,
    description: String,
    condition: String,
    then_branch: Arc<dyn ToolNode>,
    else_branch: Option<Arc<dyn ToolNode>>,
    expression_engine: ExpressionEngine,
}

impl ConditionalTool {
    /// Create a new conditional tool
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        condition: impl Into<String>,
        then_branch: Arc<dyn ToolNode>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            condition: condition.into(),
            then_branch,
            else_branch: None,
            expression_engine: ExpressionEngine::new(),
        }
    }

    /// Set the else branch
    pub fn with_else_branch(mut self, tool: Arc<dyn ToolNode>) -> Self {
        self.else_branch = Some(tool);
        self
    }
}

#[async_trait]
impl ComposableTool for ConditionalTool {
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        info!("Evaluating condition for tool '{}'", self.name);

        // Build expression context from params
        let mut expr_context = ExpressionContext::new();
        if let Value::Object(map) = &params {
            for (key, value) in map {
                expr_context.set(key, value.clone());
            }
        }

        // Evaluate condition
        let condition_result = self
            .expression_engine
            .evaluate_condition(&self.condition, &expr_context)
            .map_err(|e| WorkflowError::workflow_execution(format!("Condition evaluation failed: {}", e)))?;

        debug!("Condition '{}' evaluated to: {}", self.condition, condition_result);

        if condition_result {
            info!("Executing THEN branch");
            let result = self.then_branch.execute(params, context).await?;
            Ok(json!({
                "branch": "then",
                "condition": &self.condition,
                "result": result
            }))
        } else if let Some(else_tool) = &self.else_branch {
            info!("Executing ELSE branch");
            let result = else_tool.execute(params, context).await?;
            Ok(json!({
                "branch": "else",
                "condition": &self.condition,
                "result": result
            }))
        } else {
            Ok(json!({
                "branch": "none",
                "condition": &self.condition,
                "result": Value::Null
            }))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> String {
        format!("{} (conditional: {})", self.description, self.condition)
    }
}

/// Parallel tool execution
pub struct ParallelTools {
    name: String,
    description: String,
    tools: Vec<(String, Arc<dyn ToolNode>)>,
    max_concurrency: usize,
}

impl ParallelTools {
    /// Create new parallel tools container
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            tools: Vec::new(),
            max_concurrency: 4,
        }
    }

    /// Add a tool to the parallel set
    pub fn add_tool(mut self, name: impl Into<String>, tool: Arc<dyn ToolNode>) -> Self {
        self.tools.push((name.into(), tool));
        self
    }

    /// Set maximum concurrency
    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max;
        self
    }
}

#[async_trait]
impl ComposableTool for ParallelTools {
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        info!(
            "Executing {} tools in parallel (max concurrency: {})",
            self.tools.len(),
            self.max_concurrency
        );

        use std::sync::Arc;
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        let tools: Vec<(String, Arc<dyn ToolNode>)> = self.tools.clone();
        let mut handles = Vec::new();

        for (name, tool) in tools {
            let semaphore = semaphore.clone();
            let params = params.clone();
            let context = context.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                debug!("Executing parallel tool '{}'", name);

                match tool.execute(params, context).await {
                    Ok(result) => (name, json!({ "success": true, "result": result })),
                    Err(e) => (
                        name,
                        json!({
                            "success": false,
                            "error": e.to_string()
                        }),
                    ),
                }
            });

            handles.push(handle);
        }

        let mut results = HashMap::new();
        for handle in handles {
            if let Ok((name, result)) = handle.await {
                results.insert(name, result);
            }
        }

        Ok(json!({
            "parallel_execution": &self.name,
            "tools_executed": results.len(),
            "results": results
        }))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> String {
        format!("{} (parallel execution of {} tools)", self.description, self.tools.len())
    }
}

/// Tool composer for registering and managing composable tools
#[derive(Clone)]
pub struct ToolComposer {
    tools: HashMap<String, Arc<dyn ComposableTool>>,
}

impl ToolComposer {
    /// Create a new tool composer
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a composable tool
    pub fn register(mut self, name: impl Into<String>, tool: Arc<dyn ComposableTool>) -> Self {
        let name = name.into();
        info!("Registering composable tool: {}", name);
        self.tools.insert(name, tool);
        self
    }

    /// Get a registered tool
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn ComposableTool>> {
        self.tools.get(name).cloned()
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<(String, String)> {
        self.tools
            .iter()
            .map(|(name, tool)| (name.clone(), tool.description()))
            .collect()
    }

    /// Execute a registered tool
    pub async fn execute(
        &self,
        name: &str,
        params: Value,
        context: ExecutionContext,
    ) -> Result<Value> {
        let tool = self
            .tools
            .get(name)
            .cloned()
            .ok_or_else(|| WorkflowError::NotFound {
                resource: format!("composable tool '{}'", name),
            })?;

        tool.execute(params, context).await
    }
}

impl Default for ToolComposer {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter to make ComposableTool work as a ToolNode
pub struct ComposableToolAdapter {
    inner: Arc<dyn ComposableTool>,
    info: crate::core::ToolInfo,
}

impl ComposableToolAdapter {
    /// Create a new adapter
    pub fn new(inner: Arc<dyn ComposableTool>) -> Self {
        let info = crate::core::ToolInfo {
            name: inner.name().to_string(),
            version: "1.0.0".to_string(),
            description: inner.description(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            return_schema: serde_json::json!({}),
            category: Some("composable".to_string()),
            tags: vec!["composed".to_string()],
            dependencies: vec![],
            plugin_name: None,
            version_requirements: std::collections::HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Self { inner, info }
    }
}

#[async_trait]
impl ToolNode for ComposableToolAdapter {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> String {
        self.inner.description()
    }

    fn definition(&self) -> crate::core::ToolInfo {
        self.info.clone()
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        self.inner.validate_params(params)
    }

    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        self.inner.execute(params, context).await
    }

    fn get_info(&self) -> crate::core::ToolInfo {
        self.info.clone()
    }
}

/// Builder for creating complex tool compositions using a fluent API
pub struct ToolCompositionBuilder {
    composer: ToolComposer,
}

impl ToolCompositionBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            composer: ToolComposer::new(),
        }
    }

    /// Build a tool chain
    pub fn chain(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> ToolChainBuilder {
        ToolChainBuilder::new(self, name, description)
    }

    /// Build a conditional tool
    pub fn conditional(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        condition: impl Into<String>,
    ) -> ConditionalToolBuilder {
        ConditionalToolBuilder::new(self, name, description, condition)
    }

    /// Build parallel tools
    pub fn parallel(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> ParallelToolBuilder {
        ParallelToolBuilder::new(self, name, description)
    }

    /// Finalize and get the composer
    pub fn build(self) -> ToolComposer {
        self.composer
    }
}

impl Default for ToolCompositionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for tool chains
pub struct ToolChainBuilder<'a> {
    builder: &'a mut ToolCompositionBuilder,
    name: String,
    description: String,
    steps: Vec<(String, Arc<dyn ToolNode>)>,
}

impl<'a> ToolChainBuilder<'a> {
    fn new(
        builder: &'a mut ToolCompositionBuilder,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            builder,
            name: name.into(),
            description: description.into(),
            steps: Vec::new(),
        }
    }

    /// Add a step
    pub fn step(mut self, name: impl Into<String>, tool: Arc<dyn ToolNode>) -> Self {
        self.steps.push((name.into(), tool));
        self
    }

    /// Finalize and register
    pub fn register(self) -> &'a mut ToolCompositionBuilder {
        let chain = ToolChain {
            name: self.name.clone(),
            description: self.description,
            steps: self.steps,
            expression_engine: ExpressionEngine::new(),
        };

        self.builder.composer.tools.insert(self.name, Arc::new(chain));
        self.builder
    }
}

/// Builder for conditional tools
pub struct ConditionalToolBuilder<'a> {
    builder: &'a mut ToolCompositionBuilder,
    name: String,
    description: String,
    condition: String,
    then_branch: Option<Arc<dyn ToolNode>>,
    else_branch: Option<Arc<dyn ToolNode>>,
}

impl<'a> ConditionalToolBuilder<'a> {
    fn new(
        builder: &'a mut ToolCompositionBuilder,
        name: impl Into<String>,
        description: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        Self {
            builder,
            name: name.into(),
            description: description.into(),
            condition: condition.into(),
            then_branch: None,
            else_branch: None,
        }
    }

    /// Set the then branch
    pub fn then(mut self, tool: Arc<dyn ToolNode>) -> Self {
        self.then_branch = Some(tool);
        self
    }

    /// Set the else branch
    pub fn otherwise(mut self, tool: Arc<dyn ToolNode>) -> Self {
        self.else_branch = Some(tool);
        self
    }

    /// Finalize and register
    pub fn register(self) -> Result<&'a mut ToolCompositionBuilder> {
        let then_branch = self.then_branch.ok_or_else(|| {
            WorkflowError::workflow_validation("Conditional tool must have a 'then' branch")
        })?;

        let conditional = ConditionalTool {
            name: self.name.clone(),
            description: self.description,
            condition: self.condition,
            then_branch,
            else_branch: self.else_branch,
            expression_engine: ExpressionEngine::new(),
        };

        self.builder.composer.tools.insert(self.name, Arc::new(conditional));
        Ok(self.builder)
    }
}

/// Builder for parallel tools
pub struct ParallelToolBuilder<'a> {
    builder: &'a mut ToolCompositionBuilder,
    name: String,
    description: String,
    tools: Vec<(String, Arc<dyn ToolNode>)>,
    max_concurrency: usize,
}

impl<'a> ParallelToolBuilder<'a> {
    fn new(
        builder: &'a mut ToolCompositionBuilder,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            builder,
            name: name.into(),
            description: description.into(),
            tools: Vec::new(),
            max_concurrency: 4,
        }
    }

    /// Add a tool
    pub fn tool(mut self, name: impl Into<String>, tool: Arc<dyn ToolNode>) -> Self {
        self.tools.push((name.into(), tool));
        self
    }

    /// Set max concurrency
    pub fn max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max;
        self
    }

    /// Finalize and register
    pub fn register(self) -> &'a mut ToolCompositionBuilder {
        let parallel = ParallelTools {
            name: self.name.clone(),
            description: self.description,
            tools: self.tools,
            max_concurrency: self.max_concurrency,
        };

        self.builder.composer.tools.insert(self.name, Arc::new(parallel));
        self.builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::BasicTool;

    #[test]
    fn test_tool_chain_creation() {
        let chain = ToolChain::new("test_chain", "A test chain");
        assert_eq!(chain.name(), "test_chain");
        assert_eq!(chain.step_count(), 0);
    }

    #[test]
    fn test_tool_composer() {
        let composer = ToolComposer::new();
        assert!(composer.list_tools().is_empty());
    }

    #[test]
    fn test_composition_builder() {
        let mut builder = ToolCompositionBuilder::new();
        // Just test that builder can be created
        assert_eq!(builder.composer.list_tools().len(), 0);
    }
}
