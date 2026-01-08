---
inclusion: always
---

# Project Structure & Code Organization

## Core Architecture Layers

The codebase follows a layered architecture with clear separation of concerns:

```
interfaces/ (CLI, TUI, MCP) → workflow/ → tools/ → plugins/
     ↓                          ↓         ↓         ↓
   core/ ← storage/ ← config/ ← error/ ← performance/
```

## Key Source Directories (`src/`)

### Core Foundation
- **`lib.rs`**: Public API exports - check here for available types and functions
- **`main.rs`**: CLI entry point - modify for new CLI features
- **`config.rs`**: Configuration management - hierarchical TOML with precedence rules
- **`core.rs`**: Shared types and structures - central definitions used across modules
- **`error.rs`**: Centralized error handling using `thiserror` - add new error types here

### Domain Modules (Primary Business Logic)

#### Workflow Engine (`src/workflow/`)
- **`definition.rs`**: Workflow data structures - modify for new workflow features
- **`engine.rs`**: Execution engine trait - implement for new execution strategies
- **`scheduler.rs`**: DAG-based task scheduling - handles parallel execution
- **`validator.rs`**: Pre-execution validation - add validation rules here
- **`execution.rs`**: State management during execution
- **`audit.rs`**: Execution logging and history tracking

#### Plugin System (`src/plugins/`)
- **`types.rs`**: Core plugin type definitions - start here for plugin development
- **`manager.rs`**: Plugin lifecycle management - registration, loading, unloading
- **`file_management/`**: Specialized file processing tools with human-in-the-loop capabilities
  - **`plugin.rs`**: Main file management plugin implementation
  - **`classification_tool.rs`**: Automated file classification with rules
  - **`human_decision_tool.rs`**: Interactive decision-making interface
  - **`batch_processor.rs`**: Bulk file operations with progress tracking
  - **`rule_config.rs`**: JSON-based classification rule management

#### Storage Layer (`src/storage/`)
- **`backends.rs`**: Storage backend trait - implement for new storage types
- **`state_manager.rs`**: Unified state persistence across components
- **`backup.rs`**: Backup and recovery functionality

#### Interface Layer (`src/interfaces/`)
- **`cli/`**: Command-line interface using `clap`
- **`mcp.rs`**: Model Context Protocol server for AI assistant integration
- **`tui.rs`**: Terminal user interface using `ratatui`

#### Performance (`src/performance/`)
- **`cache.rs`**: Multi-level caching with `moka`
- **`metrics.rs`**: Performance monitoring and profiling
- **`concurrency.rs`**: Async execution patterns and synchronization

## File Naming & Organization Rules

### When Creating New Files
- Use `snake_case` for all file and directory names
- Place related functionality in the same module directory
- Add `DESIGN.md` for complex modules explaining architecture decisions
- Use descriptive names that indicate purpose (e.g., `batch_processor.rs`, not `processor.rs`)

### Module Structure Pattern
```rust
// In mod.rs - always re-export public types
pub use self::engine::WorkflowEngine;
pub use self::definition::{WorkflowDefinition, StepDefinition};

// In implementation files - use structured imports
use crate::core::{ExecutionContext, Result};
use crate::error::WorkflowError;
```

## Code Style Conventions

### Rust Naming
- **Types/Structs/Enums**: `PascalCase` (e.g., `WorkflowDefinition`, `ExecutionStatus`)
- **Functions/Variables**: `snake_case` (e.g., `execute_workflow`, `node_id`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT_SECONDS`)
- **Traits**: Descriptive `PascalCase` (e.g., `ToolNode`, `StorageBackend`)

### Error Handling Pattern
```rust
// Use thiserror for structured errors
#[derive(thiserror::Error, Debug)]
pub enum WorkflowError {
    #[error("Validation failed: {message}")]
    ValidationError { message: String },
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}
```

### Async Patterns
- All I/O operations must be async using tokio
- Use `async-trait` for async trait methods
- Prefer `Arc<dyn Trait>` for shared trait objects
- Use `dashmap` or `Arc<Mutex<T>>` for shared mutable state

## Testing Strategy

### Test Organization
- **Unit tests**: In same file with `#[cfg(test)]` module
- **Integration tests**: Separate `tests.rs` files within modules
- **Property tests**: Use `proptest` for complex validation logic
- **Regression tests**: Store in `proptest-regressions/` directory

### Test Naming Pattern
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_workflow_validation_success() { /* ... */ }
    
    #[test]
    fn test_workflow_execution_with_retry() { /* ... */ }
}
```

## Configuration Management

### Hierarchical Configuration (Precedence Order)
1. Command line arguments (highest priority)
2. Environment variables (prefixed with `WORKFLOW_TOOLKIT_`)
3. Configuration files (`config/default.toml`)
4. Default values (lowest priority)

### Configuration Structure
- Use `serde` derive macros for all config structs
- Implement `Default` trait for all configuration types
- Group related settings in TOML sections
- Use descriptive names with units (e.g., `timeout_seconds`, `max_retries`)

## Examples Directory Usage

### When Adding Examples
- **Workflow examples**: Use `.yaml` format in `examples/` root
- **Plugin examples**: Create `.rs` files demonstrating plugin usage
- **Tool directories**: Group external tools by language (`python_tools/`, `nodejs_tools/`)
- **Templates**: Use `examples/templates/` for reusable workflow patterns

### Example Naming
- Use descriptive names: `comprehensive_workflow_example.rs` not `example1.rs`
- Include language/technology in name: `python_plugin_example.rs`
- Group related examples: `interactive-*-example.rs` for human-in-the-loop workflows

## Import and Dependency Guidelines

### Module Imports
```rust
// Prefer trait imports over concrete types at boundaries
use crate::workflow::WorkflowEngine;  // trait
use crate::storage::StorageBackend;   // trait

// Re-export commonly used types in lib.rs
pub use workflow::{WorkflowDefinition, ExecutionStatus};
pub use error::{Result, WorkflowError};
```

### Cross-Module Dependencies
- Interfaces depend on workflow, tools, and plugins
- Workflow depends on tools and plugins
- Tools depend on plugins for extensibility
- All modules can depend on core, config, and error
- Avoid circular dependencies between domain modules