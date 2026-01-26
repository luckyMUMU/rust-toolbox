# examples/ - Comprehensive Examples

## OVERVIEW
23 examples demonstrating all features: workflows, plugins, configurations, and patterns. 19 files in examples directory.

## EXAMPLE CATEGORIES

### Workflow Examples
- **comprehensive_workflow_example.rs**: Full workflow lifecycle
- **async_execution_example.rs**: Async patterns and parallel execution
- **parameter_template_example.rs**: Template-based parameter expansion
- **system_recovery_example.rs**: Checkpoint and recovery
- **interactive_classification_example.rs**: Human-in-the-loop classification
- **interactive_merge_example.rs**: Folder merging with decisions
- **interactive_batch_processing_example.rs**: Batch operations with oversight
- **real_world_scenario_example.rs**: Creative agency asset management

### Plugin Examples
- **python_plugin_example.rs**: Python integration
- **nodejs_plugin_example.rs**: Node.js integration
- **docker_plugin_example.rs**: Docker container execution
- **native_plugin_example.rs**: Native Rust plugins
- **wasm_plugin_example.rs**: WebAssembly (disabled)
- **plugin_integration_example.rs**: Multi-plugin workflows

### File Management
- **file_management_example.rs**: Classification and batch ops

### Configuration
- **config_priority_example.rs**: Config hierarchy demonstration
- **backup_recovery_example.rs**: Backup and restore
- **audit_logging_example.rs**: Audit trail

### Interface Examples
- **tui_example.rs**: TUI usage
- **tools_example.rs**: Tool registry

### Templates
- **hello-world.yaml**: Simple workflow
- **batch-workflows.yaml**: Batch processing
- **test-workflow.yaml**: Test patterns

## RUNNING EXAMPLES
```bash
# Comprehensive workflow
cargo run --example comprehensive_workflow_example

# Plugin integration
cargo run --example plugin_integration_example

# Python plugin
cargo run --example python_plugin_example

# Configuration
cargo run --example config_priority_example

# Interactive examples
cargo run --example interactive_classification_example
cargo run --example interactive_merge_example
cargo run --example interactive_batch_processing_example
```

## TEMPLATES
The `templates/` directory contains comprehensive workflow templates and documentation (114 lines in AGENTS.md):

### Interactive Workflows
- **interactive-classification-workflow.yaml**: Human review with AI classification
- **interactive-batch-processing-workflow.yaml**: Bulk operations with progress tracking
- **interactive-merge-workflow.yaml**: Folder merging with conflict resolution

### Configuration Templates
- **environment-config-examples.yaml**: Config management
- **common-use-cases.yaml**: Real-world scenarios
- **workflow-composition-examples.yaml**: Complex patterns

### Documentation
- **CONFIGURATION_GUIDE.md**: Complete configuration reference
- **HUMAN_DECISION_BEST_PRACTICES.md**: Interactive workflow guidelines
- **PARAMETER_DOCUMENTATION.md**: Template syntax reference
- **USAGE_EXAMPLES.md**: Step-by-step tutorials
- **README.md**: Template overview

### Classification Rules
- `classification-rules-example.json`: Basic rules
- `classification-rules-comprehensive.json`: Advanced rules
- `classification-rules-chinese.json`: Chinese-specific rules

## PLUGIN IMPLEMENTATIONS
The `*_tools/` directories contain actual plugin implementations for testing:

### docker_tools/
- `Dockerfile`: Container definition
- `simple_processor.sh`: Shell script processor

### nodejs_tools/
- `simple_calculator.js`: Basic calculator
- `data_processor.js`: Data processing tool
- `text_processor.js`: Text analysis

### python_tools/
- `simple_calculator.py`: Basic calculator
- `data_processor.py`: Data processing tool

### wasm_tools/
- `simple_calculator.wat`: WebAssembly calculator
- `README.md`: WASM plugin documentation

## WHAT THEY DEMONSTRATE
- **DAG execution**: Dependencies, parallelism
- **Error handling**: Retry, recovery, cleanup
- **State management**: Persistence, checkpointing
- **Plugin systems**: Multi-language integration
- **Tool composition**: Reusable components
- **Configuration**: Hierarchical, hot reload
- **Interfaces**: CLI, TUI, server modes
- **Performance**: Caching, concurrency
- **Human decisions**: Interactive workflows
- **File operations**: Classification, batch, merge
- **Chinese processing**: Pinyin, traditional/simplified
- **Experimental mode**: Preview before execution

## EXAMPLE DETAILS

### Real-World Scenario Example
**File**: `real_world_scenario_example.rs`  
**Purpose**: Creative agency digital asset management system  
**Features**:
- Client project organization
- Media asset consolidation
- Deliverable preparation
- Human decision integration
- Batch processing with progress
- Error recovery and rollback

### Interactive Classification Example
**File**: `interactive_classification_example.rs`  
**Purpose**: Demonstrate AI-powered classification with human review  
**Features**:
- Experimental mode with preview
- Human decision points
- Confidence threshold configuration
- Batch processing
- Result verification

### Interactive Merge Example
**File**: `interactive_merge_example.rs`  
**Purpose**: Folder merging with multiple strategies  
**Features**:
- Multiple merge strategies
- Conflict resolution
- Human validation
- Backup creation
- Rollback capability

### Interactive Batch Processing Example
**File**: `interactive_batch_processing_example.rs`  
**Purpose**: Bulk file operations with oversight  
**Features**:
- Generic operation support
- Progress tracking
- Conflict resolution
- Human confirmation
- Error isolation

## TEMPLATE USAGE

### Basic Classification
```bash
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/folders" \
  --param output_directory="/path/to/organized" \
  --param classification_rules="examples/templates/classification-rules-example.json" \
  --param experimental_mode=true \
  --param enable_user_interaction=true
```

### Folder Merge
```bash
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/path/to/dir1", "/path/to/dir2"]' \
  --param target_directory="/path/to/merged" \
  --param merge_strategy="UserDecision" \
  --param experimental_mode=true
```

### Batch Processing
```bash
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/path/to/source" \
  --param target_directory="/path/to/target" \
  --param operation_type="move" \
  --param experimental_mode=true \
  --param enable_user_interaction=true
```

## PLUGIN USAGE

### Python Plugin
```bash
cargo run --example python_plugin_example
```

Demonstrates:
- Python script execution
- Parameter passing
- Result handling
- Error management

### Node.js Plugin
```bash
cargo run --example nodejs_plugin_example
```

Demonstrates:
- Node.js script execution
- JSON parameter handling
- Async execution
- Result parsing

### Docker Plugin
```bash
cargo run --example docker_plugin_example
```

Demonstrates:
- Container execution
- Isolated environments
- Resource limits
- Volume mounting

## CONFIGURATION EXAMPLES

### Config Priority
```bash
cargo run --example config_priority_example
```

Demonstrates:
- CLI arguments (highest)
- Environment variables
- Config file
- Built-in defaults (lowest)

### Backup and Recovery
```bash
cargo run --example backup_recovery_example
```

Demonstrates:
- Automated backups
- Checkpoint creation
- State recovery
- Rollback operations

### Audit Logging
```bash
cargo run --example audit_logging_example
```

Demonstrates:
- Comprehensive audit trail
- Compliance tracking
- Queryable logs
- Security events

## INTERFACE EXAMPLES

### TUI Usage
```bash
cargo run --example tui_example
```

Demonstrates:
- TUI initialization
- Widget usage
- Event handling
- State management

### Tool Registry
```bash
cargo run --example tools_example
```

Demonstrates:
- Tool registration
- Parameter validation
- Tool execution
- Version management

## BEST PRACTICES

### Running Examples
1. **Build First**: `cargo build --examples`
2. **Check Dependencies**: Ensure Python/Node.js/Docker available
3. **Use Test Data**: Create isolated test directories
4. **Start Simple**: Begin with basic examples
5. **Read Logs**: Enable debug logging for learning

### Learning Path
1. **Start**: `hello-world.yaml` - Basic workflow
2. **Progress**: `comprehensive_workflow_example.rs` - Full lifecycle
3. **Explore**: `interactive_classification_example.rs` - Human decisions
4. **Advanced**: `real_world_scenario_example.rs` - Complex scenarios
5. **Customize**: Modify templates for your use case

### Customization
1. **Copy Template**: Start with existing template
2. **Modify Rules**: Adjust classification rules
3. **Change Parameters**: Update workflow parameters
4. **Test Experimentally**: Use experimental mode first
5. **Deploy**: Run in production mode

## TESTING EXAMPLES

### Example Tests
```bash
# Test all examples compile
cargo check --examples

# Run specific example
cargo run --example comprehensive_workflow_example

# With logging
RUST_LOG=debug cargo run --example interactive_classification_example
```

### Example Validation
- **Compilation**: All examples must compile
- **Execution**: Examples should run to completion
- **Documentation**: Examples should be documented
- **Maintenance**: Examples must stay current

## ARCHITECTURE PATTERNS

### Workflow Composition
```yaml
# Multi-stage workflow
steps:
  - name: "stage1"
    tool: "tool-a"
    params: {...}
  
  - name: "stage2"
    tool: "tool-b"
    params: "{{ stage1.result }}"
    depends_on: ["stage1"]
  
  - name: "stage3"
    tool: "tool-c"
    params: "{{ stage2.result }}"
    depends_on: ["stage2"]
```

### Tool Composition
```rust
// Compose multiple tools
let composed = BasicTool::builder()
    .name("composed")
    .executor(Arc::new(AsyncFunctionExecutor::new(|params, ctx| async {
        let r1 = registry.execute_tool("tool1", params.clone()).await?;
        let r2 = registry.execute_tool("tool2", r1).await?;
        Ok(r2)
    })))
    .build()?;
```

### Error Recovery
```rust
// Retry with exponential backoff
let policy = RetryPolicy {
    max_attempts: 3,
    base_delay: Duration::from_secs(1),
    backoff_multiplier: 2.0,
    strategy: RetryStrategy::ExponentialBackoff,
};
```

## SEE ALSO

- [Templates AGENTS.md](templates/AGENTS.md) - Template documentation
- [Root AGENTS.md](../AGENTS.md) - Project overview
- [Workflow AGENTS.md](../src/workflow/AGENTS.md) - Workflow engine
- [File Management AGENTS.md](../src/plugins/file_management/AGENTS.md) - File operations

<!-- AUTO-GENERATED-AGENT-MAP:START -->
## 🗺️ Agent Map & Directory Structure

> **Auto-generated** on 2026-01-17 20:44:16

- **docker_tools/**: Empty or asset-only directory.
- **nodejs_tools/**: Empty or asset-only directory.
- **python_tools/**: Contains 2 files (e.g., data_processor.py, simple_calculator.py).
- **[templates/](templates/AGENTS.md)**: Production-ready workflow templates with comprehensive documentation and interactive examples.
- **wasm_tools/**: Contains 1 files (e.g., README.md).

<!-- AUTO-GENERATED-AGENT-MAP:END -->
