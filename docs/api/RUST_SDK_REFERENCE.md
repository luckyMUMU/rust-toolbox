# Rust SDK Reference

> **Comprehensive reference for the Rust Workflow Toolkit SDK**  
> *Last Updated: 2026-01-14*

## Overview

This document provides the complete API reference for the Rust Workflow Toolkit, including core traits, data structures, and usage examples.

## Core API Interfaces

### Workflow Engine

#### WorkflowEngine Trait

The core interface for the workflow execution engine.

```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// Execute a workflow definition
    async fn execute_workflow(&self, definition: WorkflowDefinition) -> Result<WorkflowExecution>;
    
    /// Pause a running workflow
    async fn pause_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// Resume a paused workflow
    async fn resume_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// Stop a running workflow
    async fn stop_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// Get workflow status
    async fn get_workflow_status(&self, id: WorkflowId) -> Result<WorkflowStatus>;
    
    /// List all workflows
    async fn list_workflows(&self) -> Result<Vec<WorkflowInfo>>;
    
    /// Delete a workflow
    async fn delete_workflow(&self, id: WorkflowId) -> Result<()>;
}
```

#### Data Structures

##### WorkflowDefinition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub global_config: WorkflowConfig,
}
```

##### WorkflowNode

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub node_type: NodeType,
    pub tool_name: Option<String>,
    pub parameters: Value,
    pub retry_policy: Option<RetryPolicy>,
    pub timeout: Option<Duration>,
    pub condition: Option<String>,
}
```

##### NodeType

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Tool,
    Condition,
    Loop,
    Parallel,
    Checkpoint,
    Start,
    End,
}
```

##### ExecutionStatus

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}
```

### Tool Registry

#### ToolRegistry Trait

```rust
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    async fn register_tool(&mut self, tool: Box<dyn ToolNode>) -> Result<()>;
    fn get_tool(&self, name: &str) -> Option<&dyn ToolNode>;
    fn list_tools(&self) -> Vec<ToolInfo>;
    async fn execute_tool(&self, name: &str, params: Value) -> Result<Value>;
    fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()>;
    fn get_tool_schema(&self, name: &str) -> Option<ToolDefinition>;
    async fn unregister_tool(&mut self, name: &str) -> Result<()>;
}
```

#### ToolNode Trait

```rust
#[async_trait]
pub trait ToolNode: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;
    fn validate_parameters(&self, params: &Value) -> Result<()>;
    fn get_schema(&self) -> ToolDefinition;
    fn get_plugin_info(&self) -> Option<&PluginInfo>;
}
```

### Plugin System

#### Plugin Trait

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    fn get_tools(&self) -> Vec<Box<dyn ToolNode>>;
    async fn shutdown(&mut self) -> Result<()>;
    fn get_info(&self) -> PluginInfo;
    async fn health_check(&self) -> Result<PluginHealth>;
}
```

#### PluginManager

```rust
impl PluginManager {
    pub fn new() -> Self;
    pub async fn load_plugin(&mut self, name: &str, config: PluginConfig) -> Result<()>;
    pub async fn unload_plugin(&mut self, name: &str) -> Result<()>;
    pub async fn reload_plugin(&mut self, name: &str) -> Result<()>;
    pub fn list_plugins(&self) -> Vec<PluginInfo>;
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin>;
    pub fn get_all_tools(&self) -> Vec<Box<dyn ToolNode>>;
}
```

### Storage System

#### StorageBackend Trait

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn save(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
```

---

## Usage Guide

### Quick Start

#### 1. Basic Workflow Creation and Execution

```rust
use workflow_toolkit::{
    WorkflowDefinition, WorkflowEngine, DefaultWorkflowEngine,
    ToolRegistry, BasicToolRegistry, StateManager,
    storage::{FileStorage, SimpleMemoryCache},
    config::ConfigManager, Config,
};
use std::sync::Arc;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize components
    let config = Arc::new(ConfigManager::new(Config::default()));
    let storage = Arc::new(FileStorage::new("./data")?);
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));
    let tool_registry = Arc::new(BasicToolRegistry::new());
    
    // 2. Create workflow engine
    let engine = Arc::new(DefaultWorkflowEngine::new(
        state_manager,
        tool_registry,
        4, // Max concurrency
    ));
    
    // 3. Define workflow
    let workflow = WorkflowDefinition {
        name: "hello-world".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Simple Hello World Workflow".to_string()),
        metadata: std::collections::HashMap::new(),
        nodes: vec![
            workflow_toolkit::workflow::WorkflowNode {
                id: "start".to_string(),
                node_type: workflow_toolkit::workflow::NodeType::Start,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: Vec::new(),
                metadata: std::collections::HashMap::new(),
            },
            workflow_toolkit::workflow::WorkflowNode {
                id: "hello".to_string(),
                node_type: workflow_toolkit::workflow::NodeType::Tool,
                tool_name: Some("echo".to_string()),
                parameters: json!({
                    "message": "Hello, World!"
                }),
                retry_policy: None,
                timeout: Some(std::time::Duration::from_secs(30)),
                depends_on: vec!["start".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            workflow_toolkit::workflow::WorkflowNode {
                id: "end".to_string(),
                node_type: workflow_toolkit::workflow::NodeType::End,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["hello".to_string()],
                metadata: std::collections::HashMap::new(),
            },
        ],
        edges: vec![
            workflow_toolkit::workflow::WorkflowEdge {
                from: "start".to_string(),
                to: "hello".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            workflow_toolkit::workflow::WorkflowEdge {
                from: "hello".to_string(),
                to: "end".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
        ],
        global_config: workflow_toolkit::WorkflowConfig::default(),
    };
    
    // 4. Execute workflow
    let execution = engine.execute_workflow(workflow).await?;
    println!("Workflow Execution ID: {}", execution.id);
    
    // 5. Monitor status
    loop {
        let status = engine.get_workflow_status(execution.id).await?;
        println!("Current Status: {:?}", status);
        
        if status.is_terminal() {
            break;
        }
        
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    
    Ok(())
}
```

#### 2. Tool Registration and Usage

```rust
use workflow_toolkit::{
    tools::{BasicTool, AsyncFunctionExecutor, ToolRegistry, BasicToolRegistry},
    ExecutionContext,
};
use serde_json::{json, Value};
use std::sync::Arc;

async fn register_custom_tools() -> Result<Arc<BasicToolRegistry>, Box<dyn std::error::Error>> {
    let mut registry = BasicToolRegistry::new();
    
    // Register calculator tool
    let calculator_executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            let a = params["a"].as_f64().unwrap_or(0.0);
            let b = params["b"].as_f64().unwrap_or(0.0);
            let operation = params["operation"].as_str().unwrap_or("add");
            
            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b != 0.0 {
                        a / b
                    } else {
                        return Err(workflow_toolkit::WorkflowError::tool_execution(
                            "Division by zero"
                        ));
                    }
                }
                _ => return Err(workflow_toolkit::WorkflowError::tool_execution(
                    &format!("Unknown operation: {}", operation)
                )),
            };
            
            Ok(json!({
                "result": result,
                "operation": operation,
                "operands": [a, b]
            }))
        }
    ));
    
    let calculator_tool = BasicTool::builder()
        .name("calculator")
        .version("1.0.0")
        .description("Basic Calculator Tool")
        .executor(calculator_executor)
        .build()?;
    
    registry.register_tool(Arc::new(calculator_tool))?;
    
    Ok(Arc::new(registry))
}
```

### Advanced Usage

#### Conditional and Loop Workflows

```yaml
# conditional-workflow.yaml
name: "conditional-processing"
version: "1.0.0"
description: "Data processing with conditions"

global_config:
  variables:
    threshold: 100
    max_retries: 3

nodes:
  - id: "start"
    type: "start"
    
  - id: "load_data"
    type: "tool"
    tool_name: "data_loader"
    parameters:
      source: "${env.DATA_SOURCE}"
      format: "json"
    
  - id: "validate_data"
    type: "condition"
    condition: "${load_data.record_count} > ${threshold}"
    
  - id: "process_large_dataset"
    type: "tool"
    tool_name: "batch_processor"
    parameters:
      data: "${load_data.output}"
      batch_size: 1000
      parallel: true
    
  - id: "process_small_dataset"
    type: "tool"
    tool_name: "simple_processor"
    parameters:
      data: "${load_data.output}"
      
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "load_data"
    
  - from: "load_data"
    to: "validate_data"
    
  - from: "validate_data"
    to: "process_large_dataset"
    condition: "true"
    
  - from: "validate_data"
    to: "process_small_dataset"
    condition: "false"
    
  - from: "process_large_dataset"
    to: "end"
    
  - from: "process_small_dataset"
    to: "end"
```
