# Technology Stack

## Build System & Language

- **Language**: Rust 2021 Edition (requires Rust 1.70+)
- **Build System**: Cargo
- **Package Manager**: Cargo with crates.io dependencies

## Core Dependencies

### Async Runtime
- **tokio** 1.42: Full-featured async runtime
- **tokio-util** 0.7: Additional tokio utilities
- **async-trait** 0.1: Async trait support
- **futures** 0.3: Future combinators

### CLI & TUI
- **clap** 4.5: Command-line argument parsing with derive features
- **clap_complete** 4.5: Shell completion generation
- **ratatui** 0.29: Terminal user interface framework
- **crossterm** 0.28: Cross-platform terminal manipulation

### Serialization & Configuration
- **serde** 1.0: Serialization framework with derive features
- **serde_json** 1.0: JSON support
- **serde_yaml** 0.9: YAML support
- **config** 0.14: Hierarchical configuration management
- **toml** 0.8: TOML configuration format

### Storage & Caching
- **lancedb** 0.20: Vector database (optional feature)
- **arrow** 54.0: Columnar data format (optional)
- **parquet** 54.0: Parquet file format (optional)
- **moka** 0.12: High-performance caching with async support

### Workflow & Graph Processing
- **petgraph** 0.6: Graph data structures and algorithms
- **uuid** 1.11: UUID generation with v4 and serde features
- **chrono** 0.4: Date and time handling with serde

### Plugin System
- **libloading** 0.8: Dynamic library loading for native plugins
- **wasmtime** 27.0: WebAssembly runtime
- **extism** 1.13: Plugin framework
- **bollard** 0.17: Docker API client

### Error Handling & Logging
- **anyhow** 1.0: Flexible error handling
- **thiserror** 2.0: Derive macros for error types
- **tracing** 0.1: Structured logging and instrumentation
- **tracing-subscriber** 0.3: Tracing subscriber implementations

### Concurrency & Synchronization
- **dashmap** 6.1: Concurrent hash map
- **parking_lot** 0.12: High-performance synchronization primitives

### Validation & Schema
- **jsonschema** 0.18: JSON Schema validation

## Development Dependencies

- **proptest** 1.6: Property-based testing
- **tokio-test** 0.4: Async testing utilities
- **tempfile** 3.14: Temporary file and directory creation

## Common Commands

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build with all features
cargo build --all-features

# Build specific example
cargo build --example tools_example
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run property-based tests
cargo test property_tests

# Run specific test module
cargo test storage::tests
```

### Running
```bash
# Run CLI with debug logging
RUST_LOG=debug cargo run -- --help

# Run specific example
cargo run --example python_plugin_example

# Run with release optimizations
cargo run --release -- workflow execute hello-world
```

### Development Tools
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check

# Generate documentation
cargo doc --open
```

## Configuration Management

The project uses a hierarchical configuration system with the following precedence:
1. Default configuration
2. Configuration file (`config/default.toml`)
3. Environment variables (prefixed with `WORKFLOW_TOOLKIT_`)
4. Command line arguments

## Feature Flags

- **default**: No optional features enabled by default
- **lancedb**: Enables LanceDB storage backend with arrow and parquet support

## Code Organization Patterns

- Use `pub mod` declarations in `mod.rs` files
- Re-export commonly used types in `lib.rs`
- Implement `Default` trait for configuration structs
- Use `thiserror` for structured error types
- Prefer `async-trait` for async trait methods
- Use `Arc<dyn Trait>` for shared trait objects