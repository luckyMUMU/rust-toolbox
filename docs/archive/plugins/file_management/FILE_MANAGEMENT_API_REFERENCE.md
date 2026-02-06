# File Management Tools API Reference

This document provides comprehensive API reference documentation for all File Management Tools, including detailed parameter specifications, return values, error handling, and integration examples.

## Table of Contents

1. [Plugin Architecture](#plugin-architecture)
2. [Core Tools API](#core-tools-api)
3. [Utility Tools API](#utility-tools-api)
4. [Data Structures](#data-structures)
5. [Error Handling](#error-handling)
6. [Integration Examples](#integration-examples)
7. [Performance Considerations](#performance-considerations)

## Plugin Architecture

### FileManagementPlugin

The main plugin class that registers all file management tools with the workflow-toolkit.

```rust
pub struct FileManagementPlugin {
    tools: Vec<Box<dyn ToolNode>>,
    config: FileManagementConfig,
}

impl Plugin for FileManagementPlugin {
    fn name(&self) -> &str { "file-management" }
    fn version(&self) -> &str { "1.0.0" }
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    fn get_tools(&self) -> Vec<Box<dyn ToolNode>>;
    fn shutdown(&mut self) -> Result<()>;
}
```

### FileManagementConfig

Plugin configuration structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManagementConfig {
    pub max_threads: usize,
    pub temp_directory: PathBuf,
    pub default_encoding: String,
    pub enable_chinese_processing: bool,
    pub memory_limit: Option<String>,
    pub streaming_mode: bool,
}
```

## Workflow Integration & Execution Mechanism

This section explains how workflow definitions (YAML) interact with the Rust implementation of the File Management plugin.

### Tool Registration Mapping

When a workflow specifies a `tool_name` for a `Tool` node, the engine retrieves the corresponding implementation from the `FileManagementToolRegistry`.

| YAML `tool_name` | Rust Executor Implementation | Implementation File |
| :--- | :--- | :--- |
| `folder-classifier` | `ClassificationTool` | [classification_tool.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_tool.rs) |
| `file-mover` | `FileMoverExecutor` | [registry.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/registry.rs) |
| `batch-processor` | `BatchProcessorTool` | [batch_processor_tool.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/batch_processor_tool.rs) |
| `human-decision` | `HumanDecisionTool` | [human_decision_tool.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/human_decision_tool.rs) |
| `folder-merger` | `FolderMergerExecutor` | [registry.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/registry.rs) |
| `text-processor` | `TextProcessorTool` | [text_processor_tool.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/text_processor_tool.rs) |
| `ac-matcher` | `AcMatcherExecutor` | [registry.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/registry.rs) |
| `result-reviewer` | `ResultReviewTool` | [result_review_tool.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/result_review_tool.rs) |
| `rule-loader` | `RuleLoaderTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `rule-preprocessor` | `RulePreprocessorTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `ac-builder` | `AhoCorasickBuilderTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `directory-scanner` | `DirectoryScannerTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `folder-preprocessor` | `FolderPreprocessorTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `parallel-matcher` | `ParallelMatcherTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `score-calculator` | `ScoreCalculatorTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `ambiguity-detector` | `AmbiguityDetectorTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `result-merger` | `ResultMergerTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `experimental-checker` | `ExperimentalCheckerTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |
| `report-generator` | `ReportGeneratorTool` | [classification_flow.rs](file:///d:/Code/AI/rust-tool-v2/src/plugins/file_management/classification_flow.rs) |

### Execution Flow

1. **Registry Lookup**: The workflow engine uses the `tool_name` as a key to find the registered `Arc<dyn ToolNode>` in the plugin's registry.
2. **Parameter Validation**: Before execution, the engine calls `tool.validate_parameters(&params)`. This ensures the YAML parameters match the expected schema defined in Rust.
3. **Context Injection**: An `ExecutionContext` is created, containing workflow-wide metadata and state.
4. **Asynchronous Execution**: The engine calls `tool.execute(params, context).await`.
5. **Result Processing**: The JSON output from the Rust executor is returned to the workflow engine and stored in the node's state for use by subsequent nodes.

### Placeholder Implementation

For tools that are part of the workflow specification but not yet fully implemented in the core plugin, a `PlaceholderExecutor` is used. This allows workflow validation to pass while providing a clear "Not Implemented" response during runtime.

## Core Tools API

### 1. Folder Classifier Tool

**Tool Name:** `folder-classifier`  
**Version:** `1.0.0`  
**Description:** Intelligent folder classification using configurable rules

#### Parameters Schema

```json
{
  "type": "object",
  "properties": {
    "folder_path": {
      "type": "string",
      "description": "Path to folder to classify"
    },
    "classification_rules": {
      "description": "Classification rules (JSON object or file path)"
    },
    "enable_user_interaction": {
      "type": "boolean",
      "default": false,
      "description": "Enable human decision-making"
    },
    "experimental_mode": {
      "type": "boolean", 
      "default": false,
      "description": "Run in simulation mode"
    },
    "confidence_threshold": {
      "type": "number",
      "default": 0.8,
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Minimum confidence for auto-classification"
    },
    "batch_size": {
      "type": "integer",
      "default": 10,
      "minimum": 1,
      "description": "Number of folders to process in batch"
    }
  },
  "required": ["folder_path", "classification_rules"]
}
```

#### Return Schema

```json
{
  "type": "object",
  "properties": {
    "status": {
      "type": "string",
      "enum": ["classified", "unclassified", "pending", "error"],
      "description": "Classification status"
    },
    "category": {
      "type": "string",
      "description": "Assigned category (if classified)"
    },
    "candidates": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "category": {"type": "string"},
          "score": {"type": "number"},
          "reasoning": {"type": "string"}
        }
      },
      "description": "All category candidates with scores"
    },
    "score": {
      "type": "number",
      "description": "Confidence score for classification"
    },
    "folder_name": {
      "type": "string",
      "description": "Name of the classified folder"
    },
    "processing_time_ms": {
      "type": "number",
      "description": "Time taken for classification"
    },
    "operations": {
      "type": "array",
      "description": "Planned file operations (if not experimental)"
    }
  }
}
```

#### Usage Example

```rust
use workflow_toolkit::tools::ToolNode;
use serde_json::json;

let params = json!({
    "folder_path": "/home/user/Documents/Project Files",
    "classification_rules": {
        "categories": {
            "projects": {
                "keywords": [
                    {"pattern": "project", "weight": 1.0}
                ],
                "target_directory": "Projects"
            }
        }
    },
    "confidence_threshold": 0.8,
    "experimental_mode": true
});

let result = classifier_tool.execute(params, context).await?;
```

### 2. File Mover Tool

**Tool Name:** `file-mover`  
**Version:** `1.0.0`  
**Description:** Safe file and folder operations with conflict resolution

#### Parameters Schema

```json
{
  "type": "object",
  "properties": {
    "operations": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "source": {"type": "string"},
          "destination": {"type": "string"},
          "operation_type": {
            "type": "string",
            "enum": ["Move", "Copy", "Link"],
            "default": "Move"
          }
        },
        "required": ["source", "destination"]
      },
      "description": "List of file operations to perform"
    },
    "conflict_resolution": {
      "type": "string",
      "enum": ["Skip", "Overwrite", "Rename", "Fail"],
      "default": "Rename",
      "description": "How to handle file conflicts"
    },
    "check_disk_space": {
      "type": "boolean",
      "default": true,
      "description": "Check available disk space before operations"
    },
    "create_directories": {
      "type": "boolean",
      "default": true,
      "description": "Create target directories if they don't exist"
    },
    "verify_operations": {
      "type": "boolean",
      "default": false,
      "description": "Verify operations after completion"
    }
  },
  "required": ["operations"]
}
```

#### Return Schema

```json
{
  "type": "object",
  "properties": {
    "operations_completed": {
      "type": "number",
      "description": "Number of successful operations"
    },
    "operations_failed": {
      "type": "number", 
      "description": "Number of failed operations"
    },
    "operations_skipped": {
      "type": "number",
      "description": "Number of skipped operations"
    },
    "total_bytes_moved": {
      "type": "number",
      "description": "Total bytes transferred"
    },
    "duration_ms": {
      "type": "number",
      "description": "Total operation duration"
    },
    "errors": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "operation": {"type": "object"},
          "error": {"type": "string"},
          "error_code": {"type": "string"}
        }
      },
      "description": "Detailed error information"
    }
  }
}
```

### 3. Folder Merger Tool

**Tool Name:** `folder-merger`  
**Version:** `1.0.0`  
**Description:** Intelligent folder merging with duplicate handling

#### Parameters Schema

```json
{
  "type": "object",
  "properties": {
    "source_directories": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Directories to scan for mergeable folders"
    },
    "target_directory": {
      "type": "string",
      "description": "Target directory for merged folders"
    },
    "merge_strategy": {
      "type": "string",
      "enum": ["SmallerToLarger", "LargerToSmaller", "UserDecision", "TargetDirectory"],
      "default": "UserDecision",
      "description": "Strategy for merging folders"
    },
    "duplicate_handling": {
      "type": "string",
      "enum": ["Skip", "Rename", "Overwrite", "UserDecision"],
      "default": "UserDecision",
      "description": "How to handle duplicate files"
    },
    "minimum_folder_size": {
      "type": "integer",
      "default": 0,
      "description": "Minimum folder size in bytes to consider"
    },
    "max_merge_depth": {
      "type": "integer",
      "default": 3,
      "description": "Maximum directory depth for recursive merging"
    },
    "enable_size_analysis": {
      "type": "boolean",
      "default": true,
      "description": "Perform detailed size analysis"
    },
    "backup_before_merge": {
      "type": "boolean",
      "default": false,
      "description": "Create backup before merging"
    }
  },
  "required": ["source_directories"]
}
```

#### Return Schema

```json
{
  "type": "object",
  "properties": {
    "merge_groups": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "folder_name": {"type": "string"},
          "locations": {"type": "array"},
          "total_size": {"type": "number"},
          "merge_strategy": {"type": "string"},
          "conflicts": {"type": "array"}
        }
      },
      "description": "Groups of folders to merge"
    },
    "space_savings": {
      "type": "number",
      "description": "Estimated space savings in bytes"
    },
    "conflicts_detected": {
      "type": "number",
      "description": "Number of conflicts detected"
    },
    "merge_operations": {
      "type": "array",
      "description": "Planned merge operations"
    }
  }
}
```

### 4. Batch Processor Tool

**Tool Name:** `batch-processor`  
**Version:** `1.0.0`  
**Description:** Process multiple items in parallel workflows

#### Parameters Schema

```json
{
  "type": "object",
  "properties": {
    "source_directory": {
      "type": "string",
      "description": "Directory containing items to process"
    },
    "target_directory": {
      "type": "string",
      "description": "Target directory for processed items"
    },
    "operation_type": {
      "type": "string",
      "enum": ["move", "copy", "classify", "merge", "custom"],
      "default": "move",
      "description": "Type of operation to perform"
    },
    "operation_config": {
      "type": "object",
      "default": {},
      "description": "Configuration specific to operation type"
    },
    "batch_size": {
      "type": "integer",
      "default": 20,
      "minimum": 1,
      "description": "Number of items per batch"
    },
    "max_concurrent_batches": {
      "type": "integer",
      "default": 3,
      "minimum": 1,
      "description": "Maximum concurrent batches"
    },
    "filter_criteria": {
      "type": "object",
      "properties": {
        "include_hidden": {"type": "boolean", "default": false},
        "min_size_bytes": {"type": "integer", "default": 0},
        "max_size_bytes": {"type": "integer"},
        "file_extensions": {"type": "array", "items": {"type": "string"}},
        "exclude_patterns": {"type": "array", "items": {"type": "string"}}
      },
      "description": "Criteria for filtering items"
    }
  },
  "required": ["source_directory", "target_directory"]
}
```

#### Return Schema

```json
{
  "type": "object",
  "properties": {
    "batches_completed": {
      "type": "number",
      "description": "Number of completed batches"
    },
    "batches_failed": {
      "type": "number",
      "description": "Number of failed batches"
    },
    "total_items_processed": {
      "type": "number",
      "description": "Total items processed"
    },
    "processing_time_ms": {
      "type": "number",
      "description": "Total processing time"
    },
    "throughput": {
      "type": "number",
      "description": "Items processed per second"
    },
    "error_summary": {
      "type": "object",
      "description": "Summary of errors encountered"
    }
  }
}
```

### 5. Human Decision Tool

**Tool Name:** `human-decision`  
**Version:** `1.1.0`  
**Description:** Interactive decision-making for ambiguous scenarios, supporting both single and batch decisions.

#### Parameters Schema

```json
{
  "type": "object",
  "properties": {
    "decision_type": {
      "type": "string",
      "enum": ["Classification", "FileConflict", "MergeStrategy", "Custom"],
      "description": "Type of decision required"
    },
    "context": {
      "type": "object",
      "properties": {
        "title": {"type": "string"},
        "description": {"type": "string"},
        "folder_name": {"type": "string"},
        "metadata": {"type": "object"}
      },
      "required": ["title", "description"],
      "description": "Context information for the decision"
    },
    "options": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": {"type": "string"},
          "label": {"type": "string"},
          "description": {"type": "string"},
          "score": {"type": "number"},
          "recommended": {"type": "boolean", "default": false}
        },
        "required": ["id", "label"]
      },
      "description": "Available decision options (for single decision)"
    },
    "items": {
      "type": "array",
      "description": "List of items for batch decisions (mutually exclusive with 'options' in simple mode)",
      "items": {
        "type": "object",
        "description": "Arbitrary item data to be presented for decision"
      }
    },
    "experimental_mode": {
      "type": "boolean",
      "description": "Run in simulation/experimental mode (auto-selects recommended or default option)"
    },
    "timeout_seconds": {
      "type": "number",
      "description": "Decision timeout in seconds"
    },
    "default_choice": {
      "type": "number",
      "description": "Default option index if timeout occurs"
    }
  }
}
```

#### Return Schema

```json
{
  "type": "object",
  "properties": {
    "selected_option": {
      "type": "string",
      "description": "ID of selected option"
    },
    "decision_time_ms": {
      "type": "number",
      "description": "Time taken for decision"
    },
    "was_timeout": {
      "type": "boolean",
      "description": "Whether decision timed out"
    },
    "user_input": {
      "type": "string",
      "description": "Additional user input"
    },
    "folder_path": {
      "type": "string",
      "description": "Path of the folder being decided on (if applicable)"
    },
    "selected_category": {
      "type": "string",
      "description": "Selected category ID (alias for selected_option)"
    },
    "decisions": {
      "type": "array",
      "description": "List of decision results if batch processing ('items') was used",
      "items": {
        "type": "object",
        "description": "Individual decision result"
      }
    }
  }
}
```

## Granular Classification Tools API

The following tools are low-level components used to build the classification workflow.

### 8. Rule Loader Tool

**Tool Name:** `rule-loader`
**Description:** Loads classification rules from a file or object, supporting legacy formats.

#### Parameters
*   `rules`: (Required) Rule object or file path.

#### Returns
*   Valid `ClassificationRules` object.

### 9. Aho-Corasick Builder Tool

**Tool Name:** `ac-builder`
**Description:** Builds an Aho-Corasick automaton from a set of patterns.

#### Parameters
*   `patterns`: (Required) List of patterns to match.

#### Returns
*   `automaton_id`: Reference ID to the built automaton (or serialized representation).

### 10. Parallel Matcher Tool

**Tool Name:** `parallel-matcher`
**Description:** Executes parallel keyword matching using a pre-built automaton.

#### Parameters
*   `automaton_config`: (Required) Configuration/ID of the AC automaton.
*   `folders`: (Required) List of folders to match against.

#### Returns
*   `match_results`: Raw matching results for each folder.

### 11. Ambiguity Detector Tool

**Tool Name:** `ambiguity-detector`
**Description:** Analyzes scored results to identify ambiguous or unclassified items.

#### Parameters
*   `scored_results`: (Required) List of folders with calculated scores.
*   `confidence_threshold`: (Optional) Minimum score to be considered classified.

#### Returns
*   `classified`: List of successfully classified items.
*   `ambiguous`: List of items requiring human review.
*   `unclassified`: List of items with no matches.

## Utility Tools API

### 6. Text Processor Tool

**Tool Name:** `text-processor`  
**Version:** `1.0.0`  
**Description:** Text processing including Chinese and pinyin conversion

#### Parameters Schema

```json
{
  "type": "object",
  "properties": {
    "text": {
      "type": "string",
      "description": "Text to process"
    },
    "operations": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": ["NormalizeCase", "RemoveSpaces", "ConvertTraditional", "GeneratePinyin", "CreateCombinations"]
      },
      "description": "Processing operations to apply"
    },
    "chinese_processing": {
      "type": "object",
      "properties": {
        "pinyin_style": {
          "type": "string",
          "enum": ["Normal", "WithTone", "WithoutTone", "FirstLetter"],
          "default": "Normal"
        },
        "generate_combinations": {"type": "boolean", "default": false},
        "include_tones": {"type": "boolean", "default": false}
      },
      "description": "Chinese-specific processing options"
    }
  },
  "required": ["text", "operations"]
}
```

#### Return Schema

```json
{
  "type": "object",
  "properties": {
    "original": {"type": "string"},
    "processed": {"type": "string"},
    "pinyin_variants": {"type": "array", "items": {"type": "string"}},
    "combinations": {"type": "array"},
    "metadata": {"type": "object"}
  }
}
```

### 7. AC Matcher Tool

**Tool Name:** `ac-matcher`  
**Version:** `1.0.0`  
**Description:** Aho-Corasick multi-pattern string matching

#### Parameters Schema

```json
{
  "type": "object",
  "properties": {
    "text": {
      "type": "string",
      "description": "Text to search in"
    },
    "patterns": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "pattern": {"type": "string"},
          "category": {"type": "string"},
          "score": {"type": "number", "default": 1.0}
        },
        "required": ["pattern", "category"]
      },
      "description": "Patterns to search for"
    },
    "case_sensitive": {
      "type": "boolean",
      "default": false,
      "description": "Case-sensitive matching"
    },
    "find_overlapping": {
      "type": "boolean",
      "default": false,
      "description": "Find overlapping matches"
    }
  },
  "required": ["text", "patterns"]
}
```

#### Return Schema

```json
{
  "type": "object",
  "properties": {
    "matches": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "pattern": {"type": "string"},
          "category": {"type": "string"},
          "score": {"type": "number"},
          "start_pos": {"type": "number"},
          "end_pos": {"type": "number"}
        }
      }
    },
    "total_matches": {"type": "number"},
    "categories_found": {"type": "array", "items": {"type": "string"}}
  }
}
```

## Data Structures

### Classification Rules

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRules {
    pub version: String,
    pub metadata: Option<RuleMetadata>,
    pub categories: HashMap<String, Category>,
    pub settings: GlobalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub description: Option<String>,
    pub keywords: Vec<Keyword>,
    pub target_directory: String,
    pub confidence_threshold: Option<f64>,
    pub priority: Option<i32>,
    pub conditions: Option<CategoryConditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub pattern: String,
    pub weight: f64,
    pub case_sensitive: Option<bool>,
    pub match_type: Option<MatchType>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Substring,
    Regex,
    Exact,
    Fuzzy,
}
```

### File Operations

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub operation_type: OperationType,
    pub metadata: Option<OperationMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Move,
    Copy,
    Link,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub operation: FileOperation,
    pub status: OperationStatus,
    pub bytes_transferred: u64,
    pub duration: Duration,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Success,
    Failed,
    Skipped,
    Cancelled,
}
```

### Decision Context

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub title: String,
    pub description: String,
    pub folder_name: Option<String>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub score: Option<f64>,
    pub recommended: bool,
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    pub selected_option: String,
    pub decision_time: Duration,
    pub was_timeout: bool,
    pub user_input: Option<String>,
    pub context: DecisionContext,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum FileManagementError {
    #[error("Classification error: {message}")]
    Classification { message: String },
    
    #[error("File operation error: {operation} failed - {error}")]
    FileOperation { operation: String, error: String },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Human decision timeout: {timeout_seconds}s")]
    DecisionTimeout { timeout_seconds: u64 },
    
    #[error("Insufficient disk space: need {required} bytes, have {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    
    #[error("Path not found: {path}")]
    PathNotFound { path: String },
    
    #[error("Invalid rules: {message}")]
    InvalidRules { message: String },
    
    #[error("Text processing error: {message}")]
    TextProcessing { message: String },
    
    #[error("Batch processing error: batch {batch_id} failed - {error}")]
    BatchProcessing { batch_id: String, error: String },
}
```

### Error Recovery

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryConfig {
    pub auto_retry: bool,
    pub max_retry_attempts: u32,
    pub retry_delay: Duration,
    pub backoff_strategy: BackoffStrategy,
    pub recoverable_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed,
    Linear,
    Exponential,
}

impl FileManagementError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            FileManagementError::DecisionTimeout { .. } => true,
            FileManagementError::FileOperation { .. } => true,
            FileManagementError::InsufficientSpace { .. } => false,
            FileManagementError::PermissionDenied { .. } => false,
            _ => false,
        }
    }
    
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            FileManagementError::DecisionTimeout { .. } => vec![
                "Increase decision timeout".to_string(),
                "Enable default choices".to_string(),
                "Use batch decision mode".to_string(),
            ],
            FileManagementError::InsufficientSpace { required, available } => vec![
                format!("Free up {} bytes of disk space", required - available),
                "Choose a different target directory".to_string(),
                "Enable compression".to_string(),
            ],
            _ => vec![],
        }
    }
}
```

## Integration Examples

### Basic Tool Usage

```rust
use workflow_toolkit::tools::ToolNode;
use workflow_toolkit::ExecutionContext;
use serde_json::json;

async fn classify_folder_example() -> Result<(), Box<dyn std::error::Error>> {
    // Get the classifier tool from registry
    let tool_registry = get_tool_registry().await?;
    let classifier = tool_registry.get_tool("folder-classifier")?;
    
    // Prepare parameters
    let params = json!({
        "folder_path": "/home/user/Documents/Project Files",
        "classification_rules": {
            "categories": {
                "projects": {
                    "keywords": [
                        {"pattern": "project", "weight": 1.0},
                        {"pattern": "work", "weight": 0.8}
                    ],
                    "target_directory": "Projects"
                }
            }
        },
        "confidence_threshold": 0.8,
        "experimental_mode": true
    });
    
    // Execute classification
    let context = ExecutionContext::new();
    let result = classifier.execute(params, context).await?;
    
    println!("Classification result: {}", result);
    Ok(())
}
```

### Workflow Integration

```rust
use workflow_toolkit::{WorkflowDefinition, WorkflowNode, NodeType};
use serde_json::json;

fn create_classification_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        name: "folder-classification".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Intelligent folder classification workflow".to_string()),
        nodes: vec![
            WorkflowNode {
                id: "classify".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("folder-classifier".to_string()),
                parameters: json!({
                    "folder_path": "${input.folder_path}",
                    "classification_rules": "${input.rules}",
                    "experimental_mode": true
                }),
                // ... other node properties
            },
            WorkflowNode {
                id: "human_review".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("human-decision".to_string()),
                parameters: json!({
                    "decision_type": "Classification",
                    "context": {
                        "title": "Review Classification Results",
                        "description": "Please review ambiguous classifications"
                    },
                    "options": "${classify.candidates}"
                }),
                // ... other node properties
            },
            WorkflowNode {
                id: "execute_moves".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("file-mover".to_string()),
                parameters: json!({
                    "operations": "${human_review.final_operations}",
                    "conflict_resolution": "Rename"
                }),
                // ... other node properties
            },
        ],
        // ... other workflow properties
    }
}
```

### Custom Tool Development

```rust
use workflow_toolkit::tools::{ToolNode, ToolDefinition};
use workflow_toolkit::ExecutionContext;
use async_trait::async_trait;
use serde_json::Value;

pub struct CustomFileProcessor {
    name: String,
    config: ProcessorConfig,
}

#[async_trait]
impl ToolNode for CustomFileProcessor {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        // Parse parameters
        let folder_path: String = params["folder_path"]
            .as_str()
            .ok_or("Missing folder_path parameter")?
            .to_string();
        
        // Custom processing logic
        let result = self.process_folder(&folder_path).await?;
        
        // Return structured result
        Ok(serde_json::to_value(result)?)
    }
    
    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Validate required parameters
        if !params.get("folder_path").and_then(|v| v.as_str()).is_some() {
            return Err(anyhow!("Missing required parameter: folder_path"));
        }
        Ok(())
    }
    
    fn get_schema(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            description: "Custom file processor".to_string(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "folder_path": {"type": "string"}
                },
                "required": ["folder_path"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "processed_files": {"type": "number"},
                    "processing_time": {"type": "number"}
                }
            }),
            dependencies: vec![],
            metadata: HashMap::new(),
            plugin_info: None,
        }
    }
}

impl CustomFileProcessor {
    async fn process_folder(&self, path: &str) -> Result<ProcessingResult> {
        // Implementation details...
        Ok(ProcessingResult {
            processed_files: 42,
            processing_time: 1500,
        })
    }
}
```

## Performance Considerations

### Memory Management

```rust
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub max_memory_usage: Option<usize>,
    pub streaming_mode: bool,
    pub batch_size_optimization: bool,
    pub cache_strategy: CacheStrategy,
}

#[derive(Debug, Clone)]
pub enum CacheStrategy {
    None,
    LRU { capacity: usize },
    TTL { duration: Duration },
    Adaptive,
}
```

### Concurrency Control

```rust
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    pub max_concurrent_operations: usize,
    pub thread_pool_size: Option<usize>,
    pub async_io: bool,
    pub backpressure_threshold: usize,
}
```

### Monitoring and Metrics

```rust
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operations_per_second: f64,
    pub average_operation_time: Duration,
    pub memory_usage: usize,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
}

pub trait MetricsCollector {
    fn record_operation(&self, operation: &str, duration: Duration, success: bool);
    fn record_memory_usage(&self, bytes: usize);
    fn record_cache_access(&self, hit: bool);
    fn get_metrics(&self) -> PerformanceMetrics;
}
```

### Optimization Guidelines

1. **Batch Size Tuning**
   - Start with small batches (5-10 items) for interactive use
   - Increase to 25-50 items for automated processing
   - Use 100+ items for high-throughput scenarios

2. **Memory Usage**
   - Enable streaming mode for large datasets
   - Set appropriate memory limits
   - Use lazy loading for metadata

3. **Concurrency**
   - Limit concurrent operations based on system resources
   - Use async I/O for better resource utilization
   - Monitor system load and adjust dynamically

4. **Caching**
   - Cache frequently accessed classification rules
   - Use LRU cache for metadata
   - Implement TTL for temporary data

---

*This API reference is continuously updated. For the latest information and examples, please refer to the source code and integration tests.*