# src/plugins/file_management/ - File Operations Plugin

## OVERVIEW
Specialized plugin for file operations with AI-powered classification, batch processing, text operations, and human decision support. 20 files with 3,604+ total lines in utils.rs alone.

## TOOL COMPONENTS

### Classification Tools

#### FolderClassifier
**File**: `classification_tool.rs` (1,527 lines)  
**Purpose**: AI-powered folder categorization with confidence scoring  
**Features**:
- Pattern matching using Aho-Corasick algorithm
- Content analysis for classification
- Confidence scoring (0.0-1.0) per category
- Multiple category candidates
- Experimental mode with advanced heuristics
- Chinese text processing support
- Rule-based classification with JSON rules

**Classification Flow**:
1. **Scan**: Discover folders in source directory
2. **Preprocess**: Clean and normalize folder names
3. **Match**: Apply pattern matching against rules
4. **Score**: Calculate confidence scores
5. **Rank**: Sort candidates by score
6. **Decide**: Auto-classify or request human input

#### ClassificationFlow Tools
**File**: `classification_flow.rs`  
**Purpose**: Modular classification pipeline components  
**Components**:
- `DirectoryScannerTool`: Discover folders
- `FolderNamePreprocessorTool`: Clean and normalize names
- `AutomatonBuilderTool`: Build Aho-Corasick automaton
- `ParallelMatcherTool`: Parallel pattern matching
- `ScoreCalculatorTool`: Calculate confidence scores
- `AmbiguityDetectorTool`: Detect ambiguous classifications
- `ResultMergerTool`: Merge multiple classification results
- `ReportGeneratorTool`: Generate classification reports
- `RuleLoaderTool`: Load and validate classification rules
- `RulePreprocessorTool`: Preprocess rules for efficiency
- `ExperimentalCheckTool`: Apply experimental heuristics

### Batch Processing Tools

#### BatchProcessor
**File**: `batch_processor.rs`  
**Purpose**: Generic batch processing engine  
**Features**:
- Configurable batch sizes
- Parallel execution with concurrency limits
- Progress tracking with real-time updates
- Error isolation (failures don't stop entire batch)
- Performance metrics collection
- Result aggregation

#### BatchProcessorTool
**File**: `batch_processor_tool.rs`  
**Purpose**: Batch processing as a tool  
**Features**:
- Generic operation support (move, copy, classify, merge, custom)
- Operation configuration via JSON
- Conflict resolution strategies
- Human decision integration
- Result confirmation before commit
- Rollback capability

**Batch Processing Flow**:
1. **Scan**: Discover items to process
2. **Filter**: Apply filter criteria
3. **Analyze**: Detect conflicts and issues
4. **Plan**: Create execution plan
5. **Confirm**: Human confirmation (if enabled)
6. **Execute**: Process batches with progress tracking
7. **Verify**: Validate results
8. **Report**: Generate comprehensive report

### Text Processing Tools

#### TextProcessorTool
**File**: `text_processor_tool.rs`  
**Purpose**: Advanced text analysis and transformation  
**Features**:
- **Extraction**: Parse structured data from text
- **Analysis**: Content classification and sentiment
- **Transformation**: Format conversion and normalization
- **Summarization**: Content condensation
- **Chinese Processing**: Pinyin conversion, traditional/simplified

**Text Operations**:
- `NormalizeCase`: Convert to consistent case
- `ConvertTraditional`: Traditional to simplified Chinese
- `GeneratePinyin`: Generate pinyin from Chinese text
- `ExtractKeywords`: Extract key terms
- `AnalyzeSentiment`: Determine sentiment
- `Summarize`: Create text summaries

**Chinese Processing Config**:
```rust
pub struct ChineseProcessingConfig {
    pub pinyin_style: PinyinStyle,
    pub normalize_unicode: bool,
    pub handle_traditional: bool,
    pub generate_combinations: bool,
}
```

### Human Decision Tools

#### HumanDecisionTool
**File**: `human_decision_tool.rs`  
**Purpose**: Interactive decision support for workflows  
**Features**:
- **Interactive Prompts**: CLI/TUI for decisions
- **Override Capability**: Accept/reject/suggest alternatives
- **Batch Approval**: Approve multiple items at once
- **Context Display**: Rich context for informed decisions
- **Timeout Handling**: Configurable decision timeouts
- **Audit Trail**: All decisions logged

**Decision Types**:
- `Classification`: Choose category for ambiguous folders
- `FileConflict`: Resolve duplicate file conflicts
- `MergeStrategy`: Select folder merge strategy
- `BatchConfirmation`: Approve/reject batch operations
- `Custom`: User-defined decision types

#### BatchConfirmationTool
**File**: `batch_confirmation_tool.rs`  
**Purpose**: Batch-level confirmation with summary  
**Features**:
- **Operation Batching**: Group similar operations
- **Risk Assessment**: Calculate operation risks
- **Summary Generation**: Create operation summaries
- **User Preferences**: Remember user choices
- **Reversibility Analysis**: Check if operations can be undone
- **Recommended Actions**: Suggest best actions

#### ResultReviewTool
**File**: `result_review_tool.rs`  
**Purpose**: Review and validate operation results  
**Features**:
- **Preview Mode**: Show what will happen before execution
- **Impact Analysis**: Assess operation impact
- **Risk Level**: Calculate risk levels
- **Experimental Results**: Show simulated results
- **Confirmation Options**: Multiple confirmation methods

#### ResultConfirmationTool
**File**: `result_confirmation_tool.rs`  
**Purpose**: Final confirmation before commit  
**Features**:
- **Backup Strategy**: Configure backup before operations
- **Rollback Planning**: Create rollback plans
- **Phase Management**: Multi-phase confirmation
- **Resource Requirements**: Check system resources
- **Execution Summary**: Final operation summary

### Utility Tools

#### FolderMerger
**File**: `utils.rs` (3,604 lines)  
**Purpose**: Intelligent folder merging with multiple strategies  
**Features**:
- **Multiple Merge Strategies**:
  - `SmallerToLarger`: Merge smaller into larger
  - `LargerToSmaller`: Merge larger into smaller
  - `UserDecision`: Human chooses strategy
  - `TargetDirectory`: Merge to specified target
  
- **Conflict Resolution**:
  - `Skip`: Skip duplicate files
  - `Rename`: Rename to avoid conflicts
  - `KeepNewer`: Keep more recent file
  - `KeepLarger`: Keep larger file
  - `Merge`: Merge file contents (if possible)
  - `KeepBoth`: Keep both with different names
  
- **Analysis Features**:
  - Size analysis and space savings calculation
  - Duplicate detection with content comparison
  - Conflict identification and resolution
  - Merge plan generation with preview
  
- **Safety Features**:
  - Backup creation before merge
  - Rollback capability
  - Verification of operations
  - Detailed logging and audit trail

#### FileOperationManager
**File**: `utils.rs`  
**Purpose**: Core file operations with safety checks  
**Features**:
- **Move Operations**: Safe file/directory moving
- **Copy Operations**: Preserves metadata and permissions
- **Link Operations**: Hard and symbolic links
- **Cleanup**: Remove empty directories
- **Validation**: Path and permission validation

#### PathUtils
**File**: `utils.rs`  
**Purpose**: Path manipulation and validation  
**Features**:
- **Path Normalization**: Clean and normalize paths
- **Relative Path Calculation**: Compute relative paths
- **Path Validation**: Check for traversal attacks
- **Directory Traversal**: Safe recursive directory scanning
- **Path Filtering**: Filter paths by criteria

#### TextProcessor
**File**: `utils.rs`  
**Purpose**: Text analysis and transformation  
**Features**:
- **Pinyin Conversion**: Chinese to pinyin with multiple styles
- **Text Normalization**: Unicode normalization
- **Traditional/Simplified**: Conversion between Chinese variants
- **Keyword Extraction**: Extract meaningful keywords
- **Pattern Matching**: Regex and pattern-based matching

**Pinyin Styles**:
- `Normal`: ni3 hao3
- `WithTone`: nǐ hǎo
- `WithoutTone`: ni hao
- `FirstLetter`: n h
- `Numeric`: ni3 hao3

**Chinese Text Types**:
- `None`: No Chinese text
- `Simplified`: Simplified Chinese
- `Traditional`: Traditional Chinese
- `Mixed`: Both simplified and traditional
- `Unknown`: Cannot determine

### Error Handling

#### FileManagementError
**File**: `error.rs` (1,284 lines)  
**Purpose**: Comprehensive error types for file operations  
**Error Types**:
- `NotFound`: File or directory not found
- `PermissionDenied`: Access denied
- `InsufficientSpace`: Disk space issues
- `Conflict`: Operation conflict (e.g., destination exists)
- `InvalidPath`: Invalid file path or name
- `TextProcessing`: Text analysis errors
- `Classification`: Classification failures
- `HumanDecision`: Decision-related errors
- `BatchProcessing`: Batch operation errors
- `MergeError`: Folder merge failures

**Error Context**:
```rust
pub struct ErrorContext {
    pub operation: String,
    pub path: Option<PathBuf>,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub workflow_id: Option<Uuid>,
    pub node_id: Option<String>,
}
```

#### ErrorRecovery
**File**: `error_recovery.rs`  
**Purpose**: Recovery strategies for file operation errors  
**Features**:
- **Recovery Strategies**: Multiple recovery approaches
- **Confidence Scoring**: Strategy effectiveness prediction
- **Estimated Recovery Time**: Time estimates for recovery
- **Error Classification**: Categorize errors for appropriate recovery
- **Session Management**: Track recovery attempts

**Recovery Strategies**:
- `Retry`: Retry with exponential backoff
- `Skip`: Skip problematic item
- `UseDefault`: Use default values
- `RestartComponent`: Restart the component
- `ShowDialog`: Request user intervention
- `Degrade`: Graceful degradation
- `Manual`: Manual intervention required

### Monitoring and Performance

#### FileManagementMonitor
**File**: `monitoring.rs`  
**Purpose**: Real-time monitoring of file operations  
**Features**:
- **Operation Metrics**: Track operation counts and durations
- **Error Tracking**: Monitor error rates and types
- **Alert System**: Configurable alerts for issues
- **Audit Logging**: Comprehensive operation audit trail
- **Resource Usage**: Monitor system resource consumption

#### Performance
**File**: `performance.rs`  
**Purpose**: Performance optimization for file operations  
**Features**:
- **Memory Pool**: Efficient memory management
- **Compression**: Optional data compression
- **Streaming**: Stream processing for large files
- **Caching**: Result caching for repeated operations
- **Optimization**: Performance analysis and suggestions

#### ProgressTracker
**File**: `progress_tracker.rs`  
**Purpose**: Progress tracking for batch operations  
**Features**:
- **Real-time Updates**: Live progress updates
- **Per-item Tracking**: Track individual item progress
- **Aggregated Stats**: Summary statistics
- **Performance Trends**: Track performance over time
- **Tool Usage Stats**: Monitor tool usage patterns

## CLASSIFICATION SYSTEM

### Rule-Based Classification
**File**: `rule_config.rs` (in development)  
**Purpose**: Define classification rules in JSON format

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
    },
    "media": {
      "description": "Images, videos, and audio",
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

### AI-Powered Classification
- **Pattern Matching**: Aho-Corasick algorithm for efficient keyword matching
- **Content Analysis**: Analyze folder contents for better classification
- **Confidence Scoring**: Calculate confidence for each category
- **Multiple Candidates**: Return top N candidates for human review
- **Learning Mode**: Improve rules based on human decisions

### Chinese Text Processing
- **Pinyin Conversion**: Convert Chinese characters to pinyin
- **Traditional/Simplified**: Automatic conversion support
- **Mixed Language**: Handle Chinese-English content
- **Keyword Extraction**: Extract meaningful terms from Chinese text
- **Pattern Matching**: Support Chinese patterns in rules

## BATCH PROCESSING SYSTEM

### Batch Configuration
```rust
pub struct BatchProcessorConfig {
    pub batch_size: usize,
    pub max_concurrent_batches: usize,
    pub continue_on_error: bool,
    pub max_consecutive_errors: usize,
    pub progress_reporting: bool,
    pub create_backup: bool,
}
```

### Batch Processing Flow
```
1. Scan Source
   ↓
2. Filter Items
   ↓
3. Analyze Conflicts
   ↓
4. Create Plan
   ↓
5. Human Confirmation (if enabled)
   ↓
6. Execute Batches
   ├─ Batch 1 (parallel)
   ├─ Batch 2 (parallel)
   └─ Batch N (parallel)
   ↓
7. Verify Results
   ↓
8. Handle Failures (if any)
   ↓
9. Generate Report
   ↓
10. Cleanup
```

### Error Handling in Batches
- **Isolation**: Errors in one item don't stop entire batch
- **Retry**: Configurable retry for transient errors
- **Skip**: Skip failed items and continue
- **Rollback**: Rollback entire batch on critical errors
- **Reporting**: Detailed error reporting per item

## HUMAN DECISION INTEGRATION

### Decision Points
1. **Ambiguous Classifications**: Multiple categories with similar scores
2. **File Conflicts**: Duplicate files during merge/copy
3. **Merge Strategies**: Choose folder merge strategy
4. **Batch Confirmation**: Approve/reject batch operations
5. **Error Recovery**: Choose recovery strategy for errors

### Decision Context
```rust
pub struct DecisionContext {
    pub decision_type: HumanDecisionType,
    pub title: String,
    pub description: String,
    pub options: Vec<DecisionOption>,
    pub metadata: HashMap<String, Value>,
    pub timeout_seconds: Option<u64>,
}
```

### Decision Options
```rust
pub struct DecisionOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub recommended: bool,
    pub risk_level: RiskLevel,
}
```

### Batch Decisions
- **Group Similar**: Group similar decisions together
- **Apply to All**: Apply decision to all similar items
- **Pattern Recognition**: Suggest patterns based on previous decisions
- **Learning**: Improve automation based on human decisions

## MERGE STRATEGIES

### Folder Merge Strategies
1. **SmallerToLarger**: Merge smaller folders into the largest one
2. **LargerToSmaller**: Merge larger folders into the smallest one
3. **UserDecision**: Human chooses strategy per group
4. **TargetDirectory**: Merge all to specified target

### Duplicate File Handling
1. **Skip**: Skip duplicate files
2. **Rename**: Rename to avoid conflicts
3. **KeepNewer**: Keep more recent file
4. **KeepLarger**: Keep larger file
5. **KeepBoth**: Keep both with different names
6. **Merge**: Merge file contents (if possible)

### Merge Safety
- **Backup Creation**: Optional backup before merge
- **Rollback Capability**: Ability to undo merge
- **Verification**: Verify merge results
- **Conflict Detection**: Detect conflicts before merge
- **Space Analysis**: Calculate space savings

## PLUGIN ARCHITECTURE

### Plugin Registration
**File**: `plugin.rs`  
**Purpose**: Plugin lifecycle management  
**Features**:
- **Plugin Builder**: Fluent API for plugin creation
- **Configuration**: Configurable plugin behavior
- **Tool Registration**: Automatic tool registration
- **Lifecycle Hooks**: Before/after operation hooks
- **Resource Management**: Memory and file handle management

### Plugin Configuration
```rust
pub struct FileManagementConfig {
    pub directory: PathBuf,
    pub rules_file: Option<PathBuf>,
    pub experimental_mode: bool,
    pub enable_human_interaction: bool,
    pub decision_timeout: Duration,
    pub batch_size: usize,
    pub max_concurrent: usize,
}
```

### Tool Registry
**File**: `registry.rs` (1,773 lines)  
**Purpose**: Central registry for all file management tools  
**Tools Registered**:
- `folder-classifier`: Classification tool
- `batch-processor`: Batch processing tool
- `text-processor`: Text analysis tool
- `human-decision`: Interactive decision tool
- `batch-confirmation`: Batch confirmation tool
- `result-review`: Result review tool
- `result-confirmation`: Final confirmation tool
- `folder-merger`: Folder merging tool

## USAGE EXAMPLES

### Basic Classification
```rust
use workflow_toolkit::plugins::file_management::{FileManagementPlugin, FileManagementConfig};

let plugin = FileManagementPlugin::builder()
    .config(FileManagementConfig {
        directory: "./data".into(),
        rules_file: Some("classification-rules.json".into()),
        experimental_mode: true,
        enable_human_interaction: true,
        decision_timeout: Duration::from_secs(300),
        batch_size: 10,
        max_concurrent: 4,
    })
    .build()?;

let result = plugin.classify_folders("./data/unorganized").await?;
```

### Batch Processing
```rust
use workflow_toolkit::plugins::file_management::{BatchProcessorTool, BatchProcessorParams};

let tool = BatchProcessorTool::new();
let params = BatchProcessorParams {
    source_directory: "./data/incoming".into(),
    target_directory: "./data/processed".into(),
    operation_type: "move".into(),
    batch_size: 25,
    max_concurrent_batches: 4,
    experimental_mode: true,
    enable_human_interaction: true,
    conflict_resolution: "Rename".into(),
};

let result = tool.execute(params).await?;
```

### Folder Merge
```rust
use workflow_toolkit::plugins::file_management::{FolderMerger, MergeStrategy};

let merger = FolderMerger::new();
let config = FolderMergerConfig {
    source_directories: vec!["./dir1".into(), "./dir2".into()],
    target_directory: Some("./merged".into()),
    merge_strategy: MergeStrategy::SmallerToLarger,
    duplicate_handling: DuplicateHandling::Rename,
    experimental_mode: true,
    create_backup: true,
};

let result = merger.merge_folders(config).await?;
```

### Text Processing
```rust
use workflow_toolkit::plugins::file_management::{TextProcessorTool, TextProcessorParams};

let tool = TextProcessorTool::new();
let params = TextProcessorParams {
    text: "中文文本示例".to_string(),
    operations: vec![
        TextOperation::GeneratePinyin,
        TextOperation::NormalizeCase,
        TextOperation::ExtractKeywords,
    ],
    chinese_processing: ChineseProcessingConfig {
        pinyin_style: PinyinStyle::Normal,
        normalize_unicode: true,
        handle_traditional: true,
        generate_combinations: false,
    },
};

let result = tool.execute(params).await?;
```

## TESTING

### Unit Tests
- Classification rule validation
- Pattern matching accuracy
- Batch processing logic
- Error recovery strategies
- Text processing functions

### Integration Tests
**File**: `tests/file_management_integration_tests.rs` (1,431 lines)  
**Tests**:
- End-to-end classification workflows
- Batch processing with various configurations
- Human decision integration
- Error recovery scenarios
- Performance benchmarks

### Test Fixtures
```rust
struct FileManagementTestFixture {
    temp_dir: TempDir,
    plugin: Arc<FileManagementPlugin>,
    rules: ClassificationRules,
    test_data: PathBuf,
}
```

## PERFORMANCE OPTIMIZATION

### Memory Management
- **Streaming Processing**: Process large files without loading entirely into memory
- **Memory Pool**: Reuse memory allocations
- **Batch Processing**: Process in batches to limit memory usage
- **Garbage Collection**: Explicit cleanup of temporary data

### I/O Optimization
- **Async I/O**: Non-blocking file operations
- **Buffered Operations**: Efficient read/write buffering
- **Parallel Processing**: Concurrent file operations
- **Cache Results**: Cache classification results

### Algorithm Optimization
- **Aho-Corasick**: Efficient multi-pattern matching
- **Parallel Matching**: Distribute pattern matching across threads
- **Lazy Evaluation**: Defer expensive operations until needed
- **Result Caching**: Cache expensive computations

## ERROR HANDLING

### Comprehensive Error Types
**File**: `error.rs` (1,284 lines)  
**Error Categories**:
- File system errors (not found, permission denied)
- Classification errors (invalid rules, low confidence)
- Batch processing errors (conflicts, failures)
- Human decision errors (timeout, invalid choice)
- Text processing errors (encoding, parsing)
- Merge errors (conflicts, space issues)

### Error Recovery
**File**: `error_recovery.rs`  
**Recovery Strategies**:
- Automatic retry with exponential backoff
- Skip problematic items and continue
- Request human intervention
- Rollback to previous state
- Degrade functionality gracefully

### Error Context
All errors include rich context:
- Operation being performed
- File/directory paths involved
- Timestamp and user information
- Workflow and node identifiers
- Suggested recovery actions

## SECURITY CONSIDERATIONS

### Path Validation
- **Path Traversal Prevention**: Block `../` and absolute paths
- **Permission Checks**: Verify write permissions before operations
- **Sandboxing**: Restrict operations to configured directories
- **Input Sanitization**: Clean user input to prevent injection attacks

### Data Safety
- **Backup Creation**: Optional backup before destructive operations
- **Rollback Capability**: Ability to undo operations
- **Verification**: Verify operations completed successfully
- **Audit Logging**: Comprehensive audit trail for all operations

### Resource Limits
- **Memory Limits**: Prevent memory exhaustion
- **File Handle Limits**: Manage file handle usage
- **Timeout Handling**: Prevent hanging operations
- **Concurrent Limits**: Limit concurrent operations

## BEST PRACTICES

### Classification Rules
1. **Start Simple**: Begin with basic keyword rules
2. **Test Thoroughly**: Test on small dataset first
3. **Use Confidence Thresholds**: Set appropriate thresholds
4. **Enable Learning**: Improve rules based on human decisions
5. **Version Rules**: Track rule changes over time

### Batch Processing
1. **Start Small**: Begin with small batch sizes
2. **Use Experimental Mode**: Test before production
3. **Enable Human Review**: Review critical operations
4. **Monitor Progress**: Track batch processing in real-time
5. **Verify Results**: Always verify batch completion

### Human Interaction
1. **Provide Context**: Show rich context for decisions
2. **Set Timeouts**: Prevent indefinite waiting
3. **Batch Similar Decisions**: Group similar decisions
4. **Learn from Decisions**: Improve automation over time
5. **Log Decisions**: Maintain audit trail

### Error Handling
1. **Graceful Degradation**: Continue on non-critical errors
2. **Rich Error Context**: Include all relevant information
3. **Recovery Options**: Provide multiple recovery strategies
4. **User Notification**: Inform users of errors promptly
5. **Error Analysis**: Track and analyze error patterns

## ARCHITECTURE DECISIONS

### Why Aho-Corasick?
1. **Efficiency**: O(n + m) time complexity for pattern matching
2. **Multiple Patterns**: Match thousands of patterns simultaneously
3. **Memory Efficient**: Compact automaton representation
4. **Well-Tested**: Proven algorithm with decades of use

### Why Human Integration?
1. **Accuracy**: Humans provide better decisions for ambiguous cases
2. **Learning**: Human decisions improve automation over time
3. **Trust**: Users trust systems they can control
4. **Flexibility**: Handle edge cases not covered by rules

### Why Batch Processing?
1. **Efficiency**: Process multiple items in parallel
2. **Progress Tracking**: Real-time feedback to users
3. **Error Isolation**: Failures don't stop entire operation
4. **Resource Management**: Controlled resource usage

## FUTURE ENHANCEMENTS

### Planned Features
1. **Machine Learning**: Train classification models from human decisions
2. **Content Analysis**: Analyze file contents for better classification
3. **Cloud Integration**: Support cloud storage providers
4. **Distributed Processing**: Scale across multiple machines
5. **Advanced Analytics**: Detailed performance and usage analytics

### Experimental Features
1. **WASM Plugins**: WebAssembly-based custom operations
2. **Neural Networks**: Deep learning for classification
3. **Real-time Collaboration**: Multi-user decision support
4. **Predictive Analytics**: Predict user preferences

## SEE ALSO

- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Plugins AGENTS.md](../AGENTS.md) - Plugin system
- [Tools AGENTS.md](../../tools/AGENTS.md) - Tool system
- [Workflow AGENTS.md](../../workflow/AGENTS.md) - Workflow engine
- [Examples](../../../workflows/templates/AGENTS.md) - Usage examples

### 模块级补充细则
- 目标与范围
  - 本模块聚焦文件操作插件的实现、稳定性和可审计性，确保与工作流引擎、存储层和用户接口的契约清晰。
- 设计与扩展
  - 新工具/子模块应提供明确的接口、输入/输出和错误契约。
- 实现规范
  - 导入排序与命名遵循仓库通用规范。
  - 对外接口需要 Rustdoc 注释，示例与使用案例描述清晰。
  - 错误处理集中化，统一返回 Result<T, WorkflowError>（若暴露公共 API）。
  - 使用异步模式，避免阻塞 IO。
- 测试策略
  - 覆盖分类、批处理、文本处理、人机决策等核心路径的单元与集成测试。
  - 针对高风险操作（如批量修改、合并）增加回归测试。
- 变更与审阅
  - 变更前提供设计动机、影响评估和回归测试计划。
- 文档与审阅
  - 模块级 AGENTS.md 变更需同步。
- 跨模块协作
  - 与工作流、存储、接口的对接契约需要提前沟通，避免不兼容。
- Cursor/Copilot 规则
  - 将 Cursor/Copilot 规则合并到模块级 AGENTS.md 模板中，便于执行。
