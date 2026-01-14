# API & CLI Index

> **Complete reference for all CLI commands and API endpoints**  
> *Last Updated: 2026-01-14*

---

## 📋 Quick Reference

### Command Structure
```bash
rust-tool-v2 [COMMAND] [SUBCOMMAND] [OPTIONS]
```

### Global Options
```bash
--help, -h          Show help information
--version, -v       Show version
--verbose           Enable verbose output
--quiet             Suppress non-essential output
--config <PATH>     Use custom config file
```

---

## 🚀 Core Commands

### 1. Workflow Management

#### Execute Workflow
```bash
cargo run -- workflow execute <WORKFLOW_FILE>
```

**Options:**
```bash
--dry-run           Preview without executing
--verbose           Show detailed execution
--parallel <N>      Run N tasks in parallel (default: 4)
--timeout <SECS>    Set execution timeout
--retry <N>         Retry failed tasks N times
```

**Examples:**
```bash
# Basic execution
cargo run -- workflow execute examples/hello-world.yaml

# Dry run
cargo run -- workflow execute examples/hello-world.yaml --dry-run

# Parallel execution
cargo run -- workflow execute examples/large-workflow.yaml --parallel 8

# With timeout
cargo run -- workflow execute examples/long-workflow.yaml --timeout 300
```

#### Validate Workflow
```bash
cargo run -- workflow validate <WORKFLOW_FILE>
```

**Output:**
- Syntax validation
- Dependency check
- Circular dependency detection
- Resource availability

#### Show Workflow Graph
```bash
cargo run -- workflow graph <WORKFLOW_FILE>
```

**Options:**
```bash
--format <TYPE>     Output format: text, dot, mermaid
--output <FILE>     Save graph to file
```

**Example:**
```bash
cargo run -- workflow graph examples/hello-world.yaml --format mermaid
```

#### List Workflows
```bash
cargo run -- workflow list
```

**Options:**
```bash
--dir <PATH>        Search directory (default: ./workflows)
--pattern <GLOB>    File pattern (default: *.yaml)
--recursive         Search subdirectories
```

---

### 2. File Management Tools

#### File Classifier
```bash
cargo run -- file-classifier [OPTIONS]
```

**Required:**
```bash
--source <DIR>      Source directory
--dest <DIR>        Destination directory
```

**Options:**
```bash
--pattern <PATTERN>     File patterns (comma-separated)
--exclude <PATTERNS>    Exclude patterns
--recursive             Process subdirectories
--dry-run               Preview changes
--threads <N>           Thread count (default: 4)
--verbose               Show detailed classification
```

**Examples:**
```bash
# Basic classification
cargo run -- file-classifier --source ./downloads --dest ./organized

# With patterns
cargo run -- file-classifier --source ./data --dest ./sorted \
  --pattern "*.pdf,*.docx,*.txt" --exclude "*.tmp"

# Multi-threaded
cargo run -- file-classifier --source ./large --dest ./output --threads 8
```

#### File Mover
```bash
cargo run -- file-mover [OPTIONS]
```

**Required:**
```bash
--source <PATH>     Source file/directory
--dest <PATH>       Destination file/directory
```

**Options:**
```bash
--force             Overwrite existing files
--preserve          Preserve metadata
--verify            Verify after move
--batch             Batch mode (no prompts)
--dry-run           Preview changes
```

**Examples:**
```bash
# Single file
cargo run -- file-mover --source file.txt --dest ./archive/

# Directory with verification
cargo run -- file-mover --source ./old --dest ./new --verify

# Batch move
cargo run -- file-mover --source ./temp/* --dest ./permanent/ --batch
```

#### Folder Merger
```bash
cargo run -- folder-merger [OPTIONS]
```

**Required:**
```bash
--source <DIR>      Source directory
--dest <DIR>        Destination directory
```

**Options:**
```bash
--strategy <MODE>       Merge strategy: keep-both, replace, skip
--duplicate-check       Check for duplicates
--interactive           Human decision for conflicts
--dry-run               Preview merge
--backup                Create backup before merge
```

**Examples:**
```bash
# Basic merge
cargo run -- folder-merger --source ./folder1 --dest ./folder2

# With duplicate handling
cargo run -- folder-merger --source ./folder1 --dest ./folder2 \
  --strategy keep-both --duplicate-check

# Interactive mode
cargo run -- folder-merger --source ./folder1 --dest ./folder2 --interactive
```

#### Batch Processor
```bash
cargo run -- batch-processor [OPTIONS]
```

**Options:**
```bash
--batch-size <N>        Items per batch (default: 100)
--parallel <N>          Parallel batches (default: 4)
--auto                  Auto-confirm all operations
--delay <MS>            Delay between batches
--progress              Show progress bar
```

**Examples:**
```bash
# Process large directory
cargo run -- batch-processor --source ./large --dest ./output \
  --batch-size 500 --parallel 8

# With progress
cargo run -- batch-processor --source ./data --dest ./processed \
  --batch-size 100 --progress
```

#### Human Decision
```bash
cargo run -- human-decision [OPTIONS]
```

**Options:**
```bash
--interactive           Interactive mode
--auto                  Auto mode (skip prompts)
--prompt <TEXT>         Custom prompt
--timeout <SECS>        Response timeout
```

**Examples:**
```bash
# Interactive prompt
cargo run -- human-decision --interactive --prompt "Keep this file?"

# Auto mode for batch
cargo run -- human-decision --auto
```

---

### 3. TUI (Terminal User Interface)

#### Launch TUI
```bash
cargo run -- tui [OPTIONS]
```

**Options:**
```bash
--basic             Basic mode (no colors/animations)
--no-color          Disable colors
--theme <FILE>      Load custom theme
--layout <TYPE>     Layout: default, compact, wide
--debug             Show debug info
```

**Examples:**
```bash
# Standard TUI
cargo run -- tui

# Basic mode for compatibility
cargo run -- tui --basic

# With custom theme
cargo run -- tui --theme ~/.config/rust-tool-v2/theme.toml
```

#### TUI Commands (In-App)
```
Navigation:
  ↑/↓ or j/k      Move selection
  Enter           Select/Confirm
  Esc or q        Quit/Back
  Tab             Switch panels
  /               Search

Workflow:
  e               Execute workflow
  v               Validate
  g               Show graph
  l               List workflows

File Management:
  c               Classify files
  m               Move files
  r               Merge folders
  b               Batch process

System:
  s               Settings
  h               Help
  d               Debug info
```

---

### 4. MCP Server

#### Start MCP Server
```bash
cargo run -- mcp-server [OPTIONS]
```

**Options:**
```bash
--host <HOST>          Bind address (default: 127.0.0.1)
--port <PORT>          Port number (default: 8080)
--config <FILE>        Config file path
--workers <N>          Worker threads (default: 4)
--log-level <LEVEL>    Log level: trace, debug, info, warn, error
```

**Examples:**
```bash
# Default server
cargo run -- mcp-server

# Custom port
cargo run -- mcp-server --port 8081

# Production server
cargo run -- mcp-server --host 0.0.0.0 --port 8080 --workers 8
```

#### MCP Client Commands
```bash
# Health check
curl http://localhost:8080/health

# Execute workflow
curl -X POST http://localhost:8080/workflow/execute \
  -H "Content-Type: application/json" \
  -d '{"file": "examples/hello-world.yaml"}'

# List workflows
curl http://localhost:8080/workflow/list

# File operations
curl -X POST http://localhost:8080/file/classify \
  -H "Content-Type: application/json" \
  -d '{"source": "./data", "dest": "./output"}'
```

---

### 5. Plugin Management

#### List Plugins
```bash
cargo run -- plugin list
```

**Output:**
- Plugin name
- Version
- Status (enabled/disabled)
- Description

#### Load Plugin
```bash
cargo run -- plugin load <PLUGIN_PATH>
```

**Options:**
```bash
--name <NAME>      Plugin name
--enable           Enable after loading
```

#### Unload Plugin
```bash
cargo run -- plugin unload <PLUGIN_NAME>
```

#### Enable/Disable Plugin
```bash
cargo run -- plugin enable <PLUGIN_NAME>
cargo run -- plugin disable <PLUGIN_NAME>
```

---

### 6. Configuration

#### Show Config
```bash
cargo run -- config show
```

**Output:**
- Current configuration
- Default values
- Environment overrides

#### Set Config
```bash
cargo run -- config set <KEY> <VALUE>
```

**Examples:**
```bash
cargo run -- config set parallel_tasks 8
cargo run -- config set timeout 300
cargo run -- config set log_level debug
```

#### Reset Config
```bash
cargo run -- config reset
```

**Warning:** This will reset all settings to defaults.

#### Config Locations
```bash
# Linux/macOS
~/.config/rust-tool-v2/config.toml

# Windows
%APPDATA%\rust-tool-v2\config.toml

# Environment variable
export RUST_TOOL_V2_CONFIG=/path/to/config.toml
```

---

### 7. Database (LanceDB)

#### Initialize Database
```bash
cargo run -- lancedb init
```

**Options:**
```bash
--dir <PATH>        Database directory
--reset             Reset existing database
```

#### Query Database
```bash
cargo run -- lancedb query <SEARCH_TERM>
```

**Options:**
```bash
--limit <N>         Limit results (default: 10)
--table <NAME>      Specific table
--filter <EXPR>     Filter expression
```

#### Import to Database
```bash
cargo run -- lancedb import <SOURCE>
```

**Options:**
```bash
--table <NAME>      Target table
--format <TYPE>     Input format: json, csv, yaml
--batch <N>         Batch size
```

---

## 🔧 API Reference

### Rust API (For Developers)

#### Core Types
```rust
use rust_tool_v2::core::{WorkflowId, ExecutionContext, ExecutionStatus};

// Workflow ID
let id = WorkflowId::new();

// Execution context
let ctx = ExecutionContext::builder()
    .timeout(Duration::from_secs(300))
    .parallel(4)
    .build();

// Status checking
let status = ctx.status();
if status.is_terminal() {
    println!("Execution completed: {:?}", status);
}
```

#### Workflow Engine
```rust
use rust_tool_v2::workflow::{WorkflowEngine, WorkflowDefinition};

// Create engine
let engine = WorkflowEngine::new();

// Load workflow
let def = WorkflowDefinition::from_file("examples/hello-world.yaml")?;

// Execute
let result = engine.execute(def).await?;
println!("Result: {:?}", result.status);
```

#### Tool Registry
```rust
use rust_tool_v2::tools::{ToolRegistry, ToolNode};

// Create registry
let registry = ToolRegistry::new();

// Register tool
let tool = ToolNode::builder()
    .name("custom-tool")
    .version("1.0.0")
    .executor(executor)
    .build()?;

registry.register(tool)?;

// Execute tool
let result = registry.execute("custom-tool", params, context).await?;
```

#### Error Handling
```rust
use rust_tool_v2::error::{Result, WorkflowError};

fn fallible_operation() -> Result<()> {
    let value = operation().map_err(|e| {
        WorkflowError::workflow_execution(&format!("Failed: {}", e))
    })?;
    Ok(())
}

// Error types
match error {
    WorkflowError::Tool(msg) => eprintln!("Tool error: {}", msg),
    WorkflowError::Workflow(msg) => eprintln!("Workflow error: {}", msg),
    WorkflowError::Plugin(msg) => eprintln!("Plugin error: {}", msg),
    WorkflowError::Storage(msg) => eprintln!("Storage error: {}", msg),
}
```

#### Async Patterns
```rust
use async_trait::async_trait;
use tokio::sync::Semaphore;
use std::sync::Arc;

#[async_trait]
pub trait CustomExecutor: Send + Sync {
    async fn execute(&self, params: Params) -> Result<Output>;
}

// Concurrent execution
let semaphore = Arc::new(Semaphore::new(4));
for task in tasks {
    let permit = semaphore.acquire().await?;
    let handle = tokio::spawn(async move {
        // Execute task
        task.execute().await
    });
    handles.push(handle);
}
```

---

## 📊 Command Summary Table

| Category | Command | Purpose | Required Options |
|----------|---------|---------|------------------|
| **Workflow** | `workflow execute` | Run workflow | `--file` |
| | `workflow validate` | Check workflow | `--file` |
| | `workflow graph` | Show dependencies | `--file` |
| | `workflow list` | List workflows | None |
| **File Mgmt** | `file-classifier` | Organize files | `--source`, `--dest` |
| | `file-mover` | Move files | `--source`, `--dest` |
| | `folder-merger` | Merge folders | `--source`, `--dest` |
| | `batch-processor` | Batch operations | None |
| | `human-decision` | Interactive mode | None |
| **TUI** | `tui` | Launch interface | None |
| **MCP** | `mcp-server` | Start server | None |
| **Plugins** | `plugin list` | List plugins | None |
| | `plugin load` | Load plugin | `--path` |
| **Config** | `config show` | Show config | None |
| | `config set` | Set value | `--key`, `--value` |
| **Database** | `lancedb init` | Initialize DB | None |
| | `lancedb query` | Search DB | `--term` |

---

## 🔍 Search Commands

### Find by Function
```bash
# Workflow operations
grep -r "workflow" docs/API_INDEX.md

# File operations
grep -r "file-" docs/API_INDEX.md

# TUI operations
grep -r "tui" docs/API_INDEX.md

# MCP operations
grep -r "mcp" docs/API_INDEX.md
```

### Find by Option
```bash
# Common options
grep -r "\-\-dry-run" docs/API_INDEX.md
grep -r "\-\-parallel" docs/API_INDEX.md
grep -r "\-\-verbose" docs/API_INDEX.md
```

---

## 🎯 Usage Patterns

### Pattern 1: Quick Workflow
```bash
# 1. Create workflow
cat > test.yaml << 'EOF'
name: test
steps:
  - name: hello
    tool: echo
    params:
      message: "Hello World"
EOF

# 2. Validate
cargo run -- workflow validate test.yaml

# 3. Execute
cargo run -- workflow execute test.yaml
```

### Pattern 2: File Organization
```bash
# 1. Classify
cargo run -- file-classifier \
  --source ./downloads \
  --dest ./organized \
  --pattern "*.pdf,*.docx,*.txt" \
  --dry-run

# 2. Execute (if dry-run looks good)
cargo run -- file-classifier \
  --source ./downloads \
  --dest ./organized \
  --pattern "*.pdf,*.docx,*.txt"
```

### Pattern 3: Batch Processing
```bash
# Process large directory with progress
cargo run -- batch-processor \
  --source ./large \
  --dest ./processed \
  --batch-size 500 \
  --parallel 8 \
  --progress
```

### Pattern 4: Interactive Mode
```bash
# Launch TUI for visual workflow management
cargo run -- tui

# Or use interactive file operations
cargo run -- folder-merger \
  --source ./folder1 \
  --dest ./folder2 \
  --interactive
```

---

## 📝 Common Command Combinations

### Development Workflow
```bash
# Check, build, test, run
cargo check && cargo build && cargo test && cargo run -- --help
```

### Production Deployment
```bash
# Build release
cargo build --release

# Run with optimized binary
./target/release/rust-tool-v2 workflow execute production.yaml
```

### Debugging
```bash
# Verbose logging
RUST_LOG=debug cargo run -- workflow execute test.yaml

# Trace mode
RUST_LOG=trace cargo run -- workflow execute test.yaml 2>&1 | tee debug.log
```

### Performance Testing
```bash
# Time execution
time cargo run --release -- workflow execute large.yaml

# Profile
cargo build --release
/usr/bin/time -v ./target/release/rust-tool-v2 workflow execute large.yaml
```

---

## 🔗 Related Documentation

- **[USER_GUIDE.md](USER_GUIDE.md)** - Detailed usage examples
- **[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - Development practices
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues
- **[INDEX.md](INDEX.md)** - Complete documentation index

---

## ✅ Verification Commands

After any command, verify with:

```bash
# Check exit code
echo $?

# Verify output
cargo run -- workflow execute test.yaml && echo "Success"

# Check logs
RUST_LOG=info cargo run -- workflow execute test.yaml 2>&1 | grep -i error
```

---

**← Back to [INDEX.md](INDEX.md)** | **Top** ↑