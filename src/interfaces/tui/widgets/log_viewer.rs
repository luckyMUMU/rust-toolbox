//! Log Viewer Widget for TUI
//!
//! This module implements a log viewer widget that displays system logs with
//! real-time streaming, filtering, searching, and navigation capabilities.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::super::{
    action::{Action, LogLevel},
    theme::Theme,
    widget::{
        SizeConstraints, UpdateFrequency, Widget, WidgetCapabilities, WidgetContext, WidgetError,
        WidgetId,
    },
};
use crate::workflow::audit::{ExecutionLogEntry, LogLevel as AuditLogLevel};

/// Maximum number of log entries to keep in memory
const MAX_LOG_ENTRIES: usize = 10000;

/// Default auto-scroll delay in milliseconds
const AUTO_SCROLL_DELAY_MS: u64 = 100;

/// Log entry for display in the TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: Option<String>,
    pub execution_id: Option<String>,
    pub workflow_id: Option<String>,
    pub node_id: Option<String>,
}

impl From<ExecutionLogEntry> for LogEntry {
    fn from(entry: ExecutionLogEntry) -> Self {
        let level = match entry.level {
            AuditLogLevel::Trace => LogLevel::Trace,
            AuditLogLevel::Debug => LogLevel::Debug,
            AuditLogLevel::Info => LogLevel::Info,
            AuditLogLevel::Warn => LogLevel::Warn,
            AuditLogLevel::Error => LogLevel::Error,
        };

        Self {
            id: entry.log_id,
            timestamp: entry.timestamp,
            level,
            message: entry.message,
            source: entry.source_location,
            execution_id: Some(entry.execution_id),
            workflow_id: Some(entry.workflow_id.to_string()),
            node_id: entry.node_id,
        }
    }
}

/// Export mode options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportMode {
    /// Export all visible logs (filtered/searched)
    All,
    /// Export only the selected log entry
    Selected,
    /// Export logs within a date range
    DateRange,
}

/// Log viewer state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogViewerState {
    /// Normal viewing mode
    Normal,
    /// Search mode
    Search,
    /// Filter mode
    Filter,
    /// Export mode - choosing export options
    Export,
    /// Export confirmation mode
    ExportConfirm,
}

/// Log viewer widget implementation
pub struct LogViewerWidget {
    // Widget metadata
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,

    // Log data
    logs: VecDeque<LogEntry>,
    filtered_logs: Vec<usize>, // Indices into logs

    // UI state
    state: LogViewerState,
    selected_index: usize,
    scroll_offset: usize,
    list_state: ListState,
    scroll_state: ScrollbarState,

    // Auto-scroll settings
    auto_scroll: bool,
    last_auto_scroll: Instant,

    // Filtering and search
    level_filter: Option<LogLevel>,
    search_query: String,
    search_results: Vec<usize>, // Indices into filtered_logs
    current_search_index: usize,

    // Input handling
    input_buffer: String,

    // Export functionality
    export_mode: ExportMode,
    export_path: Option<PathBuf>,
    last_export_path: Option<PathBuf>,

    // Performance tracking
    last_update: Instant,
    update_count: u64,

    // Log streaming
    log_receiver: Option<mpsc::UnboundedReceiver<LogEntry>>,
}

impl LogViewerWidget {
    /// Create a new log viewer widget
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

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
            .min_size(40, 10)
            .preferred_size(80, 24);

        Self {
            context: WidgetContext::new(WidgetId::from("log_viewer")),
            capabilities,
            size_constraints,
            logs: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            filtered_logs: Vec::new(),
            state: LogViewerState::Normal,
            selected_index: 0,
            scroll_offset: 0,
            list_state,
            scroll_state: ScrollbarState::default(),
            auto_scroll: true,
            last_auto_scroll: Instant::now(),
            level_filter: None,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            input_buffer: String::new(),
            export_mode: ExportMode::All,
            export_path: None,
            last_export_path: None,
            last_update: Instant::now(),
            update_count: 0,
            log_receiver: None,
        }
    }

    /// Set up log streaming receiver
    pub fn set_log_receiver(&mut self, receiver: mpsc::UnboundedReceiver<LogEntry>) {
        self.log_receiver = Some(receiver);
    }

    /// Add a log entry
    pub fn add_log_entry(&mut self, entry: LogEntry) {
        // Maintain maximum log entries
        if self.logs.len() >= MAX_LOG_ENTRIES {
            self.logs.pop_front();
        }

        self.logs.push_back(entry);
        self.apply_filters();

        // Auto-scroll to bottom if enabled
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Apply current filters to the log entries
    fn apply_filters(&mut self) {
        self.filtered_logs.clear();

        for (index, log) in self.logs.iter().enumerate() {
            // Apply level filter
            if let Some(ref filter_level) = self.level_filter {
                if !self.level_matches(filter_level, &log.level) {
                    continue;
                }
            }

            self.filtered_logs.push(index);
        }

        // Apply search if active
        if !self.search_query.is_empty() {
            self.apply_search();
        }

        self.update_selection();
    }

    /// Check if log level matches filter (hierarchical filtering)
    fn level_matches(&self, filter: &LogLevel, log_level: &LogLevel) -> bool {
        use LogLevel::*;

        let filter_priority = match filter {
            Error => 4,
            Warn => 3,
            Info => 2,
            Debug => 1,
            Trace => 0,
        };

        let log_priority = match log_level {
            Error => 4,
            Warn => 3,
            Info => 2,
            Debug => 1,
            Trace => 0,
        };

        // Show logs at the filter level and above (higher priority)
        log_priority >= filter_priority
    }

    /// Apply search filter with enhanced search capabilities
    fn apply_search(&mut self) {
        self.search_results.clear();

        if self.search_query.is_empty() {
            return;
        }

        let query = self.search_query.to_lowercase();

        // Support for advanced search syntax
        let (search_terms, source_filter, execution_filter) = self.parse_search_query(&query);

        for (result_index, &log_index) in self.filtered_logs.iter().enumerate() {
            if let Some(log) = self.logs.get(log_index) {
                let mut matches = false;

                // Check message content
                if search_terms
                    .iter()
                    .any(|term| log.message.to_lowercase().contains(term))
                {
                    matches = true;
                }

                // Check source filter
                if let Some(ref source_term) = source_filter {
                    if log
                        .source
                        .as_ref()
                        .map_or(false, |s| s.to_lowercase().contains(source_term))
                    {
                        matches = true;
                    } else if !search_terms.is_empty() {
                        matches = false; // Source filter is restrictive
                    }
                }

                // Check execution filter
                if let Some(ref exec_term) = execution_filter {
                    if log
                        .execution_id
                        .as_ref()
                        .map_or(false, |id| id.to_lowercase().contains(exec_term))
                        || log
                            .workflow_id
                            .as_ref()
                            .map_or(false, |id| id.to_lowercase().contains(exec_term))
                    {
                        matches = true;
                    } else if !search_terms.is_empty() {
                        matches = false; // Execution filter is restrictive
                    }
                }

                // Check source in general search if no specific filters
                if source_filter.is_none() && execution_filter.is_none() {
                    if log
                        .source
                        .as_ref()
                        .map_or(false, |s| s.to_lowercase().contains(&query))
                    {
                        matches = true;
                    }
                }

                if matches {
                    self.search_results.push(result_index);
                }
            }
        }

        self.current_search_index = 0;
    }

    /// Parse search query for advanced syntax
    /// Supports: source:term, exec:term, workflow:term
    fn parse_search_query(&self, query: &str) -> (Vec<String>, Option<String>, Option<String>) {
        let mut search_terms = Vec::new();
        let mut source_filter = None;
        let mut execution_filter = None;

        let parts: Vec<&str> = query.split_whitespace().collect();

        for part in parts {
            if let Some(source_term) = part.strip_prefix("source:") {
                source_filter = Some(source_term.to_string());
            } else if let Some(exec_term) = part.strip_prefix("exec:") {
                execution_filter = Some(exec_term.to_string());
            } else if let Some(workflow_term) = part.strip_prefix("workflow:") {
                execution_filter = Some(workflow_term.to_string());
            } else {
                search_terms.push(part.to_string());
            }
        }

        (search_terms, source_filter, execution_filter)
    }

    /// Update selection and scroll state
    fn update_selection(&mut self) {
        let visible_count = if self.search_query.is_empty() {
            self.filtered_logs.len()
        } else {
            self.search_results.len()
        };

        if visible_count == 0 {
            self.selected_index = 0;
            self.list_state.select(None);
        } else {
            if self.selected_index >= visible_count {
                self.selected_index = visible_count.saturating_sub(1);
            }
            self.list_state.select(Some(self.selected_index));
        }

        // Update scroll state
        self.scroll_state = self.scroll_state.content_length(visible_count);
        self.scroll_state = self.scroll_state.position(self.selected_index);
    }

    /// Scroll to the bottom of the log
    fn scroll_to_bottom(&mut self) {
        let visible_count = if self.search_query.is_empty() {
            self.filtered_logs.len()
        } else {
            self.search_results.len()
        };

        if visible_count > 0 {
            self.selected_index = visible_count - 1;
            self.update_selection();
        }
    }

    /// Scroll to the top of the log
    fn scroll_to_top(&mut self) {
        self.selected_index = 0;
        self.update_selection();
    }

    /// Move selection up
    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_selection();
            self.auto_scroll = false; // Disable auto-scroll when manually navigating
        }
    }

    /// Move selection down
    fn move_down(&mut self) {
        let visible_count = if self.search_query.is_empty() {
            self.filtered_logs.len()
        } else {
            self.search_results.len()
        };

        if self.selected_index + 1 < visible_count {
            self.selected_index += 1;
            self.update_selection();
            self.auto_scroll = false; // Disable auto-scroll when manually navigating
        }
    }

    /// Page up
    fn page_up(&mut self, page_size: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_size);
        self.update_selection();
        self.auto_scroll = false;
    }

    /// Page down
    fn page_down(&mut self, page_size: usize) {
        let visible_count = if self.search_query.is_empty() {
            self.filtered_logs.len()
        } else {
            self.search_results.len()
        };

        self.selected_index =
            (self.selected_index + page_size).min(visible_count.saturating_sub(1));
        self.update_selection();
        self.auto_scroll = false;
    }

    /// Toggle auto-scroll
    fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Set level filter
    fn set_level_filter(&mut self, level: Option<LogLevel>) {
        self.level_filter = level;
        self.apply_filters();
    }

    /// Start search mode
    fn start_search(&mut self) {
        self.state = LogViewerState::Search;
        self.input_buffer.clear();
    }

    /// Confirm search
    fn confirm_search(&mut self) {
        self.search_query = self.input_buffer.clone();
        self.state = LogViewerState::Normal;
        self.input_buffer.clear();
        self.apply_filters();
    }

    /// Cancel search
    fn cancel_search(&mut self) {
        self.state = LogViewerState::Normal;
        self.input_buffer.clear();
    }

    /// Clear search
    fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_results.clear();
        self.current_search_index = 0;
        self.apply_filters();
    }

    /// Go to next search result
    fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
            if let Some(&result_index) = self.search_results.get(self.current_search_index) {
                self.selected_index = result_index;
                self.update_selection();
            }
        }
    }

    /// Go to previous search result
    fn previous_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = if self.current_search_index == 0 {
                self.search_results.len() - 1
            } else {
                self.current_search_index - 1
            };
            if let Some(&result_index) = self.search_results.get(self.current_search_index) {
                self.selected_index = result_index;
                self.update_selection();
            }
        }
    }

    /// Start export mode
    fn start_export(&mut self) {
        self.state = LogViewerState::Export;
        self.export_mode = ExportMode::All;
    }

    /// Confirm export with current settings
    async fn confirm_export(&mut self) -> Result<Option<Action>, WidgetError> {
        let export_result = match self.export_mode {
            ExportMode::All => self.export_logs(self.export_path.clone()).await,
            ExportMode::Selected => {
                match self.export_selected_log(self.export_path.clone()).await? {
                    Some(path) => Ok(path),
                    None => return Ok(Some(Action::ShowError("没有选中的日志条目".to_string()))),
                }
            }
            ExportMode::DateRange => {
                // For now, treat as all logs - could be enhanced later
                self.export_logs(self.export_path.clone()).await
            }
        };

        match export_result {
            Ok(path) => {
                self.last_export_path = Some(path.clone());
                self.state = LogViewerState::Normal;
                self.export_path = None;
                Ok(Some(Action::Custom(
                    "export_success".to_string(),
                    serde_json::json!({
                        "path": path.to_string_lossy(),
                        "mode": format!("{:?}", self.export_mode)
                    }),
                )))
            }
            Err(e) => {
                self.state = LogViewerState::Normal;
                Ok(Some(Action::ShowError(format!("导出失败: {}", e))))
            }
        }
    }

    /// Cancel export mode
    fn cancel_export(&mut self) {
        self.state = LogViewerState::Normal;
        self.export_path = None;
    }

    /// Cycle through export modes
    fn cycle_export_mode(&mut self) {
        self.export_mode = match self.export_mode {
            ExportMode::All => ExportMode::Selected,
            ExportMode::Selected => ExportMode::DateRange,
            ExportMode::DateRange => ExportMode::All,
        };
    }

    /// Clear all logs
    fn clear_logs(&mut self) {
        self.logs.clear();
        self.filtered_logs.clear();
        self.search_results.clear();
        self.selected_index = 0;
        self.update_selection();
    }

    /// Export logs to file
    pub async fn export_logs(&self, export_path: Option<PathBuf>) -> Result<PathBuf, WidgetError> {
        let export_path = export_path.unwrap_or_else(|| {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            PathBuf::from(format!("logs_export_{}.txt", timestamp))
        });

        // Collect logs to export (filtered logs if any filter is active)
        let logs_to_export: Vec<&LogEntry> = if self.search_query.is_empty() {
            self.filtered_logs
                .iter()
                .filter_map(|&index| self.logs.get(index))
                .collect()
        } else {
            self.search_results
                .iter()
                .filter_map(|&result_index| {
                    self.filtered_logs
                        .get(result_index)
                        .and_then(|&log_index| self.logs.get(log_index))
                })
                .collect()
        };

        // Create export file
        let mut file = File::create(&export_path).map_err(|e| WidgetError::StateError {
            message: format!("Failed to create export file: {}", e),
        })?;

        // Write header with export metadata
        writeln!(file, "# 日志导出文件").map_err(|e| WidgetError::StateError {
            message: format!("Failed to write to export file: {}", e),
        })?;
        writeln!(
            file,
            "# 导出时间: {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
        .map_err(|e| WidgetError::StateError {
            message: format!("Failed to write to export file: {}", e),
        })?;
        writeln!(file, "# 总日志条目: {}", self.logs.len()).map_err(|e| {
            WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            }
        })?;
        writeln!(file, "# 导出条目: {}", logs_to_export.len()).map_err(|e| {
            WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            }
        })?;

        if let Some(ref level) = self.level_filter {
            let level_name = match level {
                LogLevel::Error => "错误",
                LogLevel::Warn => "警告",
                LogLevel::Info => "信息",
                LogLevel::Debug => "调试",
                LogLevel::Trace => "跟踪",
            };
            writeln!(file, "# 级别过滤: {}+", level_name).map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;
        }

        if !self.search_query.is_empty() {
            writeln!(file, "# 搜索查询: {}", self.search_query).map_err(|e| {
                WidgetError::StateError {
                    message: format!("Failed to write to export file: {}", e),
                }
            })?;
        }

        writeln!(file, "#").map_err(|e| WidgetError::StateError {
            message: format!("Failed to write to export file: {}", e),
        })?;
        writeln!(file, "# 格式: [时间戳] [级别] [来源] 消息").map_err(|e| {
            WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            }
        })?;
        writeln!(file, "#").map_err(|e| WidgetError::StateError {
            message: format!("Failed to write to export file: {}", e),
        })?;
        writeln!(file).map_err(|e| WidgetError::StateError {
            message: format!("Failed to write to export file: {}", e),
        })?;

        // Write log entries with enhanced metadata
        for log in logs_to_export {
            let level_name = match log.level {
                LogLevel::Error => "ERROR",
                LogLevel::Warn => "WARN ",
                LogLevel::Info => "INFO ",
                LogLevel::Debug => "DEBUG",
                LogLevel::Trace => "TRACE",
            };

            let source = log.source.as_deref().unwrap_or("unknown");
            let timestamp = log.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC");

            // Write main log line
            writeln!(
                file,
                "[{}] [{}] [{}] {}",
                timestamp, level_name, source, log.message
            )
            .map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;

            // Write additional metadata if available
            if log.execution_id.is_some() || log.workflow_id.is_some() || log.node_id.is_some() {
                write!(file, "    元数据:").map_err(|e| WidgetError::StateError {
                    message: format!("Failed to write to export file: {}", e),
                })?;

                if let Some(ref exec_id) = log.execution_id {
                    write!(file, " 执行ID={}", exec_id).map_err(|e| WidgetError::StateError {
                        message: format!("Failed to write to export file: {}", e),
                    })?;
                }

                if let Some(ref workflow_id) = log.workflow_id {
                    write!(file, " 工作流ID={}", workflow_id).map_err(|e| {
                        WidgetError::StateError {
                            message: format!("Failed to write to export file: {}", e),
                        }
                    })?;
                }

                if let Some(ref node_id) = log.node_id {
                    write!(file, " 节点ID={}", node_id).map_err(|e| WidgetError::StateError {
                        message: format!("Failed to write to export file: {}", e),
                    })?;
                }

                writeln!(file).map_err(|e| WidgetError::StateError {
                    message: format!("Failed to write to export file: {}", e),
                })?;
            }

            writeln!(file).map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?; // Empty line between entries
        }

        // Write footer
        writeln!(
            file,
            "# 导出完成: {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
        .map_err(|e| WidgetError::StateError {
            message: format!("Failed to write to export file: {}", e),
        })?;

        file.flush().map_err(|e| WidgetError::StateError {
            message: format!("Failed to flush export file: {}", e),
        })?;

        Ok(export_path)
    }

    /// Export selected log entry with full metadata
    pub async fn export_selected_log(
        &self,
        export_path: Option<PathBuf>,
    ) -> Result<Option<PathBuf>, WidgetError> {
        if let Some(selected_log) = self.selected_log() {
            let export_path = export_path.unwrap_or_else(|| {
                let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                PathBuf::from(format!("log_entry_{}.txt", timestamp))
            });

            let mut file = File::create(&export_path).map_err(|e| WidgetError::StateError {
                message: format!("Failed to create export file: {}", e),
            })?;

            // Write detailed log entry with all metadata
            writeln!(file, "# 单条日志导出").map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;
            writeln!(
                file,
                "# 导出时间: {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            )
            .map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;
            writeln!(file).map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;

            writeln!(file, "日志ID: {}", selected_log.id).map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;
            writeln!(
                file,
                "时间戳: {}",
                selected_log.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC")
            )
            .map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;
            writeln!(file, "级别: {:?}", selected_log.level).map_err(|e| {
                WidgetError::StateError {
                    message: format!("Failed to write to export file: {}", e),
                }
            })?;
            writeln!(
                file,
                "来源: {}",
                selected_log.source.as_deref().unwrap_or("unknown")
            )
            .map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;

            if let Some(ref exec_id) = selected_log.execution_id {
                writeln!(file, "执行ID: {}", exec_id).map_err(|e| WidgetError::StateError {
                    message: format!("Failed to write to export file: {}", e),
                })?;
            }

            if let Some(ref workflow_id) = selected_log.workflow_id {
                writeln!(file, "工作流ID: {}", workflow_id).map_err(|e| {
                    WidgetError::StateError {
                        message: format!("Failed to write to export file: {}", e),
                    }
                })?;
            }

            if let Some(ref node_id) = selected_log.node_id {
                writeln!(file, "节点ID: {}", node_id).map_err(|e| WidgetError::StateError {
                    message: format!("Failed to write to export file: {}", e),
                })?;
            }

            writeln!(file).map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;
            writeln!(file, "消息:").map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;
            writeln!(file, "{}", selected_log.message).map_err(|e| WidgetError::StateError {
                message: format!("Failed to write to export file: {}", e),
            })?;

            file.flush().map_err(|e| WidgetError::StateError {
                message: format!("Failed to flush export file: {}", e),
            })?;

            Ok(Some(export_path))
        } else {
            Ok(None)
        }
    }

    /// Get the currently selected log entry
    fn selected_log(&self) -> Option<&LogEntry> {
        let log_index = if self.search_query.is_empty() {
            self.filtered_logs.get(self.selected_index).copied()
        } else {
            self.search_results
                .get(self.selected_index)
                .and_then(|&result_index| self.filtered_logs.get(result_index))
                .copied()
        };

        log_index.and_then(|index| self.logs.get(index))
    }

    /// Format log entry for display (static function to avoid borrowing issues)
    fn format_log_entry_static<'a>(
        log: &'a LogEntry,
        theme: &'a Theme,
        search_query: &'a str,
    ) -> ListItem<'a> {
        let level_symbol = match log.level {
            LogLevel::Error => "✗",
            LogLevel::Warn => "⚠",
            LogLevel::Info => "ℹ",
            LogLevel::Debug => "◦",
            LogLevel::Trace => "·",
        };

        let level_style = match log.level {
            LogLevel::Error => theme.styles.log_error,
            LogLevel::Warn => theme.styles.log_warn,
            LogLevel::Info => theme.styles.log_info,
            LogLevel::Debug => theme.styles.log_debug,
            LogLevel::Trace => theme.styles.log_trace,
        };

        // Enhanced timestamp display with more precision
        let timestamp = log.timestamp.format("%H:%M:%S%.3f").to_string();
        let date = log.timestamp.format("%m-%d").to_string();
        let source = log.source.as_deref().unwrap_or("unknown");

        // Create message spans with search highlighting
        let message_spans = if !search_query.is_empty() {
            Self::highlight_search_terms(&log.message, search_query, theme)
        } else {
            vec![Span::styled(log.message.clone(), theme.styles.text_normal)]
        };

        // Create source spans with search highlighting
        let source_spans = if !search_query.is_empty()
            && log.source.as_ref().map_or(false, |s| {
                s.to_lowercase().contains(&search_query.to_lowercase())
            }) {
            Self::highlight_search_terms(source, search_query, theme)
        } else {
            vec![Span::styled(
                format!("[{}]", source),
                theme.styles.text_dimmed,
            )]
        };

        // Main log line with enhanced metadata display
        let mut main_line_spans = vec![
            Span::styled(level_symbol, level_style),
            Span::raw(" "),
            Span::styled(date, theme.styles.text_dimmed),
            Span::raw(" "),
            Span::styled(timestamp, theme.styles.text_dimmed),
            Span::raw(" "),
        ];

        // Add highlighted source spans
        main_line_spans.extend(source_spans);
        main_line_spans.push(Span::raw(" "));

        // Add highlighted message spans
        main_line_spans.extend(message_spans);

        let mut content_lines = vec![Line::from(main_line_spans)];

        // Add metadata line if execution context is available
        if log.execution_id.is_some() || log.workflow_id.is_some() || log.node_id.is_some() {
            let mut metadata_spans = vec![
                Span::raw("    "),
                Span::styled("└─ ", theme.styles.text_dimmed),
            ];

            let mut metadata_parts = Vec::new();

            if let Some(ref exec_id) = log.execution_id {
                let short_id = if exec_id.len() > 8 {
                    format!("{}...", &exec_id[..8])
                } else {
                    exec_id.clone()
                };
                metadata_parts.push(format!("执行:{}", short_id));
            }

            if let Some(ref workflow_id) = log.workflow_id {
                let short_id = if workflow_id.len() > 12 {
                    format!("{}...", &workflow_id[..12])
                } else {
                    workflow_id.clone()
                };
                metadata_parts.push(format!("工作流:{}", short_id));
            }

            if let Some(ref node_id) = log.node_id {
                metadata_parts.push(format!("节点:{}", node_id));
            }

            let metadata_text = metadata_parts.join(" | ");
            metadata_spans.push(Span::styled(metadata_text, theme.styles.text_dimmed));

            content_lines.push(Line::from(metadata_spans));
        }

        ListItem::new(content_lines)
    }

    /// Highlight search terms in text
    fn highlight_search_terms<'a>(
        text: &'a str,
        search_query: &str,
        theme: &'a Theme,
    ) -> Vec<Span<'a>> {
        if search_query.is_empty() {
            return vec![Span::styled(text.to_string(), theme.styles.text_normal)];
        }

        let mut spans = Vec::new();
        let text_lower = text.to_lowercase();
        let query_lower = search_query.to_lowercase();
        let mut last_end = 0;

        // Find all occurrences of the search term (case-insensitive)
        for (start, _) in text_lower.match_indices(&query_lower) {
            // Add text before the match
            if start > last_end {
                spans.push(Span::styled(
                    text[last_end..start].to_string(),
                    theme.styles.text_normal,
                ));
            }

            // Add the highlighted match
            let end = start + search_query.len();
            spans.push(Span::styled(
                text[start..end].to_string(),
                theme.styles.search_highlight,
            ));

            last_end = end;
        }

        // Add remaining text after the last match
        if last_end < text.len() {
            spans.push(Span::styled(
                text[last_end..].to_string(),
                theme.styles.text_normal,
            ));
        }

        // If no matches found, return the original text
        if spans.is_empty() {
            spans.push(Span::styled(text.to_string(), theme.styles.text_normal));
        }

        spans
    }

    /// Format log entry for display
    fn format_log_entry<'a>(
        &self,
        log: &'a LogEntry,
        theme: &'a Theme,
        search_query: &'a str,
    ) -> ListItem<'a> {
        Self::format_log_entry_static(log, theme, search_query)
    }

    /// Render the main log list
    fn render_log_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Collect log entries first to avoid borrowing conflicts
        let log_entries: Vec<LogEntry> = if self.search_query.is_empty() {
            self.filtered_logs
                .iter()
                .filter_map(|&index| self.logs.get(index))
                .cloned()
                .collect()
        } else {
            self.search_results
                .iter()
                .filter_map(|&result_index| {
                    self.filtered_logs
                        .get(result_index)
                        .and_then(|&log_index| self.logs.get(log_index))
                })
                .cloned()
                .collect()
        };

        let search_query = self.search_query.clone();

        // Format log entries
        let visible_logs: Vec<ListItem> = log_entries
            .iter()
            .map(|log| Self::format_log_entry_static(log, theme, &search_query))
            .collect();

        let visible_count = visible_logs.len();
        let title = self.build_title();

        let list = List::new(visible_logs)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(if self.context.has_focus {
                        theme.styles.widget_border_focused
                    } else {
                        theme.styles.widget_border
                    }),
            )
            .highlight_style(theme.styles.list_item_selected)
            .highlight_symbol("► ");

        frame.render_stateful_widget(list, area, &mut self.list_state);

        // Render scrollbar if needed
        if visible_count > area.height as usize - 2 {
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

    /// Build the widget title with status information
    fn build_title(&self) -> String {
        let mut title = String::from("日志查看器");

        // Add log count
        let visible_count = if self.search_query.is_empty() {
            self.filtered_logs.len()
        } else {
            self.search_results.len()
        };
        title.push_str(&format!(" ({}/{})", visible_count, self.logs.len()));

        // Add filter status with level name
        if let Some(ref level) = self.level_filter {
            let level_name = match level {
                LogLevel::Error => "错误",
                LogLevel::Warn => "警告",
                LogLevel::Info => "信息",
                LogLevel::Debug => "调试",
                LogLevel::Trace => "跟踪",
            };
            title.push_str(&format!(" [过滤: {}+]", level_name));
        }

        // Add search status with advanced syntax info
        if !self.search_query.is_empty() {
            title.push_str(&format!(" [搜索: {}]", self.search_query));
            if !self.search_results.is_empty() {
                title.push_str(&format!(
                    " ({}/{})",
                    self.current_search_index + 1,
                    self.search_results.len()
                ));
            } else {
                title.push_str(" (无结果)");
            }
        }

        // Add auto-scroll status
        if self.auto_scroll {
            title.push_str(" [自动滚动]");
        }

        title
    }

    /// Render input overlay for search mode
    fn render_input_overlay(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.state != LogViewerState::Search {
            return;
        }

        let popup_area = self.centered_rect(80, 5, area);

        // Clear the area
        frame.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Help text
            ])
            .split(popup_area);

        // Input field
        let input_text = format!("搜索: {}", self.input_buffer);
        let input = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("搜索日志")
                    .border_style(theme.styles.widget_border_focused),
            )
            .style(theme.styles.text_normal);

        frame.render_widget(input, chunks[0]);

        // Help text
        let help_text = "语法: text source:name exec:id workflow:id";
        let help = Paragraph::new(help_text)
            .style(theme.styles.text_dimmed)
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(help, chunks[1]);
    }

    /// Render export overlay for export mode
    fn render_export_overlay(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.state != LogViewerState::Export {
            return;
        }

        let popup_area = self.centered_rect(70, 10, area);

        // Clear the area
        frame.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(6), // Export options
                Constraint::Length(1), // Help text
            ])
            .split(popup_area);

        // Title
        let title = Paragraph::new("导出日志")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("日志导出")
                    .border_style(theme.styles.widget_border_focused),
            )
            .style(theme.styles.text_normal);

        frame.render_widget(title, chunks[0]);

        // Export options
        let export_options = vec![
            Line::from(vec![
                Span::styled(
                    if matches!(self.export_mode, ExportMode::All) {
                        "► "
                    } else {
                        "  "
                    },
                    theme.styles.list_item_selected,
                ),
                Span::raw("导出所有可见日志 ("),
                Span::styled(
                    format!(
                        "{} 条",
                        if self.search_query.is_empty() {
                            self.filtered_logs.len()
                        } else {
                            self.search_results.len()
                        }
                    ),
                    theme.styles.text_dimmed,
                ),
                Span::raw(")"),
            ]),
            Line::from(vec![
                Span::styled(
                    if matches!(self.export_mode, ExportMode::Selected) {
                        "► "
                    } else {
                        "  "
                    },
                    theme.styles.list_item_selected,
                ),
                Span::raw("导出选中的日志条目"),
            ]),
            Line::from(vec![
                Span::styled(
                    if matches!(self.export_mode, ExportMode::DateRange) {
                        "► "
                    } else {
                        "  "
                    },
                    theme.styles.list_item_selected,
                ),
                Span::raw("导出日期范围内的日志 (暂不可用)"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("导出路径: "),
                Span::styled(
                    self.export_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "自动生成".to_string()),
                    theme.styles.text_dimmed,
                ),
            ]),
        ];

        let options = Paragraph::new(export_options)
            .block(Block::default().borders(Borders::ALL))
            .style(theme.styles.text_normal);

        frame.render_widget(options, chunks[1]);

        // Help text
        let help_text = "↑/↓: 选择模式  Enter: 确认导出  Esc: 取消";
        let help = Paragraph::new(help_text)
            .style(theme.styles.text_dimmed)
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(help, chunks[2]);
    }

    /// Create a centered rectangle
    fn centered_rect(&self, percent_x: u16, height: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((r.height - height) / 2),
                Constraint::Length(height),
                Constraint::Length((r.height - height) / 2),
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

    /// Process incoming log entries from the receiver
    async fn process_log_stream(&mut self) -> Result<(), WidgetError> {
        let mut new_entries = Vec::new();

        if let Some(ref mut receiver) = self.log_receiver {
            while let Ok(log_entry) = receiver.try_recv() {
                new_entries.push(log_entry);
            }
        }

        // Add all new entries after borrowing is done
        for entry in new_entries {
            self.add_log_entry(entry);
        }

        Ok(())
    }
}

#[async_trait]
impl Widget for LogViewerWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }

    fn title(&self) -> &str {
        "日志查看器"
    }

    fn description(&self) -> Option<&str> {
        Some("显示系统日志的实时流，支持过滤、搜索和导航")
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
        UpdateFrequency::Interval(Duration::from_millis(100))
    }

    async fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
    ) -> Result<(), WidgetError> {
        // Process any incoming log entries
        self.process_log_stream().await?;

        // Render the main log list
        self.render_log_list(frame, area, theme);

        // Render input overlay if in search mode
        self.render_input_overlay(frame, area, theme);

        // Render export overlay if in export mode
        self.render_export_overlay(frame, area, theme);

        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<Option<Action>, WidgetError> {
        if !self.context.can_handle_events() {
            return Ok(None);
        }

        match event {
            Event::Key(key_event) => self.handle_key_event(key_event).await,
            _ => Ok(None),
        }
    }

    async fn update(&mut self) -> Result<(), WidgetError> {
        self.context.mark_updated();
        self.update_count += 1;

        // Process log stream
        self.process_log_stream().await?;

        // Auto-scroll if enabled and enough time has passed
        if self.auto_scroll
            && self.last_auto_scroll.elapsed() > Duration::from_millis(AUTO_SCROLL_DELAY_MS)
        {
            let current_bottom = if self.search_query.is_empty() {
                self.filtered_logs.len().saturating_sub(1)
            } else {
                self.search_results.len().saturating_sub(1)
            };

            if self.selected_index < current_bottom {
                self.scroll_to_bottom();
            }

            self.last_auto_scroll = Instant::now();
        }

        Ok(())
    }

    fn help_text(&self) -> Vec<(&str, &str)> {
        match self.state {
            LogViewerState::Search => vec![
                ("Enter", "确认搜索"),
                ("Esc", "取消搜索"),
                ("Backspace", "删除字符"),
                ("", ""),
                ("搜索语法:", ""),
                ("text", "搜索消息内容"),
                ("source:name", "按来源过滤"),
                ("exec:id", "按执行ID过滤"),
                ("workflow:id", "按工作流ID过滤"),
            ],
            LogViewerState::Export => vec![
                ("↑/↓", "选择导出模式"),
                ("Enter", "确认导出"),
                ("Esc", "取消导出"),
                ("", ""),
                ("导出模式:", ""),
                ("全部", "导出所有可见日志"),
                ("选中", "导出当前选中日志"),
                ("范围", "按日期范围导出"),
            ],
            _ => vec![
                ("↑/↓", "上下导航"),
                ("PgUp/PgDn", "翻页"),
                ("Home/End", "首页/末页"),
                ("/", "搜索 (支持高级语法)"),
                ("n/N", "下一个/上一个搜索结果"),
                ("f", "循环过滤级别 (ERROR→WARN→INFO→DEBUG→TRACE→ALL)"),
                ("c", "清除日志"),
                ("a", "切换自动滚动"),
                ("e", "导出日志"),
                ("Esc", "清除搜索/过滤"),
                ("r/F5", "刷新"),
            ],
        }
    }
}

impl LogViewerWidget {
    /// Handle keyboard events
    async fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> Result<Option<Action>, WidgetError> {
        match self.state {
            LogViewerState::Search => match key_event.code {
                KeyCode::Enter => {
                    self.confirm_search();
                    Ok(None)
                }
                KeyCode::Esc => {
                    self.cancel_search();
                    Ok(None)
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    Ok(None)
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                    Ok(None)
                }
                _ => Ok(None),
            },
            LogViewerState::Export => match key_event.code {
                KeyCode::Enter => self.confirm_export().await,
                KeyCode::Esc => {
                    self.cancel_export();
                    Ok(None)
                }
                KeyCode::Up => {
                    self.cycle_export_mode();
                    Ok(None)
                }
                KeyCode::Down => {
                    self.cycle_export_mode();
                    Ok(None)
                }
                _ => Ok(None),
            },
            _ => {
                match key_event.code {
                    // Navigation
                    KeyCode::Up => {
                        self.move_up();
                        Ok(None)
                    }
                    KeyCode::Down => {
                        self.move_down();
                        Ok(None)
                    }
                    KeyCode::PageUp => {
                        self.page_up(10);
                        Ok(None)
                    }
                    KeyCode::PageDown => {
                        self.page_down(10);
                        Ok(None)
                    }
                    KeyCode::Home => {
                        self.scroll_to_top();
                        Ok(None)
                    }
                    KeyCode::End => {
                        self.scroll_to_bottom();
                        Ok(None)
                    }

                    // Search
                    KeyCode::Char('/') => {
                        self.start_search();
                        Ok(None)
                    }
                    KeyCode::Char('n') => {
                        self.next_search_result();
                        Ok(None)
                    }
                    KeyCode::Char('N') => {
                        self.previous_search_result();
                        Ok(None)
                    }

                    // Filtering
                    KeyCode::Char('f') => {
                        // Cycle through log levels
                        let next_filter = match self.level_filter {
                            None => Some(LogLevel::Error),
                            Some(LogLevel::Error) => Some(LogLevel::Warn),
                            Some(LogLevel::Warn) => Some(LogLevel::Info),
                            Some(LogLevel::Info) => Some(LogLevel::Debug),
                            Some(LogLevel::Debug) => Some(LogLevel::Trace),
                            Some(LogLevel::Trace) => None,
                        };
                        self.set_level_filter(next_filter);
                        Ok(None)
                    }

                    // Actions
                    KeyCode::Char('c') => {
                        self.clear_logs();
                        Ok(Some(Action::ClearLogs))
                    }
                    KeyCode::Char('a') => {
                        self.toggle_auto_scroll();
                        Ok(None)
                    }
                    KeyCode::Char('e') => {
                        self.start_export();
                        Ok(None)
                    }
                    KeyCode::Char('r') | KeyCode::F(5) => Ok(Some(Action::Refresh)),

                    // Clear search/filter
                    KeyCode::Esc => {
                        if !self.search_query.is_empty() {
                            self.clear_search();
                        } else if self.level_filter.is_some() {
                            self.set_level_filter(None);
                        }
                        Ok(None)
                    }

                    _ => Ok(None),
                }
            }
        }
    }
}

impl Default for LogViewerWidget {
    fn default() -> Self {
        Self::new()
    }
}
