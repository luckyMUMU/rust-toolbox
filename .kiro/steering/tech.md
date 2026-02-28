# Technology Stack

## Language & Runtime

- **Language**: Rust (Edition 2021)
- **Async Runtime**: Tokio 1.42 with full features
- **Minimum Rust Version**: 1.70+ (for async trait support)

## Core Dependencies

### Async & Concurrency
- `tokio` - Async runtime with full feature set
- `tokio-util` - Additional Tokio utilities
- `async-trait` - Async trait support
- `futures` - Future combinators and utilities
- `dashmap` - Concurrent hash map
- `parking_lot` - Efficient synchronization primitives

### Serialization & Configuration
- `serde` + `serde_json` - JSON serialization
- `serde_yaml` - YAML support for workflow definitions
- `toml` - TOML configuration files
- `config` - Layered configuration management

### CLI & TUI
- `clap` v4.5 - Command-line argument parsing with derive macros
- `clap_complete` - Shell completion generation
- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal manipulation

### Error Handling
- `thiserror` - Library error types (use for domain/infrastructure errors)
- `anyhow` - Application error handling (use in application layer and binaries)

### Logging & Observability
- `tracing` - Structured logging and instrumentation
- `tracing-subscriber` - Log collection with env-filter support
- `sysinfo` - System resource monitoring

### Plugin System
- `libloading` - Dynamic library loading for native plugins
- `bollard` - Docker API client for container plugins
- `tar` + `flate2` - Archive handling for plugin packaging

### Workflow & Data
- `petgraph` - Graph data structures for DAG workflows
- `uuid` - Unique identifiers for executions
- `chrono` - Date and time handling
- `regex` - Regular expression support
- `jsonschema` - JSON Schema validation for tool inputs/outputs

### Optional Features
- `rmcp` + `schemars` - Model Context Protocol support (feature: "mcp")
- `lancedb` + `arrow` + `parquet` - Vector database integration (feature: "lancedb")
- `workflow-toolkit-macros` - Procedural macros (feature: "macros", enabled by default)

### Development & Testing
- `proptest` - Property-based testing
- `tokio-test` - Async test utilities
- `tempfile` - Temporary file/directory creation for tests

## Build System

### Build Commands

```bash
# Build the project
cargo build

# Build with release optimizations
cargo build --release

# Build with specific features
cargo build --features "mcp,lancedb"

# Build without default features
cargo build --no-default-features
```

### Testing Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for a specific module
cargo test module_name::

# Run property-based tests (may take longer)
cargo test --release -- --ignored
```

### Running Examples

```bash
# List available examples
cargo run --example

# Run a specific example
cargo run --example python_plugin_example
cargo run --example docker_plugin_example
cargo run --example config_priority_example
```

### Development Commands

```bash
# Check code without building
cargo check

# Run clippy linter
cargo clippy

# Format code
cargo fmt

# Generate documentation
cargo doc --open

# Run the main binary
cargo run

# Run with specific log level
RUST_LOG=debug cargo run
RUST_LOG=workflow_toolkit=trace cargo run
```

### Release Build

```bash
# Build optimized release binary
cargo build --release

# Binary location
./target/release/workflow-toolkit
```

## Project Structure

- **Workspace**: Multi-crate workspace with main crate and `workflow-toolkit-macros`
- **Binary**: `src/main.rs` - CLI/TUI entry point
- **Library**: `src/lib.rs` - Core library exports
- **Examples**: `examples/` - Runnable example code
- **Tests**: Integration tests in `tests/` directory (if present)

## Configuration Files

- `Cargo.toml` - Main project manifest
- `Cargo.lock` - Dependency lock file (committed to repo)
- `config/default.toml` - Default runtime configuration
- `.kiro/` - Kiro-specific settings and steering rules

## Feature Flags

- `default = ["macros"]` - Default features
- `macros` - Enable procedural macros for ToolInput/ToolOutput
- `mcp` - Enable Model Context Protocol server support
- `lancedb` - Enable LanceDB vector database integration

## Environment Variables

- `RUST_LOG` - Control logging verbosity (e.g., `debug`, `info`, `workflow_toolkit=trace`)
- Standard Rust environment variables for build configuration
