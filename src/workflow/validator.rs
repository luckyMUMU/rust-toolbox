//! Workflow validation logic

use crate::error::{Result, WorkflowError};
use crate::workflow::{WorkflowDefinition, WorkflowNode, WorkflowEdge, NodeType};
use petgraph::{Graph, Directed};
use petgraph::algo::is_cyclic_directed;
use std::collections::{HashMap, HashSet};

/// Workflow validator for syntax and semantic validation
pub struct WorkflowValidator {
    /// Available tool names for validation
    available_tools: HashSet<String>,
}

impl WorkflowValidator {
    /// Create a new workflow validator
    pub fn new() -> Self {
        Self {
            available_tools: HashSet::new(),
        }
    }

    /// Create a validator with available tools
    pub fn with_tools(available_tools: HashSet<String>) -> Self {
        Self {
            available_tools,
        }
    }

    /// Add an available tool
    pub fn add_tool<S: Into<String>>(&mut self, tool_name: S) {
        self.available_tools.insert(tool_name.into());
    }

    /// Validate a workflow definition
    pub fn validate(&self, workflow: &WorkflowDefinition) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Perform all validation checks
        self.validate_basic_structure(workflow, &mut result)?;
        self.validate_nodes(workflow, &mut result)?;
        self.validate_edges(workflow, &mut result)?;
        self.validate_dag_structure(workflow, &mut result)?;
        self.validate_dependencies(workflow, &mut result)?;
        self.validate_tools(workflow, &mut result)?;

        Ok(result)
    }

    /// Validate basic workflow structure
    fn validate_basic_structure(&self, workflow: &WorkflowDefinition, result: &mut ValidationResult) -> Result<()> {
        // Check workflow name
        if workflow.name.is_empty() {
            result.add_error("Workflow name cannot be empty".to_string());
        } else if workflow.name.len() > 255 {
            result.add_error("Workflow name cannot exceed 255 characters".to_string());
        }

        // Check workflow version
        if workflow.version.is_empty() {
            result.add_error("Workflow version cannot be empty".to_string());
        }

        // Check if workflow has nodes
        if workflow.nodes.is_empty() {
            result.add_error("Workflow must contain at least one node".to_string());
        }

        // Check for reasonable limits
        if workflow.nodes.len() > 10000 {
            result.add_warning("Workflow has a very large number of nodes (>10000), this may impact performance".to_string());
        }

        if workflow.edges.len() > 50000 {
            result.add_warning("Workflow has a very large number of edges (>50000), this may impact performance".to_string());
        }

        Ok(())
    }

    /// Validate workflow nodes
    fn validate_nodes(&self, workflow: &WorkflowDefinition, result: &mut ValidationResult) -> Result<()> {
        let mut node_ids = HashSet::new();

        for node in &workflow.nodes {
            // Check for duplicate node IDs
            if !node_ids.insert(&node.id) {
                result.add_error(format!("Duplicate node ID: {}", node.id));
                continue;
            }

            // Validate individual node
            if let Err(e) = self.validate_node(node) {
                result.add_error(format!("Node '{}': {}", node.id, e));
            }
        }

        Ok(())
    }

    /// Validate a single node
    fn validate_node(&self, node: &WorkflowNode) -> Result<()> {
        // Check node ID
        if node.id.is_empty() {
            return Err(WorkflowError::InvalidNodeId("Node ID cannot be empty".to_string()).into());
        }

        // Check node ID format (alphanumeric, hyphens, underscores only)
        if !node.id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(WorkflowError::InvalidNodeId(
                format!("Node ID '{}' contains invalid characters. Only alphanumeric, hyphens, and underscores are allowed", node.id)
            ).into());
        }

        // Validate based on node type
        match node.node_type {
            NodeType::Tool => {
                if node.tool_name.is_none() {
                    return Err(WorkflowError::MissingToolName(node.id.clone()).into());
                }
            }
            NodeType::Condition => {
                // Condition nodes should have condition logic in parameters
                if node.parameters.is_null() {
                    return Err(WorkflowError::workflow_validation(
                        format!("Condition node '{}' must have condition parameters", node.id)
                    ));
                }
            }
            NodeType::Loop => {
                // Loop nodes should have loop configuration
                if node.parameters.is_null() {
                    return Err(WorkflowError::workflow_validation(
                        format!("Loop node '{}' must have loop parameters", node.id)
                    ));
                }
            }
            NodeType::Parallel => {
                // Parallel nodes are validated through their edges
            }
            NodeType::Checkpoint => {
                // Checkpoint nodes are always valid
            }
        }

        Ok(())
    }

    /// Validate workflow edges
    fn validate_edges(&self, workflow: &WorkflowDefinition, result: &mut ValidationResult) -> Result<()> {
        let node_ids: HashSet<_> = workflow.nodes.iter().map(|n| &n.id).collect();

        for edge in &workflow.edges {
            // Check that source node exists
            if !node_ids.contains(&edge.from) {
                result.add_error(format!("Edge references non-existent source node: {}", edge.from));
            }

            // Check that target node exists
            if !node_ids.contains(&edge.to) {
                result.add_error(format!("Edge references non-existent target node: {}", edge.to));
            }

            // Check for self-loops
            if edge.from == edge.to {
                result.add_warning(format!("Self-loop detected on node: {}", edge.from));
            }

            // Validate conditional edges
            if let Some(condition) = &edge.condition {
                if condition.is_empty() {
                    result.add_error(format!("Empty condition on edge from {} to {}", edge.from, edge.to));
                }
            }
        }

        Ok(())
    }

    /// Validate DAG structure (no cycles)
    fn validate_dag_structure(&self, workflow: &WorkflowDefinition, result: &mut ValidationResult) -> Result<()> {
        // Build a graph using petgraph
        let mut graph = Graph::<&str, (), Directed>::new();
        let mut node_indices = HashMap::new();

        // Add nodes to graph
        for node in &workflow.nodes {
            let index = graph.add_node(&node.id);
            node_indices.insert(&node.id, index);
        }

        // Add edges to graph
        for edge in &workflow.edges {
            if let (Some(&from_idx), Some(&to_idx)) = (
                node_indices.get(&edge.from),
                node_indices.get(&edge.to)
            ) {
                graph.add_edge(from_idx, to_idx, ());
            }
        }

        // Check for cycles
        if is_cyclic_directed(&graph) {
            result.add_error("Workflow contains circular dependencies".to_string());
        }

        // Check for disconnected components
        self.validate_connectivity(&graph, result);

        Ok(())
    }

    /// Validate workflow connectivity
    fn validate_connectivity(&self, graph: &Graph<&str, (), Directed>, result: &mut ValidationResult) {
        let node_count = graph.node_count();
        if node_count == 0 {
            return;
        }

        // Find nodes with no incoming edges (potential start nodes)
        let mut start_nodes = Vec::new();
        for node_idx in graph.node_indices() {
            if graph.edges_directed(node_idx, petgraph::Incoming).count() == 0 {
                start_nodes.push(node_idx);
            }
        }

        if start_nodes.is_empty() {
            result.add_error("Workflow has no start nodes (all nodes have incoming edges)".to_string());
        }

        // Find nodes with no outgoing edges (potential end nodes)
        let mut end_nodes = Vec::new();
        for node_idx in graph.node_indices() {
            if graph.edges_directed(node_idx, petgraph::Outgoing).count() == 0 {
                end_nodes.push(node_idx);
            }
        }

        if end_nodes.is_empty() {
            result.add_warning("Workflow has no end nodes (all nodes have outgoing edges)".to_string());
        }
    }

    /// Validate node dependencies
    fn validate_dependencies(&self, workflow: &WorkflowDefinition, result: &mut ValidationResult) -> Result<()> {
        let node_map: HashMap<_, _> = workflow.nodes.iter().map(|n| (&n.id, n)).collect();

        for node in &workflow.nodes {
            // Check explicit dependencies
            for dep in &node.depends_on {
                if !node_map.contains_key(dep) {
                    result.add_error(format!("Node '{}' depends on non-existent node '{}'", node.id, dep));
                }
            }

            // Check for dependency cycles in explicit dependencies
            if self.has_dependency_cycle(node, &node_map, &mut HashSet::new()) {
                result.add_error(format!("Circular dependency detected involving node '{}'", node.id));
            }
        }

        Ok(())
    }

    /// Check for dependency cycles using DFS
    fn has_dependency_cycle(
        &self,
        node: &WorkflowNode,
        node_map: &HashMap<&String, &WorkflowNode>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visited.contains(&node.id) {
            return true;
        }

        visited.insert(node.id.clone());

        for dep in &node.depends_on {
            if let Some(dep_node) = node_map.get(dep) {
                if self.has_dependency_cycle(dep_node, node_map, visited) {
                    return true;
                }
            }
        }

        visited.remove(&node.id);
        false
    }

    /// Validate tool availability
    fn validate_tools(&self, workflow: &WorkflowDefinition, result: &mut ValidationResult) -> Result<()> {
        if self.available_tools.is_empty() {
            // Skip tool validation if no tools are registered
            return Ok(());
        }

        for node in &workflow.nodes {
            if node.node_type == NodeType::Tool {
                if let Some(tool_name) = &node.tool_name {
                    if !self.available_tools.contains(tool_name) {
                        result.add_error(format!("Node '{}' references unknown tool '{}'", node.id, tool_name));
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for WorkflowValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation result containing errors and warnings
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result
    pub fn add_error<S: Into<String>>(&mut self, error: S) {
        self.errors.push(error.into());
    }

    /// Add a warning to the result
    pub fn add_warning<S: Into<String>>(&mut self, warning: S) {
        self.warnings.push(warning.into());
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get total number of issues (errors + warnings)
    pub fn issue_count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{WorkflowDefinition, WorkflowNode, WorkflowEdge, NodeType};
    use crate::core::WorkflowConfig;

    #[test]
    fn test_valid_workflow() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow.add_node(WorkflowNode::tool("node1", "test_tool")).unwrap();
        workflow.add_node(WorkflowNode::tool("node2", "test_tool")).unwrap();
        workflow.add_edge(WorkflowEdge::new("node1", "node2")).unwrap();

        let mut validator = WorkflowValidator::new();
        validator.add_tool("test_tool");

        let result = validator.validate(&workflow).unwrap();
        assert!(result.is_valid(), "Workflow should be valid");
    }

    #[test]
    fn test_circular_dependency() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow.add_node(WorkflowNode::tool("node1", "test_tool")).unwrap();
        workflow.add_node(WorkflowNode::tool("node2", "test_tool")).unwrap();
        workflow.add_edge(WorkflowEdge::new("node1", "node2")).unwrap();
        workflow.add_edge(WorkflowEdge::new("node2", "node1")).unwrap();

        let validator = WorkflowValidator::new();
        let result = validator.validate(&workflow).unwrap();
        
        assert!(!result.is_valid(), "Workflow with circular dependency should be invalid");
        assert!(result.errors.iter().any(|e| e.contains("circular")));
    }

    #[test]
    fn test_missing_node_reference() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow.add_node(WorkflowNode::tool("node1", "test_tool")).unwrap();
        workflow.edges.push(WorkflowEdge::new("node1", "nonexistent"));

        let validator = WorkflowValidator::new();
        let result = validator.validate(&workflow).unwrap();
        
        assert!(!result.is_valid(), "Workflow with missing node reference should be invalid");
    }

    #[test]
    fn test_duplicate_node_ids() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow.nodes.push(WorkflowNode::tool("node1", "test_tool"));
        workflow.nodes.push(WorkflowNode::tool("node1", "test_tool")); // Duplicate ID

        let validator = WorkflowValidator::new();
        let result = validator.validate(&workflow).unwrap();
        
        assert!(!result.is_valid(), "Workflow with duplicate node IDs should be invalid");
    }

    #[test]
    fn test_unknown_tool() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow.add_node(WorkflowNode::tool("node1", "unknown_tool")).unwrap();

        let mut validator = WorkflowValidator::new();
        validator.add_tool("known_tool");

        let result = validator.validate(&workflow).unwrap();
        assert!(!result.is_valid(), "Workflow with unknown tool should be invalid");
    }
}