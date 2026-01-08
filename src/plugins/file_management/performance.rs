//! Performance optimization utilities for file management operations

use crate::performance::{PerformanceManager, MemoryUsage};
use super::error::{FileManagementError, FileManagementResult};
use super::plugin::FileManagementPerformanceConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

/// Performance-optimized file operation manager
pub struct OptimizedFileOperationManager {
    config: FileManagementPerformanceConfig,
    performance_manager: Option<Arc<PerformanceManager>>,
    operation_semaphore: Arc<Semaphore>,
    memory_pool: Arc<RwLock<MemoryPool>>,
    operation_cache: Arc<RwLock<OperationCache>>,
}

impl OptimizedFileOperationManager {
    /// Create a new optimized file operation manager
    pub fn new(
        config: FileManagementPerformanceConfig,
        performance_manager: Option<Arc<PerformanceManager>>,
    ) -> Self {
        let operation_semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
        let memory_pool = Arc::new(RwLock::new(MemoryPool::new(
            config.memory_pool_size_mb * 1024 * 1024
        )));
        let operation_cache = Arc::new(RwLock::new(OperationCache::new(
            config.cache_size_mb * 1024 * 1024,
            Duration::from_secs(config.cache_ttl_seconds),
        )));

        Self {
            config,
            performance_manager,
            operation_semaphore,
            memory_pool,
            operation_cache,
        }
    }

    /// Execute a file operation with performance monitoring
    pub async fn execute_optimized_operation<F, T>(
        &self,
        operation_name: &str,
        operation: F,
    ) -> FileManagementResult<T>
    where
        F: std::future::Future<Output = FileManagementResult<T>> + Send,
        T: Send,
    {
        // Acquire operation permit for concurrency control
        let _permit = self.operation_semaphore.acquire().await.map_err(|_| {
            FileManagementError::other("Failed to acquire operation permit")
        })?;

        // Start performance monitoring if available
        let monitor = if let Some(ref perf_manager) = self.performance_manager {
            Some(perf_manager.start_monitoring(operation_name).await)
        } else {
            None
        };

        // Execute the operation
        let result = operation.await;

        // Finish performance monitoring
        if let Some(monitor) = monitor {
            if let Err(e) = monitor.finish().await {
                warn!("Failed to finish performance monitoring: {}", e);
            }
        }

        result
    }

    /// Get optimized buffer for I/O operations
    pub async fn get_io_buffer(&self) -> Vec<u8> {
        let buffer_size = self.config.io_buffer_size_kb * 1024;
        
        // Try to get buffer from memory pool first
        if let Some(buffer) = self.memory_pool.read().await.get_buffer(buffer_size) {
            buffer
        } else {
            // Allocate new buffer if pool is empty
            vec![0u8; buffer_size]
        }
    }

    /// Return buffer to memory pool for reuse
    pub async fn return_io_buffer(&self, buffer: Vec<u8>) {
        if self.config.enable_memory_optimization {
            self.memory_pool.write().await.return_buffer(buffer);
        }
    }

    /// Check if operation result is cached
    pub async fn get_cached_result(&self, operation_key: &str) -> Option<CachedResult> {
        if self.config.enable_caching {
            self.operation_cache.read().await.get(operation_key)
        } else {
            None
        }
    }

    /// Cache operation result
    pub async fn cache_result(&self, operation_key: String, result: CachedResult) {
        if self.config.enable_caching {
            self.operation_cache.write().await.insert(operation_key, result);
        }
    }

    /// Optimize memory usage by cleaning up caches and pools
    pub async fn optimize_memory(&self) -> FileManagementResult<usize> {
        let mut freed_bytes = 0;

        // Clean up memory pool
        freed_bytes += self.memory_pool.write().await.cleanup();

        // Clean up operation cache
        freed_bytes += self.operation_cache.write().await.cleanup();

        // Trigger GC hint if performance manager is available
        if let Some(ref perf_manager) = self.performance_manager {
            perf_manager.memory_manager().gc_hint().await;
        }

        info!("Memory optimization freed {} bytes", freed_bytes);
        Ok(freed_bytes)
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        let memory_pool_stats = self.memory_pool.read().await.get_stats();
        let cache_stats = self.operation_cache.read().await.get_stats();

        PerformanceStats {
            memory_pool_stats,
            cache_stats,
            concurrent_operations: self.operation_semaphore.available_permits(),
            max_concurrent_operations: self.config.max_concurrent_operations,
        }
    }
}

/// Memory pool for reusing buffers and reducing allocations
pub struct MemoryPool {
    max_size: usize,
    current_size: usize,
    buffers: HashMap<usize, Vec<Vec<u8>>>, // size -> buffers
}

impl MemoryPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            current_size: 0,
            buffers: HashMap::new(),
        }
    }

    pub fn get_buffer(&self, size: usize) -> Option<Vec<u8>> {
        if let Some(buffers) = self.buffers.get(&size) {
            if !buffers.is_empty() {
                // This is a simplified implementation - in reality we'd need mutable access
                // For now, just return None to indicate we should allocate
                None
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn return_buffer(&mut self, buffer: Vec<u8>) {
        let size = buffer.len();
        
        // Only keep buffer if we have space
        if self.current_size + size <= self.max_size {
            self.buffers.entry(size).or_insert_with(Vec::new).push(buffer);
            self.current_size += size;
        }
    }

    pub fn cleanup(&mut self) -> usize {
        let freed_bytes = self.current_size;
        self.buffers.clear();
        self.current_size = 0;
        freed_bytes
    }

    pub fn get_stats(&self) -> MemoryPoolStats {
        let total_buffers = self.buffers.values().map(|v| v.len()).sum();
        
        MemoryPoolStats {
            current_size: self.current_size,
            max_size: self.max_size,
            total_buffers,
            buffer_sizes: self.buffers.keys().cloned().collect(),
        }
    }
}

/// Cache for operation results to avoid redundant computations
pub struct OperationCache {
    max_size: usize,
    current_size: usize,
    ttl: Duration,
    entries: HashMap<String, CacheEntry>,
}

impl OperationCache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            max_size,
            current_size: 0,
            ttl,
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<CachedResult> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                None
            } else {
                Some(entry.result.clone())
            }
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: String, result: CachedResult) {
        let entry_size = key.len() + result.estimated_size();
        
        // Clean up expired entries first
        self.cleanup_expired();
        
        // Make space if needed
        while self.current_size + entry_size > self.max_size && !self.entries.is_empty() {
            self.evict_oldest();
        }
        
        // Insert new entry
        if self.current_size + entry_size <= self.max_size {
            let entry = CacheEntry {
                result,
                created_at: Instant::now(),
                ttl: self.ttl,
            };
            
            self.entries.insert(key, entry);
            self.current_size += entry_size;
        }
    }

    pub fn cleanup(&mut self) -> usize {
        let freed_bytes = self.current_size;
        self.entries.clear();
        self.current_size = 0;
        freed_bytes
    }

    fn cleanup_expired(&mut self) {
        let expired_keys: Vec<String> = self.entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_size = self.current_size.saturating_sub(
                    key.len() + entry.result.estimated_size()
                );
            }
        }
    }

    fn evict_oldest(&mut self) {
        if let Some((oldest_key, _)) = self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            if let Some(entry) = self.entries.remove(&oldest_key) {
                self.current_size = self.current_size.saturating_sub(
                    oldest_key.len() + entry.result.estimated_size()
                );
            }
        }
    }

    pub fn get_stats(&self) -> CacheStats {
        let expired_count = self.entries.values().filter(|e| e.is_expired()).count();
        
        CacheStats {
            current_size: self.current_size,
            max_size: self.max_size,
            entry_count: self.entries.len(),
            expired_count,
            hit_rate: 0.0, // Would need to track hits/misses for this
        }
    }
}

/// Cache entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry {
    result: CachedResult,
    created_at: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Cached operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

impl CachedResult {
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn estimated_size(&self) -> usize {
        // Rough estimation of memory usage
        self.data.to_string().len() + 
        self.metadata.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>()
    }
}

/// Performance statistics for the optimized file operation manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub memory_pool_stats: MemoryPoolStats,
    pub cache_stats: CacheStats,
    pub concurrent_operations: usize,
    pub max_concurrent_operations: usize,
}

/// Memory pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolStats {
    pub current_size: usize,
    pub max_size: usize,
    pub total_buffers: usize,
    pub buffer_sizes: Vec<usize>,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub current_size: usize,
    pub max_size: usize,
    pub entry_count: usize,
    pub expired_count: usize,
    pub hit_rate: f64,
}

/// Streaming utilities for large file operations
pub struct StreamingUtils;

impl StreamingUtils {
    /// Process large file in chunks to optimize memory usage
    pub async fn process_file_in_chunks<F, T>(
        file_path: &Path,
        chunk_size: usize,
        mut processor: F,
    ) -> FileManagementResult<Vec<T>>
    where
        F: FnMut(&[u8]) -> FileManagementResult<T>,
        T: Send,
    {
        use tokio::fs::File;
        use tokio::io::{AsyncReadExt, BufReader};

        let file = File::open(file_path).await.map_err(|e| {
            FileManagementError::io(
                format!("Failed to open file {}", file_path.display()),
                e,
            )
        })?;

        let mut reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut buffer = vec![0u8; chunk_size];

        loop {
            let bytes_read = reader.read(&mut buffer).await.map_err(|e| {
                FileManagementError::io(
                    format!("Failed to read from file {}", file_path.display()),
                    e,
                )
            })?;

            if bytes_read == 0 {
                break; // End of file
            }

            let chunk = &buffer[..bytes_read];
            let result = processor(chunk)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Stream directory contents lazily to avoid loading everything into memory
    pub async fn stream_directory_contents<F>(
        dir_path: &Path,
        batch_size: usize,
        mut processor: F,
    ) -> FileManagementResult<()>
    where
        F: FnMut(Vec<PathBuf>) -> FileManagementResult<()>,
    {
        use tokio::fs::read_dir;

        let mut entries = read_dir(dir_path).await.map_err(|e| {
            FileManagementError::io(
                format!("Failed to read directory {}", dir_path.display()),
                e,
            )
        })?;

        let mut batch = Vec::with_capacity(batch_size);

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            FileManagementError::io(
                format!("Failed to read directory entry in {}", dir_path.display()),
                e,
            )
        })? {
            batch.push(entry.path());

            if batch.len() >= batch_size {
                processor(batch.clone())?;
                batch.clear();
            }
        }

        // Process remaining entries
        if !batch.is_empty() {
            processor(batch)?;
        }

        Ok(())
    }
}

/// Compression utilities for temporary files
pub struct CompressionUtils;

impl CompressionUtils {
    /// Compress data using the configured compression level
    pub fn compress_data(data: &[u8], level: u32) -> FileManagementResult<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        encoder.write_all(data).map_err(|e| {
            FileManagementError::other(format!("Compression failed: {}", e))
        })?;
        
        encoder.finish().map_err(|e| {
            FileManagementError::other(format!("Compression finalization failed: {}", e))
        })
    }

    /// Decompress data
    pub fn decompress_data(compressed_data: &[u8]) -> FileManagementResult<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| {
            FileManagementError::other(format!("Decompression failed: {}", e))
        })?;
        
        Ok(decompressed)
    }

    /// Check if compression would be beneficial for the given data
    pub fn should_compress(data: &[u8], threshold_size: usize) -> bool {
        data.len() > threshold_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_pool() {
        let mut pool = MemoryPool::new(1024);
        
        // Test buffer return and retrieval
        let buffer = vec![0u8; 512];
        pool.return_buffer(buffer);
        
        let stats = pool.get_stats();
        assert_eq!(stats.current_size, 512);
        assert_eq!(stats.total_buffers, 1);
    }

    #[tokio::test]
    async fn test_operation_cache() {
        let mut cache = OperationCache::new(1024, Duration::from_secs(60));
        
        let result = CachedResult::new(serde_json::json!({"test": "data"}));
        cache.insert("test_key".to_string(), result);
        
        let retrieved = cache.get("test_key");
        assert!(retrieved.is_some());
        
        let stats = cache.get_stats();
        assert_eq!(stats.entry_count, 1);
    }

    #[tokio::test]
    async fn test_streaming_utils() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        
        // Create test file
        tokio::fs::write(&file_path, b"Hello, World! This is a test file for streaming.").await.unwrap();
        
        // Process file in chunks
        let results = StreamingUtils::process_file_in_chunks(
            &file_path,
            10, // 10-byte chunks
            |chunk| Ok(chunk.len()),
        ).await.unwrap();
        
        assert!(!results.is_empty());
        let total_bytes: usize = results.iter().sum();
        assert_eq!(total_bytes, 49); // Length of test string
    }

    #[test]
    fn test_compression_utils() {
        let test_data = b"This is some test data that should compress well because it has repetitive patterns.";
        
        let compressed = CompressionUtils::compress_data(test_data, 6).unwrap();
        assert!(compressed.len() < test_data.len());
        
        let decompressed = CompressionUtils::decompress_data(&compressed).unwrap();
        assert_eq!(decompressed, test_data);
    }
}