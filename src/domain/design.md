# 领域层 (Domain Layer)

## 1. 核心定义 (Stable)

### 1.1 层职责

领域层是系统的核心，包含业务逻辑和领域模型。该层不依赖其他任何层，只通过 Port（端口）定义对外接口。

### 1.2 模块结构

```
domain/
├── model/              # 领域模型（实体和值对象）
│   ├── workflow.rs        # 工作流相关实体
│   ├── execution.rs       # 执行相关实体
│   ├── tool.rs            # 工具相关实体
│   ├── plugin.rs          # 插件相关实体
│   └── value_object.rs    # 值对象
├── event/              # 领域事件
│   ├── execution_event.rs # 执行相关事件
│   ├── workflow_event.rs  # 工作流相关事件
│   └── mod.rs             # 事件总线定义
├── service/            # 领域服务（纯领域逻辑）
│   ├── workflow_validator.rs      # 工作流验证服务
│   ├── execution_calculator.rs    # 执行状态计算服务
│   └── mod.rs
└── port/               # 端口（接口定义）
    ├── repository.rs        # 仓储接口
    ├── tool_registry.rs     # 工具注册表接口
    └── plugin_manager.rs    # 插件管理器接口
```

### 1.3 领域模型

#### 工作流实体 (Workflow)

```rust
// 核心实体
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub config: WorkflowConfig,
}

// 值对象
pub struct WorkflowConfig {
    pub max_concurrent_steps: usize,
    pub default_timeout: Option<u64>,
    pub checkpoint_interval: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
}

pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Option<Duration>,
    pub backoff_multiplier: f64,
    pub strategy: RetryStrategy,
}
```

#### 执行实体 (Execution)

```rust
// 聚合根
pub struct WorkflowExecution {
    pub id: WorkflowId,
    pub workflow_id: Uuid,
    pub status: ExecutionStatus,
    pub context: ExecutionContext,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// 值对象
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub global_variables: HashMap<String, Value>,
    pub step_results: HashMap<String, Value>,
}

pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}
```

#### 工具实体 (Tool)

```rust
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub parameters_schema: Value,
    pub return_schema: Value,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub plugin_name: Option<String>,
}
```

#### 插件实体 (Plugin)

```rust
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub plugin_type: PluginType,
    pub metadata: HashMap<String, Value>,
}

pub enum PluginType {
    Native,
    Python,
    NodeJs,
    Wasm,
    Docker,
    Go,
}
```

### 1.4 领域事件 (Domain Events)

#### 执行事件

```rust
/// 执行开始事件
pub struct ExecutionStarted {
    pub execution_id: WorkflowId,
    pub workflow_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub context: ExecutionContext,
}

/// 执行完成事件
pub struct ExecutionCompleted {
    pub execution_id: WorkflowId,
    pub workflow_id: Uuid,
    pub result: ExecutionResult,
    pub completed_at: DateTime<Utc>,
}

/// 执行失败事件
pub struct ExecutionFailed {
    pub execution_id: WorkflowId,
    pub workflow_id: Uuid,
    pub error: WorkflowError,
    pub failed_at: DateTime<Utc>,
}

/// 节点执行事件
pub struct NodeExecuted {
    pub execution_id: WorkflowId,
    pub node_id: NodeId,
    pub status: NodeExecutionStatus,
    pub output: Option<Value>,
}
```

#### 工作流事件

```rust
/// 工作流创建事件
pub struct WorkflowCreated {
    pub workflow_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// 工作流更新事件
pub struct WorkflowUpdated {
    pub workflow_id: Uuid,
    pub updated_at: DateTime<Utc>,
}

/// 工作流删除事件
pub struct WorkflowDeleted {
    pub workflow_id: Uuid,
    pub deleted_at: DateTime<Utc>,
}
```

#### 插件事件

```rust
/// 插件加载事件
pub struct PluginLoaded {
    pub plugin_name: String,
    pub version: String,
    pub loaded_at: DateTime<Utc>,
}

/// 插件卸载事件
pub struct PluginUnloaded {
    pub plugin_name: String,
    pub unloaded_at: DateTime<Utc>,
}

/// 插件重新加载事件
pub struct PluginReloaded {
    pub plugin_name: String,
    pub version: String,
    pub reloaded_at: DateTime<Utc>,
}
```

#### 事件总线

```rust
/// 领域事件总线
#[async_trait]
pub trait DomainEventBus: Send + Sync {
    /// 发布事件
    async fn publish<E: DomainEvent>(&self, event: E) -> Result<()>;
    
    /// 订阅事件
    async fn subscribe<E: DomainEvent, H: EventHandler<E>>(
        &self,
        handler: H,
    ) -> Result<SubscriptionId>;
}

/// 领域事件标记 trait
pub trait DomainEvent: Send + Sync + Clone {
    fn event_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// 事件处理器
#[async_trait]
pub trait EventHandler<E: DomainEvent>: Send + Sync {
    async fn handle(&self, event: &E) -> Result<()>;
}
```

#### 领域事件 Trait 实现示例

```rust
// 为 ExecutionStarted 实现 DomainEvent trait
impl DomainEvent for ExecutionStarted {
    fn event_type(&self) -> &'static str {
        "execution.started"
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.started_at
    }
}

// 为 ExecutionCompleted 实现 DomainEvent trait
impl DomainEvent for ExecutionCompleted {
    fn event_type(&self) -> &'static str {
        "execution.completed"
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.completed_at
    }
}

// 为 PluginLoaded 实现 DomainEvent trait
impl DomainEvent for PluginLoaded {
    fn event_type(&self) -> &'static str {
        "plugin.loaded"
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.loaded_at
    }
}
```

### 1.5 端口定义 (Port)

#### 仓储接口

```rust
pub trait WorkflowRepository: Send + Sync {
    async fn save(&self, workflow: &WorkflowDefinition) -> Result<()>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<WorkflowDefinition>>;
    async fn find_all(&self) -> Result<Vec<WorkflowDefinition>>;
    async fn delete(&self, id: &Uuid) -> Result<()>;
}

pub trait ExecutionRepository: Send + Sync {
    async fn save(&self, execution: &WorkflowExecution) -> Result<()>;
    async fn find_by_id(&self, id: &WorkflowId) -> Result<Option<WorkflowExecution>>;
    async fn find_by_workflow(&self, workflow_id: &Uuid) -> Result<Vec<WorkflowExecution>>;
}
```

#### 工具注册表接口

```rust
pub trait ToolRegistry: Send + Sync {
    fn register(&self, tool: ToolInfo) -> Result<()>;
    fn unregister(&self, name: &str) -> Result<()>;
    fn get(&self, name: &str) -> Result<Option<ToolInfo>>;
    fn list_all(&self) -> Result<Vec<ToolInfo>>;
}
```

#### 插件管理器接口

```rust
pub trait PluginManager: Send + Sync {
    async fn load(&self, plugin: PluginInfo) -> Result<()>;
    async fn unload(&self, name: &str) -> Result<()>;
    async fn reload(&self, name: &str) -> Result<()>;
    fn get(&self, name: &str) -> Result<Option<PluginInfo>>;
    fn list_all(&self) -> Result<Vec<PluginInfo>>;
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-D001: 实体与值对象划分
- **决策**: WorkflowExecution 作为聚合根，WorkflowConfig 作为值对象
- **理由**: 执行记录需要唯一标识追踪，配置数据不可变且可替换
- **风险**: 值对象变更需要重新创建实例

#### ADR-D002: 状态机设计
- **决策**: ExecutionStatus 使用枚举实现状态机
- **理由**: Rust 枚举支持方法定义，可封装状态转换逻辑
- **风险**: 复杂状态转换可能需要独立的状态机库

### 2.2 任务清单

- [x] Task 0: 领域模型基础结构
- [x] Task 1: 端口接口定义
- [x] Task 2: 领域事件定义（ExecutionStarted, ExecutionCompleted, PluginLoaded 等）
- [x] Task 3: 领域服务实现（WorkflowValidator, ExecutionStateCalculator）
- [x] Task 4: 值对象完整实现

### 2.3 接口契约

```rust
/// 领域服务：工作流验证
/// 
/// 纯领域逻辑，不依赖任何外部服务
pub trait WorkflowValidator: Send + Sync {
    /// 验证工作流定义
    fn validate(&self, workflow: &WorkflowDefinition) -> Result<ValidationResult>;
    
    /// 验证节点依赖关系（DAG 检查）
    fn validate_dag(&self, nodes: &[WorkflowNode], edges: &[WorkflowEdge]) -> Result<()>;
}

/// 领域服务：执行状态计算器
/// 
/// 根据当前执行状态计算下一步操作
pub trait ExecutionStateCalculator: Send + Sync {
    /// 计算可执行的下一个节点
    fn calculate_next_nodes(&self, execution: &WorkflowExecution) -> Vec<NodeId>;
    
    /// 检查执行是否可以完成
    fn can_complete(&self, execution: &WorkflowExecution) -> bool;
}
```

### 2.4 测试策略

- **单元测试**: 领域模型方法测试
- **属性测试**: 状态机转换正确性
- **契约测试**: 端口接口契约验证

## 3. 状态记录

- `[已完成]` | 领域事件系统完善 | 2026-02-07
- `[已完成]` | 基础模型定义 | 2026-01-15
- `[已完成]` | 端口接口定义 | 2026-01-20
- `[已完成]` | 值对象完整实现 | 2026-02-07

## 4. 子模块

- [模型详情](./model/design.md) - 详细领域模型定义
- [端口详情](./port/design.md) - 接口契约详细说明
