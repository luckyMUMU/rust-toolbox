# Project Structure

## Root Directory Layout

```
workflow-toolkit/
├── src/                    # Source code
├── examples/              # Example workflows and usage
├── config/               # Default configuration files
├── target/               # Cargo build artifacts (gitignored)
├── tmp/                  # Temporary files and test artifacts
├── rt-tools/             # Runtime tools directory
├── proptest-regressions/ # Property test regression data
├── Cargo.toml           # Project manifest and dependencies
├── Cargo.lock           # Dependency lock file
├── README.md            # Project documentation
├── DESIGN.md            # Architecture and design decisions
└── .gitignore           # Git ignore patterns
```

## Source Code Organization (`src/`)

### Core Modules
- **`lib.rs`**: Library entry point with public API exports
- **`main.rs`**: CLI application entry point
- **`config.rs`**: Hierarchical configuration management
- **`core.rs`**: Core type definitions and shared structures
- **`error.rs`**: Centralized error handling with thiserror

### Domain Modules

#### Storage Layer (`src/storage/`)
- **`mod.rs`**: Module exports and public interface
- **`backends.rs`**: Storage backend trait definitions
- **`state_manager.rs`**: Unified state management
- **`tests.rs`**: Storage layer tests
- **`DESIGN.md`**: Storage architecture documentation

#### Workflow Engine (`src/workflow/`)
- **`mod.rs`**: Module exports
- **`definition.rs`**: Workflow definition data structures
- **`engine.rs`**: Workflow execution engine trait
- **`execution.rs`**: Execution state management
- **`scheduler.rs`**: DAG-based task scheduling
- **`validator.rs`**: Workflow validation logic
- **`retry_tests.rs`**: Retry mechanism tests
- **`DESIGN.md`**: Workflow system design

#### Tool System (`src/tools/`)
- **`mod.rs`**: Module exports
- **`node.rs`**: Tool node trait and implementations
- **`registry.rs`**: Tool registration and discovery
- **`DESIGN.md`**: Tool system architecture

#### Plugin System (`src/plugins/`)
- **`mod.rs`**: Module exports
- **`types.rs`**: Plugin type definitions
- **`manager.rs`**: Plugin lifecycle management
- **`native.rs`**: Native Rust plugin support
- **`python.rs`**: Python plugin wrapper
- **`nodejs.rs`**: Node.js plugin wrapper
- **`nodejs_tests.rs`**: Node.js plugin tests
- **`docker.rs`**: Docker container plugin support
- **`wasm.rs`**: WebAssembly plugin support
- **`DESIGN.md`**: Plugin architecture documentation

#### Interface Layer (`src/interfaces/`)
- **`mod.rs`**: Module exports
- **`cli/`**: Command-line interface implementation
  - **`app.rs`**: CLI application structure
  - **`commands.rs`**: Command implementations
  - **`error.rs`**: CLI-specific error handling
  - **`output.rs`**: Output formatting and display
  - **`DESIGN.md`**: CLI design documentation
- **`tui.rs`**: Terminal user interface
- **`mcp.rs`**: Model Context Protocol server
- **`mcp_test.rs`**: MCP server tests

## Examples Directory (`examples/`)

### Workflow Examples
- **`hello-world.yaml`**: Basic workflow example
- **`simple-workflow.yaml`**: Simple multi-step workflow
- **`batch-workflows.yaml`**: Batch processing example
- **`test-workflow.yaml`**: Test workflow definition
- **`test-workflow.json`**: JSON format workflow

### Plugin Examples
- **`tools_example.rs`**: Tool system usage example
- **`python_plugin_example.rs`**: Python plugin integration
- **`nodejs_plugin_example.rs`**: Node.js plugin integration
- **`docker_plugin_example.rs`**: Docker plugin usage
- **`docker_dockerfile_example.rs`**: Dockerfile-based plugin
- **`wasm_plugin_example.rs`**: WebAssembly plugin example
- **`native_plugin_example.rs`**: Native Rust plugin example
- **`config_priority_example.rs`**: Configuration precedence demo

### Tool Directories
- **`python_tools/`**: Python tool implementations
  - **`simple_calculator.py`**: Basic calculator tool
  - **`data_processor.py`**: Data processing tool
  - **`requirements.txt`**: Python dependencies
- **`nodejs_tools/`**: Node.js tool implementations
  - **`simple_calculator.js`**: JavaScript calculator
  - **`data_processor.js`**: Data processing tool
  - **`simple_test.js`**: Test tool
  - **`package.json`**: Node.js dependencies
- **`docker_tools/`**: Docker-based tools
  - **`Dockerfile`**: Container definition
  - **`simple_processor.sh`**: Shell script processor
- **`wasm_tools/`**: WebAssembly tools
  - **`simple_calculator.wat`**: WASM calculator
  - **`text_processor.js`**: JavaScript WASM wrapper
  - **`README.md`**: WASM tools documentation

## Configuration Structure

### Default Configuration (`config/default.toml`)
Hierarchical TOML configuration with sections for:
- Server settings (HTTP/WebSocket ports)
- Storage configuration (database path, cache settings)
- Logging configuration (level, format, output)
- Plugin settings (directory, auto-load, sandboxing)
- Authentication and rate limiting
- Workflow execution parameters

## Naming Conventions

### Files and Directories
- Use snake_case for file and directory names
- Use descriptive names that indicate purpose
- Group related functionality in subdirectories
- Include `DESIGN.md` files for complex modules

### Rust Code Conventions
- **Types**: PascalCase (e.g., `WorkflowDefinition`, `ExecutionStatus`)
- **Functions/Variables**: snake_case (e.g., `execute_workflow`, `node_id`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `DEFAULT_TIMEOUT`)
- **Modules**: snake_case (e.g., `workflow`, `storage`)
- **Traits**: PascalCase with descriptive names (e.g., `ToolNode`, `StorageBackend`)

### Configuration Keys
- Use snake_case for TOML configuration keys
- Group related settings in sections
- Use descriptive names that indicate purpose
- Include units in names where applicable (e.g., `timeout_seconds`)

## Module Dependencies

### Dependency Flow
```
interfaces/ → workflow/ → tools/ → plugins/
     ↓           ↓         ↓         ↓
   core/ ← storage/ ← config/ ← error/
```

### Import Patterns
- Re-export commonly used types in `lib.rs`
- Use relative imports within modules
- Import from `crate::` for cross-module dependencies
- Prefer trait imports over concrete types at module boundaries

## Testing Organization

### Test Location
- Unit tests: In same file as implementation using `#[cfg(test)]`
- Integration tests: Separate files within modules (e.g., `tests.rs`)
- Property tests: Dedicated test files (e.g., `retry_tests.rs`)
- Regression tests: `proptest-regressions/` directory

### Test Naming
- Test functions: `test_` prefix with descriptive names
- Test modules: `tests` or specific feature names
- Property tests: `property_` prefix for proptest functions