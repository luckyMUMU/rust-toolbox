# Product Design Document - Workflow Toolkit

> **Status**: Active
> **Version**: 1.0.0
> **Last Updated**: 2026-01-17

## 1. 项目愿景 (Project Vision)

构建一个基于 Rust 的高性能、可扩展、多接口（Multi-Interface）工作流执行系统。该系统旨在通过统一的架构支持 CLI、TUI 和 MCP Server 三种交互模式，提供强大的工作流编排、插件扩展和系统监控能力，解决复杂任务自动化、文件管理和系统运维的需求。

## 2. 核心功能规格 (Functional Specs)

### 2.1 工作流引擎 (Workflow Engine)
- **DAG 执行模型**: 支持基于有向无环图 (DAG) 的任务编排，支持并行执行、条件分支和循环。
- **状态管理**: 提供持久化的状态存储，支持执行检查点 (Checkpoint) 和故障恢复 (Recovery)。
- **并发控制**: 支持配置最大并发工作流数和任务数，防止资源过载。
- **错误处理**: 内置重试机制 (Retry Policy)、错误捕获和优雅降级。

### 2.2 多接口交互 (Interfaces)
- **CLI (Command Line Interface)**:
  - 提供完整的工作流、工具、插件管理命令。
  - 支持批处理模式和脚本集成。
  - 遵循 POSIX 标准，支持 Shell 补全。
- **TUI (Terminal User Interface)**:
  - **仪表盘**: 实时展示系统资源 (CPU/Memory/Disk/Network) 和任务状态。
  - **交互式管理**: 支持通过键盘快捷键管理工作流、查看日志和执行维护任务。
  - **响应式布局**: 自动适配不同终端尺寸，支持虚拟滚动。
- **MCP Server (Model Context Protocol)**:
  - 实现 MCP 协议，允许 LLM (如 Claude, ChatGPT) 直接调用系统工具和工作流。
  - 提供工具发现、执行和上下文管理能力。

### 2.3 插件系统 (Plugin System)
- **多语言支持**:
  - **Native (Rust)**: 高性能原生插件，动态链接库形式。
  - **Python**: 支持 Python 脚本和包，自动管理虚拟环境。
  - **Node.js**: 支持 Node.js 模块，自动处理 npm 依赖。
  - **Docker**: 支持容器化插件，提供隔离执行环境。
  - **WASM**: (实验性) 支持 WebAssembly 插件，提供安全沙箱。
- **生命周期管理**: 支持插件的动态安装、加载、卸载和热重载。
- **沙箱隔离**: 限制插件的资源使用 (CPU/Memory) 和权限。

### 2.4 工具与文件管理 (Tools & File Management)
- **工具注册表**: 统一管理内置工具和插件提供的工具，支持版本控制。
- **智能文件分类**: 基于规则和 AI 的文件分类工具，支持人工确认 (Human-in-the-loop)。
- **批量处理**: 高效的文件批处理能力，支持进度追踪和结果验证。

### 2.5 系统监控与运维 (Monitoring & Maintenance)
- **实时监控**: 采集并展示系统核心指标。
- **健康诊断**: 自动检测系统异常，提供维护建议。
- **自动清理**: 根据策略自动清理日志、缓存和临时文件。

## 3. 用户流 (User Flows)

### 3.1 工作流创建与执行 (Workflow Lifecycle)
1.  **定义**: 用户编写 YAML/JSON 格式的工作流定义文件 (Definition)。
2.  **验证**: 系统解析定义文件，验证 DAG 结构、参数类型和依赖关系。
3.  **执行**: 用户通过 CLI/TUI/MCP 触发执行。
    - 引擎创建执行上下文 (ExecutionContext)。
    - 根据依赖关系调度任务。
    - 实时更新执行状态。
4.  **监控**: 用户通过 TUI 或 CLI 查看实时日志和进度。
5.  **结果**: 执行完成，输出结果并保存执行记录。

### 3.2 插件安装与使用 (Plugin Management)
1.  **发现**: 用户查看可用插件列表或指定插件路径。
2.  **安装**: 执行安装命令 (`plugin install`)。
    - 系统检测插件类型。
    - 自动安装依赖 (如 pip install, npm install)。
    - 注册插件提供的工具到工具注册表。
3.  **使用**: 在工作流定义中引用插件提供的工具。
4.  **更新/卸载**: 用户更新或移除插件，系统自动清理资源和注销工具。

### 3.3 TUI 交互流程 (TUI Interaction)
1.  **启动**: 用户运行 `workflow-toolkit tui`。
2.  **概览**: 默认进入 Dashboard，查看系统健康度和活跃任务。
3.  **导航**: 使用 `F1-F5` 切换视图 (工作流列表、日志、工具管理等)。
4.  **操作**: 在列表视图中选中项，按快捷键 (如 `r` 运行, `s` 停止, `d` 详情) 进行操作。
5.  **退出**: 按 `q` 或 `Ctrl+C` 退出，系统保存 UI 状态。

## 4. 业务术语表 (Glossary)

| 术语 (Term) | 定义 (Definition) | 对应代码 (Code Reference) |
| :--- | :--- | :--- |
| **Workflow** | 一组有序的任务集合，通常由 DAG 定义。 | `crate::workflow::WorkflowDefinition` |
| **Tool** | 可执行的最小单元，完成特定功能。 | `crate::tools::Tool` |
| **Plugin** | 一组工具的集合，提供特定的扩展能力。 | `crate::plugins::Plugin` |
| **Execution** | 工作流或工具的一次具体运行实例。 | `crate::core::ExecutionContext` |
| **Checkpoint** | 工作流执行过程中的状态快照，用于恢复。 | `crate::workflow::state::Checkpoint` |
| **Task** | 工作流中的一个节点，对应一个工具的执行。 | `crate::workflow::Task` |
| **MCP** | Model Context Protocol，用于 AI 交互的协议。 | `crate::interfaces::mcp` |

## 5. 设计原则 (Design Principles)

1.  **Performance First**: 核心路径零拷贝，利用 Rust 的所有权机制和异步运行时最大化吞吐量。
2.  **Safety & Reliability**: 严格的类型系统，全面的错误处理，确保系统在长期运行中的稳定性。
3.  **Observability**: 系统内部状态必须是可观测的 (Metrics, Logs, Traces)。
4.  **Modularity**: 核心引擎与具体实现解耦，通过插件机制扩展功能。
5.  **User Experience**: CLI 和 TUI 必须提供一致且友好的交互体验。
