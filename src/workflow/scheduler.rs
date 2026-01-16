//! DAG-based workflow scheduler

use crate::error::{Result, WorkflowError};
use crate::workflow::{WorkflowDefinition, WorkflowEdge, WorkflowNode};
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Directed, Direction, Graph};
use std::collections::{HashMap, HashSet, VecDeque};

/// DAG-based workflow scheduler
pub struct DagScheduler {
    /// The workflow graph
    graph: WorkflowGraph,
    /// Mapping from node ID to graph index
    node_indices: HashMap<String, NodeIndex>,
    /// Mapping from graph index to node ID
    index_to_id: HashMap<NodeIndex, String>,
    /// Current execution state
    execution_state: ExecutionState,
}

/// Workflow graph type
type WorkflowGraph = Graph<WorkflowNode, WorkflowEdge, Directed>;

/// Execution state tracking
#[derive(Debug, Clone)]
struct ExecutionState {
    /// Nodes that are ready to execute
    ready_nodes: VecDeque<NodeIndex>,
    /// Nodes currently executing
    executing_nodes: HashSet<NodeIndex>,
    /// Nodes that have completed
    completed_nodes: HashSet<NodeIndex>,
    /// Nodes that have failed
    failed_nodes: HashSet<NodeIndex>,
    /// Nodes that are blocked by failed dependencies
    blocked_nodes: HashSet<NodeIndex>,
}

impl ExecutionState {
    fn new() -> Self {
        Self {
            ready_nodes: VecDeque::new(),
            executing_nodes: HashSet::new(),
            completed_nodes: HashSet::new(),
            failed_nodes: HashSet::new(),
            blocked_nodes: HashSet::new(),
        }
    }

    fn is_complete(&self, total_nodes: usize) -> bool {
        self.completed_nodes.len() + self.failed_nodes.len() + self.blocked_nodes.len()
            == total_nodes
    }

    fn has_ready_nodes(&self) -> bool {
        !self.ready_nodes.is_empty()
    }
}

/// Scheduling result
#[derive(Debug, Clone)]
pub struct SchedulingResult {
    /// Execution order of nodes
    pub execution_order: Vec<Vec<String>>,
    /// Nodes that can be executed in parallel
    pub parallel_groups: Vec<Vec<String>>,
    /// Critical path through the workflow
    pub critical_path: Vec<String>,
    /// Estimated execution time (if node durations are provided)
    pub estimated_duration: Option<std::time::Duration>,
}

/// Node execution info
#[derive(Debug, Clone)]
pub struct NodeExecutionInfo {
    pub node_id: String,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub can_execute: bool,
    pub priority: f64,
}

impl DagScheduler {
    /// Create a new DAG scheduler
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_indices: HashMap::new(),
            index_to_id: HashMap::new(),
            execution_state: ExecutionState::new(),
        }
    }

    /// Build the scheduler from a workflow definition
    pub fn from_workflow(workflow: &WorkflowDefinition) -> Result<Self> {
        let mut scheduler = Self::new();
        scheduler.build_graph(workflow)?;
        scheduler.initialize_execution_state()?;
        Ok(scheduler)
    }

    /// Build the internal graph from workflow definition
    fn build_graph(&mut self, workflow: &WorkflowDefinition) -> Result<()> {
        // Clear existing state
        self.graph.clear();
        self.node_indices.clear();
        self.index_to_id.clear();

        // Add nodes to the graph
        for node in &workflow.nodes {
            let node_index = self.graph.add_node(node.clone());
            self.node_indices.insert(node.id.clone(), node_index);
            self.index_to_id.insert(node_index, node.id.clone());
        }

        // Add edges to the graph
        for edge in &workflow.edges {
            let from_index = self
                .node_indices
                .get(&edge.from)
                .ok_or_else(|| WorkflowError::NodeNotFound(edge.from.clone()))?;
            let to_index = self
                .node_indices
                .get(&edge.to)
                .ok_or_else(|| WorkflowError::NodeNotFound(edge.to.clone()))?;

            self.graph.add_edge(*from_index, *to_index, edge.clone());
        }

        // Add explicit dependency edges
        for node in &workflow.nodes {
            let node_index = self.node_indices[&node.id];
            for dep in &node.depends_on {
                if let Some(&dep_index) = self.node_indices.get(dep) {
                    // Only add edge if it doesn't already exist
                    if !self.graph.find_edge(dep_index, node_index).is_some() {
                        let dep_edge = WorkflowEdge::new(dep, &node.id);
                        self.graph.add_edge(dep_index, node_index, dep_edge);
                    }
                }
            }
        }

        // Validate the graph is acyclic
        if is_cyclic_directed(&self.graph) {
            return Err(WorkflowError::CircularDependency.into());
        }

        Ok(())
    }

    /// Initialize the execution state
    fn initialize_execution_state(&mut self) -> Result<()> {
        self.execution_state = ExecutionState::new();

        // Find nodes with no dependencies (ready to execute)
        for node_index in self.graph.node_indices() {
            if self
                .graph
                .edges_directed(node_index, Direction::Incoming)
                .count()
                == 0
            {
                self.execution_state.ready_nodes.push_back(node_index);
            }
        }

        Ok(())
    }

    /// Get the topological sort order
    pub fn get_topological_order(&self) -> Result<Vec<String>> {
        let topo_order =
            toposort(&self.graph, None).map_err(|_| WorkflowError::CircularDependency)?;

        Ok(topo_order
            .into_iter()
            .map(|idx| self.index_to_id[&idx].clone())
            .collect())
    }

    /// Get nodes that are ready to execute
    pub fn get_ready_nodes(&self) -> Vec<String> {
        self.execution_state
            .ready_nodes
            .iter()
            .map(|&idx| self.index_to_id[&idx].clone())
            .collect()
    }

    /// Get nodes that can be executed in parallel
    pub fn get_parallel_executable_nodes(&self, max_parallel: Option<usize>) -> Vec<String> {
        let available_count = max_parallel
            .unwrap_or(usize::MAX)
            .saturating_sub(self.execution_state.executing_nodes.len());

        self.execution_state
            .ready_nodes
            .iter()
            .take(available_count)
            .map(|&idx| self.index_to_id[&idx].clone())
            .collect()
    }

    /// Mark a node as started
    pub fn mark_node_started(&mut self, node_id: &str) -> Result<()> {
        let node_index = self
            .node_indices
            .get(node_id)
            .ok_or_else(|| WorkflowError::NodeNotFound(node_id.to_string()))?;

        // Remove from ready nodes
        self.execution_state
            .ready_nodes
            .retain(|&idx| idx != *node_index);

        // Add to executing nodes
        self.execution_state.executing_nodes.insert(*node_index);

        Ok(())
    }

    /// Mark a node as completed successfully
    pub fn mark_node_completed(&mut self, node_id: &str) -> Result<()> {
        let node_index = self
            .node_indices
            .get(node_id)
            .ok_or_else(|| WorkflowError::NodeNotFound(node_id.to_string()))?;

        // Remove from executing nodes
        self.execution_state.executing_nodes.remove(node_index);

        // Add to completed nodes
        self.execution_state.completed_nodes.insert(*node_index);

        // Check if any dependent nodes are now ready
        self.update_ready_nodes(*node_index)?;

        Ok(())
    }

    /// Mark a node as failed
    pub fn mark_node_failed(&mut self, node_id: &str) -> Result<()> {
        let node_index = self
            .node_indices
            .get(node_id)
            .ok_or_else(|| WorkflowError::NodeNotFound(node_id.to_string()))?;

        // Remove from executing nodes
        self.execution_state.executing_nodes.remove(node_index);

        // Add to failed nodes
        self.execution_state.failed_nodes.insert(*node_index);

        // Mark dependent nodes as blocked
        self.mark_dependents_blocked(*node_index)?;

        Ok(())
    }

    /// Update ready nodes after a node completion
    fn update_ready_nodes(&mut self, completed_node: NodeIndex) -> Result<()> {
        // Check all dependent nodes
        for edge in self
            .graph
            .edges_directed(completed_node, Direction::Outgoing)
        {
            let dependent_node = edge.target();

            // Skip if already processed
            if self
                .execution_state
                .completed_nodes
                .contains(&dependent_node)
                || self.execution_state.failed_nodes.contains(&dependent_node)
                || self.execution_state.blocked_nodes.contains(&dependent_node)
                || self
                    .execution_state
                    .executing_nodes
                    .contains(&dependent_node)
                || self.execution_state.ready_nodes.contains(&dependent_node)
            {
                continue;
            }

            // Check if all dependencies are satisfied
            if self.are_dependencies_satisfied(dependent_node) {
                self.execution_state.ready_nodes.push_back(dependent_node);
            }
        }

        Ok(())
    }

    /// Check if all dependencies of a node are satisfied
    fn are_dependencies_satisfied(&self, node_index: NodeIndex) -> bool {
        for edge in self.graph.edges_directed(node_index, Direction::Incoming) {
            let dependency = edge.source();
            if !self.execution_state.completed_nodes.contains(&dependency) {
                return false;
            }
        }
        true
    }

    /// Mark dependent nodes as blocked due to failed dependency
    fn mark_dependents_blocked(&mut self, failed_node: NodeIndex) -> Result<()> {
        let mut to_block = Vec::new();
        let mut visited = HashSet::new();

        self.collect_dependents(failed_node, &mut to_block, &mut visited);

        for node_index in to_block {
            self.execution_state.blocked_nodes.insert(node_index);
            self.execution_state
                .ready_nodes
                .retain(|&idx| idx != node_index);
        }

        Ok(())
    }

    /// Recursively collect all dependent nodes
    fn collect_dependents(
        &self,
        node_index: NodeIndex,
        to_block: &mut Vec<NodeIndex>,
        visited: &mut HashSet<NodeIndex>,
    ) {
        if visited.contains(&node_index) {
            return;
        }
        visited.insert(node_index);

        for edge in self.graph.edges_directed(node_index, Direction::Outgoing) {
            let dependent = edge.target();
            if !self.execution_state.completed_nodes.contains(&dependent) {
                to_block.push(dependent);
                self.collect_dependents(dependent, to_block, visited);
            }
        }
    }

    /// Get execution information for a node
    pub fn get_node_info(&self, node_id: &str) -> Result<NodeExecutionInfo> {
        let node_index = self
            .node_indices
            .get(node_id)
            .ok_or_else(|| WorkflowError::NodeNotFound(node_id.to_string()))?;

        let dependencies = self
            .graph
            .edges_directed(*node_index, Direction::Incoming)
            .map(|edge| self.index_to_id[&edge.source()].clone())
            .collect();

        let dependents = self
            .graph
            .edges_directed(*node_index, Direction::Outgoing)
            .map(|edge| self.index_to_id[&edge.target()].clone())
            .collect();

        let can_execute = self.execution_state.ready_nodes.contains(node_index);

        // Calculate priority based on number of dependents (more dependents = higher priority)
        let priority = self
            .graph
            .edges_directed(*node_index, Direction::Outgoing)
            .count() as f64;

        Ok(NodeExecutionInfo {
            node_id: node_id.to_string(),
            dependencies,
            dependents,
            can_execute,
            priority,
        })
    }

    /// Generate a complete scheduling result
    pub fn generate_schedule(&self) -> Result<SchedulingResult> {
        let execution_order = self.calculate_execution_levels()?;
        let parallel_groups = execution_order.clone();
        let critical_path = self.calculate_critical_path()?;

        Ok(SchedulingResult {
            execution_order,
            parallel_groups,
            critical_path,
            estimated_duration: None, // TODO: Calculate based on node durations
        })
    }

    /// Calculate execution levels (nodes that can be executed in parallel)
    fn calculate_execution_levels(&self) -> Result<Vec<Vec<String>>> {
        let mut levels = Vec::new();
        let mut remaining_nodes: HashSet<_> = self.graph.node_indices().collect();
        let mut completed = HashSet::new();

        while !remaining_nodes.is_empty() {
            let mut current_level = Vec::new();

            // Find nodes with no uncompleted dependencies
            for &node_index in &remaining_nodes {
                let has_uncompleted_deps = self
                    .graph
                    .edges_directed(node_index, Direction::Incoming)
                    .any(|edge| !completed.contains(&edge.source()));

                if !has_uncompleted_deps {
                    current_level.push(self.index_to_id[&node_index].clone());
                }
            }

            if current_level.is_empty() {
                return Err(WorkflowError::CircularDependency.into());
            }

            // Remove processed nodes
            for node_id in &current_level {
                let node_index = self.node_indices[node_id];
                remaining_nodes.remove(&node_index);
                completed.insert(node_index);
            }

            levels.push(current_level);
        }

        Ok(levels)
    }

    /// Calculate the critical path (longest path through the workflow)
    fn calculate_critical_path(&self) -> Result<Vec<String>> {
        // For now, return the topological order as a simple critical path
        // TODO: Implement proper critical path calculation with node durations
        self.get_topological_order()
    }

    /// Check if execution is complete
    pub fn is_execution_complete(&self) -> bool {
        self.execution_state.is_complete(self.graph.node_count())
    }

    /// Check if there are nodes ready to execute
    pub fn has_ready_nodes(&self) -> bool {
        self.execution_state.has_ready_nodes()
    }

    /// Check if there are nodes currently executing
    pub fn has_executing_nodes(&self) -> bool {
        !self.execution_state.executing_nodes.is_empty()
    }

    /// Get execution statistics
    pub fn get_execution_stats(&self) -> ExecutionStats {
        ExecutionStats {
            total_nodes: self.graph.node_count(),
            ready_nodes: self.execution_state.ready_nodes.len(),
            executing_nodes: self.execution_state.executing_nodes.len(),
            completed_nodes: self.execution_state.completed_nodes.len(),
            failed_nodes: self.execution_state.failed_nodes.len(),
            blocked_nodes: self.execution_state.blocked_nodes.len(),
        }
    }
}

impl Default for DagScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution statistics
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_nodes: usize,
    pub ready_nodes: usize,
    pub executing_nodes: usize,
    pub completed_nodes: usize,
    pub failed_nodes: usize,
    pub blocked_nodes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{WorkflowDefinition, WorkflowEdge, WorkflowNode};
    use proptest::prelude::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_simple_linear_workflow() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow
            .add_node(WorkflowNode::tool("node1", "tool1"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("node2", "tool2"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("node3", "tool3"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("node1", "node2"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("node2", "node3"))
            .unwrap();

        let scheduler = DagScheduler::from_workflow(&workflow).unwrap();
        let topo_order = scheduler.get_topological_order().unwrap();

        assert_eq!(topo_order, vec!["node1", "node2", "node3"]);

        let ready_nodes = scheduler.get_ready_nodes();
        assert_eq!(ready_nodes, vec!["node1"]);
    }

    #[test]
    fn test_parallel_workflow() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow
            .add_node(WorkflowNode::tool("start", "tool1"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("parallel1", "tool2"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("parallel2", "tool3"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("end", "tool4"))
            .unwrap();

        workflow
            .add_edge(WorkflowEdge::new("start", "parallel1"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("start", "parallel2"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("parallel1", "end"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("parallel2", "end"))
            .unwrap();

        let scheduler = DagScheduler::from_workflow(&workflow).unwrap();
        let schedule = scheduler.generate_schedule().unwrap();

        assert_eq!(schedule.execution_order.len(), 3);
        assert_eq!(schedule.execution_order[0], vec!["start"]);
        assert_eq!(schedule.execution_order[1].len(), 2); // parallel1 and parallel2
        assert_eq!(schedule.execution_order[2], vec!["end"]);
    }

    #[test]
    fn test_node_completion_flow() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow
            .add_node(WorkflowNode::tool("node1", "tool1"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("node2", "tool2"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("node1", "node2"))
            .unwrap();

        let mut scheduler = DagScheduler::from_workflow(&workflow).unwrap();

        // Initially only node1 should be ready
        assert_eq!(scheduler.get_ready_nodes(), vec!["node1"]);

        // Start node1
        scheduler.mark_node_started("node1").unwrap();
        assert!(scheduler.get_ready_nodes().is_empty());

        // Complete node1
        scheduler.mark_node_completed("node1").unwrap();
        assert_eq!(scheduler.get_ready_nodes(), vec!["node2"]);

        // Start and complete node2
        scheduler.mark_node_started("node2").unwrap();
        scheduler.mark_node_completed("node2").unwrap();

        assert!(scheduler.is_execution_complete());
    }

    #[test]
    fn test_node_failure_blocking() {
        let mut workflow = WorkflowDefinition::new("test", "1.0");
        workflow
            .add_node(WorkflowNode::tool("node1", "tool1"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("node2", "tool2"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("node3", "tool3"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("node1", "node2"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("node2", "node3"))
            .unwrap();

        let mut scheduler = DagScheduler::from_workflow(&workflow).unwrap();

        // Start and fail node1
        scheduler.mark_node_started("node1").unwrap();
        scheduler.mark_node_failed("node1").unwrap();

        let stats = scheduler.get_execution_stats();
        assert_eq!(stats.failed_nodes, 1);
        assert_eq!(stats.blocked_nodes, 2); // node2 and node3 should be blocked

        assert!(scheduler.is_execution_complete());
    }

    // Property-based test generators
    fn valid_node_id() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_-]{0,31}".prop_map(|s| s)
    }

    fn workflow_name() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_-]{0,63}".prop_map(|s| s)
    }

    fn workflow_version() -> impl Strategy<Value = String> {
        r"[0-9]+\.[0-9]+(\.[0-9]+)?".prop_map(|s| s)
    }

    // Property-based test for dependency execution order correctness
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_dependency_execution_order_correctness(
            name in workflow_name(),
            version in workflow_version(),
            node_count in 2usize..15,
            node_ids in prop::collection::vec(valid_node_id(), 2..15),
            edge_probability in 0.1f64..0.7f64, // Probability of creating edges between nodes
        ) {
            // **Feature: workflow-toolkit, Property 3: Dependency execution order correctness**
            // *For any* workflow containing dependencies, task execution order should follow topological sorting rules
            // **Validates: Requirements 1.4**

            prop_assume!(node_ids.len() >= node_count);

            // Create a workflow with unique node IDs
            let mut workflow = WorkflowDefinition::new(&name, &version);
            let mut unique_node_ids = Vec::new();
            let mut seen_ids = HashSet::new();

            // Ensure we have unique node IDs
            for node_id in node_ids.iter().take(node_count) {
                if seen_ids.insert(node_id.clone()) {
                    unique_node_ids.push(node_id.clone());
                }
                if unique_node_ids.len() >= node_count {
                    break;
                }
            }

            prop_assume!(unique_node_ids.len() >= 2);

            // Add nodes to workflow
            for node_id in &unique_node_ids {
                workflow.add_node(WorkflowNode::tool(node_id.clone(), "test_tool".to_string())).unwrap();
            }

            // Create a DAG by adding edges that don't create cycles
            // We'll create edges from earlier nodes to later nodes to ensure acyclicity
            let mut dependency_map: HashMap<String, HashSet<String>> = HashMap::new();

            for i in 0..unique_node_ids.len() {
                dependency_map.insert(unique_node_ids[i].clone(), HashSet::new());

                for j in 0..i {
                    // Add edge from j to i with given probability
                    if edge_probability > (j as f64 / unique_node_ids.len() as f64) {
                        let from_node = &unique_node_ids[j];
                        let to_node = &unique_node_ids[i];

                        workflow.add_edge(WorkflowEdge::new(from_node, to_node)).unwrap();
                        dependency_map.get_mut(to_node).unwrap().insert(from_node.clone());
                    }
                }
            }

            // Create scheduler and get topological order
            let scheduler = DagScheduler::from_workflow(&workflow).unwrap();
            let topo_order = scheduler.get_topological_order().unwrap();

            // Property 1: All nodes should be in the topological order
            prop_assert_eq!(topo_order.len(), unique_node_ids.len(),
                "Topological order should contain all nodes");

            for node_id in &unique_node_ids {
                prop_assert!(topo_order.contains(node_id),
                    "Node {} should be in topological order", node_id);
            }

            // Property 2: Dependencies should be satisfied in the topological order
            let node_positions: HashMap<String, usize> = topo_order.iter()
                .enumerate()
                .map(|(pos, node)| (node.clone(), pos))
                .collect();

            for (node, dependencies) in &dependency_map {
                let node_pos = node_positions[node];

                for dependency in dependencies {
                    let dep_pos = node_positions[dependency];
                    prop_assert!(dep_pos < node_pos,
                        "Dependency {} (pos {}) should come before {} (pos {}) in topological order",
                        dependency, dep_pos, node, node_pos);
                }
            }

            // Property 3: Execution levels should respect dependencies
            let schedule = scheduler.generate_schedule().unwrap();
            let execution_levels = &schedule.execution_order;

            // Build position map for execution levels
            let mut level_positions: HashMap<String, usize> = HashMap::new();
            for (level, nodes) in execution_levels.iter().enumerate() {
                for node in nodes {
                    level_positions.insert(node.clone(), level);
                }
            }

            // Check that dependencies are in earlier levels
            for (node, dependencies) in &dependency_map {
                let node_level = level_positions[node];

                for dependency in dependencies {
                    let dep_level = level_positions[dependency];
                    prop_assert!(dep_level < node_level,
                        "Dependency {} (level {}) should be in earlier level than {} (level {})",
                        dependency, dep_level, node, node_level);
                }
            }

            // Property 4: Ready nodes should have no unmet dependencies
            let ready_nodes = scheduler.get_ready_nodes();
            for ready_node in &ready_nodes {
                let dependencies = dependency_map.get(ready_node).unwrap();
                prop_assert!(dependencies.is_empty(),
                    "Ready node {} should have no dependencies, but has: {:?}",
                    ready_node, dependencies);
            }

            // Property 5: Simulate execution and verify order is maintained
            let mut execution_scheduler = DagScheduler::from_workflow(&workflow).unwrap();
            let mut executed_nodes = HashSet::new();
            let mut execution_order = Vec::new();

            while !execution_scheduler.is_execution_complete() && execution_scheduler.has_ready_nodes() {
                let ready = execution_scheduler.get_ready_nodes();

                // Verify all ready nodes have their dependencies satisfied
                for ready_node in &ready {
                    let dependencies = dependency_map.get(ready_node).unwrap();
                    for dep in dependencies {
                        prop_assert!(executed_nodes.contains(dep),
                            "Ready node {} has unmet dependency {}", ready_node, dep);
                    }
                }

                // Execute the first ready node
                if let Some(node_to_execute) = ready.first() {
                    execution_scheduler.mark_node_started(node_to_execute).unwrap();
                    execution_scheduler.mark_node_completed(node_to_execute).unwrap();
                    executed_nodes.insert(node_to_execute.clone());
                    execution_order.push(node_to_execute.clone());
                }
            }

            // Property 6: All nodes should be executed if no failures occur
            prop_assert_eq!(executed_nodes.len(), unique_node_ids.len(),
                "All nodes should be executed");

            // Property 7: Execution order should respect dependencies
            let exec_positions: HashMap<String, usize> = execution_order.iter()
                .enumerate()
                .map(|(pos, node)| (node.clone(), pos))
                .collect();

            for (node, dependencies) in &dependency_map {
                let node_pos = exec_positions[node];

                for dependency in dependencies {
                    let dep_pos = exec_positions[dependency];
                    prop_assert!(dep_pos < node_pos,
                        "In execution order, dependency {} (pos {}) should come before {} (pos {})",
                        dependency, dep_pos, node, node_pos);
                }
            }
        }
    }
}
