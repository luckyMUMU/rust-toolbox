# Technology Stack

## Language & Runtime

- **Language**: Rust (Edition 2021)
- **Async Runtime**: Tokio with full features
- **Minimum Rust Version**: 1.70+

## Core Dependencies

### Async & Concurrency
- `tokio` - Async runtime with full feature set
- `async-trait` - Async trait support
- `futures` - Future combinators
- `dashmap` - Concurrent hash maps
- `parking_lot` - High-performance synchronization primitives

### Serialization & Configuration
- `serde` + `serde_json` - JSON serialization
- `serde_yaml` - YAML support
- `toml` - TOML configuration
- `config` - Layered configuration management

### CLI & TUI
- `clap` (v4.5) - CLI framework with derive macros
- `ratatui` (v0.29) - Terminal UI framework
- `crossterm` (v0.28) - Cross-platform terminal manipulation

### Workflow & Scheduling
- `petgraph` - DAG representation and graph algorithms
- `uuid` - Unique identifiers for executions
- `chrono` - Date/time handling

### Plugin System
- `libloading` - Dynamic library loading for native plugins
- `bollard` - Docker API client
- `tar` + `flate2` - Archive handling for Docker contexts

### Storage & Caching
- `moka` - High-performance async cache
- `lancedb` (optional) - Vector database for time-travel queries
- `arrow` + `parquet` (optional) - Columnar data formats

### Error Handling & Validation
- `thiserror` - Error type definitions (for library code)
- `anyhow` - Error handling (for application code)
- `jsonschema` - JSON Schema validation

### Observability
- `tracing` + `tracing-subscriber` - Structured logging
- `sysinfo` - System resource monitoring

### Testing
- `proptest` - Property-based testing
- `tokio-test` - Async test utilities
- `tempfile` - Temporary file/directory creation

### Optional Features
- `mcp` - Model Context Protocol support (rmcp + schemars)
- `lancedb` - Vector database integration
- `macros` - Procedural macros for tool definitions

## Build System

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run with default features
cargo run

# Run with all features
cargo run --all-features

# Run specific example
cargo run --example workflow_example

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run property-based tests
cargo test --test proptest

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

### Feature Flags

Enable optional features during build:

```bash
# Build with MCP support
cargo build --features mcp

# Build with LanceDB support
cargo build --features lancedb

# Build without macros
cargo build --no-default-features
```

## Project Structure Conventions

- Use `mod.rs` for module organization
- Each major module should have a `design.md` file
- Tests can be inline (`#[cfg(test)]`) or in separate `*_tests.rs` files
- Examples go in `examples/` directory
- Integration tests go in `tests/` directory (if present)

## Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use `clippy` for linting
- Prefer `thiserror` for library error types
- Prefer `anyhow` for application error handling
- Use `async-trait` for async trait methods
- Document public APIs with `///` doc comments
- Use `tracing` macros for logging (not `println!`)

## Configuration

Configuration files use TOML format and are located in `config/default.toml`. The system supports layered configuration with environment-specific overrides.

## Environment Variables

- `RUST_LOG` - Controls logging verbosity (e.g., `workflow_toolkit=debug`)
- Standard Rust environment variables apply
