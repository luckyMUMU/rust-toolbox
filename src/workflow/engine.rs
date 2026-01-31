//! Refactored workflow execution engine (v2).
//!
//! This module provides a refactored workflow engine following LiteFlow design principles:
//! - Component-based architecture
//! - Single responsibility principle
//! - True parallel execution
//! - Unified state management
//!
//! # Architecture
//!
//! The engine delegates execution to components through an executor chain:
//!
//! ```text
//! WorkflowEngine
//!     ├── ComponentRegistry (stores components)
//!     ├── ExecutorChain (handles cross-cutting concerns)
//!     ├── ExecutionTracker (unified state management)
//!     └── DagScheduler (determines execution order)
//! ```

use async_trait::async_trait;
use crate::core::{ExecutionContext, ExecutionStatus};
use crate::error::{Result, WorkflowError};
use crate::storage::StateManager;
use crate::tools::ToolRegistry;
use crate::workflow::component::{ComponentOutput, ComponentRegistry, ComponentStatus};
use crate::workflow::context::DataContext;
use crate::workflow::executor::{BoxedExecutor, ExecutorChainBuilder};
use crate::workflow::flow_node::FlowNode;
use crate::workflow::scheduler::DagScheduler;
use crate::workflow::state::{CheckpointManager, ExecutionTracker};
use crate::workflow::{
    AuditLogger, CacheConfig, ResultCache, WorkflowDefinition, WorkflowExecution,
};
use chrono::Utc;
use futures::future::join_all;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Context for the workflow execution loop to reduce argument count
struct WorkflowLoopContext<'a> {
    workflow_id: Uuid,
    component_registry: &'a ComponentRegistry,
    tracker: Arc<ExecutionTracker>,
}

/// Refactored workflow engine with component-based architecture.
///
/// This engine implements true parallel execution and follows
/// the single responsibility principle.
pub struct RefactoredWorkflowEngine {
    /// State manager for persistence
    state_manager: Arc<StateManager>,

    /// Tool registry for creating tool components
    tool_registry: Arc<dyn ToolRegistry>,

    /// Executor chain for component execution
    executor: BoxedExecutor,

    /// Audit logger
    audit_logger: Arc<AuditLogger>,

    /// Result cache (optional)
    result_cache: Option<Arc<ResultCache>>,

    /// Maximum parallel workflows
    #[allow(dead_code)]
    max_parallel_workflows: usize,

    /// Workflow semaphore for controlling concurrent workflow execution
    workflow_semaphore: Arc<Semaphore>,

    /// Default max concurrency for parallel nodes
    default_max_concurrency: usize,

    /// Checkpoint interval
    checkpoint_interval: Duration,
}

/// Type alias for backward compatibility
pub type DefaultWorkflowEngine = RefactoredWorkflowEngine;

/// WorkflowEngine trait for execution managers
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// Execute a workflow definition
    async fn execute(
        &self,
        definition: WorkflowDefinition,
        initial_params: HashMap<String, Value>,
    ) -> Result<WorkflowExecution>;
}

#[async_trait]
impl WorkflowEngine for RefactoredWorkflowEngine {
    async fn execute(
        &self,
        definition: WorkflowDefinition,
        initial_params: HashMap<String, Value>,
    ) -> Result<WorkflowExecution> {
        self.execute(definition, initial_params).await
    }
}

impl RefactoredWorkflowEngine {
    /// Create a new refactored workflow engine.
    pub fn new(
        state_manager: Arc<StateManager>,
        tool_registry: Arc<dyn ToolRegistry>,
        max_parallel_workflows: usize,
    ) -> Self {
        let audit_logger = Arc::new(AuditLogger::new(state_manager.clone(), false, 30));

        // Build the executor chain: Basic -> Retry -> Audit
        let executor = ExecutorChainBuilder::new()
            .with_retry(3, Duration::from_secs(1))
            .with_audit(audit_logger.clone())
            .build();

        Self {
            state_manager,
            tool_registry,
            executor,
            audit_logger,
            result_cache: None,
            max_parallel_workflows,
            workflow_semaphore: Arc::new(Semaphore::new(max_parallel_workflows)),
            default_max_concurrency: 4,
            checkpoint_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Create a new engine with caching enabled.
    pub fn new_with_cache(
        state_manager: Arc<StateManager>,
        tool_registry: Arc<dyn ToolRegistry>,
        max_parallel_workflows: usize,
        cache_config: CacheConfig,
    ) -> Self {
        let mut engine = Self::new(state_manager.clone(), tool_registry, max_parallel_workflows);

        if cache_config.enabled {
            let cache_backend = state_manager.get_cache_backend();
            engine.result_cache = Some(Arc::new(ResultCache::new(cache_backend, cache_config)));

            // Rebuild executor chain with cache
            engine.executor = ExecutorChainBuilder::new()
                .with_retry(3, Duration::from_secs(1))
                .with_cache(
                    crate::workflow::executor::cache::CacheBuilder::new()
                        .max_capacity(1000)
                        .time_to_live(Duration::from_secs(3600))
                        .build(),
                )
                .with_audit(engine.audit_logger.clone())
                .build();
        }

        engine
    }

    /// Execute a workflow definition.
    pub async fn execute(
        &self,
        definition: WorkflowDefinition,
        initial_params: HashMap<String, Value>,
    ) -> Result<WorkflowExecution> {
        // Acquire workflow semaphore
        let _permit = self
            .workflow_semaphore
            .acquire()
            .await
            .map_err(|_| WorkflowError::execution("Failed to acquire workflow permit"))?;

        let workflow_id = Uuid::new_v4();

        tracing::info!(
            workflow_id = %workflow_id,
            workflow_name = %definition.name,
            "Starting workflow execution"
        );

        // Validate the workflow
        definition.validate()?;

        // Create component registry and register components
        let component_registry = ComponentRegistry::new();
        component_registry.register_from_definition(&definition, self.tool_registry.clone())?;

        // Initialize data context
        let mut context = DataContext::new();
        if !initial_params.is_empty() {
            context.set_input_params(initial_params)?;
        }

        // Create execution tracker
        let tracker = Arc::new(ExecutionTracker::new(workflow_id, &definition.name));
        tracker.initialize_nodes(definition.nodes.iter().map(|n| n.id.clone()));
        tracker.mark_running().await;

        // Create scheduler
        let mut scheduler = DagScheduler::from_workflow(&definition)?;

        // Create checkpoint manager
        let checkpoint_manager = Arc::new(CheckpointManager::new(
            self.state_manager.clone(),
            self.checkpoint_interval,
        ));

        // Execute the workflow
        let loop_ctx = WorkflowLoopContext {
            workflow_id,
            component_registry: &component_registry,
            tracker: tracker.clone(),
        };

        let result = self.execute_workflow_loop(loop_ctx, &mut context, &mut scheduler, checkpoint_manager)
            .await;

        // Build final execution result
        let final_status = tracker.status().await;
        let execution = self
            .build_workflow_execution(workflow_id, &definition.name, final_status, tracker.clone())
            .await;

        match result {
            Ok(_) => {
                tracing::info!(
                    workflow_id = %workflow_id,
                    status = ?final_status,
                    "Workflow execution completed"
                );
                Ok(execution)
            }
            Err(e) => {
                tracing::error!(
                    workflow_id = %workflow_id,
                    error = %e,
                    "Workflow execution failed"
                );
                Err(e)
            }
        }
    }

    /// Main workflow execution loop.
    async fn execute_workflow_loop(
        &self,
        ctx: WorkflowLoopContext<'_>,
        context: &mut DataContext,
        scheduler: &mut DagScheduler,
        checkpoint_manager: Arc<CheckpointManager>,
    ) -> Result<()> {
        let tracker = &ctx.tracker;
        while !scheduler.is_execution_complete() {
            // Check for stop/pause signals
            if tracker.should_stop().await {
                tracker.mark_failed().await;
                return Err(WorkflowError::ExecutionCancelled);
            }

            if tracker.should_pause().await {
                tracker.mark_paused().await;
                self.wait_for_resume(tracker.clone()).await?;
                tracker.mark_running().await;
            }

            // Get nodes ready for execution
            let ready_nodes =
                scheduler.get_parallel_executable_nodes(Some(self.default_max_concurrency));

            if ready_nodes.is_empty() {
                // No nodes ready, but not complete - might be waiting for parallel nodes
                if scheduler.has_executing_nodes() {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }
                break;
            }

            // Execute nodes in parallel
            let results = self
                .execute_nodes_parallel(&ctx, &ready_nodes, context, scheduler)
                .await;

            // Process results
            for (node_id, result) in ready_nodes.iter().zip(results.into_iter()) {
                match result {
                    Ok(output) => {
                        self.handle_node_success(node_id, output, tracker.clone(), scheduler)
                            .await?;
                    }
                    Err(e) => {
                        self.handle_node_failure(node_id, e, tracker.clone(), scheduler)?;
                    }
                }
            }

            // Maybe create checkpoint
            if checkpoint_manager.should_checkpoint().await {
                let _ = checkpoint_manager
                    .create_checkpoint(
                        ctx.workflow_id,
                        tracker.workflow_name(),
                        tracker.get_all_node_states(),
                        context,
                    )
                    .await;
            }
        }

        // Determine final status
        let stats = tracker.get_stats();
        if stats.has_failures() {
            tracker.mark_failed().await;
        } else {
            tracker.mark_completed().await;
        }

        Ok(())
    }

    /// Execute multiple nodes in parallel.
    ///
    /// This is the core method that enables true parallel execution.
    async fn execute_nodes_parallel(
        &self,
        ctx: &WorkflowLoopContext<'_>,
        node_ids: &[String],
        context: &mut DataContext,
        scheduler: &mut DagScheduler,
    ) -> Vec<Result<ComponentOutput>> {
        let tracker = &ctx.tracker;
        let component_registry = ctx.component_registry;
        let workflow_id = ctx.workflow_id;

        // Mark all nodes as executing in scheduler
        for node_id in node_ids {
            let _ = scheduler.mark_node_started(node_id);
            tracker.mark_node_started(node_id).await;
        }

        // Create semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.default_max_concurrency));

        // Build futures for parallel execution
        let futures: Vec<_> = node_ids
            .iter()
            .map(|node_id| {
                let node_id = node_id.clone();
                let sem = Arc::clone(&semaphore);
                let executor = Arc::clone(&self.executor);
                let registry = component_registry;
                let wf_id = workflow_id;

                // Clone context for each parallel execution
                let mut node_context = context.enter_scope(&node_id);

                async move {
                    // Acquire semaphore permit
                    let _permit = sem.acquire().await.map_err(|_| {
                        WorkflowError::execution("Failed to acquire parallel permit")
                    })?;

                    // Get the component
                    let component = registry.get(&node_id)?;

                    // Create execution context
                    let exec_ctx = ExecutionContext::new().with_workflow_id(wf_id);

                    // Execute through the executor chain
                    executor
                        .execute(component.as_ref(), &mut node_context, &exec_ctx)
                        .await
                }
            })
            .collect();

        // Execute all futures in parallel and collect results
        join_all(futures).await
    }

    /// Handle successful node execution.
    async fn handle_node_success(
        &self,
        node_id: &str,
        output: ComponentOutput,
        tracker: Arc<ExecutionTracker>,
        scheduler: &mut DagScheduler,
    ) -> Result<()> {
        match output.status {
            ComponentStatus::Success => {
                tracker.mark_node_completed(node_id, output.result);
                scheduler.mark_node_completed(node_id)?;
            }
            ComponentStatus::Failure(msg) => {
                tracker.mark_node_failed(node_id, &msg);
                scheduler.mark_node_failed(node_id)?;
            }
            ComponentStatus::Skip => {
                tracker.mark_node_skipped(node_id);
                scheduler.mark_node_completed(node_id)?;
            }
            ComponentStatus::Break | ComponentStatus::Continue => {
                // These are handled specially for loop components
                tracker.mark_node_completed(node_id, output.result);
                scheduler.mark_node_completed(node_id)?;
            }
        }
        Ok(())
    }

    /// Handle node execution failure.
    fn handle_node_failure(
        &self,
        node_id: &str,
        error: WorkflowError,
        tracker: Arc<ExecutionTracker>,
        scheduler: &mut DagScheduler,
    ) -> Result<()> {
        tracing::error!(
            node_id = %node_id,
            error = %error,
            "Node execution failed"
        );

        tracker.mark_node_failed(node_id, error.to_string());
        scheduler.mark_node_failed(node_id)?;

        Ok(())
    }

    /// Wait for resume signal after pause.
    async fn wait_for_resume(&self, tracker: Arc<ExecutionTracker>) -> Result<()> {
        loop {
            if tracker.should_stop().await {
                return Err(WorkflowError::ExecutionCancelled);
            }
            if !tracker.should_pause().await {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Build the final WorkflowExecution result.
    async fn build_workflow_execution(
        &self,
        workflow_id: Uuid,
        workflow_name: &str,
        status: ExecutionStatus,
        tracker: Arc<ExecutionTracker>,
    ) -> WorkflowExecution {
        WorkflowExecution {
            id: workflow_id,
            workflow_name: workflow_name.to_string(),
            status,
            started_at: tracker.started_at(),
            completed_at: if status.is_terminal() {
                Some(Utc::now())
            } else {
                None
            },
            current_node: None,
            node_states: tracker.get_all_node_states(),
            global_context: Value::Null,
        }
    }

    /// Pause a running workflow.
    pub async fn pause_workflow(&self, tracker: Arc<ExecutionTracker>) -> Result<()> {
        tracker.request_pause().await;
        Ok(())
    }

    /// Resume a paused workflow.
    pub async fn resume_workflow(&self, tracker: Arc<ExecutionTracker>) -> Result<()> {
        tracker.resume().await;
        Ok(())
    }

    /// Stop a running workflow by tracker.
    pub async fn stop_workflow(&self, tracker: Arc<ExecutionTracker>) -> Result<()> {
        tracker.request_stop().await;
        Ok(())
    }

    /// Stop a running workflow by workflow_id (compatibility method).
    pub async fn stop_workflow_by_id(&self, workflow_id: Uuid) -> Result<()> {
        // TODO: Implement actual workflow stopping by ID
        tracing::info!(workflow_id = %workflow_id, "Stop workflow by ID requested (not yet implemented)");
        Ok(())
    }

    /// Execute a FlowNode execution plan.
    pub async fn execute_flow(
        &self,
        flow: &FlowNode,
        context: &mut DataContext,
        tracker: Arc<ExecutionTracker>,
    ) -> Result<()> {
        match flow {
            FlowNode::Empty => Ok(()),
            FlowNode::Tool {
                id,
                tool_name,
                params,
            } => {
                // Create a temporary component
                let component = crate::workflow::component::ToolComponent::new(
                    id,
                    tool_name,
                    self.tool_registry.clone(),
                    params.clone(),
                );

                let exec_ctx = ExecutionContext::new().with_workflow_id(tracker.workflow_id());

                // Mark started
                tracker.mark_node_started(id).await;

                // Execute
                let result = self.executor.execute(&component, context, &exec_ctx).await;

                match result {
                    Ok(output) => {
                        if output.status.is_success() {
                            tracker.mark_node_completed(id, output.result);
                        } else if output.status.is_failure() {
                            // Extract error message
                            let msg = if let ComponentStatus::Failure(m) = output.status {
                                m
                            } else {
                                "Unknown error".to_string()
                            };
                            tracker.mark_node_failed(id, &msg);
                            return Err(WorkflowError::execution(msg));
                        }
                    }
                    Err(e) => {
                        tracker.mark_node_failed(id, e.to_string());
                        return Err(e);
                    }
                }
                Ok(())
            }
            FlowNode::Chain(nodes) => {
                for node in nodes {
                    Box::pin(self.execute_flow(node, context, tracker.clone())).await?;
                }
                Ok(())
            }
            FlowNode::Parallel(nodes) => {
                // Execute in parallel
                let futures: Vec<_> = nodes
                    .iter()
                    .map(|node| {
                        let tracker = tracker.clone();
                        // Clone context for each branch
                        let mut branch_context = context.clone();

                        async move {
                            Box::pin(self.execute_flow(node, &mut branch_context, tracker)).await
                        }
                    })
                    .collect();

                let results = join_all(futures).await;
                // Check errors
                for res in results {
                    res?;
                }
                Ok(())
            }
            // TODO: Implement Switch and Loop
            _ => Ok(()),
        }
    }

    /// Execute workflow (alias for compatibility)
    pub async fn execute_workflow(&self, definition: WorkflowDefinition) -> Result<WorkflowExecution> {
        self.execute(definition, HashMap::new()).await
    }

    /// Get workflow status (stub for compatibility)
    pub async fn get_workflow_status(&self, _workflow_id: Uuid) -> Result<ExecutionStatus> {
        // TODO: Implement actual status retrieval from state manager
        Ok(ExecutionStatus::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{FileStorage, SimpleMemoryCache};
    use crate::tools::BasicToolRegistry;
    use crate::workflow::definition::{WorkflowEdge, WorkflowNode};
    use tempfile::TempDir;

    async fn create_test_engine() -> (RefactoredWorkflowEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        let storage = Arc::new(FileStorage::new(&temp_dir.path().join("storage")).unwrap());
        let cache = Arc::new(SimpleMemoryCache::new());
        let state_manager = Arc::new(StateManager::new(storage, cache));

        let mut tool_registry = BasicToolRegistry::new();

        // Register a simple echo tool
        let echo_executor = Arc::new(crate::tools::AsyncFunctionExecutor::new(
            |params, _ctx| Box::pin(async move { Ok(params) }),
        ));

        let echo_tool = crate::tools::BasicTool::builder()
            .name("echo")
            .version("1.0.0")
            .description("Echo tool")
            .executor(echo_executor)
            .build()
            .unwrap();

        tool_registry.register_tool(Arc::new(echo_tool)).unwrap();
        let tool_registry = Arc::new(tool_registry);

        let engine = RefactoredWorkflowEngine::new(state_manager, tool_registry, 4);

        (engine, temp_dir)
    }

    #[tokio::test]
    async fn test_simple_workflow_execution() {
        let (engine, _temp_dir) = create_test_engine().await;

        let mut workflow = WorkflowDefinition::new("test-workflow", "1.0.0");
        workflow
            .add_node(WorkflowNode::tool("step1", "echo"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("step2", "echo"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("step1", "step2"))
            .unwrap();

        let result = engine.execute(workflow, HashMap::new()).await;
        assert!(result.is_ok());

        let execution = result.unwrap();
        assert_eq!(execution.status, ExecutionStatus::Completed);
    }

    #[tokio::test]
    async fn test_parallel_workflow_execution() {
        let (engine, _temp_dir) = create_test_engine().await;

        // Create a workflow with parallel nodes:
        // start -> [task_a, task_b, task_c] -> end
        let mut workflow = WorkflowDefinition::new("parallel-workflow", "1.0.0");

        workflow
            .add_node(WorkflowNode::tool("start", "echo"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("task_a", "echo"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("task_b", "echo"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("task_c", "echo"))
            .unwrap();
        workflow
            .add_node(WorkflowNode::tool("end", "echo"))
            .unwrap();

        // Start connects to all parallel tasks
        workflow
            .add_edge(WorkflowEdge::new("start", "task_a"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("start", "task_b"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("start", "task_c"))
            .unwrap();

        // All parallel tasks connect to end
        workflow
            .add_edge(WorkflowEdge::new("task_a", "end"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("task_b", "end"))
            .unwrap();
        workflow
            .add_edge(WorkflowEdge::new("task_c", "end"))
            .unwrap();

        let result = engine.execute(workflow, HashMap::new()).await;
        assert!(result.is_ok());

        let execution = result.unwrap();
        assert_eq!(execution.status, ExecutionStatus::Completed);

        // All nodes should be completed
        assert!(execution
            .node_states
            .get("start")
            .map(|s| s.status == ExecutionStatus::Completed)
            .unwrap_or(false));
        assert!(execution
            .node_states
            .get("task_a")
            .map(|s| s.status == ExecutionStatus::Completed)
            .unwrap_or(false));
        assert!(execution
            .node_states
            .get("task_b")
            .map(|s| s.status == ExecutionStatus::Completed)
            .unwrap_or(false));
        assert!(execution
            .node_states
            .get("task_c")
            .map(|s| s.status == ExecutionStatus::Completed)
            .unwrap_or(false));
        assert!(execution
            .node_states
            .get("end")
            .map(|s| s.status == ExecutionStatus::Completed)
            .unwrap_or(false));
    }
}
