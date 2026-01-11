//! TUI Performance Monitoring Module
//!
//! This module provides comprehensive performance monitoring capabilities for the TUI interface,
//! including performance metrics collection, debugging information, and performance analysis tools.

use crate::error::Result;
use crate::performance::{PerformanceManager, PerformanceMonitor};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable performance monitoring
    pub enabled: bool,

    /// Monitoring interval
    pub monitoring_interval: Duration,

    /// Enable real-time metrics collection
    pub enable_realtime_metrics: bool,

    /// Enable performance profiling
    pub enable_profiling: bool,

    /// Enable debug information collection
    pub enable_debug_info: bool,

    /// Maximum number of metrics to keep in history
    pub max_history_size: usize,

    /// Enable performance alerts
    pub enable_alerts: bool,

    /// Performance alert thresholds
    pub alert_thresholds: AlertThresholds,

    /// Enable performance logging
    pub enable_logging: bool,

    /// Log level for performance events
    pub log_level: LogLevel,

    /// Enable metrics export
    pub enable_export: bool,

    /// Export format
    pub export_format: ExportFormat,

    /// Export interval
    pub export_interval: Duration,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            monitoring_interval: Duration::from_secs(1),
            enable_realtime_metrics: true,
            enable_profiling: true,
            enable_debug_info: true,
            max_history_size: 1000,
            enable_alerts: true,
            alert_thresholds: AlertThresholds::default(),
            enable_logging: true,
            log_level: LogLevel::Info,
            enable_export: false,
            export_format: ExportFormat::Json,
            export_interval: Duration::from_secs(60),
        }
    }
}

/// Performance alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU usage threshold (percentage)
    pub cpu_threshold: f64,

    /// Memory usage threshold (bytes)
    pub memory_threshold: usize,

    /// Frame rate threshold (FPS)
    pub fps_threshold: f64,

    /// Response time threshold (milliseconds)
    pub response_time_threshold: u64,

    /// Error rate threshold (percentage)
    pub error_rate_threshold: f64,

    /// Cache miss rate threshold (percentage)
    pub cache_miss_threshold: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold: 80.0,
            memory_threshold: 40 * 1024 * 1024, // 40MB
            fps_threshold: 30.0,
            response_time_threshold: 100,
            error_rate_threshold: 5.0,
            cache_miss_threshold: 50.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Prometheus,
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: usize,
    pub memory_peak: usize,
    pub fps: f64,
    pub frame_time: Duration,
    pub response_times: HashMap<String, Duration>,
    pub error_count: u64,
    pub cache_stats: CacheStats,
    pub widget_stats: HashMap<String, WidgetStats>,
    pub system_stats: SystemStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub max_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetStats {
    pub render_time: Duration,
    pub update_time: Duration,
    pub memory_usage: usize,
    pub event_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_memory: usize,
    pub available_memory: usize,
    pub cpu_cores: usize,
    pub load_average: f64,
    pub uptime: Duration,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub metric_value: f64,
    pub threshold: f64,
    pub component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    HighCpuUsage,
    HighMemoryUsage,
    LowFrameRate,
    HighResponseTime,
    HighErrorRate,
    HighCacheMissRate,
    SystemResource,
    ComponentFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Performance profiling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingData {
    pub function_name: String,
    pub call_count: u64,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub memory_allocated: usize,
    pub stack_trace: Vec<String>,
}

/// Debug information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub component: String,
    pub event_type: String,
    pub timestamp: u64,
    pub data: HashMap<String, String>,
    pub stack_trace: Option<Vec<String>>,
}

/// Performance analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub summary: PerformanceSummary,
    pub trends: PerformanceTrends,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub recommendations: Vec<PerformanceRecommendation>,
    pub alerts: Vec<PerformanceAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub average_fps: f64,
    pub average_memory_usage: usize,
    pub average_cpu_usage: f64,
    pub total_errors: u64,
    pub uptime: Duration,
    pub cache_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub fps_trend: TrendDirection,
    pub memory_trend: TrendDirection,
    pub cpu_trend: TrendDirection,
    pub error_trend: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub component: String,
    pub bottleneck_type: BottleneckType,
    pub impact_score: f64,
    pub description: String,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    CpuBound,
    MemoryBound,
    IoBound,
    RenderingBound,
    CacheMiss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub recommendation_type: RecommendationType,
    pub priority: RecommendationPriority,
    pub description: String,
    pub estimated_improvement: f64,
    pub implementation_effort: ImplementationEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    OptimizeRendering,
    ReduceMemoryUsage,
    ImproveCache,
    OptimizeAlgorithm,
    UpgradeHardware,
    ConfigurationChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

/// TUI Performance Monitor
pub struct TuiPerformanceMonitor {
    config: Arc<RwLock<MonitoringConfig>>,
    performance_manager: Arc<PerformanceManager>,

    // Metrics collection
    metrics_history: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
    current_metrics: Arc<RwLock<PerformanceSnapshot>>,

    // Profiling data
    profiling_data: Arc<RwLock<HashMap<String, ProfilingData>>>,

    // Debug information
    debug_info: Arc<RwLock<VecDeque<DebugInfo>>>,

    // Alerts
    active_alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    alert_history: Arc<RwLock<VecDeque<PerformanceAlert>>>,

    // Monitoring task
    monitoring_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,

    // Performance counters
    frame_counter: Arc<RwLock<u64>>,
    error_counter: Arc<RwLock<u64>>,
    event_counter: Arc<RwLock<HashMap<String, u64>>>,

    // Timing measurements
    operation_timings: Arc<RwLock<HashMap<String, VecDeque<Duration>>>>,
}

impl TuiPerformanceMonitor {
    /// Create a new TUI performance monitor
    pub fn new(config: MonitoringConfig, performance_manager: Arc<PerformanceManager>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            performance_manager,
            metrics_history: Arc::new(RwLock::new(VecDeque::new())),
            current_metrics: Arc::new(RwLock::new(PerformanceSnapshot::default())),
            profiling_data: Arc::new(RwLock::new(HashMap::new())),
            debug_info: Arc::new(RwLock::new(VecDeque::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            monitoring_task: Arc::new(RwLock::new(None)),
            frame_counter: Arc::new(RwLock::new(0)),
            error_counter: Arc::new(RwLock::new(0)),
            event_counter: Arc::new(RwLock::new(HashMap::new())),
            operation_timings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start performance monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let config = self.config.read().await;
        if !config.enabled {
            return Ok(());
        }

        let monitoring_interval = config.monitoring_interval;
        drop(config);

        let config_clone = Arc::clone(&self.config);
        let metrics_history_clone = Arc::clone(&self.metrics_history);
        let current_metrics_clone = Arc::clone(&self.current_metrics);
        let active_alerts_clone = Arc::clone(&self.active_alerts);
        let alert_history_clone = Arc::clone(&self.alert_history);
        let frame_counter_clone = Arc::clone(&self.frame_counter);
        let error_counter_clone = Arc::clone(&self.error_counter);

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitoring_interval);

            loop {
                interval.tick().await;

                let config = config_clone.read().await;
                if !config.enabled {
                    break;
                }

                // Collect current metrics
                let snapshot =
                    Self::collect_metrics_snapshot(&frame_counter_clone, &error_counter_clone)
                        .await;

                // Update current metrics
                {
                    let mut current = current_metrics_clone.write().await;
                    *current = snapshot.clone();
                }

                // Add to history
                {
                    let mut history = metrics_history_clone.write().await;
                    history.push_back(snapshot.clone());

                    // Limit history size
                    while history.len() > config.max_history_size {
                        history.pop_front();
                    }
                }

                // Check for alerts
                if config.enable_alerts {
                    let alerts = Self::check_alerts(&snapshot, &config.alert_thresholds).await;

                    if !alerts.is_empty() {
                        let mut active_alerts = active_alerts_clone.write().await;
                        let mut alert_history = alert_history_clone.write().await;

                        for alert in alerts {
                            // Add to active alerts if not already present
                            if !active_alerts.iter().any(|a| {
                                a.alert_type == alert.alert_type && a.component == alert.component
                            }) {
                                active_alerts.push(alert.clone());
                            }

                            // Add to history
                            alert_history.push_back(alert.clone());

                            // Limit alert history
                            while alert_history.len() > config.max_history_size {
                                alert_history.pop_front();
                            }

                            // Log alert
                            if config.enable_logging {
                                match alert.severity {
                                    AlertSeverity::Critical => {
                                        tracing::error!("Performance Alert: {}", alert.message)
                                    }
                                    AlertSeverity::Warning => {
                                        tracing::warn!("Performance Alert: {}", alert.message)
                                    }
                                    AlertSeverity::Info => {
                                        tracing::info!("Performance Alert: {}", alert.message)
                                    }
                                }
                            }
                        }
                    }
                }

                drop(config);
            }
        });

        let mut monitoring_task = self.monitoring_task.write().await;
        *monitoring_task = Some(task);

        tracing::info!("TUI performance monitoring started");
        Ok(())
    }

    /// Stop performance monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut monitoring_task = self.monitoring_task.write().await;
        if let Some(task) = monitoring_task.take() {
            task.abort();
        }

        tracing::info!("TUI performance monitoring stopped");
        Ok(())
    }

    /// Record frame rendering
    pub async fn record_frame(&self, render_time: Duration) -> Result<()> {
        let mut frame_counter = self.frame_counter.write().await;
        *frame_counter += 1;

        // Record render time
        let mut timings = self.operation_timings.write().await;
        let render_timings = timings
            .entry("frame_render".to_string())
            .or_insert_with(VecDeque::new);
        render_timings.push_back(render_time);

        // Keep only recent timings
        while render_timings.len() > 100 {
            render_timings.pop_front();
        }

        Ok(())
    }

    /// Record error occurrence
    pub async fn record_error(&self, component: &str, error: &str) -> Result<()> {
        let mut error_counter = self.error_counter.write().await;
        *error_counter += 1;

        // Record debug info
        let config = self.config.read().await;
        if config.enable_debug_info {
            let debug_info = DebugInfo {
                component: component.to_string(),
                event_type: "error".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                data: {
                    let mut data = HashMap::new();
                    data.insert("error".to_string(), error.to_string());
                    data
                },
                stack_trace: None, // Could be enhanced to capture actual stack trace
            };

            let mut debug_info_queue = self.debug_info.write().await;
            debug_info_queue.push_back(debug_info);

            // Limit debug info size
            while debug_info_queue.len() > config.max_history_size {
                debug_info_queue.pop_front();
            }
        }

        Ok(())
    }

    /// Record operation timing
    pub async fn record_operation_time(&self, operation: &str, duration: Duration) -> Result<()> {
        let mut timings = self.operation_timings.write().await;
        let operation_timings = timings
            .entry(operation.to_string())
            .or_insert_with(VecDeque::new);
        operation_timings.push_back(duration);

        // Keep only recent timings
        while operation_timings.len() > 100 {
            operation_timings.pop_front();
        }

        Ok(())
    }

    /// Record event occurrence
    pub async fn record_event(&self, event_type: &str) -> Result<()> {
        let mut event_counter = self.event_counter.write().await;
        let count = event_counter.entry(event_type.to_string()).or_insert(0);
        *count += 1;

        Ok(())
    }

    /// Start profiling a function
    pub async fn start_profiling(&self, function_name: &str) -> ProfilingSession {
        ProfilingSession::new(function_name.to_string(), Arc::clone(&self.profiling_data))
    }

    /// Get current performance metrics
    pub async fn get_current_metrics(&self) -> PerformanceSnapshot {
        self.current_metrics.read().await.clone()
    }

    /// Get performance history
    pub async fn get_metrics_history(&self) -> Vec<PerformanceSnapshot> {
        self.metrics_history.read().await.iter().cloned().collect()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<PerformanceAlert> {
        self.active_alerts.read().await.clone()
    }

    /// Clear active alerts
    pub async fn clear_alerts(&self) -> Result<()> {
        let mut active_alerts = self.active_alerts.write().await;
        active_alerts.clear();

        tracing::info!("Performance alerts cleared");
        Ok(())
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> Result<PerformanceReport> {
        let history = self.get_metrics_history().await;
        let alerts = self.active_alerts.read().await.clone();

        if history.is_empty() {
            return Ok(PerformanceReport {
                summary: PerformanceSummary::default(),
                trends: PerformanceTrends::default(),
                bottlenecks: Vec::new(),
                recommendations: Vec::new(),
                alerts,
            });
        }

        // Calculate summary
        let summary = self.calculate_summary(&history).await;

        // Analyze trends
        let trends = self.analyze_trends(&history).await;

        // Identify bottlenecks
        let bottlenecks = self.identify_bottlenecks(&history).await;

        // Generate recommendations
        let recommendations = self
            .generate_recommendations(&summary, &trends, &bottlenecks)
            .await;

        Ok(PerformanceReport {
            summary,
            trends,
            bottlenecks,
            recommendations,
            alerts,
        })
    }

    /// Export performance data
    pub async fn export_data(&self, format: ExportFormat) -> Result<String> {
        let history = self.get_metrics_history().await;

        match format {
            ExportFormat::Json => serde_json::to_string_pretty(&history)
                .map_err(|e| crate::error::WorkflowError::ValidationError(e.to_string()).into()),
            ExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("timestamp,cpu_usage,memory_usage,fps,frame_time_ms,error_count\n");

                for snapshot in history {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        snapshot.timestamp,
                        snapshot.cpu_usage,
                        snapshot.memory_usage,
                        snapshot.fps,
                        snapshot.frame_time.as_millis(),
                        snapshot.error_count
                    ));
                }

                Ok(csv)
            }
            ExportFormat::Prometheus => {
                let mut prometheus = String::new();

                if let Some(latest) = history.last() {
                    prometheus.push_str(&format!("tui_cpu_usage {}\n", latest.cpu_usage));
                    prometheus.push_str(&format!("tui_memory_usage {}\n", latest.memory_usage));
                    prometheus.push_str(&format!("tui_fps {}\n", latest.fps));
                    prometheus.push_str(&format!(
                        "tui_frame_time_ms {}\n",
                        latest.frame_time.as_millis()
                    ));
                    prometheus.push_str(&format!("tui_error_count {}\n", latest.error_count));
                }

                Ok(prometheus)
            }
        }
    }

    // Private methods

    async fn collect_metrics_snapshot(
        frame_counter: &Arc<RwLock<u64>>,
        error_counter: &Arc<RwLock<u64>>,
    ) -> PerformanceSnapshot {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Get system metrics (simplified - would use actual system monitoring)
        let cpu_usage = Self::get_cpu_usage().await;
        let memory_usage = Self::get_memory_usage().await;
        let memory_peak = Self::get_peak_memory_usage().await;

        // Calculate FPS from frame counter
        let frame_count = *frame_counter.read().await;
        let fps = frame_count as f64; // Simplified - would calculate actual FPS

        let error_count = *error_counter.read().await;

        PerformanceSnapshot {
            timestamp,
            cpu_usage,
            memory_usage,
            memory_peak,
            fps,
            frame_time: Duration::from_millis(16), // Placeholder
            response_times: HashMap::new(),
            error_count,
            cache_stats: CacheStats {
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                size: 0,
                max_size: 0,
            },
            widget_stats: HashMap::new(),
            system_stats: SystemStats {
                total_memory: 8 * 1024 * 1024 * 1024,     // 8GB placeholder
                available_memory: 4 * 1024 * 1024 * 1024, // 4GB placeholder
                cpu_cores: 4,
                load_average: cpu_usage / 100.0,
                uptime: Duration::from_secs(3600), // 1 hour placeholder
            },
        }
    }

    async fn get_cpu_usage() -> f64 {
        // Placeholder - would use actual system monitoring
        20.0
    }

    async fn get_memory_usage() -> usize {
        // Placeholder - would use actual memory monitoring
        30 * 1024 * 1024 // 30MB
    }

    async fn get_peak_memory_usage() -> usize {
        // Placeholder - would track actual peak usage
        40 * 1024 * 1024 // 40MB
    }

    async fn check_alerts(
        snapshot: &PerformanceSnapshot,
        thresholds: &AlertThresholds,
    ) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        let timestamp = snapshot.timestamp;

        // Check CPU usage
        if snapshot.cpu_usage > thresholds.cpu_threshold {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::HighCpuUsage,
                severity: if snapshot.cpu_usage > thresholds.cpu_threshold * 1.2 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                message: format!("High CPU usage: {:.1}%", snapshot.cpu_usage),
                timestamp,
                metric_value: snapshot.cpu_usage,
                threshold: thresholds.cpu_threshold,
                component: "system".to_string(),
            });
        }

        // Check memory usage
        if snapshot.memory_usage > thresholds.memory_threshold {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::HighMemoryUsage,
                severity: if snapshot.memory_usage > thresholds.memory_threshold * 2 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                message: format!("High memory usage: {} bytes", snapshot.memory_usage),
                timestamp,
                metric_value: snapshot.memory_usage as f64,
                threshold: thresholds.memory_threshold as f64,
                component: "tui".to_string(),
            });
        }

        // Check frame rate
        if snapshot.fps < thresholds.fps_threshold {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::LowFrameRate,
                severity: if snapshot.fps < thresholds.fps_threshold * 0.5 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                message: format!("Low frame rate: {:.1} FPS", snapshot.fps),
                timestamp,
                metric_value: snapshot.fps,
                threshold: thresholds.fps_threshold,
                component: "renderer".to_string(),
            });
        }

        alerts
    }

    async fn calculate_summary(&self, history: &[PerformanceSnapshot]) -> PerformanceSummary {
        if history.is_empty() {
            return PerformanceSummary::default();
        }

        let count = history.len() as f64;
        let average_fps = history.iter().map(|s| s.fps).sum::<f64>() / count;
        let average_memory_usage =
            (history.iter().map(|s| s.memory_usage).sum::<usize>() as f64 / count) as usize;
        let average_cpu_usage = history.iter().map(|s| s.cpu_usage).sum::<f64>() / count;
        let total_errors = history.last().map(|s| s.error_count).unwrap_or(0);

        // Calculate uptime from first to last timestamp
        let uptime = if history.len() > 1 {
            Duration::from_millis(
                history.last().unwrap().timestamp - history.first().unwrap().timestamp,
            )
        } else {
            Duration::ZERO
        };

        let cache_efficiency = history.iter().map(|s| s.cache_stats.hit_rate).sum::<f64>() / count;

        PerformanceSummary {
            average_fps,
            average_memory_usage,
            average_cpu_usage,
            total_errors,
            uptime,
            cache_efficiency,
        }
    }

    async fn analyze_trends(&self, history: &[PerformanceSnapshot]) -> PerformanceTrends {
        if history.len() < 2 {
            return PerformanceTrends::default();
        }

        let mid_point = history.len() / 2;
        let first_half = &history[..mid_point];
        let second_half = &history[mid_point..];

        let fps_trend = Self::calculate_trend(
            first_half.iter().map(|s| s.fps).sum::<f64>() / first_half.len() as f64,
            second_half.iter().map(|s| s.fps).sum::<f64>() / second_half.len() as f64,
        );

        let memory_trend = Self::calculate_trend(
            first_half.iter().map(|s| s.memory_usage).sum::<usize>() as f64
                / first_half.len() as f64,
            second_half.iter().map(|s| s.memory_usage).sum::<usize>() as f64
                / second_half.len() as f64,
        );

        let cpu_trend = Self::calculate_trend(
            first_half.iter().map(|s| s.cpu_usage).sum::<f64>() / first_half.len() as f64,
            second_half.iter().map(|s| s.cpu_usage).sum::<f64>() / second_half.len() as f64,
        );

        let error_trend = Self::calculate_trend(
            first_half.last().map(|s| s.error_count).unwrap_or(0) as f64,
            second_half.last().map(|s| s.error_count).unwrap_or(0) as f64,
        );

        PerformanceTrends {
            fps_trend,
            memory_trend,
            cpu_trend,
            error_trend,
        }
    }

    fn calculate_trend(first_value: f64, second_value: f64) -> TrendDirection {
        let change_ratio = (second_value - first_value) / first_value.max(1.0);

        if change_ratio > 0.1 {
            TrendDirection::Degrading // For metrics where higher is worse
        } else if change_ratio < -0.1 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    async fn identify_bottlenecks(
        &self,
        history: &[PerformanceSnapshot],
    ) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        if let Some(latest) = history.last() {
            // Check for CPU bottleneck
            if latest.cpu_usage > 80.0 {
                bottlenecks.push(PerformanceBottleneck {
                    component: "cpu".to_string(),
                    bottleneck_type: BottleneckType::CpuBound,
                    impact_score: latest.cpu_usage / 100.0,
                    description: format!("High CPU usage: {:.1}%", latest.cpu_usage),
                    suggested_fix: "Optimize rendering algorithms or reduce update frequency"
                        .to_string(),
                });
            }

            // Check for memory bottleneck
            if latest.memory_usage > 40 * 1024 * 1024 {
                // 40MB
                bottlenecks.push(PerformanceBottleneck {
                    component: "memory".to_string(),
                    bottleneck_type: BottleneckType::MemoryBound,
                    impact_score: latest.memory_usage as f64 / (50.0 * 1024.0 * 1024.0),
                    description: format!("High memory usage: {} bytes", latest.memory_usage),
                    suggested_fix: "Implement memory pooling or reduce cache sizes".to_string(),
                });
            }

            // Check for rendering bottleneck
            if latest.fps < 30.0 {
                bottlenecks.push(PerformanceBottleneck {
                    component: "renderer".to_string(),
                    bottleneck_type: BottleneckType::RenderingBound,
                    impact_score: (60.0 - latest.fps) / 60.0,
                    description: format!("Low frame rate: {:.1} FPS", latest.fps),
                    suggested_fix: "Enable render caching or reduce widget complexity".to_string(),
                });
            }
        }

        bottlenecks
    }

    async fn generate_recommendations(
        &self,
        summary: &PerformanceSummary,
        trends: &PerformanceTrends,
        bottlenecks: &[PerformanceBottleneck],
    ) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();

        // Recommendations based on bottlenecks
        for bottleneck in bottlenecks {
            match bottleneck.bottleneck_type {
                BottleneckType::CpuBound => {
                    recommendations.push(PerformanceRecommendation {
                        recommendation_type: RecommendationType::OptimizeRendering,
                        priority: RecommendationPriority::High,
                        description: "Optimize CPU-intensive operations".to_string(),
                        estimated_improvement: 30.0,
                        implementation_effort: ImplementationEffort::Medium,
                    });
                }
                BottleneckType::MemoryBound => {
                    recommendations.push(PerformanceRecommendation {
                        recommendation_type: RecommendationType::ReduceMemoryUsage,
                        priority: RecommendationPriority::High,
                        description: "Implement memory optimization strategies".to_string(),
                        estimated_improvement: 25.0,
                        implementation_effort: ImplementationEffort::Medium,
                    });
                }
                BottleneckType::RenderingBound => {
                    recommendations.push(PerformanceRecommendation {
                        recommendation_type: RecommendationType::OptimizeRendering,
                        priority: RecommendationPriority::Critical,
                        description: "Optimize rendering pipeline".to_string(),
                        estimated_improvement: 50.0,
                        implementation_effort: ImplementationEffort::High,
                    });
                }
                _ => {}
            }
        }

        // Recommendations based on trends
        if matches!(trends.memory_trend, TrendDirection::Degrading) {
            recommendations.push(PerformanceRecommendation {
                recommendation_type: RecommendationType::ReduceMemoryUsage,
                priority: RecommendationPriority::Medium,
                description: "Memory usage is increasing over time".to_string(),
                estimated_improvement: 20.0,
                implementation_effort: ImplementationEffort::Low,
            });
        }

        if matches!(trends.fps_trend, TrendDirection::Degrading) {
            recommendations.push(PerformanceRecommendation {
                recommendation_type: RecommendationType::OptimizeRendering,
                priority: RecommendationPriority::Medium,
                description: "Frame rate is decreasing over time".to_string(),
                estimated_improvement: 25.0,
                implementation_effort: ImplementationEffort::Medium,
            });
        }

        recommendations
    }
}

/// Profiling session for measuring function performance
pub struct ProfilingSession {
    function_name: String,
    start_time: Instant,
    profiling_data: Arc<RwLock<HashMap<String, ProfilingData>>>,
}

impl ProfilingSession {
    fn new(
        function_name: String,
        profiling_data: Arc<RwLock<HashMap<String, ProfilingData>>>,
    ) -> Self {
        Self {
            function_name,
            start_time: Instant::now(),
            profiling_data,
        }
    }

    /// End the profiling session
    pub async fn end(self) {
        let duration = self.start_time.elapsed();

        let mut profiling_data = self.profiling_data.write().await;
        let data = profiling_data
            .entry(self.function_name.clone())
            .or_insert_with(|| ProfilingData {
                function_name: self.function_name.clone(),
                call_count: 0,
                total_time: Duration::ZERO,
                average_time: Duration::ZERO,
                min_time: Duration::MAX,
                max_time: Duration::ZERO,
                memory_allocated: 0,
                stack_trace: Vec::new(),
            });

        data.call_count += 1;
        data.total_time += duration;
        data.average_time = data.total_time / data.call_count as u32;
        data.min_time = data.min_time.min(duration);
        data.max_time = data.max_time.max(duration);
    }
}

// Default implementations

impl Default for PerformanceSnapshot {
    fn default() -> Self {
        Self {
            timestamp: 0,
            cpu_usage: 0.0,
            memory_usage: 0,
            memory_peak: 0,
            fps: 0.0,
            frame_time: Duration::ZERO,
            response_times: HashMap::new(),
            error_count: 0,
            cache_stats: CacheStats {
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                size: 0,
                max_size: 0,
            },
            widget_stats: HashMap::new(),
            system_stats: SystemStats {
                total_memory: 0,
                available_memory: 0,
                cpu_cores: 0,
                load_average: 0.0,
                uptime: Duration::ZERO,
            },
        }
    }
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            average_fps: 0.0,
            average_memory_usage: 0,
            average_cpu_usage: 0.0,
            total_errors: 0,
            uptime: Duration::ZERO,
            cache_efficiency: 0.0,
        }
    }
}

impl Default for PerformanceTrends {
    fn default() -> Self {
        Self {
            fps_trend: TrendDirection::Stable,
            memory_trend: TrendDirection::Stable,
            cpu_trend: TrendDirection::Stable,
            error_trend: TrendDirection::Stable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::PerformanceConfig;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let config = MonitoringConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));

        let monitor = TuiPerformanceMonitor::new(config, perf_manager);

        let metrics = monitor.get_current_metrics().await;
        assert_eq!(metrics.timestamp, 0); // Default value
    }

    #[tokio::test]
    async fn test_frame_recording() {
        let config = MonitoringConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));

        let monitor = TuiPerformanceMonitor::new(config, perf_manager);

        monitor
            .record_frame(Duration::from_millis(16))
            .await
            .unwrap();

        let frame_count = *monitor.frame_counter.read().await;
        assert_eq!(frame_count, 1);
    }

    #[tokio::test]
    async fn test_error_recording() {
        let config = MonitoringConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));

        let monitor = TuiPerformanceMonitor::new(config, perf_manager);

        monitor
            .record_error("test_component", "test error")
            .await
            .unwrap();

        let error_count = *monitor.error_counter.read().await;
        assert_eq!(error_count, 1);
    }

    #[tokio::test]
    async fn test_profiling_session() {
        let config = MonitoringConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));

        let monitor = TuiPerformanceMonitor::new(config, perf_manager);

        let session = monitor.start_profiling("test_function").await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        session.end().await;

        let profiling_data = monitor.profiling_data.read().await;
        assert!(profiling_data.contains_key("test_function"));

        let data = profiling_data.get("test_function").unwrap();
        assert_eq!(data.call_count, 1);
        assert!(data.total_time >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_performance_report_generation() {
        let config = MonitoringConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));

        let monitor = TuiPerformanceMonitor::new(config, perf_manager);

        // Add some test data
        monitor
            .record_frame(Duration::from_millis(16))
            .await
            .unwrap();
        monitor.record_error("test", "error").await.unwrap();

        let report = monitor.generate_report().await.unwrap();

        // Report should be generated successfully
        assert_eq!(report.summary.total_errors, 0); // No history yet, so summary shows defaults
    }

    #[tokio::test]
    async fn test_data_export() {
        let config = MonitoringConfig::default();
        let perf_config = PerformanceConfig::default();
        let perf_manager = Arc::new(PerformanceManager::new(perf_config));

        let monitor = TuiPerformanceMonitor::new(config, perf_manager);

        // Test JSON export
        let json_data = monitor.export_data(ExportFormat::Json).await.unwrap();
        assert!(json_data.starts_with('['));

        // Test CSV export
        let csv_data = monitor.export_data(ExportFormat::Csv).await.unwrap();
        assert!(csv_data.starts_with("timestamp,cpu_usage"));

        // Test Prometheus export
        let prometheus_data = monitor.export_data(ExportFormat::Prometheus).await.unwrap();
        assert!(prometheus_data.is_empty() || prometheus_data.contains("tui_"));
    }
}
