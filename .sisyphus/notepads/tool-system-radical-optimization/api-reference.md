# 工具系统 API 参考文档

## 概述

本文档提供工具系统激进优化后的完整API参考，包括所有公共类型、trait、函数和宏。

**版本**: 0.2.0-alpha  
**最后更新**: 2026-02-01  

---

## 核心类型

### Tool 枚举

工具类型的核心枚举，替代了旧的 `Arc<dyn ToolNode>` 动态分发。

```rust
pub enum Tool {
    Native(Arc<NativeTool>),
    Python(Arc<PythonTool>),
    NodeJs(Arc<NodeJsTool>),
    Docker(Arc<DockerTool>),
    Wasm(Arc<WasmTool>),
    Composed(Arc<ComposedTool>),
}
```

**方法**:

#### `execute`
```rust
pub async fn execute(&self, input: ToolInput, ctx: ExecutionContext) -> Result<ToolOutput>
```
执行工具并返回输出。

**示例**:
```rust
let tool = registry.get("echo").unwrap();
let input = ToolInput::new(json!({"message": "Hello"}));
let output = tool.execute(input, ctx).await?;
```

---

### ToolId

工具的唯一标识符，使用新类型模式。

```rust
pub struct ToolId(u64);
```

**方法**:

#### `new`
```rust
pub fn new() -> Self
```
生成新的唯一ID。

#### `as_u64`
```rust
pub fn as_u64(&self) -> u64
```
获取内部u64值。

**特性**:
- `Copy` - 可以按值复制
- `Eq`, `Hash` - 可用于HashMap键
- `Display` - 可格式化为字符串

---

### ToolInput

工具输入参数容器。

```rust
pub struct ToolInput {
    pub params: Value,
    pub metadata: Option<Value>,
}
```

**方法**:

#### `new`
```rust
pub fn new(params: Value) -> Self
```
创建新的输入。

#### `with_metadata`
```rust
pub fn with_metadata(mut self, metadata: Value) -> Self
```
添加元数据。

---

### ToolOutput

工具输出结果容器。

```rust
pub struct ToolOutput {
    pub result: Value,
    pub metadata: Option<Value>,
    pub success: bool,
}
```

**方法**:

#### `success`
```rust
pub fn success(result: Value) -> Self
```
创建成功输出。

#### `failure`
```rust
pub fn failure(error: impl Into<String>) -> Self
```
创建失败输出。

---

## 工具注册表

### ToolRegistry

高性能工具注册表，使用DashMap实现O(1)查找。

```rust
pub struct ToolRegistry {
    tools: DashMap<ToolId, Tool>,
    name_index: DashMap<String, ToolId>,
    metadata_cache: DashMap<ToolId, Arc<ToolMetadata>>,
    // ... 其他字段
}
```

**方法**:

#### `new`
```rust
pub fn new() -> Self
```
创建空注册表。

#### `register`
```rust
pub fn register(&self, tool: Tool) -> ToolId
```
注册工具，返回ToolId。

**示例**:
```rust
let registry = ToolRegistry::new();
let tool = create_echo_tool()?;
let id = registry.register(tool);
```

#### `get`
```rust
pub fn get(&self, name: &str) -> Option<Tool>
```
通过名称获取工具（O(1)）。

#### `get_by_id`
```rust
pub fn get_by_id(&self, id: ToolId) -> Option<Tool>
```
通过ID获取工具（O(1)）。

#### `contains`
```rust
pub fn contains(&self, name: &str) -> bool
```
检查工具是否存在。

#### `list_names`
```rust
pub fn list_names(&self) -> Vec<String>
```
列出所有工具名称。

#### `list_tools`
```rust
pub fn list_tools(&self) -> Vec<ToolInfo>
```
列出所有工具的元数据。

#### `execute`
```rust
pub async fn execute(&self, name: &str, input: ToolInput) -> Result<ToolOutput>
```
执行指定名称的工具。

---

## 工具构建器

### NativeToolBuilder

用于构建原生工具的Builder模式实现。

```rust
pub struct NativeToolBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    executor: Option<Arc<dyn Fn(ToolInput, ExecutionContext) -> BoxFuture<'static, Result<ToolOutput>> + Send + Sync>>,
    middleware_stack: Option<MiddlewareStack>,
    // ... 其他字段
}
```

**方法**:

#### `new`
```rust
pub fn new() -> Self
```
创建新的构建器。

#### `name`
```rust
pub fn name(mut self, name: impl Into<String>) -> Self
```
设置工具名称。

#### `version`
```rust
pub fn version(mut self, version: impl Into<String>) -> Self
```
设置工具版本。

#### `description`
```rust
pub fn description(mut self, description: impl Into<String>) -> Self
```
设置工具描述。

#### `executor`
```rust
pub fn executor<F, Fut>(mut self, executor: F) -> Self
where
    F: Fn(ToolInput, ExecutionContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ToolOutput>> + Send + 'static,
```
设置执行器闭包。

**示例**:
```rust
let tool = NativeToolBuilder::new()
    .name("echo")
    .version("1.0.0")
    .description("Echo tool")
    .executor(|input, _ctx| async move {
        Ok(ToolOutput::success(input.params))
    })
    .build()?;
```

#### `with_middleware`
```rust
pub fn with_middleware(mut self, stack: MiddlewareStack) -> Self
```
设置中间件栈。

#### `build`
```rust
pub fn build(self) -> Result<NativeTool>
```
构建工具，验证所有必需字段。

---

## 中间件系统

### Middleware trait

中间件核心trait。

```rust
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput>;
    
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap_or("UnknownMiddleware")
    }
}
```

**实现示例**:
```rust
pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        println!("Before execution");
        let result = next.run(ctx).await;
        println!("After execution");
        result
    }
}
```

---

### MiddlewareContext

中间件上下文，包含执行信息和自定义数据。

```rust
pub struct MiddlewareContext {
    pub input: ToolInput,
    pub metadata: ExecutionMetadata,
    custom_data: DashMap<String, Box<dyn Any + Send + Sync>>,
}
```

**方法**:

#### `new`
```rust
pub fn new(input: ToolInput, metadata: ExecutionMetadata) -> Self
```

#### `insert`
```rust
pub fn insert<T: Any + Send + Sync>(&self, key: impl Into<String>, value: T)
```
存储自定义数据。

#### `get`
```rust
pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<T>
```
获取自定义数据。

---

### MiddlewareStack

中间件栈，管理中间件链。

```rust
pub struct MiddlewareStack {
    middlewares: Vec<Arc<dyn Middleware>>,
}
```

**方法**:

#### `new`
```rust
pub fn new() -> Self
```

#### `add`
```rust
pub fn add(&mut self, middleware: Arc<dyn Middleware>)
```
添加中间件。

#### `execute`
```rust
pub async fn execute(&self, input: ToolInput, metadata: ExecutionMetadata, tool: &Tool) -> Result<ToolOutput>
```
通过中间件链执行工具。

---

### 内置中间件

#### LoggingMiddleware
```rust
pub struct LoggingMiddleware {
    log_start: bool,
    log_completion: bool,
}

impl LoggingMiddleware {
    pub fn new() -> Self
    pub fn log_completion_only(mut self) -> Self
    pub fn silent(mut self) -> Self
}
```

#### TimingMiddleware
```rust
pub struct TimingMiddleware {
    store_in_context: bool,
}

impl TimingMiddleware {
    pub fn new() -> Self
    pub fn without_context_storage(mut self) -> Self
}
```

#### RetryMiddleware
```rust
pub struct RetryMiddleware {
    max_retries: u32,
    retry_delay: Duration,
    retryable_errors: Vec<String>,
}

impl RetryMiddleware {
    pub fn new(max_retries: u32) -> Self
    pub fn with_delay(mut self, delay: Duration) -> Self
    pub fn retryable_on(mut self, error_patterns: Vec<String>) -> Self
}
```

#### TimeoutMiddleware
```rust
pub struct TimeoutMiddleware {
    timeout: Duration,
}

impl TimeoutMiddleware {
    pub fn new(timeout: Duration) -> Self
    pub fn seconds(secs: u64) -> Self
    pub fn millis(ms: u64) -> Self
}
```

#### CircuitBreakerMiddleware
```rust
pub struct CircuitBreakerMiddleware {
    failure_threshold: u32,
    reset_timeout: Duration,
    // ...
}

impl CircuitBreakerMiddleware {
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self
}
```

#### MetricsMiddleware
```rust
pub struct MetricsMiddleware {
    prefix: String,
}

impl MetricsMiddleware {
    pub fn new() -> Self
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self
}
```

---

## 强类型参数系统

### ToolInputConvert trait

强类型输入转换trait。

```rust
pub trait ToolInputConvert: Sized {
    fn into_tool_input(self) -> ToolInput;
    fn from_tool_input(input: &ToolInput) -> Result<Self, WorkflowError>;
    fn validate(&self) -> Result<(), WorkflowError>;
    fn schema() -> InputSchema;
}
```

**自动实现**: 使用 `#[derive(ToolInput)]` 宏自动生成。

---

### ToolOutputConvert trait

强类型输出转换trait。

```rust
pub trait ToolOutputConvert: Sized {
    fn into_tool_output(self) -> ToolOutput;
    fn from_tool_output(output: &ToolOutput) -> Result<Self, WorkflowError>;
}
```

**自动实现**: 使用 `#[derive(ToolOutput)]` 宏自动生成。

---

### 派生宏

#### `#[derive(ToolInput)]`

为结构体自动生成 `ToolInputConvert` 实现。

**字段属性**:
- `#[tool_input(description = "...")]` - 字段描述
- `#[tool_input(required = true)]` - 必需字段
- `#[tool_input(default = ...)]` - 默认值
- `#[tool_input(validate = "...")]` - 验证规则

**示例**:
```rust
#[derive(ToolInput, Serialize, Deserialize)]
pub struct EchoInput {
    #[tool_input(description = "Message to echo", required = true)]
    pub message: String,
    
    #[tool_input(description = "Number of times", default = 1)]
    pub count: u32,
}
```

#### `#[derive(ToolOutput)]`

为结构体自动生成 `ToolOutputConvert` 实现。

**示例**:
```rust
#[derive(ToolOutput, Serialize, Deserialize)]
pub struct EchoOutput {
    pub echoed_messages: Vec<String>,
    pub total_chars: usize,
}
```

---

## 组合工具

### CompositionType

工具组合类型枚举。

```rust
pub enum CompositionType {
    Chain(Vec<ToolId>),
    Conditional {
        condition: String,
        then_tool: ToolId,
        else_tool: Option<ToolId>,
    },
    Parallel(Vec<ToolId>),
}
```

---

### ComposedTool

组合工具结构。

```rust
pub struct ComposedTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub composition_type: CompositionType,
    pub tools: Vec<ToolId>,
    pub middleware_stack: Option<MiddlewareStack>,
}
```

---

## 兼容性层

### 旧Trait（compat模块）

为向后兼容提供的旧trait定义。

```rust
pub mod compat {
    #[async_trait]
    pub trait ToolNode: Send + Sync { ... }
    
    #[async_trait]
    pub trait ToolExecutor: Send + Sync { ... }
    
    #[async_trait]
    pub trait ToolRegistry: Send + Sync { ... }
    
    #[async_trait]
    pub trait ComposableTool: Send + Sync { ... }
    
    pub struct BasicToolRegistry;
}
```

**注意**: 这些trait将在未来版本中移除，请尽快迁移到新API。

---

## 错误处理

### WorkflowError

工具系统错误类型。

```rust
pub enum WorkflowError {
    ValidationError(String),
    ToolExecution { tool_name: String, message: String },
    ToolNotFound { name: String },
    InvalidToolId { id: ToolId },
    MiddlewareError { middleware: String, message: String },
    // ... 其他变体
}
```

**方法**:

#### `validation_error`
```rust
pub fn validation_error(msg: impl Into<String>) -> Self
```

#### `tool_execution`
```rust
pub fn tool_execution(msg: impl Into<String>) -> Self
```

---

## 完整示例

### 示例1: 简单工具

```rust
use workflow_toolkit::tools::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建注册表
    let registry = ToolRegistry::new();
    
    // 创建工具
    let tool = NativeToolBuilder::new()
        .name("echo")
        .version("1.0.0")
        .executor(|input, _ctx| async move {
            Ok(ToolOutput::success(input.params))
        })
        .build()?;
    
    // 注册
    registry.register(Tool::Native(Arc::new(tool)));
    
    // 执行
    let input = ToolInput::new(json!("Hello, World!"));
    let output = registry.execute("echo", input).await?;
    
    println!("Result: {:?}", output.result);
    Ok(())
}
```

### 示例2: 带中间件的工具

```rust
use workflow_toolkit::tools::*;

let mut stack = MiddlewareStack::new();
stack.add(Arc::new(LoggingMiddleware::new()));
stack.add(Arc::new(TimingMiddleware::new()));

let tool = NativeToolBuilder::new()
    .name("logged_tool")
    .version("1.0.0")
    .executor(|input, _ctx| async move {
        // 执行逻辑
        Ok(ToolOutput::success(input.params))
    })
    .with_middleware(stack)
    .build()?;
```

### 示例3: 强类型工具

```rust
use workflow_toolkit::tools::*;
use workflow_toolkit::macros::ToolInput;
use serde::{Serialize, Deserialize};

#[derive(ToolInput, Serialize, Deserialize)]
pub struct CalculatorInput {
    #[tool_input(description = "First number", required = true)]
    pub a: f64,
    
    #[tool_input(description = "Second number", required = true)]
    pub b: f64,
    
    #[tool_input(description = "Operation", required = true)]
    pub op: String,
}

let tool = NativeToolBuilder::new()
    .name("calculator")
    .executor(|input, _ctx| async move {
        let calc = CalculatorInput::from_tool_input(&input)?;
        
        let result = match calc.op.as_str() {
            "add" => calc.a + calc.b,
            "sub" => calc.a - calc.b,
            _ => return Err(WorkflowError::validation_error("Invalid op")),
        };
        
        Ok(ToolOutput::success(json!(result)))
    })
    .build()?;
```

---

## 性能提示

### 1. 使用枚举而非动态分发
```rust
// 好 - 静态分发
let tool: Tool = registry.get("name").unwrap();
tool.execute(input, ctx).await?;

// 避免 - 动态分发
let tool: Arc<dyn ToolNode> = ...; // 旧方式
```

### 2. 批量注册使用并行
```rust
use tokio::join;

let handles = tools.into_iter().map(|tool| {
    let registry = registry.clone();
    tokio::spawn(async move {
        registry.register(tool)
    })
});

join_all(handles).await;
```

### 3. 缓存工具元数据
```rust
// 注册表已内置缓存
let metadata = registry.get_metadata("tool_name"); // O(1)
```

---

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| 0.2.0-alpha | 2026-02-01 | 激进优化：枚举类型系统、中间件、强类型参数 |
| 0.1.0 | 2026-01-01 | 初始版本：trait-based系统 |

---

## 另请参阅

- [迁移指南](./migration-guide.md) - 从旧系统迁移
- [测试计划](./test-plan.md) - 测试策略
- [学习记录](./learnings.md) - 设计决策

---

**文档版本**: 1.0  
**最后更新**: 2026-02-01  
**维护者**: Atlas Orchestrator
