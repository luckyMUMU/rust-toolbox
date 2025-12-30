//! End-to-end integration tests for the workflow toolkit

use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::{json, Value};

use workflow_toolkit::{
    Config,
    WorkflowDefinition, WorkflowEngine, ExecutionStatus,
    ToolRegistry, 
    workflow::{engine::DefaultWorkflowEngine, WorkflowState},
    Result, WorkflowError, WorkflowConfig,
    config::ConfigManager,
    tools::{BasicToolRegistry, BasicTool, AsyncFunctionExecutor},
    storage::{SimpleMemoryCache, FileStorage, StateManager},
};

/// Test fixture for integration tests
struct IntegrationTestFixture {
    temp_dir: TempDir,
    config: Arc<ConfigManager>,
    state_manager: Arc<StateManager>,
    tool_registry: Arc<BasicToolRegistry>,
    workflow_engine: Arc<DefaultWorkflowEngine>,
}

impl IntegrationTestFixture {
    async fn new() -> Result<Self> {
        let temp_dir = TempDir::new().map_err(|e| {
            WorkflowError::workflow_execution(&format!("Failed to create temp dir: {}", e))
        })?;
        
        // Create configuration manager
        let config = Arc::new(ConfigManager::new(Config::default()));
        
        // Create storage components
        let storage = Arc::new(FileStorage::new(&temp_dir.path().join("storage")).map_err(|e| {
            WorkflowError::workflow_execution(&format!("Failed to create storage: {}", e))
        })?);
        let cache = Arc::new(SimpleMemoryCache::new());
        let state_manager = Arc::new(StateManager::new(storage, cache));
        
        // Create tool registry with test tools
        let mut tool_registry = BasicToolRegistry::new();
        
        // Add test tools
        Self::register_test_tools(&mut tool_registry).await?;
        let tool_registry = Arc::new(tool_registry);
        
        // Create workflow engine
        let workflow_engine = Arc::new(DefaultWorkflowEngine::new(
            state_manager.clone(),
            tool_registry.clone(),
            4,
        ));
        
        Ok(Self {
            temp_dir,
            config,
            state_manager,
            tool_registry,
            workflow_engine,
        })
    }
    
    async fn register_test_tools(registry: &mut BasicToolRegistry) -> Result<()> {
        // Echo tool
        let echo_executor = Arc::new(AsyncFunctionExecutor::new(|params: Value, _context| async move {
            Ok(json!({
                "output": params.get("message").unwrap_or(&json!("Hello, World!")),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }));
        
        let echo_tool = BasicTool::builder()
            .name("echo")
            .version("1.0.0")
            .description("Echo tool for testing")
            .executor(echo_executor)
            .build()
            .map_err(|e| WorkflowError::workflow_execution(&format!("Failed to create echo tool: {}", e)))?;
        
        registry.register_tool(Arc::new(echo_tool))?;
        
        // Math tool
        let math_executor = Arc::new(AsyncFunctionExecutor::new(|params: Value, _context| async move {
            let a = params.get("a").and_then(|v: &Value| v.as_f64()).unwrap_or(0.0);
            let b = params.get("b").and_then(|v: &Value| v.as_f64()).unwrap_or(0.0);
            let operation = params.get("operation").and_then(|v: &Value| v.as_str()).unwrap_or("add");
            
            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => if b != 0.0 { a / b } else { return Err(WorkflowError::workflow_execution("Division by zero")); },
                _ => return Err(WorkflowError::workflow_execution("Unknown operation")),
            };
            
            Ok(json!({
                "result": result,
                "operation": operation,
                "operands": [a, b]
            }))
        }));
        
        let math_tool = BasicTool::builder()
            .name("math")
            .version("1.0.0")
            .description("Math operations tool")
            .executor(math_executor)
            .build()
            .map_err(|e| WorkflowError::workflow_execution(&format!("Failed to create math tool: {}", e)))?;
        
        registry.register_tool(Arc::new(math_tool))?;
        
        Ok(())
    }
}

#[tokio::test]
async fn test_basic_workflow_execution() -> Result<()> {
    let fixture = IntegrationTestFixture::new().await?;
    
    // Create a simple workflow
    let workflow_def = WorkflowDefinition {
        name: "test_basic_workflow".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Basic test workflow".to_string()),
        metadata: std::collections::HashMap::new(),
        nodes: vec![
            workflow_toolkit::workflow::WorkflowNode {
                id: "echo_step".to_string(),
                node_type: workflow_toolkit::workflow::NodeType::Tool,
                tool_name: Some("echo".to_string()),
                parameters: json!({
                    "message": "Hello from workflow!"
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: Vec::new(),
                metadata: std::collections::HashMap::new(),
            }
        ],
        edges: vec![],
        global_config: WorkflowConfig::default(),
    };
    
    // Execute workflow
    let execution = fixture.workflow_engine.execute_workflow(workflow_def).await?;
    
    // Wait for completion with timeout
    let result: Result<ExecutionStatus> = timeout(Duration::from_secs(10), async {
        loop {
            let status = fixture.workflow_engine.get_workflow_status(execution.id).await?;
            if status.is_terminal() {
                return Ok(status);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await.map_err(|_| WorkflowError::workflow_execution("Workflow execution timeout"))?;
    
    let final_status = result?;
    assert_eq!(final_status, ExecutionStatus::Completed);
    
    // Verify the execution was saved to state
    let workflow_state: Option<WorkflowState> = fixture.state_manager.load_workflow_state(execution.id).await?;
    assert!(workflow_state.is_some());
    let state = workflow_state.unwrap();
    assert_eq!(state.execution.status, ExecutionStatus::Completed);
    
    Ok(())
}

#[tokio::test]
async fn test_tool_registry_functionality() -> Result<()> {
    let fixture = IntegrationTestFixture::new().await?;
    
    // Test tool listing
    let tools = fixture.tool_registry.list_tools();
    assert!(tools.len() >= 2); // echo, math
    
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"echo"));
    assert!(tool_names.contains(&"math"));
    
    // Test direct tool execution
    let echo_result = fixture.tool_registry.execute_tool(
        "echo",
        json!({ "message": "Direct execution test" }),
        workflow_toolkit::ExecutionContext::new(),
    ).await?;
    
    assert_eq!(echo_result["output"], "Direct execution test");
    
    // Test math tool
    let math_result = fixture.tool_registry.execute_tool(
        "math",
        json!({ "a": 7, "b": 3, "operation": "multiply" }),
        workflow_toolkit::ExecutionContext::new(),
    ).await?;
    
    assert_eq!(math_result["result"], 21.0);
    
    Ok(())
}

/// Performance benchmark tests
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn benchmark_workflow_execution_throughput() -> Result<()> {
        let fixture = IntegrationTestFixture::new().await?;
        
        let num_workflows = 5;
        let start_time = Instant::now();
        
        let mut handles = Vec::new();
        
        for i in 0..num_workflows {
            let engine = fixture.workflow_engine.clone();
            let handle: tokio::task::JoinHandle<Result<ExecutionStatus>> = tokio::spawn(async move {
                let workflow_def = WorkflowDefinition {
                    name: format!("benchmark_workflow_{}", i),
                    version: "1.0.0".to_string(),
                    description: Some("Benchmark workflow".to_string()),
                    metadata: std::collections::HashMap::new(),
                    nodes: vec![
                        workflow_toolkit::workflow::WorkflowNode {
                            id: "step".to_string(),
                            node_type: workflow_toolkit::workflow::NodeType::Tool,
                            tool_name: Some("echo".to_string()),
                            parameters: json!({
                                "message": format!("Benchmark message {}", i)
                            }),
                            retry_policy: None,
                            timeout: Some(Duration::from_secs(30)),
                            depends_on: Vec::new(),
                            metadata: std::collections::HashMap::new(),
                        }
                    ],
                    edges: vec![],
                    global_config: WorkflowConfig::default(),
                };
                
                let execution = engine.execute_workflow(workflow_def).await?;
                
                // Wait for completion
                loop {
                    let status = engine.get_workflow_status(execution.id).await?;
                    if status.is_terminal() {
                        return Ok(status);
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all workflows to complete
        for handle in handles {
            let result = handle.await.map_err(|e| {
                WorkflowError::workflow_execution(&format!("Task join error: {}", e))
            })??;
            assert_eq!(result, ExecutionStatus::Completed);
        }
        
        let elapsed = start_time.elapsed();
        let throughput = num_workflows as f64 / elapsed.as_secs_f64();
        
        println!("Workflow execution throughput: {:.2} workflows/second", throughput);
        
        // Should be able to execute at least 0.5 workflows per second
        assert!(throughput >= 0.5);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn benchmark_tool_execution_latency() -> Result<()> {
        let fixture = IntegrationTestFixture::new().await?;
        
        let num_executions = 50;
        let mut total_duration = Duration::new(0, 0);
        
        for i in 0..num_executions {
            let start_time = Instant::now();
            
            let result = fixture.tool_registry.execute_tool(
                "echo",
                json!({ "message": format!("Latency test {}", i) }),
                workflow_toolkit::ExecutionContext::new(),
            ).await?;
            
            let elapsed = start_time.elapsed();
            total_duration += elapsed;
            
            assert_eq!(result["output"], format!("Latency test {}", i));
        }
        
        let avg_latency = total_duration / num_executions;
        println!("Average tool execution latency: {:?}", avg_latency);
        
        // Should complete in less than 10ms on average
        assert!(avg_latency < Duration::from_millis(10));
        
        Ok(())
    }
}