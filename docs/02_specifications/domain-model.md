# Workflow Toolkit 领域模型规范 (Domain Model Specification)

## 文档元数据

- **版本**: v1.0.0
- **规范级别**: P2 (模块规范)
- **最后更新**: 2026-03-01
- **状态**: 已批准

## 1. 领域模型概述

### 1.1 领域范围

Workflow Toolkit 的领域模型涵盖以下核心领域：
- **执行领域**: 工作流执行、节点执行、执行状态
- **工作流领域**: 工作流定义、节点、边、配置
- **插件领域**: 插件信息、插件类型、插件配置
- **工具领域**: 工具信息、工具类型、工具输入输出

### 1.2 领域原则

1. **纯净性**: 领域模型不依赖任何基础设施细节
2. **不变性**: 值对象不可变，实体通过事件记录变更
3. **聚合根**: 每个聚合有唯一的聚合根，外部只能通过聚合根访问
4. **领域事件**: 状态变更通过领域事件发布

## 2. 聚合设计

### 2.1 工作流聚合

**聚合根**: `WorkflowDefinition`

```
WorkflowDefinition (聚合根)
├── WorkflowNode[] (实体)
├── WorkflowEdge[] (实体)
├── WorkflowConfig (值对象)
└── WorkflowMetadata (值对象)
```

**不变量**:
- 节点 ID 在工作流内唯一
- 边必须连接已存在的节点
- 入口节点必须有且仅有一个
- 工作流名称不能为空

### 2.2 执行聚合

**聚合根**: `WorkflowExecution`

```
WorkflowExecution (聚合根)
├── ExecutionId (值对象)
├── ExecutionStatus (枚举)
├── NodeExecutionState[] (实体)
├── Checkpoint[] (值对象)
├── ExecutionRecord[] (值对象)
└── DataContext (值对象)
```

**不变量**:
- 执行 ID 唯一
- 状态转换必须遵循状态机规则
- 节点执行状态与工作流执行状态一致

### 2.3 插件聚合

**聚合根**: `PluginInfo`

```
PluginInfo (聚合根)
├── PluginId (值对象)
├── PluginType (枚举)
├── PluginStatus (枚举)
├── PluginConfig (值对象)
└── ToolInfo[] (值对象)
```

**不变量**:
- 插件 ID 唯一
- 插件状态转换必须遵循状态机规则
- 工具名称在插件内唯一

### 2.4 工具聚合

**聚合根**: `ToolInfo`

```
ToolInfo (聚合根)
├── ToolId (值对象)
├── ToolKind (枚举)
├── InputSchema (值对象)
├── OutputSchema (值对象)
└── ToolMetadata (值对象)
```

**不变量**:
- 工具 ID 唯一
- 工具名称在注册表内唯一
- Schema 必须符合 JSON Schema 规范

## 3. 实体定义

### 3.1 WorkflowNode

**职责**: 表示工作流中的一个节点

**属性**:
```rust
pub struct WorkflowNode {
    pub id: NodeId,                    // 节点 ID
    pub name: String,                  // 节点名称
    pub node_type: NodeType,           // 节点类型
    pub config: NodeConfig,            // 节点配置
    pub position: Option<Position>,    // UI 位置
    pub metadata: HashMap<String, Value>, // 元数据
}

pub enum NodeType {
    Tool,        // 工具节点
    Condition,   // 条件节点
    Loop,        // 循环节点
    Parallel,    // 并行节点
    Switch,      // 切换节点
    Checkpoint,  // 检查点节点
}
```

**行为**:
- `validate()`: 验证节点配置
- `get_dependencies()`: 获取依赖的节点

### 3.2 WorkflowEdge

**职责**: 表示工作流中的一条边

**属性**:
```rust
pub struct WorkflowEdge {
    pub source: NodeId,           // 源节点 ID
    pub target: NodeId,           // 目标节点 ID
    pub condition: Option<String>, // 条件表达式
    pub weight: Option<u32>,      // 权重
    pub metadata: HashMap<String, Value>, // 元数据
}
```

**行为**:
- `evaluate_condition()`: 评估条件表达式

### 3.3 NodeExecutionState

**职责**: 表示节点的执行状态

**属性**:
```rust
pub struct NodeExecutionState {
    pub node_id: NodeId,           // 节点 ID
    pub status: ExecutionStatus,   // 执行状态
    pub started_at: Option<DateTime>, // 开始时间
    pub ended_at: Option<DateTime>,   // 结束时间
    pub result: Option<Value>,     // 执行结果
    pub error: Option<String>,     // 错误信息
    pub retry_count: u32,          // 重试次数
}
```

**行为**:
- `start()`: 开始执行
- `complete()`: 完成执行
- `fail()`: 失败
- `retry()`: 重试

## 4. 值对象定义

### 4.1 ExecutionId

**职责**: 唯一标识一次工作流执行

```rust
pub struct ExecutionId(String);

impl ExecutionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    pub fn from_string(s: String) -> Result<Self> {
        // 验证格式
        Ok(Self(s))
    }
}
```

### 4.2 ToolId

**职责**: 唯一标识一个工具

```rust
pub struct ToolId {
    pub name: String,
    pub version: Option<ToolVersion>,
}

impl ToolId {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: None,
        }
    }
    
    pub fn with_version(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: Some(ToolVersion::parse(version)?),
        }
    }
}
```

### 4.3 PluginId

**职责**: 唯一标识一个插件

```rust
pub struct PluginId(String);

impl PluginId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}
```

### 4.4 DataContext

**职责**: 存储工作流执行的数据上下文

```rust
pub struct DataContext {
    pub global: HashMap<String, Value>,    // 全局数据
    pub scopes: Vec<HashMap<String, Value>>, // 作用域栈
    pub slots: HashMap<String, SlotValue>, // 插槽数据
}

impl DataContext {
    pub fn get(&self, key: &str) -> Option<&Value>;
    pub fn set(&mut self, key: &str, value: Value);
    pub fn push_scope(&mut self);
    pub fn pop_scope(&mut self);
    pub fn get_slot(&self, name: &str) -> Option<&SlotValue>;
    pub fn set_slot(&mut self, name: &str, value: SlotValue);
}
```

### 4.5 WorkflowConfig

**职责**: 工作流配置

```rust
pub struct WorkflowConfig {
    pub concurrency: ConcurrencyConfig,   // 并发配置
    pub retry: RetryPolicy,               // 重试策略
    pub timeout: Option<Duration>,        // 超时设置
    pub resource_limits: ResourceLimits,  // 资源限制
    pub checkpoint: CheckpointConfig,     // 检查点配置
}

pub struct ConcurrencyConfig {
    pub max_parallel_nodes: usize,        // 最大并行节点数
    pub max_parallel_tools: usize,        // 最大并行工具数
}

pub struct RetryPolicy {
    pub max_retries: u32,                 // 最大重试次数
    pub backoff: BackoffStrategy,         // 退避策略
    pub retryable_errors: Vec<String>,    // 可重试错误
}

pub struct ResourceLimits {
    pub max_memory: Option<u64>,          // 最大内存
    pub max_cpu: Option<f32>,             // 最大 CPU
    pub max_execution_time: Option<Duration>, // 最大执行时间
}
```

### 4.6 PluginConfig

**职责**: 插件配置

```rust
pub struct PluginConfig {
    pub path: PathBuf,                    // 插件路径
    pub enabled: bool,                    // 是否启用
    pub priority: u32,                    // 优先级
    pub security: SecurityConfig,         // 安全配置
    pub resource_limits: ResourceLimits,  // 资源限制
    pub environment: HashMap<String, String>, // 环境变量
}

pub struct SecurityConfig {
    pub level: SandboxLevel,              // 安全级别
    pub file_permissions: FileSystemPermissions, // 文件权限
    pub network_permissions: NetworkPermissions, // 网络权限
}
```

### 4.7 ToolInput / ToolOutput

**职责**: 工具的输入输出

```rust
pub struct ToolInput {
    pub parameters: HashMap<String, Value>, // 参数
    pub context: Option<DataContext>,       // 上下文
}

pub struct ToolOutput {
    pub result: Value,                      // 结果
    pub status: ToolStatus,                 // 状态
    pub metadata: HashMap<String, Value>,   // 元数据
}

pub enum ToolStatus {
    Success,
    Failure(String),
    Timeout,
    Cancelled,
}
```

## 5. 枚举定义

### 5.1 ExecutionStatus

**职责**: 执行状态

```rust
pub enum ExecutionStatus {
    Pending,      // 等待中
    Running,      // 执行中
    Paused,       // 已暂停
    Completed,    // 已完成
    Failed,       // 已失败
    Cancelled,    // 已取消
    Timeout,      // 超时
}
```

**状态转换**:
```
Pending -> Running -> Completed
                  -> Failed
                  -> Cancelled
                  -> Timeout
         -> Paused -> Running
```

### 5.2 PluginType

**职责**: 插件类型

```rust
pub enum PluginType {
    Native,    // Rust 动态库
    Python,    // Python 脚本
    NodeJs,    // Node.js 脚本
    Docker,    // Docker 容器
    Wasm,      // WebAssembly 模块
}
```

### 5.3 PluginStatus

**职责**: 插件状态

```rust
pub enum PluginStatus {
    Unloaded,   // 未加载
    Loading,    // 加载中
    Loaded,     // 已加载
    Error,      // 错误
    Disabled,   // 已禁用
}
```

### 5.4 ToolKind

**职责**: 工具类别

```rust
pub enum ToolKind {
    Processor,   // 处理器
    Validator,   // 验证器
    Transformer, // 转换器
    Connector,   // 连接器
    Utility,     // 工具
}
```

### 5.5 SandboxLevel

**职责**: 沙箱安全级别

```rust
pub enum SandboxLevel {
    Unrestricted, // 无限制
    Basic,        // 基础隔离
    Strict,       // 严格隔离
    Maximum,      // 最大隔离
}
```

## 6. 领域事件

### 6.1 工作流事件

```rust
pub enum WorkflowEvent {
    WorkflowCreated { workflow_id: WorkflowId },
    WorkflowStarted { execution_id: ExecutionId },
    WorkflowPaused { execution_id: ExecutionId },
    WorkflowResumed { execution_id: ExecutionId },
    WorkflowCompleted { execution_id: ExecutionId, result: Value },
    WorkflowFailed { execution_id: ExecutionId, error: String },
    WorkflowCancelled { execution_id: ExecutionId },
}
```

### 6.2 节点事件

```rust
pub enum NodeEvent {
    NodeStarted { execution_id: ExecutionId, node_id: NodeId },
    NodeCompleted { execution_id: ExecutionId, node_id: NodeId, result: Value },
    NodeFailed { execution_id: ExecutionId, node_id: NodeId, error: String },
    NodeRetried { execution_id: ExecutionId, node_id: NodeId, retry_count: u32 },
}
```

### 6.3 插件事件

```rust
pub enum PluginEvent {
    PluginLoaded { plugin_id: PluginId },
    PluginUnloaded { plugin_id: PluginId },
    PluginError { plugin_id: PluginId, error: String },
    ToolRegistered { plugin_id: PluginId, tool_id: ToolId },
    ToolUnregistered { plugin_id: PluginId, tool_id: ToolId },
}
```

## 7. 领域服务

### 7.1 ExecutionStateCalculator

**职责**: 计算执行状态和进度

```rust
pub struct ExecutionStateCalculator;

impl ExecutionStateCalculator {
    pub fn calculate_progress(&self, execution: &WorkflowExecution) -> f32;
    pub fn calculate_status(&self, execution: &WorkflowExecution) -> ExecutionStatus;
    pub fn estimate_remaining_time(&self, execution: &WorkflowExecution) -> Option<Duration>;
}
```

### 7.2 DomainWorkflowValidator

**职责**: 验证工作流定义

```rust
pub struct DomainWorkflowValidator;

impl DomainWorkflowValidator {
    pub fn validate(&self, workflow: &WorkflowDefinition) -> Result<Vec<ValidationError>>;
    pub fn validate_node(&self, node: &WorkflowNode) -> Result<Vec<ValidationError>>;
    pub fn validate_edge(&self, edge: &WorkflowEdge, nodes: &[WorkflowNode]) -> Result<Vec<ValidationError>>;
}
```

## 8. 仓储接口

### 8.1 WorkflowRepository

```rust
pub trait WorkflowRepository: Send + Sync {
    async fn save(&self, workflow: &WorkflowDefinition) -> Result<()>;
    async fn load(&self, id: &WorkflowId) -> Result<Option<WorkflowDefinition>>;
    async fn list(&self) -> Result<Vec<WorkflowDefinition>>;
    async fn delete(&self, id: &WorkflowId) -> Result<()>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<WorkflowDefinition>>;
}
```

### 8.2 ExecutionRepository

```rust
pub trait ExecutionRepository: Send + Sync {
    async fn save(&self, execution: &WorkflowExecution) -> Result<()>;
    async fn load(&self, id: &ExecutionId) -> Result<Option<WorkflowExecution>>;
    async fn list(&self, filter: ExecutionFilter) -> Result<Vec<WorkflowExecution>>;
    async fn delete(&self, id: &ExecutionId) -> Result<()>;
    async fn get_history(&self, workflow_id: &WorkflowId) -> Result<Vec<WorkflowExecution>>;
}
```

### 8.3 PluginRepository

```rust
pub trait PluginRepository: Send + Sync {
    async fn save(&self, plugin: &PluginInfo) -> Result<()>;
    async fn load(&self, id: &PluginId) -> Result<Option<PluginInfo>>;
    async fn list(&self) -> Result<Vec<PluginInfo>>;
    async fn delete(&self, id: &PluginId) -> Result<()>;
}
```

## 9. 验证规则

### 9.1 工作流验证

- 工作流名称不能为空
- 必须有且仅有一个入口节点
- 所有节点 ID 必须唯一
- 所有边必须连接已存在的节点
- 不能存在循环依赖（除非是 Loop 节点）
- 条件边必须有条件表达式

### 9.2 节点验证

- 节点名称不能为空
- 节点类型必须有效
- 工具节点必须指定工具名称
- 条件节点必须有条件表达式
- 循环节点必须有循环配置

### 9.3 插件验证

- 插件名称不能为空
- 插件类型必须有效
- 插件路径必须存在
- 安全配置必须符合沙箱级别

## 10. 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0.0 | 2026-03-01 | 初始版本 | Architecture Team |
