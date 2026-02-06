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
- [ ] Task 4: 检查点机制完善
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

- `[进行中]` | 检查点机制完善 | 2026-02-06
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

## 5. 子模块

- [组件系统](./component/design.md) - 组件详细设计
- [执行器链](./executor/design.md) - 执行器链详细设计
- [数据上下文](./context/design.md) - 数据传递机制
- [状态管理](./state/design.md) - 检查点和恢复机制
