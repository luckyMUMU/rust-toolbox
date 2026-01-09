# WORKFLOW TOOLKIT - PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-10  
**Language:** Rust (2021 Edition)  
**Architecture:** Multi-interface workflow execution system

## OVERVIEW
Workflow Toolkit is a multi-interface workflow engine with CLI, TUI, and MCP server support. Built with tokio async runtime, petgraph DAG scheduler, and extensible plugin system.

## STRUCTURE
```
rust-tool-v2/
├── src/                    # Core library (11 modules)
│   ├── workflow/          # DAG engine, scheduler, execution (13 files)
│   ├── plugins/           # Native, Python, Node.js, Docker, WASM (11 files)
│   │   └── file_management/# Specialized file ops (19 files)
│   ├── tools/             # Tool registry, templates, versioning (7 files)
│   ├── interfaces/        # CLI, TUI, MCP server (4 files)
│   │   ├── cli/           # Clap-based CLI (6 files)
│   │   └── tui/           # Ratatui-based TUI (6 files)
│   ├── storage/           # Backends, state, backup (6 files)
│   ├── performance/       # Cache, metrics, profiling (6 files)
│   ├── config.rs          # Hierarchical config (env > CLI > file > defaults)
│   ├── core.rs            # Shared types, ExecutionContext, RetryPolicy
│   └── error.rs           # WorkflowError with 20+ variants
├── examples/              # 23 comprehensive examples
├── tests/                 # Integration + property-based tests (6 files)
├── docs/                  # Documentation (11 files)
├── config/                # Default configuration
└── Cargo.toml             # 87 dependencies, 12 features
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| **Workflow execution** | `src/workflow/engine.rs` | DefaultWorkflowEngine with parallel semaphore |
| **DAG scheduling** | `src/workflow/scheduler.rs` | petgraph-based topological sort |
| **Tool registration** | `src/tools/registry.rs` | DashMap for concurrent access |
| **Plugin loading** | `src/plugins/manager.rs` | RuntimeManager + PluginManager |
| **CLI commands** | `src/interfaces/cli/commands.rs` | Clap subcommands |
| **TUI event loop** | `src/interfaces/tui/event.rs` | Crossterm + ratatui |
| **State persistence** | `src/storage/state_manager.rs` | FileStorage + SimpleMemoryCache |
| **Performance tuning** | `src/performance/` | Moka cache, concurrency control |
| **Configuration** | `src/config.rs` | Priority system, hot reload |
| **Error handling** | `src/error.rs` | thiserror with 20+ error types |

## CODE MAP (Key Symbols)
| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `WorkflowEngine` | Trait | `src/workflow/engine.rs` | Core execution trait |
| `DefaultWorkflowEngine` | Struct | `src/workflow/engine.rs` | Main implementation |
| `ToolRegistry` | Trait | `src/tools/registry.rs` | Tool management |
| `BasicToolRegistry` | Struct | `src/tools/registry.rs` | DashMap-based registry |
| `PluginManager` | Struct | `src/plugins/manager.rs` | Plugin lifecycle |
| `ExecutionContext` | Struct | `src/core.rs` | Runtime context |
| `WorkflowError` | Enum | `src/error.rs` | Error types |
| `ConfigManager` | Struct | `src/config.rs` | Config with hot reload |
| `AsyncFunctionExecutor` | Struct | `src/tools/node.rs` | Async tool executor |

## CONVENTIONS (DEVIATIONS FROM STANDARD)

### Import Order
```rust
// Standard library → External (alphabetical) → Internal modules
use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use crate::core::{ExecutionContext, WorkflowId};
```

### Error Handling
- **NEVER** use `as any`, `@ts-ignore`, or type suppression
- **ALWAYS** use `Result<T, WorkflowError>` for fallible operations
- **CONSTRUCTORS** for common errors: `WorkflowError::workflow_execution()`, `WorkflowError::tool()`
- **TIMEOUTS** use `tokio::time::timeout` with proper error mapping

### Async Patterns
- **TRAITS** use `#[async_trait]` from `async_trait` crate
- **CONCURRENCY** use `DashMap` for read-heavy, `RwLock` for mutable shared state
- **SEMAPHORES** control parallel execution: `Arc<Semaphore>`
- **CHANNELS** use `tokio::sync::mpsc` for async communication

### Type Safety
- **NEWTYPE** pattern for domain types: `WorkflowId(Uuid)`
- **ENUMS** for state: `ExecutionStatus` with `is_terminal()`, `can_pause()`, `can_resume()`
- **STRONG TYPES** over primitives: `Duration`, `PathBuf`, `DateTime<Utc>`
- **TRY_INTO** for conversions: `let x: u32 = value.try_into()?`

### Testing
- **UNIT** tests in `#[cfg(test)]` modules within same file
- **ASYNC** tests use `#[tokio::test]`
- **INTEGRATION** tests in `tests/` directory
- **PROPERTY** tests use `proptest` crate
- **FIXTURES** use `tempfile::TempDir` for isolation

### Configuration Priority
1. Command line arguments (highest)
2. Environment variables (`WORKFLOW_TOOLKIT_*`)
3. Config file (`config/default.toml`)
4. Built-in defaults (lowest)

### Logging
- **NEVER** use `println!` or `eprintln!`
- **ALWAYS** use `tracing` macros: `info!`, `debug!`, `warn!`, `error!`
- **ENV FILTER** via `RUST_LOG` environment variable

## ANTI-PATTERNS (FORBIDDEN IN THIS PROJECT)

| Pattern | Why Forbidden | Alternative |
|---------|---------------|-------------|
| `as any` | Loses type safety | Use `try_into()`, `into()`, proper conversions |
| `@ts-ignore` | Hides errors | Fix the error or use `#[allow(clippy::...)]` with justification |
| `empty catch {}` | Swallows errors | Use `?` or handle explicitly |
| `println!` | No structured logging | Use `tracing::info!` |
| `std::sync::Mutex` in async | Blocks executor | Use `tokio::sync::RwLock` or `DashMap` |
| `unwrap()` in production | Panics | Use `?` with proper error handling |
| `clone()` on Arc unnecessarily | Performance cost | Use `Arc::clone(&value)` or references |
| Manual error types | Inconsistent | Use `thiserror::Error` |

## UNIQUE STYLES

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
// For concurrent access to tools
tools: DashMap<String, Arc<dyn ToolNode>>

// Insert
tools.insert("echo".to_string(), Arc::new(echo_tool));

// Get
if let Some(tool) = tools.get("echo") {
    tool.execute(params, context).await?;
}
```

### Error Context
```rust
// Provide context for errors
pub fn load_config(path: &Path) -> Result<Config> {
    Config::load_from_path(path)
        .map_err(|e| WorkflowError::Config(e).into())
}
```

## COMMANDS

### Build & Test
```bash
cargo build                    # Debug build
cargo build --release          # Optimized build
cargo build --all-features     # With LanceDB
cargo check                    # Quick check

cargo test                     # All tests
cargo test -- --nocapture      # With output
cargo test workflow::tests     # Specific module
cargo test property_tests      # Property-based tests
cargo test --test integration_tests  # Integration only
```

### Development
```bash
cargo fmt                      # Format code
cargo clippy -- -D warnings    # Lint strictly
cargo clippy --fix             # Auto-fix
cargo doc --open               # Generate docs
cargo run --example comprehensive_workflow_example  # Run examples
```

### Application
```bash
cargo run -- --help
cargo run -- workflow execute examples/hello-world.yaml
cargo run -- tool list
cargo run -- tui
cargo run -- server --http-port 8080 --ws-port 8081
```

### With Environment
```bash
RUST_LOG=debug cargo test
RUST_LOG=trace cargo run -- workflow execute workflow.yaml
WORKFLOW_TOOLKIT_LOGGING__LEVEL=debug cargo run
```

## GOTCHAS

1. **LanceDB Feature**: Optional, requires `--features lancedb` or `--all-features`
2. **Temp Directory**: Creates `std::env::temp_dir()/workflow-toolkit/` at runtime
3. **Config Hot Reload**: Spawns background task, errors printed to stderr
4. **Parallel Limit**: Default 4 parallel workflows, configurable via `Config`
5. **Checkpoint Interval**: 5 minutes by default, stores in storage backend
6. **Async Tests**: Must use `#[tokio::test]`, not `#[test]`
7. **Tool Registry**: Thread-safe via `Arc<dyn ToolRegistry>`, not `RwLock`
8. **Plugin Sandboxing**: Configurable via `plugins.sandbox_enabled` in config
9. **MCP Server**: Currently commented out in Cargo.toml (dependency issues)
10. **WASM Support**: Temporarily disabled, use Docker/Python/Node.js plugins

## PERFORMANCE NOTES

- **Caching**: Uses `moka::future::Cache` with TTL support
- **Concurrency**: `dashmap::DashMap` for read-heavy workloads
- **Memory**: `parking_lot::RwLock` for mutable shared state
- **Metrics**: Optional `metrics` crate integration (commented out)
- **Profiling**: Built-in profiler in `src/performance/profiler.rs`

## DEPENDENCY HIGHLIGHTS

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.42+ | Async runtime |
| `petgraph` | 0.6+ | DAG operations |
| `clap` | 4.5+ | CLI parsing |
| `ratatui` | 0.29+ | TUI framework |
| `serde` | 1.0+ | Serialization |
| `thiserror` | 2.0+ | Error handling |
| `dashmap` | 6.1+ | Concurrent HashMap |
| `moka` | 0.12+ | Caching |
| `async-trait` | 0.1+ | Async traits |
| `uuid` | 1.11+ | UUID generation |
| `chrono` | 0.4+ | Date/time |
| `lancedb` | 0.20+ | Vector DB (optional) |

## NEXT STEPS FOR NEW CONTRIBUTORS

1. **Read**: `README.md` and `DESIGN.md` for architecture
2. **Build**: `cargo build && cargo test` to verify setup
3. **Examples**: Run `cargo run --example comprehensive_workflow_example`
4. **Tests**: Study `tests/integration_tests.rs` for patterns
5. **Pick**: Choose a module from "WHERE TO LOOK" table
6. **Follow**: Conventions and anti-patterns strictly
7. **Test**: Add tests for all new code
8. **Lint**: Run `cargo fmt && cargo clippy -- -D warnings`
