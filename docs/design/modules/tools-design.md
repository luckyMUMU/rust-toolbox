# 工具系统 (Tool System)

> **版本**: v1.0  
> **创建日期**: 2026-01-15  
> **最后更新**: 2026-02-27  
> **维护者**: Workflow Toolkit Team

## 1. 核心定义 (Stable)

### 1.1 模块职责

工具系统负责工具的注册、发现、组合和执行。采用 Enum-based 架构（已重构自 trait-based），提供更好的性能和类型安全。

### 1.2 模块结构

```
tools/
├── types.rs        # 工具类型定义（Enum-based）
├── registry.rs     # 工具注册表
├── node.rs         # 工具节点实现
├── composable.rs   # 可组合工具
├── middleware.rs   # 中间件系统
├── template.rs     # 模板系统
├── version.rs      # 版本管理
├── compat.rs       # 兼容层（旧 trait）
└── algo/           # 算法模块
    └── mod.rs
```

### 1.3 核心类型

#### 工具 Enum

```rust
/// 工具 Enum（新架构）
pub enum Tool {
    /// 原生 Rust 实现
    Native(NativeTool),
    /// Python 脚本
    Python(PythonTool),
    /// Node.js 模块
    NodeJs(NodeJsTool),
    /// Docker 容器
    Docker(DockerTool),
    /// WebAssembly
    Wasm(WasmTool),
    /// 组合工具
    Composed(ComposedTool),
}

impl Tool {
    /// 执行工具
    pub async fn execute(&self, input: ToolInput) -> Result<ToolOutput>;
    
    /// 获取元数据
    pub fn metadata(&self) -> &ToolMetadata;
    
    /// 获取输入 Schema
    pub fn input_schema(&self) -> &InputSchema;
    
    /// 获取输出 Schema
    pub fn output_schema(&self) -> &OutputSchema;
}
```

#### 工具元数据

```rust
/// 工具元数据
pub struct ToolMetadata {
    pub id: ToolId,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 工具 ID
pub struct ToolId(pub Uuid);

/// 工具种类
pub enum ToolKind {
    Native,
    Python,
    NodeJs,
    Docker,
    Wasm,
    Composed,
}
```

#### 输入输出

```rust
/// 工具输入
pub struct ToolInput {
    pub params: Value,
    pub context: Option<ExecutionContext>,
}

/// 工具输出
pub struct ToolOutput {
    pub result: Value,
    pub metadata: OutputMetadata,
}

/// 输出元数据
pub struct OutputMetadata {
    pub execution_time_ms: u64,
    pub cached: bool,
}
```

### 1.4 工具注册表

```rust
/// 工具注册表
pub struct ToolRegistry {
    tools: DashMap<ToolId, Tool>,
    name_index: DashMap<String, ToolId>,
}

impl ToolRegistry {
    /// 注册工具
    pub fn register(&self, tool: Tool) -> Result<ToolId>;
    
    /// 注销工具
    pub fn unregister(&self, id: &ToolId) -> Result<()>;
    
    /// 通过 ID 获取工具
    pub fn get(&self, id: &ToolId) -> Option<Tool>;
    
    /// 通过名称获取工具
    pub fn get_by_name(&self, name: &str) -> Option<Tool>;
    
    /// 列出所有工具
    pub fn list_all(&self) -> Vec<ToolMetadata>;
    
    /// 按类别筛选
    pub fn filter_by_category(&self, category: &str) -> Vec<ToolMetadata>;
}

/// 注册表构建器
pub struct ToolRegistryBuilder;
```

### 1.5 中间件系统

```rust
/// 中间件（Middleware）trait
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, ctx: MiddlewareContext, next: Next<'_>) -> Result<ToolOutput>;
}

/// 中间件上下文
pub struct MiddlewareContext {
    pub tool_id: ToolId,
    pub input: ToolInput,
    pub execution_metadata: ExecutionMetadata,
}

/// 中间件栈
pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareStack {
    pub fn builder() -> MiddlewareStackBuilder;
}

/// 内置中间件
pub struct RetryMiddleware;
pub struct CacheMiddleware;
pub struct TimeoutMiddleware;
pub struct LoggingMiddleware;
pub struct MetricsMiddleware;
pub struct CircuitBreakerMiddleware;
pub struct TimingMiddleware;
```

### 1.6 可组合工具

#### 组合工具结构

```rust
/// 组合工具（Composed Tool）
pub struct ComposedTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub composition_type: CompositionType,
    pub tools: Vec<ToolId>,
    pub data_flow: Option<DataFlowMapping>,      // 数据流映射（可选）
    pub error_strategy: ErrorPropagationStrategy, // 错误处理策略
    pub max_concurrency: usize,                   // 最大并发数（并行模式）
}

/// 组合类型
pub enum CompositionType {
    /// 链式执行：A 的输出 → B 的输入 → C 的输入
    Chain(Vec<ToolId>),
    
    /// 条件执行：根据条件选择分支
    Conditional {
        condition: String,  // EL 表达式
        then_tool: ToolId,
        else_tool: Option<ToolId>,
    },
    
    /// 并行执行：同时执行多个工具
    Parallel(Vec<ToolId>),
}
```

#### 数据流映射

```rust
/// 数据流映射配置
pub struct DataFlowMapping {
    /// 映射规则：输出路径 → 输入路径
    /// 示例："/result/data" → "/params/input"
    pub mappings: HashMap<String, String>,
}

impl DataFlowMapping {
    /// 将上一个工具的输出转换为下一个工具的输入
    pub fn transform(&self, output: &Value) -> Result<Value>;
}
```

#### 错误传播策略

```rust
/// 错误处理策略
pub enum ErrorPropagationStrategy {
    /// 快速失败：第一个错误发生时立即停止
    FailFast,
    
    /// 继续执行：收集所有错误，最后统一返回
    ContinueOnError,
    
    /// 重试策略：失败时重试指定次数
    Retry {
        max_retries: u32,
        delay_ms: u64,
    },
}
```

#### 设计原则

1. **原子性**: 组合工具对外表现为单个工具，调用方无需关心里面的组合细节
2. **透明性**: 中间件对组合工具同样生效（日志、缓存、重试等）
3. **嵌套支持**: 组合工具可以包含其他组合工具，支持复杂编排
4. **数据流可控**: 通过 `DataFlowMapping` 精确控制工具间的数据传递
5. **错误可配置**: 通过 `ErrorPropagationStrategy` 配置错误处理行为

### 1.7 模板系统

```rust
/// 模板引擎（Template Engine）
pub struct TemplateEngine;

/// 参数模板
pub struct ParameterTemplate {
    pub template: String,
    pub variables: Vec<String>,
}

/// 模板上下文
pub struct TemplateContext {
    pub variables: HashMap<String, Value>,
    pub execution_context: Option<ExecutionContext>,
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-T001: Enum vs Trait
- **决策**: 从 trait-based 重构为 Enum-based
- **理由**: 更好的性能（避免虚表调用），更简单的序列化
- **风险**: 扩展性稍差，新增类型需要修改 Enum

#### ADR-T002: 中间件模式
- **决策**: 采用洋葱模型（Onion Model）中间件链
- **理由**: 支持横切关注点（日志、缓存、重试）的复用
- **风险**: 调用栈深度增加

#### ADR-T003: 组合工具设计
- **决策**: 组合也是工具（ComposedTool 是 Tool 的变体）
- **理由**: 统一接口，支持嵌套组合
- **风险**: 调试复杂度增加

### 2.1.1 设计决策理由详解

#### 决策 1: 为什么从 Trait 重构为 Enum？

**背景问题**：
原有的 trait-based 架构存在性能和序列化问题：
- 动态分发（vtable）有运行时开销
- `Box<dyn Tool>` 难以序列化和反序列化
- 类型信息在运行时丢失

**考虑的选项**：

| 选项 | 性能 | 序列化 | 类型安全 | 扩展性 |
|------|------|--------|----------|--------|
| Trait (dyn) | 中 | 困难 | 弱 | 高 |
| **Enum** | **高** | **简单** | **强** | 中 |
| Generic | 最高 | 中 | 最强 | 低 |

**选择理由**：
1. **性能优化**：Enum 匹配是编译时确定的，无虚表调用开销
2. **序列化友好**：Enum 可直接使用 serde 序列化
3. **类型安全**：编译器可检查所有变体的处理
4. **代码简洁**：不需要 `Box<dyn Trait>` 包装

**性能对比**：
```rust
// Trait-based: 动态分发
let tool: Box<dyn Tool> = get_tool();
tool.execute(input).await?;  // 虚表调用

// Enum-based: 静态分发
let tool: Tool = get_tool();
match &tool {
    Tool::Native(t) => t.execute(input).await?,
    Tool::Python(t) => t.execute(input).await?,
    // 编译器可内联优化
}
```

**权衡与限制**：
- 新增工具类型需要修改 Enum 定义
- Enum 变体数量不宜过多（建议 < 10 个）
- 可通过 `#[non_exhaustive]` 保持 API 兼容性

#### 决策 2: 为什么采用洋葱模型中间件？

**背景问题**：
工具执行需要处理多种横切关注点（日志、缓存、重试、超时），直接在执行代码中处理会导致：
- 代码重复
- 关注点耦合
- 难以统一管理

**考虑的选项**：

| 选项 | 灵活性 | 复用性 | 实现复杂度 |
|------|--------|--------|------------|
| 直接编码 | 低 | 无 | 低 |
| 装饰器模式 | 中 | 中 | 中 |
| **洋葱中间件** | **高** | **高** | 中 |
| AOP 框架 | 最高 | 最高 | 高 |

**选择理由**：
1. **关注点分离**：每个中间件专注一个功能
2. **可组合**：中间件可任意组合和排序
3. **请求/响应拦截**：可在执行前后添加逻辑
4. **生态成熟**：参考 Tower middleware 设计

**洋葱模型示意**：
```
请求 → [日志] → [缓存] → [重试] → [工具执行] → [重试] → [缓存] → [日志] → 响应
        ↓         ↓         ↓           ↑         ↑         ↑         ↑
      记录开始  检查缓存  准备重试     执行     处理重试  更新缓存  记录结束
```

**中间件执行流程**：
```rust
// 中间件链执行示例
async fn execute_with_middleware(&self, input: ToolInput) -> Result<ToolOutput> {
    let stack = MiddlewareStack::new()
        .with(LoggingMiddleware::new())
        .with(CacheMiddleware::new(cache))
        .with(RetryMiddleware::new(3));
    
    stack.execute(input, |input| self.inner_execute(input)).await
}
```

#### 决策 3: 为什么组合工具也是 Tool 变体？

**背景问题**：
需要支持工具的组合（顺序执行、并行执行、条件执行），如何设计组合工具的类型？

**考虑的选项**：

| 选项 | 类型统一 | 嵌套组合 | 接口一致性 |
|------|----------|----------|------------|
| 独立类型 | 否 | 复杂 | 不一致 |
| **Tool 变体** | **是** | **简单** | **一致** |
| 包装器 | 部分 | 中 | 中 |

**选择理由**：
1. **统一接口**：所有工具（包括组合）都通过 `Tool::execute` 调用
2. **嵌套组合**：组合工具可以包含其他组合工具，支持复杂编排
3. **透明性**：调用方无需关心是单个工具还是组合工具
4. **递归处理**：中间件对组合工具同样生效

**组合工具示例**：
```rust
// 顺序组合
let pipeline = Tool::Composed(ComposedTool {
    composition_type: CompositionType::Sequence,
    tools: vec![tool_a_id, tool_b_id, tool_c_id],
});

// 并行组合
let parallel = Tool::Composed(ComposedTool {
    composition_type: CompositionType::Parallel,
    tools: vec![tool_x_id, tool_y_id],
});

// 嵌套组合：顺序执行中包含并行
let nested = Tool::Composed(ComposedTool {
    composition_type: CompositionType::Sequence,
    tools: vec![pipeline_id, parallel_id],
});
```

### 2.2 任务清单

- [x] Task 0: 渐进式重构（修复旧逻辑/格式）
- [x] Task 1: Tool Enum 实现
- [x] Task 2: 新注册表实现
- [x] Task 3: 工具节点实现
- [x] Task 4: 中间件系统
- [x] Task 5: 组合工具完善
- [ ] Task 6: 版本管理实现

### 2.3 接口契约

```rust
/// 工具执行配置
pub struct ToolExecutionConfig {
    pub timeout: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
    pub use_cache: bool,
    pub cache_ttl: Option<Duration>,
}

/// 工具执行结果
pub struct ToolExecutionResult {
    pub success: bool,
    pub output: Option<ToolOutput>,
    pub error: Option<ToolError>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
}

/// 工具错误
pub enum ToolError {
    NotFound { tool_id: ToolId },
    InvalidInput { message: String },
    ExecutionFailed { message: String },
    Timeout { duration: Duration },
    Cancelled,
}
```

### 2.4 测试策略

- **单元测试（Unit Test）**: 各工具类型独立测试
- **集成测试（Integration Test）**: 工具注册/执行/组合完整流程
- **性能测试（Performance Test）**: Enum vs Trait 性能对比
- **兼容性测试（Compatibility Test）**: 旧 trait 兼容层测试

### 2.5 组合工具执行语义

#### 链式执行（Chain Execution）

**执行流程**:
```
Input → [Tool A] → Output A → [DataFlow Transform] → Input B → [Tool B] → ... → Final Output
```

**执行语义**:
1. 按顺序依次执行每个工具
2. 前一个工具的输出通过 `DataFlowMapping` 转换为下一个工具的输入
3. 任一步骤失败时，根据 `ErrorPropagationStrategy` 处理：
   - `FailFast`: 立即停止并返回错误
   - `ContinueOnError`: 记录错误，继续执行后续工具
   - `Retry`: 重试指定次数
4. 返回所有工具的执行结果和最终输出

**伪代码**:
```rust
async fn execute_chain(&self, input: ToolInput, ctx: ExecutionContext) -> Result<ToolOutput> {
    let mut current_input = input;
    let mut results = Vec::new();
    
    for (index, &tool_id) in tools.iter().enumerate() {
        // 获取工具
        let tool = registry.get_by_id(tool_id)?;
        
        // 执行工具
        let output = tool.execute(current_input, ctx.clone()).await?;
        
        // 错误处理
        if !output.success {
            match error_strategy {
                FailFast => return Err(...),
                ContinueOnError => { results.push(output); continue; },
                Retry { max_retries, delay } => { /* 重试逻辑 */ }
            }
        }
        
        // 数据流转换（为下一个工具准备输入）
        if let Some(mapping) = &self.data_flow {
            current_input = ToolInput::new(mapping.transform(&output.result)?);
        }
        
        results.push(output);
    }
    
    Ok(ToolOutput::success(final_result))
}
```

#### 条件执行（Conditional Execution）

**执行流程**:
```
Input → [Evaluate Condition] → (true ? Then Tool : Else Tool) → Output
```

**执行语义**:
1. 使用 EL 表达式引擎评估条件表达式
2. 根据条件结果选择执行 `then_tool` 或 `else_tool`
3. 返回选中分支的执行结果

**伪代码**:
```rust
async fn execute_conditional(&self, input: ToolInput, ctx: ExecutionContext) -> Result<ToolOutput> {
    // 评估条件
    let engine = ExpressionEngine::new();
    let condition_result = engine.evaluate(&self.condition, &input.params)?;
    
    // 选择分支
    let selected_tool = if condition_result.as_bool().unwrap_or(false) {
        self.then_tool
    } else {
        self.else_tool.ok_or_else(|| Error::NoElseBranch)?
    };
    
    // 执行选中的工具
    let tool = registry.get_by_id(selected_tool)?;
    let output = tool.execute(input, ctx).await?;
    
    Ok(ToolOutput::success(json!({
        "condition": self.condition,
        "result": condition_result,
        "branch": if selected_tool == self.then_tool { "then" } else { "else" },
        "output": output.result
    })))
}
```

#### 并行执行（Parallel Execution）

**执行流程**:
```
Input → [Tool A] ─┬→ [Merge Results] → Output
        → [Tool B] ─┤
        → [Tool C] ─┘
```

**执行语义**:
1. 并发执行所有工具（受 `max_concurrency` 限制）
2. 所有工具共享相同的输入
3. 收集所有工具的执行结果
4. 错误处理策略：
   - `FailFast`: 任一失败则整体失败
   - `ContinueOnError`: 部分成功也返回成功，包含失败信息

**伪代码**:
```rust
async fn execute_parallel(&self, input: ToolInput, ctx: ExecutionContext) -> Result<ToolOutput> {
    let futures = tools.iter().map(|&tool_id| {
        let tool = registry.get_by_id(tool_id)?;
        tool.execute(input.clone(), ctx.clone())
    });
    
    // 并发执行（限制并发数）
    let results = futures::stream::iter(futures)
        .buffer_unordered(self.max_concurrency)
        .collect::<Vec<_>>()
        .await;
    
    // 错误处理
    let (successes, failures): (Vec<_>, Vec<_>) = results.into_iter()
        .partition(|r| r.is_ok() && r.as_ref().unwrap().success);
    
    if !failures.is_empty() && matches!(error_strategy, FailFast) {
        return Err(...);
    }
    
    Ok(ToolOutput::success(json!({
        "total": tools.len(),
        "successful": successes.len(),
        "failed": failures.len(),
        "results": successes
    })))
}
```

### 2.6 数据流映射详解

#### 路径语法

数据流映射使用 **JSON Pointer** 风格的路径语法：

| 路径示例 | 说明 |
|---------|------|
| `/result` | 访问根对象的 `result` 字段 |
| `/data/items/0` | 访问 `data.items` 数组的第一个元素 |
| `/user/name` | 访问嵌套的 `user.name` |

#### 映射示例

**示例 1: 简单映射**
```json
{
  "mappings": {
    "/result/value": "/params/input",
    "/result/status": "/params.status"
  }
}
```

**示例 2: 数组访问**
```json
{
  "mappings": {
    "/data/items/0/id": "/params/user_id"
  }
}
```

#### 转换算法

```rust
fn transform(&self, output: &Value) -> Result<Value> {
    let mut input_map = serde_json::Map::new();
    
    for (from_path, to_path) in &self.mappings {
        // 1. 从输出中提取值（支持嵌套路径和数组索引）
        let value = self.extract_path(output, from_path)?;
        
        // 2. 设置到输入路径（自动创建嵌套结构）
        self.set_path(&mut input_map, to_path, value)?;
    }
    
    Ok(Value::Object(input_map))
}
```

## 3. 状态记录

- `[已完成]` | 组合工具完善 | 2026-02-07
- `[已完成]` | 中间件系统 | 2026-02-01
- `[已完成]` | Enum 重构 | 2026-01-28
- `[已完成]` | 新注册表 | 2026-01-25

## 4. 架构演进

### 旧架构（Trait-based）
```rust
// 已废弃，移至 compat 模块
trait Tool { async fn execute(&self, input: Value) -> Result<Value>; }
```

### 新架构（Enum-based）
```rust
// 当前实现
enum Tool { Native(...), Python(...), ... }
impl Tool { async fn execute(&self, ...) -> ... }
```

## 5. 使用示例

```rust
// 注册工具
let registry = ToolRegistry::new();
let tool = Tool::Native(NativeTool::new("calculator", calc_fn));
let id = registry.register(tool)?;

// 执行工具
let input = ToolInput::new(json!({"a": 1, "b": 2}));
let output = registry.get(&id).unwrap().execute(input).await?;

// 组合工具
let chain = ToolChain::builder()
    .add(tool_a)
    .add(tool_b)
    .map_output("result", "input")
    .build();
```

---

## 6. 工具开发指南

### 6.1 自定义工具开发规范

#### 工具接口要求

所有自定义工具必须满足以下要求：

1. **实现 ToolNode trait** 或 **包装为 Tool Enum 变体**
2. **提供完整的 Schema 定义**（输入/输出）
3. **支持参数验证**
4. **正确处理错误**

#### 开发原生工具（Native Tool）

```rust
use workflow_toolkit::{
    tools::{ToolNode, ToolDefinition, ToolInfo},
    ExecutionContext, WorkflowError,
};
use serde_json::{json, Value};
use async_trait::async_trait;

/// 自定义计算器工具
pub struct CalculatorTool {
    info: ToolInfo,
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            info: ToolInfo {
                name: "calculator".to_string(),
                version: "1.0.0".to_string(),
                description: "基础数学计算工具".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["add", "subtract", "multiply", "divide"]
                        },
                        "a": { "type": "number" },
                        "b": { "type": "number" }
                    },
                    "required": ["operation", "a", "b"]
                }),
                return_schema: json!({
                    "type": "object",
                    "properties": {
                        "result": { "type": "number" }
                    }
                }),
                category: Some("math".to_string()),
                tags: vec!["calculation".to_string()],
                dependencies: vec![],
                plugin_name: None,
            },
        }
    }
}

#[async_trait]
impl ToolNode for CalculatorTool {
    fn name(&self) -> &str { &self.info.name }
    fn version(&self) -> &str { &self.info.version }
    
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value, WorkflowError> {
        let operation = params["operation"].as_str()
            .ok_or_else(|| WorkflowError::invalid_input("缺少 operation 参数"))?;
        let a = params["a"].as_f64()
            .ok_or_else(|| WorkflowError::invalid_input("缺少 a 参数"))?;
        let b = params["b"].as_f64()
            .ok_or_else(|| WorkflowError::invalid_input("缺少 b 参数"))?;
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(WorkflowError::tool_execution("除零错误"));
                }
                a / b
            }
            _ => return Err(WorkflowError::invalid_input(&format!("未知操作: {}", operation))),
        };
        
        Ok(json!({ "result": result }))
    }
    
    fn validate_parameters(&self, params: &Value) -> Result<(), WorkflowError> {
        // 使用 JSON Schema 验证
        if params["operation"].as_str().is_none() {
            return Err(WorkflowError::invalid_input("缺少 operation 参数"));
        }
        if params["a"].as_f64().is_none() {
            return Err(WorkflowError::invalid_input("缺少 a 参数或类型错误"));
        }
        if params["b"].as_f64().is_none() {
            return Err(WorkflowError::invalid_input("缺少 b 参数或类型错误"));
        }
        Ok(())
    }
    
    fn get_schema(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.info.name.clone(),
            version: self.info.version.clone(),
            description: self.info.description.clone(),
            parameters_schema: self.info.parameters_schema.clone(),
            return_schema: self.info.return_schema.clone(),
        }
    }
    
    fn get_plugin_info(&self) -> Option<&PluginInfo> { None }
}
```

### 6.2 工具注册流程

```rust
// 方式一：直接注册到注册表
let mut registry = ToolRegistry::new();
let calculator = CalculatorTool::new();
registry.register_tool(Arc::new(calculator))?;

// 方式二：使用 Tool Enum
let tool = Tool::Native(NativeTool::from_fn("my_tool", |params, ctx| async move {
    // 工具逻辑
    Ok(json!({"status": "success"}))
}));
registry.register(tool)?;
```

### 6.3 工具配置规范

```yaml
# tool-config.yaml
name: my-custom-tool
version: 1.0.0
description: 自定义工具描述
category: utility
tags:
  - custom
  - utility

# 输入 Schema (JSON Schema 格式)
input_schema:
  type: object
  properties:
    input_path:
      type: string
      description: 输入文件路径
    output_path:
      type: string
      description: 输出文件路径
  required:
    - input_path

# 输出 Schema
output_schema:
  type: object
  properties:
    success:
      type: boolean
    message:
      type: string

# 执行配置
execution:
  timeout_seconds: 30
  retry_count: 3
  retry_delay_ms: 1000
  cache_enabled: true
  cache_ttl_seconds: 300
```

### 6.4 工具测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_calculator_add() {
        let tool = CalculatorTool::new();
        let params = json!({"operation": "add", "a": 2, "b": 3});
        
        let result = tool.execute(params, ExecutionContext::default()).await.unwrap();
        
        assert_eq!(result["result"], 5.0);
    }
    
    #[test]
    fn test_parameter_validation() {
        let tool = CalculatorTool::new();
        
        // 缺少必需参数
        let result = tool.validate_parameters(&json!({"a": 1}));
        assert!(result.is_err());
        
        // 参数类型错误
        let result = tool.validate_parameters(&json!({"operation": "add", "a": "not_a_number", "b": 2}));
        assert!(result.is_err());
        
        // 正确参数
        let result = tool.validate_parameters(&json!({"operation": "add", "a": 1, "b": 2}));
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_divide_by_zero() {
        let tool = CalculatorTool::new();
        let params = json!({"operation": "divide", "a": 1, "b": 0});
        
        let result = tool.execute(params, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
}
```

### 6.5 工具扩展点

| 扩展点 | 接口 | 用途 |
|--------|------|------|
| 自定义执行器 | `AsyncFunctionExecutor` | 包装异步函数为工具 |
| 中间件 | `Middleware` trait | 添加横切关注点（日志、缓存、重试） |
| 参数验证器 | `ParameterValidator` trait | 自定义参数验证逻辑 |
| 结果处理器 | `ResultProcessor` trait | 后处理工具输出 |

### 6.6 工具版本管理

```rust
/// 工具版本兼容性检查
pub fn check_compatibility(tool_version: &str, required_version: &str) -> bool {
    // 使用语义化版本检查
    let tool_parts: Vec<u32> = tool_version.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let required_parts: Vec<u32> = required_version.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    // 主版本号必须匹配
    if tool_parts.get(0) != required_parts.get(0) {
        return false;
    }
    
    // 工具次版本号 >= 要求次版本号
    tool_parts.get(1).unwrap_or(&0) >= required_parts.get(1).unwrap_or(&0)
}
```
