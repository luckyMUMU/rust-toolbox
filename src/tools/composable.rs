//! Composable Tool System for Workflow Toolkit
//!
//! NOTE: This module is being refactored as part of the radical optimization.
//! The old trait-based system is being replaced with an enum-based system.
//!
//! ComposableTool trait - REMOVED (replaced with ComposedTool enum variant in types.rs)

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::tools::types::{Tool, ToolId};
use crate::workflow::el::{ExpressionContext, ExpressionEngine};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

// MIGRATION: Old composable system replaced with new enum-based composition
// Use types::ComposedTool instead of the structs below

/// Tool chain for sequential execution (DEPRECATED - use ComposedTool::Chain)
///
/// NOTE: This is kept for backward compatibility during migration.
/// New code should use types::ComposedTool with CompositionType::Chain
pub struct ToolChain {
    name: String,
    description: String,
    steps: Vec<(String, ToolId)>,
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
    pub fn add_step(mut self, name: impl Into<String>, tool_id: ToolId) -> Self {
        self.steps.push((name.into(), tool_id));
        self
    }

    /// Get number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get chain name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get description
    pub fn description(&self) -> &str {
        &self.description
    }
}

// REMOVED: #[async_trait] impl ComposableTool for ToolChain
// Use types::ComposedTool::execute() instead

/// Conditional tool for branching execution (DEPRECATED - use ComposedTool::Conditional)
///
/// NOTE: This is kept for backward compatibility during migration.
/// New code should use types::ComposedTool with CompositionType::Conditional
pub struct ConditionalTool {
    name: String,
    description: String,
    condition: String,
    then_branch: ToolId,
    else_branch: Option<ToolId>,
    expression_engine: ExpressionEngine,
}

impl ConditionalTool {
    /// Create a new conditional tool
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        condition: impl Into<String>,
        then_branch: ToolId,
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
    pub fn with_else_branch(mut self, tool_id: ToolId) -> Self {
        self.else_branch = Some(tool_id);
        self
    }

    /// Get tool name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get condition
    pub fn condition(&self) -> &str {
        &self.condition
    }
}

// REMOVED: #[async_trait] impl ComposableTool for ConditionalTool
// Use types::ComposedTool::execute() instead

/// Parallel tool execution (DEPRECATED - use ComposedTool::Parallel)
///
/// NOTE: This is kept for backward compatibility during migration.
/// New code should use types::ComposedTool with CompositionType::Parallel
pub struct ParallelTools {
    name: String,
    description: String,
    tools: Vec<(String, ToolId)>,
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
    pub fn add_tool(mut self, name: impl Into<String>, tool_id: ToolId) -> Self {
        self.tools.push((name.into(), tool_id));
        self
    }

    /// Alias for add_tool for builder API consistency
    pub fn with_tool(self, name: impl Into<String>, tool_id: ToolId) -> Self {
        self.add_tool(name, tool_id)
    }

    /// Set maximum concurrency
    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max;
        self
    }

    /// Get tool name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get description
    pub fn description(&self) -> &str {
        &self.description
    }
}

// REMOVED: #[async_trait] impl ComposableTool for ParallelTools
// Use types::ComposedTool::execute() instead

/// Tool composer for registering and managing composable tools (DEPRECATED)
///
/// NOTE: This is kept for backward compatibility during migration.
/// New code should use ToolRegistry with types::ComposedTool
#[derive(Clone)]
pub struct ToolComposer {
    chains: HashMap<String, ToolChain>,
    conditionals: HashMap<String, ConditionalTool>,
    parallels: HashMap<String, ParallelTools>,
}

impl ToolComposer {
    /// Create a new tool composer
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
            conditionals: HashMap::new(),
            parallels: HashMap::new(),
        }
    }

    /// Register a tool chain
    pub fn register_chain(mut self, name: impl Into<String>, chain: ToolChain) -> Self {
        let name = name.into();
        info!("Registering tool chain: {}", name);
        self.chains.insert(name, chain);
        self
    }

    /// Register a conditional tool
    pub fn register_conditional(
        mut self,
        name: impl Into<String>,
        conditional: ConditionalTool,
    ) -> Self {
        let name = name.into();
        info!("Registering conditional tool: {}", name);
        self.conditionals.insert(name, conditional);
        self
    }

    /// Register parallel tools
    pub fn register_parallel(mut self, name: impl Into<String>, parallel: ParallelTools) -> Self {
        let name = name.into();
        info!("Registering parallel tools: {}", name);
        self.parallels.insert(name, parallel);
        self
    }

    /// Get a registered tool chain
    pub fn get_chain(&self, name: &str) -> Option<&ToolChain> {
        self.chains.get(name)
    }

    /// Get a registered conditional tool
    pub fn get_conditional(&self, name: &str) -> Option<&ConditionalTool> {
        self.conditionals.get(name)
    }

    /// Get registered parallel tools
    pub fn get_parallel(&self, name: &str) -> Option<&ParallelTools> {
        self.parallels.get(name)
    }

    /// List all registered chains
    pub fn list_chains(&self) -> Vec<(String, String)> {
        self.chains
            .iter()
            .map(|(name, chain)| (name.clone(), chain.description().to_string()))
            .collect()
    }
}

impl Default for ToolComposer {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter to convert old composable tools to new Tool enum (DEPRECATED)
///
/// NOTE: Use types::ComposedTool directly instead
pub struct ComposableToolAdapter;

impl ComposableToolAdapter {
    /// Create a new adapter (placeholder for compatibility)
    pub fn new() -> Self {
        Self
    }
}

// REMOVED: #[async_trait] impl ToolNode for ComposableToolAdapter
// ToolNode trait has been removed. Use types::Tool enum instead.

/// Builder for creating complex tool compositions using a fluent API (DEPRECATED)
///
/// NOTE: Use types::ComposedTool and ToolRegistry instead
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
    steps: Vec<(String, ToolId)>,
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
    pub fn step(mut self, name: impl Into<String>, tool_id: ToolId) -> Self {
        self.steps.push((name.into(), tool_id));
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

        self.builder.composer.register_chain(self.name, chain);
        self.builder
    }
}

/// Builder for conditional tools
pub struct ConditionalToolBuilder<'a> {
    builder: &'a mut ToolCompositionBuilder,
    name: String,
    description: String,
    condition: String,
    then_branch: Option<ToolId>,
    else_branch: Option<ToolId>,
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
    pub fn then(mut self, tool_id: ToolId) -> Self {
        self.then_branch = Some(tool_id);
        self
    }

    /// Set the else branch
    pub fn otherwise(mut self, tool_id: ToolId) -> Self {
        self.else_branch = Some(tool_id);
        self
    }

    /// Finalize and register
    pub fn register(self) -> Result<&'a mut ToolCompositionBuilder> {
        let then_branch = self.then_branch.ok_or_else(|| {
            WorkflowError::ValidationError("Conditional tool must have a 'then' branch".to_string())
        })?;

        let conditional = ConditionalTool {
            name: self.name.clone(),
            description: self.description,
            condition: self.condition,
            then_branch,
            else_branch: self.else_branch,
            expression_engine: ExpressionEngine::new(),
        };

        self.builder
            .composer
            .register_conditional(self.name, conditional);
        Ok(self.builder)
    }
}

/// Builder for parallel tools
pub struct ParallelToolBuilder<'a> {
    builder: &'a mut ToolCompositionBuilder,
    name: String,
    description: String,
    tools: Vec<(String, ToolId)>,
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
    pub fn tool(mut self, name: impl Into<String>, tool_id: ToolId) -> Self {
        self.tools.push((name.into(), tool_id));
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

        self.builder.composer.register_parallel(self.name, parallel);
        self.builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_chain_creation() {
        let chain = ToolChain::new("test_chain", "A test chain");
        assert_eq!(chain.name(), "test_chain");
        assert_eq!(chain.step_count(), 0);
    }

    #[test]
    fn test_tool_composer() {
        let composer = ToolComposer::new();
        assert!(composer.list_chains().is_empty());
    }

    #[test]
    fn test_composition_builder() {
        let mut builder = ToolCompositionBuilder::new();
        // Just test that builder can be created
        assert_eq!(builder.composer.list_chains().len(), 0);
    }
}
