# Rust SDK参考

> **Rust工作流工具包SDK完整参考**  
> *最后更新：2026-02-27*

## 概述

本文档提供Rust工作流工具包的完整API参考，包括核心traits、数据结构和用法示例。

---

## 异步编程指南

### 运行时要求

本 SDK 基于 Tokio 异步运行时，所有异步方法需要在 Tokio 运行时环境中执行。

```rust
// 推荐方式：使用 #[tokio::main] 宏
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 异步代码
    Ok(())
}

// 或者手动创建运行时
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // 异步代码
        Ok(())
    })
}
```

### 异步最佳实践

#### 1. 避免阻塞异步执行器

```rust
// ❌ 错误：阻塞调用
async fn bad_example() -> Result<()> {
    std::thread::sleep(Duration::from_secs(1)); // 阻塞整个执行器！
    Ok(())
}

// ✅ 正确：使用异步睡眠
async fn good_example() -> Result<()> {
    tokio::time::sleep(Duration::from_secs(1)).await; // 不阻塞执行器
    Ok(())
}
```

#### 2. 并发执行多个任务

```rust
use tokio::join;

// 并发执行多个工作流
async fn run_parallel_workflows(engine: Arc<DefaultWorkflowEngine>) -> Result<()> {
    let workflow1 = engine.execute(def1, params1);
    let workflow2 = engine.execute(def2, params2);
    let workflow3 = engine.execute(def3, params3);
    
    // 使用 join! 宏并发执行
    let (result1, result2, result3) = join!(workflow1, workflow2, workflow3);
    
    Ok(())
}
```

#### 3. 使用 tokio::spawn 处理后台任务

```rust
// 后台执行工作流，不等待结果
async fn execute_in_background(
    engine: Arc<DefaultWorkflowEngine>,
    workflow: WorkflowDefinition,
) -> Result<()> {
    tokio::spawn(async move {
        match engine.execute(workflow, HashMap::new()).await {
            Ok(execution) => println!("执行完成: {}", execution.id),
            Err(e) => eprintln!("执行失败: {}", e),
        }
    });
    
    Ok(())
}
```

#### 4. 超时控制

```rust
use tokio::time::{timeout, Duration};

async fn execute_with_timeout(
    engine: Arc<DefaultWorkflowEngine>,
    workflow: WorkflowDefinition,
) -> Result<WorkflowExecution> {
    // 设置 30 秒超时
    match timeout(Duration::from_secs(30), engine.execute(workflow, HashMap::new())).await {
        Ok(result) => result,
        Err(_) => Err(WorkflowError::timeout("工作流执行超时")),
    }
}
```

#### 5. 取消执行

```rust
use tokio_util::sync::CancellationToken;

async fn cancellable_execution(
    engine: Arc<DefaultWorkflowEngine>,
    workflow: WorkflowDefinition,
    cancel_token: CancellationToken,
) -> Result<WorkflowExecution> {
    tokio::select! {
        result = engine.execute(workflow, HashMap::new()) => result,
        _ = cancel_token.cancelled() => {
            Err(WorkflowError::cancelled("用户取消"))
        }
    }
}
```

### 异步错误处理

```rust
// 使用 ? 运算符传播异步错误
async fn handle_workflow() -> Result<(), WorkflowError> {
    let engine = create_engine().await?;
    let execution = engine.execute(workflow, params).await?;
    
    if execution.status != ExecutionStatus::Completed {
        return Err(WorkflowError::execution_failed("工作流未完成"));
    }
    
    Ok(())
}

// 使用 map_err 转换错误类型
async fn convert_errors() -> Result<(), MyError> {
    engine.execute(workflow, params)
        .await
        .map_err(|e| MyError::from(e))?;
    Ok(())
}
```

---

## 核心API接口

### 工作流引擎

#### WorkflowEngine Trait

工作流执行引擎的核心接口。

```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 执行工作流定义
    async fn execute(
        &self,
        definition: WorkflowDefinition,
        initial_params: HashMap<String, Value>,
    ) -> Result<WorkflowExecution>;
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
    pub metadata: HashMap<String, Value>,
    pub depends_on: Vec<String>,
}
```

##### NodeType

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Tool,
    Condition,
    Loop,
    Parallel,
    Checkpoint,
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
                id: "hello".to_string(),
                node_type: workflow_toolkit::workflow::NodeType::Tool,
                tool_name: Some("echo".to_string()),
                parameters: json!({
                    "message": "Hello, World!"
                }),
                retry_policy: None,
                timeout: Some(std::time::Duration::from_secs(30)),
                metadata: std::collections::HashMap::new(),
                depends_on: Vec::new(),
            },
        ],
        edges: vec![],
        global_config: workflow_toolkit::WorkflowConfig::default(),
    };
    
    // 4. 执行工作流
    let initial_params = std::collections::HashMap::new();
    let execution = engine.execute(workflow, initial_params).await?;
    println!("工作流执行ID: {}", execution.id);
    
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
  - id: "load_data"
    type: "tool"
    tool_name: "data_loader"
    parameters:
      source: "${env.DATA_SOURCE}"
      format: "json"
    
  - id: "validate_data"
    type: "condition"
    parameters:
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

edges:
  - from: "load_data"
    to: "validate_data"
    
  - from: "validate_data"
    to: "process_large_dataset"
    condition: "true"
    
  - from: "validate_data"
    to: "process_small_dataset"
    condition: "false"
```

---

## 迁移指南

### 版本兼容性策略

本项目遵循语义化版本控制（Semantic Versioning）：

- **主版本号（Major）**：不兼容的 API 变更
- **次版本号（Minor）**：向后兼容的功能新增
- **修订号（Patch）**：向后兼容的问题修复

### 从 v1.x 迁移到 v2.x

#### 重大变更概览

| 变更项 | v1.x | v2.x | 迁移难度 |
|--------|------|------|----------|
| 工具系统架构 | Trait-based | Enum-based | 中 |
| WorkflowEngine 接口 | 同步方法 | 异步方法 | 低 |
| 错误处理 | 自定义枚举 | thiserror | 低 |
| 配置格式 | TOML only | TOML/YAML/JSON | 低 |

#### 1. 工具系统迁移

**v1.x 代码**：

```rust
// 旧版 trait-based 工具
use workflow_toolkit::tools::Tool;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    async fn execute(&self, input: Value) -> Result<Value> {
        Ok(json!({"result": "success"}))
    }
}

// 注册
let tool: Box<dyn Tool> = Box::new(MyTool);
registry.register("my_tool", tool)?;
```

**v2.x 代码**：

```rust
// 新版 Enum-based 工具
use workflow_toolkit::tools::{Tool, NativeTool, ToolInput, ToolOutput};

let tool = Tool::Native(NativeTool::from_fn("my_tool", |input: ToolInput| async move {
    Ok(ToolOutput::new(json!({"result": "success"})))
}));

// 注册
registry.register(tool)?;
```

**迁移步骤**：

1. 将 `impl Tool` 改为使用 `NativeTool::from_fn` 包装
2. 更新输入参数类型从 `Value` 到 `ToolInput`
3. 更新返回类型从 `Result<Value>` 到 `Result<ToolOutput>`
4. 更新注册调用方式

#### 2. WorkflowEngine 接口迁移

**v1.x 代码**：

```rust
// 旧版同步接口
let result = engine.execute(workflow)?;
```

**v2.x 代码**：

```rust
// 新版异步接口
let result = engine.execute(workflow, params).await?;
```

**迁移步骤**：

1. 在调用处添加 `.await`
2. 确保调用函数是 `async fn`
3. 添加 `initial_params` 参数（可传空 HashMap）

#### 3. 错误处理迁移

**v1.x 代码**：

```rust
// 旧版错误枚举
#[derive(Debug)]
pub enum WorkflowError {
    NotFound(String),
    ExecutionFailed(String),
    // ...
}
```

**v2.x 代码**：

```rust
// 新版 thiserror 错误
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("资源未找到: {0}")]
    NotFound(String),
    
    #[error("执行失败: {0}")]
    ExecutionFailed(String),
    
    // ...
}
```

**迁移步骤**：

1. 添加 `thiserror` 依赖到 `Cargo.toml`
2. 为错误枚举添加 `#[derive(Error)]`
3. 为每个变体添加 `#[error("...")]` 属性
4. 更新错误创建方式

#### 4. 配置格式迁移

**v1.x 代码**：

```rust
// 仅支持 TOML
let config = Config::from_toml_file("config.toml")?;
```

**v2.x 代码**：

```rust
// 支持多种格式
let config = Config::from_file("config.toml")?;  // 自动检测格式
let config = Config::from_file("config.yaml")?;
let config = Config::from_file("config.json")?;
```

**迁移步骤**：

1. 将 `from_toml_file` 改为 `from_file`
2. 可选择迁移到 YAML 格式以获得更好的可读性

### 废弃 API 清单

以下 API 已废弃，将在下一个主版本中移除：

| 废弃 API | 替代方案 | 废弃版本 | 移除版本 |
|----------|----------|----------|----------|
| `Tool::execute(&self, Value)` | `Tool::execute(&self, ToolInput)` | v1.5.0 | v2.0.0 |
| `WorkflowEngine::execute_sync` | `WorkflowEngine::execute` | v1.8.0 | v2.0.0 |
| `Config::from_toml_file` | `Config::from_file` | v1.9.0 | v2.0.0 |
| `ToolRegistry::register_boxed` | `ToolRegistry::register` | v1.9.0 | v2.0.0 |

### 迁移检查清单

在升级版本时，请按以下清单检查：

- [ ] 更新 `Cargo.toml` 中的版本号
- [ ] 运行 `cargo check` 检查编译错误
- [ ] 更新工具注册代码（如使用旧版 trait）
- [ ] 添加 `.await` 到异步调用（如使用同步接口）
- [ ] 更新错误处理代码（如自定义错误类型）
- [ ] 运行测试套件确保功能正常
- [ ] 检查废弃 API 警告并更新

### 迁移脚本

以下脚本可帮助自动化部分迁移工作：

```bash
#!/bin/bash
# migrate-v1-to-v2.sh

# 1. 更新 Cargo.toml 版本
sed -i 's/workflow-toolkit = "1\..*"/workflow-toolkit = "2.0.0"/' Cargo.toml

# 2. 替换废弃方法名
find src -name "*.rs" -exec sed -i \
    -e 's/from_toml_file/from_file/g' \
    -e 's/register_boxed/register/g' \
    -e 's/execute_sync/execute/g' \
    {} \;

# 3. 运行编译检查
cargo check 2>&1 | tee migration-errors.log

echo "迁移完成，请检查 migration-errors.log 中的错误"
```

### 常见迁移问题

#### Q1: 编译错误 "trait bound not satisfied"

**原因**：新版工具系统使用 Enum 而非 trait object。

**解决方案**：
```rust
// 错误
fn my_function(tool: Box<dyn Tool>) { ... }

// 正确
fn my_function(tool: Tool) { ... }
```

#### Q2: 运行时错误 "future cannot be sent between threads safely"

**原因**：异步函数中使用了非 Send 类型。

**解决方案**：
```rust
// 确保所有跨 await 的变量都是 Send
async fn my_function() -> Result<()> {
    let data = Arc::new(Mutex::new(vec![])); // Send
    // ... 跨 await 使用 data
    Ok(())
}
```

#### Q3: 配置文件解析错误

**原因**：配置格式变更。

**解决方案**：
```yaml
# 新版配置格式示例
version: "2.0"
engine:
  max_concurrent: 10
  timeout_seconds: 300
  
tools:
  - name: my_tool
    type: native
    config:
      # 工具特定配置
```

### 获取迁移帮助

如果在迁移过程中遇到问题：

1. 查阅 [CHANGELOG.md](../../../CHANGELOG.md) 了解详细变更
2. 在 GitHub Issues 中搜索类似问题
3. 提交新 Issue 并标注 `[migration]` 标签
4. 加入社区讨论群获取实时帮助
