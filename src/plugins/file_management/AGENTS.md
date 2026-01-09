# src/plugins/file_management/ - File Operations Plugin

## OVERVIEW
Specialized tools for file classification, batch processing, and text operations with human review.

## TOOLS
| Tool | Purpose |
|------|---------|
| **Folder Classifier** | AI-powered folder categorization |
| **Batch Processor** | Bulk file operations with progress |
| **Text Processor** | Analysis and transformation |
| **Human Review** | Interactive decision support |

## CLASSIFICATION
- **AI-powered**: Uses pattern matching and content analysis
- **Confidence scoring**: 0.0-1.0 score per category
- **Candidates**: Multiple category suggestions
- **Experimental mode**: Advanced heuristics

## BATCH OPERATIONS
- **Progress tracking**: Real-time status updates
- **Error handling**: Per-file error isolation
- **Result confirmation**: Human validation before commit
- **Performance monitoring**: Built-in metrics

## HUMAN DECISION
- **Interactive prompts**: CLI/TUI for decisions
- **Override capability**: Accept/reject/suggest alternatives
- **Audit trail**: All decisions logged
- **Batch approval**: Approve multiple items at once

## TEXT PROCESSING
- **Extraction**: Parse structured data
- **Analysis**: Content classification
- **Transformation**: Format conversion
- **Summarization**: Content condensation

## USAGE
```rust
use workflow_toolkit::plugins::file_management::{FileManagementPlugin, FileManagementConfig};

let plugin = FileManagementPlugin::builder()
    .config(FileManagementConfig {
        directory: "./data".into(),
        rules_file: Some("classification-rules.json".into()),
        // ... config
    })
    .build()?;
```
