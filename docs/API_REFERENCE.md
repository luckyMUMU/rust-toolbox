# API参考文档

## 概述

本文档提供工作流工具包的完整API参考，包括核心API接口、CLI命令、配置文件格式和错误处理指南。

## 核心API接口

### 工作流引擎API

#### WorkflowEngine Trait

工作流执行引擎的核心接口。

```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 执行工作流
    async fn execute_workflow(&self, definition: WorkflowDefinition) -> Result<WorkflowExecution>;
    
    /// 暂停工作流
    async fn pause_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// 恢复工作流
    async fn resume_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// 停止工作流
    async fn stop_workflow(&self, id: WorkflowId) -> Result<()>;
    
    /// 获取工作流状态
    async fn get_workflow_status(&self, id: WorkflowId) -> Result<WorkflowStatus>;
    
    /// 列出所有工作流
    async fn list_workflows(&self) -> Result<Vec<WorkflowInfo>>;
    
    /// 删除工作流
    async fn delete_workflow(&self, id: WorkflowId) -> Result<()>;
}
```

#### 数据结构

##### WorkflowDefinition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// 工作流名称
    pub name: String,
    
    /// 版本号
    pub version: String,
    
    /// 描述信息
    pub description: Option<String>,
    
    /// 元数据
    pub metadata: HashMap<String, Value>,
    
    /// 工作流节点
    pub nodes: Vec<WorkflowNode>,
    
    /// 节点连接
    pub edges: Vec<WorkflowEdge>,
    
    /// 全局配置
    pub global_config: WorkflowConfig,
}
```

##### WorkflowNode

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// 节点ID
    pub id: String,
    
    /// 节点类型
    pub node_type: NodeType,
    
    /// 工具名称（对于工具节点）
    pub tool_name: Option<String>,
    
    /// 节点参数
    pub parameters: Value,
    
    /// 重试策略
    pub retry_policy: Option<RetryPolicy>,
    
    /// 超时设置
    pub timeout: Option<Duration>,
    
    /// 条件表达式（对于条件节点）
    pub condition: Option<String>,
}
```

##### NodeType

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// 工具节点
    Tool,
    
    /// 条件节点
    Condition,
    
    /// 循环节点
    Loop,
    
    /// 并行节点
    Parallel,
    
    /// 检查点节点
    Checkpoint,
    
    /// 开始节点
    Start,
    
    /// 结束节点
    End,
}
```

##### WorkflowExecution

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// 执行ID
    pub id: WorkflowId,
    
    /// 工作流名称
    pub workflow_name: String,
    
    /// 执行状态
    pub status: ExecutionStatus,
    
    /// 开始时间
    pub started_at: DateTime<Utc>,
    
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    
    /// 当前节点
    pub current_node: Option<String>,
    
    /// 节点状态
    pub node_states: HashMap<String, NodeExecutionState>,
    
    /// 全局上下文
    pub global_context: Value,
    
    /// 执行结果
    pub result: Option<Value>,
    
    /// 错误信息
    pub error: Option<String>,
}
```

##### ExecutionStatus

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// 等待中
    Pending,
    
    /// 运行中
    Running,
    
    /// 已暂停
    Paused,
    
    /// 已完成
    Completed,
    
    /// 已失败
    Failed,
    
    /// 已取消
    Cancelled,
}
```

### 工具注册表API

#### ToolRegistry Trait

```rust
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// 注册工具
    async fn register_tool(&mut self, tool: Box<dyn ToolNode>) -> Result<()>;
    
    /// 获取工具
    fn get_tool(&self, name: &str) -> Option<&dyn ToolNode>;
    
    /// 列出所有工具
    fn list_tools(&self) -> Vec<ToolInfo>;
    
    /// 执行工具
    async fn execute_tool(&self, name: &str, params: Value) -> Result<Value>;
    
    /// 验证工具参数
    fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()>;
    
    /// 获取工具schema
    fn get_tool_schema(&self, name: &str) -> Option<ToolDefinition>;
    
    /// 卸载工具
    async fn unregister_tool(&mut self, name: &str) -> Result<()>;
}
```

#### ToolNode Trait

```rust
#[async_trait]
pub trait ToolNode: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;
    
    /// 工具版本
    fn version(&self) -> &str;
    
    /// 执行工具
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;
    
    /// 验证参数
    fn validate_parameters(&self, params: &Value) -> Result<()>;
    
    /// 获取工具定义
    fn get_schema(&self) -> ToolDefinition;
    
    /// 获取插件信息
    fn get_plugin_info(&self) -> Option<&PluginInfo>;
}
```

#### 数据结构

##### ToolDefinition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    
    /// 版本号
    pub version: String,
    
    /// 描述信息
    pub description: String,
    
    /// 参数schema
    pub parameters_schema: Value,
    
    /// 返回值schema
    pub return_schema: Value,
    
    /// 依赖列表
    pub dependencies: Vec<String>,
    
    /// 元数据
    pub metadata: HashMap<String, Value>,
    
    /// 插件信息
    pub plugin_info: Option<PluginInfo>,
}
```

##### ExecutionContext

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// 工作流ID
    pub workflow_id: Option<WorkflowId>,
    
    /// 执行ID
    pub execution_id: Option<String>,
    
    /// 节点ID
    pub node_id: Option<String>,
    
    /// 全局变量
    pub global_variables: HashMap<String, Value>,
    
    /// 环境变量
    pub environment: HashMap<String, String>,
    
    /// 配置信息
    pub config: Value,
    
    /// 超时设置
    pub timeout: Option<Duration>,
}
```

### 插件系统API

#### Plugin Trait

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    
    /// 插件版本
    fn version(&self) -> &str;
    
    /// 初始化插件
    async fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    
    /// 获取工具列表
    fn get_tools(&self) -> Vec<Box<dyn ToolNode>>;
    
    /// 关闭插件
    async fn shutdown(&mut self) -> Result<()>;
    
    /// 获取插件信息
    fn get_info(&self) -> PluginInfo;
    
    /// 健康检查
    async fn health_check(&self) -> Result<PluginHealth>;
}
```

#### PluginManager

```rust
impl PluginManager {
    /// 创建插件管理器
    pub fn new() -> Self;
    
    /// 加载插件
    pub async fn load_plugin(&mut self, name: &str, config: PluginConfig) -> Result<()>;
    
    /// 卸载插件
    pub async fn unload_plugin(&mut self, name: &str) -> Result<()>;
    
    /// 重新加载插件
    pub async fn reload_plugin(&mut self, name: &str) -> Result<()>;
    
    /// 列出插件
    pub fn list_plugins(&self) -> Vec<PluginInfo>;
    
    /// 获取插件
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin>;
    
    /// 获取所有工具
    pub fn get_all_tools(&self) -> Vec<Box<dyn ToolNode>>;
}
```

#### 数据结构

##### PluginConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 插件类型
    pub plugin_type: String,
    
    /// 插件路径
    pub plugin_path: PathBuf,
    
    /// 配置参数
    pub config: Value,
    
    /// 环境变量
    pub environment: HashMap<String, String>,
    
    /// 资源限制
    pub resource_limits: Option<ResourceLimits>,
    
    /// 安全设置
    pub security_config: Option<SecurityConfig>,
}
```

##### PluginInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    
    /// 版本号
    pub version: String,
    
    /// 插件类型
    pub plugin_type: PluginType,
    
    /// 描述信息
    pub description: String,
    
    /// 作者信息
    pub author: Option<String>,
    
    /// 许可证
    pub license: Option<String>,
    
    /// 主页URL
    pub homepage: Option<String>,
    
    /// 工具列表
    pub tools: Vec<String>,
    
    /// 状态
    pub status: PluginStatus,
}
```

### 存储系统API

#### StorageBackend Trait

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// 保存数据
    async fn save(&self, key: &str, value: &[u8]) -> Result<()>;
    
    /// 加载数据
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// 删除数据
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// 列出键
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
    
    /// 检查存在
    async fn exists(&self, key: &str) -> Result<bool>;
    
    /// 批量保存
    async fn batch_save(&self, items: Vec<(String, Vec<u8>)>) -> Result<()>;
    
    /// 批量加载
    async fn batch_load(&self, keys: Vec<String>) -> Result<Vec<Option<Vec<u8>>>>;
    
    /// 批量删除
    async fn batch_delete(&self, keys: Vec<String>) -> Result<()>;
}
```

#### StateManager

```rust
impl StateManager {
    /// 创建状态管理器
    pub fn new(storage: Arc<dyn StorageBackend>, cache: Arc<dyn CacheBackend>) -> Self;
    
    /// 保存工作流状态
    pub async fn save_workflow_state(&self, id: WorkflowId, state: WorkflowState) -> Result<()>;
    
    /// 加载工作流状态
    pub async fn load_workflow_state(&self, id: WorkflowId) -> Result<Option<WorkflowState>>;
    
    /// 删除工作流状态
    pub async fn delete_workflow_state(&self, id: WorkflowId) -> Result<()>;
    
    /// 保存执行历史
    pub async fn save_execution_record(&self, record: ExecutionRecord) -> Result<()>;
    
    /// 获取执行历史
    pub async fn get_execution_history(&self, id: WorkflowId) -> Result<Vec<ExecutionRecord>>;
    
    /// 清理旧数据
    pub async fn cleanup_old_data(&self, retention_policy: RetentionPolicy) -> Result<()>;
}
```

## CLI命令参考

### 全局选项

```bash
workflow-toolkit [OPTIONS] <COMMAND>

OPTIONS:
    -c, --config <FILE>     配置文件路径 [default: config/default.toml]
    -v, --verbose           详细输出
    -q, --quiet             静默模式
    -h, --help              显示帮助信息
    -V, --version           显示版本信息
        --log-level <LEVEL> 日志级别 [default: info] [possible: trace, debug, info, warn, error]
```

### 工作流管理命令

#### workflow create

创建新的工作流定义。

```bash
workflow-toolkit workflow create [OPTIONS] <DEFINITION_FILE>

ARGS:
    <DEFINITION_FILE>    工作流定义文件路径

OPTIONS:
    -n, --name <NAME>        工作流名称（覆盖文件中的名称）
    -d, --description <DESC> 工作流描述
    -v, --validate           仅验证不保存
    -f, --force              强制覆盖已存在的工作流
    -h, --help               显示帮助信息

EXAMPLES:
    # 创建工作流
    workflow-toolkit workflow create my-workflow.yaml
    
    # 仅验证工作流定义
    workflow-toolkit workflow create --validate my-workflow.yaml
    
    # 强制覆盖已存在的工作流
    workflow-toolkit workflow create --force my-workflow.yaml
```

#### workflow list

列出所有工作流。

```bash
workflow-toolkit workflow list [OPTIONS]

OPTIONS:
    -f, --format <FORMAT>    输出格式 [default: table] [possible: table, json, yaml]
    -s, --status <STATUS>    按状态过滤 [possible: pending, running, paused, completed, failed]
    -l, --limit <LIMIT>      限制输出数量
        --sort <FIELD>       排序字段 [default: name] [possible: name, created, status]
    -h, --help               显示帮助信息

EXAMPLES:
    # 列出所有工作流
    workflow-toolkit workflow list
    
    # 以JSON格式输出
    workflow-toolkit workflow list --format json
    
    # 只显示运行中的工作流
    workflow-toolkit workflow list --status running
```

#### workflow execute

执行工作流。

```bash
workflow-toolkit workflow execute [OPTIONS] <WORKFLOW_NAME>

ARGS:
    <WORKFLOW_NAME>    工作流名称

OPTIONS:
    -p, --params <FILE>      参数文件路径
    -e, --env <KEY=VALUE>    环境变量
    -w, --wait               等待执行完成
    -t, --timeout <SECONDS>  执行超时时间
        --dry-run            模拟执行（不实际运行）
    -h, --help               显示帮助信息

EXAMPLES:
    # 执行工作流
    workflow-toolkit workflow execute my-workflow
    
    # 使用参数文件执行
    workflow-toolkit workflow execute --params params.json my-workflow
    
    # 等待执行完成
    workflow-toolkit workflow execute --wait my-workflow
    
    # 模拟执行
    workflow-toolkit workflow execute --dry-run my-workflow
```

#### workflow status

查看工作流执行状态。

```bash
workflow-toolkit workflow status [OPTIONS] <EXECUTION_ID>

ARGS:
    <EXECUTION_ID>    执行ID

OPTIONS:
    -f, --format <FORMAT>    输出格式 [default: table] [possible: table, json, yaml]
    -w, --watch              实时监控状态变化
    -i, --interval <SECONDS> 监控间隔 [default: 2]
    -h, --help               显示帮助信息

EXAMPLES:
    # 查看执行状态
    workflow-toolkit workflow status abc123
    
    # 实时监控
    workflow-toolkit workflow status --watch abc123
    
    # JSON格式输出
    workflow-toolkit workflow status --format json abc123
```

#### workflow pause/resume/stop

控制工作流执行。

```bash
# 暂停工作流
workflow-toolkit workflow pause <EXECUTION_ID>

# 恢复工作流
workflow-toolkit workflow resume <EXECUTION_ID>

# 停止工作流
workflow-toolkit workflow stop [OPTIONS] <EXECUTION_ID>

OPTIONS (for stop):
    -f, --force    强制停止
    -h, --help     显示帮助信息

EXAMPLES:
    # 暂停执行
    workflow-toolkit workflow pause abc123
    
    # 恢复执行
    workflow-toolkit workflow resume abc123
    
    # 停止执行
    workflow-toolkit workflow stop abc123
    
    # 强制停止
    workflow-toolkit workflow stop --force abc123
```

#### workflow delete

删除工作流。

```bash
workflow-toolkit workflow delete [OPTIONS] <WORKFLOW_NAME>

ARGS:
    <WORKFLOW_NAME>    工作流名称

OPTIONS:
    -f, --force              强制删除（不确认）
        --delete-executions  同时删除执行历史
    -h, --help               显示帮助信息

EXAMPLES:
    # 删除工作流
    workflow-toolkit workflow delete my-workflow
    
    # 强制删除包括执行历史
    workflow-toolkit workflow delete --force --delete-executions my-workflow
```

### 工具管理命令

#### tool list

列出所有可用工具。

```bash
workflow-toolkit tool list [OPTIONS]

OPTIONS:
    -f, --format <FORMAT>    输出格式 [default: table] [possible: table, json, yaml]
    -p, --plugin <PLUGIN>    按插件过滤
    -c, --category <CAT>     按类别过滤
        --show-schema        显示工具schema
    -h, --help               显示帮助信息

EXAMPLES:
    # 列出所有工具
    workflow-toolkit tool list
    
    # 显示工具schema
    workflow-toolkit tool list --show-schema
    
    # 按插件过滤
    workflow-toolkit tool list --plugin python-tools
```

#### tool execute

独立执行工具。

```bash
workflow-toolkit tool execute [OPTIONS] <TOOL_NAME>

ARGS:
    <TOOL_NAME>    工具名称

OPTIONS:
    -p, --params <PARAMS>    参数（JSON格式）
    -f, --params-file <FILE> 参数文件
    -o, --output <FILE>      输出文件
        --format <FORMAT>    输出格式 [default: json] [possible: json, yaml, table]
    -h, --help               显示帮助信息

EXAMPLES:
    # 执行工具
    workflow-toolkit tool execute calculator --params '{"operation":"add","a":5,"b":3}'
    
    # 使用参数文件
    workflow-toolkit tool execute calculator --params-file params.json
    
    # 输出到文件
    workflow-toolkit tool execute calculator --params '{}' --output result.json
```

#### tool schema

查看工具schema。

```bash
workflow-toolkit tool schema [OPTIONS] <TOOL_NAME>

ARGS:
    <TOOL_NAME>    工具名称

OPTIONS:
    -f, --format <FORMAT>    输出格式 [default: json] [possible: json, yaml]
    -h, --help               显示帮助信息

EXAMPLES:
    # 查看工具schema
    workflow-toolkit tool schema calculator
    
    # YAML格式输出
    workflow-toolkit tool schema --format yaml calculator
```

### 插件管理命令

#### plugin list

列出所有插件。

```bash
workflow-toolkit plugin list [OPTIONS]

OPTIONS:
    -f, --format <FORMAT>    输出格式 [default: table] [possible: table, json, yaml]
    -s, --status <STATUS>    按状态过滤 [possible: loaded, unloaded, error]
    -t, --type <TYPE>        按类型过滤 [possible: native, python, nodejs, docker, wasm]
    -h, --help               显示帮助信息

EXAMPLES:
    # 列出所有插件
    workflow-toolkit plugin list
    
    # 只显示已加载的插件
    workflow-toolkit plugin list --status loaded
    
    # 只显示Python插件
    workflow-toolkit plugin list --type python
```

#### plugin install

安装插件。

```bash
workflow-toolkit plugin install [OPTIONS] <PLUGIN_SOURCE>

ARGS:
    <PLUGIN_SOURCE>    插件源（路径、URL或包名）

OPTIONS:
    -t, --type <TYPE>        插件类型 [possible: native, python, nodejs, docker, wasm]
    -n, --name <NAME>        插件名称
    -c, --config <CONFIG>    配置参数（JSON格式）
        --force              强制安装（覆盖已存在）
    -h, --help               显示帮助信息

EXAMPLES:
    # 安装本地插件
    workflow-toolkit plugin install --type python ./my-plugin
    
    # 安装远程插件
    workflow-toolkit plugin install --type docker my-plugin:latest
    
    # 使用配置安装
    workflow-toolkit plugin install --config '{"env":"prod"}' ./my-plugin
```

#### plugin uninstall

卸载插件。

```bash
workflow-toolkit plugin uninstall [OPTIONS] <PLUGIN_NAME>

ARGS:
    <PLUGIN_NAME>    插件名称

OPTIONS:
    -f, --force    强制卸载
    -h, --help     显示帮助信息

EXAMPLES:
    # 卸载插件
    workflow-toolkit plugin uninstall my-plugin
    
    # 强制卸载
    workflow-toolkit plugin uninstall --force my-plugin
```

#### plugin reload

重新加载插件。

```bash
workflow-toolkit plugin reload [OPTIONS] <PLUGIN_NAME>

ARGS:
    <PLUGIN_NAME>    插件名称

OPTIONS:
    -h, --help    显示帮助信息

EXAMPLES:
    # 重新加载插件
    workflow-toolkit plugin reload my-plugin
```

### 系统管理命令

#### config show

显示当前配置。

```bash
workflow-toolkit config show [OPTIONS]

OPTIONS:
    -f, --format <FORMAT>    输出格式 [default: toml] [possible: toml, json, yaml]
    -s, --section <SECTION>  显示特定配置段
    -h, --help               显示帮助信息

EXAMPLES:
    # 显示完整配置
    workflow-toolkit config show
    
    # 显示特定配置段
    workflow-toolkit config show --section storage
    
    # JSON格式输出
    workflow-toolkit config show --format json
```

#### config validate

验证配置文件。

```bash
workflow-toolkit config validate [OPTIONS] [CONFIG_FILE]

ARGS:
    [CONFIG_FILE]    配置文件路径 [default: config/default.toml]

OPTIONS:
    -h, --help    显示帮助信息

EXAMPLES:
    # 验证默认配置
    workflow-toolkit config validate
    
    # 验证指定配置文件
    workflow-toolkit config validate my-config.toml
```

#### server start

启动MCP服务器。

```bash
workflow-toolkit server start [OPTIONS]

OPTIONS:
    -p, --port <PORT>        HTTP端口 [default: 8080]
        --ws-port <PORT>     WebSocket端口 [default: 8081]
    -b, --bind <ADDRESS>     绑定地址 [default: 127.0.0.1]
    -d, --daemon             后台运行
        --pid-file <FILE>    PID文件路径
    -h, --help               显示帮助信息

EXAMPLES:
    # 启动服务器
    workflow-toolkit server start
    
    # 指定端口启动
    workflow-toolkit server start --port 9000 --ws-port 9001
    
    # 后台运行
    workflow-toolkit server start --daemon --pid-file /var/run/workflow-toolkit.pid
```

## 配置文件格式

### 主配置文件 (config/default.toml)

```toml
# 应用程序配置
[app]
name = "workflow-toolkit"
version = "1.0.0"
log_level = "info"
data_dir = "./data"
temp_dir = "./tmp"

# 服务器配置
[server]
http_port = 8080
ws_port = 8081
bind_address = "127.0.0.1"
cors_origins = ["*"]
request_timeout = 30
max_connections = 1000

# 存储配置
[storage]
backend = "lancedb"
database_path = "./data/workflow.db"
backup_enabled = true
backup_interval = "1h"
retention_days = 30

# 缓存配置
[cache]
backend = "memory"
max_capacity = 1000
ttl = "1h"
cleanup_interval = "10m"

# 工作流引擎配置
[workflow]
max_concurrent_workflows = 10
default_timeout = "30m"
checkpoint_interval = "5m"
retry_attempts = 3
retry_delay = "1s"

# 插件系统配置
[plugins]
plugin_dir = "./plugins"
auto_load = true
sandbox_enabled = true
resource_limits = { memory = "512MB", cpu = "1.0" }

# 日志配置
[logging]
level = "info"
format = "json"
output = "stdout"
file_path = "./logs/workflow-toolkit.log"
rotation = "daily"
max_files = 7

# 监控配置
[monitoring]
metrics_enabled = true
metrics_port = 9090
health_check_interval = "30s"
prometheus_enabled = false

# 安全配置
[security]
auth_enabled = false
jwt_secret = "your-secret-key"
token_expiry = "24h"
rate_limit = { requests = 100, window = "1m" }

# 环境特定配置
[environments.development]
log_level = "debug"
auto_reload = true

[environments.production]
log_level = "warn"
metrics_enabled = true
auth_enabled = true
```

### 工作流定义格式

#### YAML格式

```yaml
# 工作流基本信息
name: "data-processing-pipeline"
version: "1.0.0"
description: "数据处理管道示例"

# 元数据
metadata:
  author: "开发团队"
  tags: ["data", "processing", "etl"]
  category: "data-pipeline"

# 全局配置
global_config:
  timeout: "30m"
  retry_policy:
    max_attempts: 3
    delay: "5s"
    backoff: "exponential"
  variables:
    input_path: "/data/input"
    output_path: "/data/output"

# 工作流节点
nodes:
  - id: "start"
    type: "start"
    
  - id: "validate_input"
    type: "tool"
    tool_name: "data_validator"
    parameters:
      schema_file: "${input_path}/schema.json"
      data_file: "${input_path}/data.csv"
    timeout: "5m"
    
  - id: "process_data"
    type: "tool"
    tool_name: "data_processor"
    parameters:
      operation: "transform"
      input: "${validate_input.output}"
      config:
        remove_duplicates: true
        normalize: true
    retry_policy:
      max_attempts: 2
      delay: "10s"
      
  - id: "parallel_tasks"
    type: "parallel"
    
  - id: "generate_report"
    type: "tool"
    tool_name: "report_generator"
    parameters:
      template: "summary"
      data: "${process_data.output}"
      
  - id: "send_notification"
    type: "tool"
    tool_name: "notification_sender"
    parameters:
      message: "数据处理完成"
      recipients: ["admin@example.com"]
      
  - id: "quality_check"
    type: "condition"
    condition: "${process_data.quality_score} > 0.8"
    
  - id: "save_results"
    type: "tool"
    tool_name: "file_writer"
    parameters:
      path: "${output_path}/results.json"
      data: "${process_data.output}"
      
  - id: "end"
    type: "end"

# 节点连接
edges:
  - from: "start"
    to: "validate_input"
    
  - from: "validate_input"
    to: "process_data"
    
  - from: "process_data"
    to: "parallel_tasks"
    
  - from: "parallel_tasks"
    to: "generate_report"
    
  - from: "parallel_tasks"
    to: "send_notification"
    
  - from: "generate_report"
    to: "quality_check"
    
  - from: "send_notification"
    to: "quality_check"
    
  - from: "quality_check"
    to: "save_results"
    condition: "true"
    
  - from: "quality_check"
    to: "end"
    condition: "false"
    
  - from: "save_results"
    to: "end"
```

#### JSON格式

```json
{
  "name": "api-integration-workflow",
  "version": "1.0.0",
  "description": "API集成工作流示例",
  "metadata": {
    "author": "API团队",
    "tags": ["api", "integration", "http"],
    "category": "integration"
  },
  "global_config": {
    "timeout": "15m",
    "variables": {
      "api_base_url": "https://api.example.com",
      "api_key": "${env.API_KEY}"
    }
  },
  "nodes": [
    {
      "id": "start",
      "type": "start"
    },
    {
      "id": "fetch_data",
      "type": "tool",
      "tool_name": "http_client",
      "parameters": {
        "method": "GET",
        "url": "${api_base_url}/data",
        "headers": {
          "Authorization": "Bearer ${api_key}",
          "Content-Type": "application/json"
        }
      }
    },
    {
      "id": "transform_data",
      "type": "tool",
      "tool_name": "json_transformer",
      "parameters": {
        "input": "${fetch_data.response.body}",
        "transformations": [
          {
            "field": "created_at",
            "operation": "parse_date",
            "format": "ISO8601"
          },
          {
            "field": "amount",
            "operation": "to_number"
          }
        ]
      }
    },
    {
      "id": "save_to_database",
      "type": "tool",
      "tool_name": "database_writer",
      "parameters": {
        "connection": "postgresql://localhost/mydb",
        "table": "processed_data",
        "data": "${transform_data.output}",
        "mode": "insert"
      }
    },
    {
      "id": "end",
      "type": "end"
    }
  ],
  "edges": [
    {"from": "start", "to": "fetch_data"},
    {"from": "fetch_data", "to": "transform_data"},
    {"from": "transform_data", "to": "save_to_database"},
    {"from": "save_to_database", "to": "end"}
  ]
}
```

### 插件配置格式

```yaml
# 插件配置文件 (plugins.yaml)
plugins:
  # Python插件
  - name: "data-processing-tools"
    type: "python"
    enabled: true
    config:
      requirements_file: "./plugins/data-tools/requirements.txt"
      entry_point: "main.py"
      virtual_env: "./venvs/data-tools"
      environment:
        PYTHONPATH: "./plugins/data-tools"
      resource_limits:
        memory: "1GB"
        timeout: "30m"
      tools:
        - name: "csv_processor"
          function: "process_csv"
        - name: "json_transformer"
          function: "transform_json"
        - name: "data_validator"
          function: "validate_data"

  # Node.js插件
  - name: "web-scraping-tools"
    type: "nodejs"
    enabled: true
    config:
      package_json: "./plugins/web-tools/package.json"
      entry_point: "index.js"
      node_modules: "./plugins/web-tools/node_modules"
      environment:
        NODE_ENV: "production"
      tools:
        - name: "web_scraper"
          function: "scrapeWebsite"
        - name: "html_parser"
          function: "parseHtml"

  # Docker插件
  - name: "image-processing-tools"
    type: "docker"
    enabled: true
    config:
      image: "my-image-processor:latest"
      container_config:
        ports:
          - "8080:8080"
        environment:
          - "LOG_LEVEL=INFO"
        volumes:
          - "./data:/app/data"
        memory_limit: "2GB"
        cpu_limit: "2.0"
      api_endpoints:
        - path: "/tools/{tool_name}/execute"
          method: "POST"
      health_check:
        path: "/health"
        interval: "30s"
        timeout: "5s"

  # WASM插件
  - name: "math-tools"
    type: "wasm"
    enabled: true
    config:
      module_path: "./plugins/math-tools/calculator.wasm"
      memory_limit: "64MB"
      timeout: "10s"
      tools:
        - name: "calculator"
          export_name: "calculate"
        - name: "statistics"
          export_name: "compute_stats"

  # Native插件
  - name: "system-tools"
    type: "native"
    enabled: true
    config:
      library_path: "./plugins/system-tools/libsystem_tools.so"
      symbols:
        - name: "file_monitor"
          symbol: "create_file_monitor"
        - name: "process_manager"
          symbol: "create_process_manager"
```

## 错误代码和处理指南

### 错误代码分类

#### 1000-1999: 系统错误

| 错误代码 | 错误名称 | 描述 | 解决方案 |
|---------|---------|------|----------|
| 1001 | ConfigurationError | 配置文件错误 | 检查配置文件格式和内容 |
| 1002 | StorageError | 存储系统错误 | 检查数据库连接和权限 |
| 1003 | NetworkError | 网络连接错误 | 检查网络连接和防火墙设置 |
| 1004 | PermissionError | 权限不足 | 检查文件和目录权限 |
| 1005 | ResourceExhausted | 资源耗尽 | 释放资源或增加系统资源 |

#### 2000-2999: 工作流错误

| 错误代码 | 错误名称 | 描述 | 解决方案 |
|---------|---------|------|----------|
| 2001 | WorkflowNotFound | 工作流不存在 | 检查工作流名称是否正确 |
| 2002 | InvalidWorkflowDefinition | 工作流定义无效 | 验证工作流定义格式 |
| 2003 | WorkflowExecutionFailed | 工作流执行失败 | 检查工作流逻辑和依赖 |
| 2004 | CircularDependency | 循环依赖 | 检查工作流节点依赖关系 |
| 2005 | NodeExecutionTimeout | 节点执行超时 | 增加超时时间或优化节点逻辑 |

#### 3000-3999: 工具错误

| 错误代码 | 错误名称 | 描述 | 解决方案 |
|---------|---------|------|----------|
| 3001 | ToolNotFound | 工具不存在 | 检查工具名称和插件状态 |
| 3002 | InvalidParameters | 参数无效 | 检查参数格式和类型 |
| 3003 | ToolExecutionFailed | 工具执行失败 | 检查工具逻辑和依赖 |
| 3004 | ParameterValidationFailed | 参数验证失败 | 根据schema修正参数 |
| 3005 | ToolTimeout | 工具执行超时 | 增加超时时间或优化工具 |

#### 4000-4999: 插件错误

| 错误代码 | 错误名称 | 描述 | 解决方案 |
|---------|---------|------|----------|
| 4001 | PluginNotFound | 插件不存在 | 检查插件路径和配置 |
| 4002 | PluginLoadFailed | 插件加载失败 | 检查插件依赖和权限 |
| 4003 | PluginInitializationFailed | 插件初始化失败 | 检查插件配置和环境 |
| 4004 | PluginCommunicationError | 插件通信错误 | 检查进程间通信机制 |
| 4005 | PluginCrashed | 插件崩溃 | 检查插件日志和错误信息 |

#### 5000-5999: API错误

| 错误代码 | 错误名称 | 描述 | 解决方案 |
|---------|---------|------|----------|
| 5001 | InvalidRequest | 请求无效 | 检查请求格式和参数 |
| 5002 | AuthenticationFailed | 认证失败 | 检查认证凭据 |
| 5003 | AuthorizationFailed | 授权失败 | 检查用户权限 |
| 5004 | RateLimitExceeded | 请求频率超限 | 降低请求频率 |
| 5005 | ServiceUnavailable | 服务不可用 | 检查服务状态和健康度 |

### 错误处理示例

#### Rust错误处理

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("工作流不存在: {name}")]
    WorkflowNotFound { name: String },
    
    #[error("工作流定义无效: {message}")]
    InvalidDefinition { message: String },
    
    #[error("节点执行失败: {node_id} - {source}")]
    NodeExecutionFailed { 
        node_id: String, 
        #[source] source: Box<dyn std::error::Error + Send + Sync> 
    },
    
    #[error("存储错误: {source}")]
    StorageError { 
        #[from] source: StorageError 
    },
}

// 错误处理函数
pub fn handle_workflow_error(error: WorkflowError) -> (u16, String) {
    match error {
        WorkflowError::WorkflowNotFound { name } => {
            (2001, format!("工作流 '{}' 不存在", name))
        }
        WorkflowError::InvalidDefinition { message } => {
            (2002, format!("工作流定义无效: {}", message))
        }
        WorkflowError::NodeExecutionFailed { node_id, source } => {
            (2003, format!("节点 '{}' 执行失败: {}", node_id, source))
        }
        WorkflowError::StorageError { source } => {
            (1002, format!("存储错误: {}", source))
        }
    }
}
```

#### CLI错误处理

```bash
# 错误输出格式
{
  "error": {
    "code": 2001,
    "message": "工作流 'my-workflow' 不存在",
    "details": {
      "workflow_name": "my-workflow",
      "available_workflows": ["workflow1", "workflow2"]
    },
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### HTTP API错误响应

```json
{
  "error": {
    "code": 3002,
    "message": "参数无效",
    "details": {
      "field": "operation",
      "expected": "string",
      "actual": "number",
      "allowed_values": ["add", "subtract", "multiply", "divide"]
    },
    "request_id": "req_123456789",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### 故障排除指南

#### 1. 工作流执行问题

**问题**: 工作流卡在某个节点
**排查步骤**:
1. 检查节点日志: `workflow-toolkit workflow status <execution-id>`
2. 查看节点依赖: 确认前置节点是否完成
3. 检查资源使用: 确认内存和CPU是否充足
4. 验证工具可用性: `workflow-toolkit tool list`

**问题**: 工作流执行失败
**排查步骤**:
1. 查看错误信息: `workflow-toolkit workflow status <execution-id> --format json`
2. 检查工具参数: 验证参数格式和类型
3. 测试工具独立执行: `workflow-toolkit tool execute <tool-name> --params <params>`
4. 检查插件状态: `workflow-toolkit plugin list`

#### 2. 插件加载问题

**问题**: 插件加载失败
**排查步骤**:
1. 检查插件路径: 确认文件存在且可访问
2. 验证插件配置: `workflow-toolkit config validate`
3. 检查依赖安装: 确认所需依赖已安装
4. 查看插件日志: 检查插件初始化日志

**问题**: 插件工具不可用
**排查步骤**:
1. 确认插件已加载: `workflow-toolkit plugin list`
2. 检查工具注册: `workflow-toolkit tool list --plugin <plugin-name>`
3. 测试插件健康: 检查插件健康检查接口
4. 重新加载插件: `workflow-toolkit plugin reload <plugin-name>`

#### 3. 性能问题

**问题**: 执行速度慢
**优化建议**:
1. 启用缓存: 配置结果缓存机制
2. 并行执行: 使用并行节点提高并发度
3. 资源调优: 增加内存和CPU限制
4. 数据库优化: 优化存储后端配置

**问题**: 内存使用过高
**优化建议**:
1. 限制并发数: 减少同时执行的工作流数量
2. 清理缓存: 定期清理过期缓存数据
3. 优化工具: 检查工具内存使用情况
4. 监控资源: 启用资源监控和告警

---

*本文档持续更新，如有问题请查阅用户手册或提交Issue。*