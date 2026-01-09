//! Widget trait and basic interfaces
//! 
//! This module defines the core Widget trait and related types for the TUI system.

use async_trait::async_trait;
use ratatui::{Frame, layout::Rect};
use ratatui::crossterm::event::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

use super::action::Action;
use super::theme::Theme;

/// Unique identifier for widgets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WidgetId(pub String);

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for WidgetId {
    fn from(s: &str) -> Self {
        WidgetId(s.to_string())
    }
}

impl From<String> for WidgetId {
    fn from(s: String) -> Self {
        WidgetId(s)
    }
}

/// Widget state information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetState {
    /// Widget is not initialized
    Uninitialized,
    /// Widget is initialized but not active
    Inactive,
    /// Widget is active and can receive events
    Active,
    /// Widget is focused and receives keyboard input
    Focused,
    /// Widget is disabled and cannot receive events
    Disabled,
    /// Widget encountered an error
    Error(String),
}

/// Widget context information
#[derive(Debug, Clone)]
pub struct WidgetContext {
    /// Widget's unique identifier
    pub id: WidgetId,
    /// Current widget state
    pub state: WidgetState,
    /// Widget's position and size
    pub area: Option<Rect>,
    /// Whether the widget has focus
    pub has_focus: bool,
    /// Whether the widget is visible
    pub is_visible: bool,
    /// Last update timestamp
    pub last_update: Option<Instant>,
    /// Widget metadata
    pub metadata: HashMap<String, String>,
}

impl WidgetContext {
    /// Create a new widget context
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            state: WidgetState::Uninitialized,
            area: None,
            has_focus: false,
            is_visible: true,
            last_update: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if the widget is in an active state
    pub fn is_active(&self) -> bool {
        matches!(self.state, WidgetState::Active | WidgetState::Focused)
    }
    
    /// Check if the widget can handle events
    pub fn can_handle_events(&self) -> bool {
        matches!(self.state, WidgetState::Active | WidgetState::Focused) && self.is_visible
    }
    
    /// Update the last update timestamp
    pub fn mark_updated(&mut self) {
        self.last_update = Some(Instant::now());
    }
    
    /// Get time since last update
    pub fn time_since_update(&self) -> Option<Duration> {
        self.last_update.map(|last| last.elapsed())
    }
}

/// Widget error types
#[derive(Debug, thiserror::Error)]
pub enum WidgetError {
    #[error("Widget initialization failed: {message}")]
    InitializationError { message: String },
    
    #[error("Widget render error: {message}")]
    RenderError { message: String },
    
    #[error("Widget event handling error: {message}")]
    EventError { message: String },
    
    #[error("Widget update error: {message}")]
    UpdateError { message: String },
    
    #[error("Widget state error: {message}")]
    StateError { message: String },
    
    #[error("Widget configuration error: {message}")]
    ConfigError { message: String },
}

/// Widget size constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeConstraints {
    /// Minimum width in characters
    pub min_width: Option<u16>,
    /// Maximum width in characters
    pub max_width: Option<u16>,
    /// Minimum height in characters
    pub min_height: Option<u16>,
    /// Maximum height in characters
    pub max_height: Option<u16>,
    /// Preferred width in characters
    pub preferred_width: Option<u16>,
    /// Preferred height in characters
    pub preferred_height: Option<u16>,
}

impl SizeConstraints {
    /// Create new size constraints with no limits
    pub fn new() -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            preferred_width: None,
            preferred_height: None,
        }
    }
    
    /// Set minimum size
    pub fn min_size(mut self, width: u16, height: u16) -> Self {
        self.min_width = Some(width);
        self.min_height = Some(height);
        self
    }
    
    /// Set maximum size
    pub fn max_size(mut self, width: u16, height: u16) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }
    
    /// Set preferred size
    pub fn preferred_size(mut self, width: u16, height: u16) -> Self {
        self.preferred_width = Some(width);
        self.preferred_height = Some(height);
        self
    }
    
    /// Check if a size satisfies the constraints
    pub fn satisfies(&self, width: u16, height: u16) -> bool {
        if let Some(min_w) = self.min_width {
            if width < min_w { return false; }
        }
        if let Some(max_w) = self.max_width {
            if width > max_w { return false; }
        }
        if let Some(min_h) = self.min_height {
            if height < min_h { return false; }
        }
        if let Some(max_h) = self.max_height {
            if height > max_h { return false; }
        }
        true
    }
    
    /// Clamp a size to fit within constraints
    pub fn clamp(&self, width: u16, height: u16) -> (u16, u16) {
        let mut w = width;
        let mut h = height;
        
        if let Some(min_w) = self.min_width {
            w = w.max(min_w);
        }
        if let Some(max_w) = self.max_width {
            w = w.min(max_w);
        }
        if let Some(min_h) = self.min_height {
            h = h.max(min_h);
        }
        if let Some(max_h) = self.max_height {
            h = h.min(max_h);
        }
        
        (w, h)
    }
}

impl Default for SizeConstraints {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget update frequency
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateFrequency {
    /// Never update automatically
    Never,
    /// Update once
    Once,
    /// Update at regular intervals
    Interval(Duration),
    /// Update on every frame (60fps)
    EveryFrame,
    /// Update when data changes
    OnDataChange,
}

/// Widget capability flags
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetCapabilities {
    /// Can handle keyboard input
    pub keyboard_input: bool,
    /// Can handle mouse input
    pub mouse_input: bool,
    /// Can be focused
    pub focusable: bool,
    /// Can be resized
    pub resizable: bool,
    /// Can be scrolled
    pub scrollable: bool,
    /// Supports themes
    pub themeable: bool,
    /// Can be configured
    pub configurable: bool,
}

impl WidgetCapabilities {
    /// Create default capabilities
    pub fn new() -> Self {
        Self {
            keyboard_input: true,
            mouse_input: false,
            focusable: true,
            resizable: true,
            scrollable: false,
            themeable: true,
            configurable: false,
        }
    }
    
    /// Enable all capabilities
    pub fn all() -> Self {
        Self {
            keyboard_input: true,
            mouse_input: true,
            focusable: true,
            resizable: true,
            scrollable: true,
            themeable: true,
            configurable: true,
        }
    }
    
    /// Disable all capabilities
    pub fn none() -> Self {
        Self {
            keyboard_input: false,
            mouse_input: false,
            focusable: false,
            resizable: false,
            scrollable: false,
            themeable: false,
            configurable: false,
        }
    }
}

impl Default for WidgetCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced Widget trait for TUI components
#[async_trait]
pub trait Widget: Send + Sync {
    /// Get the widget's unique identifier
    fn id(&self) -> &WidgetId;
    
    /// Get the widget's display title
    fn title(&self) -> &str;
    
    /// Get the widget's description
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// Get the widget's current context
    fn context(&self) -> &WidgetContext;
    
    /// Get mutable reference to the widget's context
    fn context_mut(&mut self) -> &mut WidgetContext;
    
    /// Get the widget's capabilities
    fn capabilities(&self) -> &WidgetCapabilities;
    
    /// Get the widget's size constraints
    fn size_constraints(&self) -> &SizeConstraints;
    
    /// Get the widget's update frequency
    fn update_frequency(&self) -> UpdateFrequency {
        UpdateFrequency::Never
    }
    
    /// Check if the widget needs periodic updates
    fn needs_update(&self) -> bool {
        !matches!(self.update_frequency(), UpdateFrequency::Never)
    }
    
    /// Get the widget's update interval in milliseconds
    fn update_interval(&self) -> u64 {
        match self.update_frequency() {
            UpdateFrequency::Never => 0,
            UpdateFrequency::Once => 0,
            UpdateFrequency::Interval(duration) => duration.as_millis() as u64,
            UpdateFrequency::EveryFrame => 16, // ~60fps
            UpdateFrequency::OnDataChange => 100, // Check every 100ms
        }
    }
    
    /// Initialize the widget (called once when registered)
    async fn initialize(&mut self) -> std::result::Result<(), WidgetError> {
        self.context_mut().state = WidgetState::Inactive;
        tracing::debug!("Initialized widget: {}", self.id());
        Ok(())
    }
    
    /// Cleanup the widget (called when unregistering)
    async fn cleanup(&mut self) -> std::result::Result<(), WidgetError> {
        self.context_mut().state = WidgetState::Uninitialized;
        tracing::debug!("Cleaned up widget: {}", self.id());
        Ok(())
    }
    
    /// Called when the widget becomes active (gains focus)
    async fn on_activate(&mut self) -> std::result::Result<(), WidgetError> {
        if self.context().state == WidgetState::Inactive {
            self.context_mut().state = WidgetState::Active;
            tracing::debug!("Activated widget: {}", self.id());
        }
        Ok(())
    }
    
    /// Called when the widget becomes inactive (loses focus)
    async fn on_deactivate(&mut self) -> std::result::Result<(), WidgetError> {
        if matches!(self.context().state, WidgetState::Active | WidgetState::Focused) {
            self.context_mut().state = WidgetState::Inactive;
            tracing::debug!("Deactivated widget: {}", self.id());
        }
        Ok(())
    }
    
    /// Called when the widget gains focus
    async fn on_focus(&mut self) -> std::result::Result<(), WidgetError> {
        if self.capabilities().focusable && self.context().state == WidgetState::Active {
            self.context_mut().state = WidgetState::Focused;
            self.context_mut().has_focus = true;
            tracing::debug!("Widget gained focus: {}", self.id());
        }
        Ok(())
    }
    
    /// Called when the widget loses focus
    async fn on_blur(&mut self) -> std::result::Result<(), WidgetError> {
        if self.context().state == WidgetState::Focused {
            self.context_mut().state = WidgetState::Active;
            self.context_mut().has_focus = false;
            tracing::debug!("Widget lost focus: {}", self.id());
        }
        Ok(())
    }
    
    /// Called when the widget's area changes
    async fn on_resize(&mut self, new_area: Rect) -> std::result::Result<(), WidgetError> {
        let (width, height) = self.size_constraints().clamp(new_area.width, new_area.height);
        let adjusted_area = Rect {
            x: new_area.x,
            y: new_area.y,
            width,
            height,
        };
        
        self.context_mut().area = Some(adjusted_area);
        tracing::debug!("Widget resized: {} to {:?}", self.id(), adjusted_area);
        Ok(())
    }
    
    /// Render the widget to the given frame area
    async fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) -> std::result::Result<(), WidgetError>;
    
    /// Handle an event and return the resulting action
    async fn handle_event(&mut self, event: Event) -> std::result::Result<Option<Action>, WidgetError>;
    
    /// Update the widget state (called periodically)
    async fn update(&mut self) -> std::result::Result<(), WidgetError> {
        self.context_mut().mark_updated();
        Ok(())
    }
    
    /// Check if the widget can handle a specific event type
    fn can_handle_event(&self, event: &Event) -> bool {
        if !self.context().can_handle_events() {
            return false;
        }
        
        match event {
            Event::Key(_) => self.capabilities().keyboard_input,
            Event::Mouse(_) => self.capabilities().mouse_input,
            Event::Resize(_, _) => self.capabilities().resizable,
            _ => false,
        }
    }
    
    /// Get help text for keyboard shortcuts
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![]
    }
    
    /// Get widget-specific configuration
    fn get_config(&self) -> Option<serde_json::Value> {
        None
    }
    
    /// Set widget-specific configuration
    async fn set_config(&mut self, _config: serde_json::Value) -> std::result::Result<(), WidgetError> {
        Ok(())
    }
    
    /// Validate the widget's current state
    fn validate(&self) -> std::result::Result<(), WidgetError> {
        match self.context().state {
            WidgetState::Error(ref msg) => Err(WidgetError::StateError {
                message: format!("Widget state error: {}", msg)
            }),
            _ => Ok(()),
        }
    }
    
    /// Get widget performance metrics
    fn metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        if let Some(duration) = self.context().time_since_update() {
            metrics.insert("time_since_update_ms".to_string(), duration.as_millis() as f64);
        }
        
        metrics.insert("is_active".to_string(), if self.context().is_active() { 1.0 } else { 0.0 });
        metrics.insert("has_focus".to_string(), if self.context().has_focus { 1.0 } else { 0.0 });
        metrics.insert("is_visible".to_string(), if self.context().is_visible { 1.0 } else { 0.0 });
        
        metrics
    }
    
    /// Get debug information about the widget
    fn debug_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        info.insert("id".to_string(), self.id().to_string());
        info.insert("title".to_string(), self.title().to_string());
        info.insert("state".to_string(), format!("{:?}", self.context().state));
        info.insert("capabilities".to_string(), format!("{:?}", self.capabilities()));
        info.insert("size_constraints".to_string(), format!("{:?}", self.size_constraints()));
        info.insert("update_frequency".to_string(), format!("{:?}", self.update_frequency()));
        
        if let Some(area) = self.context().area {
            info.insert("area".to_string(), format!("{:?}", area));
        }
        
        info
    }
}

/// Base widget implementation that other widgets can extend
pub struct BaseWidget {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
    update_frequency: UpdateFrequency,
    title: String,
    description: Option<String>,
}

impl BaseWidget {
    /// Create a new base widget
    pub fn new(id: WidgetId, title: String) -> Self {
        Self {
            context: WidgetContext::new(id),
            capabilities: WidgetCapabilities::default(),
            size_constraints: SizeConstraints::default(),
            update_frequency: UpdateFrequency::Never,
            title,
            description: None,
        }
    }
    
    /// Set the widget's description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// Set the widget's capabilities
    pub fn with_capabilities(mut self, capabilities: WidgetCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }
    
    /// Set the widget's size constraints
    pub fn with_size_constraints(mut self, constraints: SizeConstraints) -> Self {
        self.size_constraints = constraints;
        self
    }
    
    /// Set the widget's update frequency
    pub fn with_update_frequency(mut self, frequency: UpdateFrequency) -> Self {
        self.update_frequency = frequency;
        self
    }
}

#[async_trait]
impl Widget for BaseWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }
    
    fn title(&self) -> &str {
        &self.title
    }
    
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    fn context(&self) -> &WidgetContext {
        &self.context
    }
    
    fn context_mut(&mut self) -> &mut WidgetContext {
        &mut self.context
    }
    
    fn capabilities(&self) -> &WidgetCapabilities {
        &self.capabilities
    }
    
    fn size_constraints(&self) -> &SizeConstraints {
        &self.size_constraints
    }
    
    fn update_frequency(&self) -> UpdateFrequency {
        self.update_frequency.clone()
    }
    
    async fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) -> std::result::Result<(), WidgetError> {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::text::Text;
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title())
            .border_style(theme.styles.widget_border);
        
        let content = if let Some(desc) = self.description() {
            format!("{}\n\n{}", self.title(), desc)
        } else {
            format!("{}\n\n(Base Widget)", self.title())
        };
        
        let paragraph = Paragraph::new(Text::from(content))
            .block(block)
            .style(theme.styles.info);
        
        frame.render_widget(paragraph, area);
        Ok(())
    }
    
    async fn handle_event(&mut self, _event: Event) -> std::result::Result<Option<Action>, WidgetError> {
        Ok(None)
    }
}