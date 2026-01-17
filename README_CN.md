# 工作流工具集 (Workflow Toolkit)

简体中文 | [English](./README.md)

一个使用 Rust 构建的综合性多接口工作流执行系统，通过 CLI、TUI 和 MCP 服务器接口提供灵活的工作流管理，具备先进的文件管理能力和全面的系统监控。

## 特性

### 核心工作流引擎
- **多接口支持**：CLI 命令、终端用户界面 (TUI) 和模型上下文协议 (MCP) 服务器
- **基于 DAG 的工作流引擎**：使用 petgraph 实现的有向无环图执行，支持条件、循环和并行处理
- **状态管理**：具有检查点和恢复能力的持久化工作流状态
- **配置管理**：支持环境变量、配置文件和 CLI 参数覆盖的分层配置

### 先进的 TUI 系统
- **增强型组件系统**：具有生命周期管理、主题和布局的全面组件框架
- **系统监控**：实时 CPU、内存、磁盘和网络监控，配有可视化图表
- **交互式维护**：系统诊断、维护建议和自动化清理操作
- **性能优化**：内置性能监控、内存管理和渲染优化
- **响应式设计**：适配不同终端大小的自适应布局，支持虚拟化
- **全面测试**：完整的单元测试和集成测试覆盖，包含性能基准测试

### 插件系统
- **可扩展架构**：支持 Native (Rust)、Python、Node.js、Docker 和 WebAssembly 插件
- **工具注册表**：用于工作流和独立执行的可重用工具节点
- **插件管理器**：插件的动态加载和生命周期管理
- **沙箱执行**：具有资源限制的安全插件执行环境

### 文件管理工具
- **交互式分类**：AI 驱动的文件分类，支持人工决策
- **批量处理**：具有进度跟踪的高效大批量文件操作
- **文本处理**：先进的文本分析和转换能力
- **结果确认**：关键操作的人机回环验证
- **性能监控**：内置剖析和指标收集

### 系统运维
- **系统诊断**：全面的健康评估和问题检测
- **维护建议**：基于系统状态的智能建议
- **自动化操作**：内存清理、磁盘优化和进程管理
- **告警管理**：可配置的阈值和通知系统
- **资源监控**：实时跟踪系统资源并提供历史数据

### 高级特性
- **异步执行**：基于 tokio 运行时的完整 async/await 支持
- **缓存系统**：使用 moka 实现的高性能缓存和多级缓存策略
- **向量存储**：可选的 LanceDB 集成，用于高级数据操作
- **审计日志**：全面的操作跟踪和日志记录
- **错误恢复**：具有重试机制和优雅降级的健壮错误处理

## 快速开始

### 前置条件

- **Rust**: 1.70+ (2021 Edition)
- **系统**: Linux, macOS, 或 Windows
- **内存**: 最少 2GB RAM, 推荐 4GB+
- **存储**: 至少 1GB 可用空间

### 安装

```bash
# 克隆仓库
git clone https://github.com/workflow-toolkit/workflow-toolkit.git
cd workflow-toolkit

# 构建项目 (debug)
cargo build

# 构建生产版本
cargo build --release

# 运行测试验证安装
cargo test

# 全局安装 (可选)
cargo install --path .
```

### 基本用法

```bash
# 显示可用命令
workflow-toolkit --help

# 从定义创建工作流
workflow-toolkit workflow create examples/hello-world.yaml

# 列出所有工作流
workflow-toolkit workflow list

# 执行工作流
workflow-toolkit workflow execute hello-world

# 监控工作流执行
workflow-toolkit workflow status <workflow-id> --watch

# 启动交互式 TUI
workflow-toolkit tui

# 启动 MCP 服务器
workflow-toolkit server --http-port 8080 --ws-port 8081

# 列出可用工具
workflow-toolkit tool list

# 直接执行工具
workflow-toolkit tool execute echo --params '{"message": "Hello World"}'
```

## 文档

提供多种格式的全面文档：

### 快速入门
- **[README.md](README.md)**: 英文版本项目概述
- **[docs/INDEX.md](docs/INDEX.md)**: 完整文档索引
- **[docs/CHEATSHEET.md](docs/CHEATSHEET.md)**: 快速命令参考

### 用户文档
- **[docs/USER_GUIDE.md](docs/USER_GUIDE.md)**: 包含示例的完整用户手册
- **[docs/TUTORIAL.md](docs/TUTORIAL.md)**: 分步教程
- **[docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)**: 问题解决指南
- **[examples/](examples/)**: 运行示例和模板

### API 与参考
- **[docs/API_INDEX.md](docs/API_INDEX.md)**: 完整的命令和 API 参考
- **[docs/API_REFERENCE.md](docs/API_REFERENCE.md)**: 详细的 API 文档
- **[docs/API_USAGE_GUIDE.md](docs/API_USAGE_GUIDE.md)**: 使用模式和示例

### 开发者文档
- **[docs/DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md)**: 开发工作流和最佳实践
- **[docs/PROJECT_OVERVIEW.md](docs/PROJECT_OVERVIEW.md)**: 架构概述
- **[docs/PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md)**: 插件开发指南
- **[AGENTS.md](AGENTS.md)**: 根开发指南
- **[docs/AGENTS.md](docs/AGENTS.md)**: 文档指南
- **[docs/DOCUMENTATION_STANDARDS.md](docs/DOCUMENTATION_STANDARDS.md)**: 编写标准

### 特定特性文档
- **[docs/FILE_MANAGEMENT_TOOLS_GUIDE.md](docs/FILE_MANAGEMENT_TOOLS_GUIDE.md)**: 文件操作
- **[docs/FILE_MANAGEMENT_API_REFERENCE.md](docs/FILE_MANAGEMENT_API_REFERENCE.md)**: 文件管理 API
- **[docs/FILE_MANAGEMENT_WORKFLOW_TEMPLATES.md](docs/FILE_MANAGEMENT_WORKFLOW_TEMPLATES.md)**: 工作流模板

### Python 脚本迁移
- **[scripts/](scripts/)**: 原始 Python 脚本参考
- **[docs/MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md)**: Python 到 Rust 迁移指南（即将推出）

### 历史与存档
- **[.backup/](.backup/)**: 存档的历史文档
- **[.backup/ARCHIVE_INDEX.md](.backup/ARCHIVE_INDEX.md)**: 存档内容索引

## 界面与交互

### TUI 系统
- **系统监控**: 实时 CPU、内存、磁盘和网络监控，配有可视化图表
- **日志查看器**: 具有过滤和搜索能力的实时日志流
- **工具管理器**: 交互式浏览、配置和执行工具
- **插件管理器**: 安装、配置和管理插件
- **系统状态**: 带有维护工具的全面系统健康仪表盘
- **性能监控器**: 实时性能指标和优化建议
- **错误管理**: 具有恢复建议的交互式错误处理

#### 系统维护特性

```bash
# 在 TUI 中访问维护模式
workflow-toolkit tui
# 按 'm' 进入维护模式

# 可用的维护操作：
# c: 清除告警和通知
# o: 内存优化
# d: 磁盘清理建议
# p: 进程优化
# s: 系统诊断
# r: 重启建议
```

#### TUI 性能特性

- **虚拟化**: 使用虚拟滚动高效处理大数据集
- **内存管理**: 自动内存清理和泄漏检测
- **渲染优化**: 根据终端能力进行自适应渲染
- **响应式设计**: 针对不同屏幕尺寸自动调整布局
- **主题系统**: 具有可自定义配色方案的多套主题

### 文件管理工具

工具集包含专门的文件管理能力：

```bash
# 交互式文件分类
workflow-toolkit tool execute file_classification \
  --params '{"directory": "./data", "rules_file": "classification-rules.json"}'

# 批量文件处理
workflow-toolkit tool execute batch_processor \
  --params '{"input_dir": "./input", "output_dir": "./output", "operation": "transform"}'

# 人工审核的文本处理
workflow-toolkit tool execute text_processor \
  --params '{"input_file": "document.txt", "operations": ["extract", "analyze"]}'
```

### 系统监控与维护

全面的系统监控和维护能力：

```bash
# 系统健康检查
workflow-toolkit system health

# 资源监控
workflow-toolkit system monitor --watch

# 维护建议
workflow-toolkit system maintenance --recommendations

# 性能分析
workflow-toolkit system performance --analyze
```

## 配置

工具集使用分层配置系统，优先级如下：

1. **默认配置** (内置默认值)
2. **配置文件** (`config/default.toml`)
3. **环境变量** (以 `WORKFLOW_TOOLKIT_` 为前缀)
4. **命令行参数** (最高优先级)

### 配置文件示例

```toml
[server]
http_port = 8080
ws_port = 8081
max_connections = 100

[storage]
database_path = "./data/workflow.db"
cache_size = 104857600  # 100MB
enable_lancedb = false

[logging]
level = "info"
format = "Pretty"
file_path = "./logs/workflow-toolkit.log"

[plugins]
plugin_dir = "./plugins"
auto_load = true
sandbox_enabled = true
max_concurrent_plugins = 10

[workflow]
max_concurrent_workflows = 5
default_timeout = "30m"
checkpoint_interval = "5m"

[file_management]
temp_dir = "./tmp"
max_file_size = "100MB"
classification_confidence_threshold = 0.8
batch_size = 100

[performance]
enable_profiling = false
metrics_collection = true
cache_ttl = "1h"

[tui]
theme = "dark"  # dark, light, auto
refresh_rate = 60  # Hz
enable_mouse = true
enable_virtualization = true
max_log_entries = 10000

[system_monitoring]
cpu_threshold_warning = 70.0
cpu_threshold_critical = 90.0
memory_threshold_warning = 80.0
memory_threshold_critical = 95.0
disk_threshold_warning = 85.0
disk_threshold_critical = 95.0
network_threshold_warning = 100.0  # MB/s
update_interval = "5s"

[maintenance]
auto_cleanup_enabled = true
cleanup_interval = "1h"
max_alert_age = "24h"
memory_cleanup_threshold = 90.0
disk_cleanup_threshold = 90.0
```

### 环境变量

```bash
# 服务器配置
export WORKFLOW_TOOLKIT_SERVER__HTTP_PORT=8080
export WORKFLOW_TOOLKIT_SERVER__WS_PORT=8081

# 存储配置
export WORKFLOW_TOOLKIT_STORAGE__DATABASE_PATH="./data/workflow.db"

# 插件配置
export WORKFLOW_TOOLKIT_PLUGINS__PLUGIN_DIR="./plugins"

# 日志配置
export WORKFLOW_TOOLKIT_LOGGING__LEVEL="debug"

# TUI 配置
export WORKFLOW_TOOLKIT_TUI__THEME="dark"
export WORKFLOW_TOOLKIT_TUI__REFRESH_RATE=60

# 系统监控配置
export WORKFLOW_TOOLKIT_SYSTEM_MONITORING__CPU_THRESHOLD_WARNING=70.0
export WORKFLOW_TOOLKIT_SYSTEM_MONITORING__UPDATE_INTERVAL="5s"

# 维护配置
export WORKFLOW_TOOLKIT_MAINTENANCE__AUTO_CLEANUP_ENABLED=true
```

## 架构

工具集遵循模块化、分层架构设计，具有良好的可扩展性：

### 核心架构层

```
┌─────────────────────────────────────────────────────────────┐
│                    接口层 (Interface Layer)                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │     CLI     │  │     TUI     │  │    MCP Server       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    应用层 (Application Layer)               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   工作流    │  │    工具     │  │      文件管理       │  │
│  │    引擎     │  │   注册表    │  │        工具         │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    插件层 (Plugin Layer)                    │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────┐  │
│  │ Native  │ │ Python  │ │ Node.js │ │ Docker  │ │ WASM  │  │
│  │  插件   │ │  插件   │ │  插件   │ │  插件   │ │ 插件  │  │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └───────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   基础设施层 (Infrastructure Layer)          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │    存储     │  │    缓存     │  │      性能监控       │  │
│  │  (LanceDB)  │  │   (Moka)    │  │                    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 关键组件

- **核心层**: 类型定义、错误处理、配置管理
- **存储层**: 带有 moka 缓存的高性能 LanceDB 持久化
- **工作流引擎**: 基于 petgraph 的 DAG 执行，支持异步
- **工具系统**: 可重用工具节点和全面注册表
- **插件系统**: 带有沙箱的多语言插件支持
- **文件管理**: 专门的文件操作和分类工具
- **接口层**: CLI、TUI 和 MCP 服务器实现
- **性能层**: 剖析、指标收集和优化

### 技术栈

- **语言**: Rust 2021 Edition (1.70+)
- **异步运行时**: 带有完整 async/await 支持的 Tokio
- **图处理**: 用于 DAG 操作的 petgraph
- **存储**: 带有 Arrow/Parquet 支持的 LanceDB (可选)
- **缓存**: 用于高性能内存缓存的 moka
- **CLI**: 带有 derive 特性和 shell 补全的 clap 4.5
- **TUI**: 带有 crossterm 的 ratatui 0.29，用于终端处理
- **系统监控**: 用于实时系统指标的 sysinfo
- **序列化**: 支持 JSON/YAML/TOML 的 serde
- **插件运行时**: libloading, bollard (Docker)
- **错误处理**: 用于结构化错误类型的 thiserror
- **并发**: 用于线程安全操作的 dashmap 和 parking_lot

## 开发

### 前置条件

- **Rust**: 1.70+ 支持 2021 Edition
- **Cargo**: 最新版本
- **系统依赖**: 
  - Python 3.8+ (用于 Python 插件)
  - Node.js 16+ (用于 Node.js 插件)
  - Docker (用于 Docker 插件)

### 构建

```bash
# Debug 构建 (编译较快)
cargo build

# Release 构建 (优化版本)
cargo build --release

# 包含所有特性的构建
cargo build --all-features

# 构建特定示例
cargo build --example comprehensive_workflow_example
cargo build --example file_management_example
```

### 测试

项目使用全面的测试，包括单元测试、集成测试、基于属性的测试和性能基准测试：

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试模块
cargo test storage::tests
cargo test file_management::tests
cargo test tui::tests

# 运行 TUI 相关测试
cargo test tui_unit_tests --test tui_standalone_unit_tests
cargo test tui_integration_tests --test tui_integration_tests
cargo test tui_performance_tests --test tui_performance_benchmark_tests

# 运行基于属性的测试
cargo test property_tests

# 运行集成测试
cargo test --test integration_tests

# 以特定日志级别运行
RUST_LOG=debug cargo test

# 运行性能基准测试
cargo test --release performance_benchmark
```

### 开发命令

```bash
# 格式化代码
cargo fmt

# 代码检查 (Lint)
cargo clippy

# 检查但不构建
cargo check

# 生成文档
cargo doc --open

# 运行示例
cargo run --example python_plugin_example
cargo run --example file_management_example

# 带有调试日志运行
RUST_LOG=debug cargo run -- --help
```

### 项目结构

```
workflow-toolkit/
├── src/                          # 源代码
│   ├── core.rs                   # 核心类型和定义
│   ├── config.rs                 # 配置管理
│   ├── error.rs                  # 错误处理
│   ├── workflow/                 # 工作流引擎
│   ├── tools/                    # 工具系统
│   ├── plugins/                  # 插件系统
│   │   └── file_management/      # 文件管理工具
│   ├── storage/                  # 存储层
│   ├── interfaces/               # CLI, TUI, MCP 接口
│   │   ├── cli/                  # 命令行接口
│   │   └── tui/                  # 终端用户界面
│   │       ├── widgets/          # TUI 组件 (WorkflowList, SystemStatus 等)
│   │       ├── theme.rs          # 主题系统和配色方案
│   │       ├── layout.rs         # 布局管理和约束
│   │       ├── event.rs          # 事件处理和键绑定
│   │       ├── performance.rs    # 性能监控和优化
│   │       ├── memory.rs         # 内存管理和泄漏检测
│   │       ├── monitoring.rs     # 系统监控和指标
│   │       └── config.rs         # TUI 特定配置
│   └── performance/              # 性能监控
├── examples/                     # 工作流示例和用法
│   ├── templates/                # 工作流模板
│   └── tools/                    # 示例工具实现
├── docs/                         # 文档
├── tests/                        # 集成和单元测试
│   ├── tui_standalone_unit_tests.rs      # 全面的 TUI 单元测试
│   ├── tui_integration_tests.rs          # TUI 集成测试
│   ├── tui_basic_integration_tests.rs    # 基础 TUI 集成测试
│   └── tui_performance_benchmark_tests.rs # TUI 性能基准测试
└── config/                       # 默认配置
```

## 示例与用例

### 工作流示例

`examples/` 目录包含各种用例的全面示例：

```bash
# 基础工作流执行
cargo run --example comprehensive_workflow_example

# 插件集成
cargo run --example plugin_integration_example

# 文件管理操作
cargo run --example file_management_example
```
