# 接口契约规范

> **版本**: v0.1.0  
> **状态**: Active  
> **关联代码**: `src/domain/port/`

---

## 1. 工作流引擎接口

### 1.1 WorkflowEngine

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
```

**约束**:
- `execute` 必须是幂等的（相同输入产生相同结果）
- 并发执行数不得超过 `max_concurrent_workflows`
- 执行超时后必须返回 `Timeout` 状态

### 1.2 ExecutionManager

```rust
/// 执行管理器接口
#[async_trait]
pub trait ExecutionManager: Send + Sync {
    /// 提交工作流执行
    async fn submit(
        &self,
        workflow_id: String,
        parameters: HashMap<String, Value>,
    ) -> Result<ExecutionHandle>;
    
    /// 查询执行状态
    async fn query_status(&self, execution_id: String) -> Result<ExecutionStatus>;
    
    /// 取消执行
    async fn cancel(&self, execution_id: String) -> Result<()>;
    
    /// 暂停执行
    async fn pause(&self, execution_id: String) -> Result<()>;
    
    /// 恢复执行
    async fn resume(&self, execution_id: String) -> Result<()>;
}
```

---

## 2. 工具接口

### 2.1 ToolRegistry

```rust
/// 工具注册表接口
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// 注册工具
    async fn register(&self, tool: Tool) -> Result<()>;
    
    /// 注销工具
    async fn unregister(&self, tool_id: ToolId) -> Result<()>;
    
    /// 获取工具
    async fn get(&self, tool_id: &ToolId) -> Result<Option<Tool>>;
    
    /// 列出所有工具
    async fn list(&self) -> Result<Vec<Tool>>;
    
    /// 检查工具是否存在
    async fn exists(&self, tool_id: &ToolId) -> Result<bool>;
}
```

### 2.2 Tool 枚举

```rust
/// 工具枚举 - 统一抽象
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
    pub async fn execute(
        &self,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> Result<ToolOutput>;
    
    /// 获取工具元数据
    pub fn metadata(&self) -> &ToolMetadata;
    
    /// 验证输入
    pub fn validate_input(&self, params: &Value) -> Result<()>;
}
```

**约束**:
- `execute` 必须是线程安全的 (Send + Sync)
- 执行时间不得超过 `timeout_secs`
- 资源使用不得超过 `ResourceRequirements` 定义

---

## 3. 插件接口

### 3.1 Plugin

```rust
/// 插件接口
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件信息
    fn info(&self) -> &PluginInfo;
    
    /// 初始化插件
    async fn initialize(&mut self) -> Result<()>;
    
    /// 启动插件
    async fn start(&self) -> Result<()>;
    
    /// 停止插件
    async fn stop(&self) -> Result<()>;
    
    /// 获取插件提供的工具
    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>>;
    
    /// 检查插件健康状态
    async fn health_check(&self) -> Result<HealthStatus>;
}
```

### 3.2 PluginManager

```rust
/// 插件管理器接口
#[async_trait]
pub trait PluginManager: Send + Sync {
    /// 加载插件
    async fn load(&self, source: PluginSource) -> Result<PluginId>;
    
    /// 卸载插件
    async fn unload(&self, plugin_id: PluginId) -> Result<()>;
    
    /// 重新加载插件
    async fn reload(&self, plugin_id: PluginId) -> Result<()>;
    
    /// 获取插件
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<Arc<dyn Plugin>>>;
    
    /// 列出所有插件
    async fn list(&self) -> Result<Vec<Arc<dyn Plugin>>>;
}
```

---

## 4. 存储接口

### 4.1 WorkflowRepository

```rust
/// 工作流仓库接口
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    /// 保存工作流定义
    async fn save_definition(&self, definition: &WorkflowDefinition) -> Result<()>;
    
    /// 获取工作流定义
    async fn get_definition(&self, id: &str) -> Result<Option<WorkflowDefinition>>;
    
    /// 删除工作流定义
    async fn delete_definition(&self, id: &str) -> Result<()>;
    
    /// 列出工作流定义
    async fn list_definitions(&self) -> Result<Vec<WorkflowDefinition>>;
    
    /// 保存执行记录
    async fn save_execution(&self, execution: &WorkflowExecution) -> Result<()>;
    
    /// 获取执行记录
    async fn get_execution(&self, id: &str) -> Result<Option<WorkflowExecution>>;
}
```

### 4.2 StateManager

```rust
/// 状态管理器接口
#[async_trait]
pub trait StateManager: Send + Sync {
    /// 保存状态
    async fn save_state(&self, key: &str, state: &State) -> Result<()>;
    
    /// 加载状态
    async fn load_state(&self, key: &str) -> Result<Option<State>>;
    
    /// 删除状态
    async fn delete_state(&self, key: &str) -> Result<()>;
    
    /// 创建检查点
    async fn create_checkpoint(&self, execution_id: &str) -> Result<Checkpoint>;
    
    /// 从检查点恢复
    async fn restore_from_checkpoint(&self, checkpoint: &Checkpoint) -> Result<State>;
}
```

---

## 5. 数据类型定义

### 5.1 核心类型

```rust
/// 工作流定义
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub input_schema: Option<Value>,
    pub output_schema: Option<Value>,
}

/// 工作流节点
pub struct WorkflowNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub component: Box<dyn Component>,
    pub config: NodeConfig,
}

/// 工作流边
pub struct WorkflowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub condition: Option<Condition>,
}

/// 执行状态
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// 工具输入
pub struct ToolInput {
    pub params: Value,
    pub context: ExecutionContext,
}

/// 工具输出
pub struct ToolOutput {
    pub success: bool,
    pub result: Value,
    pub execution_time: Duration,
    pub metadata: HashMap<String, String>,
}

/// 插件信息
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub plugin_type: PluginType,
}

/// 插件类型
pub enum PluginType {
    Native,
    Python,
    NodeJs,
    Docker,
    Wasm,
}
```

### 5.2 配置类型

```rust
/// 资源需求
pub struct ResourceRequirements {
    pub min_memory_mb: u64,
    pub recommended_memory_mb: u64,
    pub cpu_intensity: u8,  // 1-10
    pub network_required: bool,
    pub estimated_duration_ms: u64,
}

/// 重试策略
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: Option<u64>,
    pub backoff_strategy: BackoffStrategy,
}

/// 退避策略
pub enum BackoffStrategy {
    Fixed,
    Linear,
    Exponential,
    Jitter { max_jitter_ms: u64 },
}

/// 缓存策略
pub struct CachePolicy {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub key_pattern: String,
}

/// 安全策略
pub struct SecurityPolicy {
    pub strict_mode: bool,
    pub allow_network: bool,
    pub allow_file_system: bool,
    pub allowed_paths: Vec<String>,
}
```

---

## 6. 错误码定义

### 6.1 系统错误码

| 错误码 | 描述 | HTTP状态码 |
|--------|------|-----------|
| `E0001` | 内部系统错误 | 500 |
| `E0002` | 服务不可用 | 503 |
| `E0003` | 请求超时 | 408 |
| `E0004` | 资源耗尽 | 503 |

### 6.2 工作流错误码

| 错误码 | 描述 | HTTP状态码 |
|--------|------|-----------|
| `E1001` | 工作流验证失败 | 400 |
| `E1002` | 工作流未找到 | 404 |
| `E1003` | 执行失败 | 422 |
| `E1004` | 执行超时 | 408 |
| `E1005` | 执行被取消 | 409 |
| `E1006` | 循环依赖 | 400 |

### 6.3 工具错误码

| 错误码 | 描述 | HTTP状态码 |
|--------|------|-----------|
| `E2001` | 工具未找到 | 404 |
| `E2002` | 参数验证失败 | 400 |
| `E2003` | 执行超时 | 408 |
| `E2004` | 执行失败 | 422 |
| `E2005` | 输出验证失败 | 500 |

### 6.4 插件错误码

| 错误码 | 描述 | HTTP状态码 |
|--------|------|-----------|
| `E3001` | 插件加载失败 | 500 |
| `E3002` | 插件未找到 | 404 |
| `E3003` | 依赖安装失败 | 500 |
| `E3004` | 安全验证失败 | 403 |
| `E3005` | 插件正在使用 | 409 |

---

## 7. 版本兼容性

### 7.1 API 版本策略

- 公共 API 必须保持向后兼容
- 破坏性变更必须增加主版本号
- 废弃 API 必须标记 `#[deprecated]` 并保留至少一个版本

### 7.2 数据版本策略

- 序列化数据必须包含版本字段
- 支持至少两个版本的向后兼容读取

---

## 8. 性能约束

### 8.1 响应时间目标

| 操作 | 目标 | 最大容忍 |
|------|------|----------|
| 工作流提交 | < 10ms | 100ms |
| 节点调度 | < 1ms | 10ms |
| 工具执行 | 取决于工具 | 可配置 timeout |
| 状态查询 | < 5ms | 50ms |

### 8.2 并发约束

```rust
pub struct ConcurrencyConfig {
    pub max_concurrent_workflows: usize,  // 默认: 10
    pub max_concurrent_tasks: usize,      // 默认: 50
    pub task_queue_size: usize,           // 默认: 100
}
```

- 超过 `max_concurrent_workflows` 应返回 `Backpressure` 错误
- 任务队列满时应触发背压机制
