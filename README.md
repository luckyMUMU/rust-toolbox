# Workflow Toolkit

A comprehensive multi-interface workflow execution system built with Rust that provides flexible workflow management through CLI, TUI, and MCP server interfaces with advanced file management capabilities.

## Features

### Core Workflow Engine
- **Multi-Interface Support**: CLI commands, Terminal UI (TUI), and Model Context Protocol (MCP) server
- **DAG-based Workflow Engine**: Directed Acyclic Graph execution with conditions, loops, and parallel processing
- **State Management**: Persistent workflow state with checkpoint and recovery capabilities
- **Configuration Management**: Hierarchical configuration with environment variables, config files, and CLI parameter overrides

### Plugin System
- **Extensible Architecture**: Support for Native (Rust), Python, Node.js, Docker, and WebAssembly plugins
- **Tool Registry**: Reusable tool nodes for workflows and standalone execution
- **Plugin Manager**: Dynamic loading and lifecycle management of plugins

### File Management Tools
- **Interactive Classification**: AI-powered file classification with human decision support
- **Batch Processing**: Efficient bulk file operations with progress tracking
- **Text Processing**: Advanced text analysis and transformation capabilities
- **Result Confirmation**: Human-in-the-loop validation for critical operations
- **Performance Monitoring**: Built-in profiling and metrics collection

### Advanced Features
- **Async Execution**: Full async/await support with tokio runtime
- **Caching System**: High-performance caching with moka
- **Vector Storage**: Optional LanceDB integration for advanced data operations
- **Audit Logging**: Comprehensive operation tracking and logging
- **Error Recovery**: Robust error handling with retry mechanisms

## Quick Start

### Prerequisites

- **Rust**: 1.70+ (2021 Edition)
- **System**: Linux, macOS, or Windows
- **Memory**: Minimum 2GB RAM, recommended 4GB+
- **Storage**: At least 1GB available space

### Installation

```bash
# Clone the repository
git clone https://github.com/workflow-toolkit/workflow-toolkit.git
cd workflow-toolkit

# Build the project (debug)
cargo build

# Build for production
cargo build --release

# Run tests to verify installation
cargo test

# Install globally (optional)
cargo install --path .
```

### Basic Usage

```bash
# Show available commands
workflow-toolkit --help

# Create a workflow from definition
workflow-toolkit workflow create examples/hello-world.yaml

# List all workflows
workflow-toolkit workflow list

# Execute a workflow
workflow-toolkit workflow execute hello-world

# Monitor workflow execution
workflow-toolkit workflow status <workflow-id> --watch

# Start interactive TUI
workflow-toolkit tui

# Start MCP server
workflow-toolkit server --http-port 8080 --ws-port 8081

# List available tools
workflow-toolkit tool list

# Execute a tool directly
workflow-toolkit tool execute echo --params '{"message": "Hello World"}'
```

### File Management Tools

The toolkit includes specialized file management capabilities:

```bash
# Interactive file classification
workflow-toolkit tool execute file_classification \
  --params '{"directory": "./data", "rules_file": "classification-rules.json"}'

# Batch file processing
workflow-toolkit tool execute batch_processor \
  --params '{"input_dir": "./input", "output_dir": "./output", "operation": "transform"}'

# Text processing with human review
workflow-toolkit tool execute text_processor \
  --params '{"input_file": "document.txt", "operations": ["extract", "analyze"]}'
```

## Configuration

The toolkit uses a hierarchical configuration system with the following precedence:

1. **Default configuration** (built-in defaults)
2. **Configuration file** (`config/default.toml`)
3. **Environment variables** (prefixed with `WORKFLOW_TOOLKIT_`)
4. **Command line arguments** (highest priority)

### Configuration File Example

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
```

### Environment Variables

```bash
# Server configuration
export WORKFLOW_TOOLKIT_SERVER__HTTP_PORT=8080
export WORKFLOW_TOOLKIT_SERVER__WS_PORT=8081

# Storage configuration
export WORKFLOW_TOOLKIT_STORAGE__DATABASE_PATH="./data/workflow.db"

# Plugin configuration
export WORKFLOW_TOOLKIT_PLUGINS__PLUGIN_DIR="./plugins"

# Logging configuration
export WORKFLOW_TOOLKIT_LOGGING__LEVEL="debug"
```

## Architecture

The toolkit follows a modular, layered architecture designed for scalability and extensibility:

### Core Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    Interface Layer                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │     CLI     │  │     TUI     │  │    MCP Server       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Application Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Workflow   │  │    Tool     │  │   File Management  │  │
│  │   Engine    │  │  Registry   │  │      Tools          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    Plugin Layer                             │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────┐  │
│  │ Native  │ │ Python  │ │ Node.js │ │ Docker  │ │ WASM  │  │
│  │ Plugins │ │ Plugins │ │ Plugins │ │ Plugins │ │Plugins│  │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └───────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Storage   │  │    Cache    │  │    Performance      │  │
│  │  (LanceDB)  │  │   (Moka)    │  │     Monitoring      │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

- **Core Layer**: Type definitions, error handling, configuration management
- **Storage Layer**: LanceDB persistence with moka caching for high performance
- **Workflow Engine**: petgraph-based DAG execution with async support
- **Tool System**: Reusable tool nodes and comprehensive registry
- **Plugin System**: Multi-language plugin support with sandboxing
- **File Management**: Specialized tools for file operations and classification
- **Interface Layer**: CLI, TUI, and MCP server implementations
- **Performance Layer**: Profiling, metrics collection, and optimization

### Technology Stack

- **Language**: Rust 2021 Edition (1.70+)
- **Async Runtime**: Tokio with full async/await support
- **Graph Processing**: petgraph for DAG operations
- **Storage**: LanceDB with Arrow/Parquet support
- **Caching**: moka for high-performance in-memory caching
- **CLI**: clap 4.5 with derive features
- **TUI**: ratatui with crossterm
- **Serialization**: serde with JSON/YAML/TOML support
- **Plugin Runtime**: wasmtime, libloading, bollard (Docker)

## Development

### Prerequisites

- **Rust**: 1.70+ with 2021 Edition support
- **Cargo**: Latest version
- **System Dependencies**: 
  - Python 3.8+ (for Python plugins)
  - Node.js 16+ (for Node.js plugins)
  - Docker (for Docker plugins)

### Building

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Build with all features
cargo build --all-features

# Build specific examples
cargo build --example comprehensive_workflow_example
cargo build --example file_management_example
```

### Testing

The project uses comprehensive testing including unit tests, integration tests, and property-based tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test modules
cargo test storage::tests
cargo test file_management::tests

# Run property-based tests
cargo test property_tests

# Run integration tests
cargo test --test integration_tests

# Run with specific log level
RUST_LOG=debug cargo test
```

### Development Commands

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check

# Generate documentation
cargo doc --open

# Run examples
cargo run --example python_plugin_example
cargo run --example file_management_example

# Run with debug logging
RUST_LOG=debug cargo run -- --help
```

### Project Structure

```
workflow-toolkit/
├── src/                          # Source code
│   ├── core.rs                   # Core types and definitions
│   ├── config.rs                 # Configuration management
│   ├── error.rs                  # Error handling
│   ├── workflow/                 # Workflow engine
│   ├── tools/                    # Tool system
│   ├── plugins/                  # Plugin system
│   │   └── file_management/      # File management tools
│   ├── storage/                  # Storage layer
│   ├── interfaces/               # CLI, TUI, MCP interfaces
│   └── performance/              # Performance monitoring
├── examples/                     # Example workflows and usage
│   ├── templates/                # Workflow templates
│   └── tools/                    # Example tool implementations
├── docs/                         # Documentation
├── tests/                        # Integration tests
└── config/                       # Default configuration
```

## Examples and Use Cases

### Workflow Examples

The `examples/` directory contains comprehensive examples for various use cases:

```bash
# Basic workflow execution
cargo run --example comprehensive_workflow_example

# Plugin integration
cargo run --example plugin_integration_example

# File management operations
cargo run --example file_management_example

# Async execution patterns
cargo run --example async_execution_example

# Performance monitoring
cargo run --example performance_monitoring_example
```

### Common Use Cases

1. **Data Processing Pipelines**: ETL workflows with validation and transformation
2. **File Management**: Bulk file operations with classification and organization
3. **API Integration**: Automated data synchronization and processing
4. **System Monitoring**: Health checks and alerting workflows
5. **Machine Learning**: Model training and deployment pipelines
6. **Document Processing**: Text extraction, analysis, and summarization

### Template Workflows

Pre-built workflow templates are available in `examples/templates/`:

- `interactive-classification-workflow.yaml`: File classification with human review
- `batch-processing-workflow.yaml`: Bulk file processing operations
- `workflow-composition-examples.yaml`: Complex workflow patterns
- `environment-config-examples.yaml`: Configuration management examples

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[User Manual](docs/USER_MANUAL.md)**: Complete user guide with tutorials
- **[API Reference](docs/API_REFERENCE.md)**: Detailed API documentation
- **[Development Guide](docs/DEVELOPMENT_GUIDE.md)**: Developer setup and guidelines
- **[Plugin Development](docs/PLUGIN_DEVELOPMENT.md)**: Creating custom plugins
- **[File Management Guide](docs/FILE_MANAGEMENT_TOOLS_GUIDE.md)**: File management tools
- **[Design Document](DESIGN.md)**: Architecture and design decisions

## Contributing

We welcome contributions! Please follow these steps:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** with appropriate tests
4. **Run the test suite**: `cargo test`
5. **Format your code**: `cargo fmt`
6. **Lint your code**: `cargo clippy`
7. **Commit your changes**: `git commit -m 'Add amazing feature'`
8. **Push to the branch**: `git push origin feature/amazing-feature`
9. **Submit a pull request**

### Development Guidelines

- Follow Rust naming conventions and best practices
- Add tests for new functionality
- Update documentation for API changes
- Use `cargo fmt` and `cargo clippy` before submitting
- Write clear commit messages
- Include examples for new features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support and Community

- **Documentation**: [User Manual](docs/USER_MANUAL.md) and [API Reference](docs/API_REFERENCE.md)
- **Issues**: [GitHub Issues](https://github.com/workflow-toolkit/workflow-toolkit/issues)
- **Discussions**: [GitHub Discussions](https://github.com/workflow-toolkit/workflow-toolkit/discussions)
- **Contributing**: See [Contributing Guidelines](#contributing)

## Acknowledgments

Built with powerful Rust ecosystem libraries including tokio, serde, clap, ratatui, petgraph, and many others. Special thanks to the Rust community for creating such excellent tools.