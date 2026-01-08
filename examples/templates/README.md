# Interactive File Management Workflow Templates

This directory contains workflow templates for the File Management Tools plugin, demonstrating how to create interactive, human-decision-enabled workflows with experimental mode support.

## Templates

### 1. Interactive Folder Classification (`interactive-classification-workflow.yaml`)

**Purpose**: Intelligent folder classification with human decision support and experimental mode.

**Features**:
- ✅ Batch processing of folders with configurable batch sizes
- ✅ Human decision-making for ambiguous classifications
- ✅ Experimental mode with confirmation steps
- ✅ Comprehensive error handling and reporting
- ✅ Support for Chinese text processing
- ✅ Configurable confidence thresholds
- ✅ Automatic cleanup of empty directories

**Requirements Validated**:
- **8.1**: Workflow templates for folder classification
- **11.5**: Human decision-making modes
- **12.5**: Experimental mode with confirmation steps

### 2. Interactive Folder Merge (`interactive-merge-workflow.yaml`)

**Purpose**: Intelligent folder merging with user decisions and multiple merge strategies.

**Features**:
- ✅ Multiple merge strategies (SmallerToLarger, LargerToSmaller, UserDecision, TargetDirectory)
- ✅ Human decision-making for merge strategies and duplicate conflicts
- ✅ Experimental mode with detailed preview and confirmation
- ✅ Comprehensive size analysis and space savings calculation
- ✅ Backup creation before merge operations
- ✅ Duplicate file conflict resolution with multiple options
- ✅ Recursive merging with configurable depth limits
- ✅ Detailed verification and reporting

**Requirements Validated**:
- **8.2**: Workflow templates for folder merging
- **11.5**: Human decision-making modes
- **12.5**: Experimental mode with confirmation steps

### 3. Interactive Batch Processing (`interactive-batch-processing-workflow.yaml`)

**Purpose**: Generic batch file operation workflow with human oversight and decision points.

**Features**:
- ✅ Generic batch processing for any file operation (move, copy, classify, merge, custom)
- ✅ Configurable batch sizes and concurrent processing
- ✅ Human decision-making for conflicts and ambiguous scenarios
- ✅ Experimental mode with detailed operation preview
- ✅ Comprehensive error handling and recovery strategies
- ✅ Progress tracking and performance metrics
- ✅ Backup creation before operations
- ✅ Flexible filtering criteria for source items
- ✅ Automatic cleanup and finalization
- ✅ Tool composition with decision points

**Requirements Validated**:
- **8.3**: Workflow templates for batch file operations
- **8.4**: Templates configurable through workflow parameters

## Usage Examples

### Interactive Folder Classification

#### Basic Usage

```bash
# Run the interactive classification workflow
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized/folders" \
  --param classification_rules="examples/templates/classification-rules-example.json"
```

#### Experimental Mode (Recommended for First Run)

```bash
# Run in experimental mode to preview operations
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized/folders" \
  --param classification_rules="examples/templates/classification-rules-example.json" \
  --param experimental_mode=true \
  --param enable_user_interaction=true
```

#### Production Mode (Auto-Execute)

```bash
# Run in production mode without user interaction
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized/folders" \
  --param classification_rules="examples/templates/classification-rules-example.json" \
  --param experimental_mode=false \
  --param enable_user_interaction=false \
  --param confidence_threshold=0.8
```

### Interactive Folder Merge

#### Basic Merge with User Decisions

```bash
# Run interactive merge workflow with user decisions
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/path/to/dir1", "/path/to/dir2", "/path/to/dir3"]' \
  --param target_directory="/path/to/merged/output" \
  --param merge_strategy="UserDecision" \
  --param experimental_mode=true
```

#### Automatic Merge Strategy

```bash
# Run with automatic "smaller to larger" merge strategy
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/home/user/Downloads", "/home/user/Documents"]' \
  --param merge_strategy="SmallerToLarger" \
  --param duplicate_handling="Rename" \
  --param experimental_mode=false
```

#### Production Merge with Backup

```bash
# Run production merge with backup creation
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/data/photos/2023", "/data/photos/2024"]' \
  --param target_directory="/data/photos/consolidated" \
  --param merge_strategy="TargetDirectory" \
  --param backup_before_merge=true \
  --param experimental_mode=false
```

### Interactive Batch Processing

#### Basic Batch Move Operation

```bash
# Run batch move operation in experimental mode
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/path/to/source/files" \
  --param target_directory="/path/to/target/location" \
  --param operation_type="move" \
  --param experimental_mode=true \
  --param enable_user_interaction=true
```

#### Batch Copy with Conflict Resolution

```bash
# Run batch copy with automatic conflict resolution
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/path/to/documents" \
  --param target_directory="/path/to/backup" \
  --param operation_type="copy" \
  --param conflict_resolution="Rename" \
  --param batch_size=25 \
  --param max_concurrent_batches=3 \
  --param create_backup=true
```

#### Custom Batch Operations

```bash
# Run custom batch operations with filtering
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/path/to/media" \
  --param target_directory="/path/to/processed" \
  --param operation_type="custom" \
  --param operation_config='{"custom_tool":"media-processor","resize_images":true}' \
  --param filter_criteria='{"file_extensions":[".jpg",".png",".mp4"],"min_size_bytes":10240}' \
  --param experimental_mode=true
```

#### Production Batch Processing

```bash
# Run production batch processing without user interaction
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/data/files" \
  --param target_directory="/data/organized" \
  --param operation_type="move" \
  --param experimental_mode=false \
  --param enable_user_interaction=false \
  --param conflict_resolution="Skip" \
  --param progress_reporting=true
```

### Custom Configuration

```bash
# Run with custom settings
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/folders" \
  --param output_directory="/path/to/organized" \
  --param classification_rules='{"categories":{"docs":{"keywords":[{"pattern":"doc","weight":1.0}],"target_directory":"Documents"}}}' \
  --param batch_size=5 \
  --param decision_timeout=180
```

## Additional Example Configurations

### Comprehensive Classification Rules (`classification-rules-comprehensive.json`)

An extensive set of classification rules covering common organizational patterns:
- **Personal file organization**: Documents, media, software, projects
- **Professional document management**: Financial, education, development
- **Advanced features**: Learning mode, performance optimization, validation

### Chinese Text Classification (`classification-rules-chinese.json`)

Specialized rules for Chinese text processing with pinyin support:
- **Bilingual support**: Chinese and English keywords
- **Pinyin conversion**: Multiple pinyin styles and combinations
- **Traditional/Simplified**: Automatic conversion support
- **Mixed language handling**: Seamless Chinese-English content processing

### Environment Configuration Examples (`environment-config-examples.yaml`)

Comprehensive environment-specific configurations:
- **Development**: Debug settings, small batches, safety features
- **Testing**: Automated processing, validation, profiling
- **Production**: Optimized performance, audit logging, monitoring
- **Enterprise**: Governance, compliance, security features
- **High-performance**: Large dataset optimization, streaming mode
- **Cloud**: Auto-scaling, distributed processing, cost optimization

### Common Use Cases (`common-use-cases.yaml`)

Real-world scenarios with complete configurations:
- **Personal desktop cleanup**: Organize mixed desktop files
- **Downloads organization**: Automatic file type sorting with date folders
- **Project archive**: Client-based project organization
- **Media library**: Photo/video organization by date and event
- **Development cleanup**: Programming project organization by language
- **Document management**: Business document compliance and retention
- **Backup organization**: Systematic backup file management
- **Research papers**: Academic paper organization by topic
- **Email attachments**: Sender and date-based organization
- **Temporary cleanup**: System cleanup with age-based policies

### Workflow Composition Examples (`workflow-composition-examples.yaml`)

Advanced workflow patterns demonstrating:
- **Multi-stage classification with learning**: Progressive improvement from human decisions
- **Intelligent folder consolidation**: Complex merge operations with conflict resolution
- **Robust batch processing**: Comprehensive error handling and recovery
- **Enterprise document workflow**: Governance, compliance, and audit features

### Real-World Scenario Example (`real-world-scenario-example.rs`)

Complete Rust implementation of a creative agency digital asset management system:
- **Client project organization**: Automatic client and project detection
- **Media asset consolidation**: Duplicate detection and resolution
- **Deliverable preparation**: Quality checks and client packaging
- **Human decision integration**: Rich context and batch decision support

### Parameter Documentation (`parameter-documentation.md`)

Comprehensive documentation covering:
- **Core parameters**: Directory paths, classification rules, confidence thresholds
- **Human decision parameters**: Timeouts, batch decisions, escalation
- **Performance parameters**: Batch sizes, concurrency, memory management
- **Security parameters**: Access control, audit logging, compliance
- **Environment variables**: Substitution syntax, common variables, best practices

## Configuration

### Interactive Folder Classification

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

### Interactive Folder Merge

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

### Interactive Batch Processing

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

#### Filter Criteria Options

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `include_hidden` | boolean | `false` | Include hidden files and directories |
| `min_size_bytes` | integer | `0` | Minimum file size in bytes |
| `max_size_bytes` | integer | `null` | Maximum file size in bytes (null for no limit) |
| `file_extensions` | array | `[]` | List of file extensions to include (empty for all) |
| `exclude_patterns` | array | `[]` | List of patterns to exclude |

#### Operation Config Examples

**Move Operation:**
```json
{
  "preserve_structure": true,
  "create_target_dirs": true,
  "verify_moves": true
}
```

**Copy Operation:**
```json
{
  "preserve_timestamps": true,
  "preserve_permissions": true,
  "verify_checksums": true,
  "copy_mode": "incremental"
}
```

**Custom Operation:**
```json
{
  "custom_tool": "media-processor",
  "custom_params": {
    "resize_images": true,
    "target_resolution": "1920x1080",
    "compress_videos": true
  }
}
```

## Classification Rules Format

The classification rules can be provided as either:

1. **File path**: Path to a JSON file containing rules
2. **Inline JSON**: Direct JSON object in the parameter

### Example Rules Structure

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
      "target_directory": "Documents"
    },
    "media": {
      "description": "Images, videos, and audio files",
      "keywords": [
        {"pattern": "photo", "weight": 1.0, "case_sensitive": false},
        {"pattern": "video", "weight": 1.0, "case_sensitive": false}
      ],
      "target_directory": "Media"
    }
  },
  "settings": {
    "minimum_score_threshold": 0.5,
    "case_sensitive": false,
    "enable_chinese_processing": false
  }
}
```

## Workflow Flows

### Interactive Classification Workflow

The interactive classification workflow follows this sequence:

1. **📁 Scan Folders**: Discover all folders in the source directory
2. **✅ Validate Rules**: Verify classification rules are valid
3. **🔄 Classify Batch**: Process folders in parallel batches
4. **📊 Review Results**: Display classification statistics
5. **❓ Experimental Check**: Branch based on experimental mode
6. **👤 Human Confirmation**: Get user approval (if experimental mode)
7. **⚡ Execute Operations**: Move folders to classified locations
8. **📋 Generate Report**: Create detailed operation report
9. **🧹 Cleanup**: Remove empty directories

### Interactive Merge Workflow

The interactive merge workflow follows this sequence:

1. **📁 Scan Directories**: Discover all folders in source directories
2. **🔍 Find Common Folders**: Identify folders with identical names
3. **📊 Analyze Merge Candidates**: Calculate sizes and detect conflicts
4. **📋 Review Analysis**: Display merge analysis and statistics
5. **🎯 Determine Strategies**: Choose merge strategies (user or automatic)
6. **👤 Decide Strategies**: Get user decisions for merge strategies (if needed)
7. **📝 Create Merge Plan**: Generate detailed execution plan
8. **⚠️ Handle Conflicts**: Resolve duplicate file conflicts (if needed)
9. **✅ Finalize Plan**: Complete merge plan with all resolutions
10. **📊 Review Final Plan**: Display final merge plan
11. **❓ Experimental Check**: Branch based on experimental mode
12. **👤 Confirm Execution**: Get user confirmation (if experimental mode)
13. **💾 Create Backup**: Create backup before merge (if enabled)
14. **⚡ Execute Merge**: Perform folder merge operations
15. **🔍 Verify Results**: Verify merge operations completed successfully
16. **📋 Generate Report**: Create comprehensive merge report
17. **🧹 Cleanup**: Remove empty directories after merge

### Interactive Batch Processing Workflow

The interactive batch processing workflow follows this sequence:

1. **📁 Scan Source Items**: Discover and filter items in the source directory
2. **✅ Validate Configuration**: Verify operation configuration and target paths
3. **📊 Analyze Items**: Create batch processing plan and detect conflicts
4. **📋 Review Plan**: Display processing plan and statistics
5. **⚠️ Handle Conflicts**: Resolve conflicts (user decisions or automatic)
6. **✅ Finalize Plan**: Complete processing plan with all resolutions
7. **📊 Review Final Plan**: Display final processing plan
8. **❓ Experimental Check**: Branch based on experimental mode
9. **👤 Confirm Execution**: Get user confirmation (if experimental mode)
10. **💾 Create Backup**: Create backup before operations (if enabled)
11. **⚡ Execute Batches**: Perform batch processing with progress tracking
12. **🔍 Verify Results**: Verify batch operations completed successfully
13. **🚨 Handle Failures**: Handle any failures with user decisions (if needed)
14. **📋 Generate Report**: Create comprehensive batch processing report
15. **🧹 Cleanup**: Clean up after batch processing operations
16. **📊 Final Summary**: Generate final status summary

## Human Decision Points

### Interactive Classification Workflow

The classification workflow includes several human decision points:

#### 1. Ambiguous Classifications
When multiple categories have similar confidence scores, the system will:
- Pause workflow execution
- Display all candidate categories with scores
- Ask the user to select the correct category
- Continue with the user's choice

#### 2. Experimental Mode Confirmation
In experimental mode, the system will:
- Show a preview of all planned operations
- Display statistics (total folders, classified, unclassified)
- Ask for confirmation before executing file operations
- Allow cancellation or modification of rules

#### 3. Error Recovery
If errors occur during execution:
- Display detailed error information
- Offer options to retry, skip, or abort
- Maintain workflow state for recovery

### Interactive Merge Workflow

The merge workflow includes additional human decision points:

#### 1. Merge Strategy Selection
For each group of folders with identical names:
- Display folder sizes and locations
- Present merge strategy options:
  - **Smaller to Larger**: Merge smaller folders into the largest one
  - **Larger to Smaller**: Merge larger folders into the smallest one
  - **To Target Directory**: Merge all folders to a specified target location
  - **Skip**: Skip merging this particular group
- Allow per-group strategy decisions

#### 2. Duplicate File Conflicts
When duplicate files are found during merge:
- Display file details (size, modification date, location)
- Present resolution options:
  - **Keep Larger**: Keep the file with larger size
  - **Keep Newer**: Keep the file with more recent modification date
  - **Rename and Keep All**: Rename files to avoid conflicts and keep all versions
  - **Skip Merge**: Skip merging this folder due to conflicts
- Allow batch resolution for similar conflicts

#### 3. Merge Plan Confirmation
In experimental mode:
- Show detailed merge plan with all operations
- Display estimated space savings and duration
- Show backup creation plan (if enabled)
- Allow final confirmation or cancellation

#### 4. Error Recovery and Rollback
If errors occur during merge execution:
- Display detailed error context
- Offer rollback options for completed operations
- Allow continuation with modified strategy
- Maintain operation state for recovery

### Interactive Batch Processing Workflow

The batch processing workflow includes comprehensive human decision points:

#### 1. Conflict Resolution
When file conflicts are detected during batch processing:
- Display conflict details (file names, sizes, locations)
- Present resolution options:
  - **Skip**: Skip processing this item and continue with others
  - **Rename**: Rename the item to avoid conflict and process it
  - **Overwrite**: Overwrite the existing item (use with caution)
  - **Abort Batch**: Stop processing this batch due to conflicts
- Allow batch resolution for similar conflicts

#### 2. Batch Plan Confirmation
In experimental mode:
- Show detailed batch processing plan with all operations
- Display estimated duration and space requirements
- Show batch breakdown and concurrency settings
- Allow final confirmation or configuration changes

#### 3. Failure Handling
When operations fail during batch processing:
- Display failure details and statistics
- Present recovery options:
  - **Retry Failed**: Attempt to retry the failed operations
  - **Skip Failed**: Continue with successful operations, skip failures
  - **Rollback All**: Rollback all operations including successful ones
  - **Manual Review**: Mark for manual review and continue
- Allow selective retry or rollback strategies

#### 4. Operation Configuration
For custom operations:
- Present operation-specific configuration options
- Allow modification of batch sizes and concurrency
- Enable/disable progress reporting and verification
- Configure backup and recovery settings

## Advanced Features

### Chinese Text Processing

Enable Chinese text processing in classification rules:

```json
{
  "settings": {
    "enable_chinese_processing": true
  },
  "text_processing": {
    "enable_pinyin_conversion": true,
    "normalize_traditional_chinese": true
  }
}
```

### Batch Processing Configuration

Optimize for different scenarios:

```yaml
# For large datasets
batch_size: 50
parallel_execution: true
max_concurrent: 10

# For interactive use
batch_size: 5
enable_user_interaction: true
decision_timeout: 300
```

### Error Handling

Configure error handling behavior:

```yaml
global_config:
  error_handling:
    continue_on_error: false
    save_state_on_error: true
  retry_policy:
    max_attempts: 3
    initial_delay: "5s"
```

## Running the Examples

To run the complete examples with all scenarios:

### Interactive Classification Example

```bash
# Build the project
cargo build --example interactive-classification-example

# Run the example
cargo run --example interactive-classification-example
```

This will demonstrate:
1. Experimental mode with human decisions
2. Production mode (auto-execute)
3. Custom classification rules with Chinese support

### Interactive Merge Example

```bash
# Build the project
cargo build --example interactive-merge-example

# Run the example
cargo run --example interactive-merge-example
```

This will demonstrate:
1. Experimental mode with user decisions for merge strategies
2. Production mode with automatic merge strategy
3. Custom strategy with target directory and backup creation

### Interactive Batch Processing Example

```bash
# Build the project
cargo build --example interactive-batch-processing-example

# Run the example
cargo run --example interactive-batch-processing-example
```

This will demonstrate:
1. Batch file moving with conflict resolution
2. Batch file copying with user decisions
3. Custom batch operations with experimental mode
4. Error handling and recovery scenarios

## Integration with Workflow Toolkit

This template integrates seamlessly with the workflow-toolkit system:

- **Plugin System**: Uses the file-management plugin
- **Tool Registry**: Leverages registered file management tools
- **Execution Context**: Supports workflow-toolkit's parameter system
- **Error Handling**: Uses structured error reporting
- **State Management**: Supports checkpoints and recovery
- **Monitoring**: Integrates with workflow-toolkit's logging and metrics

## Best Practices

### General Workflow Best Practices

1. **Always start with experimental mode** for new classification rules
2. **Use appropriate batch sizes** based on your system resources
3. **Set reasonable timeouts** for human decisions
4. **Test classification rules** on a small subset first
5. **Monitor workflow logs** for performance optimization
6. **Use version control** for classification rule files
7. **Backup important data** before running in production mode

### Human Decision Integration Best Practices

#### 1. Decision Timeout Configuration

Configure appropriate timeouts based on the complexity of decisions:

```yaml
# For simple classification decisions
decision_timeout: 60  # 1 minute

# For complex merge strategy decisions
decision_timeout: 300  # 5 minutes

# For batch conflict resolution
decision_timeout: 180  # 3 minutes

# For critical production decisions
decision_timeout: 600  # 10 minutes
```

#### 2. Decision Context Optimization

Provide rich context for better human decisions:

```yaml
# Enable detailed analysis for better decision context
enable_size_analysis: true
show_file_previews: true
display_conflict_details: true
include_metadata: true

# Configure decision presentation
max_options_displayed: 5
sort_by_confidence: true
highlight_recommended: true
```

#### 3. Batch Decision Strategies

Optimize human decision workflows for batch operations:

```yaml
# Group similar decisions together
group_similar_decisions: true
apply_decision_to_similar: true
remember_user_preferences: true

# Provide batch decision options
enable_batch_apply: true
suggest_patterns: true
learn_from_decisions: true
```

#### 4. Progressive Decision Making

Structure workflows to minimize human intervention:

```yaml
# Start with high-confidence automatic decisions
auto_decision_threshold: 0.9
human_decision_threshold: 0.7
skip_low_confidence: true

# Escalate only when necessary
escalation_strategy: "confidence_based"
batch_similar_decisions: true
provide_undo_options: true
```

#### 5. Decision Audit and Learning

Track and learn from human decisions:

```yaml
# Enable decision tracking
log_human_decisions: true
track_decision_patterns: true
suggest_rule_improvements: true

# Decision analytics
analyze_decision_accuracy: true
identify_common_patterns: true
recommend_automation: true
```

### Tool Composition Best Practices

#### 1. Classification + File Operations

Combine classification with file operations for complete workflows:

```yaml
# Sequential composition
steps:
  - name: "classify_folders"
    tool: "folder-classifier"
    params:
      confidence_threshold: 0.8
  
  - name: "move_classified"
    tool: "file-mover"
    params:
      operations: "{{ classify_folders.results }}"
      conflict_resolution: "Rename"
```

#### 2. Merge + Batch Processing

Combine merge operations with batch processing:

```yaml
# Parallel batch merging
steps:
  - name: "find_merge_candidates"
    tool: "folder-merger"
    params:
      analysis_only: true
  
  - name: "batch_merge"
    tool: "batch-processor"
    params:
      items: "{{ find_merge_candidates.candidates }}"
      operation_type: "merge"
      batch_size: 5
```

#### 3. Text Processing + Classification

Enhance classification with text processing:

```yaml
# Text preprocessing pipeline
steps:
  - name: "preprocess_text"
    tool: "text-processor"
    params:
      operations: ["NormalizeCase", "ConvertTraditional", "GeneratePinyin"]
  
  - name: "classify_enhanced"
    tool: "folder-classifier"
    params:
      preprocessed_text: "{{ preprocess_text.processed }}"
      enable_chinese_processing: true
```

#### 4. Human Decision + Automation

Balance human decisions with automation:

```yaml
# Hybrid decision workflow
steps:
  - name: "auto_classify"
    tool: "folder-classifier"
    params:
      confidence_threshold: 0.9
      auto_execute: true
  
  - name: "human_review"
    tool: "human-decision"
    params:
      items: "{{ auto_classify.ambiguous_results }}"
      decision_type: "Classification"
  
  - name: "execute_decisions"
    tool: "file-mover"
    params:
      operations: "{{ human_review.decisions }}"
```

### Performance Optimization Best Practices

#### 1. Batch Size Optimization

Choose optimal batch sizes for different scenarios:

```yaml
# For SSDs and fast storage
batch_size: 50
max_concurrent_batches: 8

# For HDDs and slower storage
batch_size: 20
max_concurrent_batches: 4

# For network storage
batch_size: 10
max_concurrent_batches: 2

# For memory-constrained systems
batch_size: 5
max_concurrent_batches: 2
```

#### 2. Memory Management

Configure memory usage for large datasets:

```yaml
# Memory optimization settings
streaming_mode: true
max_memory_usage: "2GB"
cache_strategy: "LRU"
preload_metadata: false

# Garbage collection hints
gc_frequency: "per_batch"
clear_cache_interval: 100
```

#### 3. I/O Optimization

Optimize file system operations:

```yaml
# I/O optimization
use_async_io: true
buffer_size: "64KB"
read_ahead: true
write_behind: true

# Disk space management
check_space_frequency: "per_operation"
reserve_space_percentage: 10
cleanup_temp_files: true
```

### Error Handling Best Practices

#### 1. Graceful Degradation

Handle errors gracefully without stopping workflows:

```yaml
# Error handling configuration
continue_on_error: true
max_consecutive_errors: 5
error_escalation_threshold: 10

# Recovery strategies
auto_retry_transient_errors: true
save_state_on_error: true
enable_manual_recovery: true
```

#### 2. Error Context and Reporting

Provide detailed error context for troubleshooting:

```yaml
# Error reporting
detailed_error_logs: true
include_stack_traces: true
capture_system_state: true

# Error categorization
classify_error_types: true
suggest_solutions: true
track_error_patterns: true
```

#### 3. Rollback and Recovery

Implement robust rollback mechanisms:

```yaml
# Rollback configuration
enable_operation_rollback: true
create_rollback_points: true
verify_rollback_integrity: true

# Recovery options
partial_rollback_support: true
selective_recovery: true
state_consistency_checks: true
```

### Security and Safety Best Practices

#### 1. Safe Operation Modes

Use safe operation modes for critical data:

```yaml
# Safety settings
experimental_mode: true
require_confirmation: true
create_backups: true
verify_operations: true

# Permission checks
check_write_permissions: true
validate_target_paths: true
prevent_system_directory_operations: true
```

#### 2. Data Validation

Validate data integrity throughout operations:

```yaml
# Data validation
checksum_verification: true
size_verification: true
timestamp_preservation: true
metadata_validation: true

# Integrity checks
pre_operation_validation: true
post_operation_verification: true
periodic_integrity_checks: true
```

#### 3. Access Control

Implement proper access control:

```yaml
# Access control
restrict_to_user_directories: true
validate_path_traversal: true
check_file_permissions: true
log_access_attempts: true

# Sandboxing
enable_operation_sandboxing: true
limit_system_access: true
restrict_network_access: true
```

## Advanced Usage Patterns

### Pattern 1: Multi-Stage Classification with Human Review

This pattern demonstrates a sophisticated classification workflow with multiple stages and human review points:

```yaml
name: "multi_stage_classification"
description: "Advanced classification with preprocessing, auto-classification, human review, and execution"

parameters:
  source_directory: "/path/to/unorganized/folders"
  output_directory: "/path/to/organized/folders"
  classification_rules: "classification-rules-advanced.json"
  confidence_threshold: 0.85
  human_review_threshold: 0.6

steps:
  # Stage 1: Text preprocessing for better classification
  - name: "preprocess_folder_names"
    tool: "text-processor"
    params:
      text: "{{ folder_names }}"
      operations: ["NormalizeCase", "ConvertTraditional", "GeneratePinyin"]
      chinese_processing:
        pinyin_style: "Normal"
        generate_combinations: true

  # Stage 2: High-confidence automatic classification
  - name: "auto_classify_high_confidence"
    tool: "folder-classifier"
    params:
      folder_path: "{{ source_directory }}"
      classification_rules: "{{ classification_rules }}"
      preprocessed_text: "{{ preprocess_folder_names.processed }}"
      confidence_threshold: "{{ confidence_threshold }}"
      experimental_mode: false
      enable_user_interaction: false

  # Stage 3: Human review for medium-confidence results
  - name: "human_review_medium_confidence"
    tool: "human-decision"
    condition: "{{ auto_classify_high_confidence.ambiguous_count > 0 }}"
    params:
      decision_type: "Classification"
      context:
        title: "Review Ambiguous Classifications"
        description: "Please review folders that need manual classification"
      items: "{{ auto_classify_high_confidence.ambiguous_results }}"
      timeout_seconds: 300

  # Stage 4: Execute high-confidence classifications
  - name: "execute_auto_classifications"
    tool: "file-mover"
    params:
      operations: "{{ auto_classify_high_confidence.operations }}"
      conflict_resolution: "Rename"
      experimental_mode: false

  # Stage 5: Execute human-reviewed classifications
  - name: "execute_human_classifications"
    tool: "file-mover"
    condition: "{{ human_review_medium_confidence.completed }}"
    params:
      operations: "{{ human_review_medium_confidence.operations }}"
      conflict_resolution: "Rename"
      experimental_mode: false

  # Stage 6: Generate comprehensive report
  - name: "generate_final_report"
    tool: "report-generator"
    params:
      auto_classified: "{{ execute_auto_classifications.results }}"
      human_classified: "{{ execute_human_classifications.results }}"
      unclassified: "{{ auto_classify_high_confidence.unclassified }}"
```

### Pattern 2: Intelligent Folder Consolidation with Conflict Resolution

This pattern shows how to consolidate duplicate folders across multiple locations with intelligent conflict resolution:

```yaml
name: "intelligent_folder_consolidation"
description: "Consolidate duplicate folders with smart conflict resolution and backup"

parameters:
  source_directories: 
    - "/home/user/Downloads"
    - "/home/user/Desktop"
    - "/home/user/Documents/Temp"
  target_directory: "/home/user/Organized"
  backup_directory: "/home/user/Backups"
  merge_strategy: "UserDecision"
  create_backup: true

steps:
  # Stage 1: Analyze all source directories
  - name: "analyze_source_directories"
    tool: "folder-merger"
    params:
      source_directories: "{{ source_directories }}"
      analysis_only: true
      enable_size_analysis: true
      minimum_folder_size: 1024  # 1KB minimum

  # Stage 2: Create backup before operations
  - name: "create_backup"
    tool: "file-mover"
    condition: "{{ create_backup }}"
    params:
      operations: "{{ analyze_source_directories.backup_operations }}"
      target_directory: "{{ backup_directory }}"
      operation_type: "Copy"
      preserve_structure: true

  # Stage 3: Human decision for merge strategies
  - name: "decide_merge_strategies"
    tool: "human-decision"
    params:
      decision_type: "MergeStrategy"
      context:
        title: "Folder Merge Strategy Selection"
        description: "Choose merge strategies for duplicate folders"
      options: "{{ analyze_source_directories.merge_options }}"
      timeout_seconds: 600  # 10 minutes for complex decisions

  # Stage 4: Handle duplicate file conflicts
  - name: "resolve_file_conflicts"
    tool: "human-decision"
    condition: "{{ decide_merge_strategies.has_conflicts }}"
    params:
      decision_type: "FileConflict"
      context:
        title: "Duplicate File Resolution"
        description: "Resolve conflicts for duplicate files"
      conflicts: "{{ decide_merge_strategies.file_conflicts }}"
      batch_resolution: true
      timeout_seconds: 300

  # Stage 5: Execute merge operations
  - name: "execute_merge_operations"
    tool: "folder-merger"
    params:
      source_directories: "{{ source_directories }}"
      target_directory: "{{ target_directory }}"
      merge_strategies: "{{ decide_merge_strategies.strategies }}"
      conflict_resolutions: "{{ resolve_file_conflicts.resolutions }}"
      experimental_mode: false
      verify_operations: true

  # Stage 6: Cleanup empty directories
  - name: "cleanup_empty_directories"
    tool: "file-mover"
    params:
      operation_type: "Cleanup"
      directories: "{{ source_directories }}"
      remove_empty_only: true
      recursive: true
```

### Pattern 3: Batch Processing with Progressive Error Handling

This pattern demonstrates robust batch processing with progressive error handling and recovery:

```yaml
name: "robust_batch_processing"
description: "Batch file processing with comprehensive error handling and recovery"

parameters:
  source_directory: "/data/incoming"
  target_directory: "/data/processed"
  operation_type: "move"
  batch_size: 25
  max_concurrent_batches: 4
  error_threshold: 5

steps:
  # Stage 1: Analyze and prepare batch operations
  - name: "prepare_batch_operations"
    tool: "batch-processor"
    params:
      source_directory: "{{ source_directory }}"
      target_directory: "{{ target_directory }}"
      operation_type: "{{ operation_type }}"
      analysis_only: true
      batch_size: "{{ batch_size }}"
      filter_criteria:
        min_size_bytes: 1024
        exclude_patterns: ["*.tmp", "*.lock"]

  # Stage 2: Execute first batch (test run)
  - name: "execute_test_batch"
    tool: "batch-processor"
    params:
      operations: "{{ prepare_batch_operations.batches[0] }}"
      experimental_mode: false
      max_concurrent_batches: 1
      detailed_logging: true

  # Stage 3: Human review of test results
  - name: "review_test_results"
    tool: "human-decision"
    params:
      decision_type: "Custom"
      context:
        title: "Test Batch Results Review"
        description: "Review the results of the test batch before proceeding"
        metadata:
          successful_operations: "{{ execute_test_batch.operations_completed }}"
          failed_operations: "{{ execute_test_batch.operations_failed }}"
          errors: "{{ execute_test_batch.errors }}"
      options:
        - id: "continue"
          label: "Continue with remaining batches"
          recommended: true
        - id: "adjust_settings"
          label: "Adjust settings and retry"
        - id: "abort"
          label: "Abort batch processing"
      timeout_seconds: 180

  # Stage 4: Execute remaining batches (if approved)
  - name: "execute_remaining_batches"
    tool: "batch-processor"
    condition: "{{ review_test_results.selected_option == 'continue' }}"
    params:
      operations: "{{ prepare_batch_operations.remaining_batches }}"
      batch_size: "{{ batch_size }}"
      max_concurrent_batches: "{{ max_concurrent_batches }}"
      experimental_mode: false
      continue_on_error: true
      max_consecutive_errors: "{{ error_threshold }}"

  # Stage 5: Handle failures (if any)
  - name: "handle_batch_failures"
    tool: "human-decision"
    condition: "{{ execute_remaining_batches.operations_failed > 0 }}"
    params:
      decision_type: "Custom"
      context:
        title: "Batch Processing Failures"
        description: "Some operations failed during batch processing"
        metadata:
          failed_operations: "{{ execute_remaining_batches.failed_operations }}"
          error_summary: "{{ execute_remaining_batches.error_summary }}"
      options:
        - id: "retry_failed"
          label: "Retry failed operations"
          recommended: true
        - id: "manual_review"
          label: "Mark for manual review"
        - id: "skip_failed"
          label: "Skip failed operations"
      timeout_seconds: 300

  # Stage 6: Retry failed operations (if requested)
  - name: "retry_failed_operations"
    tool: "batch-processor"
    condition: "{{ handle_batch_failures.selected_option == 'retry_failed' }}"
    params:
      operations: "{{ execute_remaining_batches.failed_operations }}"
      batch_size: 5  # Smaller batches for retries
      max_concurrent_batches: 1
      experimental_mode: false
      retry_strategy: "exponential_backoff"

  # Stage 7: Generate comprehensive report
  - name: "generate_processing_report"
    tool: "report-generator"
    params:
      test_batch_results: "{{ execute_test_batch.results }}"
      main_batch_results: "{{ execute_remaining_batches.results }}"
      retry_results: "{{ retry_failed_operations.results }}"
      summary_statistics: true
      error_analysis: true
```

### Pattern 4: Hybrid Automation with Learning

This pattern shows how to implement a learning system that improves automation over time:

```yaml
name: "learning_classification_system"
description: "Classification system that learns from human decisions to improve automation"

parameters:
  source_directory: "/path/to/folders"
  output_directory: "/path/to/organized"
  classification_rules: "adaptive-rules.json"
  learning_enabled: true
  confidence_improvement_target: 0.1

steps:
  # Stage 1: Load historical decision data
  - name: "load_decision_history"
    tool: "decision-analyzer"
    condition: "{{ learning_enabled }}"
    params:
      history_file: "decision_history.json"
      analyze_patterns: true
      suggest_rule_improvements: true

  # Stage 2: Update classification rules based on learning
  - name: "update_classification_rules"
    tool: "rule-optimizer"
    condition: "{{ load_decision_history.has_improvements }}"
    params:
      current_rules: "{{ classification_rules }}"
      decision_patterns: "{{ load_decision_history.patterns }}"
      improvement_suggestions: "{{ load_decision_history.suggestions }}"
      confidence_target: "{{ confidence_improvement_target }}"

  # Stage 3: Classify with updated rules
  - name: "classify_with_learning"
    tool: "folder-classifier"
    params:
      folder_path: "{{ source_directory }}"
      classification_rules: "{{ update_classification_rules.optimized_rules || classification_rules }}"
      experimental_mode: true
      enable_user_interaction: true
      track_decisions: true

  # Stage 4: Human decisions with learning feedback
  - name: "human_decisions_with_learning"
    tool: "human-decision"
    condition: "{{ classify_with_learning.requires_decisions }}"
    params:
      decision_type: "Classification"
      context:
        title: "Classification Decisions (Learning Mode)"
        description: "Your decisions will help improve future automation"
      items: "{{ classify_with_learning.ambiguous_results }}"
      learning_mode: true
      provide_feedback: true
      timeout_seconds: 300

  # Stage 5: Update decision history
  - name: "update_decision_history"
    tool: "decision-tracker"
    condition: "{{ human_decisions_with_learning.completed }}"
    params:
      history_file: "decision_history.json"
      new_decisions: "{{ human_decisions_with_learning.decisions }}"
      classification_context: "{{ classify_with_learning.context }}"
      update_patterns: true

  # Stage 6: Execute operations
  - name: "execute_learned_operations"
    tool: "file-mover"
    params:
      operations: "{{ human_decisions_with_learning.final_operations }}"
      conflict_resolution: "Rename"
      experimental_mode: false
      track_success_rate: true

  # Stage 7: Analyze learning effectiveness
  - name: "analyze_learning_effectiveness"
    tool: "learning-analyzer"
    params:
      previous_accuracy: "{{ load_decision_history.accuracy_baseline }}"
      current_decisions: "{{ human_decisions_with_learning.decisions }}"
      rule_changes: "{{ update_classification_rules.changes }}"
      generate_improvement_report: true
```

### Pattern 5: Enterprise-Grade File Organization

This pattern demonstrates an enterprise-grade file organization system with compliance and audit features:

```yaml
name: "enterprise_file_organization"
description: "Enterprise-grade file organization with compliance, audit, and governance"

parameters:
  source_directories: 
    - "/enterprise/shared/incoming"
    - "/enterprise/departments/*/temp"
  compliance_rules: "enterprise-compliance.json"
  retention_policies: "retention-policies.json"
  audit_enabled: true
  governance_mode: true

steps:
  # Stage 1: Compliance and governance validation
  - name: "validate_compliance"
    tool: "compliance-validator"
    params:
      source_directories: "{{ source_directories }}"
      compliance_rules: "{{ compliance_rules }}"
      check_data_classification: true
      validate_retention_requirements: true
      scan_for_sensitive_data: true

  # Stage 2: Risk assessment for operations
  - name: "assess_operation_risks"
    tool: "risk-assessor"
    params:
      compliance_results: "{{ validate_compliance.results }}"
      proposed_operations: "{{ validate_compliance.recommended_operations }}"
      risk_tolerance: "low"
      require_approval_for_high_risk: true

  # Stage 3: Governance approval (if required)
  - name: "governance_approval"
    tool: "human-decision"
    condition: "{{ assess_operation_risks.requires_approval }}"
    params:
      decision_type: "Custom"
      context:
        title: "Governance Approval Required"
        description: "High-risk operations require governance approval"
        metadata:
          risk_level: "{{ assess_operation_risks.risk_level }}"
          affected_files: "{{ assess_operation_risks.affected_files }}"
          compliance_impact: "{{ assess_operation_risks.compliance_impact }}"
      options:
        - id: "approve"
          label: "Approve operations"
        - id: "approve_with_conditions"
          label: "Approve with additional conditions"
        - id: "reject"
          label: "Reject operations"
      timeout_seconds: 3600  # 1 hour for governance decisions

  # Stage 4: Execute compliant operations
  - name: "execute_compliant_operations"
    tool: "batch-processor"
    condition: "{{ governance_approval.selected_option != 'reject' }}"
    params:
      operations: "{{ validate_compliance.approved_operations }}"
      compliance_mode: true
      audit_logging: "{{ audit_enabled }}"
      retention_enforcement: true
      data_classification_preservation: true

  # Stage 5: Apply retention policies
  - name: "apply_retention_policies"
    tool: "retention-manager"
    params:
      processed_files: "{{ execute_compliant_operations.results }}"
      retention_policies: "{{ retention_policies }}"
      schedule_future_actions: true
      compliance_tracking: true

  # Stage 6: Generate audit report
  - name: "generate_audit_report"
    tool: "audit-reporter"
    condition: "{{ audit_enabled }}"
    params:
      compliance_validation: "{{ validate_compliance.audit_trail }}"
      risk_assessment: "{{ assess_operation_risks.audit_trail }}"
      governance_decisions: "{{ governance_approval.audit_trail }}"
      operations_executed: "{{ execute_compliant_operations.audit_trail }}"
      retention_actions: "{{ apply_retention_policies.audit_trail }}"
      report_format: "comprehensive"
      include_recommendations: true
```

## Advanced Configuration Examples

### Configuration for Large-Scale Operations

```yaml
# Configuration for processing millions of files
large_scale_config:
  performance:
    batch_size: 1000
    max_concurrent_batches: 16
    memory_limit: "8GB"
    streaming_mode: true
    
  optimization:
    use_parallel_io: true
    enable_compression: true
    cache_metadata: true
    preload_directory_structure: false
    
  monitoring:
    progress_reporting_interval: 1000
    memory_usage_monitoring: true
    performance_metrics: true
    bottleneck_detection: true
```

### Configuration for High-Security Environments

```yaml
# Configuration for high-security environments
security_config:
  access_control:
    restrict_to_user_directories: true
    validate_all_paths: true
    prevent_privilege_escalation: true
    
  audit:
    log_all_operations: true
    include_file_hashes: true
    track_user_actions: true
    compliance_reporting: true
    
  safety:
    mandatory_experimental_mode: true
    require_dual_approval: true
    create_operation_backups: true
    verify_all_operations: true
```

## Troubleshooting

### Common Issues

#### 1. Classification rules not loading
**Symptoms**: Classification fails with "Invalid rules" error
**Solutions**:
- Verify JSON syntax in rules file using `jq` or online validator
- Check file path permissions and accessibility
- Validate rules structure against schema
- Ensure all required fields are present

```bash
# Validate JSON syntax
jq '.' classification-rules.json

# Check file permissions
ls -la classification-rules.json

# Test with minimal rules
echo '{"categories":{"test":{"keywords":[{"pattern":"test","weight":1.0}],"target_directory":"Test"}}}' > test-rules.json
```

#### 2. Human decisions timing out
**Symptoms**: Workflow stops with "Decision timeout" error
**Solutions**:
- Increase `decision_timeout` parameter
- Check terminal/UI responsiveness
- Verify user interaction is enabled
- Consider batch decision mode for multiple items

```yaml
# Increase timeout for complex decisions
decision_timeout: 600  # 10 minutes

# Enable batch decisions
batch_decision_mode: true
group_similar_decisions: true
```

#### 3. File operations failing
**Symptoms**: "Permission denied" or "Disk full" errors
**Solutions**:
- Check disk space availability
- Verify directory permissions
- Review conflict resolution settings
- Ensure target directories exist or can be created

```bash
# Check disk space
df -h /target/directory

# Check permissions
ls -ld /target/directory

# Test write permissions
touch /target/directory/test_file && rm /target/directory/test_file
```

#### 4. Performance issues with large datasets
**Symptoms**: Slow processing, high memory usage, system unresponsiveness
**Solutions**:
- Reduce batch size
- Limit concurrent operations
- Enable streaming mode
- Monitor system resources

```yaml
# Optimized settings for large datasets
batch_size: 10
max_concurrent_batches: 2
streaming_mode: true
memory_limit: "2GB"
```

#### 5. Chinese text processing errors
**Symptoms**: Incorrect pinyin conversion, encoding issues
**Solutions**:
- Verify Unicode support in terminal
- Check input text encoding
- Enable proper Chinese processing settings
- Test with simplified examples

```yaml
# Chinese processing configuration
chinese_processing:
  encoding: "UTF-8"
  pinyin_style: "Normal"
  handle_traditional: true
  normalize_unicode: true
```

### Advanced Troubleshooting

#### Debug Mode Configuration

Enable comprehensive debugging for complex issues:

```yaml
debug_config:
  logging:
    level: "debug"
    include_stack_traces: true
    log_file: "workflow_debug.log"
    
  tracing:
    enable_operation_tracing: true
    trace_decision_flow: true
    capture_intermediate_results: true
    
  monitoring:
    memory_usage_tracking: true
    performance_profiling: true
    bottleneck_detection: true
```

#### Performance Profiling

Profile workflow performance to identify bottlenecks:

```bash
# Enable performance profiling
RUST_LOG=debug cargo run -- workflow execute template.yaml \
  --param enable_profiling=true \
  --param profile_output="performance_profile.json"

# Analyze performance data
cargo run --example analyze_performance -- performance_profile.json
```

#### Memory Usage Analysis

Monitor and optimize memory usage:

```yaml
memory_analysis:
  enable_memory_tracking: true
  memory_limit_warnings: true
  garbage_collection_hints: true
  memory_usage_reports: true
```

#### Network and I/O Diagnostics

Diagnose network and I/O related issues:

```bash
# Test network storage performance
time ls -la /network/storage/path

# Monitor I/O during operations
iostat -x 1 &
cargo run -- workflow execute template.yaml
kill %1
```

### Error Recovery Strategies

#### Automatic Recovery

Configure automatic recovery for transient errors:

```yaml
recovery_config:
  auto_retry:
    enabled: true
    max_attempts: 3
    backoff_strategy: "exponential"
    retry_delay: "5s"
    
  error_classification:
    transient_errors: ["NetworkTimeout", "TemporaryUnavailable"]
    permanent_errors: ["PermissionDenied", "FileNotFound"]
    
  recovery_actions:
    save_state_on_error: true
    create_recovery_checkpoint: true
    enable_partial_rollback: true
```

#### Manual Recovery

Handle complex errors with manual intervention:

```yaml
manual_recovery:
  error_escalation:
    escalate_after_attempts: 2
    provide_recovery_options: true
    include_error_context: true
    
  recovery_options:
    - "Retry with different settings"
    - "Skip problematic items"
    - "Rollback to previous state"
    - "Manual intervention required"
```

#### State Recovery

Recover from interrupted workflows:

```bash
# Resume interrupted workflow
cargo run -- workflow resume workflow_state.json

# Rollback to previous checkpoint
cargo run -- workflow rollback --checkpoint checkpoint_001

# Analyze workflow state
cargo run -- workflow analyze-state workflow_state.json
```

### Monitoring and Diagnostics

#### Real-time Monitoring

Monitor workflow execution in real-time:

```yaml
monitoring_config:
  real_time_metrics:
    enabled: true
    update_interval: "1s"
    metrics_endpoint: "http://localhost:8080/metrics"
    
  alerts:
    error_rate_threshold: 5
    memory_usage_threshold: "4GB"
    processing_time_threshold: "30m"
    
  dashboards:
    enable_web_dashboard: true
    dashboard_port: 8080
    include_performance_graphs: true
```

#### Health Checks

Implement comprehensive health checks:

```yaml
health_checks:
  system_resources:
    check_disk_space: true
    check_memory_availability: true
    check_cpu_usage: true
    
  dependencies:
    check_file_system_access: true
    check_network_connectivity: true
    check_external_services: true
    
  workflow_state:
    validate_configuration: true
    check_tool_availability: true
    verify_permissions: true
```

### Debug Mode Usage

Enable debug logging for troubleshooting:

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml [parameters]

# Enable trace logging for detailed analysis
RUST_LOG=trace cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml [parameters]

# Log to file for analysis
RUST_LOG=debug cargo run -- workflow execute template.yaml 2> debug.log

# Filter logs by component
RUST_LOG=workflow_toolkit::plugins::file_management=debug cargo run -- workflow execute template.yaml
```

### Performance Optimization

#### System Resource Optimization

```bash
# Monitor system resources during execution
top -p $(pgrep -f workflow-toolkit)
htop -p $(pgrep -f workflow-toolkit)

# Check I/O performance
iotop -p $(pgrep -f workflow-toolkit)

# Monitor memory usage
watch -n 1 'ps -p $(pgrep -f workflow-toolkit) -o pid,ppid,cmd,%mem,%cpu --sort=-%mem'
```

#### Configuration Tuning

```yaml
# Performance tuning configuration
performance_tuning:
  cpu_optimization:
    thread_pool_size: "auto"  # or specific number
    cpu_affinity: true
    numa_awareness: true
    
  memory_optimization:
    memory_pool_size: "4GB"
    garbage_collection_strategy: "generational"
    memory_mapping: true
    
  io_optimization:
    async_io: true
    io_buffer_size: "1MB"
    read_ahead: true
    write_behind: true
```

### Getting Help

#### Community Resources

- **GitHub Issues**: Report bugs and request features
- **Documentation**: Comprehensive guides and API reference
- **Examples**: Working examples for common use cases
- **Community Forum**: Ask questions and share experiences

#### Professional Support

- **Enterprise Support**: Available for enterprise deployments
- **Custom Development**: Tailored solutions for specific needs
- **Training**: Workshops and training sessions
- **Consulting**: Architecture and implementation guidance

#### Diagnostic Information

When reporting issues, include:

```bash
# System information
uname -a
cargo --version
rustc --version

# Workflow toolkit version
cargo run -- --version

# Configuration dump
cargo run -- config dump

# Recent logs
tail -n 100 workflow.log

# Performance metrics
cargo run -- metrics export
```

### Common Issues

1. **Classification rules not loading**
   - Verify JSON syntax in rules file
   - Check file path permissions
   - Validate rules structure

2. **Human decisions timing out**
   - Increase `decision_timeout` parameter
   - Check terminal/UI responsiveness
   - Verify user interaction is enabled

3. **File operations failing**
   - Check disk space availability
   - Verify directory permissions
   - Review conflict resolution settings

4. **Performance issues with large datasets**
   - Reduce batch size
   - Limit concurrent operations
   - Enable streaming mode

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml [parameters]
```

## Contributing

To add new workflow templates:

1. Create the template YAML file
2. Add example configuration files
3. Create a usage example in Rust
4. Update this README with documentation
5. Add integration tests

Follow the existing patterns for consistency and maintainability.