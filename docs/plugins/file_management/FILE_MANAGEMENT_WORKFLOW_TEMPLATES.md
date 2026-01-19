# File Management Workflow Templates Guide

This guide provides comprehensive documentation for the File Management Tools workflow templates, including detailed usage instructions, configuration options, and best practices for creating interactive file management workflows.

## Table of Contents

1. [Template Overview](#template-overview)
2. [Interactive Classification Workflow](#interactive-classification-workflow)
3. [Interactive Merge Workflow](#interactive-merge-workflow)
4. [Interactive Batch Processing Workflow](#interactive-batch-processing-workflow)
5. [Configuration Reference](#configuration-reference)
6. [Usage Examples](#usage-examples)
7. [Advanced Patterns](#advanced-patterns)
8. [Troubleshooting](#troubleshooting)

## Template Overview

The File Management Tools include three comprehensive workflow templates designed for different file management scenarios:

| Template | Purpose | Key Features |
|----------|---------|--------------|
| **Interactive Classification** | Intelligent folder organization | Human decisions, experimental mode, Chinese support |
| **Interactive Merge** | Duplicate folder consolidation | Multiple strategies, conflict resolution, backup |
| **Interactive Batch Processing** | Generic batch operations | Parallel processing, error recovery, filtering |

### Common Features

All templates share these core capabilities:

✅ **Human Decision Integration**: Interactive decision-making for ambiguous scenarios  
✅ **Experimental Mode**: Preview operations before execution with detailed reporting  
✅ **Error Handling**: Comprehensive error recovery and rollback capabilities  
✅ **Progress Tracking**: Real-time progress updates and performance metrics  
✅ **Configurable Batching**: Adjustable batch sizes and concurrency levels  
✅ **Conflict Resolution**: Multiple strategies for handling file conflicts  
✅ **Audit Trail**: Complete logging and reporting of all operations  

## Interactive Classification Workflow

**File:** `examples/templates/interactive-classification-workflow.yaml`

### Purpose

Intelligently classify and organize folders based on configurable rules with support for human decision-making and experimental mode preview.

### Workflow Steps

1. **📁 Scan Folders**: Discover all folders in the source directory
2. **✅ Validate Rules**: Verify classification rules are valid and complete
3. **🔄 Classify Batch**: Process folders in configurable batches with parallel execution
4. **📊 Review Results**: Display classification statistics and confidence scores
5. **❓ Experimental Check**: Branch execution based on experimental mode setting
6. **👤 Human Confirmation**: Get user approval for operations (if experimental mode)
7. **⚡ Execute Operations**: Move folders to classified locations with conflict resolution
8. **📋 Generate Report**: Create detailed operation report with statistics
9. **🧹 Cleanup**: Remove empty directories and perform final cleanup

### Parameters

#### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `source_directory` | string | Directory containing folders to classify |
| `output_directory` | string | Root directory for organized folders |
| `classification_rules` | object/string | Classification rules (JSON object or file path) |

#### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `experimental_mode` | boolean | `true` | Run in experimental mode (simulate operations) |
| `enable_user_interaction` | boolean | `true` | Enable human decision-making |
| `decision_timeout` | integer | `300` | Timeout for human decisions (seconds) |
| `batch_size` | integer | `10` | Number of folders per batch |
| `confidence_threshold` | number | `0.8` | Minimum confidence for auto-classification |
| `enable_chinese_processing` | boolean | `false` | Enable Chinese text processing and pinyin |
| `cleanup_empty_directories` | boolean | `true` | Remove empty directories after operations |

### Usage Examples

#### Basic Classification

```bash
# Simple folder classification with basic rules
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/Downloads" \
  --param output_directory="/home/user/Organized" \
  --param classification_rules="basic-rules.json" \
  --param experimental_mode=true
```

#### Production Classification

```bash
# Production mode with auto-execution
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/data/incoming" \
  --param output_directory="/data/organized" \
  --param classification_rules="production-rules.json" \
  --param experimental_mode=false \
  --param enable_user_interaction=false \
  --param confidence_threshold=0.9
```

#### Chinese Text Processing

```bash
# Classification with Chinese text support
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/中文文件夹" \
  --param output_directory="/home/user/整理后" \
  --param classification_rules="chinese-rules.json" \
  --param enable_chinese_processing=true \
  --param experimental_mode=true
```

### Classification Rules Format

```json
{
  "version": "1.0",
  "metadata": {
    "name": "Standard Classification Rules",
    "description": "General-purpose folder classification",
    "author": "System Administrator"
  },
  "categories": {
    "documents": {
      "description": "Document and text files",
      "keywords": [
        {"pattern": "doc", "weight": 1.0, "case_sensitive": false},
        {"pattern": "pdf", "weight": 1.0, "case_sensitive": false},
        {"pattern": "report", "weight": 1.2, "case_sensitive": false}
      ],
      "target_directory": "Documents",
      "confidence_threshold": 0.7,
      "priority": 1
    },
    "projects": {
      "description": "Project-related folders",
      "keywords": [
        {"pattern": "project", "weight": 1.0},
        {"pattern": "work", "weight": 0.8},
        {"pattern": "client", "weight": 1.1}
      ],
      "target_directory": "Projects/{year}",
      "confidence_threshold": 0.8,
      "priority": 2
    }
  },
  "settings": {
    "minimum_score_threshold": 0.5,
    "case_sensitive": false,
    "enable_chinese_processing": false,
    "enable_regex_patterns": true
  }
}
```

### Node to Implementation Mapping

The `interactive-classification-workflow.yaml` has been refactored into 13 granular steps, each performing a specific function.

| Node ID | `tool_name` | Rust Implementation | Role in Workflow |
| :--- | :--- | :--- | :--- |
| `step1_load_rules` | `rule-loader` | `RuleLoaderTool` | Loads and validates classification rules (supports legacy JSON) |
| `step2_preprocess_rules` | `rule-preprocessor` | `RulePreprocessorTool` | Generates variants (pinyin, traditional) for keywords |
| `step3_build_automaton` | `ac-builder` | `AhoCorasickBuilderTool` | Builds Aho-Corasick automaton for efficient matching |
| `step4_scan_source` | `directory-scanner` | `DirectoryScannerTool` | Scans source directory for subfolders |
| `step5_preprocess_folders` | `folder-preprocessor` | `FolderPreprocessorTool` | Preprocesses folder names (lowercase, etc.) |
| `step6_parallel_match` | `parallel-matcher` | `ParallelMatcherTool` | Executes parallel keyword matching using AC automaton |
| `step7_calculate_scores` | `score-calculator` | `ScoreCalculatorTool` | Calculates scores based on matches and weights |
| `step8_detect_ambiguity` | `ambiguity-detector` | `AmbiguityDetectorTool` | Identifies ambiguous or unclassified items |
| `step9_merge_results` | `result-merger` | `ResultMergerTool` | Merges auto-classification results |
| `step9b_final_merge` | `result-merger` | `ResultMergerTool` | Merges human decisions with auto-results |
| `step10_check_experimental` | `experimental-checker` | `ExperimentalCheckerTool` | Validates experimental mode status |
| `step11_human_confirmation` | `human-decision` | `HumanDecisionTool` | Interactive user confirmation for ambiguous items |
| `step12_move_folders` | `file-mover` | `FileMoverExecutor` | Executes physical file/folder operations |
| `step13_generate_report` | `report-generator` | `ReportGeneratorTool` | Generates execution report and statistics |

---

## Interactive Merge Workflow

**File:** `examples/templates/interactive-merge-workflow.yaml`

### Purpose

Intelligently merge duplicate folders across multiple locations with user-guided strategy selection and comprehensive conflict resolution.

### Workflow Steps

1. **📁 Scan Directories**: Discover all folders in source directories
2. **🔍 Find Common Folders**: Identify folders with identical names across locations
3. **📊 Analyze Merge Candidates**: Calculate sizes, detect conflicts, and assess complexity
4. **📋 Review Analysis**: Display merge analysis with statistics and recommendations
5. **🎯 Determine Strategies**: Choose merge strategies (automatic or user-guided)
6. **👤 Decide Strategies**: Get user decisions for merge strategies (if needed)
7. **📝 Create Merge Plan**: Generate detailed execution plan with all operations
8. **⚠️ Handle Conflicts**: Resolve duplicate file conflicts with user input
9. **✅ Finalize Plan**: Complete merge plan with all conflict resolutions
10. **📊 Review Final Plan**: Display comprehensive final merge plan
11. **❓ Experimental Check**: Branch based on experimental mode setting
12. **👤 Confirm Execution**: Get user confirmation (if experimental mode)
13. **💾 Create Backup**: Create backup before merge operations (if enabled)
14. **⚡ Execute Merge**: Perform folder merge operations with progress tracking
15. **🔍 Verify Results**: Verify merge operations completed successfully
16. **📋 Generate Report**: Create comprehensive merge report with statistics
17. **🧹 Cleanup**: Remove empty directories after successful merge

### Parameters

#### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `source_directories` | array | List of directories to scan for mergeable folders |

#### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `target_directory` | string | `null` | Target directory for merged folders (uses first source if not specified) |
| `merge_strategy` | string | `"UserDecision"` | Merge strategy: SmallerToLarger, LargerToSmaller, UserDecision, TargetDirectory |
| `experimental_mode` | boolean | `true` | Run in experimental mode (simulate operations) |
| `enable_user_interaction` | boolean | `true` | Enable human decision-making |
| `decision_timeout` | integer | `300` | Timeout for human decisions (seconds) |
| `duplicate_handling` | string | `"UserDecision"` | How to handle duplicates: Skip, Rename, Overwrite, UserDecision |
| `minimum_folder_size` | integer | `0` | Minimum folder size in bytes to consider for merging |
| `max_merge_depth` | integer | `3` | Maximum directory depth for recursive merging |
| `enable_size_analysis` | boolean | `true` | Perform detailed size analysis before merging |
| `backup_before_merge` | boolean | `false` | Create backup before performing merge operations |

### Usage Examples

#### Basic Merge with User Decisions

```bash
# Interactive merge with user-guided strategy selection
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/home/user/Downloads", "/home/user/Desktop", "/home/user/Documents/Temp"]' \
  --param target_directory="/home/user/Organized" \
  --param merge_strategy="UserDecision" \
  --param experimental_mode=true
```

#### Automatic Merge Strategy

```bash
# Automatic merge using "smaller to larger" strategy
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/data/dir1", "/data/dir2"]' \
  --param merge_strategy="SmallerToLarger" \
  --param duplicate_handling="Rename" \
  --param experimental_mode=false
```

#### Production Merge with Backup

```bash
# Production merge with backup creation and verification
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/enterprise/shared1", "/enterprise/shared2"]' \
  --param target_directory="/enterprise/consolidated" \
  --param merge_strategy="TargetDirectory" \
  --param backup_before_merge=true \
  --param enable_size_analysis=true \
  --param experimental_mode=false
```

### Merge Strategies

| Strategy | Description | Use Case |
|----------|-------------|----------|
| **SmallerToLarger** | Merge smaller folders into the largest one | Consolidate scattered files |
| **LargerToSmaller** | Merge larger folders into the smallest one | Maintain existing structure |
| **UserDecision** | Let user decide strategy for each group | Complex scenarios requiring judgment |
| **TargetDirectory** | Merge all folders to specified target | Centralized organization |

### Conflict Resolution Options

| Option | Description | Safety Level |
|--------|-------------|--------------|
| **Skip** | Skip conflicting files, keep originals | High |
| **Rename** | Rename conflicting files with suffix | High |
| **Overwrite** | Replace existing files with newer/larger | Medium |
| **UserDecision** | Ask user for each conflict | Highest |

## Interactive Batch Processing Workflow

**File:** `examples/templates/interactive-batch-processing-workflow.yaml`

### Purpose

Generic batch file operation workflow with human oversight, comprehensive error handling, and flexible operation types.

### Workflow Steps

1. **📁 Scan Source Items**: Discover and filter items in the source directory
2. **✅ Validate Configuration**: Verify operation configuration and target paths
3. **📊 Analyze Items**: Create batch processing plan and detect potential conflicts
4. **📋 Review Plan**: Display processing plan with statistics and estimates
5. **⚠️ Handle Conflicts**: Resolve conflicts with user decisions or automatic strategies
6. **✅ Finalize Plan**: Complete processing plan with all conflict resolutions
7. **📊 Review Final Plan**: Display comprehensive final processing plan
8. **❓ Experimental Check**: Branch based on experimental mode setting
9. **👤 Confirm Execution**: Get user confirmation (if experimental mode)
10. **💾 Create Backup**: Create backup before operations (if enabled)
11. **⚡ Execute Batches**: Perform batch processing with parallel execution and progress tracking
12. **🔍 Verify Results**: Verify batch operations completed successfully
13. **🚨 Handle Failures**: Handle any failures with user decisions and recovery options
14. **📋 Generate Report**: Create comprehensive batch processing report
15. **🧹 Cleanup**: Clean up temporary files and perform final operations
16. **📊 Final Summary**: Generate final status summary with complete statistics

### Parameters

#### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `source_directory` | string | Directory containing files/folders to process |
| `target_directory` | string | Target directory for processed items |

#### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `operation_type` | string | `"move"` | Type of operation: move, copy, classify, merge, custom |
| `operation_config` | object | `{}` | Configuration specific to the operation type |
| `experimental_mode` | boolean | `true` | Run in experimental mode (simulate operations) |
| `enable_user_interaction` | boolean | `true` | Enable human decision-making for ambiguous scenarios |
| `decision_timeout` | integer | `300` | Timeout for human decisions (seconds) |
| `batch_size` | integer | `20` | Number of items to process in each batch |
| `max_concurrent_batches` | integer | `3` | Maximum number of batches to process concurrently |
| `conflict_resolution` | string | `"UserDecision"` | How to handle conflicts: Skip, Rename, Overwrite, UserDecision |
| `progress_reporting` | boolean | `true` | Enable detailed progress reporting |
| `create_backup` | boolean | `false` | Create backup before operations |
| `filter_criteria` | object | `{}` | Criteria for filtering items to process |

### Usage Examples

#### Basic Batch Move Operation

```bash
# Move files in batches with user interaction
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/data/incoming" \
  --param target_directory="/data/processed" \
  --param operation_type="move" \
  --param experimental_mode=true \
  --param batch_size=25
```

#### Batch Copy with Conflict Resolution

```bash
# Copy files with automatic conflict resolution
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/source/documents" \
  --param target_directory="/backup/documents" \
  --param operation_type="copy" \
  --param conflict_resolution="Rename" \
  --param create_backup=false \
  --param experimental_mode=false
```

#### Custom Batch Operations with Filtering

```bash
# Custom operations with file filtering
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/media/raw" \
  --param target_directory="/media/processed" \
  --param operation_type="custom" \
  --param operation_config='{"custom_tool":"media-processor","resize_images":true}' \
  --param filter_criteria='{"file_extensions":[".jpg",".png",".mp4"],"min_size_bytes":10240}' \
  --param batch_size=10 \
  --param max_concurrent_batches=2
```

### Operation Types

| Type | Description | Configuration Options |
|------|-------------|----------------------|
| **move** | Move files/folders | `preserve_structure`, `verify_moves` |
| **copy** | Copy files/folders | `preserve_timestamps`, `verify_checksums` |
| **classify** | Classify and organize | `classification_rules`, `confidence_threshold` |
| **merge** | Merge duplicate folders | `merge_strategy`, `duplicate_handling` |
| **custom** | Custom tool execution | `custom_tool`, `custom_params` |

### Filter Criteria Options

```json
{
  "include_hidden": false,
  "min_size_bytes": 1024,
  "max_size_bytes": 1073741824,
  "file_extensions": [".pdf", ".docx", ".txt"],
  "exclude_patterns": ["*.tmp", "*.lock", ".*"],
  "modified_after": "2024-01-01",
  "modified_before": "2024-12-31"
}
```

## Configuration Reference

### Global Configuration

```yaml
# Global workflow configuration
global_config:
  variables:
    temp_directory: "/tmp/workflow-toolkit"
    max_memory_usage: "4GB"
    default_timeout: "30m"
  
  error_handling:
    continue_on_error: false
    save_state_on_error: true
    max_consecutive_errors: 5
  
  retry_policy:
    max_attempts: 3
    initial_delay: "5s"
    backoff_strategy: "exponential"
```

### Performance Configuration

```yaml
# Performance optimization settings
performance_config:
  # Memory management
  memory_limit: "4GB"
  streaming_mode: true
  cache_strategy: "LRU"
  
  # Concurrency control
  max_concurrent_operations: 8
  thread_pool_size: "auto"
  async_io: true
  
  # Batch optimization
  adaptive_batch_sizing: true
  batch_size_min: 5
  batch_size_max: 100
  
  # I/O optimization
  buffer_size: "64KB"
  read_ahead: true
  write_behind: true
```

### Security Configuration

```yaml
# Security and safety settings
security_config:
  # Access control
  restrict_to_user_directories: true
  validate_path_traversal: true
  check_permissions: true
  
  # Operation safety
  mandatory_experimental_mode: false
  require_confirmation: true
  create_backups: true
  
  # Audit and compliance
  audit_all_operations: true
  log_user_actions: true
  compliance_mode: false
```

## Usage Examples

### Example 1: Personal Desktop Cleanup

Organize a messy desktop with mixed file types:

```bash
# Create classification rules for personal files
cat > personal-rules.json << 'EOF'
{
  "categories": {
    "documents": {
      "keywords": [
        {"pattern": "doc", "weight": 1.0},
        {"pattern": "pdf", "weight": 1.0},
        {"pattern": "resume", "weight": 1.2}
      ],
      "target_directory": "Documents"
    },
    "images": {
      "keywords": [
        {"pattern": "photo", "weight": 1.0},
        {"pattern": "screenshot", "weight": 1.1},
        {"pattern": "image", "weight": 0.9}
      ],
      "target_directory": "Pictures"
    },
    "downloads": {
      "keywords": [
        {"pattern": "download", "weight": 1.0},
        {"pattern": "installer", "weight": 1.1}
      ],
      "target_directory": "Downloads/Organized"
    }
  }
}
EOF

# Run classification workflow
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/Desktop" \
  --param output_directory="/home/user/Organized" \
  --param classification_rules="personal-rules.json" \
  --param experimental_mode=true \
  --param batch_size=5 \
  --param decision_timeout=180
```

### Example 2: Enterprise Document Management

Process enterprise documents with compliance requirements:

```bash
# Create enterprise classification rules
cat > enterprise-rules.json << 'EOF'
{
  "categories": {
    "financial": {
      "keywords": [
        {"pattern": "financial", "weight": 1.0},
        {"pattern": "budget", "weight": 1.1},
        {"pattern": "invoice", "weight": 1.2}
      ],
      "target_directory": "Financial/{year}",
      "confidence_threshold": 0.9
    },
    "legal": {
      "keywords": [
        {"pattern": "contract", "weight": 1.2},
        {"pattern": "legal", "weight": 1.0},
        {"pattern": "agreement", "weight": 1.1}
      ],
      "target_directory": "Legal/{year}",
      "confidence_threshold": 0.95
    },
    "hr": {
      "keywords": [
        {"pattern": "employee", "weight": 1.0},
        {"pattern": "hr", "weight": 1.1},
        {"pattern": "personnel", "weight": 0.9}
      ],
      "target_directory": "HR/{department}",
      "confidence_threshold": 0.85
    }
  },
  "settings": {
    "minimum_score_threshold": 0.7,
    "enable_audit_logging": true,
    "require_approval": true
  }
}
EOF

# Run with enterprise settings
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/enterprise/incoming" \
  --param output_directory="/enterprise/classified" \
  --param classification_rules="enterprise-rules.json" \
  --param experimental_mode=true \
  --param enable_user_interaction=true \
  --param confidence_threshold=0.9 \
  --param decision_timeout=600
```

### Example 3: Media Library Organization

Organize a large media collection with metadata-based classification:

```bash
# Run batch processing for media files
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/media/unsorted" \
  --param target_directory="/media/organized" \
  --param operation_type="custom" \
  --param operation_config='{
    "custom_tool": "media-organizer",
    "extract_metadata": true,
    "organize_by_date": true,
    "create_thumbnails": true
  }' \
  --param filter_criteria='{
    "file_extensions": [".jpg", ".png", ".mp4", ".mov", ".avi"],
    "min_size_bytes": 10240,
    "exclude_patterns": ["*.tmp", ".*"]
  }' \
  --param batch_size=20 \
  --param max_concurrent_batches=4 \
  --param experimental_mode=true
```

### Example 4: Development Project Cleanup

Organize development projects and repositories:

```bash
# Create development-specific rules
cat > dev-rules.json << 'EOF'
{
  "categories": {
    "active_projects": {
      "keywords": [
        {"pattern": "src", "weight": 1.2},
        {"pattern": "project", "weight": 1.0},
        {"pattern": "git", "weight": 1.1}
      ],
      "target_directory": "Development/Active/{language}",
      "conditions": {
        "has_git_repo": true,
        "recent_activity": "30d"
      }
    },
    "archived_projects": {
      "keywords": [
        {"pattern": "old", "weight": 1.0},
        {"pattern": "archive", "weight": 1.2},
        {"pattern": "backup", "weight": 1.1}
      ],
      "target_directory": "Development/Archive/{year}"
    },
    "documentation": {
      "keywords": [
        {"pattern": "doc", "weight": 1.0},
        {"pattern": "readme", "weight": 1.2},
        {"pattern": "wiki", "weight": 1.1}
      ],
      "target_directory": "Development/Documentation"
    }
  }
}
EOF

# Run classification with development settings
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/developer/workspace" \
  --param output_directory="/home/developer/organized" \
  --param classification_rules="dev-rules.json" \
  --param experimental_mode=true \
  --param enable_user_interaction=true \
  --param batch_size=10
```

## Advanced Patterns

### Pattern 1: Multi-Stage Classification

Combine multiple classification passes for complex scenarios:

```yaml
name: "multi_stage_classification"
description: "Multi-stage classification with preprocessing and refinement"

steps:
  # Stage 1: Preprocess folder names (conceptually)
  - name: "preprocess_text"
    tool: "text-processor"
    params:
      text: "${input.folder_name}"
      operations: ["NormalizeCase", "ConvertTraditional", "GeneratePinyin"]

  # Stage 2: Initial classification
  - name: "initial_classification"
    tool: "folder-classifier"
    params:
      folder_path: "${input.folder_path}"
      classification_rules: "${input.rules_file}"
      enable_user_interaction: false
      output_format: "Detailed"

  # Stage 3: Human review for ambiguous cases
  - name: "human_review"
    tool: "human-decision"
    condition: "${nodes.initial_classification.status} == 'Ambiguous'"
    params:
      decision_type: "Classification"
      context:
        title: "Ambiguous Classification"
        description: "Please select the correct category for ${input.folder_name}"
        metadata: "${nodes.initial_classification.metadata}"
      options: "${nodes.initial_classification.candidates}"

  # Stage 4: Execute operations
  - name: "execute_operations"
    tool: "file-mover"
    params:
      operations:
        - source: "${input.folder_path}"
          destination: "${nodes.human_review.selected_category}/${input.folder_name}"
          operation_type: "Move"
```

### Pattern 2: Conditional Processing

Use conditions to create adaptive workflows:

```yaml
name: "conditional_processing"
description: "Adaptive processing based on folder characteristics"

steps:
  - name: "analyze_folders"
    tool: "folder-analyzer" # Placeholder for analysis tool
    params:
      source_directory: "${input.source_directory}"

  # Process large folders differently
  - name: "process_large_folders"
    tool: "batch-processor"
    condition: "${nodes.analyze_folders.large_folder_count} > 0"
    params:
      tool_name: "large-folder-handler"
      batch_items: "${nodes.analyze_folders.large_folders}"
      max_concurrency: 2
      processing_mode: "Sequential"

  # Process small folders in larger batches
  - name: "process_small_folders"
    tool: "batch-processor"
    condition: "${nodes.analyze_folders.small_folder_count} > 0"
    params:
      tool_name: "small-folder-handler"
      batch_items: "${nodes.analyze_folders.small_folders}"
      max_concurrency: 8
      processing_mode: "Parallel"
```

### Pattern 3: Error Recovery and Retry

Implement robust error handling with retry logic:

```yaml
name: "robust_processing"
description: "Processing with comprehensive error handling"

steps:
  - name: "initial_processing"
    tool: "batch-processor"
    params:
      tool_name: "file-processor"
      batch_items: "${input.items}"
      continue_on_error: true
      retry_failed_items: false

  # Handle failures with user decisions
  - name: "handle_failures"
    tool: "human-decision"
    condition: "${nodes.initial_processing.error_summary.total_errors} > 0"
    params:
      decision_type: "Custom"
      context:
        title: "Processing Failures Detected"
        description: "Some operations failed. Choose recovery action."
      options:
        - id: "retry"
          label: "Retry failed operations"
        - id: "skip"
          label: "Skip failed operations"
        - id: "manual"
          label: "Manual review required"

  # Retry failed operations with different settings
  - name: "retry_operations"
    tool: "batch-processor"
    condition: "${nodes.handle_failures.selected_option} == 'retry'"
    params:
      tool_name: "file-processor"
      batch_items: "${nodes.initial_processing.error_summary.failed_items}"
      max_concurrency: 1
      retry_failed_items: true
      max_retries: 3
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Classification Rules Not Loading

**Symptoms:**
- Workflow fails with "Invalid rules" error
- Classification returns no results

**Solutions:**
```bash
# Validate JSON syntax
jq '.' classification-rules.json

# Check file permissions
ls -la classification-rules.json

# Test with minimal rules
echo '{"categories":{"test":{"keywords":[{"pattern":"test","weight":1.0}],"target_directory":"Test"}}}' > test-rules.json
```

#### 2. Human Decisions Timing Out

**Symptoms:**
- Workflow stops with "Decision timeout" error
- No user interaction prompt appears

**Solutions:**
```yaml
# Increase timeout for complex decisions
decision_timeout: 600  # 10 minutes

# Enable batch decisions for efficiency
batch_decision_mode: true
group_similar_decisions: true

# Provide default choices
default_choice: 0
enable_auto_fallback: true
```

#### 3. Performance Issues with Large Datasets

**Symptoms:**
- Slow processing speed
- High memory usage
- System becomes unresponsive

**Solutions:**
```yaml
# Optimize batch sizes
batch_size: 10          # Reduce for memory-constrained systems
max_concurrent_batches: 2

# Enable streaming mode
streaming_mode: true
memory_limit: "2GB"

# Use progressive loading
lazy_loading: true
preload_metadata: false
```

#### 4. File Operation Failures

**Symptoms:**
- "Permission denied" errors
- "Disk full" errors
- Operations fail silently

**Solutions:**
```bash
# Check disk space
df -h /target/directory

# Verify permissions
ls -ld /target/directory
chmod 755 /target/directory

# Test write access
touch /target/directory/test_file && rm /target/directory/test_file

# Check for file locks
lsof /path/to/locked/file
```

### Debug Mode Configuration

Enable comprehensive debugging:

```yaml
debug_config:
  # Logging configuration
  log_level: "debug"
  log_file: "workflow_debug.log"
  include_stack_traces: true
  
  # Tracing configuration
  enable_operation_tracing: true
  trace_decision_flow: true
  capture_intermediate_results: true
  
  # Performance monitoring
  enable_performance_profiling: true
  memory_usage_tracking: true
  bottleneck_detection: true
```

### Performance Monitoring

Monitor workflow performance:

```bash
# Enable performance profiling
RUST_LOG=debug cargo run -- workflow execute template.yaml \
  --param enable_profiling=true \
  --param profile_output="performance_profile.json"

# Monitor system resources
top -p $(pgrep -f workflow-toolkit)
htop -p $(pgrep -f workflow-toolkit)

# Check I/O performance
iotop -p $(pgrep -f workflow-toolkit)
```

### Recovery Procedures

#### Workflow State Recovery

```bash
# Save workflow state on error
cargo run -- workflow execute template.yaml \
  --param save_state_on_error=true \
  --param state_file="workflow_state.json"

# Resume from saved state
cargo run -- workflow resume workflow_state.json

# Rollback to previous checkpoint
cargo run -- workflow rollback --checkpoint checkpoint_001
```

#### Data Recovery

```bash
# Create backup before operations
cargo run -- workflow execute template.yaml \
  --param create_backup=true \
  --param backup_directory="/backups"

# Restore from backup if needed
cp -r /backups/original_data /restored_data
```

---

*This guide is continuously updated with new features and improvements. For the latest information, please refer to the official documentation and community resources.*