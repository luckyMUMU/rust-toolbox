//! Workflow List Widget Implementation
//! 
//! This widget displays a list of available workflows with filtering, sorting, and selection capabilities.

use crate::interfaces::tui::{
    Widget, WidgetId, WidgetContext, Theme
};
use crate::interfaces::tui::widget::{WidgetCapabilities, SizeConstraints, UpdateFrequency, WidgetError};
use crate::interfaces::tui::action::{Action, SortOrder, InputMode};
use crate::workflow::definition::WorkflowDefinition;
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

/// Information about a workflow for display in the TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub status: WorkflowStatus,
    pub last_execution: Option<DateTime<Utc>>,
    pub execution_count: u32,
    pub tags: Vec<String>,
    pub node_count: usize,
    pub estimated_duration: Option<Duration>,
    pub success_rate: Option<f64>,
}

impl WorkflowInfo {
    /// Create a new WorkflowInfo from a WorkflowDefinition
    pub fn from_definition(definition: &WorkflowDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            version: definition.version.clone(),
            description: definition.description.clone(),
            status: WorkflowStatus::Available,
            last_execution: None,
            execution_count: 0,
            tags: Vec::new(),
            node_count: definition.nodes.len(),
            estimated_duration: None,
            success_rate: None,
        }
    }
    
    /// Get a formatted status string
    pub fn status_string(&self) -> &str {
        match self.status {
            WorkflowStatus::Available => "可用",
            WorkflowStatus::Running => "运行中",
            WorkflowStatus::Paused => "已暂停",
            WorkflowStatus::Completed => "已完成",
            WorkflowStatus::Failed => "失败",
            WorkflowStatus::Disabled => "已禁用",
        }
    }
    
    /// Get the status color
    pub fn status_color(&self) -> Color {
        match self.status {
            WorkflowStatus::Available => Color::White,
            WorkflowStatus::Running => Color::Green,
            WorkflowStatus::Paused => Color::Yellow,
            WorkflowStatus::Completed => Color::Blue,
            WorkflowStatus::Failed => Color::Red,
            WorkflowStatus::Disabled => Color::DarkGray,
        }
    }
    
    /// Get the status symbol
    pub fn status_symbol(&self) -> &str {
        match self.status {
            WorkflowStatus::Available => "○",
            WorkflowStatus::Running => "●",
            WorkflowStatus::Paused => "⏸",
            WorkflowStatus::Completed => "✓",
            WorkflowStatus::Failed => "✗",
            WorkflowStatus::Disabled => "⊘",
        }
    }
    
    /// Check if the workflow matches a filter string
    pub fn matches_filter(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        
        let filter_lower = filter.to_lowercase();
        
        // Support advanced filter syntax
        if filter.starts_with("status:") {
            let status_filter = filter[7..].trim().to_lowercase();
            return self.status_string().to_lowercase().contains(&status_filter);
        }
        
        if filter.starts_with("tag:") {
            let tag_filter = filter[4..].trim().to_lowercase();
            return self.tags.iter().any(|tag| tag.to_lowercase().contains(&tag_filter));
        }
        
        if filter.starts_with("version:") {
            let version_filter = filter[8..].trim().to_lowercase();
            return self.version.to_lowercase().contains(&version_filter);
        }
        
        if filter.starts_with("nodes:") {
            if let Ok(node_count) = filter[6..].trim().parse::<usize>() {
                return self.node_count == node_count;
            }
        }
        
        if filter.starts_with("nodes>") {
            if let Ok(node_count) = filter[6..].trim().parse::<usize>() {
                return self.node_count > node_count;
            }
        }
        
        if filter.starts_with("nodes<") {
            if let Ok(node_count) = filter[6..].trim().parse::<usize>() {
                return self.node_count < node_count;
            }
        }
        
        // Check name
        if self.name.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        // Check description
        if let Some(desc) = &self.description {
            if desc.to_lowercase().contains(&filter_lower) {
                return true;
            }
        }
        
        // Check tags
        if self.tags.iter().any(|tag| tag.to_lowercase().contains(&filter_lower)) {
            return true;
        }
        
        // Check status
        if self.status_string().to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        // Check version
        if self.version.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        false
    }
}

/// Execution history entry for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistoryEntry {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub duration: Option<Duration>,
    pub error_message: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

/// Execution status for history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ExecutionStatus {
    pub fn symbol(&self) -> &str {
        match self {
            ExecutionStatus::Pending => "⏳",
            ExecutionStatus::Running => "▶",
            ExecutionStatus::Completed => "✅",
            ExecutionStatus::Failed => "❌",
            ExecutionStatus::Cancelled => "⏹",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            ExecutionStatus::Pending => Color::Yellow,
            ExecutionStatus::Running => Color::Blue,
            ExecutionStatus::Completed => Color::Green,
            ExecutionStatus::Failed => Color::Red,
            ExecutionStatus::Cancelled => Color::Gray,
        }
    }
}

/// Workflow execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Available,
    Running,
    Paused,
    Completed,
    Failed,
    Disabled,
}

impl PartialOrd for WorkflowStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorkflowStatus {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use WorkflowStatus::*;
        match (self, other) {
            (Running, Running) => std::cmp::Ordering::Equal,
            (Running, _) => std::cmp::Ordering::Less,
            (_, Running) => std::cmp::Ordering::Greater,
            (Failed, Failed) => std::cmp::Ordering::Equal,
            (Failed, _) => std::cmp::Ordering::Less,
            (_, Failed) => std::cmp::Ordering::Greater,
            (Paused, Paused) => std::cmp::Ordering::Equal,
            (Paused, _) => std::cmp::Ordering::Less,
            (_, Paused) => std::cmp::Ordering::Greater,
            (Available, Available) => std::cmp::Ordering::Equal,
            (Available, _) => std::cmp::Ordering::Less,
            (_, Available) => std::cmp::Ordering::Greater,
            (Completed, Completed) => std::cmp::Ordering::Equal,
            (Completed, _) => std::cmp::Ordering::Less,
            (_, Completed) => std::cmp::Ordering::Greater,
            (Disabled, Disabled) => std::cmp::Ordering::Equal,
        }
    }
}

/// Workflow List Widget for displaying and managing workflows
pub struct WorkflowListWidget {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
    
    // Widget state
    workflows: Vec<WorkflowInfo>,
    filtered_workflows: Vec<usize>, // Indices into workflows vec
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

impl WorkflowListWidget {
    /// Create a new WorkflowListWidget
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
            .preferred_size(80, 25);
        
        Self {
            context: WidgetContext::new(WidgetId::from("workflow_list")),
            capabilities,
            size_constraints,
            workflows: Vec::new(),
            filtered_workflows: Vec::new(),
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
    
    /// Set the workflows to display
    pub fn set_workflows(&mut self, workflows: Vec<WorkflowInfo>) {
        self.workflows = workflows;
        self.apply_filter_and_sort();
        self.update_selection();
        self.last_update = Some(std::time::Instant::now());
        self.update_count += 1;
    }
    
    /// Add a workflow to the list
    pub fn add_workflow(&mut self, workflow: WorkflowInfo) {
        self.workflows.push(workflow);
        self.apply_filter_and_sort();
        self.update_selection();
        self.update_count += 1;
    }
    
    /// Remove a workflow by name
    pub fn remove_workflow(&mut self, name: &str) -> bool {
        if let Some(pos) = self.workflows.iter().position(|w| w.name == name) {
            self.workflows.remove(pos);
            self.apply_filter_and_sort();
            self.update_selection();
            self.update_count += 1;
            true
        } else {
            false
        }
    }
    
    /// Update a workflow's status
    pub fn update_workflow_status(&mut self, name: &str, status: WorkflowStatus) -> bool {
        if let Some(workflow) = self.workflows.iter_mut().find(|w| w.name == name) {
            workflow.status = status;
            self.apply_filter_and_sort();
            self.update_selection();
            self.update_count += 1;
            true
        } else {
            false
        }
    }
    
    /// Get the currently selected workflow
    pub fn selected_workflow(&self) -> Option<&WorkflowInfo> {
        self.filtered_workflows
            .get(self.selected_index)
            .and_then(|&index| self.workflows.get(index))
    }
    
    /// Get the currently selected workflow name
    pub fn selected_workflow_name(&self) -> Option<String> {
        self.selected_workflow().map(|w| w.name.clone())
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
        if !self.filtered_workflows.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_workflows.len() - 1
            } else {
                self.selected_index - 1
            };
            self.update_selection();
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.filtered_workflows.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_workflows.len();
            self.update_selection();
        }
    }
    
    /// Move to first item
    pub fn select_first(&mut self) {
        if !self.filtered_workflows.is_empty() {
            self.selected_index = 0;
            self.update_selection();
        }
    }
    
    /// Move to last item
    pub fn select_last(&mut self) {
        if !self.filtered_workflows.is_empty() {
            self.selected_index = self.filtered_workflows.len() - 1;
            self.update_selection();
        }
    }
    
    /// Apply current filter and sort order
    fn apply_filter_and_sort(&mut self) {
        // Apply filter
        self.filtered_workflows = self.workflows
            .iter()
            .enumerate()
            .filter(|(_, workflow)| workflow.matches_filter(&self.filter))
            .map(|(i, _)| i)
            .collect();
        
        // Apply sort
        self.filtered_workflows.sort_by(|&a, &b| {
            let workflow_a = &self.workflows[a];
            let workflow_b = &self.workflows[b];
            
            match self.sort_order {
                SortOrder::NameAsc => workflow_a.name.cmp(&workflow_b.name),
                SortOrder::NameDesc => workflow_b.name.cmp(&workflow_a.name),
                SortOrder::DateAsc => workflow_a.last_execution.cmp(&workflow_b.last_execution),
                SortOrder::DateDesc => workflow_b.last_execution.cmp(&workflow_a.last_execution),
                SortOrder::StatusAsc => workflow_a.status.cmp(&workflow_b.status),
                SortOrder::StatusDesc => workflow_b.status.cmp(&workflow_a.status),
                SortOrder::Custom(ref custom) => {
                    match custom.as_str() {
                        "execution_count_asc" => workflow_a.execution_count.cmp(&workflow_b.execution_count),
                        "execution_count_desc" => workflow_b.execution_count.cmp(&workflow_a.execution_count),
                        "node_count_asc" => workflow_a.node_count.cmp(&workflow_b.node_count),
                        "node_count_desc" => workflow_b.node_count.cmp(&workflow_a.node_count),
                        "version_asc" => workflow_a.version.cmp(&workflow_b.version),
                        "version_desc" => workflow_b.version.cmp(&workflow_a.version),
                        "success_rate_asc" => workflow_a.success_rate.partial_cmp(&workflow_b.success_rate).unwrap_or(std::cmp::Ordering::Equal),
                        "success_rate_desc" => workflow_b.success_rate.partial_cmp(&workflow_a.success_rate).unwrap_or(std::cmp::Ordering::Equal),
                        _ => std::cmp::Ordering::Equal,
                    }
                }
            }
        });
    }
    
    /// Update selection state
    fn update_selection(&mut self) {
        if self.selected_index >= self.filtered_workflows.len() && !self.filtered_workflows.is_empty() {
            self.selected_index = self.filtered_workflows.len() - 1;
        }
        
        self.list_state.select(if self.filtered_workflows.is_empty() {
            None
        } else {
            Some(self.selected_index)
        });
        
        // Update scrollbar state
        self.scroll_state = self.scroll_state.content_length(self.filtered_workflows.len());
        self.scroll_state = self.scroll_state.position(self.selected_index);
    }
    
    /// Highlight search terms in text (static version to avoid borrowing issues)
    fn highlight_text_static<'a>(text: &'a str, search_term: &str, base_style: Style) -> Vec<Span<'a>> {
        if search_term.is_empty() {
            return vec![Span::styled(text, base_style)];
        }
        
        let search_lower = search_term.to_lowercase();
        let text_lower = text.to_lowercase();
        
        let mut spans = Vec::new();
        let mut last_end = 0;
        
        // Find all occurrences of the search term
        while let Some(start) = text_lower[last_end..].find(&search_lower) {
            let actual_start = last_end + start;
            let actual_end = actual_start + search_term.len();
            
            // Add text before the match
            if actual_start > last_end {
                spans.push(Span::styled(&text[last_end..actual_start], base_style));
            }
            
            // Add the highlighted match
            spans.push(Span::styled(
                &text[actual_start..actual_end],
                base_style.add_modifier(Modifier::REVERSED)
            ));
            
            last_end = actual_end;
        }
        
        // Add remaining text
        if last_end < text.len() {
            spans.push(Span::styled(&text[last_end..], base_style));
        }
        
        if spans.is_empty() {
            vec![Span::styled(text, base_style)]
        } else {
            spans
        }
    }
    
    /// Highlight search terms in text
    fn highlight_text<'a>(&self, text: &'a str, search_term: &str, base_style: Style) -> Vec<Span<'a>> {
        Self::highlight_text_static(text, search_term, base_style)
    }
    
    /// Render the workflow list
    fn render_workflow_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Collect the filter and other needed values before creating the closure
        let filter = self.filter.clone();
        let search_mode = self.search_mode;
        
        let items: Vec<ListItem> = self.filtered_workflows
            .iter()
            .map(|&index| {
                let workflow = &self.workflows[index];
                
                let status_style = Style::default().fg(workflow.status_color());
                
                let last_exec = workflow.last_execution
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "从未执行".to_string());
                
                // Highlight search terms in the workflow name
                let name_spans = if !filter.is_empty() && !filter.starts_with("status:") && !filter.starts_with("tag:") && !filter.starts_with("version:") && !filter.starts_with("nodes") {
                    Self::highlight_text_static(&workflow.name, &filter, Style::default().add_modifier(Modifier::BOLD))
                } else {
                    vec![Span::styled(&workflow.name, Style::default().add_modifier(Modifier::BOLD))]
                };
                
                let mut first_line = vec![
                    Span::styled(workflow.status_symbol(), status_style),
                    Span::raw(" "),
                ];
                first_line.extend(name_spans);
                first_line.extend(vec![
                    Span::raw(" v"),
                    Span::styled(&workflow.version, Style::default().fg(Color::Gray)),
                ]);
                
                let content = vec![
                    Line::from(first_line),
                    Line::from(vec![
                        Span::raw("  最后执行: "),
                        Span::styled(last_exec, Style::default().fg(Color::Gray)),
                        Span::raw(" | 执行次数: "),
                        Span::styled(workflow.execution_count.to_string(), Style::default().fg(Color::Cyan)),
                        Span::raw(" | 节点数: "),
                        Span::styled(workflow.node_count.to_string(), Style::default().fg(Color::Magenta)),
                    ]),
                ];
                
                ListItem::new(content)
            })
            .collect();
        
        let title = if search_mode {
            format!("工作流列表 - 搜索: {}", filter)
        } else if !filter.is_empty() {
            format!("工作流列表 - 过滤: {} ({}/{})", filter, self.filtered_workflows.len(), self.workflows.len())
        } else {
            format!("工作流列表 ({}/{})", self.filtered_workflows.len(), self.workflows.len())
        };
        
        let list = List::new(items)
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
        if self.filtered_workflows.len() > area.height as usize - 2 {
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
        
        // Show filter help if in search mode
        if self.search_mode && area.height > 10 {
            let help_area = Rect {
                x: area.x + 2,
                y: area.bottom() - 6,
                width: area.width.saturating_sub(4),
                height: 4,
            };
            
            let help_text = vec![
                Line::from("过滤语法: status:运行中, tag:数据, version:1.0, nodes:5, nodes>10, nodes<3"),
                Line::from("按 Enter 确认, Esc 取消"),
            ];
            
            let help_paragraph = Paragraph::new(help_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("过滤帮助")
                    .border_style(theme.styles.widget_border))
                .style(theme.styles.info);
            
            frame.render_widget(help_paragraph, help_area);
        }
    }
    
    /// Render the workflow details panel
    fn render_workflow_details(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(workflow) = self.selected_workflow() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),  // Basic info
                    Constraint::Min(0),     // Description and tags
                ])
                .split(area);
            
            // Basic information
            let info_lines = vec![
                Line::from(vec![
                    Span::raw("名称: "),
                    Span::styled(&workflow.name, Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::raw("版本: "),
                    Span::styled(&workflow.version, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("状态: "),
                    Span::styled(workflow.status_symbol(), Style::default().fg(workflow.status_color())),
                    Span::raw(" "),
                    Span::styled(workflow.status_string(), Style::default().fg(workflow.status_color())),
                ]),
                Line::from(vec![
                    Span::raw("节点数: "),
                    Span::styled(workflow.node_count.to_string(), Style::default().fg(Color::Magenta)),
                ]),
                Line::from(vec![
                    Span::raw("执行次数: "),
                    Span::styled(workflow.execution_count.to_string(), Style::default().fg(Color::Yellow)),
                ]),
                if let Some(success_rate) = workflow.success_rate {
                    Line::from(vec![
                        Span::raw("成功率: "),
                        Span::styled(format!("{:.1}%", success_rate * 100.0), Style::default().fg(Color::Green)),
                    ])
                } else {
                    Line::from(vec![Span::raw("成功率: N/A")])
                },
            ];
            
            let info_paragraph = Paragraph::new(info_lines)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("详细信息")
                    .border_style(theme.styles.widget_border))
                .style(theme.styles.info);
            
            frame.render_widget(info_paragraph, chunks[0]);
            
            // Description and tags
            let mut desc_lines = Vec::new();
            
            if let Some(description) = &workflow.description {
                desc_lines.push(Line::from(vec![
                    Span::styled("描述:", Style::default().add_modifier(Modifier::BOLD)),
                ]));
                desc_lines.push(Line::from(description.as_str()));
                desc_lines.push(Line::from(""));
            }
            
            if !workflow.tags.is_empty() {
                desc_lines.push(Line::from(vec![
                    Span::styled("标签:", Style::default().add_modifier(Modifier::BOLD)),
                ]));
                let tags_line = workflow.tags.iter()
                    .map(|tag| Span::styled(format!("#{} ", tag), Style::default().fg(Color::Blue)))
                    .collect::<Vec<_>>();
                desc_lines.push(Line::from(tags_line));
            }
            
            if let Some(last_exec) = workflow.last_execution {
                desc_lines.push(Line::from(""));
                desc_lines.push(Line::from(vec![
                    Span::styled("最后执行:", Style::default().add_modifier(Modifier::BOLD)),
                ]));
                desc_lines.push(Line::from(last_exec.format("%Y-%m-%d %H:%M:%S UTC").to_string()));
            }
            
            let desc_paragraph = Paragraph::new(desc_lines)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("描述")
                    .border_style(theme.styles.widget_border))
                .style(theme.styles.info)
                .wrap(Wrap { trim: true });
            
            frame.render_widget(desc_paragraph, chunks[1]);
        } else {
            let no_selection = Paragraph::new("未选择工作流")
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("详细信息")
                    .border_style(theme.styles.widget_border))
                .style(theme.styles.info);
            
            frame.render_widget(no_selection, area);
        }
    }
    
    /// Render help information
    fn render_help(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let sort_description = match &self.sort_order {
            SortOrder::NameAsc => "名称 ↑",
            SortOrder::NameDesc => "名称 ↓",
            SortOrder::DateAsc => "日期 ↑",
            SortOrder::DateDesc => "日期 ↓",
            SortOrder::StatusAsc => "状态 ↑",
            SortOrder::StatusDesc => "状态 ↓",
            SortOrder::Custom(custom) => match custom.as_str() {
                "execution_count_asc" => "执行次数 ↑",
                "execution_count_desc" => "执行次数 ↓",
                "node_count_asc" => "节点数 ↑",
                "node_count_desc" => "节点数 ↓",
                "success_rate_asc" => "成功率 ↑",
                "success_rate_desc" => "成功率 ↓",
                _ => "自定义",
            }
        };
        
        let help_text = vec![
            Line::from("键盘快捷键:"),
            Line::from(""),
            Line::from("  ↑/k      - 上一个工作流"),
            Line::from("  ↓/j      - 下一个工作流"),
            Line::from("  Home/g   - 第一个工作流"),
            Line::from("  End/G    - 最后一个工作流"),
            Line::from("  Enter    - 执行选中的工作流"),
            Line::from("  Space    - 显示/隐藏详情"),
            Line::from("  /        - 搜索/过滤工作流"),
            Line::from("  f        - 快速过滤"),
            Line::from(format!("  s        - 排序 (当前: {})", sort_description)),
            Line::from("  c        - 清除过滤器"),
            Line::from("  r        - 刷新列表"),
            Line::from("  ?        - 显示/隐藏帮助"),
            Line::from("  Esc      - 返回上级"),
            Line::from("  Ctrl+Q   - 退出"),
            Line::from(""),
            Line::from("过滤语法:"),
            Line::from("  status:运行中  - 按状态过滤"),
            Line::from("  tag:数据      - 按标签过滤"),
            Line::from("  version:1.0   - 按版本过滤"),
            Line::from("  nodes:5       - 按节点数过滤"),
            Line::from("  nodes>10      - 节点数大于10"),
            Line::from("  nodes<3       - 节点数小于3"),
        ];
        
        let help_paragraph = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("帮助")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info);
        
        frame.render_widget(help_paragraph, area);
    }
}

#[async_trait]
impl Widget for WorkflowListWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }
    
    fn title(&self) -> &str {
        "工作流列表"
    }
    
    fn description(&self) -> Option<&str> {
        Some("显示和管理可用的工作流")
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
        UpdateFrequency::Interval(Duration::from_secs(5)) // Update every 5 seconds
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
            
            self.render_workflow_list(frame, chunks[0], theme);
            self.render_workflow_details(frame, chunks[1], theme);
        } else {
            // Full width list
            self.render_workflow_list(frame, area, theme);
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
        // In a real implementation, this would fetch updated workflow data
        // For now, we just track that an update occurred
        self.update_count += 1;
        Ok(())
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("↑/k", "上一个工作流"),
            ("↓/j", "下一个工作流"),
            ("Home/g", "第一个工作流"),
            ("End/G", "最后一个工作流"),
            ("Enter", "执行工作流"),
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

impl WorkflowListWidget {
    /// Handle keyboard events
    async fn handle_key_event(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        if self.search_mode {
            return self.handle_search_key_event(key).await;
        }
        
        match key.code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                Ok(Some(Action::SelectPrevious))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Ok(Some(Action::SelectNext))
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.select_first();
                Ok(Some(Action::SelectFirst))
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.select_last();
                Ok(Some(Action::SelectLast))
            }
            
            // Actions
            KeyCode::Enter => {
                if let Some(workflow_name) = self.selected_workflow_name() {
                    Ok(Some(Action::ExecuteWorkflow(workflow_name)))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char(' ') => {
                self.toggle_details();
                Ok(Some(Action::ToggleDetails))
            }
            
            // Search and filter
            KeyCode::Char('/') => {
                self.enter_search_mode();
                Ok(Some(Action::StartInput(InputMode::Search)))
            }
            KeyCode::Char('f') => {
                Ok(Some(Action::StartInput(InputMode::Filter)))
            }
            KeyCode::Char('c') => {
                self.clear_filter();
                Ok(Some(Action::ClearFilter))
            }
            
            // Sorting
            KeyCode::Char('s') => {
                let next_sort = match self.sort_order {
                    SortOrder::NameAsc => SortOrder::NameDesc,
                    SortOrder::NameDesc => SortOrder::DateAsc,
                    SortOrder::DateAsc => SortOrder::DateDesc,
                    SortOrder::DateDesc => SortOrder::StatusAsc,
                    SortOrder::StatusAsc => SortOrder::StatusDesc,
                    SortOrder::StatusDesc => SortOrder::Custom("execution_count_desc".to_string()),
                    SortOrder::Custom(ref custom) => {
                        match custom.as_str() {
                            "execution_count_desc" => SortOrder::Custom("execution_count_asc".to_string()),
                            "execution_count_asc" => SortOrder::Custom("node_count_desc".to_string()),
                            "node_count_desc" => SortOrder::Custom("node_count_asc".to_string()),
                            "node_count_asc" => SortOrder::Custom("success_rate_desc".to_string()),
                            "success_rate_desc" => SortOrder::Custom("success_rate_asc".to_string()),
                            "success_rate_asc" => SortOrder::NameAsc,
                            _ => SortOrder::NameAsc,
                        }
                    }
                };
                self.set_sort_order(next_sort.clone());
                Ok(Some(Action::Sort(next_sort)))
            }
            
            // UI toggles
            KeyCode::Char('?') => {
                self.toggle_help();
                Ok(Some(Action::ToggleHelp))
            }
            KeyCode::Char('r') => {
                Ok(Some(Action::Refresh))
            }
            
            // Navigation
            KeyCode::Esc => {
                Ok(Some(Action::GoBack))
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Ok(Some(Action::Quit))
            }
            
            // Workflow management
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(workflow_name) = self.selected_workflow_name() {
                    Ok(Some(Action::ShowWorkflowDetails(workflow_name)))
                } else {
                    Ok(None)
                }
            }
            
            _ => Ok(None),
        }
    }
    
    /// Handle keyboard events in search mode
    async fn handle_search_key_event(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        match key.code {
            KeyCode::Esc => {
                self.exit_search_mode();
                Ok(Some(Action::CancelInput))
            }
            KeyCode::Enter => {
                self.exit_search_mode();
                Ok(Some(Action::ConfirmInput(self.filter.clone())))
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.apply_filter_and_sort();
                self.update_selection();
                Ok(Some(Action::Search(self.filter.clone())))
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.apply_filter_and_sort();
                self.update_selection();
                Ok(Some(Action::Search(self.filter.clone())))
            }
            _ => Ok(None),
        }
    }
}

impl Default for WorkflowListWidget {
    fn default() -> Self {
        Self::new()
    }
}