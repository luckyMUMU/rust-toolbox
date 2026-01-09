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
The `templates/` directory contains:
- **interactive-classification-workflow.yaml**: Human review
- **batch-processing-workflow.yaml**: Bulk operations
- **workflow-composition-examples.yaml**: Complex patterns
- **environment-config-examples.yaml**: Config management

## WHAT THEY DEMONSTRATE
- **DAG execution**: Dependencies, parallelism
- **Error handling**: Retry, recovery, cleanup
- **State management**: Persistence, checkpointing
- **Plugin systems**: Multi-language integration
- **Tool composition**: Reusable components
- **Configuration**: Hierarchical, hot reload
- **Interfaces**: CLI, TUI, server modes
- **Performance**: Caching, concurrency
