//! Shared types for file management operations
//!
//! Common types used across the file management plugin.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Disk space information for a filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSpaceInfo {
    pub path: PathBuf,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub usage_percentage: f64,
}

/// Information about insufficient space for a specific path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsufficientSpaceInfo {
    pub path: PathBuf,
    pub required_bytes: u64,
    pub available_bytes: u64,
    pub deficit_bytes: u64,
}

/// Result of batch space checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSpaceCheckResult {
    pub total_required_bytes: u64,
    pub path_requirements: HashMap<PathBuf, u64>,
    pub insufficient_paths: Vec<InsufficientSpaceInfo>,
    pub has_sufficient_space: bool,
}

/// Result of preflight checks for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightCheckResult {
    pub total_operations: usize,
    pub validation_errors: Vec<String>,
    pub total_estimated_bytes: u64,
    pub space_check: Option<BatchSpaceCheckResult>,
    pub is_valid: bool,
}

/// File operation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileOperationType {
    Move,
    Copy,
    Link,
    HardLink,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Skip the operation if target exists
    Skip,
    /// Overwrite the target file/directory
    Overwrite,
    /// Rename the target to avoid conflicts
    Rename,
    /// Fail the operation if target exists
    Fail,
    /// Ask the user what to do (requires human decision integration)
    Ask,
    /// Merge directories (for directory conflicts only)
    Merge,
    /// Keep both files with different names
    KeepBoth,
    /// Compare and keep newer file
    KeepNewer,
    /// Compare and keep larger file
    KeepLarger,
}

/// Conflict resolution context for decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictContext {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub source_metadata: Option<ConflictFileMetadata>,
    pub target_metadata: Option<ConflictFileMetadata>,
    pub operation_type: FileOperationType,
    pub suggested_resolution: ConflictResolution,
}

/// File metadata for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictFileMetadata {
    pub size: u64,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    pub is_directory: bool,
    pub permissions: Option<String>,
}

/// Merge strategies for folder merging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Merge smaller folders into the largest one
    SmallerToLarger,
    /// Merge larger folders into the smallest one
    LargerToSmaller,
    /// Let user decide the strategy
    UserDecision,
    /// Merge all to specified target directory
    TargetDirectory,
}

/// Duplicate handling strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DuplicateHandling {
    /// Skip duplicate files
    Skip,
    /// Rename duplicates to avoid conflicts
    Rename,
    /// Keep newer file
    KeepNewer,
    /// Keep larger file
    KeepLarger,
    /// Keep both files with different names
    KeepBoth,
    /// Merge file contents if possible
    Merge,
}

/// Merge direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeDirection {
    /// Source to target
    SourceToTarget,
    /// Target to source
    TargetToSource,
    /// Bi-directional
    Bidirectional,
}

/// Experimental mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExperimentalMode {
    /// Real mode - actual operations
    Real,
    /// Experimental mode - simulate operations
    Experimental,
    /// Dry run - show what would be done without changes
    DryRun,
}

impl ExperimentalMode {
    /// Check if this is real mode
    pub fn is_real(&self) -> bool {
        matches!(self, ExperimentalMode::Real)
    }

    /// Check if this is experimental mode
    pub fn is_experimental(&self) -> bool {
        matches!(self, ExperimentalMode::Experimental)
    }

    /// Check if this is dry run mode
    pub fn is_dry_run(&self) -> bool {
        matches!(self, ExperimentalMode::DryRun)
    }
}

/// Chinese text types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ChineseTextType {
    /// No Chinese text
    None,
    /// Simplified Chinese
    Simplified,
    /// Traditional Chinese
    Traditional,
    /// Mixed simplified and traditional
    Mixed,
    /// Unknown Chinese text type
    Unknown,
}

/// Pinyin output styles
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PinyinStyle {
    /// Normal style with tone marks (nǐ hǎo)
    Normal,
    /// With tone marks
    WithTone,
    /// Without tone marks (ni hao)
    WithoutTone,
    /// First letters only (n h)
    FirstLetter,
    /// Numeric tone marks (ni3 hao3)
    Numeric,
}
