//! Action system for TUI widgets
//! 
//! This module defines the action types and dispatcher for handling user interactions.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::sync::mpsc;

/// Actions that can be triggered by TUI events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// No action
    None,
    
    /// Application control
    Quit,
    Refresh,
    
    // Navigation actions
    Navigate(ViewType),
    Back,
    GoBack,
    GoForward,
    
    // Widget focus actions
    FocusNext,
    FocusPrevious,
    FocusWidget(String),
    
    // List navigation actions
    SelectNext,
    SelectPrevious,
    SelectFirst,
    SelectLast,
    SelectItem(usize),
    
    // Input actions
    StartInput(InputMode),
    ConfirmInput(String),
    CancelInput,
    
    // UI actions
    Search(String),
    Filter(String),
    Sort(SortOrder),
    ClearFilter,
    ToggleDetails,
    ToggleHelp,
    
    // Workflow actions
    ExecuteWorkflow(String),
    PauseWorkflow(String),
    ResumeWorkflow(String),
    StopWorkflow(String),
    CancelWorkflow(String),
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
    TogglePlugin(String),
    ShowPluginDetails(String),
    ShowPluginFilters,
    ShowDependencyGraph,
    RefreshPlugins,
    
    // System actions
    RefreshSystemStatus,
    ShowSystemDetails,
    
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
    
    // Custom actions (for widget-specific behavior)
    Custom(String, serde_json::Value),
}

/// Different views in the TUI
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViewType {
    WorkflowList,
    ExecutionMonitor,
    ToolManager,
    PluginManager,
    SystemStatus,
    LogViewer,
}

/// Input modes for text input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    Search,
    Filter,
    Command,
    WorkflowName,
    PluginUrl,
    Custom(String),
}

/// Sort order for lists
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
    StatusAsc,
    StatusDesc,
    Custom(String),
}

/// Log levels for filtering
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Action processing result
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Action was processed successfully
    Success,
    /// Action processing failed with an error
    Error(String),
    /// Action requires user confirmation
    RequiresConfirmation(String),
    /// Action requires user input
    RequiresInput(InputMode, String),
    /// Action was ignored or not applicable
    Ignored,
    /// Action should be forwarded to another handler
    Forward(Action),
}

/// Action priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Action with metadata
#[derive(Debug, Clone)]
pub struct ActionWithMetadata {
    pub action: Action,
    pub priority: ActionPriority,
    pub timestamp: std::time::Instant,
    pub source: Option<String>,
    pub context: std::collections::HashMap<String, String>,
}

impl ActionWithMetadata {
    /// Create a new action with metadata
    pub fn new(action: Action) -> Self {
        Self {
            action,
            priority: ActionPriority::Normal,
            timestamp: std::time::Instant::now(),
            source: None,
            context: std::collections::HashMap::new(),
        }
    }
    
    /// Set the action priority
    pub fn with_priority(mut self, priority: ActionPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set the action source
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }
    
    /// Add context information
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
    
    /// Get the age of this action
    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
}

/// Action dispatcher for handling actions
pub struct ActionDispatcher {
    action_sender: mpsc::UnboundedSender<ActionWithMetadata>,
    action_receiver: mpsc::UnboundedReceiver<ActionWithMetadata>,
    handlers: std::collections::HashMap<String, Box<dyn ActionHandler>>,
}

/// Trait for handling specific types of actions
#[async_trait::async_trait]
pub trait ActionHandler: Send + Sync {
    /// Handle an action and return the result
    async fn handle(&mut self, action: &Action) -> Result<ActionResult>;
    
    /// Check if this handler can process the given action
    fn can_handle(&self, action: &Action) -> bool;
    
    /// Get the handler's name
    fn name(&self) -> &str;
    
    /// Get the handler's priority
    fn priority(&self) -> ActionPriority {
        ActionPriority::Normal
    }
}

impl ActionDispatcher {
    /// Create a new action dispatcher
    pub fn new() -> Self {
        let (action_sender, action_receiver) = mpsc::unbounded_channel();
        Self {
            action_sender,
            action_receiver,
            handlers: std::collections::HashMap::new(),
        }
    }
    
    /// Register an action handler
    pub fn register_handler(&mut self, name: String, handler: Box<dyn ActionHandler>) {
        tracing::debug!("Registered action handler: {}", name);
        self.handlers.insert(name, handler);
    }
    
    /// Unregister an action handler
    pub fn unregister_handler(&mut self, name: &str) -> Option<Box<dyn ActionHandler>> {
        let handler = self.handlers.remove(name);
        if handler.is_some() {
            tracing::debug!("Unregistered action handler: {}", name);
        }
        handler
    }
    
    /// Dispatch an action for processing
    pub fn dispatch(&self, action: Action) -> Result<()> {
        let action_with_metadata = ActionWithMetadata::new(action);
        self.action_sender.send(action_with_metadata)
            .map_err(|e| crate::error::WorkflowError::ValidationError(
                format!("Failed to dispatch action: {}", e)
            ))?;
        Ok(())
    }
    
    /// Dispatch an action with metadata
    pub fn dispatch_with_metadata(&self, action: ActionWithMetadata) -> Result<()> {
        self.action_sender.send(action)
            .map_err(|e| crate::error::WorkflowError::ValidationError(
                format!("Failed to dispatch action: {}", e)
            ))?;
        Ok(())
    }
    
    /// Process the next action in the queue
    pub async fn process_next(&mut self) -> Option<ActionResult> {
        if let Some(action_with_metadata) = self.action_receiver.recv().await {
            Some(self.process_action_with_metadata(action_with_metadata).await)
        } else {
            None
        }
    }
    
    /// Process all pending actions
    pub async fn process_all(&mut self) -> Vec<ActionResult> {
        let mut results = Vec::new();
        
        while let Ok(action_with_metadata) = self.action_receiver.try_recv() {
            let result = self.process_action_with_metadata(action_with_metadata).await;
            results.push(result);
        }
        
        results
    }
    
    /// Process a specific action
    pub async fn process_action(&mut self, action: Action) -> ActionResult {
        let action_with_metadata = ActionWithMetadata::new(action);
        Box::pin(self.process_action_with_metadata(action_with_metadata)).await
    }
    
    /// Process an action with metadata
    async fn process_action_with_metadata(&mut self, action_with_metadata: ActionWithMetadata) -> ActionResult {
        let action = &action_with_metadata.action;
        
        // Find handlers that can process this action
        let mut capable_handlers: Vec<_> = self.handlers.iter_mut()
            .filter(|(_, handler)| handler.can_handle(action))
            .collect();
        
        // Sort by priority
        capable_handlers.sort_by(|(_, a), (_, b)| b.priority().cmp(&a.priority()));
        
        // Try each handler until one succeeds
        for (name, handler) in capable_handlers {
            match handler.handle(action).await {
                Ok(ActionResult::Success) => {
                    tracing::debug!("Action {:?} handled successfully by {}", action, name);
                    return ActionResult::Success;
                }
                Ok(ActionResult::Forward(forwarded_action)) => {
                    tracing::debug!("Action {:?} forwarded by {} to {:?}", action, name, forwarded_action);
                    return Box::pin(self.process_action(forwarded_action)).await;
                }
                Ok(result) => {
                    tracing::debug!("Action {:?} handled by {} with result: {:?}", action, name, result);
                    return result;
                }
                Err(e) => {
                    tracing::warn!("Handler {} failed to process action {:?}: {}", name, action, e);
                    continue;
                }
            }
        }
        
        // No handler could process the action
        tracing::warn!("No handler found for action: {:?}", action);
        ActionResult::Error(format!("No handler found for action: {:?}", action))
    }
    
    /// Check if there are pending actions
    pub fn has_pending_actions(&self) -> bool {
        !self.action_receiver.is_empty()
    }
    
    /// Get the number of pending actions
    pub fn pending_action_count(&self) -> usize {
        // Note: This is an approximation as the receiver doesn't expose exact count
        if self.action_receiver.is_empty() { 0 } else { 1 }
    }
    
    /// Get the names of all registered handlers
    pub fn handler_names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
    
    /// Clear all pending actions
    pub fn clear_pending(&mut self) {
        while self.action_receiver.try_recv().is_ok() {
            // Drain the queue
        }
        tracing::debug!("Cleared all pending actions");
    }
}

impl Default for ActionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::None => write!(f, "None"),
            Action::Quit => write!(f, "Quit"),
            Action::Refresh => write!(f, "Refresh"),
            Action::Navigate(view) => write!(f, "Navigate({:?})", view),
            Action::Back => write!(f, "Back"),
            Action::GoBack => write!(f, "GoBack"),
            Action::GoForward => write!(f, "GoForward"),
            Action::FocusNext => write!(f, "FocusNext"),
            Action::FocusPrevious => write!(f, "FocusPrevious"),
            Action::FocusWidget(name) => write!(f, "FocusWidget({})", name),
            Action::SelectNext => write!(f, "SelectNext"),
            Action::SelectPrevious => write!(f, "SelectPrevious"),
            Action::SelectFirst => write!(f, "SelectFirst"),
            Action::SelectLast => write!(f, "SelectLast"),
            Action::SelectItem(index) => write!(f, "SelectItem({})", index),
            Action::StartInput(mode) => write!(f, "StartInput({:?})", mode),
            Action::ConfirmInput(input) => write!(f, "ConfirmInput({})", input),
            Action::CancelInput => write!(f, "CancelInput"),
            Action::Search(query) => write!(f, "Search({})", query),
            Action::Filter(filter) => write!(f, "Filter({})", filter),
            Action::Sort(order) => write!(f, "Sort({:?})", order),
            Action::ClearFilter => write!(f, "ClearFilter"),
            Action::ToggleDetails => write!(f, "ToggleDetails"),
            Action::ToggleHelp => write!(f, "ToggleHelp"),
            Action::ExecuteWorkflow(name) => write!(f, "ExecuteWorkflow({})", name),
            Action::PauseWorkflow(name) => write!(f, "PauseWorkflow({})", name),
            Action::ResumeWorkflow(name) => write!(f, "ResumeWorkflow({})", name),
            Action::StopWorkflow(name) => write!(f, "StopWorkflow({})", name),
            Action::CancelWorkflow(name) => write!(f, "CancelWorkflow({})", name),
            Action::ShowWorkflowDetails(name) => write!(f, "ShowWorkflowDetails({})", name),
            Action::CreateWorkflow => write!(f, "CreateWorkflow"),
            Action::EditWorkflow(name) => write!(f, "EditWorkflow({})", name),
            Action::DeleteWorkflow(name) => write!(f, "DeleteWorkflow({})", name),
            Action::ExecuteTool(name) => write!(f, "ExecuteTool({})", name),
            Action::ShowToolDetails(name) => write!(f, "ShowToolDetails({})", name),
            Action::RefreshTools => write!(f, "RefreshTools"),
            Action::InstallPlugin(name) => write!(f, "InstallPlugin({})", name),
            Action::UninstallPlugin(name) => write!(f, "UninstallPlugin({})", name),
            Action::ReloadPlugin(name) => write!(f, "ReloadPlugin({})", name),
            Action::EnablePlugin(name) => write!(f, "EnablePlugin({})", name),
            Action::DisablePlugin(name) => write!(f, "DisablePlugin({})", name),
            Action::TogglePlugin(name) => write!(f, "TogglePlugin({})", name),
            Action::ShowPluginDetails(name) => write!(f, "ShowPluginDetails({})", name),
            Action::ShowPluginFilters => write!(f, "ShowPluginFilters"),
            Action::ShowDependencyGraph => write!(f, "ShowDependencyGraph"),
            Action::RefreshPlugins => write!(f, "RefreshPlugins"),
            Action::RefreshSystemStatus => write!(f, "RefreshSystemStatus"),
            Action::ShowSystemDetails => write!(f, "ShowSystemDetails"),
            Action::ShowLogs => write!(f, "ShowLogs"),
            Action::FilterLogs(level) => write!(f, "FilterLogs({:?})", level),
            Action::ClearLogs => write!(f, "ClearLogs"),
            Action::ExportLogs => write!(f, "ExportLogs"),
            Action::ShowError(msg) => write!(f, "ShowError({})", msg),
            Action::DismissError => write!(f, "DismissError"),
            Action::ShowSettings => write!(f, "ShowSettings"),
            Action::ChangeTheme(theme) => write!(f, "ChangeTheme({})", theme),
            Action::SaveSettings => write!(f, "SaveSettings"),
            Action::Custom(name, _) => write!(f, "Custom({})", name),
        }
    }
}

impl fmt::Display for ViewType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViewType::WorkflowList => write!(f, "WorkflowList"),
            ViewType::ExecutionMonitor => write!(f, "ExecutionMonitor"),
            ViewType::ToolManager => write!(f, "ToolManager"),
            ViewType::PluginManager => write!(f, "PluginManager"),
            ViewType::SystemStatus => write!(f, "SystemStatus"),
            ViewType::LogViewer => write!(f, "LogViewer"),
        }
    }
}

impl fmt::Display for ActionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionResult::Success => write!(f, "Success"),
            ActionResult::Error(msg) => write!(f, "Error: {}", msg),
            ActionResult::RequiresConfirmation(msg) => write!(f, "RequiresConfirmation: {}", msg),
            ActionResult::RequiresInput(mode, prompt) => write!(f, "RequiresInput({:?}): {}", mode, prompt),
            ActionResult::Ignored => write!(f, "Ignored"),
            ActionResult::Forward(action) => write!(f, "Forward({})", action),
        }
    }
}