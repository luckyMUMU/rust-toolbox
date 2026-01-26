//! Performance optimization module
//!
//! This module provides performance monitoring, optimization utilities,
//! and tuning capabilities for the workflow toolkit.
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.

pub mod cache;
pub mod concurrency;
pub mod memory;
pub mod metrics;
pub mod profiler;

pub use cache::*;
pub use concurrency::*;
pub use memory::*;
pub use metrics::*;
pub use profiler::*;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceConfig {
    /// Memory optimization settings
    pub memory: MemoryConfig,

    /// Concurrency settings
    pub concurrency: ConcurrencyConfig,

    /// Caching configuration
    pub cache: CacheConfig,

    /// Metrics collection settings
    pub metrics: MetricsConfig,

    /// Profiling configuration
    pub profiling: ProfilingConfig,
}


/// Performance manager for coordinating optimizations
pub struct PerformanceManager {
    config: Arc<RwLock<PerformanceConfig>>,
    memory_manager: Arc<MemoryManager>,
    concurrency_manager: Arc<ConcurrencyManager>,
    metrics_collector: Arc<MetricsCollector>,
    cache_manager: Arc<CacheManager>,
    profiler: Arc<Profiler>,

    // Performance statistics
    stats: Arc<DashMap<String, PerformanceStats>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub execution_count: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub memory_usage: MemoryUsage,
    pub concurrency_stats: ConcurrencyStats,
    pub cache_stats: CacheStats,
    pub last_updated_timestamp: u64, // Unix timestamp in milliseconds
}

impl PerformanceManager {
    /// Create a new performance manager
    pub fn new(config: PerformanceConfig) -> Self {
        let config = Arc::new(RwLock::new(config));

        let memory_manager = Arc::new(MemoryManager::new(
            config.clone().try_read().unwrap().memory.clone(),
        ));

        let concurrency_manager = Arc::new(ConcurrencyManager::new(
            config.clone().try_read().unwrap().concurrency.clone(),
        ));

        let metrics_collector = Arc::new(MetricsCollector::new(
            config.clone().try_read().unwrap().metrics.clone(),
        ));

        let cache_manager = Arc::new(CacheManager::new(
            config.clone().try_read().unwrap().cache.clone(),
        ));

        let profiler = Arc::new(Profiler::new(
            config.clone().try_read().unwrap().profiling.clone(),
        ));

        Self {
            config,
            memory_manager,
            concurrency_manager,
            metrics_collector,
            cache_manager,
            profiler,
            stats: Arc::new(DashMap::new()),
        }
    }

    /// Start performance monitoring for a component
    pub async fn start_monitoring(&self, component: &str) -> PerformanceMonitor {
        let start_time = Instant::now();

        // Start memory monitoring
        let memory_snapshot = self.memory_manager.take_snapshot().await;

        // Start profiling if enabled
        let profile_session = if self.profiler.is_enabled().await {
            Some(self.profiler.start_session(component).await)
        } else {
            None
        };

        PerformanceMonitor {
            component: component.to_string(),
            start_time,
            memory_snapshot,
            profile_session,
            manager: Arc::new(self.clone()),
        }
    }

    /// Record performance statistics
    pub async fn record_stats(
        &self,
        component: &str,
        duration: Duration,
        memory_usage: MemoryUsage,
    ) {
        let mut stats =
            self.stats
                .entry(component.to_string())
                .or_insert_with(|| PerformanceStats {
                    execution_count: 0,
                    total_duration: Duration::ZERO,
                    average_duration: Duration::ZERO,
                    min_duration: Duration::MAX,
                    max_duration: Duration::ZERO,
                    memory_usage: MemoryUsage::default(),
                    concurrency_stats: ConcurrencyStats::default(),
                    cache_stats: CacheStats::default(),
                    last_updated_timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                });

        stats.execution_count += 1;
        stats.total_duration += duration;
        stats.average_duration = stats.total_duration / stats.execution_count as u32;
        stats.min_duration = stats.min_duration.min(duration);
        stats.max_duration = stats.max_duration.max(duration);
        stats.memory_usage = memory_usage.clone();
        stats.last_updated_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Update metrics
        self.metrics_collector
            .record_execution(component, duration, memory_usage)
            .await;
    }

    /// Get performance statistics for a component
    pub async fn get_stats(&self, component: &str) -> Option<PerformanceStats> {
        self.stats.get(component).map(|entry| entry.clone())
    }

    /// Get all performance statistics
    pub async fn get_all_stats(&self) -> Vec<(String, PerformanceStats)> {
        self.stats
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Optimize performance based on collected statistics
    pub async fn optimize(&self) -> crate::Result<OptimizationReport> {
        let mut report = OptimizationReport::new();

        // Memory optimization
        let memory_optimization = self.memory_manager.optimize().await?;
        report.memory_optimizations = memory_optimization;

        // Concurrency optimization
        let concurrency_optimization = self.concurrency_manager.optimize().await?;
        report.concurrency_optimizations = concurrency_optimization;

        // Cache optimization
        let cache_optimization = self.cache_manager.optimize().await?;
        report.cache_optimizations = cache_optimization;

        Ok(report)
    }

    /// Get memory manager
    pub fn memory_manager(&self) -> &Arc<MemoryManager> {
        &self.memory_manager
    }

    /// Get concurrency manager
    pub fn concurrency_manager(&self) -> &Arc<ConcurrencyManager> {
        &self.concurrency_manager
    }

    /// Get metrics collector
    pub fn metrics_collector(&self) -> &Arc<MetricsCollector> {
        &self.metrics_collector
    }

    /// Get cache manager
    pub fn cache_manager(&self) -> &Arc<CacheManager> {
        &self.cache_manager
    }

    /// Get profiler
    pub fn profiler(&self) -> &Arc<Profiler> {
        &self.profiler
    }
}

impl Clone for PerformanceManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            memory_manager: self.memory_manager.clone(),
            concurrency_manager: self.concurrency_manager.clone(),
            metrics_collector: self.metrics_collector.clone(),
            cache_manager: self.cache_manager.clone(),
            profiler: self.profiler.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Performance monitor for tracking individual operations
pub struct PerformanceMonitor {
    component: String,
    start_time: Instant,
    memory_snapshot: MemorySnapshot,
    profile_session: Option<ProfileSession>,
    manager: Arc<PerformanceManager>,
}

impl PerformanceMonitor {
    /// Finish monitoring and record statistics
    pub async fn finish(self) -> crate::Result<PerformanceResult> {
        let duration = self.start_time.elapsed();

        // Take final memory snapshot
        let final_memory = self.manager.memory_manager.take_snapshot().await;
        let memory_usage = MemoryUsage {
            initial: self.memory_snapshot.total_bytes,
            final_usage: final_memory.total_bytes,
            peak_usage: final_memory.peak_bytes,
            allocated: final_memory
                .total_bytes
                .saturating_sub(self.memory_snapshot.total_bytes),
        };

        // Finish profiling session
        let profile_data = if let Some(session) = self.profile_session {
            Some(self.manager.profiler.finish_session(session).await?)
        } else {
            None
        };

        // Record statistics
        self.manager
            .record_stats(&self.component, duration, memory_usage.clone())
            .await;

        Ok(PerformanceResult {
            component: self.component,
            duration,
            memory_usage,
            profile_data,
        })
    }
}

/// Result of performance monitoring
#[derive(Debug, Clone)]
pub struct PerformanceResult {
    pub component: String,
    pub duration: Duration,
    pub memory_usage: MemoryUsage,
    pub profile_data: Option<ProfileData>,
}

/// Optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub memory_optimizations: Vec<MemoryOptimization>,
    pub concurrency_optimizations: Vec<ConcurrencyOptimization>,
    pub cache_optimizations: Vec<CacheOptimization>,
    pub timestamp_millis: u64, // Unix timestamp in milliseconds
}

impl Default for OptimizationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationReport {
    pub fn new() -> Self {
        Self {
            memory_optimizations: Vec::new(),
            concurrency_optimizations: Vec::new(),
            cache_optimizations: Vec::new(),
            timestamp_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}

/// Performance optimization utilities
pub mod utils {
    use super::*;

    /// Measure execution time of an async function
    pub async fn measure_async<F, T>(f: F) -> (T, Duration)
    where
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = f.await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure execution time of a sync function
    pub fn measure_sync<F, T>(f: F) -> (T, Duration)
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Create a performance-optimized channel
    pub fn create_optimized_channel<T>(
        buffer_size: usize,
    ) -> (tokio::sync::mpsc::Sender<T>, tokio::sync::mpsc::Receiver<T>) {
        tokio::sync::mpsc::channel(buffer_size)
    }

    /// Create a performance-optimized semaphore
    pub fn create_optimized_semaphore(permits: usize) -> Arc<tokio::sync::Semaphore> {
        Arc::new(tokio::sync::Semaphore::new(permits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_manager_creation() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);

        assert!(manager.stats.is_empty());
    }

    #[tokio::test]
    async fn test_performance_monitoring() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);

        let monitor = manager.start_monitoring("test_component").await;

        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;

        let result = monitor.finish().await.unwrap();

        assert_eq!(result.component, "test_component");
        assert!(result.duration >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_stats_recording() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);

        let memory_usage = MemoryUsage {
            initial: 1000,
            final_usage: 1500,
            peak_usage: 2000,
            allocated: 500,
        };

        manager
            .record_stats("test", Duration::from_millis(100), memory_usage)
            .await;

        let stats = manager.get_stats("test").await.unwrap();
        assert_eq!(stats.execution_count, 1);
        assert_eq!(stats.total_duration, Duration::from_millis(100));
    }
}
