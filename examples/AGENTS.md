# examples/ - Comprehensive Examples

## OVERVIEW
23 examples demonstrating all features: workflows, plugins, configurations, and patterns.

## EXAMPLE CATEGORIES

### Workflow Examples
- **comprehensive_workflow_example.rs**: Full workflow lifecycle
- **async_execution_example.rs**: Async patterns and parallel execution
- **parameter_template_example.rs**: Template-based parameter expansion
- **system_recovery_example.rs**: Checkpoint and recovery

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
```

## TEMPLATES
The `templates/` directory contains comprehensive workflow templates and documentation:
- **interactive-classification-workflow.yaml**: Human review with AI classification
- **interactive-batch-processing-workflow.yaml**: Bulk operations with progress tracking
- **interactive-merge-workflow.yaml**: Folder merging with conflict resolution
- **workflow-composition-examples.yaml**: Complex patterns
- **environment-config-examples.yaml**: Config management
- **CONFIGURATION_GUIDE.md**: Complete configuration reference
- **HUMAN_DECISION_BEST_PRACTICES.md**: Interactive workflow guidelines
- **USAGE_EXAMPLES.md**: Step-by-step tutorials

See `templates/AGENTS.md` for detailed documentation.

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

<!-- AUTO-GENERATED-AGENT-MAP:START -->
## 🗺️ Agent Map & Directory Structure

> **Auto-generated** on 2026-01-17 20:44:16

- **docker_tools/**: Empty or asset-only directory.
- **nodejs_tools/**: Empty or asset-only directory.
- **python_tools/**: Contains 2 files (e.g., data_processor.py, simple_calculator.py).
- **[templates/](templates/AGENTS.md)**: Production-ready workflow templates with comprehensive documentation and interactive examples.
- **wasm_tools/**: Contains 1 files (e.g., README.md).

<!-- AUTO-GENERATED-AGENT-MAP:END -->
