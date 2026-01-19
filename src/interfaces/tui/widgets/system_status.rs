//! System Status Widget
//!
//! Displays system resource usage, health status, and performance metrics.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, ScrollbarState, Wrap},
    Frame,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::super::action::Action;
use super::super::system_monitor::{
    CpuInfo, DiskInfo, MemoryInfo, ProcessInfo, ProcessSortBy, SystemMonitor,
};
use super::super::theme::Theme;
use super::super::widget::{
    SizeConstraints, UpdateFrequency, Widget, WidgetCapabilities, WidgetContext, WidgetError,
    WidgetId,
};
use crate::core::{NetworkStatus, SystemHealth, SystemHealthAssessment, SystemStatus};

/// Historical data point for performance charts
#[derive(Debug, Clone)]
struct DataPoint {
    timestamp: DateTime<Utc>,
    cpu_usage: f64,
    memory_usage: f64,
    disk_usage: f64,
    #[allow(dead_code)]
    network_activity: f64,
}

/// Display mode for the system status widget
#[derive(Debug, Clone, PartialEq, Eq)]
enum DisplayMode {
    Overview,
    DetailedMetrics,
    PerformanceCharts,
    SystemInfo,
    Alerts,
    Maintenance,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum AlertSeverity {
    #[allow(dead_code)]
    Info,
    Warning,
    Critical,
}

/// System alert information
#[derive(Debug, Clone)]
struct SystemAlert {
    severity: AlertSeverity,
    message: String,
    timestamp: DateTime<Utc>,
    category: String,
}

/// Threshold configuration for resource monitoring
#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    pub cpu_warning: f64,
    pub cpu_critical: f64,
    pub memory_warning: f64,
    pub memory_critical: f64,
    pub disk_warning: f64,
    pub disk_critical: f64,
}

/// Alert statistics
#[derive(Debug, Clone)]
pub struct AlertStats {
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub info_alerts: usize,
    pub recent_alerts: usize,
}

/// Maintenance recommendation priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaintenancePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Maintenance action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaintenanceAction {
    CleanAlerts,
    CleanMemory,
    CleanDisk,
    OptimizeProcesses,
    Restart,
    UpdateSystem,
}

/// Maintenance recommendation
#[derive(Debug, Clone)]
pub struct MaintenanceRecommendation {
    pub priority: MaintenancePriority,
    pub category: String,
    pub title: String,
    pub description: String,
    pub action: MaintenanceAction,
}

/// System diagnostics information
#[derive(Debug, Clone)]
pub struct SystemDiagnostics {
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
    pub overall_health: SystemHealth,
    pub diagnostics_time: DateTime<Utc>,
}

/// System Status Widget implementation
pub struct SystemStatusWidget {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,

    // System monitor
    system_monitor: SystemMonitor,

    // Widget state
    current_status: SystemStatus,
    display_mode: DisplayMode,
    selected_index: usize,
    #[allow(dead_code)]
    scroll_offset: usize,

    // Historical data for charts
    history: VecDeque<DataPoint>,
    max_history_points: usize,

    // Alerts and warnings
    alerts: Vec<SystemAlert>,
    max_alerts: usize,

    // Thresholds for warnings
    cpu_warning_threshold: f64,
    cpu_critical_threshold: f64,
    memory_warning_threshold: f64,
    memory_critical_threshold: f64,
    disk_warning_threshold: f64,
    disk_critical_threshold: f64,

    // UI state
    last_update: Option<Instant>,
    update_interval: Duration,
    show_help: bool,

    // Scrollbar state
    #[allow(dead_code)]
    scrollbar_state: ScrollbarState,

    // Detailed system information
    cpu_info: Option<CpuInfo>,
    memory_info: Option<MemoryInfo>,
    disk_info: Vec<DiskInfo>,
    top_processes: Vec<ProcessInfo>,
    health_assessment: Option<SystemHealthAssessment>,
}

impl SystemStatusWidget {
    /// Create a new SystemStatusWidget
    pub fn new() -> Self {
        let context = WidgetContext::new(WidgetId::from("system_status"));

        let capabilities = WidgetCapabilities {
            keyboard_input: true,
            mouse_input: false,
            focusable: true,
            resizable: true,
            scrollable: true,
            themeable: true,
            configurable: true,
        };

        let size_constraints = SizeConstraints::new()
            .min_size(60, 20)
            .preferred_size(100, 30);

        Self {
            context,
            capabilities,
            size_constraints,
            system_monitor: SystemMonitor::new(),
            current_status: SystemStatus::default(),
            display_mode: DisplayMode::Overview,
            selected_index: 0,
            scroll_offset: 0,
            history: VecDeque::new(),
            max_history_points: 100,
            alerts: Vec::new(),
            max_alerts: 50,
            cpu_warning_threshold: 70.0,
            cpu_critical_threshold: 90.0,
            memory_warning_threshold: 80.0,
            memory_critical_threshold: 95.0,
            disk_warning_threshold: 85.0,
            disk_critical_threshold: 95.0,
            last_update: None,
            update_interval: Duration::from_secs(2),
            show_help: false,
            scrollbar_state: ScrollbarState::default(),
            cpu_info: None,
            memory_info: None,
            disk_info: Vec::new(),
            top_processes: Vec::new(),
            health_assessment: None,
        }
    }

    /// Update system status data
    pub async fn update_status(&mut self) {
        // Get current system status from the monitor
        self.current_status = self.system_monitor.get_system_status().await;

        // Add to history
        let data_point = DataPoint {
            timestamp: Utc::now(),
            cpu_usage: self.current_status.cpu_usage,
            memory_usage: self.current_status.memory_usage,
            disk_usage: self.current_status.disk_usage,
            network_activity: 0.0, // TODO: Add network activity tracking
        };

        self.history.push_back(data_point);
        if self.history.len() > self.max_history_points {
            self.history.pop_front();
        }

        // Check for alerts (clone the status to avoid borrowing issues)
        let current_status = self.current_status.clone();
        self.check_and_add_alerts(&current_status);

        self.last_update = Some(Instant::now());
    }

    /// Update configurable thresholds
    pub fn update_thresholds(
        &mut self,
        cpu_warning: Option<f64>,
        cpu_critical: Option<f64>,
        memory_warning: Option<f64>,
        memory_critical: Option<f64>,
        disk_warning: Option<f64>,
        disk_critical: Option<f64>,
    ) {
        if let Some(threshold) = cpu_warning {
            self.cpu_warning_threshold = threshold.clamp(0.0, 100.0);
        }
        if let Some(threshold) = cpu_critical {
            self.cpu_critical_threshold = threshold.clamp(0.0, 100.0);
        }
        if let Some(threshold) = memory_warning {
            self.memory_warning_threshold = threshold.clamp(0.0, 100.0);
        }
        if let Some(threshold) = memory_critical {
            self.memory_critical_threshold = threshold.clamp(0.0, 100.0);
        }
        if let Some(threshold) = disk_warning {
            self.disk_warning_threshold = threshold.clamp(0.0, 100.0);
        }
        if let Some(threshold) = disk_critical {
            self.disk_critical_threshold = threshold.clamp(0.0, 100.0);
        }
    }

    /// Get current threshold configuration
    pub fn get_thresholds(&self) -> ThresholdConfig {
        ThresholdConfig {
            cpu_warning: self.cpu_warning_threshold,
            cpu_critical: self.cpu_critical_threshold,
            memory_warning: self.memory_warning_threshold,
            memory_critical: self.memory_critical_threshold,
            disk_warning: self.disk_warning_threshold,
            disk_critical: self.disk_critical_threshold,
        }
    }

    /// Get alert statistics
    pub fn get_alert_stats(&self) -> AlertStats {
        let mut stats = AlertStats {
            total_alerts: self.alerts.len(),
            critical_alerts: 0,
            warning_alerts: 0,
            info_alerts: 0,
            recent_alerts: 0,
        };

        let recent_threshold = Utc::now() - chrono::Duration::minutes(30);

        for alert in &self.alerts {
            match alert.severity {
                AlertSeverity::Critical => stats.critical_alerts += 1,
                AlertSeverity::Warning => stats.warning_alerts += 1,
                AlertSeverity::Info => stats.info_alerts += 1,
            }

            if alert.timestamp > recent_threshold {
                stats.recent_alerts += 1;
            }
        }

        stats
    }

    /// Clear all alerts
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    /// Clear alerts older than specified duration
    pub fn clear_old_alerts(&mut self, max_age: chrono::Duration) {
        let cutoff = Utc::now() - max_age;
        self.alerts.retain(|alert| alert.timestamp > cutoff);
    }

    /// Get system maintenance recommendations
    pub fn get_maintenance_recommendations(&self) -> Vec<MaintenanceRecommendation> {
        let mut recommendations = Vec::new();

        // Check system uptime
        if self.current_status.uptime.as_secs() > 7 * 24 * 3600 {
            recommendations.push(MaintenanceRecommendation {
                priority: MaintenancePriority::Low,
                category: "System".to_string(),
                title: "长时间运行".to_string(),
                description: format!(
                    "系统已运行 {}，建议考虑重启以应用更新和清理内存",
                    Self::format_duration(self.current_status.uptime)
                ),
                action: MaintenanceAction::Restart,
            });
        }

        // Check memory usage
        if self.current_status.memory_usage > 85.0 {
            recommendations.push(MaintenanceRecommendation {
                priority: MaintenancePriority::Medium,
                category: "Memory".to_string(),
                title: "内存使用率高".to_string(),
                description: format!(
                    "内存使用率达到 {:.1}%，建议清理内存或关闭不必要的程序",
                    self.current_status.memory_usage
                ),
                action: MaintenanceAction::CleanMemory,
            });
        }

        // Check disk usage
        if self.current_status.disk_usage > 90.0 {
            recommendations.push(MaintenanceRecommendation {
                priority: MaintenancePriority::High,
                category: "Storage".to_string(),
                title: "磁盘空间不足".to_string(),
                description: format!(
                    "磁盘使用率达到 {:.1}%，建议清理临时文件和日志",
                    self.current_status.disk_usage
                ),
                action: MaintenanceAction::CleanDisk,
            });
        }

        // Check for too many alerts
        if self.alerts.len() > 20 {
            recommendations.push(MaintenanceRecommendation {
                priority: MaintenancePriority::Low,
                category: "Monitoring".to_string(),
                title: "警告过多".to_string(),
                description: format!("系统有 {} 个警告，建议清理旧警告", self.alerts.len()),
                action: MaintenanceAction::CleanAlerts,
            });
        }

        // Check process count
        if self.current_status.process_count > 500 {
            recommendations.push(MaintenanceRecommendation {
                priority: MaintenancePriority::Medium,
                category: "Processes".to_string(),
                title: "进程数量多".to_string(),
                description: format!(
                    "系统运行 {} 个进程，可能影响性能",
                    self.current_status.process_count
                ),
                action: MaintenanceAction::OptimizeProcesses,
            });
        }

        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        recommendations
    }

    /// Perform system maintenance action
    pub async fn perform_maintenance(
        &mut self,
        action: MaintenanceAction,
    ) -> Result<String, String> {
        match action {
            MaintenanceAction::CleanAlerts => {
                let old_count = self.alerts.len();
                self.clear_old_alerts(chrono::Duration::hours(1));
                Ok(format!("已清理 {} 个旧警告", old_count - self.alerts.len()))
            }
            MaintenanceAction::CleanMemory => {
                // In a real implementation, this might trigger garbage collection
                // or suggest closing memory-intensive applications
                Ok("内存清理建议已生成，请手动关闭不必要的程序".to_string())
            }
            MaintenanceAction::CleanDisk => {
                // In a real implementation, this might clean temporary files
                Ok("磁盘清理建议已生成，请手动清理临时文件和日志".to_string())
            }
            MaintenanceAction::OptimizeProcesses => {
                Ok("进程优化建议已生成，请检查运行中的程序".to_string())
            }
            MaintenanceAction::Restart => Err("系统重启需要管理员权限，请手动执行".to_string()),
            MaintenanceAction::UpdateSystem => {
                Err("系统更新需要管理员权限，请手动执行".to_string())
            }
        }
    }

    /// Get system diagnostics information
    pub fn get_system_diagnostics(&self) -> SystemDiagnostics {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();

        // Check critical issues
        if self.current_status.system_health == SystemHealth::Critical {
            issues.push("系统健康状态为严重".to_string());
        }

        if self.current_status.disk_usage > 95.0 {
            issues.push(format!(
                "磁盘空间严重不足: {:.1}%",
                self.current_status.disk_usage
            ));
        }

        if self.current_status.memory_usage > 95.0 {
            issues.push(format!(
                "内存使用率过高: {:.1}%",
                self.current_status.memory_usage
            ));
        }

        // Check warnings
        if self.current_status.cpu_usage > 80.0 {
            warnings.push(format!(
                "CPU使用率较高: {:.1}%",
                self.current_status.cpu_usage
            ));
        }

        if self.current_status.memory_usage > 80.0 {
            warnings.push(format!(
                "内存使用率较高: {:.1}%",
                self.current_status.memory_usage
            ));
        }

        if self.current_status.disk_usage > 85.0 {
            warnings.push(format!(
                "磁盘使用率较高: {:.1}%",
                self.current_status.disk_usage
            ));
        }

        if matches!(
            self.current_status.network_status,
            NetworkStatus::Disconnected
        ) {
            warnings.push("网络连接断开".to_string());
        }

        // Add informational items
        info.push(format!(
            "系统运行时间: {}",
            Self::format_duration(self.current_status.uptime)
        ));
        info.push(format!("活跃进程: {}", self.current_status.process_count));
        info.push(format!("系统负载: {:.2}", self.current_status.load_average));

        if let Some(ref cpu_info) = self.cpu_info {
            info.push(format!(
                "CPU: {} ({} 核心)",
                cpu_info.name, cpu_info.core_count
            ));
        }

        if let Some(ref memory_info) = self.memory_info {
            info.push(format!(
                "内存: {} / {}",
                Self::format_bytes(memory_info.used),
                Self::format_bytes(memory_info.total)
            ));
        }

        SystemDiagnostics {
            issues,
            warnings,
            info,
            overall_health: self.current_status.system_health,
            diagnostics_time: Utc::now(),
        }
    }

    /// Refresh detailed system information
    pub async fn refresh_detailed_info(&mut self) {
        // Get detailed CPU info
        self.cpu_info = Some(self.system_monitor.get_cpu_info().await);

        // Get detailed memory info
        self.memory_info = Some(self.system_monitor.get_memory_info().await);

        // Get disk info
        self.disk_info = self.system_monitor.get_disk_info().await;

        // Get top processes
        self.top_processes = self
            .system_monitor
            .get_top_processes(10, ProcessSortBy::Cpu)
            .await;

        // Get comprehensive health assessment
        self.health_assessment = Some(self.system_monitor.get_system_health_assessment().await);
    }

    /// Check system status and add alerts if thresholds are exceeded
    fn check_and_add_alerts(&mut self, status: &SystemStatus) {
        let now = Utc::now();

        // CPU alerts
        if status.cpu_usage >= self.cpu_critical_threshold {
            self.add_alert(
                AlertSeverity::Critical,
                format!("CPU usage critical: {:.1}%", status.cpu_usage),
                "CPU".to_string(),
                now,
            );
        } else if status.cpu_usage >= self.cpu_warning_threshold {
            self.add_alert(
                AlertSeverity::Warning,
                format!("CPU usage high: {:.1}%", status.cpu_usage),
                "CPU".to_string(),
                now,
            );
        }

        // Memory alerts
        if status.memory_usage >= self.memory_critical_threshold {
            self.add_alert(
                AlertSeverity::Critical,
                format!("Memory usage critical: {:.1}%", status.memory_usage),
                "Memory".to_string(),
                now,
            );
        } else if status.memory_usage >= self.memory_warning_threshold {
            self.add_alert(
                AlertSeverity::Warning,
                format!("Memory usage high: {:.1}%", status.memory_usage),
                "Memory".to_string(),
                now,
            );
        }

        // Disk alerts
        if status.disk_usage >= self.disk_critical_threshold {
            self.add_alert(
                AlertSeverity::Critical,
                format!("Disk usage critical: {:.1}%", status.disk_usage),
                "Disk".to_string(),
                now,
            );
        } else if status.disk_usage >= self.disk_warning_threshold {
            self.add_alert(
                AlertSeverity::Warning,
                format!("Disk usage high: {:.1}%", status.disk_usage),
                "Disk".to_string(),
                now,
            );
        }

        // System health alerts
        match status.system_health {
            SystemHealth::Critical => {
                self.add_alert(
                    AlertSeverity::Critical,
                    "System health is critical".to_string(),
                    "System".to_string(),
                    now,
                );
            }
            SystemHealth::Warning => {
                self.add_alert(
                    AlertSeverity::Warning,
                    "System health warning".to_string(),
                    "System".to_string(),
                    now,
                );
            }
            SystemHealth::Healthy => {}
        }

        // Network alerts
        if matches!(status.network_status, NetworkStatus::Disconnected) {
            self.add_alert(
                AlertSeverity::Warning,
                "Network disconnected".to_string(),
                "Network".to_string(),
                now,
            );
        }
    }

    /// Add a new alert
    fn add_alert(
        &mut self,
        severity: AlertSeverity,
        message: String,
        category: String,
        timestamp: DateTime<Utc>,
    ) {
        // Check if we already have a similar recent alert
        let recent_threshold = chrono::Duration::minutes(5);
        let has_recent_similar = self.alerts.iter().any(|alert| {
            alert.category == category
                && alert.severity == severity
                && (timestamp - alert.timestamp) < recent_threshold
        });

        if !has_recent_similar {
            let alert = SystemAlert {
                severity,
                message,
                timestamp,
                category,
            };

            self.alerts.push(alert);
            self.alerts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            if self.alerts.len() > self.max_alerts {
                self.alerts.truncate(self.max_alerts);
            }
        }
    }

    /// Format bytes to human readable string
    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Format duration to human readable string
    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Get health color based on system health
    fn get_health_color(health: &SystemHealth) -> Color {
        match health {
            SystemHealth::Healthy => Color::Green,
            SystemHealth::Warning => Color::Yellow,
            SystemHealth::Critical => Color::Red,
        }
    }

    /// Get usage color based on percentage
    fn get_usage_color(usage: f64) -> Color {
        if usage >= 90.0 {
            Color::Red
        } else if usage >= 70.0 {
            Color::Yellow
        } else {
            Color::Green
        }
    }

    /// Get health status icon
    #[allow(dead_code)]
    fn get_health_icon(&self, health: &SystemHealth) -> &'static str {
        match health {
            SystemHealth::Healthy => "✓",
            SystemHealth::Warning => "⚠",
            SystemHealth::Critical => "✗",
        }
    }

    /// Format health score as a visual bar
    #[allow(dead_code)]
    fn format_health_score(&self, score: f64) -> String {
        let bar_length = 20;
        let filled = ((score / 100.0) * bar_length as f64) as usize;
        let empty = bar_length - filled;

        format!(
            "[{}{}] {:.0}%",
            "█".repeat(filled),
            "░".repeat(empty),
            score
        )
    }

    /// Generate health trend analysis
    #[allow(dead_code)]
    fn analyze_health_trends(&self) -> Vec<String> {
        let mut trends = Vec::new();

        if self.history.len() >= 2 {
            let recent = &self.history[self.history.len() - 1];
            let previous = &self.history[self.history.len() - 2];

            // Analyze CPU trend
            if recent.cpu_usage > previous.cpu_usage + 10.0 {
                trends.push("CPU usage trending upward".to_string());
            } else if recent.cpu_usage < previous.cpu_usage - 10.0 {
                trends.push("CPU usage trending downward".to_string());
            }

            // Analyze memory trend
            if recent.memory_usage > previous.memory_usage + 10.0 {
                trends.push("Memory usage increasing".to_string());
            } else if recent.memory_usage < previous.memory_usage - 10.0 {
                trends.push("Memory usage decreasing".to_string());
            }

            // Analyze disk trend
            if recent.disk_usage > previous.disk_usage + 5.0 {
                trends.push("Disk usage growing".to_string());
            }
        }

        if trends.is_empty() {
            trends.push("System metrics stable".to_string());
        }

        trends
    }

    /// Render the overview display mode
    fn render_overview(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Resource usage
                Constraint::Length(6), // System info
                Constraint::Min(0),    // Recent alerts
            ])
            .split(area);

        // Resource usage section
        self.render_resource_usage(frame, chunks[0], theme);

        // System info section
        self.render_system_info_summary(frame, chunks[1], theme);

        // Recent alerts section
        self.render_recent_alerts(frame, chunks[2], theme);
    }

    /// Render resource usage gauges
    fn render_resource_usage(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        // CPU usage
        let cpu_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("CPU使用率")
                    .border_style(theme.styles.widget_border),
            )
            .gauge_style(Style::default().fg(Self::get_usage_color(self.current_status.cpu_usage)))
            .percent(self.current_status.cpu_usage as u16)
            .label(format!("{:.1}%", self.current_status.cpu_usage));
        frame.render_widget(cpu_gauge, chunks[0]);

        // Memory usage
        let memory_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("内存使用率")
                    .border_style(theme.styles.widget_border),
            )
            .gauge_style(
                Style::default().fg(Self::get_usage_color(self.current_status.memory_usage)),
            )
            .percent(self.current_status.memory_usage as u16)
            .label(format!("{:.1}%", self.current_status.memory_usage));
        frame.render_widget(memory_gauge, chunks[1]);

        // Disk usage
        let disk_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("磁盘使用率")
                    .border_style(theme.styles.widget_border),
            )
            .gauge_style(Style::default().fg(Self::get_usage_color(self.current_status.disk_usage)))
            .percent(self.current_status.disk_usage as u16)
            .label(format!("{:.1}%", self.current_status.disk_usage));
        frame.render_widget(disk_gauge, chunks[2]);
    }

    /// Render system info summary
    fn render_system_info_summary(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left column - System health and uptime
        let left_content = vec![
            Line::from(vec![
                Span::raw("系统健康: "),
                Span::styled(
                    format!("{:?}", self.current_status.system_health),
                    Style::default()
                        .fg(Self::get_health_color(&self.current_status.system_health))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("运行时间: "),
                Span::styled(
                    Self::format_duration(self.current_status.uptime),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
            Line::from(vec![
                Span::raw("活跃工作流: "),
                Span::styled(
                    self.current_status.active_workflows.to_string(),
                    Style::default().fg(theme.colors.accent),
                ),
            ]),
        ];

        let left_paragraph = Paragraph::new(left_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("系统状态")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);
        frame.render_widget(left_paragraph, chunks[0]);

        // Right column - Network and processes
        let right_content = vec![
            Line::from(vec![
                Span::raw("网络状态: "),
                Span::styled(
                    format!("{:?}", self.current_status.network_status),
                    Style::default().fg(match self.current_status.network_status {
                        NetworkStatus::Connected => Color::Green,
                        NetworkStatus::Limited => Color::Yellow,
                        NetworkStatus::Degraded => Color::Magenta,
                        NetworkStatus::Disconnected => Color::Red,
                    }),
                ),
            ]),
            Line::from(vec![
                Span::raw("进程数: "),
                Span::styled(
                    self.current_status.process_count.to_string(),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
            Line::from(vec![
                Span::raw("负载平均: "),
                Span::styled(
                    format!("{:.2}", self.current_status.load_average),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
        ];

        let right_paragraph = Paragraph::new(right_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("系统信息")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);
        frame.render_widget(right_paragraph, chunks[1]);
    }

    /// Render recent alerts
    fn render_recent_alerts(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let recent_alerts: Vec<ListItem> = self
            .alerts
            .iter()
            .take(area.height.saturating_sub(2) as usize)
            .map(|alert| {
                let severity_symbol = match alert.severity {
                    AlertSeverity::Info => "ℹ",
                    AlertSeverity::Warning => "⚠",
                    AlertSeverity::Critical => "🔥",
                };

                let severity_color = match alert.severity {
                    AlertSeverity::Info => Color::Blue,
                    AlertSeverity::Warning => Color::Yellow,
                    AlertSeverity::Critical => Color::Red,
                };

                let time_str = alert.timestamp.format("%H:%M:%S").to_string();

                ListItem::new(Line::from(vec![
                    Span::styled(severity_symbol, Style::default().fg(severity_color)),
                    Span::raw(" "),
                    Span::styled(time_str, Style::default().fg(Color::Gray)),
                    Span::raw(" "),
                    Span::styled(&alert.category, Style::default().fg(theme.colors.accent)),
                    Span::raw(": "),
                    Span::raw(&alert.message),
                ]))
            })
            .collect();

        let alerts_list = List::new(recent_alerts)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("最近警告 ({})", self.alerts.len()))
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(alerts_list, area);
    }

    /// Handle navigation keys
    fn handle_navigation(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Tab => {
                self.display_mode = match self.display_mode {
                    DisplayMode::Overview => DisplayMode::DetailedMetrics,
                    DisplayMode::DetailedMetrics => DisplayMode::PerformanceCharts,
                    DisplayMode::PerformanceCharts => DisplayMode::SystemInfo,
                    DisplayMode::SystemInfo => DisplayMode::Alerts,
                    DisplayMode::Alerts => DisplayMode::Maintenance,
                    DisplayMode::Maintenance => DisplayMode::Overview,
                };
                None
            }
            KeyCode::Char('r') | KeyCode::F(5) => Some(Action::RefreshSystemStatus),
            KeyCode::Char('h') | KeyCode::F(1) => {
                self.show_help = !self.show_help;
                None
            }
            KeyCode::Char('t') => {
                // Toggle threshold configuration mode
                // This would open a threshold configuration dialog
                None
            }
            KeyCode::Char('c') => {
                // Clear alerts or perform maintenance action based on current view
                if matches!(self.display_mode, DisplayMode::Maintenance) {
                    // Perform clean alerts maintenance action
                    // In a real implementation, this would be async
                    self.clear_old_alerts(chrono::Duration::hours(1));
                } else {
                    self.clear_alerts();
                }
                None
            }
            KeyCode::Char('m') => {
                // Switch to maintenance view
                self.display_mode = DisplayMode::Maintenance;
                None
            }
            KeyCode::Char('o') => {
                // Memory optimization (maintenance action)
                if matches!(self.display_mode, DisplayMode::Maintenance) {
                    // In a real implementation, this would trigger memory optimization
                    // For now, just add an info message
                }
                None
            }
            KeyCode::Char('d') => {
                // Disk cleanup (maintenance action)
                if matches!(self.display_mode, DisplayMode::Maintenance) {
                    // In a real implementation, this would trigger disk cleanup
                    // For now, just add an info message
                }
                None
            }
            KeyCode::Char('p') => {
                // Process optimization (maintenance action)
                if matches!(self.display_mode, DisplayMode::Maintenance) {
                    // In a real implementation, this would trigger process optimization
                    // For now, just add an info message
                }
                None
            }
            KeyCode::Char('s') => {
                // System diagnostics (maintenance action)
                if matches!(self.display_mode, DisplayMode::Maintenance) {
                    // In a real implementation, this would run comprehensive diagnostics
                    // For now, just refresh the diagnostics
                }
                None
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                None
            }
            KeyCode::Down => {
                self.selected_index += 1;
                None
            }
            KeyCode::PageUp => {
                self.selected_index = self.selected_index.saturating_sub(10);
                None
            }
            KeyCode::PageDown => {
                self.selected_index += 10;
                None
            }
            KeyCode::Home => {
                self.selected_index = 0;
                None
            }
            KeyCode::End => {
                self.selected_index = usize::MAX; // Will be clamped by render logic
                None
            }
            _ => None,
        }
    }
}

#[async_trait]
impl Widget for SystemStatusWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }

    fn title(&self) -> &str {
        "系统状态监控"
    }

    fn description(&self) -> Option<&str> {
        Some("显示系统资源使用情况、健康状态和性能指标")
    }

    fn context(&self) -> &WidgetContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut WidgetContext {
        &mut self.context
    }

    fn capabilities(&self) -> &WidgetCapabilities {
        &self.capabilities
    }

    fn size_constraints(&self) -> &SizeConstraints {
        &self.size_constraints
    }

    fn update_frequency(&self) -> UpdateFrequency {
        UpdateFrequency::Interval(self.update_interval)
    }

    async fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
    ) -> Result<(), WidgetError> {
        let title = format!(
            "系统状态监控 - {}",
            match self.display_mode {
                DisplayMode::Overview => "概览",
                DisplayMode::DetailedMetrics => "详细指标",
                DisplayMode::PerformanceCharts => "性能图表",
                DisplayMode::SystemInfo => "系统信息",
                DisplayMode::Alerts => "警告列表",
                DisplayMode::Maintenance => "系统维护",
            }
        );

        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(theme.styles.widget_border);

        let inner_area = main_block.inner(area);
        frame.render_widget(main_block, area);

        match self.display_mode {
            DisplayMode::Overview => {
                self.render_overview(frame, inner_area, theme);
            }
            DisplayMode::DetailedMetrics => {
                self.render_detailed_metrics(frame, inner_area, theme);
            }
            DisplayMode::PerformanceCharts => {
                self.render_performance_charts(frame, inner_area, theme);
            }
            DisplayMode::SystemInfo => {
                self.render_system_info(frame, inner_area, theme);
            }
            DisplayMode::Alerts => {
                self.render_alerts_view(frame, inner_area, theme);
            }
            DisplayMode::Maintenance => {
                self.render_maintenance_view(frame, inner_area, theme);
            }
        }

        // Show help overlay if requested
        if self.show_help {
            self.render_help_overlay(frame, area, theme);
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<Option<Action>, WidgetError> {
        if !self.can_handle_event(&event) {
            return Ok(None);
        }

        match event {
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('q') => return Ok(Some(Action::Quit)),
                        KeyCode::Char('r') => return Ok(Some(Action::RefreshSystemStatus)),
                        _ => {}
                    }
                }

                Ok(self.handle_navigation(key))
            }
            _ => Ok(None),
        }
    }

    async fn update(&mut self) -> Result<(), WidgetError> {
        self.context.mark_updated();

        // Update system status
        self.update_status().await;

        // Clean up old alerts (older than 1 hour)
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        self.alerts.retain(|alert| alert.timestamp > cutoff);

        Ok(())
    }

    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("Tab", "切换视图模式"),
            ("r/F5", "刷新系统状态"),
            ("h/F1", "显示/隐藏帮助"),
            ("m", "切换到维护视图"),
            ("t", "配置阈值"),
            ("c", "清除警告/执行清理"),
            ("o", "内存优化 (维护模式)"),
            ("d", "磁盘清理 (维护模式)"),
            ("p", "进程优化 (维护模式)"),
            ("s", "系统诊断 (维护模式)"),
            ("↑/↓", "导航"),
            ("PgUp/PgDn", "快速导航"),
            ("Home/End", "跳转到开始/结束"),
            ("Ctrl+Q", "退出"),
        ]
    }
}

impl SystemStatusWidget {
    /// Render help overlay
    fn render_help_overlay(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let help_area = Rect {
            x: area.x + area.width / 4,
            y: area.y + area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };

        frame.render_widget(Clear, help_area);

        let help_items: Vec<Line> = self
            .help_text()
            .iter()
            .map(|(key, desc)| {
                Line::from(vec![
                    Span::styled(
                        *key,
                        Style::default()
                            .fg(theme.colors.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": "),
                    Span::raw(*desc),
                ])
            })
            .collect();

        let help_paragraph = Paragraph::new(help_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("快捷键帮助")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });

        frame.render_widget(help_paragraph, help_area);
    }

    /// Render detailed metrics view
    fn render_detailed_metrics(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // CPU details
                Constraint::Length(8),  // Memory details
                Constraint::Min(0),     // Disk and process info
            ])
            .split(area);

        // CPU details
        self.render_cpu_details(frame, chunks[0], theme);

        // Memory details
        self.render_memory_details(frame, chunks[1], theme);

        // Disk and process info
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        self.render_disk_info(frame, bottom_chunks[0], theme);
        self.render_top_processes(frame, bottom_chunks[1], theme);
    }

    /// Render CPU details
    fn render_cpu_details(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let content = if let Some(ref cpu_info) = self.cpu_info {
            vec![
                Line::from(vec![
                    Span::raw("CPU型号: "),
                    Span::styled(
                        &cpu_info.name,
                        Style::default().fg(theme.colors.text_primary),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("核心数: "),
                    Span::styled(
                        cpu_info.core_count.to_string(),
                        Style::default().fg(theme.colors.accent),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("频率: "),
                    Span::styled(
                        format!("{} MHz", cpu_info.frequency),
                        Style::default().fg(theme.colors.text_primary),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("总体使用率: "),
                    Span::styled(
                        format!("{:.1}%", cpu_info.usage),
                        Style::default().fg(Self::get_usage_color(cpu_info.usage)),
                    ),
                ]),
                Line::from(Span::raw("各核心使用率:")),
            ]
        } else {
            vec![Line::from(Span::raw("正在加载CPU信息..."))]
        };

        let cpu_paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("CPU详细信息")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(cpu_paragraph, area);
    }

    /// Render memory details
    fn render_memory_details(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let content = if let Some(ref mem_info) = self.memory_info {
            vec![
                Line::from(vec![
                    Span::raw("总内存: "),
                    Span::styled(
                        Self::format_bytes(mem_info.total),
                        Style::default().fg(theme.colors.text_primary),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("已使用: "),
                    Span::styled(
                        Self::format_bytes(mem_info.used),
                        Style::default().fg(theme.colors.accent),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("可用: "),
                    Span::styled(
                        Self::format_bytes(mem_info.available),
                        Style::default().fg(theme.colors.success),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("空闲: "),
                    Span::styled(
                        Self::format_bytes(mem_info.free),
                        Style::default().fg(theme.colors.text_primary),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("交换区: "),
                    Span::styled(
                        format!(
                            "{} / {}",
                            Self::format_bytes(mem_info.swap_used),
                            Self::format_bytes(mem_info.swap_total)
                        ),
                        Style::default().fg(theme.colors.text_primary),
                    ),
                ]),
            ]
        } else {
            vec![Line::from(Span::raw("正在加载内存信息..."))]
        };

        let memory_paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("内存详细信息")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(memory_paragraph, area);
    }

    /// Render disk information
    fn render_disk_info(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let disk_items: Vec<ListItem> = self
            .disk_info
            .iter()
            .map(|disk| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            &disk.name,
                            Style::default()
                                .fg(theme.colors.accent)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" ("),
                        Span::raw(&disk.file_system),
                        Span::raw(")"),
                    ]),
                    Line::from(vec![Span::raw("挂载点: "), Span::raw(&disk.mount_point)]),
                    Line::from(vec![
                        Span::raw("使用: "),
                        Span::styled(
                            format!("{:.1}%", disk.usage_percent),
                            Style::default().fg(Self::get_usage_color(disk.usage_percent)),
                        ),
                        Span::raw(" ("),
                        Span::raw(Self::format_bytes(disk.used_space)),
                        Span::raw(" / "),
                        Span::raw(Self::format_bytes(disk.total_space)),
                        Span::raw(")"),
                    ]),
                ])
            })
            .collect();

        let disk_list = List::new(disk_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("磁盘信息")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(disk_list, area);
    }

    /// Render top processes
    fn render_top_processes(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let process_items: Vec<ListItem> = self
            .top_processes
            .iter()
            .take(area.height.saturating_sub(2) as usize)
            .map(|process| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{}", process.pid),
                            Style::default().fg(theme.colors.text_secondary),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            &process.name,
                            Style::default()
                                .fg(theme.colors.text_primary)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("CPU: "),
                        Span::styled(
                            format!("{:.1}%", process.cpu_usage),
                            Style::default().fg(Self::get_usage_color(process.cpu_usage)),
                        ),
                        Span::raw(" 内存: "),
                        Span::styled(
                            Self::format_bytes(process.memory_usage),
                            Style::default().fg(theme.colors.accent),
                        ),
                    ]),
                ])
            })
            .collect();

        let process_list = List::new(process_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("热门进程 (按CPU)")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(process_list, area);
    }

    /// Render system information view
    fn render_system_info(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // System basic info
                Constraint::Length(8), // Hardware info
                Constraint::Min(0),    // Runtime and environment info
            ])
            .split(area);

        // System basic info
        self.render_system_basic_info(frame, chunks[0], theme);

        // Hardware info
        self.render_hardware_info(frame, chunks[1], theme);

        // Runtime and environment info
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        self.render_runtime_info(frame, bottom_chunks[0], theme);
        self.render_environment_info(frame, bottom_chunks[1], theme);
    }

    /// Render system basic information
    fn render_system_basic_info(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let system_info = vec![
            Line::from(vec![
                Span::raw("操作系统: "),
                Span::styled(
                    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
                    Style::default()
                        .fg(theme.colors.text_primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("系统版本: "),
                Span::styled(
                    self.get_os_version(),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
            Line::from(vec![
                Span::raw("主机名: "),
                Span::styled(
                    self.get_hostname(),
                    Style::default().fg(theme.colors.accent),
                ),
            ]),
            Line::from(vec![
                Span::raw("系统运行时间: "),
                Span::styled(
                    Self::format_duration(self.current_status.uptime),
                    Style::default().fg(theme.colors.success),
                ),
            ]),
            Line::from(vec![
                Span::raw("系统负载: "),
                Span::styled(
                    format!("{:.2}", self.current_status.load_average),
                    Style::default().fg(Self::get_usage_color(
                        self.current_status.load_average * 100.0,
                    )),
                ),
            ]),
        ];

        let system_paragraph = Paragraph::new(system_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("系统基本信息")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(system_paragraph, area);
    }

    /// Render hardware information
    fn render_hardware_info(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let hardware_info = vec![
            Line::from(vec![
                Span::raw("CPU型号: "),
                Span::styled(
                    self.cpu_info
                        .as_ref()
                        .map_or("检测中...".to_string(), |cpu| cpu.name.clone()),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
            Line::from(vec![
                Span::raw("CPU核心数: "),
                Span::styled(
                    self.cpu_info
                        .as_ref()
                        .map_or("未知".to_string(), |cpu| cpu.core_count.to_string()),
                    Style::default().fg(theme.colors.accent),
                ),
                Span::raw(" 核心"),
            ]),
            Line::from(vec![
                Span::raw("CPU频率: "),
                Span::styled(
                    self.cpu_info
                        .as_ref()
                        .map_or("未知".to_string(), |cpu| format!("{} MHz", cpu.frequency)),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
            Line::from(vec![
                Span::raw("总内存: "),
                Span::styled(
                    Self::format_bytes(self.current_status.memory_total),
                    Style::default()
                        .fg(theme.colors.text_primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("磁盘数量: "),
                Span::styled(
                    self.disk_info.len().to_string(),
                    Style::default().fg(theme.colors.text_primary),
                ),
                Span::raw(" 个磁盘"),
            ]),
        ];

        let hardware_paragraph = Paragraph::new(hardware_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("硬件信息")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(hardware_paragraph, area);
    }

    /// Render runtime information
    fn render_runtime_info(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let runtime_info = vec![
            Line::from(vec![
                Span::raw("进程数: "),
                Span::styled(
                    self.current_status.process_count.to_string(),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
            Line::from(vec![
                Span::raw("线程数: "),
                Span::styled(
                    self.current_status.thread_count.to_string(),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
            Line::from(vec![
                Span::raw("活跃工作流: "),
                Span::styled(
                    self.current_status.active_workflows.to_string(),
                    Style::default().fg(theme.colors.accent),
                ),
            ]),
            Line::from(vec![
                Span::raw("网络状态: "),
                Span::styled(
                    format!("{:?}", self.current_status.network_status),
                    Style::default().fg(match self.current_status.network_status {
                        NetworkStatus::Connected => Color::Green,
                        NetworkStatus::Limited => Color::Yellow,
                        NetworkStatus::Degraded => Color::Magenta,
                        NetworkStatus::Disconnected => Color::Red,
                    }),
                ),
            ]),
            Line::from(vec![
                Span::raw("系统健康: "),
                Span::styled(
                    format!("{:?}", self.current_status.system_health),
                    Style::default()
                        .fg(Self::get_health_color(&self.current_status.system_health))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let runtime_paragraph = Paragraph::new(runtime_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("运行时信息")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(runtime_paragraph, area);
    }

    /// Render environment information
    fn render_environment_info(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let env_info = vec![
            Line::from(vec![
                Span::raw("用户名: "),
                Span::styled(
                    std::env::var("USERNAME")
                        .or_else(|_| std::env::var("USER"))
                        .unwrap_or_else(|_| "未知".to_string()),
                    Style::default().fg(theme.colors.text_primary),
                ),
            ]),
            Line::from(vec![
                Span::raw("工作目录: "),
                Span::styled(
                    std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| "未知".to_string()),
                    Style::default().fg(theme.colors.text_secondary),
                ),
            ]),
            Line::from(vec![
                Span::raw("临时目录: "),
                Span::styled(
                    std::env::temp_dir().to_string_lossy().to_string(),
                    Style::default().fg(theme.colors.text_secondary),
                ),
            ]),
            Line::from(vec![
                Span::raw("PATH变量: "),
                Span::styled(
                    std::env::var("PATH")
                        .map(|p| {
                            if p.len() > 50 {
                                format!("{}...", &p[..50])
                            } else {
                                p
                            }
                        })
                        .unwrap_or_else(|_| "未设置".to_string()),
                    Style::default().fg(theme.colors.text_secondary),
                ),
            ]),
            Line::from(vec![
                Span::raw("Rust版本: "),
                Span::styled(
                    std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "未知".to_string()),
                    Style::default().fg(theme.colors.accent),
                ),
            ]),
        ];

        let env_paragraph = Paragraph::new(env_info)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("环境信息")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(env_paragraph, area);
    }

    /// Get OS version information
    fn get_os_version(&self) -> String {
        // This is a simplified version - in a real implementation,
        // you might want to use platform-specific APIs to get detailed version info
        match std::env::consts::OS {
            "windows" => std::env::var("OS").unwrap_or_else(|_| "Windows".to_string()),
            "linux" => {
                // Try to read from /etc/os-release or similar
                "Linux".to_string()
            }
            "macos" => "macOS".to_string(),
            other => other.to_string(),
        }
    }

    /// Get hostname
    fn get_hostname(&self) -> String {
        std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "未知主机".to_string())
    }

    /// Render alerts view
    fn render_alerts_view(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Alert statistics
                Constraint::Length(4), // Threshold configuration
                Constraint::Min(0),    // Alert list
            ])
            .split(area);

        // Alert statistics
        self.render_alert_statistics(frame, chunks[0], theme);

        // Threshold configuration display
        self.render_threshold_display(frame, chunks[1], theme);

        // Alert list
        self.render_alert_list(frame, chunks[2], theme);
    }

    /// Render alert statistics
    fn render_alert_statistics(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let stats = self.get_alert_stats();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        // Total alerts
        let total_paragraph = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "总警告",
                Style::default().fg(theme.colors.text_secondary),
            )]),
            Line::from(vec![Span::styled(
                stats.total_alerts.to_string(),
                Style::default()
                    .fg(theme.colors.text_primary)
                    .add_modifier(Modifier::BOLD),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.styles.widget_border),
        )
        .alignment(Alignment::Center);
        frame.render_widget(total_paragraph, chunks[0]);

        // Critical alerts
        let critical_paragraph = Paragraph::new(vec![
            Line::from(vec![Span::styled("严重", Style::default().fg(Color::Red))]),
            Line::from(vec![Span::styled(
                stats.critical_alerts.to_string(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.styles.widget_border),
        )
        .alignment(Alignment::Center);
        frame.render_widget(critical_paragraph, chunks[1]);

        // Warning alerts
        let warning_paragraph = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "警告",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![Span::styled(
                stats.warning_alerts.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.styles.widget_border),
        )
        .alignment(Alignment::Center);
        frame.render_widget(warning_paragraph, chunks[2]);

        // Recent alerts (last 30 minutes)
        let recent_paragraph = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "最近30分钟",
                Style::default().fg(theme.colors.text_secondary),
            )]),
            Line::from(vec![Span::styled(
                stats.recent_alerts.to_string(),
                Style::default()
                    .fg(theme.colors.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.styles.widget_border),
        )
        .alignment(Alignment::Center);
        frame.render_widget(recent_paragraph, chunks[3]);
    }

    /// Render threshold configuration display
    fn render_threshold_display(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let thresholds = self.get_thresholds();

        let content = vec![
            Line::from(vec![
                Span::raw("阈值配置: "),
                Span::styled("CPU", Style::default().fg(theme.colors.accent)),
                Span::raw(format!(
                    " {:.0}%/{:.0}% ",
                    thresholds.cpu_warning, thresholds.cpu_critical
                )),
                Span::styled("内存", Style::default().fg(theme.colors.accent)),
                Span::raw(format!(
                    " {:.0}%/{:.0}% ",
                    thresholds.memory_warning, thresholds.memory_critical
                )),
                Span::styled("磁盘", Style::default().fg(theme.colors.accent)),
                Span::raw(format!(
                    " {:.0}%/{:.0}%",
                    thresholds.disk_warning, thresholds.disk_critical
                )),
            ]),
            Line::from(vec![Span::styled(
                "按 't' 配置阈值, 按 'c' 清除警告",
                Style::default().fg(theme.colors.text_secondary),
            )]),
        ];

        let threshold_paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("阈值配置")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(threshold_paragraph, area);
    }

    /// Render alert list
    fn render_alert_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let alert_items: Vec<ListItem> = self
            .alerts
            .iter()
            .map(|alert| {
                let severity_symbol = match alert.severity {
                    AlertSeverity::Info => "ℹ",
                    AlertSeverity::Warning => "⚠",
                    AlertSeverity::Critical => "🔥",
                };

                let severity_color = match alert.severity {
                    AlertSeverity::Info => Color::Blue,
                    AlertSeverity::Warning => Color::Yellow,
                    AlertSeverity::Critical => Color::Red,
                };

                let time_str = alert.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(severity_symbol, Style::default().fg(severity_color)),
                        Span::raw(" "),
                        Span::styled(
                            &alert.category,
                            Style::default()
                                .fg(theme.colors.accent)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" - "),
                        Span::styled(
                            format!("{:?}", alert.severity),
                            Style::default().fg(severity_color),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("时间: "),
                        Span::styled(time_str, Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(vec![Span::raw("消息: "), Span::raw(&alert.message)]),
                    Line::from(Span::raw("")), // Empty line for spacing
                ])
            })
            .collect();

        let alerts_list = List::new(alert_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("系统警告详情 ({})", self.alerts.len()))
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(alerts_list, area);
    }

    /// Render performance charts view
    fn render_performance_charts(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12), // CPU chart
                Constraint::Length(12), // Memory chart
                Constraint::Min(0),     // Disk and network charts
            ])
            .split(area);

        // CPU usage chart
        self.render_cpu_chart(frame, chunks[0], theme);

        // Memory usage chart
        self.render_memory_chart(frame, chunks[1], theme);

        // Bottom charts
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        self.render_disk_chart(frame, bottom_chunks[0], theme);
        self.render_network_chart(frame, bottom_chunks[1], theme);
    }

    /// Render maintenance view
    fn render_maintenance_view(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // System diagnostics summary
                Constraint::Length(10), // Maintenance recommendations
                Constraint::Min(0),     // Available maintenance actions
            ])
            .split(area);

        // System diagnostics summary
        self.render_diagnostics_summary(frame, chunks[0], theme);

        // Maintenance recommendations
        self.render_maintenance_recommendations(frame, chunks[1], theme);

        // Available maintenance actions
        self.render_maintenance_actions(frame, chunks[2], theme);
    }

    /// Render system diagnostics summary
    fn render_diagnostics_summary(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let diagnostics = self.get_system_diagnostics();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        // Issues
        let issues_content = if diagnostics.issues.is_empty() {
            vec![Line::from(vec![Span::styled(
                "✓ 无严重问题",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )])]
        } else {
            diagnostics
                .issues
                .iter()
                .take(5)
                .map(|issue| {
                    Line::from(vec![
                        Span::styled("✗ ", Style::default().fg(Color::Red)),
                        Span::raw(issue),
                    ])
                })
                .collect()
        };

        let issues_paragraph = Paragraph::new(issues_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("严重问题 ({})", diagnostics.issues.len()))
                    .border_style(if diagnostics.issues.is_empty() {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    }),
            )
            .style(theme.styles.info);
        frame.render_widget(issues_paragraph, chunks[0]);

        // Warnings
        let warnings_content = if diagnostics.warnings.is_empty() {
            vec![Line::from(vec![Span::styled(
                "✓ 无警告",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )])]
        } else {
            diagnostics
                .warnings
                .iter()
                .take(5)
                .map(|warning| {
                    Line::from(vec![
                        Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
                        Span::raw(warning),
                    ])
                })
                .collect()
        };

        let warnings_paragraph = Paragraph::new(warnings_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("警告 ({})", diagnostics.warnings.len()))
                    .border_style(if diagnostics.warnings.is_empty() {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    }),
            )
            .style(theme.styles.info);
        frame.render_widget(warnings_paragraph, chunks[1]);

        // System health overview
        let health_content = vec![
            Line::from(vec![
                Span::raw("整体健康: "),
                Span::styled(
                    format!("{:?}", diagnostics.overall_health),
                    Style::default()
                        .fg(Self::get_health_color(&diagnostics.overall_health))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("诊断时间: "),
                Span::styled(
                    diagnostics.diagnostics_time.format("%H:%M:%S").to_string(),
                    Style::default().fg(theme.colors.text_secondary),
                ),
            ]),
            Line::from(vec![
                Span::raw("系统运行: "),
                Span::styled(
                    Self::format_duration(self.current_status.uptime),
                    Style::default().fg(theme.colors.accent),
                ),
            ]),
        ];

        let health_paragraph = Paragraph::new(health_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("系统健康概览")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);
        frame.render_widget(health_paragraph, chunks[2]);
    }

    /// Render maintenance recommendations
    fn render_maintenance_recommendations(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let recommendations = self.get_maintenance_recommendations();

        let rec_items: Vec<ListItem> = recommendations
            .iter()
            .take(area.height.saturating_sub(2) as usize)
            .map(|rec| {
                let priority_symbol = match rec.priority {
                    MaintenancePriority::Critical => "🔥",
                    MaintenancePriority::High => "⚠",
                    MaintenancePriority::Medium => "ℹ",
                    MaintenancePriority::Low => "💡",
                };

                let priority_color = match rec.priority {
                    MaintenancePriority::Critical => Color::Red,
                    MaintenancePriority::High => Color::Yellow,
                    MaintenancePriority::Medium => Color::Blue,
                    MaintenancePriority::Low => Color::Gray,
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(priority_symbol, Style::default().fg(priority_color)),
                        Span::raw(" "),
                        Span::styled(
                            &rec.title,
                            Style::default()
                                .fg(theme.colors.text_primary)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" ("),
                        Span::styled(&rec.category, Style::default().fg(theme.colors.accent)),
                        Span::raw(")"),
                    ]),
                    Line::from(vec![Span::raw("   "), Span::raw(&rec.description)]),
                ])
            })
            .collect();

        let recommendations_list = List::new(rec_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("维护建议 ({})", recommendations.len()))
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(recommendations_list, area);
    }

    /// Render available maintenance actions
    fn render_maintenance_actions(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let actions = [
            ("清理警告", "清理旧的系统警告和通知", "c"),
            ("内存优化", "释放未使用的内存资源", "o"),
            ("磁盘清理", "清理临时文件和缓存", "d"),
            ("进程优化", "优化系统进程配置", "p"),
            ("系统诊断", "运行完整系统诊断", "s"),
            ("刷新数据", "重新加载系统信息", "r"),
        ];

        let action_items: Vec<ListItem> = actions
            .iter()
            .map(|(name, desc, key)| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("[{}]", key),
                            Style::default()
                                .fg(theme.colors.accent)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            *name,
                            Style::default()
                                .fg(theme.colors.text_primary)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(*desc, Style::default().fg(theme.colors.text_secondary)),
                    ]),
                ])
            })
            .collect();

        let actions_list = List::new(action_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("可用维护操作")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(actions_list, area);
    }

    /// Render CPU usage chart
    fn render_cpu_chart(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.history.is_empty() {
            let placeholder = Paragraph::new("正在收集CPU数据...")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("CPU使用率历史")
                        .border_style(theme.styles.widget_border),
                )
                .style(theme.styles.info)
                .alignment(Alignment::Center);
            frame.render_widget(placeholder, area);
            return;
        }

        // Create a simple text-based chart
        let chart_height = area.height.saturating_sub(2) as usize;
        let chart_width = area.width.saturating_sub(2) as usize;

        let mut chart_lines = Vec::new();

        // Chart header
        chart_lines.push(Line::from(vec![
            Span::raw("CPU使用率 (最近 "),
            Span::styled(
                self.history.len().to_string(),
                Style::default().fg(theme.colors.accent),
            ),
            Span::raw(" 个数据点)"),
        ]));

        // Current value
        if let Some(latest) = self.history.back() {
            chart_lines.push(Line::from(vec![
                Span::raw("当前: "),
                Span::styled(
                    format!("{:.1}%", latest.cpu_usage),
                    Style::default()
                        .fg(Self::get_usage_color(latest.cpu_usage))
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        // Simple ASCII chart
        let max_value = self
            .history
            .iter()
            .map(|p| p.cpu_usage)
            .fold(0.0, f64::max)
            .max(100.0);

        let available_height = chart_height.saturating_sub(4); // Reserve space for header and labels

        for i in 0..available_height {
            let threshold = max_value * (1.0 - (i as f64 / available_height as f64));
            let mut line_chars = Vec::new();

            // Y-axis label
            line_chars.push(Span::styled(
                format!("{:3.0}% ", threshold),
                Style::default().fg(Color::Gray),
            ));

            // Chart data
            let points_to_show = chart_width.saturating_sub(6); // Reserve space for Y-axis
            let step = if self.history.len() > points_to_show {
                self.history.len() / points_to_show
            } else {
                1
            };

            for (j, point) in self.history.iter().step_by(step).enumerate() {
                if j >= points_to_show {
                    break;
                }

                let char = if point.cpu_usage >= threshold {
                    if point.cpu_usage >= 90.0 {
                        "█"
                    } else if point.cpu_usage >= 70.0 {
                        "▓"
                    } else {
                        "▒"
                    }
                } else {
                    " "
                };

                let color = Self::get_usage_color(point.cpu_usage);
                line_chars.push(Span::styled(char, Style::default().fg(color)));
            }

            chart_lines.push(Line::from(line_chars));
        }

        // Time axis
        if let (Some(first), Some(last)) = (self.history.front(), self.history.back()) {
            let time_diff = last.timestamp - first.timestamp;
            chart_lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(
                    format!("时间跨度: {} 分钟", time_diff.num_minutes()),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }

        let cpu_chart = Paragraph::new(chart_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("CPU使用率历史")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(cpu_chart, area);
    }

    /// Render memory usage chart
    fn render_memory_chart(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.history.is_empty() {
            let placeholder = Paragraph::new("正在收集内存数据...")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("内存使用率历史")
                        .border_style(theme.styles.widget_border),
                )
                .style(theme.styles.info)
                .alignment(Alignment::Center);
            frame.render_widget(placeholder, area);
            return;
        }

        // Create a simple text-based chart similar to CPU chart
        let chart_height = area.height.saturating_sub(2) as usize;
        let chart_width = area.width.saturating_sub(2) as usize;

        let mut chart_lines = Vec::new();

        // Chart header
        chart_lines.push(Line::from(vec![
            Span::raw("内存使用率 (最近 "),
            Span::styled(
                self.history.len().to_string(),
                Style::default().fg(theme.colors.accent),
            ),
            Span::raw(" 个数据点)"),
        ]));

        // Current value
        if let Some(latest) = self.history.back() {
            chart_lines.push(Line::from(vec![
                Span::raw("当前: "),
                Span::styled(
                    format!("{:.1}%", latest.memory_usage),
                    Style::default()
                        .fg(Self::get_usage_color(latest.memory_usage))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" ("),
                Span::styled(
                    Self::format_bytes(self.current_status.memory_used),
                    Style::default().fg(theme.colors.text_primary),
                ),
                Span::raw(" / "),
                Span::styled(
                    Self::format_bytes(self.current_status.memory_total),
                    Style::default().fg(theme.colors.text_primary),
                ),
                Span::raw(")"),
            ]));
        }

        // Simple ASCII chart
        let max_value = self
            .history
            .iter()
            .map(|p| p.memory_usage)
            .fold(0.0, f64::max)
            .max(100.0);

        let available_height = chart_height.saturating_sub(4);

        for i in 0..available_height {
            let threshold = max_value * (1.0 - (i as f64 / available_height as f64));
            let mut line_chars = Vec::new();

            // Y-axis label
            line_chars.push(Span::styled(
                format!("{:3.0}% ", threshold),
                Style::default().fg(Color::Gray),
            ));

            // Chart data
            let points_to_show = chart_width.saturating_sub(6);
            let step = if self.history.len() > points_to_show {
                self.history.len() / points_to_show
            } else {
                1
            };

            for (j, point) in self.history.iter().step_by(step).enumerate() {
                if j >= points_to_show {
                    break;
                }

                let char = if point.memory_usage >= threshold {
                    if point.memory_usage >= 90.0 {
                        "█"
                    } else if point.memory_usage >= 70.0 {
                        "▓"
                    } else {
                        "▒"
                    }
                } else {
                    " "
                };

                let color = Self::get_usage_color(point.memory_usage);
                line_chars.push(Span::styled(char, Style::default().fg(color)));
            }

            chart_lines.push(Line::from(line_chars));
        }

        let memory_chart = Paragraph::new(chart_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("内存使用率历史")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(memory_chart, area);
    }

    /// Render disk usage chart
    fn render_disk_chart(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.history.is_empty() {
            let placeholder = Paragraph::new("正在收集磁盘数据...")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("磁盘使用率历史")
                        .border_style(theme.styles.widget_border),
                )
                .style(theme.styles.info)
                .alignment(Alignment::Center);
            frame.render_widget(placeholder, area);
            return;
        }

        let mut chart_lines = Vec::new();

        // Chart header
        chart_lines.push(Line::from(vec![Span::raw("磁盘使用率趋势")]));

        // Current value
        if let Some(latest) = self.history.back() {
            chart_lines.push(Line::from(vec![
                Span::raw("当前: "),
                Span::styled(
                    format!("{:.1}%", latest.disk_usage),
                    Style::default()
                        .fg(Self::get_usage_color(latest.disk_usage))
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        // Trend analysis
        if self.history.len() >= 2 {
            let recent = self.history.back().unwrap();
            let previous = &self.history[self.history.len() - 2];
            let trend = recent.disk_usage - previous.disk_usage;

            let trend_text = if trend > 0.1 {
                ("↗ 上升", Color::Red)
            } else if trend < -0.1 {
                ("↘ 下降", Color::Green)
            } else {
                ("→ 稳定", Color::Yellow)
            };

            chart_lines.push(Line::from(vec![
                Span::raw("趋势: "),
                Span::styled(trend_text.0, Style::default().fg(trend_text.1)),
                Span::raw(format!(" ({:+.1}%)", trend)),
            ]));
        }

        // Disk space info
        chart_lines.push(Line::from(vec![
            Span::raw("已用: "),
            Span::styled(
                Self::format_bytes(self.current_status.disk_used),
                Style::default().fg(theme.colors.accent),
            ),
        ]));

        chart_lines.push(Line::from(vec![
            Span::raw("总计: "),
            Span::styled(
                Self::format_bytes(self.current_status.disk_total),
                Style::default().fg(theme.colors.text_primary),
            ),
        ]));

        let disk_chart = Paragraph::new(chart_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("磁盘使用率历史")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(disk_chart, area);
    }

    /// Render network activity chart
    fn render_network_chart(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut chart_lines = Vec::new();

        // Chart header
        chart_lines.push(Line::from(vec![Span::raw("网络状态监控")]));

        // Network status
        let (status_text, status_color) = match self.current_status.network_status {
            NetworkStatus::Connected => ("已连接", Color::Green),
            NetworkStatus::Limited => ("受限连接", Color::Yellow),
            NetworkStatus::Degraded => ("连接降级", Color::Magenta),
            NetworkStatus::Disconnected => ("未连接", Color::Red),
        };

        chart_lines.push(Line::from(vec![
            Span::raw("状态: "),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Network activity placeholder (would need actual network monitoring)
        chart_lines.push(Line::from(vec![
            Span::raw("活动: "),
            Span::styled(
                "监控中...",
                Style::default().fg(theme.colors.text_secondary),
            ),
        ]));

        // Connection quality indicator
        let quality_score = match self.current_status.network_status {
            NetworkStatus::Connected => 100,
            NetworkStatus::Limited => 50,
            NetworkStatus::Degraded => 30,
            NetworkStatus::Disconnected => 0,
        };

        chart_lines.push(Line::from(vec![
            Span::raw("质量: "),
            Span::styled(
                format!("{}%", quality_score),
                Style::default().fg(match quality_score {
                    80..=100 => Color::Green,
                    40..=79 => Color::Yellow,
                    _ => Color::Red,
                }),
            ),
        ]));

        // Network interfaces count (if available)
        chart_lines.push(Line::from(vec![
            Span::raw("接口: "),
            Span::styled(
                "检测中...",
                Style::default().fg(theme.colors.text_secondary),
            ),
        ]));

        let network_chart = Paragraph::new(chart_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("网络活动监控")
                    .border_style(theme.styles.widget_border),
            )
            .style(theme.styles.info);

        frame.render_widget(network_chart, area);
    }
}
