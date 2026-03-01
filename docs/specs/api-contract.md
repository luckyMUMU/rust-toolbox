# Interface Contract Specification（接口契约规范）

> **版本**: v0.1.0  
> **状态**: Active  
> **关联代码**: `src/domain/port/`

---

## 1. Workflow Engine Interface（工作流引擎接口）

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
- `execute` 必须是幂等的（Idempotent，相同输入产生相同结果）
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

## 2. Tool Interface（工具接口）

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

### 2.2 Tool Enum（工具枚举）

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

## 3. Plugin Interface（插件接口）

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

## 4. Storage Interface（存储接口）

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

## 5. Data Type Definition（数据类型定义）

### 5.1 Core Types（核心类型）

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

### 5.2 Configuration Types（配置类型）

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

## 6. Error Code Definition（错误码定义）

### 6.1 错误码设计原则

**错误码格式**: `E{模块码}{序号}`
- 模块码: 0=系统, 1=工作流, 2=工具, 3=插件, 4=存储, 5=认证
- 序号: 两位数字，按顺序递增

**错误响应格式**:
```json
{
  "code": "E1001",
  "message": "工作流验证失败",
  "details": "节点 'node_1' 缺少必需的 tool_name 字段",
  "timestamp": "2026-02-26T10:30:00Z",
  "request_id": "req_abc123"
}
```

### 6.2 System Error Codes（系统错误码）

| 错误码 | 名称 | 描述 | HTTP状态码 | 处理建议 |
|--------|------|------|------------|----------|
| `E0001` | `InternalError` | 内部系统错误 | 500 | 检查系统日志，联系管理员 |
| `E0002` | `ServiceUnavailable` | 服务不可用 | 503 | 稍后重试，检查服务状态 |
| `E0003` | `RequestTimeout` | 请求超时 | 408 | 增加超时时间或优化操作 |
| `E0004` | `ResourceExhausted` | 资源耗尽 | 503 | 释放资源或扩展容量 |
| `E0005` | `ConfigurationError` | 配置错误 | 500 | 检查配置文件和环境变量 |
| `E0006` | `SerializationError` | 序列化错误 | 400 | 检查数据格式 |
| `E0007` | `ConcurrentAccess` | 并发访问冲突 | 409 | 重试或使用锁机制 |

### 6.3 Workflow Error Codes（工作流错误码）

| 错误码 | 名称 | 描述 | HTTP状态码 | 处理建议 |
|--------|------|------|------------|----------|
| `E1001` | `WorkflowValidation` | 工作流验证失败 | 400 | 检查工作流定义格式 |
| `E1002` | `WorkflowNotFound` | 工作流未找到 | 404 | 确认工作流ID正确 |
| `E1003` | `WorkflowExecution` | 执行失败 | 422 | 检查执行日志 |
| `E1004` | `ExecutionTimeout` | 执行超时 | 408 | 增加超时配置 |
| `E1005` | `ExecutionCancelled` | 执行被取消 | 409 | 检查取消原因 |
| `E1006` | `CircularDependency` | 循环依赖 | 400 | 检查节点依赖关系 |
| `E1007` | `DuplicateNodeId` | 节点ID重复 | 400 | 使用唯一节点ID |
| `E1008` | `NodeNotFound` | 节点未找到 | 404 | 确认节点ID存在 |
| `E1009` | `InvalidNodeId` | 无效节点ID | 400 | 使用有效ID格式 |
| `E1010` | `EmptyWorkflow` | 空工作流 | 400 | 添加至少一个节点 |
| `E1011` | `InvalidEdge` | 无效边定义 | 400 | 检查边的source/target |
| `E1012` | `InvalidStateTransition` | 无效状态转换 | 400 | 检查状态机逻辑 |
| `E1013` | `ParameterResolution` | 参数解析错误 | 400 | 检查参数引用语法 |
| `E1014` | `ComponentNotFound` | 组件未找到 | 404 | 确认组件已注册 |

### 6.4 Tool Error Codes（工具错误码）

| 错误码 | 名称 | 描述 | HTTP状态码 | 处理建议 |
|--------|------|------|------------|----------|
| `E2001` | `ToolNotFound` | 工具未找到 | 404 | 确认工具名称正确 |
| `E2002` | `ToolValidation` | 参数验证失败 | 400 | 检查参数格式和类型 |
| `E2003` | `ToolTimeout` | 执行超时 | 408 | 增加工具超时配置 |
| `E2004` | `ToolExecution` | 执行失败 | 422 | 检查工具实现 |
| `E2005` | `ToolOutputValidation` | 输出验证失败 | 500 | 检查输出Schema |
| `E2006` | `ToolRegistration` | 工具注册失败 | 500 | 检查工具元数据 |
| `E2007` | `MissingToolName` | 缺少工具名称 | 400 | 添加tool_name字段 |
| `E2008` | `ToolNotImplemented` | 工具未实现 | 501 | 等待功能开发 |

### 6.5 Plugin Error Codes（插件错误码）

| 错误码 | 名称 | 描述 | HTTP状态码 | 处理建议 |
|--------|------|------|------------|----------|
| `E3001` | `PluginInitialization` | 插件初始化失败 | 500 | 检查插件依赖 |
| `E3002` | `PluginNotFound` | 插件未找到 | 404 | 确认插件ID正确 |
| `E3003` | `PluginDependency` | 依赖安装失败 | 500 | 检查网络和依赖源 |
| `E3004` | `PluginSecurity` | 安全验证失败 | 403 | 检查安全策略配置 |
| `E3005` | `PluginInUse` | 插件正在使用 | 409 | 先卸载再操作 |
| `E3006` | `PluginAlreadyLoaded` | 插件已加载 | 409 | 跳过或先卸载 |
| `E3007` | `PluginConfiguration` | 插件配置错误 | 400 | 检查配置格式 |
| `E3008` | `PluginRuntime` | 插件运行时错误 | 500 | 检查插件实现 |
| `E3009` | `PluginPoolExhausted` | 运行时池耗尽 | 503 | 增加池大小 |
| `E3010` | `PluginResourceLimit` | 资源限制超出 | 503 | 调整资源限制 |

### 6.6 Storage Error Codes（存储错误码）

| 错误码 | 名称 | 描述 | HTTP状态码 | 处理建议 |
|--------|------|------|------------|----------|
| `E4001` | `StorageIO` | 存储IO错误 | 500 | 检查磁盘和权限 |
| `E4002` | `StorageNotFound` | 存储资源未找到 | 404 | 确认路径正确 |
| `E4003` | `StorageCorrupted` | 数据损坏 | 500 | 从备份恢复 |
| `E4004` | `StorageBackup` | 备份失败 | 500 | 检查备份路径 |
| `E4005` | `CheckpointFailed` | 检查点创建失败 | 500 | 检查存储空间 |

### 6.7 Authentication Error Codes（认证错误码）

| 错误码 | 名称 | 描述 | HTTP状态码 | 处理建议 |
|--------|------|------|------------|----------|
| `E5001` | `Authentication` | 认证失败 | 401 | 检查凭据 |
| `E5002` | `PermissionDenied` | 权限不足 | 403 | 联系管理员授权 |
| `E5003` | `TokenExpired` | 令牌过期 | 401 | 刷新令牌 |
| `E5004` | `InvalidToken` | 无效令牌 | 401 | 重新登录 |

### 6.8 错误处理最佳实践

#### Rust 代码示例

```rust
use crate::error::{WorkflowError, CommonError};

// 使用 ? 操作符传播错误
async fn execute_workflow(def: WorkflowDefinition) -> Result<WorkflowExecution> {
    // 验证工作流
    validate_definition(&def)?;
    
    // 执行节点
    for node in &def.nodes {
        execute_node(node).await?;
    }
    
    Ok(WorkflowExecution::completed())
}

// 使用 match 处理特定错误
fn handle_error(error: WorkflowError) -> Response {
    match error {
        WorkflowError::WorkflowValidation { message } => {
            Response::bad_request("E1001", &message)
        }
        WorkflowError::NotFound { resource } => {
            Response::not_found("E0001", &format!("Resource not found: {}", resource))
        }
        WorkflowError::Timeout { duration } => {
            Response::timeout("E0003", &format!("Operation timed out after {:?}", duration))
        }
        _ => Response::internal_error("E0001", &error.to_string())
    }
}

// 使用 CommonError 统一错误
fn validate_input(data: &Value) -> std::result::Result<(), CommonError> {
    if data.get("id").is_none() {
        return Err(CommonError::validation("input", "missing required field: id"));
    }
    Ok(())
}
```

#### 错误恢复策略

| 错误类型 | 恢复策略 | 示例 |
|----------|----------|------|
| 可重试错误 | 自动重试 + 指数退避 | 网络超时、服务暂时不可用 |
| 可恢复错误 | 用户干预后继续 | 参数验证失败、权限不足 |
| 不可恢复错误 | 记录日志 + 终止 | 数据损坏、系统配置错误 |

#### 错误日志规范

```rust
// 推荐：结构化日志
tracing::error!(
    error_code = "E1003",
    workflow_id = %workflow.id,
    node_id = %node.id,
    "Workflow execution failed"
);

// 推荐：包含上下文
tracing::warn!(
    error_code = "E2003",
    tool_name = %tool_name,
    timeout_secs = timeout.as_secs(),
    "Tool execution timed out"
);
```

---

## 7. Version Compatibility（版本兼容性）

### 7.1 API Version Policy（API 版本策略）

- Public API（公共 API）必须保持向后兼容（Backward Compatible）
- Breaking Changes（破坏性变更）必须增加主版本号
- Deprecated API（废弃 API）必须标记 `#[deprecated]` 并保留至少一个版本

### 7.2 Data Version Policy（数据版本策略）

- 序列化数据必须包含版本字段
- 支持至少两个版本的向后兼容读取

---

## 8. Performance Constraints（性能约束）

### 8.1 Response Time Targets（响应时间目标）

| Operation（操作） | Target（目标） | Maximum Tolerance（最大容忍） |
|------|------|----------|
| Workflow Submission（工作流提交） | < 10ms | 100ms |
| Node Scheduling（节点调度） | < 1ms | 10ms |
| Tool Execution（工具执行） | 取决于工具 | 可配置 timeout |
| Status Query（状态查询） | < 5ms | 50ms |

### 8.2 Concurrency Constraints（并发约束）

```rust
pub struct ConcurrencyConfig {
    pub max_concurrent_workflows: usize,  // 默认: 10
    pub max_concurrent_tasks: usize,      // 默认: 50
    pub task_queue_size: usize,           // 默认: 100
}
```

- 超过 `max_concurrent_workflows` 应返回 `Backpressure` 错误
- 任务队列满时应触发背压（Backpressure）机制
