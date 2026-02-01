//! Core module for file management plugin
//!
//! Contains error types, error recovery, and shared types.

pub mod error;
pub mod error_recovery;
pub mod types;

// Re-export core components
pub use error::{ErrorContext, ErrorSeverity, FileManagementError, FileManagementResult, RecoverySuggestion};
pub use error_recovery::{ErrorRecoveryManager, RecoveryAttempt, RecoveryConfig, RecoverySession, RecoveryStats, RecoveryStrategy};
