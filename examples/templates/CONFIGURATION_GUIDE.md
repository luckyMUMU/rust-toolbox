# File Management Tools Configuration Guide

This guide provides comprehensive documentation for configuring the File Management Tools workflow templates, including detailed parameter explanations, best practices, and advanced configuration patterns.

## Table of Contents

1. [Basic Configuration](#basic-configuration)
2. [Classification Rules](#classification-rules)
3. [Human Decision Configuration](#human-decision-configuration)
4. [Performance Tuning](#performance-tuning)
5. [Security and Safety](#security-and-safety)
6. [Advanced Features](#advanced-features)
7. [Environment-Specific Configurations](#environment-specific-configurations)
8. [Troubleshooting Configuration Issues](#troubleshooting-configuration-issues)

## Basic Configuration

### Required Parameters

All file management workflow templates require these basic parameters:

#### Source and Target Directories

```yaml
# Single source directory
source_directory: "/path/to/source"

# Multiple source directories (for merge operations)
source_directories:
  - "/path/to/source1"
  - "/path/to/source2"
  - "/path/to/source3"

# Target directory for organized files
output_directory: "/path/to/organized"
target_directory: "/path/to/target"
```

**Best Practices**:
- Use absolute paths for reliability
- Ensure directories exist or enable `create_directories: true`
- Verify write permissions before execution
- Consider using environment variables for path configuration

#### Classification Rules

```yaml
# Inline JSON rules
classification_rules:
  categories:
    documents:
      keywords:
        - pattern: "doc"
          weight: 1.0
      target_directory: "Documents"

# External rules file
classification_rules: "path/to/rules.json"

# Environment variable reference
classification_rules: "${CLASSIFICATION_RULES_FILE}"
```

### Optional Parameters with Defaults

#### Execution Mode

```yaml
# Experimental mode (default: true)
experimental_mode: true  # Simulate operations without executing
experimental_mode: false # Execute operations immediately

# User interaction (default: true)
enable_user_interaction: true  # Enable human decision points
enable_user_interaction: false # Fully automated execution
```

#### Batch Processing

```yaml
# Batch size (default: varies by template)
batch_size: 10          # Small batches for interactive use
batch_size: 50          # Medium batches for balanced performance
batch_size: 100         # Large batches for high throughput

# Concurrent processing (default: 3)
max_concurrent_batches: 1   # Sequential processing
max_concurrent_batches: 4   # Moderate parallelism
max_concurrent_batches: 8   # High parallelism (requires adequate resources)
```

## Classification Rules

### Rule Structure

Classification rules define how folders are categorized based on their names and content. The rules follow a hierarchical JSON structure:

```json
{
  "version": "1.0",
  "metadata": {
    "name": "Standard Classification Rules",
    "description": "General-purpose folder classification",
    "author": "System Administrator",
    "created": "2024-01-01",
    "last_modified": "2024-01-08"
  },
  "categories": {
    "category_name": {
      "description": "Human-readable description",
      "keywords": [
        {
          "pattern": "keyword_or_regex",
          "weight": 1.0,
          "case_sensitive": false,
          "match_type": "substring"
        }
      ],
      "target_directory": "Target/Subdirectory",
      "confidence_threshold": 0.7,
      "priority": 1
    }
  },
  "settings": {
    "minimum_score_threshold": 0.5,
    "case_sensitive": false,
    "enable_chinese_processing": false,
    "enable_regex_patterns": true,
    "conflict_resolution": "highest_score"
  }
}
```

### Keyword Configuration

#### Basic Keywords

```json
{
  "pattern": "document",
  "weight": 1.0,
  "case_sensitive": false
}
```

#### Advanced Keywords with Regex

```json
{
  "pattern": "^(doc|pdf|txt).*",
  "weight": 1.5,
  "case_sensitive": false,
  "match_type": "regex"
}
```

#### Weighted Keywords

```json
{
  "keywords": [
    {"pattern": "important", "weight": 2.0},
    {"pattern": "urgent", "weight": 1.8},
    {"pattern": "normal", "weight": 1.0},
    {"pattern": "archive", "weight": 0.5}
  ]
}
```

#### Chinese Text Processing

```json
{
  "pattern": "文档",
  "weight": 1.0,
  "case_sensitive": false,
  "enable_pinyin": true,
  "pinyin_variants": ["wendang", "wen_dang"]
}
```

### Category Configuration

#### Basic Category

```json
{
  "documents": {
    "description": "Document files and folders",
    "keywords": [
      {"pattern": "doc", "weight": 1.0},
      {"pattern": "pdf", "weight": 1.0},
      {"pattern": "text", "weight": 0.8}
    ],
    "target_directory": "Documents"
  }
}
```

#### Advanced Category with Conditions

```json
{
  "media": {
    "description": "Media files (images, videos, audio)",
    "keywords": [
      {"pattern": "photo", "weight": 1.2},
      {"pattern": "video", "weight": 1.2},
      {"pattern": "music", "weight": 1.0}
    ],
    "target_directory": "Media",
    "confidence_threshold": 0.8,
    "priority": 2,
    "conditions": {
      "min_folder_size": 1048576,  // 1MB
      "max_folder_count": 1000,
      "exclude_patterns": ["temp", "cache"]
    }
  }
}
```

#### Hierarchical Categories

```json
{
  "work": {
    "description": "Work-related files",
    "target_directory": "Work",
    "subcategories": {
      "projects": {
        "keywords": [{"pattern": "project", "weight": 1.0}],
        "target_directory": "Work/Projects"
      },
      "meetings": {
        "keywords": [{"pattern": "meeting", "weight": 1.0}],
        "target_directory": "Work/Meetings"
      }
    }
  }
}
```

### Global Settings

#### Scoring Configuration

```json
{
  "settings": {
    "minimum_score_threshold": 0.5,
    "confidence_threshold": 0.8,
    "scoring_algorithm": "weighted_sum",
    "normalize_scores": true,
    "boost_exact_matches": 1.5
  }
}
```

#### Text Processing Settings

```json
{
  "settings": {
    "case_sensitive": false,
    "enable_chinese_processing": true,
    "enable_regex_patterns": true,
    "unicode_normalization": "NFC",
    "remove_special_characters": true,
    "word_boundary_matching": true
  }
}
```

## Human Decision Configuration

### Decision Timeout Configuration

Configure timeouts based on decision complexity:

```yaml
# Simple classification decisions
decision_timeout: 60      # 1 minute

# Complex merge strategy decisions  
decision_timeout: 300     # 5 minutes

# Critical production decisions
decision_timeout: 600     # 10 minutes

# No timeout (wait indefinitely)
decision_timeout: 0
```

### Decision Context Configuration

Provide rich context for better human decisions:

```yaml
decision_context:
  # Display configuration
  show_file_previews: true
  display_folder_sizes: true
  include_modification_dates: true
  show_conflict_details: true
  
  # Analysis configuration
  enable_size_analysis: true
  calculate_space_savings: true
  detect_duplicate_content: true
  analyze_folder_structure: true
  
  # Presentation configuration
  max_options_displayed: 10
  sort_by_confidence: true
  highlight_recommended: true
  group_similar_options: true
```

### Batch Decision Configuration

Optimize decision workflows for batch operations:

```yaml
batch_decisions:
  # Grouping configuration
  group_similar_decisions: true
  similarity_threshold: 0.8
  max_group_size: 20
  
  # Application configuration
  apply_decision_to_similar: true
  remember_user_preferences: true
  suggest_patterns: true
  
  # Learning configuration
  learn_from_decisions: true
  update_confidence_scores: true
  improve_grouping_algorithm: true
```

### Decision Escalation

Configure escalation for complex decisions:

```yaml
decision_escalation:
  # Escalation triggers
  escalate_on_timeout: true
  escalate_on_low_confidence: true
  escalate_on_high_risk: true
  
  # Escalation configuration
  escalation_threshold: 0.5
  require_supervisor_approval: true
  escalation_timeout: 1800  # 30 minutes
  
  # Notification configuration
  notify_on_escalation: true
  escalation_email: "supervisor@company.com"
  include_decision_context: true
```

## Performance Tuning

### Memory Configuration

```yaml
memory_config:
  # Memory limits
  max_memory_usage: "4GB"
  memory_warning_threshold: "3GB"
  enable_memory_monitoring: true
  
  # Memory optimization
  streaming_mode: true
  lazy_loading: true
  cache_strategy: "LRU"
  cache_size: "512MB"
  
  # Garbage collection
  gc_frequency: "per_batch"
  force_gc_threshold: "2GB"
  memory_pressure_handling: "reduce_batch_size"
```

### CPU and Concurrency Configuration

```yaml
cpu_config:
  # Thread configuration
  thread_pool_size: "auto"  # or specific number like 8
  max_concurrent_operations: 4
  cpu_affinity: true
  
  # Processing configuration
  parallel_processing: true
  async_io: true
  work_stealing: true
  
  # Load balancing
  dynamic_load_balancing: true
  adaptive_batch_sizing: true
  backpressure_handling: true
```

### I/O Optimization

```yaml
io_config:
  # Buffer configuration
  read_buffer_size: "64KB"
  write_buffer_size: "64KB"
  io_queue_depth: 32
  
  # Optimization strategies
  sequential_access_optimization: true
  read_ahead: true
  write_behind: true
  
  # File system specific
  use_direct_io: false
  enable_file_locking: true
  sync_frequency: "per_operation"
```

### Network Storage Configuration

```yaml
network_config:
  # Connection configuration
  connection_timeout: 30
  read_timeout: 60
  retry_attempts: 3
  
  # Optimization
  connection_pooling: true
  persistent_connections: true
  compression: true
  
  # Caching
  metadata_cache: true
  directory_cache: true
  cache_ttl: 300  # 5 minutes
```

## Security and Safety

### Access Control Configuration

```yaml
security_config:
  # Path validation
  restrict_to_user_directories: true
  validate_path_traversal: true
  allowed_path_patterns:
    - "/home/user/*"
    - "/data/shared/*"
  forbidden_path_patterns:
    - "/system/*"
    - "/etc/*"
    - "/root/*"
  
  # Permission checks
  check_read_permissions: true
  check_write_permissions: true
  verify_ownership: true
  
  # Operation restrictions
  prevent_system_modifications: true
  restrict_executable_operations: true
  limit_file_size_operations: "10GB"
```

### Audit and Compliance Configuration

```yaml
audit_config:
  # Audit logging
  enable_audit_logging: true
  audit_log_file: "/var/log/workflow-toolkit/audit.log"
  log_level: "detailed"
  
  # Compliance tracking
  track_data_lineage: true
  record_user_actions: true
  maintain_operation_history: true
  
  # Retention
  audit_retention_days: 365
  compress_old_logs: true
  secure_log_storage: true
```

### Backup and Recovery Configuration

```yaml
backup_config:
  # Backup creation
  create_backups: true
  backup_directory: "/backups/workflow-toolkit"
  backup_compression: true
  
  # Backup strategy
  backup_before_operations: true
  incremental_backups: true
  backup_verification: true
  
  # Recovery configuration
  enable_rollback: true
  rollback_verification: true
  recovery_point_interval: "1h"
```

## Advanced Features

### Chinese Text Processing

```yaml
chinese_processing:
  # Basic configuration
  enable_chinese_processing: true
  traditional_to_simplified: true
  detect_chinese_text: true
  
  # Pinyin configuration
  enable_pinyin_conversion: true
  pinyin_style: "Normal"  # Normal, WithTone, WithoutTone, FirstLetter
  include_tone_numbers: false
  
  # Advanced features
  generate_pinyin_combinations: true
  fuzzy_pinyin_matching: true
  multi_pronunciation_handling: true
```

### Machine Learning Integration

```yaml
ml_config:
  # Learning configuration
  enable_learning: true
  learning_algorithm: "adaptive_classification"
  confidence_improvement_target: 0.1
  
  # Training data
  decision_history_file: "decision_history.json"
  training_data_retention: 90  # days
  anonymize_training_data: true
  
  # Model updates
  auto_update_models: true
  model_update_frequency: "weekly"
  require_approval_for_updates: true
```

### Integration Configuration

```yaml
integration_config:
  # External systems
  enable_external_apis: true
  api_timeout: 30
  api_retry_attempts: 3
  
  # Webhooks
  enable_webhooks: true
  webhook_endpoints:
    - url: "https://api.example.com/workflow-events"
      events: ["operation_completed", "error_occurred"]
      authentication: "bearer_token"
  
  # Notifications
  enable_notifications: true
  notification_channels:
    - type: "email"
      recipients: ["admin@company.com"]
    - type: "slack"
      webhook_url: "https://hooks.slack.com/..."
```

## Environment-Specific Configurations

### Development Environment

```yaml
development_config:
  # Safety settings
  experimental_mode: true
  require_confirmation: true
  create_backups: true
  
  # Debug settings
  debug_logging: true
  verbose_output: true
  include_stack_traces: true
  
  # Performance settings
  batch_size: 5
  max_concurrent_batches: 2
  enable_profiling: true
```

### Testing Environment

```yaml
testing_config:
  # Test data
  use_test_data: true
  test_data_directory: "/test/data"
  cleanup_after_tests: true
  
  # Validation
  strict_validation: true
  verify_all_operations: true
  detailed_reporting: true
  
  # Isolation
  sandbox_mode: true
  prevent_external_access: true
  mock_external_services: true
```

### Production Environment

```yaml
production_config:
  # Safety settings
  experimental_mode: false
  require_governance_approval: true
  mandatory_backups: true
  
  # Performance settings
  batch_size: 100
  max_concurrent_batches: 8
  optimize_for_throughput: true
  
  # Monitoring
  enable_monitoring: true
  alert_on_errors: true
  performance_tracking: true
  
  # Compliance
  audit_all_operations: true
  data_classification_enforcement: true
  retention_policy_enforcement: true
```

### Enterprise Environment

```yaml
enterprise_config:
  # Governance
  governance_mode: true
  compliance_framework: "SOX"  # SOX, GDPR, HIPAA, etc.
  require_dual_approval: true
  
  # Security
  encryption_at_rest: true
  encryption_in_transit: true
  secure_key_management: true
  
  # Integration
  ldap_authentication: true
  sso_integration: true
  enterprise_logging: true
  
  # Scalability
  distributed_processing: true
  load_balancing: true
  auto_scaling: true
```

## Troubleshooting Configuration Issues

### Configuration Validation

```bash
# Validate configuration syntax
cargo run -- config validate config.yaml

# Test configuration with dry run
cargo run -- workflow execute template.yaml --dry-run --config config.yaml

# Check configuration precedence
cargo run -- config show-effective --config config.yaml
```

### Common Configuration Problems

#### 1. Path Configuration Issues

```yaml
# Problem: Relative paths causing issues
source_directory: "./data"  # ❌ Problematic

# Solution: Use absolute paths
source_directory: "/home/user/data"  # ✅ Reliable

# Alternative: Use environment variables
source_directory: "${HOME}/data"  # ✅ Flexible
```

#### 2. Memory Configuration Issues

```yaml
# Problem: Memory limits too low
max_memory_usage: "100MB"  # ❌ Too restrictive

# Solution: Appropriate memory limits
max_memory_usage: "2GB"    # ✅ Reasonable for most cases

# Alternative: Auto-detection
max_memory_usage: "auto"   # ✅ System-dependent
```

#### 3. Concurrency Configuration Issues

```yaml
# Problem: Too much concurrency
max_concurrent_batches: 20  # ❌ May overwhelm system

# Solution: Balanced concurrency
max_concurrent_batches: 4   # ✅ Balanced performance

# Alternative: Auto-tuning
max_concurrent_batches: "auto"  # ✅ System-dependent
```

### Configuration Testing

```yaml
# Test configuration with minimal data
test_config:
  source_directory: "/tmp/test_data"
  batch_size: 2
  max_concurrent_batches: 1
  experimental_mode: true
  detailed_logging: true
```

### Configuration Templates

#### Minimal Configuration

```yaml
# Minimal working configuration
source_directory: "/path/to/source"
output_directory: "/path/to/output"
classification_rules: "basic-rules.json"
experimental_mode: true
```

#### Recommended Configuration

```yaml
# Recommended production configuration
source_directory: "/data/incoming"
output_directory: "/data/organized"
classification_rules: "production-rules.json"
experimental_mode: false
enable_user_interaction: true
batch_size: 25
max_concurrent_batches: 4
decision_timeout: 300
create_backups: true
audit_logging: true
```

#### High-Performance Configuration

```yaml
# High-performance configuration for large datasets
source_directory: "/data/bulk"
output_directory: "/data/processed"
classification_rules: "optimized-rules.json"
experimental_mode: false
enable_user_interaction: false
batch_size: 100
max_concurrent_batches: 8
streaming_mode: true
max_memory_usage: "8GB"
async_io: true
```

This configuration guide provides comprehensive documentation for all aspects of configuring the File Management Tools workflow templates. Use it as a reference when setting up workflows for different environments and use cases.