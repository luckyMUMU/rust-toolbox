//! Common error types shared across the workflow toolkit.
//!
//! This module provides unified error variants that can be used by all modules,
//! reducing code duplication and ensuring consistent error handling patterns.

use std::time::Duration;
use thiserror::Error;

/// Common error variants used across all modules.
#[derive(Error, Debug, Clone)]
pub enum CommonError {
    /// IO operation failed
    #[error("IO error: {message}")]
    Io {
        /// Error message
        message: String,
        /// Path involved (if any)
        path: Option<String>,
    },

    /// Validation failed
    #[error("Validation error: {message}")]
    Validation {
        /// What was being validated
        context: String,
        /// Error message
        message: String,
    },

    /// Resource not found
    #[error("Not found: {resource}")]
    NotFound {
        /// Type of resource
        resource_type: String,
        /// Resource identifier
        resource: String,
    },

    /// Operation timed out
    #[error("Timeout after {duration:?}")]
    Timeout {
        /// Operation that timed out
        operation: String,
        /// Timeout duration
        duration: Duration,
    },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} (limit: {limit})")]
    ResourceLimitExceeded {
        /// Resource type
        resource: String,
        /// Limit that was exceeded
        limit: u64,
        /// Actual usage
        actual: u64,
    },

    /// Concurrent access conflict
    #[error("Concurrent access error: {message}")]
    ConcurrentAccess {
        /// Error message
        message: String,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration {
        /// Configuration key (if applicable)
        key: Option<String>,
        /// Error message
        message: String,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    Serialization {
        /// Format (json, yaml, etc.)
        format: String,
        /// Error message
        message: String,
    },
}

impl CommonError {
    /// Create a new IO error
    pub fn io<S: Into<String>>(message: S) -> Self {
        Self::Io {
            message: message.into(),
            path: None,
        }
    }

    /// Create a new IO error with path
    pub fn io_with_path<S: Into<String>, P: Into<String>>(message: S, path: P) -> Self {
        Self::Io {
            message: message.into(),
            path: Some(path.into()),
        }
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(context: S, message: S) -> Self {
        Self::Validation {
            context: context.into(),
            message: message.into(),
        }
    }

    /// Create a new not found error
    pub fn not_found<S: Into<String>>(resource_type: S, resource: S) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            resource: resource.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(operation: S, duration: Duration) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration,
        }
    }

    /// Create a new resource limit exceeded error
    pub fn resource_limit<S: Into<String>>(resource: S, limit: u64, actual: u64) -> Self {
        Self::ResourceLimitExceeded {
            resource: resource.into(),
            limit,
            actual,
        }
    }

    /// Create a new concurrent access error
    pub fn concurrent_access<S: Into<String>>(message: S) -> Self {
        Self::ConcurrentAccess {
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            key: None,
            message: message.into(),
        }
    }

    /// Create a new configuration error with key
    pub fn config_with_key<S: Into<String>>(key: S, message: S) -> Self {
        Self::Configuration {
            key: Some(key.into()),
            message: message.into(),
        }
    }

    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(format: S, message: S) -> Self {
        Self::Serialization {
            format: format.into(),
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for CommonError {
    fn from(err: std::io::Error) -> Self {
        Self::io(err.to_string())
    }
}

impl From<serde_json::Error> for CommonError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization("json", err.to_string())
    }
}

impl From<serde_yaml::Error> for CommonError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::serialization("yaml", err.to_string())
    }
}

impl From<config::ConfigError> for CommonError {
    fn from(err: config::ConfigError) -> Self {
        Self::config(err.to_string())
    }
}
