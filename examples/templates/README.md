# Interactive Classification Workflow Templates

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

## Usage Examples

### Basic Usage

```bash
# Run the interactive classification workflow
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized/folders" \
  --param classification_rules="examples/templates/classification-rules-example.json"
```

### Experimental Mode (Recommended for First Run)

```bash
# Run in experimental mode to preview operations
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized/folders" \
  --param classification_rules="examples/templates/classification-rules-example.json" \
  --param experimental_mode=true \
  --param enable_user_interaction=true
```

### Production Mode (Auto-Execute)

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

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `source_directory` | string | Directory containing folders to classify |
| `output_directory` | string | Root directory for organized folders |
| `classification_rules` | object/string | Classification rules (JSON object or file path) |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `experimental_mode` | boolean | `true` | Run in experimental mode (simulate operations) |
| `enable_user_interaction` | boolean | `true` | Enable human decision-making |
| `decision_timeout` | integer | `300` | Timeout for human decisions (seconds) |
| `batch_size` | integer | `10` | Number of folders per batch |
| `confidence_threshold` | number | `0.8` | Minimum confidence for auto-classification |

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

## Workflow Flow

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

## Human Decision Points

The workflow includes several human decision points:

### 1. Ambiguous Classifications
When multiple categories have similar confidence scores, the system will:
- Pause workflow execution
- Display all candidate categories with scores
- Ask the user to select the correct category
- Continue with the user's choice

### 2. Experimental Mode Confirmation
In experimental mode, the system will:
- Show a preview of all planned operations
- Display statistics (total folders, classified, unclassified)
- Ask for confirmation before executing file operations
- Allow cancellation or modification of rules

### 3. Error Recovery
If errors occur during execution:
- Display detailed error information
- Offer options to retry, skip, or abort
- Maintain workflow state for recovery

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

## Running the Example

To run the complete example with all scenarios:

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