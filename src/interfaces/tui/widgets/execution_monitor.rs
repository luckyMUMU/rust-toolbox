//! Execution Monitor Widget Implementation
//! 
//! This widget displays real-time execution status and monitoring information for workflows.

use crate::interfaces::tui::{
    Theme
};
use crate::interfaces::tui::widget::{Widget, WidgetId, WidgetContext, WidgetCapabilities, SizeConstraints, UpdateFrequency, WidgetError};
use crate::interfaces::tui::action::Action;
use crate::core::ExecutionStatus;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Gauge,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    symbols,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// Execution information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub id: String,
    pub workflow_name: String,
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: f64,
    pub current_node: Option<String>,
    pub total_nodes: usize,
    pub completed_nodes: usize,
    pub failed_nodes: usize,
}

impl ExecutionInfo {
    /// Get status text for display
    pub fn status_text(&self) -> &str {
        match self.status {
            ExecutionStatus::Pending => "等待中",
            ExecutionStatus::Running => "运行中",
            ExecutionStatus::Completed => "已完成",
            ExecutionStatus::Failed => "失败",
            ExecutionStatus::Cancelled => "已取消",
            ExecutionStatus::Paused => "已暂停",
            ExecutionStatus::Timeout => "超时",
        }
    }
    
    /// Get status symbol
    pub fn status_symbol(&self) -> &str {
        match self.status {
            ExecutionStatus::Pending => "⏳",
            ExecutionStatus::Running => "▶",
            ExecutionStatus::Completed => "✅",
            ExecutionStatus::Failed => "❌",
            ExecutionStatus::Cancelled => "⏹",
            ExecutionStatus::Paused => "⏸",
            ExecutionStatus::Timeout => "⏰",
        }
    }
}

/// Execution Monitor Widget implementation
pub struct ExecutionMonitorWidget {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
    executions: Vec<ExecutionInfo>,
    selected_index: usize,
    list_state: ListState,
    scroll_state: ScrollbarState,
    show_details: bool,
    show_help: bool,
    auto_refresh: bool,
}

impl ExecutionMonitorWidget {
    /// Create a new execution monitor widget
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
            configurable: false,
        };
        
        let size_constraints = SizeConstraints::new()
            .min_size(40, 10)
            .preferred_size(80, 20);
        
        Self {
            context: WidgetContext::new(WidgetId::from("execution_monitor")),
            capabilities,
            size_constraints,
            executions: Vec::new(),
            selected_index: 0,
            list_state,
            scroll_state: ScrollbarState::default(),
            show_details: false,
            show_help: false,
            auto_refresh: true,
        }
    }
    
    /// Set the executions to display
    pub fn set_executions(&mut self, executions: Vec<ExecutionInfo>) {
        self.executions = executions;
        self.update_selection();
    }
    
    /// Get the currently selected execution
    pub fn selected_execution(&self) -> Option<&ExecutionInfo> {
        self.executions.get(self.selected_index)
    }
    
    /// Move selection up
    pub fn select_previous(&mut self) {
        if !self.executions.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.executions.len() - 1
            } else {
                self.selected_index - 1
            };
            self.update_selection();
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.executions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.executions.len();
            self.update_selection();
        }
    }
    
    /// Move to first item
    pub fn select_first(&mut self) {
        if !self.executions.is_empty() {
            self.selected_index = 0;
            self.update_selection();
        }
    }
    
    /// Move to last item
    pub fn select_last(&mut self) {
        if !self.executions.is_empty() {
            self.selected_index = self.executions.len() - 1;
            self.update_selection();
        }
    }
    
    /// Toggle details view
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
    
    /// Toggle help view
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    /// Update selection state
    fn update_selection(&mut self) {
        if self.selected_index >= self.executions.len() && !self.executions.is_empty() {
            self.selected_index = self.executions.len() - 1;
        }
        
        self.list_state.select(if self.executions.is_empty() {
            None
        } else {
            Some(self.selected_index)
        });
        
        // Update scrollbar state
        self.scroll_state = self.scroll_state.content_length(self.executions.len());
        self.scroll_state = self.scroll_state.position(self.selected_index);
    }
    
    /// Render the execution list
    fn render_execution_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let items: Vec<ListItem> = self.executions
            .iter()
            .map(|execution| {
                let status_style = match execution.status {
                    ExecutionStatus::Running => Style::default().fg(theme.colors.success),
                    ExecutionStatus::Completed => Style::default().fg(theme.colors.info),
                    ExecutionStatus::Failed => Style::default().fg(theme.colors.error),
                    ExecutionStatus::Cancelled => Style::default().fg(theme.colors.text_secondary),
                    ExecutionStatus::Pending => Style::default().fg(theme.colors.warning),
                    ExecutionStatus::Paused => Style::default().fg(theme.colors.warning),
                    ExecutionStatus::Timeout => Style::default().fg(theme.colors.error),
                };
                
                let progress_bar = if execution.progress > 0.0 {
                    format!(" [{:>3.0}%]", execution.progress * 100.0)
                } else {
                    String::new()
                };
                
                let line = Line::from(vec![
                    Span::styled(execution.status_symbol(), status_style),
                    Span::raw(" "),
                    Span::styled(&execution.workflow_name, theme.styles.text_normal),
                    Span::styled(progress_bar, theme.styles.text_dimmed),
                    Span::raw(" - "),
                    Span::styled(execution.status_text(), status_style),
                ]);
                
                ListItem::new(line)
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("执行监控")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.text_normal)
            .highlight_style(theme.styles.list_item_selected)
            .highlight_symbol("▶ ");
        
        frame.render_stateful_widget(list, area, &mut self.list_state);
        
        // Render scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        
        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
            &mut self.scroll_state,
        );
    }
    
    /// Render execution details
    fn render_execution_details(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(execution) = self.selected_execution() {
            let details_lines = vec![
                Line::from(vec![
                    Span::styled("工作流: ", theme.styles.text_dimmed),
                    Span::styled(&execution.workflow_name, theme.styles.text_normal),
                ]),
                Line::from(vec![
                    Span::styled("执行ID: ", theme.styles.text_dimmed),
                    Span::styled(&execution.execution_id, theme.styles.text_normal),
                ]),
                Line::from(vec![
                    Span::styled("状态: ", theme.styles.text_dimmed),
                    Span::styled(execution.status_text(), match execution.status {
                        ExecutionStatus::Running => Style::default().fg(theme.colors.success),
                        ExecutionStatus::Completed => Style::default().fg(theme.colors.info),
                        ExecutionStatus::Failed => Style::default().fg(theme.colors.error),
                        ExecutionStatus::Cancelled => Style::default().fg(theme.colors.text_secondary),
                        ExecutionStatus::Pending => Style::default().fg(theme.colors.warning),
                        ExecutionStatus::Paused => Style::default().fg(theme.colors.warning),
                        ExecutionStatus::Timeout => Style::default().fg(theme.colors.error),
                    }),
                ]),
                Line::from(vec![
                    Span::styled("进度: ", theme.styles.text_dimmed),
                    Span::styled(format!("{:.1}%", execution.progress * 100.0), theme.styles.text_normal),
                ]),
                Line::from(vec![
                    Span::styled("开始时间: ", theme.styles.text_dimmed),
                    Span::styled(execution.started_at.format("%Y-%m-%d %H:%M:%S").to_string(), theme.styles.text_normal),
                ]),
            ];
            
            let details = Paragraph::new(details_lines)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("执行详情")
                    .border_style(theme.styles.widget_border))
                .style(theme.styles.text_normal)
                .wrap(Wrap { trim: true });
            
            frame.render_widget(details, area);
        } else {
            let no_selection = Paragraph::new("未选择执行")
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("执行详情")
                    .border_style(theme.styles.widget_border))
                .style(theme.styles.text_dimmed)
                .alignment(Alignment::Center);
            
            frame.render_widget(no_selection, area);
        }
    }
    
    /// Render help overlay
    fn render_help(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let help_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("执行监控 - 帮助", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from("导航:"),
            Line::from("  ↑/k        上一个执行"),
            Line::from("  ↓/j        下一个执行"),
            Line::from("  Home/g     第一个执行"),
            Line::from("  End/G      最后一个执行"),
            Line::from(""),
            Line::from("视图:"),
            Line::from("  Space      切换详情面板"),
            Line::from("  a          切换自动刷新"),
            Line::from("  F5         手动刷新"),
            Line::from(""),
            Line::from("其他:"),
            Line::from("  Enter      显示详情"),
            Line::from("  ?          显示/隐藏帮助"),
            Line::from("  Esc        返回"),
            Line::from("  Ctrl+Q     退出"),
            Line::from(""),
            Line::from("按任意键关闭帮助"),
        ];
        
        let help = Paragraph::new(help_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("帮助")
                .border_style(Style::default().fg(Color::Yellow))
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(theme.styles.info)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(help, area);
    }
    
    /// Handle keyboard events
    async fn handle_key_event(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
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
            
            // View toggles
            KeyCode::Char(' ') => {
                self.toggle_details();
                Ok(Some(Action::ToggleDetails))
            }
            KeyCode::Char('?') => {
                self.toggle_help();
                Ok(Some(Action::ToggleHelp))
            }
            KeyCode::Char('a') => {
                self.auto_refresh = !self.auto_refresh;
                Ok(Some(Action::Custom("toggle_auto_refresh".to_string(), serde_json::Value::Bool(self.auto_refresh))))
            }
            
            // Show execution details
            KeyCode::Enter => {
                if let Some(execution) = self.selected_execution() {
                    Ok(Some(Action::ShowWorkflowDetails(execution.execution_id.clone())))
                } else {
                    Ok(None)
                }
            }
            
            // Refresh
            KeyCode::F(5) => {
                Ok(Some(Action::Refresh))
            }
            
            // Navigation
            KeyCode::Esc => {
                if self.show_help {
                    self.show_help = false;
                    Ok(None)
                } else {
                    Ok(Some(Action::GoBack))
                }
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Ok(Some(Action::Quit))
            }
            
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl Widget for ExecutionMonitorWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }
    
    fn title(&self) -> &str {
        "执行监控"
    }
    
    fn description(&self) -> Option<&str> {
        Some("实时监控工作流执行状态")
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
        if !self.auto_refresh {
            return UpdateFrequency::Never;
        }
        
        if !self.executions.is_empty() {
            UpdateFrequency::Interval(Duration::from_secs(2))
        } else {
            UpdateFrequency::Interval(Duration::from_secs(5))
        }
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
        } else if self.show_details && !self.executions.is_empty() {
            // Split view: list on left, details on right
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            
            self.render_execution_list(frame, chunks[0], theme);
            self.render_execution_details(frame, chunks[1], theme);
        } else {
            // Full width list
            self.render_execution_list(frame, area, theme);
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
        Ok(())
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("↑/k", "上一个执行"),
            ("↓/j", "下一个执行"),
            ("Home/g", "第一个执行"),
            ("End/G", "最后一个执行"),
            ("Enter", "显示详情"),
            ("Space", "切换详情"),
            ("a", "自动刷新"),
            ("F5", "手动刷新"),
            ("?", "帮助"),
            ("Esc", "返回"),
            ("Ctrl+Q", "退出"),
        ]
    }
}

impl Default for ExecutionMonitorWidget {
    fn default() -> Self {
        Self::new()
    }
}