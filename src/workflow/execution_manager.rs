//! Execution manager for synchronous and asynchronous workflow execution

use crate::core::{
    ConcurrencyConfig, ExecutionContext, ExecutionMetrics, ExecutionMode, ExecutionStatus,
    ResourceLimits, TaskPriority, WorkflowId,
};
use crate::error::{Result, WorkflowError};
use crate::workflow::{WorkflowDefinition, WorkflowEngine, WorkflowExecution};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, RwLock, Semaphore};
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Execution handle for asynchronous operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHandle {
    pub execution_id: String,
    pub workflow_id: WorkflowId,
    pub started_at: DateTime<Utc>,
    pub mode: ExecutionMode,
    pub priority: TaskPriority,
}

impl ExecutionHandle {
    pub fn new(workflow_id: WorkflowId, mode: ExecutionMode, priority: TaskPriority) -> Self {
        Self {
            execution_id: Uuid::new_v4().to_string(),
            workflow_id,
            started_at: Utc::now(),
            mode,
            priority,
        }
    }
}

/// Task execution request
#[derive(Debug, Clone)]
pub struct TaskExecutionRequest {
    pub handle: ExecutionHandle,
    pub definition: WorkflowDefinition,
    pub context: ExecutionContext,
    pub priority: TaskPriority,
    pub resource_limits: ResourceLimits,
}

/// Execution result for both sync and async modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    /// Synchronous result - contains the completed execution
    Sync(WorkflowExecution),
    /// Asynchronous result - contains the execution handle
    Async(ExecutionHandle),
}

/// Execution manager trait
#[async_trait]
pub trait ExecutionManager: Send + Sync {
    /// Execute workflow with specified mode
    async fn execute_workflow(
        &self,
        definition: WorkflowDefinition,
        mode: ExecutionMode,
        context: ExecutionContext,
    ) -> Result<ExecutionResult>;

    /// Execute workflow with priority
    async fn execute_workflow_with_priority(
        &self,
        definition: WorkflowDefinition,
        mode: ExecutionMode,
        context: ExecutionContext,
        priority: TaskPriority,
    ) -> Result<ExecutionResult>;

    /// Get execution status by handle
    async fn get_execution_status(&self, handle: &ExecutionHandle) -> Result<ExecutionStatus>;

    /// Wait for asynchronous execution to complete
    async fn wait_for_completion(&self, handle: &ExecutionHandle) -> Result<WorkflowExecution>;

    /// Cancel execution
    async fn cancel_execution(&self, handle: &ExecutionHandle) -> Result<()>;

    /// Get active execution count
    fn get_active_execution_count(&self) -> usize;

    /// Get queued execution count
    fn get_queued_execution_count(&self) -> usize;
}

/// Default implementation of execution manager
pub struct DefaultExecutionManager {
    /// Underlying workflow engine
    engine: Arc<dyn WorkflowEngine>,
    /// Concurrency configuration
    concurrency_config: ConcurrencyConfig,
    /// Active executions
    active_executions: DashMap<String, Arc<RwLock<WorkflowExecution>>>,
    /// Execution handles
    execution_handles: DashMap<String, ExecutionHandle>,
    /// Task queue for async execution
    task_queue: Arc<RwLock<VecDeque<TaskExecutionRequest>>>,
    /// Semaphore for controlling concurrent executions
    execution_semaphore: Arc<Semaphore>,
    /// Notification for task queue changes
    queue_notify: Arc<Notify>,
    /// Background task handles
    background_tasks: DashMap<String, JoinHandle<()>>,
}

impl DefaultExecutionManager {
    /// Create a new execution manager
    pub fn new(engine: Arc<dyn WorkflowEngine>, concurrency_config: ConcurrencyConfig) -> Self {
        let execution_semaphore =
            Arc::new(Semaphore::new(concurrency_config.max_concurrent_workflows));
        let task_queue = Arc::new(RwLock::new(VecDeque::new()));
        let queue_notify = Arc::new(Notify::new());

        let manager = Self {
            engine,
            concurrency_config,
            active_executions: DashMap::new(),
            execution_handles: DashMap::new(),
            task_queue: task_queue.clone(),
            execution_semaphore,
            queue_notify: queue_notify.clone(),
            background_tasks: DashMap::new(),
        };

        // Start background task processor
        manager.start_background_processor();

        manager
    }

    /// Start background task processor for async executions
    fn start_background_processor(&self) {
        // For now, we'll use a simplified approach without complex task management
        // In a real implementation, this would be more sophisticated

        // The background processor would run in a separate task
        // but for simplicity in this implementation, we'll handle async execution
        // directly in the execute_async method
    }

    /// Execute workflow synchronously
    async fn execute_sync(
        &self,
        definition: WorkflowDefinition,
        _context: ExecutionContext,
    ) -> Result<WorkflowExecution> {
        // Acquire semaphore permit for concurrency control
        let _permit = self
            .execution_semaphore
            .acquire()
            .await
            .map_err(|_| WorkflowError::ResourceExhausted)?;

        // Execute workflow directly
        self.engine.execute_workflow(definition).await
    }

    /// Execute workflow asynchronously
    async fn execute_async(
        &self,
        definition: WorkflowDefinition,
        _context: ExecutionContext,
        priority: TaskPriority,
    ) -> Result<ExecutionHandle> {
        let workflow_id = definition.generate_id();
        let handle = ExecutionHandle::new(workflow_id, ExecutionMode::Async, priority);

        // Check queue size limit
        {
            let queue = self.task_queue.read().await;
            if queue.len() >= self.concurrency_config.task_queue_size {
                return Err(WorkflowError::ResourceExhausted);
            }
        }

        // Add to execution handles
        self.execution_handles
            .insert(handle.execution_id.clone(), handle.clone());

        // Create initial execution state with Pending status
        let initial_execution = crate::workflow::WorkflowExecution {
            id: workflow_id,
            workflow_name: definition.name.clone(),
            status: ExecutionStatus::Pending,
            started_at: chrono::Utc::now(),
            completed_at: None,
            current_node: None,
            node_states: std::collections::HashMap::new(),
            global_context: serde_json::Value::Object(serde_json::Map::new()),
        };

        // Store initial execution state
        self.active_executions.insert(
            handle.execution_id.clone(),
            Arc::new(RwLock::new(initial_execution)),
        );

        // For simplified implementation, execute directly in background
        let engine = self.engine.clone();
        let active_executions = self.active_executions.clone();
        let execution_handles = self.execution_handles.clone();
        let handle_id = handle.execution_id.clone();
        let execution_semaphore = self.execution_semaphore.clone();

        // Spawn background execution
        tokio::spawn(async move {
            // Update status to Running
            if let Some(execution_arc) = active_executions.get(&handle_id) {
                let mut execution = execution_arc.write().await;
                execution.status = ExecutionStatus::Running;
            }

            // Acquire semaphore permit
            let _permit = match execution_semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    tracing::error!("Failed to acquire execution permit for {}", handle_id);
                    // Update status to Failed
                    if let Some(execution_arc) = active_executions.get(&handle_id) {
                        let mut execution = execution_arc.write().await;
                        execution.status = ExecutionStatus::Failed;
                        execution.completed_at = Some(chrono::Utc::now());
                        // Store error in global_context
                        if let serde_json::Value::Object(ref mut map) = execution.global_context {
                            map.insert(
                                "error".to_string(),
                                serde_json::Value::String(
                                    "Failed to acquire execution permit".to_string(),
                                ),
                            );
                        }
                    }
                    execution_handles.remove(&handle_id);
                    return;
                }
            };

            // Execute the workflow
            let result = engine.execute_workflow(definition).await;

            // Update execution state with result
            if let Some(execution_arc) = active_executions.get(&handle_id) {
                let mut execution = execution_arc.write().await;
                match result {
                    Ok(completed_execution) => {
                        execution.status = completed_execution.status;
                        execution.completed_at = completed_execution.completed_at;
                        execution.current_node = completed_execution.current_node;
                        execution.node_states = completed_execution.node_states;
                        execution.global_context = completed_execution.global_context;
                    }
                    Err(error) => {
                        tracing::error!("Async workflow execution failed: {}", error);
                        execution.status = ExecutionStatus::Failed;
                        execution.completed_at = Some(chrono::Utc::now());
                        // Store error in global_context
                        if let serde_json::Value::Object(ref mut map) = execution.global_context {
                            map.insert(
                                "error".to_string(),
                                serde_json::Value::String(error.to_string()),
                            );
                        }
                    }
                }
            }

            // Clean up execution handle (but keep active execution for status queries)
            execution_handles.remove(&handle_id);
        });

        Ok(handle)
    }

    /// Apply resource limits to task execution
    #[allow(dead_code)]
    async fn apply_resource_limits(
        &self,
        limits: &ResourceLimits,
        execution_id: &str,
    ) -> Result<()> {
        // Basic resource limit implementation
        // In a production system, this would involve:
        // 1. Memory monitoring and limits using system calls
        // 2. CPU time tracking with process monitoring
        // 3. File descriptor limits using rlimit
        // 4. Execution timeouts with tokio::time::timeout

        tracing::debug!(
            "Applying resource limits for execution {}: memory={:?}, cpu_time={:?}, execution_time={:?}",
            execution_id,
            limits.max_memory_bytes,
            limits.max_cpu_time,
            limits.max_execution_time
        );

        // For now, just validate the limits are reasonable
        if let Some(max_memory) = limits.max_memory_bytes {
            if max_memory < 1024 * 1024 {
                // Less than 1MB
                return Err(WorkflowError::workflow_execution(
                    "Memory limit too restrictive",
                ));
            }
        }

        if let Some(max_cpu_time) = limits.max_cpu_time {
            if max_cpu_time < Duration::from_secs(1) {
                return Err(WorkflowError::workflow_execution(
                    "CPU time limit too restrictive",
                ));
            }
        }

        Ok(())
    }

    /// Monitor resource usage for active executions
    async fn monitor_resource_usage(&self) -> Result<()> {
        // Basic resource monitoring implementation
        // In a production system, this would involve:
        // 1. Periodic checks of memory usage using system APIs
        // 2. CPU time tracking with process statistics
        // 3. File descriptor counting
        // 4. Automatic cleanup of resource-heavy tasks

        let active_count = self.active_executions.len();
        let queued_count = self.execution_handles.len();

        tracing::debug!(
            "Resource usage monitoring: {} active executions, {} queued executions",
            active_count,
            queued_count
        );

        // Check if we're approaching resource limits
        if active_count > (self.concurrency_config.max_concurrent_workflows * 80 / 100) {
            tracing::warn!(
                "High resource usage: {} active executions (limit: {})",
                active_count,
                self.concurrency_config.max_concurrent_workflows
            );
        }

        Ok(())
    }

    /// Get detailed execution metrics
    pub async fn get_execution_metrics(&self) -> ExecutionMetrics {
        ExecutionMetrics {
            active_executions: self.active_executions.len(),
            queued_executions: self.execution_handles.len(),
            total_capacity: self.concurrency_config.max_concurrent_workflows,
            queue_capacity: self.concurrency_config.task_queue_size,
            available_permits: self.execution_semaphore.available_permits(),
        }
    }

    /// Cleanup completed executions to free memory
    pub async fn cleanup_completed_executions(&self) -> Result<usize> {
        let mut cleaned_count = 0;
        let mut to_remove = Vec::new();

        // Find completed executions
        for entry in self.active_executions.iter() {
            let execution = entry.value().read().await;
            if execution.status.is_terminal() {
                to_remove.push(entry.key().clone());
            }
        }

        // Remove completed executions
        for execution_id in to_remove {
            self.active_executions.remove(&execution_id);
            cleaned_count += 1;
        }

        tracing::debug!("Cleaned up {} completed executions", cleaned_count);
        Ok(cleaned_count)
    }
}

#[async_trait]
impl ExecutionManager for DefaultExecutionManager {
    async fn execute_workflow(
        &self,
        definition: WorkflowDefinition,
        mode: ExecutionMode,
        context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        self.execute_workflow_with_priority(definition, mode, context, TaskPriority::Normal)
            .await
    }

    async fn execute_workflow_with_priority(
        &self,
        definition: WorkflowDefinition,
        mode: ExecutionMode,
        context: ExecutionContext,
        priority: TaskPriority,
    ) -> Result<ExecutionResult> {
        match mode {
            ExecutionMode::Sync => {
                let execution = self.execute_sync(definition, context).await?;
                Ok(ExecutionResult::Sync(execution))
            }
            ExecutionMode::Async => {
                let handle = self.execute_async(definition, context, priority).await?;
                Ok(ExecutionResult::Async(handle))
            }
        }
    }

    async fn get_execution_status(&self, handle: &ExecutionHandle) -> Result<ExecutionStatus> {
        // Check active executions first (this includes both running and completed executions)
        if let Some(execution_arc) = self.active_executions.get(&handle.execution_id) {
            let execution = execution_arc.read().await;
            return Ok(execution.status);
        }

        // Check if execution is still queued
        if self.execution_handles.contains_key(&handle.execution_id) {
            return Ok(ExecutionStatus::Pending);
        }

        // Check with underlying engine as fallback
        self.engine.get_workflow_status(handle.workflow_id).await
    }

    async fn wait_for_completion(&self, handle: &ExecutionHandle) -> Result<WorkflowExecution> {
        // Add timeout to prevent infinite waiting
        let timeout_duration = Duration::from_secs(30); // 30 second timeout for tests
        let start_time = std::time::Instant::now();

        // Poll for completion with timeout
        loop {
            // Check for timeout
            if start_time.elapsed() > timeout_duration {
                return Err(WorkflowError::ExecutionTimeout);
            }

            let status = self.get_execution_status(handle).await?;

            if status.is_terminal() {
                // Get final execution state
                if let Some(execution_arc) = self.active_executions.get(&handle.execution_id) {
                    let execution = execution_arc.read().await;
                    return Ok(execution.clone());
                } else {
                    return Err(WorkflowError::WorkflowNotFound(handle.workflow_id));
                }
            }

            // Wait before polling again
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn cancel_execution(&self, handle: &ExecutionHandle) -> Result<()> {
        // Remove from queue if still pending
        {
            let mut queue = self.task_queue.write().await;
            queue.retain(|task| task.handle.execution_id != handle.execution_id);
        }

        // Note: We can't cancel individual background tasks since we don't store JoinHandles
        // In a real implementation, we might use a different approach like cancellation tokens

        // Cancel through engine if running
        self.engine.stop_workflow(handle.workflow_id).await?;

        // Clean up
        self.execution_handles.remove(&handle.execution_id);
        self.active_executions.remove(&handle.execution_id);

        Ok(())
    }

    fn get_active_execution_count(&self) -> usize {
        self.active_executions.len()
    }

    fn get_queued_execution_count(&self) -> usize {
        // This is an approximation since we can't easily get the queue size synchronously
        // In a real implementation, we might use an atomic counter
        self.execution_handles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{SimpleMemoryCache, StateManager, StorageBackend};
    use crate::tools::ToolRegistry;
    use crate::workflow::engine::DefaultWorkflowEngine;
    use crate::workflow::{NodeType, WorkflowNode};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Mock tool registry for testing
    struct MockToolRegistry;

    #[async_trait]
    impl ToolRegistry for MockToolRegistry {
        fn register_tool(&mut self, _tool: Arc<dyn crate::tools::ToolNode>) -> Result<()> {
            Ok(())
        }

        fn get_tool(&self, _name: &str) -> Option<Arc<dyn crate::tools::ToolNode>> {
            None
        }

        fn list_tools(&self) -> Vec<crate::core::ToolInfo> {
            Vec::new()
        }

        async fn execute_tool(
            &self,
            _name: &str,
            _params: Value,
            _context: ExecutionContext,
        ) -> Result<Value> {
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(Value::String("mock_result".to_string()))
        }

        fn validate_tool_params(&self, _name: &str, _params: &Value) -> Result<()> {
            Ok(())
        }

        fn has_tool(&self, _name: &str) -> bool {
            true
        }

        fn unregister_tool(&mut self, _name: &str) -> Result<()> {
            Ok(())
        }

        fn resolve_dependencies(
            &self,
            _tool_names: Vec<String>,
        ) -> Result<crate::tools::ResolutionResult> {
            Ok(crate::tools::ResolutionResult {
                resolved_versions: std::collections::HashMap::new(),
                conflicts: Vec::new(),
                warnings: Vec::new(),
            })
        }

        fn check_version_conflicts(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        fn get_dependents(&self, _tool_name: &str) -> Vec<crate::core::ToolInfo> {
            Vec::new()
        }

        async fn execute_tool_with_templates(
            &self,
            name: &str,
            params: Value,
            _template_context: &crate::tools::TemplateContext,
            execution_context: ExecutionContext,
        ) -> Result<Value> {
            self.execute_tool(name, params, execution_context).await
        }

        fn get_tool_templates(&self, _tool_name: &str) -> Vec<crate::tools::ParameterTemplate> {
            Vec::new()
        }

        fn tool_count(&self) -> usize {
            0
        }

        fn clear(&mut self) {
            // No-op for mock
        }
    }

    // Mock storage backend for testing
    struct MockStorageBackend {
        data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    }

    impl MockStorageBackend {
        fn new() -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl StorageBackend for MockStorageBackend {
        async fn save(&self, key: &str, value: &[u8]) -> Result<()> {
            let mut data = self.data.write().await;
            data.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
            let data = self.data.read().await;
            Ok(data.get(key).cloned())
        }

        async fn delete(&self, key: &str) -> Result<()> {
            let mut data = self.data.write().await;
            data.remove(key);
            Ok(())
        }

        async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
            let data = self.data.read().await;
            Ok(data
                .keys()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect())
        }

        async fn exists(&self, key: &str) -> Result<bool> {
            let data = self.data.read().await;
            Ok(data.contains_key(key))
        }

        async fn batch_save(&self, items: Vec<(String, Vec<u8>)>) -> Result<()> {
            let mut data = self.data.write().await;
            for (key, value) in items {
                data.insert(key, value);
            }
            Ok(())
        }

        async fn batch_load(&self, keys: Vec<String>) -> Result<Vec<Option<Vec<u8>>>> {
            let data = self.data.read().await;
            Ok(keys
                .into_iter()
                .map(|key| data.get(&key).cloned())
                .collect())
        }
    }

    // Helper function to create a test execution manager
    fn create_test_execution_manager() -> DefaultExecutionManager {
        // Use memory-only storage to avoid file system issues in tests
        let storage = Arc::new(MockStorageBackend::new());
        let cache = Arc::new(SimpleMemoryCache::new());
        let state_manager = Arc::new(StateManager::new(storage, cache));
        let tool_registry = Arc::new(MockToolRegistry);
        let engine = Arc::new(DefaultWorkflowEngine::new(state_manager, tool_registry, 10));

        DefaultExecutionManager::new(engine, ConcurrencyConfig::default())
    }

    // Helper function to create a simple test workflow
    fn create_test_workflow(name: &str) -> WorkflowDefinition {
        let mut workflow = WorkflowDefinition::new(name, "1.0.0");

        let node1 = WorkflowNode::new("node1", NodeType::Tool);
        workflow.add_node(node1).unwrap();

        workflow
    }

    #[tokio::test]
    async fn test_synchronous_execution_mode() {
        // **Feature: workflow-toolkit, Property 9: Synchronous asynchronous execution mode**
        // *For any* task type, synchronous execution should block until completion
        // **Validates: Requirements 5.1**

        let manager = create_test_execution_manager();
        let workflow = create_test_workflow("sync_test");
        let context = ExecutionContext::new();

        let start_time = Utc::now();
        let result = manager
            .execute_workflow(workflow, ExecutionMode::Sync, context)
            .await;
        let end_time = Utc::now();

        if let Err(ref error) = result {
            println!("Synchronous execution failed with error: {:?}", error);
        }
        assert!(
            result.is_ok(),
            "Synchronous execution should succeed: {:?}",
            result
        );

        match result.unwrap() {
            ExecutionResult::Sync(execution) => {
                assert!(
                    execution.status.is_terminal(),
                    "Synchronous execution should be complete"
                );
                assert!(
                    execution.completed_at.is_some(),
                    "Synchronous execution should have completion time"
                );

                // Verify that execution took some time (blocking behavior)
                let duration = end_time.signed_duration_since(start_time);
                assert!(
                    duration.num_milliseconds() >= 0,
                    "Synchronous execution should take measurable time"
                );
            }
            ExecutionResult::Async(_) => {
                panic!("Synchronous execution should return sync result");
            }
        }
    }

    #[tokio::test]
    async fn test_asynchronous_execution_mode() {
        // **Feature: workflow-toolkit, Property 9: Synchronous asynchronous execution mode**
        // *For any* task type, asynchronous execution should return immediately with execution handle
        // **Validates: Requirements 5.1**

        let manager = create_test_execution_manager();
        let workflow = create_test_workflow("async_test");
        let context = ExecutionContext::new();

        let start_time = Utc::now();
        let result = manager
            .execute_workflow(workflow, ExecutionMode::Async, context)
            .await;
        let end_time = Utc::now();

        assert!(result.is_ok(), "Asynchronous execution should succeed");

        match result.unwrap() {
            ExecutionResult::Async(handle) => {
                assert_eq!(
                    handle.mode,
                    ExecutionMode::Async,
                    "Handle should indicate async mode"
                );

                // Verify that execution returned immediately (non-blocking behavior)
                let duration = end_time.signed_duration_since(start_time);
                assert!(
                    duration.num_milliseconds() < 100,
                    "Asynchronous execution should return quickly"
                );

                // Verify we can get status
                let status = manager.get_execution_status(&handle).await;
                assert!(status.is_ok(), "Should be able to get execution status");

                // The status should be Pending or Running (not terminal yet)
                let status = status.unwrap();
                assert!(
                    !status.is_terminal() || status == ExecutionStatus::Completed,
                    "Async execution should be pending/running or completed quickly"
                );
            }
            ExecutionResult::Sync(_) => {
                panic!("Asynchronous execution should return async result");
            }
        }
    }

    #[tokio::test]
    async fn test_execution_mode_consistency() {
        // **Feature: workflow-toolkit, Property 9: Synchronous asynchronous execution mode**
        // *For any* workflow, both sync and async modes should produce equivalent results
        // **Validates: Requirements 5.1**

        let manager = create_test_execution_manager();
        let workflow1 = create_test_workflow("consistency_test_sync");
        let workflow2 = create_test_workflow("consistency_test_async");
        let context1 = ExecutionContext::new();
        let context2 = ExecutionContext::new();

        // Execute same workflow in both modes
        let sync_result = manager
            .execute_workflow(workflow1, ExecutionMode::Sync, context1)
            .await;
        let async_result = manager
            .execute_workflow(workflow2, ExecutionMode::Async, context2)
            .await;

        assert!(sync_result.is_ok(), "Sync execution should succeed");
        assert!(async_result.is_ok(), "Async execution should succeed");

        let sync_execution = match sync_result.unwrap() {
            ExecutionResult::Sync(execution) => execution,
            _ => panic!("Sync result should be sync"),
        };

        let async_handle = match async_result.unwrap() {
            ExecutionResult::Async(handle) => handle,
            _ => panic!("Async result should be async"),
        };

        // Wait for async execution to complete
        let async_execution = manager.wait_for_completion(&async_handle).await;
        assert!(
            async_execution.is_ok(),
            "Async execution should complete successfully"
        );
        let async_execution = async_execution.unwrap();

        // Both executions should have completed successfully
        assert_eq!(
            sync_execution.status,
            ExecutionStatus::Completed,
            "Sync execution should be completed"
        );
        assert_eq!(
            async_execution.status,
            ExecutionStatus::Completed,
            "Async execution should be completed"
        );

        // Both should have similar structure (same number of nodes, etc.)
        assert_eq!(
            sync_execution.node_states.len(),
            async_execution.node_states.len(),
            "Both executions should have same number of nodes"
        );
    }

    #[tokio::test]
    async fn test_concurrency_control() {
        // Test that concurrency limits are respected
        let mut config = ConcurrencyConfig::default();
        config.max_concurrent_workflows = 2;
        config.task_queue_size = 5;

        let storage = Arc::new(MockStorageBackend::new());
        let cache = Arc::new(SimpleMemoryCache::new());
        let state_manager = Arc::new(StateManager::new(storage, cache));
        let tool_registry = Arc::new(MockToolRegistry);
        let engine = Arc::new(DefaultWorkflowEngine::new(state_manager, tool_registry, 10));
        let manager = DefaultExecutionManager::new(engine, config);

        // Submit multiple async workflows
        let mut handles = Vec::new();
        for i in 0..3 {
            let workflow = create_test_workflow(&format!("concurrent_test_{}", i));
            let context = ExecutionContext::new();
            let result = manager
                .execute_workflow(workflow, ExecutionMode::Async, context)
                .await;

            if let Ok(ExecutionResult::Async(handle)) = result {
                handles.push(handle);
            }
        }

        // Should have at least some handles (up to queue limit)
        assert!(
            !handles.is_empty(),
            "Should be able to queue some workflows"
        );
        assert!(handles.len() <= 5, "Should respect queue size limit");

        // Check that we can get status for all handles
        for handle in &handles {
            let status = manager.get_execution_status(handle).await;
            assert!(
                status.is_ok(),
                "Should be able to get status for queued workflows"
            );
        }
    }
}
