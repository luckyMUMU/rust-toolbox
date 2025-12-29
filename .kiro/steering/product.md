# Product Overview

## Workflow Toolkit

A multi-interface workflow execution system built with Rust that provides flexible workflow management through CLI, TUI, and MCP server interfaces.

### Core Features

- **Multi-Interface Support**: CLI commands, Terminal UI (TUI), and Model Context Protocol (MCP) server
- **DAG-based Workflow Engine**: Directed Acyclic Graph execution with conditions, loops, and parallel processing
- **Extensible Plugin System**: Support for Native (Rust), Python, Node.js, Docker, and WebAssembly plugins
- **Tool Registry**: Reusable tool nodes for workflows and standalone execution
- **State Management**: Persistent workflow state with checkpoint and recovery capabilities
- **Hierarchical Configuration**: Environment variables, config files, and CLI parameter overrides

### Architecture

The system follows a modular architecture with clear separation of concerns:
- **Core Layer**: Type definitions, error handling, configuration management
- **Storage Layer**: LanceDB persistence with moka caching
- **Workflow Engine**: petgraph-based DAG execution
- **Tool System**: Reusable tool nodes and registry
- **Plugin System**: Multi-language plugin support with sandboxing
- **Interface Layer**: CLI, TUI, and MCP server implementations

### Target Use Cases

- Automated workflow orchestration and execution
- Tool integration and management
- Development pipeline automation
- Data processing workflows
- Multi-step task automation with dependencies