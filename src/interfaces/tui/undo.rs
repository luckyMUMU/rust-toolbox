//! Undo System and Error Reporting for TUI
//!
//! This module provides undo/redo functionality for user operations and
//! comprehensive error reporting with user feedback collection.

use crate::error::{Result, WorkflowError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Frame,
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info};
use uuid::Uuid;

/// Represents an undoable operation
#[async_trait]
pub trait UndoableOperation: Send + Sync + fmt::Debug {
    /// Execute the operation
    async fn execute(&self) -> Result<OperationResult>;

    /// Undo the operation
    async fn undo(&self) -> Result<OperationResult>;

    /// Redo the operation (default implementation calls execute)
    async fn redo(&self) -> Result<OperationResult> {
        self.execute().await
    }

    /// Get a human-readable description of the operation
    fn description(&self) -> String;

    /// Get the operation type for categorization
    fn operation_type(&self) -> OperationType;

    /// Check if this operation can be undone
    fn can_undo(&self) -> bool {
        true
    }

    /// Check if this operation can be redone
    fn can_redo(&self) -> bool {
        true
    }

    /// Get the operation ID
    fn id(&self) -> Uuid;

    /// Get any data needed for undo/redo
    fn get_undo_data(&self) -> Option<serde_json::Value> {
        None
    }

    /// Set data needed for undo/redo
    fn set_undo_data(&mut self, _data: serde_json::Value) -> Result<()> {
        Ok(())
    }
}

/// Result of an operation execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub affected_resources: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

impl OperationResult {
    pub fn success<S: Into<String>>(message: S) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            affected_resources: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn failure<S: Into<String>>(message: S) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            affected_resources: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_resources(mut self, resources: Vec<String>) -> Self {
        self.affected_resources = resources;
        self
    }
}

/// Types of operations that can be undone
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    WorkflowExecution,
    WorkflowCreation,
    WorkflowModification,
    WorkflowDeletion,
    ConfigurationChange,
    ThemeChange,
    LayoutChange,
    PluginInstallation,
    PluginUninstallation,
    ToolExecution,
    FileOperation,
    DataImport,
    DataExport,
    UserPreference,
    Custom(String),
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationType::WorkflowExecution => write!(f, "工作流执行"),
            OperationType::WorkflowCreation => write!(f, "工作流创建"),
            OperationType::WorkflowModification => write!(f, "工作流修改"),
            OperationType::WorkflowDeletion => write!(f, "工作流删除"),
            OperationType::ConfigurationChange => write!(f, "配置更改"),
            OperationType::ThemeChange => write!(f, "主题更改"),
            OperationType::LayoutChange => write!(f, "布局更改"),
            OperationType::PluginInstallation => write!(f, "插件安装"),
            OperationType::PluginUninstallation => write!(f, "插件卸载"),
            OperationType::ToolExecution => write!(f, "工具执行"),
            OperationType::FileOperation => write!(f, "文件操作"),
            OperationType::DataImport => write!(f, "数据导入"),
            OperationType::DataExport => write!(f, "数据导出"),
            OperationType::UserPreference => write!(f, "用户偏好"),
            OperationType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl OperationType {
    pub fn icon(&self) -> &'static str {
        match self {
            OperationType::WorkflowExecution => "▶",
            OperationType::WorkflowCreation => "✚",
            OperationType::WorkflowModification => "✎",
            OperationType::WorkflowDeletion => "✗",
            OperationType::ConfigurationChange => "⚙",
            OperationType::ThemeChange => "🎨",
            OperationType::LayoutChange => "⊞",
            OperationType::PluginInstallation => "📦",
            OperationType::PluginUninstallation => "🗑",
            OperationType::ToolExecution => "🔧",
            OperationType::FileOperation => "📁",
            OperationType::DataImport => "📥",
            OperationType::DataExport => "📤",
            OperationType::UserPreference => "👤",
            OperationType::Custom(_) => "⚡",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            OperationType::WorkflowExecution => Color::Blue,
            OperationType::WorkflowCreation => Color::Green,
            OperationType::WorkflowModification => Color::Yellow,
            OperationType::WorkflowDeletion => Color::Red,
            OperationType::ConfigurationChange => Color::Cyan,
            OperationType::ThemeChange => Color::Magenta,
            OperationType::LayoutChange => Color::LightBlue,
            OperationType::PluginInstallation => Color::LightGreen,
            OperationType::PluginUninstallation => Color::LightRed,
            OperationType::ToolExecution => Color::White,
            OperationType::FileOperation => Color::Gray,
            OperationType::DataImport => Color::LightCyan,
            OperationType::DataExport => Color::LightMagenta,
            OperationType::UserPreference => Color::LightYellow,
            OperationType::Custom(_) => Color::White,
        }
    }
}

/// Represents an operation in the undo stack
#[derive(Debug)]
pub struct UndoStackEntry {
    pub operation: Arc<dyn UndoableOperation>,
    pub executed_at: DateTime<Utc>,
    pub undone_at: Option<DateTime<Utc>>,
    pub result: Option<OperationResult>,
    pub undo_result: Option<OperationResult>,
    pub state: OperationState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationState {
    Executed,
    Undone,
    Failed,
    UndoFailed,
}

impl OperationState {
    pub fn color(&self) -> Color {
        match self {
            OperationState::Executed => Color::Green,
            OperationState::Undone => Color::Yellow,
            OperationState::Failed => Color::Red,
            OperationState::UndoFailed => Color::Magenta,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            OperationState::Executed => "✓",
            OperationState::Undone => "↶",
            OperationState::Failed => "✗",
            OperationState::UndoFailed => "⚠",
        }
    }
}

/// Undo manager for handling operation history
pub struct UndoManager {
    undo_stack: Arc<RwLock<VecDeque<UndoStackEntry>>>,
    redo_stack: Arc<RwLock<VecDeque<UndoStackEntry>>>,
    max_history: usize,
    operation_sender: mpsc::UnboundedSender<UndoOperation>,
    operation_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<UndoOperation>>>>,
}

#[derive(Debug)]
pub enum UndoOperation {
    Execute(Arc<dyn UndoableOperation>),
    Undo,
    Redo,
    Clear,
    SetMaxHistory(usize),
}

impl UndoManager {
    pub fn new() -> Self {
        let (operation_sender, operation_receiver) = mpsc::unbounded_channel();

        Self {
            undo_stack: Arc::new(RwLock::new(VecDeque::new())),
            redo_stack: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 50,
            operation_sender,
            operation_receiver: Arc::new(RwLock::new(Some(operation_receiver))),
        }
    }

    pub async fn start_processing(&self) -> Result<()> {
        let mut receiver_guard = self.operation_receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            let mut manager = self.clone_for_processing().await;
            tokio::spawn(async move {
                while let Some(operation) = receiver.recv().await {
                    if let Err(e) = manager.handle_operation(operation).await {
                        error!("Failed to handle undo operation: {}", e);
                    }
                }
            });
        }
        Ok(())
    }

    async fn clone_for_processing(&self) -> Self {
        Self {
            undo_stack: Arc::clone(&self.undo_stack),
            redo_stack: Arc::clone(&self.redo_stack),
            max_history: self.max_history,
            operation_sender: self.operation_sender.clone(),
            operation_receiver: Arc::new(RwLock::new(None)),
        }
    }

    async fn handle_operation(&mut self, operation: UndoOperation) -> Result<()> {
        match operation {
            UndoOperation::Execute(op) => {
                self.execute_operation_internal(op).await?;
            }
            UndoOperation::Undo => {
                self.undo_internal().await?;
            }
            UndoOperation::Redo => {
                self.redo_internal().await?;
            }
            UndoOperation::Clear => {
                self.clear_internal().await;
            }
            UndoOperation::SetMaxHistory(max) => {
                self.set_max_history_internal(max).await;
            }
        }
        Ok(())
    }

    // Public API methods
    pub async fn execute_operation(&self, operation: Arc<dyn UndoableOperation>) -> Result<()> {
        self.operation_sender
            .send(UndoOperation::Execute(operation))
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to execute operation: {}", e))
            })?;
        Ok(())
    }

    pub async fn undo(&self) -> Result<()> {
        self.operation_sender
            .send(UndoOperation::Undo)
            .map_err(|e| WorkflowError::ValidationError(format!("Failed to undo: {}", e)))?;
        Ok(())
    }

    pub async fn redo(&self) -> Result<()> {
        self.operation_sender
            .send(UndoOperation::Redo)
            .map_err(|e| WorkflowError::ValidationError(format!("Failed to redo: {}", e)))?;
        Ok(())
    }

    pub async fn clear_history(&self) -> Result<()> {
        self.operation_sender
            .send(UndoOperation::Clear)
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to clear history: {}", e))
            })?;
        Ok(())
    }

    pub async fn set_max_history(&self, max: usize) -> Result<()> {
        self.operation_sender
            .send(UndoOperation::SetMaxHistory(max))
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to set max history: {}", e))
            })?;
        Ok(())
    }

    // Internal implementation methods
    async fn execute_operation_internal(
        &self,
        operation: Arc<dyn UndoableOperation>,
    ) -> Result<()> {
        info!("Executing operation: {}", operation.description());

        let result = operation.execute().await;
        let entry = UndoStackEntry {
            operation: Arc::clone(&operation),
            executed_at: Utc::now(),
            undone_at: None,
            result: Some(match &result {
                Ok(res) => res.clone(),
                Err(e) => OperationResult::failure(e.to_string()),
            }),
            undo_result: None,
            state: if result.is_ok() {
                OperationState::Executed
            } else {
                OperationState::Failed
            },
        };

        // Add to undo stack
        let mut undo_stack = self.undo_stack.write().await;
        undo_stack.push_back(entry);

        // Maintain max history
        while undo_stack.len() > self.max_history {
            undo_stack.pop_front();
        }

        // Clear redo stack when new operation is executed
        let mut redo_stack = self.redo_stack.write().await;
        redo_stack.clear();

        result.map(|_| ())
    }

    async fn undo_internal(&self) -> Result<()> {
        let mut undo_stack = self.undo_stack.write().await;
        let mut redo_stack = self.redo_stack.write().await;

        if let Some(mut entry) = undo_stack.pop_back() {
            if entry.operation.can_undo() && entry.state == OperationState::Executed {
                info!("Undoing operation: {}", entry.operation.description());

                let undo_result = entry.operation.undo().await;
                entry.undone_at = Some(Utc::now());
                entry.undo_result = Some(match &undo_result {
                    Ok(res) => res.clone(),
                    Err(e) => OperationResult::failure(e.to_string()),
                });
                entry.state = if undo_result.is_ok() {
                    OperationState::Undone
                } else {
                    OperationState::UndoFailed
                };

                redo_stack.push_back(entry);

                undo_result.map(|_| ())
            } else {
                // Put it back if it can't be undone
                undo_stack.push_back(entry);
                Err(WorkflowError::ValidationError("Operation cannot be undone".to_string()))
            }
        } else {
            Err(WorkflowError::ValidationError("No operations to undo".to_string()))
        }
    }

    async fn redo_internal(&self) -> Result<()> {
        let mut undo_stack = self.undo_stack.write().await;
        let mut redo_stack = self.redo_stack.write().await;

        if let Some(mut entry) = redo_stack.pop_back() {
            if entry.operation.can_redo() && entry.state == OperationState::Undone {
                info!("Redoing operation: {}", entry.operation.description());

                let redo_result = entry.operation.redo().await;
                entry.result = Some(match &redo_result {
                    Ok(res) => res.clone(),
                    Err(e) => OperationResult::failure(e.to_string()),
                });
                entry.state = if redo_result.is_ok() {
                    OperationState::Executed
                } else {
                    OperationState::Failed
                };
                entry.undone_at = None;
                entry.undo_result = None;

                undo_stack.push_back(entry);

                redo_result.map(|_| ())
            } else {
                // Put it back if it can't be redone
                redo_stack.push_back(entry);
                Err(WorkflowError::ValidationError("Operation cannot be redone".to_string()))
            }
        } else {
            Err(WorkflowError::ValidationError("No operations to redo".to_string()))
        }
    }

    async fn clear_internal(&self) {
        let mut undo_stack = self.undo_stack.write().await;
        let mut redo_stack = self.redo_stack.write().await;
        undo_stack.clear();
        redo_stack.clear();
        info!("Cleared undo/redo history");
    }

    async fn set_max_history_internal(&mut self, max: usize) {
        self.max_history = max;

        // Trim existing history if needed
        let mut undo_stack = self.undo_stack.write().await;
        while undo_stack.len() > max {
            undo_stack.pop_front();
        }

        let mut redo_stack = self.redo_stack.write().await;
        while redo_stack.len() > max {
            redo_stack.pop_front();
        }
    }

    // Query methods
    pub async fn can_undo(&self) -> bool {
        let undo_stack = self.undo_stack.read().await;
        undo_stack.back().is_some_and(|entry| {
            entry.operation.can_undo() && entry.state == OperationState::Executed
        })
    }

    pub async fn can_redo(&self) -> bool {
        let redo_stack = self.redo_stack.read().await;
        redo_stack.back().is_some_and(|entry| {
            entry.operation.can_redo() && entry.state == OperationState::Undone
        })
    }

    pub async fn get_undo_history(&self) -> Vec<UndoStackEntry> {
        let undo_stack = self.undo_stack.read().await;
        undo_stack.iter().cloned().collect()
    }

    pub async fn get_redo_history(&self) -> Vec<UndoStackEntry> {
        let redo_stack = self.redo_stack.read().await;
        redo_stack.iter().cloned().collect()
    }

    pub async fn get_next_undo_description(&self) -> Option<String> {
        let undo_stack = self.undo_stack.read().await;
        undo_stack.back().map(|entry| entry.operation.description())
    }

    pub async fn get_next_redo_description(&self) -> Option<String> {
        let redo_stack = self.redo_stack.read().await;
        redo_stack.back().map(|entry| entry.operation.description())
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for UndoStackEntry {
    fn clone(&self) -> Self {
        Self {
            operation: Arc::clone(&self.operation),
            executed_at: self.executed_at,
            undone_at: self.undone_at,
            result: self.result.clone(),
            undo_result: self.undo_result.clone(),
            state: self.state.clone(),
        }
    }
}

/// Error report for collecting user feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub error_type: ErrorReportType,
    pub severity: ErrorSeverity,
    pub steps_to_reproduce: Vec<String>,
    pub expected_behavior: String,
    pub actual_behavior: String,
    pub system_info: SystemInfo,
    pub logs: Vec<LogEntry>,
    pub screenshots: Vec<String>, // Base64 encoded or file paths
    pub user_contact: Option<String>,
    pub created_at: DateTime<Utc>,
    pub status: ReportStatus,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorReportType {
    Bug,
    FeatureRequest,
    Performance,
    Usability,
    Documentation,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportStatus {
    Draft,
    Submitted,
    InReview,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
    pub terminal: Option<String>,
    pub shell: Option<String>,
    pub locale: Option<String>,
    pub memory_total: u64,
    pub cpu_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl ErrorReport {
    pub fn new<S: Into<String>>(title: S, description: S, error_type: ErrorReportType) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            error_type,
            severity: ErrorSeverity::Medium,
            steps_to_reproduce: Vec::new(),
            expected_behavior: String::new(),
            actual_behavior: String::new(),
            system_info: SystemInfo::collect(),
            logs: Vec::new(),
            screenshots: Vec::new(),
            user_contact: None,
            created_at: Utc::now(),
            status: ReportStatus::Draft,
            tags: Vec::new(),
        }
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_steps(mut self, steps: Vec<String>) -> Self {
        self.steps_to_reproduce = steps;
        self
    }

    pub fn with_expected_behavior<S: Into<String>>(mut self, behavior: S) -> Self {
        self.expected_behavior = behavior.into();
        self
    }

    pub fn with_actual_behavior<S: Into<String>>(mut self, behavior: S) -> Self {
        self.actual_behavior = behavior.into();
        self
    }

    pub fn with_logs(mut self, logs: Vec<LogEntry>) -> Self {
        self.logs = logs;
        self
    }

    pub fn with_contact<S: Into<String>>(mut self, contact: S) -> Self {
        self.user_contact = Some(contact.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl SystemInfo {
    pub fn collect() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            terminal: std::env::var("TERM").ok(),
            shell: std::env::var("SHELL").ok(),
            locale: std::env::var("LANG").ok(),
            memory_total: 0,
            cpu_count: std::thread::available_parallelism()
                .map(|p| p.get() as u32)
                .unwrap_or(4),
        }
    }
}

/// Error reporting manager
pub struct ErrorReportManager {
    reports: Arc<RwLock<Vec<ErrorReport>>>,
    report_sender: mpsc::UnboundedSender<ErrorReportEvent>,
    report_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ErrorReportEvent>>>>,
}

#[derive(Debug)]
pub enum ErrorReportEvent {
    CreateReport(ErrorReport),
    UpdateReport(Uuid, ErrorReport),
    SubmitReport(Uuid),
    DeleteReport(Uuid),
    ExportReports(String), // File path
}

impl ErrorReportManager {
    pub fn new() -> Self {
        let (report_sender, report_receiver) = mpsc::unbounded_channel();

        Self {
            reports: Arc::new(RwLock::new(Vec::new())),
            report_sender,
            report_receiver: Arc::new(RwLock::new(Some(report_receiver))),
        }
    }

    pub async fn start_processing(&self) -> Result<()> {
        let mut receiver_guard = self.report_receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            let manager = self.clone_for_processing().await;

            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    if let Err(e) = manager.handle_report_event(event).await {
                        error!("Failed to handle error report event: {}", e);
                    }
                }
            });
        }
        Ok(())
    }

    async fn clone_for_processing(&self) -> Self {
        Self {
            reports: Arc::clone(&self.reports),
            report_sender: self.report_sender.clone(),
            report_receiver: Arc::new(RwLock::new(None)),
        }
    }

    async fn handle_report_event(&self, event: ErrorReportEvent) -> Result<()> {
        match event {
            ErrorReportEvent::CreateReport(report) => {
                let mut reports = self.reports.write().await;
                reports.push(report);
                info!("Created new error report");
            }
            ErrorReportEvent::UpdateReport(id, updated_report) => {
                let mut reports = self.reports.write().await;
                if let Some(report) = reports.iter_mut().find(|r| r.id == id) {
                    *report = updated_report;
                    info!("Updated error report {}", id);
                }
            }
            ErrorReportEvent::SubmitReport(id) => {
                let mut reports = self.reports.write().await;
                if let Some(report) = reports.iter_mut().find(|r| r.id == id) {
                    report.status = ReportStatus::Submitted;
                    info!("Submitted error report {}", id);
                    // Here you would send the report to your error tracking service
                }
            }
            ErrorReportEvent::DeleteReport(id) => {
                let mut reports = self.reports.write().await;
                reports.retain(|r| r.id != id);
                info!("Deleted error report {}", id);
            }
            ErrorReportEvent::ExportReports(path) => {
                let reports = self.reports.read().await;
                let json = serde_json::to_string_pretty(&*reports)?;
                tokio::fs::write(&path, json).await?;
                info!("Exported {} reports to {}", reports.len(), path);
            }
        }
        Ok(())
    }

    // Public API methods
    pub async fn create_report(&self, report: ErrorReport) -> Result<Uuid> {
        let id = report.id;
        self.report_sender
            .send(ErrorReportEvent::CreateReport(report))
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to create report: {}", e))
            })?;
        Ok(id)
    }

    pub async fn update_report(&self, id: Uuid, report: ErrorReport) -> Result<()> {
        self.report_sender
            .send(ErrorReportEvent::UpdateReport(id, report))
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to update report: {}", e))
            })?;
        Ok(())
    }

    pub async fn submit_report(&self, id: Uuid) -> Result<()> {
        self.report_sender
            .send(ErrorReportEvent::SubmitReport(id))
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to submit report: {}", e))
            })?;
        Ok(())
    }

    pub async fn delete_report(&self, id: Uuid) -> Result<()> {
        self.report_sender
            .send(ErrorReportEvent::DeleteReport(id))
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to delete report: {}", e))
            })?;
        Ok(())
    }

    pub async fn export_reports<S: Into<String>>(&self, path: S) -> Result<()> {
        self.report_sender
            .send(ErrorReportEvent::ExportReports(path.into()))
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to export reports: {}", e))
            })?;
        Ok(())
    }

    pub async fn get_reports(&self) -> Vec<ErrorReport> {
        self.reports.read().await.clone()
    }

    pub async fn get_report(&self, id: Uuid) -> Option<ErrorReport> {
        let reports = self.reports.read().await;
        reports.iter().find(|r| r.id == id).cloned()
    }

    pub async fn get_reports_by_status(&self, status: ReportStatus) -> Vec<ErrorReport> {
        let reports = self.reports.read().await;
        reports
            .iter()
            .filter(|r| r.status == status)
            .cloned()
            .collect()
    }
}

impl Default for ErrorReportManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget for displaying undo history
pub struct UndoHistoryWidget {
    undo_manager: Arc<UndoManager>,
    selected_index: usize,
    scroll_state: ScrollbarState,
    show_redo: bool,
}

impl UndoHistoryWidget {
    pub fn new(undo_manager: Arc<UndoManager>) -> Self {
        Self {
            undo_manager,
            selected_index: 0,
            scroll_state: ScrollbarState::default(),
            show_redo: false,
        }
    }

    pub fn toggle_redo_view(&mut self) {
        self.show_redo = !self.show_redo;
        self.selected_index = 0;
    }

    pub async fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let history = if self.show_redo {
            self.undo_manager.get_redo_history().await
        } else {
            self.undo_manager.get_undo_history().await
        };

        let title = if self.show_redo {
            format!("重做历史 ({})", history.len())
        } else {
            format!("撤销历史 ({})", history.len())
        };

        let items: Vec<ListItem> = history
            .iter()
            .map(|entry| {
                let age = entry.executed_at.format("%H:%M:%S").to_string();
                let status_icon = entry.state.icon();
                let type_icon = entry.operation.operation_type().icon();
                let description = entry.operation.description();

                let content = vec![
                    Line::from(vec![
                        Span::styled(status_icon, Style::default().fg(entry.state.color())),
                        Span::raw(" "),
                        Span::styled(
                            type_icon,
                            Style::default().fg(entry.operation.operation_type().color()),
                        ),
                        Span::raw(" "),
                        Span::styled(description, Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::raw("  时间: "),
                        Span::styled(age, Style::default().fg(Color::Gray)),
                        Span::raw(" | 类型: "),
                        Span::styled(
                            entry.operation.operation_type().to_string(),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]),
                ];

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        frame.render_widget(list, area);

        // Render scrollbar if needed
        if history.len() > area.height as usize - 2 {
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
}

/// Widget for creating error reports
pub struct ErrorReportWidget {
    report_manager: Arc<ErrorReportManager>,
    current_report: Option<ErrorReport>,
    editing_field: EditingField,
    input_buffer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditingField {
    None,
    Title,
    Description,
    StepsToReproduce,
    ExpectedBehavior,
    ActualBehavior,
    Contact,
}

impl ErrorReportWidget {
    pub fn new(report_manager: Arc<ErrorReportManager>) -> Self {
        Self {
            report_manager,
            current_report: None,
            editing_field: EditingField::None,
            input_buffer: String::new(),
        }
    }

    pub fn start_new_report(&mut self, error_type: ErrorReportType) {
        self.current_report = Some(ErrorReport::new("", "", error_type));
        self.editing_field = EditingField::Title;
        self.input_buffer.clear();
    }

    pub async fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if let Some(ref report) = self.current_report {
            self.render_report_form(frame, area, report);
        } else {
            self.render_report_list(frame, area).await;
        }
    }

    fn render_report_form(&self, frame: &mut Frame, area: Rect, report: &ErrorReport) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(5), // Description
                Constraint::Length(3), // Type and Severity
                Constraint::Min(0),    // Other fields
                Constraint::Length(2), // Controls
            ])
            .split(area);

        // Title field
        let title_style = if self.editing_field == EditingField::Title {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let title_text = if self.editing_field == EditingField::Title {
            &self.input_buffer
        } else {
            &report.title
        };

        let title_paragraph = Paragraph::new(title_text.as_str())
            .block(Block::default().borders(Borders::ALL).title("标题"))
            .style(title_style)
            .wrap(Wrap { trim: true });
        frame.render_widget(title_paragraph, chunks[0]);

        // Description field
        let desc_style = if self.editing_field == EditingField::Description {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let desc_text = if self.editing_field == EditingField::Description {
            &self.input_buffer
        } else {
            &report.description
        };

        let desc_paragraph = Paragraph::new(desc_text.as_str())
            .block(Block::default().borders(Borders::ALL).title("描述"))
            .style(desc_style)
            .wrap(Wrap { trim: true });
        frame.render_widget(desc_paragraph, chunks[1]);

        // Controls
        let controls_text = match self.editing_field {
            EditingField::None => "Tab: 编辑字段 | Enter: 提交报告 | Esc: 取消",
            _ => "Enter: 确认 | Esc: 取消编辑",
        };

        let controls_paragraph = Paragraph::new(controls_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(controls_paragraph, chunks[4]);
    }

    async fn render_report_list(&self, frame: &mut Frame<'_>, area: Rect) {
        let reports = self.report_manager.get_reports().await;

        let items: Vec<ListItem> = reports
            .iter()
            .map(|report| {
                let status_color = match report.status {
                    ReportStatus::Draft => Color::Gray,
                    ReportStatus::Submitted => Color::Blue,
                    ReportStatus::InReview => Color::Yellow,
                    ReportStatus::Resolved => Color::Green,
                    ReportStatus::Closed => Color::Red,
                };

                let content = vec![
                    Line::from(vec![
                        Span::styled(&report.title, Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{:?}]", report.status),
                            Style::default().fg(status_color),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("创建时间: "),
                        Span::styled(
                            report.created_at.format("%Y-%m-%d %H:%M").to_string(),
                            Style::default().fg(Color::Gray),
                        ),
                    ]),
                ];

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("错误报告"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        frame.render_widget(list, area);
    }
}

/// Example implementation of an undoable operation
#[derive(Debug)]
pub struct ExampleUndoableOperation {
    id: Uuid,
    description: String,
    operation_type: OperationType,
    #[allow(dead_code)]
    data: serde_json::Value,
    undo_data: Option<serde_json::Value>,
}

impl ExampleUndoableOperation {
    pub fn new<S: Into<String>>(
        description: S,
        operation_type: OperationType,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            description: description.into(),
            operation_type,
            data,
            undo_data: None,
        }
    }
}

#[async_trait]
impl UndoableOperation for ExampleUndoableOperation {
    async fn execute(&self) -> Result<OperationResult> {
        // Simulate operation execution
        debug!("Executing operation: {}", self.description);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(OperationResult::success(format!(
            "Executed: {}",
            self.description
        )))
    }

    async fn undo(&self) -> Result<OperationResult> {
        // Simulate operation undo
        debug!("Undoing operation: {}", self.description);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(OperationResult::success(format!(
            "Undone: {}",
            self.description
        )))
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn operation_type(&self) -> OperationType {
        self.operation_type.clone()
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn get_undo_data(&self) -> Option<serde_json::Value> {
        self.undo_data.clone()
    }

    fn set_undo_data(&mut self, data: serde_json::Value) -> Result<()> {
        self.undo_data = Some(data);
        Ok(())
    }
}
