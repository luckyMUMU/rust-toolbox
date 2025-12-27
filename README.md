# Workflow Toolkit

A multi-interface workflow execution system built with Rust, supporting CLI, TUI, and MCP server interfaces.

## Features

- **Multi-Interface Support**: CLI, Terminal UI (TUI), and Model Context Protocol (MCP) server
- **Flexible Workflow Engine**: DAG-based workflow execution with support for conditions, loops, and parallel execution
- **Plugin System**: Extensible plugin architecture supporting Native, Python, Node.js, Docker, and WebAssembly plugins
- **Tool Registry**: Reusable tool nodes that can be used in workflows or called independently
- **State Management**: Persistent workflow state with checkpoint and recovery capabilities
- **Configuration Management**: Hierarchical configuration with environment variable and CLI parameter overrides

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/workflow-toolkit/workflow-toolkit.git
cd workflow-toolkit

# Build the project
cargo build --release

# Run the CLI
./target/release/workflow-toolkit --help
```

### Basic Usage

```bash
# List available commands
workflow-toolkit --help

# Create a workflow
workflow-toolkit workflow create examples/hello-world.yaml

# Execute a workflow
workflow-toolkit workflow execute hello-world

# Check workflow status
workflow-toolkit workflow status <workflow-id>

# Start TUI interface
workflow-toolkit tui

# Start MCP server
workflow-toolkit server --http-port 8080 --ws-port 8081
```

## Configuration

The toolkit uses a hierarchical configuration system:

1. Default configuration
2. Configuration file (`config/default.toml`)
3. Environment variables (prefixed with `WORKFLOW_TOOLKIT_`)
4. Command line arguments

Example configuration file:

```toml
[server]
http_port = 8080
ws_port = 8081

[storage]
database_path = "./data/workflow.db"
cache_size = 104857600  # 100MB

[logging]
level = "info"
format = "Pretty"

[plugins]
plugin_dir = "./plugins"
auto_load = true
sandbox_enabled = true
```

## Architecture

The toolkit is built with a modular architecture:

- **Core Layer**: Type definitions, error handling, and configuration
- **Storage Layer**: Persistent storage and caching with LanceDB and Moka
- **Workflow Engine**: DAG-based execution engine with petgraph
- **Tool System**: Reusable tool nodes and registry
- **Plugin System**: Multi-language plugin support
- **Interface Layer**: CLI, TUI, and MCP server implementations

## Development

### Prerequisites

- Rust 1.70+ 
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- --help
```

### Testing

The project uses both unit tests and property-based tests:

```bash
# Run all tests
cargo test

# Run property tests specifically
cargo test property_tests

# Run with output
cargo test -- --nocapture
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Documentation

- [Design Document](DESIGN.md) - Detailed architecture and design decisions
- [API Documentation](docs/api.md) - API reference (coming soon)
- [Plugin Development Guide](docs/plugins.md) - How to create plugins (coming soon)
- [User Manual](docs/user-guide.md) - Comprehensive user guide (coming soon)