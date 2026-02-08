# Workflow Toolkit 代码库系统性审查报告

> **审查日期**: 2026-02-08  
> **审查范围**: 工具设计、插件系统、工作流执行、类型系统  
> **审查模式**: ULW (Ultra Large Work) 深度分析

---

## 目录

1. [执行摘要](#执行摘要)
2. [架构概览](#架构概览)
3. [工具系统设计分析](#工具系统设计分析)
4. [插件系统设计分析](#插件系统设计分析)
5. [工作流执行系统分析](#工作流执行系统分析)
6. [类型系统与序列化分析](#类型系统与序列化分析)
7. [设计模式与最佳实践](#设计模式与最佳实践)
8. [发现的问题与风险](#发现的问题与风险)
9. [改进建议](#改进建议)
10. [总结](#总结)

---

## 执行摘要

### 项目概况
Workflow Toolkit 是一个基于 Rust 的多接口工作流执行系统，支持 CLI、TUI 和 MCP Server 三种交互模式。项目采用 **DDD 分层架构** 和 **LiteFlow 风格的组件化设计**。

### 关键指标
| 指标 | 值 | 评价 |
|------|-----|------|
| 源代码文件数 | ~100+ .rs 文件 | 中等规模 |
| 架构复杂度 | 高 (分层 + 组件 + 插件) | 需要良好的文档 |
| 类型安全 | 高 (强类型 + 枚举系统) | Rust 优势发挥充分 |
| 并发模型 | 基于 Tokio 的异步 | 现代化 |
| 测试覆盖 | 单元测试 + 集成测试 | 需要更多 E2E 测试 |
| 文档完整性 | 较好 (设计文档齐全) | 代码注释可加强 |

### 整体评价
**优势**: 架构设计先进，模块化程度高，遵循 Rust 最佳实践，安全性考虑充分。  
**风险**: 部分功能处于重构中，兼容层代码较多，WASM 插件暂时禁用。

---

## 架构概览

### 1. 分层架构 (DDD 风格)

```
┌─────────────────────────────────────────────────────────────┐
│                      接口层 (Interfaces)                      │
│         CLI        TUI        MCP Server                     │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    应用层 (Application)                       │
│         UseCase    Service    Workflow Orchestration         │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    领域层 (Domain)                            │
│         Model      Port (Repository/Service Interface)       │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                  基础设施层 (Infrastructure)                  │
│    Persistence   Plugin   Cache   External   Config          │
└─────────────────────────────────────────────────────────────┘
```

### 2. 核心组件关系

```
WorkflowEngine
├── ComponentRegistry (组件注册表)
├── ExecutorChain (执行器链)
│   ├── BasicExecutor (基础执行)
│   ├── RetryExecutor (重试)
│   ├── CacheExecutor (缓存)
│   └── AuditExecutor (审计)
├── ExecutionTracker (执行追踪)
├── DagScheduler (DAG 调度器)
└── StateManager (状态管理)
```

### 3. 关键领域模型

| 领域实体 | 文件位置 | 核心职责 |
|---------|----------|----------|
| **Tool** | `src/tools/types.rs:146` | 工具枚举，统一所有工具类型的接口 |
| **Plugin** | `src/plugins/types.rs:42` | 插件 trait，定义插件标准接口 |
| **WorkflowDefinition** | `src/workflow/definition.rs` | 工作流定义，DAG 结构 |
| **Component** | `src/workflow/component/mod.rs` | 组件 trait，LiteFlow 风格执行单元 |
| **ExecutionContext** | `src/core/mod.rs` | 执行上下文，贯穿执行链路 |

---

## 工具系统设计分析

### 1. 核心设计: 枚举 vs 特质

**关键决策**: 从特质对象 (`dyn ToolNode`) 迁移到枚举 (`Tool`)

```rust
// 新系统: 枚举实现 (src/tools/types.rs:146)
pub enum Tool {
    Native(Arc<NativeTool>),
    Python(Arc<PythonTool>),
    NodeJs(Arc<NodeJsTool>),
    Docker(Arc<DockerTool>),
    Wasm(Arc<WasmTool>),
    Composed(Arc<ComposedTool>),
}
```

**优势分析**:
- ✅ **零成本抽象**: 静态分发，无动态调度开销
- ✅ **类型安全**: 编译期检查所有工具类型
- ✅ **性能提升**: 30-50% 性能提升（文档声明）
- ✅ **更好的错误信息**: 编译器可提供更精确的错误

**迁移状态** (src/tools/mod.rs:6-16):
```
- [x] ToolRegistry trait - REMOVED (compat 层保留)
- [x] ToolNode trait - REMOVED (compat 层保留)
- [x] Tool enum - DONE
- [x] New registry implementation - DONE
- [x] Middleware system - DONE
- [x] Typed tool system - DONE
```

### 2. 工具类型分析

#### 2.1 Native Tool (原生 Rust 工具)

**位置**: `src/tools/types.rs:242-301`

```rust
pub struct NativeTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub executor: Arc<dyn Fn(ToolInput, ExecutionContext) -> BoxFuture<'static, Result<ToolOutput>> + Send + Sync>,
    pub middleware_stack: Option<MiddlewareStack>,
}
```

**设计亮点**:
- 使用 `Arc<...>` 实现共享所有权
- 闭包存储执行器，灵活性高
- 支持中间件栈注入

#### 2.2 Python Tool

**位置**: `src/tools/types.rs:304-354`

```rust
pub struct PythonTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub script_path: PathBuf,
    pub python_path: PathBuf,
    pub timeout_secs: u64,
    pub middleware_stack: Option<MiddlewareStack>,
}
```

**现状**: 执行逻辑为 TODO (src/tools/types.rs:326)
```rust
// TODO: Implement actual Python execution
// For now, return a placeholder
```

#### 2.3 Node.js / Docker / WASM Tool

同样结构，路径/镜像替换即可。

**共同特点**:
- 统一的基础元数据 (`ToolMetadata`)
- 统一的中间件支持
- 统一的超时配置

### 3. 工具注册表设计

**位置**: `src/tools/registry.rs`

#### 3.1 多索引结构

```rust
pub struct ToolRegistry {
    tools: DashMap<ToolId, Tool>,           // 主存储
    name_index: DashMap<String, ToolId>,    // 名称索引
    metadata_cache: DashMap<ToolId, Arc<ToolMetadata>>, // 元数据缓存
    versions: DashMap<ToolId, Vec<Version>>, // 版本管理
    category_index: DashMap<String, Vec<ToolId>>, // 分类索引
    tag_index: DashMap<String, Vec<ToolId>>, // 标签索引
}
```

**性能分析**:
- ✅ 使用 `DashMap` 实现高并发读写
- ✅ O(1) 名称/ID 查找
- ✅ 多维度索引（分类、标签）

#### 3.2 版本管理

支持版本冲突检测 (src/tools/registry.rs:284-326):
```rust
pub fn check_version_conflicts(&self) -> Vec<String> {
    // 主版本号必须匹配才能兼容
    if v1.major != v2.major {
        conflicts.push(format!("{}: incompatible versions {} and {}", ...));
    }
}
```

### 4. 中间件系统设计

**位置**: `src/tools/middleware.rs`

#### 4.1 架构设计

基于 Tower-rs 风格的责任链模式:

```rust
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput>;
}

pub struct MiddlewareStack {
    middlewares: Vec<Arc<dyn Middleware>>,
}
```

#### 4.2 内置中间件

| 中间件 | 用途 | 复杂度 |
|--------|------|--------|
| `LoggingMiddleware` | 执行日志记录 | 低 |
| `TimingMiddleware` | 执行时间统计 | 低 |
| `RetryMiddleware` | 失败重试 | 中 |
| `TimeoutMiddleware` | 超时控制 | 低 |
| `CircuitBreakerMiddleware` | 熔断保护 | 中 |
| `MetricsMiddleware` | 指标收集 | 低 |
| `CacheMiddleware` | 结果缓存 | 中 |

#### 4.3 中间件链执行流程

```
Request
  ↓
LoggingMiddleware
  ↓
TimingMiddleware
  ↓
RetryMiddleware
  ↓
TimeoutMiddleware
  ↓
Actual Tool Execution
  ↓
Response (back through chain)
```

**代码质量**: 优秀的实现，包含完整的测试覆盖。

### 5. 可组合工具设计

**位置**: `src/tools/composable.rs`

支持三种组合模式:
- **Chain**: 顺序执行
- **Conditional**: 条件分支
- **Parallel**: 并行执行

**设计理念**: 工具可以组合成更复杂的工具，保持统一的 `Tool` 接口。

---

## 插件系统设计分析

### 1. 插件架构

**位置**: `src/plugins/mod.rs`

```rust
pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    fn get_tools(&self) -> Vec<Tool>;  // 迁移到新系统
    fn shutdown(&mut self) -> Result<()>;
    fn is_initialized(&self) -> bool;
    fn status(&self) -> PluginStatus;
}
```

### 2. 插件类型支持

| 类型 | 状态 | 实现位置 | 隔离级别 |
|------|------|----------|----------|
| **Native** | 已实现 | `src/plugins/native.rs` | 进程内 (动态库) |
| **Python** | 已实现 | `src/plugins/python.rs` | 子进程 |
| **Node.js** | 已实现 | `src/plugins/nodejs.rs` | 子进程 |
| **Docker** | 已实现 | `src/plugins/docker.rs` | 容器 |
| **WASM** | ⚠️ 禁用 | `src/plugins/wasm.rs` | 沙箱 |

**WASM 禁用原因** (src/plugins/mod.rs:34):
```rust
// pub mod wasm;  // Temporarily disabled due to wasmtime/extism dependency issues
```

### 3. 安全配置

**位置**: `src/plugins/types.rs:94-114`

```rust
pub struct SecurityPolicy {
    pub allow_network_access: bool,
    pub allow_file_system_access: bool,
    pub allowed_paths: Vec<PathBuf>,
    pub environment_variables: HashMap<String, String>,
    pub sandbox_enabled: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allow_network_access: false,
            allow_file_system_access: false,
            allowed_paths: Vec::new(),
            environment_variables: HashMap::new(),
            sandbox_enabled: true,  // 默认启用沙箱
        }
    }
}
```

**资源限制** (src/plugins/types.rs:116-136):
```rust
pub struct ResourceLimits {
    pub max_memory: Option<u64>,        // 默认 1GB
    pub max_cpu_time: Option<Duration>, // 默认 5分钟
    pub max_execution_time: Option<Duration>, // 默认 10分钟
    pub max_file_size: Option<u64>,     // 默认 100MB
    pub max_network_connections: Option<u32>, // 默认 10
}
```

### 4. 插件包装器模式

**设计模式**: 使用宏生成重复代码

```rust
// src/plugins/types.rs:186-216
impl_plugin_wrapper!(NativePlugin, crate::plugins::native::NativePlugin);
impl_plugin_wrapper!(PythonPlugin, crate::plugins::python::PythonPlugin);
impl_plugin_wrapper!(NodeJsPlugin, crate::plugins::nodejs::NodeJsPlugin);
impl_plugin_wrapper!(DockerPlugin, crate::plugins::docker::DockerPlugin);
```

**评价**: 聪明地消除代码重复，但增加了编译时复杂性。

### 5. 运行时管理

**位置**: `src/plugins/runtime.rs`

- 进程池管理
- 资源统计
- 生命周期管理

---

## 工作流执行系统分析

### 1. 核心引擎

**位置**: `src/workflow/engine.rs`

#### 1.1 引擎架构

```rust
pub struct RefactoredWorkflowEngine {
    state_manager: Arc<StateManager>,
    tool_registry: Arc<dyn ToolRegistry>,
    executor: BoxedExecutor,          // 执行器链
    audit_logger: Arc<AuditLogger>,
    result_cache: Option<Arc<ResultCache>>,
    workflow_semaphore: Arc<Semaphore>,
    default_max_concurrency: usize,
    checkpoint_interval: Duration,
}
```

#### 1.2 执行流程

```
1. Validation (工作流验证)
   ↓
2. Component Registry (组件注册)
   ↓
3. Data Context Init (数据上下文初始化)
   ↓
4. Execution Tracker (执行追踪器)
   ↓
5. DagScheduler (DAG 调度器)
   ↓
6. Checkpoint Manager (检查点管理器)
   ↓
7. Main Execution Loop (主执行循环)
```

### 2. DAG 调度器

**位置**: `src/workflow/scheduler.rs`

**关键算法**: 拓扑排序
- 复杂度: O(V + E)
- V: 节点数, E: 边数

**并行执行支持**:
```rust
pub fn get_parallel_executable_nodes(&mut self, max_concurrency: Option<usize>) -> Vec<String> {
    // 获取所有入度为 0 的节点，可以同时执行
}
```

### 3. 组件系统 (LiteFlow 风格)

**位置**: `src/workflow/component/mod.rs`

#### 3.1 组件 trait

```rust
pub trait Component: Send + Sync {
    fn id(&self) -> &str;
    fn component_type(&self) -> ComponentType;
    async fn execute(&self, context: &mut DataContext) -> Result<ComponentOutput>;
}
```

#### 3.2 组件类型

| 类型 | 描述 | 实现 |
|------|------|------|
| **ToolComponent** | 工具执行 | `src/workflow/component/tool.rs` |
| **ParallelComponent** | 并行执行 | `src/workflow/component/parallel.rs` |

### 4. 执行器链

**位置**: `src/workflow/executor/mod.rs`

```rust
pub trait Executor: Send + Sync {
    async fn execute(&self, component: &dyn Component, context: &mut DataContext, ctx: &ExecutionContext) -> Result<ComponentOutput>;
}
```

**内置执行器**:
- `BasicExecutor`: 基础执行
- `RetryExecutor`: 重试逻辑
- `CacheExecutor`: 缓存结果
- `AuditExecutor`: 审计日志

### 5. 状态管理

**位置**: `src/workflow/state/mod.rs`

**功能**:
- 执行追踪 (ExecutionTracker)
- 检查点管理 (CheckpointManager)
- 控制信号 (pause/resume/stop)

### 6. 错误处理

**位置**: `src/workflow/error_handler.rs`

```rust
pub enum ErrorHandlingStrategy {
    FailWorkflow,      // 终止工作流
    Continue,          // 继续执行
    Retry { max_attempts: u32, delay: Duration }, // 重试
    Fallback(String),  // 使用备用工具
}
```

---

## 类型系统与序列化分析

### 1. 工具输入/输出类型

**位置**: `src/tools/types.rs:79-140`

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInput {
    pub params: Value,        // JSON 参数
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolOutput {
    pub result: Value,        // JSON 结果
    pub metadata: Option<Value>,
    pub success: bool,        // 执行状态
}
```

**评价**: 使用 `serde_json::Value` 作为通用容器，灵活但失去类型安全。

### 2. 强类型转换 trait

**位置**: `src/tools/types.rs:863-886`

```rust
pub trait ToolInputConvert: Sized {
    fn into_tool_input(self) -> ToolInput;
    fn from_tool_input(input: &ToolInput) -> crate::Result<Self>;
    fn validate(&self) -> crate::Result<()>;
    fn schema() -> InputSchema;
}

pub trait ToolOutputConvert: Sized {
    fn into_tool_output(self) -> ToolOutput;
    fn from_tool_output(output: &ToolOutput) -> crate::Result<Self>;
}
```

**宏支持**:
```rust
// workflow-toolkit-macros 提供自动派生
#[cfg(feature = "macros")]
pub use workflow_toolkit_macros::{ToolInput, ToolOutput};
```

### 3. 错误处理设计

**位置**: `src/error/mod.rs`, `src/error.rs`

使用 `thiserror` + `anyhow` 组合:
- `thiserror`: 库代码定义错误类型
- `anyhow`: 应用代码简化错误处理

**错误类型** (src/error.rs):
```rust
pub enum WorkflowError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Execution cancelled")]
    ExecutionCancelled,
    
    #[error("Workflow execution failed: {message}")]
    WorkflowExecution { message: String },
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    // ... 更多错误类型
}
```

---

## 设计模式与最佳实践

### 1. 成功应用的设计模式

| 模式 | 应用位置 | 评价 |
|------|----------|------|
| **Builder Pattern** | `NativeToolBuilder`, `ToolRegistryBuilder` | 清晰流畅的 API |
| **Strategy Pattern** | Middleware 系统 | 灵活的扩展机制 |
| **Chain of Responsibility** | ExecutorChain | AOP 风格横切关注点 |
| **Registry Pattern** | ToolRegistry, ComponentRegistry | 统一管理 |
| **Wrapper Pattern** | 插件包装器 | 代码复用 |
| **Newtype Pattern** | ToolId(u64) | 类型安全 |

### 2. Rust 最佳实践

#### 2.1 内存管理
- 大量使用 `Arc<...>` 共享所有权
- `DashMap` 用于并发数据结构
- 避免 `Mutex`，优先使用消息传递

#### 2.2 异步编程
- 全程使用 `async/await`
- `tokio` 作为异步运行时
- `async-trait` 用于 trait 方法

#### 2.3 错误处理
- 严格避免 `unwrap()`/`expect()` (设计原则文档声明)
- `Result` 类型贯穿始终
- 详细的错误上下文

### 3. 代码组织

**优秀实践**:
- ✅ 清晰的模块划分
- ✅ 每个模块有 design.md 文档
- ✅ 模块 mod.rs 有详细注释
- ✅ 测试代码内聚在模块中

---

## 发现的问题与风险

### 🔴 严重问题 (Critical)

#### 1. Python/Node.js/Docker 工具未实现

**位置**: `src/tools/types.rs:316-334, 369-386, 421-437`

```rust
// PythonTool::execute()
pub async fn execute(&self, input: ToolInput, _ctx: ExecutionContext) -> Result<ToolOutput> {
    // TODO: Implement actual Python execution
    // For now, return a placeholder
    Ok(ToolOutput::success(serde_json::json!({
        "status": "executed",
        "tool": "python",
        "script": self.script_path.to_string_lossy(),
        "input": input.params
    })))
}
```

**影响**: 这些工具类型完全无法工作，只是返回占位符。  
**建议**: 优先实现这些执行逻辑。

#### 2. WASM 插件禁用

**位置**: `src/plugins/mod.rs:34`

```rust
// pub mod wasm;  // Temporarily disabled due to wasmtime/extism dependency issues
```

**影响**: WASM 沙箱插件功能不可用。  
**建议**: 解决依赖问题或提供替代方案。

### 🟡 中等问题 (Major)

#### 3. 兼容层代码过多

**位置**: `src/tools/compat.rs`

新旧系统同时存在，增加了维护复杂性。

**迁移状态** (src/tools/mod.rs:6-16):
```
当前状态: 旧 trait 系统与新 enum 系统共存
- compat 模块保留旧系统代码
- 文档声明 6 个月后移除
```

**建议**: 加速迁移，尽早移除兼容层。

#### 4. 缓存中间件未完成

**位置**: `src/tools/middleware.rs:746-760`

```rust
#[async_trait]
impl Middleware for CacheMiddleware {
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        // For now, just pass through - full cache implementation would
        // require integration with the storage/cache module
        // TODO: Integrate with moka cache from performance module
        debug!("Cache middleware (pass-through - full implementation pending)");
        next.run(ctx).await
    }
}
```

#### 5. 工具输入/输出使用 `serde_json::Value`

**位置**: `src/tools/types.rs:79-140`

使用 `Value` 作为通用容器，运行时才能发现类型错误。

**建议**: 推广 `ToolInputConvert`/`ToolOutputConvert` trait 的使用。

### 🟢 轻微问题 (Minor)

#### 6. 部分文档为中文，部分为英文

虽然不影响功能，但降低了国际化程度。

#### 7. Circuit Breaker 使用 `std::sync::Mutex`

**位置**: `src/tools/middleware.rs:578-621`

```rust
pub struct CircuitBreakerMiddleware {
    last_failure: Arc<std::sync::Mutex<Option<Instant>>>,  // 应该用 parking_lot::Mutex
}
```

建议使用 `parking_lot::Mutex` 获得更好的性能。

---

## 改进建议

### 短期 (1-2 周)

1. **实现 Python/Node.js/Docker 工具执行**
   - 使用 `tokio::process::Command` 启动子进程
   - 实现 JSON-RPC 风格的通信协议
   - 添加超时和信号处理

2. **修复 CacheMiddleware**
   - 集成 `moka` 缓存 (已在依赖中)
   - 实现缓存键生成策略
   - 支持 TTL 和驱逐策略

3. **替换 `std::sync::Mutex`**
   - 全局替换为 `parking_lot::Mutex`
   - 提升并发性能

### 中期 (1-2 月)

4. **加速移除兼容层**
   - 统计 compat 模块使用情况
   - 逐步迁移调用点
   - 最终删除 compat 模块

5. **强化类型安全**
   - 推广强类型工具 trait
   - 提供宏自动生成转换代码
   - 示例和文档更新

6. **完善错误处理**
   - 统一错误分类
   - 添加错误恢复建议
   - 实现错误链追踪

### 长期 (3-6 月)

7. **WASM 插件恢复**
   - 调研 wasmtime 替代方案
   - 或提供预编译插件方案

8. **性能优化**
   - 零拷贝优化
   - 内存池管理
   - 并发度调优

9. **测试增强**
   - E2E 测试覆盖
   - 性能基准测试
   - 混沌测试

---

## 总结

### 优势总结

1. **架构先进**: DDD + LiteFlow 风格，模块化程度高
2. **技术选型优秀**: Tokio、DashMap、serde 等现代化 Rust 生态
3. **设计模式应用得当**: Builder、Strategy、Chain of Responsibility 等
4. **安全性考虑充分**: 沙箱、资源限制、安全配置
5. **扩展性强**: 插件系统、中间件系统、组件系统

### 风险总结

1. **功能不完整**: Python/Node.js/Docker 工具未实现，WASM 禁用
2. **技术债务**: 兼容层代码较多，需要清理
3. **类型安全**: 大量使用 `serde_json::Value`，编译期检查不足

### 推荐行动

| 优先级 | 行动项 | 预计工作量 |
|--------|--------|------------|
| 🔴 P0 | 实现 Python/Node.js/Docker 工具 | 1 周 |
| 🔴 P0 | 修复 CacheMiddleware | 2 天 |
| 🟡 P1 | 移除兼容层 | 2 周 |
| 🟡 P1 | 强化类型安全 | 2 周 |
| 🟢 P2 | WASM 插件恢复 | 1 月 |
| 🟢 P2 | 性能优化 | 1 月 |

---

**审查完成时间**: 2026-02-08  
**审查人**: Prometheus (AI Assistant)  
**下次审查建议**: 3 个月后

---

## 附录

### A. 关键文件索引

| 组件 | 关键文件 | 行数 |
|------|----------|------|
| 工具类型 | `src/tools/types.rs` | 907 |
| 工具注册表 | `src/tools/registry.rs` | 492 |
| 中间件系统 | `src/tools/middleware.rs` | 903 |
| 插件 trait | `src/plugins/types.rs` | 347 |
| 工作流引擎 | `src/workflow/engine.rs` | 765 |
| 错误处理 | `src/error.rs` | ~100 |
| 主库 | `src/lib.rs` | 188 |

### B. 依赖分析

```toml
# 核心异步
[dependencies]
tokio = { version = "1.42", features = ["full"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# CLI/TUI
clap = { version = "4.5", features = ["derive", "env"] }
ratatui = "0.29"

# 并发/性能
dashmap = "6.1"
parking_lot = "0.12"
moka = { version = "0.12", features = ["future"] }

# MCP 协议 (可选)
rmcp = { version = "0.14", optional = true }

# 数据库 (可选)
lancedb = { version = "0.20", optional = true }
```

### C. 测试覆盖

| 测试类型 | 覆盖度 | 位置 |
|----------|--------|------|
| 单元测试 | 中等 | 模块内 `#[cfg(test)]` |
| 集成测试 | 中等 | `tests/` 目录 |
| 属性测试 | 部分 | `proptest` 使用 |
| E2E 测试 | 缺失 | 需要补充 |

---

*报告结束*
