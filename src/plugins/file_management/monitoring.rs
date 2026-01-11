//! Monitoring and metrics for file management operations

use super::error::{FileManagementError, FileManagementResult};
use super::error_recovery::RecoveryStats;
use crate::performance::{MetricsCollector, PerformanceManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Monitoring configuration for file management operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,

    /// Metrics collection interval
    pub collection_interval_seconds: u64,

    /// Enable audit trail logging
    pub enable_audit_trail: bool,

    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,

    /// Enable error tracking
    pub enable_error_tracking: bool,

    /// Enable resource usage monitoring
    pub enable_resource_monitoring: bool,

    /// Maximum number of audit entries to keep in memory
    pub max_audit_entries: usize,

    /// Maximum number of performance samples to keep
    pub max_performance_samples: usize,

    /// Enable real-time alerts
    pub enable_alerts: bool,

    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_seconds: 30,
            enable_audit_trail: true,
            enable_performance_monitoring: true,
            enable_error_tracking: true,
            enable_resource_monitoring: true,
            max_audit_entries: 10000,
            max_performance_samples: 1000,
            enable_alerts: true,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Error rate threshold (errors per minute)
    pub error_rate_threshold: f64,

    /// Average operation duration threshold (milliseconds)
    pub avg_duration_threshold_ms: u64,

    /// Memory usage threshold (percentage)
    pub memory_usage_threshold: f64,

    /// Disk usage threshold (percentage)
    pub disk_usage_threshold: f64,

    /// Recovery failure rate threshold
    pub recovery_failure_rate_threshold: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            error_rate_threshold: 10.0,           // 10 errors per minute
            avg_duration_threshold_ms: 5000,      // 5 seconds
            memory_usage_threshold: 0.9,          // 90%
            disk_usage_threshold: 0.95,           // 95%
            recovery_failure_rate_threshold: 0.5, // 50%
        }
    }
}

/// File management monitoring and metrics collector
pub struct FileManagementMonitor {
    config: MonitoringConfig,
    performance_manager: Option<Arc<PerformanceManager>>,
    metrics_collector: Option<Arc<MetricsCollector>>,
    audit_trail: Arc<RwLock<Vec<AuditEntry>>>,
    operation_metrics: Arc<RwLock<OperationMetrics>>,
    error_tracker: Arc<RwLock<ErrorTracker>>,
    resource_monitor: Arc<RwLock<ResourceMonitor>>,
    alert_manager: Arc<RwLock<AlertManager>>,
}

/// Audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub tool_name: String,
    pub user_id: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub result: AuditResult,
    pub duration_ms: u64,
    pub resource_usage: ResourceUsage,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Error { error_type: String, message: String },
    Cancelled,
}

/// Resource usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: u64,
    pub cpu_percent: f64,
    pub disk_io_bytes: u64,
    pub network_io_bytes: u64,
}

/// Operation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub cancelled_operations: u64,
    pub total_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub operations_by_tool: HashMap<String, ToolMetrics>,
    pub operations_by_type: HashMap<String, u64>,
    pub hourly_stats: Vec<HourlyStats>,
}

/// Tool-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolMetrics {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Hourly statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: chrono::DateTime<chrono::Utc>,
    pub operations: u64,
    pub errors: u64,
    pub avg_duration_ms: f64,
    pub peak_memory_mb: u64,
}

/// Error tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorTracker {
    pub total_errors: u64,
    pub errors_by_type: HashMap<String, u64>,
    pub errors_by_tool: HashMap<String, u64>,
    pub recent_errors: Vec<ErrorEntry>,
    pub error_rate_per_minute: f64,
    pub recovery_stats: Option<RecoveryStats>,
}

/// Error entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error_type: String,
    pub tool_name: String,
    pub message: String,
    pub severity: String,
    pub recovered: bool,
}

/// Resource monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMonitor {
    pub current_memory_mb: u64,
    pub peak_memory_mb: u64,
    pub current_cpu_percent: f64,
    pub peak_cpu_percent: f64,
    pub disk_usage_percent: f64,
    pub network_io_total_bytes: u64,
    pub disk_io_total_bytes: u64,
    pub active_operations: u64,
    pub resource_history: Vec<ResourceSnapshot>,
}

/// Resource snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub memory_mb: u64,
    pub cpu_percent: f64,
    pub disk_usage_percent: f64,
    pub active_operations: u64,
}

/// Alert management
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertManager {
    pub active_alerts: Vec<Alert>,
    pub alert_history: Vec<Alert>,
    pub last_alert_check: Option<chrono::DateTime<chrono::Utc>>,
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Alert type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighErrorRate,
    SlowPerformance,
    HighMemoryUsage,
    HighDiskUsage,
    RecoveryFailure,
    SystemOverload,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl FileManagementMonitor {
    /// Create a new file management monitor
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            performance_manager: None,
            metrics_collector: None,
            audit_trail: Arc::new(RwLock::new(Vec::new())),
            operation_metrics: Arc::new(RwLock::new(OperationMetrics::default())),
            error_tracker: Arc::new(RwLock::new(ErrorTracker::default())),
            resource_monitor: Arc::new(RwLock::new(ResourceMonitor::default())),
            alert_manager: Arc::new(RwLock::new(AlertManager::default())),
        }
    }

    /// Create monitor with performance manager integration
    pub fn with_performance_manager(
        config: MonitoringConfig,
        performance_manager: Arc<PerformanceManager>,
        metrics_collector: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            config,
            performance_manager: Some(performance_manager),
            metrics_collector: Some(metrics_collector),
            audit_trail: Arc::new(RwLock::new(Vec::new())),
            operation_metrics: Arc::new(RwLock::new(OperationMetrics::default())),
            error_tracker: Arc::new(RwLock::new(ErrorTracker::default())),
            resource_monitor: Arc::new(RwLock::new(ResourceMonitor::default())),
            alert_manager: Arc::new(RwLock::new(AlertManager::default())),
        }
    }

    /// Start monitoring background tasks
    pub async fn start_monitoring(&self) -> FileManagementResult<()> {
        if !self.config.enabled {
            debug!("File management monitoring is disabled");
            return Ok(());
        }

        info!("Starting file management monitoring");

        // Start metrics collection task
        if self.config.enable_performance_monitoring {
            self.start_metrics_collection().await?;
        }

        // Start resource monitoring task
        if self.config.enable_resource_monitoring {
            self.start_resource_monitoring().await?;
        }

        // Start alert checking task
        if self.config.enable_alerts {
            self.start_alert_monitoring().await?;
        }

        info!("File management monitoring started successfully");
        Ok(())
    }

    /// Record an operation in the audit trail
    pub async fn record_operation(
        &self,
        operation: &str,
        tool_name: &str,
        user_id: Option<&str>,
        parameters: HashMap<String, serde_json::Value>,
        result: AuditResult,
        duration: Duration,
        resource_usage: ResourceUsage,
    ) -> FileManagementResult<()> {
        if !self.config.enable_audit_trail {
            return Ok(());
        }

        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            operation: operation.to_string(),
            tool_name: tool_name.to_string(),
            user_id: user_id.map(|s| s.to_string()),
            parameters,
            result: result.clone(),
            duration_ms: duration.as_millis() as u64,
            resource_usage,
        };

        // Add to audit trail
        {
            let mut audit_trail = self.audit_trail.write().map_err(|_| {
                FileManagementError::concurrency("Failed to acquire write lock on audit trail")
            })?;

            audit_trail.push(entry);

            // Trim if necessary
            if audit_trail.len() > self.config.max_audit_entries {
                audit_trail.remove(0);
            }
        }

        // Update operation metrics
        self.update_operation_metrics(tool_name, &result, duration)
            .await?;

        debug!("Recorded operation: {} by tool: {}", operation, tool_name);
        Ok(())
    }

    /// Record an error
    pub async fn record_error(
        &self,
        error: &FileManagementError,
        tool_name: &str,
        recovered: bool,
    ) -> FileManagementResult<()> {
        if !self.config.enable_error_tracking {
            return Ok(());
        }

        let error_entry = ErrorEntry {
            timestamp: chrono::Utc::now(),
            error_type: error.category().to_string(),
            tool_name: tool_name.to_string(),
            message: error.to_string(),
            severity: format!("{:?}", error.severity()),
            recovered,
        };

        {
            let mut error_tracker = self.error_tracker.write().map_err(|_| {
                FileManagementError::concurrency("Failed to acquire write lock on error tracker")
            })?;

            error_tracker.total_errors += 1;
            *error_tracker
                .errors_by_type
                .entry(error.category().to_string())
                .or_insert(0) += 1;
            *error_tracker
                .errors_by_tool
                .entry(tool_name.to_string())
                .or_insert(0) += 1;
            error_tracker.recent_errors.push(error_entry);

            // Keep only recent errors (last 1000)
            if error_tracker.recent_errors.len() > 1000 {
                error_tracker.recent_errors.remove(0);
            }

            // Update error rate
            self.calculate_error_rate(&mut error_tracker);
        }

        debug!(
            "Recorded error: {} from tool: {}",
            error.category(),
            tool_name
        );
        Ok(())
    }

    /// Update recovery statistics
    pub async fn update_recovery_stats(&self, stats: &RecoveryStats) -> FileManagementResult<()> {
        let mut error_tracker = self.error_tracker.write().map_err(|_| {
            FileManagementError::concurrency("Failed to acquire write lock on error tracker")
        })?;

        error_tracker.recovery_stats = Some(stats.clone());
        Ok(())
    }

    /// Get current monitoring statistics
    pub async fn get_monitoring_stats(&self) -> FileManagementResult<MonitoringStats> {
        let operation_metrics = self
            .operation_metrics
            .read()
            .map_err(|_| {
                FileManagementError::concurrency("Failed to acquire read lock on operation metrics")
            })?
            .clone();

        let error_tracker = self
            .error_tracker
            .read()
            .map_err(|_| {
                FileManagementError::concurrency("Failed to acquire read lock on error tracker")
            })?
            .clone();

        let resource_monitor = self
            .resource_monitor
            .read()
            .map_err(|_| {
                FileManagementError::concurrency("Failed to acquire read lock on resource monitor")
            })?
            .clone();

        let alert_manager = self
            .alert_manager
            .read()
            .map_err(|_| {
                FileManagementError::concurrency("Failed to acquire read lock on alert manager")
            })?
            .clone();

        Ok(MonitoringStats {
            operation_metrics,
            error_tracker,
            resource_monitor,
            active_alerts: alert_manager.active_alerts,
            monitoring_enabled: self.config.enabled,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Get audit trail entries
    pub async fn get_audit_trail(
        &self,
        limit: Option<usize>,
    ) -> FileManagementResult<Vec<AuditEntry>> {
        let audit_trail = self.audit_trail.read().map_err(|_| {
            FileManagementError::concurrency("Failed to acquire read lock on audit trail")
        })?;

        let entries = if let Some(limit) = limit {
            audit_trail.iter().rev().take(limit).cloned().collect()
        } else {
            audit_trail.clone()
        };

        Ok(entries)
    }

    /// Start metrics collection background task
    async fn start_metrics_collection(&self) -> FileManagementResult<()> {
        let interval_duration = Duration::from_secs(self.config.collection_interval_seconds);
        let mut interval = interval(interval_duration);

        // This would typically spawn a background task
        // For now, we'll just log that it would start
        info!(
            "Metrics collection task would start with interval: {:?}",
            interval_duration
        );

        Ok(())
    }

    /// Start resource monitoring background task
    async fn start_resource_monitoring(&self) -> FileManagementResult<()> {
        info!("Resource monitoring task would start");
        Ok(())
    }

    /// Start alert monitoring background task
    async fn start_alert_monitoring(&self) -> FileManagementResult<()> {
        info!("Alert monitoring task would start");
        Ok(())
    }

    /// Update operation metrics
    async fn update_operation_metrics(
        &self,
        tool_name: &str,
        result: &AuditResult,
        duration: Duration,
    ) -> FileManagementResult<()> {
        let mut metrics = self.operation_metrics.write().map_err(|_| {
            FileManagementError::concurrency("Failed to acquire write lock on operation metrics")
        })?;

        metrics.total_operations += 1;
        let duration_ms = duration.as_millis() as u64;
        metrics.total_duration_ms += duration_ms;

        // Update min/max duration
        if metrics.min_duration_ms == 0 || duration_ms < metrics.min_duration_ms {
            metrics.min_duration_ms = duration_ms;
        }
        if duration_ms > metrics.max_duration_ms {
            metrics.max_duration_ms = duration_ms;
        }

        // Update result counters
        match result {
            AuditResult::Success => metrics.successful_operations += 1,
            AuditResult::Error { .. } => metrics.failed_operations += 1,
            AuditResult::Cancelled => metrics.cancelled_operations += 1,
        }

        // Update tool-specific metrics
        let tool_metrics = metrics
            .operations_by_tool
            .entry(tool_name.to_string())
            .or_default();
        tool_metrics.total_calls += 1;
        tool_metrics.total_duration_ms += duration_ms;
        tool_metrics.avg_duration_ms =
            tool_metrics.total_duration_ms as f64 / tool_metrics.total_calls as f64;
        tool_metrics.last_used = Some(chrono::Utc::now());

        match result {
            AuditResult::Success => tool_metrics.successful_calls += 1,
            AuditResult::Error { .. } => tool_metrics.failed_calls += 1,
            AuditResult::Cancelled => {} // Don't count cancelled as failed
        }

        Ok(())
    }

    /// Calculate error rate per minute
    fn calculate_error_rate(&self, error_tracker: &mut ErrorTracker) {
        let now = chrono::Utc::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);

        let recent_errors = error_tracker
            .recent_errors
            .iter()
            .filter(|e| e.timestamp > one_minute_ago)
            .count();

        error_tracker.error_rate_per_minute = recent_errors as f64;
    }
}

/// Complete monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub operation_metrics: OperationMetrics,
    pub error_tracker: ErrorTracker,
    pub resource_monitor: ResourceMonitor,
    pub active_alerts: Vec<Alert>,
    pub monitoring_enabled: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_monitor_creation() {
        let config = MonitoringConfig::default();
        let monitor = FileManagementMonitor::new(config);

        let stats = monitor.get_monitoring_stats().await.unwrap();
        assert_eq!(stats.operation_metrics.total_operations, 0);
        assert!(stats.monitoring_enabled);
    }

    #[tokio::test]
    async fn test_operation_recording() {
        let config = MonitoringConfig::default();
        let monitor = FileManagementMonitor::new(config);

        let mut params = HashMap::new();
        params.insert(
            "test_param".to_string(),
            serde_json::Value::String("test_value".to_string()),
        );

        let resource_usage = ResourceUsage {
            memory_mb: 100,
            cpu_percent: 50.0,
            disk_io_bytes: 1024,
            network_io_bytes: 0,
        };

        monitor
            .record_operation(
                "test_operation",
                "test_tool",
                Some("test_user"),
                params,
                AuditResult::Success,
                Duration::from_millis(500),
                resource_usage,
            )
            .await
            .unwrap();

        let stats = monitor.get_monitoring_stats().await.unwrap();
        assert_eq!(stats.operation_metrics.total_operations, 1);
        assert_eq!(stats.operation_metrics.successful_operations, 1);

        let audit_trail = monitor.get_audit_trail(None).await.unwrap();
        assert_eq!(audit_trail.len(), 1);
        assert_eq!(audit_trail[0].operation, "test_operation");
    }

    #[tokio::test]
    async fn test_error_recording() {
        let config = MonitoringConfig::default();
        let monitor = FileManagementMonitor::new(config);

        let error = FileManagementError::not_found("/test/path");
        monitor
            .record_error(&error, "test_tool", false)
            .await
            .unwrap();

        let stats = monitor.get_monitoring_stats().await.unwrap();
        assert_eq!(stats.error_tracker.total_errors, 1);
        assert_eq!(stats.error_tracker.recent_errors.len(), 1);
        assert_eq!(stats.error_tracker.recent_errors[0].error_type, "not_found");
    }
}
