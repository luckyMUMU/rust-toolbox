# src/ - Core Library Modules

## OVERVIEW
Main library entry point with re-exports and module declarations.

## STRUCTURE
```
src/
├── config.rs          # Hierarchical configuration (env > CLI > file > defaults)
├── core.rs           # Shared types, ExecutionContext, RetryPolicy, enums
├── error.rs          # WorkflowError with 20+ variants, Result type alias
├── storage/          # Persistence layer (backends, state, backup)
├── workflow/         # DAG engine (scheduler, executor, validator, audit)
├── tools/            # Tool system (registry, nodes, templates, versioning)
├── plugins/          # Plugin system (native, python, nodejs, docker, file mgmt)
├── interfaces/       # User interfaces (CLI, TUI, MCP server)
└── performance/      # Optimization tools (cache, concurrency, metrics, profiling)
```

## KEY MODULES
- **init_logging()**: Initialize tracing subscriber with env filter
- **ConfigManager**: Hierarchical config with hot reload capability
- **WorkflowEngine**: Trait for DAG-based execution engines
- **ToolRegistry**: Trait for concurrent tool management
- **WorkflowError**: Centralized error type with constructors

## RE-EXPORTS
The library re-exports commonly used types:
```rust
pub use crate::config::{Config, ConfigManager, CliConfigOverrides};
pub use crate::core::*;
pub use crate::error::{Result, WorkflowError};
pub use crate::workflow::{WorkflowDefinition, WorkflowEngine, ExecutionManager};
pub use crate::tools::{ToolNode, ToolRegistry};
pub use crate::performance::{PerformanceManager, PerformanceConfig};
```

## USAGE
```rust
use workflow_toolkit::{Config, WorkflowEngine, Result, init_logging};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;
    let config = Config::load_with_priority()?;
    // Use components...
    Ok(())
}
```
