use std::sync::Arc;
use std::path::PathBuf;
use serde::{Serialize, de::DeserializeOwned};
use crate::error::{CoreError, Result};
use super::{Storage, sled_backend::SledBackend, cache::CacheLayer};
use tempfile::TempDir;

pub struct PersistenceManager {
    storage: Arc<dyn Storage>,
    cache: CacheLayer,
}

impl PersistenceManager {
    pub fn new(path: PathBuf) -> Result<Self> {
        let storage = Arc::new(SledBackend::new(path)?);
        let cache = CacheLayer::new(1000, std::time::Duration::from_secs(3600)); // Default 1 hour TTL
        
        Ok(Self { storage, cache })
    }

    /// 获取 KV 数据 (优先查缓存，未命中查 DB)
    pub async fn get_data<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        // 1. Check cache
        if let Some(data) = self.cache.get(key).await {
            // Decompress
            let decompressed = zstd::decode_all(&data[..])
                .map_err(|e| CoreError::ConfigError(format!("Decompression failed: {}", e)))?;
            
            // Deserialize
            let val: T = bincode::serde::decode_from_slice(&decompressed, bincode::config::standard())
                .map_err(|e| CoreError::ConfigError(format!("Deserialization failed: {}", e)))?
                .0;
            
            return Ok(Some(val));
        }

        // 2. Check storage
        if let Some(data) = self.storage.get(key).await? {
            // Decompress
            let decompressed = zstd::decode_all(&data[..])
                .map_err(|e| CoreError::ConfigError(format!("Decompression failed: {}", e)))?;
            
            // Deserialize
            let val: T = bincode::serde::decode_from_slice(&decompressed, bincode::config::standard())
                .map_err(|e| CoreError::ConfigError(format!("Deserialization failed: {}", e)))?
                .0;
            
            // Backfill cache (store compressed to save memory)
            self.cache.insert(key.to_string(), data).await;

            return Ok(Some(val));
        }

        Ok(None)
    }

    /// 保存 KV 数据 (同时更新缓存和 DB)
    pub async fn set_data<T: Serialize + ?Sized>(&self, key: &str, value: &T) -> Result<()> {
        // Serialize
        let bytes = bincode::serde::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| CoreError::ConfigError(format!("Serialization failed: {}", e)))?;
            
        // Compress
        let compressed = zstd::encode_all(&bytes[..], 3) // Level 3 is default
            .map_err(|e| CoreError::ConfigError(format!("Compression failed: {}", e)))?;

        // Update storage
        self.storage.set(key, &compressed).await?;
        
        // Update cache
        self.cache.insert(key.to_string(), compressed).await;
        
        Ok(())
    }
    
    /// 加载配置 (应用级或工具级)
    pub fn load_config<T: Serialize + DeserializeOwned + Default>(&self, app_name: &str, config_name: &str) -> Result<T> {
        confy::load(app_name, config_name)
            .map_err(|e| CoreError::ConfigError(format!("Failed to load config: {}", e)))
    }
    
    /// 保存配置
    pub fn save_config<T: Serialize>(&self, app_name: &str, config_name: &str, config: &T) -> Result<()> {
        confy::store(app_name, config_name, config)
            .map_err(|e| CoreError::ConfigError(format!("Failed to save config: {}", e)))
    }
    
    /// 创建临时目录 (自动清理)
    pub async fn create_temp_dir(&self) -> Result<TempDir> {
        tempfile::tempdir().map_err(|e| CoreError::ConfigError(format!("Failed to create temp dir: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
    struct TestData {
        id: u32,
        name: String,
        values: Vec<f64>,
    }

    #[tokio::test]
    async fn test_persistence_manager() -> Result<()> {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let manager = PersistenceManager::new(db_path)?;

        let data = TestData {
            id: 42,
            name: "Test".to_string(),
            values: vec![1.0, 2.0, 3.14],
        };

        // Test Set
        manager.set_data("test_key", &data).await?;

        // Test Get (from cache)
        let cached: Option<TestData> = manager.get_data("test_key").await?;
        assert_eq!(cached, Some(data));

        // Test Persistence (simulate restart by clearing cache)
        // Since we can't easily clear private cache, we create a new manager instance pointing to same DB
        // But sled locks the DB. So we drop the first manager.
        drop(manager);
        
        // Re-open
        let db_path = temp_dir.path().join("test.db");
        let manager2 = PersistenceManager::new(db_path)?;
        let loaded: Option<TestData> = manager2.get_data("test_key").await?;
        
        let expected = TestData {
            id: 42,
            name: "Test".to_string(),
            values: vec![1.0, 2.0, 3.14],
        };
        assert_eq!(loaded, Some(expected));

        Ok(())
    }
}
