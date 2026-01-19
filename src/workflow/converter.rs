//! Workflow converter to transform graph-based definitions to tree-based FlowNodes.

use crate::error::Result;
use crate::workflow::definition::{NodeType, WorkflowDefinition};
use crate::workflow::flow_node::FlowNode;
use crate::workflow::scheduler::DagScheduler;

/// Converter from WorkflowDefinition to FlowNode.
pub struct WorkflowConverter;

impl WorkflowConverter {
    /// Convert a workflow definition to a FlowNode execution tree.
    pub fn convert(definition: &WorkflowDefinition) -> Result<FlowNode> {
        // Use DagScheduler to get execution levels (topological sort with parallelism)
        let scheduler = DagScheduler::from_workflow(definition)?;
        let schedule = scheduler.generate_schedule()?;

        let mut chain_nodes = Vec::new();

        for level in schedule.execution_order {
            if level.is_empty() {
                continue;
            }

            if level.len() == 1 {
                // Sequential node
                let node_id = &level[0];
                if let Some(flow_node) = Self::create_node(definition, node_id)? {
                    chain_nodes.push(flow_node);
                }
            } else {
                // Parallel nodes
                let mut parallel_nodes = Vec::new();
                for node_id in level {
                    if let Some(flow_node) = Self::create_node(definition, &node_id)? {
                        parallel_nodes.push(flow_node);
                    }
                }

                if !parallel_nodes.is_empty() {
                    if parallel_nodes.len() == 1 {
                        chain_nodes.push(parallel_nodes[0].clone());
                    } else {
                        chain_nodes.push(FlowNode::Parallel(parallel_nodes));
                    }
                }
            }
        }

        // Optimize the chain
        if chain_nodes.is_empty() {
            Ok(FlowNode::Empty)
        } else if chain_nodes.len() == 1 {
            Ok(chain_nodes[0].clone())
        } else {
            Ok(FlowNode::Chain(chain_nodes))
        }
    }

    fn create_node(definition: &WorkflowDefinition, node_id: &str) -> Result<Option<FlowNode>> {
        let node = definition
            .get_node(node_id)
            .ok_or_else(|| crate::error::WorkflowError::NodeNotFound(node_id.to_string()))?;

        match node.node_type {
            NodeType::Tool => {
                let tool_name = node.tool_name.clone().ok_or_else(|| {
                    crate::error::WorkflowError::MissingToolName(node_id.to_string())
                })?;

                Ok(Some(FlowNode::tool(
                    node.id.clone(),
                    tool_name,
                    node.parameters.clone(),
                )))
            }
            NodeType::Condition => {
                // For now, we treat condition nodes as tools if they have tool_name,
                // or we need to parse their parameters to create Switch flow nodes.
                // This is a simplified implementation.
                if let Some(tool_name) = &node.tool_name {
                    Ok(Some(FlowNode::tool(
                        node.id.clone(),
                        tool_name.clone(),
                        node.parameters.clone(),
                    )))
                } else {
                    // TODO: Implement proper conversion for control flow nodes
                    Ok(None)
                }
            }
            _ => {
                // Other node types (Loop, Parallel, Checkpoint)
                // If they have tool_name, treat as tool, else skip for now
                if let Some(tool_name) = &node.tool_name {
                    Ok(Some(FlowNode::tool(
                        node.id.clone(),
                        tool_name.clone(),
                        node.parameters.clone(),
                    )))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
