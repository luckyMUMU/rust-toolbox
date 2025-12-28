# CLI接口设计文档

## 概述

CLI接口是工作流工具包的主要用户交互界面，提供完整的命令行功能来管理工作流、工具和插件。基于clap v4框架构建，支持子命令、参数验证、帮助文档和命令补全。

## 架构设计

### 命令结构

```
workflow-toolkit
├── workflow
│   ├── create <definition-file>
│   ├── execute <workflow-name> [--params <params-file>]
│   ├── status <workflow-id>
│   ├── pause <workflow-id>
│   ├── resume <workflow-id>
│   ├── stop <workflow-id>
│   └── list
├── tool
│   ├── list
│   └── execute <tool-name> [--params <params>]
├── plugin
│   ├── install <plugin-path>
│   ├── list
│   └── reload <plugin-name>
├── batch
│   └── execute <workflow-list-file> [--parallel <count>]
├── tui
└── server [--http-port <port>] [--ws-port <port>]
```

### 核心组件

#### 1. CliApp - 主应用程序结构
```rust
pub struct CliApp {
    config: Config,
    workflow_engine: Arc<dyn WorkflowEngine>,
    tool_registry: Arc<dyn ToolRegistry>,
    state_manager: Arc<StateManager>,
}
```

#### 2. CommandHandler - 命令处理器
```rust
pub trait CommandHandler {
    async fn handle(&self, args: &CommandArgs) -> Result<CommandOutput>;
}
```

#### 3. OutputFormatter - 输出格式化器
```rust
pub trait OutputFormatter {
    fn format_workflow_status(&self, execution: &WorkflowExecution) -> String;
    fn format_tool_list(&self, tools: &[ToolInfo]) -> String;
    fn format_execution_result(&self, result: &ExecutionResult) -> String;
}
```

## 数据结构

### 命令参数结构
```rust
#[derive(Parser)]
#[command(name = "workflow-toolkit")]
#[command(about = "A multi-interface workflow execution system")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Configuration file path
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    
    /// Log level
    #[arg(long, global = true, default_value = "info")]
    pub log_level: String,
    
    /// Output format (json, yaml, table)
    #[arg(long, global = true, default_value = "table")]
    pub output: OutputFormat,
}

#[derive(Subcommand)]
pub enum Commands {
    Workflow { #[command(subcommand)] action: WorkflowAction },
    Tool { #[command(subcommand)] action: ToolAction },
    Plugin { #[command(subcommand)] action: PluginAction },
    Batch { #[command(subcommand)] action: BatchAction },
    Tui,
    Server { 
        #[arg(long, default_value = "8080")] http_port: u16,
        #[arg(long, default_value = "8081")] ws_port: u16,
    },
}
```

### 工作流命令
```rust
#[derive(Subcommand)]
pub enum WorkflowAction {
    Create {
        /// Path to workflow definition file (YAML)
        definition_file: PathBuf,
        /// Validate only, don't save
        #[arg(long)]
        validate_only: bool,
    },
    Execute {
        /// Workflow name or path to definition file
        workflow_name: String,
        /// Parameters file (JSON/YAML)
        #[arg(long)]
        params: Option<PathBuf>,
        /// Execute in background
        #[arg(long)]
        background: bool,
        /// Wait for completion and show progress
        #[arg(long)]
        wait: bool,
    },
    Status {
        /// Workflow execution ID
        workflow_id: String,
        /// Show detailed node status
        #[arg(long)]
        detailed: bool,
        /// Follow status updates
        #[arg(long)]
        follow: bool,
    },
    Pause {
        /// Workflow execution ID
        workflow_id: String,
    },
    Resume {
        /// Workflow execution ID
        workflow_id: String,
    },
    Stop {
        /// Workflow execution ID
        workflow_id: String,
        /// Force stop without graceful shutdown
        #[arg(long)]
        force: bool,
    },
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<ExecutionStatus>,
        /// Show only recent executions
        #[arg(long)]
        recent: bool,
    },
}
```

### 工具命令
```rust
#[derive(Subcommand)]
pub enum ToolAction {
    List {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Search pattern
        #[arg(long)]
        search: Option<String>,
    },
    Execute {
        /// Tool name
        tool_name: String,
        /// Tool parameters (JSON format)
        #[arg(long)]
        params: Option<String>,
        /// Parameters file (JSON/YAML)
        #[arg(long)]
        params_file: Option<PathBuf>,
        /// Timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },
}
```

### 批量执行命令
```rust
#[derive(Subcommand)]
pub enum BatchAction {
    Execute {
        /// File containing list of workflows to execute
        workflow_list_file: PathBuf,
        /// Maximum parallel executions
        #[arg(long, default_value = "4")]
        parallel: usize,
        /// Continue on failure
        #[arg(long)]
        continue_on_failure: bool,
        /// Output directory for results
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
}
```

## 输出格式

### 表格格式 (默认)
```
┌─────────────────────────────────────┬──────────┬─────────────────────┬──────────┐
│ Workflow ID                         │ Status   │ Started At          │ Progress │
├─────────────────────────────────────┼──────────┼─────────────────────┼──────────┤
│ 550e8400-e29b-41d4-a716-446655440000│ Running  │ 2024-01-15 10:30:00 │ 3/5      │
│ 6ba7b810-9dad-11d1-80b4-00c04fd430c8│ Completed│ 2024-01-15 09:15:00 │ 5/5      │
└─────────────────────────────────────┴──────────┴─────────────────────┴──────────┘
```

### JSON格式
```json
{
  "workflows": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "data-processing-pipeline",
      "status": "Running",
      "started_at": "2024-01-15T10:30:00Z",
      "progress": {
        "completed": 3,
        "total": 5
      }
    }
  ]
}
```

### YAML格式
```yaml
workflows:
  - id: 550e8400-e29b-41d4-a716-446655440000
    name: data-processing-pipeline
    status: Running
    started_at: 2024-01-15T10:30:00Z
    progress:
      completed: 3
      total: 5
```

## 错误处理

### 错误类型
```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Invalid command arguments: {0}")]
    InvalidArguments(String),
    
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
```

### 错误输出格式
```
Error: Workflow not found: 'non-existent-workflow'

Suggestion: Use 'workflow-toolkit workflow list' to see available workflows

For more information, try '--help'
```

## 进度显示

### 实时进度条
```
Executing workflow: data-processing-pipeline
┌─────────────────────────────────────────────────────────────┐
│ ● Data Ingestion     [Completed] ✓                         │
│ ● Data Validation    [Completed] ✓                         │
│ ● Data Transformation[Running]   ████████░░ 80%            │
│ ○ Data Export        [Pending]                             │
│ ○ Cleanup           [Pending]                             │
└─────────────────────────────────────────────────────────────┘

Overall Progress: 2/5 tasks completed (40%)
Elapsed: 00:02:34 | Estimated remaining: 00:01:26
```

## 配置管理

### 配置文件位置
1. `/etc/workflow-toolkit/config.toml` (系统级)
2. `~/.config/workflow-toolkit/config.toml` (用户级)
3. `./workflow-toolkit.toml` (项目级)
4. 通过 `--config` 参数指定

### 配置优先级
命令行参数 > 环境变量 > 项目配置 > 用户配置 > 系统配置 > 默认值

### 示例配置文件
```toml
[cli]
default_output_format = "table"
show_progress = true
auto_save_history = true

[workflow]
default_timeout = 3600
checkpoint_interval = 300
max_parallel_workflows = 4

[tools]
auto_discover = true
plugin_directories = ["~/.workflow-toolkit/plugins", "./plugins"]

[server]
http_port = 8080
ws_port = 8081
auth_enabled = false
```

## 命令补全

支持以下shell的命令补全：
- Bash
- Zsh
- Fish
- PowerShell

生成补全脚本：
```bash
workflow-toolkit completion bash > /etc/bash_completion.d/workflow-toolkit
workflow-toolkit completion zsh > ~/.zsh/completions/_workflow-toolkit
```

## 性能考虑

1. **延迟加载**: 只在需要时初始化重量级组件
2. **缓存**: 缓存工具列表和工作流定义
3. **并发**: 批量操作使用并发执行
4. **流式输出**: 大量数据使用流式输出避免内存占用
5. **增量更新**: 状态跟踪使用增量更新减少网络开销

## 测试策略

### 单元测试
- 命令解析正确性
- 参数验证逻辑
- 输出格式化功能
- 错误处理机制

### 集成测试
- 端到端命令执行
- 与工作流引擎集成
- 与工具注册表集成
- 配置文件加载

### 属性测试
- CLI输出信息完整性
- 批量执行完整性
- 参数验证一致性

## 扩展性

### 插件化命令
支持通过插件添加新的CLI命令：
```rust
pub trait CliPlugin {
    fn name(&self) -> &str;
    fn commands(&self) -> Vec<Command>;
    fn handle_command(&self, command: &str, args: &[String]) -> Result<()>;
}
```

### 自定义输出格式
支持注册自定义输出格式化器：
```rust
pub trait CustomFormatter {
    fn format_name(&self) -> &str;
    fn format_output(&self, data: &Value) -> Result<String>;
}
```