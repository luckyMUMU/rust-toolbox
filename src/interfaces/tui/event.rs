//! Event handling system for TUI widgets
//!
//! This module provides event processing and handling for the TUI interface.

use crate::error::Result;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::action::{Action, ViewType};

/// TUI-specific events
#[derive(Debug, Clone)]
pub enum TuiEvent {
    /// Keyboard event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize event
    Resize(u16, u16),
    /// Periodic tick event
    Tick,
    /// Application quit event
    Quit,
    /// Focus gained event
    FocusGained,
    /// Focus lost event
    FocusLost,
    /// Paste event
    Paste(String),
    /// Custom event
    Custom(String, serde_json::Value),
}

/// Event processing result
#[derive(Debug, Clone)]
pub enum EventResult {
    /// Event was consumed and produced an action
    Consumed(Action),
    /// Event was not handled
    NotHandled,
    /// Event processing resulted in an error
    Error(String),
    /// Event was consumed but produced no action
    ConsumedNoAction,
}

/// Key binding configuration
#[derive(Debug, Clone)]
pub struct KeyBindings {
    /// Global shortcuts that work in any context
    pub global_shortcuts: HashMap<(KeyCode, KeyModifiers), Action>,
    /// Context-specific shortcuts for different views
    pub context_shortcuts: HashMap<ViewType, HashMap<(KeyCode, KeyModifiers), Action>>,
    /// Widget-specific shortcuts
    pub widget_shortcuts: HashMap<String, HashMap<(KeyCode, KeyModifiers), Action>>,
}

/// Shortcut help information
#[derive(Debug, Clone)]
pub struct ShortcutHelp {
    pub category: String,
    pub shortcuts: Vec<(String, String)>, // (key combination, description)
}

/// Event handler for processing terminal events
pub struct EventHandler {
    event_sender: mpsc::UnboundedSender<TuiEvent>,
    event_receiver: mpsc::UnboundedReceiver<TuiEvent>,
    key_bindings: KeyBindings,
    mouse_enabled: bool,
    last_resize: Option<Instant>,
    resize_debounce: Duration,
    event_filters: Vec<Box<dyn EventFilter>>,
}

/// Trait for filtering events before processing
pub trait EventFilter: Send + Sync {
    /// Filter an event, returning None to drop it or Some to keep it
    fn filter(&self, event: &TuiEvent) -> Option<TuiEvent>;

    /// Get the filter's name
    fn name(&self) -> &str;

    /// Get the filter's priority (higher priority filters run first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Event statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    pub total_events: u64,
    pub key_events: u64,
    pub mouse_events: u64,
    pub resize_events: u64,
    pub tick_events: u64,
    pub custom_events: u64,
    pub filtered_events: u64,
    pub error_events: u64,
    pub last_event_time: Option<Instant>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        Self {
            event_sender,
            event_receiver,
            key_bindings: KeyBindings::default(),
            mouse_enabled: true,
            last_resize: None,
            resize_debounce: Duration::from_millis(100),
            event_filters: Vec::new(),
        }
    }

    /// Start the event handling loop
    pub async fn start_event_loop(&mut self) -> Result<()> {
        let sender = self.event_sender.clone();

        tokio::spawn(async move {
            loop {
                match ratatui::crossterm::event::read() {
                    Ok(terminal_event) => {
                        if let Some(tui_event) = Self::convert_terminal_event(terminal_event) {
                            if sender.send(tui_event).is_err() {
                                break; // Channel closed, exit loop
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to read terminal event: {}", e);
                        break;
                    }
                }
            }
        });

        tracing::info!("Started event handling loop");
        Ok(())
    }

    /// Convert crossterm events to TUI events
    fn convert_terminal_event(event: Event) -> Option<TuiEvent> {
        match event {
            Event::Key(key_event) => {
                // Only process key press events, ignore release
                if key_event.kind == KeyEventKind::Press {
                    Some(TuiEvent::Key(key_event))
                } else {
                    None
                }
            }
            Event::Mouse(mouse_event) => Some(TuiEvent::Mouse(mouse_event)),
            Event::Resize(width, height) => Some(TuiEvent::Resize(width, height)),
            Event::FocusGained => Some(TuiEvent::FocusGained),
            Event::FocusLost => Some(TuiEvent::FocusLost),
            Event::Paste(text) => Some(TuiEvent::Paste(text)),
        }
    }

    /// Process the next event from the queue
    pub async fn process_next_event(&mut self) -> Option<EventResult> {
        if let Some(event) = self.event_receiver.recv().await {
            Some(self.process_event(event).await)
        } else {
            None
        }
    }

    /// Process a TUI event and convert it to an action
    pub async fn process_event(&mut self, event: TuiEvent) -> EventResult {
        // Apply event filters
        let filtered_event = self.apply_filters(event);
        let event = match filtered_event {
            Some(e) => e,
            None => return EventResult::ConsumedNoAction, // Event was filtered out
        };

        match event {
            TuiEvent::Key(key_event) => self.process_key_event(key_event).await,
            TuiEvent::Mouse(mouse_event) => self.process_mouse_event(mouse_event).await,
            TuiEvent::Resize(width, height) => self.process_resize_event(width, height).await,
            TuiEvent::Tick => EventResult::Consumed(Action::None),
            TuiEvent::Quit => EventResult::Consumed(Action::Quit),
            TuiEvent::FocusGained => self.process_focus_event(true).await,
            TuiEvent::FocusLost => self.process_focus_event(false).await,
            TuiEvent::Paste(text) => self.process_paste_event(text).await,
            TuiEvent::Custom(name, data) => self.process_custom_event(name, data).await,
        }
    }

    /// Apply event filters to an event
    fn apply_filters(&self, event: TuiEvent) -> Option<TuiEvent> {
        let mut current_event = Some(event);

        // Sort filters by priority (highest first)
        let mut filters: Vec<_> = self.event_filters.iter().collect();
        filters.sort_by(|a, b| b.priority().cmp(&a.priority()));

        for filter in filters {
            if let Some(e) = current_event {
                current_event = filter.filter(&e);
                if current_event.is_none() {
                    tracing::debug!("Event filtered out by: {}", filter.name());
                    break;
                }
            }
        }

        current_event
    }

    /// Process keyboard events
    async fn process_key_event(&self, key_event: KeyEvent) -> EventResult {
        // Handle global shortcuts first
        if let Some(action) = self.handle_global_shortcuts(&key_event) {
            return EventResult::Consumed(action);
        }

        // Convert key event to action based on context
        match self.key_event_to_action(key_event) {
            Ok(action) => EventResult::Consumed(action),
            Err(e) => EventResult::Error(e.to_string()),
        }
    }

    /// Handle global keyboard shortcuts
    fn handle_global_shortcuts(&self, key_event: &KeyEvent) -> Option<Action> {
        let key_combo = (key_event.code, key_event.modifiers);

        // Check if this key combination is bound to a global action
        if let Some(action) = self.key_bindings.global_shortcuts.get(&key_combo) {
            return Some(action.clone());
        }

        // Fallback to hardcoded shortcuts for compatibility
        self.handle_hardcoded_shortcuts(key_event)
    }

    /// Handle hardcoded shortcuts (fallback)
    fn handle_hardcoded_shortcuts(&self, key_event: &KeyEvent) -> Option<Action> {
        match (key_event.code, key_event.modifiers) {
            // Quit shortcuts
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Action::Quit),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),

            // Navigation shortcuts
            (KeyCode::F(1), KeyModifiers::NONE) => Some(Action::Navigate(ViewType::WorkflowList)),
            (KeyCode::F(2), KeyModifiers::NONE) => {
                Some(Action::Navigate(ViewType::ExecutionMonitor))
            }
            (KeyCode::F(3), KeyModifiers::NONE) => Some(Action::Navigate(ViewType::ToolManager)),
            (KeyCode::F(4), KeyModifiers::NONE) => Some(Action::Navigate(ViewType::PluginManager)),
            (KeyCode::F(5), KeyModifiers::NONE) => Some(Action::Navigate(ViewType::SystemStatus)),
            (KeyCode::F(6), KeyModifiers::NONE) => Some(Action::Navigate(ViewType::LogViewer)),

            // Refresh
            (KeyCode::F(12), KeyModifiers::NONE) => Some(Action::Refresh),
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(Action::Refresh),

            // Focus management
            (KeyCode::Tab, KeyModifiers::NONE) => Some(Action::FocusNext),
            (KeyCode::Tab, KeyModifiers::SHIFT) => Some(Action::FocusPrevious),

            // Search
            (KeyCode::Char('/'), KeyModifiers::NONE) => Some(Action::Search(String::new())),
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => Some(Action::Search(String::new())),

            // Back/Cancel
            (KeyCode::Esc, KeyModifiers::NONE) => Some(Action::GoBack),

            _ => None,
        }
    }

    /// Convert key event to action
    fn key_event_to_action(&self, key_event: KeyEvent) -> Result<Action> {
        match key_event.code {
            // Navigation keys - let widgets handle these
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End => Ok(Action::None),

            // Action keys - let widgets handle these
            KeyCode::Enter | KeyCode::Delete | KeyCode::Backspace => Ok(Action::None),

            // Character input - let widgets handle these
            KeyCode::Char(_) => Ok(Action::None),

            _ => Ok(Action::None),
        }
    }

    /// Process mouse events
    async fn process_mouse_event(&self, _mouse_event: MouseEvent) -> EventResult {
        if !self.mouse_enabled {
            return EventResult::NotHandled;
        }

        // For now, let widgets handle mouse events
        EventResult::NotHandled
    }

    /// Process terminal resize events
    async fn process_resize_event(&mut self, width: u16, height: u16) -> EventResult {
        let now = Instant::now();

        // Debounce resize events to avoid excessive redraws
        if let Some(last_resize) = self.last_resize {
            if now.duration_since(last_resize) < self.resize_debounce {
                return EventResult::NotHandled;
            }
        }

        self.last_resize = Some(now);

        tracing::debug!("Terminal resized to {}x{}", width, height);

        // The resize will be handled by the main render loop
        EventResult::Consumed(Action::None)
    }

    /// Process focus events
    async fn process_focus_event(&self, gained: bool) -> EventResult {
        if gained {
            tracing::debug!("Terminal focus gained");
            EventResult::Consumed(Action::Refresh)
        } else {
            tracing::debug!("Terminal focus lost");
            EventResult::NotHandled
        }
    }

    /// Process paste events
    async fn process_paste_event(&self, text: String) -> EventResult {
        tracing::debug!("Text pasted: {} characters", text.len());

        // For now, ignore paste events - will be handled by specific widgets
        // that support text input (like search boxes)
        EventResult::NotHandled
    }

    /// Process custom events
    async fn process_custom_event(&self, name: String, data: serde_json::Value) -> EventResult {
        tracing::debug!("Custom event: {} with data: {:?}", name, data);
        EventResult::Consumed(Action::Custom(name, data))
    }

    /// Send a custom event
    pub fn send_custom_event(&self, name: String, data: serde_json::Value) -> Result<()> {
        let event = TuiEvent::Custom(name, data);
        self.event_sender.send(event).map_err(|e| {
            crate::error::WorkflowError::ValidationError(format!(
                "Failed to send custom event: {}",
                e
            ))
        })?;
        Ok(())
    }

    /// Send a tick event
    pub fn send_tick(&self) -> Result<()> {
        self.event_sender.send(TuiEvent::Tick).map_err(|e| {
            crate::error::WorkflowError::ValidationError(format!(
                "Failed to send tick event: {}",
                e
            ))
        })?;
        Ok(())
    }

    /// Check if there are pending events
    pub fn has_pending_events(&self) -> bool {
        !self.event_receiver.is_empty()
    }

    /// Enable or disable mouse support
    pub fn set_mouse_enabled(&mut self, enabled: bool) {
        self.mouse_enabled = enabled;
        tracing::debug!(
            "Mouse support {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Get current mouse support status
    pub fn is_mouse_enabled(&self) -> bool {
        self.mouse_enabled
    }

    /// Set resize debounce duration
    pub fn set_resize_debounce(&mut self, duration: Duration) {
        self.resize_debounce = duration;
    }

    /// Get reference to key bindings
    pub fn key_bindings(&self) -> &KeyBindings {
        &self.key_bindings
    }

    /// Get mutable reference to key bindings
    pub fn key_bindings_mut(&mut self) -> &mut KeyBindings {
        &mut self.key_bindings
    }

    /// Update key bindings
    pub fn set_key_bindings(&mut self, bindings: KeyBindings) {
        self.key_bindings = bindings;
        tracing::debug!("Updated key bindings");
    }

    /// Add an event filter
    pub fn add_event_filter(&mut self, filter: Box<dyn EventFilter>) {
        tracing::debug!("Added event filter: {}", filter.name());
        self.event_filters.push(filter);

        // Sort filters by priority
        self.event_filters
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Remove an event filter by name
    pub fn remove_event_filter(&mut self, name: &str) -> bool {
        let initial_len = self.event_filters.len();
        self.event_filters.retain(|f| f.name() != name);
        let removed = self.event_filters.len() < initial_len;

        if removed {
            tracing::debug!("Removed event filter: {}", name);
        }

        removed
    }

    /// Get help for current global shortcuts
    pub fn get_global_shortcuts_help(&self) -> Vec<ShortcutHelp> {
        self.key_bindings.get_global_shortcuts_help()
    }

    /// Get help for view-specific shortcuts
    pub fn get_view_shortcuts_help(&self, view: &ViewType) -> Vec<ShortcutHelp> {
        self.key_bindings.get_view_shortcuts_help(view)
    }

    /// Get help for widget-specific shortcuts
    pub fn get_widget_shortcuts_help(&self, widget_id: &str) -> Vec<ShortcutHelp> {
        self.key_bindings.get_widget_shortcuts_help(widget_id)
    }
}

impl KeyBindings {
    /// Create default key bindings
    pub fn default() -> Self {
        let mut global_shortcuts = HashMap::new();

        // Quit shortcuts
        global_shortcuts.insert((KeyCode::Char('q'), KeyModifiers::CONTROL), Action::Quit);
        global_shortcuts.insert((KeyCode::Char('c'), KeyModifiers::CONTROL), Action::Quit);

        // Function key navigation
        global_shortcuts.insert(
            (KeyCode::F(1), KeyModifiers::NONE),
            Action::Navigate(ViewType::WorkflowList),
        );
        global_shortcuts.insert(
            (KeyCode::F(2), KeyModifiers::NONE),
            Action::Navigate(ViewType::ExecutionMonitor),
        );
        global_shortcuts.insert(
            (KeyCode::F(3), KeyModifiers::NONE),
            Action::Navigate(ViewType::ToolManager),
        );
        global_shortcuts.insert(
            (KeyCode::F(4), KeyModifiers::NONE),
            Action::Navigate(ViewType::PluginManager),
        );
        global_shortcuts.insert(
            (KeyCode::F(5), KeyModifiers::NONE),
            Action::Navigate(ViewType::SystemStatus),
        );
        global_shortcuts.insert(
            (KeyCode::F(6), KeyModifiers::NONE),
            Action::Navigate(ViewType::LogViewer),
        );

        // Refresh
        global_shortcuts.insert((KeyCode::F(12), KeyModifiers::NONE), Action::Refresh);
        global_shortcuts.insert((KeyCode::Char('r'), KeyModifiers::CONTROL), Action::Refresh);

        // Focus management
        global_shortcuts.insert((KeyCode::Tab, KeyModifiers::NONE), Action::FocusNext);
        global_shortcuts.insert((KeyCode::Tab, KeyModifiers::SHIFT), Action::FocusPrevious);

        // Search and filter
        global_shortcuts.insert(
            (KeyCode::Char('/'), KeyModifiers::NONE),
            Action::Search(String::new()),
        );
        global_shortcuts.insert(
            (KeyCode::Char('f'), KeyModifiers::CONTROL),
            Action::Search(String::new()),
        );

        // Back/Cancel
        global_shortcuts.insert((KeyCode::Esc, KeyModifiers::NONE), Action::GoBack);

        Self {
            global_shortcuts,
            context_shortcuts: HashMap::new(),
            widget_shortcuts: HashMap::new(),
        }
    }

    /// Get all global shortcuts help
    pub fn get_global_shortcuts_help(&self) -> Vec<ShortcutHelp> {
        vec![
            ShortcutHelp {
                category: "应用程序".to_string(),
                shortcuts: vec![
                    ("Ctrl+Q/C".to_string(), "退出应用程序".to_string()),
                    ("F12, Ctrl+R".to_string(), "刷新数据".to_string()),
                ],
            },
            ShortcutHelp {
                category: "视图导航".to_string(),
                shortcuts: vec![
                    ("F1".to_string(), "工作流列表".to_string()),
                    ("F2".to_string(), "执行监控".to_string()),
                    ("F3".to_string(), "工具管理".to_string()),
                    ("F4".to_string(), "插件管理".to_string()),
                    ("F5".to_string(), "系统状态".to_string()),
                    ("F6".to_string(), "日志查看器".to_string()),
                ],
            },
            ShortcutHelp {
                category: "搜索和导航".to_string(),
                shortcuts: vec![
                    ("/".to_string(), "开始搜索".to_string()),
                    ("Ctrl+F".to_string(), "搜索".to_string()),
                    ("Esc".to_string(), "返回/取消".to_string()),
                    ("Tab".to_string(), "下一个元素".to_string()),
                    ("Shift+Tab".to_string(), "上一个元素".to_string()),
                ],
            },
        ]
    }

    /// Get shortcuts for a specific view
    pub fn get_view_shortcuts_help(&self, view: &ViewType) -> Vec<ShortcutHelp> {
        match view {
            ViewType::WorkflowList => vec![ShortcutHelp {
                category: "工作流操作".to_string(),
                shortcuts: vec![
                    ("Enter".to_string(), "执行选中的工作流".to_string()),
                    ("↑/↓".to_string(), "选择工作流".to_string()),
                    ("d".to_string(), "切换详情视图".to_string()),
                ],
            }],
            ViewType::ExecutionMonitor => vec![ShortcutHelp {
                category: "执行控制".to_string(),
                shortcuts: vec![
                    ("Space".to_string(), "暂停/恢复执行".to_string()),
                    ("s".to_string(), "停止执行".to_string()),
                    ("c".to_string(), "取消执行".to_string()),
                ],
            }],
            ViewType::ToolManager => vec![ShortcutHelp {
                category: "工具操作".to_string(),
                shortcuts: vec![
                    ("Enter".to_string(), "执行工具".to_string()),
                    ("i".to_string(), "查看工具信息".to_string()),
                ],
            }],
            ViewType::PluginManager => vec![ShortcutHelp {
                category: "插件操作".to_string(),
                shortcuts: vec![
                    ("i".to_string(), "安装插件".to_string()),
                    ("u".to_string(), "卸载插件".to_string()),
                    ("r".to_string(), "重新加载插件".to_string()),
                ],
            }],
            ViewType::SystemStatus => vec![ShortcutHelp {
                category: "系统监控".to_string(),
                shortcuts: vec![
                    ("r".to_string(), "刷新系统状态".to_string()),
                    ("d".to_string(), "显示详细信息".to_string()),
                ],
            }],
            ViewType::LogViewer => vec![ShortcutHelp {
                category: "日志操作".to_string(),
                shortcuts: vec![
                    ("1-5".to_string(), "按级别过滤日志".to_string()),
                    ("c".to_string(), "清除日志".to_string()),
                    ("e".to_string(), "导出日志".to_string()),
                ],
            }],
        }
    }

    /// Get shortcuts for a specific widget
    pub fn get_widget_shortcuts_help(&self, widget_id: &str) -> Vec<ShortcutHelp> {
        if let Some(shortcuts) = self.widget_shortcuts.get(widget_id) {
            let mut help_shortcuts = Vec::new();
            for ((key_code, modifiers), action) in shortcuts {
                let key_str = format_key_combination(*key_code, *modifiers);
                let description = format!("{}", action);
                help_shortcuts.push((key_str, description));
            }

            vec![ShortcutHelp {
                category: format!("Widget: {}", widget_id),
                shortcuts: help_shortcuts,
            }]
        } else {
            vec![]
        }
    }

    /// Add a global shortcut
    pub fn add_global_shortcut(
        &mut self,
        key_code: KeyCode,
        modifiers: KeyModifiers,
        action: Action,
    ) {
        self.global_shortcuts.insert((key_code, modifiers), action);
    }

    /// Add a context-specific shortcut
    pub fn add_context_shortcut(
        &mut self,
        view: ViewType,
        key_code: KeyCode,
        modifiers: KeyModifiers,
        action: Action,
    ) {
        self.context_shortcuts
            .entry(view)
            .or_default()
            .insert((key_code, modifiers), action);
    }

    /// Add a widget-specific shortcut
    pub fn add_widget_shortcut(
        &mut self,
        widget_id: String,
        key_code: KeyCode,
        modifiers: KeyModifiers,
        action: Action,
    ) {
        self.widget_shortcuts
            .entry(widget_id)
            .or_default()
            .insert((key_code, modifiers), action);
    }

    /// Remove a global shortcut
    pub fn remove_global_shortcut(
        &mut self,
        key_code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Option<Action> {
        self.global_shortcuts.remove(&(key_code, modifiers))
    }

    /// Clear all shortcuts for a view
    pub fn clear_context_shortcuts(&mut self, view: &ViewType) {
        self.context_shortcuts.remove(view);
    }

    /// Clear all shortcuts for a widget
    pub fn clear_widget_shortcuts(&mut self, widget_id: &str) {
        self.widget_shortcuts.remove(widget_id);
    }
}

/// Format a key combination for display
fn format_key_combination(key_code: KeyCode, modifiers: KeyModifiers) -> String {
    let mut parts = Vec::new();

    if modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }

    let key_str = match key_code {
        KeyCode::Char(c) => {
            if c == ' ' {
                "Space".to_string()
            } else {
                c.to_uppercase().to_string()
            }
        }
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        _ => format!("{:?}", key_code),
    };

    parts.push(&key_str);
    parts.join("+")
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple event filter that drops events based on a predicate
pub struct PredicateEventFilter {
    name: String,
    priority: i32,
    predicate: Box<dyn Fn(&TuiEvent) -> bool + Send + Sync>,
}

impl PredicateEventFilter {
    /// Create a new predicate-based event filter
    pub fn new<F>(name: String, priority: i32, predicate: F) -> Self
    where
        F: Fn(&TuiEvent) -> bool + Send + Sync + 'static,
    {
        Self {
            name,
            priority,
            predicate: Box::new(predicate),
        }
    }
}

impl EventFilter for PredicateEventFilter {
    fn filter(&self, event: &TuiEvent) -> Option<TuiEvent> {
        if (self.predicate)(event) {
            Some(event.clone())
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}
