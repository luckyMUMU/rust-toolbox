# src/interfaces/ - User Interfaces

## OVERVIEW
Multiple interface implementations: CLI, TUI, and MCP server.

## INTERFACES
| Interface | Framework | Use Case | Key Files |
|-----------|-----------|----------|-----------|
| **CLI** | Clap 4.5 | Scripting, automation | `cli/app.rs`, `cli/commands.rs` |
| **TUI** | Ratatui 0.29 | Interactive terminal UI | `tui/app.rs`, `tui/widgets/` |
| **MCP** | JSON-RPC | IDE integration | `mcp.rs` (stub) |

## SUBDIRECTORIES
- **cli/**: Command-line interface with subcommands, output formatting (5 files, 1,583+ lines)
- **tui/**: Terminal UI with event loop, widgets, themes, monitoring (25 files, 3,275+ lines in layout.rs)
  - **widgets/**: 8 specialized widgets (3,553+ lines in plugin_manager.rs)
- **mcp.rs**: Model Context Protocol server (stub implementation)

## ARCHITECTURE
All interfaces share:
- `CliApp` with component injection (storage, registry, engine)
- Common command structures via `Commands` enum
- Unified output formatting via `OutputFormatter` trait
- Error handling patterns using `WorkflowError`

## SHARED COMPONENTS
- **ConfigManager**: Hierarchical configuration
- **StateManager**: Persistence layer
- **ToolRegistry**: Tool management
- **WorkflowEngine**: DAG execution
- **PluginManager**: Plugin lifecycle

## CLI INTERFACE
**Files**: `cli/app.rs` (1,583 lines), `cli/commands.rs`, `cli/output.rs`, `cli/error.rs`

**Features**:
- Clap-based command parsing
- Multiple output formats (Table, JSON, YAML, Text)
- Hot reload configuration
- Batch workflow execution
- MCP server (stub)

**Commands**:
```
workflow-toolkit workflow <create|list|execute|status|pause|resume|stop>
workflow-toolkit tool <list|execute|info>
workflow-toolkit plugin <install|list|reload|uninstall>
workflow-toolkit batch execute <file>
workflow-toolkit tui
workflow-toolkit server [--http-port] [--ws-port]
```

## TUI INTERFACE
**Files**: `tui/app.rs` (1,340 lines), `tui/event.rs`, `tui/layout.rs` (3,275 lines), `tui/theme.rs` (1,385 lines)

**Features**:
- Event-driven architecture
- 8 specialized widgets
- Real-time monitoring
- System health dashboard
- Interactive workflow management

**Widgets**:
- `WorkflowListWidget`: Browse and execute workflows
- `ExecutionMonitorWidget`: Real-time execution tracking
- `LogViewerWidget`: Audit log viewing and filtering
- `SystemStatusWidget`: CPU, memory, disk, network monitoring
- `ToolManagerWidget`: Tool registry management
- `PluginManagerWidget`: Plugin lifecycle management
- `SyncStatusWidget`: Synchronization state display
- `ToolManagerWidget`: Tool execution interface

## USAGE
```bash
# CLI
cargo run -- workflow execute workflow.yaml
cargo run -- tool list
cargo run -- tui

# TUI
cargo run -- tui
# Navigate: Tab, ↑/↓, Enter, Esc, q

# MCP Server (stub)
cargo run -- server --http-port 8080
```

## KEY FILES
- `src/interfaces/cli/app.rs` - Main CLI application (1,583 lines)
- `src/interfaces/cli/commands.rs` - Command definitions
- `src/interfaces/cli/output.rs` - Output formatting
- `src/interfaces/cli/error.rs` - CLI-specific errors
- `src/interfaces/tui/app.rs` - TUI application (1,340 lines)
- `src/interfaces/tui/event.rs` - Event handling
- `src/interfaces/tui/layout.rs` - Layout management (3,275 lines)
- `src/interfaces/tui/theme.rs` - Theme system (1,385 lines)
- `src/interfaces/tui/widgets/` - 8 widget components
- `src/interfaces/mcp.rs` - MCP server (stub)

## IMPORTANT NOTES
- **MCP Server**: Stub implementation, compiles but non-functional
- **TUI**: Full-featured with widgets, themes, and monitoring
- **CLI**: Uses clap derive for command parsing
- All interfaces use the same backend components
- **Performance**: TUI optimized with virtual scrolling for large datasets

<!-- AUTO-GENERATED-AGENT-MAP:START -->
## 🗺️ Agent Map & Directory Structure

> **Auto-generated** on 2026-01-17 20:44:16

- **[cli/](cli/AGENTS.md)**: Clap-based CLI with subcommands, output formatting, and error handling.
- **[tui/](tui/AGENTS.md)**: Ratatui-based TUI with event-driven architecture, action system, and reactive widgets.

<!-- AUTO-GENERATED-AGENT-MAP:END -->
