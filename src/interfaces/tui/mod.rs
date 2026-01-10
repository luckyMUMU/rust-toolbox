//! TUI Widget System Module
//! 
//! This module provides the enhanced Widget system for the TUI interface,
//! including the Widget trait, layout management, and component lifecycle.

pub mod widget;
pub mod layout;
pub mod theme;
pub mod event;
pub mod action;
pub mod widgets;
pub mod system_monitor;
pub mod app;
pub mod state;

// Re-export public types
pub use widget::{Widget, WidgetId, WidgetState, WidgetContext, WidgetError, BaseWidget};
pub use layout::{LayoutManager, LayoutConstraints, LayoutDirection, LayoutNode};
pub use theme::{Theme, ColorScheme, StyleScheme, ThemeManager};
pub use event::{EventHandler, TuiEvent, EventResult};
pub use action::{Action, ActionDispatcher, ActionResult};
pub use widgets::{WorkflowListWidget, ExecutionMonitorWidget, LogViewerWidget, ToolManagerWidget, SystemStatusWidget};
pub use system_monitor::{SystemMonitor, CpuInfo, MemoryInfo, DiskInfo, NetworkInfo, ProcessInfo, ProcessSortBy};
pub use app::{MainTuiInterface};
pub use state::{SharedAppState, StateChangeEvent, StateSubscriber, SystemStatus, ConnectionStatus};

use crate::error::Result;
use async_trait::async_trait;
use ratatui::{Frame, layout::Rect};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Widget registry for managing all registered widgets
pub struct WidgetRegistry {
    widgets: Arc<RwLock<HashMap<WidgetId, Box<dyn Widget>>>>,
    layout_manager: Arc<RwLock<LayoutManager>>,
    theme: Arc<RwLock<Theme>>,
}

impl WidgetRegistry {
    /// Create a new widget registry
    pub fn new() -> Self {
        Self {
            widgets: Arc::new(RwLock::new(HashMap::new())),
            layout_manager: Arc::new(RwLock::new(LayoutManager::new())),
            theme: Arc::new(RwLock::new(Theme::default())),
        }
    }
    
    /// Register a widget with the registry
    pub async fn register_widget(&self, id: WidgetId, widget: Box<dyn Widget>) -> Result<()> {
        let mut widgets = self.widgets.write().await;
        widgets.insert(id, widget);
        Ok(())
    }
    
    /// Unregister a widget from the registry
    pub async fn unregister_widget(&self, id: &WidgetId) -> Result<Option<Box<dyn Widget>>> {
        let mut widgets = self.widgets.write().await;
        Ok(widgets.remove(id))
    }
    
    /// Get a widget by ID
    pub async fn get_widget(&self, id: &WidgetId) -> Option<Box<dyn Widget>> {
        let widgets = self.widgets.read().await;
        // Note: This is a simplified implementation
        // In practice, we'd need to handle borrowing differently
        None
    }
    
    /// Initialize all registered widgets
    pub async fn initialize_all(&self) -> Result<()> {
        let mut widgets = self.widgets.write().await;
        for (id, widget) in widgets.iter_mut() {
            widget.initialize().await.map_err(|e| {
                tracing::error!("Failed to initialize widget {}: {}", id, e);
                crate::error::WorkflowError::ValidationError(format!("Widget initialization failed: {}", e))
            })?;
        }
        tracing::info!("Initialized all widgets");
        Ok(())
    }
    
    /// Cleanup all registered widgets
    pub async fn cleanup_all(&self) -> Result<()> {
        let mut widgets = self.widgets.write().await;
        for (id, widget) in widgets.iter_mut() {
            widget.cleanup().await.map_err(|e| {
                tracing::error!("Failed to cleanup widget {}: {}", id, e);
                crate::error::WorkflowError::ValidationError(format!("Widget cleanup failed: {}", e))
            })?;
        }
        tracing::info!("Cleaned up all widgets");
        Ok(())
    }
    
    /// Update all widgets that need periodic updates
    pub async fn update_all(&self) -> Result<()> {
        let mut widgets = self.widgets.write().await;
        for (id, widget) in widgets.iter_mut() {
            if widget.needs_update() {
                widget.update().await.map_err(|e| {
                    tracing::error!("Failed to update widget {}: {}", id, e);
                    crate::error::WorkflowError::ValidationError(format!("Widget update failed: {}", e))
                })?;
            }
        }
        Ok(())
    }
    
    /// Get the layout manager
    pub async fn layout_manager(&self) -> Arc<RwLock<LayoutManager>> {
        Arc::clone(&self.layout_manager)
    }
    
    /// Get the theme
    pub async fn theme(&self) -> Arc<RwLock<Theme>> {
        Arc::clone(&self.theme)
    }
    
    /// Set a new theme
    pub async fn set_theme(&self, theme: Theme) -> Result<()> {
        let mut current_theme = self.theme.write().await;
        *current_theme = theme;
        tracing::info!("Theme updated");
        Ok(())
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// TUI Application with enhanced widget system
pub struct EnhancedTuiApp {
    widget_registry: WidgetRegistry,
    theme_manager: ThemeManager,
    event_handler: EventHandler,
    action_dispatcher: ActionDispatcher,
    should_quit: bool,
}

impl EnhancedTuiApp {
    /// Create a new TUI application
    pub async fn new() -> Result<Self> {
        Ok(Self {
            widget_registry: WidgetRegistry::new(),
            theme_manager: ThemeManager::new(),
            event_handler: EventHandler::new(),
            action_dispatcher: ActionDispatcher::new(),
            should_quit: false,
        })
    }
    
    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Initialize widgets
        self.widget_registry.initialize_all().await?;
        
        // Start event handling
        self.event_handler.start_event_loop().await?;
        
        // Main loop would go here
        tracing::info!("TUI application started with enhanced widget system");
        
        // Cleanup
        self.widget_registry.cleanup_all().await?;
        
        Ok(())
    }
    
    /// Get the widget registry
    pub fn widget_registry(&self) -> &WidgetRegistry {
        &self.widget_registry
    }
    
    /// Get the theme manager
    pub fn theme_manager(&self) -> &ThemeManager {
        &self.theme_manager
    }
    
    /// Get the event handler
    pub fn event_handler(&self) -> &EventHandler {
        &self.event_handler
    }
    
    /// Get the action dispatcher
    pub fn action_dispatcher(&self) -> &ActionDispatcher {
        &self.action_dispatcher
    }
}

/// TUI interface trait for compatibility
pub trait EnhancedTuiInterface: Send + Sync {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn is_running(&self) -> bool;
    fn refresh(&self) -> Result<()>;
}

/// Basic TUI interface implementation
pub struct BasicTuiInterface {
    app: Option<EnhancedTuiApp>,
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
        self.app = Some(EnhancedTuiApp::new().await?);
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

impl EnhancedTuiInterface for BasicTuiInterface {
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