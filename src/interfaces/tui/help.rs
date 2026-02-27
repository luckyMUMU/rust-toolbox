//! Help system for TUI interface
//!
//! This module provides dynamic help and shortcut information for the TUI interface.

use crate::error::Result;
use crate::interfaces::tui::{
    action::{Action, ViewType},
    theme::Theme,
    widget::WidgetId,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Help system manager
pub struct HelpSystem {
    /// Global shortcuts that work in any context
    global_shortcuts: Vec<ShortcutInfo>,
    /// View-specific shortcuts
    view_shortcuts: HashMap<ViewType, Vec<ShortcutInfo>>,
    /// Widget-specific shortcuts
    widget_shortcuts: HashMap<WidgetId, Vec<ShortcutInfo>>,
    /// Context-sensitive help content
    context_help: HashMap<String, HelpContent>,
    /// Whether help is currently visible
    help_visible: bool,
    /// Current help context
    current_context: Option<String>,
    /// Help display mode
    display_mode: HelpDisplayMode,
    /// Last help update time
    last_update: Option<Instant>,
    /// Help animation state
    animation_state: HelpAnimationState,
}

/// Shortcut information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutInfo {
    /// Key combination (e.g., "Ctrl+Q", "F1", "Tab")
    pub key: String,
    /// Description of what the shortcut does
    pub description: String,
    /// Category for grouping shortcuts
    pub category: String,
    /// Whether this shortcut is available in current context
    pub available: bool,
    /// Priority for display order (higher = more important)
    pub priority: i32,
    /// Whether this is a primary shortcut (highlighted)
    pub primary: bool,
}

/// Help content for context-sensitive help
#[derive(Debug, Clone)]
pub struct HelpContent {
    /// Title of the help section
    pub title: String,
    /// Main help text
    pub content: String,
    /// Additional tips
    pub tips: Vec<String>,
    /// Related shortcuts
    pub shortcuts: Vec<ShortcutInfo>,
    /// See also references
    pub see_also: Vec<String>,
}

/// Help display modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpDisplayMode {
    /// Show help as overlay popup
    Overlay,
    /// Show help in sidebar
    Sidebar,
    /// Show help as bottom panel
    BottomPanel,
    /// Show help as tooltip
    Tooltip,
    /// Show minimal shortcuts in status bar
    StatusBar,
}

/// Help animation state
#[derive(Debug, Clone)]
pub struct HelpAnimationState {
    /// Whether help is animating in/out
    pub animating: bool,
    /// Animation start time
    pub start_time: Option<Instant>,
    /// Animation duration
    pub duration: Duration,
    /// Animation progress (0.0 to 1.0)
    pub progress: f32,
}

/// Help popup configuration
#[derive(Debug, Clone)]
pub struct HelpPopupConfig {
    /// Popup width as percentage of screen
    pub width_percent: u16,
    /// Popup height as percentage of screen
    pub height_percent: u16,
    /// Whether to center the popup
    pub centered: bool,
    /// Whether to show close button
    pub show_close_button: bool,
    /// Whether to dim background
    pub dim_background: bool,
    /// Animation duration
    pub animation_duration: Duration,
}

impl HelpSystem {
    /// Create a new help system
    pub fn new() -> Self {
        let mut system = Self {
            global_shortcuts: Vec::new(),
            view_shortcuts: HashMap::new(),
            widget_shortcuts: HashMap::new(),
            context_help: HashMap::new(),
            help_visible: false,
            current_context: None,
            display_mode: HelpDisplayMode::Overlay,
            last_update: None,
            animation_state: HelpAnimationState {
                animating: false,
                start_time: None,
                duration: Duration::from_millis(300),
                progress: 0.0,
            },
        };

        // Initialize with default shortcuts
        system.setup_default_shortcuts();
        system.setup_default_help_content();

        system
    }

    /// Setup default global shortcuts
    fn setup_default_shortcuts(&mut self) {
        self.global_shortcuts = vec![
            ShortcutInfo {
                key: "Ctrl+Q".to_string(),
                description: "退出应用程序".to_string(),
                category: "应用程序".to_string(),
                available: true,
                priority: 100,
                primary: true,
            },
            ShortcutInfo {
                key: "F12".to_string(),
                description: "刷新数据".to_string(),
                category: "应用程序".to_string(),
                available: true,
                priority: 80,
                primary: false,
            },
            ShortcutInfo {
                key: "?".to_string(),
                description: "显示/隐藏帮助".to_string(),
                category: "帮助".to_string(),
                available: true,
                priority: 90,
                primary: true,
            },
            ShortcutInfo {
                key: "Tab".to_string(),
                description: "下一个元素".to_string(),
                category: "导航".to_string(),
                available: true,
                priority: 70,
                primary: false,
            },
            ShortcutInfo {
                key: "Shift+Tab".to_string(),
                description: "上一个元素".to_string(),
                category: "导航".to_string(),
                available: true,
                priority: 70,
                primary: false,
            },
            ShortcutInfo {
                key: "Esc".to_string(),
                description: "返回/取消".to_string(),
                category: "导航".to_string(),
                available: true,
                priority: 85,
                primary: true,
            },
        ];

        // Setup view-specific shortcuts
        self.setup_view_shortcuts();
    }

    /// Setup view-specific shortcuts
    fn setup_view_shortcuts(&mut self) {
        // Function key navigation
        let navigation_shortcuts = vec![
            ShortcutInfo {
                key: "F1".to_string(),
                description: "工作流列表".to_string(),
                category: "视图切换".to_string(),
                available: true,
                priority: 60,
                primary: false,
            },
            ShortcutInfo {
                key: "F2".to_string(),
                description: "执行监控".to_string(),
                category: "视图切换".to_string(),
                available: true,
                priority: 60,
                primary: false,
            },
            ShortcutInfo {
                key: "F3".to_string(),
                description: "工具管理".to_string(),
                category: "视图切换".to_string(),
                available: true,
                priority: 60,
                primary: false,
            },
            ShortcutInfo {
                key: "F4".to_string(),
                description: "插件管理".to_string(),
                category: "视图切换".to_string(),
                available: true,
                priority: 60,
                primary: false,
            },
            ShortcutInfo {
                key: "F5".to_string(),
                description: "系统状态".to_string(),
                category: "视图切换".to_string(),
                available: true,
                priority: 60,
                primary: false,
            },
            ShortcutInfo {
                key: "F6".to_string(),
                description: "日志查看器".to_string(),
                category: "视图切换".to_string(),
                available: true,
                priority: 60,
                primary: false,
            },
        ];

        // Add navigation shortcuts to all views
        for view in [
            ViewType::WorkflowList,
            ViewType::ExecutionMonitor,
            ViewType::ToolManager,
            ViewType::PluginManager,
            ViewType::SystemStatus,
            ViewType::LogViewer,
        ] {
            self.view_shortcuts
                .insert(view, navigation_shortcuts.clone());
        }

        // Add view-specific shortcuts
        self.view_shortcuts
            .get_mut(&ViewType::WorkflowList)
            .unwrap()
            .extend(vec![
                ShortcutInfo {
                    key: "Enter".to_string(),
                    description: "执行选中的工作流".to_string(),
                    category: "工作流操作".to_string(),
                    available: true,
                    priority: 95,
                    primary: true,
                },
                ShortcutInfo {
                    key: "↑/↓".to_string(),
                    description: "选择工作流".to_string(),
                    category: "工作流操作".to_string(),
                    available: true,
                    priority: 75,
                    primary: false,
                },
                ShortcutInfo {
                    key: "d".to_string(),
                    description: "切换详情视图".to_string(),
                    category: "工作流操作".to_string(),
                    available: true,
                    priority: 65,
                    primary: false,
                },
                ShortcutInfo {
                    key: "/".to_string(),
                    description: "搜索工作流".to_string(),
                    category: "工作流操作".to_string(),
                    available: true,
                    priority: 70,
                    primary: false,
                },
            ]);

        self.view_shortcuts
            .get_mut(&ViewType::ExecutionMonitor)
            .unwrap()
            .extend(vec![
                ShortcutInfo {
                    key: "Space".to_string(),
                    description: "暂停/恢复执行".to_string(),
                    category: "执行控制".to_string(),
                    available: true,
                    priority: 95,
                    primary: true,
                },
                ShortcutInfo {
                    key: "s".to_string(),
                    description: "停止执行".to_string(),
                    category: "执行控制".to_string(),
                    available: true,
                    priority: 90,
                    primary: true,
                },
                ShortcutInfo {
                    key: "c".to_string(),
                    description: "取消执行".to_string(),
                    category: "执行控制".to_string(),
                    available: true,
                    priority: 85,
                    primary: false,
                },
                ShortcutInfo {
                    key: "d".to_string(),
                    description: "切换依赖关系图".to_string(),
                    category: "显示选项".to_string(),
                    available: true,
                    priority: 60,
                    primary: false,
                },
            ]);

        self.view_shortcuts
            .get_mut(&ViewType::LogViewer)
            .unwrap()
            .extend(vec![
                ShortcutInfo {
                    key: "1-5".to_string(),
                    description: "按级别过滤日志".to_string(),
                    category: "日志操作".to_string(),
                    available: true,
                    priority: 80,
                    primary: false,
                },
                ShortcutInfo {
                    key: "c".to_string(),
                    description: "清除日志".to_string(),
                    category: "日志操作".to_string(),
                    available: true,
                    priority: 70,
                    primary: false,
                },
                ShortcutInfo {
                    key: "e".to_string(),
                    description: "导出日志".to_string(),
                    category: "日志操作".to_string(),
                    available: true,
                    priority: 65,
                    primary: false,
                },
                ShortcutInfo {
                    key: "/".to_string(),
                    description: "搜索日志".to_string(),
                    category: "日志操作".to_string(),
                    available: true,
                    priority: 75,
                    primary: false,
                },
            ]);
    }

    /// Setup default help content
    fn setup_default_help_content(&mut self) {
        self.context_help.insert(
            "getting_started".to_string(),
            HelpContent {
                title: "快速开始".to_string(),
                content: "欢迎使用工作流工具包TUI界面！\n\n使用F1-F6键在不同视图间切换，Tab键在界面元素间导航，Esc键返回上一级。".to_string(),
                tips: vec![
                    "按?键随时显示帮助信息".to_string(),
                    "使用Ctrl+Q快速退出应用程序".to_string(),
                    "F12键可以刷新所有数据".to_string(),
                ],
                shortcuts: vec![],
                see_also: vec!["navigation".to_string(), "shortcuts".to_string()],
            },
        );

        self.context_help.insert(
            "navigation".to_string(),
            HelpContent {
                title: "导航指南".to_string(),
                content: "TUI界面支持完整的键盘导航：\n\n• 使用Tab和Shift+Tab在元素间切换\n• 使用方向键进行精确导航\n• 使用F1-F6快速切换视图\n• 使用Esc键返回或取消操作".to_string(),
                tips: vec![
                    "Home键跳转到第一个元素".to_string(),
                    "End键跳转到最后一个元素".to_string(),
                    "在列表中使用Page Up/Down快速滚动".to_string(),
                ],
                shortcuts: vec![],
                see_also: vec!["shortcuts".to_string()],
            },
        );

        self.context_help.insert(
            "shortcuts".to_string(),
            HelpContent {
                title: "快捷键参考".to_string(),
                content: "所有可用的键盘快捷键按类别组织显示。快捷键会根据当前上下文动态更新。"
                    .to_string(),
                tips: vec![
                    "主要快捷键会高亮显示".to_string(),
                    "不可用的快捷键会变暗显示".to_string(),
                    "某些快捷键只在特定视图中可用".to_string(),
                ],
                shortcuts: vec![],
                see_also: vec!["navigation".to_string()],
            },
        );
    }

    /// Toggle help visibility
    pub fn toggle_help(&mut self) -> bool {
        self.help_visible = !self.help_visible;

        if self.help_visible {
            self.start_animation();
        } else {
            self.start_animation();
        }

        tracing::debug!("Help visibility toggled to: {}", self.help_visible);
        self.help_visible
    }

    /// Show help with specific context
    pub fn show_help(&mut self, context: Option<String>) {
        self.help_visible = true;
        self.current_context = context;
        self.start_animation();
        self.last_update = Some(Instant::now());
        tracing::debug!("Help shown with context: {:?}", self.current_context);
    }

    /// Hide help
    pub fn hide_help(&mut self) {
        self.help_visible = false;
        self.start_animation();
        tracing::debug!("Help hidden");
    }

    /// Check if help is visible
    pub fn is_help_visible(&self) -> bool {
        self.help_visible
    }

    /// Set help display mode
    pub fn set_display_mode(&mut self, mode: HelpDisplayMode) {
        tracing::debug!("Help display mode set to: {:?}", mode);
        self.display_mode = mode;
    }

    /// Get current display mode
    pub fn display_mode(&self) -> &HelpDisplayMode {
        &self.display_mode
    }

    /// Start help animation
    fn start_animation(&mut self) {
        self.animation_state.animating = true;
        self.animation_state.start_time = Some(Instant::now());
        self.animation_state.progress = 0.0;
    }

    /// Update animation state
    pub fn update_animation(&mut self) {
        if !self.animation_state.animating {
            return;
        }

        if let Some(start_time) = self.animation_state.start_time {
            let elapsed = start_time.elapsed();
            self.animation_state.progress = (elapsed.as_millis() as f32
                / self.animation_state.duration.as_millis() as f32)
                .min(1.0);

            if self.animation_state.progress >= 1.0 {
                self.animation_state.animating = false;
                self.animation_state.start_time = None;
            }
        }
    }

    /// Get shortcuts for current context
    pub fn get_current_shortcuts(
        &self,
        view: &ViewType,
        widget: Option<&WidgetId>,
    ) -> Vec<ShortcutInfo> {
        let mut shortcuts = Vec::new();

        // Add global shortcuts
        shortcuts.extend(self.global_shortcuts.clone());

        // Add view-specific shortcuts
        if let Some(view_shortcuts) = self.view_shortcuts.get(view) {
            shortcuts.extend(view_shortcuts.clone());
        }

        // Add widget-specific shortcuts
        if let Some(widget_id) = widget {
            if let Some(widget_shortcuts) = self.widget_shortcuts.get(widget_id) {
                shortcuts.extend(widget_shortcuts.clone());
            }
        }

        // Sort by priority (highest first) and then by category
        shortcuts.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.category.cmp(&b.category))
                .then_with(|| a.key.cmp(&b.key))
        });

        shortcuts
    }

    /// Get help content for context
    pub fn get_help_content(&self, context: &str) -> Option<&HelpContent> {
        self.context_help.get(context)
    }

    /// Add custom shortcut
    pub fn add_shortcut(&mut self, shortcut: ShortcutInfo, scope: ShortcutScope) {
        match scope {
            ShortcutScope::Global => {
                self.global_shortcuts.push(shortcut);
            }
            ShortcutScope::View(view) => {
                self.view_shortcuts.entry(view).or_default().push(shortcut);
            }
            ShortcutScope::Widget(widget_id) => {
                self.widget_shortcuts
                    .entry(widget_id)
                    .or_default()
                    .push(shortcut);
            }
        }
    }

    /// Remove shortcut by key
    pub fn remove_shortcut(&mut self, key: &str, scope: ShortcutScope) -> bool {
        match scope {
            ShortcutScope::Global => {
                let initial_len = self.global_shortcuts.len();
                self.global_shortcuts.retain(|s| s.key != key);
                self.global_shortcuts.len() < initial_len
            }
            ShortcutScope::View(view) => {
                if let Some(shortcuts) = self.view_shortcuts.get_mut(&view) {
                    let initial_len = shortcuts.len();
                    shortcuts.retain(|s| s.key != key);
                    shortcuts.len() < initial_len
                } else {
                    false
                }
            }
            ShortcutScope::Widget(widget_id) => {
                if let Some(shortcuts) = self.widget_shortcuts.get_mut(&widget_id) {
                    let initial_len = shortcuts.len();
                    shortcuts.retain(|s| s.key != key);
                    shortcuts.len() < initial_len
                } else {
                    false
                }
            }
        }
    }

    /// Render help overlay
    pub fn render_help_overlay(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !self.help_visible {
            return;
        }

        self.update_animation();

        match self.display_mode {
            HelpDisplayMode::Overlay => self.render_overlay_popup(frame, area, theme),
            HelpDisplayMode::Sidebar => self.render_sidebar(frame, area, theme),
            HelpDisplayMode::BottomPanel => self.render_bottom_panel(frame, area, theme),
            HelpDisplayMode::Tooltip => self.render_tooltip(frame, area, theme),
            HelpDisplayMode::StatusBar => self.render_status_bar(frame, area, theme),
        }
    }

    /// Render help as overlay popup
    fn render_overlay_popup(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Calculate popup size
        let popup_width = (area.width * 80) / 100;
        let popup_height = (area.height * 70) / 100;
        let popup_x = (area.width - popup_width) / 2;
        let popup_y = (area.height - popup_height) / 2;

        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Clear background
        frame.render_widget(Clear, popup_area);

        // Apply animation effect
        let _alpha = if self.animation_state.animating {
            if self.help_visible {
                self.animation_state.progress
            } else {
                1.0 - self.animation_state.progress
            }
        } else {
            1.0
        };

        // Render popup background
        let block = Block::default()
            .borders(Borders::ALL)
            .title("帮助 - 按Esc或?键关闭")
            .border_style(theme.styles.widget_border_focused);

        frame.render_widget(block, popup_area);

        // Inner area for content
        let inner_area = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        // Split into sections
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
            ])
            .split(inner_area);

        // Render header
        let header_text = if let Some(ref context) = self.current_context {
            if let Some(content) = self.context_help.get(context) {
                format!("帮助: {}", content.title)
            } else {
                "键盘快捷键".to_string()
            }
        } else {
            "键盘快捷键".to_string()
        };

        let header = Paragraph::new(header_text)
            .style(theme.styles.widget_title)
            .alignment(Alignment::Center);
        frame.render_widget(header, sections[0]);

        // Render shortcuts content
        self.render_shortcuts_content(frame, sections[1], theme);
    }

    /// Render shortcuts content
    fn render_shortcuts_content(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // For now, render a simple shortcuts list
        // In a full implementation, this would show categorized shortcuts
        let shortcuts_text = vec![
            Line::from(vec![Span::styled("全局快捷键:", theme.styles.text_bold)]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Ctrl+Q", theme.styles.success),
                Span::raw("  退出应用程序"),
            ]),
            Line::from(vec![
                Span::styled("F12", theme.styles.info),
                Span::raw("    刷新数据"),
            ]),
            Line::from(vec![
                Span::styled("?", theme.styles.info),
                Span::raw("      显示/隐藏帮助"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("导航快捷键:", theme.styles.text_bold)]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Tab", theme.styles.info),
                Span::raw("    下一个元素"),
            ]),
            Line::from(vec![
                Span::styled("Shift+Tab", theme.styles.info),
                Span::raw(" 上一个元素"),
            ]),
            Line::from(vec![
                Span::styled("Esc", theme.styles.warning),
                Span::raw("    返回/取消"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("视图切换:", theme.styles.text_bold)]),
            Line::from(""),
            Line::from(vec![
                Span::styled("F1", theme.styles.info),
                Span::raw("     工作流列表"),
            ]),
            Line::from(vec![
                Span::styled("F2", theme.styles.info),
                Span::raw("     执行监控"),
            ]),
            Line::from(vec![
                Span::styled("F3", theme.styles.info),
                Span::raw("     工具管理"),
            ]),
            Line::from(vec![
                Span::styled("F4", theme.styles.info),
                Span::raw("     插件管理"),
            ]),
            Line::from(vec![
                Span::styled("F5", theme.styles.info),
                Span::raw("     系统状态"),
            ]),
            Line::from(vec![
                Span::styled("F6", theme.styles.info),
                Span::raw("     日志查看器"),
            ]),
        ];

        let paragraph = Paragraph::new(shortcuts_text)
            .style(theme.styles.text_normal)
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        frame.render_widget(paragraph, area);
    }

    /// Render help as sidebar (placeholder)
    fn render_sidebar(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {
        // Placeholder for sidebar implementation
    }

    /// Render help as bottom panel (placeholder)
    fn render_bottom_panel(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {
        // Placeholder for bottom panel implementation
    }

    /// Render help as tooltip (placeholder)
    fn render_tooltip(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {
        // Placeholder for tooltip implementation
    }

    /// Render help in status bar
    fn render_status_bar(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let shortcuts_text = "F1-F6: 视图切换 | Tab: 导航 | ?: 帮助 | Ctrl+Q: 退出";

        let paragraph = Paragraph::new(shortcuts_text)
            .style(theme.styles.status_bar)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Handle help-related key events
    pub fn handle_key_event(
        &mut self,
        key: ratatui::crossterm::event::KeyEvent,
    ) -> Result<Option<Action>> {
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};

        match (key.code, key.modifiers) {
            // Toggle help with ? key
            (KeyCode::Char('?'), KeyModifiers::NONE) => {
                self.toggle_help();
                Ok(Some(Action::ToggleHelp))
            }
            // Close help with Esc
            (KeyCode::Esc, KeyModifiers::NONE) if self.help_visible => {
                self.hide_help();
                Ok(Some(Action::ToggleHelp))
            }
            // F1 for context help
            (KeyCode::F(1), KeyModifiers::SHIFT) => {
                self.show_help(Some("getting_started".to_string()));
                Ok(Some(Action::ToggleHelp))
            }
            _ => Ok(None),
        }
    }

    /// Get help statistics
    pub fn get_help_stats(&self) -> HelpStats {
        HelpStats {
            global_shortcuts_count: self.global_shortcuts.len(),
            view_shortcuts_count: self.view_shortcuts.values().map(|v| v.len()).sum(),
            widget_shortcuts_count: self.widget_shortcuts.values().map(|v| v.len()).sum(),
            help_content_count: self.context_help.len(),
            is_visible: self.help_visible,
            current_context: self.current_context.clone(),
            display_mode: self.display_mode.clone(),
        }
    }
}

/// Shortcut scope for organizing shortcuts
#[derive(Debug, Clone)]
pub enum ShortcutScope {
    /// Global shortcuts available everywhere
    Global,
    /// View-specific shortcuts
    View(ViewType),
    /// Widget-specific shortcuts
    Widget(WidgetId),
}

/// Help system statistics
#[derive(Debug, Clone)]
pub struct HelpStats {
    pub global_shortcuts_count: usize,
    pub view_shortcuts_count: usize,
    pub widget_shortcuts_count: usize,
    pub help_content_count: usize,
    pub is_visible: bool,
    pub current_context: Option<String>,
    pub display_mode: HelpDisplayMode,
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HelpPopupConfig {
    fn default() -> Self {
        Self {
            width_percent: 80,
            height_percent: 70,
            centered: true,
            show_close_button: true,
            dim_background: true,
            animation_duration: Duration::from_millis(300),
        }
    }
}
