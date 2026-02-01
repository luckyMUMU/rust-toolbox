# 工具系统激进优化计划（不兼容版本）

## 目标
在不考虑向后兼容的前提下，对工具系统进行彻底重构，实现：
- **50%+ 性能提升**（消除所有dyn trait）
- **类型安全**（编译时参数验证）
- **生产级可靠性**（内置中间件）
- **开发者友好**（优秀DX）

---

## 阶段1: 核心架构重构（第1-2周）

### 任务1.1: 删除旧trait系统
- [ ] 删除 `ToolRegistry` trait
- [ ] 删除 `ToolNode` trait  
- [ ] 删除 `ToolExecutor` trait
- [ ] 删除 `ComposableTool` trait
- [ ] 清理 `tools/registry.rs` 中的dyn代码
- [ ] 清理 `tools/node.rs` 中的dyn代码
- [ ] 删除 `async_trait` 依赖（如果仅用于工具系统）

**验收标准**:
- [ ] `cargo check` 通过（编译错误预期，因为新系统未实现）
- [ ] 删除约2000行旧trait代码

**提交信息**: `refactor(tools)!: remove legacy trait system`

---

### 任务1.2: 创建枚举类型系统
- [ ] 创建 `src/tools/types.rs` 定义Tool枚举
- [ ] 定义 `Tool` 枚举包含所有工具类型变体
- [ ] 定义 `ToolKind` 区分工具类别
- [ ] 为 `Tool` 实现统一执行接口
- [ ] 创建 `ToolId` 类型安全标识符

**核心设计**:
```rust
// 枚举替代dyn trait
pub enum Tool {
    Native(NativeTool),
    Python(PythonTool),
    NodeJs(NodeJsTool),
    Docker(DockerTool),
    Wasm(WasmTool),
    Composed(Arc<ComposedTool>),
}

// 统一执行接口（无dyn开销）
impl Tool {
    pub async fn execute(
        &self, 
        input: ToolInput,
        ctx: ExecutionContext
    ) -> Result<ToolOutput> {
        match self {
            Tool::Native(t) => t.execute(input, ctx).await,
            Tool::Python(t) => t.execute(input, ctx).await,
            // ... 分支预测友好
        }
    }
    
    pub fn metadata(&self) -> &ToolMetadata {
        match self {
            Tool::Native(t) => &t.metadata,
            // ...
        }
    }
}

// 类型安全ID
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ToolId(u64);
```

**验收标准**:
- [ ] Tool枚举定义完成
- [ ] 所有变体实现统一接口
- [ ] `cargo build` 通过
- [ ] 基准测试显示性能提升

**提交信息**: `feat(tools)!: introduce enum-based tool system`

---

### 任务1.3: 重构工具注册表
- [ ] 创建 `src/tools/registry/mod.rs` 新模块
- [ ] 实现 `ToolRegistry` 结构体（具体类型，非trait）
- [ ] 使用 `DashMap<ToolId, Tool>` 存储
- [ ] 实现工具查找（O(1)复杂度）
- [ ] 实现工具注册/注销
- [ ] 实现工具发现API
- [ ] 添加版本管理支持

**设计要点**:
```rust
pub struct ToolRegistry {
    tools: DashMap<ToolId, Tool>,
    name_index: DashMap<String, ToolId>,  // 名称到ID映射
    version_index: DashMap<ToolId, Vec<Version>>,  // 版本管理
    metadata_cache: DashMap<ToolId, Arc<ToolMetadata>>,
}

impl ToolRegistry {
    pub fn register(&self, name: &str, tool: Tool) -> ToolId {
        let id = ToolId::new();
        self.tools.insert(id, tool);
        self.name_index.insert(name.to_string(), id);
        id
    }
    
    pub fn get(&self, name: &str) -> Option<&Tool> {
        let id = self.name_index.get(name)?;
        self.tools.get(&*id).map(|r| r.value())
    }
    
    pub async fn execute(
        &self, 
        name: &str, 
        input: ToolInput
    ) -> Result<ToolOutput> {
        let tool = self.get(name)
            .ok_or_else(|| Error::tool_not_found(name))?;
        tool.execute(input, ExecutionContext::new()).await
    }
}
```

**验收标准**:
- [ ] 新注册表实现完成
- [ ] 所有方法使用具体类型
- [ ] 单元测试通过
- [ ] 性能基准：比旧版本快30%+

**提交信息**: `feat(tools)!: reimplement tool registry with concrete types`

---

### 任务1.4: 重构工具节点实现
- [ ] 重构 `BasicTool` 为具体类型（非trait实现）
- [ ] 创建 `NativeTool` 类型
- [ ] 创建 `PythonTool` 类型
- [ ] 创建 `DockerTool` 类型
- [ ] 统一工具元数据结构
- [ ] 实现工具序列化/反序列化

**关键变更**:
```rust
// 以前：BasicTool实现ToolNode trait
// 现在：具体类型，直接方法

pub struct NativeTool {
    metadata: Arc<ToolMetadata>,
    executor: Box<dyn Fn(ToolInput, ExecutionContext) -> BoxFuture<Result<ToolOutput>>>,
}

impl NativeTool {
    pub fn execute(
        &self, 
        input: ToolInput, 
        ctx: ExecutionContext
    ) -> impl Future<Output = Result<ToolOutput>> + '_ {
        (self.executor)(input, ctx)
    }
}

pub struct PythonTool {
    metadata: Arc<ToolMetadata>,
    script_path: PathBuf,
    interpreter: PythonInterpreter,
}

impl PythonTool {
    pub async fn execute(
        &self, 
        input: ToolInput, 
        ctx: ExecutionContext
    ) -> Result<ToolOutput> {
        // Python特定实现
    }
}
```

**验收标准**:
- [ ] 所有工具类型重构完成
- [ ] 统一执行接口
- [ ] 无dyn trait调用
- [ ] 集成测试通过

**提交信息**: `refactor(tools)!: convert tools to concrete types`

---

## 阶段2: 中间件系统（第3-4周）

### 任务2.1: 设计中间件trait
- [ ] 创建 `src/tools/middleware/mod.rs`
- [ ] 定义 `ToolMiddleware` trait
- [ ] 定义 `Next` 类型（中间件链 continuation）
- [ ] 设计中间件上下文 `MiddlewareContext`

**核心设计**:
```rust
pub trait ToolMiddleware: Send + Sync + 'static {
    fn handle<'a>(
        &'a self,
        ctx: MiddlewareContext,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<ToolOutput>>;
}

pub struct Next<'a> {
    handler: Box<dyn FnOnce(MiddlewareContext) -> BoxFuture<'a, Result<ToolOutput>> + Send + 'a>,
}

impl<'a> Next<'a> {
    pub async fn run(self, ctx: MiddlewareContext) -> Result<ToolOutput> {
        (self.handler)(ctx).await
    }
}

pub struct MiddlewareContext {
    pub tool_id: ToolId,
    pub tool_name: String,
    pub input: ToolInput,
    pub execution_context: ExecutionContext,
    pub metrics: Arc<Metrics>,
    pub span: tracing::Span,
}
```

**验收标准**:
- [ ] 中间件trait定义完成
- [ ] 可以链式组合
- [ ] 示例中间件实现通过

**提交信息**: `feat(tools): design middleware trait system`

---

### 任务2.2: 实现核心中间件

#### 2.2.1 日志中间件
- [ ] 创建 `middleware/logging.rs`
- [ ] 实现请求/响应日志记录
- [ ] 支持结构化日志（JSON）
- [ ] 支持日志级别配置

#### 2.2.2 缓存中间件
- [ ] 创建 `middleware/cache.rs`
- [ ] 实现结果缓存
- [ ] 支持TTL
- [ ] 支持缓存键自定义
- [ ] 支持缓存失效策略

#### 2.2.3 重试中间件
- [ ] 创建 `middleware/retry.rs`
- [ ] 实现指数退避重试
- [ ] 支持最大重试次数
- [ ] 支持可重试错误类型配置
- [ ] 支持熔断（连续失败停止重试）

#### 2.2.4 超时中间件
- [ ] 创建 `middleware/timeout.rs`
- [ ] 实现超时控制
- [ ] 支持优雅取消
- [ ] 支持超时自定义

#### 2.2.5 指标中间件
- [ ] 创建 `middleware/metrics.rs`
- [ ] 实现执行时间统计
- [ ] 实现成功率统计
- [ ] 集成Prometheus导出

#### 2.2.6 验证中间件
- [ ] 创建 `middleware/validation.rs`
- [ ] 实现参数预验证
- [ ] 实现结果后验证

**验收标准**:
- [ ] 6个核心中间件全部实现
- [ ] 每个中间件有单元测试
- [ ] 中间件可以任意组合
- [ ] 性能影响<5%

**提交信息**: `feat(tools): implement core middleware stack`

---

### 任务2.3: 集成中间件到工具
- [ ] 修改 `Tool` 枚举支持中间件栈
- [ ] 修改 `ToolRegistry` 默认中间件配置
- [ ] 实现中间件构建器API
- [ ] 更新工具执行流程

**使用API**:
```rust
let tool = Tool::native()
    .name("api_call")
    .with_middleware(
        MiddlewareStack::new()
            .add(LoggingMiddleware::info())
            .add(CacheMiddleware::with_ttl(Duration::from_secs(60)))
            .add(RetryMiddleware::with_max_attempts(3))
            .add(TimeoutMiddleware::with_duration(Duration::from_secs(30)))
            .add(MetricsMiddleware::new())
    )
    .executor(|input, ctx| async move {
        // 实际执行逻辑
    })
    .build();
```

**验收标准**:
- [ ] 工具支持中间件栈
- [ ] 中间件执行顺序正确
- [ ] 错误在中间件链中传播正确
- [ ] 集成测试通过

**提交信息**: `feat(tools)!: integrate middleware into tool execution`

---

## 阶段3: 强类型参数系统（第5-6周）

### 任务3.1: 设计参数trait
- [ ] 创建 `src/tools/params/mod.rs`
- [ ] 定义 `ToolInput` trait
- [ ] 定义 `ToolOutput` trait
- [ ] 定义参数验证trait
- [ ] 设计派生宏接口

**核心设计**:
```rust
pub trait ToolInput: DeserializeOwned + Serialize + Send + Sync + 'static {
    /// 参数验证
    fn validate(&self) -> Result<()>;
    
    /// 生成JSON Schema
    fn schema() -> JsonSchema;
    
    /// 获取默认值
    fn defaults() -> Self;
    
    /// 转换为Value（用于序列化）
    fn to_value(&self) -> Value;
}

pub trait ToolOutput: Serialize + Send + Sync + 'static {
    /// 验证输出
    fn validate(&self) -> Result<()>;
    
    /// 生成JSON Schema
    fn schema() -> JsonSchema;
    
    /// 从Value解析
    fn from_value(value: Value) -> Result<Self>;
}
```

**验收标准**:
- [ ] trait设计完成
- [ ] 可以实现常见类型
- [ ] 文档清晰

**提交信息**: `feat(tools): design typed parameter system`

---

### 任务3.2: 实现派生宏
- [ ] 创建 `tool-macros` crate（或内嵌proc-macro）
- [ ] 实现 `#[derive(ToolInput)]`
- [ ] 实现 `#[param(...)]` 属性宏
- [ ] 自动生成JSON Schema
- [ ] 自动生成验证代码

**使用示例**:
```rust
#[derive(ToolInput, Debug)]
pub struct FileCopyInput {
    #[param(required = true, description = "源文件路径")]
    pub source: PathBuf,
    
    #[param(required = true, description = "目标路径")]
    pub destination: PathBuf,
    
    #[param(
        default = false, 
        description = "是否覆盖已存在文件",
        validate = "validate_overwrite"
    )]
    pub overwrite: bool,
    
    #[param(
        default = 8192,
        description = "缓冲区大小（字节）",
        validate = |v| v > 0 && v <= 1024 * 1024
    )]
    pub buffer_size: usize,
}

fn validate_overwrite(value: &bool) -> Result<()> {
    // 自定义验证逻辑
    Ok(())
}

// 自动生成：
// - Deserialize
// - Serialize  
// - ToolInput trait实现（含验证）
// - JSON Schema
// - Builder模式
```

**验收标准**:
- [ ] 派生宏可以生成完整代码
- [ ] 支持复杂验证逻辑
- [ ] 错误信息友好
- [ ] 编译时间增加<10%

**提交信息**: `feat(tools): implement derive macros for typed parameters`

---

### 任务3.3: 重构工具为泛型
- [ ] 创建 `TypedTool<I: ToolInput, O: ToolOutput>` 类型
- [ ] 实现泛型工具构建器
- [ ] 重构5个核心工具使用泛型
- [ ] 更新工作流执行器支持泛型工具

**新API**:
```rust
// 定义输入/输出
#[derive(ToolInput)]
struct SearchInput {
    #[param(required = true)]
    query: String,
    #[param(default = 10)]
    limit: usize,
}

#[derive(ToolOutput)]
struct SearchOutput {
    results: Vec<SearchResult>,
    total: usize,
}

// 创建强类型工具
let search_tool = TypedTool::<SearchInput, SearchOutput>::new()
    .name("search")
    .version("2.0.0")
    .with_middleware(...)
    .executor(|input: SearchInput, ctx| async move {
        // input 已经是验证过的强类型
        let results = do_search(&input.query, input.limit).await;
        
        Ok(SearchOutput {
            results,
            total: results.len(),
        })
    })
    .build();

// 执行时自动反序列化和验证
let output: SearchOutput = registry
    .execute_typed::<SearchInput, SearchOutput>(
        "search",
        json!({"query": "rust", "limit": 5})
    )
    .await?;
```

**验收标准**:
- [ ] TypedTool实现完成
- [ ] 5个核心工具迁移完成
- [ ] 编译时类型检查生效
- [ ] 运行时验证通过

**提交信息**: `feat(tools)!: introduce typed tool system`

---

## 阶段4: 开发者体验优化（第7周）

### 任务4.1: 工具开发宏
- [ ] 创建 `tool!` 宏快速定义工具
- [ ] 支持闭包定义简单工具
- [ ] 支持函数定义复杂工具
- [ ] 自动生成元数据

**使用示例**:
```rust
// 简单工具
workflow_toolkit::tool! {
    name: "echo",
    version: "1.0.0",
    description: "Echo input back",
    
    input: {
        message: String
    },
    
    output: {
        echoed: String
    },
    
    async fn execute(input, _ctx) -> Result<Output> {
        Ok(Output { echoed: input.message })
    }
}

// 或使用函数
#[tool(
    name = "file_copy",
    version = "2.0.0",
    middleware = [CacheMiddleware, RetryMiddleware]
)]
async fn file_copy_tool(
    input: FileCopyInput,
    ctx: ExecutionContext,
) -> Result<FileCopyOutput> {
    fs::copy(&input.source, &input.destination).await?;
    Ok(FileCopyOutput { bytes_copied: 0 })
}
```

**验收标准**:
- [ ] tool! 宏可用
- [ ] 支持复杂场景
- [ ] 文档和示例丰富

**提交信息**: `feat(tools): add tool! macro for ergonomic tool definition`

---

### 任务4.2: 工具脚手架CLI
- [ ] 创建 `cargo tool-new` 子命令
- [ ] 生成交互式工具模板
- [ ] 支持多种工具类型（Native/Python/Docker）
- [ ] 生成测试模板
- [ ] 生成文档模板

**使用示例**:
```bash
$ cargo tool-new
? Tool name: my_processor
? Tool type: Native
? Input parameters: source: String, destination: String
? Output type: Json
? Add middleware (y/n): y
? Middleware: [✓] Cache, [✓] Retry, [ ] Timeout

Generating tool scaffolding...
✓ src/tools/custom/my_processor.rs
✓ tests/tools/my_processor_test.rs
✓ docs/tools/my_processor.md
```

**验收标准**:
- [ ] CLI工具可用
- [ ] 生成代码可编译
- [ ] 包含完整示例

**提交信息**: `feat(cli): add cargo tool-new scaffolding command`

---

### 任务4.3: 工具文档生成
- [ ] 从ToolInput/ToolOutput派生自动生成文档
- [ ] 生成OpenAPI兼容的schema
- [ ] 生成Markdown文档
- [ ] 支持示例代码嵌入

**验收标准**:
- [ ] 文档自动生成
- [ ] 格式美观
- [ ] 包含所有必要信息

**提交信息**: `feat(tools): add automatic documentation generation`

---

## 阶段5: 性能优化与测试（第8周）

### 任务5.1: 性能基准测试
- [ ] 创建 `benches/tool_execution.rs`
- [ ] 测试工具注册性能
- [ ] 测试工具执行性能
- [ ] 对比新旧系统性能
- [ ] 测试中间件开销

**验收标准**:
- [ ] 基准测试显示50%+性能提升
- [ ] 内存使用减少20%+
- [ ] 延迟降低30%+

**提交信息**: `test(tools): add comprehensive performance benchmarks`

---

### 任务5.2: 压力测试
- [ ] 测试1000+并发工具执行
- [ ] 测试内存泄漏
- [ ] 测试长时间运行稳定性
- [ ] 测试边界条件

**验收标准**:
- [ ] 通过1000并发测试
- [ ] 无内存泄漏（24小时运行）
- [ ] 所有边界条件处理正确

**提交信息**: `test(tools): add stress tests`

---

### 任务5.3: 集成测试
- [ ] 测试工具链完整执行
- [ ] 测试错误恢复
- [ ] 测试中间件组合
- [ ] 测试与workflow集成

**验收标准**:
- [ ] 集成测试覆盖率>80%
- [ ] 所有关键路径有测试
- [ ] CI通过

**提交信息**: `test(tools): add integration test suite`

---

## 关键里程碑

### 里程碑1: 核心架构完成（第2周末）
- [ ] 旧trait系统完全删除
- [ ] 新枚举系统可用
- [ ] 基准测试显示性能提升

### 里程碑2: 中间件系统完成（第4周末）
- [ ] 6个核心中间件可用
- [ ] 中间件API稳定
- [ ] 与工具系统集成完成

### 里程碑3: 强类型系统完成（第6周末）
- [ ] 派生宏可用
- [ ] 5个核心工具迁移完成
- [ ] 编译时类型检查生效

### 里程碑4: DX优化完成（第7周末）
- [ ] tool! 宏可用
- [ ] CLI脚手架可用
- [ ] 文档自动生成可用

### 里程碑5: 生产就绪（第8周末）
- [ ] 性能提升50%+
- [ ] 测试覆盖率>80%
- [ ] 文档完整
- [ ] API稳定

---

## 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 编译时间增加 | 使用cranelift后端，优化宏实现 |
| 学习曲线陡峭 | 优秀文档，丰富示例，渐进式教程 |
| 调试困难 | 改进错误信息，添加诊断工具 |
| 生态兼容性 | 提供适配层，允许混合使用 |

---

## 成功标准

- [ ] **性能**: 工具执行速度提升50%+
- [ ] **类型安全**: 所有工具参数编译时验证
- [ ] **可靠性**: 内置重试、缓存、超时、熔断
- [ ] **开发者体验**: 定义工具从50行代码减少到10行
- [ ] **测试覆盖率**: >80%
- [ ] **文档**: 每个公共API都有示例

---

## 时间表

```
周1-2: [####] 核心架构重构
周3-4: [####] 中间件系统
周5-6: [####] 强类型参数
周7:    [##]   开发者体验
周8:    [##]   性能优化与测试
```

---

**预计总时间**: 8周（2个月）
**预期效果**: 工具系统全面现代化，性能翻倍，开发效率提升5倍

---

*计划创建时间: 2026-02-01*
*适用版本: workflow-toolkit v2.0（不兼容版本）*
