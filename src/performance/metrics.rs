//! Performance metrics collection and reporting

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::memory::MemoryUsage;

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics collection interval
    pub collection_interval: Duration,

    /// Maximum number of data points to keep
    pub max_data_points: usize,

    /// Enable Prometheus metrics export
    pub prometheus_enabled: bool,

    /// Prometheus metrics port
    pub prometheus_port: u16,

    /// Custom metrics configuration
    pub custom_metrics: HashMap<String, CustomMetricConfig>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(10),
            max_data_points: 1000,
            prometheus_enabled: false,
            prometheus_port: 9090,
            custom_metrics: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricConfig {
    pub metric_type: MetricType,
    pub description: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Performance metrics collector
pub struct MetricsCollector {
    config: Arc<RwLock<MetricsConfig>>,

    // Core metrics
    execution_metrics: Arc<DashMap<String, ExecutionMetrics>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
    custom_metrics: Arc<DashMap<String, CustomMetric>>,

    // Time series data
    time_series: Arc<DashMap<String, TimeSeries>>,

    // Collection state
    #[allow(dead_code)]
    last_collection: Arc<RwLock<Instant>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            execution_metrics: Arc::new(DashMap::new()),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            custom_metrics: Arc::new(DashMap::new()),
            time_series: Arc::new(DashMap::new()),
            last_collection: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Record execution metrics
    pub async fn record_execution(
        &self,
        component: &str,
        duration: Duration,
        memory_usage: MemoryUsage,
    ) {
        let config = self.config.read().await;
        if !config.enabled {
            return;
        }

        let mut metrics = self
            .execution_metrics
            .entry(component.to_string())
            .or_insert_with(ExecutionMetrics::default);

        metrics.total_executions += 1;
        metrics.total_duration += duration;
        metrics.average_duration = metrics.total_duration / metrics.total_executions as u32;
        metrics.min_duration = metrics.min_duration.min(duration);
        metrics.max_duration = metrics.max_duration.max(duration);

        // Update memory metrics
        metrics.total_memory_allocated += memory_usage.allocated;
        metrics.peak_memory_usage = metrics.peak_memory_usage.max(memory_usage.peak_usage);

        // Record in time series
        self.record_time_series(
            &format!("{}_duration", component),
            duration.as_millis() as f64,
        )
        .await;
        self.record_time_series(
            &format!("{}_memory", component),
            memory_usage.allocated as f64,
        )
        .await;
    }

    /// Record custom metric
    pub async fn record_custom_metric(
        &self,
        name: &str,
        value: f64,
        labels: Option<HashMap<String, String>>,
    ) {
        let config = self.config.read().await;
        if !config.enabled {
            return;
        }

        let labels_clone = labels.clone();
        let mut metric = self
            .custom_metrics
            .entry(name.to_string())
            .or_insert_with(|| CustomMetric {
                name: name.to_string(),
                metric_type: MetricType::Gauge,
                value: 0.0,
                labels: labels_clone.unwrap_or_default(),
                last_updated_millis: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });

        metric.value = value;
        metric.last_updated_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        if let Some(new_labels) = labels {
            metric.labels = new_labels;
        }

        // Record in time series
        self.record_time_series(name, value).await;
    }

    /// Record counter increment
    pub async fn increment_counter(&self, name: &str, increment: f64) {
        let mut metric = self
            .custom_metrics
            .entry(name.to_string())
            .or_insert_with(|| CustomMetric {
                name: name.to_string(),
                metric_type: MetricType::Counter,
                value: 0.0,
                labels: HashMap::new(),
                last_updated_millis: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });

        metric.value += increment;
        metric.last_updated_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Record in time series
        self.record_time_series(name, metric.value).await;
    }

    /// Update gauge value
    pub async fn set_gauge(&self, name: &str, value: f64) {
        self.record_custom_metric(name, value, None).await;
    }

    /// Record histogram observation
    pub async fn observe_histogram(&self, name: &str, value: f64) {
        // For simplicity, we'll treat histograms as time series for now
        self.record_time_series(name, value).await;
    }

    /// Collect system metrics
    pub async fn collect_system_metrics(&self) -> SystemMetrics {
        let mut metrics = SystemMetrics::default();

        // Collect CPU usage
        metrics.cpu_usage = self.get_cpu_usage().await;

        // Collect memory usage
        metrics.memory_usage = self.get_memory_usage().await;

        // Collect disk usage
        metrics.disk_usage = self.get_disk_usage().await;

        // Collect network usage
        metrics.network_usage = self.get_network_usage().await;

        // Update stored metrics
        let mut stored_metrics = self.system_metrics.write().await;
        *stored_metrics = metrics.clone();

        metrics
    }

    /// Get execution metrics for a component
    pub async fn get_execution_metrics(&self, component: &str) -> Option<ExecutionMetrics> {
        self.execution_metrics
            .get(component)
            .map(|entry| entry.clone())
    }

    /// Get all execution metrics
    pub async fn get_all_execution_metrics(&self) -> HashMap<String, ExecutionMetrics> {
        self.execution_metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Get custom metric
    pub async fn get_custom_metric(&self, name: &str) -> Option<CustomMetric> {
        self.custom_metrics.get(name).map(|entry| entry.clone())
    }

    /// Get all custom metrics
    pub async fn get_all_custom_metrics(&self) -> HashMap<String, CustomMetric> {
        self.custom_metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Get time series data
    pub async fn get_time_series(&self, name: &str) -> Option<TimeSeries> {
        self.time_series.get(name).map(|entry| entry.clone())
    }

    /// Get metrics summary
    pub async fn get_metrics_summary(&self) -> MetricsSummary {
        let execution_metrics = self.get_all_execution_metrics().await;
        let custom_metrics = self.get_all_custom_metrics().await;
        let system_metrics = self.system_metrics.read().await.clone();

        MetricsSummary {
            execution_metrics,
            custom_metrics,
            system_metrics,
            collection_timestamp_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Export execution metrics
        for entry in self.execution_metrics.iter() {
            let component = entry.key();
            let metrics = entry.value();

            output.push_str(&format!(
                "# HELP {}_executions_total Total number of executions\n",
                component
            ));
            output.push_str(&format!("# TYPE {}_executions_total counter\n", component));
            output.push_str(&format!(
                "{}_executions_total {}\n",
                component, metrics.total_executions
            ));

            output.push_str(&format!(
                "# HELP {}_duration_seconds Execution duration in seconds\n",
                component
            ));
            output.push_str(&format!(
                "# TYPE {}_duration_seconds histogram\n",
                component
            ));
            output.push_str(&format!(
                "{}_duration_seconds_sum {}\n",
                component,
                metrics.total_duration.as_secs_f64()
            ));
            output.push_str(&format!(
                "{}_duration_seconds_count {}\n",
                component, metrics.total_executions
            ));
        }

        // Export custom metrics
        for entry in self.custom_metrics.iter() {
            let name = entry.key();
            let metric = entry.value();

            let metric_type = match metric.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
                MetricType::Summary => "summary",
            };

            output.push_str(&format!("# TYPE {} {}\n", name, metric_type));

            let labels = if metric.labels.is_empty() {
                String::new()
            } else {
                let label_pairs: Vec<String> = metric
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", label_pairs.join(","))
            };

            output.push_str(&format!("{}{} {}\n", name, labels, metric.value));
        }

        output
    }

    async fn record_time_series(&self, name: &str, value: f64) {
        let config = self.config.read().await;
        let mut series = self
            .time_series
            .entry(name.to_string())
            .or_insert_with(TimeSeries::new);

        series.add_point(TimeSeriesPoint {
            timestamp_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            value,
        });

        // Limit data points
        if series.points.len() > config.max_data_points {
            let len = series.points.len();
            series.points.drain(0..len - config.max_data_points);
        }
    }

    async fn get_cpu_usage(&self) -> f64 {
        // Mock CPU usage - in real implementation, would query system
        0.65 // 65%
    }

    async fn get_memory_usage(&self) -> f64 {
        // Mock memory usage - in real implementation, would query system
        0.72 // 72%
    }

    async fn get_disk_usage(&self) -> f64 {
        // Mock disk usage - in real implementation, would query system
        0.45 // 45%
    }

    async fn get_network_usage(&self) -> NetworkUsage {
        // Mock network usage - in real implementation, would query system
        NetworkUsage {
            bytes_sent: 1024 * 1024,         // 1MB
            bytes_received: 2 * 1024 * 1024, // 2MB
            packets_sent: 1000,
            packets_received: 1500,
        }
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionMetrics {
    pub total_executions: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_memory_allocated: usize,
    pub peak_memory_usage: usize,
    pub success_rate: f64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_usage: NetworkUsage,
    pub load_average: f64,
    pub uptime: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkUsage {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub last_updated_millis: u64, // Unix timestamp in milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    pub points: Vec<TimeSeriesPoint>,
}

impl TimeSeries {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    pub fn add_point(&mut self, point: TimeSeriesPoint) {
        self.points.push(point);
    }

    pub fn get_latest(&self) -> Option<&TimeSeriesPoint> {
        self.points.last()
    }

    pub fn get_average(&self) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.points.iter().map(|p| p.value).sum();
        sum / self.points.len() as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp_millis: u64, // Unix timestamp in milliseconds
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub execution_metrics: HashMap<String, ExecutionMetrics>,
    pub custom_metrics: HashMap<String, CustomMetric>,
    pub system_metrics: SystemMetrics,
    pub collection_timestamp_millis: u64, // Unix timestamp in milliseconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        let summary = collector.get_metrics_summary().await;
        assert!(summary.execution_metrics.is_empty());
    }

    #[tokio::test]
    async fn test_execution_metrics_recording() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        let memory_usage = MemoryUsage {
            initial: 1000,
            final_usage: 1500,
            peak_usage: 2000,
            allocated: 500,
        };

        collector
            .record_execution("test_component", Duration::from_millis(100), memory_usage)
            .await;

        let metrics = collector
            .get_execution_metrics("test_component")
            .await
            .unwrap();
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.total_duration, Duration::from_millis(100));
        assert_eq!(metrics.total_memory_allocated, 500);
    }

    #[tokio::test]
    async fn test_custom_metrics() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        collector.increment_counter("test_counter", 5.0).await;
        collector.set_gauge("test_gauge", 42.0).await;

        let counter = collector.get_custom_metric("test_counter").await.unwrap();
        assert_eq!(counter.value, 5.0);

        let gauge = collector.get_custom_metric("test_gauge").await.unwrap();
        assert_eq!(gauge.value, 42.0);
    }

    #[tokio::test]
    async fn test_time_series() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        collector.record_time_series("test_metric", 10.0).await;
        collector.record_time_series("test_metric", 20.0).await;
        collector.record_time_series("test_metric", 30.0).await;

        let series = collector.get_time_series("test_metric").await.unwrap();
        assert_eq!(series.points.len(), 3);
        assert_eq!(series.get_average(), 20.0);
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        let memory_usage = MemoryUsage::default();
        collector
            .record_execution("test", Duration::from_millis(100), memory_usage)
            .await;
        collector.increment_counter("test_counter", 1.0).await;

        let prometheus_output = collector.export_prometheus().await;
        assert!(prometheus_output.contains("test_executions_total"));
        assert!(prometheus_output.contains("test_counter"));
    }
}
