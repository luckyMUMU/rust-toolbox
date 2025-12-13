use moka::future::Cache;
use std::time::Duration;

#[derive(Clone)]
pub struct CacheLayer {
    inner: Cache<String, Vec<u8>>,
}

impl CacheLayer {
    pub fn new(capacity: u64, ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(ttl)
            .build();
        
        Self { inner: cache }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.inner.get(key).await
    }

    pub async fn insert(&self, key: String, value: Vec<u8>) {
        self.inner.insert(key, value).await;
    }
    
    pub async fn invalidate(&self, key: &str) {
        self.inner.invalidate(key).await;
    }
}
