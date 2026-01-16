//! Parallel component implementation.
//!
//! Enables true parallel execution of multiple workflow nodes.

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::workflow::component::{Component, ComponentOutput, ComponentType};
use crate::workflow::context::DataContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Strategy for waiting on parallel node completion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WaitStrategy {
    /// Wait for all nodes to complete
    WaitAll,
    /// Wait for any single node to complete
    WaitAny,
    /// Wait for N nodes to complete
    WaitN(usize),
}

impl Default for WaitStrategy {
    fn default() -> Self {
        WaitStrategy::WaitAll
    }
}

/// Parallel component that executes multiple nodes concurrently.
///
/// This component signals to the workflow engine which nodes should be
/// executed in parallel. The actual parallel execution is handled by
/// the engine.
///
/// # Example
///
/// ```ignore
/// let parallel = ParallelComponent::new(
///     "parallel-1",
///     vec!["task-a".to_string(), "task-b".to_string(), "task-c".to_string()],
///     Some(2), // Max 2 concurrent
///     WaitStrategy::WaitAll,
/// );
/// ```
pub struct ParallelComponent {
    /// Component ID
    id: String,

    /// IDs of nodes to execute in parallel
    parallel_nodes: Vec<String>,

    /// Maximum number of concurrent executions (None = unlimited)
    max_concurrency: Option<usize>,

    /// Strategy for waiting on node completion
    wait_strategy: WaitStrategy,
}

impl ParallelComponent {
    /// Create a new parallel component.
    pub fn new(
        id: impl Into<String>,
        parallel_nodes: Vec<String>,
        max_concurrency: Option<usize>,
        wait_strategy: WaitStrategy,
    ) -> Self {
        Self {
            id: id.into(),
            parallel_nodes,
            max_concurrency,
            wait_strategy,
        }
    }

    /// Create a parallel component that waits for all nodes.
    pub fn wait_all(id: impl Into<String>, nodes: Vec<String>) -> Self {
        Self::new(id, nodes, None, WaitStrategy::WaitAll)
    }

    /// Create a parallel component that waits for any node.
    pub fn wait_any(id: impl Into<String>, nodes: Vec<String>) -> Self {
        Self::new(id, nodes, None, WaitStrategy::WaitAny)
    }

    /// Get the parallel node IDs.
    pub fn parallel_nodes(&self) -> &[String] {
        &self.parallel_nodes
    }

    /// Get the maximum concurrency.
    pub fn max_concurrency(&self) -> Option<usize> {
        self.max_concurrency
    }

    /// Get the wait strategy.
    pub fn wait_strategy(&self) -> &WaitStrategy {
        &self.wait_strategy
    }

    /// Set the maximum concurrency.
    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = Some(max);
        self
    }
}

#[async_trait]
impl Component for ParallelComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn component_type(&self) -> ComponentType {
        ComponentType::Parallel
    }

    async fn execute(
        &self,
        _context: &mut DataContext,
        _execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput> {
        tracing::debug!(
            component_id = %self.id,
            parallel_nodes = ?self.parallel_nodes,
            max_concurrency = ?self.max_concurrency,
            wait_strategy = ?self.wait_strategy,
            "Parallel component returning nodes for parallel execution"
        );

        // The parallel component doesn't execute anything itself.
        // It returns the nodes that should be executed in parallel,
        // and the engine handles the actual parallel execution.
        Ok(
            ComponentOutput::success_with_next(self.parallel_nodes.clone())
                .with_metadata("parallel", json!(true))
                .with_metadata("max_concurrency", json!(self.max_concurrency))
                .with_metadata(
                    "wait_strategy",
                    json!(match &self.wait_strategy {
                        WaitStrategy::WaitAll => "all",
                        WaitStrategy::WaitAny => "any",
                        WaitStrategy::WaitN(n) => {
                            // Return the count as a separate metadata field
                            return Ok(ComponentOutput::success_with_next(
                                self.parallel_nodes.clone(),
                            )
                            .with_metadata("parallel", json!(true))
                            .with_metadata("max_concurrency", json!(self.max_concurrency))
                            .with_metadata("wait_strategy", json!("n"))
                            .with_metadata("wait_n_count", json!(n)));
                        }
                    }),
                ),
        )
    }

    fn validate(&self) -> Result<()> {
        if self.parallel_nodes.is_empty() {
            return Err(crate::error::WorkflowError::validation(
                "Parallel component must have at least one node to execute",
            ));
        }

        if let Some(max) = self.max_concurrency {
            if max == 0 {
                return Err(crate::error::WorkflowError::validation(
                    "max_concurrency must be greater than 0",
                ));
            }
        }

        if let WaitStrategy::WaitN(n) = self.wait_strategy {
            if n == 0 {
                return Err(crate::error::WorkflowError::validation(
                    "WaitN count must be greater than 0",
                ));
            }
            if n > self.parallel_nodes.len() {
                return Err(crate::error::WorkflowError::validation(&format!(
                    "WaitN count ({}) cannot exceed number of parallel nodes ({})",
                    n,
                    self.parallel_nodes.len()
                )));
            }
        }

        Ok(())
    }

    fn description(&self) -> Option<&str> {
        Some("Executes multiple nodes in parallel")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_parallel_component_returns_nodes() {
        let component = ParallelComponent::new(
            "parallel-1",
            vec!["task-a".to_string(), "task-b".to_string()],
            Some(2),
            WaitStrategy::WaitAll,
        );

        let mut context = DataContext::new();
        let exec_ctx = ExecutionContext::new().with_workflow_id(Uuid::new_v4());

        let result = component.execute(&mut context, &exec_ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.status.is_success());
        assert_eq!(output.next_nodes.len(), 2);
        assert!(output.next_nodes.contains(&"task-a".to_string()));
        assert!(output.next_nodes.contains(&"task-b".to_string()));
        assert_eq!(output.metadata.get("parallel"), Some(&json!(true)));
    }

    #[test]
    fn test_parallel_component_validation() {
        // Valid component
        let valid =
            ParallelComponent::new("p1", vec!["a".to_string()], Some(1), WaitStrategy::WaitAll);
        assert!(valid.validate().is_ok());

        // Empty nodes
        let empty = ParallelComponent::new("p2", vec![], None, WaitStrategy::WaitAll);
        assert!(empty.validate().is_err());

        // Zero concurrency
        let zero_conc =
            ParallelComponent::new("p3", vec!["a".to_string()], Some(0), WaitStrategy::WaitAll);
        assert!(zero_conc.validate().is_err());

        // WaitN exceeds node count
        let wait_n_invalid =
            ParallelComponent::new("p4", vec!["a".to_string()], None, WaitStrategy::WaitN(5));
        assert!(wait_n_invalid.validate().is_err());
    }

    #[test]
    fn test_wait_strategy_builders() {
        let wait_all = ParallelComponent::wait_all("p1", vec!["a".to_string(), "b".to_string()]);
        assert_eq!(wait_all.wait_strategy(), &WaitStrategy::WaitAll);

        let wait_any = ParallelComponent::wait_any("p2", vec!["a".to_string(), "b".to_string()]);
        assert_eq!(wait_any.wait_strategy(), &WaitStrategy::WaitAny);
    }
}
