# 工作流引擎 (Workflow Engine)

## 0. 分层定位

本模块属于**基础设施层**实现，提供工作流引擎的具体技术实现。

### 0.1 与 DDD 分层的关系

```
┌─────────────────────────────────────────┐
│  接口层 (Interfaces)                     │
│  └── 调用 WorkflowEngine trait          │
├─────────────────────────────────────────┤
│  应用层 (Application)                    │
│  ├── WorkflowEngine trait (端口定义)    │
│  └── WorkflowService (编排逻辑)         │
├─────────────────────────────────────────┤
│  领域层 (Domain)                         │
│  ├── WorkflowExecution (聚合根)         │
│  ├── WorkflowDefinition (聚合根)        │
│  └── ExecutionContext (值对象)          │
├─────────────────────────────────────────┤
│  基础设施层 (Infrastructure)             │
│  └── src/workflow/ (本模块)             │
│      ├── 实现 WorkflowEngine trait      │
│      ├── Component 适配领域层 Tool      │
│      ├── DataContext 适配 ExecutionContext│
│      └── 具体技术实现 (调度、执行、检查点)│
└─────────────────────────────────────────┘
```

### 0.2 职责边界

| 层级 | 职责 | 本模块角色 |
|------|------|-----------|
| 领域层 | 定义工作流模型、执行状态 | 消费领域模型 |
| 应用层 | 定义 WorkflowEngine 接口、编排逻辑 | 实现接口 |
| 基础设施层 | 技术实现：调度、执行、持久化 | **本模块** |

### 0.3 依赖关系

```
src/workflow/
├── 依赖: domain (领域层)
│   ├── WorkflowDefinition
│   ├── WorkflowExecution
│   ├── ExecutionContext
│   └── ToolInfo
├── 依赖: application (应用层)
│   └── port::WorkflowEngine (实现此接口)
└── 被依赖: interfaces (接口层)
    └── 通过应用层间接使用
```

---

## 1. 核心定义 (Stable)

### 1.1 模块职责

工作流引擎负责工作流的定义、验证、调度和执行。采用 LiteFlow 架构风格，包含组件系统、执行器链、数据上下文等核心概念。

### 1.2 模块结构

```
workflow/
├── definition.rs       # 工作流定义
├── engine.rs           # 引擎实现
├── execution.rs        # 执行状态管理
├── execution_manager.rs # 执行管理器
├── scheduler.rs        # DAG 调度器
├── validator.rs        # 验证器
├── audit.rs            # 审计日志
├── result_cache.rs     # 结果缓存
├── component/          # 组件系统
│   ├── mod.rs
│   ├── tool.rs         # 工具组件
│   └── parallel.rs     # 并行组件
├── context/            # 数据上下文
│   ├── mod.rs
│   └── slot.rs         # 数据槽
├── executor/           # 执行器链
│   ├── mod.rs
│   ├── basic.rs        # 基础执行器
│   ├── retry.rs        # 重试执行器
│   ├── cache.rs        # 缓存执行器
│   └── audit.rs        # 审计执行器
└── state/              # 状态管理
    ├── mod.rs
    └── checkpoint.rs   # 检查点
```

### 1.3 核心组件

#### 工作流定义

```rust
/// 工作流定义
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub config: WorkflowConfig,
}

/// 工作流节点
pub struct WorkflowNode {
    pub id: String,
    pub node_type: NodeType,
    pub config: NodeConfig,
}

pub enum NodeType {
    Tool(String),       // 工具节点
    Parallel,           // 并行节点
    Condition,          // 条件节点
    SubWorkflow,        // 子工作流
}

/// 工作流边
pub struct WorkflowEdge {
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
}
```

#### 工作流引擎

```rust
/// 工作流引擎 trait
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute(
        &self,
        definition: WorkflowDefinition,
        params: HashMap<String, Value>,
    ) -> Result<WorkflowExecution>;
}

/// 默认引擎实现
pub struct DefaultWorkflowEngine {
    scheduler: Arc<DagScheduler>,
    component_registry: Arc<ComponentRegistry>,
    executor_chain: BoxedExecutor,
}

/// 重构版引擎（LiteFlow 风格）
pub struct RefactoredWorkflowEngine {
    component_registry: Arc<ComponentRegistry>,
    executor_chain: BoxedExecutor,
    checkpoint_manager: Arc<CheckpointManager>,
}
```

#### DAG 调度器

```rust
/// DAG 调度器
pub struct DagScheduler {
    max_concurrent: usize,
}

impl DagScheduler {
    /// 拓扑排序
    pub fn schedule(&self, definition: &WorkflowDefinition) -> Result<SchedulingResult>;
    
    /// 获取可执行节点
    pub fn get_executable_nodes(&self, execution: &WorkflowExecution) -> Vec<String>;
}

/// 调度结果
pub struct SchedulingResult {
    pub execution_order: Vec<Vec<String>>, // 分层执行顺序
    pub stats: ExecutionStats,
}
```

### 1.4 组件系统

```rust
/// 组件 trait
#[async_trait]
pub trait Component: Send + Sync {
    fn get_type(&self) -> ComponentType;
    async fn execute(&self, ctx: &mut DataContext) -> Result<ComponentOutput>;
}

/// 组件类型
pub enum ComponentType {
    Tool,
    Parallel,
    Condition,
    Loop,
}

/// 组件输出
pub struct ComponentOutput {
    pub status: ComponentStatus,
    pub data: Option<Value>,
    pub next_nodes: Vec<String>,
}

/// 组件注册表
pub struct ComponentRegistry {
    components: DashMap<String, Box<dyn Component>>,
}
```

#### Component 与领域层 Tool 的映射

```rust
/// Component 是对领域层 Tool 的包装和适配
/// 
/// 关系映射：
/// - Component::execute() → Tool 的业务逻辑执行
/// - Component::get_type() → ToolInfo::category
/// - DataContext → ExecutionContext 的包装
/// 
/// 架构位置：
/// - ToolInfo (领域层) → ToolComponentAdapter (基础设施层) → Component trait
pub trait Component: Send + Sync {
    fn get_type(&self) -> ComponentType;
    async fn execute(&self, ctx: &mut DataContext) -> Result<ComponentOutput>;
}

/// Component 到 Tool 的适配器
/// 
/// 将领域层的 Tool 包装为工作流引擎的 Component
pub struct ToolComponentAdapter {
    tool_info: ToolInfo,           // 来自领域层
    tool_executor: Box<dyn ToolExecutor>, // 工具执行器
}

impl Component for ToolComponentAdapter {
    fn get_type(&self) -> ComponentType {
        ComponentType::Tool
    }
    
    async fn execute(&self, ctx: &mut DataContext) -> Result<ComponentOutput> {
        // 1. 从 DataContext 提取 ExecutionContext
        let execution_context = ctx.to_execution_context();
        
        // 2. 调用工具执行
        let result = self.tool_executor.execute(
            &self.tool_info,
            ctx.get_input(),
            &execution_context
        ).await?;
        
        // 3. 将结果写回 DataContext
        ctx.set_output(result);
        
        Ok(ComponentOutput {
            status: ComponentStatus::Success,
            data: Some(result),
            next_nodes: vec![], // 由调度器决定
        })
    }
}

/// DataContext 与 ExecutionContext 的映射
/// 
/// DataContext 是 ExecutionContext 的技术包装，添加工作流引擎特定的功能
pub struct DataContext {
    execution_context: ExecutionContext,  // 领域层上下文
    node_inputs: HashMap<String, Value>,  // 当前节点输入
    node_outputs: HashMap<String, Value>, // 当前节点输出
    global_variables: HashMap<String, Value>, // 全局变量
}

impl DataContext {
    /// 转换为领域层 ExecutionContext
    pub fn to_execution_context(&self) -> ExecutionContext {
        ExecutionContext {
            execution_id: self.execution_context.execution_id.clone(),
            workflow_id: self.execution_context.workflow_id,
            user_id: self.execution_context.user_id.clone(),
            global_variables: self.global_variables.clone(),
            step_results: self.node_outputs.clone(),
        }
    }
    
    /// 从 ExecutionContext 创建
    pub fn from_execution_context(ctx: ExecutionContext) -> Self {
        Self {
            execution_context: ctx,
            node_inputs: HashMap::new(),
            node_outputs: HashMap::new(),
            global_variables: HashMap::new(),
        }
    }
}
```

### 1.5 执行器链

```rust
/// 执行器 trait
#[async_trait]
pub trait Executor: Send + Sync {
    async fn execute(&self, ctx: ExecutionContext, next: Next<'_>) -> Result<Value>;
}

/// 基础执行器
pub struct BasicExecutor;

/// 重试执行器
pub struct RetryExecutor {
    retry_policy: RetryPolicy,
    inner: BoxedExecutor,
}

/// 缓存执行器
pub struct CacheExecutor {
    cache: Arc<ResultCache>,
    inner: BoxedExecutor,
}

/// 审计执行器
pub struct AuditExecutor {
    logger: Arc<AuditLogger>,
    inner: BoxedExecutor,
}

/// 执行器链构建器
pub struct ExecutorChainBuilder;
```

### 1.6 并发控制

```rust
/// 并发控制器
/// 
/// 使用信号量控制并行执行的最大数量
pub struct ConcurrencyController {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl ConcurrencyController {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }
    
    /// 获取执行许可
    pub async fn acquire_permit(&self) -> Result<ExecutionPermit> {
        let permit = self.semaphore.acquire().await?;
        Ok(ExecutionPermit { _permit: permit })
    }
    
    /// 获取当前可用许可数
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// 执行许可（RAII 模式）
pub struct ExecutionPermit {
    _permit: SemaphorePermit<'static>,
}

/// 并行执行管理器
/// 
/// 管理同一层节点的并行执行
pub struct ParallelExecutionManager {
    concurrency_controller: ConcurrencyController,
}

impl ParallelExecutionManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            concurrency_controller: ConcurrencyController::new(max_concurrent),
        }
    }
    
    /// 并行执行多个节点
    pub async fn execute_parallel(
        &self,
        nodes: Vec<WorkflowNode>,
        ctx: &ExecutionContext,
        executor_chain: &BoxedExecutor,
    ) -> Result<Vec<NodeResult>> {
        let mut task_set = JoinSet::new();
        
        for node in nodes {
            // 获取并发许可
            let permit = self.concurrency_controller.acquire_permit().await?;
            let ctx = ctx.clone();
            let executor = executor_chain.clone();
            
            // 提交并行任务
            task_set.spawn(async move {
                let _permit = permit; // 保持许可生命周期
                execute_node_with_chain(node, ctx, executor).await
            });
        }
        
        // 收集结果
        let mut results = Vec::new();
        while let Some(result) = task_set.join_next().await {
            results.push(result??);
        }
        
        Ok(results)
    }
}

/// 资源限制配置
pub struct ResourceLimits {
    pub max_concurrent_nodes: usize,
    pub max_memory_mb: Option<usize>,
    pub max_execution_time: Option<Duration>,
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-W001: 引擎架构选择
- **决策**: 采用 LiteFlow 风格的组件 + 执行器链架构
- **理由**: 更好的扩展性，支持 AOP 风格的横切关注点
- **风险**: 学习成本，需要理解组件和执行器链的概念

#### ADR-W002: 调度算法
- **决策**: 使用拓扑排序（Topological Sort） + 分层执行
- **理由**: 支持复杂 DAG（有向无环图），最大化并行度
- **风险**: 内存占用，需要维护依赖图

#### ADR-W003: 状态持久化
- **决策**: 检查点机制（Checkpoint） + 增量保存
- **理由**: 支持暂停/恢复，容错能力强
- **风险**: 性能开销，需要优化保存频率

### 2.2 任务清单

- [x] Task 0: 基础引擎框架
- [x] Task 1: DAG 调度器
- [x] Task 2: 组件系统
- [x] Task 3: 执行器链
- [x] Task 4: 检查点机制完善
- [ ] Task 5: 性能优化

### 2.3 接口契约

#### 执行配置与结果

```rust
/// 执行配置
pub struct ExecutionConfig {
    pub max_concurrent: usize,
    pub timeout: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
    pub enable_checkpoint: bool,
    pub checkpoint_interval: Duration,
}

/// 执行结果
pub struct ExecutionResult {
    pub execution_id: WorkflowId,
    pub status: ExecutionStatus,
    pub output: Option<Value>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub node_results: HashMap<String, NodeResult>,
}

/// 节点结果
pub struct NodeResult {
    pub status: ExecutionStatus,
    pub output: Option<Value>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempts: u32,
}
```

#### 检查点与恢复机制

```rust
/// 检查点管理器
/// 
/// 负责检查点的保存和恢复
pub struct CheckpointManager {
    storage: Arc<dyn CheckpointStorage>,
    checkpoint_interval: Duration,
}

impl CheckpointManager {
    /// 保存检查点
    pub async fn save_checkpoint(
        &self,
        execution: &WorkflowExecution,
        context: &ExecutionContext,
    ) -> Result<Checkpoint> {
        let checkpoint = Checkpoint {
            id: Uuid::new_v4(),
            execution_id: execution.id.clone(),
            execution_state: execution.clone(),
            context_state: context.clone(),
            completed_nodes: self.extract_completed_nodes(execution),
            in_progress_nodes: self.extract_in_progress_nodes(execution),
            created_at: Utc::now(),
        };
        
        self.storage.save(&checkpoint).await?;
        
        Ok(checkpoint)
    }
    
    /// 从检查点恢复执行
    /// 
    /// 恢复流程：
    /// 1. 加载最新检查点
    /// 2. 验证检查点完整性
    /// 3. 重建执行上下文
    /// 4. 确定恢复点
    /// 5. 继续执行
    pub async fn restore_execution(
        &self,
        execution_id: WorkflowId,
    ) -> Result<RestoredExecution> {
        // 1. 加载最新检查点
        let checkpoint = self.storage
            .load_latest(execution_id)
            .await?
            .ok_or(Error::CheckpointNotFound)?;
        
        // 2. 验证检查点完整性
        self.validate_checkpoint(&checkpoint)?;
        
        // 3. 重建执行上下文
        let execution = checkpoint.execution_state;
        let context = checkpoint.context_state;
        
        // 4. 确定恢复点
        let resume_point = self.calculate_resume_point(&checkpoint);
        
        Ok(RestoredExecution {
            execution,
            context,
            resume_point,
            checkpoint,
        })
    }
    
    /// 验证检查点完整性
    fn validate_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        // 验证执行状态一致性
        if checkpoint.execution_state.status != ExecutionStatus::Paused {
            return Err(Error::InvalidCheckpointStatus);
        }
        
        // 验证时间戳合理性
        if checkpoint.created_at > Utc::now() {
            return Err(Error::InvalidCheckpointTimestamp);
        }
        
        Ok(())
    }
    
    /// 计算恢复点
    /// 
    /// 确定从哪个节点开始恢复执行
    fn calculate_resume_point(&self, checkpoint: &Checkpoint) -> ResumePoint {
        // 找到最后一个未完成的节点
        let last_completed = checkpoint.completed_nodes.last();
        
        ResumePoint {
            node_id: last_completed.map(|n| n.next_node()),
            retry_count: checkpoint.in_progress_nodes.iter()
                .map(|n| n.retry_count)
                .max()
                .unwrap_or(0),
            resume_from: self.determine_resume_strategy(checkpoint),
        }
    }
    
    /// 确定恢复策略
    fn determine_resume_strategy(&self, checkpoint: &Checkpoint) -> ResumeStrategy {
        if checkpoint.in_progress_nodes.is_empty() {
            // 没有进行中的节点，从下一层开始
            ResumeStrategy::NextLayer
        } else {
            // 有进行中的节点，需要重试
            ResumeStrategy::RetryInProgress
        }
    }
}

/// 检查点存储接口
#[async_trait]
pub trait CheckpointStorage: Send + Sync {
    async fn save(&self, checkpoint: &Checkpoint) -> Result<()>;
    async fn load(&self, checkpoint_id: Uuid) -> Result<Option<Checkpoint>>;
    async fn load_latest(&self, execution_id: WorkflowId) -> Result<Option<Checkpoint>>;
    async fn list(&self, execution_id: WorkflowId) -> Result<Vec<Checkpoint>>;
    async fn delete(&self, checkpoint_id: Uuid) -> Result<()>;
}

/// 检查点数据结构
pub struct Checkpoint {
    pub id: Uuid,
    pub execution_id: WorkflowId,
    pub execution_state: WorkflowExecution,
    pub context_state: ExecutionContext,
    pub completed_nodes: Vec<CompletedNode>,
    pub in_progress_nodes: Vec<InProgressNode>,
    pub created_at: DateTime<Utc>,
}

/// 已完成的节点记录
pub struct CompletedNode {
    pub node_id: String,
    pub output: Value,
    pub completed_at: DateTime<Utc>,
}

/// 进行中的节点记录
pub struct InProgressNode {
    pub node_id: String,
    pub retry_count: u32,
    pub started_at: DateTime<Utc>,
}

/// 恢复后的执行状态
pub struct RestoredExecution {
    pub execution: WorkflowExecution,
    pub context: ExecutionContext,
    pub resume_point: ResumePoint,
    pub checkpoint: Checkpoint,
}

/// 恢复点
pub struct ResumePoint {
    pub node_id: Option<String>,
    pub retry_count: u32,
    pub resume_from: ResumeStrategy,
}

/// 恢复策略
pub enum ResumeStrategy {
    /// 从下一层开始
    NextLayer,
    /// 重试进行中的节点
    RetryInProgress,
    /// 从特定节点开始
    FromNode(String),
}
```

### 2.4 测试策略

- **单元测试（Unit Test）**: 调度器、组件、执行器独立测试
- **集成测试（Integration Test）**: 完整工作流执行测试
- **性能测试（Performance Test）**: 大规模 DAG 执行性能
- **容错测试（Fault Tolerance Test）**: 检查点恢复、失败重试

### 2.5 架构完善任务

#### ADR-W004: 与 DDD 分层关系
- **决策**: 明确本模块属于基础设施层实现
- **理由**: 符合 DDD 分层架构，应用层定义接口，基础设施层实现
- **风险**: 需要梳理现有代码，确保依赖方向正确

#### ADR-W005: 事务补偿机制
- **决策**: 实现 Saga 模式的补偿事务
- **理由**: 工作流可能涉及多个外部系统，需要最终一致性
- **风险**: 补偿逻辑复杂，需要完善的日志和监控

- [x] Task 6: 添加 DDD 分层定位说明（第 0 节）
- [x] Task 7: 设计补偿事务机制
- [x] Task 8: 添加 Component-Tool 映射说明（第 1.4 节）
- [x] Task 9: 设计并发控制机制（第 1.6 节）
- [x] Task 10: 完善检查点恢复流程（第 2.3 节扩展）

---

## 3. 状态记录

- `[已完成]` | 检查点机制完善 | 2026-02-07
- `[已完成]` | 组件系统重构 | 2026-02-01
- `[已完成]` | 执行器链实现 | 2026-01-28
- `[已完成]` | DAG 调度器 | 2026-01-25

## 4. 执行流程

```
1. 接收 WorkflowDefinition（工作流定义）
2. 验证定义（validator）
3. DAG 调度（scheduler）
4. 初始化执行上下文
5. 按层执行节点：
   a. 获取可执行节点
   b. 并行执行（max_concurrent 限制）
   c. 执行器链处理（retry/cache/audit）
   d. 更新执行状态
   e. 保存检查点（如启用）
6. 返回执行结果
```

## 5. 生产就绪改进 (Production Readiness)

### 5.1 错误处理与容错机制

#### 错误分类

```rust
/// 错误分类
/// 
/// 根据错误性质分类，指导错误处理策略
pub enum ErrorClassification {
    /// 业务错误（可预期，可处理）
    /// 例如：参数验证失败、业务规则违反
    BusinessError { 
        code: String, 
        message: String,
        recoverable: bool, // 是否可恢复
    },
    /// 系统错误（需要重试或降级）
    /// 例如：内存不足、线程池耗尽
    SystemError { 
        retryable: bool,
        severity: ErrorSeverity,
    },
    /// 网络错误（通常可重试）
    /// 例如：连接超时、DNS 解析失败
    NetworkError { 
        timeout: bool,
        retry_after: Option<Duration>,
    },
    /// 资源错误（资源不足）
    /// 例如：磁盘空间不足、配额超限
    ResourceError { 
        resource_type: String,
        current_usage: u64,
        limit: u64,
    },
}

/// 错误严重程度
pub enum ErrorSeverity {
    Warning,    // 警告，可继续
    Error,      // 错误，需要处理
    Critical,   // 严重，需要熔断
    Fatal,      // 致命，终止执行
}
```

#### 错误处理策略

```rust
/// 错误处理策略
/// 
/// 定义遇到错误时的处理方式
pub enum ErrorHandlingStrategy {
    /// 立即重试
    ImmediateRetry { 
        max_attempts: u32,
        delay: Duration,
    },
    /// 指数退避重试
    ExponentialBackoff { 
        max_attempts: u32, 
        base_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    },
    /// 熔断（暂停服务）
    CircuitBreaker { 
        failure_threshold: u32,
        success_threshold: u32,
        timeout: Duration,
    },
    /// 降级（使用备用方案）
    Fallback { 
        fallback_node: Option<String>,
        default_value: Option<Value>,
    },
    /// 快速失败
    FailFast,
    /// 忽略错误继续
    Ignore,
}

/// 错误处理器
pub struct ErrorHandler {
    strategy: ErrorHandlingStrategy,
    classifier: Box<dyn ErrorClassifier>,
}

impl ErrorHandler {
    /// 处理错误
    pub async fn handle(
        &self,
        error: &Error,
        context: &ExecutionContext,
    ) -> ErrorAction {
        let classification = self.classifier.classify(error);
        
        match (&self.strategy, classification) {
            (ErrorHandlingStrategy::CircuitBreaker { .. }, 
             ErrorClassification::SystemError { .. }) => {
                ErrorAction::TriggerCircuitBreaker
            }
            (ErrorHandlingStrategy::Fallback { fallback_node, .. }, _) => {
                ErrorAction::ExecuteFallback(fallback_node.clone())
            }
            _ => ErrorAction::Retry,
        }
    }
}
```

#### 熔断器实现

```rust
/// 熔断器
/// 
/// 防止级联故障，保护系统稳定性
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    metrics: Arc<CircuitBreakerMetrics>,
}

pub struct CircuitBreakerConfig {
    /// 失败阈值（触发熔断的失败次数）
    pub failure_threshold: u32,
    /// 成功阈值（恢复关闭状态的成功次数）
    pub success_threshold: u32,
    /// 熔断超时时间
    pub timeout: Duration,
    /// 半开状态测试请求数
    pub half_open_max_calls: u32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// 关闭状态 - 正常处理请求
    Closed,
    /// 打开状态 - 拒绝请求，直接失败
    Open { opened_at: DateTime<Utc> },
    /// 半开状态 - 允许部分请求测试恢复
    HalfOpen { test_calls: u32 },
}

impl CircuitBreaker {
    /// 执行受保护的操作
    pub async fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        // 检查当前状态
        match *self.state.read().await {
            CircuitState::Open { opened_at } => {
                // 检查是否到达恢复时间
                if Utc::now() - opened_at > self.config.timeout {
                    // 切换到半开状态
                    *self.state.write().await = CircuitState::HalfOpen { test_calls: 0 };
                } else {
                    return Err(Error::CircuitBreakerOpen);
                }
            }
            CircuitState::HalfOpen { test_calls } => {
                if test_calls >= self.config.half_open_max_calls {
                    return Err(Error::CircuitBreakerOpen);
                }
            }
            CircuitState::Closed => {}
        }
        
        // 执行操作
        match operation.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(error)
            }
        }
    }
    
    /// 成功回调
    async fn on_success(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::HalfOpen { test_calls } => {
                if test_calls + 1 >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    self.metrics.record_state_change("half_open", "closed");
                } else {
                    *state = CircuitState::HalfOpen { test_calls: test_calls + 1 };
                }
            }
            _ => {}
        }
        self.metrics.record_success();
    }
    
    /// 失败回调
    async fn on_failure(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed => {
                let failures = self.metrics.increment_failure();
                if failures >= self.config.failure_threshold {
                    *state = CircuitState::Open { opened_at: Utc::now() };
                    self.metrics.record_state_change("closed", "open");
                }
            }
            CircuitState::HalfOpen { .. } => {
                *state = CircuitState::Open { opened_at: Utc::now() };
                self.metrics.record_state_change("half_open", "open");
            }
            _ => {}
        }
    }
}

/// 熔断器指标
pub struct CircuitBreakerMetrics {
    success_count: AtomicU32,
    failure_count: AtomicU32,
    state_changes: Vec<StateChangeEvent>,
}
```

### 5.2 可观测性设计

#### 指标收集

```rust
/// 指标收集器接口
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// 记录工作流执行时间
    fn record_workflow_duration(
        &self, 
        workflow_id: &str, 
        duration: Duration
    );
    
    /// 记录工作流执行结果
    fn record_workflow_result(
        &self, 
        workflow_id: &str, 
        success: bool
    );
    
    /// 记录节点执行时间
    fn record_node_duration(
        &self, 
        node_type: &str, 
        duration: Duration
    );
    
    /// 记录队列深度
    fn record_queue_depth(&self, depth: usize);
    
    /// 记录并发执行数
    fn record_concurrent_executions(&self, count: usize);
    
    /// 记录熔断器状态变化
    fn record_circuit_breaker_state(
        &self, 
        name: &str, 
        from: &str, 
        to: &str
    );
}

/// Prometheus 指标收集器实现
pub struct PrometheusMetricsCollector {
    workflow_duration: HistogramVec,
    workflow_results: CounterVec,
    node_duration: HistogramVec,
    queue_depth: Gauge,
    concurrent_executions: Gauge,
}
```

#### 分布式追踪

```rust
/// 追踪上下文
/// 
/// 在节点间传递追踪信息
#[derive(Clone)]
pub struct TracingContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sampled: bool,
}

impl TracingContext {
    /// 创建根上下文
    pub fn new_root() -> Self {
        Self {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            parent_span_id: None,
            sampled: true,
        }
    }
    
    /// 创建子上下文
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            sampled: self.sampled,
        }
    }
}

/// 追踪 Span
pub struct TracingSpan {
    context: TracingContext,
    name: String,
    start_time: DateTime<Utc>,
    attributes: HashMap<String, Value>,
}

impl TracingSpan {
    pub fn new(name: &str, context: TracingContext) -> Self {
        Self {
            context,
            name: name.to_string(),
            start_time: Utc::now(),
            attributes: HashMap::new(),
        }
    }
    
    /// 添加属性
    pub fn set_attribute(&mut self, key: &str, value: Value) {
        self.attributes.insert(key.to_string(), value);
    }
    
    /// 结束 Span
    pub fn end(self) -> SpanRecord {
        SpanRecord {
            trace_id: self.context.trace_id,
            span_id: self.context.span_id,
            parent_span_id: self.context.parent_span_id,
            name: self.name,
            start_time: self.start_time,
            end_time: Utc::now(),
            attributes: self.attributes,
        }
    }
}
```

#### 结构化日志

```rust
/// 工作流日志
#[derive(Serialize)]
pub struct WorkflowLog {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 日志级别
    pub level: LogLevel,
    /// 追踪 ID
    pub trace_id: String,
    /// 执行 ID
    pub execution_id: String,
    /// 节点 ID
    pub node_id: Option<String>,
    /// 消息
    pub message: String,
    /// 上下文
    pub context: HashMap<String, Value>,
}

#[derive(Serialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// 日志记录器
pub struct WorkflowLogger {
    collector: Arc<dyn LogCollector>,
}

impl WorkflowLogger {
    pub fn log(&self, log: WorkflowLog) {
        self.collector.collect(log);
    }
    
    /// 记录节点开始
    pub fn node_started(&self, execution_id: &str, node_id: &str) {
        self.log(WorkflowLog {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            trace_id: generate_trace_id(),
            execution_id: execution_id.to_string(),
            node_id: Some(node_id.to_string()),
            message: "Node execution started".to_string(),
            context: HashMap::new(),
        });
    }
}
```

### 5.3 限流与熔断

#### 限流器

```rust
/// 限流器
/// 
/// 使用令牌桶算法控制请求速率
pub struct RateLimiter {
    /// 令牌桶
    tokens: Arc<Mutex<f64>>,
    /// 令牌生成速率（每秒）
    rate: f64,
    /// 桶容量
    capacity: f64,
    /// 上次更新时间
    last_update: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(rate: f64, capacity: f64) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(capacity)),
            rate,
            capacity,
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    /// 更新令牌数量
    fn update_tokens(&self) {
        let mut last_update = self.last_update.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_update).as_secs_f64();
        
        let mut tokens = self.tokens.lock().unwrap();
        *tokens = (*tokens + elapsed * self.rate).min(self.capacity);
        *last_update = now;
    }
    
    /// 尝试获取许可
    pub async fn acquire(&self, tokens: f64) -> Result<()> {
        self.update_tokens();
        
        let mut current = self.tokens.lock().unwrap();
        if *current >= tokens {
            *current -= tokens;
            Ok(())
        } else {
            Err(Error::RateLimitExceeded)
        }
    }
    
    /// 等待获取许可
    pub async fn acquire_with_wait(&self, tokens: f64) -> Result<()> {
        loop {
            match self.acquire(tokens).await {
                Ok(()) => return Ok(()),
                Err(Error::RateLimitExceeded) => {
                    sleep(Duration::from_millis(100)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// 工作流限流配置
pub struct WorkflowRateLimitConfig {
    /// 每秒最大执行数
    pub max_executions_per_second: u32,
    /// 并发执行限制
    pub max_concurrent: usize,
    /// 是否启用熔断
    pub enable_circuit_breaker: bool,
    /// 熔断配置
    pub circuit_breaker: CircuitBreakerConfig,
}
```

### 5.4 配置管理增强

```rust
/// 配置加载器
#[async_trait]
pub trait ConfigLoader: Send + Sync {
    /// 加载配置
    async fn load(&self) -> Result<WorkflowConfig>;
    /// 重新加载配置
    async fn reload(&self) -> Result<WorkflowConfig>;
    /// 监听配置变化
    async fn watch(&self) -> Result<mpsc::Receiver<ConfigChange>>;
}

/// 文件配置加载器
pub struct FileConfigLoader {
    path: PathBuf,
}

#[async_trait]
impl ConfigLoader for FileConfigLoader {
    async fn load(&self) -> Result<WorkflowConfig> {
        let content = fs::read_to_string(&self.path).await?;
        let config: WorkflowConfig = serde_yaml::from_str(&content)?;
        ConfigValidator::validate(&config)?;
        Ok(config)
    }
    
    async fn reload(&self) -> Result<WorkflowConfig> {
        self.load().await
    }
    
    async fn watch(&self) -> Result<mpsc::Receiver<ConfigChange>> {
        // 实现文件监听
        todo!()
    }
}

/// 配置验证器
pub struct ConfigValidator;

impl ConfigValidator {
    /// 验证配置
    pub fn validate(config: &WorkflowConfig) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // 验证超时设置
        if let Some(timeout) = config.default_timeout {
            if timeout == 0 {
                report.add_error("timeout", "超时时间不能为0");
            }
        }
        
        // 验证并发设置
        if config.max_concurrent_steps == 0 {
            report.add_error("max_concurrent", "并发数必须大于0");
        }
        
        // 验证重试策略
        if let Some(ref retry) = config.retry_policy {
            if retry.max_attempts == 0 {
                report.add_warning("retry", "重试次数为0，相当于禁用重试");
            }
        }
        
        Ok(report)
    }
}

/// 验证报告
pub struct ValidationReport {
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
}

/// 热更新管理器
pub struct ConfigHotReloader {
    loader: Box<dyn ConfigLoader>,
    current_config: Arc<RwLock<WorkflowConfig>>,
    change_handlers: Vec<Box<dyn Fn(&ConfigChange)>>,
}

impl ConfigHotReloader {
    pub async fn start(&self) -> Result<()> {
        let mut receiver = self.loader.watch().await?;
        
        while let Some(change) = receiver.recv().await {
            match self.loader.reload().await {
                Ok(new_config) => {
                    *self.current_config.write().await = new_config;
                    for handler in &self.change_handlers {
                        handler(&change);
                    }
                }
                Err(e) => {
                    error!("Failed to reload config: {}", e);
                }
            }
        }
        
        Ok(())
    }
}
```

### 5.5 版本兼容性

```rust
/// 工作流版本
#[derive(Clone, PartialEq)]
pub struct WorkflowVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl WorkflowVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
    
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 版本兼容性检查器
pub struct VersionCompatibilityChecker;

impl VersionCompatibilityChecker {
    /// 检查版本兼容性
    pub fn check_compatibility(
        &self,
        current: &WorkflowVersion,
        target: &WorkflowVersion,
    ) -> CompatibilityResult {
        if current.major != target.major {
            CompatibilityResult::Incompatible(
                format!("主版本不兼容: {} vs {}", current.major, target.major)
            )
        } else if current.minor < target.minor {
            CompatibilityResult::UpgradeRequired
        } else {
            CompatibilityResult::Compatible
        }
    }
}

pub enum CompatibilityResult {
    Compatible,
    UpgradeRequired,
    Incompatible(String),
}

/// 迁移策略
#[async_trait]
pub trait MigrationStrategy: Send + Sync {
    /// 迁移工作流定义
    async fn migrate(&self, definition: &mut WorkflowDefinition) -> Result<()>;
    /// 检查是否支持迁移
    fn can_migrate(&self, from: &WorkflowVersion, to: &WorkflowVersion) -> bool;
}

/// 版本迁移管理器
pub struct MigrationManager {
    strategies: Vec<Box<dyn MigrationStrategy>>,
}

impl MigrationManager {
    /// 迁移工作流
    pub async fn migrate(
        &self,
        definition: &mut WorkflowDefinition,
        target_version: &WorkflowVersion,
    ) -> Result<()> {
        let current_version = definition.version.clone();
        
        // 检查兼容性
        let checker = VersionCompatibilityChecker;
        match checker.check_compatibility(&current_version, target_version) {
            CompatibilityResult::Compatible => return Ok(()),
            CompatibilityResult::Incompatible(reason) => {
                return Err(Error::IncompatibleVersion(reason));
            }
            CompatibilityResult::UpgradeRequired => {}
        }
        
        // 查找并执行迁移策略
        for strategy in &self.strategies {
            if strategy.can_migrate(&current_version, target_version) {
                strategy.migrate(definition).await?;
                definition.version = target_version.clone();
                return Ok(());
            }
        }
        
        Err(Error::NoMigrationPath)
    }
}
```

### 5.6 安全设计

```rust
/// 权限检查器
#[async_trait]
pub trait PermissionChecker: Send + Sync {
    /// 检查执行权限
    async fn can_execute(&self, user_id: &str, workflow_id: &str) -> bool;
    /// 检查管理权限
    async fn can_manage(&self, user_id: &str, workflow_id: &str) -> bool;
    /// 检查查看权限
    async fn can_view(&self, user_id: &str, workflow_id: &str) -> bool;
}

/// 基于角色的权限检查器
pub struct RBACPermissionChecker {
    role_store: Arc<dyn RoleStore>,
}

#[async_trait]
impl PermissionChecker for RBACPermissionChecker {
    async fn can_execute(&self, user_id: &str, workflow_id: &str) -> bool {
        let roles = self.role_store.get_user_roles(user_id).await;
        roles.iter().any(|r| r.has_permission("workflow:execute"))
    }
    
    async fn can_manage(&self, user_id: &str, workflow_id: &str) -> bool {
        let roles = self.role_store.get_user_roles(user_id).await;
        roles.iter().any(|r| r.has_permission("workflow:manage"))
    }
    
    async fn can_view(&self, user_id: &str, workflow_id: &str) -> bool {
        let roles = self.role_store.get_user_roles(user_id).await;
        roles.iter().any(|r| r.has_permission("workflow:view"))
    }
}

/// 数据加密
pub struct DataEncryption {
    cipher: Box<dyn Cipher>,
}

impl DataEncryption {
    pub fn new(cipher: Box<dyn Cipher>) -> Self {
        Self { cipher }
    }
    
    /// 加密敏感数据
    pub fn encrypt(&self, data: &str) -> Result<String> {
        self.cipher.encrypt(data)
    }
    
    /// 解密数据
    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        self.cipher.decrypt(encrypted)
    }
    
    /// 加密工作流上下文中的敏感数据
    pub fn encrypt_context(&self, context: &mut ExecutionContext) -> Result<()> {
        for (key, value) in &mut context.global_variables {
            if is_sensitive_key(key) {
                *value = Value::String(self.encrypt(&value.to_string())?);
            }
        }
        Ok(())
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let sensitive_patterns = ["password", "secret", "token", "key", "credential"];
    sensitive_patterns.iter().any(|p| key.to_lowercase().contains(p))
}
```

### 5.7 测试策略细化

```markdown
#### 测试场景矩阵

| 场景 | 类型 | 覆盖率要求 | 关键验证点 |
|------|------|-----------|-----------|
| 简单线性工作流 | 单元测试 | 100% | 节点顺序执行 |
| 复杂 DAG 工作流 | 集成测试 | 90% | 依赖解析正确 |
| 并行节点执行 | 并发测试 | 85% | 无竞态条件 |
| 检查点保存/恢复 | 容错测试 | 90% | 状态一致性 |
| 熔断器触发 | 容错测试 | 80% | 状态转换正确 |
| 限流器生效 | 性能测试 | 75% | 速率控制准确 |
| 1000+ 节点工作流 | 压力测试 | 70% | 内存/性能稳定 |
| 内存限制场景 | 资源测试 | 80% | 优雅降级 |
| 网络超时恢复 | 容错测试 | 85% | 重试机制有效 |
| 配置热更新 | 集成测试 | 75% | 配置生效及时 |

#### 测试工具

- **单元测试**: `cargo test` + `mockall` 模拟依赖
- **集成测试**: `cargo test --test integration`
- **性能测试**: `criterion` 基准测试
- **压力测试**: `k6` 或自定义负载生成器
- **混沌测试**: 随机注入故障验证容错能力
```

## 6. 子模块

- [组件系统](./component/design.md) - 组件详细设计
- [执行器链](./executor/design.md) - 执行器链详细设计
- [数据上下文](./context/design.md) - 数据传递机制
- [状态管理](./state/design.md) - 检查点和恢复机制
