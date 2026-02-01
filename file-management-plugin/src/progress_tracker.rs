//! Progress tracking and aggregation for batch processing
//!
//! This module provides real-time progress tracking, result aggregation,
//! and performance monitoring for batch operations.

use super::batch_processor::{BatchItemResult, BatchItemStatus, BatchProgress};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};

/// Progress event types for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressEvent {
    /// Batch processing started
    BatchStarted {
        batch_id: String,
        tool_name: String,
        total_items: usize,
        started_at: chrono::DateTime<chrono::Utc>,
    },

    /// Item processing started
    ItemStarted {
        batch_id: String,
        item_id: String,
        started_at: chrono::DateTime<chrono::Utc>,
    },

    /// Item processing completed
    ItemCompleted {
        batch_id: String,
        item_id: String,
        status: BatchItemStatus,
        duration: Duration,
        result_size: Option<usize>,
    },

    /// Progress update
    ProgressUpdate {
        batch_id: String,
        progress: BatchProgress,
        updated_at: chrono::DateTime<chrono::Utc>,
    },

    /// Batch processing completed
    BatchCompleted {
        batch_id: String,
        total_duration: Duration,
        final_progress: BatchProgress,
        completed_at: chrono::DateTime<chrono::Utc>,
    },

    /// Error occurred during processing
    ErrorOccurred {
        batch_id: String,
        item_id: Option<String>,
        error_message: String,
        error_type: String,
        occurred_at: chrono::DateTime<chrono::Utc>,
    },
}

/// Configuration for progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressTrackerConfig {
    /// How often to emit progress updates
    pub update_interval: Duration,

    /// Maximum number of events to keep in history
    pub max_event_history: usize,

    /// Whether to track detailed item-level events
    pub track_item_events: bool,

    /// Whether to calculate performance metrics
    pub calculate_metrics: bool,

    /// Buffer size for event channels
    pub event_buffer_size: usize,

    /// Whether to persist progress to storage
    pub persist_progress: bool,
}

impl Default for ProgressTrackerConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(2),
            max_event_history: 1000,
            track_item_events: true,
            calculate_metrics: true,
            event_buffer_size: 100,
            persist_progress: false,
        }
    }
}

/// Aggregated statistics for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    /// Total number of batches processed
    pub total_batches: usize,

    /// Total number of items processed across all batches
    pub total_items_processed: usize,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Average batch processing time
    pub average_batch_duration: Duration,

    /// Average item processing time
    pub average_item_duration: Duration,

    /// Peak throughput (items per second)
    pub peak_throughput: f64,

    /// Average throughput (items per second)
    pub average_throughput: f64,

    /// Most common error types
    pub common_error_types: HashMap<String, usize>,

    /// Tool usage statistics
    pub tool_usage_stats: HashMap<String, ToolUsageStats>,

    /// Performance trends over time
    pub performance_trends: Vec<PerformanceTrend>,
}

/// Statistics for individual tool usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    /// Number of times this tool was used
    pub usage_count: usize,

    /// Total items processed by this tool
    pub items_processed: usize,

    /// Success rate for this tool
    pub success_rate: f64,

    /// Average processing time for this tool
    pub average_duration: Duration,

    /// Peak throughput for this tool
    pub peak_throughput: f64,
}

/// Performance trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    /// Timestamp of this data point
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Throughput at this time
    pub throughput: f64,

    /// Success rate at this time
    pub success_rate: f64,

    /// Average latency at this time
    pub average_latency: Duration,

    /// Number of active batches at this time
    pub active_batches: usize,
}

/// Real-time progress tracker for batch operations
pub struct ProgressTracker {
    config: ProgressTrackerConfig,

    // Event broadcasting
    event_sender: broadcast::Sender<ProgressEvent>,

    // Progress state tracking
    active_batches: Arc<RwLock<HashMap<String, BatchProgressState>>>,

    // Event history
    event_history: Arc<RwLock<Vec<ProgressEvent>>>,

    // Aggregated statistics
    aggregated_stats: Arc<RwLock<AggregatedStats>>,

    // Performance monitoring
    performance_monitor: Arc<PerformanceMonitor>,
}

/// Internal state for tracking batch progress
#[derive(Debug, Clone)]
pub struct BatchProgressState {
    #[allow(dead_code)]
    batch_id: String,
    tool_name: String,
    started_at: Instant,
    progress: BatchProgress,
    item_states: HashMap<String, ItemProgressState>,
    last_update: Instant,
}

/// Internal state for tracking individual item progress
#[derive(Debug, Clone)]
struct ItemProgressState {
    #[allow(dead_code)]
    item_id: String,
    status: BatchItemStatus,
    #[allow(dead_code)]
    started_at: Option<Instant>,
    completed_at: Option<Instant>,
    duration: Option<Duration>,
    retry_count: usize,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(config: ProgressTrackerConfig) -> Self {
        let (event_sender, _) = broadcast::channel(config.event_buffer_size);

        let performance_monitor = Arc::new(PerformanceMonitor::new(config.clone()));

        Self {
            config,
            event_sender,
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            aggregated_stats: Arc::new(RwLock::new(AggregatedStats::default())),
            performance_monitor,
        }
    }

    /// Subscribe to progress events
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressEvent> {
        self.event_sender.subscribe()
    }

    /// Start tracking a new batch
    pub async fn start_batch(&self, batch_id: String, tool_name: String, total_items: usize) {
        let started_at = Instant::now();
        let progress = BatchProgress::new(total_items);

        // Create batch state
        let batch_state = BatchProgressState {
            batch_id: batch_id.clone(),
            tool_name: tool_name.clone(),
            started_at,
            progress,
            item_states: HashMap::new(),
            last_update: started_at,
        };

        // Store batch state
        self.active_batches
            .write()
            .await
            .insert(batch_id.clone(), batch_state);

        // Emit event
        let event = ProgressEvent::BatchStarted {
            batch_id,
            tool_name,
            total_items,
            started_at: chrono::Utc::now(),
        };

        self.emit_event(event).await;
    }

    /// Record that an item has started processing
    pub async fn start_item(&self, batch_id: &str, item_id: String) {
        if let Some(batch_state) = self.active_batches.write().await.get_mut(batch_id) {
            let item_state = ItemProgressState {
                item_id: item_id.clone(),
                status: BatchItemStatus::Processing,
                started_at: Some(Instant::now()),
                completed_at: None,
                duration: None,
                retry_count: 0,
            };

            batch_state.item_states.insert(item_id.clone(), item_state);
            batch_state.progress.processing_items += 1;
            batch_state.progress.pending_items =
                batch_state.progress.pending_items.saturating_sub(1);

            if self.config.track_item_events {
                let event = ProgressEvent::ItemStarted {
                    batch_id: batch_id.to_string(),
                    item_id,
                    started_at: chrono::Utc::now(),
                };

                self.emit_event(event).await;
            }
        }
    }

    /// Record that an item has completed processing
    pub async fn complete_item(&self, batch_id: &str, item_result: BatchItemResult) {
        if let Some(batch_state) = self.active_batches.write().await.get_mut(batch_id) {
            let completed_at = Instant::now();

            // Update item state
            if let Some(item_state) = batch_state.item_states.get_mut(&item_result.id) {
                item_state.status = item_result.status;
                item_state.completed_at = Some(completed_at);
                item_state.duration = Some(item_result.duration);
                item_state.retry_count = item_result.retry_attempts;
            }

            // Update batch progress
            batch_state.progress.processing_items =
                batch_state.progress.processing_items.saturating_sub(1);

            match item_result.status {
                BatchItemStatus::Completed => {
                    batch_state.progress.completed_items += 1;
                }
                BatchItemStatus::Failed => {
                    batch_state.progress.failed_items += 1;
                }
                BatchItemStatus::Skipped => {
                    batch_state.progress.skipped_items += 1;
                }
                _ => {}
            }

            // Update progress timing
            let elapsed = batch_state.started_at.elapsed();
            batch_state.progress.update(elapsed);
            batch_state.last_update = completed_at;

            // Emit item completion event
            if self.config.track_item_events {
                let result_size = item_result
                    .result
                    .as_ref()
                    .and_then(|v| serde_json::to_string(v).ok())
                    .map(|s| s.len());

                let event = ProgressEvent::ItemCompleted {
                    batch_id: batch_id.to_string(),
                    item_id: item_result.id.clone(),
                    status: item_result.status,
                    duration: item_result.duration,
                    result_size,
                };

                self.emit_event(event).await;
            }

            // Record performance metrics
            self.performance_monitor
                .record_item_completion(&item_result)
                .await;
        }
    }

    /// Record an error during processing
    pub async fn record_error(
        &self,
        batch_id: &str,
        item_id: Option<String>,
        error: &str,
        error_type: &str,
    ) {
        let event = ProgressEvent::ErrorOccurred {
            batch_id: batch_id.to_string(),
            item_id,
            error_message: error.to_string(),
            error_type: error_type.to_string(),
            occurred_at: chrono::Utc::now(),
        };

        self.emit_event(event).await;

        // Update aggregated stats
        let mut stats = self.aggregated_stats.write().await;
        *stats
            .common_error_types
            .entry(error_type.to_string())
            .or_insert(0) += 1;
    }

    /// Complete batch tracking
    pub async fn complete_batch(&self, batch_id: &str) {
        if let Some(batch_state) = self.active_batches.write().await.remove(batch_id) {
            let total_duration = batch_state.started_at.elapsed();
            let final_progress = batch_state.progress.clone();

            // Emit completion event
            let event = ProgressEvent::BatchCompleted {
                batch_id: batch_id.to_string(),
                total_duration,
                final_progress: final_progress.clone(),
                completed_at: chrono::Utc::now(),
            };

            self.emit_event(event).await;

            // Update aggregated statistics
            self.update_aggregated_stats(&batch_state, total_duration)
                .await;

            // Record performance trend
            self.performance_monitor
                .record_batch_completion(&batch_state, total_duration)
                .await;
        }
    }

    /// Get current progress for a batch
    pub async fn get_batch_progress(&self, batch_id: &str) -> Option<BatchProgress> {
        self.active_batches.read().await.get(batch_id).map(|state| {
            let mut progress = state.progress.clone();
            progress.update(state.started_at.elapsed());
            progress
        })
    }

    /// Get progress for all active batches
    pub async fn get_all_batch_progress(&self) -> HashMap<String, BatchProgress> {
        let mut result = HashMap::new();
        let batches = self.active_batches.read().await;

        for (batch_id, state) in batches.iter() {
            let mut progress = state.progress.clone();
            progress.update(state.started_at.elapsed());
            result.insert(batch_id.clone(), progress);
        }

        result
    }

    /// Get aggregated statistics
    pub async fn get_aggregated_stats(&self) -> AggregatedStats {
        self.aggregated_stats.read().await.clone()
    }

    /// Get recent event history
    pub async fn get_event_history(&self, limit: Option<usize>) -> Vec<ProgressEvent> {
        let history = self.event_history.read().await;
        let limit = limit.unwrap_or(self.config.max_event_history);

        if history.len() <= limit {
            history.clone()
        } else {
            history[history.len() - limit..].to_vec()
        }
    }

    /// Start background progress reporting
    pub async fn start_progress_reporting(&self) -> tokio::task::JoinHandle<()> {
        let active_batches = self.active_batches.clone();
        let event_sender = self.event_sender.clone();
        let update_interval = self.config.update_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(update_interval);

            loop {
                interval.tick().await;

                let batches = active_batches.read().await;
                for (batch_id, state) in batches.iter() {
                    let mut progress = state.progress.clone();
                    progress.update(state.started_at.elapsed());

                    let event = ProgressEvent::ProgressUpdate {
                        batch_id: batch_id.clone(),
                        progress,
                        updated_at: chrono::Utc::now(),
                    };

                    let _ = event_sender.send(event);
                }

                if batches.is_empty() {
                    // No active batches, sleep longer
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        })
    }

    /// Emit a progress event
    async fn emit_event(&self, event: ProgressEvent) {
        // Send to subscribers
        let _ = self.event_sender.send(event.clone());

        // Add to history
        let mut history = self.event_history.write().await;
        history.push(event);

        // Trim history if needed
        if history.len() > self.config.max_event_history {
            let excess = history.len() - self.config.max_event_history;
            history.drain(0..excess);
        }
    }

    /// Update aggregated statistics after batch completion
    async fn update_aggregated_stats(
        &self,
        batch_state: &BatchProgressState,
        total_duration: Duration,
    ) {
        let mut stats = self.aggregated_stats.write().await;

        stats.total_batches += 1;
        stats.total_items_processed += batch_state.progress.total_items;

        // Update success rate
        let total_completed = stats.total_items_processed;
        let total_successful = total_completed - batch_state.progress.failed_items;
        stats.success_rate = if total_completed > 0 {
            total_successful as f64 / total_completed as f64
        } else {
            0.0
        };

        // Update average batch duration
        let total_batch_time =
            stats.average_batch_duration * (stats.total_batches - 1) as u32 + total_duration;
        stats.average_batch_duration = total_batch_time / stats.total_batches as u32;

        // Update tool usage stats
        let current_throughput = if total_duration.as_secs() > 0 {
            batch_state.progress.total_items as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        let peak_throughput = stats.peak_throughput;

        {
            let tool_stats = stats
                .tool_usage_stats
                .entry(batch_state.tool_name.clone())
                .or_insert_with(|| ToolUsageStats {
                    usage_count: 0,
                    items_processed: 0,
                    success_rate: 0.0,
                    average_duration: Duration::from_secs(0),
                    peak_throughput: 0.0,
                });

            tool_stats.usage_count += 1;
            tool_stats.items_processed += batch_state.progress.total_items;

            // Calculate tool success rate
            let tool_successful = batch_state.progress.completed_items;
            let tool_total = batch_state.progress.total_items;
            tool_stats.success_rate = if tool_total > 0 {
                tool_successful as f64 / tool_total as f64
            } else {
                0.0
            };

            // Update tool average duration
            let total_tool_time =
                tool_stats.average_duration * (tool_stats.usage_count - 1) as u32 + total_duration;
            tool_stats.average_duration = total_tool_time / tool_stats.usage_count as u32;

            if current_throughput > tool_stats.peak_throughput {
                tool_stats.peak_throughput = current_throughput;
            }
        }

        if current_throughput > peak_throughput {
            stats.peak_throughput = current_throughput;
        }

        // Update average throughput
        let total_throughput =
            stats.average_throughput * (stats.total_batches - 1) as f64 + current_throughput;
        stats.average_throughput = total_throughput / stats.total_batches as f64;
    }
}

impl Default for AggregatedStats {
    fn default() -> Self {
        Self {
            total_batches: 0,
            total_items_processed: 0,
            success_rate: 0.0,
            average_batch_duration: Duration::from_secs(0),
            average_item_duration: Duration::from_secs(0),
            peak_throughput: 0.0,
            average_throughput: 0.0,
            common_error_types: HashMap::new(),
            tool_usage_stats: HashMap::new(),
            performance_trends: Vec::new(),
        }
    }
}

/// Performance monitoring component
pub struct PerformanceMonitor {
    config: ProgressTrackerConfig,
    item_durations: Arc<RwLock<Vec<Duration>>>,
    throughput_history: Arc<RwLock<Vec<(chrono::DateTime<chrono::Utc>, f64)>>>,
    success_rate_history: Arc<RwLock<Vec<(chrono::DateTime<chrono::Utc>, f64)>>>,
}

impl PerformanceMonitor {
    pub fn new(config: ProgressTrackerConfig) -> Self {
        Self {
            config,
            item_durations: Arc::new(RwLock::new(Vec::new())),
            throughput_history: Arc::new(RwLock::new(Vec::new())),
            success_rate_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record_item_completion(&self, item_result: &BatchItemResult) {
        if self.config.calculate_metrics {
            let mut durations = self.item_durations.write().await;
            durations.push(item_result.duration);

            // Keep only recent durations for performance
            if durations.len() > 10000 {
                let len = durations.len();
                durations.drain(0..len - 10000);
            }
        }
    }

    pub async fn record_batch_completion(
        &self,
        batch_state: &BatchProgressState,
        total_duration: Duration,
    ) {
        if self.config.calculate_metrics {
            let now = chrono::Utc::now();

            // Record throughput
            let throughput = if total_duration.as_secs() > 0 {
                batch_state.progress.total_items as f64 / total_duration.as_secs_f64()
            } else {
                0.0
            };

            let mut throughput_history = self.throughput_history.write().await;
            throughput_history.push((now, throughput));

            // Record success rate
            let success_rate = if batch_state.progress.total_items > 0 {
                batch_state.progress.completed_items as f64
                    / batch_state.progress.total_items as f64
            } else {
                0.0
            };

            let mut success_rate_history = self.success_rate_history.write().await;
            success_rate_history.push((now, success_rate));

            // Trim history
            let max_history = 1000;
            if throughput_history.len() > max_history {
                let len = throughput_history.len();
                throughput_history.drain(0..len - max_history);
            }

            if success_rate_history.len() > max_history {
                let len = success_rate_history.len();
                success_rate_history.drain(0..len - max_history);
            }
        }
    }

    pub async fn get_performance_trends(&self, limit: Option<usize>) -> Vec<PerformanceTrend> {
        let limit = limit.unwrap_or(100);
        let throughput_history = self.throughput_history.read().await;
        let success_rate_history = self.success_rate_history.read().await;
        let item_durations = self.item_durations.read().await;

        let mut trends = Vec::new();

        // Create trends from throughput history
        let start_idx = if throughput_history.len() > limit {
            throughput_history.len() - limit
        } else {
            0
        };

        for (timestamp, throughput) in throughput_history[start_idx..].iter() {
            // Find corresponding success rate
            let success_rate = success_rate_history
                .iter()
                .find(|(ts, _)| (ts.timestamp() - timestamp.timestamp()).abs() < 60) // Within 1 minute
                .map(|(_, rate)| *rate)
                .unwrap_or(0.0);

            // Calculate average latency from recent item durations
            let average_latency = if !item_durations.is_empty() {
                let recent_count = item_durations.len().min(100);
                let recent_durations = &item_durations[item_durations.len() - recent_count..];
                recent_durations.iter().sum::<Duration>() / recent_count as u32
            } else {
                Duration::from_secs(0)
            };

            trends.push(PerformanceTrend {
                timestamp: *timestamp,
                throughput: *throughput,
                success_rate,
                average_latency,
                active_batches: 1, // Simplified for now
            });
        }

        trends
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_progress_tracker_creation() {
        let config = ProgressTrackerConfig::default();
        let tracker = ProgressTracker::new(config);

        let stats = tracker.get_aggregated_stats().await;
        assert_eq!(stats.total_batches, 0);
        assert_eq!(stats.total_items_processed, 0);
    }

    #[tokio::test]
    async fn test_batch_progress_tracking() {
        let config = ProgressTrackerConfig::default();
        let tracker = ProgressTracker::new(config);

        let batch_id = "test-batch-1".to_string();
        let tool_name = "test-tool".to_string();

        // Start batch
        tracker.start_batch(batch_id.clone(), tool_name, 3).await;

        // Check initial progress
        let progress = tracker.get_batch_progress(&batch_id).await.unwrap();
        assert_eq!(progress.total_items, 3);
        assert_eq!(progress.pending_items, 3);
        assert_eq!(progress.completed_items, 0);

        // Start and complete items
        tracker.start_item(&batch_id, "item-1".to_string()).await;

        let item_result = BatchItemResult {
            id: "item-1".to_string(),
            status: BatchItemStatus::Completed,
            result: Some(serde_json::json!({"success": true})),
            error: None,
            duration: Duration::from_millis(100),
            retry_attempts: 0,
            metadata: HashMap::new(),
        };

        tracker.complete_item(&batch_id, item_result).await;

        // Check updated progress
        let progress = tracker.get_batch_progress(&batch_id).await.unwrap();
        assert_eq!(progress.completed_items, 1);
        assert_eq!(progress.processing_items, 0);
        assert_eq!(progress.pending_items, 2);

        // Complete batch
        tracker.complete_batch(&batch_id).await;

        // Check that batch is no longer active
        assert!(tracker.get_batch_progress(&batch_id).await.is_none());

        // Check aggregated stats
        let stats = tracker.get_aggregated_stats().await;
        assert_eq!(stats.total_batches, 1);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let config = ProgressTrackerConfig::default();
        let tracker = ProgressTracker::new(config);

        let mut receiver = tracker.subscribe();

        let batch_id = "test-batch-2".to_string();
        let tool_name = "test-tool".to_string();

        // Start batch - should emit event
        tracker
            .start_batch(batch_id.clone(), tool_name.clone(), 1)
            .await;

        // Receive the event
        let event = receiver.recv().await.unwrap();
        match event {
            ProgressEvent::BatchStarted {
                batch_id: received_id,
                tool_name: received_tool,
                total_items,
                ..
            } => {
                assert_eq!(received_id, batch_id);
                assert_eq!(received_tool, tool_name);
                assert_eq!(total_items, 1);
            }
            _ => panic!("Expected BatchStarted event"),
        }
    }

    #[tokio::test]
    async fn test_error_tracking() {
        let config = ProgressTrackerConfig::default();
        let tracker = ProgressTracker::new(config);

        let batch_id = "error-batch".to_string();

        // Record some errors
        tracker
            .record_error(
                &batch_id,
                Some("item-1".to_string()),
                "File not found",
                "not_found",
            )
            .await;
        tracker
            .record_error(
                &batch_id,
                Some("item-2".to_string()),
                "Permission denied",
                "permission",
            )
            .await;
        tracker
            .record_error(
                &batch_id,
                Some("item-3".to_string()),
                "File not found",
                "not_found",
            )
            .await;

        // Check aggregated stats
        let stats = tracker.get_aggregated_stats().await;
        assert_eq!(stats.common_error_types.get("not_found"), Some(&2));
        assert_eq!(stats.common_error_types.get("permission"), Some(&1));
    }

    #[tokio::test]
    async fn test_performance_monitoring() {
        let config = ProgressTrackerConfig {
            calculate_metrics: true,
            ..Default::default()
        };
        let monitor = PerformanceMonitor::new(config);

        // Record some item completions
        let item_result1 = BatchItemResult {
            id: "item-1".to_string(),
            status: BatchItemStatus::Completed,
            result: None,
            error: None,
            duration: Duration::from_millis(100),
            retry_attempts: 0,
            metadata: HashMap::new(),
        };

        let item_result2 = BatchItemResult {
            id: "item-2".to_string(),
            status: BatchItemStatus::Completed,
            result: None,
            error: None,
            duration: Duration::from_millis(200),
            retry_attempts: 0,
            metadata: HashMap::new(),
        };

        monitor.record_item_completion(&item_result1).await;
        monitor.record_item_completion(&item_result2).await;

        // Record batch completion
        let batch_state = BatchProgressState {
            batch_id: "test-batch".to_string(),
            tool_name: "test-tool".to_string(),
            started_at: Instant::now(),
            progress: BatchProgress::new(2),
            item_states: HashMap::new(),
            last_update: Instant::now(),
        };

        monitor
            .record_batch_completion(&batch_state, Duration::from_millis(300))
            .await;

        // Get performance trends
        let trends = monitor.get_performance_trends(Some(10)).await;
        assert!(!trends.is_empty());
    }

    #[tokio::test]
    async fn test_progress_reporting_task() {
        let config = ProgressTrackerConfig {
            update_interval: Duration::from_millis(100),
            ..Default::default()
        };
        let tracker = ProgressTracker::new(config);

        let mut receiver = tracker.subscribe();

        // Start progress reporting
        let _reporting_task = tracker.start_progress_reporting().await;

        let batch_id = "reporting-batch".to_string();
        tracker
            .start_batch(batch_id.clone(), "test-tool".to_string(), 1)
            .await;

        // Skip the BatchStarted event
        let _ = receiver.recv().await.unwrap();

        // Wait for a progress update
        let event = tokio::time::timeout(Duration::from_millis(500), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        match event {
            ProgressEvent::ProgressUpdate {
                batch_id: received_id,
                ..
            } => {
                assert_eq!(received_id, batch_id);
            }
            _ => panic!("Expected ProgressUpdate event, got {:?}", event),
        }
    }

    #[tokio::test]
    async fn test_event_history() {
        let config = ProgressTrackerConfig {
            max_event_history: 5,
            ..Default::default()
        };
        let tracker = ProgressTracker::new(config);

        // Generate more events than the history limit
        for i in 0..10 {
            let batch_id = format!("batch-{}", i);
            tracker
                .start_batch(batch_id, "test-tool".to_string(), 1)
                .await;
        }

        // Check that history is limited
        let history = tracker.get_event_history(None).await;
        assert_eq!(history.len(), 5);

        // Check that we can get a smaller limit
        let limited_history = tracker.get_event_history(Some(3)).await;
        assert_eq!(limited_history.len(), 3);
    }
}
