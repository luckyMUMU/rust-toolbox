//! Example demonstrating system recovery and caching functionality

use std::sync::Arc;
use tempfile::TempDir;
use workflow_toolkit::{
    core::{ExecutionContext, WorkflowConfig},
    storage::{StateManager, FileStorage, SimpleMemoryCache},
    tools::ToolRegistry,
    workflow::{
        DefaultWorkflowEngine, WorkflowDefinition, WorkflowNode, NodeType,
        CacheConfig, InvalidationStrategy
    },
};
use serde_json::Value;
use std::time::Duration;

// Mock tool registry for demonstration
struct MockToolRegistry;

#[async_trait::async_trait]
impl ToolRegistry for MockToolRegistry {
    fn register_tool(&mut self, _tool: Arc<dyn workflow_toolkit::tools::ToolNode>) -> workflow_toolkit::error::Result<()> {
        Ok(())
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn workflow_toolkit::tools::ToolNode>> {
        None
    }

    fn list_tools(&self) -> Vec<workflow_toolkit::core::ToolInfo> {
        Vec::new()
    }

    async fn execute_tool(&self, _name: &str, _params: Value, _context: ExecutionContext) -> workflow_toolkit::error::Result<Value> {
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(serde_json::json!({"result": "success", "timestamp": chrono::Utc::now()}))
    }

    fn validate_tool_params(&self, _name: &str, _params: &Value) -> workflow_toolkit::error::Result<()> {
        Ok(())
    }

    fn has_tool(&self, _name: &str) -> bool {
        true
    }

    fn unregister_tool(&mut self, _name: &str) -> workflow_toolkit::error::Result<()> {
        Ok(())
    }

    fn tool_count(&self) -> usize {
        0
    }

    fn clear(&mut self) {
        // No-op for mock
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 System Recovery and Caching Example");
    println!("=====================================");

    // Create temporary directory for storage
    let temp_dir = TempDir::new()?;
    let storage_path = temp_dir.path().join("storage");

    // Set up storage and state manager
    let storage = Arc::new(FileStorage::new(&storage_path)?);
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));

    // Set up tool registry
    let tool_registry = Arc::new(MockToolRegistry);

    // Configure caching
    let cache_config = CacheConfig {
        enabled: true,
        default_ttl: Duration::from_secs(300), // 5 minutes
        max_cache_size: 1000,
        invalidation_strategy: InvalidationStrategy::TimeToLive(Duration::from_secs(300)),
        cache_node_results: true,
        cache_workflow_results: true,
    };

    // Create workflow engine with caching
    let engine = Arc::new(DefaultWorkflowEngine::new_with_cache(
        state_manager.clone(),
        tool_registry,
        5, // max parallel workflows
        cache_config,
    ));

    println!("\n📋 Creating sample workflow...");

    // Create a sample workflow
    let mut workflow = WorkflowDefinition::new("data-processing-pipeline", "1.0.0");
    workflow.global_config = WorkflowConfig {
        checkpoint_interval: Some(Duration::from_secs(30)), // Create checkpoints every 30 seconds
        enable_caching: true,
        ..Default::default()
    };

    // Add nodes to the workflow
    let node1 = WorkflowNode::new("fetch_data", NodeType::Tool);
    let node2 = WorkflowNode::new("process_data", NodeType::Tool);
    let node3 = WorkflowNode::new("validate_data", NodeType::Tool);

    workflow.add_node(node1)?;
    workflow.add_node(node2)?;
    workflow.add_node(node3)?;

    // Add edges to create a pipeline
    workflow.add_edge(workflow_toolkit::workflow::WorkflowEdge::new("fetch_data", "process_data"))?;
    workflow.add_edge(workflow_toolkit::workflow::WorkflowEdge::new("process_data", "validate_data"))?;

    println!("✅ Workflow created with {} nodes", workflow.nodes.len());

    // Execute the workflow for the first time
    println!("\n🔄 Executing workflow for the first time...");
    let start_time = std::time::Instant::now();
    let execution1 = engine.execute_workflow(workflow.clone()).await?;
    let first_duration = start_time.elapsed();

    println!("✅ First execution completed in {:?}", first_duration);
    println!("   Status: {:?}", execution1.status);
    println!("   Workflow ID: {}", execution1.id);

    // Execute the same workflow again (should use cache)
    println!("\n🔄 Executing workflow again (should use cache)...");
    let start_time = std::time::Instant::now();
    let execution2 = engine.execute_workflow(workflow.clone()).await?;
    let second_duration = start_time.elapsed();

    println!("✅ Second execution completed in {:?}", second_duration);
    println!("   Status: {:?}", execution2.status);
    println!("   Workflow ID: {}", execution2.id);

    if second_duration < first_duration {
        println!("🚀 Cache hit! Second execution was faster.");
    }

    // Demonstrate cache statistics
    if let Some(cache_stats) = engine.get_cache_stats().await {
        println!("\n📊 Cache Statistics:");
        println!("   Cache size: {} entries", cache_stats.size);
        println!("   Caching enabled: {}", cache_stats.config.enabled);
        println!("   Default TTL: {:?}", cache_stats.config.default_ttl);
    }

    // Demonstrate recovery functionality
    println!("\n🔄 Demonstrating recovery functionality...");
    
    // Create a workflow that we'll simulate as interrupted
    let mut recovery_workflow = WorkflowDefinition::new("recovery-test", "1.0.0");
    recovery_workflow.global_config.checkpoint_interval = Some(Duration::from_millis(100)); // Frequent checkpoints
    
    let recovery_node = WorkflowNode::new("long_running_task", NodeType::Tool);
    recovery_workflow.add_node(recovery_node)?;

    // Start execution (this will create checkpoints)
    println!("   Starting workflow that will create checkpoints...");
    let recovery_execution = engine.execute_workflow(recovery_workflow.clone()).await?;
    
    println!("   Workflow completed: {}", recovery_execution.id);

    // Simulate system restart by checking for recoverable workflows
    println!("\n🔄 Simulating system restart - checking for recoverable workflows...");
    let recovered_workflows = engine.recover_incomplete_workflows().await?;
    
    if recovered_workflows.is_empty() {
        println!("   No incomplete workflows found (all completed successfully)");
    } else {
        println!("   Recovered {} incomplete workflows", recovered_workflows.len());
        for recovered in &recovered_workflows {
            println!("     - Workflow: {} (ID: {})", recovered.workflow_name, recovered.id);
            println!("       Status: {:?}", recovered.status);
        }
    }

    // Demonstrate cache invalidation
    println!("\n🗑️  Demonstrating cache invalidation...");
    let invalidated_count = engine.invalidate_workflow_cache("data-processing-pipeline", Some("1.0.0")).await?;
    println!("   Invalidated {} cache entries for workflow", invalidated_count);

    // Clear all cache
    engine.clear_cache().await?;
    println!("   Cleared all cached results");

    // Final cache statistics
    if let Some(final_stats) = engine.get_cache_stats().await {
        println!("\n📊 Final Cache Statistics:");
        println!("   Cache size: {} entries", final_stats.size);
    }

    println!("\n✅ System Recovery and Caching Example completed successfully!");
    println!("\nKey features demonstrated:");
    println!("  ✓ Workflow result caching with TTL");
    println!("  ✓ Node-level result caching");
    println!("  ✓ Automatic checkpoint creation during execution");
    println!("  ✓ System recovery after restart");
    println!("  ✓ Cache invalidation and management");
    println!("  ✓ Performance improvements through caching");

    Ok(())
}