//! Storage layer for the workflow toolkit

pub mod backends;
pub mod state_manager;

#[cfg(test)]
mod tests;

pub use backends::{CacheBackend, StorageBackend, SimpleMemoryCache, FileStorage, LocalMemoryCache, CacheConfig, StorageRecord, RetentionPolicy, CacheStats};

#[cfg(feature = "lancedb")]
pub use backends::LanceDbStorage;
pub use state_manager::{StateManager, ExecutionStatistics};