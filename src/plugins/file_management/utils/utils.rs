//! Utility functions and shared components for file management operations

use crate::plugins::file_management::core::error::{FileManagementError, FileManagementResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};
use uuid::Uuid;

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

impl ConflictContext {
    pub fn new(
        source_path: PathBuf,
        target_path: PathBuf,
        operation_type: FileOperationType,
    ) -> Self {
        let source_metadata = Self::get_file_metadata(&source_path);
        let target_metadata = Self::get_file_metadata(&target_path);

        // Suggest a resolution based on metadata
        let suggested_resolution =
            Self::suggest_resolution(&source_metadata, &target_metadata, &operation_type);

        Self {
            source_path,
            target_path,
            source_metadata,
            target_metadata,
            operation_type,
            suggested_resolution,
        }
    }

    fn get_file_metadata(path: &Path) -> Option<ConflictFileMetadata> {
        if let Ok(metadata) = std::fs::metadata(path) {
            let modified = metadata.modified().ok().and_then(|time| {
                time.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|duration| {
                        chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                            .unwrap_or_else(chrono::Utc::now)
                    })
            });

            Some(ConflictFileMetadata {
                size: metadata.len(),
                modified,
                is_directory: metadata.is_dir(),
                permissions: None, // Could be enhanced with platform-specific permissions
            })
        } else {
            None
        }
    }

    fn suggest_resolution(
        source_metadata: &Option<ConflictFileMetadata>,
        target_metadata: &Option<ConflictFileMetadata>,
        _operation_type: &FileOperationType,
    ) -> ConflictResolution {
        match (source_metadata, target_metadata) {
            (Some(source), Some(target)) => {
                // If both are directories, suggest merge
                if source.is_directory && target.is_directory {
                    return ConflictResolution::Merge;
                }

                // If source is newer, suggest overwrite
                if let (Some(source_time), Some(target_time)) = (&source.modified, &target.modified)
                {
                    if source_time > target_time {
                        return ConflictResolution::Overwrite;
                    } else if target_time > source_time {
                        return ConflictResolution::KeepBoth;
                    }
                }

                // If source is larger, suggest overwrite
                if source.size > target.size {
                    ConflictResolution::Overwrite
                } else {
                    ConflictResolution::KeepBoth
                }
            }
            _ => ConflictResolution::Rename,
        }
    }
}

/// Result of a file operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    pub operation_type: FileOperationType,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub success: bool,
    pub bytes_moved: u64,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub renamed_target: Option<PathBuf>,
}

/// File operation manager for safe file operations
pub struct FileOperationManager {
    dry_run: bool,
    temp_directory: PathBuf,
    conflict_resolution: ConflictResolution,
    create_directories: bool,
    check_disk_space: bool,
}

impl FileOperationManager {
    /// Create a new file operation manager
    pub fn new(temp_directory: PathBuf, dry_run: bool) -> Self {
        Self {
            dry_run,
            temp_directory,
            conflict_resolution: ConflictResolution::Rename,
            create_directories: true,
            check_disk_space: true,
        }
    }

    /// Create a new file operation manager with configuration
    pub fn with_config(
        temp_directory: PathBuf,
        dry_run: bool,
        conflict_resolution: ConflictResolution,
        create_directories: bool,
        check_disk_space: bool,
    ) -> Self {
        Self {
            dry_run,
            temp_directory,
            conflict_resolution,
            create_directories,
            check_disk_space,
        }
    }

    /// Check if running in dry run mode
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Set dry run mode
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Set conflict resolution strategy
    pub fn set_conflict_resolution(&mut self, strategy: ConflictResolution) {
        self.conflict_resolution = strategy;
    }

    /// Get available disk space for a path
    pub fn get_available_space<P: AsRef<Path>>(&self, path: P) -> FileManagementResult<u64> {
        let path = path.as_ref();

        // Try to get the parent directory if path doesn't exist
        let check_path = if path.exists() {
            path
        } else if let Some(parent) = path.parent() {
            parent
        } else {
            Path::new(".")
        };

        // Platform-specific disk space checking
        #[cfg(unix)]
        {
            self.get_unix_disk_space(check_path)
        }

        #[cfg(windows)]
        {
            self.get_windows_disk_space(check_path)
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Fallback for other platforms
            Ok(1024 * 1024 * 1024 * 10) // 10GB estimate
        }
    }

    #[cfg(unix)]
    fn get_unix_disk_space(&self, path: &Path) -> FileManagementResult<u64> {
        use std::ffi::CString;
        use std::mem;
        use std::os::raw::{c_char, c_int};

        // Define statvfs structure (simplified)
        #[repr(C)]
        struct StatVfs {
            f_bsize: u64,   // File system block size
            f_frsize: u64,  // Fragment size
            f_blocks: u64,  // Size of fs in f_frsize units
            f_bfree: u64,   // Number of free blocks
            f_bavail: u64,  // Number of free blocks for unprivileged users
            f_files: u64,   // Number of inodes
            f_ffree: u64,   // Number of free inodes
            f_favail: u64,  // Number of free inodes for unprivileged users
            f_fsid: u64,    // File system ID
            f_flag: u64,    // Mount flags
            f_namemax: u64, // Maximum filename length
        }

        extern "C" {
            fn statvfs(path: *const c_char, buf: *mut StatVfs) -> c_int;
        }

        let path_cstring = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|_| FileManagementError::invalid_path(path, "Path contains null bytes"))?;

        let mut stat: StatVfs = unsafe { mem::zeroed() };
        let result = unsafe { statvfs(path_cstring.as_ptr(), &mut stat) };

        if result == 0 {
            // Available space = available blocks * block size
            let available_space = stat.f_bavail * stat.f_frsize;
            Ok(available_space)
        } else {
            // Fallback to a reasonable estimate if statvfs fails
            warn!(
                "statvfs failed for path {}, using fallback estimate",
                path.display()
            );
            Ok(1024 * 1024 * 1024 * 10) // 10GB estimate
        }
    }

    #[cfg(windows)]
    fn get_windows_disk_space(&self, path: &Path) -> FileManagementResult<u64> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        // Windows API function
        extern "system" {
            fn GetDiskFreeSpaceExW(
                lpDirectoryName: *const u16,
                lpFreeBytesAvailableToCaller: *mut u64,
                lpTotalNumberOfBytes: *mut u64,
                lpTotalNumberOfFreeBytes: *mut u64,
            ) -> i32;
        }

        // Convert path to wide string
        let wide_path: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;

        let result = unsafe {
            GetDiskFreeSpaceExW(
                wide_path.as_ptr(),
                &mut free_bytes_available,
                &mut total_bytes,
                &mut total_free_bytes,
            )
        };

        if result != 0 {
            Ok(free_bytes_available)
        } else {
            // Fallback to a reasonable estimate if Windows API fails
            warn!(
                "GetDiskFreeSpaceExW failed for path {}, using fallback estimate",
                path.display()
            );
            Ok(1024 * 1024 * 1024 * 10) // 10GB estimate
        }
    }

    /// Get detailed disk space information
    pub fn get_disk_space_info<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> FileManagementResult<DiskSpaceInfo> {
        let path = path.as_ref();
        let available_space = self.get_available_space(path)?;

        // Get total space (simplified - in a real implementation you'd get this from the OS)
        let total_space = available_space * 2; // Rough estimate
        let used_space = total_space - available_space;
        let usage_percentage = (used_space as f64 / total_space as f64) * 100.0;

        Ok(DiskSpaceInfo {
            path: path.to_path_buf(),
            total_bytes: total_space,
            available_bytes: available_space,
            used_bytes: used_space,
            usage_percentage,
        })
    }

    /// Check if multiple operations would have sufficient space
    pub fn check_batch_space_requirements<P: AsRef<Path>>(
        &self,
        operations: &[(P, u64)], // (target_path, required_bytes)
    ) -> FileManagementResult<BatchSpaceCheckResult> {
        let mut total_required = 0;
        let mut path_requirements: HashMap<PathBuf, u64> = HashMap::new();
        let mut insufficient_paths = Vec::new();

        // Group requirements by filesystem/drive
        for (path, required_bytes) in operations {
            let path = path.as_ref();
            total_required += required_bytes;

            // Get the root path for this operation
            let root_path = self.get_filesystem_root(path)?;
            *path_requirements.entry(root_path).or_insert(0) += required_bytes;
        }

        // Check each filesystem
        for (root_path, required_bytes) in &path_requirements {
            let available = self.get_available_space(root_path)?;
            if available < *required_bytes {
                insufficient_paths.push(InsufficientSpaceInfo {
                    path: root_path.clone(),
                    required_bytes: *required_bytes,
                    available_bytes: available,
                    deficit_bytes: required_bytes - available,
                });
            }
        }

        Ok(BatchSpaceCheckResult {
            total_required_bytes: total_required,
            path_requirements,
            insufficient_paths: insufficient_paths.clone(),
            has_sufficient_space: insufficient_paths.is_empty(),
        })
    }

    /// Get the filesystem root for a given path
    fn get_filesystem_root<P: AsRef<Path>>(&self, path: P) -> FileManagementResult<PathBuf> {
        let path = path.as_ref();

        #[cfg(windows)]
        {
            // On Windows, get the drive letter
            if let Some(prefix) = path.components().next() {
                if let std::path::Component::Prefix(prefix_component) = prefix {
                    return Ok(PathBuf::from(format!(
                        "{}\\",
                        prefix_component.as_os_str().to_string_lossy()
                    )));
                }
            }
            Ok(PathBuf::from("C:\\")) // Fallback to C: drive
        }

        #[cfg(unix)]
        {
            // On Unix, find the mount point
            // This is a simplified implementation - in production you'd parse /proc/mounts
            let mut current_path = path;
            loop {
                if current_path.exists() {
                    return Ok(current_path.to_path_buf());
                }
                if let Some(parent) = current_path.parent() {
                    current_path = parent;
                } else {
                    break;
                }
            }
            Ok(PathBuf::from("/")) // Fallback to root
        }

        #[cfg(not(any(unix, windows)))]
        {
            Ok(path.to_path_buf())
        }
    }

    /// Pre-flight check for a batch of operations
    pub fn preflight_check<P: AsRef<Path>>(
        &self,
        operations: &[(P, P, FileOperationType)], // (source, target, operation_type)
    ) -> FileManagementResult<PreflightCheckResult> {
        let mut space_requirements = Vec::new();
        let mut validation_errors = Vec::new();
        let mut total_estimated_bytes = 0;

        for (source, target, operation_type) in operations {
            let source_path = source.as_ref();
            let target_path = target.as_ref();

            // Validate paths
            if let Err(e) = self.validate_path(source_path) {
                validation_errors.push(format!("Source path {}: {}", source_path.display(), e));
                continue;
            }

            if let Err(e) = self.validate_path(target_path) {
                validation_errors.push(format!("Target path {}: {}", target_path.display(), e));
                continue;
            }

            // Check if source exists
            if !source_path.exists() {
                validation_errors.push(format!(
                    "Source path does not exist: {}",
                    source_path.display()
                ));
                continue;
            }

            // Calculate space requirements
            let required_bytes = match operation_type {
                FileOperationType::Move => {
                    // Move operations don't require additional space if on same filesystem
                    let source_root = self.get_filesystem_root(source_path)?;
                    let target_root = self.get_filesystem_root(target_path)?;

                    if source_root == target_root {
                        0 // Same filesystem, no additional space needed
                    } else {
                        // Different filesystem, need space for copy
                        if source_path.is_file() {
                            PathUtils::get_file_size(source_path)?
                        } else {
                            PathUtils::get_directory_size(source_path)?
                        }
                    }
                }
                FileOperationType::Copy => {
                    // Copy operations always require space
                    if source_path.is_file() {
                        PathUtils::get_file_size(source_path)?
                    } else {
                        PathUtils::get_directory_size(source_path)?
                    }
                }
                FileOperationType::Link | FileOperationType::HardLink => {
                    // Links don't require significant additional space
                    0
                }
            };

            if required_bytes > 0 {
                space_requirements.push((target_path.to_path_buf(), required_bytes));
                total_estimated_bytes += required_bytes;
            }
        }

        // Check space requirements
        let space_check = if !space_requirements.is_empty() {
            Some(self.check_batch_space_requirements(&space_requirements)?)
        } else {
            None
        };

        Ok(PreflightCheckResult {
            total_operations: operations.len(),
            validation_errors: validation_errors.clone(),
            total_estimated_bytes,
            space_check: space_check.clone(),
            is_valid: validation_errors.is_empty()
                && space_check
                    .as_ref()
                    .is_none_or(|sc| sc.has_sufficient_space),
        })
    }

    /// Check if a file operation would have sufficient space
    pub fn check_space_requirements<P: AsRef<Path>>(
        &self,
        target_path: P,
        required_bytes: u64,
    ) -> FileManagementResult<()> {
        if !self.check_disk_space {
            return Ok(());
        }

        let available = self.get_available_space(&target_path)?;
        if available < required_bytes {
            return Err(FileManagementError::insufficient_space(
                required_bytes,
                available,
            ));
        }
        Ok(())
    }

    /// Validate that a path is safe for operations
    pub fn validate_path<P: AsRef<Path>>(&self, path: P) -> FileManagementResult<()> {
        let path = path.as_ref();

        // Check for null bytes
        if path.to_string_lossy().contains('\0') {
            return Err(FileManagementError::invalid_path(
                path,
                "Path contains null bytes",
            ));
        }

        // Check for extremely long paths
        if path.to_string_lossy().len() > 4096 {
            return Err(FileManagementError::invalid_path(path, "Path is too long"));
        }

        // Check for invalid characters (platform-specific)
        #[cfg(windows)]
        {
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            let path_str = path.to_string_lossy();
            for &ch in &invalid_chars {
                if path_str.contains(ch) {
                    return Err(FileManagementError::invalid_path(
                        path,
                        format!("Path contains invalid character: {}", ch),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Move a file or directory
    pub async fn move_file<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        target: P2,
    ) -> FileManagementResult<FileOperationResult> {
        self.execute_operation(source, target, FileOperationType::Move)
            .await
    }

    /// Copy a file or directory
    pub async fn copy_file<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        target: P2,
    ) -> FileManagementResult<FileOperationResult> {
        self.execute_operation(source, target, FileOperationType::Copy)
            .await
    }

    /// Create a symbolic link
    pub async fn link_file<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        target: P2,
    ) -> FileManagementResult<FileOperationResult> {
        self.execute_operation(source, target, FileOperationType::Link)
            .await
    }

    /// Create a hard link
    pub async fn hard_link_file<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        target: P2,
    ) -> FileManagementResult<FileOperationResult> {
        self.execute_operation(source, target, FileOperationType::HardLink)
            .await
    }

    /// Execute a file operation with atomic behavior and error recovery
    async fn execute_operation<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        target: P2,
        operation_type: FileOperationType,
    ) -> FileManagementResult<FileOperationResult> {
        let start_time = std::time::Instant::now();
        let source_path = source.as_ref().to_path_buf();
        let mut target_path = target.as_ref().to_path_buf();

        // Validate paths
        self.validate_path(&source_path)?;
        self.validate_path(&target_path)?;

        // Check source exists
        if !source_path.exists() {
            return Err(FileManagementError::not_found(&source_path));
        }

        // Get source size for space checking
        let source_size = if source_path.is_file() {
            PathUtils::get_file_size(&source_path)?
        } else {
            PathUtils::get_directory_size(&source_path)?
        };

        // Check disk space requirements (except for links)
        if matches!(
            operation_type,
            FileOperationType::Move | FileOperationType::Copy
        ) {
            self.check_space_requirements(&target_path, source_size)?;
        }

        // Create target directory if needed
        if self.create_directories {
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    if self.dry_run {
                        debug!("Would create directory: {}", parent.display());
                    } else {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            FileManagementError::io(
                                format!("Failed to create directory {}", parent.display()),
                                e,
                            )
                        })?;
                    }
                }
            }
        }

        // Handle conflicts
        let final_target = self
            .resolve_conflict(&source_path, &target_path, &operation_type)
            .await?;
        if final_target != target_path {
            target_path = final_target;
        }

        let bytes_moved = if self.dry_run {
            debug!(
                "Would {} {} to {}",
                match operation_type {
                    FileOperationType::Move => "move",
                    FileOperationType::Copy => "copy",
                    FileOperationType::Link => "link",
                    FileOperationType::HardLink => "hard link",
                },
                source_path.display(),
                target_path.display()
            );
            source_size
        } else {
            self.perform_operation(&source_path, &target_path, &operation_type)
                .await?
        };

        let duration = start_time.elapsed().as_millis() as u64;

        Ok(FileOperationResult {
            operation_type,
            source_path,
            target_path: target_path.clone(),
            success: true,
            bytes_moved,
            duration_ms: duration,
            error: None,
            renamed_target: if target_path != target.as_ref() {
                Some(target_path)
            } else {
                None
            },
        })
    }

    /// Resolve file conflicts based on the configured strategy
    async fn resolve_conflict<P: AsRef<Path>>(
        &self,
        source_path: &Path,
        target_path: P,
        operation_type: &FileOperationType,
    ) -> FileManagementResult<PathBuf> {
        let target_path = target_path.as_ref();

        if !target_path.exists() {
            return Ok(target_path.to_path_buf());
        }

        let conflict_context = ConflictContext::new(
            source_path.to_path_buf(),
            target_path.to_path_buf(),
            operation_type.clone(),
        );

        match self.conflict_resolution {
            ConflictResolution::Skip => Err(FileManagementError::conflict_simple(format!(
                "Target exists and conflict resolution is set to skip: {}",
                target_path.display()
            ))),
            ConflictResolution::Overwrite => {
                debug!("Overwriting existing target: {}", target_path.display());
                Ok(target_path.to_path_buf())
            }
            ConflictResolution::Rename => {
                let new_path = PathUtils::generate_unique_name(target_path);
                debug!(
                    "Renaming target to avoid conflict: {} -> {}",
                    target_path.display(),
                    new_path.display()
                );
                Ok(new_path)
            }
            ConflictResolution::Fail => Err(FileManagementError::conflict_simple(format!(
                "Target exists and conflict resolution is set to fail: {}",
                target_path.display()
            ))),
            ConflictResolution::Ask => {
                // For now, use the suggested resolution - in a real implementation this would prompt the user
                warn!(
                    "Human decision required for conflict, using suggested resolution: {:?}",
                    conflict_context.suggested_resolution
                );
                self.apply_suggested_resolution(&conflict_context).await
            }
            ConflictResolution::Merge => self.resolve_merge_conflict(&conflict_context).await,
            ConflictResolution::KeepBoth => {
                let new_path = PathUtils::generate_unique_name(target_path);
                debug!(
                    "Keeping both files, renaming target: {} -> {}",
                    target_path.display(),
                    new_path.display()
                );
                Ok(new_path)
            }
            ConflictResolution::KeepNewer => self.resolve_by_date(&conflict_context, true).await,
            ConflictResolution::KeepLarger => self.resolve_by_size(&conflict_context, true).await,
        }
    }

    /// Apply the suggested resolution from conflict context
    async fn apply_suggested_resolution(
        &self,
        context: &ConflictContext,
    ) -> FileManagementResult<PathBuf> {
        match context.suggested_resolution {
            ConflictResolution::Overwrite => Ok(context.target_path.clone()),
            ConflictResolution::Rename | ConflictResolution::KeepBoth => {
                Ok(PathUtils::generate_unique_name(&context.target_path))
            }
            ConflictResolution::Merge => self.resolve_merge_conflict(context).await,
            _ => Ok(context.target_path.clone()),
        }
    }

    /// Resolve merge conflicts for directories
    async fn resolve_merge_conflict(
        &self,
        context: &ConflictContext,
    ) -> FileManagementResult<PathBuf> {
        if let (Some(source_meta), Some(target_meta)) =
            (&context.source_metadata, &context.target_metadata)
        {
            if source_meta.is_directory && target_meta.is_directory {
                // For directory merges, we return the target path and handle merging in the operation
                debug!(
                    "Directory merge conflict resolved: merging into {}",
                    context.target_path.display()
                );
                return Ok(context.target_path.clone());
            }
        }

        // For non-directories, fall back to rename
        Ok(PathUtils::generate_unique_name(&context.target_path))
    }

    /// Resolve conflict by comparing file dates
    async fn resolve_by_date(
        &self,
        context: &ConflictContext,
        keep_newer: bool,
    ) -> FileManagementResult<PathBuf> {
        if let (Some(source_meta), Some(target_meta)) =
            (&context.source_metadata, &context.target_metadata)
        {
            if let (Some(source_time), Some(target_time)) =
                (&source_meta.modified, &target_meta.modified)
            {
                let source_is_newer = source_time > target_time;

                if (keep_newer && source_is_newer) || (!keep_newer && !source_is_newer) {
                    debug!(
                        "Resolving by date: overwriting target (source is {})",
                        if source_is_newer { "newer" } else { "older" }
                    );
                    return Ok(context.target_path.clone());
                } else {
                    debug!(
                        "Resolving by date: keeping target (target is {})",
                        if source_is_newer { "older" } else { "newer" }
                    );
                    return Err(FileManagementError::conflict_simple(
                        "Target is newer/older, skipping operation",
                    ));
                }
            }
        }

        // If we can't compare dates, fall back to rename
        Ok(PathUtils::generate_unique_name(&context.target_path))
    }

    /// Resolve conflict by comparing file sizes
    async fn resolve_by_size(
        &self,
        context: &ConflictContext,
        keep_larger: bool,
    ) -> FileManagementResult<PathBuf> {
        if let (Some(source_meta), Some(target_meta)) =
            (&context.source_metadata, &context.target_metadata)
        {
            let source_is_larger = source_meta.size > target_meta.size;

            if (keep_larger && source_is_larger) || (!keep_larger && !source_is_larger) {
                debug!(
                    "Resolving by size: overwriting target (source is {})",
                    if source_is_larger {
                        "larger"
                    } else {
                        "smaller"
                    }
                );
                return Ok(context.target_path.clone());
            } else {
                debug!(
                    "Resolving by size: keeping target (target is {})",
                    if source_is_larger {
                        "smaller"
                    } else {
                        "larger"
                    }
                );
                return Err(FileManagementError::conflict_simple(
                    "Target is larger/smaller, skipping operation",
                ));
            }
        }

        // If we can't compare sizes, fall back to rename
        Ok(PathUtils::generate_unique_name(&context.target_path))
    }

    /// Perform the actual file operation
    async fn perform_operation(
        &self,
        source_path: &Path,
        target_path: &Path,
        operation_type: &FileOperationType,
    ) -> FileManagementResult<u64> {
        match operation_type {
            FileOperationType::Move => self.perform_move(source_path, target_path).await,
            FileOperationType::Copy => self.perform_copy(source_path, target_path).await,
            FileOperationType::Link => self.perform_symlink(source_path, target_path).await,
            FileOperationType::HardLink => self.perform_hardlink(source_path, target_path).await,
        }
    }

    /// Perform atomic move operation
    async fn perform_move(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> FileManagementResult<u64> {
        let source_size = if source_path.is_file() {
            PathUtils::get_file_size(source_path)?
        } else {
            PathUtils::get_directory_size(source_path)?
        };

        // Try atomic rename first (works if on same filesystem)
        match std::fs::rename(source_path, target_path) {
            Ok(_) => {
                debug!(
                    "Atomic move successful: {} -> {}",
                    source_path.display(),
                    target_path.display()
                );
                Ok(source_size)
            }
            Err(e) => {
                // If atomic rename fails, fall back to copy + delete
                warn!("Atomic move failed, falling back to copy+delete: {}", e);

                // Create a temporary target to ensure atomicity
                let temp_target = self.create_temp_path(target_path)?;

                // Copy to temporary location first
                let copied_size = self.perform_copy(source_path, &temp_target).await?;

                // Atomically move temp to final location
                std::fs::rename(&temp_target, target_path).map_err(|e| {
                    // Clean up temp file on failure
                    let _ = std::fs::remove_file(&temp_target);
                    FileManagementError::io(
                        format!(
                            "Failed to move temp file to target: {} -> {}",
                            temp_target.display(),
                            target_path.display()
                        ),
                        e,
                    )
                })?;

                // Remove source after successful copy
                if source_path.is_dir() {
                    std::fs::remove_dir_all(source_path).map_err(|e| {
                        FileManagementError::io(
                            format!(
                                "Failed to remove source directory after move: {}",
                                source_path.display()
                            ),
                            e,
                        )
                    })?;
                } else {
                    std::fs::remove_file(source_path).map_err(|e| {
                        FileManagementError::io(
                            format!(
                                "Failed to remove source file after move: {}",
                                source_path.display()
                            ),
                            e,
                        )
                    })?;
                }

                Ok(copied_size)
            }
        }
    }

    /// Perform copy operation
    async fn perform_copy(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> FileManagementResult<u64> {
        if source_path.is_file() {
            self.copy_file_atomic(source_path, target_path).await
        } else {
            // Check if target exists and is a directory for potential merging
            if target_path.exists()
                && target_path.is_dir()
                && self.conflict_resolution == ConflictResolution::Merge
            {
                self.merge_directories(source_path, target_path).await
            } else {
                self.copy_directory_recursive(source_path, target_path)
                    .await
            }
        }
    }

    /// Merge source directory into existing target directory
    fn merge_directories<'a>(
        &'a self,
        source_path: &'a Path,
        target_path: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileManagementResult<u64>> + Send + 'a>>
    {
        Box::pin(async move {
            let mut total_bytes = 0;

            // Read source directory
            let mut entries = tokio::fs::read_dir(source_path).await.map_err(|e| {
                FileManagementError::io(
                    format!("Failed to read source directory {}", source_path.display()),
                    e,
                )
            })?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                FileManagementError::io(
                    format!(
                        "Failed to read directory entry in {}",
                        source_path.display()
                    ),
                    e,
                )
            })? {
                let entry_path = entry.path();
                let entry_name = entry.file_name();
                let target_entry = target_path.join(entry_name);

                if entry_path.is_dir() {
                    if target_entry.exists() && target_entry.is_dir() {
                        // Recursively merge subdirectories
                        total_bytes += self.merge_directories(&entry_path, &target_entry).await?;
                    } else {
                        // Copy directory normally
                        total_bytes += self
                            .copy_directory_recursive(&entry_path, &target_entry)
                            .await?;
                    }
                } else {
                    // For files, apply conflict resolution
                    if target_entry.exists() {
                        let resolved_target = self
                            .resolve_conflict(&entry_path, &target_entry, &FileOperationType::Copy)
                            .await?;
                        total_bytes += self.copy_file_atomic(&entry_path, &resolved_target).await?;
                    } else {
                        total_bytes += self.copy_file_atomic(&entry_path, &target_entry).await?;
                    }
                }
            }

            Ok(total_bytes)
        })
    }

    /// Atomic file copy
    async fn copy_file_atomic(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> FileManagementResult<u64> {
        let temp_target = self.create_temp_path(target_path)?;

        // Copy to temporary file first
        let bytes_copied = tokio::fs::copy(source_path, &temp_target)
            .await
            .map_err(|e| {
                FileManagementError::io(
                    format!(
                        "Failed to copy {} to temp file {}",
                        source_path.display(),
                        temp_target.display()
                    ),
                    e,
                )
            })?;

        // Atomically move temp file to final location
        std::fs::rename(&temp_target, target_path).map_err(|e| {
            // Clean up temp file on failure
            let _ = std::fs::remove_file(&temp_target);
            FileManagementError::io(
                format!(
                    "Failed to move temp file to target: {} -> {}",
                    temp_target.display(),
                    target_path.display()
                ),
                e,
            )
        })?;

        Ok(bytes_copied)
    }

    /// Recursive directory copy
    fn copy_directory_recursive<'a>(
        &'a self,
        source_path: &'a Path,
        target_path: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileManagementResult<u64>> + Send + 'a>>
    {
        Box::pin(async move {
            let mut total_bytes = 0;

            // Create target directory
            tokio::fs::create_dir_all(target_path).await.map_err(|e| {
                FileManagementError::io(
                    format!(
                        "Failed to create target directory {}",
                        target_path.display()
                    ),
                    e,
                )
            })?;

            // Read source directory
            let mut entries = tokio::fs::read_dir(source_path).await.map_err(|e| {
                FileManagementError::io(
                    format!("Failed to read source directory {}", source_path.display()),
                    e,
                )
            })?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                FileManagementError::io(
                    format!(
                        "Failed to read directory entry in {}",
                        source_path.display()
                    ),
                    e,
                )
            })? {
                let entry_path = entry.path();
                let entry_name = entry.file_name();
                let target_entry = target_path.join(entry_name);

                if entry_path.is_dir() {
                    total_bytes += self
                        .copy_directory_recursive(&entry_path, &target_entry)
                        .await?;
                } else {
                    total_bytes += self.copy_file_atomic(&entry_path, &target_entry).await?;
                }
            }

            Ok(total_bytes)
        })
    }

    /// Create symbolic link
    async fn perform_symlink(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> FileManagementResult<u64> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(source_path, target_path).map_err(|e| {
                FileManagementError::io(
                    format!(
                        "Failed to create symlink {} -> {}",
                        target_path.display(),
                        source_path.display()
                    ),
                    e,
                )
            })?;
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::{symlink_dir, symlink_file};
            if source_path.is_dir() {
                symlink_dir(source_path, target_path).map_err(|e| {
                    FileManagementError::io(
                        format!(
                            "Failed to create directory symlink {} -> {}",
                            target_path.display(),
                            source_path.display()
                        ),
                        e,
                    )
                })?;
            } else {
                symlink_file(source_path, target_path).map_err(|e| {
                    FileManagementError::io(
                        format!(
                            "Failed to create file symlink {} -> {}",
                            target_path.display(),
                            source_path.display()
                        ),
                        e,
                    )
                })?;
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            return Err(FileManagementError::unsupported_operation(
                "Symbolic links not supported on this platform",
            ));
        }

        Ok(0) // Symlinks don't consume additional space
    }

    /// Create hard link
    async fn perform_hardlink(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> FileManagementResult<u64> {
        if source_path.is_dir() {
            return Err(FileManagementError::unsupported_operation(
                "Hard links to directories are not supported",
            ));
        }

        std::fs::hard_link(source_path, target_path).map_err(|e| {
            FileManagementError::io(
                format!(
                    "Failed to create hard link {} -> {}",
                    target_path.display(),
                    source_path.display()
                ),
                e,
            )
        })?;

        Ok(0) // Hard links don't consume additional space
    }

    /// Create a temporary path for atomic operations
    fn create_temp_path(&self, target_path: &Path) -> FileManagementResult<PathBuf> {
        let temp_name = format!(
            ".tmp_{}_{}",
            target_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file"),
            uuid::Uuid::new_v4().simple()
        );

        let temp_path = if let Some(parent) = target_path.parent() {
            parent.join(temp_name)
        } else {
            self.temp_directory.join(temp_name)
        };

        Ok(temp_path)
    }

    /// Batch operation for multiple files
    pub async fn batch_operation<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        operations: Vec<(P1, P2, FileOperationType)>,
    ) -> Vec<FileManagementResult<FileOperationResult>> {
        let mut results = Vec::new();

        for (source, target, op_type) in operations {
            let result = self.execute_operation(source, target, op_type).await;
            results.push(result);
        }

        results
    }
}

/// Pinyin conversion styles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PinyinStyle {
    Normal,      // ni3 hao3
    WithTone,    // nǐ hǎo
    WithoutTone, // ni hao
    FirstLetter, // n h
    Numeric,     // ni3 hao3
}

/// Pinyin conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinyinResult {
    pub original: String,
    pub pinyin_variants: Vec<String>,
    pub style: PinyinStyle,
    pub combinations: Vec<Vec<String>>,
}

/// Chinese text type detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChineseTextType {
    None,
    Simplified,
    Traditional,
    Mixed,
    Unknown,
}

/// Result of mixed text processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedTextResult {
    pub original: String,
    pub chinese_chars: String,
    pub non_chinese_chars: String,
    pub simplified_chinese: String,
    pub chinese_type: ChineseTextType,
    pub has_mixed_content: bool,
}

/// Text normalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextNormalizationConfig {
    pub normalize_case: bool,
    pub normalize_whitespace: bool,
    pub remove_punctuation: bool,
    pub normalize_unicode: bool,
    pub filter_characters: Option<Vec<char>>,
    pub preserve_alphanumeric_only: bool,
}

impl Default for TextNormalizationConfig {
    fn default() -> Self {
        Self {
            normalize_case: true,
            normalize_whitespace: true,
            remove_punctuation: false,
            normalize_unicode: true,
            filter_characters: None,
            preserve_alphanumeric_only: false,
        }
    }
}

/// Text processor for normalization and Chinese text handling
pub struct TextProcessor {
    enable_chinese: bool,
    normalization_config: TextNormalizationConfig,
}

impl TextProcessor {
    /// Create a new text processor
    pub fn new(enable_chinese: bool) -> Self {
        Self {
            enable_chinese,
            normalization_config: TextNormalizationConfig::default(),
        }
    }

    /// Create a new text processor with custom normalization config
    pub fn with_config(enable_chinese: bool, config: TextNormalizationConfig) -> Self {
        Self {
            enable_chinese,
            normalization_config: config,
        }
    }

    /// Normalize text case
    pub fn normalize_case(&self, text: &str) -> String {
        if self.normalization_config.normalize_case {
            text.to_lowercase()
        } else {
            text.to_string()
        }
    }

    /// Remove extra spaces and normalize whitespace
    pub fn normalize_whitespace(&self, text: &str) -> String {
        if self.normalization_config.normalize_whitespace {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        } else {
            text.to_string()
        }
    }

    /// Remove spaces entirely
    pub fn remove_spaces(&self, text: &str) -> String {
        text.replace(' ', "")
    }

    /// Remove punctuation characters
    pub fn remove_punctuation(&self, text: &str) -> String {
        if self.normalization_config.remove_punctuation {
            text.chars().filter(|c| !c.is_ascii_punctuation()).collect()
        } else {
            text.to_string()
        }
    }

    /// Normalize Unicode characters (NFD normalization)
    pub fn normalize_unicode(&self, text: &str) -> String {
        if self.normalization_config.normalize_unicode {
            // Basic Unicode normalization - remove diacritics and normalize
            text.chars()
                .map(|c| {
                    // Simple ASCII folding for common diacritics
                    match c {
                        'à'..='ÿ' => self.fold_latin_char(c),
                        _ => c,
                    }
                })
                .collect()
        } else {
            text.to_string()
        }
    }

    /// Filter specific characters
    pub fn filter_characters(&self, text: &str) -> String {
        if let Some(ref filter_chars) = self.normalization_config.filter_characters {
            text.chars().filter(|c| !filter_chars.contains(c)).collect()
        } else {
            text.to_string()
        }
    }

    /// Keep only alphanumeric characters
    pub fn preserve_alphanumeric_only(&self, text: &str) -> String {
        if self.normalization_config.preserve_alphanumeric_only {
            text.chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect()
        } else {
            text.to_string()
        }
    }

    /// Comprehensive text normalization
    pub fn normalize_text(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Apply normalization steps in order
        result = self.normalize_unicode(&result);
        result = self.normalize_case(&result);
        result = self.filter_characters(&result);
        result = self.remove_punctuation(&result);
        result = self.preserve_alphanumeric_only(&result);
        result = self.normalize_whitespace(&result);

        result
    }

    /// Segment text into words
    pub fn segment_text(&self, text: &str) -> Vec<String> {
        if self.enable_chinese && self.contains_chinese(text) {
            // For Chinese text, we need more sophisticated segmentation
            self.segment_chinese_text(text)
        } else {
            // For non-Chinese text, simple whitespace splitting
            text.split_whitespace().map(|s| s.to_string()).collect()
        }
    }

    /// Segment Chinese text (basic implementation)
    fn segment_chinese_text(&self, text: &str) -> Vec<String> {
        let mut segments = Vec::new();
        let mut current_segment = String::new();

        for ch in text.chars() {
            if self.is_chinese_char(ch) {
                // For Chinese characters, each character can be a segment
                if !current_segment.is_empty() {
                    segments.push(current_segment.clone());
                    current_segment.clear();
                }
                segments.push(ch.to_string());
            } else if ch.is_whitespace() {
                if !current_segment.is_empty() {
                    segments.push(current_segment.clone());
                    current_segment.clear();
                }
            } else {
                current_segment.push(ch);
            }
        }

        if !current_segment.is_empty() {
            segments.push(current_segment);
        }

        segments
    }

    /// Check if character is Chinese
    fn is_chinese_char(&self, c: char) -> bool {
        let code = c as u32;
        // Basic Chinese character ranges
        (0x4E00..=0x9FFF).contains(&code) || // CJK Unified Ideographs
        (0x3400..=0x4DBF).contains(&code) || // CJK Extension A
        (0x20000..=0x2A6DF).contains(&code) || // CJK Extension B
        (0x2A700..=0x2B73F).contains(&code) || // CJK Extension C
        (0x2B740..=0x2B81F).contains(&code) || // CJK Extension D
        (0x2B820..=0x2CEAF).contains(&code) // CJK Extension E
    }

    /// Check if text contains Chinese characters
    pub fn contains_chinese(&self, text: &str) -> bool {
        text.chars().any(|c| self.is_chinese_char(c))
    }

    /// Simple Latin character folding for diacritics
    fn fold_latin_char(&self, c: char) -> char {
        match c {
            'à'..='å' | 'À'..='Å' => 'a',
            'è'..='ë' | 'È'..='Ë' => 'e',
            'ì'..='ï' | 'Ì'..='Ï' => 'i',
            'ò'..='ö' | 'Ò'..='Ö' | 'ø' | 'Ø' => 'o',
            'ù'..='ü' | 'Ù'..='Ü' => 'u',
            'ý' | 'ÿ' | 'Ý' => 'y',
            'ñ' | 'Ñ' => 'n',
            'ç' | 'Ç' => 'c',
            _ => c,
        }
    }

    /// Convert traditional Chinese to simplified (enhanced implementation)
    pub fn convert_traditional(&self, text: &str) -> String {
        if !self.enable_chinese {
            return text.to_string();
        }

        // Enhanced traditional to simplified conversion
        // This is still a basic implementation - in production you'd use a proper library
        let mut result = text.to_string();

        // Common traditional to simplified mappings
        let mappings = [
            ("繁體", "繁体"),
            ("簡體", "简体"),
            ("國家", "国家"),
            ("學習", "学习"),
            ("電腦", "电脑"),
            ("網路", "网络"),
            ("資料", "资料"),
            ("檔案", "档案"),
            ("軟體", "软件"),
            ("開發", "开发"),
            ("測試", "测试"),
            ("設計", "设计"),
            ("應用", "应用"),
            ("系統", "系统"),
            ("處理", "处理"),
            ("執行", "执行"),
            ("運行", "运行"),
            ("環境", "环境"),
            ("變數", "变量"),
            ("函數", "函数"),
            ("類別", "类别"),
            ("物件", "对象"),
            ("屬性", "属性"),
            ("方法", "方法"),
            ("介面", "接口"),
            ("實作", "实现"),
            ("繼承", "继承"),
            ("封裝", "封装"),
            ("多型", "多态"),
            ("抽象", "抽象"),
        ];

        for (traditional, simplified) in &mappings {
            result = result.replace(traditional, simplified);
        }

        debug!("Converted traditional Chinese: {} -> {}", text, result);
        result
    }

    /// Detect Chinese text type (simplified, traditional, or mixed)
    pub fn detect_chinese_type(&self, text: &str) -> ChineseTextType {
        if !self.enable_chinese || !self.contains_chinese(text) {
            return ChineseTextType::None;
        }

        let mut simplified_count = 0;
        let mut traditional_count = 0;
        let mut total_chinese_chars = 0;

        for ch in text.chars() {
            if self.is_chinese_char(ch) {
                total_chinese_chars += 1;

                // Check if character is likely traditional or simplified
                if self.is_likely_traditional_char(ch) {
                    traditional_count += 1;
                } else if self.is_likely_simplified_char(ch) {
                    simplified_count += 1;
                }
            }
        }

        if total_chinese_chars == 0 {
            return ChineseTextType::None;
        }

        let traditional_ratio = traditional_count as f64 / total_chinese_chars as f64;
        let simplified_ratio = simplified_count as f64 / total_chinese_chars as f64;

        if traditional_ratio > 0.3 && simplified_ratio > 0.3 {
            ChineseTextType::Mixed
        } else if traditional_ratio > simplified_ratio {
            ChineseTextType::Traditional
        } else if simplified_ratio > 0.1 {
            ChineseTextType::Simplified
        } else {
            ChineseTextType::Unknown
        }
    }

    /// Check if character is likely traditional Chinese
    fn is_likely_traditional_char(&self, c: char) -> bool {
        // Common traditional Chinese characters that have simplified variants
        matches!(
            c,
            '繁' | '體'
                | '國'
                | '學'
                | '電'
                | '網'
                | '資'
                | '檔'
                | '軟'
                | '開'
                | '測'
                | '設'
                | '應'
                | '統'
                | '處'
                | '執'
                | '運'
                | '環'
                | '變'
                | '數'
                | '類'
                | '別'
                | '物'
                | '件'
                | '屬'
                | '性'
                | '實'
                | '作'
                | '繼'
                | '承'
                | '封'
                | '裝'
                | '態'
                | '象'
                | '議'
                | '題'
                | '問'
                | '決'
        )
    }

    /// Check if character is likely simplified Chinese
    fn is_likely_simplified_char(&self, c: char) -> bool {
        // Common simplified Chinese characters
        matches!(
            c,
            '简' | '体'
                | '国'
                | '学'
                | '电'
                | '网'
                | '资'
                | '档'
                | '软'
                | '开'
                | '测'
                | '设'
                | '应'
                | '统'
                | '处'
                | '执'
                | '运'
                | '环'
                | '变'
                | '量'
                | '类'
                | '别'
                | '对'
                | '象'
                | '属'
                | '性'
                | '实'
                | '现'
                | '继'
                | '承'
                | '封'
                | '装'
                | '态'
                | '议'
                | '题'
                | '问'
                | '决'
        )
    }

    /// Extract Chinese characters from mixed text
    pub fn extract_chinese_chars(&self, text: &str) -> String {
        if !self.enable_chinese {
            return String::new();
        }

        text.chars().filter(|&c| self.is_chinese_char(c)).collect()
    }

    /// Extract non-Chinese characters from mixed text
    pub fn extract_non_chinese_chars(&self, text: &str) -> String {
        text.chars().filter(|&c| !self.is_chinese_char(c)).collect()
    }

    /// Process mixed Chinese and English text
    pub fn process_mixed_text(&self, text: &str) -> MixedTextResult {
        let chinese_chars = self.extract_chinese_chars(text);
        let non_chinese_chars = self.extract_non_chinese_chars(text);
        let chinese_type = self.detect_chinese_type(text);

        let simplified_chinese = if chinese_type == ChineseTextType::Traditional {
            self.convert_traditional(&chinese_chars)
        } else {
            chinese_chars.clone()
        };

        MixedTextResult {
            original: text.to_string(),
            chinese_chars: chinese_chars.clone(),
            non_chinese_chars: non_chinese_chars.trim().to_string(),
            simplified_chinese,
            chinese_type,
            has_mixed_content: !chinese_chars.is_empty() && !non_chinese_chars.trim().is_empty(),
        }
    }

    /// Generate pinyin variants (enhanced implementation)
    pub fn generate_pinyin_variants(&self, text: &str) -> Vec<String> {
        if !self.enable_chinese || !self.contains_chinese(text) {
            return vec![text.to_string()];
        }

        self.generate_pinyin_with_style(text, &PinyinStyle::Normal)
    }

    /// Generate pinyin with specific style
    pub fn generate_pinyin_with_style(&self, text: &str, style: &PinyinStyle) -> Vec<String> {
        if !self.enable_chinese || !self.contains_chinese(text) {
            return vec![text.to_string()];
        }

        let mut variants = Vec::new();

        // Basic pinyin mapping for common Chinese characters
        // In a real implementation, you would use a comprehensive pinyin dictionary
        let pinyin_map = self.get_basic_pinyin_map();

        let mut current_variant = String::new();
        let mut has_chinese = false;

        for ch in text.chars() {
            if self.is_chinese_char(ch) {
                has_chinese = true;
                if let Some(pinyin_options) = pinyin_map.get(&ch) {
                    // For now, just take the first pinyin option
                    let pinyin = &pinyin_options[0];
                    let styled_pinyin = self.apply_pinyin_style(pinyin, style);
                    current_variant.push_str(&styled_pinyin);
                } else {
                    // Fallback for unknown characters
                    current_variant.push(ch);
                }
            } else {
                current_variant.push(ch);
            }
        }

        if has_chinese {
            variants.push(current_variant);

            // Generate additional variants for different styles
            if style == &PinyinStyle::Normal {
                variants
                    .extend(self.generate_pinyin_with_style(text, &PinyinStyle::WithoutTone));
                variants
                    .extend(self.generate_pinyin_with_style(text, &PinyinStyle::FirstLetter));
            }
        } else {
            variants.push(text.to_string());
        }

        // Remove duplicates and empty strings
        variants.sort();
        variants.dedup();
        variants.retain(|s| !s.trim().is_empty());

        debug!("Generated pinyin variants for '{}': {:?}", text, variants);
        variants
    }

    /// Apply pinyin style formatting
    fn apply_pinyin_style(&self, pinyin: &str, style: &PinyinStyle) -> String {
        match style {
            PinyinStyle::Normal | PinyinStyle::Numeric => pinyin.to_string(),
            PinyinStyle::WithTone => self.convert_numeric_tone_to_diacritic(pinyin),
            PinyinStyle::WithoutTone => self.remove_tone_marks(pinyin),
            PinyinStyle::FirstLetter => pinyin.chars().next().unwrap_or(' ').to_string(),
        }
    }

    /// Convert numeric tones to diacritic marks
    fn convert_numeric_tone_to_diacritic(&self, pinyin: &str) -> String {
        // Basic tone mark conversion
        let tone_map = [
            ("a1", "ā"),
            ("a2", "á"),
            ("a3", "ǎ"),
            ("a4", "à"),
            ("e1", "ē"),
            ("e2", "é"),
            ("e3", "ě"),
            ("e4", "è"),
            ("i1", "ī"),
            ("i2", "í"),
            ("i3", "ǐ"),
            ("i4", "ì"),
            ("o1", "ō"),
            ("o2", "ó"),
            ("o3", "ǒ"),
            ("o4", "ò"),
            ("u1", "ū"),
            ("u2", "ú"),
            ("u3", "ǔ"),
            ("u4", "ù"),
            ("v1", "ǖ"),
            ("v2", "ǘ"),
            ("v3", "ǚ"),
            ("v4", "ǜ"),
        ];

        let mut result = pinyin.to_string();
        for (numeric, diacritic) in &tone_map {
            result = result.replace(numeric, diacritic);
        }

        // Remove remaining numbers
        result = result.chars().filter(|c| !c.is_ascii_digit()).collect();
        result
    }

    /// Remove tone marks from pinyin
    fn remove_tone_marks(&self, pinyin: &str) -> String {
        let tone_map = [
            ("ā", "a"),
            ("á", "a"),
            ("ǎ", "a"),
            ("à", "a"),
            ("ē", "e"),
            ("é", "e"),
            ("ě", "e"),
            ("è", "e"),
            ("ī", "i"),
            ("í", "i"),
            ("ǐ", "i"),
            ("ì", "i"),
            ("ō", "o"),
            ("ó", "o"),
            ("ǒ", "o"),
            ("ò", "o"),
            ("ū", "u"),
            ("ú", "u"),
            ("ǔ", "u"),
            ("ù", "u"),
            ("ǖ", "v"),
            ("ǘ", "v"),
            ("ǚ", "v"),
            ("ǜ", "v"),
        ];

        let mut result = pinyin.to_string();
        for (diacritic, base) in &tone_map {
            result = result.replace(diacritic, base);
        }

        // Remove numbers
        result = result.chars().filter(|c| !c.is_ascii_digit()).collect();
        result
    }

    /// Get basic pinyin mapping for common characters
    fn get_basic_pinyin_map(&self) -> HashMap<char, Vec<String>> {
        let mut map = HashMap::new();

        // Common Chinese characters with their pinyin
        // This is a very basic set - a real implementation would have thousands
        map.insert('中', vec!["zhong1".to_string()]);
        map.insert('文', vec!["wen2".to_string()]);
        map.insert('你', vec!["ni3".to_string()]);
        map.insert('好', vec!["hao3".to_string()]);
        map.insert('世', vec!["shi4".to_string()]);
        map.insert('界', vec!["jie4".to_string()]);
        map.insert('学', vec!["xue2".to_string()]);
        map.insert('習', vec!["xi2".to_string()]);
        map.insert('习', vec!["xi2".to_string()]);
        map.insert('电', vec!["dian4".to_string()]);
        map.insert('電', vec!["dian4".to_string()]);
        map.insert('脑', vec!["nao3".to_string()]);
        map.insert('腦', vec!["nao3".to_string()]);
        map.insert('网', vec!["wang3".to_string()]);
        map.insert('網', vec!["wang3".to_string()]);
        map.insert('络', vec!["luo4".to_string()]);
        map.insert('路', vec!["lu4".to_string()]);
        map.insert('文', vec!["wen2".to_string()]);
        map.insert('件', vec!["jian4".to_string()]);
        map.insert('档', vec!["dang4".to_string()]);
        map.insert('檔', vec!["dang4".to_string()]);
        map.insert('案', vec!["an4".to_string()]);
        map.insert('管', vec!["guan3".to_string()]);
        map.insert('理', vec!["li3".to_string()]);
        map.insert('工', vec!["gong1".to_string()]);
        map.insert('具', vec!["ju4".to_string()]);
        map.insert('系', vec!["xi4".to_string()]);
        map.insert('统', vec!["tong3".to_string()]);
        map.insert('統', vec!["tong3".to_string()]);

        map
    }

    /// Generate comprehensive pinyin result
    pub fn generate_comprehensive_pinyin(&self, text: &str, style: PinyinStyle) -> PinyinResult {
        let pinyin_variants = self.generate_pinyin_with_style(text, &style);
        let combinations = self.create_combinations(text, &pinyin_variants);

        PinyinResult {
            original: text.to_string(),
            pinyin_variants,
            style,
            combinations,
        }
    }

    /// Create keyword combinations
    pub fn create_combinations(&self, original: &str, variants: &[String]) -> Vec<Vec<String>> {
        let mut combinations = Vec::new();

        // Add original
        combinations.push(vec![original.to_string()]);

        // Add variants
        for variant in variants {
            combinations.push(vec![variant.clone()]);
        }

        // Add combinations of original + variants
        for variant in variants {
            if variant != original {
                combinations.push(vec![original.to_string(), variant.clone()]);
            }
        }

        combinations
    }
}

/// Path utilities for common path operations
pub struct PathUtils;

impl PathUtils {
    /// Get the file size in bytes
    pub fn get_file_size<P: AsRef<Path>>(path: P) -> FileManagementResult<u64> {
        let metadata = std::fs::metadata(path.as_ref()).map_err(|e| {
            FileManagementError::io(
                format!("Failed to get metadata for {}", path.as_ref().display()),
                e,
            )
        })?;
        Ok(metadata.len())
    }

    /// Get the directory size recursively
    pub fn get_directory_size<P: AsRef<Path>>(path: P) -> FileManagementResult<u64> {
        let path = path.as_ref();
        let mut total_size = 0;

        if path.is_file() {
            return Self::get_file_size(path);
        }

        let entries = std::fs::read_dir(path).map_err(|e| {
            FileManagementError::io(format!("Failed to read directory {}", path.display()), e)
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                FileManagementError::io(
                    format!("Failed to read directory entry in {}", path.display()),
                    e,
                )
            })?;

            let entry_path = entry.path();
            if entry_path.is_dir() {
                total_size += Self::get_directory_size(&entry_path)?;
            } else {
                total_size += Self::get_file_size(&entry_path)?;
            }
        }

        Ok(total_size)
    }

    /// Check if two paths refer to the same file/directory
    pub fn paths_equal<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> bool {
        match (path1.as_ref().canonicalize(), path2.as_ref().canonicalize()) {
            (Ok(p1), Ok(p2)) => p1 == p2,
            _ => false,
        }
    }

    /// Generate a unique filename if the target already exists
    pub fn generate_unique_name<P: AsRef<Path>>(target_path: P) -> PathBuf {
        let path = target_path.as_ref();

        if !path.exists() {
            return path.to_path_buf();
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let extension = path.extension().and_then(|s| s.to_str());

        for i in 1..=9999 {
            let new_name = if let Some(ext) = extension {
                format!("{}_{}.{}", stem, i, ext)
            } else {
                format!("{}_{}", stem, i)
            };

            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return new_path;
            }
        }

        // Fallback with timestamp
        let timestamp = chrono::Utc::now().timestamp();
        let uuid = Uuid::new_v4().simple();
        let new_name = if let Some(ext) = extension {
            format!("{}_{}_{}.{}", stem, timestamp, uuid, ext)
        } else {
            format!("{}_{}", stem, timestamp)
        };

        parent.join(new_name)
    }
}

/// Validation utilities for parameters and configurations
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate that a string is not empty after trimming
    pub fn validate_non_empty_string(value: &str, field_name: &str) -> FileManagementResult<()> {
        if value.trim().is_empty() {
            return Err(FileManagementError::validation(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }

    /// Validate that a number is within a range
    pub fn validate_range<T: PartialOrd + std::fmt::Display>(
        value: T,
        min: T,
        max: T,
        field_name: &str,
    ) -> FileManagementResult<()> {
        if value < min || value > max {
            return Err(FileManagementError::validation(format!(
                "{} must be between {} and {}, got {}",
                field_name, min, max, value
            )));
        }
        Ok(())
    }

    /// Validate that a path exists and is accessible
    pub fn validate_path_exists<P: AsRef<Path>>(path: P) -> FileManagementResult<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(FileManagementError::not_found(path));
        }
        Ok(())
    }

    /// Validate that a directory exists and is writable
    pub fn validate_writable_directory<P: AsRef<Path>>(path: P) -> FileManagementResult<()> {
        let path = path.as_ref();

        Self::validate_path_exists(path)?;

        if !path.is_dir() {
            return Err(FileManagementError::invalid_path(
                path,
                "Path is not a directory",
            ));
        }

        // Try to create a temporary file to test writability
        let test_file = path.join(format!(".test_write_{}", Uuid::new_v4().simple()));
        match std::fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                Ok(())
            }
            Err(_e) => Err(FileManagementError::permission_denied(path)),
        }
    }
}

/// Experimental mode context for simulating operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalMode {
    pub enabled: bool,
    pub operations_log: Vec<ExperimentalOperation>,
}

impl ExperimentalMode {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            operations_log: Vec::new(),
        }
    }

    pub fn log_operation(&mut self, operation: ExperimentalOperation) {
        if self.enabled {
            self.operations_log.push(operation);
        }
    }

    pub fn get_operations(&self) -> &[ExperimentalOperation] {
        &self.operations_log
    }

    pub fn clear_log(&mut self) {
        self.operations_log.clear();
    }
}

/// Represents an operation that would be performed in experimental mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalOperation {
    pub operation_type: String,
    pub source_path: Option<PathBuf>,
    pub target_path: Option<PathBuf>,
    pub description: String,
    pub estimated_size: Option<u64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ExperimentalOperation {
    pub fn new<S: Into<String>>(operation_type: S, description: S) -> Self {
        Self {
            operation_type: operation_type.into(),
            source_path: None,
            target_path: None,
            description: description.into(),
            estimated_size: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_source_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.source_path = Some(path.into());
        self
    }

    pub fn with_target_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.target_path = Some(path.into());
        self
    }

    pub fn with_estimated_size(mut self, size: u64) -> Self {
        self.estimated_size = Some(size);
        self
    }
}

/// Folder comparison result for merging operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderComparisonResult {
    pub common_folders: Vec<CommonFolderInfo>,
    pub unique_folders: Vec<UniqueFolderInfo>,
    pub total_folders_analyzed: usize,
    pub total_size_bytes: u64,
    pub merge_recommendations: Vec<MergeRecommendation>,
}

/// Information about folders with identical names found in multiple locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonFolderInfo {
    pub folder_name: String,
    pub locations: Vec<FolderLocationInfo>,
    pub recommended_merge_direction: MergeDirection,
    pub total_size_all_locations: u64,
    pub duplicate_files_count: usize,
    pub unique_files_count: usize,
}

/// Information about a folder location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderLocationInfo {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: usize,
    pub subdirectory_count: usize,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub is_writable: bool,
}

/// Information about folders that are unique to one location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueFolderInfo {
    pub folder_name: String,
    pub location: FolderLocationInfo,
}

/// Merge direction recommendation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeDirection {
    /// Merge all into the largest folder
    IntoLargest,
    /// Merge all into the most recently modified folder
    IntoNewest,
    /// Merge all into the first location found
    IntoFirst,
    /// Merge all into a specific location (index in locations array)
    IntoSpecific(usize),
    /// Manual decision required
    Manual,
}

/// Merge recommendation with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRecommendation {
    pub folder_name: String,
    pub recommended_direction: MergeDirection,
    pub reasoning: String,
    pub confidence_score: f64, // 0.0 to 1.0
    pub estimated_space_saved: u64,
    pub potential_conflicts: usize,
}

/// Folder merger for intelligent folder merging operations
pub struct FolderMerger {
    config: FolderMergerConfig,
}

/// Configuration for folder merger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMergerConfig {
    pub merge_strategy: MergeStrategy,
    pub duplicate_handling: DuplicateHandling,
    pub max_recursion_depth: usize,
    pub min_confidence_threshold: f64,
    pub enable_size_based_decisions: bool,
    pub enable_date_based_decisions: bool,
    pub dry_run: bool,
}

impl Default for FolderMergerConfig {
    fn default() -> Self {
        Self {
            merge_strategy: MergeStrategy::SizeBased,
            duplicate_handling: DuplicateHandling::Rename,
            max_recursion_depth: 10,
            min_confidence_threshold: 0.7,
            enable_size_based_decisions: true,
            enable_date_based_decisions: true,
            dry_run: false,
        }
    }
}

/// Merge strategy for determining merge direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Merge into the folder with the largest size
    SizeBased,
    /// Merge into the most recently modified folder
    DateBased,
    /// Always ask for manual decision
    Manual,
    /// Use a combination of size and date with confidence scoring
    Intelligent,
}

/// How to handle duplicate files during merge
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DuplicateHandling {
    /// Skip duplicate files
    Skip,
    /// Rename duplicate files
    Rename,
    /// Keep the newer file
    KeepNewer,
    /// Keep the larger file
    KeepLarger,
    /// Merge file contents if possible
    Merge,
}

impl FolderMerger {
    /// Create a new folder merger with default configuration
    pub fn new() -> Self {
        Self {
            config: FolderMergerConfig::default(),
        }
    }

    /// Create a new folder merger with custom configuration
    pub fn with_config(config: FolderMergerConfig) -> Self {
        Self { config }
    }

    /// Compare folders across multiple source directories to identify merge candidates
    pub fn compare_folders<P: AsRef<Path>>(
        &self,
        source_directories: &[P],
    ) -> FileManagementResult<FolderComparisonResult> {
        debug!(
            "Comparing folders across {} source directories",
            source_directories.len()
        );

        let mut folder_map: HashMap<String, Vec<FolderLocationInfo>> = HashMap::new();
        let mut total_folders_analyzed = 0;
        let mut total_size_bytes = 0;

        // Scan all source directories
        for source_dir in source_directories {
            let source_path = source_dir.as_ref();

            if !source_path.exists() {
                warn!("Source directory does not exist: {}", source_path.display());
                continue;
            }

            if !source_path.is_dir() {
                warn!("Source path is not a directory: {}", source_path.display());
                continue;
            }

            // Read directory contents
            let entries = std::fs::read_dir(source_path).map_err(|e| {
                FileManagementError::io(
                    format!("Failed to read source directory {}", source_path.display()),
                    e,
                )
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    FileManagementError::io(
                        format!(
                            "Failed to read directory entry in {}",
                            source_path.display()
                        ),
                        e,
                    )
                })?;

                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let folder_name = entry_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let location_info = self.analyze_folder_location(&entry_path)?;
                    total_size_bytes += location_info.size_bytes;
                    total_folders_analyzed += 1;

                    folder_map
                        .entry(folder_name)
                        .or_default()
                        .push(location_info);
                }
            }
        }

        // Separate common and unique folders
        let mut common_folders = Vec::new();
        let mut unique_folders = Vec::new();

        for (folder_name, locations) in folder_map {
            if locations.len() > 1 {
                // Common folder - found in multiple locations
                let common_info = self.analyze_common_folder(&folder_name, locations)?;
                common_folders.push(common_info);
            } else if let Some(location) = locations.into_iter().next() {
                // Unique folder - found in only one location
                unique_folders.push(UniqueFolderInfo {
                    folder_name,
                    location,
                });
            }
        }

        // Generate merge recommendations
        let merge_recommendations = self.generate_merge_recommendations(&common_folders)?;

        debug!(
            "Folder comparison complete: {} common folders, {} unique folders, {} recommendations",
            common_folders.len(),
            unique_folders.len(),
            merge_recommendations.len()
        );

        Ok(FolderComparisonResult {
            common_folders,
            unique_folders,
            total_folders_analyzed,
            total_size_bytes,
            merge_recommendations,
        })
    }

    /// Analyze a single folder location to gather metadata
    fn analyze_folder_location(
        &self,
        folder_path: &Path,
    ) -> FileManagementResult<FolderLocationInfo> {
        let size_bytes = PathUtils::get_directory_size(folder_path)?;

        let metadata = std::fs::metadata(folder_path).map_err(|e| {
            FileManagementError::io(
                format!("Failed to get metadata for {}", folder_path.display()),
                e,
            )
        })?;

        let last_modified = metadata.modified().ok().and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|duration| {
                    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                        .unwrap_or_else(chrono::Utc::now)
                })
        });

        // Count files and subdirectories
        let (file_count, subdirectory_count) = self.count_folder_contents(folder_path)?;

        // Check if folder is writable
        let is_writable = self.check_folder_writable(folder_path);

        Ok(FolderLocationInfo {
            path: folder_path.to_path_buf(),
            size_bytes,
            file_count,
            subdirectory_count,
            last_modified,
            is_writable,
        })
    }

    /// Count files and subdirectories in a folder
    fn count_folder_contents(&self, folder_path: &Path) -> FileManagementResult<(usize, usize)> {
        let mut file_count = 0;
        let mut subdirectory_count = 0;

        let entries = std::fs::read_dir(folder_path).map_err(|e| {
            FileManagementError::io(
                format!("Failed to read folder contents {}", folder_path.display()),
                e,
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                FileManagementError::io(
                    format!("Failed to read folder entry in {}", folder_path.display()),
                    e,
                )
            })?;

            if entry.path().is_dir() {
                subdirectory_count += 1;
            } else {
                file_count += 1;
            }
        }

        Ok((file_count, subdirectory_count))
    }

    /// Check if a folder is writable
    fn check_folder_writable(&self, folder_path: &Path) -> bool {
        // Try to create a temporary file to test writability
        let test_file = folder_path.join(format!(".test_write_{}", uuid::Uuid::new_v4().simple()));
        match std::fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                true
            }
            Err(_) => false,
        }
    }

    /// Analyze a common folder found in multiple locations
    fn analyze_common_folder(
        &self,
        folder_name: &str,
        locations: Vec<FolderLocationInfo>,
    ) -> FileManagementResult<CommonFolderInfo> {
        let total_size_all_locations: u64 = locations.iter().map(|loc| loc.size_bytes).sum();

        // Determine recommended merge direction
        let recommended_merge_direction = self.determine_merge_direction(&locations)?;

        // Analyze for duplicate files (simplified - in a real implementation this would be more thorough)
        let (duplicate_files_count, unique_files_count) =
            self.estimate_duplicate_files(&locations)?;

        Ok(CommonFolderInfo {
            folder_name: folder_name.to_string(),
            locations,
            recommended_merge_direction,
            total_size_all_locations,
            duplicate_files_count,
            unique_files_count,
        })
    }

    /// Determine the best merge direction based on configuration and folder analysis
    fn determine_merge_direction(
        &self,
        locations: &[FolderLocationInfo],
    ) -> FileManagementResult<MergeDirection> {
        match self.config.merge_strategy {
            MergeStrategy::SizeBased => {
                if self.config.enable_size_based_decisions {
                    // Find the location with the largest size
                    let largest_index = locations
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, loc)| loc.size_bytes)
                        .map(|(index, _)| index)
                        .unwrap_or(0);
                    Ok(MergeDirection::IntoSpecific(largest_index))
                } else {
                    Ok(MergeDirection::IntoLargest)
                }
            }
            MergeStrategy::DateBased => {
                if self.config.enable_date_based_decisions {
                    // Find the location with the most recent modification
                    let newest_index = locations
                        .iter()
                        .enumerate()
                        .filter_map(|(index, loc)| loc.last_modified.map(|date| (index, date)))
                        .max_by_key(|(_, date)| *date)
                        .map(|(index, _)| index)
                        .unwrap_or(0);
                    Ok(MergeDirection::IntoSpecific(newest_index))
                } else {
                    Ok(MergeDirection::IntoNewest)
                }
            }
            MergeStrategy::Manual => Ok(MergeDirection::Manual),
            MergeStrategy::Intelligent => {
                // Use intelligent decision making combining size and date
                self.intelligent_merge_direction(locations)
            }
        }
    }

    /// Intelligent merge direction using multiple factors
    fn intelligent_merge_direction(
        &self,
        locations: &[FolderLocationInfo],
    ) -> FileManagementResult<MergeDirection> {
        let mut scores: Vec<(usize, f64)> = Vec::new();

        for (index, location) in locations.iter().enumerate() {
            let mut score = 0.0;

            // Size factor (normalized)
            if self.config.enable_size_based_decisions {
                let max_size = locations
                    .iter()
                    .map(|loc| loc.size_bytes)
                    .max()
                    .unwrap_or(1);
                let size_score = location.size_bytes as f64 / max_size as f64;
                score += size_score * 0.4; // 40% weight for size
            }

            // Date factor (normalized)
            if self.config.enable_date_based_decisions {
                if let Some(modified) = location.last_modified {
                    let newest_time = locations
                        .iter()
                        .filter_map(|loc| loc.last_modified)
                        .max()
                        .unwrap_or(modified);

                    let oldest_time = locations
                        .iter()
                        .filter_map(|loc| loc.last_modified)
                        .min()
                        .unwrap_or(modified);

                    if newest_time != oldest_time {
                        let time_range = (newest_time - oldest_time).num_seconds() as f64;
                        let time_score = if time_range > 0.0 {
                            (modified - oldest_time).num_seconds() as f64 / time_range
                        } else {
                            1.0
                        };
                        score += time_score * 0.3; // 30% weight for recency
                    }
                }
            }

            // Writability factor
            if location.is_writable {
                score += 0.2; // 20% bonus for writable locations
            }

            // File count factor (more files might indicate more active use)
            let max_files = locations
                .iter()
                .map(|loc| loc.file_count)
                .max()
                .unwrap_or(1);
            let file_score = location.file_count as f64 / max_files as f64;
            score += file_score * 0.1; // 10% weight for file count

            scores.push((index, score));
        }

        // Find the highest scoring location
        let best_location = scores
            .into_iter()
            .max_by(|(_, score_a), (_, score_b)| {
                score_a
                    .partial_cmp(score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, score)| (index, score))
            .unwrap_or((0, 0.0));

        // If confidence is too low, require manual decision
        if best_location.1 < self.config.min_confidence_threshold {
            Ok(MergeDirection::Manual)
        } else {
            Ok(MergeDirection::IntoSpecific(best_location.0))
        }
    }

    /// Estimate duplicate and unique files across locations (simplified implementation)
    fn estimate_duplicate_files(
        &self,
        locations: &[FolderLocationInfo],
    ) -> FileManagementResult<(usize, usize)> {
        // This is a simplified estimation - a real implementation would compare file contents
        let total_files: usize = locations.iter().map(|loc| loc.file_count).sum();
        let max_files = locations
            .iter()
            .map(|loc| loc.file_count)
            .max()
            .unwrap_or(0);

        // Rough estimation: assume some overlap based on folder sizes
        let estimated_duplicates = if total_files > max_files {
            (total_files - max_files) / 2 // Conservative estimate
        } else {
            0
        };

        let estimated_unique = total_files - estimated_duplicates;

        Ok((estimated_duplicates, estimated_unique))
    }

    /// Generate merge recommendations for all common folders
    fn generate_merge_recommendations(
        &self,
        common_folders: &[CommonFolderInfo],
    ) -> FileManagementResult<Vec<MergeRecommendation>> {
        let mut recommendations = Vec::new();

        for common_folder in common_folders {
            let recommendation = self.generate_single_merge_recommendation(common_folder)?;
            recommendations.push(recommendation);
        }

        // Sort recommendations by confidence score (highest first)
        recommendations.sort_by(|a, b| {
            b.confidence_score
                .partial_cmp(&a.confidence_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(recommendations)
    }

    /// Generate a merge recommendation for a single common folder
    fn generate_single_merge_recommendation(
        &self,
        common_folder: &CommonFolderInfo,
    ) -> FileManagementResult<MergeRecommendation> {
        let mut reasoning = String::new();
        let mut confidence_score = 0.5; // Base confidence

        // Calculate estimated space saved
        let total_size = common_folder.total_size_all_locations;
        let largest_size = common_folder
            .locations
            .iter()
            .map(|loc| loc.size_bytes)
            .max()
            .unwrap_or(0);
        let estimated_space_saved = total_size.saturating_sub(largest_size);

        // Generate reasoning based on merge direction
        match &common_folder.recommended_merge_direction {
            MergeDirection::IntoLargest => {
                reasoning.push_str("Merge into largest folder to minimize data movement");
                confidence_score += 0.2;
            }
            MergeDirection::IntoNewest => {
                reasoning.push_str(
                    "Merge into most recently modified folder to preserve recent changes",
                );
                confidence_score += 0.2;
            }
            MergeDirection::IntoSpecific(index) => {
                if let Some(target_location) = common_folder.locations.get(*index) {
                    reasoning.push_str(&format!(
                        "Merge into {} based on intelligent analysis (size: {} bytes, writable: {})",
                        target_location.path.display(),
                        target_location.size_bytes,
                        target_location.is_writable
                    ));
                    confidence_score += 0.3;
                } else {
                    reasoning.push_str("Merge into first available location");
                    confidence_score -= 0.1;
                }
            }
            MergeDirection::Manual => {
                reasoning.push_str("Manual decision required due to ambiguous merge conditions");
                confidence_score = 0.1;
            }
            _ => {
                reasoning.push_str("Standard merge operation");
            }
        }

        // Adjust confidence based on potential conflicts
        let potential_conflicts = common_folder.duplicate_files_count;
        if potential_conflicts > 0 {
            reasoning.push_str(&format!(
                ", {} potential file conflicts detected",
                potential_conflicts
            ));
            confidence_score -= (potential_conflicts as f64 * 0.05).min(0.3); // Reduce confidence for conflicts
        }

        // Ensure confidence is within bounds
        confidence_score = confidence_score.clamp(0.0, 1.0);

        Ok(MergeRecommendation {
            folder_name: common_folder.folder_name.clone(),
            recommended_direction: common_folder.recommended_merge_direction.clone(),
            reasoning,
            confidence_score,
            estimated_space_saved,
            potential_conflicts,
        })
    }

    /// Execute folder merge operations based on comparison results
    pub async fn execute_merge_operations(
        &self,
        comparison_result: &FolderComparisonResult,
        file_operation_manager: &FileOperationManager,
    ) -> FileManagementResult<FolderMergeResult> {
        debug!(
            "Executing merge operations for {} common folders",
            comparison_result.common_folders.len()
        );

        let mut merge_results = Vec::new();
        let mut total_operations = 0;
        let mut successful_operations = 0;
        let mut failed_operations = 0;
        let mut total_bytes_moved = 0;
        let start_time = std::time::Instant::now();

        for common_folder in &comparison_result.common_folders {
            let merge_result = self
                .execute_single_folder_merge(common_folder, file_operation_manager)
                .await?;

            total_operations += merge_result.operations_performed;
            successful_operations += merge_result.successful_operations;
            failed_operations += merge_result.failed_operations;
            total_bytes_moved += merge_result.bytes_moved;

            merge_results.push(merge_result);
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(FolderMergeResult {
            merge_results,
            total_operations,
            successful_operations,
            failed_operations,
            total_bytes_moved,
            duration_ms,
            folders_merged: comparison_result.common_folders.len(),
        })
    }

    /// Execute merge operation for a single common folder
    async fn execute_single_folder_merge(
        &self,
        common_folder: &CommonFolderInfo,
        file_operation_manager: &FileOperationManager,
    ) -> FileManagementResult<SingleFolderMergeResult> {
        debug!("Executing merge for folder: {}", common_folder.folder_name);

        // Determine target location based on merge direction
        let target_index = match &common_folder.recommended_merge_direction {
            MergeDirection::IntoLargest => common_folder
                .locations
                .iter()
                .enumerate()
                .max_by_key(|(_, loc)| loc.size_bytes)
                .map(|(index, _)| index)
                .unwrap_or(0),
            MergeDirection::IntoNewest => common_folder
                .locations
                .iter()
                .enumerate()
                .filter_map(|(index, loc)| loc.last_modified.map(|date| (index, date)))
                .max_by_key(|(_, date)| *date)
                .map(|(index, _)| index)
                .unwrap_or(0),
            MergeDirection::IntoFirst => 0,
            MergeDirection::IntoSpecific(index) => *index,
            MergeDirection::Manual => {
                return Err(FileManagementError::unsupported_operation(
                    "Manual merge direction requires human decision",
                ));
            }
        };

        let target_location = common_folder.locations.get(target_index).ok_or_else(|| {
            FileManagementError::validation(format!(
                "Invalid target index {} for folder {}",
                target_index, common_folder.folder_name
            ))
        })?;

        let mut operations_performed = 0;
        let mut successful_operations = 0;
        let mut failed_operations = 0;
        let mut bytes_moved = 0;
        let mut merge_errors = Vec::new();

        // Merge all other locations into the target
        for (source_index, source_location) in common_folder.locations.iter().enumerate() {
            if source_index == target_index {
                continue; // Skip the target location
            }

            debug!(
                "Merging {} into {}",
                source_location.path.display(),
                target_location.path.display()
            );

            match self
                .merge_single_location(
                    &source_location.path,
                    &target_location.path,
                    file_operation_manager,
                )
                .await
            {
                Ok(merge_stats) => {
                    operations_performed += merge_stats.operations_performed;
                    successful_operations += merge_stats.successful_operations;
                    failed_operations += merge_stats.failed_operations;
                    bytes_moved += merge_stats.bytes_moved;
                }
                Err(e) => {
                    failed_operations += 1;
                    merge_errors.push(FolderMergeError {
                        source_path: source_location.path.clone(),
                        target_path: target_location.path.clone(),
                        error_message: e.to_string(),
                        error_category: e.category().to_string(),
                    });
                    warn!(
                        "Failed to merge {} into {}: {}",
                        source_location.path.display(),
                        target_location.path.display(),
                        e
                    );
                }
            }
        }

        Ok(SingleFolderMergeResult {
            folder_name: common_folder.folder_name.clone(),
            target_location: target_location.path.clone(),
            source_locations: common_folder
                .locations
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != target_index)
                .map(|(_, loc)| loc.path.clone())
                .collect(),
            operations_performed,
            successful_operations,
            failed_operations,
            bytes_moved,
            merge_errors,
        })
    }

    /// Merge contents of source location into target location
    fn merge_single_location<'a>(
        &'a self,
        source_path: &'a Path,
        target_path: &'a Path,
        file_operation_manager: &'a FileOperationManager,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = FileManagementResult<MergeOperationStats>> + Send + 'a,
        >,
    > {
        Box::pin(async move {
            let mut stats = MergeOperationStats {
                operations_performed: 0,
                successful_operations: 0,
                failed_operations: 0,
                bytes_moved: 0,
            };

            // Read source directory contents
            let entries = std::fs::read_dir(source_path).map_err(|e| {
                FileManagementError::io(
                    format!("Failed to read source directory {}", source_path.display()),
                    e,
                )
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    FileManagementError::io(
                        format!(
                            "Failed to read directory entry in {}",
                            source_path.display()
                        ),
                        e,
                    )
                })?;

                let entry_path = entry.path();
                let entry_name = entry.file_name();
                let target_entry_path = target_path.join(&entry_name);

                stats.operations_performed += 1;

                if entry_path.is_dir() {
                    // Handle directory merge
                    match self
                        .merge_directory(&entry_path, &target_entry_path, file_operation_manager)
                        .await
                    {
                        Ok(dir_stats) => {
                            stats.successful_operations += 1;
                            stats.bytes_moved += dir_stats.bytes_moved;
                            stats.operations_performed += dir_stats.operations_performed;
                            stats.successful_operations += dir_stats.successful_operations;
                            stats.failed_operations += dir_stats.failed_operations;
                        }
                        Err(e) => {
                            stats.failed_operations += 1;
                            warn!(
                                "Failed to merge directory {} to {}: {}",
                                entry_path.display(),
                                target_entry_path.display(),
                                e
                            );
                        }
                    }
                } else {
                    // Handle file merge
                    match self
                        .merge_file(&entry_path, &target_entry_path, file_operation_manager)
                        .await
                    {
                        Ok(bytes) => {
                            stats.successful_operations += 1;
                            stats.bytes_moved += bytes;
                        }
                        Err(e) => {
                            stats.failed_operations += 1;
                            warn!(
                                "Failed to merge file {} to {}: {}",
                                entry_path.display(),
                                target_entry_path.display(),
                                e
                            );
                        }
                    }
                }
            }

            // Remove source directory if all operations were successful and it's empty
            if stats.failed_operations == 0 && !self.config.dry_run {
                if let Err(e) = std::fs::remove_dir(source_path) {
                    warn!(
                        "Failed to remove source directory after merge {}: {}",
                        source_path.display(),
                        e
                    );
                } else {
                    debug!(
                        "Successfully removed source directory: {}",
                        source_path.display()
                    );
                }
            }

            Ok(stats)
        })
    }

    /// Merge a directory, handling conflicts based on configuration
    fn merge_directory<'a>(
        &'a self,
        source_dir: &'a Path,
        target_dir: &'a Path,
        file_operation_manager: &'a FileOperationManager,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = FileManagementResult<MergeOperationStats>> + Send + 'a,
        >,
    > {
        Box::pin(async move {
            if target_dir.exists() && target_dir.is_dir() {
                // Target directory exists - merge contents recursively
                self.merge_single_location(source_dir, target_dir, file_operation_manager)
                    .await
            } else {
                // Target directory doesn't exist - move the entire directory
                let result = file_operation_manager
                    .move_file(source_dir, target_dir)
                    .await?;
                Ok(MergeOperationStats {
                    operations_performed: 1,
                    successful_operations: 1,
                    failed_operations: 0,
                    bytes_moved: result.bytes_moved,
                })
            }
        })
    }

    /// Merge a file, handling conflicts based on configuration
    async fn merge_file(
        &self,
        source_file: &Path,
        target_file: &Path,
        file_operation_manager: &FileOperationManager,
    ) -> FileManagementResult<u64> {
        if target_file.exists() {
            // Handle file conflict based on duplicate handling strategy
            match self.config.duplicate_handling {
                DuplicateHandling::Skip => {
                    debug!("Skipping duplicate file: {}", target_file.display());
                    Ok(0)
                }
                DuplicateHandling::Rename => {
                    let unique_target = PathUtils::generate_unique_name(target_file);
                    let result = file_operation_manager
                        .move_file(source_file, &unique_target)
                        .await?;
                    Ok(result.bytes_moved)
                }
                DuplicateHandling::KeepNewer => {
                    if self.is_source_newer(source_file, target_file)? {
                        let result = file_operation_manager
                            .move_file(source_file, target_file)
                            .await?;
                        Ok(result.bytes_moved)
                    } else {
                        debug!("Target file is newer, skipping: {}", target_file.display());
                        Ok(0)
                    }
                }
                DuplicateHandling::KeepLarger => {
                    if self.is_source_larger(source_file, target_file)? {
                        let result = file_operation_manager
                            .move_file(source_file, target_file)
                            .await?;
                        Ok(result.bytes_moved)
                    } else {
                        debug!("Target file is larger, skipping: {}", target_file.display());
                        Ok(0)
                    }
                }
                DuplicateHandling::Merge => {
                    // For now, treat merge as rename - actual content merging would be file-type specific
                    let unique_target = PathUtils::generate_unique_name(target_file);
                    let result = file_operation_manager
                        .move_file(source_file, &unique_target)
                        .await?;
                    Ok(result.bytes_moved)
                }
            }
        } else {
            // No conflict - move file directly
            let result = file_operation_manager
                .move_file(source_file, target_file)
                .await?;
            Ok(result.bytes_moved)
        }
    }

    /// Check if source file is newer than target file
    fn is_source_newer(
        &self,
        source_file: &Path,
        target_file: &Path,
    ) -> FileManagementResult<bool> {
        let source_metadata = std::fs::metadata(source_file).map_err(|e| {
            FileManagementError::io(
                format!(
                    "Failed to get metadata for source file {}",
                    source_file.display()
                ),
                e,
            )
        })?;

        let target_metadata = std::fs::metadata(target_file).map_err(|e| {
            FileManagementError::io(
                format!(
                    "Failed to get metadata for target file {}",
                    target_file.display()
                ),
                e,
            )
        })?;

        let source_modified = source_metadata.modified().map_err(|e| {
            FileManagementError::io(
                format!(
                    "Failed to get modification time for source file {}",
                    source_file.display()
                ),
                e,
            )
        })?;

        let target_modified = target_metadata.modified().map_err(|e| {
            FileManagementError::io(
                format!(
                    "Failed to get modification time for target file {}",
                    target_file.display()
                ),
                e,
            )
        })?;

        Ok(source_modified > target_modified)
    }

    /// Check if source file is larger than target file
    fn is_source_larger(
        &self,
        source_file: &Path,
        target_file: &Path,
    ) -> FileManagementResult<bool> {
        let source_size = PathUtils::get_file_size(source_file)?;
        let target_size = PathUtils::get_file_size(target_file)?;
        Ok(source_size > target_size)
    }
}

/// Result of folder merge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMergeResult {
    pub merge_results: Vec<SingleFolderMergeResult>,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_bytes_moved: u64,
    pub duration_ms: u64,
    pub folders_merged: usize,
}

/// Result of merging a single folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleFolderMergeResult {
    pub folder_name: String,
    pub target_location: PathBuf,
    pub source_locations: Vec<PathBuf>,
    pub operations_performed: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub bytes_moved: u64,
    pub merge_errors: Vec<FolderMergeError>,
}

/// Error information for merge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMergeError {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub error_message: String,
    pub error_category: String,
}

/// Statistics for merge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOperationStats {
    pub operations_performed: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub bytes_moved: u64,
}

/// Context for human decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDecisionContext {
    pub decision_id: String,
    pub decision_type: HumanDecisionType,
    pub title: String,
    pub description: String,
    pub options: Vec<HumanDecisionOption>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timeout_seconds: Option<u64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl HumanDecisionContext {
    pub fn new<S: Into<String>>(
        decision_type: HumanDecisionType,
        title: S,
        description: S,
    ) -> Self {
        Self {
            decision_id: uuid::Uuid::new_v4().to_string(),
            decision_type,
            title: title.into(),
            description: description.into(),
            options: Vec::new(),
            metadata: HashMap::new(),
            timeout_seconds: None,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn add_option<S: Into<String>>(mut self, id: S, label: S, description: Option<S>) -> Self {
        self.options.push(HumanDecisionOption {
            id: id.into(),
            label: label.into(),
            description: description.map(|s| s.into()),
            recommended: false,
            metadata: HashMap::new(),
        });
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    pub fn with_metadata<K: Into<String>>(mut self, key: K, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Types of human decisions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HumanDecisionType {
    Classification,
    FileConflict,
    MergeStrategy,
    BatchConfirmation,
    Custom(String),
}

/// Option for human decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDecisionOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub recommended: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_operation_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileOperationManager::new(temp_dir.path().to_path_buf(), false);

        assert!(!manager.is_dry_run());
        assert!(manager.get_available_space(temp_dir.path()).is_ok());
    }

    #[test]
    fn test_text_processor() {
        let processor = TextProcessor::new(true);

        assert_eq!(processor.normalize_case("Hello World"), "hello world");
        assert_eq!(
            processor.normalize_whitespace("  hello   world  "),
            "hello world"
        );
        assert_eq!(processor.remove_spaces("hello world"), "helloworld");

        // Test Chinese detection
        assert!(processor.contains_chinese("你好世界"));
        assert!(!processor.contains_chinese("hello world"));
    }

    #[test]
    fn test_text_normalization_functions() {
        let config = TextNormalizationConfig {
            normalize_case: true,
            normalize_whitespace: true,
            remove_punctuation: true,
            normalize_unicode: true,
            filter_characters: Some(vec!['@', '#']),
            preserve_alphanumeric_only: false,
        };

        let processor = TextProcessor::with_config(true, config);

        // Test punctuation removal
        assert_eq!(processor.remove_punctuation("Hello, World!"), "Hello World");

        // Test character filtering
        assert_eq!(
            processor.filter_characters("hello@world#test"),
            "helloworldtest"
        );

        // Test alphanumeric preservation
        let alphanumeric_config = TextNormalizationConfig {
            preserve_alphanumeric_only: true,
            ..Default::default()
        };
        let alphanumeric_processor = TextProcessor::with_config(false, alphanumeric_config);
        assert_eq!(
            alphanumeric_processor.preserve_alphanumeric_only("Hello, World! 123"),
            "Hello World 123"
        );

        // Test comprehensive normalization
        let result = processor.normalize_text("  Hello@World!  ");
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_text_segmentation() {
        let processor = TextProcessor::new(true);

        // Test English text segmentation
        let segments = processor.segment_text("hello world test");
        assert_eq!(segments, vec!["hello", "world", "test"]);

        // Test mixed text segmentation
        let segments = processor.segment_text("hello 世界 test");
        assert_eq!(segments, vec!["hello", "世", "界", "test"]);
    }

    #[test]
    fn test_unicode_normalization() {
        let processor = TextProcessor::new(false);

        // Test diacritic folding
        assert_eq!(processor.normalize_unicode("café"), "cafe");
        assert_eq!(processor.normalize_unicode("naïve"), "naive");
        assert_eq!(processor.normalize_unicode("résumé"), "resume");
    }

    #[test]
    fn test_chinese_text_processing() {
        let processor = TextProcessor::new(true);

        // Test Chinese character detection
        assert!(processor.is_chinese_char('中'));
        assert!(processor.is_chinese_char('文'));
        assert!(!processor.is_chinese_char('a'));
        assert!(!processor.is_chinese_char('1'));

        // Test Chinese text type detection
        assert_eq!(
            processor.detect_chinese_type("hello world"),
            ChineseTextType::None
        );
        assert_eq!(
            processor.detect_chinese_type("简体中文"),
            ChineseTextType::Simplified
        );
        assert_eq!(
            processor.detect_chinese_type("繁體中文"),
            ChineseTextType::Traditional
        );

        // Test character extraction
        assert_eq!(processor.extract_chinese_chars("hello 中文 world"), "中文");
        assert_eq!(
            processor.extract_non_chinese_chars("hello 中文 world"),
            "hello  world"
        );

        // Test traditional to simplified conversion
        let converted = processor.convert_traditional("繁體中文");
        assert!(converted.contains("繁体")); // Should convert 繁體 to 繁体
    }

    #[test]
    fn test_mixed_text_processing() {
        let processor = TextProcessor::new(true);

        let result = processor.process_mixed_text("Hello 世界 World 中文");
        assert_eq!(result.chinese_chars, "世界中文");
        assert_eq!(result.non_chinese_chars, "Hello  World");
        assert!(result.has_mixed_content);
        assert_eq!(result.chinese_type, ChineseTextType::Simplified);
    }

    #[test]
    fn test_pinyin_conversion() {
        let processor = TextProcessor::new(true);

        // Test basic pinyin generation
        let variants = processor.generate_pinyin_variants("中文");
        assert!(!variants.is_empty());
        assert!(variants
            .iter()
            .any(|v| v.contains("zhong") || v.contains("wen")));

        // Test different pinyin styles
        let normal_pinyin = processor.generate_pinyin_with_style("你好", &PinyinStyle::Normal);
        let tone_pinyin = processor.generate_pinyin_with_style("你好", &PinyinStyle::WithTone);
        let no_tone_pinyin =
            processor.generate_pinyin_with_style("你好", &PinyinStyle::WithoutTone);
        let first_letter = processor.generate_pinyin_with_style("你好", &PinyinStyle::FirstLetter);

        assert!(!normal_pinyin.is_empty());
        assert!(!tone_pinyin.is_empty());
        assert!(!no_tone_pinyin.is_empty());
        assert!(!first_letter.is_empty());

        // Test comprehensive pinyin result
        let result = processor.generate_comprehensive_pinyin("中文", PinyinStyle::Normal);
        assert_eq!(result.original, "中文");
        assert_eq!(result.style, PinyinStyle::Normal);
        assert!(!result.pinyin_variants.is_empty());
        assert!(!result.combinations.is_empty());
    }

    #[test]
    fn test_pinyin_style_conversion() {
        let processor = TextProcessor::new(true);

        // Test tone mark conversion
        assert_eq!(processor.convert_numeric_tone_to_diacritic("ni3"), "nǐ");
        assert_eq!(processor.convert_numeric_tone_to_diacritic("hao3"), "hǎo");

        // Test tone mark removal
        assert_eq!(processor.remove_tone_marks("nǐ"), "ni");
        assert_eq!(processor.remove_tone_marks("hǎo"), "hao");
        assert_eq!(processor.remove_tone_marks("ni3"), "ni");
    }

    #[test]
    fn test_path_utils() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, b"hello world").unwrap();

        let size = PathUtils::get_file_size(&test_file).unwrap();
        assert_eq!(size, 11);

        let dir_size = PathUtils::get_directory_size(temp_dir.path()).unwrap();
        assert!(dir_size >= 11);
    }

    #[test]
    fn test_validation_utils() {
        assert!(ValidationUtils::validate_non_empty_string("hello", "test").is_ok());
        assert!(ValidationUtils::validate_non_empty_string("", "test").is_err());
        assert!(ValidationUtils::validate_non_empty_string("   ", "test").is_err());

        assert!(ValidationUtils::validate_range(5, 1, 10, "test").is_ok());
        assert!(ValidationUtils::validate_range(15, 1, 10, "test").is_err());
    }

    #[test]
    fn test_experimental_mode() {
        let mut exp_mode = ExperimentalMode::new(true);

        let operation = ExperimentalOperation::new("move", "Move file from A to B")
            .with_source_path("/path/a")
            .with_target_path("/path/b")
            .with_estimated_size(1024);

        exp_mode.log_operation(operation);
        assert_eq!(exp_mode.get_operations().len(), 1);

        exp_mode.clear_log();
        assert_eq!(exp_mode.get_operations().len(), 0);
    }

    #[test]
    fn test_human_decision_context() {
        let context = HumanDecisionContext::new(
            HumanDecisionType::Classification,
            "Test Decision",
            "Please choose an option",
        )
        .add_option("option1", "Option 1", Some("First option"))
        .add_option("option2", "Option 2", None)
        .with_timeout(300)
        .with_metadata(
            "test_key",
            serde_json::Value::String("test_value".to_string()),
        );

        assert_eq!(context.options.len(), 2);
        assert_eq!(context.timeout_seconds, Some(300));
        assert!(context.metadata.contains_key("test_key"));
    }
}
