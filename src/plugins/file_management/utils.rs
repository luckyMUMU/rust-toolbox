//! Utility functions and shared components for file management operations

use crate::plugins::file_management::error::{FileManagementError, FileManagementResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// File operation manager for safe file operations
pub struct FileOperationManager {
    dry_run: bool,
    temp_directory: PathBuf,
}

impl FileOperationManager {
    /// Create a new file operation manager
    pub fn new(temp_directory: PathBuf, dry_run: bool) -> Self {
        Self {
            dry_run,
            temp_directory,
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

    /// Get available disk space for a path
    pub fn get_available_space<P: AsRef<Path>>(&self, path: P) -> FileManagementResult<u64> {
        // This is a simplified implementation
        // In a real implementation, you would use platform-specific APIs
        // For now, we'll return a large number to avoid blocking operations
        Ok(u64::MAX)
    }

    /// Check if a file operation would have sufficient space
    pub fn check_space_requirements<P: AsRef<Path>>(
        &self,
        target_path: P,
        required_bytes: u64,
    ) -> FileManagementResult<()> {
        let available = self.get_available_space(&target_path)?;
        if available < required_bytes {
            return Err(FileManagementError::insufficient_space(required_bytes, available));
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
            return Err(FileManagementError::invalid_path(
                path,
                "Path is too long",
            ));
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
}

/// Text processor for normalization and Chinese text handling
pub struct TextProcessor {
    enable_chinese: bool,
}

impl TextProcessor {
    /// Create a new text processor
    pub fn new(enable_chinese: bool) -> Self {
        Self { enable_chinese }
    }

    /// Normalize text case
    pub fn normalize_case(&self, text: &str) -> String {
        text.to_lowercase()
    }

    /// Remove extra spaces and normalize whitespace
    pub fn normalize_whitespace(&self, text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Remove spaces entirely
    pub fn remove_spaces(&self, text: &str) -> String {
        text.replace(' ', "")
    }

    /// Check if text contains Chinese characters
    pub fn contains_chinese(&self, text: &str) -> bool {
        text.chars().any(|c| {
            let code = c as u32;
            // Basic Chinese character ranges
            (0x4E00..=0x9FFF).contains(&code) || // CJK Unified Ideographs
            (0x3400..=0x4DBF).contains(&code) || // CJK Extension A
            (0x20000..=0x2A6DF).contains(&code) || // CJK Extension B
            (0x2A700..=0x2B73F).contains(&code) || // CJK Extension C
            (0x2B740..=0x2B81F).contains(&code) || // CJK Extension D
            (0x2B820..=0x2CEAF).contains(&code) // CJK Extension E
        })
    }

    /// Convert traditional Chinese to simplified (placeholder implementation)
    pub fn convert_traditional(&self, text: &str) -> String {
        if !self.enable_chinese {
            return text.to_string();
        }
        
        // This is a placeholder implementation
        // In a real implementation, you would use a proper Chinese conversion library
        debug!("Converting traditional Chinese text (placeholder implementation)");
        text.to_string()
    }

    /// Generate pinyin variants (placeholder implementation)
    pub fn generate_pinyin_variants(&self, text: &str) -> Vec<String> {
        if !self.enable_chinese || !self.contains_chinese(text) {
            return vec![text.to_string()];
        }

        // This is a placeholder implementation
        // In a real implementation, you would use a proper pinyin conversion library
        debug!("Generating pinyin variants (placeholder implementation)");
        vec![text.to_string()]
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
            FileManagementError::io(
                format!("Failed to read directory {}", path.display()),
                e,
            )
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
        let new_name = if let Some(ext) = extension {
            format!("{}_{}_{}.{}", stem, timestamp, rand::random::<u32>(), ext)
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
        let test_file = path.join(format!(".test_write_{}", rand::random::<u32>()));
        match std::fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) => Err(FileManagementError::permission_denied(path)),
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
    pub fn new<S: Into<String>>(
        operation_type: S,
        description: S,
    ) -> Self {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        assert_eq!(processor.normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(processor.remove_spaces("hello world"), "helloworld");
        
        // Test Chinese detection
        assert!(processor.contains_chinese("你好世界"));
        assert!(!processor.contains_chinese("hello world"));
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
        .with_metadata("test_key", serde_json::Value::String("test_value".to_string()));
        
        assert_eq!(context.options.len(), 2);
        assert_eq!(context.timeout_seconds, Some(300));
        assert!(context.metadata.contains_key("test_key"));
    }
}