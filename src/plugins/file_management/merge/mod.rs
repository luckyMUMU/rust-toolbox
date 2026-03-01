//! 文件夹合并模块
//!
//! 提供智能文件夹合并功能，包括：
//! - 文件夹扫描和分析
//! - 冲突检测
//! - 合并计划生成
//! - 批量合并执行

mod folder_merge_tool;

pub use folder_merge_tool::{
    ConflictResolution, FolderMergeParams, FolderMergeTool,
    MergeConflict, MergeConflictType, MergeExecutionResult, MergeOperationResult,
    MergeOperationType, MergePlan, MergePlanStatus, SetFolderInfo,
};
