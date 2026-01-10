//! Focus management system for TUI widgets
//! 
//! This module provides focus management and keyboard navigation for TUI widgets.

use crate::error::Result;
use crate::interfaces::tui::{
    action::{Action, ViewType},
    widget::WidgetId,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Focus manager for handling widget focus and navigation
pub struct FocusManager {
    /// Current focused widget
    current_focus: Option<WidgetId>,
    /// Focus history for back navigation
    focus_history: VecDeque<WidgetId>,
    /// Maximum history size
    max_history: usize,
    /// Focus order for each view
    focus_orders: HashMap<ViewType, Vec<WidgetId>>,
    /// Widget focus capabilities
    focusable_widgets: HashMap<WidgetId, FocusCapability>,
    /// Focus change listeners
    listeners: Vec<Box<dyn FocusListener>>,
    /// Last focus change time
    last_focus_change: Option<Instant>,
    /// Focus lock (prevents focus changes)
    focus_locked: bool,
    /// Focus lock reason
    lock_reason: Option<String>,
}

/// Focus capability configuration for widgets
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusCapability {
    /// Whether the widget can receive focus
    pub focusable: bool,
    /// Whether the widget can be navigated to via Tab
    pub tab_navigable: bool,
    /// Whether the widget can be navigated to via arrow keys
    pub arrow_navigable: bool,
    /// Custom navigation keys
    pub custom_keys: Vec<KeyCode>,
    /// Focus priority (higher priority gets focus first)
    pub priority: i32,
    /// Whether focus should wrap around at boundaries
    pub wrap_navigation: bool,
}

/// Focus change event
#[derive(Debug, Clone)]
pub struct FocusChangeEvent {
    pub previous_focus: Option<WidgetId>,
    pub new_focus: Option<WidgetId>,
    pub timestamp: Instant,
    pub trigger: FocusTrigger,
}

/// What triggered the focus change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusTrigger {
    /// Tab key navigation
    Tab,
    /// Shift+Tab navigation
    ShiftTab,
    /// Arrow key navigation
    Arrow(ArrowDirection),
    /// Direct focus request
    Direct,
    /// Mouse click
    Mouse,
    /// Programmatic focus
    Programmatic,
    /// View change
    ViewChange,
}

/// Arrow key directions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Focus listener trait
pub trait FocusListener: Send + Sync {
    /// Called when focus changes
    fn on_focus_change(&mut self, event: &FocusChangeEvent);
    
    /// Get listener name
    fn name(&self) -> &str;
}

/// Navigation mode for different interaction patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationMode {
    /// Standard navigation (Tab, Shift+Tab, arrows)
    Standard,
    /// Vim-style navigation (hjkl keys)
    Vim,
    /// Emacs-style navigation (Ctrl combinations)
    Emacs,
    /// Custom navigation (user-defined keys)
    Custom(HashMap<String, String>),
}

/// Navigation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationConfig {
    /// Navigation mode
    pub mode: NavigationMode,
    /// Whether to wrap focus at boundaries
    pub wrap_focus: bool,
    /// Whether to remember focus history
    pub remember_history: bool,
    /// Focus change animation duration
    pub animation_duration: Duration,
    /// Whether to show focus indicators
    pub show_focus_indicators: bool,
}

/// Navigation actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationAction {
    /// Move focus to next widget
    FocusNext,
    /// Move focus to previous widget
    FocusPrevious,
    /// Move focus up
    FocusUp,
    /// Move focus down
    FocusDown,
    /// Move focus left
    FocusLeft,
    /// Move focus right
    FocusRight,
    /// Move to first widget
    FocusFirst,
    /// Move to last widget
    FocusLast,
    /// Go back in focus history
    FocusBack,
    /// Clear focus
    ClearFocus,
    /// Lock focus
    LockFocus,
    /// Unlock focus
    UnlockFocus,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            current_focus: None,
            focus_history: VecDeque::new(),
            max_history: 10,
            focus_orders: HashMap::new(),
            focusable_widgets: HashMap::new(),
            listeners: Vec::new(),
            last_focus_change: None,
            focus_locked: false,
            lock_reason: None,
        }
    }
    
    /// Register a widget as focusable
    pub fn register_focusable_widget(&mut self, widget_id: WidgetId, capability: FocusCapability) {
        tracing::debug!("Registered focusable widget: {}", widget_id);
        self.focusable_widgets.insert(widget_id, capability);
    }
    
    /// Unregister a focusable widget
    pub fn unregister_focusable_widget(&mut self, widget_id: &WidgetId) {
        self.focusable_widgets.remove(widget_id);
        
        // Remove from current focus if it was focused
        if self.current_focus.as_ref() == Some(widget_id) {
            self.current_focus = None;
        }
        
        // Remove from history
        self.focus_history.retain(|id| id != widget_id);
        
        // Remove from focus orders
        for order in self.focus_orders.values_mut() {
            order.retain(|id| id != widget_id);
        }
        
        tracing::debug!("Unregistered focusable widget: {}", widget_id);
    }
    
    /// Set focus order for a view
    pub fn set_focus_order(&mut self, view: ViewType, order: Vec<WidgetId>) {
        // Filter to only include registered focusable widgets
        let filtered_order: Vec<WidgetId> = order
            .into_iter()
            .filter(|id| self.focusable_widgets.contains_key(id))
            .collect();
        
        tracing::debug!("Set focus order for view {:?}", view);
        self.focus_orders.insert(view, filtered_order);
    }
    
    /// Get current focused widget
    pub fn current_focus(&self) -> Option<&WidgetId> {
        self.current_focus.as_ref()
    }
    
    /// Set focus to a specific widget
    pub fn set_focus(&mut self, widget_id: Option<WidgetId>, trigger: FocusTrigger) -> Result<bool> {
        if self.focus_locked {
            tracing::debug!("Focus change blocked: focus is locked ({})", 
                self.lock_reason.as_deref().unwrap_or("unknown reason"));
            return Ok(false);
        }
        
        // Check if widget is focusable
        if let Some(ref id) = widget_id {
            if let Some(capability) = self.focusable_widgets.get(id) {
                if !capability.focusable {
                    tracing::debug!("Widget {} is not focusable", id);
                    return Ok(false);
                }
            } else {
                tracing::debug!("Widget {} is not registered as focusable", id);
                return Ok(false);
            }
        }
        
        let previous_focus = self.current_focus.clone();
        
        // Update focus history
        if let Some(ref prev_id) = previous_focus {
            if widget_id.as_ref() != Some(prev_id) {
                self.focus_history.push_back(prev_id.clone());
                
                // Limit history size
                while self.focus_history.len() > self.max_history {
                    self.focus_history.pop_front();
                }
            }
        }
        
        self.current_focus = widget_id.clone();
        self.last_focus_change = Some(Instant::now());
        
        // Create focus change event
        let event = FocusChangeEvent {
            previous_focus,
            new_focus: widget_id,
            timestamp: Instant::now(),
            trigger,
        };
        
        // Notify listeners
        for listener in &mut self.listeners {
            listener.on_focus_change(&event);
        }
        
        tracing::debug!("Focus changed from {:?} to {:?}", event.previous_focus, event.new_focus);
        Ok(true)
    }
    
    /// Move focus to next widget in current view
    pub fn focus_next(&mut self, view: &ViewType) -> Result<bool> {
        let focus_order = match self.focus_orders.get(view) {
            Some(order) => order,
            None => {
                tracing::debug!("No focus order defined for view {:?}", view);
                return Ok(false);
            }
        };
        
        if focus_order.is_empty() {
            return Ok(false);
        }
        
        let next_widget = if let Some(ref current) = self.current_focus {
            // Find current widget in focus order
            if let Some(current_index) = focus_order.iter().position(|id| id == current) {
                let next_index = (current_index + 1) % focus_order.len();
                focus_order[next_index].clone()
            } else {
                // Current widget not in order, go to first
                focus_order[0].clone()
            }
        } else {
            // No current focus, go to first
            focus_order[0].clone()
        };
        
        self.set_focus(Some(next_widget), FocusTrigger::Tab)
    }
    
    /// Move focus to previous widget in current view
    pub fn focus_previous(&mut self, view: &ViewType) -> Result<bool> {
        let focus_order = match self.focus_orders.get(view) {
            Some(order) => order,
            None => {
                tracing::debug!("No focus order defined for view {:?}", view);
                return Ok(false);
            }
        };
        
        if focus_order.is_empty() {
            return Ok(false);
        }
        
        let prev_widget = if let Some(ref current) = self.current_focus {
            // Find current widget in focus order
            if let Some(current_index) = focus_order.iter().position(|id| id == current) {
                let prev_index = if current_index == 0 {
                    focus_order.len() - 1
                } else {
                    current_index - 1
                };
                focus_order[prev_index].clone()
            } else {
                // Current widget not in order, go to last
                focus_order[focus_order.len() - 1].clone()
            }
        } else {
            // No current focus, go to last
            focus_order[focus_order.len() - 1].clone()
        };
        
        self.set_focus(Some(prev_widget), FocusTrigger::ShiftTab)
    }
    
    /// Move focus in arrow direction
    pub fn focus_arrow(&mut self, view: &ViewType, direction: ArrowDirection) -> Result<bool> {
        // For now, treat arrow navigation the same as tab navigation
        // In a more sophisticated implementation, this could use spatial navigation
        match direction {
            ArrowDirection::Down | ArrowDirection::Right => {
                self.focus_next(view)
            }
            ArrowDirection::Up | ArrowDirection::Left => {
                self.focus_previous(view)
            }
        }
    }
    
    /// Go back in focus history
    pub fn focus_back(&mut self) -> Result<bool> {
        if let Some(previous_widget) = self.focus_history.pop_back() {
            self.set_focus(Some(previous_widget), FocusTrigger::Programmatic)
        } else {
            Ok(false)
        }
    }
    
    /// Clear current focus
    pub fn clear_focus(&mut self) -> Result<bool> {
        self.set_focus(None, FocusTrigger::Programmatic)
    }
    
    /// Lock focus (prevent focus changes)
    pub fn lock_focus(&mut self, reason: Option<String>) {
        self.focus_locked = true;
        self.lock_reason = reason;
        tracing::debug!("Focus locked: {}", self.lock_reason.as_deref().unwrap_or("no reason"));
    }
    
    /// Unlock focus
    pub fn unlock_focus(&mut self) {
        self.focus_locked = false;
        self.lock_reason = None;
        tracing::debug!("Focus unlocked");
    }
    
    /// Check if focus is locked
    pub fn is_focus_locked(&self) -> bool {
        self.focus_locked
    }
    
    /// Get focus lock reason
    pub fn focus_lock_reason(&self) -> Option<&str> {
        self.lock_reason.as_deref()
    }
    
    /// Handle keyboard event for navigation
    pub fn handle_navigation_key(&mut self, key: KeyEvent, view: &ViewType) -> Result<Option<Action>> {
        if self.focus_locked {
            return Ok(None);
        }
        
        match (key.code, key.modifiers) {
            // Tab navigation
            (KeyCode::Tab, KeyModifiers::NONE) => {
                if self.focus_next(view)? {
                    Ok(Some(Action::FocusNext))
                } else {
                    Ok(None)
                }
            }
            (KeyCode::Tab, KeyModifiers::SHIFT) => {
                if self.focus_previous(view)? {
                    Ok(Some(Action::FocusPrevious))
                } else {
                    Ok(None)
                }
            }
            
            // Arrow navigation
            (KeyCode::Up, KeyModifiers::NONE) => {
                if self.focus_arrow(view, ArrowDirection::Up)? {
                    Ok(Some(Action::FocusPrevious))
                } else {
                    Ok(None)
                }
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                if self.focus_arrow(view, ArrowDirection::Down)? {
                    Ok(Some(Action::FocusNext))
                } else {
                    Ok(None)
                }
            }
            (KeyCode::Left, KeyModifiers::NONE) => {
                if self.focus_arrow(view, ArrowDirection::Left)? {
                    Ok(Some(Action::FocusPrevious))
                } else {
                    Ok(None)
                }
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                if self.focus_arrow(view, ArrowDirection::Right)? {
                    Ok(Some(Action::FocusNext))
                } else {
                    Ok(None)
                }
            }
            
            // Home/End navigation
            (KeyCode::Home, KeyModifiers::NONE) => {
                if let Some(order) = self.focus_orders.get(view).cloned() {
                    if !order.is_empty() {
                        if self.set_focus(Some(order[0].clone()), FocusTrigger::Programmatic)? {
                            Ok(Some(Action::FocusWidget(order[0].0.clone())))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            (KeyCode::End, KeyModifiers::NONE) => {
                if let Some(order) = self.focus_orders.get(view).cloned() {
                    if !order.is_empty() {
                        let last_index = order.len() - 1;
                        if self.set_focus(Some(order[last_index].clone()), FocusTrigger::Programmatic)? {
                            Ok(Some(Action::FocusWidget(order[last_index].0.clone())))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            
            // Escape to clear focus
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if self.clear_focus()? {
                    Ok(Some(Action::GoBack))
                } else {
                    Ok(None)
                }
            }
            
            _ => Ok(None),
        }
    }
    
    /// Add a focus listener
    pub fn add_listener(&mut self, listener: Box<dyn FocusListener>) {
        tracing::debug!("Added focus listener: {}", listener.name());
        self.listeners.push(listener);
    }
    
    /// Remove a focus listener by name
    pub fn remove_listener(&mut self, name: &str) -> bool {
        let initial_len = self.listeners.len();
        self.listeners.retain(|l| l.name() != name);
        let removed = self.listeners.len() < initial_len;
        
        if removed {
            tracing::debug!("Removed focus listener: {}", name);
        }
        
        removed
    }
    
    /// Get focus statistics
    pub fn get_focus_stats(&self) -> FocusStats {
        FocusStats {
            current_focus: self.current_focus.clone(),
            focus_history_size: self.focus_history.len(),
            registered_widgets: self.focusable_widgets.len(),
            focus_orders_count: self.focus_orders.len(),
            is_locked: self.focus_locked,
            last_change: self.last_focus_change,
            listeners_count: self.listeners.len(),
        }
    }
    
    /// Reset focus manager state
    pub fn reset(&mut self) {
        self.current_focus = None;
        self.focus_history.clear();
        self.last_focus_change = None;
        self.focus_locked = false;
        self.lock_reason = None;
        tracing::debug!("Focus manager reset");
    }
    
    /// Get focusable widgets for a view
    pub fn get_focusable_widgets(&self, view: &ViewType) -> Vec<WidgetId> {
        self.focus_orders.get(view).cloned().unwrap_or_default()
    }
    
    /// Check if a widget is focusable
    pub fn is_widget_focusable(&self, widget_id: &WidgetId) -> bool {
        self.focusable_widgets
            .get(widget_id)
            .map(|cap| cap.focusable)
            .unwrap_or(false)
    }
    
    /// Get widget focus capability
    pub fn get_widget_capability(&self, widget_id: &WidgetId) -> Option<&FocusCapability> {
        self.focusable_widgets.get(widget_id)
    }
    
    /// Update widget focus capability
    pub fn update_widget_capability(&mut self, widget_id: &WidgetId, capability: FocusCapability) {
        if let Some(existing) = self.focusable_widgets.get_mut(widget_id) {
            *existing = capability;
            tracing::debug!("Updated focus capability for widget: {}", widget_id);
        }
    }
}

impl FocusCapability {
    /// Create default focus capability
    pub fn new() -> Self {
        Self {
            focusable: true,
            tab_navigable: true,
            arrow_navigable: true,
            custom_keys: Vec::new(),
            priority: 0,
            wrap_navigation: true,
        }
    }
    
    /// Create capability for non-focusable widget
    pub fn none() -> Self {
        Self {
            focusable: false,
            tab_navigable: false,
            arrow_navigable: false,
            custom_keys: Vec::new(),
            priority: 0,
            wrap_navigation: false,
        }
    }
    
    /// Create capability with custom settings
    pub fn custom(focusable: bool, tab_navigable: bool, arrow_navigable: bool) -> Self {
        Self {
            focusable,
            tab_navigable,
            arrow_navigable,
            custom_keys: Vec::new(),
            priority: 0,
            wrap_navigation: true,
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Add custom navigation key
    pub fn with_custom_key(mut self, key: KeyCode) -> Self {
        self.custom_keys.push(key);
        self
    }
    
    /// Set wrap navigation
    pub fn with_wrap_navigation(mut self, wrap: bool) -> Self {
        self.wrap_navigation = wrap;
        self
    }
}

impl Default for FocusCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationConfig {
    /// Create default navigation configuration
    pub fn new() -> Self {
        Self {
            mode: NavigationMode::Standard,
            wrap_focus: true,
            remember_history: true,
            animation_duration: Duration::from_millis(150),
            show_focus_indicators: true,
            custom_bindings: HashMap::new(),
        }
    }
    
    /// Create vim-style navigation configuration
    pub fn vim() -> Self {
        let mut config = Self::new();
        config.mode = NavigationMode::Vim;
        
        // Add vim-style key bindings
        config.custom_bindings.insert(KeyCode::Char('h'), NavigationAction::FocusLeft);
        config.custom_bindings.insert(KeyCode::Char('j'), NavigationAction::FocusDown);
        config.custom_bindings.insert(KeyCode::Char('k'), NavigationAction::FocusUp);
        config.custom_bindings.insert(KeyCode::Char('l'), NavigationAction::FocusRight);
        config.custom_bindings.insert(KeyCode::Char('g'), NavigationAction::FocusFirst);
        config.custom_bindings.insert(KeyCode::Char('G'), NavigationAction::FocusLast);
        
        config
    }
    
    /// Create emacs-style navigation configuration
    pub fn emacs() -> Self {
        let mut config = Self::new();
        config.mode = NavigationMode::Emacs;
        
        // Add emacs-style key bindings (would need Ctrl modifier support)
        config
    }
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Focus statistics
#[derive(Debug, Clone)]
pub struct FocusStats {
    pub current_focus: Option<WidgetId>,
    pub focus_history_size: usize,
    pub registered_widgets: usize,
    pub focus_orders_count: usize,
    pub is_locked: bool,
    pub last_change: Option<Instant>,
    pub listeners_count: usize,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple focus listener implementation for logging
pub struct LoggingFocusListener {
    name: String,
}

impl LoggingFocusListener {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl FocusListener for LoggingFocusListener {
    fn on_focus_change(&mut self, event: &FocusChangeEvent) {
        tracing::debug!(
            "Focus changed from {:?} to {:?} (trigger: {:?})",
            event.previous_focus,
            event.new_focus,
            event.trigger
        );
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}