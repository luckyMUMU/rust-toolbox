//! Error types for the File Management Plugin

use crate::error::WorkflowError;
use std::path::PathBuf;
use thiserror::Error;

/// Result type for file management operations
pub type FileManagementResult<T> = Result<T, FileManagementError>;

/// Errors specific to file management operations
#[derive(Error, Debug)]
pub enum FileManagementError {
    /// File or directory not found
    #[error("File or directory not found: {path}")]
    NotFound { path: PathBuf },

    /// Permission denied for file operation
    #[error("Permission denied for operation on: {path}")]
    PermissionDenied { path: PathBuf },

    /// Insufficient disk space
    #[error("Insufficient disk space: required {required} bytes, available {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },

    /// File operation conflict (e.g., destination already exists)
    #[error("File operation conflict: {message}")]
    Conflict { message: String },

    /// Invalid file path or name
    #[error("Invalid path: {path} - {reason}")]
    InvalidPath { path: PathBuf, reason: String },

    /// Text processing error
    #[error("Text processing error: {message}")]
    TextProcessing { message: String },

    /// Classification error
    #[error("Classification error: {message}")]
    Classification { message: String },

    /// Pattern matching error
    #[error("Pattern matching error: {message}")]
    PatternMatching { message: String },

    /// Human decision timeout
    #[error("Human decision timed out after {timeout_seconds} seconds")]
    HumanDecisionTimeout { timeout_seconds: u64 },

    /// Human decision cancelled
    #[error("Human decision was cancelled by user")]
    HumanDecisionCancelled,

    /// Experimental mode violation (trying to perform real operations in experimental mode)
    #[error("Operation not allowed in experimental mode: {operation}")]
    ExperimentalModeViolation { operation: String },

    /// Batch processing error
    #[error("Batch processing error: {failed_count} of {total_count} operations failed")]
    BatchProcessing { failed_count: usize, total_count: usize },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Validation error
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// I/O error wrapper
    #[error("I/O error: {message}")]
    Io {
        message: String,
        #[source]
        source: std::io::Error,
    },

    /// JSON processing error
    #[error("JSON processing error: {message}")]
    Json {
        message: String,
        #[source]
        source: serde_json::Error,
    },

    /// Generic error for other cases
    #[error("File management error: {message}")]
    Other { message: String },
}

impl FileManagementError {
    /// Create a not found error
    pub fn not_found<P: Into<PathBuf>>(path: P) -> Self {
        Self::NotFound { path: path.into() }
    }

    /// Create a permission denied error
    pub fn permission_denied<P: Into<PathBuf>>(path: P) -> Self {
        Self::PermissionDenied { path: path.into() }
    }

    /// Create an insufficient space error
    pub fn insufficient_space(required: u64, available: u64) -> Self {
        Self::InsufficientSpace { required, available }
    }

    /// Create a conflict error
    pub fn conflict<P: AsRef<std::path::Path>, S: Into<String>>(path: P, message: S) -> Self {
        Self::Conflict {
            message: format!("{}: {}", path.as_ref().display(), message.into()),
        }
    }

    /// Create a simple conflict error with just a message
    pub fn conflict_simple<S: Into<String>>(message: S) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Create an unsupported operation error
    pub fn unsupported_operation<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: format!("Unsupported operation: {}", message.into()),
        }
    }

    /// Create an invalid path error
    pub fn invalid_path<P: Into<PathBuf>, S: Into<String>>(path: P, reason: S) -> Self {
        Self::InvalidPath {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a text processing error
    pub fn text_processing<S: Into<String>>(message: S) -> Self {
        Self::TextProcessing {
            message: message.into(),
        }
    }

    /// Create a classification error
    pub fn classification<S: Into<String>>(message: S) -> Self {
        Self::Classification {
            message: message.into(),
        }
    }

    /// Create a pattern matching error
    pub fn pattern_matching<S: Into<String>>(message: S) -> Self {
        Self::PatternMatching {
            message: message.into(),
        }
    }

    /// Create a human decision timeout error
    pub fn human_decision_timeout(timeout_seconds: u64) -> Self {
        Self::HumanDecisionTimeout { timeout_seconds }
    }

    /// Create a human decision cancelled error
    pub fn human_decision_cancelled() -> Self {
        Self::HumanDecisionCancelled
    }

    /// Create an experimental mode violation error
    pub fn experimental_mode_violation<S: Into<String>>(operation: S) -> Self {
        Self::ExperimentalModeViolation {
            operation: operation.into(),
        }
    }

    /// Create a batch processing error
    pub fn batch_processing(failed_count: usize, total_count: usize) -> Self {
        Self::BatchProcessing {
            failed_count,
            total_count,
        }
    }

    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create an I/O error
    pub fn io<S: Into<String>>(message: S, source: std::io::Error) -> Self {
        Self::Io {
            message: message.into(),
            source,
        }
    }

    /// Create a JSON error
    pub fn json<S: Into<String>>(message: S, source: serde_json::Error) -> Self {
        Self::Json {
            message: message.into(),
            source,
        }
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: message.into(),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::NotFound { .. } => false,
            Self::PermissionDenied { .. } => false,
            Self::InsufficientSpace { .. } => false,
            Self::Conflict { .. } => true, // Can be resolved with different strategy
            Self::InvalidPath { .. } => false,
            Self::TextProcessing { .. } => true, // Might work with different input
            Self::Classification { .. } => true, // Might work with different rules
            Self::PatternMatching { .. } => true, // Might work with different patterns
            Self::HumanDecisionTimeout { .. } => true, // Can retry with longer timeout
            Self::HumanDecisionCancelled => false,
            Self::ExperimentalModeViolation { .. } => false,
            Self::BatchProcessing { .. } => true, // Can retry failed items
            Self::Configuration { .. } => false,
            Self::Validation { .. } => false,
            Self::Io { .. } => true, // Might be temporary
            Self::Json { .. } => false,
            Self::Other { .. } => true, // Unknown, assume recoverable
        }
    }

    /// Get the error category for metrics and logging
    pub fn category(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "not_found",
            Self::PermissionDenied { .. } => "permission_denied",
            Self::InsufficientSpace { .. } => "insufficient_space",
            Self::Conflict { .. } => "conflict",
            Self::InvalidPath { .. } => "invalid_path",
            Self::TextProcessing { .. } => "text_processing",
            Self::Classification { .. } => "classification",
            Self::PatternMatching { .. } => "pattern_matching",
            Self::HumanDecisionTimeout { .. } => "human_decision_timeout",
            Self::HumanDecisionCancelled => "human_decision_cancelled",
            Self::ExperimentalModeViolation { .. } => "experimental_mode_violation",
            Self::BatchProcessing { .. } => "batch_processing",
            Self::Configuration { .. } => "configuration",
            Self::Validation { .. } => "validation",
            Self::Io { .. } => "io",
            Self::Json { .. } => "json",
            Self::Other { .. } => "other",
        }
    }
}

/// Convert FileManagementError to WorkflowError
impl From<FileManagementError> for WorkflowError {
    fn from(err: FileManagementError) -> Self {
        match err {
            FileManagementError::NotFound { path } => WorkflowError::NotFound {
                resource: format!("file or directory: {}", path.display()),
            },
            FileManagementError::PermissionDenied { path } => WorkflowError::PermissionDenied {
                message: format!("Permission denied for: {}", path.display()),
            },
            FileManagementError::Validation { message } => WorkflowError::ValidationError(message),
            FileManagementError::Configuration { message } => WorkflowError::ValidationError(message),
            FileManagementError::Io { message, source } => WorkflowError::Io(source),
            other => WorkflowError::Plugin {
                message: other.to_string(),
            },
        }
    }
}

/// Convert std::io::Error to FileManagementError
impl From<std::io::Error> for FileManagementError {
    fn from(err: std::io::Error) -> Self {
        Self::io("I/O operation failed", err)
    }
}

/// Convert serde_json::Error to FileManagementError
impl From<serde_json::Error> for FileManagementError {
    fn from(err: serde_json::Error) -> Self {
        Self::json("JSON processing failed", err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_error_creation() {
        let path = Path::new("/test/path");
        
        let err = FileManagementError::not_found(path);
        assert!(matches!(err, FileManagementError::NotFound { .. }));
        
        let err = FileManagementError::permission_denied(path);
        assert!(matches!(err, FileManagementError::PermissionDenied { .. }));
        
        let err = FileManagementError::insufficient_space(1000, 500);
        assert!(matches!(err, FileManagementError::InsufficientSpace { .. }));
    }

    #[test]
    fn test_error_recoverability() {
        assert!(!FileManagementError::not_found("/test").is_recoverable());
        assert!(!FileManagementError::permission_denied("/test").is_recoverable());
        assert!(FileManagementError::conflict("/test/path", "test conflict").is_recoverable());
        assert!(FileManagementError::text_processing("test error").is_recoverable());
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(FileManagementError::not_found("/test").category(), "not_found");
        assert_eq!(FileManagementError::conflict("/test/path", "test").category(), "conflict");
        assert_eq!(FileManagementError::text_processing("test").category(), "text_processing");
    }

    #[test]
    fn test_workflow_error_conversion() {
        let fm_err = FileManagementError::not_found("/test/path");
        let workflow_err: WorkflowError = fm_err.into();
        
        match workflow_err {
            WorkflowError::NotFound { resource } => {
                assert!(resource.contains("/test/path"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}