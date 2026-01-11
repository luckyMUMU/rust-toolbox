<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Workflow Toolkit - Agent Instructions

**Project:** Multi-interface workflow execution system (Rust 2021 Edition)  
**Architecture:** CLI, TUI, and MCP server with DAG-based workflow engine  
**Generated:** 2026-01-11

## Quick Reference

### Build & Test Commands
```bash
# Build
cargo build                    # Debug build
cargo build --release          # Optimized build  
cargo build --all-features     # With LanceDB support
cargo check                    # Quick type check

# Test
cargo test                     # All tests
cargo test -- --nocapture      # With output
cargo test workflow::tests     # Specific module
cargo test property_tests      # Property-based tests
cargo test --test integration_tests  # Integration only
cargo test tui_unit_tests --test tui_standalone_unit_tests  # Single test file

# Run
cargo run -- --help            # CLI help
cargo run -- workflow execute examples/hello-world.yaml
cargo run -- tui               # Start TUI
cargo run -- tool list         # List tools

# Format & Lint
cargo fmt
cargo clippy -- -D warnings
cargo clippy --fix

# Examples
cargo run --example comprehensive_workflow_example
cargo run --example python_plugin_example
cargo run --example file_management_example
```

### Single Test Execution
```bash
# Run one specific test
cargo test test_name -- --nocapture

# Run test in specific file
cargo test --test tui_standalone_unit_tests -- test_name

# Run with filter
cargo test workflow -- --test-threads=1
```

## Code Style Guidelines

### Import Order (STRICT)
```rust
// 1. Standard library
use std::sync::Arc;
use std::time::Duration;

// 2. External crates (alphabetical)
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};

// 3. Internal modules
use crate::core::{ExecutionContext, WorkflowId};
use crate::error::{Result, WorkflowError};
use crate::tools::ToolNode;
```

### Error Handling
```rust
// ✅ CORRECT: Use thiserror with constructors
pub fn do_something() -> Result<()> {
    let value = operation().map_err(|e| {
        WorkflowError::workflow_execution(&format!("Failed: {}", e))
    })?;
    Ok(())
}

// ✅ CORRECT: Constructor methods for common errors
return Err(WorkflowError::tool("Invalid parameters"));
return Err(WorkflowError::plugin("Loading failed"));
return Err(WorkflowError::storage("Connection lost"));

// ❌ NEVER: Type suppression
let x: u32 = value as any;           // Forbidden
#[ts-ignore]                         // Forbidden
let x = value as u32;                // Use try_into() instead

// ❌ NEVER: Empty error handling
catch(e) {}                          // Forbidden - always handle errors
```

### Async Patterns
```rust
// ✅ CORRECT: Async traits
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute(&self, def: WorkflowDefinition) -> Result<WorkflowExecution>;
}

// ✅ CORRECT: Concurrent collections
use dashmap::DashMap;  // For read-heavy concurrent access
use tokio::sync::RwLock; // For mutable shared state

// ✅ CORRECT: Semaphore for concurrency control
let semaphore = Arc::new(Semaphore::new(4));
let permit = semaphore.acquire().await?;
```

### Type Safety
```rust
// ✅ CORRECT: Newtype pattern
pub struct WorkflowId(Uuid);

// ✅ CORRECT: Strong types over primitives
use std::path::PathBuf;
use std::time::Duration;
use chrono::{DateTime, Utc};

// ✅ CORRECT: TryInto for conversions
let count: usize = value.try_into()?;

// ✅ CORRECT: Enums with helper methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending, Running, Paused, Completed, Failed, Cancelled, Timeout,
}

impl ExecutionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled | Self::Timeout)
    }
}
```

### Testing
```rust
// ✅ CORRECT: Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() {
        // Test logic
    }
    
    #[tokio::test]
    async fn test_async_something() {
        // Async test logic
    }
}

// ✅ CORRECT: Use tempfile for isolation
#[tokio::test]
async fn test_with_temp_dir() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    // Use temp_dir.path()
}

// ✅ CORRECT: Property-based tests
#[test]
fn test_property() {
    proptest!(|(input: Vec<String>)| {
        // Test with random input
    });
}
```

### Logging
```rust
// ✅ CORRECT: Use tracing macros
use tracing::{info, debug, warn, error};

info!("Workflow started: {}", workflow_id);
debug!("Parameters: {:?}", params);
warn!("Resource threshold approaching");
error!("Execution failed: {}", error);

// ❌ NEVER: println! or eprintln!
println!("Debug info");  // Forbidden - use tracing
```

### Configuration Priority
1. Command line arguments (highest)
2. Environment variables (`WORKFLOW_TOOLKIT_*`)
3. Config file (`config/default.toml`)
4. Built-in defaults (lowest)

```rust
// Environment variable format
export WORKFLOW_TOOLKIT_SERVER__HTTP_PORT=8080
export WORKFLOW_TOOLKIT_LOGGING__LEVEL=debug
```

## Project Structure

```
rust-tool-v2/
├── src/                          # Core library
│   ├── lib.rs                    # Module declarations, re-exports
│   ├── core.rs                   # Shared types, ExecutionContext, enums
│   ├── error.rs                  # WorkflowError with 20+ variants
│   ├── config.rs                 # Hierarchical config with hot reload
│   │
│   ├── workflow/                 # DAG engine (13 files)
│   │   ├── engine.rs             # DefaultWorkflowEngine with semaphore
│   │   ├── scheduler.rs          # DagScheduler (petgraph-based)
│   │   ├── validator.rs          # Workflow validation
│   │   ├── execution_manager.rs  # Parallel execution control
│   │   ├── audit.rs              # Audit logging
│   │   └── result_cache.rs       # TTL-based caching
│   │
│   ├── tools/                    # Tool system (7 files)
│   │   ├── registry.rs           # BasicToolRegistry (DashMap-based)
│   │   ├── node.rs               # ToolNode trait, BasicTool
│   │   ├── templates.rs          # Parameter templates
│   │   ├── versioning.rs         # Dependency resolution
│   │   └── dependency.rs         # Version matching
│   │
│   ├── plugins/                  # Plugin system (11 files)
│   │   ├── manager.rs            # PluginManager, RuntimeManager
│   │   ├── native.rs             # Native Rust plugins
│   │   ├── python.rs             # Python integration
│   │   ├── nodejs.rs             # Node.js integration
│   │   ├── docker.rs             # Docker container execution
│   │   └── file_management/      # Specialized file ops (19 files)
│   │       ├── classifier.rs     # AI-powered classification
│   │       ├── batch_processor.rs # Bulk operations
│   │       └── text_processor.rs # Text analysis
│   │
│   ├── interfaces/               # User interfaces
│   │   ├── cli/                  # Clap-based CLI (6 files)
│   │   │   ├── app.rs            # Main CLI application
│   │   │   ├── commands.rs       # Command definitions
│   │   │   └── output.rs         # Formatted output
│   │   └── tui/                  # Ratatui-based TUI (20+ files)
│   │       ├── app.rs            # TUI application
│   │       ├── widgets/          # Widget system
│   │       ├── event.rs          # Event handling
│   │       ├── layout.rs         # Layout management
│   │       ├── theme.rs          # Theme system
│   │       ├── monitoring.rs     # System monitoring
│   │       ├── memory.rs         # Memory management
│   │       ├── performance.rs    # Performance optimization
│   │       └── config.rs         # TUI configuration
│   │
│   ├── storage/                  # Persistence layer (6 files)
│   │   ├── state_manager.rs      # StateManager
│   │   ├── backends.rs           # FileStorage, LanceDB
│   │   ├── cache.rs              # SimpleMemoryCache
│   │   └── backup.rs             # Backup/restore
│   │
│   └── performance/              # Optimization (6 files)
│       ├── cache.rs              # Moka-based caching
│       ├── metrics.rs            # Metrics collection
│       ├── profiler.rs           # Performance profiling
│       └── concurrency.rs        # Concurrency control
│
├── examples/                     # 23 comprehensive examples
│   ├── comprehensive_workflow_example.rs
│   ├── python_plugin_example.rs
│   ├── file_management_example.rs
│   └── templates/                # Workflow templates
│
├── tests/                        # Test suite
│   ├── integration_tests.rs      # End-to-end workflows
│   ├── tui_standalone_unit_tests.rs  # TUI unit tests
│   ├── tui_integration_tests.rs  # TUI integration
│   ├── tui_performance_benchmark_tests.rs
│   ├── file_management_integration_tests.rs
│   └── template_property_tests.rs
│
├── config/                       # Default configuration
│   └── default.toml
│
├── docs/                         # Documentation
│   └── AGENTS.md                 # Per-directory instructions
│
├── openspec/                     # Spec-driven development
│   ├── AGENTS.md                 # OpenSpec instructions
│   ├── specs/                    # Current capabilities
│   └── changes/                  # Proposed changes
│
└── Cargo.toml                    # 87 dependencies, 12 features
```

## Key Components & Locations

| Component | Location | Key Types |
|-----------|----------|-----------|
| **Workflow Engine** | `src/workflow/engine.rs` | `WorkflowEngine` trait, `DefaultWorkflowEngine` |
| **DAG Scheduler** | `src/workflow/scheduler.rs` | `DagScheduler` (petgraph-based) |
| **Tool Registry** | `src/tools/registry.rs` | `ToolRegistry` trait, `BasicToolRegistry` |
| **Plugin Manager** | `src/plugins/manager.rs` | `PluginManager`, `RuntimeManager` |
| **Config Manager** | `src/config.rs` | `ConfigManager` with hot reload |
| **Error Types** | `src/error.rs` | `WorkflowError` with 20+ variants |
| **Core Types** | `src/core.rs` | `ExecutionContext`, `ExecutionStatus`, `RetryPolicy` |
| **Storage** | `src/storage/` | `StateManager`, `FileStorage`, `SimpleMemoryCache` |
| **CLI** | `src/interfaces/cli/` | `CliApp`, `Cli`, commands |
| **TUI** | `src/interfaces/tui/` | `TuiApp`, widgets, event loop |

## Dependency Highlights

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.42+ | Async runtime (full features) |
| `petgraph` | 0.6+ | DAG operations |
| `clap` | 4.5+ | CLI parsing with derive |
| `ratatui` | 0.29+ | TUI framework |
| `serde` | 1.0+ | Serialization |
| `thiserror` | 2.0+ | Error handling |
| `dashmap` | 6.1+ | Concurrent HashMap |
| `moka` | 0.12+ | High-performance caching |
| `async-trait` | 0.1+ | Async trait support |
| `uuid` | 1.11+ | UUID generation |
| `chrono` | 0.4+ | Date/time handling |
| `lancedb` | 0.20+ | Vector DB (optional) |

## Anti-Patterns (FORBIDDEN)

| Pattern | Why Forbidden | Alternative |
|---------|---------------|-------------|
| `as any` | Loses type safety | Use `try_into()`, `into()` |
| `@ts-ignore` | Hides errors | Fix error or use `#[allow(...)]` |
| `empty catch {}` | Swallows errors | Use `?` or handle explicitly |
| `println!` | No structured logging | Use `tracing::info!` |
| `std::sync::Mutex` in async | Blocks executor | Use `tokio::sync::RwLock` or `DashMap` |
| `unwrap()` in production | Panics | Use `?` with proper error handling |
| `clone()` on Arc unnecessarily | Performance cost | Use `Arc::clone(&value)` |
| Manual error types | Inconsistent | Use `thiserror::Error` |
| Type suppression | Hides bugs | Fix types properly |

## Common Patterns

### Builder Pattern
```rust
let tool = BasicTool::builder()
    .name("echo")
    .version("1.0.0")
    .description("Echo tool")
    .executor(executor)
    .build()?;
```

### Async Function Executor
```rust
let executor = Arc::new(AsyncFunctionExecutor::new(|params, context| async move {
    // Async logic here
    Ok(result)
}));
```

### DashMap Usage
```rust
// Insert
tools.insert("echo".to_string(), Arc::new(echo_tool));

// Get
if let Some(tool) = tools.get("echo") {
    tool.execute(params, context).await?;
}
```

### Error Context
```rust
pub fn load_config(path: &Path) -> Result<Config> {
    Config::load_from_path(path)
        .map_err(|e| WorkflowError::Config(e).into())
}
```

## Gotchas & Important Notes

1. **LanceDB Feature**: Optional, requires `--features lancedb` or `--all-features`
2. **Temp Directory**: Creates `std::env::temp_dir()/workflow-toolkit/` at runtime
3. **Config Hot Reload**: Spawns background task, errors printed to stderr
4. **Parallel Limit**: Default 4 parallel workflows, configurable via `Config`
5. **Checkpoint Interval**: 5 minutes by default, stores in storage backend
6. **Async Tests**: Must use `#[tokio::test]`, not `#[test]`
7. **Tool Registry**: Thread-safe via `Arc<dyn ToolRegistry>`, not `RwLock`
8. **Plugin Sandboxing**: Configurable via `plugins.sandbox_enabled` in config
9. **MCP Server**: ⚠️ **DISABLED** - Dependencies commented out in Cargo.toml. Code exists in `src/interfaces/mcp.rs` but requires `mcp-protocol-server`, `jsonrpc-*` crates. Documentation references it but won't compile without dependencies.
10. **WASM Support**: ⚠️ **DISABLED** - `wasmtime` and `extism` commented out. Use Docker/Python/Node.js plugins instead.
11. **Build Issues**: On Windows, large dependencies may cause memory allocation failures. Use `cargo check` for verification.

## Current Build Status

✅ **Compiles**: `cargo check` passes with 0 errors  
⚠️ **Full Build**: May fail on Windows due to memory issues  
❌ **MCP Server**: Disabled (dependencies commented out)  
❌ **WASM Plugins**: Disabled (dependencies commented out)  

## Performance Notes

- **Caching**: Uses `moka::future::Cache` with TTL support
- **Concurrency**: `dashmap::DashMap` for read-heavy workloads
- **Memory**: `parking_lot::RwLock` for mutable shared state
- **Metrics**: Optional `metrics` crate integration (commented out)
- **Profiling**: Built-in profiler in `src/performance/profiler.rs`

## Testing Strategy

### Test Types
- **Unit**: In-module `#[cfg(test)]` (same file)
- **Integration**: `tests/` directory
- **Property**: Randomized input testing (`proptest`)
- **E2E**: Full system workflows

### Test Execution
```bash
# All tests
cargo test

# Specific test file
cargo test --test tui_standalone_unit_tests

# Single test
cargo test test_name -- --nocapture

# With environment
RUST_LOG=debug cargo test
```

### Test Patterns
```rust
#[tokio::test]
async fn test_workflow_execution() {
    let fixture = IntegrationTestFixture::new().await;
    let result = fixture.workflow_engine.execute_workflow(workflow).await;
    assert!(result.is_ok());
}
```

## Development Workflow

### Before Starting Any Task
1. Read `openspec/AGENTS.md` for spec-driven development
2. Check existing specs in `openspec/specs/`
3. Run `cargo check` to verify compilation
4. Review related module AGENTS.md files

### Implementation Steps
1. **Plan**: Create TODO list for multi-step tasks
2. **Implement**: Follow code style guidelines strictly
3. **Test**: Add tests for all new code
4. **Verify**: Run `cargo test`, `cargo fmt`, `cargo clippy -- -D warnings`
5. **Check**: Run `lsp_diagnostics` on changed files

### Before Committing
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
```

## Environment Setup

### Required Tools
- Rust 1.70+ (2021 Edition)
- Cargo (latest)
- Python 3.8+ (for Python plugins)
- Node.js 16+ (for Node.js plugins)
- Docker (for Docker plugins)

### Environment Variables
```bash
# Logging
export RUST_LOG=info
export WORKFLOW_TOOLKIT_LOGGING__LEVEL=info

# Server
export WORKFLOW_TOOLKIT_SERVER__HTTP_PORT=8080
export WORKFLOW_TOOLKIT_SERVER__WS_PORT=8081

# Storage
export WORKFLOW_TOOLKIT_STORAGE__DATABASE_PATH="./data/workflow.db"

# TUI
export WORKFLOW_TOOLKIT_TUI__THEME="dark"
export WORKFLOW_TOOLKIT_TUI__REFRESH_RATE=60
```

## When to Consult Oracle

Consult Oracle for:
- Complex architecture decisions
- After 2+ failed fix attempts
- Unfamiliar code patterns
- Security/performance concerns
- Multi-system tradeoffs

Don't consult for:
- Simple file operations
- First attempt at fixes
- Questions answerable from code
- Trivial decisions

## Summary for Agents

**Always:**
- Use strict type safety (no `as any`, no `@ts-ignore`)
- Follow import order (std → external → internal)
- Use `tracing` macros, never `println!`
- Use `Result<T, WorkflowError>` for fallible operations
- Use `#[tokio::test]` for async tests
- Use `tempfile::TempDir` for test isolation
- Create TODO lists for multi-step tasks
- Run `cargo fmt && cargo clippy -- -D warnings && cargo test` before completion

**Never:**
- Suppress types or errors
- Use blocking mutex in async code
- Leave errors unhandled
- Commit without testing
- Skip TODO tracking for complex tasks

**Key Files to Remember:**
- `src/core.rs` - Shared types
- `src/error.rs` - Error handling
- `src/config.rs` - Configuration
- `src/workflow/engine.rs` - Workflow execution
- `src/tools/registry.rs` - Tool management
- `src/interfaces/cli/app.rs` - CLI interface
- `src/interfaces/tui/app.rs` - TUI interface
- `openspec/AGENTS.md` - Spec-driven development
