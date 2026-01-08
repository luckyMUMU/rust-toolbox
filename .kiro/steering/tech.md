---
inclusion: always
---

# Technology Stack & Development Guidelines

## Language & Build System

**Rust 2021 Edition** (requires Rust 1.70+) with **Cargo** build system. All code must be async-first using tokio runtime.

## Critical Dependencies & Usage Patterns

### Async Programming (MANDATORY)
- **tokio** 1.42: Use `#[tokio::main]` for entry points, `tokio::spawn` for concurrent tasks
- **async-trait** 0.1: Required for async trait methods - use `#[async_trait]` macro
- **futures** 0.3: Use `join!` and `select!` for concurrent operations

### Error Handling (REQUIRED PATTERN)
```rust
// Use thiserror for all error types
#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("Validation failed: {message}")]
    ValidationError { message: String },
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

// Always use Result<T, E> return types
pub async fn my_function() -> Result<String, MyError> { /* ... */ }
```

### Serialization (STANDARD PATTERN)
```rust
// Always derive serde traits for data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyStruct {
    pub field: String,
}

// Use serde_json for JSON, serde_yaml for YAML workflows
```

### Concurrency & State Management
- **dashmap** 6.1: Use for concurrent hash maps instead of `Arc<Mutex<HashMap>>`
- **parking_lot** 0.12: Use `parking_lot::Mutex` instead of `std::sync::Mutex`
- **Arc<dyn Trait>**: Standard pattern for shared trait objects

### Plugin System Architecture
- **libloading** 0.8: Native plugin loading
- **wasmtime** 27.0: WebAssembly plugin execution
- **extism** 1.13: Plugin framework abstraction
- **bollard** 0.17: Docker container execution

## Mandatory Code Patterns

### Configuration Structs
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub timeout_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self { timeout_seconds: 30 }
    }
}
```

### Trait Implementations
```rust
#[async_trait]
pub trait ToolNode: Send + Sync {
    async fn execute(&self, input: Value) -> Result<Value, ToolError>;
}
```

### Module Organization
```rust
// In mod.rs files - always re-export public types
pub use self::engine::WorkflowEngine;
pub use self::definition::{WorkflowDefinition, StepDefinition};

// In implementation files
use crate::core::{ExecutionContext, Result};
use crate::error::WorkflowError;
```

## Development Commands

### Essential Workflow
```bash
# Always run before committing
cargo fmt && cargo clippy && cargo test

# Build with all features for CI
cargo build --all-features

# Run specific example
cargo run --example comprehensive_workflow_example
```

### Testing Requirements
- Unit tests: `#[cfg(test)]` modules in same file
- Property tests: Use `proptest` for complex validation
- Integration tests: Separate `tests.rs` files within modules
- All async functions must use `#[tokio::test]`

## Configuration Hierarchy (STRICT ORDER)
1. Command line arguments (highest priority)
2. Environment variables (`WORKFLOW_TOOLKIT_*`)
3. Config files (`config/default.toml`)
4. Default values (lowest priority)

## Feature Flags
- **default**: No optional features (keep minimal)
- **lancedb**: Vector database with arrow/parquet support

## Critical Rules for AI Assistant

1. **Always use async/await** - Never write blocking I/O code
2. **Use thiserror for errors** - Never use `anyhow` for library code
3. **Implement Default trait** - Required for all config structs
4. **Use Arc<dyn Trait>** - Standard pattern for shared trait objects
5. **Derive serde traits** - Required for all data structures
6. **Use Result<T, E>** - Never panic in library code
7. **Follow module re-export pattern** - Export public types in mod.rs
8. **Use structured logging** - `tracing::info!`, `tracing::error!` etc.