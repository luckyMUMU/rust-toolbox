//! Recursive FlowNode definition for R-Flow v2.1.
//!
//! This module defines the recursive tree structure used for workflow execution.
//! Unlike the graph-based `WorkflowDefinition`, `FlowNode` represents a fully resolved
//! execution plan where control flow is explicit.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Recursive workflow node structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowNode {
    /// Sequential execution of multiple nodes.
    Chain(Vec<FlowNode>),
    
    /// Parallel execution of multiple nodes.
    Parallel(Vec<FlowNode>),
    
    /// A leaf node executing a tool.
    Tool {
        id: String,
        tool_name: String,
        params: Value,
    },
    
    /// Conditional branching (Switch/Case).
    Switch {
        /// Expression to evaluate.
        condition: String,
        /// Map of values to nodes.
        cases: Vec<(String, FlowNode)>,
        /// Default path if no case matches.
        default: Box<FlowNode>,
    },
    
    /// Loop execution.
    Loop {
        /// Condition to continue looping.
        condition: String,
        /// Body of the loop.
        body: Box<FlowNode>,
    },

    /// Empty node (no-op).
    Empty,
}

impl FlowNode {
    /// Check if the node is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, FlowNode::Empty)
    }

    /// Create a tool node
    pub fn tool(id: impl Into<String>, tool_name: impl Into<String>, params: Value) -> Self {
        Self::Tool {
            id: id.into(),
            tool_name: tool_name.into(),
            params,
        }
    }

    /// Create a chain node
    pub fn chain(nodes: Vec<FlowNode>) -> Self {
        if nodes.is_empty() {
            Self::Empty
        } else if nodes.len() == 1 {
            nodes[0].clone()
        } else {
            Self::Chain(nodes)
        }
    }
}
