# File Management Plugin 设计文档

## 1. 核心定义 (Stable)

### 1.1 模块职责

| 模块 | 职责 | 关键文件 |
|------|------|----------|
| `core` | 错误处理、恢复机制、共享类型定义 | error.rs, error_recovery.rs, types.rs |
| `classification` | 文件夹分类工具集，包含分类引擎、规则处理、流程编排 | classification_tool.rs, classification_flow.rs, rule_config.rs |
| `batch` | 批处理工具，包含批处理器、进度追踪 | batch_processor.rs, batch_processor_tool.rs, progress_tracker.rs |
| `text` | 文本处理工具，支持中文处理、多种输出格式 | text_processor_tool.rs |
| `ui` | 人工决策和确认工具集，包含批量确认、结果审核 | human_decision_tool.rs, batch_confirmation_tool.rs, result_confirmation_tool.rs, result_review_tool.rs |
| `utils` | 工具函数、监控、性能优化、注册表 | utils.rs, monitoring.rs, performance.rs, registry.rs |
| `plugin` | 插件定义与配置 | plugin.rs |

### 1.2 模块结构图

```
file_management/
├── mod.rs              # 模块入口，重导出类型
├── plugin.rs           # 插件定义
├── core/               # 核心模块
│   ├── mod.rs
│   ├── error.rs        # 错误类型定义
│   ├── error_recovery.rs # 错误恢复机制
│   └── types.rs        # 共享类型
├── classification/     # 分类模块
│   ├── mod.rs
│   ├── classification_tool.rs
│   ├── classification_flow.rs
│   └── rule_config.rs
├── batch/              # 批处理模块
│   ├── mod.rs
│   ├── batch_processor.rs
│   ├── batch_processor_tool.rs
│   └── progress_tracker.rs
├── text/               # 文本处理模块
│   ├── mod.rs
│   └── text_processor_tool.rs
├── ui/                 # 用户交互模块
│   ├── mod.rs
│   ├── human_decision_tool.rs
│   ├── batch_confirmation_tool.rs
│   ├── result_confirmation_tool.rs
│   └── result_review_tool.rs
└── utils/              # 工具模块
    ├── mod.rs
    ├── utils.rs
    ├── monitoring.rs
    ├── performance.rs
    └── registry.rs
```

### 1.3 核心类型导出

#### 插件层
```rust
pub use plugin::{
    FileManagementConfig,
    FileManagementPerformanceConfig,
    FileManagementPlugin,
    FileManagementPluginBuilder,
};
```

#### Core 模块
```rust
pub use core::{
    ErrorContext, ErrorSeverity,
    FileManagementError, FileManagementResult, RecoverySuggestion,
    error_recovery::{
        ErrorRecoveryManager, RecoveryAttempt, RecoveryConfig, RecoverySession,
        RecoveryStats, RecoveryStrategy,
    },
};
```

#### Classification 模块
```rust
pub use classification::{
    ClassificationTool, ClassificationEngine, ClassificationRule, ClassificationRules,
    ClassificationResult, ClassificationStatus, ClassificationCandidate,
    ClassificationParams, ClassificationOutputFormat,
    AmbiguityDetectorTool, AutomatonBuilderTool, DirectoryScannerTool,
    ExperimentalCheckTool, FolderNamePreprocessorTool, ParallelMatcherTool,
    ReportGeneratorTool, ResultMergerTool, RuleLoaderTool, RulePreprocessorTool,
    ScoreCalculatorTool,
};
```

#### Batch 模块
```rust
pub use batch::{
    BatchProcessor, BatchProcessorTool, BatchProgress,
    BatchProcessorConfig, BatchProcessorParams, BatchProcessorResult,
    BatchItem, BatchItemParams, BatchItemResult, BatchItemStatus,
    BatchResult, BatchStatus, BatchProcessingMode,
    AggregatedStats, BatchPerformanceMetrics, PerformanceTrend, ToolUsageStats,
    ProgressEvent, ProgressTracker, ProgressTrackerConfig,
};
```

#### Text 模块
```rust
pub use text::{
    TextProcessorTool, TextOperation,
    ChineseProcessingConfig, TextOutputFormat,
    TextProcessorParams, TextProcessorResult,
};
```

#### UI 模块
```rust
pub use ui::{
    HumanDecisionTool, ResultConfirmationTool, ResultReviewTool,
    HumanDecisionParams, HumanDecisionResult, HumanDecisionExecutor,
    ResultConfirmationConfig, ResultConfirmationParams, ResultConfirmationResult,
    ResultReviewConfig, ResultReviewParams, ResultReviewResult,
    BatchConfirmationTool, BatchConfirmationConfig, BatchConfirmationParams, BatchConfirmationResult,
    BatchDecision, BatchDecisionType, BatchOptions, BatchSummary,
    ConfirmationDecision, ConfirmationDetail, ConfirmationMethod, ConfirmationMode,
    ConfirmationOptions, ConfirmationPhase, ConfirmationStrategy,
    DecisionContext, DecisionOption, DefaultAction, ExecutionSummary,
    ExperimentalResult, ModificationType, OperationBatch, OperationImpact,
    OperationModification, OverallSummary, PhaseOutput, PhaseType,
    ProcessedBatch, RecommendedAction, ResourceRequirements,
    ReviewMode, ReviewOptions, ReviewSummary,
    RiskAssessment, RiskDistribution, RiskLevel,
    RollbackOperation, RollbackOptions, RollbackPlan, RollbackType,
    ReversibilitySummary, UserPreferences,
};
```

#### Utils 模块
```rust
pub use utils::{
    FileManagementToolRegistry, FileManagementMonitor, FileOperationManager,
    OptimizedFileOperationManager, FolderMerger, FolderMergerConfig,
    TextProcessor, CompressionUtils, PathUtils, StreamingUtils, ValidationUtils,
    Alert, AlertSeverity, AlertType,
    AuditEntry, AuditResult, CacheStats, CachedResult,
    ChineseTextType, CommonFolderInfo, DuplicateHandling, ErrorTracker,
    ExperimentalMode, FolderComparisonResult, FolderLocationInfo,
    FolderMergeError, FolderMergeResult, HumanDecisionContext,
    MemoryPoolStats, MergeDirection, MergeOperationStats, MergeRecommendation,
    MergeStrategy, MixedTextResult, MonitoringConfig, MonitoringStats,
    OperationMetrics, PerformanceStats, PinyinResult, PinyinStyle,
    ResourceUsage, SingleFolderMergeResult, TextNormalizationConfig,
    UniqueFolderInfo,
};
```

### 1.4 外部依赖类型

```rust
pub use crate::tools::algo::ac_automaton::{
    AutomatonConfig, AutomatonError, AutomatonNode, AutomatonResult,
    AutomatonStats, Pattern, PatternMatch,
};
```

---

## 2. 待实现方案 (In Progress)

### 2.1 决策记录

| 日期 | 决策内容 | 状态 | 备注 |
|------|----------|------|------|
| - | 模块化架构设计 | 已完成 | 7个子模块，职责清晰 |
| - | 错误恢复机制 | 已完成 | core/error_recovery.rs |
| - | 批处理进度追踪 | 已完成 | batch/progress_tracker.rs |
| - | 人工决策工具集 | 已完成 | ui/ 模块 |

### 2.2 任务清单

#### 高优先级
- [ ] 性能优化：批处理大数据量场景
- [ ] 错误处理：完善错误上下文信息

#### 中优先级
- [ ] 文档：API 文档完善
- [ ] 测试：覆盖率提升

#### 低优先级
- [ ] 功能：扩展文本处理操作类型
- [ ] 功能：新增分类规则类型

---

## 3. 状态记录

### 3.1 模块成熟度

| 模块 | 成熟度 | 说明 |
|------|--------|------|
| core | Stable | 错误类型、恢复机制完善 |
| classification | Stable | 分类流程完整 |
| batch | Stable | 批处理核心功能完备 |
| text | Stable | 基础文本处理功能 |
| ui | Stable | 人工决策工具齐全 |
| utils | Stable | 工具函数稳定 |
| plugin | Stable | 插件定义清晰 |

### 3.2 接口稳定性

- 所有 `pub use` 导出类型为稳定接口
- 内部实现可随时调整
- 新增功能通过扩展实现，不破坏现有接口

### 3.3 变更历史

| 版本 | 变更内容 | 影响范围 |
|------|----------|----------|
| - | 初始架构 | 全部模块 |
