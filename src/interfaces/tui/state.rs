//! TUI State Management
//!
//! This module provides centralized state management for TUI widgets,
//! enabling data sharing and synchronization between components.

use crate::core::{PluginInfo, ToolInfo};
use crate::error::Result;
use crate::interfaces::tui::action::LogLevel;
use crate::interfaces::tui::widgets::{
    log_viewer::LogEntry,
    workflow_list::{ExecutionStatus, WorkflowInfo, WorkflowStatus},
};
use crate::workflow::definition::WorkflowDefinition;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};

/// Simple execution information for state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub id: String,
    pub workflow_name: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: f64,
}

/// Centralized application state for widget data sharing
pub struct SharedAppState {
    /// Workflow data
    workflows: Arc<RwLock<Vec<WorkflowInfo>>>,
    /// Execution data
    executions: Arc<RwLock<Vec<ExecutionInfo>>>,
    /// Tool data
    tools: Arc<RwLock<Vec<ToolInfo>>>,
    /// Plugin data
    plugins: Arc<RwLock<Vec<PluginInfo>>>,
    /// System status data
    system_status: Arc<RwLock<SystemStatus>>,
    /// Log entries
    logs: Arc<RwLock<Vec<LogEntry>>>,
    /// Connection status
    connection_status: Arc<RwLock<ConnectionStatus>>,
    /// Last update timestamp
    last_update: Arc<RwLock<DateTime<Utc>>>,
    /// Event broadcaster for state changes
    event_broadcaster: broadcast::Sender<StateChangeEvent>,
    /// Performance metrics
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_usage: f64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub active_workflows: u32,
    pub system_health: SystemHealth,
    pub uptime: Duration,
    pub network_status: NetworkStatus,
    pub load_average: [f64; 3], // 1min, 5min, 15min
    pub process_count: u32,
    pub thread_count: u32,
}

/// System health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemHealth {
    Healthy,
    Warning,
    Critical,
}

/// Network connection status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkStatus {
    Connected,
    Disconnected,
    Limited,
}

/// Connection status for backend communication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub render_time_ms: f64,
    pub update_time_ms: f64,
    pub event_processing_time_ms: f64,
    pub memory_usage_mb: f64,
    pub fps: f64,
    pub widget_update_count: HashMap<String, u64>,
    pub last_measurement: DateTime<Utc>,
}

/// State change events for widget synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChangeEvent {
    /// Workflows data updated
    WorkflowsUpdated,
    /// Specific workflow status changed
    WorkflowStatusChanged {
        name: String,
        status: WorkflowStatus,
    },
    /// New workflow added
    WorkflowAdded { workflow: WorkflowInfo },
    /// Workflow removed
    WorkflowRemoved { name: String },

    /// Executions data updated
    ExecutionsUpdated,
    /// Specific execution status changed
    ExecutionStatusChanged { id: String, status: ExecutionStatus },
    /// New execution started
    ExecutionStarted { execution: ExecutionInfo },
    /// Execution completed
    ExecutionCompleted { id: String },

    /// Tools data updated
    ToolsUpdated,
    /// New tool registered
    ToolRegistered { tool: ToolInfo },
    /// Tool unregistered
    ToolUnregistered { name: String },

    /// Plugins data updated
    PluginsUpdated,
    /// Plugin status changed
    PluginStatusChanged { name: String, enabled: bool },
    /// New plugin installed
    PluginInstalled { plugin: PluginInfo },
    /// Plugin uninstalled
    PluginUninstalled { name: String },

    /// System status updated
    SystemStatusUpdated,
    /// System health changed
    SystemHealthChanged { health: SystemHealth },

    /// New log entry added
    LogEntryAdded { entry: LogEntry },
    /// Logs cleared
    LogsCleared,

    /// Connection status changed
    ConnectionStatusChanged { status: ConnectionStatus },

    /// Performance metrics updated
    MetricsUpdated,

    /// Generic data refresh
    DataRefreshed,
}

/// Trait for widgets that can subscribe to state changes
#[async_trait]
pub trait StateSubscriber: Send + Sync {
    /// Handle a state change event
    async fn handle_state_change(&mut self, event: &StateChangeEvent) -> Result<()>;

    /// Get the subscriber's ID for tracking
    fn subscriber_id(&self) -> String;

    /// Get the events this subscriber is interested in
    fn interested_events(&self) -> Vec<StateChangeEventType>;
}

/// Types of state change events for filtering subscriptions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StateChangeEventType {
    Workflows,
    Executions,
    Tools,
    Plugins,
    SystemStatus,
    Logs,
    Connection,
    Metrics,
    All,
}

impl SharedAppState {
    /// Create a new shared application state
    pub fn new() -> Self {
        let (event_broadcaster, _) = broadcast::channel(1000);

        Self {
            workflows: Arc::new(RwLock::new(Vec::new())),
            executions: Arc::new(RwLock::new(Vec::new())),
            tools: Arc::new(RwLock::new(Vec::new())),
            plugins: Arc::new(RwLock::new(Vec::new())),
            system_status: Arc::new(RwLock::new(SystemStatus::default())),
            logs: Arc::new(RwLock::new(Vec::new())),
            connection_status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            last_update: Arc::new(RwLock::new(Utc::now())),
            event_broadcaster,
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    /// Subscribe to state change events
    pub fn subscribe(&self) -> broadcast::Receiver<StateChangeEvent> {
        self.event_broadcaster.subscribe()
    }

    /// Broadcast a state change event
    async fn broadcast_event(&self, event: StateChangeEvent) {
        if let Err(e) = self.event_broadcaster.send(event.clone()) {
            tracing::warn!("Failed to broadcast state change event: {}", e);
        }
        tracing::debug!("Broadcasted state change event: {:?}", event);
    }

    // Workflow management methods

    /// Get all workflows
    pub async fn get_workflows(&self) -> Vec<WorkflowInfo> {
        self.workflows.read().await.clone()
    }

    /// Set workflows data
    pub async fn set_workflows(&self, workflows: Vec<WorkflowInfo>) -> Result<()> {
        *self.workflows.write().await = workflows;
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::WorkflowsUpdated)
            .await;
        Ok(())
    }

    /// Add a new workflow
    pub async fn add_workflow(&self, workflow: WorkflowInfo) -> Result<()> {
        self.workflows.write().await.push(workflow.clone());
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::WorkflowAdded { workflow })
            .await;
        Ok(())
    }

    /// Update workflow status
    pub async fn update_workflow_status(&self, name: &str, status: WorkflowStatus) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.iter_mut().find(|w| w.name == name) {
            workflow.status = status.clone();
            *self.last_update.write().await = Utc::now();
            drop(workflows); // Release lock before broadcasting
            self.broadcast_event(StateChangeEvent::WorkflowStatusChanged {
                name: name.to_string(),
                status,
            })
            .await;
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ValidationError(format!(
                "Workflow not found: {}",
                name
            )))
        }
    }

    /// Remove a workflow
    pub async fn remove_workflow(&self, name: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        let initial_len = workflows.len();
        workflows.retain(|w| w.name != name);

        if workflows.len() < initial_len {
            *self.last_update.write().await = Utc::now();
            drop(workflows); // Release lock before broadcasting
            self.broadcast_event(StateChangeEvent::WorkflowRemoved {
                name: name.to_string(),
            })
            .await;
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ValidationError(format!(
                "Workflow not found: {}",
                name
            )))
        }
    }

    // Execution management methods

    /// Get all executions
    pub async fn get_executions(&self) -> Vec<ExecutionInfo> {
        self.executions.read().await.clone()
    }

    /// Set executions data
    pub async fn set_executions(&self, executions: Vec<ExecutionInfo>) -> Result<()> {
        *self.executions.write().await = executions;
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::ExecutionsUpdated)
            .await;
        Ok(())
    }

    /// Add a new execution
    pub async fn add_execution(&self, execution: ExecutionInfo) -> Result<()> {
        self.executions.write().await.push(execution.clone());
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::ExecutionStarted { execution })
            .await;
        Ok(())
    }

    /// Update execution status
    pub async fn update_execution_status(&self, id: &str, status: ExecutionStatus) -> Result<()> {
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.iter_mut().find(|e| e.id == id) {
            execution.status = status.clone();
            *self.last_update.write().await = Utc::now();
            drop(executions); // Release lock before broadcasting

            let event = if matches!(
                status,
                ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled
            ) {
                StateChangeEvent::ExecutionCompleted { id: id.to_string() }
            } else {
                StateChangeEvent::ExecutionStatusChanged {
                    id: id.to_string(),
                    status,
                }
            };
            self.broadcast_event(event).await;
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ValidationError(format!(
                "Execution not found: {}",
                id
            )))
        }
    }

    // Tool management methods

    /// Get all tools
    pub async fn get_tools(&self) -> Vec<ToolInfo> {
        self.tools.read().await.clone()
    }

    /// Set tools data
    pub async fn set_tools(&self, tools: Vec<ToolInfo>) -> Result<()> {
        *self.tools.write().await = tools;
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::ToolsUpdated).await;
        Ok(())
    }

    /// Add a new tool
    pub async fn add_tool(&self, tool: ToolInfo) -> Result<()> {
        self.tools.write().await.push(tool.clone());
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::ToolRegistered { tool })
            .await;
        Ok(())
    }

    /// Remove a tool
    pub async fn remove_tool(&self, name: &str) -> Result<()> {
        let mut tools = self.tools.write().await;
        let initial_len = tools.len();
        tools.retain(|t| t.name != name);

        if tools.len() < initial_len {
            *self.last_update.write().await = Utc::now();
            drop(tools); // Release lock before broadcasting
            self.broadcast_event(StateChangeEvent::ToolUnregistered {
                name: name.to_string(),
            })
            .await;
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ValidationError(format!(
                "Tool not found: {}",
                name
            )))
        }
    }

    // Plugin management methods

    /// Get all plugins
    pub async fn get_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.read().await.clone()
    }

    /// Set plugins data
    pub async fn set_plugins(&self, plugins: Vec<PluginInfo>) -> Result<()> {
        *self.plugins.write().await = plugins;
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::PluginsUpdated).await;
        Ok(())
    }

    /// Add a new plugin
    pub async fn add_plugin(&self, plugin: PluginInfo) -> Result<()> {
        self.plugins.write().await.push(plugin.clone());
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::PluginInstalled { plugin })
            .await;
        Ok(())
    }

    /// Remove a plugin
    pub async fn remove_plugin(&self, name: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        let initial_len = plugins.len();
        plugins.retain(|p| p.name != name);

        if plugins.len() < initial_len {
            *self.last_update.write().await = Utc::now();
            drop(plugins); // Release lock before broadcasting
            self.broadcast_event(StateChangeEvent::PluginUninstalled {
                name: name.to_string(),
            })
            .await;
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ValidationError(format!(
                "Plugin not found: {}",
                name
            )))
        }
    }

    // System status methods

    /// Get system status
    pub async fn get_system_status(&self) -> SystemStatus {
        self.system_status.read().await.clone()
    }

    /// Set system status
    pub async fn set_system_status(&self, status: SystemStatus) -> Result<()> {
        let old_health = self.system_status.read().await.system_health.clone();
        *self.system_status.write().await = status.clone();
        *self.last_update.write().await = Utc::now();

        self.broadcast_event(StateChangeEvent::SystemStatusUpdated)
            .await;

        // Broadcast health change if it changed
        if old_health != status.system_health {
            self.broadcast_event(StateChangeEvent::SystemHealthChanged {
                health: status.system_health,
            })
            .await;
        }

        Ok(())
    }

    // Log management methods

    /// Get all logs
    pub async fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.read().await.clone()
    }

    /// Add a log entry
    pub async fn add_log_entry(&self, entry: LogEntry) -> Result<()> {
        let mut logs = self.logs.write().await;
        logs.push(entry.clone());

        // Keep only the last 10000 log entries to prevent memory issues
        if logs.len() > 10000 {
            let excess = logs.len() - 10000;
            logs.drain(0..excess);
        }

        drop(logs); // Release lock before broadcasting
        self.broadcast_event(StateChangeEvent::LogEntryAdded { entry })
            .await;
        Ok(())
    }

    /// Clear all logs
    pub async fn clear_logs(&self) -> Result<()> {
        self.logs.write().await.clear();
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::LogsCleared).await;
        Ok(())
    }

    // Connection status methods

    /// Get connection status
    pub async fn get_connection_status(&self) -> ConnectionStatus {
        self.connection_status.read().await.clone()
    }

    /// Set connection status
    pub async fn set_connection_status(&self, status: ConnectionStatus) -> Result<()> {
        *self.connection_status.write().await = status.clone();
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::ConnectionStatusChanged { status })
            .await;
        Ok(())
    }

    /// Get connection status as display string
    pub async fn connection_status_string(&self) -> String {
        match &*self.connection_status.read().await {
            ConnectionStatus::Connected => "已连接".to_string(),
            ConnectionStatus::Connecting => "连接中...".to_string(),
            ConnectionStatus::Disconnected => "未连接".to_string(),
            ConnectionStatus::Error(err) => format!("错误: {}", err),
        }
    }

    // Performance metrics methods

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Update performance metrics
    pub async fn update_metrics(&self, metrics: PerformanceMetrics) -> Result<()> {
        *self.metrics.write().await = metrics;
        self.broadcast_event(StateChangeEvent::MetricsUpdated).await;
        Ok(())
    }

    /// Record widget update
    pub async fn record_widget_update(&self, widget_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let count = metrics
            .widget_update_count
            .entry(widget_id.to_string())
            .or_insert(0);
        *count += 1;
        metrics.last_measurement = Utc::now();
        Ok(())
    }

    // Utility methods

    /// Get last update timestamp
    pub async fn last_update(&self) -> DateTime<Utc> {
        *self.last_update.read().await
    }

    /// Get time since last update
    pub async fn time_since_last_update(&self) -> chrono::Duration {
        Utc::now() - *self.last_update.read().await
    }

    /// Check if data is stale
    pub async fn is_data_stale(&self, threshold: chrono::Duration) -> bool {
        self.time_since_last_update().await > threshold
    }

    /// Refresh all data (placeholder for backend integration)
    pub async fn refresh_all(&self) -> Result<()> {
        // This would integrate with actual backend services
        // For now, just update the timestamp and broadcast
        *self.last_update.write().await = Utc::now();
        self.broadcast_event(StateChangeEvent::DataRefreshed).await;
        tracing::info!("Refreshed all data");
        Ok(())
    }
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_total: 0,
            memory_used: 0,
            disk_usage: 0.0,
            disk_total: 0,
            disk_used: 0,
            active_workflows: 0,
            system_health: SystemHealth::Healthy,
            uptime: Duration::from_secs(0),
            network_status: NetworkStatus::Disconnected,
            load_average: [0.0, 0.0, 0.0],
            process_count: 0,
            thread_count: 0,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            render_time_ms: 0.0,
            update_time_ms: 0.0,
            event_processing_time_ms: 0.0,
            memory_usage_mb: 0.0,
            fps: 0.0,
            widget_update_count: HashMap::new(),
            last_measurement: Utc::now(),
        }
    }
}

impl From<StateChangeEvent> for StateChangeEventType {
    fn from(event: StateChangeEvent) -> Self {
        match event {
            StateChangeEvent::WorkflowsUpdated
            | StateChangeEvent::WorkflowStatusChanged { .. }
            | StateChangeEvent::WorkflowAdded { .. }
            | StateChangeEvent::WorkflowRemoved { .. } => StateChangeEventType::Workflows,

            StateChangeEvent::ExecutionsUpdated
            | StateChangeEvent::ExecutionStatusChanged { .. }
            | StateChangeEvent::ExecutionStarted { .. }
            | StateChangeEvent::ExecutionCompleted { .. } => StateChangeEventType::Executions,

            StateChangeEvent::ToolsUpdated
            | StateChangeEvent::ToolRegistered { .. }
            | StateChangeEvent::ToolUnregistered { .. } => StateChangeEventType::Tools,

            StateChangeEvent::PluginsUpdated
            | StateChangeEvent::PluginStatusChanged { .. }
            | StateChangeEvent::PluginInstalled { .. }
            | StateChangeEvent::PluginUninstalled { .. } => StateChangeEventType::Plugins,

            StateChangeEvent::SystemStatusUpdated
            | StateChangeEvent::SystemHealthChanged { .. } => StateChangeEventType::SystemStatus,

            StateChangeEvent::LogEntryAdded { .. } | StateChangeEvent::LogsCleared => {
                StateChangeEventType::Logs
            }

            StateChangeEvent::ConnectionStatusChanged { .. } => StateChangeEventType::Connection,

            StateChangeEvent::MetricsUpdated => StateChangeEventType::Metrics,

            StateChangeEvent::DataRefreshed => StateChangeEventType::All,
        }
    }
}
