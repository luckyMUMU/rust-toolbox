//! Memory usage optimization and monitoring

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum memory usage per workflow (in bytes)
    pub max_workflow_memory: usize,

    /// Maximum memory usage per tool (in bytes)
    pub max_tool_memory: usize,

    /// Memory cleanup threshold (percentage)
    pub cleanup_threshold: f64,

    /// Memory monitoring interval
    pub monitoring_interval: Duration,

    /// Enable memory pooling
    pub enable_pooling: bool,

    /// Pool sizes for different object types
    pub pool_sizes: HashMap<String, usize>,

    /// Enable garbage collection hints
    pub enable_gc_hints: bool,

    /// Memory pressure detection threshold
    pub pressure_threshold: f64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        let mut pool_sizes = HashMap::new();
        pool_sizes.insert("workflow_execution".to_string(), 100);
        pool_sizes.insert("tool_result".to_string(), 500);
        pool_sizes.insert("node_state".to_string(), 1000);

        Self {
            max_workflow_memory: 1024 * 1024 * 1024, // 1GB
            max_tool_memory: 512 * 1024 * 1024,      // 512MB
            cleanup_threshold: 0.8,                  // 80%
            monitoring_interval: Duration::from_secs(30),
            enable_pooling: true,
            pool_sizes,
            enable_gc_hints: true,
            pressure_threshold: 0.9, // 90%
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryUsage {
    pub initial: usize,
    pub final_usage: usize,
    pub peak_usage: usize,
    pub allocated: usize,
}

/// Memory snapshot
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub total_bytes: usize,
    pub peak_bytes: usize,
    pub allocated_objects: usize,
    pub timestamp_millis: u64, // Unix timestamp in milliseconds
}

/// Memory optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptimization {
    pub optimization_type: MemoryOptimizationType,
    pub description: String,
    pub estimated_savings: usize,
    pub priority: OptimizationPriority,
    pub component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryOptimizationType {
    ReduceBufferSize,
    EnablePooling,
    IncreaseCleanupFrequency,
    OptimizeDataStructures,
    ReduceCacheSize,
    CompressData,
    LazyLoading,
    StreamProcessing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Memory manager for optimization and monitoring
pub struct MemoryManager {
    config: Arc<RwLock<MemoryConfig>>,
    usage_history: Arc<DashMap<String, Vec<MemoryUsage>>>,
    pools: Arc<DashMap<String, Arc<ObjectPool>>>,
    pressure_detector: Arc<MemoryPressureDetector>,
    cleanup_scheduler: Arc<CleanupScheduler>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(config: MemoryConfig) -> Self {
        let config = Arc::new(RwLock::new(config));
        let usage_history = Arc::new(DashMap::new());
        let pools = Arc::new(DashMap::new());

        let pressure_detector = Arc::new(MemoryPressureDetector::new(config.clone()));

        let cleanup_scheduler =
            Arc::new(CleanupScheduler::new(config.clone(), usage_history.clone()));

        Self {
            config,
            usage_history,
            pools,
            pressure_detector,
            cleanup_scheduler,
        }
    }

    /// Take a memory snapshot
    pub async fn take_snapshot(&self) -> MemorySnapshot {
        let memory_info = self.get_system_memory_info().await;

        MemorySnapshot {
            total_bytes: memory_info.used_bytes,
            peak_bytes: memory_info.peak_bytes,
            allocated_objects: memory_info.allocated_objects,
            timestamp_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Record memory usage for a component
    pub async fn record_usage(&self, component: &str, usage: MemoryUsage) {
        let mut history = self
            .usage_history
            .entry(component.to_string())
            .or_default();
        history.push(usage.clone());

        // Keep only recent history (last 1000 entries)
        if history.len() > 1000 {
            let len = history.len();
            history.drain(0..len - 1000);
        }

        // Check for memory pressure
        self.pressure_detector.check_pressure(&usage).await;
    }

    /// Get or create an object pool
    pub async fn get_pool<T>(&self, pool_name: &str) -> Arc<ObjectPool>
    where
        T: Clone + Send + Sync + 'static,
    {
        if let Some(pool_ref) = self.pools.get(pool_name) {
            pool_ref.value().clone()
        } else {
            let config = self.config.read().await;
            let pool_size = config.pool_sizes.get(pool_name).copied().unwrap_or(100);
            let pool = Arc::new(ObjectPool::new(pool_size));
            self.pools.insert(pool_name.to_string(), pool.clone());
            pool
        }
    }

    /// Optimize memory usage
    pub async fn optimize(&self) -> crate::Result<Vec<MemoryOptimization>> {
        let mut optimizations = Vec::new();

        // Analyze usage patterns
        for entry in self.usage_history.iter() {
            let component = entry.key();
            let history = entry.value();

            if let Some(optimization) = self.analyze_component_usage(component, history).await {
                optimizations.push(optimization);
            }
        }

        // Check for memory pressure
        if self.pressure_detector.is_under_pressure().await {
            optimizations.extend(self.generate_pressure_optimizations().await);
        }

        // Sort by priority
        optimizations.sort_by(|a, b| match (&a.priority, &b.priority) {
            (OptimizationPriority::Critical, _) => std::cmp::Ordering::Less,
            (_, OptimizationPriority::Critical) => std::cmp::Ordering::Greater,
            (
                OptimizationPriority::High,
                OptimizationPriority::Low | OptimizationPriority::Medium,
            ) => std::cmp::Ordering::Less,
            (
                OptimizationPriority::Low | OptimizationPriority::Medium,
                OptimizationPriority::High,
            ) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        });

        Ok(optimizations)
    }

    /// Trigger garbage collection hint
    pub async fn gc_hint(&self) {
        let config = self.config.read().await;
        if config.enable_gc_hints {
            // In a real implementation, this would trigger GC
            // For now, we'll just log it
            tracing::debug!("Memory GC hint triggered");
        }
    }

    /// Force cleanup of unused memory
    pub async fn force_cleanup(&self) -> crate::Result<usize> {
        let mut freed_bytes = 0;

        // Clean up object pools
        for pool_entry in self.pools.iter() {
            let pool = pool_entry.value();
            freed_bytes += pool.cleanup().await;
        }

        // Trigger cleanup scheduler
        freed_bytes += self.cleanup_scheduler.force_cleanup().await?;

        // Trigger GC hint
        self.gc_hint().await;

        Ok(freed_bytes)
    }

    /// Get memory statistics
    pub async fn get_statistics(&self) -> MemoryStatistics {
        let system_info = self.get_system_memory_info().await;
        let pool_stats = self.get_pool_statistics().await;

        MemoryStatistics {
            system_memory: system_info,
            pool_statistics: pool_stats,
            component_usage: self.get_component_usage_summary().await,
            pressure_level: self.pressure_detector.get_pressure_level().await,
        }
    }

    async fn analyze_component_usage(
        &self,
        component: &str,
        history: &[MemoryUsage],
    ) -> Option<MemoryOptimization> {
        if history.is_empty() {
            return None;
        }

        let avg_usage = history.iter().map(|u| u.final_usage).sum::<usize>() / history.len();
        let _max_usage = history.iter().map(|u| u.peak_usage).max().unwrap_or(0);
        let config = self.config.read().await;

        // Check if component is using too much memory
        if avg_usage > config.max_tool_memory / 2 {
            return Some(MemoryOptimization {
                optimization_type: MemoryOptimizationType::ReduceBufferSize,
                description: format!(
                    "Component '{}' is using high average memory: {} bytes",
                    component, avg_usage
                ),
                estimated_savings: avg_usage / 4, // Estimate 25% savings
                priority: if avg_usage > config.max_tool_memory {
                    OptimizationPriority::Critical
                } else {
                    OptimizationPriority::High
                },
                component: component.to_string(),
            });
        }

        // Check for memory leaks (consistently increasing usage)
        if history.len() >= 10 {
            let recent_avg = history[history.len() - 5..]
                .iter()
                .map(|u| u.final_usage)
                .sum::<usize>()
                / 5;
            let older_avg = history[history.len() - 10..history.len() - 5]
                .iter()
                .map(|u| u.final_usage)
                .sum::<usize>()
                / 5;

            if recent_avg > older_avg * 2 {
                return Some(MemoryOptimization {
                    optimization_type: MemoryOptimizationType::IncreaseCleanupFrequency,
                    description: format!(
                        "Potential memory leak detected in component '{}'",
                        component
                    ),
                    estimated_savings: recent_avg - older_avg,
                    priority: OptimizationPriority::High,
                    component: component.to_string(),
                });
            }
        }

        None
    }

    async fn generate_pressure_optimizations(&self) -> Vec<MemoryOptimization> {
        vec![
            MemoryOptimization {
                optimization_type: MemoryOptimizationType::IncreaseCleanupFrequency,
                description: "System under memory pressure - increase cleanup frequency"
                    .to_string(),
                estimated_savings: 0, // Unknown
                priority: OptimizationPriority::Critical,
                component: "system".to_string(),
            },
            MemoryOptimization {
                optimization_type: MemoryOptimizationType::ReduceCacheSize,
                description: "Reduce cache sizes to free memory".to_string(),
                estimated_savings: 0, // Unknown
                priority: OptimizationPriority::High,
                component: "cache".to_string(),
            },
        ]
    }

    async fn get_system_memory_info(&self) -> SystemMemoryInfo {
        // In a real implementation, this would query actual system memory
        // For now, we'll return mock data
        SystemMemoryInfo {
            total_bytes: 8 * 1024 * 1024 * 1024,     // 8GB
            used_bytes: 4 * 1024 * 1024 * 1024,      // 4GB
            available_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            peak_bytes: 6 * 1024 * 1024 * 1024,      // 6GB
            allocated_objects: 10000,
        }
    }

    async fn get_pool_statistics(&self) -> HashMap<String, PoolStatistics> {
        let mut stats = HashMap::new();

        for entry in self.pools.iter() {
            let pool_name = entry.key();
            let pool = entry.value();
            stats.insert(pool_name.clone(), pool.get_statistics().await);
        }

        stats
    }

    async fn get_component_usage_summary(&self) -> HashMap<String, ComponentUsageSummary> {
        let mut summary = HashMap::new();

        for entry in self.usage_history.iter() {
            let component = entry.key();
            let history = entry.value();

            if !history.is_empty() {
                let total_usage = history.iter().map(|u| u.final_usage).sum::<usize>();
                let avg_usage = total_usage / history.len();
                let max_usage = history.iter().map(|u| u.peak_usage).max().unwrap_or(0);
                let min_usage = history.iter().map(|u| u.final_usage).min().unwrap_or(0);

                summary.insert(
                    component.clone(),
                    ComponentUsageSummary {
                        average_usage: avg_usage,
                        peak_usage: max_usage,
                        minimum_usage: min_usage,
                        total_allocations: history.len(),
                    },
                );
            }
        }

        summary
    }
}

/// Object pool for memory optimization
pub struct ObjectPool {
    capacity: usize,
    objects: Arc<RwLock<Vec<Box<dyn std::any::Any + Send + Sync>>>>,
    statistics: Arc<RwLock<PoolStatistics>>,
}

impl ObjectPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            objects: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
            statistics: Arc::new(RwLock::new(PoolStatistics::default())),
        }
    }

    pub async fn get<T>(&self) -> Option<Box<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let mut objects = self.objects.write().await;
        let mut stats = self.statistics.write().await;

        if let Some(obj) = objects.pop() {
            if let Ok(typed_obj) = obj.downcast::<T>() {
                stats.hits += 1;
                return Some(typed_obj);
            }
        }

        stats.misses += 1;
        None
    }

    pub async fn put<T>(&self, obj: Box<T>)
    where
        T: Send + Sync + 'static,
    {
        let mut objects = self.objects.write().await;

        if objects.len() < self.capacity {
            objects.push(obj);
        }
    }

    pub async fn cleanup(&self) -> usize {
        let mut objects = self.objects.write().await;
        let freed_count = objects.len();
        objects.clear();

        // Estimate freed bytes (rough approximation)
        freed_count * std::mem::size_of::<Box<dyn std::any::Any>>()
    }

    pub async fn get_statistics(&self) -> PoolStatistics {
        self.statistics.read().await.clone()
    }
}

/// Memory pressure detector
pub struct MemoryPressureDetector {
    config: Arc<RwLock<MemoryConfig>>,
    pressure_level: Arc<RwLock<PressureLevel>>,
}

impl MemoryPressureDetector {
    pub fn new(config: Arc<RwLock<MemoryConfig>>) -> Self {
        Self {
            config,
            pressure_level: Arc::new(RwLock::new(PressureLevel::Normal)),
        }
    }

    pub async fn check_pressure(&self, usage: &MemoryUsage) {
        let config = self.config.read().await;
        let pressure_ratio = usage.final_usage as f64 / config.max_workflow_memory as f64;

        let new_level = if pressure_ratio > config.pressure_threshold {
            PressureLevel::Critical
        } else if pressure_ratio > config.cleanup_threshold {
            PressureLevel::High
        } else if pressure_ratio > 0.5 {
            PressureLevel::Medium
        } else {
            PressureLevel::Normal
        };

        let mut current_level = self.pressure_level.write().await;
        if new_level != *current_level {
            tracing::info!(
                "Memory pressure level changed: {:?} -> {:?}",
                *current_level,
                new_level
            );
            *current_level = new_level;
        }
    }

    pub async fn is_under_pressure(&self) -> bool {
        matches!(
            *self.pressure_level.read().await,
            PressureLevel::High | PressureLevel::Critical
        )
    }

    pub async fn get_pressure_level(&self) -> PressureLevel {
        *self.pressure_level.read().await
    }
}

/// Cleanup scheduler for automatic memory management
pub struct CleanupScheduler {
    #[allow(dead_code)]
    config: Arc<RwLock<MemoryConfig>>,
    usage_history: Arc<DashMap<String, Vec<MemoryUsage>>>,
}

impl CleanupScheduler {
    pub fn new(
        config: Arc<RwLock<MemoryConfig>>,
        usage_history: Arc<DashMap<String, Vec<MemoryUsage>>>,
    ) -> Self {
        Self {
            config,
            usage_history,
        }
    }

    pub async fn force_cleanup(&self) -> crate::Result<usize> {
        let mut freed_bytes = 0;

        // Clean up old usage history
        for mut entry in self.usage_history.iter_mut() {
            let history = entry.value_mut();
            if history.len() > 100 {
                let to_remove = history.len() - 100;
                history.drain(0..to_remove);
                freed_bytes += to_remove * std::mem::size_of::<MemoryUsage>();
            }
        }

        Ok(freed_bytes)
    }
}

// Supporting types and structures

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureLevel {
    Normal,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMemoryInfo {
    pub total_bytes: usize,
    pub used_bytes: usize,
    pub available_bytes: usize,
    pub peak_bytes: usize,
    pub allocated_objects: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolStatistics {
    pub hits: u64,
    pub misses: u64,
    pub current_size: usize,
    pub max_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUsageSummary {
    pub average_usage: usize,
    pub peak_usage: usize,
    pub minimum_usage: usize,
    pub total_allocations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub system_memory: SystemMemoryInfo,
    pub pool_statistics: HashMap<String, PoolStatistics>,
    pub component_usage: HashMap<String, ComponentUsageSummary>,
    pub pressure_level: PressureLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let config = MemoryConfig::default();
        let manager = MemoryManager::new(config);

        let snapshot = manager.take_snapshot().await;
        assert!(snapshot.total_bytes > 0);
    }

    #[tokio::test]
    async fn test_memory_usage_recording() {
        let config = MemoryConfig::default();
        let manager = MemoryManager::new(config);

        let usage = MemoryUsage {
            initial: 1000,
            final_usage: 1500,
            peak_usage: 2000,
            allocated: 500,
        };

        manager.record_usage("test_component", usage).await;

        let history = manager.usage_history.get("test_component").unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].final_usage, 1500);
    }

    #[tokio::test]
    async fn test_object_pool() {
        let pool = ObjectPool::new(10);

        // Test putting and getting objects
        let test_obj = Box::new(42i32);
        pool.put(test_obj).await;

        let retrieved: Option<Box<i32>> = pool.get().await;
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_pressure_detection() {
        let config = Arc::new(RwLock::new(MemoryConfig::default()));
        let detector = MemoryPressureDetector::new(config);

        // Test normal pressure
        let normal_usage = MemoryUsage {
            initial: 1000,
            final_usage: 100_000_000, // 100MB
            peak_usage: 120_000_000,
            allocated: 99_000_000,
        };

        detector.check_pressure(&normal_usage).await;
        assert_eq!(detector.get_pressure_level().await, PressureLevel::Normal);

        // Test high pressure
        let high_usage = MemoryUsage {
            initial: 1000,
            final_usage: 800_000_000, // 800MB (close to 1GB limit)
            peak_usage: 900_000_000,
            allocated: 799_000_000,
        };

        detector.check_pressure(&high_usage).await;
        assert_eq!(detector.get_pressure_level().await, PressureLevel::High);
    }
}
