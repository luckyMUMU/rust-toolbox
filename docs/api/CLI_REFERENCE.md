# CLI Reference

> **Complete reference for the Workflow Toolkit Command Line Interface**  
> *Last Updated: 2026-01-14*

## Overview

The Workflow Toolkit CLI (`workflow-toolkit`) provides a unified interface for managing workflows, tools, plugins, and system services.

## Command Structure

```bash
workflow-toolkit [GLOBAL_OPTIONS] <COMMAND> [SUBCOMMAND] [ARGS]
```

## Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `--config <FILE>` | Path to configuration file | `config/default.toml` |
| `--log-level <LEVEL>` | Set log level (trace, debug, info, warn, error) | `info` |
| `--output <FORMAT>` | Output format (table, json, yaml, text) | `table` |
| `--verbose`, `-v` | Enable verbose output | `false` |
| `--quiet`, `-q` | Suppress non-error output | `false` |
| `--help`, `-h` | Show help information | - |
| `--version`, `-V` | Show version information | - |

---

## Workflow Management (`workflow`)

Manage workflow definitions and executions.

### `workflow create`
Create or validate a workflow definition.

```bash
workflow-toolkit workflow create [OPTIONS] <DEFINITION_FILE>
```

**Options:**
- `--validate-only`: Validate definition without saving
- `--force`: Force overwrite existing workflow

**Examples:**
```bash
# Validate a workflow file
workflow-toolkit workflow create --validate-only examples/hello-world.yaml

# Create/Import a workflow
workflow-toolkit workflow create examples/my-workflow.yaml
```

### `workflow execute`
Execute a workflow.

```bash
workflow-toolkit workflow execute [OPTIONS] <WORKFLOW_NAME>
```

**Options:**
- `--params <FILE>`: Path to parameters file (JSON/YAML)
- `--params-json <JSON>`: Parameters as JSON string
- `--background`: Execute in background
- `--wait`: Wait for completion and show progress
- `--timeout <SECONDS>`: Execution timeout
- `--dry-run`: Preview without executing

**Examples:**
```bash
# Execute with default parameters
workflow-toolkit workflow execute hello-world

# Execute with JSON parameters
workflow-toolkit workflow execute hello-world --params-json '{"name": "Alice"}' --wait

# Dry run
workflow-toolkit workflow execute hello-world --dry-run
```

### `workflow status`
Get workflow execution status.

```bash
workflow-toolkit workflow status [OPTIONS] <WORKFLOW_ID>
```

**Options:**
- `--detailed`: Show detailed node status
- `--follow`: Follow status updates in real-time
- `--interval <SECONDS>`: Refresh interval for follow mode (default: 2)

**Examples:**
```bash
# Check status
workflow-toolkit workflow status 1234-5678

# Watch status updates
workflow-toolkit workflow status 1234-5678 --follow --detailed
```

### `workflow pause` / `resume` / `stop`
Control workflow execution.

```bash
workflow-toolkit workflow pause <WORKFLOW_ID>
workflow-toolkit workflow resume <WORKFLOW_ID>
workflow-toolkit workflow stop <WORKFLOW_ID> [--force]
```

### `workflow list`
List workflows.

```bash
workflow-toolkit workflow list [OPTIONS]
```

**Options:**
- `--status <STATUS>`: Filter by execution status (Pending, Running, etc.)
- `--recent`: Show only recent executions
- `--limit <N>`: Maximum number of results (default: 50)

---

## Tool Management (`tool`)

Inspect and execute individual tools.

### `tool list`
List available tools.

```bash
workflow-toolkit tool list [OPTIONS]
```

**Options:**
- `--category <CAT>`: Filter by category
- `--tag <TAG>`: Filter by tag
- `--search <QUERY>`: Search by name or description
- `--detailed`: Show detailed information

**Examples:**
```bash
# List all tools
workflow-toolkit tool list

# Search for file tools
workflow-toolkit tool list --search "file"
```

### `tool execute`
Execute a specific tool directly.

```bash
workflow-toolkit tool execute [OPTIONS] <TOOL_NAME>
```

**Options:**
- `--params <JSON>`: Tool parameters as JSON string
- `--params-file <FILE>`: Path to parameters file
- `--timeout <SECONDS>`: Execution timeout
- `--dry-run`: Validate parameters without executing

**Examples:**
```bash
# Execute echo tool
workflow-toolkit tool execute echo --params '{"message": "Hello"}'

# Execute file read
workflow-toolkit tool execute file_read --params '{"path": "data.txt"}'
```

### `tool info`
Show detailed information about a tool.

```bash
workflow-toolkit tool info <TOOL_NAME>
```

---

## Plugin Management (`plugin`)

Manage extensions and plugins.

### `plugin install`
Install a new plugin.

```bash
workflow-toolkit plugin install [OPTIONS] <PLUGIN_PATH>
```

**Options:**
- `--plugin-type <TYPE>`: Explicitly set plugin type (native, python, nodejs, docker, wasm)
- `--force`: Reinstall if exists

**Examples:**
```bash
# Install local Python plugin
workflow-toolkit plugin install ./plugins/my-python-plugin

# Install from URL (if supported)
workflow-toolkit plugin install https://example.com/plugins/my-plugin.zip
```

### `plugin list`
List installed plugins.

```bash
workflow-toolkit plugin list [OPTIONS]
```

**Options:**
- `--detailed`: Show detailed info
- `--plugin-type <TYPE>`: Filter by type

### `plugin reload` / `uninstall`
Manage plugin lifecycle.

```bash
workflow-toolkit plugin reload <PLUGIN_NAME>
workflow-toolkit plugin uninstall <PLUGIN_NAME> [--force]
```

---

## Batch Operations (`batch`)

Execute multiple workflows in batch.

### `batch execute`
Execute workflows from a list file.

```bash
workflow-toolkit batch execute [OPTIONS] <WORKFLOW_LIST_FILE>
```

**Options:**
- `--parallel <N>`: Maximum parallel executions (default: 4)
- `--continue-on-failure`: Continue executing even if some workflows fail
- `--output-dir <DIR>`: Directory to save results
- `--timeout <SECONDS>`: Timeout per workflow

**Examples:**
```bash
# Run batch with 8 parallel workers
workflow-toolkit batch execute batch-jobs.yaml --parallel 8
```

---

## Other Commands

### `tui`
Start the Terminal User Interface.

```bash
workflow-toolkit tui
```

### `server`
Start the MCP (Model Context Protocol) Server.

```bash
workflow-toolkit server [OPTIONS]
```

**Options:**
- `--http-port <PORT>`: HTTP port (default: 8080)
- `--ws-port <PORT>`: WebSocket port (default: 8081)
- `--auth`: Enable authentication

### `completion`
Generate shell completion scripts.

```bash
workflow-toolkit completion <SHELL>
```

**Supported Shells:** bash, zsh, fish, powershell

**Example:**
```bash
# Generate PowerShell completion
workflow-toolkit completion powershell > completion.ps1
```
