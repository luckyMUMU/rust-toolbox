//! Workflow definition structures.
//!
//! This module defines the static structure of a workflow, including its nodes, edges,
//! and configuration. It provides a builder-like API for constructing workflows programmatically.
//!
//! # Example
//!
//! ```rust
//! use rust_tool_v2::workflow::{WorkflowDefinition, WorkflowNode, WorkflowEdge, NodeType};
//!
//! let mut workflow = WorkflowDefinition::new("my-workflow", "1.0.0");
//!
//! // Add nodes
//! workflow.add_node(WorkflowNode::tool("step1", "echo-tool")).unwrap();
//! workflow.add_node(WorkflowNode::tool("step2", "grep-tool")).unwrap();
//!
//! // Connect nodes
//! workflow.add_edge(WorkflowEdge::new("step1", "step2")).unwrap();
//!
//! // Validate
//! workflow.validate().unwrap();
//! ```

use crate::core::{RetryPolicy, WorkflowConfig, WorkflowId};
use crate::error::{Result, WorkflowError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use uuid::Uuid;

/// Workflow definition structure.
///
/// Represents the blueprint of a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Unique name of the workflow
    pub name: String,
    /// Semantic version of the workflow
    pub version: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Arbitrary metadata
    pub metadata: HashMap<String, Value>,
    /// List of nodes in the workflow graph
    pub nodes: Vec<WorkflowNode>,
    /// List of directed edges connecting nodes
    pub edges: Vec<WorkflowEdge>,
    /// Global configuration for this workflow
    pub global_config: WorkflowConfig,
}

impl WorkflowDefinition {
    /// Create a new workflow definition
    pub fn new<S: Into<String>>(name: S, version: S) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            metadata: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            global_config: WorkflowConfig::default(),
        }
    }

    /// Add a node to the workflow
    pub fn add_node(&mut self, node: WorkflowNode) -> Result<()> {
        // Check for duplicate node IDs
        if self.nodes.iter().any(|n| n.id == node.id) {
            return Err(WorkflowError::DuplicateNodeId(node.id).into());
        }
        self.nodes.push(node);
        Ok(())
    }

    /// Add an edge to the workflow
    pub fn add_edge(&mut self, edge: WorkflowEdge) -> Result<()> {
        // Validate that both nodes exist
        let node_ids: HashSet<_> = self.nodes.iter().map(|n| &n.id).collect();
        if !node_ids.contains(&edge.from) {
            return Err(WorkflowError::NodeNotFound(edge.from).into());
        }
        if !node_ids.contains(&edge.to) {
            return Err(WorkflowError::NodeNotFound(edge.to).into());
        }
        self.edges.push(edge);
        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get all nodes of a specific type
    pub fn get_nodes_by_type(&self, node_type: NodeType) -> Vec<&WorkflowNode> {
        self.nodes
            .iter()
            .filter(|n| n.node_type == node_type)
            .collect()
    }

    /// Get incoming edges for a node
    pub fn get_incoming_edges(&self, node_id: &str) -> Vec<&WorkflowEdge> {
        self.edges.iter().filter(|e| e.to == node_id).collect()
    }

    /// Get outgoing edges for a node
    pub fn get_outgoing_edges(&self, node_id: &str) -> Vec<&WorkflowEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }

    /// Get all root nodes (nodes with no incoming edges)
    pub fn get_root_nodes(&self) -> Vec<&WorkflowNode> {
        let nodes_with_incoming: HashSet<_> = self.edges.iter().map(|e| &e.to).collect();
        self.nodes
            .iter()
            .filter(|n| !nodes_with_incoming.contains(&n.id))
            .collect()
    }

    /// Get all leaf nodes (nodes with no outgoing edges)
    pub fn get_leaf_nodes(&self) -> Vec<&WorkflowNode> {
        let nodes_with_outgoing: HashSet<_> = self.edges.iter().map(|e| &e.from).collect();
        self.nodes
            .iter()
            .filter(|n| !nodes_with_outgoing.contains(&n.id))
            .collect()
    }

    /// Generate a unique workflow ID
    pub fn generate_id(&self) -> WorkflowId {
        Uuid::new_v4()
    }

    /// Validate the workflow definition
    pub fn validate(&self) -> Result<()> {
        // Basic validation - more comprehensive validation will be in the validator module
        if self.name.is_empty() {
            return Err(
                WorkflowError::InvalidWorkflowName("Name cannot be empty".to_string()).into(),
            );
        }

        if self.version.is_empty() {
            return Err(WorkflowError::InvalidWorkflowVersion(
                "Version cannot be empty".to_string(),
            )
            .into());
        }

        if self.nodes.is_empty() {
            return Err(WorkflowError::EmptyWorkflow.into());
        }

        // Check for duplicate node IDs
        let mut node_ids = HashSet::new();
        for node in &self.nodes {
            if !node_ids.insert(&node.id) {
                return Err(WorkflowError::DuplicateNodeId(node.id.clone()).into());
            }
        }

        // Validate edges reference existing nodes
        let node_id_set: HashSet<_> = self.nodes.iter().map(|n| &n.id).collect();
        for edge in &self.edges {
            if !node_id_set.contains(&edge.from) {
                return Err(WorkflowError::NodeNotFound(edge.from.clone()).into());
            }
            if !node_id_set.contains(&edge.to) {
                return Err(WorkflowError::NodeNotFound(edge.to.clone()).into());
            }
        }

        Ok(())
    }
}

/// Workflow node definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub node_type: NodeType,
    pub tool_name: Option<String>,
    pub parameters: Value,
    pub retry_policy: Option<RetryPolicy>,
    pub timeout: Option<Duration>,
    pub metadata: HashMap<String, Value>,
    pub depends_on: Vec<String>, // Explicit dependencies beyond edges
}

impl WorkflowNode {
    /// Create a new workflow node
    pub fn new<S: Into<String>>(id: S, node_type: NodeType) -> Self {
        Self {
            id: id.into(),
            node_type,
            tool_name: None,
            parameters: Value::Null,
            retry_policy: None,
            timeout: None,
            metadata: HashMap::new(),
            depends_on: Vec::new(),
        }
    }

    /// Create a tool node
    pub fn tool<S: Into<String>>(id: S, tool_name: S) -> Self {
        Self {
            id: id.into(),
            node_type: NodeType::Tool,
            tool_name: Some(tool_name.into()),
            parameters: Value::Null,
            retry_policy: None,
            timeout: None,
            metadata: HashMap::new(),
            depends_on: Vec::new(),
        }
    }

    /// Set parameters for the node
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set retry policy for the node
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    /// Set timeout for the node
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add a dependency
    pub fn with_dependency<S: Into<String>>(mut self, dependency: S) -> Self {
        self.depends_on.push(dependency.into());
        self
    }

    /// Check if this node is a tool node
    pub fn is_tool(&self) -> bool {
        matches!(self.node_type, NodeType::Tool)
    }

    /// Check if this node is a control flow node
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self.node_type,
            NodeType::Condition | NodeType::Loop | NodeType::Parallel
        )
    }

    /// Validate the node
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(WorkflowError::InvalidNodeId("Node ID cannot be empty".to_string()).into());
        }

        // Tool nodes must have a tool name
        if self.node_type == NodeType::Tool && self.tool_name.is_none() {
            return Err(WorkflowError::MissingToolName(self.id.clone()).into());
        }

        Ok(())
    }
}

/// Node type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Tool execution node
    Tool,
    /// Conditional branching node
    Condition,
    /// Loop execution node
    Loop,
    /// Parallel execution node
    Parallel,
    /// Checkpoint node for state saving
    Checkpoint,
}

impl NodeType {
    /// Check if this node type can have children
    pub fn can_have_children(&self) -> bool {
        matches!(self, Self::Loop | Self::Parallel)
    }

    /// Check if this node type requires a condition
    pub fn requires_condition(&self) -> bool {
        matches!(self, Self::Condition)
    }
}

/// Workflow edge definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
    pub weight: Option<f64>,
    pub metadata: HashMap<String, Value>,
}

impl WorkflowEdge {
    /// Create a new workflow edge
    pub fn new<S: Into<String>>(from: S, to: S) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            condition: None,
            weight: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a conditional edge
    pub fn conditional<S: Into<String>>(from: S, to: S, condition: S) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            condition: Some(condition.into()),
            weight: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the weight of the edge
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Check if this is a conditional edge
    pub fn is_conditional(&self) -> bool {
        self.condition.is_some()
    }
}

/// Workflow template for creating common workflow patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<TemplateParameter>,
    pub template: WorkflowDefinition,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub parameter_type: ParameterType,
    pub default_value: Option<Value>,
    pub required: bool,
}

/// Parameter type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}
