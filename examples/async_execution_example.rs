//! Example demonstrating synchronous and asynchronous execution modes
//! 
//! This example shows how to use the ExecutionManager to run workflows
//! in both synchronous and asynchronous modes, demonstrating the key
//! features of task 20.1: "完善异步任务执行"

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use workflow_toolkit::{
    Config, ExecutionContext, ExecutionMode, TaskPriority,
    WorkflowDefinition, WorkflowNode, NodeType,
    DefaultExecutionManager, ExecutionManager, ExecutionResult,
    ConcurrencyConfig, ResourceLimits,
    storage::{StateManager, SimpleMemoryCache, FileStorage},
    tools::ToolRegistry,
    workflow::DefaultWorkflowEngine,
};

// Mock tool registry for the example
struct ExampleToolRegistry;

#[async_trait::async_trait]
impl ToolRegistry for ExampleToolRegistry {
    fn register_tool(&mut self, _tool: Arc<dyn workflow_toolkit::tools::ToolNode>) -> workflow_toolkit::Result<()> {
        Ok(())
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn workflow_toolkit::tools::ToolNode>> {
        None
    }

    fn list_tools(&self) -> Vec<workflow_toolkit::ToolInfo> {
        Vec::new()
    }

    async fn execute_tool(&self, _name: &str, _params: serde_json::Value, _context: ExecutionContext) -> workflow_toolkit::Result<serde_json::Value> {
        // Simulate some work
        sleep(Duration::from_millis(100)).await;
        Ok(serde_json::Value::String("example_result".to_string()))
    }

    fn validate_tool_params(&self, _name: &str, _params: &serde_json::Value) -> workflow_toolkit::Result<()> {
        Ok(())
    }

    fn has_tool(&self, _name: &str) -> bool {
        true
    }

    fn unregister_tool(&mut self, _name: &str) -> workflow_toolkit::Result<()> {
        Ok(())
    }

    fn tool_count(&self) -> usize {
        1
    }

    fn clear(&mut self) {
        // No-op for example
    }
}

#[tokio::main]
async fn main() -> workflow_toolkit::Result<()> {
    // Initialize logging
    workflow_toolkit::init_logging()?;
    
    println!("🚀 Async Execution Example");
    println!("==========================");
    
    // Create execution manager with custom concurrency config
    let concurrency_config = ConcurrencyConfig {
        max_concurrent_tasks: 50,
        max_concurrent_workflows: 5,
        task_queue_size: 100,
        enable_prioritization: true,
        resource_limits: ResourceLimits {
            max_memory_bytes: Some(512 * 1024 * 1024), // 512MB
            max_cpu_time: Some(Duration::from_secs(60)),
            max_execution_time: Some(Duration::from_secs(120)),
            max_file_descriptors: Some(512),
        },
    };
    
    // Set up storage and engine
    let cache = Arc::new(SimpleMemoryCache::new());
    let storage = Arc::new(FileStorage::new("./tmp/async_example").unwrap());
    let state_manager = Arc::new(StateManager::new(storage, cache));
    let tool_registry = Arc::new(ExampleToolRegistry);
    let engine = Arc::new(DefaultWorkflowEngine::new(state_manager, tool_registry, 10));
    
    let execution_manager = DefaultExecutionManager::new(engine, concurrency_config);
    
    // Create a simple workflow
    let mut workflow = WorkflowDefinition::new("async_example", "1.0.0");
    let node = WorkflowNode::new("example_task", NodeType::Tool);
    workflow.add_node(node)?;
    
    println!("\n📋 Created workflow: {}", workflow.name);
    
    // Demonstrate synchronous execution
    println!("\n🔄 Synchronous Execution:");
    let sync_start = std::time::Instant::now();
    
    let sync_result = execution_manager.execute_workflow(
        workflow.clone(),
        ExecutionMode::Sync,
        ExecutionContext::new(),
    ).await?;
    
    let sync_duration = sync_start.elapsed();
    
    match sync_result {
        ExecutionResult::Sync(execution) => {
            println!("✅ Sync execution completed in {:?}", sync_duration);
            println!("   Status: {:?}", execution.status);
            println!("   Started: {}", execution.started_at);
            if let Some(completed) = execution.completed_at {
                println!("   Completed: {}", completed);
            }
        }
        ExecutionResult::Async(_) => {
            println!("❌ Expected sync result but got async");
        }
    }
    
    // Demonstrate asynchronous execution
    println!("\n⚡ Asynchronous Execution:");
    let async_start = std::time::Instant::now();
    
    let async_result = execution_manager.execute_workflow_with_priority(
        workflow.clone(),
        ExecutionMode::Async,
        ExecutionContext::new(),
        TaskPriority::High,
    ).await?;
    
    let async_submit_duration = async_start.elapsed();
    
    match async_result {
        ExecutionResult::Async(handle) => {
            println!("✅ Async execution submitted in {:?}", async_submit_duration);
            println!("   Execution ID: {}", handle.execution_id);
            println!("   Workflow ID: {}", handle.workflow_id);
            println!("   Priority: {:?}", handle.priority);
            println!("   Mode: {:?}", handle.mode);
            
            // Monitor execution status
            println!("\n📊 Monitoring execution status:");
            loop {
                let status = execution_manager.get_execution_status(&handle).await?;
                println!("   Status: {:?}", status);
                
                if status.is_terminal() {
                    break;
                }
                
                sleep(Duration::from_millis(50)).await;
            }
            
            // Wait for completion and get final result
            println!("\n⏳ Waiting for completion...");
            let final_execution = execution_manager.wait_for_completion(&handle).await?;
            let total_duration = async_start.elapsed();
            
            println!("✅ Async execution completed in {:?}", total_duration);
            println!("   Final Status: {:?}", final_execution.status);
            if let Some(completed) = final_execution.completed_at {
                println!("   Completed: {}", completed);
            }
        }
        ExecutionResult::Sync(_) => {
            println!("❌ Expected async result but got sync");
        }
    }
    
    // Demonstrate concurrent executions
    println!("\n🔀 Concurrent Executions:");
    let mut handles = Vec::new();
    
    for i in 0..3 {
        let mut concurrent_workflow = WorkflowDefinition::new(&format!("concurrent_{}", i), "1.0.0");
        let node = WorkflowNode::new(&format!("task_{}", i), NodeType::Tool);
        concurrent_workflow.add_node(node)?;
        
        let result = execution_manager.execute_workflow(
            concurrent_workflow,
            ExecutionMode::Async,
            ExecutionContext::new(),
        ).await?;
        
        if let ExecutionResult::Async(handle) = result {
            println!("   Submitted workflow {}: {}", i, handle.execution_id);
            handles.push(handle);
        }
    }
    
    // Wait for all concurrent executions
    println!("\n⏳ Waiting for all concurrent executions...");
    for (i, handle) in handles.iter().enumerate() {
        let _execution = execution_manager.wait_for_completion(handle).await?;
        println!("   Workflow {} completed", i);
    }
    
    // Show execution metrics
    println!("\n📈 Execution Metrics:");
    let metrics = execution_manager.get_execution_metrics().await;
    println!("   Active executions: {}", metrics.active_executions);
    println!("   Queued executions: {}", metrics.queued_executions);
    println!("   Total capacity: {}", metrics.total_capacity);
    println!("   Queue capacity: {}", metrics.queue_capacity);
    println!("   Available permits: {}", metrics.available_permits);
    
    // Cleanup completed executions
    let cleaned = execution_manager.cleanup_completed_executions().await?;
    println!("   Cleaned up {} completed executions", cleaned);
    
    println!("\n🎉 Example completed successfully!");
    
    Ok(())
}