use async_trait::async_trait;
use crate::error::{CoreError, Result};
use super::Storage;
use std::path::PathBuf;

pub struct SledBackend {
    db: sled::Db,
}

impl SledBackend {
    pub fn new(path: PathBuf) -> Result<Self> {
        let db = sled::open(path).map_err(|e| CoreError::ConfigError(format!("Failed to open sled db: {}", e)))?;
        Ok(Self { db })
    }
}

#[async_trait]
impl Storage for SledBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let res = self.db.get(key).map_err(|e| CoreError::ConfigError(format!("Sled get error: {}", e)))?;
        Ok(res.map(|ivec| ivec.to_vec()))
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        self.db.insert(key, value).map_err(|e| CoreError::ConfigError(format!("Sled insert error: {}", e)))?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<()> {
        self.db.remove(key).map_err(|e| CoreError::ConfigError(format!("Sled remove error: {}", e)))?;
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        self.db.flush_async().await.map_err(|e| CoreError::ConfigError(format!("Sled flush error: {}", e)))?;
        Ok(())
    }
}
