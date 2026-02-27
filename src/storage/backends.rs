//! Storage backend traits and implementations

use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

// Conditional imports for LanceDB (only when feature is enabled)
#[cfg(feature = "lancedb")]
use arrow::array::{BinaryArray, StringArray, TimestampMillisecondArray};
#[cfg(feature = "lancedb")]
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
#[cfg(feature = "lancedb")]
use arrow::record_batch::RecordBatch;
#[cfg(feature = "lancedb")]
use std::path::Path;

/// Configuration for cache backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_capacity: u64,
    pub ttl: Option<Duration>,
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 1000,
            ttl: Some(Duration::from_secs(3600)), // 1 hour default TTL
            enable_metrics: false,
        }
    }
}

/// Storage record structure for LanceDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRecord {
    pub key: String,
    pub value: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<String>,
}

/// Retention policy for data cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_age: chrono::Duration,
    pub max_count: Option<usize>,
}

/// Trait for storage backends
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn save(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn batch_save(&self, items: Vec<(String, Vec<u8>)>) -> Result<()>;
    async fn batch_load(&self, keys: Vec<String>) -> Result<Vec<Option<Vec<u8>>>>;
}

/// Trait for cache backends
#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
    fn size(&self) -> usize;
}

/// Simple in-memory cache implementation
pub struct SimpleMemoryCache {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl SimpleMemoryCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for SimpleMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheBackend for SimpleMemoryCache {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.read().ok()?.get(key).cloned()
    }

    async fn set(&self, key: &str, value: Vec<u8>, _ttl: Option<Duration>) -> Result<()> {
        if let Ok(mut data) = self.data.write() {
            data.insert(key.to_string(), value);
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        if let Ok(mut data) = self.data.write() {
            data.remove(key);
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
        Ok(())
    }

    fn size(&self) -> usize {
        self.data.read().map(|data| data.len()).unwrap_or(0)
    }
}

/// Simple file-based storage implementation
pub struct FileStorage {
    base_path: std::path::PathBuf,
}

impl FileStorage {
    pub fn new<P: Into<std::path::PathBuf>>(base_path: P) -> Result<Self> {
        let base_path = base_path.into();
        std::fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    fn get_file_path(&self, key: &str) -> std::path::PathBuf {
        // Simple key to filename mapping (in production, would need proper escaping)
        let filename = key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.base_path.join(format!("{}.dat", filename))
    }
}

#[async_trait]
impl StorageBackend for FileStorage {
    async fn save(&self, key: &str, value: &[u8]) -> Result<()> {
        let path = self.get_file_path(key);
        tokio::fs::write(path, value).await?;
        Ok(())
    }

    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_file_path(key);
        match tokio::fs::read(path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.get_file_path(key);
        match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".dat") {
                    // Convert filename back to original key format
                    let key = filename.trim_end_matches(".dat").replace('_', ":");
                    if key.starts_with(prefix) {
                        keys.push(key);
                    }
                }
            }
        }

        Ok(keys)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.get_file_path(key);
        Ok(path.exists())
    }

    async fn batch_save(&self, items: Vec<(String, Vec<u8>)>) -> Result<()> {
        use futures::future::join_all;

        let futures: Vec<_> = items
            .iter()
            .map(|(key, value)| self.save(key, value))
            .collect();

        let results = join_all(futures).await;
        for result in results {
            result?;
        }
        Ok(())
    }

    async fn batch_load(&self, keys: Vec<String>) -> Result<Vec<Option<Vec<u8>>>> {
        use futures::future::join_all;

        let futures: Vec<_> = keys.iter().map(|key| self.load(key)).collect();

        let results = join_all(futures).await;
        let mut final_results = Vec::new();
        for result in results {
            final_results.push(result?);
        }
        Ok(final_results)
    }
}

#[cfg(feature = "lancedb")]
/// LanceDB storage backend implementation
pub struct LanceDbStorage {
    connection: Arc<lancedb::Connection>,
    table_name: String,
}

#[cfg(feature = "lancedb")]
impl LanceDbStorage {
    /// Create a new LanceDB storage backend
    pub async fn new(db_path: &Path, table_name: &str) -> Result<Self> {
        let connection = lancedb::connect(db_path).execute().await?;

        // Create table structure for storing workflow data
        let schema = Arc::new(Schema::new(vec![
            Field::new("key", DataType::Utf8, false),
            Field::new("value", DataType::Binary, false),
            Field::new(
                "timestamp",
                DataType::Timestamp(TimeUnit::Millisecond, None),
                false,
            ),
            Field::new("metadata", DataType::Utf8, true),
        ]));

        // Try to create the table if it doesn't exist
        let table_exists = connection
            .table_names()
            .execute()
            .await?
            .contains(&table_name.to_string());

        if !table_exists {
            // Create empty table with schema
            let empty_batch = RecordBatch::try_new(
                schema.clone(),
                vec![
                    Arc::new(StringArray::from(Vec::<String>::new())),
                    Arc::new(BinaryArray::from(Vec::<Vec<u8>>::new())),
                    Arc::new(TimestampMillisecondArray::from(Vec::<i64>::new())),
                    Arc::new(StringArray::from(Vec::<Option<String>>::new())),
                ],
            )?;

            connection
                .create_table(table_name, vec![empty_batch])
                .execute()
                .await?;
        }

        Ok(Self {
            connection: Arc::new(connection),
            table_name: table_name.to_string(),
        })
    }
}

#[cfg(feature = "lancedb")]
#[async_trait]
impl StorageBackend for LanceDbStorage {
    async fn save(&self, key: &str, value: &[u8]) -> Result<()> {
        let table = self
            .connection
            .open_table(&self.table_name)
            .execute()
            .await?;

        let batch = RecordBatch::try_new(
            table.schema().clone(),
            vec![
                Arc::new(StringArray::from(vec![key])),
                Arc::new(BinaryArray::from(vec![value])),
                Arc::new(TimestampMillisecondArray::from(vec![
                    Utc::now().timestamp_millis()
                ])),
                Arc::new(StringArray::from(vec![Option::<String>::None])),
            ],
        )?;

        table.add(batch).execute().await?;
        Ok(())
    }

    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let table = self
            .connection
            .open_table(&self.table_name)
            .execute()
            .await?;

        let results = table
            .search(vec![])
            .where_clause(format!("key = '{}'", key))
            .execute()
            .await?;

        if let Some(batch) = results.first() {
            if let Some(binary_array) = batch.column(1).as_any().downcast_ref::<BinaryArray>() {
                if binary_array.len() > 0 {
                    return Ok(Some(binary_array.value(0).to_vec()));
                }
            }
        }

        Ok(None)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let table = self
            .connection
            .open_table(&self.table_name)
            .execute()
            .await?;
        table.delete(&format!("key = '{}'", key)).await?;
        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let table = self
            .connection
            .open_table(&self.table_name)
            .execute()
            .await?;

        let results = table
            .search(vec![])
            .where_clause(format!("key LIKE '{}%'", prefix))
            .execute()
            .await?;

        let mut keys = Vec::new();
        for batch in results {
            if let Some(string_array) = batch.column(0).as_any().downcast_ref::<StringArray>() {
                for i in 0..string_array.len() {
                    if let Some(key) = string_array.value(i) {
                        keys.push(key.to_string());
                    }
                }
            }
        }

        Ok(keys)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.load(key).await?.is_some())
    }

    async fn batch_save(&self, items: Vec<(String, Vec<u8>)>) -> Result<()> {
        let table = self
            .connection
            .open_table(&self.table_name)
            .execute()
            .await?;

        let keys: Vec<String> = items.iter().map(|(k, _)| k.clone()).collect();
        let values: Vec<Vec<u8>> = items.iter().map(|(_, v)| v.clone()).collect();
        let timestamps: Vec<i64> = vec![Utc::now().timestamp_millis(); items.len()];
        let metadata: Vec<Option<String>> = vec![None; items.len()];

        let batch = RecordBatch::try_new(
            table.schema().clone(),
            vec![
                Arc::new(StringArray::from(keys)),
                Arc::new(BinaryArray::from(values)),
                Arc::new(TimestampMillisecondArray::from(timestamps)),
                Arc::new(StringArray::from(metadata)),
            ],
        )?;

        table.add(batch).execute().await?;
        Ok(())
    }

    async fn batch_load(&self, keys: Vec<String>) -> Result<Vec<Option<Vec<u8>>>> {
        let mut results = Vec::new();
        for key in keys {
            results.push(self.load(&key).await?);
        }
        Ok(results)
    }
}
/// High-performance local memory cache using moka
pub struct LocalMemoryCache {
    cache: Arc<moka::future::Cache<String, Vec<u8>>>,
    #[allow(dead_code)]
    config: CacheConfig,
}

impl LocalMemoryCache {
    /// Create a new local memory cache with configuration
    pub fn new(config: CacheConfig) -> Self {
        let mut builder = moka::future::Cache::builder().max_capacity(config.max_capacity);

        if let Some(ttl) = config.ttl {
            builder = builder.time_to_live(ttl);
        }

        Self {
            cache: Arc::new(builder.build()),
            config,
        }
    }

    /// Create a new cache with default configuration
    pub fn with_capacity(max_capacity: u64) -> Self {
        Self::new(CacheConfig {
            max_capacity,
            ..Default::default()
        })
    }

    /// Create a new cache with capacity and TTL
    pub fn with_capacity_and_ttl(max_capacity: u64, ttl: Duration) -> Self {
        Self::new(CacheConfig {
            max_capacity,
            ttl: Some(ttl),
            ..Default::default()
        })
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: u64,
    pub weighted_size: u64,
}

#[async_trait]
impl CacheBackend for LocalMemoryCache {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.get(key).await
    }

    async fn set(&self, key: &str, value: Vec<u8>, _ttl: Option<Duration>) -> Result<()> {
        // Note: moka doesn't support per-key TTL, so we use the global TTL
        // In a production system, you might want to use a different cache implementation
        // that supports per-key TTL if this is a requirement
        self.cache.insert(key.to_string(), value).await;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.cache.invalidate(key).await;
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        Ok(())
    }

    fn size(&self) -> usize {
        self.cache.entry_count() as usize
    }
}
