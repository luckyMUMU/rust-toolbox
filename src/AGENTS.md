# src/ - Core Library Modules

## 概述 - Overview
Main library entry point with re-exports and module declarations.

**中文概述**: 核心库入口点，包含重新导出和模块声明。

## 模块结构 - Structure
```
src/
├── config.rs          # Hierarchical configuration (env > CLI > file > defaults)
├── core.rs           # Shared types, ExecutionContext, RetryPolicy, enums
├── error.rs          # WorkflowError with 20+ variants, Result type alias
├── storage/          # Persistence layer (backends, state, backup)
├── workflow/         # DAG engine (scheduler, executor, validator, audit)
│   ├── executor/    # Execution strategies (basic, parallel, retry)
│   ├── component/   # Workflow components (parallel, switch, loop)
│   ├── context/     # Execution context and data slots
│   └── state/       # State management and checkpoints
├── tools/            # Tool system (registry, nodes, templates, versioning)
│   └── algo/        # Algorithm implementations (Aho-Corasick)
├── plugins/          # Plugin system (native, python, nodejs, docker, file mgmt)
├── interfaces/       # User interfaces (CLI, TUI, MCP server)
│   ├── cli/         # Command-line interface (5 files, 1.5k+ lines)
│   └── tui/         # Terminal UI (25 files, 3.2k+ lines in layout.rs)
│       └── widgets/ # 8 specialized widgets (3.5k+ lines in plugin_manager.rs)
└── performance/      # Optimization tools (cache, concurrency, metrics, profiling)
```

## 关键模块 - Key Modules
- **init_logging()**: Initialize tracing subscriber with env filter
- **ConfigManager**: Hierarchical config with hot reload capability
- **WorkflowEngine**: Trait for DAG-based execution engines
- **ToolRegistry**: Trait for concurrent tool management
- **WorkflowError**: Centralized error type with constructors

## 重新导出 - Re-Exports
The library re-exports commonly used types:
```rust
pub use crate::config::{Config, ConfigManager, CliConfigOverrides};
pub use crate::core::*;
pub use crate::error::{Result, WorkflowError};
pub use crate::workflow::{WorkflowDefinition, WorkflowEngine, ExecutionManager};
pub use crate::tools::{ToolNode, ToolRegistry};
pub use crate::performance::{PerformanceManager, PerformanceConfig};
```

## 使用示例 - Usage
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
 
<!-- AUTO-GENERATED-AGENT-MAP:START -->
## 🗺️ Agent Map & Directory Structure

> **Auto-generated** on 2026-01-17 20:44:16

- **[interfaces/](interfaces/AGENTS.md)**: Multiple interface implementations: CLI, TUI, and MCP server.
- **[performance/](performance/AGENTS.md)**: Caching, concurrency control, memory optimization, metrics, and profiling.
- **[plugins/](plugins/AGENTS.md)**: Extensible plugin architecture supporting Native, Python, Node.js, Docker, and WASM plugins.
- **[storage/](storage/AGENTS.md)**: Storage backends, state management, backup system, and persistence.
- **[tools/](tools/AGENTS.md)**: Tool registry, node system, parameter templates, and version management.
- **[workflow/](workflow/AGENTS.md)**: Core workflow execution with DAG-based scheduling, state management, and audit logging.

<!-- AUTO-GENERATED-AGENT-MAP:END -->
