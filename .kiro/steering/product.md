---
inclusion: always
---

# Workflow Toolkit Product Guide

## System Overview

A multi-interface workflow execution system built with Rust that provides flexible workflow management through CLI, TUI, and MCP server interfaces. The system emphasizes modularity, extensibility, and robust error handling.

## Core Architecture Principles

### Modular Design
- **Separation of Concerns**: Each module has a single responsibility (workflow, storage, plugins, interfaces)
- **Trait-Based Abstractions**: Use traits for extensibility (e.g., `ToolNode`, `StorageBackend`, `WorkflowEngine`)
- **Dependency Injection**: Components are loosely coupled through trait objects (`Arc<dyn Trait>`)

### Error Handling Strategy
- Use `thiserror` for structured error types with context
- Implement `From` conversions between error types for seamless propagation
- Prefer `Result<T, E>` over panics for recoverable errors
- Include detailed error context for debugging and user feedback

### Async-First Design
- All I/O operations are async using tokio runtime
- Use `async-trait` for async trait methods
- Prefer `Arc<Mutex<T>>` or `dashmap` for shared state in async contexts

## Key Components

### Workflow Engine (`src/workflow/`)
- **DAG Execution**: Uses `petgraph` for dependency resolution and parallel execution
- **State Persistence**: Checkpoints and recovery through storage layer
- **Validation**: Pre-execution validation of workflow definitions
- **Retry Logic**: Configurable retry strategies with exponential backoff

### Plugin System (`src/plugins/`)
- **Multi-Language Support**: Native (Rust), Python, Node.js, Docker, WebAssembly
- **Sandboxing**: Isolated execution environments for security
- **Dynamic Loading**: Runtime plugin discovery and loading
- **Tool Registration**: Plugins register tools in the global registry

### File Management Tools (`src/plugins/file_management/`)
- **Interactive Processing**: Human-in-the-loop decision making
- **Batch Operations**: Efficient bulk file processing with progress tracking
- **Classification Rules**: JSON-based rule configuration for automated decisions
- **Error Recovery**: Robust error handling with rollback capabilities

### Storage Layer (`src/storage/`)
- **Backend Abstraction**: Pluggable storage backends (LanceDB, in-memory)
- **Caching Strategy**: Multi-level caching with moka for performance
- **State Management**: Unified state persistence across components
- **Backup/Recovery**: Automated backup and point-in-time recovery

## Development Guidelines

### Code Organization
- Follow the established module structure in `src/`
- Use `mod.rs` files for module exports and public interfaces
- Include `DESIGN.md` files for complex modules
- Place examples in `examples/` with descriptive names

### Configuration Management
- Use hierarchical configuration with precedence: CLI args > env vars > config files > defaults
- Define configuration structs with `serde` derive macros
- Implement `Default` trait for all configuration types
- Use `config` crate for loading and merging configuration sources

### Testing Strategy
- Unit tests: In-module with `#[cfg(test)]`
- Integration tests: Separate files within modules
- Property tests: Use `proptest` for complex logic validation
- Example tests: Ensure all examples compile and run successfully

### Plugin Development
- Implement the `ToolNode` trait for new tools
- Use structured input/output with `serde` serialization
- Include comprehensive error handling and validation
- Provide clear documentation and usage examples

## Interface Patterns

### CLI Interface (`src/interfaces/cli/`)
- Use `clap` with derive macros for argument parsing
- Implement structured output formatting (JSON, YAML, table)
- Provide shell completion generation
- Include comprehensive help text and examples

### MCP Server (`src/interfaces/mcp.rs`)
- Implement Model Context Protocol for AI assistant integration
- Expose workflow and tool capabilities as MCP tools
- Handle async request/response patterns
- Provide detailed tool schemas and documentation

### TUI Interface (`src/interfaces/tui.rs`)
- Use `ratatui` for terminal-based user interfaces
- Implement real-time status updates and progress tracking
- Provide keyboard shortcuts and navigation
- Handle terminal resize and cleanup gracefully

## Performance Considerations

- Use `Arc` for shared immutable data, `Arc<Mutex<T>>` for shared mutable data
- Implement connection pooling for external resources
- Use streaming for large data processing
- Profile memory usage and implement appropriate caching strategies
- Leverage Rust's zero-cost abstractions and compile-time optimizations

## Security Guidelines

- Validate all external inputs (workflow definitions, plugin parameters)
- Use sandboxing for plugin execution environments
- Implement proper authentication and authorization for MCP server
- Sanitize file paths and prevent directory traversal attacks
- Use secure defaults for all configuration options