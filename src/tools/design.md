# 工具系统 (Tool System)

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

```rust
/// 工具组合器
pub struct ToolComposer;

/// 组合工具
pub struct ComposedTool {
    pub composition_type: CompositionType,
    pub tools: Vec<ToolId>,
}

pub enum CompositionType {
    /// 顺序执行
    Sequence,
    /// 并行执行
    Parallel,
    /// 条件执行
    Conditional { condition: Box<dyn Fn(&Value) -> bool> },
    /// 分支执行
    Branch { selector: Box<dyn Fn(&Value) -> usize> },
}

/// 工具链
pub struct ToolChain {
    tools: Vec<ToolId>,
    data_flow: HashMap<String, String>, // 输出到输入的映射
}

/// 并行工具
pub struct ParallelTools {
    tools: Vec<ToolId>,
    merge_strategy: MergeStrategy,
}
```

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
