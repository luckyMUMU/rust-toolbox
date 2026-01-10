//! Tool Manager Widget Implementation
//! 
//! This widget displays and manages available tools with filtering, sorting, and execution capabilities.

use crate::interfaces::tui::{
    Widget, WidgetId, WidgetContext, Theme
};
use crate::interfaces::tui::widget::{WidgetCapabilities, SizeConstraints, UpdateFrequency, WidgetError};
use crate::interfaces::tui::action::{Action, SortOrder, InputMode};
use crate::core::ToolInfo;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Tool execution status for display
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolStatus {
    Available,
    Running,
    Error,
    Disabled,
}

impl ToolStatus {
    pub fn symbol(&self) -> &str {
        match self {
            ToolStatus::Available => "○",
            ToolStatus::Running => "●",
            ToolStatus::Error => "✗",
            ToolStatus::Disabled => "⊘",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            ToolStatus::Available => Color::White,
            ToolStatus::Running => Color::Green,
            ToolStatus::Error => Color::Red,
            ToolStatus::Disabled => Color::DarkGray,
        }
    }
    
    pub fn description(&self) -> &str {
        match self {
            ToolStatus::Available => "可用",
            ToolStatus::Running => "运行中",
            ToolStatus::Error => "错误",
            ToolStatus::Disabled => "已禁用",
        }
    }
}

impl PartialOrd for ToolStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ToolStatus {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use ToolStatus::*;
        match (self, other) {
            (Running, Running) => std::cmp::Ordering::Equal,
            (Running, _) => std::cmp::Ordering::Less,
            (_, Running) => std::cmp::Ordering::Greater,
            (Error, Error) => std::cmp::Ordering::Equal,
            (Error, _) => std::cmp::Ordering::Less,
            (_, Error) => std::cmp::Ordering::Greater,
            (Available, Available) => std::cmp::Ordering::Equal,
            (Available, _) => std::cmp::Ordering::Less,
            (_, Available) => std::cmp::Ordering::Greater,
            (Disabled, Disabled) => std::cmp::Ordering::Equal,
        }
    }
}

/// Enhanced tool information for display in the TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDisplayInfo {
    pub info: ToolInfo,
    pub status: ToolStatus,
    pub last_execution: Option<DateTime<Utc>>,
    pub execution_count: u32,
    pub average_duration: Option<Duration>,
    pub success_rate: Option<f64>,
    pub source: String, // Plugin name or "Built-in"
}

impl ToolDisplayInfo {
    /// Create a new ToolDisplayInfo from a ToolInfo
    pub fn from_tool_info(info: ToolInfo) -> Self {
        let source = info.plugin_name.clone().unwrap_or_else(|| "Built-in".to_string());
        
        Self {
            info,
            status: ToolStatus::Available,
            last_execution: None,
            execution_count: 0,
            average_duration: None,
            success_rate: None,
            source,
        }
    }
    
    /// Check if the tool matches a filter string
    pub fn matches_filter(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        
        let filter_lower = filter.to_lowercase();
        
        // Support advanced filter syntax
        if filter.starts_with("status:") {
            let status_filter = filter[7..].trim().to_lowercase();
            return self.status.description().to_lowercase().contains(&status_filter);
        }
        
        if filter.starts_with("category:") {
            let category_filter = filter[9..].trim().to_lowercase();
            return self.info.category.as_ref()
                .map_or(false, |cat| cat.to_lowercase().contains(&category_filter));
        }
        
        if filter.starts_with("tag:") {
            let tag_filter = filter[4..].trim().to_lowercase();
            return self.info.tags.iter().any(|tag| tag.to_lowercase().contains(&tag_filter));
        }
        
        if filter.starts_with("source:") {
            let source_filter = filter[7..].trim().to_lowercase();
            return self.source.to_lowercase().contains(&source_filter);
        }
        
        if filter.starts_with("version:") {
            let version_filter = filter[8..].trim().to_lowercase();
            return self.info.version.to_lowercase().contains(&version_filter);
        }
        
        // Check name
        if self.info.name.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        // Check description
        if self.info.description.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        // Check category
        if let Some(category) = &self.info.category {
            if category.to_lowercase().contains(&filter_lower) {
                return true;
            }
        }
        
        // Check tags
        if self.info.tags.iter().any(|tag| tag.to_lowercase().contains(&filter_lower)) {
            return true;
        }
        
        // Check source
        if self.source.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        // Check version
        if self.info.version.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        false
    }
}

/// Tool Manager Widget for displaying and managing tools
pub struct ToolManagerWidget {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
    
    // Widget state
    tools: Vec<ToolDisplayInfo>,
    filtered_tools: Vec<usize>, // Indices into tools vec
    selected_index: usize,
    scroll_state: ScrollbarState,
    list_state: ListState,
    
    // Filtering and sorting
    filter: String,
    sort_order: SortOrder,
    search_mode: bool,
    
    // Display options
    show_details: bool,
    show_help: bool,
    
    // Performance tracking
    last_update: Option<std::time::Instant>,
    update_count: u64,
}

impl ToolManagerWidget {
    /// Create a new ToolManagerWidget
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
            .min_size(50, 10)
            .preferred_size(90, 30);
        
        Self {
            context: WidgetContext::new(WidgetId::from("tool_manager")),
            capabilities,
            size_constraints,
            tools: Vec::new(),
            filtered_tools: Vec::new(),
            selected_index: 0,
            scroll_state: ScrollbarState::default(),
            list_state,
            filter: String::new(),
            sort_order: SortOrder::NameAsc,
            search_mode: false,
            show_details: false,
            show_help: false,
            last_update: None,
            update_count: 0,
        }
    }
    
    /// Set the tools to display
    pub fn set_tools(&mut self, tools: Vec<ToolDisplayInfo>) {
        self.tools = tools;
        self.apply_filter_and_sort();
        self.update_selection();
        self.last_update = Some(std::time::Instant::now());
        self.update_count += 1;
    }
    
    /// Add a tool to the list
    pub fn add_tool(&mut self, tool: ToolDisplayInfo) {
        self.tools.push(tool);
        self.apply_filter_and_sort();
        self.update_selection();
        self.update_count += 1;
    }
    
    /// Remove a tool by name
    pub fn remove_tool(&mut self, name: &str) -> bool {
        if let Some(pos) = self.tools.iter().position(|t| t.info.name == name) {
            self.tools.remove(pos);
            self.apply_filter_and_sort();
            self.update_selection();
            self.update_count += 1;
            true
        } else {
            false
        }
    }
    
    /// Update a tool's status
    pub fn update_tool_status(&mut self, name: &str, status: ToolStatus) -> bool {
        if let Some(tool) = self.tools.iter_mut().find(|t| t.info.name == name) {
            tool.status = status;
            self.apply_filter_and_sort();
            self.update_selection();
            self.update_count += 1;
            true
        } else {
            false
        }
    }
    
    /// Get the currently selected tool
    pub fn selected_tool(&self) -> Option<&ToolDisplayInfo> {
        self.filtered_tools
            .get(self.selected_index)
            .and_then(|&index| self.tools.get(index))
    }
    
    /// Get the currently selected tool name
    pub fn selected_tool_name(&self) -> Option<String> {
        self.selected_tool().map(|t| t.info.name.clone())
    }
    
    /// Set the filter string
    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Clear the filter
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Set the sort order
    pub fn set_sort_order(&mut self, sort_order: SortOrder) {
        self.sort_order = sort_order;
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Toggle details view
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
    
    /// Toggle help view
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
    }
    
    /// Exit search mode
    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
    }
    
    /// Move selection up
    pub fn select_previous(&mut self) {
        if !self.filtered_tools.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_tools.len() - 1
            } else {
                self.selected_index - 1
            };
            self.update_selection();
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.filtered_tools.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_tools.len();
            self.update_selection();
        }
    }
    
    /// Move to first item
    pub fn select_first(&mut self) {
        if !self.filtered_tools.is_empty() {
            self.selected_index = 0;
            self.update_selection();
        }
    }
    
    /// Move to last item
    pub fn select_last(&mut self) {
        if !self.filtered_tools.is_empty() {
            self.selected_index = self.filtered_tools.len() - 1;
            self.update_selection();
        }
    }
    
    /// Apply current filter and sort order
    fn apply_filter_and_sort(&mut self) {
        // Apply filter
        self.filtered_tools = self.tools
            .iter()
            .enumerate()
            .filter(|(_, tool)| tool.matches_filter(&self.filter))
            .map(|(i, _)| i)
            .collect();
        
        // Apply sort
        self.filtered_tools.sort_by(|&a, &b| {
            let tool_a = &self.tools[a];
            let tool_b = &self.tools[b];
            
            match self.sort_order {
                SortOrder::NameAsc => tool_a.info.name.cmp(&tool_b.info.name),
                SortOrder::NameDesc => tool_b.info.name.cmp(&tool_a.info.name),
                SortOrder::DateAsc => tool_a.last_execution.cmp(&tool_b.last_execution),
                SortOrder::DateDesc => tool_b.last_execution.cmp(&tool_a.last_execution),
                SortOrder::StatusAsc => tool_a.status.cmp(&tool_b.status),
                SortOrder::StatusDesc => tool_b.status.cmp(&tool_a.status),
                SortOrder::Custom(ref custom) => {
                    match custom.as_str() {
                        "category_asc" => tool_a.info.category.cmp(&tool_b.info.category),
                        "category_desc" => tool_b.info.category.cmp(&tool_a.info.category),
                        "source_asc" => tool_a.source.cmp(&tool_b.source),
                        "source_desc" => tool_b.source.cmp(&tool_a.source),
                        "execution_count_asc" => tool_a.execution_count.cmp(&tool_b.execution_count),
                        "execution_count_desc" => tool_b.execution_count.cmp(&tool_a.execution_count),
                        "success_rate_asc" => tool_a.success_rate.partial_cmp(&tool_b.success_rate).unwrap_or(std::cmp::Ordering::Equal),
                        "success_rate_desc" => tool_b.success_rate.partial_cmp(&tool_a.success_rate).unwrap_or(std::cmp::Ordering::Equal),
                        "version_asc" => tool_a.info.version.cmp(&tool_b.info.version),
                        "version_desc" => tool_b.info.version.cmp(&tool_a.info.version),
                        _ => std::cmp::Ordering::Equal,
                    }
                }
            }
        });
    }
    
    /// Update selection state
    fn update_selection(&mut self) {
        if self.selected_index >= self.filtered_tools.len() && !self.filtered_tools.is_empty() {
            self.selected_index = self.filtered_tools.len() - 1;
        }
        
        self.list_state.select(if self.filtered_tools.is_empty() {
            None
        } else {
            Some(self.selected_index)
        });
        
        // Update scrollbar state
        self.scroll_state = self.scroll_state.content_length(self.filtered_tools.len());
        self.scroll_state = self.scroll_state.position(self.selected_index);
    }
}

#[async_trait]
impl Widget for ToolManagerWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }
    
    fn title(&self) -> &str {
        "工具管理器"
    }
    
    fn description(&self) -> Option<&str> {
        Some("显示和管理可用的工具")
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
        UpdateFrequency::Interval(Duration::from_secs(10)) // Update every 10 seconds
    }
    
    async fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) -> std::result::Result<(), WidgetError> {
        if self.show_help {
            // Show help overlay
            let help_area = Rect {
                x: area.x + area.width / 4,
                y: area.y + area.height / 4,
                width: area.width / 2,
                height: area.height / 2,
            };
            
            // Clear the help area
            frame.render_widget(Clear, help_area);
            self.render_help(frame, help_area, theme);
            return Ok(());
        }
        
        if self.show_details {
            // Split view: list on left, details on right
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);
            
            self.render_tool_list(frame, chunks[0], theme);
            self.render_tool_details(frame, chunks[1], theme);
        } else {
            // Full width list
            self.render_tool_list(frame, area, theme);
        }
        
        Ok(())
    }
    
    async fn handle_event(&mut self, event: Event) -> std::result::Result<Option<Action>, WidgetError> {
        if let Event::Key(key) = event {
            return self.handle_key_event(key).await;
        }
        Ok(None)
    }
    
    async fn update(&mut self) -> std::result::Result<(), WidgetError> {
        self.context.mark_updated();
        // In a real implementation, this would fetch updated tool data
        // For now, we just track that an update occurred
        self.update_count += 1;
        Ok(())
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("↑/k", "上一个工具"),
            ("↓/j", "下一个工具"),
            ("Home/g", "第一个工具"),
            ("End/G", "最后一个工具"),
            ("Enter", "执行工具"),
            ("Space", "切换详情"),
            ("/", "搜索/过滤"),
            ("f", "快速过滤"),
            ("s", "循环排序"),
            ("c", "清除过滤"),
            ("r", "刷新"),
            ("?", "帮助"),
            ("Esc", "返回"),
            ("Ctrl+Q", "退出"),
        ]
    }
}

impl Default for ToolManagerWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManagerWidget {
    /// Render the tool list
    fn render_tool_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Collect the filter and other needed values before creating the closure
        let filter = self.filter.clone();
        let search_mode = self.search_mode;
        
        let items: Vec<ListItem> = self.filtered_tools
            .iter()
            .map(|&index| {
                let tool = &self.tools[index];
                
                let status_style = Style::default().fg(tool.status.color());
                
                let last_exec = tool.last_execution
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "从未执行".to_string());
                
                // Highlight search terms in the tool name
                let name_spans = if !filter.is_empty() && !filter.starts_with("status:") && !filter.starts_with("category:") && !filter.starts_with("tag:") && !filter.starts_with("source:") && !filter.starts_with("version:") {
                    Self::highlight_text_static(&tool.info.name, &filter, Style::default().add_modifier(Modifier::BOLD))
                } else {
                    vec![Span::styled(&tool.info.name, Style::default().add_modifier(Modifier::BOLD))]
                };
                
                let mut first_line = vec![
                    Span::styled(tool.status.symbol(), status_style),
                    Span::raw(" "),
                ];
                first_line.extend(name_spans);
                first_line.extend(vec![
                    Span::raw(" v"),
                    Span::styled(&tool.info.version, Style::default().fg(Color::Gray)),
                    Sp