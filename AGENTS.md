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
**Last Updated:** 2026-01-13

## Quick Build & Test Commands

```bash
# Build & Verify
cargo check                    # Quick type check (preferred)
cargo build                    # Debug build
cargo build --release          # Optimized
cargo build --all-features     # With LanceDB

# Test Commands
cargo test                     # All tests
cargo test --test integration_tests -- test_workflow_execution  # Single test
cargo test --test tui_standalone_unit_tests -- test_widget_rendering
cargo test -- --nocapture      # Show output
RUST_LOG=debug cargo test      # With logging

# Format & Lint (Run before completion)
cargo fmt
cargo clippy -- -D warnings
cargo test

# Run & Examples
cargo run -- --help
cargo run -- workflow execute examples/hello-world.yaml
cargo run -- tui
cargo run --example comprehensive_workflow_example
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

// ✅ CORRECT: Constructor methods
return Err(WorkflowError::tool("Invalid parameters"));
return Err(WorkflowError::plugin("Loading failed"));
return Err(WorkflowError::storage("Connection lost"));

// ❌ NEVER: Type suppression
let x: u32 = value as any;           // Forbidden
#[ts-ignore]                         // Forbidden

// ❌ NEVER: Empty error handling
catch(e) {}                          // Forbidden

// ❌ NEVER: unwrap() in production
let value = some_result.unwrap();    // Forbidden - use ?
```

### Async Patterns
```rust
// ✅ CORRECT: Async traits
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute(&self, def: WorkflowDefinition) -> Result<WorkflowExecution>;
}

// ✅ CORRECT: Concurrent collections
use dashmap::DashMap;              // Read-heavy access
use tokio::sync::RwLock;           // Mutable shared state

// ✅ CORRECT: Semaphore for concurrency
let semaphore = Arc::new(Semaphore::new(4));
let permit = semaphore.acquire().await?;
```

### Type Safety
```rust
// ✅ CORRECT: Newtype pattern
pub struct WorkflowId(Uuid);

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

### Testing Patterns
```rust
// ✅ CORRECT: Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() { }
    
    #[tokio::test]
    async fn test_async_something() { }
}

// ✅ CORRECT: Use tempfile for isolation
#[tokio::test]
async fn test_with_temp_dir() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    // Use temp_dir.path()
}
```

### Logging
```rust
// ✅ CORRECT: Use tracing macros
use tracing::{info, debug, warn, error};

info!("Workflow started: {}", workflow_id);
debug!("Parameters: {:?}", params);
info!(workflow_id = %workflow_id, node_id = %node_id, "Executing node");

// ❌ NEVER: println! or eprintln!
println!("Debug info");  // Forbidden - use tracing
```

## Key Project Structure

```
rust-tool-v2/
├── src/
│   ├── lib.rs                    # Module declarations
│   ├── core.rs                   # Shared types
│   ├── error.rs                  # WorkflowError
│   ├── config.rs                 # Config management
│   ├── workflow/                 # DAG engine
│   │   ├── engine.rs             # Execution engine
│   │   ├── scheduler.rs          # DAG scheduler
│   │   └── validator.rs          # Validation
│   ├── tools/                    # Tool system
│   │   ├── registry.rs           # Tool registry
│   │   └── node.rs               # Tool nodes
│   ├── plugins/                  # Plugin system
│   │   ├── manager.rs            # Plugin manager
│   │   └── file_management/      # File ops
│   ├── interfaces/               # User interfaces
│   │   ├── cli/                  # CLI (clap)
│   │   └── tui/                  # TUI (ratatui)
│   ├── storage/                  # Persistence
│   └── performance/              # Optimization
├── tests/                        # Test suite
│   ├── integration_tests.rs
│   ├── tui_standalone_unit_tests.rs
│   └── file_management_integration_tests.rs
├── examples/                     # Example workflows
└── Cargo.toml                    # 87 dependencies
```

## Critical Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime (full features) |
| `petgraph` | DAG operations |
| `clap` | CLI parsing |
| `ratatui` | TUI framework |
| `serde` | Serialization |
| `thiserror` | Error handling |
| `dashmap` | Concurrent HashMap |
| `moka` | High-performance caching |
| `async-trait` | Async trait support |

## Anti-Patterns (FORBIDDEN)

| Pattern | Why | Alternative |
|---------|-----|-------------|
| `as any` | Loses type safety | `try_into()`, `into()` |
| `@ts-ignore` | Hides errors | Fix error or `#[allow(...)]` |
| `empty catch {}` | Swallows errors | Use `?` or handle explicitly |
| `println!` | No structured logging | `tracing::info!` |
| `std::sync::Mutex` in async | Blocks executor | `tokio::sync::RwLock` or `DashMap` |
| `unwrap()` in production | Panics | Use `?` with proper error handling |
| `clone()` on Arc unnecessarily | Performance cost | `Arc::clone(&value)` |
| Manual error types | Inconsistent | Use `thiserror::Error` |
| Type suppression | Hides bugs | Fix types properly |
| Empty error handling | Swallows errors | Always handle or propagate |
| Deleting failing tests | Hides problems | Fix test or code |

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
tools.insert("echo".to_string(), Arc::new(echo_tool));
if let Some(tool) = tools.get("echo") {
    tool.execute(params, context).await?;
}
```

## Important Notes

1. **LanceDB**: Optional, requires `--features lancedb`
2. **MCP Server**: Stub implementation (compiles, non-functional)
3. **WASM**: Disabled (use Docker/Python/Node.js plugins)
4. **Windows**: May have memory issues with full build; use `cargo check`
5. **Async Tests**: Must use `#[tokio::test]`, not `#[test]`
6. **Tool Registry**: Thread-safe via `Arc<dyn ToolRegistry>`

## Development Workflow

### Before Starting
1. Read `openspec/AGENTS.md` for spec-driven development
2. Run `cargo check` to verify compilation
3. Review related module AGENTS.md files

### Implementation Steps
1. **Plan**: Create TODO list for multi-step tasks
2. **Implement**: Follow code style strictly
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

## When to Consult Oracle

**Consult Oracle for:**
- Complex architecture decisions
- After 2+ failed fix attempts
- Unfamiliar code patterns
- Security/performance concerns
- Multi-system tradeoffs

**Don't consult for:**
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

**Key Files:**
- `src/core.rs`, `src/error.rs`, `src/config.rs`
- `src/workflow/engine.rs`, `src/tools/registry.rs`
- `src/interfaces/cli/app.rs`, `src/interfaces/tui/app.rs`
- `openspec/AGENTS.md`
