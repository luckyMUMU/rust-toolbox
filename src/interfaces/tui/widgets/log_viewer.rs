//! Log Viewer Widget for TUI
//! 
//! This module implements a log viewer widget that displays system logs with
//! real-time streaming, filtering, searching, and navigation capabilities.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, 
        Scrollbar, ScrollbarOrientation, ScrollbarState
    },
};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::workflow::audit::{ExecutionLogEntry, LogLevel as AuditLogLevel};
use super::super::{
    action::{Action, LogLevel},
    theme::Theme,
    widget::{Widget, WidgetContext, WidgetId, WidgetCapabilities, SizeConstraints, UpdateFrequency, WidgetError},
};

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

/// Log viewer state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogViewerState {
    /// Normal viewing mode
    Normal,
    /// Search mode
    Search,
    /// Filter mode
    Filter,
    /// Export mode
    Export,
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
    
    /// Check if log level matches filter
    fn level_matches(&self, filter: &LogLevel, log_level: &LogLevel) -> bool {
        match filter {
            LogLevel::Error => matches!(log_level, LogLevel::Error),
            LogLevel::Warn => matches!(log_level, LogLevel::Error | LogLevel::Warn),
            LogLevel::Info => matches!(log_level, LogLevel::Error | LogLevel::Warn | LogLevel::Info),
            LogLevel::Debug => matches!(log_level, LogLevel::Error | LogLevel::Warn | LogLevel::Info | LogLevel::Debug),
            LogLevel::Trace => true, // Show all levels
        }
    }
    
    /// Apply search filter
    fn apply_search(&mut self) {
        self.search_results.clear();
        let query = self.search_query.to_lowercase();
        
        for (result_index, &log_index) in self.filtered_logs.iter().enumerate() {
            if let Some(log) = self.logs.get(log_index) {
                if log.message.to_lowercase().contains(&query) ||
                   log.source.as_ref().map_or(false, |s| s.to_lowercase().contains(&query)) {
                    self.search_results.push(result_index);
                }
            }
        }
        
        self.current_search_index = 0;
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
        
        self.selected_index = (self.selected_index + page_size).min(visible_count.saturating_sub(1));
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
    
    /// Clear all logs
    fn clear_logs(&mut self) {
        self.logs.clear();
        self.filtered_logs.clear();
        self.search_results.clear();
        self.selected_index = 0;
        self.update_selection();
    }
    
    /// Get the currently selected log entry
    fn selected_log(&self) -> Option<&LogEntry> {
        let log_index = if self.search_query.is_empty() {
            self.filtered_logs.get(self.selected_index).copied()
        } else {
            self.search_results.get(self.selected_index)
                .and_then(|&result_index| self.filtered_logs.get(result_index))
                .copied()
        };
        
        log_index.and_then(|index| self.logs.get(index))
    }
    
    /// Format log entry for display (static function to avoid borrowing issues)
    fn format_log_entry_static<'a>(log: &'a LogEntry, theme: &'a Theme, search_query: &'a str) -> ListItem<'a> {
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
        
        let timestamp = log.timestamp.format("%H:%M:%S%.3f").to_string();
        let source = log.source.as_deref().unwrap_or("unknown");
        
        // Highlight search terms if searching
        let message = if !search_query.is_empty() && 
                         log.message.to_lowercase().contains(&search_query.to_lowercase()) {
            // Simple highlighting - in a real implementation, you'd want more sophisticated highlighting
            log.message.clone()
        } else {
            log.message.clone()
        };
        
        let content = vec![
            Line::from(vec![
                Span::styled(level_symbol, level_style),
                Span::raw(" "),
                Span::styled(timestamp, theme.styles.text_dimmed),
                Span::raw(" "),
                Span::styled(format!("[{}]", source), theme.styles.text_dimmed),
                Span::raw(" "),
                Span::styled(message, theme.styles.text_normal),
            ]),
        ];
        
        ListItem::new(content)
    }
    
    /// Format log entry for display
    fn format_log_entry<'a>(&self, log: &'a LogEntry, theme: &'a Theme, search_query: &'a str) -> ListItem<'a> {
        Self::format_log_entry_static(log, theme, search_query)
    }
    
    /// Render the main log list
    fn render_log_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Collect log entries first to avoid borrowing conflicts
        let log_entries: Vec<LogEntry> = if self.search_query.is_empty() {
            self.filtered_logs.iter()
                .filter_map(|&index| self.logs.get(index))
                .cloned()
                .collect()
        } else {
            self.search_results.iter()
                .filter_map(|&result_index| {
                    self.filtered_logs.get(result_index)
                        .and_then(|&log_index| self.logs.get(log_index))
                })
                .cloned()
                .collect()
        };
        
        let search_query = self.search_query.clone();
        
        // Format log entries
        let visible_logs: Vec<ListItem> = log_entries.iter()
            .map(|log| Self::format_log_entry_static(log, theme, &search_query))
            .collect();
        
        let visible_count = visible_logs.len();
        let title = self.build_title();
        
        let list = List::new(visible_logs)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if self.context.has_focus {
                    theme.styles.widget_border_focused
                } else {
                    theme.styles.widget_border
                }))
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
        
        // Add filter status
        if let Some(ref level) = self.level_filter {
            title.push_str(&format!(" [过滤: {:?}]", level));
        }
        
        // Add search status
        if !self.search_query.is_empty() {
            title.push_str(&format!(" [搜索: {}]", self.search_query));
            if !self.search_results.is_empty() {
                title.push_str(&format!(" ({}/{})", 
                    self.current_search_index + 1, 
                    self.search_results.len()));
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
        
        let popup_area = self.centered_rect(60, 3, area);
        
        // Clear the area
        frame.render_widget(Clear, popup_area);
        
        let input_text = format!("搜索: {}", self.input_buffer);
        let input = Paragraph::new(input_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("搜索日志")
                .border_style(theme.styles.widget_border_focused))
            .style(theme.styles.text_normal);
        
        frame.render_widget(input, popup_area);
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
    
    async fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<(), WidgetError> {
        // Process any incoming log entries
        self.process_log_stream().await?;
        
        // Render the main log list
        self.render_log_list(frame, area, theme);
        
        // Render input overlay if in search mode
        self.render_input_overlay(frame, area, theme);
        
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
        if self.auto_scroll && self.last_auto_scroll.elapsed() > Duration::from_millis(AUTO_SCROLL_DELAY_MS) {
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
            ],
            _ => vec![
                ("↑/↓", "上下导航"),
                ("PgUp/PgDn", "翻页"),
                ("Home/End", "首页/末页"),
                ("/", "搜索"),
                ("n/N", "下一个/上一个搜索结果"),
                ("f", "过滤级别"),
                ("c", "清除日志"),
                ("a", "切换自动滚动"),
                ("e", "导出日志"),
                ("Esc", "清除搜索/过滤"),
            ],
        }
    }
}

impl LogViewerWidget {
    /// Handle keyboard events
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<Option<Action>, WidgetError> {
        match self.state {
            LogViewerState::Search => {
                match key_event.code {
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
                }
            }
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
                        Ok(Some(Action::ExportLogs))
                    }
                    KeyCode::Char('r') | KeyCode::F(5) => {
                        Ok(Some(Action::Refresh))
                    }
                    
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