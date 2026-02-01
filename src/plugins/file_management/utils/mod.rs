//! Utilities module for file management plugin
//!
//! Contains shared utilities, monitoring, performance optimization, and registry.

pub mod monitoring;
pub mod performance;
pub mod registry;
pub mod utils;

// Re-export utility components
pub use monitoring::{
    Alert, AlertSeverity, AlertType, AuditEntry, AuditResult, ErrorTracker, FileManagementMonitor,
    MonitoringConfig, MonitoringStats, OperationMetrics, ResourceUsage,
};
pub use performance::{
    CacheStats, CachedResult, CompressionUtils, MemoryPoolStats, OptimizedFileOperationManager,
    PerformanceStats, StreamingUtils,
};
pub use registry::FileManagementToolRegistry;
pub use utils::{
    ChineseTextType, CommonFolderInfo, DuplicateHandling, ExperimentalMode, FileOperationManager,
    FolderComparisonResult, FolderLocationInfo, FolderMergeError, FolderMergeResult, FolderMerger,
    FolderMergerConfig, HumanDecisionContext, MergeDirection, MergeOperationStats,
    MergeRecommendation, MergeStrategy, MixedTextResult, PathUtils, PinyinResult, PinyinStyle,
    SingleFolderMergeResult, TextNormalizationConfig, TextProcessor, UniqueFolderInfo,
    ValidationUtils,
};
