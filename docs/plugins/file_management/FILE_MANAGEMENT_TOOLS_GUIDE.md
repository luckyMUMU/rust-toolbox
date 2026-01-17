# File Management Tools User Guide

This comprehensive guide covers the File Management Tools plugin for the workflow-toolkit system, providing intelligent file and folder management capabilities through reusable workflow components.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Tool Reference](#tool-reference)
4. [Workflow Templates](#workflow-templates)
5. [Configuration Guide](#configuration-guide)
6. [Human Decision Integration](#human-decision-integration)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)
9. [Advanced Usage](#advanced-usage)

## Overview

The File Management Tools plugin provides a comprehensive suite of tools for intelligent file and folder organization, including:

- **Intelligent Classification**: Categorize folders based on configurable rules with Chinese text support
- **Batch Processing**: Process multiple files/folders in parallel with progress tracking
- **Folder Merging**: Intelligently merge duplicate folders with conflict resolution
- **Human Decision Integration**: Interactive decision-making for ambiguous scenarios
- **Experimental Mode**: Preview operations before execution with detailed reporting
- **Text Processing**: Advanced text normalization and Chinese/pinyin conversion
- **Pattern Matching**: Efficient multi-pattern string matching using Aho-Corasick automaton

### Key Features

✅ **Workflow Integration**: Seamlessly integrates with workflow-toolkit's plugin system  
✅ **Human-in-the-Loop**: Interactive decision-making for complex scenarios  
✅ **Experimental Mode**: Safe preview and confirmation before execution  
✅ **Chinese Text Support**: Full Unicode support with pinyin conversion  
✅ **Batch Operations**: Parallel processing with configurable concurrency  
✅ **Conflict Resolution**: Multiple strategies for handling file conflicts  
✅ **Progress Tracking**: Real-time progress reporting and performance metrics  
✅ **Error Recovery**: Comprehensive error handling with rollback capabilities  

### Architecture & Implementation

The File Management Tools are implemented in Rust and registered as a plugin within the `workflow-toolkit` ecosystem. Each tool in a workflow YAML corresponds to a specific Rust executor:

- **YAML Tool Mapping**: Workflow nodes use `tool_name` to look up Rust executors in the `FileManagementToolRegistry`.
- **Safe Execution**: All operations support an `experimental_mode` for safe dry-runs.
- **Detailed Mapping**: For a complete list of YAML-to-Rust mappings, see the [API Reference - Workflow Integration](FILE_MANAGEMENT_API_REFERENCE.md#workflow-integration--execution-mechanism).

## Quick Start

### Installation

The File Management Tools are included as a plugin in the workflow-toolkit system:

```bash
# Build the project with file management tools
cargo build --features file-management

# Verify plugin is available
cargo run -- plugin list | grep file-management
```

### Basic Usage

#### 1. Simple Folder Classification

```bash
# Create basic classification rules
cat > basic-rules.json << 'EOF'
{
  "categories": {
    "documents": {
      "keywords": [
        {"pattern": "doc", "weight": 1.0},
        {"pattern": "pdf", "weight": 1.0}
      ],
      "target_directory": "Documents"
    },
    "images": {
      "keywords": [
        {"pattern": "photo", "weight": 1.0},
        {"pattern": "image", "weight": 1.0}
      ],
      "target_directory": "Images"
    }
  }
}
EOF

# Run classification workflow
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized" \
  --param classification_rules="basic-rules.json" \
  --param experimental_mode=true
```

#### 2. Folder Merging

```bash
# Merge duplicate folders across locations
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/home/user/Downloads", "/home/user/Desktop"]' \
  --param target_directory="/home/user/Organized" \
  --param merge_strategy="UserDecision" \
  --param experimental_mode=true
```

#### 3. Batch File Processing

```bash
# Process files in batches
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/data/incoming" \
  --param target_directory="/data/processed" \
  --param operation_type="move" \
  --param batch_size=20 \
  --param experimental_mode=true
```

## Tool Reference

### Core Tools

#### 1. Folder Classifier (`folder-classifier`)

Intelligently categorizes folders based on configurable rules.

**Parameters:**
- `folder_path` (string): Path to folder to classify
- `classification_rules` (object/string): Classification rules (JSON object or file path)
- `enable_user_interaction` (boolean): Enable human decision-making
- `experimental_mode` (boolean): Run in simulation mode
- `confidence_threshold` (number): Minimum confidence for auto-classification

**Returns:**
- `status`: Classification status (classified, unclassified, pending, error)
- `category`: Assigned category (if classified)
- `candidates`: All category candidates with scores
- `score`: Confidence score for the classification
- `processing_time_ms`: Time taken for classification

**Example:**
```yaml
- name: "classify_folder"
  tool: "folder-classifier"
  params:
    folder_path: "/path/to/folder"
    classification_rules: "rules.json"
    confidence_threshold: 0.8
    experimental_mode: true
```

#### 2. File Mover (`file-mover`)

Safe file and folder operations with conflict resolution.

**Parameters:**
- `operations` (array): List of move operations
- `conflict_resolution` (string): How to handle conflicts (Skip, Overwrite, Rename, Fail)
- `check_disk_space` (boolean): Verify disk space before operations
- `create_directories` (boolean): Create target directories as needed

**Returns:**
- `operations_completed`: Number of successful operations
- `operations_failed`: Number of failed operations
- `total_bytes_moved`: Total data transferred
- `duration_ms`: Operation duration
- `errors`: Detailed error information

**Example:**
```yaml
- name: "move_files"
  tool: "file-mover"
  params:
    operations:
      - source: "/source/file.txt"
        destination: "/target/file.txt"
        operation_type: "Move"
    conflict_resolution: "Rename"
    check_disk_space: true
```

#### 3. Folder Merger (`folder-merger`)

Intelligently merge folders with duplicate handling.

**Parameters:**
- `source_directories` (array): Directories to scan for mergeable folders
- `target_directory` (string): Target directory for merged folders
- `merge_strategy` (string): Merge strategy (SmallerToLarger, LargerToSmaller, UserDecision, TargetDirectory)
- `duplicate_handling` (string): How to handle duplicates
- `minimum_folder_size` (number): Minimum folder size to consider

**Returns:**
- `merge_operations`: List of planned merge operations
- `space_savings`: Estimated space savings
- `conflicts_detected`: Number of conflicts found
- `merge_summary`: Summary of merge analysis

**Example:**
```yaml
- name: "merge_folders"
  tool: "folder-merger"
  params:
    source_directories: ["/dir1", "/dir2"]
    merge_strategy: "SmallerToLarger"
    duplicate_handling: "Rename"
```

#### 4. Batch Processor (`batch-processor`)

Process multiple items in parallel workflows.

**Parameters:**
- `source_directory` (string): Directory containing items to process
- `target_directory` (string): Target directory for processed items
- `operation_type` (string): Type of operation (move, copy, classify, merge, custom)
- `batch_size` (number): Items per batch
- `max_concurrent_batches` (number): Maximum concurrent batches

**Returns:**
- `batches_completed`: Number of completed batches
- `total_items_processed`: Total items processed
- `processing_time`: Total processing time
- `throughput`: Items processed per second
- `error_summary`: Summary of any errors

**Example:**
```yaml
- name: "batch_process"
  tool: "batch-processor"
  params:
    source_directory: "/data/input"
    target_directory: "/data/output"
    operation_type: "move"
    batch_size: 25
    max_concurrent_batches: 4
```

#### 5. Human Decision (`human-decision`)

Interactive decision-making for ambiguous scenarios.

**Parameters:**
- `decision_type` (string): Type of decision (Classification, FileConflict, MergeStrategy, Custom)
- `context` (object): Decision context with title, description, and metadata
- `options` (array): Available decision options
- `timeout_seconds` (number): Decision timeout
- `default_choice` (number): Default option if timeout occurs

**Returns:**
- `selected_option`: ID of selected option
- `decision_time_ms`: Time taken for decision
- `was_timeout`: Whether decision timed out
- `user_input`: Additional user input (if any)

**Example:**
```yaml
- name: "user_decision"
  tool: "human-decision"
  params:
    decision_type: "Classification"
    context:
      title: "Folder Classification Decision"
      description: "Multiple categories found"
    options:
      - id: "documents"
        label: "Documents"
        recommended: true
      - id: "projects"
        label: "Projects"
    timeout_seconds: 300
```

### Utility Tools

#### 6. Text Processor (`text-processor`)

Advanced text processing including Chinese and pinyin conversion.

**Parameters:**
- `text` (string): Text to process
- `operations` (array): Processing operations to apply
- `chinese_processing` (object): Chinese-specific processing options

**Returns:**
- `original`: Original text
- `processed`: Processed text
- `pinyin_variants`: Generated pinyin variants
- `combinations`: Keyword combinations
- `metadata`: Processing metadata

#### 7. AC Matcher (`ac-matcher`)

Efficient multi-pattern string matching using Aho-Corasick automaton.

**Parameters:**
- `text` (string): Text to search in
- `patterns` (array): Patterns to search for
- `case_sensitive` (boolean): Case-sensitive matching
- `find_overlapping` (boolean): Find overlapping matches

**Returns:**
- `matches`: Found pattern matches
- `total_matches`: Total number of matches
- `categories_found`: Categories with matches

## Workflow Templates

The File Management Tools include three comprehensive workflow templates:

### 1. Interactive Classification Workflow

**File:** `examples/templates/interactive-classification-workflow.yaml`

**Purpose:** Intelligent folder classification with human decision support and experimental mode.

**Key Features:**
- Batch processing with configurable sizes
- Human decision-making for ambiguous classifications
- Experimental mode with confirmation steps
- Chinese text processing support
- Automatic cleanup of empty directories

**Usage:**
```bash
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/folders" \
  --param output_directory="/path/to/organized" \
  --param classification_rules="rules.json"
```

### 2. Interactive Merge Workflow

**File:** `examples/templates/interactive-merge-workflow.yaml`

**Purpose:** Intelligent folder merging with user decisions and multiple strategies.

**Key Features:**
- Multiple merge strategies (SmallerToLarger, LargerToSmaller, UserDecision, TargetDirectory)
- Human decision-making for merge strategies and conflicts
- Experimental mode with detailed preview
- Backup creation before operations
- Comprehensive verification and reporting

**Usage:**
```bash
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/dir1", "/dir2"]' \
  --param merge_strategy="UserDecision"
```

### 3. Interactive Batch Processing Workflow

**File:** `examples/templates/interactive-batch-processing-workflow.yaml`

**Purpose:** Generic batch file operation workflow with human oversight.

**Key Features:**
- Generic batch processing for any operation type
- Configurable batch sizes and concurrency
- Human decision-making for conflicts
- Comprehensive error handling and recovery
- Flexible filtering criteria

**Usage:**
```bash
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/data/input" \
  --param target_directory="/data/output" \
  --param operation_type="move"
```

## Configuration Guide

### Classification Rules Format

Classification rules define how folders are categorized:

```json
{
  "version": "1.0",
  "categories": {
    "documents": {
      "description": "Document and text files",
      "keywords": [
        {"pattern": "doc", "weight": 1.0, "case_sensitive": false},
        {"pattern": "pdf", "weight": 1.0, "case_sensitive": false}
      ],
      "target_directory": "Documents",
      "confidence_threshold": 0.7
    }
  },
  "settings": {
    "minimum_score_threshold": 0.5,
    "case_sensitive": false,
    "enable_chinese_processing": false
  }
}
```

### Chinese Text Processing

Enable Chinese text processing for multilingual environments:

```json
{
  "settings": {
    "enable_chinese_processing": true,
    "enable_pinyin_conversion": true,
    "normalize_traditional_chinese": true
  },
  "categories": {
    "documents": {
      "keywords": [
        {"pattern": "文档", "weight": 1.0, "language": "zh"},
        {"pattern": "document", "weight": 1.0, "language": "en"},
        {"pattern": "wendang", "weight": 0.9, "type": "pinyin"}
      ],
      "target_directory": "文档_Documents"
    }
  }
}
```

### Performance Configuration

Optimize performance for different scenarios:

```yaml
# For large datasets
performance_config:
  batch_size: 100
  max_concurrent_batches: 8
  streaming_mode: true
  memory_limit: "4GB"

# For interactive use
interactive_config:
  batch_size: 10
  enable_user_interaction: true
  decision_timeout: 300
  detailed_progress: true
```

## Human Decision Integration

### Decision Types

The system supports several types of human decisions:

#### 1. Classification Decisions
When multiple categories have similar confidence scores:
- Display all candidate categories with scores
- Show folder context and analysis
- Allow user to select the correct category
- Learn from decisions to improve future classifications

#### 2. Conflict Resolution
When file conflicts are detected:
- Show conflicting files with details (size, date, checksum)
- Present resolution options (keep newer, rename, skip)
- Allow batch resolution for similar conflicts
- Maintain operation safety and data integrity

#### 3. Merge Strategy Selection
When merging duplicate folders:
- Display folder sizes and locations
- Present merge strategy options
- Show estimated space savings
- Allow per-group strategy decisions

### Decision Context

Rich context is provided for informed decisions:

```yaml
decision_context:
  title: "Folder Classification Decision"
  description: "Multiple categories found for folder 'Project Documents'"
  metadata:
    folder_name: "Project Documents"
    folder_size: "2.3 GB"
    file_count: 156
    confidence_scores:
      documents: 0.75
      projects: 0.72
```

### Timeout and Escalation

Configure timeouts and escalation for different scenarios:

```yaml
timeout_config:
  classification: 120      # 2 minutes
  conflict_resolution: 300 # 5 minutes
  merge_strategy: 600      # 10 minutes
  
escalation_config:
  escalate_on_timeout: true
  escalation_hierarchy:
    - "team_lead"
    - "supervisor"
```

## Best Practices

### 1. Start with Experimental Mode

Always begin with experimental mode to preview operations:

```yaml
experimental_mode: true
enable_user_interaction: true
```

### 2. Use Appropriate Batch Sizes

Choose batch sizes based on your system and use case:

```yaml
# For interactive use
batch_size: 5-10

# For automated processing
batch_size: 25-50

# For high-performance systems
batch_size: 100+
```

### 3. Configure Reasonable Timeouts

Set timeouts based on decision complexity:

```yaml
# Simple decisions
decision_timeout: 60

# Complex decisions
decision_timeout: 300

# Critical decisions
decision_timeout: 600
```

### 4. Test Classification Rules

Test rules on small datasets first:

```bash
# Test with a small subset
cargo run -- workflow execute template.yaml \
  --param source_directory="/test/small_dataset" \
  --param experimental_mode=true
```

### 5. Monitor Performance

Enable performance monitoring for optimization:

```yaml
performance_monitoring:
  enable_metrics: true
  track_processing_time: true
  monitor_memory_usage: true
  report_bottlenecks: true
```

### 6. Backup Important Data

Always create backups before operations:

```yaml
backup_config:
  create_backup: true
  backup_directory: "/backups"
  verify_backup: true
```

## Troubleshooting

### Common Issues

#### 1. Classification Rules Not Loading

**Symptoms:** Classification fails with "Invalid rules" error

**Solutions:**
- Verify JSON syntax using `jq '.' rules.json`
- Check file permissions and accessibility
- Validate rules structure against schema
- Ensure all required fields are present

#### 2. Human Decisions Timing Out

**Symptoms:** Workflow stops with "Decision timeout" error

**Solutions:**
- Increase `decision_timeout` parameter
- Check terminal/UI responsiveness
- Verify user interaction is enabled
- Consider batch decision mode

#### 3. File Operations Failing

**Symptoms:** "Permission denied" or "Disk full" errors

**Solutions:**
- Check disk space: `df -h /target/directory`
- Verify permissions: `ls -ld /target/directory`
- Review conflict resolution settings
- Ensure target directories exist

#### 4. Performance Issues

**Symptoms:** Slow processing, high memory usage

**Solutions:**
- Reduce batch size
- Limit concurrent operations
- Enable streaming mode
- Monitor system resources

#### 5. Chinese Text Processing Errors

**Symptoms:** Incorrect pinyin conversion, encoding issues

**Solutions:**
- Verify Unicode support in terminal
- Check input text encoding (UTF-8)
- Enable proper Chinese processing settings
- Test with simplified examples

### Debug Mode

Enable debug logging for troubleshooting:

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- workflow execute template.yaml

# Enable trace logging for detailed analysis
RUST_LOG=trace cargo run -- workflow execute template.yaml

# Log to file for analysis
RUST_LOG=debug cargo run -- workflow execute template.yaml 2> debug.log
```

### Performance Profiling

Profile performance to identify bottlenecks:

```bash
# Enable performance profiling
cargo run -- workflow execute template.yaml \
  --param enable_profiling=true \
  --param profile_output="profile.json"
```

## Advanced Usage

### Custom Classification Rules

Create sophisticated classification rules:

```json
{
  "categories": {
    "work_projects": {
      "keywords": [
        {"pattern": "project", "weight": 1.0},
        {"pattern": "work", "weight": 0.9},
        {"pattern": "client", "weight": 1.1}
      ],
      "target_directory": "Work/Projects/{year}",
      "conditions": {
        "min_folder_size": 1048576,
        "exclude_patterns": ["temp", "cache"]
      },
      "metadata_extraction": true
    }
  }
}
```

### Workflow Composition

Combine multiple tools in complex workflows:

```yaml
steps:
  - name: "preprocess_text"
    tool: "text-processor"
    params:
      operations: ["NormalizeCase", "ConvertTraditional"]
  
  - name: "classify_folders"
    tool: "folder-classifier"
    params:
      preprocessed_text: "{{ preprocess_text.processed }}"
  
  - name: "human_review"
    tool: "human-decision"
    condition: "{{ classify_folders.ambiguous_count > 0 }}"
  
  - name: "execute_operations"
    tool: "file-mover"
    params:
      operations: "{{ human_review.final_operations }}"
```

### Integration with External Systems

Integrate with databases and external APIs:

```yaml
integration_config:
  database:
    track_operations: true
    store_metadata: true
    connection: "postgresql://localhost/filemanagement"
  
  webhooks:
    notify_completion: true
    webhook_url: "https://api.example.com/file-operations"
  
  monitoring:
    metrics_endpoint: "http://localhost:8080/metrics"
    alert_on_errors: true
```

### Enterprise Features

Configure enterprise-grade features:

```yaml
enterprise_config:
  compliance:
    audit_all_operations: true
    data_classification: true
    retention_policies: true
  
  security:
    encrypt_sensitive_data: true
    access_control: true
    dual_approval: true
  
  governance:
    approval_workflows: true
    escalation_hierarchy: true
    compliance_reporting: true
```

## Getting Help

### Documentation Resources

- **API Reference**: Detailed API documentation for all tools
- **Configuration Guide**: Comprehensive configuration options
- **Usage Examples**: Working examples for common scenarios
- **Best Practices**: Recommended patterns and practices

### Community Support

- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share experiences
- **Examples Repository**: Community-contributed examples
- **Wiki**: Community-maintained documentation

### Professional Support

- **Enterprise Support**: Available for enterprise deployments
- **Custom Development**: Tailored solutions for specific needs
- **Training**: Workshops and training sessions
- **Consulting**: Architecture and implementation guidance

---

*This guide is continuously updated. For the latest information, please refer to the official documentation and community resources.*