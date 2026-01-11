//! Comprehensive workflow example demonstrating advanced features
//!
//! This example shows how to:
//! - Create complex workflows with conditions and parallel processing
//! - Handle errors and retries
//! - Use custom tools and plugins
//! - Monitor execution progress
//! - Implement performance optimizations

use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use workflow_toolkit::{
    config::ConfigManager,
    storage::{FileStorage, SimpleMemoryCache, StateManager},
    tools::{AsyncFunctionExecutor, BasicTool, BasicToolRegistry},
    workflow::engine::DefaultWorkflowEngine,
    workflow::{BackoffStrategy, NodeType, RetryPolicy, WorkflowEdge, WorkflowNode},
    Config, ExecutionContext, ExecutionStatus, Result, ToolRegistry, WorkflowConfig,
    WorkflowDefinition, WorkflowEngine, WorkflowError,
};

/// Advanced workflow example with real-world scenarios
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();

    println!("🚀 启动综合工作流示例");

    // 1. Setup components
    let components = setup_workflow_components().await?;

    // 2. Register custom tools
    register_advanced_tools(&components.tool_registry).await?;

    // 3. Create and execute data processing workflow
    println!("\n📊 执行数据处理工作流");
    let data_workflow = create_data_processing_workflow();
    execute_and_monitor_workflow(&components.engine, data_workflow).await?;

    // 4. Create and execute API integration workflow
    println!("\n🌐 执行API集成工作流");
    let api_workflow = create_api_integration_workflow();
    execute_and_monitor_workflow(&components.engine, api_workflow).await?;

    // 5. Create and execute parallel processing workflow
    println!("\n⚡执行并行处理工作流");
    let parallel_workflow = create_parallel_processing_workflow();
    execute_and_monitor_workflow(&components.engine, parallel_workflow).await?;

    // 6. Demonstrate error handling and recovery
    println!("\n🔧 演示错误处理和恢复");
    let error_workflow = create_error_handling_workflow();
    execute_and_monitor_workflow(&components.engine, error_workflow).await?;

    println!("\n✅ 所有工作流示例执行完成");

    Ok(())
}

/// Workflow components container
struct WorkflowComponents {
    engine: Arc<DefaultWorkflowEngine>,
    tool_registry: Arc<BasicToolRegistry>,
    state_manager: Arc<StateManager>,
}

/// Setup all workflow components
async fn setup_workflow_components() -> Result<WorkflowComponents> {
    let temp_dir = tempfile::TempDir::new().map_err(|e| {
        WorkflowError::workflow_execution(&format!("Failed to create temp dir: {}", e))
    })?;

    // Configuration
    let config = Arc::new(ConfigManager::new(Config::default()));

    // Storage
    let storage = Arc::new(
        FileStorage::new(&temp_dir.path().join("storage")).map_err(|e| {
            WorkflowError::workflow_execution(&format!("Failed to create storage: {}", e))
        })?,
    );
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));

    // Tool registry
    let tool_registry = Arc::new(BasicToolRegistry::new());

    // Workflow engine
    let engine = Arc::new(DefaultWorkflowEngine::new(
        state_manager.clone(),
        tool_registry.clone(),
        8, // Higher concurrency for parallel processing
    ));

    Ok(WorkflowComponents {
        engine,
        tool_registry,
        state_manager,
    })
}

/// Register advanced tools for demonstration
async fn register_advanced_tools(registry: &Arc<BasicToolRegistry>) -> Result<()> {
    // Data processor tool
    let data_processor = create_data_processor_tool()?;
    registry.register_tool(Arc::new(data_processor))?;

    // HTTP client tool
    let http_client = create_http_client_tool()?;
    registry.register_tool(Arc::new(http_client))?;

    // File operations tool
    let file_ops = create_file_operations_tool()?;
    registry.register_tool(Arc::new(file_ops))?;

    // Math calculator tool
    let calculator = create_calculator_tool()?;
    registry.register_tool(Arc::new(calculator))?;

    // Data validator tool
    let validator = create_validator_tool()?;
    registry.register_tool(Arc::new(validator))?;

    // Notification sender tool
    let notifier = create_notification_tool()?;
    registry.register_tool(Arc::new(notifier))?;

    // Error simulator tool (for testing error handling)
    let error_simulator = create_error_simulator_tool()?;
    registry.register_tool(Arc::new(error_simulator))?;

    println!("✅ 注册了 {} 个高级工具", 7);

    Ok(())
}

/// Create data processing workflow
fn create_data_processing_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        name: "data-processing-pipeline".to_string(),
        version: "1.0.0".to_string(),
        description: Some("高级数据处理管道".to_string()),
        metadata: [
            ("category".to_string(), json!("data-processing")),
            ("priority".to_string(), json!("high")),
        ]
        .into_iter()
        .collect(),
        nodes: vec![
            WorkflowNode {
                id: "start".to_string(),
                node_type: NodeType::Start,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: Vec::new(),
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "load_data".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("file_operations".to_string()),
                parameters: json!({
                    "operation": "read",
                    "path": "./examples/sample_data.json",
                    "format": "json"
                }),
                retry_policy: Some(RetryPolicy {
                    max_attempts: 3,
                    delay: Duration::from_secs(2),
                    backoff: BackoffStrategy::Exponential,
                }),
                timeout: Some(Duration::from_secs(30)),
                depends_on: vec!["start".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "validate_data".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("data_validator".to_string()),
                parameters: json!({
                    "data": "${load_data.content}",
                    "schema": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["id", "name", "value"]
                        }
                    }
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(15)),
                depends_on: vec!["load_data".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "process_data".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("data_processor".to_string()),
                parameters: json!({
                    "data": "${load_data.content}",
                    "operations": [
                        {"type": "filter", "condition": "value > 10"},
                        {"type": "transform", "field": "value", "operation": "multiply", "factor": 2},
                        {"type": "sort", "field": "name", "order": "asc"}
                    ]
                }),
                retry_policy: Some(RetryPolicy {
                    max_attempts: 2,
                    delay: Duration::from_secs(5),
                    backoff: BackoffStrategy::Linear,
                }),
                timeout: Some(Duration::from_secs(60)),
                depends_on: vec!["validate_data".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "save_results".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("file_operations".to_string()),
                parameters: json!({
                    "operation": "write",
                    "path": "./output/processed_data.json",
                    "content": "${process_data.result}",
                    "format": "json"
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: vec!["process_data".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "send_notification".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("notification_sender".to_string()),
                parameters: json!({
                    "message": "数据处理完成，处理了 ${process_data.record_count} 条记录",
                    "channel": "console",
                    "level": "info"
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(10)),
                depends_on: vec!["save_results".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: NodeType::End,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["send_notification".to_string()],
                metadata: std::collections::HashMap::new(),
            },
        ],
        edges: vec![
            WorkflowEdge {
                from: "start".to_string(),
                to: "load_data".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "load_data".to_string(),
                to: "validate_data".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "validate_data".to_string(),
                to: "process_data".to_string(),
                condition: Some("${validate_data.valid} == true".to_string()),
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "process_data".to_string(),
                to: "save_results".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "save_results".to_string(),
                to: "send_notification".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "send_notification".to_string(),
                to: "end".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
        ],
        global_config: WorkflowConfig::default(),
    }
}

/// Create API integration workflow
fn create_api_integration_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        name: "api-integration-pipeline".to_string(),
        version: "1.0.0".to_string(),
        description: Some("API集成和数据同步工作流".to_string()),
        metadata: [
            ("category".to_string(), json!("integration")),
            ("api_version".to_string(), json!("v1")),
        ]
        .into_iter()
        .collect(),
        nodes: vec![
            WorkflowNode {
                id: "start".to_string(),
                node_type: NodeType::Start,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: Vec::new(),
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "fetch_user_data".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("http_client".to_string()),
                parameters: json!({
                    "method": "GET",
                    "url": "https://jsonplaceholder.typicode.com/users",
                    "headers": {
                        "Accept": "application/json",
                        "User-Agent": "WorkflowToolkit/1.0"
                    },
                    "timeout": 30
                }),
                retry_policy: Some(RetryPolicy {
                    max_attempts: 3,
                    delay: Duration::from_secs(5),
                    backoff: BackoffStrategy::Exponential,
                }),
                timeout: Some(Duration::from_secs(45)),
                depends_on: vec!["start".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "process_users".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("data_processor".to_string()),
                parameters: json!({
                    "data": "${fetch_user_data.body}",
                    "operations": [
                        {"type": "transform", "field": "email", "operation": "lowercase"},
                        {"type": "add_field", "field": "processed_at", "value": "${now()}"},
                        {"type": "select", "fields": ["id", "name", "email", "processed_at"]}
                    ]
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: vec!["fetch_user_data".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "save_to_file".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("file_operations".to_string()),
                parameters: json!({
                    "operation": "write",
                    "path": "./output/users.json",
                    "content": "${process_users.result}",
                    "format": "json"
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(20)),
                depends_on: vec!["process_users".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: NodeType::End,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["save_to_file".to_string()],
                metadata: std::collections::HashMap::new(),
            },
        ],
        edges: vec![
            WorkflowEdge {
                from: "start".to_string(),
                to: "fetch_user_data".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "fetch_user_data".to_string(),
                to: "process_users".to_string(),
                condition: Some("${fetch_user_data.status} == 200".to_string()),
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "process_users".to_string(),
                to: "save_to_file".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "save_to_file".to_string(),
                to: "end".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
        ],
        global_config: WorkflowConfig::default(),
    }
}

/// Create parallel processing workflow
fn create_parallel_processing_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        name: "parallel-computation".to_string(),
        version: "1.0.0".to_string(),
        description: Some("并行计算演示工作流".to_string()),
        metadata: [
            ("category".to_string(), json!("computation")),
            ("parallel".to_string(), json!(true)),
        ]
        .into_iter()
        .collect(),
        nodes: vec![
            WorkflowNode {
                id: "start".to_string(),
                node_type: NodeType::Start,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: Vec::new(),
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "parallel_group".to_string(),
                node_type: NodeType::Parallel,
                tool_name: None,
                parameters: json!({
                    "max_concurrency": 4
                }),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["start".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "calc_fibonacci".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("calculator".to_string()),
                parameters: json!({
                    "operation": "fibonacci",
                    "n": 20
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: vec!["parallel_group".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "calc_factorial".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("calculator".to_string()),
                parameters: json!({
                    "operation": "factorial",
                    "n": 10
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: vec!["parallel_group".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "calc_prime".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("calculator".to_string()),
                parameters: json!({
                    "operation": "prime_check",
                    "n": 97
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: vec!["parallel_group".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "calc_power".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("calculator".to_string()),
                parameters: json!({
                    "operation": "power",
                    "base": 2,
                    "exponent": 16
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: vec!["parallel_group".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "aggregate_results".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("data_processor".to_string()),
                parameters: json!({
                    "data": {
                        "fibonacci": "${calc_fibonacci.result}",
                        "factorial": "${calc_factorial.result}",
                        "prime_check": "${calc_prime.result}",
                        "power": "${calc_power.result}"
                    },
                    "operations": [
                        {"type": "aggregate", "operation": "collect"}
                    ]
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(15)),
                depends_on: vec![
                    "calc_fibonacci".to_string(),
                    "calc_factorial".to_string(),
                    "calc_prime".to_string(),
                    "calc_power".to_string(),
                ],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: NodeType::End,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["aggregate_results".to_string()],
                metadata: std::collections::HashMap::new(),
            },
        ],
        edges: vec![
            WorkflowEdge {
                from: "start".to_string(),
                to: "parallel_group".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "parallel_group".to_string(),
                to: "calc_fibonacci".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "parallel_group".to_string(),
                to: "calc_factorial".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "parallel_group".to_string(),
                to: "calc_prime".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "parallel_group".to_string(),
                to: "calc_power".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "calc_fibonacci".to_string(),
                to: "aggregate_results".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "calc_factorial".to_string(),
                to: "aggregate_results".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "calc_prime".to_string(),
                to: "aggregate_results".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "calc_power".to_string(),
                to: "aggregate_results".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "aggregate_results".to_string(),
                to: "end".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
        ],
        global_config: WorkflowConfig::default(),
    }
}

/// Create error handling workflow
fn create_error_handling_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        name: "error-handling-demo".to_string(),
        version: "1.0.0".to_string(),
        description: Some("错误处理和恢复演示工作流".to_string()),
        metadata: [
            ("category".to_string(), json!("testing")),
            ("error_handling".to_string(), json!(true)),
        ]
        .into_iter()
        .collect(),
        nodes: vec![
            WorkflowNode {
                id: "start".to_string(),
                node_type: NodeType::Start,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: Vec::new(),
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "simulate_error".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("error_simulator".to_string()),
                parameters: json!({
                    "error_type": "temporary",
                    "failure_rate": 0.7,
                    "message": "模拟临时错误"
                }),
                retry_policy: Some(RetryPolicy {
                    max_attempts: 5,
                    delay: Duration::from_secs(2),
                    backoff: BackoffStrategy::Exponential,
                }),
                timeout: Some(Duration::from_secs(10)),
                depends_on: vec!["start".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "recovery_action".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("notification_sender".to_string()),
                parameters: json!({
                    "message": "错误恢复成功: ${simulate_error.result}",
                    "channel": "console",
                    "level": "success"
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(10)),
                depends_on: vec!["simulate_error".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: NodeType::End,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["recovery_action".to_string()],
                metadata: std::collections::HashMap::new(),
            },
        ],
        edges: vec![
            WorkflowEdge {
                from: "start".to_string(),
                to: "simulate_error".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "simulate_error".to_string(),
                to: "recovery_action".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "recovery_action".to_string(),
                to: "end".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
        ],
        global_config: WorkflowConfig::default(),
    }
}

/// Execute workflow and monitor progress
async fn execute_and_monitor_workflow(
    engine: &Arc<DefaultWorkflowEngine>,
    workflow: WorkflowDefinition,
) -> Result<()> {
    let workflow_name = workflow.name.clone();
    println!("  🎯 启动工作流: {}", workflow_name);

    let start_time = std::time::Instant::now();
    let execution = engine.execute_workflow(workflow).await?;

    println!("  📋 执行ID: {}", execution.id);

    // Monitor execution with timeout
    let monitor_result = timeout(Duration::from_secs(120), async {
        let mut last_status = ExecutionStatus::Pending;
        let mut check_count = 0;

        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            check_count += 1;

            match engine.get_workflow_status(execution.id).await {
                Ok(status) => {
                    if status != last_status {
                        println!(
                            "  📊 状态变更: {:?} -> {:?} (检查 #{})",
                            last_status, status, check_count
                        );
                        last_status = status.clone();
                    }

                    if status.is_terminal() {
                        let elapsed = start_time.elapsed();
                        match status {
                            ExecutionStatus::Completed => {
                                println!("  ✅ 工作流完成 (耗时: {:?})", elapsed);
                            }
                            ExecutionStatus::Failed => {
                                println!("  ❌ 工作流失败 (耗时: {:?})", elapsed);
                            }
                            ExecutionStatus::Cancelled => {
                                println!("  🚫 工作流取消 (耗时: {:?})", elapsed);
                            }
                            _ => {
                                println!("  🏁 工作流结束: {:?} (耗时: {:?})", status, elapsed);
                            }
                        }
                        return Ok(status);
                    }
                }
                Err(e) => {
                    println!("  ⚠️  获取状态失败: {}", e);
                }
            }
        }
    })
    .await;

    match monitor_result {
        Ok(Ok(_)) => {
            println!("  🎉 工作流 '{}' 监控完成", workflow_name);
        }
        Ok(Err(e)) => {
            println!("  💥 工作流 '{}' 执行错误: {}", workflow_name, e);
        }
        Err(_) => {
            println!("  ⏰ 工作流 '{}' 监控超时", workflow_name);
        }
    }

    Ok(())
}

// Tool creation functions (implementations would be more complex in real scenarios)

fn create_data_processor_tool() -> Result<BasicTool> {
    let executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            // Simulate data processing
            tokio::time::sleep(Duration::from_millis(100)).await;

            let data = params.get("data").cloned().unwrap_or(json!([]));
            let operations = params
                .get("operations")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![]);

            let mut result = data;
            let mut record_count = 0;

            if let Some(array) = result.as_array() {
                record_count = array.len();
            }

            // Simulate processing operations
            for _op in operations {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            Ok(json!({
                "result": result,
                "record_count": record_count,
                "operations_applied": operations.len(),
                "processed_at": chrono::Utc::now().to_rfc3339()
            }))
        },
    ));

    BasicTool::builder()
        .name("data_processor")
        .version("1.0.0")
        .description("Advanced data processing tool")
        .executor(executor)
        .build()
        .map_err(|e| WorkflowError::tool_execution(&e.to_string()))
}

fn create_http_client_tool() -> Result<BasicTool> {
    let executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            // Simulate HTTP request
            tokio::time::sleep(Duration::from_millis(200)).await;

            let method = params
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET");
            let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");

            // Simulate successful response
            Ok(json!({
                "status": 200,
                "headers": {
                    "content-type": "application/json"
                },
                "body": [
                    {"id": 1, "name": "John Doe", "email": "john@example.com"},
                    {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
                ],
                "method": method,
                "url": url,
                "response_time_ms": 200
            }))
        },
    ));

    BasicTool::builder()
        .name("http_client")
        .version("1.0.0")
        .description("HTTP client tool")
        .executor(executor)
        .build()
        .map_err(|e| WorkflowError::tool_execution(&e.to_string()))
}

fn create_file_operations_tool() -> Result<BasicTool> {
    let executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            let operation = params
                .get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("read");
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");

            match operation {
                "read" => {
                    // Simulate file read
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    Ok(json!({
                        "content": [
                            {"id": 1, "name": "Item 1", "value": 15},
                            {"id": 2, "name": "Item 2", "value": 25},
                            {"id": 3, "name": "Item 3", "value": 5}
                        ],
                        "path": path,
                        "size": 1024,
                        "format": "json"
                    }))
                }
                "write" => {
                    // Simulate file write
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    Ok(json!({
                        "success": true,
                        "path": path,
                        "bytes_written": 512
                    }))
                }
                _ => Err(WorkflowError::tool_execution(&format!(
                    "Unknown operation: {}",
                    operation
                ))),
            }
        },
    ));

    BasicTool::builder()
        .name("file_operations")
        .version("1.0.0")
        .description("File operations tool")
        .executor(executor)
        .build()
        .map_err(|e| WorkflowError::tool_execution(&e.to_string()))
}

fn create_calculator_tool() -> Result<BasicTool> {
    let executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            let operation = params
                .get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("add");

            match operation {
                "fibonacci" => {
                    let n = params.get("n").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    let result = fibonacci(n);
                    Ok(json!({
                        "result": result,
                        "operation": "fibonacci",
                        "input": n
                    }))
                }
                "factorial" => {
                    let n = params.get("n").and_then(|v| v.as_u64()).unwrap_or(5);
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    let result = factorial(n);
                    Ok(json!({
                        "result": result,
                        "operation": "factorial",
                        "input": n
                    }))
                }
                "prime_check" => {
                    let n = params.get("n").and_then(|v| v.as_u64()).unwrap_or(97);
                    tokio::time::sleep(Duration::from_millis(40)).await;
                    let result = is_prime(n);
                    Ok(json!({
                        "result": result,
                        "operation": "prime_check",
                        "input": n
                    }))
                }
                "power" => {
                    let base = params.get("base").and_then(|v| v.as_f64()).unwrap_or(2.0);
                    let exponent = params
                        .get("exponent")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(8.0);
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    let result = base.powf(exponent);
                    Ok(json!({
                        "result": result,
                        "operation": "power",
                        "base": base,
                        "exponent": exponent
                    }))
                }
                _ => Err(WorkflowError::tool_execution(&format!(
                    "Unknown operation: {}",
                    operation
                ))),
            }
        },
    ));

    BasicTool::builder()
        .name("calculator")
        .version("1.0.0")
        .description("Mathematical calculator tool")
        .executor(executor)
        .build()
        .map_err(|e| WorkflowError::tool_execution(&e.to_string()))
}

fn create_validator_tool() -> Result<BasicTool> {
    let executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            tokio::time::sleep(Duration::from_millis(25)).await;

            let data = params.get("data").cloned().unwrap_or(json!(null));
            let _schema = params.get("schema").cloned().unwrap_or(json!({}));

            // Simple validation simulation
            let valid = !data.is_null();

            Ok(json!({
                "valid": valid,
                "errors": if valid { json!([]) } else { json!(["Data is null"]) },
                "validated_at": chrono::Utc::now().to_rfc3339()
            }))
        },
    ));

    BasicTool::builder()
        .name("data_validator")
        .version("1.0.0")
        .description("Data validation tool")
        .executor(executor)
        .build()
        .map_err(|e| WorkflowError::tool_execution(&e.to_string()))
}

fn create_notification_tool() -> Result<BasicTool> {
    let executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            let message = params
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("No message");
            let channel = params
                .get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("console");
            let level = params
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("info");

            // Simulate notification sending
            tokio::time::sleep(Duration::from_millis(10)).await;

            match level {
                "success" => println!("  ✅ {}", message),
                "info" => println!("  ℹ️  {}", message),
                "warning" => println!("  ⚠️  {}", message),
                "error" => println!("  ❌ {}", message),
                _ => println!("  📢 {}", message),
            }

            Ok(json!({
                "sent": true,
                "message": message,
                "channel": channel,
                "level": level,
                "sent_at": chrono::Utc::now().to_rfc3339()
            }))
        },
    ));

    BasicTool::builder()
        .name("notification_sender")
        .version("1.0.0")
        .description("Notification sender tool")
        .executor(executor)
        .build()
        .map_err(|e| WorkflowError::tool_execution(&e.to_string()))
}

fn create_error_simulator_tool() -> Result<BasicTool> {
    let executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            let error_type = params
                .get("error_type")
                .and_then(|v| v.as_str())
                .unwrap_or("temporary");
            let failure_rate = params
                .get("failure_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5);
            let message = params
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Simulated error");

            tokio::time::sleep(Duration::from_millis(100)).await;

            // Simulate random failures
            let random_value: f64 = rand::random();

            if random_value < failure_rate {
                Err(WorkflowError::tool_execution(&format!(
                    "{}: {}",
                    error_type, message
                )))
            } else {
                Ok(json!({
                    "success": true,
                    "message": "Error simulation passed",
                    "error_type": error_type,
                    "failure_rate": failure_rate,
                    "random_value": random_value
                }))
            }
        },
    ));

    BasicTool::builder()
        .name("error_simulator")
        .version("1.0.0")
        .description("Error simulation tool for testing")
        .executor(executor)
        .build()
        .map_err(|e| WorkflowError::tool_execution(&e.to_string()))
}

// Helper functions for mathematical operations
fn fibonacci(n: usize) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    let sqrt_n = (n as f64).sqrt() as u64;
    for i in (3..=sqrt_n).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}
