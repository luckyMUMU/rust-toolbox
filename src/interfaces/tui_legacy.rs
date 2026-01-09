//! TUI interface implementation
//! 
//! This module provides the TUI (Terminal User Interface) implementation
//! for the workflow toolkit using ratatui and crossterm.
//! 
//! This module now includes the enhanced Widget system with proper layout management,
//! theme support, and event handling.

pub mod widget;
pub mod layout;
pub mod theme;
pub mod event;
pub mod action;

// Re-export the new widget system
pub use widget::{Widget, WidgetId, WidgetState, WidgetContext, WidgetError, BaseWidget};
pub use layout::{LayoutManager, LayoutConstraints, LayoutDirection, LayoutNode};
pub use theme::{Theme, ColorScheme, StyleScheme, ThemeManager};
pub use event::{EventHandler, TuiEvent, EventResult};
pub use action::{Action, ActionDispatcher, ActionResult};

use crate::error::Result;
use crate::workflow::WorkflowExecution;
use crate::core::{ToolInfo, WorkflowId, PluginInfo};
use crate::plugins::manager::PluginManager;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::io;
use std::collections::HashMap;

// Ratatui and crossterm imports
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Tabs, Wrap,
    },
    Frame, Terminal,
};
use tokio::sync::mpsc;
use super::tui::{WidgetRegistry, ThemeManager, EventHandler, ActionDispatcher};

/// Main TUI application structure with enhanced widget system
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    widget_registry: super::tui::WidgetRegistry,
    router: Router,
    state: AppState,
    theme_manager: ThemeManager,
    event_handler: EventHandler,
    action_dispatcher: ActionDispatcher,
    should_quit: bool,
    tick_rate: Duration,
}

/// Router for managing view navigation and Widget lifecycle
pub struct Router {
    current_view: ViewType,
    view_stack: Vec<ViewType>,
    widgets: std::collections::HashMap<ViewType, Box<dyn Widget>>,
}

/// Application state manager for data synchronization
pub struct AppState {
    workflows: Arc<tokio::sync::RwLock<Vec<WorkflowInfo>>>,
    executions: Arc<tokio::sync::RwLock<Vec<ExecutionInfo>>>,
    tools: Arc<tokio::sync::RwLock<Vec<ToolInfo>>>,
    plugins: Arc<tokio::sync::RwLock<Vec<PluginInfo>>>,
    system_status: Arc<tokio::sync::RwLock<SystemStatus>>,
    logs: Arc<tokio::sync::RwLock<Vec<LogEntry>>>,
    connection_status: Arc<tokio::sync::RwLock<ConnectionStatus>>,
    last_update: Arc<tokio::sync::RwLock<DateTime<Utc>>>,
}

/// Event handler for processing terminal events
pub struct EventHandler {
    event_sender: mpsc::UnboundedSender<TuiEvent>,
    event_receiver: mpsc::UnboundedReceiver<TuiEvent>,
    mouse_enabled: bool,
    last_resize: Option<Instant>,
    resize_debounce: Duration,
    key_bindings: KeyBindings,
}

/// Theme system for managing colors and styles
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub styles: StyleScheme,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub highlight: Color,
}

#[derive(Debug, Clone)]
pub struct StyleScheme {
    pub header: Style,
    pub status_bar: Style,
    pub border: Style,
    pub highlight: Style,
    pub error: Style,
    pub warning: Style,
    pub success: Style,
    pub info: Style,
}

/// Different views in the TUI
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewType {
    WorkflowList,
    ExecutionMonitor,
    ToolManager,
    PluginManager,
    SystemStatus,
    LogViewer,
}

/// Actions that can be triggered by TUI events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    
    // Navigation actions
    Navigate(ViewType),
    GoBack,
    GoForward,
    
    // Workflow actions
    ExecuteWorkflow(String),
    PauseWorkflow(WorkflowId),
    ResumeWorkflow(WorkflowId),
    StopWorkflow(WorkflowId),
    CancelWorkflow(WorkflowId),
    ShowWorkflowDetails(String),
    CreateWorkflow,
    EditWorkflow(String),
    DeleteWorkflow(String),
    
    // Tool actions
    ExecuteTool(String),
    ShowToolDetails(String),
    RefreshTools,
    
    // Plugin actions
    InstallPlugin(String),
    UninstallPlugin(String),
    ReloadPlugin(String),
    EnablePlugin(String),
    DisablePlugin(String),
    ShowPluginDetails(String),
    RefreshPlugins,
    
    // System actions
    Refresh,
    RefreshSystemStatus,
    ShowSystemDetails,
    
    // UI actions
    Search(String),
    Filter(String),
    Sort(SortOrder),
    ClearFilter,
    ToggleDetails,
    ToggleHelp,
    
    // Focus and navigation
    FocusNext,
    FocusPrevious,
    FocusWidget(String),
    
    // List actions
    SelectNext,
    SelectPrevious,
    SelectFirst,
    SelectLast,
    SelectItem(usize),
    
    // Input actions
    StartInput(InputMode),
    ConfirmInput(String),
    CancelInput,
    
    // Log actions
    ShowLogs,
    FilterLogs(LogLevel),
    ClearLogs,
    ExportLogs,
    
    // Error handling
    ShowError(String),
    DismissError,
    
    // Configuration
    ShowSettings,
    ChangeTheme(String),
    SaveSettings,
}

/// Input modes for text input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Search,
    Filter,
    Command,
    WorkflowName,
    PluginUrl,
}

/// Action dispatcher for handling actions
pub struct ActionDispatcher {
    action_queue: mpsc::UnboundedSender<Action>,
    action_receiver: mpsc::UnboundedReceiver<Action>,
}

/// Key binding configuration
#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub global_shortcuts: std::collections::HashMap<(KeyCode, KeyModifiers), Action>,
    pub context_shortcuts: std::collections::HashMap<ViewType, std::collections::HashMap<(KeyCode, KeyModifiers), Action>>,
}

/// Shortcut help information
#[derive(Debug, Clone)]
pub struct ShortcutHelp {
    pub category: String,
    pub shortcuts: Vec<(String, String)>, // (key combination, description)
}

/// Sort order for lists
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
    StatusAsc,
    StatusDesc,
}

/// Internal TUI events
#[derive(Debug, Clone)]
pub enum TuiEvent {
    Key(event::KeyEvent),
    Mouse(event::MouseEvent),
    Resize(u16, u16),
    Tick,
    Quit,
    FocusGained,
    FocusLost,
    Paste(String),
}

/// Event processing result
#[derive(Debug, Clone)]
pub enum EventResult {
    Consumed(Action),
    NotHandled,
    Error(String),
}

/// Connection status for backend communication
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub active_workflows: u32,
    pub system_health: SystemHealth,
    pub uptime: Duration,
    pub network_status: NetworkStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemHealth {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkStatus {
    Connected,
    Disconnected,
    Limited,
}

/// Execution information for monitoring
#[derive(Debug, Clone)]
pub struct ExecutionInfo {
    pub id: String,
    pub workflow_name: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: f64,
    pub current_task: Option<String>,
    pub tasks: Vec<TaskInfo>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub name: String,
    pub status: TaskStatus,
    pub progress: f64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Widget trait for TUI components
#[async_trait]
pub trait Widget: Send + Sync {
    /// Render the widget to the given frame area
    async fn render(&mut self, frame: &mut Frame, area: Rect);
    
    /// Handle an event and return the resulting action
    async fn handle_event(&mut self, event: Event) -> Result<Action>;
    
    /// Update the widget state (called periodically)
    async fn update(&mut self) -> Result<()>;
    
    /// Get the widget title for display
    fn title(&self) -> &str;
    
    /// Get help text for keyboard shortcuts
    fn help_text(&self) -> Vec<(&str, &str)>; // (key, description)
    
    /// Initialize the widget (called once when registered)
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Cleanup the widget (called when unregistering)
    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the widget becomes active (gains focus)
    async fn on_activate(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the widget becomes inactive (loses focus)
    async fn on_deactivate(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Check if the widget can handle a specific event type
    fn can_handle_event(&self, event: &Event) -> bool {
        match event {
            Event::Key(_) => true,
            Event::Mouse(_) => false, // Default: no mouse support
            Event::Resize(_, _) => true,
            _ => false,
        }
    }
    
    /// Get the widget's preferred size constraints
    fn size_constraints(&self) -> (Option<u16>, Option<u16>) {
        (None, None) // (min_width, min_height)
    }
    
    /// Check if the widget needs periodic updates
    fn needs_update(&self) -> bool {
        false
    }
    
    /// Get the widget's update interval in milliseconds
    fn update_interval(&self) -> u64 {
        1000 // Default: 1 second
    }
}

/// Information about a workflow for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub status: WorkflowStatus,
    pub last_execution: Option<DateTime<Utc>>,
    pub execution_count: u32,
    pub tags: Vec<String>,
}

/// Workflow status for display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkflowStatus {
    Available,
    Running,
    Paused,
    Completed,
    Failed,
}

/// Progress bar information
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub label: String,
    pub current: u64,
    pub total: u64,
    pub percentage: f64,
}

/// Log entry for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: Option<String>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl TuiApp {
    /// Create a new TUI application
    pub async fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        Ok(Self {
            terminal,
            widget_registry: WidgetRegistry::new(),
            router: Router::new(),
            state: AppState::new(),
            theme_manager: ThemeManager::new(),
            event_handler: EventHandler::new(),
            action_dispatcher: ActionDispatcher::new(),
            should_quit: false,
            tick_rate: Duration::from_millis(16), // 60 FPS
        })
    }
    
    /// Run the TUI application main loop
    pub async fn run(&mut self) -> Result<()> {
        // Initialize widget registry
        self.widget_registry.initialize_all().await?;
        
        // Start periodic data refresh
        self.state.start_periodic_refresh(Duration::from_secs(30)).await?;
        
        // Start the event handling loop
        self.event_handler.start_event_loop().await?;
        
        let mut last_tick = Instant::now();
        
        loop {
            // Process events from the event handler
            while let Ok(event) = self.event_handler.event_receiver.try_recv() {
                let event_result = self.event_handler.process_event(event).await?;
                
                match event_result {
                    EventResult::Consumed(action) => {
                        let action_result = self.action_dispatcher.process_action(
                            action.clone(), 
                            &mut self.state, 
                            &mut self.router
                        ).await?;
                        
                        match action_result {
                            ActionResult::Success => {
                                if action == Action::Quit {
                                    self.should_quit = true;
                                }
                            }
                            ActionResult::Error(err) => {
                                tracing::error!("Action processing error: {}", err);
                                // TODO: Show error in UI
                            }
                            ActionResult::RequiresConfirmation(message) => {
                                tracing::info!("Action requires confirmation: {}", message);
                                // TODO: Show confirmation dialog
                            }
                            ActionResult::RequiresInput(mode, prompt) => {
                                tracing::info!("Action requires input: {:?} - {}", mode, prompt);
                                // TODO: Show input dialog
                            }
                        }
                    }
                    EventResult::NotHandled => {
                        // Event not handled by global handler, delegate to current widget
                        // This will be implemented when widgets are enhanced
                    }
                    EventResult::Error(err) => {
                        tracing::error!("Event processing error: {}", err);
                    }
                }
            }
            
            // Periodic updates
            if last_tick.elapsed() >= self.tick_rate {
                self.update().await?;
                last_tick = Instant::now();
            }
            
            // Render interface
            self.render()?;
            
            if self.should_quit {
                break;
            }
            
            // Small sleep to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        
        // Cleanup widget registry before exit
        self.widget_registry.cleanup_all().await?;
        
        Ok(())
    }
    
    async fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<Action> {
        // This method is now deprecated - event handling is done by EventHandler
        // Keeping for compatibility during transition
        Ok(Action::None)
    }
    
    async fn process_action(&mut self, action: Action) -> Result<()> {
        // This method is now deprecated - action processing is done by ActionDispatcher
        // Keeping for compatibility during transition
        match action {
            Action::Quit => self.should_quit = true,
            _ => {}
        }
        Ok(())
    }
    
    async fn update(&mut self) -> Result<()> {
        // Update current widget
        if let Some(widget) = self.router.get_current_widget_mut() {
            widget.update().await?;
        }
        Ok(())
    }
    
    fn render(&mut self) -> Result<()> {
        let current_view_name = self.router.current_view_name().to_string();
        
        // Get current theme
        let theme = self.theme_manager.current_theme()
            .unwrap_or(&Theme::default())
            .clone();
        
        let header_style = theme.styles.header;
        let status_bar_style = theme.styles.status_bar;
        
        // Get data from state before drawing
        let rt = tokio::runtime::Handle::current();
        let (connection_status, data_freshness, breadcrumb, system_status) = rt.block_on(async {
            let connection = self.state.connection_status().await;
            let freshness = self.state.data_freshness_status().await;
            let breadcrumb = self.router.get_breadcrumb();
            let system = self.state.get_system_status().await;
            (connection, freshness, breadcrumb, system)
        });
        
        self.terminal.draw(|frame| {
            let size = frame.area();
            
            // Main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Length(2), // Breadcrumb
                    Constraint::Min(0),    // Main content
                    Constraint::Length(3), // Status bar
                ])
                .split(size);
            
            // Render header with system info
            let header_text = format!(
                "工作流工具包 TUI v1.0.0 | CPU: {:.1}% | 内存: {:.1}% | 活跃工作流: {}",
                system_status.cpu_usage,
                system_status.memory_usage,
                system_status.active_workflows
            );
            let title = Paragraph::new(header_text)
                .style(header_style)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(title, chunks[0]);
            
            // Render breadcrumb navigation
            let breadcrumb_text = if breadcrumb.len() > 1 {
                breadcrumb.join(" > ")
            } else {
                current_view_name.clone()
            };
            let breadcrumb_widget = Paragraph::new(breadcrumb_text)
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::BOTTOM));
            frame.render_widget(breadcrumb_widget, chunks[1]);
            
            // Render main content
            if let Some(widget) = self.router.get_current_widget_mut() {
                // Use async block to render widget
                let rt = tokio::runtime::Handle::current();
                let _ = rt.block_on(async {
                    widget.render(frame, chunks[2]).await
                });
            } else {
                // Fallback content when no widget is available
                let content = Paragraph::new(format!(
                    "TUI核心框架已建立\n\n当前视图: {}\n\n使用F1-F6切换视图\nCtrl+Q退出\n\n数据状态: {}\n连接状态: {}",
                    current_view_name,
                    data_freshness,
                    connection_status
                ))
                .block(Block::default().borders(Borders::ALL).title("主内容区"))
                .wrap(Wrap { trim: true });
                frame.render_widget(content, chunks[2]);
            }
            
            // Render enhanced status bar
            let status_text = format!(
                "视图: {} | 快捷键: F1-F6切换视图, Ctrl+Q退出 | 连接: {} | 数据: {} | 健康: {:?}",
                current_view_name,
                connection_status,
                data_freshness,
                system_status.system_health
            );
            
            let status_style = match system_status.system_health {
                SystemHealth::Healthy => status_bar_style,
                SystemHealth::Warning => status_bar_style.fg(Color::Yellow),
                SystemHealth::Critical => status_bar_style.fg(Color::Red),
            };
            
            let status = Paragraph::new(status_text)
                .style(status_style)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(status, chunks[3]);
        })?;
        
        Ok(())
    }
}

impl Drop for TuiApp {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            current_view: ViewType::WorkflowList,
            view_stack: Vec::new(),
            widgets: std::collections::HashMap::new(),
        }
    }
    
    /// Navigate to a specific view, managing the view stack
    pub fn navigate_to(&mut self, view: ViewType) {
        if view != self.current_view {
            // Notify current widget it's being deactivated
            if let Some(widget) = self.widgets.get_mut(&self.current_view) {
                let _ = tokio::runtime::Handle::current().block_on(async {
                    widget.on_deactivate().await
                });
            }
            
            self.view_stack.push(self.current_view.clone());
            self.current_view = view.clone();
            
            // Notify new widget it's being activated
            if let Some(widget) = self.widgets.get_mut(&view) {
                let _ = tokio::runtime::Handle::current().block_on(async {
                    widget.on_activate().await
                });
            }
            
            tracing::debug!("Navigated to view: {:?}", view);
        }
    }
    
    /// Go back to the previous view in the stack
    pub fn go_back(&mut self) {
        if let Some(previous_view) = self.view_stack.pop() {
            // Notify current widget it's being deactivated
            if let Some(widget) = self.widgets.get_mut(&self.current_view) {
                let _ = tokio::runtime::Handle::current().block_on(async {
                    widget.on_deactivate().await
                });
            }
            
            self.current_view = previous_view.clone();
            
            // Notify previous widget it's being reactivated
            if let Some(widget) = self.widgets.get_mut(&previous_view) {
                let _ = tokio::runtime::Handle::current().block_on(async {
                    widget.on_activate().await
                });
            }
            
            tracing::debug!("Went back to view: {:?}", previous_view);
        }
    }
    
    /// Go forward in the view stack (if available)
    pub fn go_forward(&mut self) {
        // For now, forward navigation is not implemented
        // This would require maintaining a forward stack as well
        tracing::debug!("Forward navigation not implemented yet");
    }
    
    /// Get the current widget mutably
    pub fn get_current_widget_mut(&mut self) -> Option<&mut Box<dyn Widget>> {
        self.widgets.get_mut(&self.current_view)
    }
    
    /// Get the current widget immutably
    pub fn get_current_widget(&self) -> Option<&Box<dyn Widget>> {
        self.widgets.get(&self.current_view)
    }
    
    /// Get the current view type
    pub fn current_view(&self) -> &ViewType {
        &self.current_view
    }
    
    /// Get the current view name for display
    pub fn current_view_name(&self) -> &str {
        match self.current_view {
            ViewType::WorkflowList => "工作流列表",
            ViewType::ExecutionMonitor => "执行监控",
            ViewType::ToolManager => "工具管理",
            ViewType::PluginManager => "插件管理",
            ViewType::SystemStatus => "系统状态",
            ViewType::LogViewer => "日志查看器",
        }
    }
    
    /// Register a widget for a specific view type
    pub fn register_widget(&mut self, view_type: ViewType, widget: Box<dyn Widget>) {
        tracing::debug!("Registered widget for view: {:?}", view_type);
        self.widgets.insert(view_type, widget);
    }
    
    /// Unregister a widget for a specific view type
    pub fn unregister_widget(&mut self, view_type: &ViewType) -> Option<Box<dyn Widget>> {
        let widget = self.widgets.remove(view_type);
        if widget.is_some() {
            tracing::debug!("Unregistered widget for view: {:?}", view_type);
        }
        widget
    }
    
    /// Check if a widget is registered for a view type
    pub fn has_widget(&self, view_type: &ViewType) -> bool {
        self.widgets.contains_key(view_type)
    }
    
    /// Get the view stack depth
    pub fn stack_depth(&self) -> usize {
        self.view_stack.len()
    }
    
    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        !self.view_stack.is_empty()
    }
    
    /// Get the previous view in the stack (without popping)
    pub fn previous_view(&self) -> Option<&ViewType> {
        self.view_stack.last()
    }
    
    /// Clear the view stack
    pub fn clear_stack(&mut self) {
        self.view_stack.clear();
        tracing::debug!("Cleared view stack");
    }
    
    /// Get all registered view types
    pub fn registered_views(&self) -> Vec<ViewType> {
        self.widgets.keys().cloned().collect()
    }
    
    /// Initialize all widgets
    pub async fn initialize_widgets(&mut self) -> Result<()> {
        for (view_type, widget) in &mut self.widgets {
            widget.initialize().await.map_err(|e| {
                tracing::error!("Failed to initialize widget for {:?}: {}", view_type, e);
                e
            })?;
        }
        tracing::info!("Initialized all widgets");
        Ok(())
    }
    
    /// Cleanup all widgets
    pub async fn cleanup_widgets(&mut self) -> Result<()> {
        for (view_type, widget) in &mut self.widgets {
            widget.cleanup().await.map_err(|e| {
                tracing::error!("Failed to cleanup widget for {:?}: {}", view_type, e);
                e
            })?;
        }
        tracing::info!("Cleaned up all widgets");
        Ok(())
    }
    
    /// Update the current widget
    pub async fn update_current_widget(&mut self) -> Result<()> {
        if let Some(widget) = self.widgets.get_mut(&self.current_view) {
            widget.update().await?;
        }
        Ok(())
    }
    
    /// Handle an event with the current widget
    pub async fn handle_event_with_current_widget(&mut self, event: Event) -> Result<Action> {
        if let Some(widget) = self.widgets.get_mut(&self.current_view) {
            widget.handle_event(event).await
        } else {
            Ok(Action::None)
        }
    }
    
    /// Get breadcrumb navigation path
    pub fn get_breadcrumb(&self) -> Vec<String> {
        let mut breadcrumb = Vec::new();
        
        // Add all views in the stack
        for view in &self.view_stack {
            breadcrumb.push(self.view_type_to_name(view).to_string());
        }
        
        // Add current view
        breadcrumb.push(self.current_view_name().to_string());
        
        breadcrumb
    }
    
    /// Convert view type to display name
    fn view_type_to_name(&self, view_type: &ViewType) -> &str {
        match view_type {
            ViewType::WorkflowList => "工作流列表",
            ViewType::ExecutionMonitor => "执行监控",
            ViewType::ToolManager => "工具管理",
            ViewType::PluginManager => "插件管理",
            ViewType::SystemStatus => "系统状态",
            ViewType::LogViewer => "日志查看器",
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            executions: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            tools: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            plugins: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            system_status: Arc::new(tokio::sync::RwLock::new(SystemStatus::default())),
            logs: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            connection_status: Arc::new(tokio::sync::RwLock::new(ConnectionStatus::Disconnected)),
            last_update: Arc::new(tokio::sync::RwLock::new(chrono::Utc::now())),
        }
    }
    
    /// Refresh all data from backend systems
    pub async fn refresh_all(&self) -> Result<()> {
        tracing::info!("Starting full data refresh");
        
        // Update connection status to connecting
        *self.connection_status.write().await = ConnectionStatus::Connecting;
        
        // Concurrent refresh of all data sources
        let (workflows_result, executions_result, tools_result, plugins_result, system_result, logs_result) = tokio::join!(
            self.refresh_workflows(),
            self.refresh_executions(),
            self.refresh_tools(),
            self.refresh_plugins(),
            self.refresh_system_status(),
            self.refresh_logs()
        );
        
        // Check results and update connection status
        let mut errors = Vec::new();
        
        if let Err(e) = workflows_result {
            errors.push(format!("workflows: {}", e));
        }
        if let Err(e) = executions_result {
            errors.push(format!("executions: {}", e));
        }
        if let Err(e) = tools_result {
            errors.push(format!("tools: {}", e));
        }
        if let Err(e) = plugins_result {
            errors.push(format!("plugins: {}", e));
        }
        if let Err(e) = system_result {
            errors.push(format!("system: {}", e));
        }
        if let Err(e) = logs_result {
            errors.push(format!("logs: {}", e));
        }
        
        if errors.is_empty() {
            *self.connection_status.write().await = ConnectionStatus::Connected;
            *self.last_update.write().await = chrono::Utc::now();
            tracing::info!("Full data refresh completed successfully");
            Ok(())
        } else {
            let error_msg = errors.join(", ");
            *self.connection_status.write().await = ConnectionStatus::Error(error_msg.clone());
            tracing::error!("Data refresh failed: {}", error_msg);
            Err(crate::error::WorkflowError::ValidationError(
                format!("Data refresh failed: {}", error_msg)
            ))
        }
    }
    
    /// Refresh only workflows data
    pub async fn refresh_workflows(&self) -> Result<()> {
        tracing::debug!("Refreshing workflows data");
        
        // Mock implementation - will be replaced with actual backend calls
        let workflows = vec![
            WorkflowInfo {
                name: "数据处理流水线".to_string(),
                version: "1.0.0".to_string(),
                description: Some("处理和转换数据的工作流".to_string()),
                status: WorkflowStatus::Available,
                last_execution: Some(chrono::Utc::now() - chrono::Duration::hours(2)),
                execution_count: 15,
                tags: vec!["数据".to_string(), "ETL".to_string()],
            },
            WorkflowInfo {
                name: "系统监控".to_string(),
                version: "2.1.0".to_string(),
                description: Some("监控系统健康状态".to_string()),
                status: WorkflowStatus::Running,
                last_execution: Some(chrono::Utc::now() - chrono::Duration::minutes(5)),
                execution_count: 142,
                tags: vec!["监控".to_string(), "系统".to_string()],
            },
            WorkflowInfo {
                name: "文件备份".to_string(),
                version: "1.2.1".to_string(),
                description: Some("定期备份重要文件".to_string()),
                status: WorkflowStatus::Completed,
                last_execution: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
                execution_count: 87,
                tags: vec!["备份".to_string(), "文件".to_string()],
            },
        ];
        
        *self.workflows.write().await = workflows;
        tracing::debug!("Workflows data refreshed");
        Ok(())
    }
    
    /// Refresh only executions data
    pub async fn refresh_executions(&self) -> Result<()> {
        tracing::debug!("Refreshing executions data");
        
        // Mock implementation
        let executions = vec![
            ExecutionInfo {
                id: "exec-001".to_string(),
                workflow_name: "系统监控".to_string(),
                status: ExecutionStatus::Running,
                started_at: chrono::Utc::now() - chrono::Duration::minutes(5),
                completed_at: None,
                progress: 0.65,
                current_task: Some("检查磁盘空间".to_string()),
                tasks: vec![
                    TaskInfo {
                        name: "检查CPU使用率".to_string(),
                        status: TaskStatus::Completed,
                        progress: 1.0,
                        started_at: Some(chrono::Utc::now() - chrono::Duration::minutes(5)),
                        completed_at: Some(chrono::Utc::now() - chrono::Duration::minutes(4)),
                        error: None,
                    },
                    TaskInfo {
                        name: "检查内存使用率".to_string(),
                        status: TaskStatus::Completed,
                        progress: 1.0,
                        started_at: Some(chrono::Utc::now() - chrono::Duration::minutes(4)),
                        completed_at: Some(chrono::Utc::now() - chrono::Duration::minutes(3)),
                        error: None,
                    },
                    TaskInfo {
                        name: "检查磁盘空间".to_string(),
                        status: TaskStatus::Running,
                        progress: 0.3,
                        started_at: Some(chrono::Utc::now() - chrono::Duration::minutes(3)),
                        completed_at: None,
                        error: None,
                    },
                ],
            },
            ExecutionInfo {
                id: "exec-002".to_string(),
                workflow_name: "文件备份".to_string(),
                status: ExecutionStatus::Completed,
                started_at: chrono::Utc::now() - chrono::Duration::hours(1),
                completed_at: Some(chrono::Utc::now() - chrono::Duration::minutes(45)),
                progress: 1.0,
                current_task: None,
                tasks: vec![
                    TaskInfo {
                        name: "扫描文件".to_string(),
                        status: TaskStatus::Completed,
                        progress: 1.0,
                        started_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
                        completed_at: Some(chrono::Utc::now() - chrono::Duration::minutes(55)),
                        error: None,
                    },
                    TaskInfo {
                        name: "创建备份".to_string(),
                        status: TaskStatus::Completed,
                        progress: 1.0,
                        started_at: Some(chrono::Utc::now() - chrono::Duration::minutes(55)),
                        completed_at: Some(chrono::Utc::now() - chrono::Duration::minutes(45)),
                        error: None,
                    },
                ],
            },
        ];
        
        *self.executions.write().await = executions;
        tracing::debug!("Executions data refreshed");
        Ok(())
    }
    
    /// Refresh only tools data
    pub async fn refresh_tools(&self) -> Result<()> {
        tracing::debug!("Refreshing tools data");
        
        // Mock implementation - will be replaced with actual tool registry calls
        let tools = vec![
            ToolInfo {
                name: "文件分类器".to_string(),
                version: "1.0.0".to_string(),
                description: "自动分类文件的工具".to_string(),
                plugin_name: Some("file_management".to_string()),
                category: Some("文件管理".to_string()),
                tags: vec!["分类".to_string(), "自动化".to_string()],
                parameters_schema: serde_json::Value::Object(serde_json::Map::new()),
                return_schema: serde_json::Value::Object(serde_json::Map::new()),
                dependencies: Vec::new(),
                version_requirements: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            ToolInfo {
                name: "批量处理器".to_string(),
                version: "1.1.0".to_string(),
                description: "批量处理文件的工具".to_string(),
                plugin_name: Some("file_management".to_string()),
                category: Some("文件管理".to_string()),
                tags: vec!["批量".to_string(), "处理".to_string()],
                parameters_schema: serde_json::Value::Object(serde_json::Map::new()),
                return_schema: serde_json::Value::Object(serde_json::Map::new()),
                dependencies: Vec::new(),
                version_requirements: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];
        
        *self.tools.write().await = tools;
        tracing::debug!("Tools data refreshed");
        Ok(())
    }
    
    /// Refresh only plugins data
    pub async fn refresh_plugins(&self) -> Result<()> {
        tracing::debug!("Refreshing plugins data");
        
        // Mock implementation - will be replaced with actual plugin manager calls
        let plugins = vec![
            PluginInfo {
                name: "file_management".to_string(),
                version: "1.0.0".to_string(),
                description: Some("文件管理插件".to_string()),
                plugin_type: crate::core::PluginType::Native,
                author: Some("Workflow Toolkit Team".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        ];
        
        *self.plugins.write().await = plugins;
        tracing::debug!("Plugins data refreshed");
        Ok(())
    }
    
    /// Refresh only system status data
    pub async fn refresh_system_status(&self) -> Result<()> {
        tracing::debug!("Refreshing system status data");
        
        // Use actual system information where possible
        let status = SystemStatus {
            cpu_usage: self.get_cpu_usage().await,
            memory_usage: self.get_memory_usage().await,
            memory_total: self.get_memory_total().await,
            memory_used: self.get_memory_used().await,
            active_workflows: self.executions.read().await.iter()
                .filter(|e| matches!(e.status, ExecutionStatus::Running | ExecutionStatus::Pending))
                .count() as u32,
            system_health: self.calculate_system_health().await,
            uptime: self.get_system_uptime().await,
            network_status: self.check_network_status().await,
        };
        
        *self.system_status.write().await = status;
        tracing::debug!("System status data refreshed");
        Ok(())
    }
    
    /// Refresh only logs data
    pub async fn refresh_logs(&self) -> Result<()> {
        tracing::debug!("Refreshing logs data");
        
        // Mock implementation - will be replaced with actual log collection
        let logs = vec![
            LogEntry {
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(1),
                level: LogLevel::Info,
                message: "系统监控工作流已启动".to_string(),
                source: Some("workflow_engine".to_string()),
            },
            LogEntry {
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(2),
                level: LogLevel::Debug,
                message: "检查CPU使用率: 25.5%".to_string(),
                source: Some("system_monitor".to_string()),
            },
            LogEntry {
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(3),
                level: LogLevel::Warn,
                message: "内存使用率较高: 78.2%".to_string(),
                source: Some("system_monitor".to_string()),
            },
        ];
        
        *self.logs.write().await = logs;
        tracing::debug!("Logs data refreshed");
        Ok(())
    }
    
    // Getter methods for accessing data
    
    /// Get workflows data (cloned for thread safety)
    pub async fn get_workflows(&self) -> Vec<WorkflowInfo> {
        self.workflows.read().await.clone()
    }
    
    /// Get executions data (cloned for thread safety)
    pub async fn get_executions(&self) -> Vec<ExecutionInfo> {
        self.executions.read().await.clone()
    }
    
    /// Get tools data (cloned for thread safety)
    pub async fn get_tools(&self) -> Vec<ToolInfo> {
        self.tools.read().await.clone()
    }
    
    /// Get plugins data (cloned for thread safety)
    pub async fn get_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.read().await.clone()
    }
    
    /// Get system status data (cloned for thread safety)
    pub async fn get_system_status(&self) -> SystemStatus {
        self.system_status.read().await.clone()
    }
    
    /// Get logs data (cloned for thread safety)
    pub async fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.read().await.clone()
    }
    
    /// Get connection status as a display string
    pub async fn connection_status(&self) -> String {
        match &*self.connection_status.read().await {
            ConnectionStatus::Connected => "已连接".to_string(),
            ConnectionStatus::Connecting => "连接中...".to_string(),
            ConnectionStatus::Disconnected => "未连接".to_string(),
            ConnectionStatus::Error(err) => format!("错误: {}", err),
        }
    }
    
    /// Get the last update timestamp
    pub async fn last_update(&self) -> DateTime<Utc> {
        *self.last_update.read().await
    }
    
    /// Get time since last update
    pub async fn time_since_last_update(&self) -> chrono::Duration {
        chrono::Utc::now() - *self.last_update.read().await
    }
    
    // Data modification methods
    
    /// Add a new workflow
    pub async fn add_workflow(&self, workflow: WorkflowInfo) -> Result<()> {
        self.workflows.write().await.push(workflow);
        tracing::debug!("Added new workflow");
        Ok(())
    }
    
    /// Update an existing workflow
    pub async fn update_workflow(&self, name: &str, workflow: WorkflowInfo) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        if let Some(existing) = workflows.iter_mut().find(|w| w.name == name) {
            *existing = workflow;
            tracing::debug!("Updated workflow: {}", name);
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ValidationError(
                format!("Workflow not found: {}", name)
            ))
        }
    }
    
    /// Remove a workflow
    pub async fn remove_workflow(&self, name: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        let initial_len = workflows.len();
        workflows.retain(|w| w.name != name);
        
        if workflows.len() < initial_len {
            tracing::debug!("Removed workflow: {}", name);
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ValidationError(
                format!("Workflow not found: {}", name)
            ))
        }
    }
    
    /// Add a new execution
    pub async fn add_execution(&self, execution: ExecutionInfo) -> Result<()> {
        self.executions.write().await.push(execution);
        tracing::debug!("Added new execution");
        Ok(())
    }
    
    /// Update an existing execution
    pub async fn update_execution(&self, id: &str, execution: ExecutionInfo) -> Result<()> {
        let mut executions = self.executions.write().await;
        if let Some(existing) = executions.iter_mut().find(|e| e.id == id) {
            *existing = execution;
            tracing::debug!("Updated execution: {}", id);
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ValidationError(
                format!("Execution not found: {}", id)
            ))
        }
    }
    
    /// Add a log entry
    pub async fn add_log_entry(&self, entry: LogEntry) -> Result<()> {
        let mut logs = self.logs.write().await;
        logs.push(entry);
        
        // Keep only the last 1000 log entries to prevent memory issues
        if logs.len() > 1000 {
            let excess = logs.len() - 1000;
            logs.drain(0..excess);
        }
        
        Ok(())
    }
    
    /// Clear all logs
    pub async fn clear_logs(&self) -> Result<()> {
        self.logs.write().await.clear();
        tracing::debug!("Cleared all logs");
        Ok(())
    }
    
    /// Set connection status
    pub async fn set_connection_status(&self, status: ConnectionStatus) {
        *self.connection_status.write().await = status;
    }
    
    // Helper methods for system status
    
    async fn get_cpu_usage(&self) -> f64 {
        // Mock implementation - would use sysinfo or similar
        25.5
    }
    
    async fn get_memory_usage(&self) -> f64 {
        // Mock implementation
        45.2
    }
    
    async fn get_memory_total(&self) -> u64 {
        // Mock implementation
        16_000_000_000
    }
    
    async fn get_memory_used(&self) -> u64 {
        // Mock implementation
        7_200_000_000
    }
    
    async fn calculate_system_health(&self) -> SystemHealth {
        let cpu = self.get_cpu_usage().await;
        let memory = self.get_memory_usage().await;
        
        if cpu > 90.0 || memory > 90.0 {
            SystemHealth::Critical
        } else if cpu > 70.0 || memory > 70.0 {
            SystemHealth::Warning
        } else {
            SystemHealth::Healthy
        }
    }
    
    async fn get_system_uptime(&self) -> Duration {
        // Mock implementation
        Duration::from_secs(86400) // 1 day
    }
    
    async fn check_network_status(&self) -> NetworkStatus {
        // Mock implementation - would check actual network connectivity
        NetworkStatus::Connected
    }
    
    /// Start periodic data refresh
    pub async fn start_periodic_refresh(&self, interval: Duration) -> Result<()> {
        let state = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                if let Err(e) = state.refresh_all().await {
                    tracing::error!("Periodic refresh failed: {}", e);
                }
            }
        });
        
        tracing::info!("Started periodic data refresh with interval: {:?}", interval);
        Ok(())
    }
    
    /// Check if data is stale (older than threshold)
    pub async fn is_data_stale(&self, threshold: chrono::Duration) -> bool {
        self.time_since_last_update().await > threshold
    }
    
    /// Get data freshness status
    pub async fn data_freshness_status(&self) -> String {
        let time_since = self.time_since_last_update().await;
        
        if time_since < chrono::Duration::seconds(30) {
            "最新".to_string()
        } else if time_since < chrono::Duration::minutes(5) {
            format!("{}秒前", time_since.num_seconds())
        } else if time_since < chrono::Duration::hours(1) {
            format!("{}分钟前", time_since.num_minutes())
        } else {
            format!("{}小时前", time_since.num_hours())
        }
    }
}

// Clone implementation for AppState to support Arc sharing
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            workflows: Arc::clone(&self.workflows),
            executions: Arc::clone(&self.executions),
            tools: Arc::clone(&self.tools),
            plugins: Arc::clone(&self.plugins),
            system_status: Arc::clone(&self.system_status),
            logs: Arc::clone(&self.logs),
            connection_status: Arc::clone(&self.connection_status),
            last_update: Arc::clone(&self.last_update),
        }
    }
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_total: 0,
            memory_used: 0,
            active_workflows: 0,
            system_health: SystemHealth::Healthy,
            uptime: Duration::from_secs(0),
            network_status: NetworkStatus::Disconnected,
        }
    }
}

impl EventHandler {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        Self {
            event_sender,
            event_receiver,
            mouse_enabled: true,
            last_resize: None,
            resize_debounce: Duration::from_millis(100),
            key_bindings: KeyBindings::default(),
        }
    }
    
    /// Start the event handling loop
    pub async fn start_event_loop(&mut self) -> Result<()> {
        let sender = self.event_sender.clone();
        
        tokio::spawn(async move {
            loop {
                match event::read() {
                    Ok(terminal_event) => {
                        let tui_event = Self::convert_terminal_event(terminal_event);
                        if let Some(event) = tui_event {
                            if sender.send(event).is_err() {
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
    
    /// Process a TUI event and convert it to an action
    pub async fn process_event(&mut self, event: TuiEvent) -> Result<EventResult> {
        match event {
            TuiEvent::Key(key_event) => self.process_key_event(key_event).await,
            TuiEvent::Mouse(mouse_event) => self.process_mouse_event(mouse_event).await,
            TuiEvent::Resize(width, height) => self.process_resize_event(width, height).await,
            TuiEvent::Tick => Ok(EventResult::Consumed(Action::None)),
            TuiEvent::Quit => Ok(EventResult::Consumed(Action::Quit)),
            TuiEvent::FocusGained => self.process_focus_event(true).await,
            TuiEvent::FocusLost => self.process_focus_event(false).await,
            TuiEvent::Paste(text) => self.process_paste_event(text).await,
        }
    }
    
    /// Process keyboard events
    async fn process_key_event(&self, key_event: event::KeyEvent) -> Result<EventResult> {
        // Handle global shortcuts first
        if let Some(action) = self.handle_global_shortcuts(&key_event) {
            return Ok(EventResult::Consumed(action));
        }
        
        // Convert key event to action based on context
        let action = self.key_event_to_action(key_event)?;
        Ok(EventResult::Consumed(action))
    }
    
    /// Handle global keyboard shortcuts
    fn handle_global_shortcuts(&self, key_event: &event::KeyEvent) -> Option<Action> {
        let key_combo = (key_event.code, key_event.modifiers);
        
        // Check if this key combination is bound to a global action
        if let Some(action) = self.key_bindings.global_shortcuts.get(&key_combo) {
            return Some(action.clone());
        }
        
        // Fallback to hardcoded shortcuts for compatibility
        self.handle_hardcoded_shortcuts(key_event)
    }
    
    /// Handle hardcoded shortcuts (fallback)
    fn handle_hardcoded_shortcuts(&self, key_event: &event::KeyEvent) -> Option<Action> {
        use event::{KeyCode, KeyModifiers};
        
        match (key_event.code, key_event.modifiers) {
            // Additional shortcuts not in the main bindings
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(Action::StartInput(InputMode::Search)),
            (KeyCode::Char('\\'), KeyModifiers::NONE) => Some(Action::StartInput(InputMode::Filter)),
            (KeyCode::Esc, KeyModifiers::NONE) => Some(Action::ClearFilter),
            
            // List navigation shortcuts
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => Some(Action::SelectNext),
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => Some(Action::SelectPrevious),
            (KeyCode::Char('g'), KeyModifiers::CONTROL) => Some(Action::SelectFirst),
            (KeyCode::Char('G'), KeyModifiers::SHIFT) => Some(Action::SelectLast),
            (KeyCode::Home, KeyModifiers::CONTROL) => Some(Action::SelectFirst),
            (KeyCode::End, KeyModifiers::CONTROL) => Some(Action::SelectLast),
            
            // Quick actions
            (KeyCode::Enter, KeyModifiers::CONTROL) => Some(Action::None), // Context-dependent execute
            (KeyCode::Delete, KeyModifiers::SHIFT) => Some(Action::None), // Context-dependent delete
            (KeyCode::F(2), KeyModifiers::SHIFT) => Some(Action::None), // Context-dependent rename
            
            // View toggles
            (KeyCode::Char('d'), KeyModifiers::ALT) => Some(Action::ToggleDetails),
            (KeyCode::F(9), KeyModifiers::NONE) => Some(Action::ToggleDetails),
            
            // System shortcuts
            (KeyCode::F(10), KeyModifiers::NONE) => Some(Action::ShowSystemDetails),
            (KeyCode::F(11), KeyModifiers::NONE) => Some(Action::ShowSettings),
            
            // Log shortcuts
            (KeyCode::Char('l'), KeyModifiers::ALT) => Some(Action::ShowLogs),
            (KeyCode::F(8), KeyModifiers::NONE) => Some(Action::ShowLogs),
            
            // Workflow shortcuts
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => Some(Action::CreateWorkflow),
            (KeyCode::F(7), KeyModifiers::NONE) => Some(Action::CreateWorkflow),
            
            // Plugin shortcuts
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => Some(Action::Navigate(ViewType::PluginManager)),
            (KeyCode::Char('i'), KeyModifiers::CONTROL) => Some(Action::StartInput(InputMode::PluginUrl)),
            
            // Tool shortcuts
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => Some(Action::Navigate(ViewType::ToolManager)),
            
            // Sort shortcuts
            (KeyCode::Char('1'), KeyModifiers::ALT) => Some(Action::Sort(SortOrder::NameAsc)),
            (KeyCode::Char('2'), KeyModifiers::ALT) => Some(Action::Sort(SortOrder::DateAsc)),
            (KeyCode::Char('3'), KeyModifiers::ALT) => Some(Action::Sort(SortOrder::StatusAsc)),
            
            // Theme shortcuts
            (KeyCode::F(12), KeyModifiers::SHIFT) => Some(Action::ChangeTheme("dark".to_string())),
            (KeyCode::F(12), KeyModifiers::ALT) => Some(Action::ChangeTheme("light".to_string())),
            
            _ => None,
        }
    }
    
    /// Convert key event to action
    fn key_event_to_action(&self, key_event: event::KeyEvent) -> Result<Action> {
        use event::KeyCode;
        
        match key_event.code {
            // Navigation keys
            KeyCode::Up => Ok(Action::None), // Will be handled by widgets
            KeyCode::Down => Ok(Action::None),
            KeyCode::Left => Ok(Action::None),
            KeyCode::Right => Ok(Action::None),
            KeyCode::PageUp => Ok(Action::None),
            KeyCode::PageDown => Ok(Action::None),
            KeyCode::Home => Ok(Action::None),
            KeyCode::End => Ok(Action::None),
            
            // Tab navigation
            KeyCode::Tab => Ok(Action::None), // Focus switching
            KeyCode::BackTab => Ok(Action::None), // Reverse focus switching
            
            // Action keys
            KeyCode::Enter => Ok(Action::None), // Context-dependent action
            KeyCode::Esc => Ok(Action::None), // Back/Cancel action
            KeyCode::Delete => Ok(Action::None),
            KeyCode::Backspace => Ok(Action::None),
            
            // Character input
            KeyCode::Char(c) => {
                // Handle special character combinations
                match c {
                    '/' => Ok(Action::Search(String::new())), // Start search
                    '?' => Ok(Action::None), // Help (reserved)
                    _ => Ok(Action::None), // Regular character input
                }
            }
            
            _ => Ok(Action::None),
        }
    }
    
    /// Process mouse events
    async fn process_mouse_event(&self, mouse_event: event::MouseEvent) -> Result<EventResult> {
        if !self.mouse_enabled {
            return Ok(EventResult::NotHandled);
        }
        
        use event::{MouseEventKind, MouseButton};
        
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Left click - selection or activation
                Ok(EventResult::NotHandled) // Let widgets handle specific click logic
            }
            MouseEventKind::Down(MouseButton::Right) => {
                // Right click - context menu (future implementation)
                Ok(EventResult::NotHandled)
            }
            MouseEventKind::ScrollDown => {
                // Scroll down
                Ok(EventResult::NotHandled) // Let widgets handle scrolling
            }
            MouseEventKind::ScrollUp => {
                // Scroll up
                Ok(EventResult::NotHandled)
            }
            MouseEventKind::Moved => {
                // Mouse movement - hover effects (future implementation)
                Ok(EventResult::NotHandled)
            }
            _ => Ok(EventResult::NotHandled),
        }
    }
    
    /// Process terminal resize events
    async fn process_resize_event(&mut self, width: u16, height: u16) -> Result<EventResult> {
        let now = Instant::now();
        
        // Debounce resize events to avoid excessive redraws
        if let Some(last_resize) = self.last_resize {
            if now.duration_since(last_resize) < self.resize_debounce {
                return Ok(EventResult::NotHandled);
            }
        }
        
        self.last_resize = Some(now);
        
        tracing::debug!("Terminal resized to {}x{}", width, height);
        
        // The resize will be handled by the main render loop
        Ok(EventResult::Consumed(Action::None))
    }
    
    /// Process focus events
    async fn process_focus_event(&self, gained: bool) -> Result<EventResult> {
        if gained {
            tracing::debug!("Terminal focus gained");
            // Refresh data when focus is regained
            Ok(EventResult::Consumed(Action::Refresh))
        } else {
            tracing::debug!("Terminal focus lost");
            Ok(EventResult::NotHandled)
        }
    }
    
    /// Process paste events
    async fn process_paste_event(&self, text: String) -> Result<EventResult> {
        tracing::debug!("Text pasted: {} characters", text.len());
        
        // For now, ignore paste events - will be handled by specific widgets
        // that support text input (like search boxes)
        Ok(EventResult::NotHandled)
    }
    
    /// Receive the next event from the event queue
    pub async fn next_event(&mut self) -> Option<TuiEvent> {
        self.event_receiver.recv().await
    }
    
    /// Check if there are pending events
    pub fn has_pending_events(&self) -> bool {
        !self.event_receiver.is_empty()
    }
    
    /// Enable or disable mouse support
    pub fn set_mouse_enabled(&mut self, enabled: bool) {
        self.mouse_enabled = enabled;
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
    }
    
    /// Get help for current global shortcuts
    pub fn get_global_shortcuts_help(&self) -> Vec<ShortcutHelp> {
        self.key_bindings.get_global_shortcuts_help()
    }
    
    /// Get help for view-specific shortcuts
    pub fn get_view_shortcuts_help(&self, view: &ViewType) -> Vec<ShortcutHelp> {
        self.key_bindings.get_view_shortcuts_help(view)
    }
}

impl Theme {
    pub fn dark() -> Self {
        let colors = ColorScheme {
            primary: Color::Blue,
            secondary: Color::Cyan,
            background: Color::Black,
            surface: Color::DarkGray,
            error: Color::Red,
            warning: Color::Yellow,
            success: Color::Green,
            info: Color::Blue,
            text_primary: Color::White,
            text_secondary: Color::Gray,
            border: Color::Gray,
            highlight: Color::Blue,
        };
        
        let styles = StyleScheme {
            header: Style::default()
                .fg(colors.text_primary)
                .bg(colors.primary)
                .add_modifier(Modifier::BOLD),
            status_bar: Style::default()
                .fg(colors.text_secondary)
                .bg(colors.surface),
            border: Style::default().fg(colors.border),
            highlight: Style::default()
                .fg(colors.text_primary)
                .bg(colors.highlight)
                .add_modifier(Modifier::BOLD),
            error: Style::default()
                .fg(colors.error)
                .add_modifier(Modifier::BOLD),
            warning: Style::default()
                .fg(colors.warning)
                .add_modifier(Modifier::BOLD),
            success: Style::default()
                .fg(colors.success)
                .add_modifier(Modifier::BOLD),
            info: Style::default()
                .fg(colors.info)
                .add_modifier(Modifier::BOLD),
        };
        
        Self {
            name: "Dark".to_string(),
            colors,
            styles,
        }
    }
    
    pub fn header_style(&self) -> Style {
        self.styles.header
    }
    
    pub fn status_bar_style(&self) -> Style {
        self.styles.status_bar
    }
    
    pub fn border_style(&self) -> Style {
        self.styles.border
    }
    
    pub fn highlight_style(&self) -> Style {
        self.styles.highlight
    }
}

/// Action processing result
#[derive(Debug, Clone)]
pub enum ActionResult {
    Success,
    Error(String),
    RequiresConfirmation(String),
    RequiresInput(InputMode, String),
}

impl KeyBindings {
    pub fn default() -> Self {
        let mut global_shortcuts = std::collections::HashMap::new();
        
        // Quit shortcuts
        global_shortcuts.insert((KeyCode::Char('q'), KeyModifiers::CONTROL), Action::Quit);
        global_shortcuts.insert((KeyCode::Char('c'), KeyModifiers::CONTROL), Action::Quit);
        global_shortcuts.insert((KeyCode::Char('d'), KeyModifiers::CONTROL), Action::Quit);
        
        // Function key navigation
        global_shortcuts.insert((KeyCode::F(1), KeyModifiers::NONE), Action::Navigate(ViewType::WorkflowList));
        global_shortcuts.insert((KeyCode::F(2), KeyModifiers::NONE), Action::Navigate(ViewType::ExecutionMonitor));
        global_shortcuts.insert((KeyCode::F(3), KeyModifiers::NONE), Action::Navigate(ViewType::ToolManager));
        global_shortcuts.insert((KeyCode::F(4), KeyModifiers::NONE), Action::Navigate(ViewType::PluginManager));
        global_shortcuts.insert((KeyCode::F(5), KeyModifiers::NONE), Action::Navigate(ViewType::SystemStatus));
        global_shortcuts.insert((KeyCode::F(6), KeyModifiers::NONE), Action::Navigate(ViewType::LogViewer));
        
        // Help
        global_shortcuts.insert((KeyCode::F(1), KeyModifiers::SHIFT), Action::ToggleHelp);
        
        // Refresh
        global_shortcuts.insert((KeyCode::F(12), KeyModifiers::NONE), Action::Refresh);
        global_shortcuts.insert((KeyCode::Char('r'), KeyModifiers::CONTROL), Action::Refresh);
        global_shortcuts.insert((KeyCode::F(5), KeyModifiers::CONTROL), Action::Refresh);
        
        // Navigation
        global_shortcuts.insert((KeyCode::Char('h'), KeyModifiers::CONTROL), Action::GoBack);
        global_shortcuts.insert((KeyCode::Char('l'), KeyModifiers::CONTROL), Action::GoForward);
        global_shortcuts.insert((KeyCode::Backspace, KeyModifiers::ALT), Action::GoBack);
        
        // Search and filter
        global_shortcuts.insert((KeyCode::Char('/'), KeyModifiers::NONE), Action::StartInput(InputMode::Search));
        global_shortcuts.insert((KeyCode::Char('f'), KeyModifiers::CONTROL), Action::StartInput(InputMode::Search));
        global_shortcuts.insert((KeyCode::Char('f'), KeyModifiers::ALT), Action::StartInput(InputMode::Filter));
        global_shortcuts.insert((KeyCode::Char('c'), KeyModifiers::ALT), Action::ClearFilter);
        
        // Focus management
        global_shortcuts.insert((KeyCode::Tab, KeyModifiers::NONE), Action::FocusNext);
        global_shortcuts.insert((KeyCode::Tab, KeyModifiers::SHIFT), Action::FocusPrevious);
        global_shortcuts.insert((KeyCode::BackTab, KeyModifiers::NONE), Action::FocusPrevious);
        
        Self {
            global_shortcuts,
            context_shortcuts: std::collections::HashMap::new(),
        }
    }
    
    /// Get all global shortcuts help
    pub fn get_global_shortcuts_help(&self) -> Vec<ShortcutHelp> {
        vec![
            ShortcutHelp {
                category: "应用程序".to_string(),
                shortcuts: vec![
                    ("Ctrl+Q/C/D".to_string(), "退出应用程序".to_string()),
                    ("F12, Ctrl+R".to_string(), "刷新数据".to_string()),
                    ("Shift+F1".to_string(), "显示/隐藏帮助".to_string()),
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
                    ("Ctrl+H".to_string(), "返回上一视图".to_string()),
                    ("Ctrl+L".to_string(), "前进到下一视图".to_string()),
                ],
            },
            ShortcutHelp {
                category: "搜索和过滤".to_string(),
                shortcuts: vec![
                    ("/".to_string(), "开始搜索".to_string()),
                    ("Ctrl+F".to_string(), "搜索".to_string()),
                    ("Alt+F".to_string(), "过滤".to_string()),
                    ("Alt+C".to_string(), "清除过滤器".to_string()),
                    ("Esc".to_string(), "取消/返回".to_string()),
                ],
            },
            ShortcutHelp {
                category: "焦点和导航".to_string(),
                shortcuts: vec![
                    ("Tab".to_string(), "下一个元素".to_string()),
                    ("Shift+Tab".to_string(), "上一个元素".to_string()),
                    ("↑/↓".to_string(), "列表导航".to_string()),
                    ("Ctrl+G".to_string(), "跳到第一项".to_string()),
                    ("Shift+G".to_string(), "跳到最后一项".to_string()),
                ],
            },
            ShortcutHelp {
                category: "快速操作".to_string(),
                shortcuts: vec![
                    ("Ctrl+N".to_string(), "创建新工作流".to_string()),
                    ("Ctrl+T".to_string(), "工具管理".to_string()),
                    ("Ctrl+P".to_string(), "插件管理".to_string()),
                    ("Alt+D".to_string(), "切换详情视图".to_string()),
                    ("Alt+L".to_string(), "显示日志".to_string()),
                ],
            },
        ]
    }
    
    /// Get shortcuts for a specific view
    pub fn get_view_shortcuts_help(&self, view: &ViewType) -> Vec<ShortcutHelp> {
        match view {
            ViewType::WorkflowList => vec![
                ShortcutHelp {
                    category: "工作流操作".to_string(),
                    shortcuts: vec![
                        ("Enter".to_string(), "执行选中的工作流".to_string()),
                        ("Space".to_string(), "暂停/恢复工作流".to_string()),
                        ("Delete".to_string(), "删除工作流".to_string()),
                        ("F2".to_string(), "重命名工作流".to_string()),
                        ("Ctrl+E".to_string(), "编辑工作流".to_string()),
                    ],
                },
            ],
            ViewType::ExecutionMonitor => vec![
                ShortcutHelp {
                    category: "执行控制".to_string(),
                    shortcuts: vec![
                        ("Space".to_string(), "暂停/恢复执行".to_string()),
                        ("S".to_string(), "停止执行".to_string()),
                        ("C".to_string(), "取消执行".to_string()),
                        ("R".to_string(), "重新启动".to_string()),
                    ],
                },
            ],
            ViewType::ToolManager => vec![
                ShortcutHelp {
                    category: "工具操作".to_string(),
                    shortcuts: vec![
                        ("Enter".to_string(), "执行工具".to_string()),
                        ("I".to_string(), "查看工具信息".to_string()),
                        ("R".to_string(), "刷新工具列表".to_string()),
                    ],
                },
            ],
            ViewType::PluginManager => vec![
                ShortcutHelp {
                    category: "插件操作".to_string(),
                    shortcuts: vec![
                        ("I".to_string(), "安装插件".to_string()),
                        ("U".to_string(), "卸载插件".to_string()),
                        ("R".to_string(), "重新加载插件".to_string()),
                        ("E".to_string(), "启用/禁用插件".to_string()),
                    ],
                },
            ],
            ViewType::SystemStatus => vec![
                ShortcutHelp {
                    category: "系统监控".to_string(),
                    shortcuts: vec![
                        ("R".to_string(), "刷新系统状态".to_string()),
                        ("D".to_string(), "显示详细信息".to_string()),
                        ("H".to_string(), "显示历史数据".to_string()),
                    ],
                },
            ],
            ViewType::LogViewer => vec![
                ShortcutHelp {
                    category: "日志操作".to_string(),
                    shortcuts: vec![
                        ("1-5".to_string(), "按级别过滤日志".to_string()),
                        ("C".to_string(), "清除日志".to_string()),
                        ("E".to_string(), "导出日志".to_string()),
                        ("F".to_string(), "跟踪模式".to_string()),
                    ],
                },
            ],
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl ActionDispatcher {
    pub fn new() -> Self {
        let (action_queue, action_receiver) = mpsc::unbounded_channel();
        Self {
            action_queue,
            action_receiver,
        }
    }
    
    /// Queue an action for processing
    pub fn queue_action(&self, action: Action) -> Result<()> {
        self.action_queue.send(action)
            .map_err(|e| crate::error::WorkflowError::ValidationError(
                format!("Failed to queue action: {}", e)
            ))?;
        Ok(())
    }
    
    /// Get the next action from the queue
    pub async fn next_action(&mut self) -> Option<Action> {
        self.action_receiver.recv().await
    }
    
    /// Check if there are pending actions
    pub fn has_pending_actions(&self) -> bool {
        !self.action_receiver.is_empty()
    }
    
    /// Process an action and return the result
    pub async fn process_action(&self, action: Action, app_state: &mut AppState, router: &mut Router) -> Result<ActionResult> {
        match action {
            Action::None => Ok(ActionResult::Success),
            
            // Navigation actions
            Action::Navigate(view) => {
                router.navigate_to(view);
                Ok(ActionResult::Success)
            }
            Action::GoBack => {
                router.go_back();
                Ok(ActionResult::Success)
            }
            Action::GoForward => {
                // TODO: Implement forward navigation
                Ok(ActionResult::Success)
            }
            
            // System actions
            Action::Quit => Ok(ActionResult::Success), // Handled by main loop
            Action::Refresh => {
                app_state.refresh_all().await?;
                Ok(ActionResult::Success)
            }
            Action::RefreshSystemStatus => {
                app_state.refresh_system_status().await?;
                Ok(ActionResult::Success)
            }
            
            // Workflow actions
            Action::ExecuteWorkflow(name) => {
                tracing::info!("Executing workflow: {}", name);
                // TODO: Implement actual workflow execution
                Ok(ActionResult::Success)
            }
            Action::PauseWorkflow(id) => {
                tracing::info!("Pausing workflow: {:?}", id);
                // TODO: Implement workflow pause
                Ok(ActionResult::Success)
            }
            Action::ResumeWorkflow(id) => {
                tracing::info!("Resuming workflow: {:?}", id);
                // TODO: Implement workflow resume
                Ok(ActionResult::Success)
            }
            Action::StopWorkflow(id) => {
                tracing::info!("Stopping workflow: {:?}", id);
                // TODO: Implement workflow stop
                Ok(ActionResult::RequiresConfirmation(
                    format!("Are you sure you want to stop workflow {:?}?", id)
                ))
            }
            Action::CancelWorkflow(id) => {
                tracing::info!("Cancelling workflow: {:?}", id);
                // TODO: Implement workflow cancellation
                Ok(ActionResult::RequiresConfirmation(
                    format!("Are you sure you want to cancel workflow {:?}?", id)
                ))
            }
            Action::ShowWorkflowDetails(name) => {
                tracing::debug!("Showing details for workflow: {}", name);
                // TODO: Implement workflow details view
                Ok(ActionResult::Success)
            }
            Action::CreateWorkflow => {
                Ok(ActionResult::RequiresInput(
                    InputMode::WorkflowName,
                    "Enter workflow name:".to_string()
                ))
            }
            Action::EditWorkflow(name) => {
                tracing::info!("Editing workflow: {}", name);
                // TODO: Implement workflow editing
                Ok(ActionResult::Success)
            }
            Action::DeleteWorkflow(name) => {
                Ok(ActionResult::RequiresConfirmation(
                    format!("Are you sure you want to delete workflow '{}'?", name)
                ))
            }
            
            // Tool actions
            Action::ExecuteTool(name) => {
                tracing::info!("Executing tool: {}", name);
                // TODO: Implement tool execution
                Ok(ActionResult::Success)
            }
            Action::ShowToolDetails(name) => {
                tracing::debug!("Showing details for tool: {}", name);
                // TODO: Implement tool details view
                Ok(ActionResult::Success)
            }
            Action::RefreshTools => {
                app_state.refresh_tools().await?;
                Ok(ActionResult::Success)
            }
            
            // Plugin actions
            Action::InstallPlugin(url) => {
                tracing::info!("Installing plugin from: {}", url);
                // TODO: Implement plugin installation
                Ok(ActionResult::Success)
            }
            Action::UninstallPlugin(name) => {
                Ok(ActionResult::RequiresConfirmation(
                    format!("Are you sure you want to uninstall plugin '{}'?", name)
                ))
            }
            Action::ReloadPlugin(name) => {
                tracing::info!("Reloading plugin: {}", name);
                // TODO: Implement plugin reload
                Ok(ActionResult::Success)
            }
            Action::EnablePlugin(name) => {
                tracing::info!("Enabling plugin: {}", name);
                // TODO: Implement plugin enable
                Ok(ActionResult::Success)
            }
            Action::DisablePlugin(name) => {
                tracing::info!("Disabling plugin: {}", name);
                // TODO: Implement plugin disable
                Ok(ActionResult::Success)
            }
            Action::ShowPluginDetails(name) => {
                tracing::debug!("Showing details for plugin: {}", name);
                // TODO: Implement plugin details view
                Ok(ActionResult::Success)
            }
            Action::RefreshPlugins => {
                app_state.refresh_plugins().await?;
                Ok(ActionResult::Success)
            }
            
            // UI actions
            Action::Search(query) => {
                tracing::debug!("Starting search with query: {}", query);
                // TODO: Implement search functionality
                Ok(ActionResult::Success)
            }
            Action::Filter(filter) => {
                tracing::debug!("Applying filter: {}", filter);
                // TODO: Implement filtering
                Ok(ActionResult::Success)
            }
            Action::Sort(order) => {
                tracing::debug!("Applying sort order: {:?}", order);
                // TODO: Implement sorting
                Ok(ActionResult::Success)
            }
            Action::ClearFilter => {
                tracing::debug!("Clearing filters");
                // TODO: Implement filter clearing
                Ok(ActionResult::Success)
            }
            Action::ToggleDetails => {
                tracing::debug!("Toggling details view");
                // TODO: Implement details toggle
                Ok(ActionResult::Success)
            }
            Action::ToggleHelp => {
                tracing::debug!("Toggling help view");
                // TODO: Implement help toggle
                Ok(ActionResult::Success)
            }
            
            // Focus and navigation
            Action::FocusNext => {
                tracing::debug!("Moving focus to next element");
                // TODO: Implement focus management
                Ok(ActionResult::Success)
            }
            Action::FocusPrevious => {
                tracing::debug!("Moving focus to previous element");
                // TODO: Implement focus management
                Ok(ActionResult::Success)
            }
            Action::FocusWidget(widget_name) => {
                tracing::debug!("Focusing widget: {}", widget_name);
                // TODO: Implement widget focusing
                Ok(ActionResult::Success)
            }
            
            // List actions
            Action::SelectNext => {
                tracing::debug!("Selecting next item");
                // TODO: Implement list navigation
                Ok(ActionResult::Success)
            }
            Action::SelectPrevious => {
                tracing::debug!("Selecting previous item");
                // TODO: Implement list navigation
                Ok(ActionResult::Success)
            }
            Action::SelectFirst => {
                tracing::debug!("Selecting first item");
                // TODO: Implement list navigation
                Ok(ActionResult::Success)
            }
            Action::SelectLast => {
                tracing::debug!("Selecting last item");
                // TODO: Implement list navigation
                Ok(ActionResult::Success)
            }
            Action::SelectItem(index) => {
                tracing::debug!("Selecting item at index: {}", index);
                // TODO: Implement item selection
                Ok(ActionResult::Success)
            }
            
            // Input actions
            Action::StartInput(mode) => {
                tracing::debug!("Starting input mode: {:?}", mode);
                // TODO: Implement input mode
                Ok(ActionResult::Success)
            }
            Action::ConfirmInput(input) => {
                tracing::debug!("Confirming input: {}", input);
                // TODO: Implement input confirmation
                Ok(ActionResult::Success)
            }
            Action::CancelInput => {
                tracing::debug!("Cancelling input");
                // TODO: Implement input cancellation
                Ok(ActionResult::Success)
            }
            
            // Log actions
            Action::ShowLogs => {
                router.navigate_to(ViewType::LogViewer);
                Ok(ActionResult::Success)
            }
            Action::FilterLogs(level) => {
                tracing::debug!("Filtering logs by level: {:?}", level);
                // TODO: Implement log filtering
                Ok(ActionResult::Success)
            }
            Action::ClearLogs => {
                Ok(ActionResult::RequiresConfirmation(
                    "Are you sure you want to clear all logs?".to_string()
                ))
            }
            Action::ExportLogs => {
                tracing::info!("Exporting logs");
                // TODO: Implement log export
                Ok(ActionResult::Success)
            }
            
            // Error handling
            Action::ShowError(message) => {
                tracing::error!("Showing error: {}", message);
                // TODO: Implement error display
                Ok(ActionResult::Success)
            }
            Action::DismissError => {
                tracing::debug!("Dismissing error");
                // TODO: Implement error dismissal
                Ok(ActionResult::Success)
            }
            
            // Configuration
            Action::ShowSettings => {
                tracing::debug!("Showing settings");
                // TODO: Implement settings view
                Ok(ActionResult::Success)
            }
            Action::ChangeTheme(theme_name) => {
                tracing::info!("Changing theme to: {}", theme_name);
                // TODO: Implement theme changing
                Ok(ActionResult::Success)
            }
            Action::SaveSettings => {
                tracing::info!("Saving settings");
                // TODO: Implement settings save
                Ok(ActionResult::Success)
            }
            
            // System status
            Action::ShowSystemDetails => {
                router.navigate_to(ViewType::SystemStatus);
                Ok(ActionResult::Success)
            }
        }
    }
}

/// Basic workflow list widget (stub implementation)
pub struct WorkflowListWidget {
    workflows: Vec<WorkflowInfo>,
    selected_index: usize,
    filter: String,
}

impl WorkflowListWidget {
    pub fn new() -> Self {
        Self {
            workflows: Vec::new(),
            selected_index: 0,
            filter: String::new(),
        }
    }
}

#[async_trait]
impl Widget for WorkflowListWidget {
    async fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("工作流列表");
        
        let paragraph = Paragraph::new("工作流列表 (待实现)")
            .block(block)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
    
    async fn handle_event(&mut self, event: Event) -> Result<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    Ok(Action::None)
                }
                KeyCode::Down => {
                    if self.selected_index < self.workflows.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                    Ok(Action::None)
                }
                KeyCode::Enter => {
                    if let Some(workflow) = self.workflows.get(self.selected_index) {
                        Ok(Action::ExecuteWorkflow(workflow.name.clone()))
                    } else {
                        Ok(Action::None)
                    }
                }
                _ => Ok(Action::None),
            },
            _ => Ok(Action::None),
        }
    }
    
    async fn update(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn title(&self) -> &str {
        "工作流列表"
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("↑/↓", "选择工作流"),
            ("Enter", "执行工作流"),
            ("F1-F6", "切换视图"),
        ]
    }
}

/// TUI interface trait for compatibility
pub trait TuiInterface: Send + Sync {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn is_running(&self) -> bool;
    fn refresh(&self) -> Result<()>;
}

/// Basic TUI interface implementation
pub struct BasicTuiInterface {
    app: Option<TuiApp>,
    is_running: bool,
}

impl BasicTuiInterface {
    pub fn new() -> Self {
        Self {
            app: None,
            is_running: false,
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        self.app = Some(TuiApp::new().await?);
        Ok(())
    }
    
    pub async fn run(&mut self) -> Result<()> {
        if let Some(ref mut app) = self.app {
            app.run().await
        } else {
            Err(crate::error::WorkflowError::ValidationError(
                "TUI app not initialized".to_string()
            ).into())
        }
    }
}

impl Default for BasicTuiInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiInterface for BasicTuiInterface {
    fn start(&self) -> Result<()> {
        println!("TUI interface starting (use run() method for async operation)");
        Ok(())
    }
    
    fn stop(&self) -> Result<()> {
        println!("TUI interface stopping");
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.is_running
    }
    
    fn refresh(&self) -> Result<()> {
        Ok(())
    }
}