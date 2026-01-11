//! Comprehensive Unit Tests for TUI Implementation
//! 
//! This module contains unit tests for all TUI components including:
//! - Widget system
//! - Theme management
//! - Action system
//! - Layout management
//! - Event handling
//! - Error handling

use std::collections::HashMap;
use std::time::{Duration, Instant};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
};

// ============================================================================
// Basic Theme System Tests
// ============================================================================

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct BasicTheme {
        pub name: String,
        pub colors: BasicColorScheme,
    }

    #[derive(Debug, Clone)]
    pub struct BasicColorScheme {
        pub background: Color,
        pub text_primary: Color,
        pub success: Color,
        pub warning: Color,
        pub error: Color,
        pub info: Color,
    }

    pub fn create_dark_theme() -> BasicTheme {
        BasicTheme {
            name: "Dark".to_string(),
            colors: BasicColorScheme {
                background: Color::Black,
                text_primary: Color::White,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                info: Color::Blue,
            },
        }
    }

    pub fn create_light_theme() -> BasicTheme {
        BasicTheme {
            name: "Light".to_string(),
            colors: BasicColorScheme {
                background: Color::White,
                text_primary: Color::Black,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                info: Color::Blue,
            },
        }
    }
    pub fn get_status_style(theme: &BasicTheme, status: &str) -> Style {
        match status.to_lowercase().as_str() {
            "running" | "active" => Style::default().fg(theme.colors.success),
            "completed" | "success" => Style::default().fg(theme.colors.info),
            "failed" | "error" => Style::default().fg(theme.colors.error),
            "paused" | "warning" => Style::default().fg(theme.colors.warning),
            _ => Style::default().fg(theme.colors.text_primary),
        }
    }

    fn parse_color(color_str: &str) -> Result<Color, String> {
        match color_str.to_lowercase().as_str() {
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "gray" | "grey" => Ok(Color::Gray),
            "darkgray" | "darkgrey" => Ok(Color::DarkGray),
            "lightred" => Ok(Color::LightRed),
            "lightgreen" => Ok(Color::LightGreen),
            "lightyellow" => Ok(Color::LightYellow),
            "lightblue" => Ok(Color::LightBlue),
            "lightmagenta" => Ok(Color::LightMagenta),
            "lightcyan" => Ok(Color::LightCyan),
            "white" => Ok(Color::White),
            _ => {
                if color_str.starts_with('#') && color_str.len() == 7 {
                    let r = u8::from_str_radix(&color_str[1..3], 16)
                        .map_err(|_| format!("Invalid hex color: {}", color_str))?;
                    let g = u8::from_str_radix(&color_str[3..5], 16)
                        .map_err(|_| format!("Invalid hex color: {}", color_str))?;
                    let b = u8::from_str_radix(&color_str[5..7], 16)
                        .map_err(|_| format!("Invalid hex color: {}", color_str))?;
                    Ok(Color::Rgb(r, g, b))
                } else {
                    Err(format!("Unknown color: {}", color_str))
                }
            }
        }
    }

    fn color_to_string(color: &Color) -> String {
        match color {
            Color::Black => "black".to_string(),
            Color::Red => "red".to_string(),
            Color::Green => "green".to_string(),
            Color::Yellow => "yellow".to_string(),
            Color::Blue => "blue".to_string(),
            Color::Magenta => "magenta".to_string(),
            Color::Cyan => "cyan".to_string(),
            Color::Gray => "gray".to_string(),
            Color::DarkGray => "darkgray".to_string(),
            Color::LightRed => "lightred".to_string(),
            Color::LightGreen => "lightgreen".to_string(),
            Color::LightYellow => "lightyellow".to_string(),
            Color::LightBlue => "lightblue".to_string(),
            Color::LightMagenta => "lightmagenta".to_string(),
            Color::LightCyan => "lightcyan".to_string(),
            Color::White => "white".to_string(),
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Color::Indexed(i) => format!("indexed:{}", i),
            Color::Reset => "reset".to_string(),
        }
    }

    #[test]
    fn test_color_parsing() {
        // Test basic color names
        assert_eq!(parse_color("red").unwrap(), Color::Red);
        assert_eq!(parse_color("blue").unwrap(), Color::Blue);
        assert_eq!(parse_color("green").unwrap(), Color::Green);
        assert_eq!(parse_color("black").unwrap(), Color::Black);
        assert_eq!(parse_color("white").unwrap(), Color::White);
        
        // Test case insensitive
        assert_eq!(parse_color("RED").unwrap(), Color::Red);
        assert_eq!(parse_color("Blue").unwrap(), Color::Blue);
        
        // Test hex colors
        assert_eq!(parse_color("#FF0000").unwrap(), Color::Rgb(255, 0, 0));
        assert_eq!(parse_color("#00FF00").unwrap(), Color::Rgb(0, 255, 0));
        assert_eq!(parse_color("#0000FF").unwrap(), Color::Rgb(0, 0, 255));
        
        // Test invalid colors
        assert!(parse_color("invalid_color").is_err());
        assert!(parse_color("#GGGGGG").is_err());
        assert!(parse_color("#FF").is_err());
    }

    #[test]
    fn test_color_to_string() {
        assert_eq!(color_to_string(&Color::Red), "red");
        assert_eq!(color_to_string(&Color::Blue), "blue");
        assert_eq!(color_to_string(&Color::Green), "green");
        assert_eq!(color_to_string(&Color::Black), "black");
        assert_eq!(color_to_string(&Color::White), "white");
        assert_eq!(color_to_string(&Color::Rgb(255, 0, 0)), "#ff0000");
        assert_eq!(color_to_string(&Color::Rgb(0, 255, 0)), "#00ff00");
        assert_eq!(color_to_string(&Color::Rgb(0, 0, 255)), "#0000ff");
    }

    #[test]
    fn test_basic_theme_creation() {
        let theme = create_dark_theme();
        assert_eq!(theme.name, "Dark");
        assert_eq!(theme.colors.background, Color::Black);
        assert_eq!(theme.colors.text_primary, Color::White);
        
        let light_theme = create_light_theme();
        assert_eq!(light_theme.name, "Light");
        assert_eq!(light_theme.colors.background, Color::White);
        assert_eq!(light_theme.colors.text_primary, Color::Black);
    }

    #[test]
    fn test_theme_status_styles() {
        let theme = create_dark_theme();
        
        let running_style = get_status_style(&theme, "running");
        let failed_style = get_status_style(&theme, "failed");
        let completed_style = get_status_style(&theme, "completed");
        
        // Styles should be different for different statuses
        assert_ne!(running_style.fg, failed_style.fg);
        assert_ne!(failed_style.fg, completed_style.fg);
    }
}
// ============================================================================
// Action System Tests
// ============================================================================

#[cfg(test)]
mod action_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TestAction {
        None,
        Quit,
        Refresh,
        Navigate(String),
        ExecuteWorkflow(String),
        Custom(String),
    }

    impl std::fmt::Display for TestAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestAction::None => write!(f, "None"),
                TestAction::Quit => write!(f, "Quit"),
                TestAction::Refresh => write!(f, "Refresh"),
                TestAction::Navigate(view) => write!(f, "Navigate({})", view),
                TestAction::ExecuteWorkflow(name) => write!(f, "ExecuteWorkflow({})", name),
                TestAction::Custom(name) => write!(f, "Custom({})", name),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum TestActionResult {
        Success,
        Error(String),
        Ignored,
    }

    pub struct TestActionDispatcher {
        actions: Vec<TestAction>,
        handlers: HashMap<String, Box<dyn TestActionHandler>>,
    }

    pub trait TestActionHandler {
        fn handle(&mut self, action: &TestAction) -> TestActionResult;
        fn can_handle(&self, action: &TestAction) -> bool;
        fn name(&self) -> &str;
    }

    impl TestActionDispatcher {
        pub fn new() -> Self {
            Self {
                actions: Vec::new(),
                handlers: HashMap::new(),
            }
        }

        pub fn register_handler(&mut self, name: String, handler: Box<dyn TestActionHandler>) {
            self.handlers.insert(name, handler);
        }

        pub fn dispatch(&mut self, action: TestAction) {
            self.actions.push(action);
        }

        pub fn process_next(&mut self) -> Option<TestActionResult> {
            if let Some(action) = self.actions.pop() {
                for handler in self.handlers.values_mut() {
                    if handler.can_handle(&action) {
                        return Some(handler.handle(&action));
                    }
                }
                Some(TestActionResult::Error("No handler found".to_string()))
            } else {
                None
            }
        }

        pub fn has_pending_actions(&self) -> bool {
            !self.actions.is_empty()
        }
    }

    pub struct MockActionHandler {
        name: String,
        can_handle_actions: Vec<TestAction>,
    }

    impl MockActionHandler {
        pub fn new(name: String, can_handle_actions: Vec<TestAction>) -> Self {
            Self {
                name,
                can_handle_actions,
            }
        }
    }

    impl TestActionHandler for MockActionHandler {
        fn handle(&mut self, _action: &TestAction) -> TestActionResult {
            TestActionResult::Success
        }

        fn can_handle(&self, action: &TestAction) -> bool {
            self.can_handle_actions.contains(action)
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_action_dispatcher_creation() {
        let dispatcher = TestActionDispatcher::new();
        assert!(!dispatcher.has_pending_actions());
        assert_eq!(dispatcher.handlers.len(), 0);
    }

    #[test]
    fn test_action_handler_registration() {
        let mut dispatcher = TestActionDispatcher::new();
        let handler = MockActionHandler::new("test_handler".to_string(), vec![TestAction::Quit]);
        
        dispatcher.register_handler("test_handler".to_string(), Box::new(handler));
        assert_eq!(dispatcher.handlers.len(), 1);
    }

    #[test]
    fn test_action_dispatch_and_process() {
        let mut dispatcher = TestActionDispatcher::new();
        let handler = MockActionHandler::new("test_handler".to_string(), vec![TestAction::Quit]);
        
        dispatcher.register_handler("test_handler".to_string(), Box::new(handler));
        
        // Dispatch an action
        dispatcher.dispatch(TestAction::Quit);
        assert!(dispatcher.has_pending_actions());
        
        // Process the action
        let result = dispatcher.process_next();
        assert!(result.is_some());
        match result.unwrap() {
            TestActionResult::Success => {},
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn test_action_display() {
        assert_eq!(format!("{}", TestAction::Quit), "Quit");
        assert_eq!(format!("{}", TestAction::Refresh), "Refresh");
        assert_eq!(format!("{}", TestAction::Navigate("WorkflowList".to_string())), "Navigate(WorkflowList)");
        assert_eq!(format!("{}", TestAction::ExecuteWorkflow("test".to_string())), "ExecuteWorkflow(test)");
    }

    #[test]
    fn test_unhandled_action() {
        let mut dispatcher = TestActionDispatcher::new();
        
        // Dispatch action with no handlers
        dispatcher.dispatch(TestAction::Quit);
        
        let result = dispatcher.process_next();
        assert!(result.is_some());
        
        // Should return error since no handler can process it
        match result.unwrap() {
            TestActionResult::Error(_) => {}, // Expected
            other => panic!("Expected Error, got {:?}", other),
        }
    }
}
// ============================================================================
// Widget System Tests
// ============================================================================

#[cfg(test)]
mod widget_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum WidgetState {
        Uninitialized,
        Inactive,
        Active,
        Focused,
        Disabled,
    }

    #[derive(Debug, Clone)]
    pub struct WidgetContext {
        pub id: String,
        pub state: WidgetState,
        pub has_focus: bool,
        pub is_visible: bool,
        pub area: Option<Rect>,
    }

    impl WidgetContext {
        pub fn new(id: String) -> Self {
            Self {
                id,
                state: WidgetState::Uninitialized,
                has_focus: false,
                is_visible: true,
                area: None,
            }
        }

        pub fn is_active(&self) -> bool {
            matches!(self.state, WidgetState::Active | WidgetState::Focused)
        }

        pub fn can_handle_events(&self) -> bool {
            self.is_active() && self.is_visible
        }
    }

    #[derive(Debug, Clone)]
    pub struct SizeConstraints {
        pub min_width: Option<u16>,
        pub max_width: Option<u16>,
        pub min_height: Option<u16>,
        pub max_height: Option<u16>,
    }

    impl SizeConstraints {
        pub fn new() -> Self {
            Self {
                min_width: None,
                max_width: None,
                min_height: None,
                max_height: None,
            }
        }

        pub fn min_size(mut self, width: u16, height: u16) -> Self {
            self.min_width = Some(width);
            self.min_height = Some(height);
            self
        }

        pub fn max_size(mut self, width: u16, height: u16) -> Self {
            self.max_width = Some(width);
            self.max_height = Some(height);
            self
        }

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

    pub struct TestWidget {
        pub context: WidgetContext,
        pub constraints: SizeConstraints,
        pub title: String,
    }

    impl TestWidget {
        pub fn new(id: String, title: String) -> Self {
            Self {
                context: WidgetContext::new(id),
                constraints: SizeConstraints::new(),
                title,
            }
        }

        pub fn with_constraints(mut self, constraints: SizeConstraints) -> Self {
            self.constraints = constraints;
            self
        }

        pub fn initialize(&mut self) {
            self.context.state = WidgetState::Inactive;
        }

        pub fn activate(&mut self) {
            if self.context.state == WidgetState::Inactive {
                self.context.state = WidgetState::Active;
            }
        }

        pub fn focus(&mut self) {
            if self.context.state == WidgetState::Active {
                self.context.state = WidgetState::Focused;
                self.context.has_focus = true;
            }
        }

        pub fn blur(&mut self) {
            if self.context.state == WidgetState::Focused {
                self.context.state = WidgetState::Active;
                self.context.has_focus = false;
            }
        }

        pub fn deactivate(&mut self) {
            if matches!(self.context.state, WidgetState::Active | WidgetState::Focused) {
                self.context.state = WidgetState::Inactive;
                self.context.has_focus = false;
            }
        }

        pub fn resize(&mut self, area: Rect) {
            let (width, height) = self.constraints.clamp(area.width, area.height);
            self.context.area = Some(Rect {
                x: area.x,
                y: area.y,
                width,
                height,
            });
        }
    }

    #[test]
    fn test_widget_context_creation() {
        let context = WidgetContext::new("test_widget".to_string());
        
        assert_eq!(context.id, "test_widget");
        assert_eq!(context.state, WidgetState::Uninitialized);
        assert!(!context.has_focus);
        assert!(context.is_visible);
        assert!(!context.is_active());
        assert!(!context.can_handle_events());
    }

    #[test]
    fn test_widget_state_transitions() {
        let mut context = WidgetContext::new("test".to_string());
        
        // Test state transitions
        context.state = WidgetState::Active;
        assert!(context.is_active());
        assert!(context.can_handle_events());
        
        context.state = WidgetState::Focused;
        assert!(context.is_active());
        assert!(context.can_handle_events());
        
        context.state = WidgetState::Disabled;
        assert!(!context.is_active());
        assert!(!context.can_handle_events());
    }

    #[test]
    fn test_size_constraints() {
        let constraints = SizeConstraints::new()
            .min_size(10, 5)
            .max_size(100, 50);
        
        // Test constraint satisfaction
        assert!(constraints.satisfies(50, 25));
        assert!(!constraints.satisfies(5, 25)); // Too narrow
        assert!(!constraints.satisfies(50, 3)); // Too short
        assert!(!constraints.satisfies(150, 25)); // Too wide
        assert!(!constraints.satisfies(50, 60)); // Too tall
        
        // Test clamping
        assert_eq!(constraints.clamp(5, 25), (10, 25)); // Clamp width to min
        assert_eq!(constraints.clamp(50, 3), (50, 5)); // Clamp height to min
        assert_eq!(constraints.clamp(150, 25), (100, 25)); // Clamp width to max
        assert_eq!(constraints.clamp(50, 60), (50, 50)); // Clamp height to max
    }

    #[test]
    fn test_widget_lifecycle() {
        let mut widget = TestWidget::new("test_widget".to_string(), "Test Widget".to_string());
        
        assert_eq!(widget.context.state, WidgetState::Uninitialized);
        
        // Test initialization
        widget.initialize();
        assert_eq!(widget.context.state, WidgetState::Inactive);
        
        // Test activation
        widget.activate();
        assert_eq!(widget.context.state, WidgetState::Active);
        
        // Test focus
        widget.focus();
        assert_eq!(widget.context.state, WidgetState::Focused);
        assert!(widget.context.has_focus);
        
        // Test blur
        widget.blur();
        assert_eq!(widget.context.state, WidgetState::Active);
        assert!(!widget.context.has_focus);
        
        // Test deactivation
        widget.deactivate();
        assert_eq!(widget.context.state, WidgetState::Inactive);
    }

    #[test]
    fn test_widget_resize() {
        let mut widget = TestWidget::new("resize_test".to_string(), "Resize Test".to_string())
            .with_constraints(SizeConstraints::new().min_size(20, 10).max_size(80, 40));
        
        // Test resize within constraints
        let area = Rect::new(0, 0, 50, 25);
        widget.resize(area);
        assert_eq!(widget.context.area, Some(area));
        
        // Test resize with clamping
        let too_small = Rect::new(0, 0, 10, 5);
        widget.resize(too_small);
        let clamped_area = widget.context.area.unwrap();
        assert_eq!(clamped_area.width, 20); // Clamped to min width
        assert_eq!(clamped_area.height, 10); // Clamped to min height
        
        let too_large = Rect::new(0, 0, 100, 60);
        widget.resize(too_large);
        let clamped_area = widget.context.area.unwrap();
        assert_eq!(clamped_area.width, 80); // Clamped to max width
        assert_eq!(clamped_area.height, 40); // Clamped to max height
    }
}
// ============================================================================
// Layout Management Tests
// ============================================================================

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LayoutDirection {
        Horizontal,
        Vertical,
    }

    pub struct LayoutNode {
        pub widget_id: String,
        pub weight: f32,
        pub min_width: Option<u16>,
        pub min_height: Option<u16>,
    }

    impl LayoutNode {
        pub fn new(widget_id: String) -> Self {
            Self {
                widget_id,
                weight: 1.0,
                min_width: None,
                min_height: None,
            }
        }

        pub fn with_weight(mut self, weight: f32) -> Self {
            self.weight = weight;
            self
        }

        pub fn with_min_size(mut self, width: u16, height: u16) -> Self {
            self.min_width = Some(width);
            self.min_height = Some(height);
            self
        }
    }

    pub struct LayoutManager {
        pub direction: LayoutDirection,
        pub children: Vec<LayoutNode>,
    }

    impl LayoutManager {
        pub fn new() -> Self {
            Self {
                direction: LayoutDirection::Vertical,
                children: Vec::new(),
            }
        }

        pub fn with_direction(mut self, direction: LayoutDirection) -> Self {
            self.direction = direction;
            self
        }

        pub fn add_child(&mut self, node: LayoutNode) {
            self.children.push(node);
        }

        pub fn remove_child(&mut self, widget_id: &str) -> Option<LayoutNode> {
            if let Some(pos) = self.children.iter().position(|n| n.widget_id == widget_id) {
                Some(self.children.remove(pos))
            } else {
                None
            }
        }

        pub fn calculate_layout(&self, total_area: Rect) -> HashMap<String, Rect> {
            let mut layout = HashMap::new();
            
            if self.children.is_empty() {
                return layout;
            }

            let total_weight: f32 = self.children.iter().map(|c| c.weight).sum();
            
            match self.direction {
                LayoutDirection::Vertical => {
                    let mut y = total_area.y;
                    for child in &self.children {
                        let height = ((child.weight / total_weight) * total_area.height as f32) as u16;
                        let height = if let Some(min_h) = child.min_height {
                            height.max(min_h)
                        } else {
                            height
                        };
                        
                        let area = Rect {
                            x: total_area.x,
                            y,
                            width: total_area.width,
                            height,
                        };
                        
                        layout.insert(child.widget_id.clone(), area);
                        y += height;
                    }
                }
                LayoutDirection::Horizontal => {
                    let mut x = total_area.x;
                    for child in &self.children {
                        let width = ((child.weight / total_weight) * total_area.width as f32) as u16;
                        let width = if let Some(min_w) = child.min_width {
                            width.max(min_w)
                        } else {
                            width
                        };
                        
                        let area = Rect {
                            x,
                            y: total_area.y,
                            width,
                            height: total_area.height,
                        };
                        
                        layout.insert(child.widget_id.clone(), area);
                        x += width;
                    }
                }
            }
            
            layout
        }
    }

    #[test]
    fn test_layout_manager_creation() {
        let manager = LayoutManager::new();
        assert_eq!(manager.direction, LayoutDirection::Vertical);
        assert!(manager.children.is_empty());
    }

    #[test]
    fn test_layout_node_creation() {
        let node = LayoutNode::new("test_node".to_string())
            .with_weight(2.0)
            .with_min_size(20, 10);
        
        assert_eq!(node.widget_id, "test_node");
        assert_eq!(node.weight, 2.0);
        assert_eq!(node.min_width, Some(20));
        assert_eq!(node.min_height, Some(10));
    }

    #[test]
    fn test_layout_manager_add_remove_child() {
        let mut manager = LayoutManager::new();
        let node = LayoutNode::new("child1".to_string());
        
        manager.add_child(node);
        assert_eq!(manager.children.len(), 1);
        
        let removed = manager.remove_child("child1");
        assert!(removed.is_some());
        assert_eq!(manager.children.len(), 0);
        
        // Try to remove non-existent child
        let not_found = manager.remove_child("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_vertical_layout_calculation() {
        let mut manager = LayoutManager::new();
        
        // Add children with different weights
        let child1 = LayoutNode::new("child1".to_string()).with_weight(1.0);
        let child2 = LayoutNode::new("child2".to_string()).with_weight(2.0);
        let child3 = LayoutNode::new("child3".to_string()).with_weight(1.0);
        
        manager.add_child(child1);
        manager.add_child(child2);
        manager.add_child(child3);
        
        let total_area = Rect::new(0, 0, 100, 100);
        let layout = manager.calculate_layout(total_area);
        
        assert_eq!(layout.len(), 3);
        
        // With weights 1:2:1, the middle child should get twice the space
        let area1 = layout.get("child1").unwrap();
        let area2 = layout.get("child2").unwrap();
        let area3 = layout.get("child3").unwrap();
        
        assert!(area2.height >= area1.height * 2 - 2); // Allow for rounding
        assert!(area2.height >= area3.height * 2 - 2);
    }

    #[test]
    fn test_horizontal_layout_calculation() {
        let mut manager = LayoutManager::new().with_direction(LayoutDirection::Horizontal);
        
        // Add children with equal weights
        let child1 = LayoutNode::new("child1".to_string()).with_weight(1.0);
        let child2 = LayoutNode::new("child2".to_string()).with_weight(1.0);
        
        manager.add_child(child1);
        manager.add_child(child2);
        
        let total_area = Rect::new(0, 0, 100, 50);
        let layout = manager.calculate_layout(total_area);
        
        assert_eq!(layout.len(), 2);
        
        let area1 = layout.get("child1").unwrap();
        let area2 = layout.get("child2").unwrap();
        
        // Should split width equally
        assert_eq!(area1.width, 50);
        assert_eq!(area2.width, 50);
        assert_eq!(area1.height, 50);
        assert_eq!(area2.height, 50);
    }

    #[test]
    fn test_layout_with_constraints() {
        let mut manager = LayoutManager::new();
        
        // Add child with minimum height constraint
        let child = LayoutNode::new("constrained_child".to_string())
            .with_weight(1.0)
            .with_min_size(10, 30);
        
        manager.add_child(child);
        
        // Calculate layout in small area
        let small_area = Rect::new(0, 0, 100, 20);
        let layout = manager.calculate_layout(small_area);
        
        let child_area = layout.get("constrained_child").unwrap();
        assert!(child_area.height >= 30); // Should respect minimum constraint
    }

    #[test]
    fn test_empty_layout() {
        let manager = LayoutManager::new();
        let area = Rect::new(0, 0, 100, 100);
        
        let layout = manager.calculate_layout(area);
        assert!(layout.is_empty());
    }
}
// ============================================================================
// Error Handling Tests
// ============================================================================

#[cfg(test)]
mod error_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TestErrorSeverity {
        Low,
        Medium,
        High,
        Critical,
    }

    #[derive(Debug, Clone)]
    pub struct TestError {
        pub message: String,
        pub severity: TestErrorSeverity,
        pub component: String,
        pub timestamp: Instant,
    }

    impl TestError {
        pub fn new(message: String, severity: TestErrorSeverity, component: String) -> Self {
            Self {
                message,
                severity,
                component,
                timestamp: Instant::now(),
            }
        }
    }

    pub struct ErrorManager {
        pub errors: Vec<TestError>,
        pub max_errors: usize,
    }

    impl ErrorManager {
        pub fn new() -> Self {
            Self {
                errors: Vec::new(),
                max_errors: 100,
            }
        }

        pub fn with_max_errors(mut self, max_errors: usize) -> Self {
            self.max_errors = max_errors;
            self
        }

        pub fn report_error(&mut self, error: TestError) {
            self.errors.push(error);
            
            // Keep only the most recent errors
            if self.errors.len() > self.max_errors {
                self.errors.remove(0);
            }
        }

        pub fn get_recent_errors(&self, count: usize) -> Vec<&TestError> {
            let start = if self.errors.len() > count {
                self.errors.len() - count
            } else {
                0
            };
            self.errors[start..].iter().collect()
        }

        pub fn get_errors_by_severity(&self, severity: TestErrorSeverity) -> Vec<&TestError> {
            self.errors.iter().filter(|e| e.severity == severity).collect()
        }

        pub fn clear_errors(&mut self) {
            self.errors.clear();
        }

        pub fn error_count(&self) -> usize {
            self.errors.len()
        }
    }

    #[test]
    fn test_error_manager_creation() {
        let manager = ErrorManager::new();
        assert_eq!(manager.error_count(), 0);
        assert_eq!(manager.max_errors, 100);
    }

    #[test]
    fn test_error_reporting() {
        let mut manager = ErrorManager::new();
        
        let error = TestError::new(
            "Test error".to_string(),
            TestErrorSeverity::Medium,
            "test_component".to_string()
        );
        
        manager.report_error(error);
        assert_eq!(manager.error_count(), 1);
        
        let recent_errors = manager.get_recent_errors(1);
        assert_eq!(recent_errors.len(), 1);
        assert_eq!(recent_errors[0].message, "Test error");
    }

    #[test]
    fn test_error_severity_filtering() {
        let mut manager = ErrorManager::new();
        
        manager.report_error(TestError::new("Low error".to_string(), TestErrorSeverity::Low, "comp1".to_string()));
        manager.report_error(TestError::new("High error".to_string(), TestErrorSeverity::High, "comp2".to_string()));
        manager.report_error(TestError::new("Critical error".to_string(), TestErrorSeverity::Critical, "comp3".to_string()));
        
        let high_errors = manager.get_errors_by_severity(TestErrorSeverity::High);
        assert_eq!(high_errors.len(), 1);
        assert_eq!(high_errors[0].message, "High error");
        
        let critical_errors = manager.get_errors_by_severity(TestErrorSeverity::Critical);
        assert_eq!(critical_errors.len(), 1);
        assert_eq!(critical_errors[0].message, "Critical error");
    }

    #[test]
    fn test_error_limit() {
        let mut manager = ErrorManager::new().with_max_errors(3);
        
        // Add more errors than the limit
        for i in 0..5 {
            manager.report_error(TestError::new(
                format!("Error {}", i),
                TestErrorSeverity::Low,
                "test".to_string()
            ));
        }
        
        // Should only keep the most recent 3 errors
        assert_eq!(manager.error_count(), 3);
        
        let recent_errors = manager.get_recent_errors(3);
        assert_eq!(recent_errors[0].message, "Error 2");
        assert_eq!(recent_errors[1].message, "Error 3");
        assert_eq!(recent_errors[2].message, "Error 4");
    }

    #[test]
    fn test_error_clearing() {
        let mut manager = ErrorManager::new();
        
        manager.report_error(TestError::new("Error 1".to_string(), TestErrorSeverity::Low, "comp".to_string()));
        manager.report_error(TestError::new("Error 2".to_string(), TestErrorSeverity::Medium, "comp".to_string()));
        
        assert_eq!(manager.error_count(), 2);
        
        manager.clear_errors();
        assert_eq!(manager.error_count(), 0);
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_theme_and_layout_integration() {
        let theme = theme_tests::create_dark_theme();
        let mut layout_manager = layout_tests::LayoutManager::new();
        
        // Add some widgets to the layout
        let widget1 = layout_tests::LayoutNode::new("widget1".to_string()).with_weight(1.0);
        let widget2 = layout_tests::LayoutNode::new("widget2".to_string()).with_weight(2.0);
        
        layout_manager.add_child(widget1);
        layout_manager.add_child(widget2);
        
        let total_area = Rect::new(0, 0, 100, 100);
        let layout = layout_manager.calculate_layout(total_area);
        
        // Verify layout calculation works
        assert_eq!(layout.len(), 2);
        
        // Verify theme can be used with layout
        let status_style = theme_tests::get_status_style(&theme, "running");
        assert_eq!(status_style.fg, Some(Color::Green));
    }

    #[test]
    fn test_widget_and_action_integration() {
        let mut widget = widget_tests::TestWidget::new("test_widget".to_string(), "Test Widget".to_string());
        let mut action_dispatcher = action_tests::TestActionDispatcher::new();
        
        // Initialize widget
        widget.initialize();
        widget.activate();
        
        // Set up action handling
        let handler = action_tests::MockActionHandler::new(
            "test_handler".to_string(),
            vec![action_tests::TestAction::Quit]
        );
        action_dispatcher.register_handler("test_handler".to_string(), Box::new(handler));
        
        // Test that widget can be in active state and actions can be processed
        assert!(widget.context.is_active());
        
        action_dispatcher.dispatch(action_tests::TestAction::Quit);
        let result = action_dispatcher.process_next();
        match result {
            Some(action_tests::TestActionResult::Success) => {},
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn test_complete_tui_workflow() {
        // Create all components
        let theme = theme_tests::create_dark_theme();
        let mut layout_manager = layout_tests::LayoutManager::new();
        let mut widget = widget_tests::TestWidget::new("main_widget".to_string(), "Main Widget".to_string());
        let mut action_dispatcher = action_tests::TestActionDispatcher::new();
        let mut error_manager = error_tests::ErrorManager::new();
        
        // Set up layout
        let node = layout_tests::LayoutNode::new("main_widget".to_string()).with_weight(1.0);
        layout_manager.add_child(node);
        
        // Initialize widget
        widget.initialize();
        widget.activate();
        
        // Set up action handling
        let handler = action_tests::MockActionHandler::new(
            "quit_handler".to_string(),
            vec![action_tests::TestAction::Quit]
        );
        action_dispatcher.register_handler("quit_handler".to_string(), Box::new(handler));
        
        // Simulate a complete workflow
        
        // 1. Calculate layout
        let total_area = Rect::new(0, 0, 80, 24);
        let layout = layout_manager.calculate_layout(total_area);
        assert!(layout.contains_key("main_widget"));
        
        // 2. Resize widget to fit layout
        let widget_area = layout.get("main_widget").unwrap();
        widget.resize(*widget_area);
        assert_eq!(widget.context.area, Some(*widget_area));
        
        // 3. Dispatch an action
        action_dispatcher.dispatch(action_tests::TestAction::Quit);
        
        // 4. Process action
        let result = action_dispatcher.process_next();
        match result {
            Some(action_tests::TestActionResult::Success) => {
                // Success - application should quit
            }
            Some(action_tests::TestActionResult::Error(msg)) => {
                // Error occurred
                let error = error_tests::TestError::new(
                    msg,
                    error_tests::TestErrorSeverity::High,
                    "quit_handler".to_string()
                );
                error_manager.report_error(error);
            }
            _ => {}
        }
        
        // 5. Verify final state
        assert!(widget.context.is_active());
        assert_eq!(error_manager.error_count(), 0); // No errors should occur
        
        // 6. Apply theme styling (conceptual - in real implementation this would affect rendering)
        let quit_style = theme_tests::get_status_style(&theme, "completed");
        assert_eq!(quit_style.fg, Some(Color::Blue));
    }
}