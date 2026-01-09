//! TUI interface implementation
//! 
//! This module provides the TUI (Terminal User Interface) implementation
//! for the workflow toolkit using ratatui and crossterm.

use crate::error::Result;
use crate::workflow::WorkflowExecution;
use crate::core::{ToolInfo, WorkflowId, PluginInfo};
use crate::plugins::manager::PluginManager;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::io;

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
use async_trait::async_trait;

/// Main TUI application structure with ratatui integration
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    router: Router,
    state: AppState,
    theme: Theme,
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
    async fn render(&mut self, frame: &mut Frame, area: Rect);
    async fn handle_event(&mut self, event: Event) -> Result<Action>;
    async fn update(&mut self) -> Result<()>;
    fn title(&self) -> &str;
    fn help_text(&self) -> Vec<(&str, &str)>; // (key, description)
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
            router: Router::new(),
            state: AppState::new(),
            theme: Theme::default(),
            event_handler: EventHandler::new(),
            action_dispatcher: ActionDispatcher::new(),
            should_quit: false,
            tick_rate: Duration::from_millis(16), // 60 FPS
        })
    }
    
    /// Run the TUI application main loop
    pub async fn run(&mut self) -> Result<()> {
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
        let header_style = self.theme.header_style();
        let status_bar_style = self.theme.status_bar_style();
        
        // Get connection status before drawing
        let rt = tokio::runtime::Handle::current();
        let connection_status = rt.block_on(async {
            self.state.connection_status().await
        });
        
        self.terminal.draw(|frame| {
            let size = frame.area();
            
            // Main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Main content
                    Constraint::Length(3), // Status bar
                ])
                .split(size);
            
            // Render header
            let title = Paragraph::new("工作流工具包 TUI v1.0.0")
                .style(header_style)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(title, chunks[0]);
            
            // Render main content (placeholder for now)
            let content = Paragraph::new("TUI核心框架已建立\n\n使用F1-F6切换视图\nCtrl+Q退出")
                .block(Block::default().borders(Borders::ALL).title("主内容区"));
            frame.render_widget(content, chunks[1]);
            
            // Render status bar
            let status_text = format!(
                "视图: {} | 快捷键: F1-F6切换视图, Ctrl+Q退出 | 状态: {}",
                current_view_name,
                connection_status
            );
            
            let status = Paragraph::new(status_text)
                .style(status_bar_style)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(status, chunks[2]);
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
    
    pub fn navigate_to(&mut self, view: ViewType) {
        if view != self.current_view {
            self.view_stack.push(self.current_view.clone());
            self.current_view = view;
        }
    }
    
    pub fn go_back(&mut self) {
        if let Some(previous_view) = self.view_stack.pop() {
            self.current_view = previous_view;
        }
    }
    
    pub fn get_current_widget_mut(&mut self) -> Option<&mut Box<dyn Widget>> {
        self.widgets.get_mut(&self.current_view)
    }
    
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
    
    pub fn register_widget(&mut self, view_type: ViewType, widget: Box<dyn Widget>) {
        self.widgets.insert(view_type, widget);
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
    
    pub async fn refresh_all(&self) -> Result<()> {
        // Concurrent refresh of all data
        let (workflows_result, executions_result, tools_result, plugins_result, system_result) = tokio::join!(
            self.refresh_workflows(),
            self.refresh_executions(),
            self.refresh_tools(),
            self.refresh_plugins(),
            self.refresh_system_status()
        );
        
        workflows_result?;
        executions_result?;
        tools_result?;
        plugins_result?;
        system_result?;
        
        *self.last_update.write().await = chrono::Utc::now();
        Ok(())
    }
    
    pub async fn get_workflows(&self) -> Vec<WorkflowInfo> {
        self.workflows.read().await.clone()
    }
    
    pub async fn get_executions(&self) -> Vec<ExecutionInfo> {
        self.executions.read().await.clone()
    }
    
    pub async fn get_system_status(&self) -> SystemStatus {
        self.system_status.read().await.clone()
    }
    
    pub async fn connection_status(&self) -> String {
        match &*self.connection_status.read().await {
            ConnectionStatus::Connected => "已连接".to_string(),
            ConnectionStatus::Connecting => "连接中...".to_string(),
            ConnectionStatus::Disconnected => "未连接".to_string(),
            ConnectionStatus::Error(err) => format!("错误: {}", err),
        }
    }
    
    async fn refresh_workflows(&self) -> Result<()> {
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
        ];
        
        *self.workflows.write().await = workflows;
        Ok(())
    }
    
    async fn refresh_executions(&self) -> Result<()> {
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
                ],
            },
        ];
        
        *self.executions.write().await = executions;
        Ok(())
    }
    
    async fn refresh_tools(&self) -> Result<()> {
        // Mock implementation
        Ok(())
    }
    
    async fn refresh_plugins(&self) -> Result<()> {
        // Mock implementation
        Ok(())
    }
    
    async fn refresh_system_status(&self) -> Result<()> {
        // Mock implementation using system information
        let status = SystemStatus {
            cpu_usage: 25.5,
            memory_usage: 45.2,
            memory_total: 16_000_000_000,
            memory_used: 7_200_000_000,
            active_workflows: self.executions.read().await.len() as u32,
            system_health: SystemHealth::Healthy,
            uptime: Duration::from_secs(86400), // 1 day
            network_status: NetworkStatus::Connected,
        };
        
        *self.system_status.write().await = status;
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