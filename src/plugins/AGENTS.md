# src/plugins/ - Multi-Language Plugin System

## OVERVIEW
Extensible plugin architecture supporting Native, Python, Node.js, Docker, and WASM plugins.

## PLUGIN TYPES
| Type | Runtime | Use Case |
|------|---------|----------|
| **Native** | libloading | High-performance Rust code |
| **Python** | Python subprocess | ML, data processing |
| **Node.js** | Node subprocess | JavaScript/TypeScript tools |
| **Docker** | bollard | Isolated, reproducible environments |
| **WASM** | wasmtime (disabled) | Sandboxed, portable code |

## RUNTIME SYSTEM
- **RuntimeManager**: Creates/cleans up runtime environments
- **PluginManager**: Loads plugins, manages lifecycle
- **Integration**: Unified interface via `IntegratedPluginSystem`

## SECURITY
- **Sandboxing**: Configurable via `plugins.sandbox_enabled`
- **Resource Limits**: Memory, timeout, CPU constraints
- **SecurityPolicy**: Defines allowed operations

## FILE MANAGEMENT SUBDIRECTORY
Specialized plugin for file operations:
- Interactive classification with AI
- Batch processing with progress tracking
- Text analysis and transformation
- Human-in-the-loop validation

## USAGE
```rust
use workflow_toolkit::{PluginManager, PluginConfig};

let manager = PluginManager::new();
let config = PluginConfig {
    name: "python_ml".to_string(),
    plugin_type: PluginType::Python,
    // ... config
};
manager.load_plugin(plugin, config)?;
```

## CURRENT LIMITATIONS
- WASM support temporarily disabled
- MCP server commented out (dependency issues)
- Plugin sandboxing in development
