//! # Workflow Toolkit
//! 
//! A multi-interface workflow execution system built with Rust.
//! Supports CLI, TUI, and MCP server interfaces for workflow management.

pub mod config;
pub mod core;
pub mod error;
pub mod storage;
pub mod workflow;
pub mod tools;
pub mod plugins;
pub mod interfaces;

// Re-export commonly used types
pub use crate::config::{Config, ConfigManager, CliConfigOverrides};
pub use crate::core::*;
pub use crate::error::{Result, WorkflowError};
pub use crate::workflow::{WorkflowDefinition, WorkflowEngine, WorkflowExecution};
pub use crate::tools::{ToolNode, ToolRegistry};

/// Initialize the logging system
pub fn init_logging() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "workflow_toolkit=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        let config = Config::default();
        assert_eq!(config.server.http_port, 8080);
        assert_eq!(config.server.ws_port, 8081);
    }
    
    #[test]
    fn test_execution_context() {
        let mut context = ExecutionContext::new();
        context.set_variable("test_key", serde_json::Value::String("test_value".to_string()));
        
        assert!(context.get_variable("test_key").is_some());
        assert!(context.get_variable("nonexistent").is_none());
    }
    
    #[test]
    fn test_execution_status() {
        assert!(ExecutionStatus::Completed.is_terminal());
        assert!(ExecutionStatus::Failed.is_terminal());
        assert!(!ExecutionStatus::Running.is_terminal());
        
        assert!(ExecutionStatus::Running.can_pause());
        assert!(!ExecutionStatus::Completed.can_pause());
        
        assert!(ExecutionStatus::Paused.can_resume());
        assert!(!ExecutionStatus::Running.can_resume());
    }
    
    #[tokio::test]
    async fn test_storage_backends() {
        use crate::storage::{SimpleMemoryCache, FileStorage, CacheBackend, StorageBackend};
        use tempfile::TempDir;
        
        // Test memory cache
        let cache = SimpleMemoryCache::new();
        cache.set("test_key", b"test_value".to_vec(), None).await.unwrap();
        let value = cache.get("test_key").await;
        assert_eq!(value, Some(b"test_value".to_vec()));
        
        // Test file storage
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        storage.save("test_key", b"test_value").await.unwrap();
        let value = storage.load("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));
    }
}