# Rust Toolbox Project Design Document

## 1. Overview

This document records the overall architectural design, technology selection, and core principles of the Rust Toolbox project, providing guidance and reference for project development.

## 1.1 Related Documentation

- [Architecture Design Document](ARCHITECTURE_DESIGN.md): Detailed description of project architecture design, core components and deployment architecture
- [Persistence Design Document](rt-core/PERSISTENCE_DESIGN.md): Detailed description of project persistence design
- [Workflow Design Document](rt-core/WORKFLOW_DESIGN.md): Detailed description of project workflow design

## 2. Core Principles

*   **Modularity and Decoupling**: Emphasizes separation of core logic from UI layer, ensuring single responsibility for each component, easy maintenance and extension.
*   **Extensibility**: Supports dynamic integration of external tools through plugin system, improving project flexibility and vitality.
*   **Multi-language Support**: All user-facing text supports multiple languages, enhancing user experience.
*   **Persistence Management**: Provides unified data storage, caching, and configuration management services, supporting modularity and high-performance read/write operations (see `rt-core/PERSISTENCE_DESIGN.md`).
*   **Workflow Orchestration**: Supports orchestrating multiple tools through DAG (Directed Acyclic Graph) for automated task processing (see `rt-core/WORKFLOW_DESIGN.md`).
*   **Model Context Protocol (MCP) Support**: Implements standardized context management and tool calling protocol, supporting external system integration.
*   **Service-Oriented Architecture**: Provides unified service layer with role-based access control and standardized error handling.
*   **Configuration-Driven Design**: Supports multi-source configuration loading with caching, hot reload, and priority management.
*   **Comprehensive Logging**: Implements structured logging with multiple output targets and hierarchical log levels.
*   **Multi-Tool Plugin Architecture**: Supports both single-tool and multi-tool plugins through array-based metadata format.
*   **Design-First Approach**: Any new features or major changes require design documentation first.
*   **Documentation Consistency**: Code changes are synchronized with documentation updates to ensure accuracy and timeliness.

## 3. Project Structure (Cargo Workspace)

The project adopts Cargo Workspace structure, organizing different functional modules as independent crates for easy management and reuse. For detailed architectural design, please refer to [Architecture Design Document](ARCHITECTURE_DESIGN.md).

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

## 4. Technology Selection

### 4.1 Programming Language

*   **Rust 2021 Edition**: Primary development language, leveraging its high performance, memory safety, and concurrency advantages.

### 4.2 Core Library (`rt-core`)

*   **Responsibilities**: Defines `Tool` trait, `Plugin` system, `Locale` enumeration, error handling mechanisms, persistence management (`PersistenceManager`), configuration management, logging, MCP support, and service layer core abstractions. For detailed architectural design, please refer to [Architecture Design Document](ARCHITECTURE_DESIGN.md).
*   **Key Technologies**: 
    *   `serde` for JSON/YAML serialization/deserialization
    *   `thiserror` + `anyhow` for unified error handling
    *   `sled` + `moka` for high-performance persistence and caching
    *   `chrono` for time handling
    *   `uuid` for unique identifier generation
    *   `tokio` for async runtime
    *   `warp` for web framework (MCP server)
    *   `tokio-tungstenite` for WebSocket support

### 4.3 Model Context Protocol (MCP) Support

*   **Design Goals**: Implement standardized context management and tool calling protocol, support external system integration, ensure context consistency and traceability of tool execution.
*   **Core Components**:
    *   **McpContext**: MCP context containing execution state and history
    *   **McpRequest**: MCP request defining standardized tool invocation format
    *   **McpResponse**: MCP response defining standardized execution result format
    *   **ContextManager**: Context manager responsible for context creation, updates, and propagation
    *   **McpServer**: MCP server providing REST API and WebSocket endpoints
*   **Technology Selection**:
    *   REST API: Implemented using `warp` framework
    *   WebSocket: Based on `warp` and `tokio-tungstenite`
    *   JSON: Data exchange format for MCP protocol
    *   UUID: For unique identification of contexts and requests
*   **Design Features**:
    *   Hierarchical context relationship management
    *   Execution history tracking
    *   Support for synchronous and asynchronous calls
    *   Standardized error handling
    *   WebSocket real-time communication support

### 4.4 Configuration Management Module (`rt-core::config`)

*   **Design Goals**: Provide unified configuration management services, support multi-source configuration loading, caching, and hot reload, ensuring configuration consistency and reliability.
*   **Core Components**:
    *   **ConfigService**: Configuration service core logic for loading, merging, and managing configurations
    *   **ConfigManager**: Configuration manager providing external access interface
    *   **ConfigSourcePort**: Configuration source port defining configuration loading interface
    *   **ConfigCachePort**: Configuration cache port defining configuration caching interface
*   **Technology Selection**:
    *   Support for JSON and YAML format configuration files
    *   Support for environment variable configuration
    *   Use `serde` for configuration serialization/deserialization
*   **Design Features**:
    *   Multi-source configuration priority management
    *   Type-safe configuration item access
    *   Configuration caching and hot reload support

### 4.5 Logging System Module (`rt-core::logger`)

*   **Design Goals**: Provide comprehensive logging services, support hierarchical logging, multiple output targets, and structured logging for system monitoring and troubleshooting.
*   **Core Components**:
    *   **LogService**: Logging service core logic for log processing and distribution
    *   **LogManager**: Log manager providing external access interface
    *   **LogWriterPort**: Log writer port defining log writing interface
    *   **LogSinkPort**: Log sink port defining log output interface
    *   **LogFormatterPort**: Log formatter port defining log formatting interface
*   **Technology Selection**:
    *   Hierarchical logging: DEBUG, INFO, WARN, ERROR
    *   Support for console and file output
    *   Support for text and JSON formatting
    *   Use `chrono` for timestamp processing
*   **Design Features**:
    *   Log rotation support
    *   In-memory log storage
    *   Structured logging support
    *   Extensible log output targets

### 4.6 Service Layer (`rt-core::service`)

*   **Design Goals**: Provide standardized service invocation interfaces with permission control and error handling, ensuring security and efficiency of tool and plugin calls.
*   **Core Components**:
    *   **ServiceManager**: Service manager for service registration and invocation
    *   **ServicePort**: Service port defining service invocation interface
    *   **ServiceRequest**: Service request containing parameters and context
    *   **ServiceResponse**: Service response containing data and status
*   **Technology Selection**:
    *   JSON-based service invocation protocol
    *   Support for asynchronous service calls
    *   Use `uuid` for request tracking IDs
*   **Design Features**:
    *   Role-based access control
    *   Standardized error handling
    *   Unified service invocation entry point
    *   Support for unified tool and plugin calls

### 4.7 Enhanced Plugin System

*   **Multi-Tool Plugin Support**: Supports both single-tool and multi-tool plugins through array-based metadata format
*   **Plugin Types**:
    *   **Process Plugins**: External executables (single-tool and multi-tool)
    *   **WASM Plugins**: WebAssembly modules with WASI support
*   **Technology Selection**:
    *   `wasmtime` for WebAssembly runtime
    *   `wasmtime-wasi` for WASI support
    *   JSON-based metadata and communication protocol
*   **Design Features**:
    *   Dynamic plugin discovery and loading
    *   Array-based metadata for multi-tool plugins
    *   Unified tool interface regardless of plugin type
    *   Plugin isolation and error handling

### 4.8 Built-in Tool Collection (`rt-tools`)

*   **Responsibilities**: Implement specific tool logic such as file operations, text processing, etc.
*   **Guiding Principles**: Each tool should implement the `rt-core::Tool` trait and provide multi-language metadata and JSON Schema.
*   **Dependency Selection**: Prioritize pure Rust implementations to avoid complex C/C++ dependencies.
*   **Current Tools**:
    *   **File Operations**: `file.move_folder` - Move/rename folders with collision handling
    *   **Text Processing**: `text.ac_automaton` - Aho-Corasick pattern matching, `text.convert_chinese` - Traditional/Simplified Chinese conversion

### 4.9 Command Line Interface (`rt-cli`)

*   **Responsibilities**: Provide command-line interface for discovering, running, and managing tools, including MCP server functionality.
*   **Key Technologies**: `clap` v4 with derive macros for command-line argument parsing.
*   **Features**:
    *   Tool listing and execution
    *   Plugin management
    *   Workflow execution
    *   MCP server startup

### 4.10 Graphical User Interface (`rt-gui`)

*   **Responsibilities**: Provide graphical interface for displaying tool lists, auto-generating forms, and showing tool execution results.
*   **Key Technologies**: `egui` as GUI framework, leveraging its lightweight and cross-platform characteristics; dynamic UI generation through JSON Schema.
*   **Features**:
    *   Dynamic form generation from JSON Schema
    *   Multi-language UI support
    *   Real-time tool execution feedback
    *   Plugin management interface

### 4.11 Enhanced Plugin System

*   **Architecture**: Based on external executable protocol with enhanced support for multi-tool plugins. Uses `plugin spec` to get metadata (array or object format), `plugin run` to execute tool logic.
*   **Protocol**: JSON format metadata and input/output data ensuring cross-language and cross-platform compatibility.
*   **Multi-Tool Support**: Plugins can provide multiple tools through array-based metadata format.
*   **Multi-language**: Plugin metadata (`display_name`, `description`, `user_guide`, `input_fields`, `output_fields`) supports multiple languages, handled uniformly by `rt-core`.

### 4.12 Data Exchange and Configuration

*   **JSON**: Primary data exchange format for plugin protocols and Schema definitions.
*   **JSON Schema**: Used to define tool input and output structures, enabling data validation and automatic GUI generation.
*   **YAML**: Supported for configuration files alongside JSON.

### 4.13 Error Handling

*   **Unified Error Types**: Use `thiserror` + `anyhow` libraries to create and manage error types in the project, ensuring clear and easily propagated error information.
*   **Avoid `unwrap()`/`expect()`**: Strictly prohibited in production code, enforcing proper error handling.
*   **Structured Error Responses**: All APIs return structured error information with error codes and context.

## 5. Development Tools and Workflow

*   **Code Formatting**: `rustfmt` ensures consistent code style.
*   **Static Analysis**: `clippy` checks for potential code issues and style suggestions.
*   **Version Control**: `Git` for version management.
*   **Task Management**: `tasks.md` for tracking development progress.
*   **Testing**: Comprehensive unit and integration testing with `cargo test`.
*   **Documentation**: Inline documentation with `rustdoc` and external documentation files.

## 6. Architectural Patterns

### 6.1 Hexagonal Architecture
- **Core**: Business logic in `rt-core`
- **Ports**: Trait definitions for external interfaces
- **Adapters**: Concrete implementations (CLI, GUI, plugins, MCP server)

### 6.2 Plugin Protocol
- **Process Plugins**: External executables communicating via stdin/stdout JSON
- **WASM Plugins**: WebAssembly modules with WASI support
- **Multi-Tool Plugins**: Single executable providing multiple tools via array metadata
- **Discovery**: Auto-scan `plugins/` directory for files prefixed with `rt-plugin-`

### 6.3 Service-Oriented Design
- **Unified Service Layer**: Standardized service invocation with role-based access control
- **MCP Integration**: Context-aware tool execution with standardized protocols
- **Configuration Management**: Multi-source configuration with priority and caching
- **Comprehensive Logging**: Structured logging with multiple output targets

### 6.4 Internationalization
- JSON-based i18n resources in `locales/` directories
- Runtime locale switching
- Schema title injection for dynamic UI generation

## 7. Future Roadmap

*   **Advanced Plugin Management**: Version control, dependency management, and plugin marketplace.
*   **Enhanced GUI Framework**: Explore additional GUI frameworks for richer user experiences.
*   **Performance Optimization**: Continuous optimization of performance and memory usage.
*   **Distributed Execution**: Support for cross-node tool and workflow execution.
*   **Web Interface**: Browser-based interface for remote access and management.
*   **Advanced MCP Features**: Enhanced context management and external system integrations.
*   **AI Integration**: Native support for AI-powered tool suggestions and workflow optimization.