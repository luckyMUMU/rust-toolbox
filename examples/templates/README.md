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

1. **Always start with experimental mode** for new classification rules
2. **Use appropriate batch sizes** based on your system resources
3. **Set reasonable timeouts** for human decisions
4. **Test classification rules** on a small subset first
5. **Monitor workflow logs** for performance optimization
6. **Use version control** for classification rule files
7. **Backup important data** before running in production mode

## Troubleshooting

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