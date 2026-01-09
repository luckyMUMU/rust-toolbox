# src/interfaces/cli/ - Command-Line Interface

## OVERVIEW
Clap-based CLI with subcommands, output formatting, and error handling.

## COMMAND STRUCTURE
```
workflow-toolkit
├── workflow
│   ├── create
│   ├── list
│   ├── execute
│   ├── status
│   ├── pause
│   ├── resume
│   └── stop
├── tool
│   ├── list
│   ├── execute
│   └── validate
├── plugin
│   ├── load
│   ├── unload
│   └── list
├── batch
│   ├── process
│   └── classify
├── tui
├── server
└── config
```

## OUTPUT FORMATTING
**OutputFormat**: Enum for display modes
- **Table**: Human-readable tables
- **JSON**: Machine-parseable
- **YAML**: Structured configuration

**OutputFormatter**: Trait for formatting
- `JsonFormatter`, `YamlFormatter`, `TableFormatter`

## APP LIFECYCLE
1. **Parse**: Clap parses CLI arguments
2. **Config**: Load with priority (CLI > Env > File > Defaults)
3. **Components**: Inject dependencies (engine, registry, storage)
4. **Hot Reload**: Background config monitoring
5. **Execute**: Run command with proper output format
6. **Exit**: Clean shutdown with error codes

## ERROR HANDLING
- **CliError**: CLI-specific errors
- Structured error messages
- Exit codes: 0 success, 1 error, 2 usage error

## USAGE EXAMPLES
```bash
# Execute workflow
cargo run -- workflow execute examples/hello-world.yaml

# List tools
cargo run -- tool list

# Start TUI
cargo run -- tui

# Start server
cargo run -- server --http-port 8080 --ws-port 8081
```
