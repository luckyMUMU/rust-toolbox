//! Component system for workflow nodes.
//!
//! This module provides the core abstraction for workflow components,
//! following the LiteFlow design principle: "All logic is a component".
//!
//! Each node type (Tool, Condition, Loop, Parallel, etc.) implements
//! the `Component` trait, ensuring single responsibility and clear separation
//! of concerns.

pub mod parallel;
pub mod registry;
pub mod tool;

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::workflow::context::DataContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Component type enumeration.
///
/// Defines the different types of components that can exist in a workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    /// Tool execution component - executes a registered tool
    Tool,
    /// Condition component - evaluates expressions for branching
    Condition,
    /// Loop component - iterates over collections or conditions
    Loop,
    /// Parallel component - executes multiple nodes concurrently
    Parallel,
    /// Switch component - multi-way branching based on expression value
    Switch,
    /// Checkpoint component - saves execution state for recovery
    Checkpoint,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentType::Tool => write!(f, "Tool"),
            ComponentType::Condition => write!(f, "Condition"),
            ComponentType::Loop => write!(f, "Loop"),
            ComponentType::Parallel => write!(f, "Parallel"),
            ComponentType::Switch => write!(f, "Switch"),
            ComponentType::Checkpoint => write!(f, "Checkpoint"),
        }
    }
}

/// Component execution status.
///
/// Indicates the outcome of a component's execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentStatus {
    /// Component executed successfully
    Success,
    /// Component execution failed with an error message
    Failure(String),
    /// Component was skipped (e.g., condition evaluated to false)
    Skip,
    /// Break out of a loop
    Break,
    /// Continue to next iteration of a loop
    Continue,
}

impl ComponentStatus {
    /// Check if the status indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, ComponentStatus::Success)
    }

    /// Check if the status indicates failure
    pub fn is_failure(&self) -> bool {
        matches!(self, ComponentStatus::Failure(_))
    }

    /// Check if the status indicates the component was skipped
    pub fn is_skip(&self) -> bool {
        matches!(self, ComponentStatus::Skip)
    }
}

/// Output from component execution.
///
/// Contains the execution status, dynamically determined next nodes,
/// and any metadata produced during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOutput {
    /// Execution status
    pub status: ComponentStatus,
    /// Nodes to execute next (for dynamic control flow)
    pub next_nodes: Vec<String>,
    /// Result value (if any)
    pub result: Option<Value>,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

impl ComponentOutput {
    /// Create a successful output with no specific next nodes
    pub fn success() -> Self {
        Self {
            status: ComponentStatus::Success,
            next_nodes: Vec::new(),
            result: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a successful output with a result value
    pub fn success_with_result(result: Value) -> Self {
        Self {
            status: ComponentStatus::Success,
            next_nodes: Vec::new(),
            result: Some(result),
            metadata: HashMap::new(),
        }
    }

    /// Create a successful output with specific next nodes
    pub fn success_with_next(next_nodes: Vec<String>) -> Self {
        Self {
            status: ComponentStatus::Success,
            next_nodes,
            result: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a failure output
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            status: ComponentStatus::Failure(message.into()),
            next_nodes: Vec::new(),
            result: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a skip output
    pub fn skip() -> Self {
        Self {
            status: ComponentStatus::Skip,
            next_nodes: Vec::new(),
            result: None,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the output
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set the result value
    pub fn with_result(mut self, result: Value) -> Self {
        self.result = Some(result);
        self
    }
}

impl Default for ComponentOutput {
    fn default() -> Self {
        Self::success()
    }
}

/// Core component trait.
///
/// All workflow node types must implement this trait.
/// Following the single responsibility principle, each component
/// only handles its own execution logic.
///
/// # Example
///
/// ```ignore
/// use workflow_toolkit::workflow::component::{Component, ComponentOutput, ComponentType};
///
/// struct MyComponent {
///     id: String,
/// }
///
/// #[async_trait]
/// impl Component for MyComponent {
///     fn id(&self) -> &str {
///         &self.id
///     }
///
///     fn component_type(&self) -> ComponentType {
///         ComponentType::Tool
///     }
///
///     async fn execute(
///         &self,
///         context: &mut DataContext,
///         execution_ctx: &ExecutionContext,
///     ) -> Result<ComponentOutput> {
///         // Component logic here
///         Ok(ComponentOutput::success())
///     }
/// }
/// ```
#[async_trait]
pub trait Component: Send + Sync {
    /// Returns the unique identifier for this component
    fn id(&self) -> &str;

    /// Returns the type of this component
    fn component_type(&self) -> ComponentType;

    /// Execute the component logic.
    ///
    /// # Arguments
    ///
    /// * `context` - Mutable data context for reading/writing slot values
    /// * `execution_ctx` - Execution context with workflow metadata
    ///
    /// # Returns
    ///
    /// A `ComponentOutput` containing the execution status and any dynamic next nodes
    async fn execute(
        &self,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput>;

    /// Validate the component configuration.
    ///
    /// Called during workflow validation phase before execution.
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Check if this component's results can be cached.
    ///
    /// Returns `true` if the component produces deterministic output
    /// for the same input parameters.
    fn cacheable(&self) -> bool {
        false
    }

    /// Get the component's description for documentation/debugging
    fn description(&self) -> Option<&str> {
        None
    }
}

// Re-exports
pub use parallel::{ParallelComponent, WaitStrategy};
pub use registry::ComponentRegistry;
pub use tool::ToolComponent;
