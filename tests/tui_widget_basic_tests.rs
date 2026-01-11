//! Basic TUI Widget Tests
//!
//! This module contains basic tests to verify Widget functionality
//! without the complex TUI application setup.

use ratatui::{
    backend::TestBackend,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    Terminal,
};
use std::sync::Arc;
use std::time::Duration;
use workflow_toolkit::error::Result;
use workflow_toolkit::interfaces::tui::widget::{
    Widget, WidgetCapabilities, WidgetContext, WidgetError, WidgetId, WidgetState,
};
use workflow_toolkit::interfaces::tui::Theme;

/// Test helper to create a mock widget for testing
struct MockWidget {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
}

impl MockWidget {
    fn new() -> Self {
        Self {
            context: WidgetContext::new(WidgetId::from("mock-widget")),
            capabilities: WidgetCapabilities::default(),
        }
    }
}

#[async_trait::async_trait]
impl Widget for MockWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }

    fn title(&self) -> &str {
        "Mock Widget"
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

    fn size_constraints(&self) -> &workflow_toolkit::interfaces::tui::widget::SizeConstraints {
        &workflow_toolkit::interfaces::tui::widget::SizeConstraints::default()
    }

    async fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: Rect,
        theme: &Theme,
    ) -> std::result::Result<(), WidgetError> {
        use ratatui::widgets::{Block, Borders, Paragraph};

        let block = Block::default().borders(Borders::ALL).title(self.title());

        let paragraph = Paragraph::new("Mock Widget Content").block(block);

        frame.render_widget(paragraph, area);
        Ok(())
    }

    async fn handle_event(
        &mut self,
        _event: Event,
    ) -> std::result::Result<Option<workflow_toolkit::interfaces::tui::action::Action>, WidgetError>
    {
        Ok(None)
    }
}

/// Test suite for basic Widget functionality
#[cfg(test)]
mod basic_widget_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_widget_creation() -> Result<()> {
        let mut widget = MockWidget::new();

        // Test initialization
        widget.initialize().await?;
        assert_eq!(widget.context().state, WidgetState::Inactive);

        // Test activation
        widget.on_activate().await?;
        assert_eq!(widget.context().state, WidgetState::Active);

        // Test capabilities
        let capabilities = widget.capabilities();
        assert!(capabilities.keyboard_input);
        assert!(capabilities.focusable);

        // Test cleanup
        widget.cleanup().await?;
        assert_eq!(widget.context().state, WidgetState::Uninitialized);

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_lifecycle() -> Result<()> {
        let mut widget = MockWidget::new();

        // Initial state
        assert_eq!(widget.context().state, WidgetState::Uninitialized);

        // Initialize
        widget.initialize().await?;
        assert_eq!(widget.context().state, WidgetState::Inactive);

        // Activate
        widget.on_activate().await?;
        assert_eq!(widget.context().state, WidgetState::Active);

        // Focus
        widget.on_focus().await?;
        assert_eq!(widget.context().state, WidgetState::Focused);
        assert!(widget.context().has_focus);

        // Blur
        widget.on_blur().await?;
        assert_eq!(widget.context().state, WidgetState::Active);
        assert!(!widget.context().has_focus);

        // Deactivate
        widget.on_deactivate().await?;
        assert_eq!(widget.context().state, WidgetState::Inactive);

        // Cleanup
        widget.cleanup().await?;
        assert_eq!(widget.context().state, WidgetState::Uninitialized);

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_event_handling() -> Result<()> {
        let mut widget = MockWidget::new();
        widget.initialize().await?;
        widget.on_activate().await?;

        // Test key event
        let key_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let result = widget.handle_event(key_event).await?;

        // Mock widget should return None for all events
        assert!(result.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_rendering() -> Result<()> {
        let mut widget = MockWidget::new();
        widget.initialize().await?;
        widget.on_activate().await?;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;
        let theme = Theme::default();

        terminal.draw(|frame| {
            let area = frame.area();
            // Use async block to render widget
            let rt = tokio::runtime::Handle::current();
            let _ = rt.block_on(async { widget.render(frame, area, &theme).await });
        })?;

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_update() -> Result<()> {
        let mut widget = MockWidget::new();
        widget.initialize().await?;

        // Test update
        let before_update = widget.context().last_update;
        widget.update().await?;
        let after_update = widget.context().last_update;

        // Should have updated the timestamp
        assert!(after_update.is_some());
        assert!(after_update != before_update);

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_help_text() -> Result<()> {
        let widget = MockWidget::new();
        let help_text = widget.help_text();

        // Mock widget should have empty help text
        assert!(help_text.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_metrics() -> Result<()> {
        let mut widget = MockWidget::new();
        widget.initialize().await?;
        widget.update().await?;

        let metrics = widget.metrics();

        // Should have basic metrics
        assert!(metrics.contains_key("is_active"));
        assert!(metrics.contains_key("has_focus"));
        assert!(metrics.contains_key("is_visible"));

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_validation() -> Result<()> {
        let mut widget = MockWidget::new();
        widget.initialize().await?;

        // Widget should be valid after initialization
        assert!(widget.validate().is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_resize() -> Result<()> {
        let mut widget = MockWidget::new();
        widget.initialize().await?;

        let new_area = Rect::new(10, 10, 50, 20);
        widget.on_resize(new_area).await?;

        // Should have updated the area
        assert_eq!(widget.context().area, Some(new_area));

        Ok(())
    }
}

/// Test suite for Widget interaction patterns
#[cfg(test)]
mod widget_interaction_tests {
    use super::*;

    #[tokio::test]
    async fn test_multiple_widgets() -> Result<()> {
        let mut widget1 = MockWidget::new();
        let mut widget2 = MockWidget::new();

        // Initialize both widgets
        widget1.initialize().await?;
        widget2.initialize().await?;

        // Activate first widget
        widget1.on_activate().await?;
        assert_eq!(widget1.context().state, WidgetState::Active);
        assert_eq!(widget2.context().state, WidgetState::Inactive);

        // Switch to second widget
        widget1.on_deactivate().await?;
        widget2.on_activate().await?;
        assert_eq!(widget1.context().state, WidgetState::Inactive);
        assert_eq!(widget2.context().state, WidgetState::Active);

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_error_handling() -> Result<()> {
        let mut widget = MockWidget::new();

        // Try to handle events before initialization
        let key_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let result = widget.handle_event(key_event).await;

        // Should either succeed or fail gracefully (not panic)
        match result {
            Ok(_) => {}  // Widget handled it gracefully
            Err(_) => {} // Widget returned an error (acceptable)
        }

        Ok(())
    }
}
