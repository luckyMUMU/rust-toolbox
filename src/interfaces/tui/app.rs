//! Main TUI Application
//! 
//! This module implements the main TUI application that integrates all widgets
//! and provides the complete user interface experience.

use crate::error::Result;
use crate::interfaces::tui::{
    Widget, WidgetId, Theme, ThemeManager, EventHandler, ActionDispatcher,
    widgets::{
        WorkflowListWidget, ExecutionMonitorWidget, LogViewerWidget, 
        ToolManagerWidget, PluginManagerWidget, SystemStatusWidget
    },
    action::{Action, ViewType, ActionResult},
    event::TuiEvent,
    state::SharedAppState,
};
use async_trait::async_trait;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Main TUI Application
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    router: Router,
    shared_state: Arc<SharedAppState>,
    theme_manager: ThemeManager,
    event_handler: EventHandler,
    action_dispatcher: ActionDispatcher,
    pub should_quit: bool,
    tick_rate: Duration,
}

/// Router for managing view navigation and Widget lifecycle
pub struct Router {
    current_view: ViewType,
    view_stack: Vec<ViewType>,
    widgets: HashMap<ViewType, Box<dyn Widget>>,
}

/// Application state manager for data synchronization
pub struct AppState {
    // This will be populated with actual state management
    last_update: Arc<RwLock<Instant>>,
}

impl TuiApp {
    /// Create a new TUI application
    pub async fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        let mut router = Router::new();
        let shared_state = Arc::new(SharedAppState::new());
        
        // Register all widgets with shared state
        router.register_widget(ViewType::WorkflowList, Box::new(WorkflowListWidget::new()));
        router.register_widget(ViewType::ExecutionMonitor, Box::new(ExecutionMonitorWidget::new()));
        router.register_widget(ViewType::LogViewer, Box::new(LogViewerWidget::new()));
        router.register_widget(ViewType::ToolManager, Box::new(ToolManagerWidget::new()));
        router.register_widget(ViewType::PluginManager, Box::new(PluginManagerWidget::new()));
        router.register_widget(ViewType::SystemStatus, Box::new(SystemStatusWidget::new()));
        
        // Initialize all widgets
        router.initialize_widgets().await?;
        
        Ok(Self {
            terminal,
            router,
            shared_state,
            theme_manager: ThemeManager::new(),
            event_handler: EventHandler::new(),
            action_dispatcher: ActionDispatcher::new(),
            should_quit: false,
            tick_rate: Duration::from_millis(16), // 60 FPS
        })
    }
    
    /// Get shared state for widgets
    pub fn shared_state(&self) -> Arc<SharedAppState> {
        Arc::clone(&self.shared_state)
    }
    
    /// Run the TUI application main loop
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Starting TUI application");
        
        // Initialize application
        if let Err(e) = self.initialize().await {
            tracing::error!("Failed to initialize TUI application: {}", e);
            return Err(e);
        }
        
        let mut last_tick = Instant::now();
        let mut error_count = 0;
        const MAX_ERRORS: u32 = 10;
        
        loop {
            // Handle terminal events with error recovery
            match self.handle_events().await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                    error_count = 0; // Reset error count on success
                }
                Err(e) => {
                    error_count += 1;
                    tracing::error!("Event handling error ({}): {}", error_count, e);
                    
                    if error_count >= MAX_ERRORS {
                        tracing::error!("Too many errors, shutting down");
                        break;
                    }
                    
                    // Continue with degraded functionality
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            
            // Periodic updates
            if last_tick.elapsed() >= self.tick_rate {
                if let Err(e) = self.update().await {
                    tracing::warn!("Update error: {}", e);
                }
                last_tick = Instant::now();
            }
            
            // Render interface with error handling
            if let Err(e) = self.render().await {
                tracing::warn!("Render error: {}", e);
                // Try to recover by clearing the terminal
                if let Err(clear_err) = self.terminal.clear() {
                    tracing::error!("Failed to clear terminal: {}", clear_err);
                }
            }
            
            if self.should_quit {
                break;
            }
            
            // Small sleep to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        
        // Graceful shutdown
        self.shutdown().await?;
        
        tracing::info!("TUI application stopped");
        Ok(())
    }
    
    /// Initialize the application
    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing TUI application");
        
        // Set up panic handler for graceful shutdown
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // Restore terminal before panicking
            let _ = disable_raw_mode();
            let _ = execute!(
                io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture
            );
            original_hook(panic_info);
        }));
        
        // Initialize shared state with sample data
        self.populate_sample_data().await?;
        
        // Start background tasks
        self.start_background_tasks().await?;
        
        tracing::info!("TUI application initialized successfully");
        Ok(())
    }
    
    /// Handle terminal events
    async fn handle_events(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        let action = self.handle_key_event(key).await?;
                        self.process_action(action).await?;
                    }
                }
                Event::Resize(width, height) => {
                    tracing::debug!("Terminal resized to {}x{}", width, height);
                    // The terminal will automatically handle the resize
                }
                Event::Mouse(_) => {
                    // Mouse events are not currently handled
                }
                _ => {}
            }
        }
        
        Ok(!self.should_quit)
    }
    
    /// Populate sample data for demonstration
    async fn populate_sample_data(&self) -> Result<()> {
        use crate::interfaces::tui::widgets::workflow_list::{WorkflowInfo, WorkflowStatus};
        use crate::interfaces::tui::widgets::log_viewer::LogEntry;
        use crate::interfaces::tui::action::LogLevel;
        use chrono::Utc;
        
        // Add sample workflows
        let workflows = vec![
            WorkflowInfo {
                name: "数据处理流水线".to_string(),
                version: "1.0.0".to_string(),
                description: Some("处理和转换数据的工作流".to_string()),
                status: WorkflowStatus::Available,
                last_execution: Some(Utc::now() - chrono::Duration::hours(2)),
                execution_count: 15,
                tags: vec!["数据".to_string(), "ETL".to_string()],
                node_count: 5,
                estimated_duration: Some(Duration::from_secs(300)),
                success_rate: Some(0.95),
            },
            WorkflowInfo {
                name: "系统监控".to_string(),
                version: "2.1.0".to_string(),
                description: Some("监控系统健康状态".to_string()),
                status: WorkflowStatus::Running,
                last_execution: Some(Utc::now() - chrono::Duration::minutes(5)),
                execution_count: 142,
                tags: vec!["监控".to_string(), "系统".to_string()],
                node_count: 3,
                estimated_duration: Some(Duration::from_secs(60)),
                success_rate: Some(0.98),
            },
            WorkflowInfo {
                name: "文件备份".to_string(),
                version: "1.2.1".to_string(),
                description: Some("定期备份重要文件".to_string()),
                status: WorkflowStatus::Completed,
                last_execution: Some(Utc::now() - chrono::Duration::hours(1)),
                execution_count: 87,
                tags: vec!["备份".to_string(), "文件".to_string()],
                node_count: 4,
                estimated_duration: Some(Duration::from_secs(1800)),
                success_rate: Some(0.92),
            },
        ];
        
        self.shared_state.set_workflows(workflows).await?;
        
        // Add sample log entries
        let logs = vec![
            LogEntry {
                id: "log-001".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(1),
                level: LogLevel::Info,
                message: "TUI应用程序已启动".to_string(),
                source: Some("tui_app".to_string()),
                execution_id: None,
                workflow_id: None,
                node_id: None,
            },
            LogEntry {
                id: "log-002".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(2),
                level: LogLevel::Debug,
                message: "系统监控工作流已启动".to_string(),
                source: Some("workflow_engine".to_string()),
                execution_id: Some("exec-001".to_string()),
                workflow_id: Some("system_monitor".to_string()),
                node_id: Some("node-001".to_string()),
            },
            LogEntry {
                id: "log-003".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(3),
                level: LogLevel::Warn,
                message: "内存使用率较高: 78.2%".to_string(),
                source: Some("system_monitor".to_string()),
                execution_id: Some("exec-001".to_string()),
                workflow_id: Some("system_monitor".to_string()),
                node_id: Some("node-002".to_string()),
            },
        ];
        
        for log in logs {
            self.shared_state.add_log_entry(log).await?;
        }
        
        // Set initial system status
        use crate::interfaces::tui::state::{SystemStatus, SystemHealth, NetworkStatus};
        let system_status = SystemStatus {
            cpu_usage: 25.5,
            memory_usage: 45.2,
            memory_total: 16_000_000_000,
            memory_used: 7_200_000_000,
            disk_usage: 60.0,
            disk_total: 500_000_000_000,
            disk_used: 300_000_000_000,
            active_workflows: 1,
            system_health: SystemHealth::Healthy,
            uptime: Duration::from_secs(86400),
            network_status: NetworkStatus::Connected,
            load_average: [1.2, 1.5, 1.8],
            process_count: 156,
            thread_count: 892,
        };
        
        self.shared_state.set_system_status(system_status).await?;
        
        tracing::info!("Sample data populated");
        Ok(())
    }
    
    /// Start background tasks
    async fn start_background_tasks(&self) -> Result<()> {
        let shared_state = Arc::clone(&self.shared_state);
        
        // Start periodic data refresh task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = shared_state.refresh_all().await {
                    tracing::error!("Background refresh failed: {}", e);
                }
            }
        });
        
        // Start system monitoring task
        let shared_state_monitor = Arc::clone(&self.shared_state);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // Update system status (mock implementation)
                use crate::interfaces::tui::state::{SystemStatus, SystemHealth, NetworkStatus};
                
                // Simple mock data without rand dependency
                let time_factor = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() % 60) as f64 / 60.0;
                
                let system_status = SystemStatus {
                    cpu_usage: 20.0 + (time_factor * 20.0), // 20-40% varying over time
                    memory_usage: 40.0 + (time_factor * 20.0), // 40-60% varying over time
                    memory_total: 16_000_000_000,
                    memory_used: 7_200_000_000,
                    disk_usage: 60.0,
                    disk_total: 500_000_000_000,
                    disk_used: 300_000_000_000,
                    active_workflows: 1,
                    system_health: SystemHealth::Healthy,
                    uptime: Duration::from_secs(86400),
                    network_status: NetworkStatus::Connected,
                    load_average: [1.2, 1.5, 1.8],
                    process_count: 156,
                    thread_count: 892,
                };
                
                if let Err(e) = shared_state_monitor.set_system_status(system_status).await {
                    tracing::error!("Failed to update system status: {}", e);
                }
            }
        });
        
        tracing::info!("Background tasks started");
        Ok(())
    }
    
    /// Graceful shutdown
    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down TUI application");
        
        // Cleanup widgets
        if let Err(e) = self.router.cleanup_widgets().await {
            tracing::error!("Widget cleanup failed: {}", e);
        }
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        
        tracing::info!("TUI application shutdown complete");
        Ok(())
    }
    
    /// Handle keyboard events
    async fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<Action> {
        // Global shortcuts
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(Action::Quit);
            }
            KeyCode::F(1) => return Ok(Action::Navigate(ViewType::WorkflowList)),
            KeyCode::F(2) => return Ok(Action::Navigate(ViewType::ExecutionMonitor)),
            KeyCode::F(3) => return Ok(Action::Navigate(ViewType::ToolManager)),
            KeyCode::F(4) => return Ok(Action::Navigate(ViewType::PluginManager)),
            KeyCode::F(5) => return Ok(Action::Navigate(ViewType::SystemStatus)),
            KeyCode::F(6) => return Ok(Action::Navigate(ViewType::LogViewer)),
            KeyCode::F(12) => return Ok(Action::Refresh),
            _ => {}
        }
        
        // Delegate to current widget
        if let Some(action) = self.router.handle_event_with_current_widget(Event::Key(key)).await? {
            Ok(action)
        } else {
            Ok(Action::None)
        }
    }
    
    /// Process an action
    async fn process_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                tracing::info!("Quit action received");
                self.should_quit = true;
            }
            Action::Navigate(view) => {
                tracing::debug!("Navigating to view: {:?}", view);
                self.router.navigate_to(view);
            }
            Action::GoBack => {
                tracing::debug!("Going back to previous view");
                self.router.go_back();
            }
            Action::Refresh => {
                tracing::debug!("Refreshing data");
                if let Err(e) = self.shared_state.refresh_all().await {
                    tracing::error!("Failed to refresh data: {}", e);
                    // Continue execution despite refresh failure
                }
                
                // Also refresh current widget
                if let Some(widget) = self.router.get_current_widget_mut() {
                    if let Err(e) = widget.update().await {
                        tracing::warn!("Widget update failed during refresh: {}", e);
                    }
                }
            }
            Action::ExecuteWorkflow(name) => {
                tracing::info!("Executing workflow: {}", name);
                // This would integrate with the actual workflow execution system
                if let Err(e) = self.shared_state.update_workflow_status(&name, 
                    crate::interfaces::tui::widgets::workflow_list::WorkflowStatus::Running).await {
                    tracing::error!("Failed to update workflow status: {}", e);
                }
            }
            Action::ShowError(message) => {
                tracing::error!("Application error: {}", message);
                // In a full implementation, this would show an error dialog
            }
            _ => {
                // Other actions will be handled by specific widgets or action handlers
                tracing::debug!("Unhandled action: {:?}", action);
            }
        }
        Ok(())
    }
    
    /// Update the application state
    async fn update(&mut self) -> Result<()> {
        // Update current widget
        self.router.update_current_widget().await?;
        
        // Record performance metrics
        self.shared_state.record_widget_update("app").await?;
        
        Ok(())
    }
    
    /// Render the interface
    async fn render(&mut self) -> Result<()> {
        let current_theme = self.theme_manager.current_theme()
            .unwrap_or(&Theme::default())
            .clone();
        
        let current_view_name = self.router.current_view_name().to_string();
        
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
            let header_text = format!("工作流工具包 TUI v1.0.0 - {}", current_view_name);
            let header = Paragraph::new(header_text)
                .style(current_theme.styles.header)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);
            
            // Render main content (current widget)
            if let Some(widget) = self.router.get_current_widget_mut() {
                // Use async block to render widget
                let rt = tokio::runtime::Handle::current();
                let _ = rt.block_on(async {
                    widget.render(frame, chunks[1], &current_theme).await
                });
            } else {
                // Fallback content when no widget is available
                let content = Paragraph::new(format!(
                    "当前视图: {}\n\n使用F1-F6切换视图\nCtrl+Q退出",
                    current_view_name
                ))
                .block(Block::default().borders(Borders::ALL).title("主内容区"));
                frame.render_widget(content, chunks[1]);
            }
            
            // Render status bar
            let status_text = format!(
                "视图: {} | F1-F6: 切换视图 | Ctrl+Q: 退出 | ?: 帮助",
                current_view_name
            );
            let status = Paragraph::new(status_text)
                .style(current_theme.styles.status_bar)
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
    /// Create a new router
    pub fn new() -> Self {
        Self {
            current_view: ViewType::WorkflowList,
            view_stack: Vec::new(),
            widgets: HashMap::new(),
        }
    }
    
    /// Register a widget for a specific view type
    pub fn register_widget(&mut self, view_type: ViewType, widget: Box<dyn Widget>) {
        tracing::debug!("Registered widget for view: {:?}", view_type);
        self.widgets.insert(view_type, widget);
    }
    
    /// Navigate to a specific view
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
    
    /// Go back to the previous view
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
    
    /// Get the current widget mutably
    pub fn get_current_widget_mut(&mut self) -> Option<&mut Box<dyn Widget>> {
        self.widgets.get_mut(&self.current_view)
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
    
    /// Initialize all widgets
    pub async fn initialize_widgets(&mut self) -> Result<()> {
        for (view_type, widget) in &mut self.widgets {
            widget.initialize().await.map_err(|e| {
                tracing::error!("Failed to initialize widget for {:?}: {}", view_type, e);
                crate::error::WorkflowError::ValidationError(
                    format!("Widget initialization failed: {}", e)
                )
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
                crate::error::WorkflowError::ValidationError(
                    format!("Widget cleanup failed: {}", e)
                )
            })?;
        }
        tracing::info!("Cleaned up all widgets");
        Ok(())
    }
    
    /// Update the current widget
    pub async fn update_current_widget(&mut self) -> Result<()> {
        if let Some(widget) = self.widgets.get_mut(&self.current_view) {
            widget.update().await.map_err(|e| {
                crate::error::WorkflowError::ValidationError(
                    format!("Widget update failed: {}", e)
                )
            })?;
        }
        Ok(())
    }
    
    /// Handle an event with the current widget
    pub async fn handle_event_with_current_widget(&mut self, event: Event) -> Result<Option<Action>> {
        if let Some(widget) = self.widgets.get_mut(&self.current_view) {
            widget.handle_event(event).await.map_err(|e| {
                crate::error::WorkflowError::ValidationError(
                    format!("Widget event handling failed: {}", e)
                )
            })
        } else {
            Ok(None)
        }
    }
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }
}

/// TUI interface trait for compatibility with the existing system
#[async_trait]
pub trait TuiInterface: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
    async fn refresh(&mut self) -> Result<()>;
}

/// Implementation of TuiInterface for the main TUI app
pub struct MainTuiInterface {
    app: Option<TuiApp>,
    is_running: bool,
    initialization_attempts: u32,
}

impl MainTuiInterface {
    /// Create a new TUI interface
    pub fn new() -> Self {
        Self {
            app: None,
            is_running: false,
            initialization_attempts: 0,
        }
    }
    
    /// Initialize the TUI application with retry logic
    pub async fn initialize(&mut self) -> Result<()> {
        const MAX_ATTEMPTS: u32 = 3;
        
        while self.initialization_attempts < MAX_ATTEMPTS {
            self.initialization_attempts += 1;
            
            match TuiApp::new().await {
                Ok(app) => {
                    self.app = Some(app);
                    tracing::info!("TUI interface initialized successfully on attempt {}", 
                        self.initialization_attempts);
                    return Ok(());
                }
                Err(e) => {
                    tracing::error!("TUI initialization attempt {} failed: {}", 
                        self.initialization_attempts, e);
                    
                    if self.initialization_attempts >= MAX_ATTEMPTS {
                        return Err(crate::error::WorkflowError::ValidationError(
                            format!("Failed to initialize TUI after {} attempts: {}", 
                                MAX_ATTEMPTS, e)
                        ));
                    }
                    
                    // Wait before retrying
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        }
        
        Err(crate::error::WorkflowError::ValidationError(
            "TUI initialization failed".to_string()
        ))
    }
    
    /// Check if the interface is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.app.is_some()
    }
    
    /// Get initialization attempt count
    pub fn initialization_attempts(&self) -> u32 {
        self.initialization_attempts
    }
}

#[async_trait]
impl TuiInterface for MainTuiInterface {
    async fn start(&mut self) -> Result<()> {
        if !self.is_initialized() {
            self.initialize().await?;
        }
        
        if let Some(ref mut app) = self.app {
            self.is_running = true;
            
            // Run with error recovery
            let result = app.run().await;
            
            self.is_running = false;
            
            match result {
                Ok(()) => {
                    tracing::info!("TUI application completed successfully");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("TUI application error: {}", e);
                    
                    // Attempt graceful cleanup
                    if let Err(cleanup_err) = app.shutdown().await {
                        tracing::error!("Cleanup failed: {}", cleanup_err);
                    }
                    
                    Err(e)
                }
            }
        } else {
            Err(crate::error::WorkflowError::ValidationError(
                "TUI app not initialized".to_string()
            ))
        }
    }
    
    async fn stop(&mut self) -> Result<()> {
        if let Some(ref mut app) = self.app {
            app.should_quit = true;
            tracing::info!("TUI stop requested");
        }
        self.is_running = false;
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.is_running
    }
    
    async fn refresh(&mut self) -> Result<()> {
        if let Some(ref mut app) = self.app {
            app.update().await
        } else {
            Ok(())
        }
    }
}

impl Default for MainTuiInterface {
    fn default() -> Self {
        Self::new()
    }
}