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
**Path**: `src/plugins/file_management/` (20 files, 3,604+ lines in utils.rs)

Specialized plugin for file operations with comprehensive capabilities:

### Classification Tools
- **FolderClassifier**: AI-powered categorization with confidence scoring
- **ClassificationFlow**: Modular pipeline components (11 tools)
- **Pattern Matching**: Aho-Corasick algorithm for efficient keyword matching
- **Chinese Processing**: Pinyin conversion, traditional/simplified support

### Batch Processing
- **BatchProcessor**: Generic batch processing engine
- **BatchProcessorTool**: Tool-based batch operations
- **Progress Tracking**: Real-time updates with metrics
- **Error Isolation**: Per-item error handling

### Text Processing
- **TextProcessorTool**: Advanced text analysis
- **Operations**: Extract, analyze, transform, summarize
- **Chinese Support**: Pinyin, Unicode normalization, keyword extraction

### Human Decision
- **HumanDecisionTool**: Interactive decision support
- **BatchConfirmation**: Batch-level confirmation with summaries
- **ResultReview**: Preview and validate results
- **ResultConfirmation**: Final commit confirmation

### Utilities
- **FolderMerger**: Intelligent folder merging (4 strategies)
- **FileOperationManager**: Core file operations
- **PathUtils**: Path manipulation and validation
- **TextProcessor**: Text analysis and transformation

### Error Handling
- **FileManagementError**: Comprehensive error types (1,284 lines)
- **ErrorRecovery**: Recovery strategies with confidence scoring

### Monitoring
- **FileManagementMonitor**: Real-time operation monitoring
- **Performance**: Optimization and caching
- **ProgressTracker**: Batch progress tracking

**See**: [file_management/AGENTS.md](file_management/AGENTS.md) for detailed documentation

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

<!-- AUTO-GENERATED-AGENT-MAP:START -->
## 🗺️ Agent Map & Directory Structure

> **Auto-generated** on 2026-01-17 20:44:16

- **[file_management/](file_management/AGENTS.md)**: Specialized tools for file classification, batch processing, and text operations with human review.

<!-- AUTO-GENERATED-AGENT-MAP:END -->
