# File Management Tools Parameter Documentation

This document provides comprehensive documentation for all parameters used in the File Management Tools workflow templates, including detailed explanations, examples, and best practices.

## Table of Contents

1. [Core Parameters](#core-parameters)
2. [Classification Parameters](#classification-parameters)
3. [Human Decision Parameters](#human-decision-parameters)
4. [Performance Parameters](#performance-parameters)
5. [Security and Compliance Parameters](#security-and-compliance-parameters)
6. [Advanced Configuration Parameters](#advanced-configuration-parameters)
7. [Environment Variable Integration](#environment-variable-integration)
8. [Parameter Validation](#parameter-validation)

## Core Parameters

### Directory Parameters

#### `source_directory`
- **Type**: String (path)
- **Required**: Yes
- **Description**: Directory containing files/folders to be processed
- **Examples**:
  ```yaml
  source_directory: "/home/user/Documents/ToOrganize"
  source_directory: "${HOME}/Desktop"
  source_directory: "C:\\Users\\User\\Downloads"
  ```
- **Validation**: Must be an existing directory with read permissions
- **Environment Variables**: Supports variable substitution
- **Best Practices**:
  - Use absolute paths for reliability
  - Ensure proper permissions before execution
  - Consider using environment variables for portability

#### `source_directories`
- **Type**: Array of strings (paths)
- **Required**: Yes (for merge operations)
- **Description**: Multiple directories to scan for mergeable content
- **Examples**:
  ```yaml
  source_directories:
    - "/home/user/Downloads"
    - "/home/user/Desktop"
    - "/home/user/Documents/Temp"
  ```
- **Validation**: All directories must exist and be readable
- **Use Cases**: Folder merging, multi-location consolidation

#### `output_directory` / `target_directory`
- **Type**: String (path)
- **Required**: Yes
- **Description**: Destination directory for organized files
- **Examples**:
  ```yaml
  output_directory: "/home/user/Documents/Organized"
  target_directory: "${ORGANIZED_DIR}/Sorted"
  ```
- **Validation**: Parent directory must exist or `create_directories` must be enabled
- **Auto-Creation**: Can be created automatically if `create_directories: true`

#### `backup_directory`
- **Type**: String (path)
- **Required**: No
- **Default**: None (no backup created)
- **Description**: Directory for storing backups before operations
- **Examples**:
  ```yaml
  backup_directory: "/home/user/Backups/FileManagement"
  backup_directory: "${BACKUP_ROOT}/${DATE}"
  ```
- **Best Practices**: Always use for production operations

### Classification Parameters

#### `classification_rules`
- **Type**: String (file path) or Object (inline JSON)
- **Required**: Yes (for classification operations)
- **Description**: Rules defining how folders should be classified
- **Examples**:
  ```yaml
  # File reference
  classification_rules: "examples/templates/classification-rules-example.json"
  
  # Environment variable
  classification_rules: "${CLASSIFICATION_RULES_FILE}"
  
  # Inline JSON
  classification_rules:
    categories:
      documents:
        keywords:
          - pattern: "doc"
            weight: 1.0
        target_directory: "Documents"
  ```
- **Validation**: Must be valid JSON matching the classification schema
- **File Format**: JSON with categories, keywords, and settings

#### `confidence_threshold`
- **Type**: Number (0.0 to 1.0)
- **Required**: No
- **Default**: 0.8
- **Description**: Minimum confidence score for automatic classification
- **Examples**:
  ```yaml
  confidence_threshold: 0.9  # High confidence required
  confidence_threshold: 0.6  # Lower threshold, more automation
  ```
- **Impact**: Higher values require more human decisions, lower values increase automation
- **Recommended Range**: 0.7 - 0.9 for most use cases

#### `ambiguous_threshold`
- **Type**: Number (0.0 to 1.0)
- **Required**: No
- **Default**: 0.3
- **Description**: Maximum difference between top candidates to trigger human decision
- **Examples**:
  ```yaml
  ambiguous_threshold: 0.2  # Stricter ambiguity detection
  ambiguous_threshold: 0.4  # More lenient, fewer human decisions
  ```
- **Use Case**: When multiple categories have similar scores

## Human Decision Parameters

### Basic Decision Configuration

#### `enable_user_interaction`
- **Type**: Boolean
- **Required**: No
- **Default**: true
- **Description**: Enable human decision-making for ambiguous scenarios
- **Examples**:
  ```yaml
  enable_user_interaction: true   # Interactive mode
  enable_user_interaction: false  # Fully automated
  ```
- **Impact**: When false, system uses default choices or skips ambiguous items

#### `decision_timeout`
- **Type**: Integer (seconds)
- **Required**: No
- **Default**: 300 (5 minutes)
- **Description**: Maximum time to wait for human decisions
- **Examples**:
  ```yaml
  decision_timeout: 60    # 1 minute for simple decisions
  decision_timeout: 600   # 10 minutes for complex decisions
  decision_timeout: 0     # No timeout (wait indefinitely)
  ```
- **Recommendations**:
  - Simple classifications: 60-180 seconds
  - Complex merge decisions: 300-600 seconds
  - Enterprise governance: 1800-3600 seconds

#### `default_choice`
- **Type**: String or Integer
- **Required**: No
- **Default**: None
- **Description**: Default option to select when timeout occurs
- **Examples**:
  ```yaml
  default_choice: "skip"        # Skip ambiguous items
  default_choice: 0             # Select first option
  default_choice: "recommended" # Use recommended option
  ```

### Advanced Decision Configuration

#### `batch_decision_mode`
- **Type**: Boolean
- **Required**: No
- **Default**: false
- **Description**: Group similar decisions for batch processing
- **Examples**:
  ```yaml
  batch_decision_mode: true
  group_similar_decisions: true
  similarity_threshold: 0.8
  ```
- **Benefits**: Reduces decision fatigue, improves consistency

#### `decision_context`
- **Type**: Object
- **Required**: No
- **Description**: Additional context for decision presentation
- **Examples**:
  ```yaml
  decision_context:
    show_file_previews: true
    display_folder_sizes: true
    include_modification_dates: true
    max_options_displayed: 10
  ```

#### `escalation_config`
- **Type**: Object
- **Required**: No
- **Description**: Configuration for decision escalation
- **Examples**:
  ```yaml
  escalation_config:
    escalate_on_timeout: true
    escalation_timeout: 1800
    escalation_email: "supervisor@company.com"
    require_supervisor_approval: true
  ```

## Performance Parameters

### Batch Processing

#### `batch_size`
- **Type**: Integer
- **Required**: No
- **Default**: Varies by operation (10-50)
- **Description**: Number of items to process in each batch
- **Examples**:
  ```yaml
  batch_size: 5    # Small batches for interactive use
  batch_size: 25   # Balanced performance
  batch_size: 100  # High throughput
  ```
- **Considerations**:
  - Smaller batches: More responsive, better for user interaction
  - Larger batches: Better performance, less overhead
  - Memory usage increases with batch size

#### `max_concurrent_batches`
- **Type**: Integer
- **Required**: No
- **Default**: 3-4
- **Description**: Maximum number of batches to process simultaneously
- **Examples**:
  ```yaml
  max_concurrent_batches: 1   # Sequential processing
  max_concurrent_batches: 4   # Moderate parallelism
  max_concurrent_batches: 8   # High parallelism
  ```
- **System Requirements**: Higher values require more CPU cores and memory

#### `streaming_mode`
- **Type**: Boolean
- **Required**: No
- **Default**: false
- **Description**: Process items as they're discovered rather than loading all at once
- **Examples**:
  ```yaml
  streaming_mode: true   # Memory efficient for large datasets
  streaming_mode: false  # Load all items first
  ```
- **Use Cases**: Large directories, memory-constrained systems

### Memory Management

#### `memory_limit`
- **Type**: String (size specification)
- **Required**: No
- **Default**: "auto" (system-dependent)
- **Description**: Maximum memory usage for operations
- **Examples**:
  ```yaml
  memory_limit: "2GB"    # Explicit limit
  memory_limit: "512MB"  # Conservative limit
  memory_limit: "auto"   # System-dependent
  ```
- **Format**: Supports KB, MB, GB suffixes

#### `cache_strategy`
- **Type**: String (enum)
- **Required**: No
- **Default**: "LRU"
- **Description**: Caching strategy for metadata and results
- **Options**: "LRU", "FIFO", "None"
- **Examples**:
  ```yaml
  cache_strategy: "LRU"   # Least Recently Used
  cache_strategy: "None"  # Disable caching
  ```

## Security and Compliance Parameters

### Access Control

#### `validate_paths`
- **Type**: Boolean
- **Required**: No
- **Default**: true
- **Description**: Validate all file paths for security
- **Examples**:
  ```yaml
  validate_paths: true   # Enable path validation
  validate_paths: false  # Skip validation (not recommended)
  ```
- **Security Impact**: Prevents path traversal attacks

#### `allowed_path_patterns`
- **Type**: Array of strings (glob patterns)
- **Required**: No
- **Description**: Whitelist of allowed path patterns
- **Examples**:
  ```yaml
  allowed_path_patterns:
    - "/home/user/*"
    - "/data/shared/*"
    - "${HOME}/**"
  ```

#### `forbidden_path_patterns`
- **Type**: Array of strings (glob patterns)
- **Required**: No
- **Description**: Blacklist of forbidden path patterns
- **Examples**:
  ```yaml
  forbidden_path_patterns:
    - "/system/*"
    - "/etc/*"
    - "/root/*"
    - "C:\\Windows\\*"
  ```

### Audit and Compliance

#### `audit_logging`
- **Type**: Boolean
- **Required**: No
- **Default**: false
- **Description**: Enable comprehensive audit logging
- **Examples**:
  ```yaml
  audit_logging: true
  audit_log_file: "/var/log/workflow-audit.log"
  ```

#### `compliance_mode`
- **Type**: Boolean
- **Required**: No
- **Default**: false
- **Description**: Enable compliance-specific features
- **Examples**:
  ```yaml
  compliance_mode: true
  compliance_framework: "SOX"
  data_retention_days: 2555  # 7 years
  ```

#### `data_classification`
- **Type**: Boolean
- **Required**: No
- **Default**: false
- **Description**: Enable data classification enforcement
- **Examples**:
  ```yaml
  data_classification: true
  classification_levels:
    - "public"
    - "internal"
    - "confidential"
    - "restricted"
  ```

## Advanced Configuration Parameters

### Experimental Mode

#### `experimental_mode`
- **Type**: Boolean
- **Required**: No
- **Default**: true
- **Description**: Simulate operations without executing them
- **Examples**:
  ```yaml
  experimental_mode: true   # Preview operations
  experimental_mode: false  # Execute immediately
  ```
- **Best Practice**: Always start with experimental mode for new configurations

#### `detailed_preview`
- **Type**: Boolean
- **Required**: No
- **Default**: true (when experimental_mode is true)
- **Description**: Show detailed preview of planned operations
- **Examples**:
  ```yaml
  detailed_preview: true
  preview_format: "table"  # table, list, json
  ```

### Error Handling

#### `continue_on_error`
- **Type**: Boolean
- **Required**: No
- **Default**: false
- **Description**: Continue processing when errors occur
- **Examples**:
  ```yaml
  continue_on_error: true
  max_consecutive_errors: 5
  error_escalation_threshold: 10
  ```

#### `conflict_resolution`
- **Type**: String (enum)
- **Required**: No
- **Default**: "UserDecision"
- **Description**: How to handle file conflicts
- **Options**: "Skip", "Rename", "Overwrite", "UserDecision"
- **Examples**:
  ```yaml
  conflict_resolution: "Rename"     # Automatic renaming
  conflict_resolution: "Skip"       # Skip conflicting files
  conflict_resolution: "UserDecision"  # Ask user
  ```

#### `retry_policy`
- **Type**: Object
- **Required**: No
- **Description**: Configuration for operation retries
- **Examples**:
  ```yaml
  retry_policy:
    max_attempts: 3
    initial_delay: "5s"
    backoff_strategy: "exponential"
    retry_on_errors: ["NetworkTimeout", "TemporaryUnavailable"]
  ```

### Chinese Text Processing

#### `enable_chinese_processing`
- **Type**: Boolean
- **Required**: No
- **Default**: false
- **Description**: Enable Chinese text processing features
- **Examples**:
  ```yaml
  enable_chinese_processing: true
  chinese_processing:
    pinyin_style: "Normal"
    traditional_to_simplified: true
    generate_combinations: true
  ```

#### `pinyin_style`
- **Type**: String (enum)
- **Required**: No (when Chinese processing enabled)
- **Default**: "Normal"
- **Options**: "Normal", "WithTone", "WithoutTone", "FirstLetter"
- **Examples**:
  ```yaml
  pinyin_style: "Normal"      # ni hao
  pinyin_style: "WithTone"    # nǐ hǎo
  pinyin_style: "FirstLetter" # nh
  ```

## Environment Variable Integration

### Variable Substitution Syntax

Environment variables can be used in any string parameter using the following syntax:

```yaml
# Basic substitution
source_directory: "${HOME}/Documents"

# With default value
batch_size: "${BATCH_SIZE:-25}"

# Nested substitution
output_directory: "${BASE_DIR}/${USER}/organized"
```

### Common Environment Variables

#### System Variables
- `HOME` / `USERPROFILE`: User home directory
- `USER` / `USERNAME`: Current username
- `TEMP` / `TMP`: Temporary directory
- `PWD`: Current working directory

#### Custom Variables
```bash
# Configuration paths
export WORKFLOW_CONFIG_DIR="/etc/workflow-toolkit"
export CLASSIFICATION_RULES_FILE="${WORKFLOW_CONFIG_DIR}/rules.json"

# Processing parameters
export BATCH_SIZE=50
export MAX_CONCURRENT=8
export DECISION_TIMEOUT=300

# Directory paths
export ORGANIZED_DIR="/data/organized"
export BACKUP_DIR="/data/backups"
export LOG_DIR="/var/log/workflow-toolkit"
```

### Environment-Specific Configurations

#### Development
```yaml
source_directory: "${HOME}/dev/test-data"
batch_size: "${DEV_BATCH_SIZE:-5}"
experimental_mode: true
debug_logging: true
```

#### Production
```yaml
source_directory: "${PROD_INPUT_DIR}"
batch_size: "${PROD_BATCH_SIZE:-50}"
experimental_mode: false
audit_logging: true
```

#### Testing
```yaml
source_directory: "${TEST_DATA_DIR}/input"
batch_size: "${TEST_BATCH_SIZE:-3}"
enable_user_interaction: false
```

## Parameter Validation

### Validation Rules

The system validates parameters according to these rules:

#### Path Parameters
- Must be valid file system paths
- Source paths must exist and be readable
- Target paths must be writable (or creatable)
- No path traversal attempts (../, ..\)

#### Numeric Parameters
- Must be within valid ranges
- Batch sizes: 1-1000
- Timeouts: 0-86400 (24 hours)
- Confidence thresholds: 0.0-1.0

#### Enum Parameters
- Must match predefined values
- Case-sensitive matching
- No partial matches allowed

### Validation Examples

```yaml
# Valid configurations
batch_size: 25                    # ✓ Within range
confidence_threshold: 0.8         # ✓ Valid range
conflict_resolution: "Rename"     # ✓ Valid enum value

# Invalid configurations
batch_size: 0                     # ✗ Below minimum
confidence_threshold: 1.5         # ✗ Above maximum
conflict_resolution: "rename"     # ✗ Wrong case
source_directory: "../../../etc"  # ✗ Path traversal
```

### Custom Validation

You can add custom validation rules:

```yaml
validation_rules:
  custom_checks:
    - name: "disk_space_check"
      rule: "available_space > required_space * 1.1"
      message: "Insufficient disk space (need 10% buffer)"
    - name: "permission_check"
      rule: "can_write_to_target_directory"
      message: "No write permission to target directory"
```

## Best Practices

### Parameter Organization

1. **Group Related Parameters**:
   ```yaml
   # Directory configuration
   source_directory: "${HOME}/Documents"
   output_directory: "${HOME}/Organized"
   backup_directory: "${HOME}/Backups"
   
   # Processing configuration
   batch_size: 25
   max_concurrent_batches: 4
   experimental_mode: true
   
   # Human decision configuration
   enable_user_interaction: true
   decision_timeout: 300
   batch_decision_mode: true
   ```

2. **Use Environment Variables for Flexibility**:
   ```yaml
   # Instead of hardcoded paths
   source_directory: "/home/john/Documents"
   
   # Use environment variables
   source_directory: "${HOME}/Documents"
   ```

3. **Provide Sensible Defaults**:
   ```yaml
   batch_size: "${BATCH_SIZE:-25}"
   decision_timeout: "${TIMEOUT:-300}"
   experimental_mode: "${EXPERIMENTAL:-true}"
   ```

### Performance Optimization

1. **Adjust Batch Size Based on System**:
   - SSD storage: 50-100 items per batch
   - HDD storage: 20-50 items per batch
   - Network storage: 10-25 items per batch

2. **Configure Concurrency Appropriately**:
   - CPU cores × 1-2 for CPU-bound operations
   - CPU cores × 2-4 for I/O-bound operations
   - Consider memory constraints

3. **Use Streaming for Large Datasets**:
   ```yaml
   streaming_mode: true
   memory_limit: "2GB"
   cache_strategy: "LRU"
   ```

### Security Considerations

1. **Always Validate Paths**:
   ```yaml
   validate_paths: true
   allowed_path_patterns:
     - "${HOME}/*"
     - "/data/shared/*"
   forbidden_path_patterns:
     - "/system/*"
     - "/etc/*"
   ```

2. **Enable Audit Logging for Production**:
   ```yaml
   audit_logging: true
   audit_log_file: "/var/log/workflow-audit.log"
   include_user_actions: true
   ```

3. **Use Experimental Mode First**:
   ```yaml
   experimental_mode: true  # Always test first
   create_backups: true     # Safety net
   verify_operations: true  # Double-check results
   ```

This comprehensive parameter documentation provides the foundation for configuring file management workflows effectively and securely.