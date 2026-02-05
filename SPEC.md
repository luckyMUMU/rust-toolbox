# Workflow Toolkit 软件约束规范 (SPEC)

> **版本**: v1.0.0  
> **日期**: 2026-02-04  
> **状态**: 基于代码库分析生成

---

## 1. 架构约束 (Architecture Constraints)

### 1.1 分层架构规范

```
┌─────────────────────────────────────────────┐
│  接入层 (Interface Layer)                    │
│  - CLI: clap 派生宏实现                       │
│  - TUI: ratatui 异步事件循环                  │
│  - MCP: rmcp 协议实现                        │
├─────────────────────────────────────────────┤
│  应用层 (Application Layer)                  │
│  - UseCase: 业务流程编排                      │
│  - Service: 领域服务封装                      │
│  - Workflow: 工作流编排器                     │
├─────────────────────────────────────────────┤
│  领域层 (Domain Layer)                       │
│  - Model: 实体、值对象、聚合根                │
│  - Port: 仓库接口、工具注册表接口             │
├─────────────────────────────────────────────┤
│  基础设施层 (Infrastructure Layer)           │
│  - Persistence: 存储实现                      │
│  - Plugin: 插件系统实现                       │
│  - External: 外部服务客户端                   │
├─────────────────────────────────────────────┤
│  适配器层 (Adapter Layer)                    │
│  - DTO: 数据传输对象                          │
│  - 接口适配: CLI/TUI/MCP 适配器               │
└─────────────────────────────────────────────┘
```

**约束规则**:
- AC-001: 严禁跨层调用，必须通过公开接口
- AC-002: 领域层不得依赖其他层
- AC-003: 基础设施层通过 Port 接口与领域层交互

### 1.2 模块依赖方向

```rust
// 允许: 上层依赖下层
src/interfaces/cli/ -> src/application/ -> src/domain/

// 禁止: 下层依赖上层
src/domain/ -> X -> src/application/

// 禁止: 同层跨模块直接依赖
src/plugins/ -> X -> src/workflow/ (必须通过 domain/port)
```

---

## 2. 接口契约 (Interface Contracts)

### 2.1 工作流引擎接口

```rust
/// 工作流引擎核心接口
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 执行工作流定义
    /// 
    /// # 前置条件
    /// - definition 必须通过验证
    /// - initial_params 必须符合输入 Schema
    /// 
    /// # 后置条件
    /// - 返回的 Execution 包含完整执行记录
    /// - 状态必须是终态 (Completed/Failed/Cancelled/Timeout)
    async fn execute(
        &self,
        definition: WorkflowDefinition,
        initial_params: HashMap<String, Value>,
    ) -> Result<WorkflowExecution>;
}

/// 约束:
/// - CE-001: execute 必须是幂等的（相同输入产生相同结果）
/// - CE-002: 并发执行数不得超过 max_concurrent_workflows
/// - CE-003: 执行超时后必须返回 Timeout 状态
```

### 2.2 工具接口

```rust
/// 工具执行接口 (枚举实现，零成本抽象)
pub enum Tool {
    Native(Arc<NativeTool>),
    Python(Arc<PythonTool>),
    NodeJs(Arc<NodeJsTool>),
    Docker(Arc<DockerTool>),
    Wasm(Arc<WasmTool>),
    Composed(Arc<ComposedTool>),
}

impl Tool {
    /// 执行工具
    /// 
    /// # 前置条件
    /// - input.params 必须符合工具定义的 input_schema
    /// - ctx 必须包含有效的 execution_id
    /// 
    /// # 后置条件
    /// - 成功时 output.success == true
    /// - 失败时 output.success == false，且 result 包含错误信息
    pub fn execute(
        &self,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> BoxFuture<'_, Result<ToolOutput>>;
}

/// 约束:
/// - CT-001: execute 必须是线程安全的 (Send + Sync)
/// - CT-002: 执行时间不得超过 timeout_secs
/// - CT-003: 资源使用不得超过 ResourceRequirements 定义
```

### 2.3 组件接口

```rust
/// 工作流组件接口 (LiteFlow 风格)
#[async_trait]
pub trait Component: Send + Sync {
    /// 组件唯一标识
    fn id(&self) -> &str;

    /// 核心执行逻辑
    async fn execute(&self, ctx: &mut DataContext) -> Result<ComponentOutput>;

    /// 准入判断
    async fn is_access(&self, ctx: &DataContext) -> bool {
        true
    }

    /// 异常回滚
    async fn rollback(&self, ctx: &DataContext) -> Result<()> {
        Ok(())
    }
}

/// 约束:
/// - CC-001: id 必须在同工作流内唯一
/// - CC-002: execute 必须是可重入的
/// - CC-003: rollback 必须在 execute 失败后调用
```

---

## 3. 数据模型约束 (Data Model Constraints)

### 3.1 工作流定义

```rust
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

/// 约束:
/// - CD-001: id 必须符合 ^[a-zA-Z][a-zA-Z0-9_-]*$ 模式
/// - CD-002: nodes 不能为空
/// - CD-003: edges 不能形成环 (DAG 约束)
/// - CD-004: 所有 edge.source 和 edge.target 必须对应存在的 node.id
```

### 3.2 执行状态

```rust
pub enum ExecutionStatus {
    Pending,    // 初始状态
    Running,    // 执行中
    Paused,     // 暂停
    Completed,  // 成功完成
    Failed,     // 失败
    Cancelled,  // 取消
    Timeout,    // 超时
}

/// 状态转换约束:
/// - CS-001: Pending -> Running | Cancelled
/// - CS-002: Running -> Paused | Completed | Failed | Cancelled | Timeout
/// - CS-003: Paused -> Running | Cancelled
/// - CS-004: Completed, Failed, Cancelled, Timeout 为终态，不可转换
```

### 3.3 执行上下文

```rust
pub struct ExecutionContext {
    pub execution_id: String,        // 必填，UUID v4 格式
    pub workflow_id: Option<Uuid>,   // 可选
    pub user_id: Option<String>,     // 可选
    pub global_variables: HashMap<String, Value>,
    pub step_results: HashMap<String, Value>,
}

/// 约束:
/// - CEC-001: execution_id 必须是有效的 UUID
/// - CEC-002: global_variables 键名不能包含 '.' 或 '$'
```

---

## 4. 行为约束 (Behavioral Constraints)

### 4.1 执行器链行为

```rust
/// 执行器链顺序 (责任链模式)
pub struct ExecutorChain {
    audit: AuditExecutor,      // 1. 记录审计日志
    cache: CacheExecutor,      // 2. 检查缓存
    retry: RetryExecutor,      // 3. 重试逻辑
    basic: BasicExecutor,      // 4. 实际执行
}

/// 约束:
/// - CB-001: 执行器必须按顺序调用
/// - CB-002: 任一执行器失败应终止链
/// - CB-003: AuditExecutor 必须在最外层 (保证记录完整)
```

### 4.2 并行执行行为

```rust
/// 并行节点执行
async fn execute_parallel(nodes: &[FlowNode], ctx: ExecutionContext) -> Result<()> {
    // 使用 tokio::spawn 实现真正并行
    let handles: Vec<JoinHandle<Result<()>>> = nodes
        .iter()
        .map(|node| {
            let node = node.clone();
            let ctx = ctx.clone();
            tokio::spawn(async move {
                execute(node, ctx).await
            })
        })
        .collect();
    
    // 等待所有完成
    let results = join_all(handles).await;
    // 任一失败则整体失败
}

/// 约束:
/// - CP-001: 并行任务必须使用 tokio::spawn
/// - CP-002: 默认最大并发数: 4
/// - CP-003: 任一失败应取消其他任务 (可配置)
```

### 4.3 数据槽行为

```rust
pub struct DataContext {
    slots: Arc<DashMap<String, SlotValue>>,
}

/// 约束:
/// - CDS-001: 槽名区分大小写
/// - CDS-002: 槽值必须是 JSON 可序列化的
/// - CDS-003: 并发读写必须是线程安全的
```

---

## 5. 性能约束 (Performance Constraints)

### 5.1 响应时间

| 操作 | 目标 | 最大容忍 |
|------|------|----------|
| 工作流提交 | < 10ms | 100ms |
| 节点调度 | < 1ms | 10ms |
| 工具执行 | 取决于工具 | 可配置 timeout |
| 状态查询 | < 5ms | 50ms |

### 5.2 资源限制

```rust
pub struct ResourceRequirements {
    pub min_memory_mb: u64,           // 默认: 64
    pub recommended_memory_mb: u64,   // 默认: 256
    pub cpu_intensity: u8,            // 1-10，默认: 5
    pub network_required: bool,       // 默认: false
    pub estimated_duration_ms: u64,   // 默认: 1000
}

/// 约束:
/// - CPR-001: 内存使用不得超过 min_memory_mb * 2
/// - CPR-002: CPU 密集型任务 (intensity > 7) 应限制并发数
```

### 5.3 并发约束

```rust
pub struct ConcurrencyConfig {
    pub max_concurrent_workflows: usize,  // 默认: 10
    pub max_concurrent_tasks: usize,      // 默认: 50
    pub task_queue_size: usize,           // 默认: 100
}

/// 约束:
/// - CPC-001: 超过 max_concurrent_workflows 应返回 Backpressure 错误
/// - CPC-002: 任务队列满时应触发背压机制
```

---

## 6. 错误处理约束 (Error Handling Constraints)

### 6.1 错误类型规范

```rust
pub enum WorkflowError {
    // 配置错误
    Config(config::ConfigError),
    
    // 验证错误
    WorkflowValidation { message: String },
    ValidationError(String),
    
    // 执行错误
    WorkflowExecution { message: String },
    ExecutionCancelled,
    ExecutionTimeout,
    
    // 资源错误
    NotFound { resource: String },
    ResourceExhausted,
    
    // 并发错误
    ConcurrentAccess { message: String },
    
    // 通用错误
    Generic(anyhow::Error),
}

/// 约束:
/// - CEH-001: 严禁使用 unwrap/expect (除测试代码外)
/// - CEH-002: 所有错误必须包含上下文信息
/// - CEH-003: 用户-facing 错误消息必须是中文
```

### 6.2 错误传播

```rust
/// 约束:
/// - CEP-001: 使用 ? 运算符传播错误
/// - CEP-002: 转换错误类型时使用 #[from] 或 map_err
/// - CEP-003: 在边界处 (API/接口) 统一错误格式
```

---

## 7. 安全约束 (Security Constraints)

### 7.1 输入验证

```rust
/// 约束:
/// - CSI-001: 所有外部输入必须验证
/// - CSI-002: 字符串输入必须限制长度 (默认 10KB)
/// - CSI-003: 文件路径必须规范化 (防止目录遍历)
```

### 7.2 插件沙箱

```rust
/// 约束:
/// - CSP-001: 插件必须在沙箱中执行
/// - CSP-002: 插件资源使用必须受限 (CPU/内存)
/// - CSP-003: 插件网络访问默认关闭
```

---

## 8. 可观测性约束 (Observability Constraints)

### 8.1 日志规范

```rust
/// 约束:
/// - COL-001: 使用 tracing  crate 进行结构化日志
/// - COL-002: 日志级别: ERROR > WARN > INFO > DEBUG > TRACE
/// - COL-003: 关键操作必须记录: workflow_id, node_id, duration
```

### 8.2 指标规范

```rust
/// 约束:
/// - COM-001: 核心指标: 执行次数、延迟、成功率
/// - COM-002: 资源指标: CPU、内存、磁盘、网络
/// - COM-003: 业务指标: 队列深度、并发数
```

---

## 9. 版本兼容性约束 (Version Constraints)

### 9.1 API 版本

```rust
/// 约束:
/// - CV-001: 公共 API 必须保持向后兼容
/// - CV-002: 破坏性变更必须增加主版本号
/// - CV-003: 废弃 API 必须标记 #[deprecated] 并保留至少一个版本
```

### 9.2 数据版本

```rust
/// 约束:
/// - CVD-001: 序列化数据必须包含版本字段
/// - CVD-002: 支持至少两个版本的向后兼容读取
```

---

## 10. 测试约束 (Testing Constraints)

### 10.1 测试覆盖

```rust
/// 约束:
/// - CT-001: 核心模块覆盖率 >= 80%
/// - CT-002: 公共 API 必须有单元测试
/// - CT-003: 工作流执行路径必须有集成测试
```

### 10.2 测试规范

```rust
/// 约束:
/// - CTS-001: 测试名称必须描述行为: test_<scenario>_<expected_result>
/// - CTS-002: 使用 tempfile 创建临时资源
/// - CTS-003: 异步测试使用 #[tokio::test]
```

---

## 附录 A: 约束验证清单

### A.1 代码审查清单

- [ ] 是否违反分层架构 (AC-001, AC-002, AC-003)
- [ ] 是否存在 unwrap/expect (CEH-001)
- [ ] 错误消息是否为中文 (CEH-003)
- [ ] 公共 API 是否有文档
- [ ] 是否包含必要的 tracing 日志

### A.2 性能审查清单

- [ ] 是否使用 tokio::spawn 实现并行 (CP-001)
- [ ] 是否有资源限制检查 (CPR-001)
- [ ] 是否有背压机制 (CPC-001)

### A.3 安全审查清单

- [ ] 外部输入是否验证 (CSI-001)
- [ ] 文件路径是否规范化 (CSI-003)
- [ ] 插件是否沙箱化 (CSP-001)

---

## 附录 B: 术语表

| 术语 | 定义 | 代码引用 |
|------|------|----------|
| Workflow | 一组有序的任务集合 | `WorkflowDefinition` |
| Tool | 可执行的最小单元 | `Tool` enum |
| Plugin | 一组工具的集合 | `PluginInfo` |
| Execution | 工作流的一次运行实例 | `WorkflowExecution` |
| Component | 工作流节点组件 | `Component` trait |
| Data Slot | 数据交换上下文 | `DataContext` |
| Executor Chain | 执行器责任链 | `Executor` trait |

---

**生成信息**:
- 分析日期: 2026-02-04
- 代码版本: v0.1.0
- 基于: PRODUCT_DESIGN.md + design_v2.1.md + 代码库分析
