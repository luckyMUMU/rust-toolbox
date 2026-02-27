//! TUI Widget System Module
//!
//! This module provides the enhanced Widget system for the TUI interface,
//! including the Widget trait, layout management, and component lifecycle.

pub mod action;
pub mod app;
pub mod backends;
pub mod config;
pub mod error;
pub mod event;
pub mod feedback;
pub mod focus;
pub mod help;
pub mod layout;
pub mod memory;
pub mod monitoring;
pub mod navigation;
pub mod performance;
pub mod platform;
pub mod startup;
pub mod state;
pub mod sync;
pub mod system_monitor;
pub mod theme;
pub mod undo;
pub mod virtualization;
pub mod widget;
pub mod widgets;

// Re-export public types
pub use action::{Action, ActionDispatcher, ActionResult};
pub use app::MainTuiInterface;
pub use backends::{HttpSyncBackend, MemoryCacheBackend, MockSyncBackend};
pub use config::{
    AccessibilityConfig, ConfigPresetManager, InterfaceConfig, KeybindingsConfig, LayoutConfig,
    LayoutDefinition, PerformanceConfig, ResponsiveBreakpoints, ThemeManagerConfig, TuiConfig,
    TuiConfigManager, UserPreferences,
};
pub use error::{
    ErrorContext, ErrorDisplayWidget, ErrorManager, ErrorRecoveryHandler, ErrorSeverity,
    RecoveryStrategy, TuiError,
};
pub use event::{EventHandler, EventResult, TuiEvent};
pub use feedback::{
    ConfirmationAction, ConfirmationDialog, ConfirmationOption, DialogType, FeedbackManager,
    FeedbackWidget, Notification, NotificationType, ProgressIndicator, ProgressStatus,
    StatusMessage, StatusType,
};
pub use focus::{
    FocusCapability, FocusChangeEvent, FocusManager, NavigationConfig, NavigationMode,
};
pub use help::{HelpContent, HelpDisplayMode, HelpSystem, ShortcutInfo};
pub use layout::{LayoutConstraints, LayoutDirection, LayoutManager, LayoutNode};
pub use memory::{
    CleanupReport, LeakAction, MemoryLeakDetection, MemoryLeakDetector, TuiCleanupScheduler,
    TuiMemoryConfig, TuiMemoryManager, TuiMemoryStatistics, WidgetMemoryUsage,
};
pub use monitoring::{
    AlertSeverity, AlertType, DebugInfo, ExportFormat, LogLevel, MonitoringConfig,
    PerformanceAlert, PerformanceBottleneck, PerformanceRecommendation, PerformanceReport,
    PerformanceSnapshot, PerformanceSummary, PerformanceTrends, ProfilingData, ProfilingSession,
    TuiPerformanceMonitor,
};
pub use navigation::{EscKeyBehavior, ModalDialog, NavigationStack, NavigationTrigger};
pub use performance::{
    DirtyRegion, FrameRateMonitor, FrameRateStats, OptimizationPriority, RenderOperation,
    RenderingConfig, RenderingOptimizationReport, RenderingOptimizationType,
    RenderingPerformanceManager, RenderingRecommendation,
};
pub use platform::{
    ColorSupport, CompatibilitySettings, Platform, PlatformCompatibilityReport, PlatformConfig,
    PlatformManager, PlatformOptimizations, TerminalCapabilities, TerminalInfo,
    TerminalTestResults, TerminalTester, TestResult,
};
pub use startup::{
    ComponentInfo, ComponentInitializer, InitializationPriority, InitializationStatus,
    LazyComponentLoader, OptimizationPriority as StartupOptimizationPriority, StartupConfig,
    StartupMetrics, StartupOptimization, StartupOptimizationType, StartupPerformanceManager,
    StartupPhase,
};
pub use state::{
    ConnectionStatus, SharedAppState, StateChangeEvent, StateSubscriber, SystemStatus,
};
pub use sync::{
    CacheBackend, CacheMetadata, CachePriority, CompactionResult, DataSyncManager,
    OfflineCacheManager, SyncBackend, SyncConfig, SyncEvent, SyncMetrics, SyncStatus,
};
pub use system_monitor::{
    CpuInfo, DiskInfo, MemoryInfo, NetworkInfo, ProcessInfo, ProcessSortBy, SystemMonitor,
};
pub use theme::{ColorScheme, StyleScheme, Theme, ThemeManager};
pub use undo::{
    ErrorReport, ErrorReportManager, ErrorReportType, ErrorReportWidget, OperationResult,
    OperationType, UndoHistoryWidget, UndoManager, UndoableOperation,
};
pub use virtualization::{
    MockDataProvider, VirtualDataProvider, VirtualItem, VirtualListWidget, VirtualTableWidget,
    VirtualizationConfig, VirtualizationMetrics,
};
pub use widget::{BaseWidget, Widget, WidgetContext, WidgetError, WidgetId, WidgetState};
pub use widgets::{
    ExecutionMonitorWidget, LogViewerWidget, SyncStatusWidget, SystemStatusWidget,
    ToolManagerWidget, WorkflowListWidget,
};

use crate::error::Result;
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
    pub async fn get_widget(&self, _id: &WidgetId) -> Option<Box<dyn Widget>> {
        let _widgets = self.widgets.read().await;
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
                crate::error::WorkflowError::ValidationError(format!(
                    "Widget initialization failed: {}",
                    e
                ))
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
                crate::error::WorkflowError::ValidationError(format!(
                    "Widget cleanup failed: {}",
                    e
                ))
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
                    crate::error::WorkflowError::ValidationError(format!(
                        "Widget update failed: {}",
                        e
                    ))
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
    #[allow(dead_code)]
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
                "TUI app not initialized".to_string(),
            ))
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
