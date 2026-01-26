//! Storage layer for the workflow toolkit
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.

pub mod backends;
pub mod backup;
pub mod state_manager;

#[cfg(test)]
mod tests;

pub use backends::{
    CacheBackend, CacheConfig, CacheStats, FileStorage, LocalMemoryCache, RetentionPolicy,
    SimpleMemoryCache, StorageBackend, StorageRecord,
};
pub use backup::{
    BackupConfig, BackupData, BackupManager, BackupMetadata, BackupStatistics, BackupType,
    BackupVerification, RestoreResult,
};

#[cfg(feature = "lancedb")]
pub use backends::LanceDbStorage;
pub use state_manager::{ExecutionStatistics, StateManager};
