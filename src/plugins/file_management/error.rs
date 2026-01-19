//! Error types for the File Management Plugin

use crate::error::WorkflowError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Result type for file management operations
pub type FileManagementResult<T> = Result<T, FileManagementError>;

/// Errors specific to file management operations
#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum FileManagementError {
    /// File or directory not found
    #[error("File or directory not found: {path}")]
    NotFound {
        path: PathBuf,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Permission denied for file operation
    #[error("Permission denied for operation on: {path}")]
    PermissionDenied {
        path: PathBuf,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Insufficient disk space
    #[error("Insufficient disk space: required {required} bytes, available {available} bytes")]
    InsufficientSpace {
        required: u64,
        available: u64,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// File operation conflict (e.g., destination already exists)
    #[error("File operation conflict: {message}")]
    Conflict {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Invalid file path or name
    #[error("Invalid path: {path} - {reason}")]
    InvalidPath {
        path: PathBuf,
        reason: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Text processing error
    #[error("Text processing error: {message}")]
    TextProcessing {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Classification error
    #[error("Classification error: {message}")]
    Classification {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Pattern matching error
    #[error("Pattern matching error: {message}")]
    PatternMatching {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Human decision timeout
    #[error("Human decision timed out after {timeout_seconds} seconds")]
    HumanDecisionTimeout {
        timeout_seconds: u64,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Human decision cancelled
    #[error("Human decision was cancelled by user")]
    HumanDecisionCancelled {
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Experimental mode violation (trying to perform real operations in experimental mode)
    #[error("Operation not allowed in experimental mode: {operation}")]
    ExperimentalModeViolation {
        operation: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Batch processing error
    #[error("Batch processing error: {failed_count} of {total_count} operations failed")]
    BatchProcessing {
        failed_count: usize,
        total_count: usize,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Validation error
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// I/O error wrapper
    #[error("I/O error: {message}")]
    Io {
        message: String,
        #[serde(skip)]
        source: std::io::Error,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// JSON processing error
    #[error("JSON processing error: {message}")]
    Json {
        message: String,
        #[serde(skip)]
        source: serde_json::Error,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Performance optimization error
    #[error("Performance optimization error: {message}")]
    Performance {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Resource exhaustion error
    #[error("Resource exhaustion: {resource} - {message}")]
    ResourceExhaustion {
        resource: String,
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Timeout error
    #[error("Operation timed out after {duration_seconds} seconds: {operation}")]
    Timeout {
        operation: String,
        duration_seconds: u64,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Concurrency error
    #[error("Concurrency error: {message}")]
    Concurrency {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Recovery error
    #[error("Recovery failed: {message}")]
    Recovery {
        message: String,
        original_error: Box<FileManagementError>,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },

    /// Generic error for other cases
    #[error("File management error: {message}")]
    Other {
        message: String,
        #[serde(skip)]
        context: Box<ErrorContext>,
    },
}

/// Error context for enhanced error reporting and recovery
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Operation that was being performed when error occurred
    pub operation: Option<String>,

    /// Component or tool that generated the error
    pub component: Option<String>,

    /// Additional metadata about the error
    pub metadata: HashMap<String, String>,

    /// Timestamp when error occurred
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Stack trace or call chain
    pub call_stack: Vec<String>,

    /// Recovery suggestions
    pub recovery_suggestions: Vec<RecoverySuggestion>,

    /// Error severity level
    pub severity: ErrorSeverity,

    /// Whether this error should be retried
    pub retryable: bool,

    /// User-friendly error message
    pub user_message: Option<String>,
}

/// Recovery suggestion for error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySuggestion {
    pub action: String,
    pub description: String,
    pub confidence: f64,             // 0.0 to 1.0
    pub estimated_success_rate: f64, // 0.0 to 1.0
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ErrorSeverity {
    /// Low severity - operation can continue
    Low,
    /// Medium severity - operation should be retried
    #[default]
    Medium,
    /// High severity - operation should be aborted but system can continue
    High,
    /// Critical severity - system should stop
    Critical,
}


impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self {
            operation: None,
            component: None,
            metadata: HashMap::new(),
            timestamp: Some(chrono::Utc::now()),
            call_stack: Vec::new(),
            recovery_suggestions: Vec::new(),
            severity: ErrorSeverity::Medium,
            retryable: true,
            user_message: None,
        }
    }

    /// Create error context with operation
    pub fn with_operation<S: Into<String>>(operation: S) -> Self {
        Self {
            operation: Some(operation.into()),
            component: None,
            metadata: HashMap::new(),
            timestamp: Some(chrono::Utc::now()),
            call_stack: Vec::new(),
            recovery_suggestions: Vec::new(),
            severity: ErrorSeverity::Medium,
            retryable: true,
            user_message: None,
        }
    }

    /// Create error context with component
    pub fn with_component<S: Into<String>>(component: S) -> Self {
        Self {
            operation: None,
            component: Some(component.into()),
            metadata: HashMap::new(),
            timestamp: Some(chrono::Utc::now()),
            call_stack: Vec::new(),
            recovery_suggestions: Vec::new(),
            severity: ErrorSeverity::Medium,
            retryable: true,
            user_message: None,
        }
    }

    /// Add metadata entry
    pub fn add_metadata<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Add call stack entry
    pub fn add_call_stack_entry<S: Into<String>>(&mut self, entry: S) {
        self.call_stack.push(entry.into());
    }

    /// Add recovery suggestion
    pub fn add_recovery_suggestion(&mut self, suggestion: RecoverySuggestion) {
        self.recovery_suggestions.push(suggestion);
    }

    /// Set severity
    pub fn set_severity(&mut self, severity: ErrorSeverity) {
        self.severity = severity;
    }

    /// Set retryable flag
    pub fn set_retryable(&mut self, retryable: bool) {
        self.retryable = retryable;
    }

    /// Set user message
    pub fn set_user_message<S: Into<String>>(&mut self, message: S) {
        self.user_message = Some(message.into());
    }

    /// Get formatted call stack
    pub fn formatted_call_stack(&self) -> String {
        if self.call_stack.is_empty() {
            "No call stack available".to_string()
        } else {
            self.call_stack.join(" -> ")
        }
    }

    /// Get context summary for logging
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref op) = self.operation {
            parts.push(format!("operation: {}", op));
        }

        if let Some(ref comp) = self.component {
            parts.push(format!("component: {}", comp));
        }

        parts.push(format!("severity: {:?}", self.severity));
        parts.push(format!("retryable: {}", self.retryable));

        if !self.recovery_suggestions.is_empty() {
            parts.push(format!("suggestions: {}", self.recovery_suggestions.len()));
        }

        parts.join(", ")
    }
}

impl RecoverySuggestion {
    /// Create a new recovery suggestion
    pub fn new<S: Into<String>>(action: S, description: S) -> Self {
        Self {
            action: action.into(),
            description: description.into(),
            confidence: 0.5,
            estimated_success_rate: 0.5,
        }
    }

    /// Create a recovery suggestion with confidence
    pub fn with_confidence<S: Into<String>>(action: S, description: S, confidence: f64) -> Self {
        Self {
            action: action.into(),
            description: description.into(),
            confidence: confidence.clamp(0.0, 1.0),
            estimated_success_rate: 0.5,
        }
    }

    /// Create a recovery suggestion with success rate
    pub fn with_success_rate<S: Into<String>>(
        action: S,
        description: S,
        success_rate: f64,
    ) -> Self {
        Self {
            action: action.into(),
            description: description.into(),
            confidence: 0.5,
            estimated_success_rate: success_rate.clamp(0.0, 1.0),
        }
    }

    /// Create a fully specified recovery suggestion
    pub fn new_full<S: Into<String>>(
        action: S,
        description: S,
        confidence: f64,
        success_rate: f64,
    ) -> Self {
        Self {
            action: action.into(),
            description: description.into(),
            confidence: confidence.clamp(0.0, 1.0),
            estimated_success_rate: success_rate.clamp(0.0, 1.0),
        }
    }
}

impl FileManagementError {
    /// Create a not found error with context
    pub fn not_found<P: Into<PathBuf>>(path: P) -> Self {
        Self::NotFound {
            path: path.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a not found error with enhanced context
    pub fn not_found_with_context<P: Into<PathBuf>>(path: P, context: ErrorContext) -> Self {
        Self::NotFound {
            path: path.into(),
            context: Box::new(context),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied<P: Into<PathBuf>>(path: P) -> Self {
        Self::PermissionDenied {
            path: path.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a permission denied error with context
    pub fn permission_denied_with_context<P: Into<PathBuf>>(
        path: P,
        context: ErrorContext,
    ) -> Self {
        Self::PermissionDenied {
            path: path.into(),
            context: Box::new(context),
        }
    }

    /// Create an insufficient space error
    pub fn insufficient_space(required: u64, available: u64) -> Self {
        Self::InsufficientSpace {
            required,
            available,
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create an insufficient space error with context
    pub fn insufficient_space_with_context(
        required: u64,
        available: u64,
        context: ErrorContext,
    ) -> Self {
        Self::InsufficientSpace {
            required,
            available,
            context: Box::new(context),
        }
    }

    /// Create a conflict error
    pub fn conflict<P: AsRef<std::path::Path>, S: Into<String>>(path: P, message: S) -> Self {
        Self::Conflict {
            message: format!("{}: {}", path.as_ref().display(), message.into()),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a simple conflict error with just a message
    pub fn conflict_simple<S: Into<String>>(message: S) -> Self {
        Self::Conflict {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a conflict error with context
    pub fn conflict_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Conflict {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create an unsupported operation error
    pub fn unsupported_operation<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: format!("Unsupported operation: {}", message.into()),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create an invalid path error
    pub fn invalid_path<P: Into<PathBuf>, S: Into<String>>(path: P, reason: S) -> Self {
        Self::InvalidPath {
            path: path.into(),
            reason: reason.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create an invalid path error with context
    pub fn invalid_path_with_context<P: Into<PathBuf>, S: Into<String>>(
        path: P,
        reason: S,
        context: ErrorContext,
    ) -> Self {
        Self::InvalidPath {
            path: path.into(),
            reason: reason.into(),
            context: Box::new(context),
        }
    }

    /// Create a text processing error
    pub fn text_processing<S: Into<String>>(message: S) -> Self {
        Self::TextProcessing {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a text processing error with context
    pub fn text_processing_with_context<S: Into<String>>(
        message: S,
        context: ErrorContext,
    ) -> Self {
        Self::TextProcessing {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create a classification error
    pub fn classification<S: Into<String>>(message: S) -> Self {
        Self::Classification {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a classification error with context
    pub fn classification_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Classification {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create a pattern matching error
    pub fn pattern_matching<S: Into<String>>(message: S) -> Self {
        Self::PatternMatching {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a pattern matching error with context
    pub fn pattern_matching_with_context<S: Into<String>>(
        message: S,
        context: ErrorContext,
    ) -> Self {
        Self::PatternMatching {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create a human decision timeout error
    pub fn human_decision_timeout(timeout_seconds: u64) -> Self {
        Self::HumanDecisionTimeout {
            timeout_seconds,
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a human decision timeout error with context
    pub fn human_decision_timeout_with_context(
        timeout_seconds: u64,
        context: ErrorContext,
    ) -> Self {
        Self::HumanDecisionTimeout {
            timeout_seconds,
            context: Box::new(context),
        }
    }

    /// Create a human decision cancelled error
    pub fn human_decision_cancelled() -> Self {
        Self::HumanDecisionCancelled {
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a human decision cancelled error with context
    pub fn human_decision_cancelled_with_context(context: ErrorContext) -> Self {
        Self::HumanDecisionCancelled {
            context: Box::new(context),
        }
    }

    /// Create an experimental mode violation error
    pub fn experimental_mode_violation<S: Into<String>>(operation: S) -> Self {
        Self::ExperimentalModeViolation {
            operation: operation.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create an experimental mode violation error with context
    pub fn experimental_mode_violation_with_context<S: Into<String>>(
        operation: S,
        context: ErrorContext,
    ) -> Self {
        Self::ExperimentalModeViolation {
            operation: operation.into(),
            context: Box::new(context),
        }
    }

    /// Create a batch processing error
    pub fn batch_processing(failed_count: usize, total_count: usize) -> Self {
        Self::BatchProcessing {
            failed_count,
            total_count,
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a batch processing error with context
    pub fn batch_processing_with_context(
        failed_count: usize,
        total_count: usize,
        context: ErrorContext,
    ) -> Self {
        Self::BatchProcessing {
            failed_count,
            total_count,
            context: Box::new(context),
        }
    }

    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a configuration error with context
    pub fn configuration_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Configuration {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a validation error with context
    pub fn validation_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Validation {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create an I/O error
    pub fn io<S: Into<String>>(message: S, source: std::io::Error) -> Self {
        Self::Io {
            message: message.into(),
            source,
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create an I/O error with context
    pub fn io_with_context<S: Into<String>>(
        message: S,
        source: std::io::Error,
        context: ErrorContext,
    ) -> Self {
        Self::Io {
            message: message.into(),
            source,
            context: Box::new(context),
        }
    }

    /// Create a JSON error
    pub fn json<S: Into<String>>(message: S, source: serde_json::Error) -> Self {
        Self::Json {
            message: message.into(),
            source,
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a JSON error with context
    pub fn json_with_context<S: Into<String>>(
        message: S,
        source: serde_json::Error,
        context: ErrorContext,
    ) -> Self {
        Self::Json {
            message: message.into(),
            source,
            context: Box::new(context),
        }
    }

    /// Create a performance error
    pub fn performance<S: Into<String>>(message: S) -> Self {
        Self::Performance {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a performance error with context
    pub fn performance_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Performance {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create a resource exhaustion error
    pub fn resource_exhaustion<S: Into<String>>(resource: S, message: S) -> Self {
        Self::ResourceExhaustion {
            resource: resource.into(),
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a resource exhaustion error with context
    pub fn resource_exhaustion_with_context<S: Into<String>>(
        resource: S,
        message: S,
        context: ErrorContext,
    ) -> Self {
        Self::ResourceExhaustion {
            resource: resource.into(),
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S, duration_seconds: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration_seconds,
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a timeout error with context
    pub fn timeout_with_context<S: Into<String>>(
        operation: S,
        duration_seconds: u64,
        context: ErrorContext,
    ) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration_seconds,
            context: Box::new(context),
        }
    }

    /// Create a concurrency error
    pub fn concurrency<S: Into<String>>(message: S) -> Self {
        Self::Concurrency {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a concurrency error with context
    pub fn concurrency_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Concurrency {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Create a recovery error
    pub fn recovery<S: Into<String>>(message: S, original_error: FileManagementError) -> Self {
        Self::Recovery {
            message: message.into(),
            original_error: Box::new(original_error),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a recovery error with context
    pub fn recovery_with_context<S: Into<String>>(
        message: S,
        original_error: FileManagementError,
        context: ErrorContext,
    ) -> Self {
        Self::Recovery {
            message: message.into(),
            original_error: Box::new(original_error),
            context: Box::new(context),
        }
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: message.into(),
            context: Box::new(ErrorContext::default()),
        }
    }

    /// Create a generic error with context
    pub fn other_with_context<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        Self::Other {
            message: message.into(),
            context: Box::new(context),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::NotFound { context, .. } => context.retryable,
            Self::PermissionDenied { context, .. } => context.retryable,
            Self::InsufficientSpace { context, .. } => context.retryable,
            Self::Conflict { context: _, .. } => true, // Can be resolved with different strategy
            Self::InvalidPath { context, .. } => context.retryable,
            Self::TextProcessing { context: _, .. } => true, // Might work with different input
            Self::Classification { context: _, .. } => true, // Might work with different rules
            Self::PatternMatching { context: _, .. } => true, // Might work with different patterns
            Self::HumanDecisionTimeout { context: _, .. } => true, // Can retry with longer timeout
            Self::HumanDecisionCancelled { context, .. } => context.retryable,
            Self::ExperimentalModeViolation { context: _, .. } => false,
            Self::BatchProcessing { context: _, .. } => true, // Can retry failed items
            Self::Configuration { context, .. } => context.retryable,
            Self::Validation { context, .. } => context.retryable,
            Self::Io { context: _, .. } => true, // Might be temporary
            Self::Json { context, .. } => context.retryable,
            Self::Performance { context: _, .. } => true, // Can retry with different settings
            Self::ResourceExhaustion { context: _, .. } => true, // Can retry after cleanup
            Self::Timeout { context: _, .. } => true,     // Can retry with longer timeout
            Self::Concurrency { context: _, .. } => true, // Can retry with different concurrency
            Self::Recovery { context, .. } => context.retryable,
            Self::Other { context, .. } => context.retryable,
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
            Self::HumanDecisionCancelled { .. } => "human_decision_cancelled",
            Self::ExperimentalModeViolation { .. } => "experimental_mode_violation",
            Self::BatchProcessing { .. } => "batch_processing",
            Self::Configuration { .. } => "configuration",
            Self::Validation { .. } => "validation",
            Self::Io { .. } => "io",
            Self::Json { .. } => "json",
            Self::Performance { .. } => "performance",
            Self::ResourceExhaustion { .. } => "resource_exhaustion",
            Self::Timeout { .. } => "timeout",
            Self::Concurrency { .. } => "concurrency",
            Self::Recovery { .. } => "recovery",
            Self::Other { .. } => "other",
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotFound { context, .. } => context.severity,
            Self::PermissionDenied { context, .. } => context.severity,
            Self::InsufficientSpace { context, .. } => context.severity,
            Self::Conflict { context, .. } => context.severity,
            Self::InvalidPath { context, .. } => context.severity,
            Self::TextProcessing { context, .. } => context.severity,
            Self::Classification { context, .. } => context.severity,
            Self::PatternMatching { context, .. } => context.severity,
            Self::HumanDecisionTimeout { context, .. } => context.severity,
            Self::HumanDecisionCancelled { context, .. } => context.severity,
            Self::ExperimentalModeViolation { context: _, .. } => ErrorSeverity::Low,
            Self::BatchProcessing { context, .. } => context.severity,
            Self::Configuration { context: _, .. } => ErrorSeverity::High,
            Self::Validation { context: _, .. } => ErrorSeverity::Medium,
            Self::Io { context, .. } => context.severity,
            Self::Json { context, .. } => context.severity,
            Self::Performance { context: _, .. } => ErrorSeverity::Medium,
            Self::ResourceExhaustion { context: _, .. } => ErrorSeverity::High,
            Self::Timeout { context: _, .. } => ErrorSeverity::Medium,
            Self::Concurrency { context: _, .. } => ErrorSeverity::Medium,
            Self::Recovery { context: _, .. } => ErrorSeverity::High,
            Self::Other { context, .. } => context.severity,
        }
    }

    /// Get error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            Self::NotFound { context, .. } => context,
            Self::PermissionDenied { context, .. } => context,
            Self::InsufficientSpace { context, .. } => context,
            Self::Conflict { context, .. } => context,
            Self::InvalidPath { context, .. } => context,
            Self::TextProcessing { context, .. } => context,
            Self::Classification { context, .. } => context,
            Self::PatternMatching { context, .. } => context,
            Self::HumanDecisionTimeout { context, .. } => context,
            Self::HumanDecisionCancelled { context, .. } => context,
            Self::ExperimentalModeViolation { context, .. } => context,
            Self::BatchProcessing { context, .. } => context,
            Self::Configuration { context, .. } => context,
            Self::Validation { context, .. } => context,
            Self::Io { context, .. } => context,
            Self::Json { context, .. } => context,
            Self::Performance { context, .. } => context,
            Self::ResourceExhaustion { context, .. } => context,
            Self::Timeout { context, .. } => context,
            Self::Concurrency { context, .. } => context,
            Self::Recovery { context, .. } => context,
            Self::Other { context, .. } => context,
        }
    }

    /// Get mutable error context
    pub fn context_mut(&mut self) -> &mut ErrorContext {
        match self {
            Self::NotFound { context, .. } => context,
            Self::PermissionDenied { context, .. } => context,
            Self::InsufficientSpace { context, .. } => context,
            Self::Conflict { context, .. } => context,
            Self::InvalidPath { context, .. } => context,
            Self::TextProcessing { context, .. } => context,
            Self::Classification { context, .. } => context,
            Self::PatternMatching { context, .. } => context,
            Self::HumanDecisionTimeout { context, .. } => context,
            Self::HumanDecisionCancelled { context, .. } => context,
            Self::ExperimentalModeViolation { context, .. } => context,
            Self::BatchProcessing { context, .. } => context,
            Self::Configuration { context, .. } => context,
            Self::Validation { context, .. } => context,
            Self::Io { context, .. } => context,
            Self::Json { context, .. } => context,
            Self::Performance { context, .. } => context,
            Self::ResourceExhaustion { context, .. } => context,
            Self::Timeout { context, .. } => context,
            Self::Concurrency { context, .. } => context,
            Self::Recovery { context, .. } => context,
            Self::Other { context, .. } => context,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        *self.context_mut() = context;
        self
    }

    /// Add operation context
    pub fn with_operation<S: Into<String>>(mut self, operation: S) -> Self {
        self.context_mut().operation = Some(operation.into());
        self
    }

    /// Add component context
    pub fn with_component<S: Into<String>>(mut self, component: S) -> Self {
        self.context_mut().component = Some(component.into());
        self
    }

    /// Add metadata
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.context_mut().metadata.insert(key.into(), value.into());
        self
    }

    /// Add recovery suggestion
    pub fn with_recovery_suggestion(mut self, suggestion: RecoverySuggestion) -> Self {
        self.context_mut().recovery_suggestions.push(suggestion);
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.context_mut().severity = severity;
        self
    }

    /// Set retryable flag
    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.context_mut().retryable = retryable;
        self
    }

    /// Set user-friendly message
    pub fn with_user_message<S: Into<String>>(mut self, message: S) -> Self {
        self.context_mut().user_message = Some(message.into());
        self
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        if let Some(ref msg) = self.context().user_message {
            msg.clone()
        } else {
            // Generate user-friendly message based on error type
            match self {
                Self::NotFound { path, .. } => {
                    format!(
                        "The file or folder '{}' could not be found.",
                        path.display()
                    )
                }
                Self::PermissionDenied { path, .. } => {
                    format!("Permission denied when accessing '{}'.", path.display())
                }
                Self::InsufficientSpace {
                    required,
                    available,
                    ..
                } => {
                    format!(
                        "Not enough disk space. Need {} MB but only {} MB available.",
                        required / (1024 * 1024),
                        available / (1024 * 1024)
                    )
                }
                Self::Conflict { message, .. } => {
                    format!("File operation conflict: {}", message)
                }
                Self::HumanDecisionTimeout {
                    timeout_seconds, ..
                } => {
                    format!(
                        "Decision timed out after {} seconds. Please try again.",
                        timeout_seconds
                    )
                }
                Self::HumanDecisionCancelled { .. } => {
                    "Operation was cancelled by user.".to_string()
                }
                Self::BatchProcessing {
                    failed_count,
                    total_count,
                    ..
                } => {
                    format!("{} out of {} operations failed.", failed_count, total_count)
                }
                _ => self.to_string(),
            }
        }
    }

    /// Generate recovery suggestions based on error type
    pub fn generate_recovery_suggestions(&self) -> Vec<RecoverySuggestion> {
        let mut suggestions = self.context().recovery_suggestions.clone();

        // Add default suggestions based on error type
        match self {
            Self::NotFound { path, .. } => {
                suggestions.push(RecoverySuggestion {
                    action: "check_path".to_string(),
                    description: format!(
                        "Verify that the path '{}' exists and is accessible.",
                        path.display()
                    ),
                    confidence: 0.8,
                    estimated_success_rate: 0.7,
                });
                suggestions.push(RecoverySuggestion {
                    action: "create_path".to_string(),
                    description: "Create the missing directory structure.".to_string(),
                    confidence: 0.6,
                    estimated_success_rate: 0.8,
                });
            }
            Self::PermissionDenied { path, .. } => {
                suggestions.push(RecoverySuggestion {
                    action: "check_permissions".to_string(),
                    description: format!("Check and adjust permissions for '{}'.", path.display()),
                    confidence: 0.9,
                    estimated_success_rate: 0.8,
                });
                suggestions.push(RecoverySuggestion {
                    action: "run_as_admin".to_string(),
                    description: "Try running the operation with administrator privileges."
                        .to_string(),
                    confidence: 0.7,
                    estimated_success_rate: 0.9,
                });
            }
            Self::InsufficientSpace {
                required,
                available,
                ..
            } => {
                suggestions.push(RecoverySuggestion {
                    action: "free_space".to_string(),
                    description: format!(
                        "Free up at least {} MB of disk space.",
                        (required - available) / (1024 * 1024)
                    ),
                    confidence: 0.9,
                    estimated_success_rate: 0.9,
                });
                suggestions.push(RecoverySuggestion {
                    action: "use_different_location".to_string(),
                    description: "Choose a different destination with more available space."
                        .to_string(),
                    confidence: 0.8,
                    estimated_success_rate: 0.9,
                });
            }
            Self::Timeout { .. } => {
                suggestions.push(RecoverySuggestion {
                    action: "increase_timeout".to_string(),
                    description: "Increase the timeout duration and retry the operation."
                        .to_string(),
                    confidence: 0.8,
                    estimated_success_rate: 0.7,
                });
                suggestions.push(RecoverySuggestion {
                    action: "reduce_batch_size".to_string(),
                    description: "Process fewer items at once to reduce operation time."
                        .to_string(),
                    confidence: 0.7,
                    estimated_success_rate: 0.8,
                });
            }
            Self::ResourceExhaustion { resource, .. } => {
                suggestions.push(RecoverySuggestion {
                    action: "cleanup_resources".to_string(),
                    description: format!("Clean up {} resources and retry.", resource),
                    confidence: 0.8,
                    estimated_success_rate: 0.8,
                });
                suggestions.push(RecoverySuggestion {
                    action: "reduce_concurrency".to_string(),
                    description: "Reduce the number of concurrent operations.".to_string(),
                    confidence: 0.7,
                    estimated_success_rate: 0.9,
                });
            }
            _ => {
                suggestions.push(RecoverySuggestion {
                    action: "retry".to_string(),
                    description: "Retry the operation after a short delay.".to_string(),
                    confidence: 0.5,
                    estimated_success_rate: 0.6,
                });
            }
        }

        suggestions
    }
}

/// Convert FileManagementError to WorkflowError
impl From<FileManagementError> for WorkflowError {
    fn from(err: FileManagementError) -> Self {
        match err {
            FileManagementError::NotFound { path, .. } => WorkflowError::NotFound {
                resource: format!("file or directory: {}", path.display()),
            },
            FileManagementError::PermissionDenied { path, .. } => WorkflowError::PermissionDenied {
                message: format!("Permission denied for: {}", path.display()),
            },
            FileManagementError::Validation { message, .. } => {
                WorkflowError::ValidationError(message)
            }
            FileManagementError::Configuration { message, .. } => {
                WorkflowError::ValidationError(message)
            }
            FileManagementError::Io { source, .. } => WorkflowError::Io(source),
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
        assert_eq!(
            FileManagementError::not_found("/test").category(),
            "not_found"
        );
        assert_eq!(
            FileManagementError::conflict("/test/path", "test").category(),
            "conflict"
        );
        assert_eq!(
            FileManagementError::text_processing("test").category(),
            "text_processing"
        );
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
