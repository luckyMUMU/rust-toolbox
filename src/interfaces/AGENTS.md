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
- **cli/**: Command-line interface with subcommands, output formatting
- **tui/**: Terminal UI with event loop, widgets, themes, monitoring
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
- `src/interfaces/cli/app.rs` - Main CLI application
- `src/interfaces/cli/commands.rs` - Command definitions
- `src/interfaces/cli/output.rs` - Output formatting
- `src/interfaces/tui/app.rs` - TUI application
- `src/interfaces/tui/event.rs` - Event handling
- `src/interfaces/tui/widgets/` - Widget components
- `src/interfaces/mcp.rs` - MCP server (stub)

## IMPORTANT NOTES
- **MCP Server**: Stub implementation, compiles but non-functional
- **TUI**: Full-featured with widgets, themes, and monitoring
- **CLI**: Uses clap derive for command parsing
- All interfaces use the same backend components
