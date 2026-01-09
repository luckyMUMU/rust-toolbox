# src/interfaces/ - User Interfaces

## OVERVIEW
Multiple interface implementations: CLI, TUI, and MCP server.

## INTERFACES
| Interface | Framework | Use Case |
|-----------|-----------|----------|
| **CLI** | Clap 4.5 | Scripting, automation |
| **TUI** | Ratatui 0.29 | Interactive terminal UI |
| **MCP** | JSON-RPC | IDE integration |

## SUBDIRECTORIES
- **cli/**: Command-line interface with subcommands
- **tui/**: Terminal UI with event loop and widgets
- **mcp.rs**: Model Context Protocol server (basic)

## ARCHITECTURE
All interfaces share:
- `CliApp` with component injection
- Common command structures
- Unified output formatting
- Error handling patterns

## USAGE
```bash
# CLI
cargo run -- workflow execute workflow.yaml

# TUI
cargo run -- tui

# MCP Server
cargo run -- server --http-port 8080
```
