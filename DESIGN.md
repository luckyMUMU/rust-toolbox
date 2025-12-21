# 模块名称：Rust Toolbox 项目设计

## 1. 目标 (Goal)
- **核心功能**：记录 Rust Toolbox 项目的整体架构设计、技术选型和核心原则，为项目开发提供指导和参考。
- **非目标**：不包含具体工具的实现细节，不包含详细的部署指南。

## 2. 核心数据结构 (Data Structures)
- **Tool**：工具的核心抽象，定义了工具的基本行为和接口。
- **Plugin**：插件的数据结构，包含插件的元数据和执行逻辑。
- **WorkflowDefinition**：工作流定义，包含节点、边和执行规则。
- **McpContext**：MCP 上下文，包含执行状态和历史记录。
- **McpRequest**：MCP 请求，定义标准化的工具调用格式。
- **McpResponse**：MCP 响应，定义标准化的执行结果格式。
- **PersistenceManager**：持久化管理器，提供统一的数据存储和缓存服务。

## 3. 算法与逻辑设计 (Algorithm & Logic)

### 核心原则
*   **模块化与解耦**：强调核心逻辑与 UI 层分离，确保每个组件职责单一，易于维护和扩展。
*   **可扩展性**：通过插件系统支持外部工具的动态集成，提高项目的灵活性和活力。
*   **多语言支持**：所有面向用户的文本支持多种语言，提升用户体验。
*   **持久化管理**：提供统一的数据存储、缓存和配置管理服务，支持模块化和高性能读写操作。
*   **工作流编排**：支持通过 DAG（有向无环图）编排多个工具，实现自动化任务处理。
*   **模型上下文协议 (MCP) 支持**：实现标准化的上下文管理和工具调用协议，支持外部系统集成。
*   **面向服务架构**：提供统一的服务层，包含基于角色的访问控制和标准化的错误处理。
*   **配置驱动设计**：支持多源配置加载，包含缓存、热重载和优先级管理。
*   **全面日志记录**：实现结构化日志，支持多种输出目标和分层日志级别。
*   **多工具插件架构**：通过基于数组的元数据格式支持单工具和多工具插件。
*   **设计先行**：任何新功能或重大变更都需要先编写设计文档。
*   **文档一致性**：代码变更与文档更新同步，确保准确性和及时性。

### 项目结构 (Cargo Workspace)
项目采用 Cargo Workspace 结构，将不同功能模块组织为独立的 crate，便于管理和复用。

```
/ (Root)
├── Cargo.toml (Workspace definition)
├── AI_WORK_PROTOCOL.md
├── README.md
├── USER_GUIDE.md
├── PLUGIN_GUIDE.md
├── ARCHITECTURE_DESIGN.md  (Architecture design document)
├── plugins/          (Plugin directory)
├── rt-core/          (Core library: Tool trait, Plugin system, MCP, Services)
├── rt-tools/         (Built-in tool collection)
├── rt-cli/           (Command-line interface)
├── rt-gui/           (Graphical interface)
├── rt-plugin-pinyin/ (Single-tool plugin: Chinese to Pinyin)
├── rt-plugin-ytdlp/  (Single-tool plugin: Video downloader)
└── rt-plugin-czkawka/(Multi-tool plugin: File system utilities)
```

### 技术选型

#### 核心库 (`rt-core`)
*   **职责**：定义 `Tool` trait、`Plugin` 系统、`Locale` 枚举、错误处理机制、持久化管理 (`PersistenceManager`)、配置管理、日志记录、MCP 支持和服务层核心抽象。
*   **关键技术**：
    *   `serde` 用于 JSON/YAML 序列化/反序列化
    *   `thiserror` + `anyhow` 用于统一错误处理
    *   `sled` + `moka` 用于高性能持久化和缓存
    *   `chrono` 用于时间处理
    *   `uuid` 用于生成唯一标识符
    *   `tokio` 用于异步运行时
    *   `warp` 用于 Web 框架 (MCP 服务器)
    *   `tokio-tungstenite` 用于 WebSocket 支持

#### 模型上下文协议 (MCP) 支持
*   **设计目标**：实现标准化的上下文管理和工具调用协议，支持外部系统集成，确保上下文一致性和工具执行的可追溯性。
*   **核心组件**：
    *   **McpContext**：包含执行状态和历史的 MCP 上下文
    *   **McpRequest**：定义标准化工具调用格式的 MCP 请求
    *   **McpResponse**：定义标准化执行结果格式的 MCP 响应
    *   **ContextManager**：负责上下文创建、更新和传播的上下文管理器
    *   **McpServer**：提供 REST API 和 WebSocket 端点的 MCP 服务器

#### 增强的插件系统
*   **多工具插件支持**：通过基于数组的元数据格式支持单工具和多工具插件
*   **插件类型**：
    *   **进程插件**：外部可执行文件（单工具和多工具）
    *   **WASM 插件**：支持 WASI 的 WebAssembly 模块
*   **设计特点**：
    *   动态插件发现和加载
    *   基于数组的多工具插件元数据
    *   统一的工具接口，无论插件类型如何
    *   插件隔离和错误处理

### 架构模式

#### 六边形架构
- **核心**：`rt-core` 中的业务逻辑
- **端口**：外部接口的 trait 定义
- **适配器**：具体实现（CLI、GUI、插件、MCP 服务器）

#### 插件协议
- **进程插件**：通过 stdin/stdout JSON 通信的外部可执行文件
- **WASM 插件**：支持 WASI 的 WebAssembly 模块
- **多工具插件**：通过数组元数据提供多个工具的单个可执行文件
- **发现机制**：自动扫描 `plugins/` 目录中以 `rt-plugin-` 为前缀的文件

#### 面向服务设计
- **统一服务层**：带有基于角色访问控制的标准化服务调用
- **MCP 集成**：具有标准化协议的上下文感知工具执行
- **配置管理**：带有优先级和缓存的多源配置
- **全面日志记录**：支持多种输出目标的结构化日志

## 4. 接口契约 (Interface)

### Tool 特性
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称 (唯一标识)
    fn name(&self) -> &str;
    
    /// 显示名称 (支持多语言)
    fn display_name(&self, _locale: Locale) -> String;
    
    /// 工具描述 (用于 UI 展示)
    fn description(&self, locale: Locale) -> String;
    
    /// 用户指南 (Markdown 格式)
    fn user_guide(&self, locale: Locale) -> String;
    
    /// 输入参数 Schema (JSON Schema)
    fn input_schema(&self, locale: Locale) -> Value;
    
    /// 输出结果 Schema (JSON Schema)
    fn output_schema(&self, _locale: Locale) -> Value;
    
    /// 执行逻辑
    async fn run(&self, input: Value) -> Result<Value>;
    
    /// 是否支持 MCP
    fn mcp_supported(&self) -> bool { false }
    
    /// 使用 MCP 上下文执行工具
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse>;
}
```

### Plugin 接口
```rust
pub trait Plugin: Send + Sync {
    /// 获取插件名称
    fn name(&self) -> &str;
    
    /// 获取插件版本
    fn version(&self) -> &str;
    
    /// 获取插件提供的工具列表
    fn get_tools(&self) -> Vec<Box<dyn Tool>>;
    
    /// 初始化插件
    async fn initialize(&self) -> Result<()>;
    
    /// 关闭插件
    async fn shutdown(&self) -> Result<()>;
}
```

## 5. 变更记录 (Status)
> 格式：[状态] | 变更描述 | 日期

### 当前变更
- `[已完成]`：更新文档结构，统一命名规范，添加变更记录 | 2025-12-21

### 历史记录
- `[已完成]`：初始设计文档创建 | 2025-12-20

## 附加信息

### 开发工具与工作流
*   **代码格式化**：使用 `rustfmt` 确保一致的代码风格。
*   **静态分析**：使用 `clippy` 检查潜在的代码问题和风格建议。
*   **版本控制**：使用 `Git` 进行版本管理。
*   **任务管理**：使用 `tasks.md` 跟踪开发进度。
*   **测试**：使用 `cargo test` 进行全面的单元测试和集成测试。
*   **文档**：使用 `rustdoc` 生成内联文档，并维护外部文档文件。

### 数据交换与配置
*   **JSON**：插件协议和 Schema 定义的主要数据交换格式。
*   **JSON Schema**：用于定义工具输入和输出结构，支持数据验证和自动 GUI 生成。
*   **YAML**：与 JSON 一起支持配置文件。

### 错误处理
*   **统一错误类型**：使用 `thiserror` + `anyhow` 库创建和管理项目中的错误类型，确保清晰且易于传播的错误信息。
*   **避免 `unwrap()`/`expect()`**：在生产代码中严格禁止使用，强制正确的错误处理。
*   **结构化错误响应**：所有 API 返回带有错误代码和上下文的结构化错误信息。

### 国际化
- 基于 JSON 的国际化资源位于 `locales/` 目录中
- 运行时语言切换支持
- 用于动态 UI 生成的 Schema 标题注入

### 未来路线图
*   **高级插件管理**：版本控制、依赖管理和插件市场。
*   **增强的 GUI 框架**：探索更多 GUI 框架以提供更丰富的用户体验。
*   **性能优化**：持续优化性能和内存使用。
*   **分布式执行**：支持跨节点的工具和工作流执行。
*   **Web 界面**：用于远程访问和管理的基于浏览器的界面。
*   **高级 MCP 功能**：增强的上下文管理和外部系统集成。
*   **AI 集成**：原生支持 AI 驱动的工具建议和工作流优化。

### 相关文档
- [架构设计文档](ARCHITECTURE_DESIGN.md)：详细描述项目架构设计、核心组件和部署架构
- [持久化设计文档](rt-core/PERSISTENCE_DESIGN.md)：详细描述项目持久化设计
- [工作流设计文档](rt-core/WORKFLOW_DESIGN.md)：详细描述项目工作流设计