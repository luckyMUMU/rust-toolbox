# examples/templates/ - Workflow Templates & Patterns

## OVERVIEW
Production-ready workflow templates with comprehensive documentation and interactive examples. 114 lines in AGENTS.md plus extensive documentation files.

## WORKFLOW TEMPLATES

### Interactive Classification
**File**: `interactive-classification-workflow.yaml`  
**Purpose**: Folder classification with human decision support  
**Key Features**:
- AI-powered classification with confidence scoring
- Interactive prompts for human review
- Experimental mode for advanced heuristics
- Audit trail for all decisions
- Batch processing with configurable sizes
- Chinese text processing support

**Requirements Validated**:
- **8.1**: Workflow templates for folder classification
- **11.5**: Human decision-making modes
- **12.5**: Experimental mode with confirmation steps

**Usage**: `cargo run --example interactive-classification-example`

### Interactive Batch Processing
**File**: `interactive-batch-processing-workflow.yaml`  
**Purpose**: Generic batch file operation workflow with human oversight  
**Key Features**:
- Generic batch processing for any file operation (move, copy, classify, merge, custom)
- Configurable batch sizes and concurrent processing
- Human decision-making for conflicts and ambiguous scenarios
- Experimental mode with detailed operation preview
- Comprehensive error handling and recovery strategies
- Progress tracking and performance metrics
- Backup creation before operations
- Flexible filtering criteria for source items
- Automatic cleanup and finalization
- Tool composition with decision points

**Requirements Validated**:
- **8.3**: Workflow templates for batch file operations
- **8.4**: Templates configurable through workflow parameters

**Usage**: `cargo run --example interactive-batch-processing-example`

### Interactive Merge
**File**: `interactive-merge-workflow.yaml`  
**Purpose**: Folder merging with multiple strategies  
**Key Features**:
- Multiple merge strategies (SmallerToLarger, LargerToSmaller, UserDecision, TargetDirectory)
- Human decision-making for merge strategies and duplicate conflicts
- Experimental mode with detailed preview and confirmation
- Comprehensive size analysis and space savings calculation
- Backup creation before merge operations
- Duplicate file conflict resolution with multiple options
- Recursive merging with configurable depth limits
- Detailed verification and reporting

**Requirements Validated**:
- **8.2**: Workflow templates for folder merging
- **11.5**: Human decision-making modes
- **12.5**: Experimental mode with confirmation steps

**Usage**: `cargo run --example interactive-merge-example`

## CONFIGURATION TEMPLATES

### Environment Config Examples
**File**: `environment-config-examples.yaml`  
Demonstrates hierarchical configuration with environment variables:
- **Development**: Debug settings, small batches, safety features
- **Testing**: Automated processing, validation, profiling
- **Production**: Optimized performance, audit logging, monitoring
- **Enterprise**: Governance, compliance, security features
- **High-performance**: Large dataset optimization, streaming mode
- **Cloud**: Auto-scaling, distributed processing, cost optimization

### Common Use Cases
**File**: `common-use-cases.yaml`  
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

### Workflow Composition Examples
**File**: `workflow-composition-examples.yaml`  
Advanced workflow patterns demonstrating:
- **Multi-stage classification with learning**: Progressive improvement from human decisions
- **Intelligent folder consolidation**: Complex merge operations with conflict resolution
- **Robust batch processing**: Comprehensive error handling and recovery
- **Enterprise document workflow**: Governance, compliance, and audit features

## DOCUMENTATION

### Configuration Guide
**File**: `CONFIGURATION_GUIDE.md`  
Complete configuration reference with examples covering:
- Core parameters: Directory paths, classification rules, confidence thresholds
- Human decision parameters: Timeouts, batch decisions, escalation
- Performance parameters: Batch sizes, concurrency, memory management
- Security parameters: Access control, audit logging, compliance
- Environment variables: Substitution syntax, common variables, best practices

### Human Decision Best Practices
**File**: `HUMAN_DECISION_BEST_PRACTICES.md`  
Guidelines for interactive workflows and human-in-the-loop patterns:
- **Decision timeout configuration**: Appropriate timeouts for complexity
- **Decision context optimization**: Rich context for better decisions
- **Batch decision strategies**: Optimize human workflows
- **Progressive decision making**: Minimize human intervention
- **Decision audit and learning**: Track and learn from decisions

### Parameter Documentation
**File**: `PARAMETER_DOCUMENTATION.md`  
Template parameter syntax and expansion examples:
- Core parameters: Directory paths, classification rules, confidence thresholds
- Human decision parameters: Timeouts, batch decisions, escalation
- Performance parameters: Batch sizes, concurrency, memory management
- Security parameters: Access control, audit logging, compliance
- Environment variables: Substitution syntax, common variables, best practices

### Usage Examples
**File**: `USAGE_EXAMPLES.md`  
Step-by-step tutorials for common scenarios:
- Interactive folder classification
- Interactive folder merge
- Interactive batch processing
- Custom configuration
- Advanced features

### README
**File**: `README.md`  
Overview of all templates and how to use them:
- Template descriptions
- Quick start guide
- Configuration examples
- Best practices
- Troubleshooting

## RULES & RULE FILES

### Classification Rules
**Files**:
- `classification-rules-example.json`: Basic rules
- `classification-rules-comprehensive.json`: Advanced rules
- `classification-rules-chinese.json`: Chinese-specific rules

**Rule Structure**:
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
    }
  },
  "settings": {
    "minimum_score_threshold": 0.5,
    "case_sensitive": false,
    "enable_chinese_processing": false
  }
}
```

## EXAMPLE IMPLEMENTATIONS

Each workflow template has a corresponding Rust example:
- `interactive-classification-example.rs`: AI classification with human review
- `interactive-batch-processing-example.rs`: Bulk operations with oversight
- `interactive-merge-example.rs`: Folder merging with decisions
- `real-world-scenario-example.rs`: Creative agency asset management

These demonstrate:
- Programmatic workflow creation
- Custom tool integration
- Error handling patterns
- State management
- Human decision integration
- Performance optimization

## USAGE EXAMPLES

### Interactive Folder Classification

#### Basic Usage
```bash
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized/folders" \
  --param classification_rules="examples/templates/classification-rules-example.json"
```

#### Experimental Mode (Recommended for First Run)
```bash
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized/folders" \
  --param classification_rules="examples/templates/classification-rules-example.json" \
  --param experimental_mode=true \
  --param enable_user_interaction=true
```

#### Production Mode (Auto-Execute)
```bash
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
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/path/to/dir1", "/path/to/dir2", "/path/to/dir3"]' \
  --param target_directory="/path/to/merged/output" \
  --param merge_strategy="UserDecision" \
  --param experimental_mode=true
```

#### Automatic Merge Strategy
```bash
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/home/user/Downloads", "/home/user/Documents"]' \
  --param merge_strategy="SmallerToLarger" \
  --param duplicate_handling="Rename" \
  --param experimental_mode=false
```

#### Production Merge with Backup
```bash
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
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/path/to/source/files" \
  --param target_directory="/path/to/target/location" \
  --param operation_type="move" \
  --param experimental_mode=true \
  --param enable_user_interaction=true
```

#### Batch Copy with Conflict Resolution
```bash
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
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/data/files" \
  --param target_directory="/data/organized" \
  --param operation_type="move" \
  --param experimental_mode=false \
  --param enable_user_interaction=false \
  --param conflict_resolution="Skip" \
  --param progress_reporting=true
```

## WORKFLOW FLOWS

### Interactive Classification Workflow
```
1. 📁 Scan Folders → Discover all folders
2. ✅ Validate Rules → Verify classification rules
3. 🔄 Classify Batch → Process in parallel batches
4. 📊 Review Results → Display statistics
5. ❓ Experimental Check → Branch based on mode
6. 👤 Human Confirmation → Get user approval
7. ⚡ Execute Operations → Move folders
8. 📋 Generate Report → Create detailed report
9. 🧹 Cleanup → Remove empty directories
```

### Interactive Merge Workflow
```
1. 📁 Scan Directories → Discover folders
2. 🔍 Find Common Folders → Identify duplicates
3. 📊 Analyze Merge Candidates → Calculate sizes
4. 📋 Review Analysis → Display statistics
5. 🎯 Determine Strategies → Choose merge strategy
6. 👤 Decide Strategies → Get user decisions
7. 📝 Create Merge Plan → Generate execution plan
8. ⚠️ Handle Conflicts → Resolve duplicates
9. ✅ Finalize Plan → Complete plan
10. 📊 Review Final Plan → Display final plan
11. ❓ Experimental Check → Branch based on mode
12. 👤 Confirm Execution → Get user confirmation
13. 💾 Create Backup → Backup before merge
14. ⚡ Execute Merge → Perform merge
15. 🔍 Verify Results → Verify operations
16. 📋 Generate Report → Create comprehensive report
17. 🧹 Cleanup → Remove empty directories
```

### Interactive Batch Processing Workflow
```
1. 📁 Scan Source Items → Discover and filter items
2. ✅ Validate Configuration → Verify configuration
3. 📊 Analyze Items → Create processing plan
4. 📋 Review Plan → Display plan and statistics
5. ⚠️ Handle Conflicts → Resolve conflicts
6. ✅ Finalize Plan → Complete processing plan
7. 📊 Review Final Plan → Display final plan
8. ❓ Experimental Check → Branch based on mode
9. 👤 Confirm Execution → Get user confirmation
10. 💾 Create Backup → Backup before operations
11. ⚡ Execute Batches → Perform batch processing
12. 🔍 Verify Results → Verify operations
13. 🚨 Handle Failures → Handle failures
14. 📋 Generate Report → Create comprehensive report
15. 🧹 Cleanup → Clean up after operations
16. 📊 Final Summary → Generate final status
```

## HUMAN DECISION POINTS

### Interactive Classification Workflow
1. **Ambiguous Classifications**: Multiple categories with similar scores
2. **Experimental Mode Confirmation**: Preview and confirm operations
3. **Error Recovery**: Handle errors during execution

### Interactive Merge Workflow
1. **Merge Strategy Selection**: Choose strategy per folder group
2. **Duplicate File Conflicts**: Resolve file conflicts
3. **Merge Plan Confirmation**: Confirm final plan
4. **Error Recovery and Rollback**: Handle errors and rollback

### Interactive Batch Processing Workflow
1. **Conflict Resolution**: Resolve file conflicts
2. **Batch Plan Confirmation**: Confirm processing plan
3. **Failure Handling**: Handle operation failures
4. **Operation Configuration**: Configure custom operations

## ADVANCED FEATURES

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

## TOOL COMPOSITION PATTERNS

### Classification + File Operations
```yaml
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

### Merge + Batch Processing
```yaml
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

### Human Decision + Automation
```yaml
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

## BEST PRACTICES

### General Workflow Best Practices
1. **Always start with experimental mode** for new classification rules
2. **Use appropriate batch sizes** based on your system resources
3. **Set reasonable timeouts** for human decisions
4. **Test classification rules** on a small subset first
5. **Monitor workflow logs** for performance optimization
6. **Use version control** for classification rule files
7. **Backup important data** before running in production mode

### Human Decision Integration Best Practices
1. **Configure appropriate timeouts** based on decision complexity
2. **Provide rich context** for better human decisions
3. **Optimize batch decision workflows** for batch operations
4. **Structure workflows** to minimize human intervention
5. **Track and learn from human decisions** to improve automation

### Tool Composition Best Practices
1. **Combine classification with file operations** for complete workflows
2. **Combine merge operations with batch processing** for efficiency
3. **Enhance classification with text processing** for better accuracy
4. **Balance human decisions with automation** for optimal results

### Performance Optimization Best Practices
1. **Choose optimal batch sizes** for different scenarios
2. **Configure memory usage** for large datasets
3. **Optimize I/O operations** for file systems
4. **Use streaming mode** for very large datasets

### Error Handling Best Practices
1. **Handle errors gracefully** without stopping workflows
2. **Provide detailed error context** for troubleshooting
3. **Implement robust rollback mechanisms** for critical operations
4. **Use safe operation modes** for critical data

### Security and Safety Best Practices
1. **Use safe operation modes** for critical data
2. **Validate data integrity** throughout operations
3. **Implement proper access control**
4. **Log all operations** for audit trails

## TROUBLESHOOTING

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

## CONTRIBUTING

To add new workflow templates:

1. **Create the template YAML file**
2. **Add example configuration files**
3. **Create a usage example in Rust**
4. **Update this README with documentation**
5. **Add integration tests**

Follow the existing patterns for consistency and maintainability.

## SEE ALSO

- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [File Management AGENTS.md](../../src/plugins/file_management/AGENTS.md) - File operations
- [Workflow AGENTS.md](../../src/workflow/AGENTS.md) - Workflow engine
- [Examples AGENTS.md](../AGENTS.md) - Example implementations
