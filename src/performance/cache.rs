//! Cache optimization and management

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use dashmap::DashMap;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    
    /// Default cache TTL
    pub default_ttl: Duration,
    
    /// Maximum cache size (number of entries)
    pub max_entries: usize,
    
    /// Maximum memory usage (bytes)
    pub max_memory: usize,
    
    /// Cache eviction policy
    pub eviction_policy: EvictionPolicy,
    
    /// Cache warming configuration
    pub warming: CacheWarmingConfig,
    
    /// Cache partitioning
    pub partitions: usize,
    
    /// Enable cache compression
    pub compression: bool,
    
    /// Cache statistics collection
    pub collect_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_entries: 10000,
            max_memory: 100 * 1024 * 1024, // 100MB
            eviction_policy: EvictionPolicy::LRU,
            warming: CacheWarmingConfig::default(),
            partitions: 16,
            compression: false,
            collect_stats: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    FIFO, // First In, First Out
    TTL,  // Time To Live based
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingConfig {
    pub enabled: bool,
    pub warmup_percentage: f64,
    pub warmup_interval: Duration,
    pub preload_keys: Vec<String>,
}

impl Default for CacheWarmingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            warmup_percentage: 0.1, // 10%
            warmup_interval: Duration::from_secs(300), // 5 minutes
            preload_keys: Vec::new(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries: usize,
    pub memory_usage: usize,
    pub hit_rate: f64,
    pub average_access_time: Duration,
}

/// Cache optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOptimization {
    pub optimization_type: CacheOptimizationType,
    pub description: String,
    pub estimated_improvement: f64,
    pub priority: super::OptimizationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheOptimizationType {
    IncreaseSize,
    DecreaseSize,
    ChangeTTL,
    ChangeEvictionPolicy,
    EnableCompression,
    OptimizePartitioning,
    EnableWarming,
    OptimizeKeyDistribution,
}

/// Cache manager for optimization and monitoring
pub struct CacheManager {
    config: Arc<RwLock<CacheConfig>>,
    caches: Arc<DashMap<String, Arc<OptimizedCache>>>,
    global_stats: Arc<RwLock<CacheStats>>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            caches: Arc::new(DashMap::new()),
            global_stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    /// Get or create a cache
    pub async fn get_cache(&self, name: &str) -> Arc<OptimizedCache> {
        if let Some(cache) = self.caches.get(name) {
            cache.clone()
        } else {
            let config = self.config.read().await;
            let cache = Arc::new(OptimizedCache::new(name.to_string(), config.clone()));
            self.caches.insert(name.to_string(), cache.clone());
            cache
        }
    }
    
    /// Remove a cache
    pub async fn remove_cache(&self, name: &str) -> Option<Arc<OptimizedCache>> {
        self.caches.remove(name).map(|(_, cache)| cache)
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self, name: &str) -> Option<CacheStats> {
        if let Some(cache) = self.caches.get(name) {
            Some(cache.get_stats().await)
        } else {
            None
        }
    }
    
    /// Get global cache statistics
    pub async fn get_global_stats(&self) -> CacheStats {
        let mut global_stats = CacheStats::default();
        
        for cache_entry in self.caches.iter() {
            let cache = cache_entry.value();
            let stats = cache.get_stats().await;
            
            global_stats.hits += stats.hits;
            global_stats.misses += stats.misses;
            global_stats.evictions += stats.evictions;
            global_stats.entries += stats.entries;
            global_stats.memory_usage += stats.memory_usage;
        }
        
        // Calculate hit rate
        let total_requests = global_stats.hits + global_stats.misses;
        if total_requests > 0 {
            global_stats.hit_rate = global_stats.hits as f64 / total_requests as f64;
        }
        
        global_stats
    }
    
    /// Optimize cache configurations
    pub async fn optimize(&self) -> crate::Result<Vec<CacheOptimization>> {
        let mut optimizations = Vec::new();
        
        for cache_entry in self.caches.iter() {
            let cache_name = cache_entry.key();
            let cache = cache_entry.value();
            let stats = cache.get_stats().await;
            
            // Analyze hit rate
            if stats.hit_rate < 0.5 {
                optimizations.push(CacheOptimization {
                    optimization_type: CacheOptimizationType::IncreaseSize,
                    description: format!("Cache '{}' has low hit rate: {:.2}%", cache_name, stats.hit_rate * 100.0),
                    estimated_improvement: 0.3, // 30% improvement
                    priority: super::OptimizationPriority::High,
                });
            }
            
            // Analyze memory usage
            let config = self.config.read().await;
            let memory_usage_ratio = stats.memory_usage as f64 / config.max_memory as f64;
            
            if memory_usage_ratio > 0.9 {
                optimizations.push(CacheOptimization {
                    optimization_type: CacheOptimizationType::EnableCompression,
                    description: format!("Cache '{}' is using high memory: {:.1}%", cache_name, memory_usage_ratio * 100.0),
                    estimated_improvement: 0.4, // 40% memory reduction
                    priority: super::OptimizationPriority::Medium,
                });
            }
            
            // Analyze eviction rate
            let total_operations = stats.hits + stats.misses;
            if total_operations > 0 {
                let eviction_rate = stats.evictions as f64 / total_operations as f64;
                if eviction_rate > 0.1 {
                    optimizations.push(CacheOptimization {
                        optimization_type: CacheOptimizationType::ChangeTTL,
                        description: format!("Cache '{}' has high eviction rate: {:.2}%", cache_name, eviction_rate * 100.0),
                        estimated_improvement: 0.2, // 20% improvement
                        priority: super::OptimizationPriority::Medium,
                    });
                }
            }
        }
        
        Ok(optimizations)
    }
    
    /// Clear all caches
    pub async fn clear_all(&self) -> crate::Result<()> {
        for cache_entry in self.caches.iter() {
            let cache = cache_entry.value();
            cache.clear().await;
        }
        Ok(())
    }
    
    /// Warm up caches
    pub async fn warmup(&self) -> crate::Result<()> {
        let config = self.config.read().await;
        if !config.warming.enabled {
            return Ok(());
        }
        
        for cache_entry in self.caches.iter() {
            let cache = cache_entry.value();
            cache.warmup().await?;
        }
        
        Ok(())
    }
}

/// Optimized cache implementation
pub struct OptimizedCache {
    name: String,
    config: CacheConfig,
    partitions: Vec<Arc<RwLock<CachePartition>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl OptimizedCache {
    /// Create a new optimized cache
    pub fn new(name: String, config: CacheConfig) -> Self {
        let partition_count = config.partitions;
        let mut partitions = Vec::with_capacity(partition_count);
        
        for _ in 0..partition_count {
            partitions.push(Arc::new(RwLock::new(CachePartition::new(
                config.max_entries / partition_count,
                config.eviction_policy.clone(),
            ))));
        }
        
        Self {
            name,
            config,
            partitions,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    /// Get a value from the cache
    pub async fn get<K, V>(&self, key: &K) -> Option<V>
    where
        K: Hash + Eq + Clone + Send + Sync,
        V: Clone + Send + Sync + 'static,
    {
        if !self.config.enabled {
            return None;
        }
        
        let start_time = Instant::now();
        let partition_index = self.get_partition_index(key);
        let partition = &self.partitions[partition_index];
        
        let result = {
            let mut partition_guard = partition.write().await;
            partition_guard.get(key)
        };
        
        let access_time = start_time.elapsed();
        
        // Update statistics
        let mut stats = self.stats.write().await;
        if result.is_some() {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
        
        // Update average access time
        let total_requests = stats.hits + stats.misses;
        if total_requests > 0 {
            stats.average_access_time = 
                (stats.average_access_time * (total_requests - 1) as u32 + access_time) / total_requests as u32;
        }
        
        result
    }
    
    /// Put a value into the cache
    pub async fn put<K, V>(&self, key: K, value: V, ttl: Option<Duration>)
    where
        K: Hash + Eq + Clone + Send + Sync,
        V: Clone + Send + Sync + 'static,
    {
        if !self.config.enabled {
            return;
        }
        
        let partition_index = self.get_partition_index(&key);
        let partition = &self.partitions[partition_index];
        
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        let entry = CacheEntry {
            value: Box::new(value) as Box<dyn std::any::Any + Send + Sync>,
            expires_at: Instant::now() + ttl,
            access_count: 1,
            last_accessed: Instant::now(),
        };
        
        let mut partition_guard = partition.write().await;
        let evicted = partition_guard.put(key, entry);
        
        if evicted {
            let mut stats = self.stats.write().await;
            stats.evictions += 1;
        }
    }
    
    /// Remove a value from the cache
    pub async fn remove<K>(&self, key: &K) -> bool
    where
        K: Hash + Eq + Clone + Send + Sync,
    {
        let partition_index = self.get_partition_index(key);
        let partition = &self.partitions[partition_index];
        
        let mut partition_guard = partition.write().await;
        partition_guard.remove(key)
    }
    
    /// Clear the cache
    pub async fn clear(&self) {
        for partition in &self.partitions {
            let mut partition_guard = partition.write().await;
            partition_guard.clear();
        }
        
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
    }
    
    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let mut total_stats = self.stats.read().await.clone();
        
        // Calculate current entries and memory usage
        let mut total_entries = 0;
        let mut total_memory = 0;
        
        for partition in &self.partitions {
            let partition_guard = partition.read().await;
            total_entries += partition_guard.entries.len();
            total_memory += partition_guard.estimated_memory_usage();
        }
        
        total_stats.entries = total_entries;
        total_stats.memory_usage = total_memory;
        
        // Calculate hit rate
        let total_requests = total_stats.hits + total_stats.misses;
        if total_requests > 0 {
            total_stats.hit_rate = total_stats.hits as f64 / total_requests as f64;
        }
        
        total_stats
    }
    
    /// Warm up the cache
    pub async fn warmup(&self) -> crate::Result<()> {
        if !self.config.warming.enabled {
            return Ok(());
        }
        
        // In a real implementation, this would preload frequently accessed data
        tracing::info!("Warming up cache: {}", self.name);
        
        Ok(())
    }
    
    fn get_partition_index<K: Hash>(&self, key: &K) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.partitions.len()
    }
}

/// Cache partition for improved concurrency
struct CachePartition {
    entries: HashMap<u64, CacheEntry>,
    max_entries: usize,
    eviction_policy: EvictionPolicy,
    access_order: Vec<u64>, // For LRU/FIFO
    access_frequency: HashMap<u64, u64>, // For LFU
}

impl CachePartition {
    fn new(max_entries: usize, eviction_policy: EvictionPolicy) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            eviction_policy,
            access_order: Vec::new(),
            access_frequency: HashMap::new(),
        }
    }
    
    fn get<K, V>(&mut self, key: &K) -> Option<V>
    where
        K: Hash + Eq + Clone,
        V: Clone + 'static,
    {
        let key_hash = self.hash_key(key);
        
        if let Some(entry) = self.entries.get_mut(&key_hash) {
            // Check if entry has expired
            if entry.expires_at <= Instant::now() {
                self.entries.remove(&key_hash);
                self.remove_from_access_tracking(key_hash);
                return None;
            }
            
            // Update access information
            entry.access_count += 1;
            entry.last_accessed = Instant::now();
            
            // Try to downcast the value
            let result = if let Some(value) = entry.value.downcast_ref::<V>() {
                Some(value.clone())
            } else {
                None
            };
            
            // Update access tracking after getting the value
            if result.is_some() {
                self.update_access_tracking(key_hash);
            }
            
            result
        } else {
            None
        }
    }
    
    fn put<K>(&mut self, key: K, entry: CacheEntry) -> bool
    where
        K: Hash + Eq + Clone,
    {
        let key_hash = self.hash_key(&key);
        let mut evicted = false;
        
        // Check if we need to evict
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(&key_hash) {
            self.evict_one();
            evicted = true;
        }
        
        self.entries.insert(key_hash, entry);
        self.add_to_access_tracking(key_hash);
        
        evicted
    }
    
    fn remove<K>(&mut self, key: &K) -> bool
    where
        K: Hash + Eq + Clone,
    {
        let key_hash = self.hash_key(key);
        let removed = self.entries.remove(&key_hash).is_some();
        
        if removed {
            self.remove_from_access_tracking(key_hash);
        }
        
        removed
    }
    
    fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.access_frequency.clear();
    }
    
    fn evict_one(&mut self) {
        let key_to_evict = match self.eviction_policy {
            EvictionPolicy::LRU => {
                self.access_order.first().copied()
            }
            EvictionPolicy::FIFO => {
                self.access_order.first().copied()
            }
            EvictionPolicy::LFU => {
                self.access_frequency.iter()
                    .min_by_key(|(_, &freq)| freq)
                    .map(|(&key, _)| key)
            }
            EvictionPolicy::TTL => {
                // Find the entry with the earliest expiration
                self.entries.iter()
                    .min_by_key(|(_, entry)| entry.expires_at)
                    .map(|(&key, _)| key)
            }
            EvictionPolicy::Random => {
                let keys: Vec<u64> = self.entries.keys().copied().collect();
                if !keys.is_empty() {
                    use rand::seq::SliceRandom;
                    let mut rng = rand::thread_rng();
                    keys.choose(&mut rng).copied()
                } else {
                    None
                }
            }
        };
        
        if let Some(key) = key_to_evict {
            self.entries.remove(&key);
            self.remove_from_access_tracking(key);
        }
    }
    
    fn hash_key<K: Hash>(&self, key: &K) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
    
    fn add_to_access_tracking(&mut self, key: u64) {
        match self.eviction_policy {
            EvictionPolicy::LRU | EvictionPolicy::FIFO => {
                if let Some(pos) = self.access_order.iter().position(|&k| k == key) {
                    self.access_order.remove(pos);
                }
                self.access_order.push(key);
            }
            EvictionPolicy::LFU => {
                *self.access_frequency.entry(key).or_insert(0) += 1;
            }
            _ => {}
        }
    }
    
    fn update_access_tracking(&mut self, key: u64) {
        match self.eviction_policy {
            EvictionPolicy::LRU => {
                if let Some(pos) = self.access_order.iter().position(|&k| k == key) {
                    self.access_order.remove(pos);
                    self.access_order.push(key);
                }
            }
            EvictionPolicy::LFU => {
                *self.access_frequency.entry(key).or_insert(0) += 1;
            }
            _ => {}
        }
    }
    
    fn remove_from_access_tracking(&mut self, key: u64) {
        match self.eviction_policy {
            EvictionPolicy::LRU | EvictionPolicy::FIFO => {
                if let Some(pos) = self.access_order.iter().position(|&k| k == key) {
                    self.access_order.remove(pos);
                }
            }
            EvictionPolicy::LFU => {
                self.access_frequency.remove(&key);
            }
            _ => {}
        }
    }
    
    fn estimated_memory_usage(&self) -> usize {
        // Rough estimation - in a real implementation, this would be more accurate
        self.entries.len() * (std::mem::size_of::<u64>() + std::mem::size_of::<CacheEntry>() + 64)
    }
}

/// Cache entry
struct CacheEntry {
    value: Box<dyn std::any::Any + Send + Sync>,
    expires_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_manager_creation() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);
        
        let stats = manager.get_global_stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);
        let cache = manager.get_cache("test_cache").await;
        
        // Test put and get
        cache.put("key1", "value1", None).await;
        let result: Option<String> = cache.get(&"key1").await;
        assert_eq!(result, Some("value1".to_string()));
        
        // Test miss
        let result: Option<String> = cache.get(&"nonexistent").await;
        assert_eq!(result, None);
        
        // Test remove
        let removed = cache.remove(&"key1").await;
        assert!(removed);
        
        let result: Option<String> = cache.get(&"key1").await;
        assert_eq!(result, None);
    }
    
    #[tokio::test]
    async fn test_cache_statistics() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);
        let cache = manager.get_cache("test_cache").await;
        
        // Generate some cache activity
        cache.put("key1", "value1", None).await;
        let _: Option<String> = cache.get(&"key1").await; // Hit
        let _: Option<String> = cache.get(&"key2").await; // Miss
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
    
    #[tokio::test]
    async fn test_cache_eviction() {
        let mut config = CacheConfig::default();
        config.max_entries = 2; // Small cache for testing eviction
        
        let manager = CacheManager::new(config);
        let cache = manager.get_cache("test_cache").await;
        
        // Fill cache beyond capacity
        cache.put("key1", "value1", None).await;
        cache.put("key2", "value2", None).await;
        cache.put("key3", "value3", None).await; // Should trigger eviction
        
        let stats = cache.get_stats().await;
        assert!(stats.evictions > 0);
    }
    
    #[tokio::test]
    async fn test_cache_optimization() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);
        let cache = manager.get_cache("test_cache").await;
        
        // Generate activity with low hit rate
        for i in 0..100 {
            let _: Option<String> = cache.get(&format!("key{}", i)).await; // All misses
        }
        
        let optimizations = manager.optimize().await.unwrap();
        assert!(!optimizations.is_empty());
        
        // Should suggest increasing cache size due to low hit rate
        let has_size_optimization = optimizations.iter()
            .any(|opt| matches!(opt.optimization_type, CacheOptimizationType::IncreaseSize));
        assert!(has_size_optimization);
    }
}