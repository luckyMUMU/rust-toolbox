use async_trait::async_trait;
use crate::error::Result;

pub mod cache;
pub mod sled_backend;
pub mod manager;
pub mod file_ops;

pub use manager::PersistenceManager;
pub use file_ops::*;

/// Abstract storage backend interface
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Set value for key
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    
    /// Remove key
    async fn remove(&self, key: &str) -> Result<()>;
    
    /// Flush pending writes
    async fn flush(&self) -> Result<()>;
}
