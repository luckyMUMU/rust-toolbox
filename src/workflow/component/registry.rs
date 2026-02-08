//! Component registry for workflow execution.
//!
//! Provides a registry for storing and retrieving workflow components
//! by their ID.

use crate::error::{Result, WorkflowError};
use crate::tools::ToolRegistry;
use crate::workflow::component::{Component, ComponentType};
use crate::workflow::definition::{NodeType, WorkflowDefinition, WorkflowNode};
use dashmap::DashMap;
use std::sync::Arc;

/// Registry for workflow components.
///
/// Stores components by their ID and provides lookup functionality.
pub struct ComponentRegistry {
    /// Registered components
    components: DashMap<String, Arc<dyn Component>>,
}

impl ComponentRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            components: DashMap::new(),
        }
    }

    /// Register a component.
    pub fn register(&self, component: Arc<dyn Component>) {
        let id = component.id().to_string();
        tracing::debug!(
            component_id = %id,
            component_type = %component.component_type(),
            "Registering component"
        );
        self.components.insert(id, component);
    }

    /// Get a component by ID.
    pub fn get(&self, id: &str) -> Result<Arc<dyn Component>> {
        self.components
            .get(id)
            .map(|entry| Arc::clone(entry.value()))
            .ok_or_else(|| WorkflowError::component_not_found(id))
    }

    /// Check if a component exists.
    pub fn contains(&self, id: &str) -> bool {
        self.components.contains_key(id)
    }

    /// Remove a component.
    pub fn remove(&self, id: &str) -> Option<Arc<dyn Component>> {
        self.components.remove(id).map(|(_, v)| v)
    }

    /// Get all component IDs.
    pub fn ids(&self) -> Vec<String> {
        self.components.iter().map(|e| e.key().clone()).collect()
    }

    /// Get the number of registered components.
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Clear all components.
    pub fn clear(&self) {
        self.components.clear();
    }

    /// Register components from a workflow definition.
    ///
    /// Creates appropriate component instances based on node types
    /// and registers them in the registry.
    pub fn register_from_definition(
        &self,
        definition: &WorkflowDefinition,
        tool_registry: Arc<ToolRegistry>,
    ) -> Result<()> {
        for node in &definition.nodes {
            let component = self.create_component_from_node(node, Arc::clone(&tool_registry))?;
            self.register(component);
        }
        Ok(())
    }

    /// Create a component from a workflow node definition.
    fn create_component_from_node(
        &self,
        node: &WorkflowNode,
        tool_registry: Arc<ToolRegistry>,
    ) -> Result<Arc<dyn Component>> {
        match node.node_type {
            NodeType::Tool => {
                let tool_name = node.tool_name.as_ref().ok_or_else(|| {
                    WorkflowError::validation(format!("Tool node '{}' missing tool_name", node.id))
                })?;

                Ok(Arc::new(super::tool::ToolComponent::new(
                    node.id.clone(),
                    tool_name.clone(),
                    tool_registry,
                    node.parameters.clone(),
                )))
            }
            NodeType::Parallel => {
                // Extract parallel node configuration from node metadata
                let parallel_nodes = node
                    .metadata
                    .get("parallel_nodes")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let max_concurrency = node
                    .metadata
                    .get("max_concurrency")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);

                let wait_strategy = node
                    .metadata
                    .get("wait_strategy")
                    .and_then(|v| v.as_str())
                    .map(|s| match s {
                        "any" => super::parallel::WaitStrategy::WaitAny,
                        "all" => super::parallel::WaitStrategy::WaitAll,
                        _ => {
                            if let Some(n) = s.strip_prefix("n:") {
                                if let Ok(count) = n.parse() {
                                    return super::parallel::WaitStrategy::WaitN(count);
                                }
                            }
                            super::parallel::WaitStrategy::WaitAll
                        }
                    })
                    .unwrap_or(super::parallel::WaitStrategy::WaitAll);

                Ok(Arc::new(super::parallel::ParallelComponent::new(
                    node.id.clone(),
                    parallel_nodes,
                    max_concurrency,
                    wait_strategy,
                )))
            }
            NodeType::Condition => {
                // Condition component not yet implemented
                Err(WorkflowError::not_implemented(
                    "Condition component not yet implemented",
                ))
            }
            NodeType::Loop => {
                // Loop component not yet implemented
                Err(WorkflowError::not_implemented(
                    "Loop component not yet implemented",
                ))
            }
            NodeType::Checkpoint => {
                // Checkpoint component - simple passthrough for now
                Ok(Arc::new(CheckpointComponent::new(node.id.clone())))
            }
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple checkpoint component that marks a checkpoint in execution.
struct CheckpointComponent {
    id: String,
}

impl CheckpointComponent {
    fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait::async_trait]
impl Component for CheckpointComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn component_type(&self) -> ComponentType {
        ComponentType::Checkpoint
    }

    async fn execute(
        &self,
        _context: &mut crate::workflow::context::DataContext,
        _execution_ctx: &crate::core::ExecutionContext,
    ) -> Result<super::ComponentOutput> {
        // Checkpoint component signals that a checkpoint should be created
        Ok(super::ComponentOutput::success()
            .with_metadata("checkpoint_requested".to_string(), serde_json::json!(true)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::component::ComponentOutput;
    use crate::workflow::context::DataContext;

    struct MockComponent {
        id: String,
    }

    #[async_trait::async_trait]
    impl Component for MockComponent {
        fn id(&self) -> &str {
            &self.id
        }

        fn component_type(&self) -> ComponentType {
            ComponentType::Tool
        }

        async fn execute(
            &self,
            _context: &mut DataContext,
            _execution_ctx: &crate::core::ExecutionContext,
        ) -> Result<ComponentOutput> {
            Ok(ComponentOutput::success())
        }
    }

    #[test]
    fn test_registry_basic_operations() {
        let registry = ComponentRegistry::new();

        let component = Arc::new(MockComponent {
            id: "test".to_string(),
        });

        registry.register(component);

        assert!(registry.contains("test"));
        assert!(!registry.contains("nonexistent"));

        let retrieved = registry.get("test");
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id(), "test");

        let missing = registry.get("nonexistent");
        assert!(missing.is_err());
    }

    #[test]
    fn test_registry_remove() {
        let registry = ComponentRegistry::new();

        let component = Arc::new(MockComponent {
            id: "test".to_string(),
        });

        registry.register(component);
        assert_eq!(registry.len(), 1);

        let removed = registry.remove("test");
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);
    }
}
