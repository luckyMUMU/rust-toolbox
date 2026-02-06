# Rust SDK参考

> **Rust工作流工具包SDK完整参考**  
> *最后更新：2026-01-14*

## 概述

本文档提供Rust工作流工具包的完整API参考，包括核心traits、数据结构和用法示例。

## 核心API接口

### 工作流引擎

#### WorkflowEngine Trait

工作流执行引擎的核心接口。

```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 执行工作流定义
    async fn execute_workflow(&self, definition: WorkflowDefinition) -> Result<WorkflowExecution>;
    
    /// 暂停运行中的工作流
    async fn pause_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// 恢复暂停的工作流
    async fn resume_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// 停止运行中的工作流
    async fn stop_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// 获取工作流状态
    async fn get_workflow_status(&self, id: WorkflowId) -> Result<WorkflowStatus>;
    
    /// 列出所有工作流
    async fn list_workflows(&self) -> Result<Vec<WorkflowInfo>>;
    
    /// 删除工作流
    async fn delete_workflow(&self, id: WorkflowId) -> Result<()>;
}
```

#### 数据结构

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

### 工具注册表

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

### 插件系统

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

### 存储系统

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

## 使用指南

### 快速开始

#### 1. 基本工作流创建和执行

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
    // 1. 初始化组件
    let config = Arc::new(ConfigManager::new(Config::default()));
    let storage = Arc::new(FileStorage::new("./data")?);
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));
    let tool_registry = Arc::new(BasicToolRegistry::new());
    
    // 2. 创建工作流引擎
    let engine = Arc::new(DefaultWorkflowEngine::new(
        state_manager,
        tool_registry,
        4, // 最大并发数
    ));
    
    // 3. 定义工作流
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
    
    // 4. 执行工作流
    let execution = engine.execute_workflow(workflow).await?;
    println!("工作流执行ID: {}", execution.id);
    
    // 5. 监控状态
    loop {
        let status = engine.get_workflow_status(execution.id).await?;
        println!("当前状态: {:?}", status);
        
        if status.is_terminal() {
            break;
        }
        
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    
    Ok(())
}
```

#### 2. 工具注册和使用

```rust
use workflow_toolkit::{
    tools::{BasicTool, AsyncFunctionExecutor, ToolRegistry, BasicToolRegistry},
    ExecutionContext,
};
use serde_json::{json, Value};
use std::sync::Arc;

async fn register_custom_tools() -> Result<Arc<BasicToolRegistry>, Box<dyn std::error::Error>> {
    let mut registry = BasicToolRegistry::new();
    
    // 注册计算器工具
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
                            "除零错误"
                        ));
                    }
                }
                _ => return Err(workflow_toolkit::WorkflowError::tool_execution(
                    &format!("未知操作: {}", operation)
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
        .description("基础计算器工具")
        .executor(calculator_executor)
        .build()?;
    
    registry.register_tool(Arc::new(calculator_tool))?;
    
    Ok(Arc::new(registry))
}
```

### 高级用法

#### 条件和循环工作流

```yaml
# conditional-workflow.yaml
name: "conditional-processing"
version: "1.0.0"
description: "带条件的数据处理"

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
