# Rust Toolbox Architecture Design Document

## 1. Project Overview

Rust Toolbox is a modular tool integration platform built in Rust that provides unified management and orchestration of various tools through a workflow engine. The platform features a plugin-based architecture supporting dynamic loading and extension, with both CLI and GUI interfaces. The system implements Model Context Protocol (MCP) for standardized context management and external system integration.

## 1.1 Related Documentation

- [Design Document](DESIGN.md): Overall project design document, including technology stack and core principles
- [User Guide](USER_GUIDE.md): Detailed user guide, including tool library and usage methods
- [Plugin Development Guide](PLUGIN_GUIDE.md): Plugin development specifications and guidelines
- [AI Work Protocol](AI_WORK_PROTOCOL.md): AI-assisted development work specifications
- [Changelog](CHANGELOG.md): Project change history
- [Persistence Design Document](rt-core/PERSISTENCE_DESIGN.md): Detailed description of project persistence design
- [Workflow Design Document](rt-core/WORKFLOW_DESIGN.md): Detailed description of project workflow design

## 1.2 Core Features

- **Unified Tool Management**: Centralized tool discovery, execution, and management
- **Plugin Architecture**: Dynamic loading of external tools as plugins (executable files and WebAssembly)
- **Workflow Engine**: DAG-based workflow orchestration for automated task processing
- **Multi-language Support**: Full internationalization (i18n) for English and Chinese
- **Persistence Layer**: Unified data storage, caching, and configuration management
- **Dual Interface**: Both command-line (rt-cli) and graphical (rt-gui) interfaces
- **Model Context Protocol (MCP)**: Standardized context management and tool calling protocol
- **MCP Server**: REST API and WebSocket endpoints for external system integration
- **Service Layer**: Unified service management with role-based access control
- **Configuration Management**: Multi-source configuration loading with caching and hot reload
- **Logging System**: Comprehensive logging with multiple output targets and structured logging

## 1.3 Technology Stack

- **Language**: Rust 2021 Edition
- **Build System**: Cargo Workspace
- **Async Runtime**: Tokio
- **Serialization**: Serde (JSON/YAML)
- **Error Handling**: thiserror + anyhow
- **CLI Framework**: Clap v4 with derive macros
- **GUI Framework**: egui (cross-platform, immediate mode)
- **Web Framework**: Warp (for MCP server REST API)
- **WebSocket**: tokio-tungstenite (for MCP WebSocket support)
- **Storage**: Sled embedded database
- **Caching**: moka (async in-memory cache)
- **Compression**: zstd + async-compression
- **Plugin System**: wasmtime (WebAssembly runtime) + process-based plugins

## 2. 架构设计

### 2.1 Overall Architecture

Rust Toolbox adopts a modular architecture design based on Cargo workspace structure. The core logic is centralized in the `rt-core` module, while other modules build specific functionality on top of the core module.

```
┌───────────────────────────────────────────────────────────────────────────┐
│                            Application Layer                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │    rt-cli       │  │    rt-gui       │  │ External Apps   │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└───────────────────────────────────────────────────────────────────────────┘
                                  │
┌───────────────────────────────────────────────────────────────────────────┐
│                              Core Layer                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │   Tool API      │  │ Workflow Engine │  │ Plugin Manager  │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │ Persistence API │  │ Config Manager  │  │ Service Layer   │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │    MCP API      │  │   MCP Server    │  │ Logger System   │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└───────────────────────────────────────────────────────────────────────────┘
                                  │
┌───────────────────────────────────────────────────────────────────────────┐
│                             Plugin Layer                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │ Process Plugin  │  │  WASM Plugin    │  │ Multi-tool      │            │
│  │ (Single Tool)   │  │ (Single Tool)   │  │ Plugin          │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└───────────────────────────────────────────────────────────────────────────┘
                                  │
┌───────────────────────────────────────────────────────────────────────────┐
│                              Tool Layer                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │   rt-tools      │  │ rt-plugin-pinyin│  │rt-plugin-czkawka│            │
│  │ (Built-in)      │  │ (Single Tool)   │  │ (Multi-tool)    │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│  ┌─────────────────┐                                                       │
│  │rt-plugin-ytdlp  │                                                       │
│  │ (Single Tool)   │                                                       │
│  └─────────────────┘                                                       │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.2 MCP Integration Architecture

The Model Context Protocol (MCP) integration provides standardized context management and external system integration:

```
┌───────────────────────────────────────────────────────────────────────────┐
│                          External Systems                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │   AI Agents     │  │   Web Apps      │  │   IDEs/Editors  │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└─────────────┬─────────────────┬─────────────────┬─────────────────┘       │
              │                 │                 │                          │
              ▼                 ▼                 ▼                          │
┌───────────────────────────────────────────────────────────────────────────┐
│                            MCP Server                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │  REST API       │  │  WebSocket      │  │  Context Mgmt   │            │
│  │  Endpoints      │  │  Server         │  │  Service        │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└─────────────┬─────────────────┬─────────────────┬─────────────────┘       │
              │                 │                 │                          │
              ▼                 ▼                 ▼                          │
┌───────────────────────────────────────────────────────────────────────────┐
│                          MCP Core Layer                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │ Context Manager │  │  MCP Workflow   │  │   MCP Node      │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│  ┌─────────────────┐  ┌─────────────────┐                                  │
│  │ Request Handler │  │ Response Builder│                                  │
│  └─────────────────┘  └─────────────────┘                                  │
└─────────────┬─────────────────┬─────────────────┬─────────────────┘       │
              │                 │                 │                          │
              ▼                 ▼                 ▼                          │
┌───────────────────────────────────────────────────────────────────────────┐
│                        Tool Execution Layer                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │  Built-in Tools │  │    Plugins      │  │   Workflows     │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Hexagonal Architecture

The core module `rt-core` adopts Hexagonal Architecture design, also known as Ports and Adapters pattern. The core idea is to separate business logic from external dependencies through ports (interfaces) and adapters (implementations), improving system testability and extensibility.

```
┌───────────────────────────────────────────────────────────────────────────┐
│                            External Systems                               │
└─────────────┬─────────────────┬─────────────────┬─────────────────┘       │
              │                 │                 │                          │
┌─────────────▼─────────┐ ┌─────▼─────────────┐ ┌▼─────────────────┐        │
│   CLI Adapter         │ │   GUI Adapter     │ │ MCP Server       │        │
└─────────────┬─────────┘ └─────┬─────────────┘ └▲─────────────────┘        │
              │                 │                 │                          │
┌─────────────▼─────────────────▼─────────────────▼─────────────────┐        │
│                              Port Layer                           │        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │        │
│  │   Tool Port     │  │ Workflow Port   │  │ Plugin Port     │       │        │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘       │        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │        │
│  │    MCP Port     │  │ Service Port    │  │ Config Port     │       │        │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘       │        │
│  ┌─────────────────┐  ┌─────────────────┐                              │        │
│  │ Persistence Port│  │  Logger Port    │                              │        │
│  └─────────────────┘  └─────────────────┘                              │        │
└─────────────┬─────────────────┬─────────────────┬─────────────────┘        │
              │                 │                 │                          │
┌─────────────▼─────────────────▼─────────────────▼─────────────────┐        │
│                            Domain Layer                           │        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │        │
│  │   Tool Domain   │  │ Workflow Domain │  │ Plugin Domain   │       │        │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘       │        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │        │
│  │    MCP Domain   │  │ Service Domain  │  │ Config Domain   │       │        │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘       │        │
│  ┌─────────────────┐  ┌─────────────────┐                              │        │
│  │Persistence Domain│  │ Logger Domain   │                              │        │
│  └─────────────────┘  └─────────────────┘                              │        │
└─────────────┬─────────────────┬─────────────────┬─────────────────┘        │
              │                 │                 │                          │
┌─────────────▼─────────┐ ┌─────▼─────────────┐ ┌▼─────────────────┐        │
│   File Adapter        │ │  Cache Adapter    │ │ Database Adapter │        │
└─────────────┬─────────┘ └─────┬─────────────┘ └▲─────────────────┘        │
              │                 │                 │                          │
┌───────────────────────────────────────────────────────────────────────────┐
│                         Infrastructure Layer                              │
└───────────────────────────────────────────────────────────────────────────┘
```

## 3. Core Component Design

### 3.1 Tool Component

Tool is the core abstraction of the entire system, defining the basic behavior and interface of tools. All tools and plugins must implement the Tool interface.

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (unique identifier)
    fn name(&self) -> &str;
    
    /// Display name (multi-language support)
    fn display_name(&self, _locale: Locale) -> String;
    
    /// Tool description (for UI display)
    fn description(&self, locale: Locale) -> String;
    
    /// User guide (Markdown format)
    fn user_guide(&self, locale: Locale) -> String;
    
    /// Input parameter schema (JSON Schema)
    fn input_schema(&self, locale: Locale) -> Value;
    
    /// Output result schema (JSON Schema)
    fn output_schema(&self, _locale: Locale) -> Value;
    
    /// Execution logic
    async fn run(&self, input: Value) -> Result<Value>;
    
    /// Whether MCP is supported
    fn mcp_supported(&self) -> bool {
        false
    }
    
    /// Execute tool with MCP context
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse>;
}

/// MCP Tool trait, extends Tool trait to provide MCP-specific functionality
#[async_trait]
pub trait McpTool: Tool {
    /// Get MCP capability description
    fn get_mcp_capabilities(&self) -> Value;
    
    /// Get MCP context validation rules
    fn get_context_validation_rules(&self) -> Value;
    
    /// Whether full context is required
    fn requires_full_context(&self) -> bool;
    
    /// Execution logic (with MCP context)
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse>;
}
```

### 3.2 Service Layer

The service layer provides standardized service invocation interfaces with permission control and error handling.

```rust
/// Service manager for unified service registration and invocation
pub struct ServiceManager {
    services: RwLock<HashMap<String, Arc<dyn ServicePort>>>,
    permissions: RwLock<HashMap<String, Vec<String>>>, // role -> services
}

/// Service port defining service invocation interface
#[async_trait]
pub trait ServicePort: Send + Sync {
    async fn call(&self, request: ServiceRequest) -> Result<ServiceResponse>;
    fn get_permissions(&self) -> Vec<String>;
}

/// Service request containing parameters and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: String,
    pub service_name: String,
    pub method: String,
    pub params: Value,
    pub context: Option<McpContext>,
    pub user_role: Option<String>,
}

/// Service response containing data and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub id: String,
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
    pub context: Option<McpContext>,
}
```

### 3.3 Configuration Management

The configuration management module provides unified configuration services supporting multi-source loading, caching, and hot reload.

```rust
/// Configuration manager providing external access interface
pub struct ConfigManager {
    sources: Vec<Box<dyn ConfigSourcePort>>,
    cache: Arc<dyn ConfigCachePort>,
    watchers: RwLock<HashMap<String, Vec<ConfigWatcher>>>,
}

/// Configuration source port defining configuration loading interface
#[async_trait]
pub trait ConfigSourcePort: Send + Sync {
    async fn load_config(&self, key: &str) -> Result<Option<Value>>;
    fn get_priority(&self) -> u32;
    fn supports_watch(&self) -> bool;
}

/// Configuration cache port defining configuration caching interface
#[async_trait]
pub trait ConfigCachePort: Send + Sync {
    async fn get(&self, key: &str) -> Option<Value>;
    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>);
    async fn invalidate(&self, key: &str);
}
```

### 3.4 Logging System

The logging system provides comprehensive logging services with hierarchical logging, multiple output targets, and structured logging.

```rust
/// Log manager providing external access interface
pub struct LogManager {
    writers: Vec<Arc<dyn LogWriterPort>>,
    formatters: HashMap<String, Arc<dyn LogFormatterPort>>,
    sinks: Vec<Arc<dyn LogSinkPort>>,
    level: LogLevel,
}

/// Log writer port defining log writing interface
#[async_trait]
pub trait LogWriterPort: Send + Sync {
    async fn write(&self, entry: &LogEntry) -> Result<()>;
    fn supports_level(&self, level: LogLevel) -> bool;
}

/// Log sink port defining log output interface
#[async_trait]
pub trait LogSinkPort: Send + Sync {
    async fn emit(&self, formatted: &str) -> Result<()>;
    fn get_name(&self) -> &str;
}

/// Log formatter port defining log formatting interface
pub trait LogFormatterPort: Send + Sync {
    fn format(&self, entry: &LogEntry) -> String;
    fn get_format_name(&self) -> &str;
}
```

### 3.5 Plugin System

The plugin system is responsible for loading, managing, and executing plugins. It supports multiple types of plugins:

1. **Process Plugin (Single Tool)**: Independent executable files that communicate with the main program through inter-process communication
2. **Process Plugin (Multi-Tool)**: Independent executable files that provide multiple tools through array-based metadata
3. **WASM Plugin**: WebAssembly modules that execute directly within the main program

#### 3.5.1 PluginManager

PluginManager is the core component of the plugin system, responsible for plugin loading and management.

```rust
pub struct PluginManager {
    plugin_dir: PathBuf,
    plugins: RwLock<HashMap<String, Arc<dyn Tool>>>,
    multi_tool_plugins: RwLock<HashMap<String, Vec<Arc<dyn Tool>>>>,
}
```

Key features:
- Load plugins from specified directory
- Manage loaded plugins (both single-tool and multi-tool)
- Provide plugin query and execution interfaces
- Support for array-based metadata format for multi-tool plugins

#### 3.5.2 Multi-Tool Plugin Architecture

Multi-tool plugins support providing multiple tools through a single plugin executable:

```rust
/// Plugin metadata for multi-tool plugins (array format)
pub type MultiToolMetadata = Vec<PluginMetadata>;

/// Individual tool metadata within a multi-tool plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub display_name: LocalizedString,
    pub description: LocalizedString,
    pub user_guide: LocalizedString,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub input_fields: Option<HashMap<String, HashMap<String, String>>>,
    pub output_fields: Option<HashMap<String, HashMap<String, String>>>,
    pub mcp_supported: bool,
    pub mcp_capabilities: Value,
    pub requires_full_context: bool,
    pub context_validation_rules: Value,
}
```

#### 3.5.3 Plugin Loading Flow

```
┌───────────────────────────────────────────────────────────────────────────┐
│                            Load Plugins                                   │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                    Scan Files in Plugin Directory                         │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ├─────────────────────────────────────────────────────────────────┤
        │                                                                 │
        ▼                                                                 ▼
┌─────────────────┐                                               ┌─────────┐
│ Is .wasm file?  │                                               │Other file?│
└─────────┬───────┘                                               └─────┬───┘
          │                                                               │
          ▼                                                               ▼
┌─────────────────┐                                               ┌─────────┐
│ Load WASM Plugin│                                               │Check filename│
└─────────┬───────┘                                               └─────┬───┘
          │                                                               │
          ▼                                                               ▼
┌─────────────────┐                                               ┌─────────┐
│Create WasmPlugin│                                               │Starts with rt-plugin-?│
└─────────┬───────┘                                               └─────┬───┘
          │                                                               │
          ├───────────────────────────────────────────────────────────────┤
          │                                                               │
          ▼                                                               ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                        Create ProcessPlugin                               │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                      Execute "plugin spec" Command                        │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ├─────────────────────────────────────────────────────────────────┤
        │                                                                 │
        ▼                                                                 ▼
┌─────────────────┐                                               ┌─────────┐
│ Array Response? │                                               │Object Response?│
│ (Multi-tool)    │                                               │(Single-tool)│
└─────────┬───────┘                                               └─────┬───┘
          │                                                               │
          ▼                                                               ▼
┌─────────────────┐                                               ┌─────────┐
│Register Multiple│                                               │Register Single│
│Tools from Array │                                               │Tool from Object│
└─────────┬───────┘                                               └─────┬───┘
          │                                                               │
          ├───────────────────────────────────────────────────────────────┤
          │                                                               │
          ▼                                                               ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                        Register to PluginManager                          │
└───────────────────────────────────────────────────────────────────────────┘
```

### 3.6 Model Context Protocol (MCP) Components

The MCP system provides standardized context management and external system integration capabilities.

#### 3.6.1 Core MCP Components

```rust
/// MCP Context - maintains execution state and history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    pub id: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
    pub execution_history: Vec<ExecutionRecord>,
}

/// MCP Request - standardized tool invocation format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub context_id: String,
    pub tool_name: String,
    pub parameters: Value,
    pub metadata: HashMap<String, Value>,
}

/// MCP Response - standardized execution result format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,
    pub request_id: String,
    pub context_id: String,
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Context Manager - manages context lifecycle
pub struct ContextManager {
    contexts: RwLock<HashMap<String, McpContext>>,
    persistence: Arc<PersistenceManager>,
}
```

#### 3.6.2 MCP Server Architecture

The MCP server provides REST API and WebSocket endpoints for external system integration:

```rust
/// MCP Server providing REST API and WebSocket endpoints
pub struct McpServer {
    api_server: ApiServer,
    websocket_server: WebSocketServer,
    context_manager: Arc<ContextManager>,
    tool_manager: Arc<ToolManager>,
}

/// REST API endpoints
impl ApiServer {
    // GET /api/tools - List available tools
    // POST /api/tools/{name}/execute - Execute tool
    // GET /api/contexts - List contexts
    // POST /api/contexts - Create context
    // GET /api/contexts/{id} - Get context
    // PUT /api/contexts/{id} - Update context
    // GET /api/workflows - List workflows
    // POST /api/workflows - Create workflow
    // POST /api/workflows/{id}/execute - Execute workflow
}

/// WebSocket server for real-time communication
impl WebSocketServer {
    // Real-time tool execution
    // Context updates
    // Workflow progress notifications
    // Error notifications
}
```

### 3.7 Workflow Engine

The workflow engine is responsible for defining and executing workflows, supporting dependency management between nodes and parallel execution. It now includes enhanced support for multi-tool plugins and MCP integration.

#### 3.7.1 Core Concepts

- **WorkflowDefinition**: Workflow definition containing nodes and edges
- **WorkflowNode**: Workflow node corresponding to a tool execution (supports multi-tool plugin tools)
- **WorkflowEdge**: Workflow edge defining dependencies between nodes
- **WorkflowInstance**: Workflow instance representing an executing workflow
- **WorkflowStatus**: Workflow status including Pending, Running, Paused, Completed, Failed
- **NodeStatus**: Node status including Pending, Running, Completed, Failed, Skipped
- **McpWorkflow**: MCP-enabled workflow with context propagation

#### 3.7.2 Enhanced Workflow Definition

```rust
/// Enhanced workflow definition with MCP support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub mcp_enabled: bool,
    pub context_requirements: Option<ContextRequirements>,
}

/// Enhanced workflow node supporting multi-tool plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub tool_name: String, // Can reference tools from multi-tool plugins
    pub plugin_source: Option<String>, // Plugin name for multi-tool plugins
    pub label: Option<String>,
    pub input_mappings: HashMap<String, String>,
    pub static_inputs: Value,
    pub mcp_context_required: bool,
}
```

#### 3.7.3 Workflow Execution Flow

```
┌───────────────────────────────────────────────────────────────────────────┐
│                           启动工作流                                      │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                          验证工作流定义                                    │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                          创建工作流实例                                    │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                          执行工作流循环                                    │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ├─────────────────────────────────────────────────────────────────┤
        │                                                                 │
        ▼                                                                 ▼
┌─────────────────┐                                               ┌─────────┐
│ 检查工作流状态   │                                               │ 查找可执行节点 │
└─────────┬───────┘                                               └─────┬───┘
          │                                                               │
          ▼                                                               ▼
┌─────────────────┐                                               ┌─────────┐
│ 工作流已完成？   │                                               │ 准备节点输入 │
└─────────┬───────┘                                               └─────┬───┘
          │                                                               │
          ▼                                                               ▼
┌─────────────────┐                                               ┌─────────┐
│ 是 → 结束循环    │                                               │ 并行执行节点 │
└─────────────────┘                                               └─────┬───┘
                                                                         │
                                                                         ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                          更新节点状态和上下文                            │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                          更新工作流状态                                    │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                          继续循环或结束                                    │
└───────────────────────────────────────────────────────────────────────────┘
```

#### 3.3.3 输入映射和模板

工作流节点支持通过模板和映射来动态生成输入参数：

- **静态输入**: 直接定义的固定输入值
- **输入映射**: 从其他节点的输出中提取值
- **模板**: 使用 `{{ node.output.field }}` 语法引用其他节点的输出

### 3.4 持久化设计

持久化模块负责数据的持久化存储，支持多种后端实现。当前实现了基于 Sled 的后端。

#### 3.4.1 核心组件

- **PersistenceManager**: 持久化管理器，负责数据的存储和检索
- **Cache**: 缓存层，提供快速数据访问
- **Backend**: 存储后端接口，支持不同的存储实现

#### 3.4.2 数据模型

- **工具元数据**: 工具的基本信息和描述
- **工作流定义**: 工作流的结构和配置
- **工作流实例**: 工作流的执行状态和结果
- **插件元数据**: 插件的基本信息和描述

## 4. 模块设计

### 4.1 rt-core

核心模块，包含系统的核心逻辑和接口定义：

- **tool**: 工具接口定义
- **plugin**: 插件系统实现
- **workflow**: 工作流引擎实现
- **persistence**: 持久化实现
- **locale**: 多语言支持
- **error**: 错误处理

### 4.2 rt-tools

工具集合模块，包含内置工具的实现。

### 4.3 rt-cli

命令行接口模块，提供命令行方式使用系统功能：

- 工具执行
- 工作流管理
- 插件管理

### 4.4 rt-gui

图形用户界面模块，提供可视化方式使用系统功能：

- 工具执行界面
- 工作流编辑器
- 插件管理界面

### 4.5 rt-plugin-pinyin

拼音插件，提供拼音转换功能。

### 4.6 rt-plugin-ytdlp

YouTube 下载插件，基于 yt-dlp 实现视频下载功能。

## 5. 依赖关系

| 模块           | 依赖模块   | 说明                           |
|--------------|--------|------------------------------|
| rt-cli       | rt-core| 使用核心模块的接口实现命令行功能            |
| rt-gui       | rt-core| 使用核心模块的接口实现图形界面功能           |
| rt-plugin-pinyin | rt-core | 实现核心模块定义的插件接口             |
| rt-plugin-ytdlp | rt-core | 实现核心模块定义的插件接口             |
| rt-tools     | rt-core | 实现核心模块定义的工具接口             |

## 6. 部署架构

### 6.1 单机部署

```
┌───────────────────────────────────────────────────────────────────────────┐
│                              单机部署                                    │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                           主程序进程                                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │   rt-core       │  │   rt-cli        │  │   rt-gui        │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ├─────────────────────────────────────────────────────────────────┤
        │                                                               │
        ▼                                                               ▼
┌─────────────────────────────────┐                       ┌─────────────────┐
│          插件进程               │                       │   Wasm 插件     │
│  ┌─────────────────┐           │                       └─────────────────┘
│  │ rt-plugin-pinyin│           │
│  └─────────────────┘           │
│                                 │
│  ┌─────────────────┐           │
│  │rt-plugin-ytdlp  │           │
│  └─────────────────┘           │
└─────────────────────────────────┘
```

### 6.2 多机部署

多机部署架构允许将工作流引擎和插件分布在多个节点上执行，提高系统的扩展性和容错性。

```
┌───────────────────────────────────────────────────────────────────────────┐
│                              客户端层                                    │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │    rt-cli       │  │    rt-gui       │  │    其他客户端   │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                              服务层                                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │   API 网关      │  │ 工作流协调器     │  │ 任务调度器      │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└───────────────────────────────────────────────────────────────────────────┘
        │
        ├─────────────────────────────────────────────────────────────────┤
        │                                                               │
        ▼                                                               ▼
┌─────────────────────────────────┐                       ┌─────────────────┐
│         执行节点集群             │                       │   存储节点集群   │
│  ┌─────────────────┐           │                       └─────────────────┘
│  │ 工作流引擎实例  │           │
│  └─────────────────┘           │
│                                 │
│  ┌─────────────────┐           │
│  │ 插件执行实例    │           │
│  └─────────────────┘           │
└─────────────────────────────────┘
```

## 7. 扩展机制

### 7.1 添加新工具

1. 实现 `Tool` 接口
2. 注册到工具注册表

### 7.2 添加新插件

1. 创建新的插件项目
2. 实现 `Tool` 接口
3. 构建插件
4. 将插件文件放到插件目录中

### 7.3 添加新的存储后端

1. 实现 `Backend` 接口
2. 注册到 `PersistenceManager`

## 8. 性能优化

### 8.1 异步设计

系统采用异步设计，使用 Tokio 异步运行时，支持高并发执行。

### 8.2 并行执行

工作流引擎支持并行执行无依赖的节点，提高工作流执行效率。

### 8.3 缓存机制

持久化层实现了缓存机制，减少对存储后端的访问次数。

### 8.4 插件隔离

插件执行采用隔离机制，避免插件之间的相互影响，提高系统的稳定性。

## 9. 监控和日志

系统使用 Tracing 框架实现日志记录，支持不同级别的日志输出：

- **DEBUG**: 调试信息，用于开发和调试
- **INFO**: 普通信息，用于记录系统运行状态
- **WARN**: 警告信息，用于记录潜在问题
- **ERROR**: 错误信息，用于记录系统错误

日志包含时间戳、线程名、函数名和关键上下文信息，便于问题定位和分析。

## 10. 未来规划

1. **分布式工作流**: 支持跨节点的工作流执行
2. **更多存储后端**: 支持更多类型的存储后端，如 PostgreSQL、MongoDB 等
3. **Web 界面**: 提供基于 Web 的界面，支持远程访问
4. **更多插件类型**: 支持更多类型的插件，如 Python 插件、JavaScript 插件等
5. **工作流版本管理**: 支持工作流定义的版本管理
6. **工作流模板库**: 提供常用工作流模板，方便用户快速创建工作流
7. **高级调度功能**: 支持定时执行、事件触发等高级调度功能

## 11. 总结

Rust Tool 采用模块化、插件化、六边形架构设计，具有良好的可扩展性、可测试性和可维护性。系统支持多种工具的统一管理和调度，提供工作流引擎实现工具间的协同工作，同时提供 CLI 和 GUI 两种交互方式，满足不同用户的需求。

该架构设计为系统的未来发展提供了良好的基础，支持分布式部署、更多插件类型和更丰富的功能扩展。