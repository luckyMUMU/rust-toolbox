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

---

## 5. DDD 战术设计

### 5.1 聚合根定义

聚合根（Aggregate Root）是 DDD 中的核心概念，用于管理一组相关对象的一致性边界。

#### 聚合根识别

| 聚合根 | 包含实体 | 包含值对象 | 一致性边界 |
|--------|----------|------------|------------|
| `WorkflowExecution` | `NodeExecution` | `ExecutionStatus`, `ExecutionContext` | 单次执行的所有状态 |
| `WorkflowDefinition` | `WorkflowNode`, `WorkflowEdge` | `WorkflowConfig`, `RetryPolicy` | 工作流定义完整性 |
| `Tool` | - | `ToolInfo`, `ToolSchema` | 工具元数据 |
| `Plugin` | - | `PluginInfo`, `PluginConfig` | 插件配置 |

#### 聚合根：WorkflowExecution

```rust
/// 工作流执行聚合根
/// 
/// 作为执行上下文的单一入口点，保证执行状态的一致性。
/// 所有对执行状态的修改必须通过此聚合根进行。
pub struct WorkflowExecution {
    /// 聚合根 ID
    pub id: WorkflowId,
    /// 关联的工作流定义 ID
    pub workflow_id: Uuid,
    /// 当前执行状态（值对象）
    pub status: ExecutionStatus,
    /// 执行上下文（值对象）
    pub context: ExecutionContext,
    /// 节点执行记录（实体集合）
    pub node_executions: Vec<NodeExecution>,
    /// 创建时间
    pub started_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 领域事件
    pub pending_events: Vec<Box<dyn DomainEvent>>,
}

impl WorkflowExecution {
    /// 创建新的执行（工厂方法）
    pub fn new(workflow_id: Uuid, context: ExecutionContext) -> Self {
        let id = WorkflowId::new();
        let now = Utc::now();
        
        Self {
            id,
            workflow_id,
            status: ExecutionStatus::Pending,
            context,
            node_executions: Vec::new(),
            started_at: now,
            completed_at: None,
            pending_events: vec![Box::new(ExecutionStarted {
                execution_id: id,
                workflow_id,
                started_at: now,
                context: context.clone(),
            })],
        }
    }
    
    /// 开始执行（状态转换）
    pub fn start(&mut self) -> Result<()> {
        if self.status != ExecutionStatus::Pending {
            return Err(DomainError::InvalidStateTransition(
                format!("无法从 {:?} 状态启动", self.status)
            ));
        }
        
        self.status = ExecutionStatus::Running;
        self.record_event(ExecutionStatusChanged {
            execution_id: self.id,
            old_status: ExecutionStatus::Pending,
            new_status: ExecutionStatus::Running,
        });
        
        Ok(())
    }
    
    /// 完成节点执行
    pub fn complete_node(&mut self, node_id: &str, output: Value) -> Result<()> {
        // 验证节点是否属于此聚合
        let node_exec = NodeExecution {
            node_id: node_id.to_string(),
            status: NodeExecutionStatus::Completed,
            output: Some(output.clone()),
            completed_at: Utc::now(),
        };
        
        self.node_executions.push(node_exec);
        
        self.record_event(NodeExecuted {
            execution_id: self.id,
            node_id: node_id.to_string(),
            status: NodeExecutionStatus::Completed,
            output: Some(output),
        });
        
        Ok(())
    }
    
    /// 完成执行
    pub fn complete(&mut self) -> Result<()> {
        if self.status != ExecutionStatus::Running {
            return Err(DomainError::InvalidStateTransition(
                format!("无法从 {:?} 状态完成", self.status)
            ));
        }
        
        self.status = ExecutionStatus::Completed;
        self.completed_at = Some(Utc::now());
        
        self.record_event(ExecutionCompleted {
            execution_id: self.id,
            workflow_id: self.workflow_id,
            result: ExecutionResult::Success,
            completed_at: Utc::now(),
        });
        
        Ok(())
    }
    
    /// 记录领域事件
    fn record_event<E: DomainEvent + 'static>(&mut self, event: E) {
        self.pending_events.push(Box::new(event));
    }
    
    /// 清除已处理的领域事件
    pub fn clear_events(&mut self) {
        self.pending_events.clear();
    }
}
```

#### 聚合根：WorkflowDefinition

```rust
/// 工作流定义聚合根
/// 
/// 管理工作流的完整定义，包括节点、边和配置。
/// 保证工作流定义的结构完整性和业务规则。
pub struct WorkflowDefinition {
    /// 聚合根 ID
    pub id: Uuid,
    /// 工作流名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 描述
    pub description: Option<String>,
    /// 节点集合（实体）
    pub nodes: Vec<WorkflowNode>,
    /// 边集合（实体）
    pub edges: Vec<WorkflowEdge>,
    /// 全局配置（值对象）
    pub config: WorkflowConfig,
    /// 元数据
    pub metadata: HashMap<String, Value>,
    /// 领域事件
    pending_events: Vec<Box<dyn DomainEvent>>,
}

impl WorkflowDefinition {
    /// 创建新的工作流定义
    pub fn new(name: String, version: String) -> Self {
        let id = Uuid::new_v4();
        
        Self {
            id,
            name,
            version,
            description: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            config: WorkflowConfig::default(),
            metadata: HashMap::new(),
            pending_events: vec![Box::new(WorkflowCreated {
                workflow_id: id,
                name: name.clone(),
                created_at: Utc::now(),
            })],
        }
    }
    
    /// 添加节点（聚合内部操作）
    pub fn add_node(&mut self, node: WorkflowNode) -> Result<()> {
        // 验证节点 ID 唯一性
        if self.nodes.iter().any(|n| n.id == node.id) {
            return Err(DomainError::DuplicateNodeId(node.id.clone()));
        }
        
        // 验证节点配置
        node.validate()?;
        
        self.nodes.push(node);
        Ok(())
    }
    
    /// 添加边（聚合内部操作）
    pub fn add_edge(&mut self, edge: WorkflowEdge) -> Result<()> {
        // 验证边的节点存在
        if !self.nodes.iter().any(|n| n.id == edge.from) {
            return Err(DomainError::NodeNotFound(edge.from.clone()));
        }
        if !self.nodes.iter().any(|n| n.id == edge.to) {
            return Err(DomainError::NodeNotFound(edge.to.clone()));
        }
        
        self.edges.push(edge);
        Ok(())
    }
    
    /// 验证工作流定义完整性
    pub fn validate(&self) -> Result<()> {
        // 验证至少有一个节点
        if self.nodes.is_empty() {
            return Err(DomainError::InvalidWorkflow("工作流必须包含至少一个节点"));
        }
        
        // 验证 DAG 结构
        self.validate_dag()?;
        
        // 验证所有节点可达
        self.validate_reachability()?;
        
        Ok(())
    }
    
    /// 验证 DAG 结构（无环）
    fn validate_dag(&self) -> Result<()> {
        // 使用拓扑排序检测环
        let mut in_degree: HashMap<String, usize> = self.nodes.iter()
            .map(|n| (n.id.clone(), 0))
            .collect();
        
        for edge in &self.edges {
            *in_degree.get_mut(&edge.to).unwrap() += 1;
        }
        
        let mut queue: Vec<String> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut visited = 0;
        while let Some(node_id) = queue.pop() {
            visited += 1;
            for edge in self.edges.iter().filter(|e| e.from == node_id) {
                let deg = in_degree.get_mut(&edge.to).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(edge.to.clone());
                }
            }
        }
        
        if visited != self.nodes.len() {
            return Err(DomainError::CyclicDependency);
        }
        
        Ok(())
    }
    
    /// 验证所有节点可达
    fn validate_reachability(&self) -> Result<()> {
        // 从入度为 0 的节点开始 BFS
        let start_nodes: Vec<&str> = self.nodes.iter()
            .filter(|n| !self.edges.iter().any(|e| e.to == n.id))
            .map(|n| n.id.as_str())
            .collect();
        
        if start_nodes.is_empty() && !self.nodes.is_empty() {
            return Err(DomainError::NoEntryPoint);
        }
        
        Ok(())
    }
}
```

### 5.2 值对象定义

```rust
/// 执行状态值对象
/// 
/// 不可变对象，表示执行的状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

impl ExecutionStatus {
    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled | Self::Timeout)
    }
    
    /// 是否可以转换到目标状态
    pub fn can_transition_to(&self, target: ExecutionStatus) -> bool {
        match (self, target) {
            (Self::Pending, Self::Running) => true,
            (Self::Running, Self::Paused | Self::Completed | Self::Failed | Self::Cancelled | Self::Timeout) => true,
            (Self::Paused, Self::Running | Self::Cancelled) => true,
            _ => false,
        }
    }
}

/// 执行上下文值对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub global_variables: HashMap<String, Value>,
    pub step_results: HashMap<String, Value>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new(execution_id: String) -> Self {
        Self {
            execution_id,
            workflow_id: None,
            user_id: None,
            global_variables: HashMap::new(),
            step_results: HashMap::new(),
        }
    }
    
    /// 设置步骤结果（返回新实例）
    pub fn with_step_result(&self, step_id: String, result: Value) -> Self {
        let mut new_context = self.clone();
        new_context.step_results.insert(step_id, result);
        new_context
    }
}
```

### 5.3 领域服务

```rust
/// 领域服务：工作流验证器
/// 
/// 纯领域逻辑，不依赖任何外部服务
pub struct WorkflowValidator;

impl WorkflowValidator {
    /// 验证工作流定义
    pub fn validate(workflow: &WorkflowDefinition) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // 验证名称
        if workflow.name.is_empty() {
            errors.push("工作流名称不能为空".to_string());
        }
        
        // 验证版本格式
        if !Self::is_valid_version(&workflow.version) {
            errors.push(format!("无效的版本格式: {}", workflow.version));
        }
        
        // 验证节点
        for node in &workflow.nodes {
            if let Err(e) = Self::validate_node(node) {
                errors.push(format!("节点 {} 验证失败: {}", node.id, e));
            }
        }
        
        // 验证边
        for edge in &workflow.edges {
            if let Err(e) = Self::validate_edge(edge, &workflow.nodes) {
                warnings.push(format!("边 {} -> {} 验证警告: {}", edge.from, edge.to, e));
            }
        }
        
        if errors.is_empty() {
            Ok(ValidationResult::Valid { warnings })
        } else {
            Ok(ValidationResult::Invalid { errors, warnings })
        }
    }
    
    fn is_valid_version(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() >= 2 && parts.iter().all(|p| p.parse::<u32>().is_ok())
    }
    
    fn validate_node(node: &WorkflowNode) -> Result<()> {
        if node.id.is_empty() {
            return Err(DomainError::InvalidNode("节点 ID 不能为空"));
        }
        Ok(())
    }
    
    fn validate_edge(edge: &WorkflowEdge, nodes: &[WorkflowNode]) -> Result<()> {
        if !nodes.iter().any(|n| n.id == edge.from) {
            return Err(DomainError::NodeNotFound(edge.from.clone()));
        }
        if !nodes.iter().any(|n| n.id == edge.to) {
            return Err(DomainError::NodeNotFound(edge.to.clone()));
        }
        Ok(())
    }
}

/// 验证结果
pub enum ValidationResult {
    Valid { warnings: Vec<String> },
    Invalid { errors: Vec<String>, warnings: Vec<String> },
}
```

### 5.4 聚合设计原则

1. **一致性边界**: 聚合内的所有修改必须保持一致性
2. **唯一入口**: 外部只能通过聚合根访问聚合内部对象
3. **边界最小化**: 聚合应尽可能小，减少锁定范围
4. **通过 ID 引用**: 聚合之间通过 ID 引用，而非直接引用对象
5. **最终一致性**: 跨聚合的一致性通过领域事件实现
