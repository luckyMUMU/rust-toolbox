//! TUI Performance Optimization Module
//!
//! This module provides performance optimization features for the TUI interface,
//! including rendering optimization, frame rate monitoring, and caching.

use crate::error::Result;
use crate::performance::{PerformanceManager, PerformanceMonitor};
use dashmap::DashMap;
use ratatui::{buffer::Buffer, layout::Rect, Frame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rendering performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    /// Target frame rate (FPS)
    pub target_fps: u32,

    /// Enable frame rate monitoring
    pub monitor_fps: bool,

    /// Enable rendering cache
    pub enable_cache: bool,

    /// Cache TTL for rendered content
    pub cache_ttl: Duration,

    /// Maximum cache entries
    pub max_cache_entries: usize,

    /// Enable incremental rendering
    pub incremental_rendering: bool,

    /// Dirty region tracking
    pub dirty_region_tracking: bool,

    /// Enable render batching
    pub enable_batching: bool,

    /// Batch size for rendering operations
    pub batch_size: usize,

    /// Enable async rendering
    pub async_rendering: bool,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            monitor_fps: true,
            enable_cache: true,
            cache_ttl: Duration::from_secs(1),
            max_cache_entries: 1000,
            incremental_rendering: true,
            dirty_region_tracking: true,
            enable_batching: true,
            batch_size: 10,
            async_rendering: false, // Keep false for terminal compatibility
        }
    }
}

/// Frame rate statistics
#[derive(Debug, Clone)]
pub struct FrameRateStats {
    pub current_fps: f64,
    pub average_fps: f64,
    pub min_fps: f64,
    pub max_fps: f64,
    pub frame_count: u64,
    pub dropped_frames: u64,
    pub render_time_ms: f64,
    pub last_update: Instant,
}

impl Default for FrameRateStats {
    fn default() -> Self {
        Self {
            current_fps: 0.0,
            average_fps: 0.0,
            min_fps: f64::MAX,
            max_fps: 0.0,
            frame_count: 0,
            dropped_frames: 0,
            render_time_ms: 0.0,
            last_update: Instant::now(),
        }
    }
}

/// Rendering cache entry
#[derive(Debug, Clone)]
pub struct RenderCacheEntry {
    pub content: Buffer,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub area: Rect,
    pub hash: u64,
}

/// Dirty region for incremental rendering
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyRegion {
    pub area: Rect,
    pub priority: u8,
    pub widget_id: String,
}

/// Rendering performance manager
pub struct RenderingPerformanceManager {
    config: Arc<RwLock<RenderingConfig>>,
    performance_manager: Arc<PerformanceManager>,

    // Frame rate monitoring
    frame_stats: Arc<RwLock<FrameRateStats>>,
    frame_times: Arc<RwLock<Vec<Duration>>>,

    // Rendering cache
    render_cache: Arc<DashMap<String, RenderCacheEntry>>,
    cache_hits: Arc<RwLock<u64>>,
    cache_misses: Arc<RwLock<u64>>,

    // Incremental rendering
    dirty_regions: Arc<RwLock<Vec<DirtyRegion>>>,
    last_render_hash: Arc<RwLock<HashMap<String, u64>>>,

    // Render batching
    #[allow(dead_code)]
    render_queue: Arc<RwLock<Vec<RenderOperation>>>,
    #[allow(dead_code)]
    batch_processor: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Clone)]
pub struct RenderOperation {
    pub widget_id: String,
    pub area: Rect,
    pub priority: u8,
    pub timestamp: Instant,
}

impl RenderingPerformanceManager {
    /// Create a new rendering performance manager
    pub fn new(config: RenderingConfig, performance_manager: Arc<PerformanceManager>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            performance_manager,
            frame_stats: Arc::new(RwLock::new(FrameRateStats::default())),
            frame_times: Arc::new(RwLock::new(Vec::new())),
            render_cache: Arc::new(DashMap::new()),
            cache_hits: Arc::new(RwLock::new(0)),
            cache_misses: Arc::new(RwLock::new(0)),
            dirty_regions: Arc::new(RwLock::new(Vec::new())),
            last_render_hash: Arc::new(RwLock::new(HashMap::new())),
            render_queue: Arc::new(RwLock::new(Vec::new())),
            batch_processor: Arc::new(RwLock::new(None)),
        }
    }

    /// Start frame rate monitoring
    pub async fn start_frame_monitoring(&self) -> PerformanceMonitor {
        self.performance_manager
            .start_monitoring("tui_rendering")
            .await
    }

    /// Record frame rendering time
    pub async fn record_frame_time(&self, render_time: Duration) -> Result<()> {
        let config = self.config.read().await;
        if !config.monitor_fps {
            return Ok(());
        }

        let mut frame_times = self.frame_times.write().await;
        frame_times.push(render_time);

        // Keep only recent frame times (last 60 frames for 1-second window at 60fps)
        if frame_times.len() > 60 {
            let drain_count = frame_times.len() - 60;
            frame_times.drain(0..drain_count);
        }

        // Update frame statistics
        let mut stats = self.frame_stats.write().await;
        stats.frame_count += 1;
        stats.render_time_ms = render_time.as_millis() as f64;
        stats.last_update = Instant::now();

        if !frame_times.is_empty() {
            // Calculate current FPS based on recent frames
            let total_time: Duration = frame_times.iter().sum();
            if total_time > Duration::ZERO {
                stats.current_fps = frame_times.len() as f64 / total_time.as_secs_f64();
            }

            // Update min/max FPS
            if stats.min_fps == 0.0 || stats.current_fps < stats.min_fps {
                stats.min_fps = stats.current_fps;
            }
            if stats.current_fps > stats.max_fps {
                stats.max_fps = stats.current_fps;
            }

            // Calculate average FPS
            stats.average_fps = (stats.average_fps * (stats.frame_count - 1) as f64
                + stats.current_fps)
                / stats.frame_count as f64;

            // Check for dropped frames (if FPS is significantly below target)
            let target_fps = config.target_fps as f64;
            if stats.current_fps < target_fps * 0.8 {
                stats.dropped_frames += 1;
            }
        }

        Ok(())
    }

    /// Get current frame rate statistics
    pub async fn get_frame_stats(&self) -> FrameRateStats {
        self.frame_stats.read().await.clone()
    }

    /// Check if content should be cached
    pub async fn should_cache_render(&self, widget_id: &str, area: Rect) -> bool {
        let config = self.config.read().await;
        if !config.enable_cache {
            return false;
        }

        // Don't cache very small areas (not worth the overhead)
        if area.width < 10 || area.height < 3 {
            return false;
        }

        // Don't cache if cache is full
        if self.render_cache.len() >= config.max_cache_entries {
            return false;
        }

        true
    }

    /// Get cached render content
    pub async fn get_cached_render(
        &self,
        widget_id: &str,
        area: Rect,
        content_hash: u64,
    ) -> Option<Buffer> {
        let config = self.config.read().await;
        if !config.enable_cache {
            return None;
        }

        let cache_key = format!(
            "{}:{}:{}:{}:{}",
            widget_id, area.x, area.y, area.width, area.height
        );

        if let Some(mut entry) = self.render_cache.get_mut(&cache_key) {
            // Check if entry is still valid
            if entry.created_at.elapsed() > config.cache_ttl {
                drop(entry);
                self.render_cache.remove(&cache_key);
                let mut misses = self.cache_misses.write().await;
                *misses += 1;
                return None;
            }

            // Check if content has changed
            if entry.hash != content_hash {
                drop(entry);
                self.render_cache.remove(&cache_key);
                let mut misses = self.cache_misses.write().await;
                *misses += 1;
                return None;
            }

            // Update access information
            entry.last_accessed = Instant::now();
            entry.access_count += 1;

            let mut hits = self.cache_hits.write().await;
            *hits += 1;

            Some(entry.content.clone())
        } else {
            let mut misses = self.cache_misses.write().await;
            *misses += 1;
            None
        }
    }

    /// Cache rendered content
    pub async fn cache_render(
        &self,
        widget_id: &str,
        area: Rect,
        content: Buffer,
        content_hash: u64,
    ) -> Result<()> {
        let config = self.config.read().await;
        if !config.enable_cache {
            return Ok(());
        }

        let cache_key = format!(
            "{}:{}:{}:{}:{}",
            widget_id, area.x, area.y, area.width, area.height
        );

        // Check cache size limit
        if self.render_cache.len() >= config.max_cache_entries {
            self.evict_cache_entries().await;
        }

        let entry = RenderCacheEntry {
            content,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            area,
            hash: content_hash,
        };

        self.render_cache.insert(cache_key, entry);
        Ok(())
    }

    /// Evict old cache entries
    async fn evict_cache_entries(&self) {
        let config = self.config.read().await;
        let target_size = config.max_cache_entries * 3 / 4; // Remove 25% of entries

        // Collect entries with their access information
        let mut entries: Vec<(String, Instant, u64)> = self
            .render_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.last_accessed, entry.access_count))
            .collect();

        // Sort by least recently used and least frequently used
        entries.sort_by(|a, b| {
            let lru_cmp = a.1.cmp(&b.1);
            if lru_cmp == std::cmp::Ordering::Equal {
                a.2.cmp(&b.2) // LFU as tiebreaker
            } else {
                lru_cmp
            }
        });

        // Remove oldest entries
        let to_remove = self.render_cache.len().saturating_sub(target_size);
        for (key, _, _) in entries.iter().take(to_remove) {
            self.render_cache.remove(key);
        }
    }

    /// Mark region as dirty for incremental rendering
    pub async fn mark_dirty(&self, widget_id: String, area: Rect, priority: u8) -> Result<()> {
        let config = self.config.read().await;
        if !config.dirty_region_tracking {
            return Ok(());
        }

        let dirty_region = DirtyRegion {
            area,
            priority,
            widget_id,
        };

        let mut dirty_regions = self.dirty_regions.write().await;

        // Check if this region overlaps with existing dirty regions
        let mut merged = false;
        for existing in dirty_regions.iter_mut() {
            if existing.widget_id == dirty_region.widget_id
                && Self::regions_overlap(existing.area, dirty_region.area)
            {
                // Merge regions
                existing.area = Self::merge_rects(existing.area, dirty_region.area);
                existing.priority = existing.priority.max(dirty_region.priority);
                merged = true;
                break;
            }
        }

        if !merged {
            dirty_regions.push(dirty_region);
        }

        // Sort by priority (higher priority first)
        dirty_regions.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(())
    }

    /// Get dirty regions for rendering
    pub async fn get_dirty_regions(&self) -> Vec<DirtyRegion> {
        let mut dirty_regions = self.dirty_regions.write().await;
        let regions = dirty_regions.clone();
        dirty_regions.clear();
        regions
    }

    /// Check if incremental rendering should be used
    pub async fn should_use_incremental_rendering(
        &self,
        widget_id: &str,
        content_hash: u64,
    ) -> bool {
        let config = self.config.read().await;
        if !config.incremental_rendering {
            return false;
        }

        let mut last_hashes = self.last_render_hash.write().await;
        if let Some(&last_hash) = last_hashes.get(widget_id) {
            if last_hash == content_hash {
                return false; // No changes, skip rendering
            }
        }

        last_hashes.insert(widget_id.to_string(), content_hash);
        true
    }

    /// Add render operation to batch queue
    pub async fn queue_render_operation(&self, operation: RenderOperation) -> Result<()> {
        let config = self.config.read().await;
        if !config.enable_batching {
            return Ok(());
        }

        let mut queue = self.render_queue.write().await;
        queue.push(operation);

        // Process batch if it reaches the batch size
        if queue.len() >= config.batch_size {
            self.process_render_batch().await?;
        }

        Ok(())
    }

    /// Process batched render operations
    async fn process_render_batch(&self) -> Result<()> {
        let mut queue = self.render_queue.write().await;
        if queue.is_empty() {
            return Ok(());
        }

        // Sort operations by priority and timestamp
        queue.sort_by(|a, b| {
            let priority_cmp = b.priority.cmp(&a.priority);
            if priority_cmp == std::cmp::Ordering::Equal {
                a.timestamp.cmp(&b.timestamp)
            } else {
                priority_cmp
            }
        });

        // Process operations (in a real implementation, this would trigger actual rendering)
        tracing::debug!("Processing {} batched render operations", queue.len());

        queue.clear();
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (u64, u64, f64) {
        let hits = *self.cache_hits.read().await;
        let misses = *self.cache_misses.read().await;
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate)
    }

    /// Clear render cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.render_cache.clear();
        let mut hits = self.cache_hits.write().await;
        let mut misses = self.cache_misses.write().await;
        *hits = 0;
        *misses = 0;
        Ok(())
    }

    /// Optimize rendering performance
    pub async fn optimize_rendering(&self) -> Result<RenderingOptimizationReport> {
        let stats = self.get_frame_stats().await;
        let (cache_hits, cache_misses, hit_rate) = self.get_cache_stats().await;
        let config = self.config.read().await;

        let mut report = RenderingOptimizationReport {
            current_fps: stats.current_fps,
            target_fps: config.target_fps as f64,
            cache_hit_rate: hit_rate,
            recommendations: Vec::new(),
        };

        // Analyze FPS performance
        if stats.current_fps < config.target_fps as f64 * 0.8 {
            report.recommendations.push(RenderingRecommendation {
                recommendation_type: RenderingOptimizationType::ReduceRenderComplexity,
                description: format!(
                    "Current FPS ({:.1}) is below target ({}). Consider reducing render complexity.",
                    stats.current_fps, config.target_fps
                ),
                priority: OptimizationPriority::High,
                estimated_improvement: 0.3,
            });
        }

        // Analyze cache performance
        if hit_rate < 0.5 && cache_hits + cache_misses > 100 {
            report.recommendations.push(RenderingRecommendation {
                recommendation_type: RenderingOptimizationType::ImproveCaching,
                description: format!(
                    "Cache hit rate is low ({:.1}%). Consider increasing cache size or TTL.",
                    hit_rate * 100.0
                ),
                priority: OptimizationPriority::Medium,
                estimated_improvement: 0.2,
            });
        }

        // Check for excessive dropped frames
        if stats.dropped_frames > stats.frame_count / 10 {
            report.recommendations.push(RenderingRecommendation {
                recommendation_type: RenderingOptimizationType::EnableBatching,
                description: format!(
                    "High dropped frame rate ({:.1}%). Enable render batching to improve performance.",
                    (stats.dropped_frames as f64 / stats.frame_count as f64) * 100.0
                ),
                priority: OptimizationPriority::High,
                estimated_improvement: 0.25,
            });
        }

        Ok(report)
    }

    /// Helper function to check if two rectangles overlap
    fn regions_overlap(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
    }

    /// Helper function to merge two rectangles
    fn merge_rects(a: Rect, b: Rect) -> Rect {
        let x = a.x.min(b.x);
        let y = a.y.min(b.y);
        let right = (a.x + a.width).max(b.x + b.width);
        let bottom = (a.y + a.height).max(b.y + b.height);

        Rect {
            x,
            y,
            width: right - x,
            height: bottom - y,
        }
    }
}

/// Rendering optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingOptimizationReport {
    pub current_fps: f64,
    pub target_fps: f64,
    pub cache_hit_rate: f64,
    pub recommendations: Vec<RenderingRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingRecommendation {
    pub recommendation_type: RenderingOptimizationType,
    pub description: String,
    pub priority: OptimizationPriority,
    pub estimated_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderingOptimizationType {
    ReduceRenderComplexity,
    ImproveCaching,
    EnableBatching,
    OptimizeIncrementalRendering,
    ReduceRedrawFrequency,
    OptimizeWidgetLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Frame rate monitor for continuous monitoring
pub struct FrameRateMonitor {
    performance_manager: Arc<RenderingPerformanceManager>,
    monitoring_task: Option<tokio::task::JoinHandle<()>>,
    is_running: Arc<RwLock<bool>>,
}

impl FrameRateMonitor {
    /// Create a new frame rate monitor
    pub fn new(performance_manager: Arc<RenderingPerformanceManager>) -> Self {
        Self {
            performance_manager,
            monitoring_task: None,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start monitoring frame rate
    pub async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }

        *is_running = true;
        let performance_manager = Arc::clone(&self.performance_manager);
        let is_running_clone = Arc::clone(&self.is_running);

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            while *is_running_clone.read().await {
                interval.tick().await;

                // Log current FPS statistics
                let stats = performance_manager.get_frame_stats().await;
                if stats.frame_count > 0 {
                    tracing::debug!(
                        "FPS Stats - Current: {:.1}, Avg: {:.1}, Min: {:.1}, Max: {:.1}, Dropped: {}",
                        stats.current_fps,
                        stats.average_fps,
                        stats.min_fps,
                        stats.max_fps,
                        stats.dropped_frames
                    );
                }

                // Check for performance issues
                let config = performance_manager.config.read().await;
                if stats.current_fps < config.target_fps as f64 * 0.5 {
                    tracing::warn!(
                        "Low FPS detected: {:.1} (target: {})",
                        stats.current_fps,
                        config.target_fps
                    );
                }
            }
        });

        self.monitoring_task = Some(task);
        Ok(())
    }

    /// Stop monitoring frame rate
    pub async fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;

        if let Some(task) = self.monitoring_task.take() {
            task.abort();
        }

        Ok(())
    }

    /// Check if monitoring is active
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

impl Drop for FrameRateMonitor {
    fn drop(&mut self) {
        if let Some(task) = self.monitoring_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::PerformanceConfig;

    #[tokio::test]
    async fn test_rendering_performance_manager_creation() {
        let config = RenderingConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let render_manager = RenderingPerformanceManager::new(config, perf_manager);

        let stats = render_manager.get_frame_stats().await;
        assert_eq!(stats.frame_count, 0);
    }

    #[tokio::test]
    async fn test_frame_rate_recording() {
        let config = RenderingConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let render_manager = RenderingPerformanceManager::new(config, perf_manager);

        // Record some frame times
        render_manager
            .record_frame_time(Duration::from_millis(16))
            .await
            .unwrap(); // ~60 FPS
        render_manager
            .record_frame_time(Duration::from_millis(17))
            .await
            .unwrap();
        render_manager
            .record_frame_time(Duration::from_millis(15))
            .await
            .unwrap();

        let stats = render_manager.get_frame_stats().await;
        assert_eq!(stats.frame_count, 3);
        assert!(stats.current_fps > 0.0);
    }

    #[tokio::test]
    async fn test_render_caching() {
        let config = RenderingConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let render_manager = RenderingPerformanceManager::new(config, perf_manager);

        let area = Rect::new(0, 0, 80, 24);
        let buffer = Buffer::empty(area);
        let content_hash = 12345u64;

        // Cache should be empty initially
        let cached = render_manager
            .get_cached_render("test_widget", area, content_hash)
            .await;
        assert!(cached.is_none());

        // Cache the content
        render_manager
            .cache_render("test_widget", area, buffer.clone(), content_hash)
            .await
            .unwrap();

        // Should now be able to retrieve from cache
        let cached = render_manager
            .get_cached_render("test_widget", area, content_hash)
            .await;
        assert!(cached.is_some());

        let (hits, misses, hit_rate) = render_manager.get_cache_stats().await;
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(hit_rate, 0.5);
    }

    #[tokio::test]
    async fn test_dirty_region_tracking() {
        let config = RenderingConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let render_manager = RenderingPerformanceManager::new(config, perf_manager);

        let area1 = Rect::new(0, 0, 10, 10);
        let area2 = Rect::new(5, 5, 10, 10); // Overlapping

        render_manager
            .mark_dirty("widget1".to_string(), area1, 1)
            .await
            .unwrap();
        render_manager
            .mark_dirty("widget1".to_string(), area2, 2)
            .await
            .unwrap();

        let dirty_regions = render_manager.get_dirty_regions().await;

        // Should have merged overlapping regions
        assert_eq!(dirty_regions.len(), 1);
        assert_eq!(dirty_regions[0].priority, 2); // Higher priority should be kept
    }

    #[tokio::test]
    async fn test_incremental_rendering() {
        let config = RenderingConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let render_manager = RenderingPerformanceManager::new(config, perf_manager);

        let content_hash = 12345u64;

        // First render should be needed
        let should_render = render_manager
            .should_use_incremental_rendering("test_widget", content_hash)
            .await;
        assert!(should_render);

        // Second render with same hash should be skipped
        let should_render = render_manager
            .should_use_incremental_rendering("test_widget", content_hash)
            .await;
        assert!(!should_render);

        // Render with different hash should be needed
        let should_render = render_manager
            .should_use_incremental_rendering("test_widget", 54321u64)
            .await;
        assert!(should_render);
    }

    #[tokio::test]
    async fn test_optimization_report() {
        let mut config = RenderingConfig::default();
        config.target_fps = 60;

        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));
        let render_manager = RenderingPerformanceManager::new(config, perf_manager);

        // Simulate low FPS
        for _ in 0..10 {
            render_manager
                .record_frame_time(Duration::from_millis(50))
                .await
                .unwrap(); // ~20 FPS
        }

        let report = render_manager.optimize_rendering().await.unwrap();

        assert!(report.current_fps < report.target_fps);
        assert!(!report.recommendations.is_empty());

        // Should recommend reducing render complexity due to low FPS
        let has_complexity_recommendation = report.recommendations.iter().any(|r| {
            matches!(
                r.recommendation_type,
                RenderingOptimizationType::ReduceRenderComplexity
            )
        });
        assert!(has_complexity_recommendation);
    }
}
