//! Plugin Manager Widget
//! 
//! This widget provides a comprehensive interface for managing plugins in the TUI.
//! It displays plugin lists, status information, and provides management operations.

use async_trait::async_trait;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, 
        Scrollbar, ScrollbarOrientation, ScrollbarState,
        Tabs, Wrap, Gauge,
    },
    Frame,
};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use rand;

use crate::core::{PluginInfo, PluginType};
use crate::plugins::types::{PluginStatus, PluginConfig};
use super::super::{
    action::Action,
    theme::Theme,
    widget::{Widget, WidgetId, WidgetContext, WidgetCapabilities, SizeConstraints, UpdateFrequency, WidgetError},
};

/// Plugin information with runtime data
#[derive(Debug, Clone)]
pub struct PluginDisplayInfo {
    pub info: PluginInfo,
    pub status: PluginStatus,
    pub config: Option<PluginConfig>,
    pub last_updated: DateTime<Utc>,
    pub resource_usage: ResourceUsage,
    pub tools_count: usize,
    pub dependencies: Vec<String>,
    pub health_status: PluginHealthStatus,
    pub status_history: Vec<PluginStatusChange>,
    pub health_history: Vec<PluginHealthCheck>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub monitoring_enabled: bool,
}

/// Plugin resource usage information
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub network_connections: u32,
    pub file_handles: u32,
    pub disk_io_read_mb: f64,
    pub disk_io_write_mb: f64,
    pub uptime_seconds: u64,
    pub last_activity: Option<DateTime<Utc>>,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 0.0,
            cpu_percent: 0.0,
            network_connections: 0,
            file_handles: 0,
            disk_io_read_mb: 0.0,
            disk_io_write_mb: 0.0,
            uptime_seconds: 0,
            last_activity: None,
        }
    }
}

/// Plugin health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginHealthStatus {
    Healthy,
    Warning(String),
    Critical(String),
    Unknown,
}

/// Plugin status change record
#[derive(Debug, Clone)]
pub struct PluginStatusChange {
    pub timestamp: DateTime<Utc>,
    pub from_status: PluginStatus,
    pub to_status: PluginStatus,
    pub reason: Option<String>,
}

/// Plugin health check result
#[derive(Debug, Clone)]
pub struct PluginHealthCheck {
    pub timestamp: DateTime<Utc>,
    pub status: PluginHealthStatus,
    pub checks: Vec<HealthCheckItem>,
    pub overall_score: f32, // 0.0 to 1.0
}

/// Individual health check item
#[derive(Debug, Clone)]
pub struct HealthCheckItem {
    pub name: String,
    pub status: HealthCheckStatus,
    pub message: String,
    pub severity: HealthCheckSeverity,
}

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthCheckStatus {
    Pass,
    Fail,
    Warning,
    Skip,
}

/// Health check severity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthCheckSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Plugin installation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginInstallationStatus {
    Installing,
    Completed,
    Failed(String),
}

/// Plugin dependency information
#[derive(Debug, Clone)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
    pub optional: bool,
    pub satisfied: bool,
    pub installed_version: Option<String>,
}

/// Plugin dependency conflict
#[derive(Debug, Clone)]
pub struct DependencyConflict {
    pub plugin_name: String,
    pub conflicting_plugin: String,
    pub conflict_type: ConflictType,
    pub description: String,
}

/// Types of dependency conflicts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    VersionMismatch,
    IncompatiblePlugins,
    CircularDependency,
    MissingDependency,
}

/// Dependency resolution result
#[derive(Debug, Clone)]
pub struct DependencyResolution {
    pub install_order: Vec<String>,
    pub conflicts: Vec<DependencyConflict>,
    pub warnings: Vec<String>,
    pub can_proceed: bool,
}

/// Plugin manager view modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewMode {
    List,
    Details,
    Market,
    Dependencies,
}

/// Plugin filter criteria
#[derive(Debug, Clone)]
pub struct PluginFilter {
    pub status: Option<PluginStatus>,
    pub plugin_type: Option<PluginType>,
    pub search_term: String,
    pub show_enabled_only: bool,
    pub show_installed_only: bool,
}

impl Default for PluginFilter {
    fn default() -> Self {
        Self {
            status: None,
            plugin_type: None,
            search_term: String::new(),
            show_enabled_only: false,
            show_installed_only: true,
        }
    }
}

/// Plugin sort order
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginSortOrder {
    NameAsc,
    NameDesc,
    StatusAsc,
    StatusDesc,
    TypeAsc,
    TypeDesc,
    LastUpdatedAsc,
    LastUpdatedDesc,
}

impl Default for PluginSortOrder {
    fn default() -> Self {
        Self::NameAsc
    }
}

/// Plugin Manager Widget implementation
pub struct PluginManagerWidget {
    // Widget infrastructure
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
    
    // Plugin data
    plugins: Vec<PluginDisplayInfo>,
    filtered_plugins: Vec<usize>,
    
    // UI state
    current_view: ViewMode,
    selected_index: usize,
    list_state: ListState,
    scroll_state: ScrollbarState,
    
    // Filtering and sorting
    filter: PluginFilter,
    sort_order: PluginSortOrder,
    
    // Input handling
    search_mode: bool,
    input_buffer: String,
    
    // Details view
    show_details_panel: bool,
    details_scroll: usize,
    
    // Market view (for future implementation)
    market_plugins: Vec<PluginInfo>,
    market_selected: usize,
    
    // Dependencies view
    dependency_graph: HashMap<String, Vec<String>>,
    
    // Status tracking
    last_refresh: Option<Instant>,
    refresh_interval: Duration,
    
    // Status monitoring
    monitoring_enabled: bool,
    status_update_interval: Duration,
    last_status_update: Option<Instant>,
    
    // Health checking
    health_check_enabled: bool,
    health_check_interval: Duration,
    last_health_check: Option<Instant>,
    
    // Error handling
    last_error: Option<String>,
    error_display_time: Option<Instant>,
}

impl PluginManagerWidget {
    /// Create a new Plugin Manager Widget
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        let capabilities = WidgetCapabilities {
            keyboard_input: true,
            mouse_input: true,
            focusable: true,
            resizable: true,
            scrollable: true,
            themeable: true,
            configurable: true,
        };
        
        let size_constraints = SizeConstraints::new()
            .min_size(60, 20)
            .preferred_size(120, 40);
        
        Self {
            context: WidgetContext::new(WidgetId::from("plugin_manager")),
            capabilities,
            size_constraints,
            
            plugins: Vec::new(),
            filtered_plugins: Vec::new(),
            
            current_view: ViewMode::List,
            selected_index: 0,
            list_state,
            scroll_state: ScrollbarState::default(),
            
            filter: PluginFilter::default(),
            sort_order: PluginSortOrder::default(),
            
            search_mode: false,
            input_buffer: String::new(),
            
            show_details_panel: false,
            details_scroll: 0,
            
            market_plugins: Vec::new(),
            market_selected: 0,
            
            dependency_graph: HashMap::new(),
            
            last_refresh: None,
            refresh_interval: Duration::from_secs(5),
            
            // Status monitoring
            monitoring_enabled: true,
            status_update_interval: Duration::from_secs(2),
            last_status_update: None,
            
            // Health checking
            health_check_enabled: true,
            health_check_interval: Duration::from_secs(30),
            last_health_check: None,
            
            last_error: None,
            error_display_time: None,
        }
    }
    
    /// Set the plugin data
    pub fn set_plugins(&mut self, plugins: Vec<PluginDisplayInfo>) {
        self.plugins = plugins;
        self.apply_filter_and_sort();
        self.update_selection();
        self.last_refresh = Some(Instant::now());
    }
    
    /// Get the currently selected plugin
    pub fn selected_plugin(&self) -> Option<&PluginDisplayInfo> {
        self.filtered_plugins
            .get(self.selected_index)
            .and_then(|&index| self.plugins.get(index))
    }
    
    /// Apply current filter and sort settings
    fn apply_filter_and_sort(&mut self) {
        // Apply filters
        self.filtered_plugins = self.plugins
            .iter()
            .enumerate()
            .filter(|(_, plugin)| self.matches_filter(plugin))
            .map(|(i, _)| i)
            .collect();
        
        // Apply sorting
        self.filtered_plugins.sort_by(|&a, &b| {
            let plugin_a = &self.plugins[a];
            let plugin_b = &self.plugins[b];
            
            match self.sort_order {
                PluginSortOrder::NameAsc => plugin_a.info.name.cmp(&plugin_b.info.name),
                PluginSortOrder::NameDesc => plugin_b.info.name.cmp(&plugin_a.info.name),
                PluginSortOrder::StatusAsc => plugin_a.status.cmp(&plugin_b.status),
                PluginSortOrder::StatusDesc => plugin_b.status.cmp(&plugin_a.status),
                PluginSortOrder::TypeAsc => plugin_a.info.plugin_type.cmp(&plugin_b.info.plugin_type),
                PluginSortOrder::TypeDesc => plugin_b.info.plugin_type.cmp(&plugin_a.info.plugin_type),
                PluginSortOrder::LastUpdatedAsc => plugin_a.last_updated.cmp(&plugin_b.last_updated),
                PluginSortOrder::LastUpdatedDesc => plugin_b.last_updated.cmp(&plugin_a.last_updated),
            }
        });
    }
    
    /// Check if a plugin matches the current filter
    fn matches_filter(&self, plugin: &PluginDisplayInfo) -> bool {
        // Status filter
        if let Some(status) = &self.filter.status {
            if plugin.status != *status {
                return false;
            }
        }
        
        // Type filter
        if let Some(plugin_type) = &self.filter.plugin_type {
            if plugin.info.plugin_type != *plugin_type {
                return false;
            }
        }
        
        // Search term filter
        if !self.filter.search_term.is_empty() {
            let search_lower = self.filter.search_term.to_lowercase();
            let matches_name = plugin.info.name.to_lowercase().contains(&search_lower);
            let matches_description = plugin.info.description
                .as_ref()
                .map_or(false, |desc| desc.to_lowercase().contains(&search_lower));
            let matches_author = plugin.info.author
                .as_ref()
                .map_or(false, |author| author.to_lowercase().contains(&search_lower));
            
            if !matches_name && !matches_description && !matches_author {
                return false;
            }
        }
        
        // Enabled filter
        if self.filter.show_enabled_only {
            if let Some(config) = &plugin.config {
                if !config.enabled {
                    return false;
                }
            }
        }
        
        // Installed filter
        if self.filter.show_installed_only {
            if !matches!(plugin.status, PluginStatus::Ready | PluginStatus::Running | PluginStatus::Error) {
                return false;
            }
        }
        
        true
    }
    
    /// Update the selection state
    fn update_selection(&mut self) {
        if self.selected_index >= self.filtered_plugins.len() && !self.filtered_plugins.is_empty() {
            self.selected_index = self.filtered_plugins.len() - 1;
        }
        
        self.list_state.select(Some(self.selected_index));
        
        // Update scroll state
        self.scroll_state = self.scroll_state.content_length(self.filtered_plugins.len());
        self.scroll_state = self.scroll_state.position(self.selected_index);
    }
    
    /// Move selection up
    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.filtered_plugins.is_empty() {
            self.selected_index = self.filtered_plugins.len() - 1;
        }
        self.update_selection();
    }
    
    /// Move selection down
    fn move_selection_down(&mut self) {
        if self.selected_index + 1 < self.filtered_plugins.len() {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
        self.update_selection();
    }
    
    /// Toggle search mode
    fn toggle_search_mode(&mut self) {
        self.search_mode = !self.search_mode;
        if !self.search_mode {
            self.input_buffer.clear();
        }
    }
    
    /// Apply search filter
    fn apply_search(&mut self) {
        self.filter.search_term = self.input_buffer.clone();
        self.apply_filter_and_sort();
        self.update_selection();
        self.search_mode = false;
        self.input_buffer.clear();
    }
    
    /// Clear all filters
    fn clear_filters(&mut self) {
        self.filter = PluginFilter::default();
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Toggle details panel
    fn toggle_details_panel(&mut self) {
        self.show_details_panel = !self.show_details_panel;
        self.details_scroll = 0;
    }
    
    /// Switch view mode
    fn switch_view(&mut self, view: ViewMode) {
        self.current_view = view;
        self.details_scroll = 0;
    }
    
    /// Set error message
    fn set_error(&mut self, error: String) {
        self.last_error = Some(error);
        self.error_display_time = Some(Instant::now());
    }
    
    /// Clear error message if it's expired
    fn clear_expired_error(&mut self) {
        if let Some(error_time) = self.error_display_time {
            if error_time.elapsed() > Duration::from_secs(5) {
                self.last_error = None;
                self.error_display_time = None;
            }
        }
    }
    
    /// Check if refresh is needed
    fn needs_refresh(&self) -> bool {
        self.last_refresh
            .map_or(true, |last| last.elapsed() >= self.refresh_interval)
    }
    
    /// Check if status update is needed
    fn needs_status_update(&self) -> bool {
        if !self.monitoring_enabled {
            return false;
        }
        
        self.last_status_update
            .map_or(true, |last| last.elapsed() >= self.status_update_interval)
    }
    
    /// Check if health check is needed
    fn needs_health_check(&self) -> bool {
        if !self.health_check_enabled {
            return false;
        }
        
        self.last_health_check
            .map_or(true, |last| last.elapsed() >= self.health_check_interval)
    }
    
    /// Update plugin status monitoring
    fn update_plugin_status(&mut self, plugin_name: &str, new_status: PluginStatus, reason: Option<String>) {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.info.name == plugin_name) {
            let old_status = plugin.status;
            
            // Only update if status actually changed
            if old_status != new_status {
                plugin.status = new_status;
                plugin.last_updated = Utc::now();
                
                // Record status change
                let status_change = PluginStatusChange {
                    timestamp: Utc::now(),
                    from_status: old_status,
                    to_status: new_status,
                    reason,
                };
                
                plugin.status_history.push(status_change);
                
                // Keep only last 100 status changes
                if plugin.status_history.len() > 100 {
                    plugin.status_history.remove(0);
                }
                
                tracing::info!(
                    "Plugin '{}' status changed from {:?} to {:?}",
                    plugin_name, old_status, new_status
                );
            }
        }
        
        self.last_status_update = Some(Instant::now());
    }
    
    /// Perform health check on a plugin
    fn perform_health_check(&mut self, plugin_name: &str) -> PluginHealthCheck {
        let mut checks = Vec::new();
        let mut overall_score = 1.0f32;
        
        if let Some(plugin) = self.plugins.iter().find(|p| p.info.name == plugin_name) {
            // Check 1: Plugin status
            let status_check = match plugin.status {
                PluginStatus::Ready | PluginStatus::Running => HealthCheckItem {
                    name: "Status Check".to_string(),
                    status: HealthCheckStatus::Pass,
                    message: "Plugin is operational".to_string(),
                    severity: HealthCheckSeverity::Info,
                },
                PluginStatus::Error => HealthCheckItem {
                    name: "Status Check".to_string(),
                    status: HealthCheckStatus::Fail,
                    message: "Plugin is in error state".to_string(),
                    severity: HealthCheckSeverity::Critical,
                },
                PluginStatus::Initializing | PluginStatus::ShuttingDown => HealthCheckItem {
                    name: "Status Check".to_string(),
                    status: HealthCheckStatus::Warning,
                    message: "Plugin is in transition state".to_string(),
                    severity: HealthCheckSeverity::Medium,
                },
                _ => HealthCheckItem {
                    name: "Status Check".to_string(),
                    status: HealthCheckStatus::Fail,
                    message: "Plugin is not ready".to_string(),
                    severity: HealthCheckSeverity::High,
                },
            };
            
            if status_check.status == HealthCheckStatus::Fail {
                overall_score -= 0.4;
            } else if status_check.status == HealthCheckStatus::Warning {
                overall_score -= 0.2;
            }
            checks.push(status_check);
            
            // Check 2: Resource usage
            let memory_check = if plugin.resource_usage.memory_mb > 1024.0 {
                overall_score -= 0.2;
                HealthCheckItem {
                    name: "Memory Usage".to_string(),
                    status: HealthCheckStatus::Warning,
                    message: format!("High memory usage: {:.1}MB", plugin.resource_usage.memory_mb),
                    severity: HealthCheckSeverity::Medium,
                }
            } else {
                HealthCheckItem {
                    name: "Memory Usage".to_string(),
                    status: HealthCheckStatus::Pass,
                    message: format!("Memory usage: {:.1}MB", plugin.resource_usage.memory_mb),
                    severity: HealthCheckSeverity::Info,
                }
            };
            checks.push(memory_check);
            
            // Check 3: CPU usage
            let cpu_check = if plugin.resource_usage.cpu_percent > 80.0 {
                overall_score -= 0.2;
                HealthCheckItem {
                    name: "CPU Usage".to_string(),
                    status: HealthCheckStatus::Warning,
                    message: format!("High CPU usage: {:.1}%", plugin.resource_usage.cpu_percent),
                    severity: HealthCheckSeverity::Medium,
                }
            } else {
                HealthCheckItem {
                    name: "CPU Usage".to_string(),
                    status: HealthCheckStatus::Pass,
                    message: format!("CPU usage: {:.1}%", plugin.resource_usage.cpu_percent),
                    severity: HealthCheckSeverity::Info,
                }
            };
            checks.push(cpu_check);
            
            // Check 4: Last activity
            let activity_check = if let Some(last_activity) = plugin.resource_usage.last_activity {
                let inactive_duration = Utc::now().signed_duration_since(last_activity);
                if inactive_duration.num_minutes() > 30 {
                    overall_score -= 0.1;
                    HealthCheckItem {
                        name: "Activity Check".to_string(),
                        status: HealthCheckStatus::Warning,
                        message: format!("No activity for {} minutes", inactive_duration.num_minutes()),
                        severity: HealthCheckSeverity::Low,
                    }
                } else {
                    HealthCheckItem {
                        name: "Activity Check".to_string(),
                        status: HealthCheckStatus::Pass,
                        message: "Recent activity detected".to_string(),
                        severity: HealthCheckSeverity::Info,
                    }
                }
            } else {
                HealthCheckItem {
                    name: "Activity Check".to_string(),
                    status: HealthCheckStatus::Skip,
                    message: "No activity data available".to_string(),
                    severity: HealthCheckSeverity::Info,
                }
            };
            checks.push(activity_check);
            
            // Check 5: Dependencies
            let dep_check = if plugin.dependencies.is_empty() {
                HealthCheckItem {
                    name: "Dependencies".to_string(),
                    status: HealthCheckStatus::Pass,
                    message: "No dependencies".to_string(),
                    severity: HealthCheckSeverity::Info,
                }
            } else {
                // In a real implementation, we would check if dependencies are satisfied
                HealthCheckItem {
                    name: "Dependencies".to_string(),
                    status: HealthCheckStatus::Pass,
                    message: format!("{} dependencies", plugin.dependencies.len()),
                    severity: HealthCheckSeverity::Info,
                }
            };
            checks.push(dep_check);
        }
        
        // Ensure score is within bounds
        overall_score = overall_score.max(0.0).min(1.0);
        
        // Determine overall health status
        let health_status = if overall_score >= 0.8 {
            PluginHealthStatus::Healthy
        } else if overall_score >= 0.6 {
            let warnings: Vec<String> = checks.iter()
                .filter(|c| c.status == HealthCheckStatus::Warning)
                .map(|c| c.message.clone())
                .collect();
            PluginHealthStatus::Warning(warnings.join("; "))
        } else {
            let errors: Vec<String> = checks.iter()
                .filter(|c| c.status == HealthCheckStatus::Fail)
                .map(|c| c.message.clone())
                .collect();
            PluginHealthStatus::Critical(errors.join("; "))
        };
        
        PluginHealthCheck {
            timestamp: Utc::now(),
            status: health_status,
            checks,
            overall_score,
        }
    }
    
    /// Update health status for a plugin
    fn update_plugin_health(&mut self, plugin_name: &str) {
        let health_check = self.perform_health_check(plugin_name);
        
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.info.name == plugin_name) {
            plugin.health_status = health_check.status.clone();
            plugin.health_history.push(health_check);
            plugin.last_health_check = Some(Utc::now());
            
            // Keep only last 50 health checks
            if plugin.health_history.len() > 50 {
                plugin.health_history.remove(0);
            }
        }
        
        self.last_health_check = Some(Instant::now());
    }
    
    /// Update resource usage for a plugin
    fn update_plugin_resource_usage(&mut self, plugin_name: &str, resource_usage: ResourceUsage) {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.info.name == plugin_name) {
            plugin.resource_usage = resource_usage;
            plugin.last_updated = Utc::now();
        }
    }
    
    /// Get plugin status history
    pub fn get_plugin_status_history(&self, plugin_name: &str) -> Vec<PluginStatusChange> {
        self.plugins
            .iter()
            .find(|p| p.info.name == plugin_name)
            .map(|p| p.status_history.clone())
            .unwrap_or_default()
    }
    
    /// Get plugin health history
    pub fn get_plugin_health_history(&self, plugin_name: &str) -> Vec<PluginHealthCheck> {
        self.plugins
            .iter()
            .find(|p| p.info.name == plugin_name)
            .map(|p| p.health_history.clone())
            .unwrap_or_default()
    }
    
    /// Enable/disable monitoring for a plugin
    pub fn set_plugin_monitoring(&mut self, plugin_name: &str, enabled: bool) {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.info.name == plugin_name) {
            plugin.monitoring_enabled = enabled;
            tracing::info!("Plugin '{}' monitoring {}", plugin_name, if enabled { "enabled" } else { "disabled" });
        }
    }
    
    /// Toggle monitoring for all plugins
    pub fn toggle_monitoring(&mut self) {
        self.monitoring_enabled = !self.monitoring_enabled;
        tracing::info!("Global plugin monitoring {}", if self.monitoring_enabled { "enabled" } else { "disabled" });
    }
    
    /// Toggle health checking for all plugins
    pub fn toggle_health_checking(&mut self) {
        self.health_check_enabled = !self.health_check_enabled;
        tracing::info!("Global plugin health checking {}", if self.health_check_enabled { "enabled" } else { "disabled" });
    }
    
    /// Install a plugin
    pub async fn install_plugin(&mut self, plugin_name: &str, plugin_url: Option<String>) -> Result<(), String> {
        tracing::info!("Installing plugin: {}", plugin_name);
        
        // Set status to installing
        self.set_plugin_installation_status(plugin_name, PluginInstallationStatus::Installing);
        
        // Simulate installation process
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // In a real implementation, this would:
        // 1. Download plugin from URL or registry
        // 2. Verify plugin signature and integrity
        // 3. Check dependencies
        // 4. Install plugin files
        // 5. Register plugin with plugin manager
        // 6. Initialize plugin
        
        // For now, simulate success/failure
        if plugin_name.contains("invalid") {
            self.set_plugin_installation_status(plugin_name, PluginInstallationStatus::Failed("Invalid plugin".to_string()));
            return Err("Plugin installation failed: Invalid plugin".to_string());
        }
        
        // Create a mock plugin info for newly installed plugin
        let new_plugin = PluginDisplayInfo {
            info: PluginInfo {
                name: plugin_name.to_string(),
                version: "1.0.0".to_string(),
                plugin_type: crate::core::PluginType::Native,
                author: Some("System".to_string()),
                description: Some(format!("Newly installed plugin: {}", plugin_name)),
                metadata: std::collections::HashMap::new(),
            },
            status: PluginStatus::Ready,
            config: Some(PluginConfig {
                name: plugin_name.to_string(),
                plugin_type: crate::core::PluginType::Native,
                enabled: true,
                security_policy: crate::plugins::types::SecurityPolicy::default(),
                resource_limits: crate::plugins::types::ResourceLimits::default(),
                config: serde_json::Value::Null,
                metadata: std::collections::HashMap::new(),
                dependencies: Vec::new(),
            }),
            last_updated: chrono::Utc::now(),
            resource_usage: ResourceUsage::default(),
            tools_count: 1,
            dependencies: Vec::new(),
            health_status: PluginHealthStatus::Healthy,
            status_history: Vec::new(),
            health_history: Vec::new(),
            last_health_check: Some(chrono::Utc::now()),
            monitoring_enabled: true,
        };
        
        // Add to plugin list if not already present
        if !self.plugins.iter().any(|p| p.info.name == plugin_name) {
            self.plugins.push(new_plugin);
            self.apply_filter_and_sort();
            self.update_selection();
        }
        
        self.set_plugin_installation_status(plugin_name, PluginInstallationStatus::Completed);
        tracing::info!("Successfully installed plugin: {}", plugin_name);
        Ok(())
    }
    
    /// Uninstall a plugin
    pub async fn uninstall_plugin(&mut self, plugin_name: &str) -> Result<(), String> {
        tracing::info!("Uninstalling plugin: {}", plugin_name);
        
        // Check if plugin exists
        if !self.plugins.iter().any(|p| p.info.name == plugin_name) {
            return Err(format!("Plugin '{}' not found", plugin_name));
        }
        
        // Check if plugin has dependents
        let dependents = self.get_plugin_dependents(plugin_name);
        if !dependents.is_empty() {
            return Err(format!("Cannot uninstall plugin '{}': required by {}", 
                plugin_name, dependents.join(", ")));
        }
        
        // Set status to uninstalling
        self.update_plugin_status(plugin_name, PluginStatus::ShuttingDown, Some("Uninstalling".to_string()));
        
        // Simulate uninstallation process
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // In a real implementation, this would:
        // 1. Stop plugin if running
        // 2. Unregister plugin tools
        // 3. Clean up plugin files
        // 4. Remove plugin configuration
        // 5. Update dependency graph
        
        // Remove plugin from list
        self.plugins.retain(|p| p.info.name != plugin_name);
        self.apply_filter_and_sort();
        self.update_selection();
        
        tracing::info!("Successfully uninstalled plugin: {}", plugin_name);
        Ok(())
    }
    
    /// Enable a plugin
    pub async fn enable_plugin(&mut self, plugin_name: &str) -> Result<(), String> {
        tracing::info!("Enabling plugin: {}", plugin_name);
        
        // Check dependencies first
        let missing_deps = self.check_plugin_dependencies(plugin_name);
        if !missing_deps.is_empty() {
            return Err(format!("Cannot enable plugin '{}': missing dependencies: {}", 
                plugin_name, missing_deps.join(", ")));
        }
        
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.info.name == plugin_name) {
            if let Some(config) = &mut plugin.config {
                if config.enabled {
                    return Err(format!("Plugin '{}' is already enabled", plugin_name));
                }
                
                config.enabled = true;
                plugin.status = PluginStatus::Initializing;
                plugin.last_updated = chrono::Utc::now();
                
                // Simulate initialization
                tokio::time::sleep(Duration::from_millis(50)).await;
                
                plugin.status = PluginStatus::Ready;
                self.update_plugin_status(plugin_name, PluginStatus::Ready, Some("Enabled".to_string()));
                
                tracing::info!("Successfully enabled plugin: {}", plugin_name);
                Ok(())
            } else {
                Err(format!("Plugin '{}' has no configuration", plugin_name))
            }
        } else {
            Err(format!("Plugin '{}' not found", plugin_name))
        }
    }
    
    /// Disable a plugin
    pub async fn disable_plugin(&mut self, plugin_name: &str) -> Result<(), String> {
        tracing::info!("Disabling plugin: {}", plugin_name);
        
        // Check if other plugins depend on this one
        let dependents = self.get_plugin_dependents(plugin_name);
        if !dependents.is_empty() {
            return Err(format!("Cannot disable plugin '{}': required by {}", 
                plugin_name, dependents.join(", ")));
        }
        
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.info.name == plugin_name) {
            if let Some(config) = &mut plugin.config {
                if !config.enabled {
                    return Err(format!("Plugin '{}' is already disabled", plugin_name));
                }
                
                config.enabled = false;
                plugin.status = PluginStatus::ShuttingDown;
                plugin.last_updated = chrono::Utc::now();
                
                // Simulate shutdown
                tokio::time::sleep(Duration::from_millis(50)).await;
                
                plugin.status = PluginStatus::Shutdown;
                self.update_plugin_status(plugin_name, PluginStatus::Shutdown, Some("Disabled".to_string()));
                
                tracing::info!("Successfully disabled plugin: {}", plugin_name);
                Ok(())
            } else {
                Err(format!("Plugin '{}' has no configuration", plugin_name))
            }
        } else {
            Err(format!("Plugin '{}' not found", plugin_name))
        }
    }
    
    /// Reload a plugin
    pub async fn reload_plugin(&mut self, plugin_name: &str) -> Result<(), String> {
        tracing::info!("Reloading plugin: {}", plugin_name);
        
        let was_enabled = self.plugins.iter()
            .find(|p| p.info.name == plugin_name)
            .and_then(|p| p.config.as_ref())
            .map_or(false, |c| c.enabled);
        
        // Disable first if enabled
        if was_enabled {
            self.disable_plugin(plugin_name).await?;
        }
        
        // Update plugin status
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.info.name == plugin_name) {
            plugin.status = PluginStatus::Initializing;
            plugin.last_updated = chrono::Utc::now();
            
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            // Re-enable if it was enabled before
            if was_enabled {
                self.enable_plugin(plugin_name).await?;
            } else {
                plugin.status = PluginStatus::Ready;
                self.update_plugin_status(plugin_name, PluginStatus::Ready, Some("Reloaded".to_string()));
            }
            
            tracing::info!("Successfully reloaded plugin: {}", plugin_name);
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", plugin_name))
        }
    }
    
    /// Update a plugin to the latest version
    pub async fn update_plugin(&mut self, plugin_name: &str) -> Result<(), String> {
        tracing::info!("Updating plugin: {}", plugin_name);
        
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.info.name == plugin_name) {
            // Simulate version check
            let current_version = &plugin.info.version;
            let new_version = format!("{}.1", current_version); // Simulate version increment
            
            if current_version == &new_version {
                return Err(format!("Plugin '{}' is already up to date", plugin_name));
            }
            
            // Simulate update process
            plugin.status = PluginStatus::Initializing;
            plugin.last_updated = chrono::Utc::now();
            
            tokio::time::sleep(Duration::from_millis(150)).await;
            
            // Update version
            plugin.info.version = new_version.clone();
            plugin.status = PluginStatus::Ready;
            
            self.update_plugin_status(plugin_name, PluginStatus::Ready, 
                Some(format!("Updated to version {}", new_version)));
            
            tracing::info!("Successfully updated plugin '{}' to version {}", plugin_name, new_version);
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", plugin_name))
        }
    }
    
    /// Get plugins that depend on the given plugin
    fn get_plugin_dependents(&self, plugin_name: &str) -> Vec<String> {
        self.plugins
            .iter()
            .filter(|p| p.dependencies.contains(&plugin_name.to_string()))
            .map(|p| p.info.name.clone())
            .collect()
    }
    
    /// Check missing dependencies for a plugin
    fn check_plugin_dependencies(&self, plugin_name: &str) -> Vec<String> {
        if let Some(plugin) = self.plugins.iter().find(|p| p.info.name == plugin_name) {
            let installed_plugins: std::collections::HashSet<String> = self.plugins
                .iter()
                .filter(|p| p.config.as_ref().map_or(false, |c| c.enabled))
                .map(|p| p.info.name.clone())
                .collect();
            
            plugin.dependencies
                .iter()
                .filter(|dep| !installed_plugins.contains(*dep))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Set plugin installation status
    fn set_plugin_installation_status(&mut self, plugin_name: &str, status: PluginInstallationStatus) {
        // In a real implementation, this would update installation progress
        match status {
            PluginInstallationStatus::Installing => {
                tracing::info!("Plugin '{}' installation started", plugin_name);
            }
            PluginInstallationStatus::Completed => {
                tracing::info!("Plugin '{}' installation completed", plugin_name);
            }
            PluginInstallationStatus::Failed(ref error) => {
                tracing::error!("Plugin '{}' installation failed: {}", plugin_name, error);
                self.set_error(format!("Installation failed: {}", error));
            }
        }
    }
    
    /// Analyze plugin dependencies and detect conflicts
    pub fn analyze_dependencies(&self) -> DependencyResolution {
        let mut conflicts = Vec::new();
        let mut warnings = Vec::new();
        let mut install_order = Vec::new();
        
        // Build dependency graph
        let mut dependency_graph = std::collections::HashMap::new();
        let mut reverse_deps = std::collections::HashMap::new();
        
        for plugin in &self.plugins {
            dependency_graph.insert(plugin.info.name.clone(), plugin.dependencies.clone());
            
            for dep in &plugin.dependencies {
                reverse_deps.entry(dep.clone())
                    .or_insert_with(Vec::new)
                    .push(plugin.info.name.clone());
            }
        }
        
        // Detect circular dependencies
        let circular_deps = self.detect_circular_dependencies(&dependency_graph);
        for cycle in circular_deps {
            conflicts.push(DependencyConflict {
                plugin_name: cycle[0].clone(),
                conflicting_plugin: cycle[1].clone(),
                conflict_type: ConflictType::CircularDependency,
                description: format!("Circular dependency detected: {}", cycle.join(" -> ")),
            });
        }
        
        // Check for missing dependencies
        let installed_plugins: std::collections::HashSet<String> = self.plugins
            .iter()
            .map(|p| p.info.name.clone())
            .collect();
        
        for plugin in &self.plugins {
            for dep in &plugin.dependencies {
                if !installed_plugins.contains(dep) {
                    conflicts.push(DependencyConflict {
                        plugin_name: plugin.info.name.clone(),
                        conflicting_plugin: dep.clone(),
                        conflict_type: ConflictType::MissingDependency,
                        description: format!("Plugin '{}' requires '{}' which is not installed", 
                            plugin.info.name, dep),
                    });
                }
            }
        }
        
        // Generate topological sort for installation order
        install_order = self.topological_sort(&dependency_graph);
        
        // Check for version conflicts (simplified)
        for plugin in &self.plugins {
            for dep in &plugin.dependencies {
                if let Some(dep_plugin) = self.plugins.iter().find(|p| p.info.name == *dep) {
                    // Simplified version check - in reality this would parse semantic versions
                    if plugin.info.version != dep_plugin.info.version {
                        warnings.push(format!("Version mismatch: '{}' expects '{}' but '{}' is installed",
                            plugin.info.name, plugin.info.version, dep_plugin.info.version));
                    }
                }
            }
        }
        
        let can_proceed = conflicts.iter().all(|c| c.conflict_type != ConflictType::CircularDependency);
        
        DependencyResolution {
            install_order,
            conflicts,
            warnings,
            can_proceed,
        }
    }
    
    /// Detect circular dependencies using DFS
    fn detect_circular_dependencies(&self, graph: &std::collections::HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        let mut cycles = Vec::new();
        
        for node in graph.keys() {
            if !visited.contains(node) {
                self.dfs_cycle_detection(node, graph, &mut visited, &mut rec_stack, &mut Vec::new(), &mut cycles);
            }
        }
        
        cycles
    }
    
    /// DFS helper for cycle detection
    fn dfs_cycle_detection(
        &self,
        node: &str,
        graph: &std::collections::HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycle_detection(neighbor, graph, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    if let Some(cycle_start) = path.iter().position(|x| x == neighbor) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }
        
        rec_stack.remove(node);
        path.pop();
    }
    
    /// Perform topological sort to determine installation order
    fn topological_sort(&self, graph: &std::collections::HashMap<String, Vec<String>>) -> Vec<String> {
        let mut in_degree = std::collections::HashMap::new();
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        
        // Initialize in-degree count
        for node in graph.keys() {
            in_degree.insert(node.clone(), 0);
        }
        
        // Calculate in-degrees
        for (_, neighbors) in graph {
            for neighbor in neighbors {
                *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
            }
        }
        
        // Find nodes with no incoming edges
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }
        
        // Process nodes
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            
            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// Get detailed dependency information for a plugin
    pub fn get_plugin_dependencies(&self, plugin_name: &str) -> Vec<PluginDependency> {
        if let Some(plugin) = self.plugins.iter().find(|p| p.info.name == plugin_name) {
            plugin.dependencies
                .iter()
                .map(|dep_name| {
                    let dep_plugin = self.plugins.iter().find(|p| p.info.name == *dep_name);
                    
                    PluginDependency {
                        name: dep_name.clone(),
                        version_requirement: ">=1.0.0".to_string(), // Simplified
                        optional: false, // Simplified - in reality this would be configurable
                        satisfied: dep_plugin.is_some(),
                        installed_version: dep_plugin.map(|p| p.info.version.clone()),
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get plugins that would be affected by installing/uninstalling a plugin
    pub fn get_dependency_impact(&self, plugin_name: &str, operation: &str) -> Vec<String> {
        match operation {
            "install" => {
                // Return plugins that this plugin depends on
                if let Some(plugin) = self.plugins.iter().find(|p| p.info.name == plugin_name) {
                    plugin.dependencies.clone()
                } else {
                    Vec::new()
                }
            }
            "uninstall" => {
                // Return plugins that depend on this plugin
                self.get_plugin_dependents(plugin_name)
            }
            _ => Vec::new(),
        }
    }
    
    /// Resolve dependencies for batch operations
    pub fn resolve_batch_dependencies(&self, plugin_names: &[String], operation: &str) -> DependencyResolution {
        let mut all_plugins = std::collections::HashSet::new();
        let mut conflicts = Vec::new();
        let mut warnings = Vec::new();
        
        // Collect all affected plugins
        for plugin_name in plugin_names {
            all_plugins.insert(plugin_name.clone());
            let impact = self.get_dependency_impact(plugin_name, operation);
            all_plugins.extend(impact);
        }
        
        // Check for conflicts in batch operation
        if operation == "uninstall" {
            for plugin_name in plugin_names {
                let dependents = self.get_plugin_dependents(plugin_name);
                for dependent in dependents {
                    if !plugin_names.contains(&dependent) {
                        conflicts.push(DependencyConflict {
                            plugin_name: plugin_name.clone(),
                            conflicting_plugin: dependent.clone(),
                            conflict_type: ConflictType::MissingDependency,
                            description: format!("Cannot uninstall '{}': required by '{}'", 
                                plugin_name, dependent),
                        });
                    }
                }
            }
        }
        
        let install_order = if operation == "install" {
            // For installation, use topological sort
            let mut graph = std::collections::HashMap::new();
            for plugin_name in &all_plugins {
                if let Some(plugin) = self.plugins.iter().find(|p| p.info.name == *plugin_name) {
                    graph.insert(plugin_name.clone(), plugin.dependencies.clone());
                } else {
                    graph.insert(plugin_name.clone(), Vec::new());
                }
            }
            self.topological_sort(&graph)
        } else {
            // For uninstallation, reverse the dependency order
            let mut order: Vec<String> = all_plugins.into_iter().collect();
            order.reverse();
            order
        };
        
        let can_proceed = conflicts.is_empty();
        
        DependencyResolution {
            install_order,
            conflicts,
            warnings,
            can_proceed,
        }
    }
    
    /// Load market plugins from registry
    pub async fn load_market_plugins(&mut self) -> Result<(), String> {
        tracing::info!("Loading plugins from market registry");
        
        // Simulate loading from remote registry
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Mock market plugins
        self.market_plugins = vec![
            PluginInfo {
                name: "text-processor".to_string(),
                version: "2.1.0".to_string(),
                plugin_type: crate::core::PluginType::Python,
                author: Some("Community".to_string()),
                description: Some("Advanced text processing plugin with NLP capabilities".to_string()),
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    meta.insert("category".to_string(), serde_json::Value::String("text-processing".to_string()));
                    meta.insert("rating".to_string(), serde_json::Value::String("4.8".to_string()));
                    meta.insert("downloads".to_string(), serde_json::Value::String("15420".to_string()));
                    meta.insert("homepage".to_string(), serde_json::Value::String("https://github.com/community/text-processor".to_string()));
                    meta.insert("license".to_string(), serde_json::Value::String("MIT".to_string()));
                    meta
                },
            },
            PluginInfo {
                name: "data-analyzer".to_string(),
                version: "1.5.3".to_string(),
                plugin_type: crate::core::PluginType::NodeJs,
                author: Some("DataCorp".to_string()),
                description: Some("Comprehensive data analysis and visualization plugin".to_string()),
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    meta.insert("category".to_string(), serde_json::Value::String("data-analysis".to_string()));
                    meta.insert("rating".to_string(), serde_json::Value::String("4.6".to_string()));
                    meta.insert("downloads".to_string(), serde_json::Value::String("8932".to_string()));
                    meta.insert("homepage".to_string(), serde_json::Value::String("https://datacorp.com/analyzer".to_string()));
                    meta.insert("license".to_string(), serde_json::Value::String("Apache-2.0".to_string()));
                    meta
                },
            },
            PluginInfo {
                name: "security-scanner".to_string(),
                version: "3.0.1".to_string(),
                plugin_type: crate::core::PluginType::Native,
                author: Some("SecureTeam".to_string()),
                description: Some("Security vulnerability scanner and compliance checker".to_string()),
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    meta.insert("category".to_string(), serde_json::Value::String("security".to_string()));
                    meta.insert("rating".to_string(), serde_json::Value::String("4.9".to_string()));
                    meta.insert("downloads".to_string(), serde_json::Value::String("23156".to_string()));
                    meta.insert("homepage".to_string(), serde_json::Value::String("https://secureteam.io/scanner".to_string()));
                    meta.insert("license".to_string(), serde_json::Value::String("GPL-3.0".to_string()));
                    meta
                },
            },
            PluginInfo {
                name: "workflow-optimizer".to_string(),
                version: "1.2.0".to_string(),
                plugin_type: crate::core::PluginType::Docker,
                author: Some("OptimizeCorp".to_string()),
                description: Some("AI-powered workflow optimization and performance tuning".to_string()),
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    meta.insert("category".to_string(), serde_json::Value::String("optimization".to_string()));
                    meta.insert("rating".to_string(), serde_json::Value::String("4.7".to_string()));
                    meta.insert("downloads".to_string(), serde_json::Value::String("5678".to_string()));
                    meta.insert("homepage".to_string(), serde_json::Value::String("https://optimize.corp/workflow".to_string()));
                    meta.insert("license".to_string(), serde_json::Value::String("Commercial".to_string()));
                    meta
                },
            },
            PluginInfo {
                name: "api-connector".to_string(),
                version: "2.3.1".to_string(),
                plugin_type: crate::core::PluginType::Python,
                author: Some("APITeam".to_string()),
                description: Some("Universal API connector with authentication and rate limiting".to_string()),
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    meta.insert("category".to_string(), serde_json::Value::String("integration".to_string()));
                    meta.insert("rating".to_string(), serde_json::Value::String("4.5".to_string()));
                    meta.insert("downloads".to_string(), serde_json::Value::String("12890".to_string()));
                    meta.insert("homepage".to_string(), serde_json::Value::String("https://github.com/apiteam/connector".to_string()));
                    meta.insert("license".to_string(), serde_json::Value::String("BSD-3-Clause".to_string()));
                    meta
                },
            },
        ];
        
        tracing::info!("Loaded {} plugins from market", self.market_plugins.len());
        Ok(())
    }
    
    /// Search market plugins
    pub fn search_market_plugins(&self, query: &str) -> Vec<&PluginInfo> {
        let query_lower = query.to_lowercase();
        
        self.market_plugins
            .iter()
            .filter(|plugin| {
                plugin.name.to_lowercase().contains(&query_lower) ||
                plugin.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower)) ||
                plugin.author.as_ref().map_or(false, |author| author.to_lowercase().contains(&query_lower)) ||
                plugin.metadata.values().any(|value| {
                    match value {
                        serde_json::Value::String(s) => s.to_lowercase().contains(&query_lower),
                        _ => false,
                    }
                })
            })
            .collect()
    }
    
    /// Filter market plugins by category
    pub fn filter_market_plugins(&self, category: Option<&str>, plugin_type: Option<crate::core::PluginType>) -> Vec<&PluginInfo> {
        self.market_plugins
            .iter()
            .filter(|plugin| {
                let category_match = category.map_or(true, |cat| {
                    plugin.metadata.get("category").and_then(|v| v.as_str()).map_or(false, |plugin_cat| plugin_cat == cat)
                });
                
                let type_match = plugin_type.as_ref().map_or(true, |ptype| &plugin.plugin_type == ptype);
                
                category_match && type_match
            })
            .collect()
    }
    
    /// Get plugin details from market
    pub fn get_market_plugin_details(&self, plugin_name: &str) -> Option<&PluginInfo> {
        self.market_plugins.iter().find(|p| p.name == plugin_name)
    }
    
    /// Install plugin from market
    pub async fn install_from_market(&mut self, plugin_name: &str) -> Result<(), String> {
        tracing::info!("Installing plugin '{}' from market", plugin_name);
        
        // Find plugin in market
        let market_plugin = self.market_plugins
            .iter()
            .find(|p| p.name == plugin_name)
            .cloned()
            .ok_or_else(|| format!("Plugin '{}' not found in market", plugin_name))?;
        
        // Check if already installed
        if self.plugins.iter().any(|p| p.info.name == plugin_name) {
            return Err(format!("Plugin '{}' is already installed", plugin_name));
        }
        
        // Simulate download and installation
        self.set_plugin_installation_status(plugin_name, PluginInstallationStatus::Installing);
        
        // Simulate download progress
        for i in 1..=5 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            tracing::debug!("Downloading plugin '{}': {}%", plugin_name, i * 20);
        }
        
        // Simulate installation verification
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Create installed plugin from market info
        let installed_plugin = PluginDisplayInfo {
            info: market_plugin.clone(),
            status: PluginStatus::Ready,
            config: Some(PluginConfig {
                name: plugin_name.to_string(),
                plugin_type: market_plugin.plugin_type,
                enabled: true,
                security_policy: crate::plugins::types::SecurityPolicy::default(),
                resource_limits: crate::plugins::types::ResourceLimits::default(),
                config: serde_json::Value::Null,
                metadata: std::collections::HashMap::new(),
                dependencies: Vec::new(),
            }),
            last_updated: chrono::Utc::now(),
            resource_usage: ResourceUsage::default(),
            tools_count: rand::random::<usize>() % 5 + 1, // Random number of tools
            dependencies: Vec::new(), // Simplified - in reality would parse from plugin manifest
            health_status: PluginHealthStatus::Healthy,
            status_history: Vec::new(),
            health_history: Vec::new(),
            last_health_check: Some(chrono::Utc::now()),
            monitoring_enabled: true,
        };
        
        // Add to installed plugins
        self.plugins.push(installed_plugin);
        self.apply_filter_and_sort();
        self.update_selection();
        
        self.set_plugin_installation_status(plugin_name, PluginInstallationStatus::Completed);
        tracing::info!("Successfully installed plugin '{}' from market", plugin_name);
        Ok(())
    }
    
    /// Verify plugin package integrity
    pub async fn verify_plugin_package(&self, plugin_name: &str, package_url: &str) -> Result<bool, String> {
        tracing::info!("Verifying plugin package: {} from {}", plugin_name, package_url);
        
        // Simulate package verification
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // In a real implementation, this would:
        // 1. Download package metadata
        // 2. Verify digital signatures
        // 3. Check package integrity (checksums)
        // 4. Scan for malware
        // 5. Validate plugin manifest
        
        // Simulate verification result
        let is_valid = !plugin_name.contains("malicious") && !package_url.contains("suspicious");
        
        if is_valid {
            tracing::info!("Plugin package verification successful: {}", plugin_name);
        } else {
            tracing::warn!("Plugin package verification failed: {}", plugin_name);
        }
        
        Ok(is_valid)
    }
    
    /// Get plugin installation progress
    pub fn get_installation_progress(&self, plugin_name: &str) -> Option<f32> {
        // In a real implementation, this would track actual installation progress
        // For now, return a mock progress value
        if plugin_name.contains("installing") {
            Some(0.65) // 65% complete
        } else {
            None
        }
    }
    
    /// Get available plugin categories from market
    pub fn get_market_categories(&self) -> Vec<String> {
        let mut categories = std::collections::HashSet::new();
        
        for plugin in &self.market_plugins {
            if let Some(category) = plugin.metadata.get("category").and_then(|v| v.as_str()) {
                categories.insert(category.to_string());
            }
        }
        
        let mut result: Vec<String> = categories.into_iter().collect();
        result.sort();
        result
    }
    
    /// Get plugin ratings and reviews (mock implementation)
    pub fn get_plugin_rating(&self, plugin_name: &str) -> Option<(f32, u32)> {
        self.market_plugins
            .iter()
            .find(|p| p.name == plugin_name)
            .and_then(|plugin| {
                let rating = plugin.metadata.get("rating")?.as_str()?.parse::<f32>().ok()?;
                let downloads = plugin.metadata.get("downloads")?.as_str()?.parse::<u32>().ok()?;
                Some((rating, downloads))
            })
    }
    
    /// Perform status monitoring for all plugins
    async fn perform_status_monitoring(&mut self) {
        if !self.monitoring_enabled {
            return;
        }
        
        // Collect plugin names and current status to avoid borrow checker issues
        let plugin_updates: Vec<(String, PluginStatus, Option<String>, ResourceUsage)> = self.plugins
            .iter()
            .filter(|plugin| plugin.monitoring_enabled)
            .map(|plugin| {
                let plugin_name = plugin.info.name.clone();
                let current_status = plugin.status;
                
                // Simulate occasional status changes
                let (new_status, reason) = if current_status == PluginStatus::Running && rand::random::<f64>() < 0.01 {
                    // Rarely simulate an error
                    (PluginStatus::Error, Some("Simulated error".to_string()))
                } else if current_status == PluginStatus::Error && rand::random::<f64>() < 0.1 {
                    // More frequently recover from errors
                    (PluginStatus::Running, Some("Recovered from error".to_string()))
                } else {
                    (current_status, None)
                };
                
                // Update resource usage with some variation
                let mut new_usage = plugin.resource_usage.clone();
                new_usage.cpu_percent = (new_usage.cpu_percent + (rand::random::<f64>() - 0.5) * 10.0).max(0.0).min(100.0);
                new_usage.memory_mb = (new_usage.memory_mb + (rand::random::<f64>() - 0.5) * 50.0).max(0.0);
                new_usage.last_activity = Some(Utc::now());
                new_usage.uptime_seconds += self.status_update_interval.as_secs();
                
                (plugin_name, new_status, reason, new_usage)
            })
            .collect();
        
        // Apply the updates
        for (plugin_name, new_status, reason, new_usage) in plugin_updates {
            if let Some(reason) = reason {
                self.update_plugin_status(&plugin_name, new_status, Some(reason));
            }
            self.update_plugin_resource_usage(&plugin_name, new_usage);
        }
        
        self.last_status_update = Some(Instant::now());
    }
    
    /// Perform health checks for all plugins
    async fn perform_health_checks(&mut self) {
        if !self.health_check_enabled {
            return;
        }
        
        let plugin_names: Vec<String> = self.plugins
            .iter()
            .filter(|p| p.monitoring_enabled)
            .map(|p| p.info.name.clone())
            .collect();
        
        for plugin_name in plugin_names {
            self.update_plugin_health(&plugin_name);
        }
        
        self.last_health_check = Some(Instant::now());
    }
}

#[async_trait]
impl Widget for PluginManagerWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }
    
    fn title(&self) -> &str {
        "插件管理器"
    }
    
    fn description(&self) -> Option<&str> {
        Some("管理系统插件，查看状态，安装和卸载插件")
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
        UpdateFrequency::Interval(self.refresh_interval)
    }
    
    async fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) -> std::result::Result<(), WidgetError> {
        // Clear expired errors
        self.clear_expired_error();
        
        // Main layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with tabs
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(area);
        
        // Render header with view tabs
        self.render_header(frame, main_chunks[0], theme);
        
        // Render main content based on current view
        match self.current_view {
            ViewMode::List => self.render_list_view(frame, main_chunks[1], theme),
            ViewMode::Details => self.render_details_view(frame, main_chunks[1], theme),
            ViewMode::Market => self.render_market_view(frame, main_chunks[1], theme),
            ViewMode::Dependencies => self.render_dependencies_view(frame, main_chunks[1], theme),
        }
        
        // Render status bar
        self.render_status_bar(frame, main_chunks[2], theme);
        
        // Render search overlay if in search mode
        if self.search_mode {
            self.render_search_overlay(frame, area, theme);
        }
        
        // Render error overlay if there's an error
        if self.last_error.is_some() {
            self.render_error_overlay(frame, area, theme);
        }
        
        Ok(())
    }
    
    async fn handle_event(&mut self, event: Event) -> std::result::Result<Option<Action>, WidgetError> {
        if !self.context().can_handle_events() {
            return Ok(None);
        }
        
        match event {
            Event::Key(key) => self.handle_key_event(key).await,
            Event::Resize(_, _) => {
                // Handle resize if needed
                Ok(None)
            }
            _ => Ok(None),
        }
    }
    
    async fn update(&mut self) -> std::result::Result<(), WidgetError> {
        self.context_mut().mark_updated();
        
        // Check if we need to refresh plugin data
        if self.needs_refresh() {
            // In a real implementation, this would trigger a data refresh
            // For now, we just update the timestamp
            self.last_refresh = Some(Instant::now());
        }
        
        // Perform status monitoring if enabled
        if self.needs_status_update() {
            self.perform_status_monitoring().await;
        }
        
        // Perform health checks if enabled
        if self.needs_health_check() {
            self.perform_health_checks().await;
        }
        
        Ok(())
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        let mut help = vec![
            ("↑/↓", "导航"),
            ("Enter", "查看详情"),
            ("Tab", "切换视图"),
            ("/", "搜索"),
            ("r", "刷新"),
            ("c", "清除过滤器"),
            ("m", "切换监控"),
            ("h", "切换健康检查"),
        ];
        
        match self.current_view {
            ViewMode::List => {
                help.extend_from_slice(&[
                    ("d", "切换详情面板"),
                    ("s", "排序选项"),
                    ("f", "过滤选项"),
                    ("Space", "启用/禁用监控"),
                ]);
            }
            ViewMode::Details => {
                help.extend_from_slice(&[
                    ("PgUp/PgDn", "滚动详情"),
                    ("e", "启用/禁用"),
                    ("u", "卸载"),
                    ("R", "重新加载"),
                    ("i", "安装/重新安装"),
                    ("U", "更新插件"),
                    ("H", "查看健康历史"),
                    ("S", "查看状态历史"),
                    ("C", "配置插件"),
                    ("P", "查看权限"),
                ]);
            }
            ViewMode::Market => {
                help.extend_from_slice(&[
                    ("i", "安装插件"),
                    ("v", "查看详情"),
                    ("F", "按类别过滤"),
                    ("T", "按类型过滤"),
                    ("R", "查看评分"),
                    ("D", "查看依赖"),
                ]);
            }
            ViewMode::Dependencies => {
                help.extend_from_slice(&[
                    ("g", "显示依赖图"),
                    ("a", "分析依赖"),
                    ("C", "检查冲突"),
                    ("B", "批量操作"),
                ]);
            }
        }
        
        help
    }
}

impl PluginManagerWidget {
    /// Handle keyboard events
    async fn handle_key_event(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        // Handle search mode input
        if self.search_mode {
            return self.handle_search_input(key).await;
        }
        
        // Handle global keys
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(Some(Action::Quit));
            }
            KeyCode::Esc => {
                if self.show_details_panel {
                    self.toggle_details_panel();
                } else {
                    return Ok(Some(Action::Back));
                }
            }
            KeyCode::Char('/') => {
                self.toggle_search_mode();
            }
            KeyCode::Char('r') => {
                return Ok(Some(Action::RefreshPlugins));
            }
            KeyCode::Char('c') => {
                self.clear_filters();
            }
            KeyCode::Tab => {
                self.cycle_view_mode();
            }
            KeyCode::Char('m') => {
                self.toggle_monitoring();
            }
            KeyCode::Char('h') => {
                self.toggle_health_checking();
            }
            _ => {}
        }
        
        // Handle view-specific keys
        match self.current_view {
            ViewMode::List => self.handle_list_view_keys(key).await,
            ViewMode::Details => self.handle_details_view_keys(key).await,
            ViewMode::Market => self.handle_market_view_keys(key).await,
            ViewMode::Dependencies => self.handle_dependencies_view_keys(key).await,
        }
    }
    
    /// Handle search input
    async fn handle_search_input(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        match key.code {
            KeyCode::Enter => {
                self.apply_search();
            }
            KeyCode::Esc => {
                self.toggle_search_mode();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        Ok(None)
    }
    
    /// Handle list view keys
    async fn handle_list_view_keys(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        match key.code {
            KeyCode::Up => {
                self.move_selection_up();
            }
            KeyCode::Down => {
                self.move_selection_down();
            }
            KeyCode::Enter => {
                if let Some(plugin) = self.selected_plugin() {
                    return Ok(Some(Action::ShowPluginDetails(plugin.info.name.clone())));
                }
            }
            KeyCode::Char('d') => {
                self.toggle_details_panel();
            }
            KeyCode::Char('s') => {
                self.cycle_sort_order();
            }
            KeyCode::Char('f') => {
                return Ok(Some(Action::ShowPluginFilters));
            }
            KeyCode::Char(' ') => {
                // Toggle monitoring for selected plugin
                if let Some(plugin) = self.selected_plugin() {
                    let plugin_name = plugin.info.name.clone();
                    let new_state = !plugin.monitoring_enabled;
                    self.set_plugin_monitoring(&plugin_name, new_state);
                }
            }
            _ => {}
        }
        Ok(None)
    }
    
    /// Handle details view keys
    async fn handle_details_view_keys(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        match key.code {
            KeyCode::Up => {
                self.move_selection_up();
            }
            KeyCode::Down => {
                self.move_selection_down();
            }
            KeyCode::PageUp => {
                if self.details_scroll > 0 {
                    self.details_scroll = self.details_scroll.saturating_sub(5);
                }
            }
            KeyCode::PageDown => {
                self.details_scroll += 5;
            }
            KeyCode::Char('e') => {
                if let Some(plugin) = self.selected_plugin() {
                    return Ok(Some(Action::TogglePlugin(plugin.info.name.clone())));
                }
            }
            KeyCode::Char('u') => {
                if let Some(plugin) = self.selected_plugin() {
                    return Ok(Some(Action::UninstallPlugin(plugin.info.name.clone())));
                }
            }
            KeyCode::Char('R') => {
                if let Some(plugin) = self.selected_plugin() {
                    return Ok(Some(Action::ReloadPlugin(plugin.info.name.clone())));
                }
            }
            KeyCode::Char('i') => {
                // Install plugin (for market plugins or reinstall)
                if let Some(plugin) = self.selected_plugin() {
                    return Ok(Some(Action::InstallPlugin(plugin.info.name.clone())));
                }
            }
            KeyCode::Char('U') => {
                // Update plugin
                if let Some(plugin) = self.selected_plugin() {
                    return Ok(Some(Action::Custom("UpdatePlugin".to_string(), 
                        serde_json::json!({"plugin_name": plugin.info.name}))));
                }
            }
            KeyCode::Char('H') => {
                // Show health history (could be implemented as a popup or separate view)
                if let Some(plugin) = self.selected_plugin() {
                    let history = self.get_plugin_health_history(&plugin.info.name);
                    tracing::info!("Plugin '{}' has {} health check records", plugin.info.name, history.len());
                    return Ok(Some(Action::Custom("ShowHealthHistory".to_string(),
                        serde_json::json!({"plugin_name": plugin.info.name, "history_count": history.len()}))));
                }
            }
            KeyCode::Char('S') => {
                // Show status history
                if let Some(plugin) = self.selected_plugin() {
                    let history = self.get_plugin_status_history(&plugin.info.name);
                    tracing::info!("Plugin '{}' has {} status change records", plugin.info.name, history.len());
                    return Ok(Some(Action::Custom("ShowStatusHistory".to_string(),
                        serde_json::json!({"plugin_name": plugin.info.name, "history_count": history.len()}))));
                }
            }
            KeyCode::Char('C') => {
                // Configure plugin
                if let Some(plugin) = self.selected_plugin() {
                    return Ok(Some(Action::Custom("ConfigurePlugin".to_string(),
                        serde_json::json!({"plugin_name": plugin.info.name}))));
                }
            }
            KeyCode::Char('P') => {
                // Show plugin permissions
                if let Some(plugin) = self.selected_plugin() {
                    return Ok(Some(Action::Custom("ShowPluginPermissions".to_string(),
                        serde_json::json!({"plugin_name": plugin.info.name}))));
                }
            }
            _ => {}
        }
        Ok(None)
    }
    
    /// Handle market view keys
    async fn handle_market_view_keys(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        match key.code {
            KeyCode::Up => {
                if self.market_selected > 0 {
                    self.market_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.market_selected + 1 < self.market_plugins.len() {
                    self.market_selected += 1;
                }
            }
            KeyCode::Char('i') => {
                if let Some(plugin) = self.market_plugins.get(self.market_selected) {
                    return Ok(Some(Action::InstallPlugin(plugin.name.clone())));
                }
            }
            KeyCode::Char('v') => {
                if let Some(plugin) = self.market_plugins.get(self.market_selected) {
                    return Ok(Some(Action::ShowPluginDetails(plugin.name.clone())));
                }
            }
            KeyCode::Char('F') => {
                // Filter by category
                return Ok(Some(Action::Custom("FilterMarketByCategory".to_string(), 
                    serde_json::Value::Null)));
            }
            KeyCode::Char('T') => {
                // Filter by type
                return Ok(Some(Action::Custom("FilterMarketByType".to_string(), 
                    serde_json::Value::Null)));
            }
            KeyCode::Char('R') => {
                // Show ratings and reviews
                if let Some(plugin) = self.market_plugins.get(self.market_selected) {
                    if let Some((rating, downloads)) = self.get_plugin_rating(&plugin.name) {
                        return Ok(Some(Action::Custom("ShowPluginRating".to_string(),
                            serde_json::json!({
                                "plugin_name": plugin.name,
                                "rating": rating,
                                "downloads": downloads
                            }))));
                    }
                }
            }
            KeyCode::Char('D') => {
                // Show dependencies for market plugin
                if let Some(plugin) = self.market_plugins.get(self.market_selected) {
                    return Ok(Some(Action::Custom("ShowMarketPluginDependencies".to_string(),
                        serde_json::json!({"plugin_name": plugin.name}))));
                }
            }
            KeyCode::Char('L') => {
                // Load/refresh market data
                return Ok(Some(Action::Custom("LoadMarketPlugins".to_string(), 
                    serde_json::Value::Null)));
            }
            _ => {}
        }
        Ok(None)
    }
    
    /// Handle dependencies view keys
    async fn handle_dependencies_view_keys(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        match key.code {
            KeyCode::Up => {
                self.move_selection_up();
            }
            KeyCode::Down => {
                self.move_selection_down();
            }
            KeyCode::Char('g') => {
                return Ok(Some(Action::ShowDependencyGraph));
            }
            KeyCode::Char('a') => {
                // Analyze dependencies
                let analysis = self.analyze_dependencies();
                tracing::info!("Dependency analysis: {} conflicts, {} warnings, can_proceed: {}", 
                    analysis.conflicts.len(), analysis.warnings.len(), analysis.can_proceed);
                return Ok(Some(Action::Custom("AnalyzeDependencies".to_string(),
                    serde_json::json!({
                        "conflicts": analysis.conflicts.len(),
                        "warnings": analysis.warnings.len(),
                        "can_proceed": analysis.can_proceed
                    }))));
            }
            KeyCode::Char('C') => {
                // Check for conflicts
                let analysis = self.analyze_dependencies();
                if !analysis.conflicts.is_empty() {
                    return Ok(Some(Action::Custom("ShowDependencyConflicts".to_string(),
                        serde_json::json!({
                            "conflicts": analysis.conflicts.len()
                        }))));
                }
            }
            KeyCode::Char('B') => {
                // Batch operations
                return Ok(Some(Action::Custom("ShowBatchOperations".to_string(), 
                    serde_json::Value::Null)));
            }
            KeyCode::Char('R') => {
                // Resolve dependencies
                if let Some(plugin) = self.selected_plugin() {
                    let resolution = self.resolve_batch_dependencies(&[plugin.info.name.clone()], "install");
                    return Ok(Some(Action::Custom("ResolveDependencies".to_string(),
                        serde_json::json!({
                            "plugin_name": plugin.info.name,
                            "can_proceed": resolution.can_proceed,
                            "install_order": resolution.install_order
                        }))));
                }
            }
            KeyCode::Char('I') => {
                // Show installation order
                let analysis = self.analyze_dependencies();
                return Ok(Some(Action::Custom("ShowInstallationOrder".to_string(),
                    serde_json::json!({
                        "install_order": analysis.install_order
                    }))));
            }
            _ => {}
        }
        Ok(None)
    }
    
    /// Cycle through view modes
    fn cycle_view_mode(&mut self) {
        self.current_view = match self.current_view {
            ViewMode::List => ViewMode::Details,
            ViewMode::Details => ViewMode::Market,
            ViewMode::Market => ViewMode::Dependencies,
            ViewMode::Dependencies => ViewMode::List,
        };
    }
    
    /// Cycle through sort orders
    fn cycle_sort_order(&mut self) {
        self.sort_order = match self.sort_order {
            PluginSortOrder::NameAsc => PluginSortOrder::NameDesc,
            PluginSortOrder::NameDesc => PluginSortOrder::StatusAsc,
            PluginSortOrder::StatusAsc => PluginSortOrder::StatusDesc,
            PluginSortOrder::StatusDesc => PluginSortOrder::TypeAsc,
            PluginSortOrder::TypeAsc => PluginSortOrder::TypeDesc,
            PluginSortOrder::TypeDesc => PluginSortOrder::LastUpdatedAsc,
            PluginSortOrder::LastUpdatedAsc => PluginSortOrder::LastUpdatedDesc,
            PluginSortOrder::LastUpdatedDesc => PluginSortOrder::NameAsc,
        };
        
        self.apply_filter_and_sort();
        self.update_selection();
    }
}

impl Default for PluginManagerWidget {
    fn default() -> Self {
        Self::new()
    }
}

// Implement PartialEq and Eq for PluginStatus to enable sorting
// Note: These implementations are removed as PluginStatus already derives these traits

// Rendering methods implementation
impl PluginManagerWidget {
    /// Render the header with view tabs
    fn render_header(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let tab_titles = vec!["列表", "详情", "市场", "依赖"];
        let selected_tab = match self.current_view {
            ViewMode::List => 0,
            ViewMode::Details => 1,
            ViewMode::Market => 2,
            ViewMode::Dependencies => 3,
        };
        
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title("插件管理器"))
            .style(theme.styles.info)
            .highlight_style(theme.styles.list_item_selected)
            .select(selected_tab);
        
        frame.render_widget(tabs, area);
    }
    
    /// Render the list view
    fn render_list_view(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = if self.show_details_panel {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area)
                .to_vec()
        } else {
            vec![area]
        };
        
        // Render plugin list
        self.render_plugin_list(frame, chunks[0], theme);
        
        // Render details panel if enabled
        if self.show_details_panel && chunks.len() > 1 {
            self.render_plugin_details_panel(frame, chunks[1], theme);
        }
    }
    
    /// Render the plugin list
    fn render_plugin_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let items: Vec<ListItem> = self.filtered_plugins
            .iter()
            .map(|&index| {
                let plugin = &self.plugins[index];
                self.create_plugin_list_item(plugin, theme)
            })
            .collect();
        
        let title = format!(
            "插件列表 ({}/{}) - 排序: {}",
            self.filtered_plugins.len(),
            self.plugins.len(),
            self.sort_order_display()
        );
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(theme.styles.list_item_selected)
            .highlight_symbol("► ");
        
        let mut list_state = self.list_state.clone();
        frame.render_stateful_widget(list, area, &mut list_state);
        self.list_state = list_state;
        
        // Render scrollbar if needed
        if self.filtered_plugins.len() > area.height as usize - 2 {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            
            let scrollbar_area = Rect {
                x: area.right() - 1,
                y: area.y + 1,
                width: 1,
                height: area.height - 2,
            };
            
            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut self.scroll_state);
        }
    }
    
    /// Create a list item for a plugin
    fn create_plugin_list_item<'a>(&self, plugin: &'a PluginDisplayInfo, _theme: &Theme) -> ListItem<'a> {
        let status_symbol = match plugin.status {
            PluginStatus::Uninitialized => "○",
            PluginStatus::Initializing => "◐",
            PluginStatus::Ready => "●",
            PluginStatus::Running => "▶",
            PluginStatus::Error => "✗",
            PluginStatus::ShuttingDown => "◑",
            PluginStatus::Shutdown => "◯",
        };
        
        let status_style = match plugin.status {
            PluginStatus::Ready => Style::default().fg(Color::Green),
            PluginStatus::Running => Style::default().fg(Color::Cyan),
            PluginStatus::Error => Style::default().fg(Color::Red),
            PluginStatus::Initializing | PluginStatus::ShuttingDown => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::Gray),
        };
        
        let health_symbol = match plugin.health_status {
            PluginHealthStatus::Healthy => "✓",
            PluginHealthStatus::Warning(_) => "⚠",
            PluginHealthStatus::Critical(_) => "⚠",
            PluginHealthStatus::Unknown => "?",
        };
        
        let health_style = match plugin.health_status {
            PluginHealthStatus::Healthy => Style::default().fg(Color::Green),
            PluginHealthStatus::Warning(_) => Style::default().fg(Color::Yellow),
            PluginHealthStatus::Critical(_) => Style::default().fg(Color::Red),
            PluginHealthStatus::Unknown => Style::default().fg(Color::Gray),
        };
        
        let type_str = match plugin.info.plugin_type {
            PluginType::Native => "Native",
            PluginType::Python => "Python",
            PluginType::NodeJs => "Node.js",
            PluginType::Go => "Go",
            PluginType::Docker => "Docker",
            PluginType::Wasm => "WASM",
        };
        
        let enabled_str = plugin.config
            .as_ref()
            .map(|c| if c.enabled { "启用" } else { "禁用" })
            .unwrap_or("未知");
        
        let monitoring_str = if plugin.monitoring_enabled { "监控" } else { "无监控" };
        
        let content = vec![
            Line::from(vec![
                Span::styled(status_symbol, status_style),
                Span::raw(" "),
                Span::styled(health_symbol, health_style),
                Span::raw(" "),
                Span::styled(&plugin.info.name, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" v"),
                Span::styled(&plugin.info.version, Style::default().fg(Color::Gray)),
                Span::raw(" ["),
                Span::styled(type_str, Style::default().fg(Color::Cyan)),
                Span::raw("]"),
            ]),
            Line::from(vec![
                Span::raw("  状态: "),
                Span::styled(format!("{:?}", plugin.status), status_style),
                Span::raw(" | "),
                Span::raw(enabled_str),
                Span::raw(" | "),
                Span::styled(monitoring_str, if plugin.monitoring_enabled { 
                    Style::default().fg(Color::Green) 
                } else { 
                    Style::default().fg(Color::Gray) 
                }),
                Span::raw(" | 工具: "),
                Span::styled(plugin.tools_count.to_string(), Style::default().fg(Color::Yellow)),
                Span::raw(" | 内存: "),
                Span::styled(format!("{:.1}MB", plugin.resource_usage.memory_mb), Style::default().fg(Color::Magenta)),
            ]),
        ];
        
        ListItem::new(content)
    }
    
    /// Render the plugin details panel
    fn render_plugin_details_panel(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(plugin) = self.selected_plugin() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),  // Basic info
                    Constraint::Length(6),  // Resource usage
                    Constraint::Min(0),     // Description and metadata
                ])
                .split(area);
            
            // Basic info
            self.render_plugin_basic_info(frame, chunks[0], plugin, theme);
            
            // Resource usage
            self.render_plugin_resource_usage(frame, chunks[1], plugin, theme);
            
            // Description and metadata
            self.render_plugin_description(frame, chunks[2], plugin, theme);
        } else {
            let paragraph = Paragraph::new("未选择插件")
                .block(Block::default().borders(Borders::ALL).title("插件详情"))
                .style(theme.styles.info);
            frame.render_widget(paragraph, area);
        }
    }
    
    /// Render plugin basic info
    fn render_plugin_basic_info(&self, frame: &mut Frame, area: Rect, plugin: &PluginDisplayInfo, theme: &Theme) {
        let info_text = format!(
            "名称: {}\n版本: {}\n类型: {:?}\n状态: {:?}\n作者: {}\n工具数量: {}\n依赖: {}",
            plugin.info.name,
            plugin.info.version,
            plugin.info.plugin_type,
            plugin.status,
            plugin.info.author.as_deref().unwrap_or("未知"),
            plugin.tools_count,
            plugin.dependencies.join(", ")
        );
        
        let paragraph = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("基本信息"))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// Render plugin resource usage
    fn render_plugin_resource_usage(&self, frame: &mut Frame, area: Rect, plugin: &PluginDisplayInfo, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);
        
        let block = Block::default().borders(Borders::ALL).title("资源使用");
        frame.render_widget(block, area);
        
        // Memory usage gauge
        let memory_ratio = (plugin.resource_usage.memory_mb / 1024.0).min(1.0);
        let memory_gauge = Gauge::default()
            .block(Block::default().title("内存"))
            .gauge_style(Style::default().fg(Color::Magenta))
            .ratio(memory_ratio)
            .label(format!("{:.1}MB", plugin.resource_usage.memory_mb));
        
        frame.render_widget(memory_gauge, chunks[0]);
        
        // CPU usage gauge
        let cpu_ratio = (plugin.resource_usage.cpu_percent / 100.0).min(1.0);
        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(cpu_ratio)
            .label(format!("{:.1}%", plugin.resource_usage.cpu_percent));
        
        frame.render_widget(cpu_gauge, chunks[1]);
        
        // Network connections
        let network_text = format!("网络连接: {}", plugin.resource_usage.network_connections);
        let network_paragraph = Paragraph::new(network_text)
            .style(theme.styles.info);
        frame.render_widget(network_paragraph, chunks[2]);
        
        // File handles
        let files_text = format!("文件句柄: {}", plugin.resource_usage.file_handles);
        let files_paragraph = Paragraph::new(files_text)
            .style(theme.styles.info);
        frame.render_widget(files_paragraph, chunks[3]);
    }
    
    /// Render plugin description
    fn render_plugin_description(&self, frame: &mut Frame, area: Rect, plugin: &PluginDisplayInfo, theme: &Theme) {
        let description = plugin.info.description
            .as_deref()
            .unwrap_or("无描述信息");
        
        let paragraph = Paragraph::new(description)
            .block(Block::default().borders(Borders::ALL).title("描述"))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// Render the details view
    fn render_details_view(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(plugin) = self.selected_plugin() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(10), // Header info
                    Constraint::Min(0),     // Detailed content
                ])
                .split(area);
            
            // Render header
            self.render_plugin_basic_info(frame, chunks[0], plugin, theme);
            
            // Render detailed content with scrolling
            self.render_detailed_content(frame, chunks[1], plugin, theme);
        } else {
            let paragraph = Paragraph::new("未选择插件\n\n使用方向键选择插件查看详细信息")
                .block(Block::default().borders(Borders::ALL).title("插件详情"))
                .style(theme.styles.info);
            frame.render_widget(paragraph, area);
        }
    }
    
    /// Render detailed content with scrolling
    fn render_detailed_content(&self, frame: &mut Frame, area: Rect, plugin: &PluginDisplayInfo, theme: &Theme) {
        let mut content_lines = Vec::new();
        
        // Plugin metadata section
        content_lines.push("插件元数据:".to_string());
        content_lines.push(format!("  名称: {}", plugin.info.name));
        content_lines.push(format!("  版本: {}", plugin.info.version));
        content_lines.push(format!("  类型: {:?}", plugin.info.plugin_type));
        
        if let Some(author) = &plugin.info.author {
            content_lines.push(format!("  作者: {}", author));
        }
        
        if let Some(description) = &plugin.info.description {
            content_lines.push(format!("  描述: {}", description));
        }
        
        // Check metadata for additional fields
        if let Some(homepage) = plugin.info.metadata.get("homepage") {
            content_lines.push(format!("  主页: {}", homepage));
        }
        
        if let Some(repository) = plugin.info.metadata.get("repository") {
            content_lines.push(format!("  仓库: {}", repository));
        }
        
        if let Some(license) = plugin.info.metadata.get("license") {
            content_lines.push(format!("  许可证: {}", license));
        }
        
        content_lines.push(String::new());
        
        // Plugin tools section
        content_lines.push("提供的工具:".to_string());
        if plugin.tools_count == 0 {
            content_lines.push("  无工具".to_string());
        } else {
            content_lines.push(format!("  工具数量: {}", plugin.tools_count));
            // In a real implementation, we would list the actual tools
            content_lines.push("  工具列表:".to_string());
            for i in 1..=plugin.tools_count.min(10) {
                content_lines.push(format!("    - 工具{} (示例)", i));
            }
            if plugin.tools_count > 10 {
                content_lines.push(format!("    ... 还有{}个工具", plugin.tools_count - 10));
            }
        }
        
        content_lines.push(String::new());
        
        // Configuration section
        content_lines.push("配置信息:".to_string());
        if let Some(config) = &plugin.config {
            let enabled_status = if config.enabled { "启用" } else { "禁用" };
            content_lines.push(format!("  启用状态: {}", enabled_status));
            
            content_lines.push("  安全策略:".to_string());
            content_lines.push(format!("    网络访问: {}", if config.security_policy.allow_network_access { "允许" } else { "禁止" }));
            content_lines.push(format!("    文件系统访问: {}", if config.security_policy.allow_file_system_access { "允许" } else { "禁止" }));
            content_lines.push(format!("    沙箱模式: {}", if config.security_policy.sandbox_enabled { "启用" } else { "禁用" }));
            
            if !config.security_policy.allowed_paths.is_empty() {
                content_lines.push("    允许的路径:".to_string());
                for path in &config.security_policy.allowed_paths {
                    content_lines.push(format!("      - {}", path.display()));
                }
            }
            
            if !config.security_policy.environment_variables.is_empty() {
                content_lines.push("    环境变量:".to_string());
                for (key, value) in &config.security_policy.environment_variables {
                    content_lines.push(format!("      {}={}", key, value));
                }
            }
            
            content_lines.push("  资源限制:".to_string());
            if let Some(max_memory) = config.resource_limits.max_memory {
                content_lines.push(format!("    最大内存: {}MB", max_memory / 1024 / 1024));
            }
            if let Some(max_cpu_time) = config.resource_limits.max_cpu_time {
                content_lines.push(format!("    最大CPU时间: {:?}", max_cpu_time));
            }
            if let Some(max_execution_time) = config.resource_limits.max_execution_time {
                content_lines.push(format!("    最大执行时间: {:?}", max_execution_time));
            }
            if let Some(max_file_size) = config.resource_limits.max_file_size {
                content_lines.push(format!("    最大文件大小: {}MB", max_file_size / 1024 / 1024));
            }
            if let Some(max_connections) = config.resource_limits.max_network_connections {
                content_lines.push(format!("    最大网络连接: {}", max_connections));
            }
            
            // Plugin-specific configuration
            if !config.config.is_null() {
                content_lines.push("  插件配置:".to_string());
                if let Ok(config_str) = serde_json::to_string_pretty(&config.config) {
                    for line in config_str.lines().take(20) {
                        content_lines.push(format!("    {}", line));
                    }
                }
            }
            
            // Metadata
            if !config.metadata.is_empty() {
                content_lines.push("  元数据:".to_string());
                for (key, value) in &config.metadata {
                    content_lines.push(format!("    {}: {}", key, value));
                }
            }
        } else {
            content_lines.push("  无配置信息".to_string());
        }
        
        content_lines.push(String::new());
        
        // Dependencies section
        content_lines.push("依赖关系:".to_string());
        if plugin.dependencies.is_empty() {
            content_lines.push("  无依赖".to_string());
        } else {
            for dep in &plugin.dependencies {
                content_lines.push(format!("  - {}", dep));
            }
        }
        
        content_lines.push(String::new());
        
        // Health status section
        content_lines.push("健康状态:".to_string());
        match &plugin.health_status {
            PluginHealthStatus::Healthy => content_lines.push("  ✓ 健康".to_string()),
            PluginHealthStatus::Warning(msg) => content_lines.push(format!("  ⚠ 警告: {}", msg)),
            PluginHealthStatus::Critical(msg) => content_lines.push(format!("  ⚠ 严重: {}", msg)),
            PluginHealthStatus::Unknown => content_lines.push("  ? 未知".to_string()),
        }
        
        // Last health check
        if let Some(last_check) = plugin.last_health_check {
            let elapsed = Utc::now().signed_duration_since(last_check);
            content_lines.push(format!("  上次检查: {}分钟前", elapsed.num_minutes()));
        }
        
        // Health history summary
        if !plugin.health_history.is_empty() {
            let recent_checks = plugin.health_history.iter().rev().take(5);
            content_lines.push("  最近检查:".to_string());
            for check in recent_checks {
                let status_str = match check.status {
                    PluginHealthStatus::Healthy => "✓",
                    PluginHealthStatus::Warning(_) => "⚠",
                    PluginHealthStatus::Critical(_) => "✗",
                    PluginHealthStatus::Unknown => "?",
                };
                content_lines.push(format!("    {} {} (评分: {:.1})", 
                    check.timestamp.format("%H:%M:%S"), status_str, check.overall_score));
            }
        }
        
        content_lines.push(String::new());
        
        // Monitoring status section
        content_lines.push("监控状态:".to_string());
        content_lines.push(format!("  监控启用: {}", if plugin.monitoring_enabled { "是" } else { "否" }));
        content_lines.push(format!("  运行时间: {}秒", plugin.resource_usage.uptime_seconds));
        
        if let Some(last_activity) = plugin.resource_usage.last_activity {
            let elapsed = Utc::now().signed_duration_since(last_activity);
            content_lines.push(format!("  最后活动: {}分钟前", elapsed.num_minutes()));
        }
        
        // Resource usage details
        content_lines.push("  资源使用:".to_string());
        content_lines.push(format!("    CPU: {:.1}%", plugin.resource_usage.cpu_percent));
        content_lines.push(format!("    内存: {:.1}MB", plugin.resource_usage.memory_mb));
        content_lines.push(format!("    网络连接: {}", plugin.resource_usage.network_connections));
        content_lines.push(format!("    文件句柄: {}", plugin.resource_usage.file_handles));
        content_lines.push(format!("    磁盘读取: {:.1}MB", plugin.resource_usage.disk_io_read_mb));
        content_lines.push(format!("    磁盘写入: {:.1}MB", plugin.resource_usage.disk_io_write_mb));
        
        content_lines.push(String::new());
        
        // Status change history
        if !plugin.status_history.is_empty() {
            content_lines.push("状态变更历史:".to_string());
            let recent_changes = plugin.status_history.iter().rev().take(5);
            for change in recent_changes {
                let reason_str = change.reason.as_deref().unwrap_or("无原因");
                content_lines.push(format!("  {} {:?} → {:?} ({})", 
                    change.timestamp.format("%H:%M:%S"), 
                    change.from_status, 
                    change.to_status,
                    reason_str));
            }
            content_lines.push(String::new());
        }
        
        // Plugin documentation section
        content_lines.push("文档和帮助:".to_string());
        content_lines.push("  使用说明:".to_string());
        content_lines.push("    - 使用 'e' 键启用/禁用插件".to_string());
        content_lines.push("    - 使用 'R' 键重新加载插件".to_string());
        content_lines.push("    - 使用 'u' 键卸载插件".to_string());
        content_lines.push("    - 使用 'H' 键查看健康历史".to_string());
        content_lines.push("    - 使用 'S' 键查看状态历史".to_string());
        
        if let Some(description) = &plugin.info.description {
            content_lines.push("  插件说明:".to_string());
            for line in description.lines() {
                content_lines.push(format!("    {}", line));
            }
        }
        
        // Plugin-specific metadata
        if !plugin.info.metadata.is_empty() {
            content_lines.push("  扩展信息:".to_string());
            for (key, value) in &plugin.info.metadata {
                content_lines.push(format!("    {}: {}", key, value));
            }
        }
        
        content_lines.push(String::new());
        
        // Apply scrolling
        let visible_lines: Vec<&str> = content_lines
            .iter()
            .skip(self.details_scroll)
            .take(area.height as usize - 2)
            .map(|s| s.as_str())
            .collect();
        
        let content = visible_lines.join("\n");
        
        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("详细信息"))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    /// Render the market view
    fn render_market_view(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Search/filter bar
                Constraint::Min(0),     // Plugin list
                Constraint::Length(5),  // Plugin details preview
            ])
            .split(area);
        
        // Render search/filter bar
        let search_text = if self.filter.search_term.is_empty() {
            "搜索插件市场... (按 / 开始搜索)".to_string()
        } else {
            format!("搜索: {}", self.filter.search_term)
        };
        
        let search_paragraph = Paragraph::new(search_text)
            .block(Block::default().borders(Borders::ALL).title("插件市场搜索"))
            .style(theme.styles.info);
        frame.render_widget(search_paragraph, chunks[0]);
        
        // Render market plugin list
        if self.market_plugins.is_empty() {
            let loading_text = "正在加载插件市场...\n\n按 'r' 刷新市场数据";
            let paragraph = Paragraph::new(loading_text)
                .block(Block::default().borders(Borders::ALL).title("插件市场"))
                .style(theme.styles.info)
                .wrap(Wrap { trim: true });
            frame.render_widget(paragraph, chunks[1]);
        } else {
            let items: Vec<ListItem> = self.market_plugins
                .iter()
                .enumerate()
                .map(|(i, plugin)| {
                    let is_installed = self.plugins.iter().any(|p| p.info.name == plugin.name);
                    let status_symbol = if is_installed { "✓" } else { "○" };
                    let status_style = if is_installed { 
                        Style::default().fg(Color::Green) 
                    } else { 
                        Style::default().fg(Color::Gray) 
                    };
                    
                    let rating = plugin.metadata.get("rating")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A");
                    let downloads = plugin.metadata.get("downloads")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0");
                    let category = plugin.metadata.get("category")
                        .and_then(|v| v.as_str())
                        .unwrap_or("其他");
                    
                    let content = vec![
                        Line::from(vec![
                            Span::styled(status_symbol, status_style),
                            Span::raw(" "),
                            Span::styled(&plugin.name, Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(" v"),
                            Span::styled(&plugin.version, Style::default().fg(Color::Gray)),
                            Span::raw(" ["),
                            Span::styled(format!("{:?}", plugin.plugin_type), Style::default().fg(Color::Cyan)),
                            Span::raw("]"),
                            Span::raw(" ★"),
                            Span::styled(rating, Style::default().fg(Color::Yellow)),
                        ]),
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(category, Style::default().fg(Color::Magenta)),
                            Span::raw(" | 下载: "),
                            Span::styled(downloads, Style::default().fg(Color::Green)),
                            Span::raw(" | 作者: "),
                            Span::styled(plugin.author.as_deref().unwrap_or("未知"), Style::default().fg(Color::Blue)),
                        ]),
                    ];
                    
                    ListItem::new(content)
                })
                .collect();
            
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!("插件市场 ({} 个插件)", self.market_plugins.len())))
                .highlight_style(theme.styles.list_item_selected)
                .highlight_symbol("► ");
            
            let mut market_list_state = ListState::default();
            market_list_state.select(Some(self.market_selected));
            frame.render_stateful_widget(list, chunks[1], &mut market_list_state);
        }
        
        // Render selected plugin details preview
        if let Some(plugin) = self.market_plugins.get(self.market_selected) {
            let description = plugin.description.as_deref().unwrap_or("无描述");
            let homepage = plugin.metadata.get("homepage")
                .and_then(|v| v.as_str())
                .unwrap_or("无");
            let license = plugin.metadata.get("license")
                .and_then(|v| v.as_str())
                .unwrap_or("未知");
            
            let details_text = format!(
                "描述: {}\n主页: {}\n许可证: {}",
                description, homepage, license
            );
            
            let details_paragraph = Paragraph::new(details_text)
                .block(Block::default().borders(Borders::ALL).title("插件详情"))
                .style(theme.styles.info)
                .wrap(Wrap { trim: true });
            frame.render_widget(details_paragraph, chunks[2]);
        }
    }
    
    /// Render the dependencies view
    fn render_dependencies_view(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Dependency analysis summary
                Constraint::Min(0),     // Dependency details
            ])
            .split(area);
        
        // Render dependency analysis summary
        let analysis = self.analyze_dependencies();
        let summary_text = format!(
            "依赖分析结果:\n\n• 总插件数: {}\n• 依赖冲突: {}\n• 警告: {}\n• 可以继续操作: {}",
            self.plugins.len(),
            analysis.conflicts.len(),
            analysis.warnings.len(),
            if analysis.can_proceed { "是" } else { "否" }
        );
        
        let summary_style = if analysis.can_proceed {
            theme.styles.success
        } else {
            Style::default().fg(Color::Red)
        };
        
        let summary_paragraph = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("依赖关系分析"))
            .style(summary_style)
            .wrap(Wrap { trim: true });
        frame.render_widget(summary_paragraph, chunks[0]);
        
        // Render dependency details
        let mut details_lines = Vec::new();
        
        if !analysis.conflicts.is_empty() {
            details_lines.push("依赖冲突:".to_string());
            for conflict in &analysis.conflicts {
                details_lines.push(format!("  • {}: {}", 
                    conflict.plugin_name, conflict.description));
            }
            details_lines.push(String::new());
        }
        
        if !analysis.warnings.is_empty() {
            details_lines.push("警告:".to_string());
            for warning in &analysis.warnings {
                details_lines.push(format!("  • {}", warning));
            }
            details_lines.push(String::new());
        }
        
        if !analysis.install_order.is_empty() {
            details_lines.push("建议安装顺序:".to_string());
            for (i, plugin_name) in analysis.install_order.iter().enumerate() {
                details_lines.push(format!("  {}. {}", i + 1, plugin_name));
            }
            details_lines.push(String::new());
        }
        
        // Show individual plugin dependencies
        details_lines.push("插件依赖详情:".to_string());
        for plugin in &self.plugins {
            if plugin.dependencies.is_empty() {
                details_lines.push(format!("  • {}: 无依赖", plugin.info.name));
            } else {
                details_lines.push(format!("  • {}: {}", 
                    plugin.info.name, plugin.dependencies.join(", ")));
            }
        }
        
        let details_text = details_lines.join("\n");
        let details_paragraph = Paragraph::new(details_text)
            .block(Block::default().borders(Borders::ALL).title("依赖详情"))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        frame.render_widget(details_paragraph, chunks[1]);
    }
    
    /// Render the status bar
    fn render_status_bar(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut status_parts = Vec::new();
        
        // Current view
        let view_name = match self.current_view {
            ViewMode::List => "列表",
            ViewMode::Details => "详情",
            ViewMode::Market => "市场",
            ViewMode::Dependencies => "依赖",
        };
        status_parts.push(format!("视图: {}", view_name));
        
        // Plugin count
        status_parts.push(format!("插件: {}/{}", self.filtered_plugins.len(), self.plugins.len()));
        
        // Monitoring status
        let monitoring_status = if self.monitoring_enabled { "开启" } else { "关闭" };
        status_parts.push(format!("监控: {}", monitoring_status));
        
        // Health checking status
        let health_status = if self.health_check_enabled { "开启" } else { "关闭" };
        status_parts.push(format!("健康检查: {}", health_status));
        
        // Filter status
        if !self.filter.search_term.is_empty() {
            status_parts.push(format!("搜索: '{}'", self.filter.search_term));
        }
        
        // Last refresh
        if let Some(last_refresh) = self.last_refresh {
            let elapsed = last_refresh.elapsed();
            if elapsed < Duration::from_secs(60) {
                status_parts.push(format!("刷新: {}秒前", elapsed.as_secs()));
            } else {
                status_parts.push(format!("刷新: {}分钟前", elapsed.as_secs() / 60));
            }
        }
        
        // Error status
        if let Some(error) = &self.last_error {
            status_parts.push(format!("错误: {}", error));
        }
        
        let status_text = status_parts.join(" | ");
        
        let paragraph = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .style(theme.styles.info);
        
        frame.render_widget(paragraph, area);
    }
    
    /// Render search overlay
    fn render_search_overlay(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = self.centered_rect(60, 3, area);
        
        // Clear the area
        frame.render_widget(Clear, popup_area);
        
        let search_text = format!("搜索: {}", self.input_buffer);
        let paragraph = Paragraph::new(search_text)
            .block(Block::default().borders(Borders::ALL).title("搜索插件"))
            .style(theme.styles.input);
        
        frame.render_widget(paragraph, popup_area);
    }
    
    /// Render error overlay
    fn render_error_overlay(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(error) = &self.last_error {
            let popup_area = self.centered_rect(80, 5, area);
            
            // Clear the area
            frame.render_widget(Clear, popup_area);
            
            let error_text = format!("错误: {}\n\n按任意键继续...", error);
            let paragraph = Paragraph::new(error_text)
                .block(Block::default().borders(Borders::ALL).title("错误"))
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: true });
            
            frame.render_widget(paragraph, popup_area);
        }
    }
    
    /// Helper function to create a centered rectangle
    fn centered_rect(&self, percent_x: u16, height: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - height) / 2),
                Constraint::Length(height),
                Constraint::Percentage((100 - height) / 2),
            ])
            .split(r);
        
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
    
    /// Get display string for sort order
    fn sort_order_display(&self) -> &str {
        match self.sort_order {
            PluginSortOrder::NameAsc => "名称↑",
            PluginSortOrder::NameDesc => "名称↓",
            PluginSortOrder::StatusAsc => "状态↑",
            PluginSortOrder::StatusDesc => "状态↓",
            PluginSortOrder::TypeAsc => "类型↑",
            PluginSortOrder::TypeDesc => "类型↓",
            PluginSortOrder::LastUpdatedAsc => "更新时间↑",
            PluginSortOrder::LastUpdatedDesc => "更新时间↓",
        }
    }
}