# 接口层 (Interfaces Layer)

## 1. 核心定义 (Stable)

### 1.1 层职责

接口层负责处理用户交互，包括 CLI、TUI 和 MCP Server 三种接口形式。该层依赖应用层，通过用例执行器与应用层交互。

### 1.2 模块结构

```
interfaces/
├── cli/            # 命令行接口
│   ├── app.rs         # CLI 应用入口
│   ├── commands.rs    # 命令定义
│   ├── output.rs      # 输出格式化
│   └── error.rs       # CLI 错误处理
├── tui/            # 终端用户界面
│   ├── app.rs         # TUI 应用入口
│   ├── widgets/       # UI 组件
│   ├── event.rs       # 事件处理
│   └── theme.rs       # 主题配置
├── mcp.rs          # MCP 协议接口
└── mcp_server.rs   # MCP 服务器实现
```

### 1.3 CLI 接口

```rust
/// CLI 应用
pub struct CliApp {
    use_case_executor: Arc<dyn UseCaseExecutor>,
}

/// 命令枚举
pub enum Commands {
    /// 执行工作流
    Execute {
        workflow_file: PathBuf,
        #[arg(short, long)]
        params: Option<String>,
    },
    /// 管理工作流
    Manage {
        #[command(subcommand)]
        action: ManageAction,
    },
    /// 系统状态
    Status,
}
```

### 1.4 TUI 接口

```rust
/// TUI 应用
pub struct TuiApp {
    layout_manager: LayoutManager,
    widget_registry: WidgetRegistry,
    event_handler: EventHandler,
}

/// 主 TUI 接口
pub struct MainTuiInterface {
    app: TuiApp,
    theme: Theme,
}

/// 核心组件
pub struct LayoutManager;
pub struct WidgetRegistry;
pub struct ThemeManager;
```

### 1.5 MCP 接口

```rust
/// MCP 服务器
pub struct WorkflowMcpServer {
    config: McpServerConfig,
    use_case_executor: Arc<dyn UseCaseExecutor>,
}

/// MCP 服务器构建器
pub struct WorkflowMcpServerBuilder {
    config: McpServerConfig,
}

/// MCP 配置
pub struct McpServerConfig {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-IF001: CLI 框架选择
- **决策**: 使用 clap 作为 CLI 框架
- **理由**: 生态标准，宏支持好，文档完善
- **风险**: 编译时间增加

#### ADR-IF002: TUI 框架选择
- **决策**: 使用 ratatui 作为 TUI 框架
- **理由**: 现代 Rust TUI 标准，性能好，组件丰富
- **风险**: 学习曲线较陡

#### ADR-IF003: MCP 协议实现
- **决策**: 自定义 MCP 协议实现（参考 LiteFlow）
- **理由**: 需要与工作流引擎深度集成
- **风险**: 协议兼容性维护

### 2.2 任务清单

- [x] Task 0: CLI 基础框架
- [x] Task 1: 核心命令实现
- [x] Task 2: TUI 基础框架
- [ ] Task 3: TUI 高级组件
- [ ] Task 4: MCP 服务器完善

### 2.3 接口契约

```rust
/// 接口配置
pub struct InterfaceConfig {
    pub cli: CliConfig,
    pub tui: TuiConfig,
    pub mcp: McpConfig,
}

/// CLI 配置
pub struct CliConfig {
    pub default_output_format: OutputFormat,
    pub color_enabled: bool,
}

/// TUI 配置
pub struct TuiConfig {
    pub theme: String,
    pub refresh_rate_ms: u64,
}

/// MCP 配置
pub struct McpConfig {
    pub enabled: bool,
    pub port: u16,
}

pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}
```

### 2.4 测试策略

- **集成测试**: 端到端命令测试
- **UI 测试**: TUI 组件渲染测试
- **契约测试**: MCP 协议兼容性测试

## 3. 状态记录

- `[进行中]` | TUI 高级组件开发 | 2026-02-06
- `[已完成]` | CLI 基础实现 | 2026-01-30
- `[已完成]` | TUI 基础框架 | 2026-02-01

## 4. 依赖关系

```
interfaces/
├── 依赖: application (应用层)
│   ├── port::UseCaseExecutor
│   ├── service::WorkflowService
│   └── usecase::ExecuteWorkflowUseCase
└── 被依赖: 无（最顶层）
    └── 用户直接交互
```

## 5. 子模块详情

### CLI 模块
- [CLI 详情](./cli/design.md) - 命令行接口详细设计

### TUI 模块
- [TUI 详情](./tui/design.md) - 终端界面详细设计
- [组件详情](./tui/widgets/design.md) - UI 组件详细设计

### MCP 模块
- [MCP 详情](./mcp/design.md) - MCP 协议详细设计
