//! Simple test for execution manager functionality
//! This is a standalone test file to verify basic functionality

#[cfg(test)]
mod simple_tests {
    use super::super::*;
    use crate::core::*;
    use crate::storage::{StateManager, SimpleMemoryCache};
    use crate::tools::ToolRegistry;
    use crate::workflow::{WorkflowDefinition, WorkflowNode, NodeType};
    use crate::workflow::engine::DefaultWorkflowEngine;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Simple mock storage backend
    struct SimpleStorageBackend {
        data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    }

    impl SimpleStorageBackend {
        fn new() -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl crate::storage::StorageBackend for SimpleStorageBackend {
        async fn save(&self, key: &str, value: &[u8]) -> crate::Result<()> {
            let mut data = self.data.write().await;
            data.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn load(&self, key: &str) -> crate::Result<Option<Vec<u8>>> {
            let data = self.data.read().await;
            Ok(data.get(key).cloned())
        }

        async fn delete(&self, key: &str) -> crate::Result<()> {
            let mut data = self.data.write().await;
            data.remove(key);
            Ok(())
        }

        async fn list_keys(&self, prefix: &str) -> crate::Result<Vec<String>> {
            let data = self.data.read().await;
            Ok(data.keys()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect())
        }

        async fn exists(&self, key: &str) -> crate::Result<bool> {
            let data = self.data.read().await;
            Ok(data.contains_key(key))
        }

        async fn batch_save(&self, items: Vec<(String, Vec<u8>)>) -> crate::Result<()> {
            let mut data = self.data.write().await;
            for (key, value) in items {
                data.insert(key, value);
            }
            Ok(())
        }

        async fn batch_load(&self, keys: Vec<String>) -> crate::Result<Vec<Option<Vec<u8>>>> {
            let data = self.data.read().await;
            Ok(keys.into_iter()
                .map(|key| data.get(&key).cloned())
                .collect())
        }
    }

    // Simple mock tool registry
    struct SimpleToolRegistry;

    #[async_trait]
    impl ToolRegistry for SimpleToolRegistry {
        fn register_tool(&mut self, _tool: Arc<dyn crate::tools::ToolNode>) -> crate::Result<()> {
            Ok(())
        }

        fn get_tool(&self, _name: &str) -> Option<Arc<dyn crate::tools::ToolNode>> {
            None
        }

        fn list_tools(&self) -> Vec<crate::core::ToolInfo> {
            Vec::new()
        }

        async fn execute_tool(&self, _name: &str, _params: serde_json::Value, _context: ExecutionContext) -> crate::Result<serde_json::Value> {
            // Simulate quick work
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            Ok(serde_json::Value::String("test_result".to_string()))
        }

        fn validate_tool_params(&self, _name: &str, _params: &serde_json::Value) -> crate::Result<()> {
            Ok(())
        }

        fn has_tool(&self, _name: &str) -> bool {
            true
        }

        fn unregister_tool(&mut self, _name: &str) -> crate::Result<()> {
            Ok(())
        }

        fn tool_count(&self) -> usize {
            1
        }

        fn clear(&mut self) {
            // No-op
        }

        fn resolve_dependencies(&self, _tool_names: Vec<String>) -> crate::Result<crate::tools::ResolutionResult> {
            Ok(crate::tools::ResolutionResult {
                resolved_versions: std::collections::HashMap::new(),
                conflicts: Vec::new(),
                warnings: Vec::new(),
            })
        }
        
        fn check_version_conflicts(&self) -> crate::Result<Vec<String>> {
            Ok(Vec::new())
        }
        
        fn get_dependents(&self, _tool_name: &str) -> Vec<crate::core::ToolInfo> {
            Vec::new()
        }
        
        async fn execute_tool_with_templates(
            &self,
            name: &str,
            params: serde_json::Value,
            _template_context: &crate::tools::TemplateContext,
            execution_context: ExecutionContext
        ) -> crate::Result<serde_json::Value> {
            self.execute_tool(name, params, execution_context).await
        }
        
        fn get_tool_templates(&self, _tool_name: &str) -> Vec<crate::tools::ParameterTemplate> {
            Vec::new()
        }
    }

    fn create_simple_execution_manager() -> DefaultExecutionManager {
        let storage = Arc::new(SimpleStorageBackend::new());
        let cache = Arc::new(SimpleMemoryCache::new());
        let state_manager = Arc::new(StateManager::new(storage, cache));
        let tool_registry = Arc::new(SimpleToolRegistry);
        let engine = Arc::new(DefaultWorkflowEngine::new(state_manager, tool_registry, 10));
        
        DefaultExecutionManager::new(engine, ConcurrencyConfig::default())
    }

    fn create_simple_workflow(name: &str) -> WorkflowDefinition {
        let mut workflow = WorkflowDefinition::new(name, "1.0.0");
        let node = WorkflowNode::new("test_node", NodeType::Tool);
        workflow.add_node(node).unwrap();
        workflow
    }

    #[tokio::test]
    async fn test_basic_sync_execution() {
        let manager = create_simple_execution_manager();
        let workflow = create_simple_workflow("sync_test");
        let context = ExecutionContext::new();

        let result = manager.execute_workflow(workflow, ExecutionMode::Sync, context).await;
        assert!(result.is_ok(), "Sync execution should succeed");

        match result.unwrap() {
            ExecutionResult::Sync(execution) => {
                assert!(execution.status.is_terminal(), "Sync execution should be complete");
            }
            ExecutionResult::Async(_) => {
                panic!("Sync execution should return sync result");
            }
        }
    }

    #[tokio::test]
    async fn test_basic_async_execution() {
        let manager = create_simple_execution_manager();
        let workflow = create_simple_workflow("async_test");
        let context = ExecutionContext::new();

        let result = manager.execute_workflow(workflow, ExecutionMode::Async, context).await;
        assert!(result.is_ok(), "Async execution should succeed");

        match result.unwrap() {
            ExecutionResult::Async(handle) => {
                assert_eq!(handle.mode, ExecutionMode::Async, "Handle should indicate async mode");
                
                // Wait a bit for execution to start
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                
                // Check status
                let status = manager.get_execution_status(&handle).await;
                assert!(status.is_ok(), "Should be able to get execution status");
            }
            ExecutionResult::Sync(_) => {
                panic!("Async execution should return async result");
            }
        }
    }

    #[tokio::test]
    async fn test_execution_metrics() {
        let manager = create_simple_execution_manager();
        
        let metrics = manager.get_execution_metrics().await;
        assert_eq!(metrics.active_executions, 0, "Should start with no active executions");
        assert_eq!(metrics.queued_executions, 0, "Should start with no queued executions");
        assert!(metrics.total_capacity > 0, "Should have some capacity");
    }

    // #[tokio::test]
    // async fn test_resource_limits() {
    //     let manager = create_simple_execution_manager();
    //     let limits = ResourceLimits::default();
    //     
    //     // Note: apply_resource_limits is private, so we can't test it directly
    //     // This would be tested through the public execute methods
    //     // let result = manager.apply_resource_limits(&limits, "test_execution").await;
    //     // assert!(result.is_ok(), "Should be able to apply reasonable resource limits");
    // }

    #[tokio::test]
    async fn test_cleanup_executions() {
        let manager = create_simple_execution_manager();
        
        let cleaned = manager.cleanup_completed_executions().await;
        assert!(cleaned.is_ok(), "Should be able to cleanup executions");
    }
}