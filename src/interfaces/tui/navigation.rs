//! Navigation stack management for TUI interface
//! 
//! This module provides navigation stack management, modal dialog support,
//! and unified Esc key behavior for the TUI interface.

use crate::error::Result;
use crate::interfaces::tui::action::{Action, ViewType};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Navigation stack manager for handling view history and modal dialogs
pub struct NavigationStack {
    /// Stack of navigation states
    stack: VecDeque<NavigationState>,
    /// Maximum stack size
    max_stack_size: usize,
    /// Current modal dialog (if any)
    current_modal: Option<ModalDialog>,
    /// Modal dialog stack for nested modals
    modal_stack: VecDeque<ModalDialog>,
    /// Navigation history for analytics
    navigation_history: VecDeque<NavigationEvent>,
    /// Maximum history size
    max_history_size: usize,
}

/// Navigation state representing a point in navigation history
#[derive(Debug, Clone)]
pub struct NavigationState {
    /// Current view
    pub view: ViewType,
    /// View-specific state data
    pub state_data: Option<serde_json::Value>,
    /// Timestamp when this state was created
    pub timestamp: Instant,
    /// Whether this state can be returned to
    pub returnable: bool,
    /// Context information
    pub context: NavigationContext,
}

/// Navigation context information
#[derive(Debug, Clone)]
pub struct NavigationContext {
    /// How we got to this state
    pub trigger: NavigationTrigger,
    /// Previous view (if any)
    pub previous_view: Option<ViewType>,
    /// User action that caused this navigation
    pub user_action: Option<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// What triggered the navigation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationTrigger {
    /// User initiated navigation (F-keys, menu selection)
    UserInitiated,
    /// Programmatic navigation (automatic redirect)
    Programmatic,
    /// Back navigation (Esc key, back button)
    BackNavigation,
    /// Modal dialog opened
    ModalOpened,
    /// Modal dialog closed
    ModalClosed,
    /// Application startup
    Startup,
    /// Error recovery
    ErrorRecovery,
}

/// Modal dialog configuration
#[derive(Debug, Clone)]
pub struct ModalDialog {
    /// Unique identifier for the modal
    pub id: String,
    /// Modal title
    pub title: String,
    /// Modal content type
    pub content_type: ModalContentType,
    /// Modal size configuration
    pub size: ModalSize,
    /// Whether the modal can be closed with Esc
    pub closable: bool,
    /// Whether the modal blocks interaction with background
    pub blocking: bool,
    /// Modal buttons
    pub buttons: Vec<ModalButton>,
    /// When the modal was opened
    pub opened_at: Instant,
    /// Modal timeout (auto-close after duration)
    pub timeout: Option<Duration>,
    /// Callback action when modal is closed
    pub on_close: Option<Action>,
    /// Modal data
    pub data: Option<serde_json::Value>,
}

/// Modal content types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalContentType {
    /// Simple text message
    Message,
    /// Confirmation dialog (Yes/No)
    Confirmation,
    /// Input dialog (text input)
    Input,
    /// Error message
    Error,
    /// Warning message
    Warning,
    /// Information message
    Info,
    /// Progress dialog
    Progress,
    /// Custom content
    Custom(String),
}

/// Modal size configuration
#[derive(Debug, Clone)]
pub struct ModalSize {
    /// Width as percentage of screen (1-100)
    pub width_percent: u16,
    /// Height as percentage of screen (1-100)
    pub height_percent: u16,
    /// Minimum width in characters
    pub min_width: Option<u16>,
    /// Minimum height in characters
    pub min_height: Option<u16>,
    /// Maximum width in characters
    pub max_width: Option<u16>,
    /// Maximum height in characters
    pub max_height: Option<u16>,
}

/// Modal button configuration
#[derive(Debug, Clone)]
pub struct ModalButton {
    /// Button label
    pub label: String,
    /// Button action when clicked
    pub action: Action,
    /// Button style
    pub style: ModalButtonStyle,
    /// Whether this is the default button (Enter key)
    pub is_default: bool,
    /// Whether this is the cancel button (Esc key)
    pub is_cancel: bool,
    /// Button shortcut key
    pub shortcut: Option<char>,
}

/// Modal button styles
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalButtonStyle {
    /// Primary button (highlighted)
    Primary,
    /// Secondary button (normal)
    Secondary,
    /// Danger button (red, for destructive actions)
    Danger,
    /// Success button (green, for positive actions)
    Success,
    /// Warning button (yellow, for caution)
    Warning,
}

/// Navigation event for history tracking
#[derive(Debug, Clone)]
pub struct NavigationEvent {
    /// Event timestamp
    pub timestamp: Instant,
    /// Event type
    pub event_type: NavigationEventType,
    /// Source view
    pub from_view: Option<ViewType>,
    /// Destination view
    pub to_view: Option<ViewType>,
    /// Event trigger
    pub trigger: NavigationTrigger,
    /// Event duration (for timed events)
    pub duration: Option<Duration>,
}

/// Navigation event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationEventType {
    /// View changed
    ViewChanged,
    /// Modal opened
    ModalOpened,
    /// Modal closed
    ModalClosed,
    /// Back navigation
    BackNavigation,
    /// Navigation blocked
    NavigationBlocked,
    /// Navigation error
    NavigationError,
}

/// Esc key behavior configuration
#[derive(Debug, Clone)]
pub struct EscKeyBehavior {
    /// Whether Esc closes modals
    pub closes_modals: bool,
    /// Whether Esc goes back in navigation
    pub navigates_back: bool,
    /// Whether Esc clears focus
    pub clears_focus: bool,
    /// Whether Esc cancels current operation
    pub cancels_operation: bool,
    /// Custom Esc behavior per view
    pub view_behaviors: std::collections::HashMap<ViewType, EscAction>,
}

/// Actions that Esc key can trigger
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscAction {
    /// Go back to previous view
    GoBack,
    /// Close current modal
    CloseModal,
    /// Clear current focus
    ClearFocus,
    /// Cancel current operation
    CancelOperation,
    /// Do nothing
    None,
    /// Custom action
    Custom(String),
}

impl NavigationStack {
    /// Create a new navigation stack
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
            max_stack_size: 50,
            current_modal: None,
            modal_stack: VecDeque::new(),
            navigation_history: VecDeque::new(),
            max_history_size: 100,
        }
    }
    
    /// Push a new navigation state
    pub fn push_state(&mut self, view: ViewType, trigger: NavigationTrigger) -> Result<()> {
        let previous_view = self.current_view();
        
        let state = NavigationState {
            view: view.clone(),
            state_data: None,
            timestamp: Instant::now(),
            returnable: true,
            context: NavigationContext {
                trigger: trigger.clone(),
                previous_view: previous_view.clone(),
                user_action: None,
                metadata: std::collections::HashMap::new(),
            },
        };
        
        // Remove oldest states if stack is full
        while self.stack.len() >= self.max_stack_size {
            self.stack.pop_front();
        }
        
        self.stack.push_back(state);
        
        // Record navigation event
        self.record_navigation_event(NavigationEventType::ViewChanged, previous_view, Some(view.clone()), trigger);
        
        tracing::debug!("Pushed navigation state: {:?}", view);
        Ok(())
    }
    
    /// Pop the current navigation state and return to previous
    pub fn pop_state(&mut self) -> Result<Option<ViewType>> {
        if self.stack.len() <= 1 {
            // Don't pop the last state
            return Ok(None);
        }
        
        let current_view = self.current_view();
        self.stack.pop_back();
        let previous_view = self.current_view();
        
        // Record navigation event
        self.record_navigation_event(
            NavigationEventType::BackNavigation,
            current_view.clone(),
            previous_view.clone(),
            NavigationTrigger::BackNavigation,
        );
        
        tracing::debug!("Popped navigation state, returning to: {:?}", previous_view);
        Ok(previous_view)
    }
    
    /// Get current view
    pub fn current_view(&self) -> Option<ViewType> {
        self.stack.back().map(|state| state.view.clone())
    }
    
    /// Get current navigation state
    pub fn current_state(&self) -> Option<&NavigationState> {
        self.stack.back()
    }
    
    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        self.stack.len() > 1 && self.current_modal.is_none()
    }
    
    /// Open a modal dialog
    pub fn open_modal(&mut self, modal: ModalDialog) -> Result<()> {
        // If there's already a modal, push it to the stack
        if let Some(current_modal) = self.current_modal.take() {
            self.modal_stack.push_back(current_modal);
        }
        
        tracing::debug!("Opening modal: {}", modal.title);
        
        // Record navigation event
        self.record_navigation_event(
            NavigationEventType::ModalOpened,
            self.current_view(),
            None,
            NavigationTrigger::ModalOpened,
        );
        
        self.current_modal = Some(modal);
        Ok(())
    }
    
    /// Close the current modal dialog
    pub fn close_modal(&mut self) -> Result<Option<Action>> {
        let closed_modal = self.current_modal.take();
        
        if let Some(modal) = closed_modal {
            tracing::debug!("Closing modal: {}", modal.title);
            
            // Record navigation event
            self.record_navigation_event(
                NavigationEventType::ModalClosed,
                None,
                self.current_view(),
                NavigationTrigger::ModalClosed,
            );
            
            // Restore previous modal if any
            if let Some(previous_modal) = self.modal_stack.pop_back() {
                self.current_modal = Some(previous_modal);
            }
            
            Ok(modal.on_close)
        } else {
            Ok(None)
        }
    }
    
    /// Get current modal dialog
    pub fn current_modal(&self) -> Option<&ModalDialog> {
        self.current_modal.as_ref()
    }
    
    /// Check if a modal is currently open
    pub fn has_modal(&self) -> bool {
        self.current_modal.is_some()
    }
    
    /// Handle Esc key press
    pub fn handle_esc_key(&mut self, behavior: &EscKeyBehavior) -> Result<Option<Action>> {
        // Priority 1: Close modal if one is open
        if self.has_modal() && behavior.closes_modals {
            if let Some(modal) = &self.current_modal {
                if modal.closable {
                    return self.close_modal();
                }
            }
        }
        
        // Priority 2: Navigate back if possible
        if behavior.navigates_back && self.can_go_back() {
            if let Some(previous_view) = self.pop_state()? {
                return Ok(Some(Action::Navigate(previous_view)));
            }
        }
        
        // Priority 3: Clear focus
        if behavior.clears_focus {
            return Ok(Some(Action::GoBack));
        }
        
        // Priority 4: Check view-specific behavior
        if let Some(current_view) = self.current_view() {
            if let Some(esc_action) = behavior.view_behaviors.get(&current_view) {
                return Ok(Some(self.esc_action_to_action(esc_action.clone())));
            }
        }
        
        Ok(None)
    }
    
    /// Convert EscAction to Action
    fn esc_action_to_action(&self, esc_action: EscAction) -> Action {
        match esc_action {
            EscAction::GoBack => Action::GoBack,
            EscAction::CloseModal => Action::GoBack, // Will be handled by modal system
            EscAction::ClearFocus => Action::GoBack,
            EscAction::CancelOperation => Action::GoBack,
            EscAction::None => Action::None,
            EscAction::Custom(action_name) => Action::Custom(action_name, serde_json::Value::Null),
        }
    }
    
    /// Record a navigation event
    fn record_navigation_event(
        &mut self,
        event_type: NavigationEventType,
        from_view: Option<ViewType>,
        to_view: Option<ViewType>,
        trigger: NavigationTrigger,
    ) {
        let event = NavigationEvent {
            timestamp: Instant::now(),
            event_type,
            from_view,
            to_view,
            trigger,
            duration: None,
        };
        
        // Remove oldest events if history is full
        while self.navigation_history.len() >= self.max_history_size {
            self.navigation_history.pop_front();
        }
        
        self.navigation_history.push_back(event);
    }
    
    /// Get navigation statistics
    pub fn get_navigation_stats(&self) -> NavigationStats {
        NavigationStats {
            stack_size: self.stack.len(),
            can_go_back: self.can_go_back(),
            has_modal: self.has_modal(),
            modal_stack_size: self.modal_stack.len(),
            history_size: self.navigation_history.len(),
            current_view: self.current_view(),
        }
    }
    
    /// Clear navigation history
    pub fn clear_history(&mut self) {
        self.navigation_history.clear();
        tracing::debug!("Navigation history cleared");
    }
    
    /// Get navigation history
    pub fn get_history(&self) -> &VecDeque<NavigationEvent> {
        &self.navigation_history
    }
    
    /// Set maximum stack size
    pub fn set_max_stack_size(&mut self, size: usize) {
        self.max_stack_size = size;
        
        // Trim stack if necessary
        while self.stack.len() > self.max_stack_size {
            self.stack.pop_front();
        }
    }
    
    /// Set maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        
        // Trim history if necessary
        while self.navigation_history.len() > self.max_history_size {
            self.navigation_history.pop_front();
        }
    }
    
    /// Reset navigation stack
    pub fn reset(&mut self) {
        self.stack.clear();
        self.current_modal = None;
        self.modal_stack.clear();
        self.navigation_history.clear();
        tracing::debug!("Navigation stack reset");
    }
}

impl ModalDialog {
    /// Create a simple message modal
    pub fn message(id: String, title: String, message: String) -> Self {
        Self {
            id,
            title,
            content_type: ModalContentType::Message,
            size: ModalSize::medium(),
            closable: true,
            blocking: true,
            buttons: vec![
                ModalButton {
                    label: "确定".to_string(),
                    action: Action::GoBack,
                    style: ModalButtonStyle::Primary,
                    is_default: true,
                    is_cancel: false,
                    shortcut: Some('o'),
                }
            ],
            opened_at: Instant::now(),
            timeout: None,
            on_close: None,
            data: Some(serde_json::json!({ "message": message })),
        }
    }
    
    /// Create a confirmation modal
    pub fn confirmation(id: String, title: String, message: String, on_confirm: Action) -> Self {
        Self {
            id,
            title,
            content_type: ModalContentType::Confirmation,
            size: ModalSize::medium(),
            closable: true,
            blocking: true,
            buttons: vec![
                ModalButton {
                    label: "确定".to_string(),
                    action: on_confirm,
                    style: ModalButtonStyle::Primary,
                    is_default: true,
                    is_cancel: false,
                    shortcut: Some('y'),
                },
                ModalButton {
                    label: "取消".to_string(),
                    action: Action::GoBack,
                    style: ModalButtonStyle::Secondary,
                    is_default: false,
                    is_cancel: true,
                    shortcut: Some('n'),
                }
            ],
            opened_at: Instant::now(),
            timeout: None,
            on_close: None,
            data: Some(serde_json::json!({ "message": message })),
        }
    }
    
    /// Create an error modal
    pub fn error(id: String, title: String, error_message: String) -> Self {
        Self {
            id,
            title,
            content_type: ModalContentType::Error,
            size: ModalSize::medium(),
            closable: true,
            blocking: true,
            buttons: vec![
                ModalButton {
                    label: "确定".to_string(),
                    action: Action::GoBack,
                    style: ModalButtonStyle::Danger,
                    is_default: true,
                    is_cancel: true,
                    shortcut: Some('o'),
                }
            ],
            opened_at: Instant::now(),
            timeout: None,
            on_close: None,
            data: Some(serde_json::json!({ "error": error_message })),
        }
    }
    
    /// Check if modal has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            self.opened_at.elapsed() >= timeout
        } else {
            false
        }
    }
}

impl ModalSize {
    /// Small modal size
    pub fn small() -> Self {
        Self {
            width_percent: 40,
            height_percent: 30,
            min_width: Some(30),
            min_height: Some(10),
            max_width: Some(60),
            max_height: Some(20),
        }
    }
    
    /// Medium modal size
    pub fn medium() -> Self {
        Self {
            width_percent: 60,
            height_percent: 50,
            min_width: Some(40),
            min_height: Some(15),
            max_width: Some(80),
            max_height: Some(30),
        }
    }
    
    /// Large modal size
    pub fn large() -> Self {
        Self {
            width_percent: 80,
            height_percent: 70,
            min_width: Some(60),
            min_height: Some(20),
            max_width: Some(120),
            max_height: Some(40),
        }
    }
}

impl EscKeyBehavior {
    /// Create default Esc key behavior
    pub fn default() -> Self {
        Self {
            closes_modals: true,
            navigates_back: true,
            clears_focus: true,
            cancels_operation: true,
            view_behaviors: std::collections::HashMap::new(),
        }
    }
    
    /// Create strict Esc key behavior (only closes modals)
    pub fn strict() -> Self {
        Self {
            closes_modals: true,
            navigates_back: false,
            clears_focus: false,
            cancels_operation: false,
            view_behaviors: std::collections::HashMap::new(),
        }
    }
    
    /// Set view-specific Esc behavior
    pub fn with_view_behavior(mut self, view: ViewType, action: EscAction) -> Self {
        self.view_behaviors.insert(view, action);
        self
    }
}

/// Navigation statistics
#[derive(Debug, Clone)]
pub struct NavigationStats {
    pub stack_size: usize,
    pub can_go_back: bool,
    pub has_modal: bool,
    pub modal_stack_size: usize,
    pub history_size: usize,
    pub current_view: Option<ViewType>,
}

impl Default for NavigationStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for EscKeyBehavior {
    fn default() -> Self {
        Self::default()
    }
}