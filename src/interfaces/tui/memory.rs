//! TUI Memory and Resource Management Module
//!
//! This module provides memory usage monitoring, resource cleanup, and leak detection
//! specifically for the TUI interface components.

use crate::error::Result;
use crate::performance::memory::{
    MemoryManager, MemoryStatistics, MemoryUsage,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// TUI-specific memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiMemoryConfig {
    /// Maximum memory usage for TUI application (in bytes)
    pub max_tui_memory: usize,

    /// Maximum memory per widget (in bytes)
    pub max_widget_memory: usize,

    /// Memory monitoring interval for TUI
    pub monitoring_interval: Duration,

    /// Enable widget memory tracking
    pub track_widget_memory: bool,

    /// Enable render buffer pooling
    pub enable_buffer_pooling: bool,

    /// Maximum number of cached render buffers
    pub max_cached_buffers: usize,

    /// Buffer cache TTL
    pub buffer_cache_ttl: Duration,

    /// Enable memory leak detection
    pub enable_leak_detection: bool,

    /// Leak detection threshold (growth rate)
    pub leak_threshold: f64,

    /// Enable automatic cleanup
    pub enable_auto_cleanup: bool,

    /// Cleanup trigger threshold (percentage of max memory)
    pub cleanup_threshold: f64,

    /// Enable memory pressure warnings
    pub enable_pressure_warnings: bool,

    /// Memory pressure warning threshold
    pub pressure_warning_threshold: f64,
}

impl Default for TuiMemoryConfig {
    fn default() -> Self {
        Self {
            max_tui_memory: 50 * 1024 * 1024,   // 50MB as per requirement 12.3
            max_widget_memory: 5 * 1024 * 1024, // 5MB per widget
            monitoring_interval: Duration::from_secs(5),
            track_widget_memory: true,
            enable_buffer_pooling: true,
            max_cached_buffers: 100,
            buffer_cache_ttl: Duration::from_secs(30),
            enable_leak_detection: true,
            leak_threshold: 1.5, // 50% growth indicates potential leak
            enable_auto_cleanup: true,
            cleanup_threshold: 0.8, // 80% of max memory
            enable_pressure_warnings: true,
            pressure_warning_threshold: 0.9, // 90% of max memory
        }
    }
}

/// Widget memory usage tracking
#[derive(Debug, Clone)]
pub struct WidgetMemoryUsage {
    pub widget_id: String,
    pub widget_type: String,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: u64,
    pub last_updated: Instant,
    pub render_buffer_size: usize,
    pub cached_data_size: usize,
}

/// Memory leak detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLeakDetection {
    pub widget_id: String,
    pub leak_detected: bool,
    pub growth_rate: f64,
    pub baseline_usage: usize,
    pub current_usage: usize,
    pub detection_confidence: f64,
    pub recommended_action: LeakAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeakAction {
    Monitor,
    ClearCache,
    RestartWidget,
    ReduceBufferSize,
    ForceCleanup,
}

/// Resource cleanup report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupReport {
    pub freed_memory: usize,
    pub cleaned_widgets: Vec<String>,
    pub cleared_buffers: usize,
    pub cleanup_duration: Duration,
    pub success: bool,
    pub errors: Vec<String>,
}

/// TUI Memory and Resource Manager
pub struct TuiMemoryManager {
    config: Arc<RwLock<TuiMemoryConfig>>,
    memory_manager: Arc<MemoryManager>,

    // Widget memory tracking
    widget_usage: Arc<DashMap<String, WidgetMemoryUsage>>,
    #[allow(dead_code)]
    usage_history: Arc<DashMap<String, Vec<MemoryUsage>>>,

    // Render buffer management
    buffer_pool: Arc<RwLock<Vec<ratatui::buffer::Buffer>>>,
    buffer_cache: Arc<DashMap<String, (ratatui::buffer::Buffer, Instant)>>,

    // Leak detection
    leak_detector: Arc<MemoryLeakDetector>,

    // Resource cleanup
    cleanup_scheduler: Arc<TuiCleanupScheduler>,

    // Statistics
    total_allocations: Arc<RwLock<u64>>,
    total_deallocations: Arc<RwLock<u64>>,
    peak_memory_usage: Arc<RwLock<usize>>,
    current_memory_usage: Arc<RwLock<usize>>,
}

impl TuiMemoryManager {
    /// Create a new TUI memory manager
    pub fn new(config: TuiMemoryConfig, memory_manager: Arc<MemoryManager>) -> Self {
        let config = Arc::new(RwLock::new(config));
        let widget_usage = Arc::new(DashMap::new());
        let usage_history = Arc::new(DashMap::new());
        let buffer_pool = Arc::new(RwLock::new(Vec::new()));
        let buffer_cache = Arc::new(DashMap::new());

        let leak_detector = Arc::new(MemoryLeakDetector::new(
            config.clone(),
            widget_usage.clone(),
        ));

        let cleanup_scheduler = Arc::new(TuiCleanupScheduler::new(
            config.clone(),
            widget_usage.clone(),
            buffer_cache.clone(),
        ));

        Self {
            config,
            memory_manager,
            widget_usage,
            usage_history,
            buffer_pool,
            buffer_cache,
            leak_detector,
            cleanup_scheduler,
            total_allocations: Arc::new(RwLock::new(0)),
            total_deallocations: Arc::new(RwLock::new(0)),
            peak_memory_usage: Arc::new(RwLock::new(0)),
            current_memory_usage: Arc::new(RwLock::new(0)),
        }
    }

    /// Start memory monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let config = self.config.read().await;
        if !config.track_widget_memory {
            return Ok(());
        }

        // Start leak detection
        self.leak_detector.start_monitoring().await?;

        // Start cleanup scheduler
        self.cleanup_scheduler.start().await?;

        tracing::info!("TUI memory monitoring started");
        Ok(())
    }

    /// Stop memory monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        self.leak_detector.stop_monitoring().await?;
        self.cleanup_scheduler.stop().await?;

        tracing::info!("TUI memory monitoring stopped");
        Ok(())
    }

    /// Track widget memory allocation
    pub async fn track_widget_allocation(
        &self,
        widget_id: &str,
        widget_type: &str,
        size: usize,
    ) -> Result<()> {
        let config = self.config.read().await;
        if !config.track_widget_memory {
            return Ok(());
        }

        // Update current memory usage
        let mut current_usage = self.current_memory_usage.write().await;
        *current_usage += size;

        // Update peak usage if necessary
        let mut peak_usage = self.peak_memory_usage.write().await;
        if *current_usage > *peak_usage {
            *peak_usage = *current_usage;
        }

        // Update widget-specific usage
        let mut widget_usage = self
            .widget_usage
            .entry(widget_id.to_string())
            .or_insert_with(|| WidgetMemoryUsage {
                widget_id: widget_id.to_string(),
                widget_type: widget_type.to_string(),
                current_usage: 0,
                peak_usage: 0,
                allocation_count: 0,
                last_updated: Instant::now(),
                render_buffer_size: 0,
                cached_data_size: 0,
            });

        widget_usage.current_usage += size;
        widget_usage.allocation_count += 1;
        widget_usage.last_updated = Instant::now();

        if widget_usage.current_usage > widget_usage.peak_usage {
            widget_usage.peak_usage = widget_usage.current_usage;
        }

        // Update total allocations
        let mut total_allocations = self.total_allocations.write().await;
        *total_allocations += 1;

        // Check for memory pressure
        self.check_memory_pressure().await?;

        // Record usage in memory manager
        let usage = MemoryUsage {
            initial: widget_usage.current_usage.saturating_sub(size),
            final_usage: widget_usage.current_usage,
            peak_usage: widget_usage.peak_usage,
            allocated: size,
        };

        self.memory_manager
            .record_usage(&format!("tui_widget_{}", widget_id), usage)
            .await;

        Ok(())
    }

    /// Track widget memory deallocation
    pub async fn track_widget_deallocation(&self, widget_id: &str, size: usize) -> Result<()> {
        let config = self.config.read().await;
        if !config.track_widget_memory {
            return Ok(());
        }

        // Update current memory usage
        let mut current_usage = self.current_memory_usage.write().await;
        *current_usage = current_usage.saturating_sub(size);

        // Update widget-specific usage
        if let Some(mut widget_usage) = self.widget_usage.get_mut(widget_id) {
            widget_usage.current_usage = widget_usage.current_usage.saturating_sub(size);
            widget_usage.last_updated = Instant::now();
        }

        // Update total deallocations
        let mut total_deallocations = self.total_deallocations.write().await;
        *total_deallocations += 1;

        Ok(())
    }

    /// Get or create a render buffer from the pool
    pub async fn get_render_buffer(
        &self,
        width: u16,
        height: u16,
    ) -> Result<ratatui::buffer::Buffer> {
        let config = self.config.read().await;
        if !config.enable_buffer_pooling {
            return Ok(ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(
                0, 0, width, height,
            )));
        }

        let mut pool = self.buffer_pool.write().await;

        // Try to find a suitable buffer in the pool
        for (i, buffer) in pool.iter().enumerate() {
            if buffer.area().width >= width && buffer.area().height >= height {
                let mut buffer = pool.remove(i);
                buffer.reset();
                return Ok(buffer);
            }
        }

        // No suitable buffer found, create a new one
        Ok(ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(
            0, 0, width, height,
        )))
    }

    /// Return a render buffer to the pool
    pub async fn return_render_buffer(&self, buffer: ratatui::buffer::Buffer) -> Result<()> {
        let config = self.config.read().await;
        if !config.enable_buffer_pooling {
            return Ok(());
        }

        let mut pool = self.buffer_pool.write().await;

        // Only keep the buffer if the pool isn't full
        if pool.len() < config.max_cached_buffers {
            pool.push(buffer);
        }

        Ok(())
    }

    /// Cache rendered content
    pub async fn cache_render_buffer(
        &self,
        key: &str,
        buffer: ratatui::buffer::Buffer,
    ) -> Result<()> {
        let config = self.config.read().await;
        if !config.enable_buffer_pooling {
            return Ok(());
        }

        // Check cache size limit
        if self.buffer_cache.len() >= config.max_cached_buffers {
            self.cleanup_buffer_cache().await?;
        }

        self.buffer_cache
            .insert(key.to_string(), (buffer, Instant::now()));
        Ok(())
    }

    /// Get cached render buffer
    pub async fn get_cached_render_buffer(&self, key: &str) -> Option<ratatui::buffer::Buffer> {
        let config = self.config.read().await;
        if !config.enable_buffer_pooling {
            return None;
        }

        if let Some(entry) = self.buffer_cache.get(key) {
            let (buffer, cached_at) = &*entry;
            if cached_at.elapsed() <= config.buffer_cache_ttl {
                return Some(buffer.clone());
            } else {
                // Cache entry expired
                drop(entry);
                self.buffer_cache.remove(key);
            }
        }

        None
    }

    /// Detect memory leaks in widgets
    pub async fn detect_memory_leaks(&self) -> Result<Vec<MemoryLeakDetection>> {
        self.leak_detector.detect_leaks().await
    }

    /// Force cleanup of resources
    pub async fn force_cleanup(&self) -> Result<CleanupReport> {
        let start_time = Instant::now();
        let mut report = CleanupReport {
            freed_memory: 0,
            cleaned_widgets: Vec::new(),
            cleared_buffers: 0,
            cleanup_duration: Duration::ZERO,
            success: true,
            errors: Vec::new(),
        };

        // Cleanup buffer cache
        match self.cleanup_buffer_cache().await {
            Ok(freed_buffers) => {
                report.cleared_buffers = freed_buffers;
                report.freed_memory +=
                    freed_buffers * std::mem::size_of::<ratatui::buffer::Buffer>();
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Buffer cache cleanup failed: {}", e));
                report.success = false;
            }
        }

        // Cleanup buffer pool
        match self.cleanup_buffer_pool().await {
            Ok(freed_buffers) => {
                report.cleared_buffers += freed_buffers;
                report.freed_memory +=
                    freed_buffers * std::mem::size_of::<ratatui::buffer::Buffer>();
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Buffer pool cleanup failed: {}", e));
                report.success = false;
            }
        }

        // Cleanup widget memory tracking data
        match self.cleanup_widget_tracking().await {
            Ok(cleaned_widgets) => {
                report.cleaned_widgets = cleaned_widgets;
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Widget tracking cleanup failed: {}", e));
                report.success = false;
            }
        }

        // Force cleanup in underlying memory manager
        match self.memory_manager.force_cleanup().await {
            Ok(freed_bytes) => {
                report.freed_memory += freed_bytes;
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Memory manager cleanup failed: {}", e));
                report.success = false;
            }
        }

        report.cleanup_duration = start_time.elapsed();

        tracing::info!(
            "TUI memory cleanup completed: freed {} bytes, {} buffers, {} widgets in {:?}",
            report.freed_memory,
            report.cleared_buffers,
            report.cleaned_widgets.len(),
            report.cleanup_duration
        );

        Ok(report)
    }

    /// Get comprehensive memory statistics
    pub async fn get_memory_statistics(&self) -> Result<TuiMemoryStatistics> {
        let base_stats = self.memory_manager.get_statistics().await;
        let current_usage = *self.current_memory_usage.read().await;
        let peak_usage = *self.peak_memory_usage.read().await;
        let total_allocations = *self.total_allocations.read().await;
        let total_deallocations = *self.total_deallocations.read().await;

        let widget_stats: HashMap<String, WidgetMemoryUsage> = self
            .widget_usage
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let buffer_pool_size = self.buffer_pool.read().await.len();
        let buffer_cache_size = self.buffer_cache.len();

        Ok(TuiMemoryStatistics {
            base_statistics: base_stats,
            current_usage,
            peak_usage,
            total_allocations,
            total_deallocations,
            widget_statistics: widget_stats,
            buffer_pool_size,
            buffer_cache_size,
            memory_efficiency: if total_allocations > 0 {
                total_deallocations as f64 / total_allocations as f64
            } else {
                1.0
            },
        })
    }

    /// Check for memory pressure and trigger warnings/cleanup
    async fn check_memory_pressure(&self) -> Result<()> {
        let config = self.config.read().await;
        let current_usage = *self.current_memory_usage.read().await;
        let usage_ratio = current_usage as f64 / config.max_tui_memory as f64;

        if usage_ratio >= config.pressure_warning_threshold && config.enable_pressure_warnings {
            tracing::warn!(
                "TUI memory pressure detected: {:.1}% ({} / {} bytes)",
                usage_ratio * 100.0,
                current_usage,
                config.max_tui_memory
            );
        }

        if usage_ratio >= config.cleanup_threshold && config.enable_auto_cleanup {
            tracing::info!("Triggering automatic cleanup due to memory pressure");
            let _ = self.force_cleanup().await;
        }

        Ok(())
    }

    /// Cleanup buffer cache
    async fn cleanup_buffer_cache(&self) -> Result<usize> {
        let config = self.config.read().await;
        let mut freed_count = 0;

        // Remove expired entries
        let expired_keys: Vec<String> = self
            .buffer_cache
            .iter()
            .filter_map(|entry| {
                let (_, cached_at) = entry.value();
                if cached_at.elapsed() > config.buffer_cache_ttl {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for key in expired_keys {
            self.buffer_cache.remove(&key);
            freed_count += 1;
        }

        // If still over limit, remove oldest entries
        if self.buffer_cache.len() > config.max_cached_buffers {
            let mut entries: Vec<(String, Instant)> = self
                .buffer_cache
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().1))
                .collect();

            entries.sort_by_key(|(_, cached_at)| *cached_at);

            let to_remove = self.buffer_cache.len() - config.max_cached_buffers;
            for (key, _) in entries.into_iter().take(to_remove) {
                self.buffer_cache.remove(&key);
                freed_count += 1;
            }
        }

        Ok(freed_count)
    }

    /// Cleanup buffer pool
    async fn cleanup_buffer_pool(&self) -> Result<usize> {
        let mut pool = self.buffer_pool.write().await;
        let freed_count = pool.len();
        pool.clear();
        Ok(freed_count)
    }

    /// Cleanup widget memory tracking data
    async fn cleanup_widget_tracking(&self) -> Result<Vec<String>> {
        let mut cleaned_widgets = Vec::new();

        // Remove widgets that haven't been updated recently
        let cutoff_time = Instant::now() - Duration::from_secs(300); // 5 minutes

        let stale_widgets: Vec<String> = self
            .widget_usage
            .iter()
            .filter_map(|entry| {
                if entry.value().last_updated < cutoff_time {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for widget_id in stale_widgets {
            self.widget_usage.remove(&widget_id);
            self.usage_history.remove(&widget_id);
            cleaned_widgets.push(widget_id);
        }

        Ok(cleaned_widgets)
    }
}

/// Memory leak detector for TUI widgets
pub struct MemoryLeakDetector {
    config: Arc<RwLock<TuiMemoryConfig>>,
    #[allow(dead_code)]
    widget_usage: Arc<DashMap<String, WidgetMemoryUsage>>,
    baseline_usage: Arc<DashMap<String, usize>>,
    monitoring_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl MemoryLeakDetector {
    pub fn new(
        config: Arc<RwLock<TuiMemoryConfig>>,
        widget_usage: Arc<DashMap<String, WidgetMemoryUsage>>,
    ) -> Self {
        Self {
            config,
            widget_usage,
            baseline_usage: Arc::new(DashMap::new()),
            monitoring_task: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        let config = self.config.read().await;
        if !config.enable_leak_detection {
            return Ok(());
        }

        let config_clone = Arc::clone(&self.config);
        let widget_usage_clone = Arc::clone(&self.widget_usage);
        let baseline_usage_clone = Arc::clone(&self.baseline_usage);

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;

                let config = config_clone.read().await;
                if !config.enable_leak_detection {
                    break;
                }

                // Update baselines and check for leaks
                for entry in widget_usage_clone.iter() {
                    let widget_id = entry.key();
                    let current_usage = entry.value().current_usage;

                    if let Some(mut baseline) = baseline_usage_clone.get_mut(widget_id) {
                        let growth_rate = current_usage as f64 / *baseline as f64;

                        if growth_rate > config.leak_threshold {
                            tracing::warn!(
                                "Potential memory leak detected in widget '{}': {:.2}x growth ({} -> {} bytes)",
                                widget_id,
                                growth_rate,
                                *baseline,
                                current_usage
                            );
                        }

                        // Update baseline (slowly to avoid false positives)
                        *baseline = (*baseline + current_usage) / 2;
                    } else {
                        baseline_usage_clone.insert(widget_id.clone(), current_usage);
                    }
                }
            }
        });

        let mut monitoring_task = self.monitoring_task.write().await;
        *monitoring_task = Some(task);

        Ok(())
    }

    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut monitoring_task = self.monitoring_task.write().await;
        if let Some(task) = monitoring_task.take() {
            task.abort();
        }
        Ok(())
    }

    pub async fn detect_leaks(&self) -> Result<Vec<MemoryLeakDetection>> {
        let config = self.config.read().await;
        let mut leaks = Vec::new();

        for entry in self.widget_usage.iter() {
            let widget_id = entry.key();
            let usage = entry.value();

            if let Some(baseline) = self.baseline_usage.get(widget_id) {
                let growth_rate = usage.current_usage as f64 / *baseline as f64;
                let leak_detected = growth_rate > config.leak_threshold;

                let recommended_action = if growth_rate > 3.0 {
                    LeakAction::RestartWidget
                } else if growth_rate > 2.0 {
                    LeakAction::ForceCleanup
                } else if growth_rate > 1.5 {
                    LeakAction::ClearCache
                } else {
                    LeakAction::Monitor
                };

                let confidence = if leak_detected {
                    ((growth_rate - config.leak_threshold) / config.leak_threshold).min(1.0)
                } else {
                    0.0
                };

                leaks.push(MemoryLeakDetection {
                    widget_id: widget_id.clone(),
                    leak_detected,
                    growth_rate,
                    baseline_usage: *baseline,
                    current_usage: usage.current_usage,
                    detection_confidence: confidence,
                    recommended_action,
                });
            }
        }

        Ok(leaks)
    }
}

/// TUI-specific cleanup scheduler
pub struct TuiCleanupScheduler {
    config: Arc<RwLock<TuiMemoryConfig>>,
    #[allow(dead_code)]
    widget_usage: Arc<DashMap<String, WidgetMemoryUsage>>,
    buffer_cache: Arc<DashMap<String, (ratatui::buffer::Buffer, Instant)>>,
    cleanup_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl TuiCleanupScheduler {
    pub fn new(
        config: Arc<RwLock<TuiMemoryConfig>>,
        widget_usage: Arc<DashMap<String, WidgetMemoryUsage>>,
        buffer_cache: Arc<DashMap<String, (ratatui::buffer::Buffer, Instant)>>,
    ) -> Self {
        Self {
            config,
            widget_usage,
            buffer_cache,
            cleanup_task: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let config = self.config.read().await;
        if !config.enable_auto_cleanup {
            return Ok(());
        }

        let config_clone = Arc::clone(&self.config);
        let buffer_cache_clone = Arc::clone(&self.buffer_cache);

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Cleanup every 30 seconds

            loop {
                interval.tick().await;

                let config = config_clone.read().await;
                if !config.enable_auto_cleanup {
                    break;
                }

                // Cleanup expired buffer cache entries
                let expired_keys: Vec<String> = buffer_cache_clone
                    .iter()
                    .filter_map(|entry| {
                        let (_, cached_at) = entry.value();
                        if cached_at.elapsed() > config.buffer_cache_ttl {
                            Some(entry.key().clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                for key in expired_keys {
                    buffer_cache_clone.remove(&key);
                }
            }
        });

        let mut cleanup_task = self.cleanup_task.write().await;
        *cleanup_task = Some(task);

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut cleanup_task = self.cleanup_task.write().await;
        if let Some(task) = cleanup_task.take() {
            task.abort();
        }
        Ok(())
    }
}

/// Comprehensive TUI memory statistics
#[derive(Debug, Clone)]
pub struct TuiMemoryStatistics {
    pub base_statistics: MemoryStatistics,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub widget_statistics: HashMap<String, WidgetMemoryUsage>,
    pub buffer_pool_size: usize,
    pub buffer_cache_size: usize,
    pub memory_efficiency: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::PerformanceManager;

    #[tokio::test]
    async fn test_tui_memory_manager_creation() {
        let config = TuiMemoryConfig::default();
        let perf_config = crate::performance::PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let memory_config = MemoryConfig::default();
        let memory_manager = Arc::new(MemoryManager::new(memory_config));

        let tui_manager = TuiMemoryManager::new(config, memory_manager);

        let stats = tui_manager.get_memory_statistics().await.unwrap();
        assert_eq!(stats.current_usage, 0);
        assert_eq!(stats.total_allocations, 0);
    }

    #[tokio::test]
    async fn test_widget_memory_tracking() {
        let config = TuiMemoryConfig::default();
        let perf_config = crate::performance::PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let memory_config = MemoryConfig::default();
        let memory_manager = Arc::new(MemoryManager::new(memory_config));

        let tui_manager = TuiMemoryManager::new(config, memory_manager);

        // Track allocation
        tui_manager
            .track_widget_allocation("test_widget", "TestWidget", 1024)
            .await
            .unwrap();

        let stats = tui_manager.get_memory_statistics().await.unwrap();
        assert_eq!(stats.current_usage, 1024);
        assert_eq!(stats.total_allocations, 1);
        assert!(stats.widget_statistics.contains_key("test_widget"));

        // Track deallocation
        tui_manager
            .track_widget_deallocation("test_widget", 512)
            .await
            .unwrap();

        let stats = tui_manager.get_memory_statistics().await.unwrap();
        assert_eq!(stats.current_usage, 512);
        assert_eq!(stats.total_deallocations, 1);
    }

    #[tokio::test]
    async fn test_buffer_pooling() {
        let config = TuiMemoryConfig::default();
        let perf_config = crate::performance::PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let memory_config = MemoryConfig::default();
        let memory_manager = Arc::new(MemoryManager::new(memory_config));

        let tui_manager = TuiMemoryManager::new(config, memory_manager);

        // Get a buffer
        let buffer = tui_manager.get_render_buffer(80, 24).await.unwrap();
        assert_eq!(buffer.area().width, 80);
        assert_eq!(buffer.area().height, 24);

        // Return the buffer
        tui_manager.return_render_buffer(buffer).await.unwrap();

        // Get another buffer (should reuse from pool)
        let buffer2 = tui_manager.get_render_buffer(70, 20).await.unwrap();
        assert!(buffer2.area().width >= 70);
        assert!(buffer2.area().height >= 20);
    }

    #[tokio::test]
    async fn test_memory_leak_detection() {
        let config = TuiMemoryConfig::default();
        let perf_config = crate::performance::PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let memory_config = MemoryConfig::default();
        let memory_manager = Arc::new(MemoryManager::new(memory_config));

        let tui_manager = TuiMemoryManager::new(config, memory_manager);

        // Simulate normal usage
        tui_manager
            .track_widget_allocation("normal_widget", "NormalWidget", 1000)
            .await
            .unwrap();

        // Simulate potential leak
        tui_manager
            .track_widget_allocation("leaky_widget", "LeakyWidget", 1000)
            .await
            .unwrap();

        // Set baseline
        tui_manager
            .leak_detector
            .baseline_usage
            .insert("leaky_widget".to_string(), 1000);

        // Simulate memory growth
        tui_manager
            .track_widget_allocation("leaky_widget", "LeakyWidget", 2000)
            .await
            .unwrap();

        let leaks = tui_manager.detect_memory_leaks().await.unwrap();

        // Should detect the leak in leaky_widget
        let leaky_detection = leaks.iter().find(|l| l.widget_id == "leaky_widget");
        assert!(leaky_detection.is_some());

        let leak = leaky_detection.unwrap();
        assert!(leak.leak_detected);
        assert!(leak.growth_rate > 1.5);
    }

    #[tokio::test]
    async fn test_force_cleanup() {
        let config = TuiMemoryConfig::default();
        let perf_config = crate::performance::PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let memory_config = MemoryConfig::default();
        let memory_manager = Arc::new(MemoryManager::new(memory_config));

        let tui_manager = TuiMemoryManager::new(config, memory_manager);

        // Add some data to clean up
        tui_manager
            .track_widget_allocation("test_widget", "TestWidget", 1024)
            .await
            .unwrap();
        let buffer = tui_manager.get_render_buffer(80, 24).await.unwrap();
        tui_manager
            .cache_render_buffer("test_cache", buffer)
            .await
            .unwrap();

        let report = tui_manager.force_cleanup().await.unwrap();

        assert!(report.success);
        assert!(report.freed_memory > 0 || report.cleared_buffers > 0);
        assert!(report.cleanup_duration > Duration::ZERO);
    }
}
