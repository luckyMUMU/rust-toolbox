//! Standalone TUI Unit Tests
//!
//! This module contains comprehensive unit tests for TUI functionality
//! that are completely self-contained and don't depend on the main TUI implementation.
//! These tests validate core TUI concepts, patterns, and edge cases.

use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// Theme System Tests
// ============================================================================

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Color {
        Black,
        Red,
        Green,
        Yellow,
        Blue,
        Magenta,
        Cyan,
        Gray,
        DarkGray,
        LightRed,
        LightGreen,
        LightYellow,
        LightBlue,
        LightMagenta,
        LightCyan,
        White,
        Rgb(u8, u8, u8),
        Indexed(u8),
        Reset,
    }

    #[derive(Debug, Clone)]
    pub struct Style {
        pub fg: Option<Color>,
        pub bg: Option<Color>,
        pub bold: bool,
        pub italic: bool,
        pub underline: bool,
    }

    impl Default for Style {
        fn default() -> Self {
            Self {
                fg: None,
                bg: None,
                bold: false,
                italic: false,
                underline: false,
            }
        }
    }

    impl Style {
        pub fn fg(mut self, color: Color) -> Self {
            self.fg = Some(color);
            self
        }

        pub fn bg(mut self, color: Color) -> Self {
            self.bg = Some(color);
            self
        }

        pub fn bold(mut self) -> Self {
            self.bold = true;
            self
        }
    }

    #[derive(Debug, Clone)]
    pub struct ColorScheme {
        pub background: Color,
        pub text_primary: Color,
        pub text_secondary: Color,
        pub success: Color,
        pub warning: Color,
        pub error: Color,
        pub info: Color,
        pub accent: Color,
    }

    #[derive(Debug, Clone)]
    pub struct Theme {
        pub name: String,
        pub colors: ColorScheme,
    }

    impl Theme {
        pub fn dark() -> Self {
            Self {
                name: "Dark".to_string(),
                colors: ColorScheme {
                    background: Color::Black,
                    text_primary: Color::White,
                    text_secondary: Color::Gray,
                    success: Color::Green,
                    warning: Color::Yellow,
                    error: Color::Red,
                    info: Color::Blue,
                    accent: Color::Cyan,
                },
            }
        }

        pub fn light() -> Self {
            Self {
                name: "Light".to_string(),
                colors: ColorScheme {
                    background: Color::White,
                    text_primary: Color::Black,
                    text_secondary: Color::DarkGray,
                    success: Color::Green,
                    warning: Color::Yellow,
                    error: Color::Red,
                    info: Color::Blue,
                    accent: Color::Magenta,
                },
            }
        }

        pub fn get_status_style(&self, status: &str) -> Style {
            match status.to_lowercase().as_str() {
                "running" | "active" => Style::default().fg(self.colors.success),
                "completed" | "success" => Style::default().fg(self.colors.info),
                "failed" | "error" => Style::default().fg(self.colors.error),
                "paused" | "warning" => Style::default().fg(self.colors.warning),
                _ => Style::default().fg(self.colors.text_primary),
            }
        }
    }

    pub struct ThemeManager {
        current_theme: Theme,
        available_themes: HashMap<String, Theme>,
    }

    impl ThemeManager {
        pub fn new() -> Self {
            let mut themes = HashMap::new();
            let dark = Theme::dark();
            let light = Theme::light();

            themes.insert(dark.name.clone(), dark.clone());
            themes.insert(light.name.clone(), light);

            Self {
                current_theme: dark,
                available_themes: themes,
            }
        }

        pub fn current_theme(&self) -> &Theme {
            &self.current_theme
        }

        pub fn set_theme(&mut self, name: &str) -> Result<(), String> {
            if let Some(theme) = self.available_themes.get(name) {
                self.current_theme = theme.clone();
                Ok(())
            } else {
                Err(format!("Theme '{}' not found", name))
            }
        }

        pub fn available_themes(&self) -> Vec<&str> {
            self.available_themes.keys().map(|s| s.as_str()).collect()
        }
    }

    #[test]
    fn test_theme_creation() {
        let dark_theme = Theme::dark();
        assert_eq!(dark_theme.name, "Dark");
        assert_eq!(dark_theme.colors.background, Color::Black);
        assert_eq!(dark_theme.colors.text_primary, Color::White);

        let light_theme = Theme::light();
        assert_eq!(light_theme.name, "Light");
        assert_eq!(light_theme.colors.background, Color::White);
        assert_eq!(light_theme.colors.text_primary, Color::Black);
    }

    #[test]
    fn test_status_styles() {
        let theme = Theme::dark();

        let running_style = theme.get_status_style("running");
        let failed_style = theme.get_status_style("failed");
        let completed_style = theme.get_status_style("completed");

        assert_eq!(running_style.fg, Some(Color::Green));
        assert_eq!(failed_style.fg, Some(Color::Red));
        assert_eq!(completed_style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();

        assert_eq!(manager.current_theme().name, "Dark");

        assert!(manager.set_theme("Light").is_ok());
        assert_eq!(manager.current_theme().name, "Light");

        assert!(manager.set_theme("NonExistent").is_err());

        let available = manager.available_themes();
        assert!(available.contains(&"Dark"));
        assert!(available.contains(&"Light"));
    }

    #[test]
    fn test_style_builder() {
        let style = Style::default().fg(Color::Red).bg(Color::Blue).bold();

        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::Blue));
        assert!(style.bold);
        assert!(!style.italic);
    }
}

// ============================================================================
// Widget System Tests
// ============================================================================

#[cfg(test)]
mod widget_tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Rect {
        pub x: u16,
        pub y: u16,
        pub width: u16,
        pub height: u16,
    }

    impl Rect {
        pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
            Self {
                x,
                y,
                width,
                height,
            }
        }

        pub fn area(&self) -> u32 {
            self.width as u32 * self.height as u32
        }
    }

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
        pub last_update: Option<Instant>,
    }

    impl WidgetContext {
        pub fn new(id: String) -> Self {
            Self {
                id,
                state: WidgetState::Uninitialized,
                has_focus: false,
                is_visible: true,
                area: None,
                last_update: None,
            }
        }

        pub fn is_active(&self) -> bool {
            matches!(self.state, WidgetState::Active | WidgetState::Focused)
        }

        pub fn can_handle_events(&self) -> bool {
            self.is_active() && self.is_visible
        }

        pub fn update_timestamp(&mut self) {
            self.last_update = Some(Instant::now());
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
                if width < min_w {
                    return false;
                }
            }
            if let Some(max_w) = self.max_width {
                if width > max_w {
                    return false;
                }
            }
            if let Some(min_h) = self.min_height {
                if height < min_h {
                    return false;
                }
            }
            if let Some(max_h) = self.max_height {
                if height > max_h {
                    return false;
                }
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

    pub trait Widget {
        fn id(&self) -> &str;
        fn context(&self) -> &WidgetContext;
        fn context_mut(&mut self) -> &mut WidgetContext;
        fn constraints(&self) -> &SizeConstraints;

        fn initialize(&mut self) {
            self.context_mut().state = WidgetState::Inactive;
            self.context_mut().update_timestamp();
        }

        fn activate(&mut self) {
            if self.context().state == WidgetState::Inactive {
                self.context_mut().state = WidgetState::Active;
                self.context_mut().update_timestamp();
            }
        }

        fn focus(&mut self) {
            if self.context().state == WidgetState::Active {
                self.context_mut().state = WidgetState::Focused;
                self.context_mut().has_focus = true;
                self.context_mut().update_timestamp();
            }
        }

        fn blur(&mut self) {
            if self.context().state == WidgetState::Focused {
                self.context_mut().state = WidgetState::Active;
                self.context_mut().has_focus = false;
                self.context_mut().update_timestamp();
            }
        }

        fn resize(&mut self, area: Rect) {
            let (width, height) = self.constraints().clamp(area.width, area.height);
            self.context_mut().area = Some(Rect {
                x: area.x,
                y: area.y,
                width,
                height,
            });
            self.context_mut().update_timestamp();
        }
    }

    pub struct TestWidget {
        context: WidgetContext,
        constraints: SizeConstraints,
        title: String,
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
    }

    impl Widget for TestWidget {
        fn id(&self) -> &str {
            &self.context.id
        }

        fn context(&self) -> &WidgetContext {
            &self.context
        }

        fn context_mut(&mut self) -> &mut WidgetContext {
            &mut self.context
        }

        fn constraints(&self) -> &SizeConstraints {
            &self.constraints
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
    fn test_widget_lifecycle() {
        let mut widget = TestWidget::new("test".to_string(), "Test Widget".to_string());

        assert_eq!(widget.context().state, WidgetState::Uninitialized);

        widget.initialize();
        assert_eq!(widget.context().state, WidgetState::Inactive);

        widget.activate();
        assert_eq!(widget.context().state, WidgetState::Active);
        assert!(widget.context().is_active());

        widget.focus();
        assert_eq!(widget.context().state, WidgetState::Focused);
        assert!(widget.context().has_focus);

        widget.blur();
        assert_eq!(widget.context().state, WidgetState::Active);
        assert!(!widget.context().has_focus);
    }

    #[test]
    fn test_size_constraints() {
        let constraints = SizeConstraints::new().min_size(10, 5).max_size(100, 50);

        assert!(constraints.satisfies(50, 25));
        assert!(!constraints.satisfies(5, 25)); // Too narrow
        assert!(!constraints.satisfies(50, 3)); // Too short
        assert!(!constraints.satisfies(150, 25)); // Too wide
        assert!(!constraints.satisfies(50, 60)); // Too tall

        assert_eq!(constraints.clamp(5, 25), (10, 25));
        assert_eq!(constraints.clamp(150, 25), (100, 25));
    }

    #[test]
    fn test_widget_resize() {
        let mut widget = TestWidget::new("resize_test".to_string(), "Resize Test".to_string())
            .with_constraints(SizeConstraints::new().min_size(20, 10).max_size(80, 40));

        let area = Rect::new(0, 0, 50, 25);
        widget.resize(area);
        assert_eq!(widget.context().area, Some(area));

        let too_small = Rect::new(0, 0, 10, 5);
        widget.resize(too_small);
        let clamped_area = widget.context().area.unwrap();
        assert_eq!(clamped_area.width, 20);
        assert_eq!(clamped_area.height, 10);
    }

    #[test]
    fn test_rect_operations() {
        let rect = Rect::new(10, 20, 30, 40);
        assert_eq!(rect.area(), 1200);

        let small_rect = Rect::new(0, 0, 1, 1);
        assert_eq!(small_rect.area(), 1);
    }
}

// ============================================================================
// Layout System Tests
// ============================================================================

#[cfg(test)]
mod layout_tests {
    use super::*;
    use widget_tests::Rect;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LayoutDirection {
        Horizontal,
        Vertical,
    }

    #[derive(Debug, Clone)]
    pub struct LayoutNode {
        pub widget_id: String,
        pub weight: f32,
        pub min_width: Option<u16>,
        pub min_height: Option<u16>,
        pub max_width: Option<u16>,
        pub max_height: Option<u16>,
    }

    impl LayoutNode {
        pub fn new(widget_id: String) -> Self {
            Self {
                widget_id,
                weight: 1.0,
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
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

        pub fn with_max_size(mut self, width: u16, height: u16) -> Self {
            self.max_width = Some(width);
            self.max_height = Some(height);
            self
        }
    }

    pub struct LayoutManager {
        pub direction: LayoutDirection,
        pub children: Vec<LayoutNode>,
        pub spacing: u16,
    }

    impl LayoutManager {
        pub fn new() -> Self {
            Self {
                direction: LayoutDirection::Vertical,
                children: Vec::new(),
                spacing: 0,
            }
        }

        pub fn with_direction(mut self, direction: LayoutDirection) -> Self {
            self.direction = direction;
            self
        }

        pub fn with_spacing(mut self, spacing: u16) -> Self {
            self.spacing = spacing;
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
            let total_spacing = if self.children.len() > 1 {
                (self.children.len() - 1) as u16 * self.spacing
            } else {
                0
            };

            match self.direction {
                LayoutDirection::Vertical => {
                    let available_height = total_area.height.saturating_sub(total_spacing);
                    let mut y = total_area.y;

                    for child in &self.children {
                        let mut height =
                            ((child.weight / total_weight) * available_height as f32) as u16;

                        if let Some(min_h) = child.min_height {
                            height = height.max(min_h);
                        }
                        if let Some(max_h) = child.max_height {
                            height = height.min(max_h);
                        }

                        let area = Rect {
                            x: total_area.x,
                            y,
                            width: total_area.width,
                            height,
                        };

                        layout.insert(child.widget_id.clone(), area);
                        y += height + self.spacing;
                    }
                }
                LayoutDirection::Horizontal => {
                    let available_width = total_area.width.saturating_sub(total_spacing);
                    let mut x = total_area.x;

                    for child in &self.children {
                        let mut width =
                            ((child.weight / total_weight) * available_width as f32) as u16;

                        if let Some(min_w) = child.min_width {
                            width = width.max(min_w);
                        }
                        if let Some(max_w) = child.max_width {
                            width = width.min(max_w);
                        }

                        let area = Rect {
                            x,
                            y: total_area.y,
                            width,
                            height: total_area.height,
                        };

                        layout.insert(child.widget_id.clone(), area);
                        x += width + self.spacing;
                    }
                }
            }

            layout
        }

        pub fn get_total_min_size(&self) -> (u16, u16) {
            if self.children.is_empty() {
                return (0, 0);
            }

            match self.direction {
                LayoutDirection::Vertical => {
                    let min_width = self
                        .children
                        .iter()
                        .filter_map(|c| c.min_width)
                        .max()
                        .unwrap_or(0);
                    let min_height = self
                        .children
                        .iter()
                        .filter_map(|c| c.min_height)
                        .sum::<u16>()
                        + (self.children.len() - 1) as u16 * self.spacing;
                    (min_width, min_height)
                }
                LayoutDirection::Horizontal => {
                    let min_width = self
                        .children
                        .iter()
                        .filter_map(|c| c.min_width)
                        .sum::<u16>()
                        + (self.children.len() - 1) as u16 * self.spacing;
                    let min_height = self
                        .children
                        .iter()
                        .filter_map(|c| c.min_height)
                        .max()
                        .unwrap_or(0);
                    (min_width, min_height)
                }
            }
        }
    }

    #[test]
    fn test_layout_manager_creation() {
        let manager = LayoutManager::new();
        assert_eq!(manager.direction, LayoutDirection::Vertical);
        assert!(manager.children.is_empty());
        assert_eq!(manager.spacing, 0);
    }

    #[test]
    fn test_layout_node_builder() {
        let node = LayoutNode::new("test".to_string())
            .with_weight(2.0)
            .with_min_size(10, 5)
            .with_max_size(100, 50);

        assert_eq!(node.widget_id, "test");
        assert_eq!(node.weight, 2.0);
        assert_eq!(node.min_width, Some(10));
        assert_eq!(node.min_height, Some(5));
        assert_eq!(node.max_width, Some(100));
        assert_eq!(node.max_height, Some(50));
    }

    #[test]
    fn test_vertical_layout() {
        let mut manager = LayoutManager::new();

        manager.add_child(LayoutNode::new("child1".to_string()).with_weight(1.0));
        manager.add_child(LayoutNode::new("child2".to_string()).with_weight(2.0));
        manager.add_child(LayoutNode::new("child3".to_string()).with_weight(1.0));

        let total_area = Rect::new(0, 0, 100, 100);
        let layout = manager.calculate_layout(total_area);

        assert_eq!(layout.len(), 3);

        let area1 = layout.get("child1").unwrap();
        let area2 = layout.get("child2").unwrap();
        let area3 = layout.get("child3").unwrap();

        // Child2 should get twice the height of child1 and child3
        assert!(area2.height >= area1.height * 2 - 2); // Allow for rounding
        assert!(area2.height >= area3.height * 2 - 2);
    }

    #[test]
    fn test_horizontal_layout() {
        let mut manager = LayoutManager::new().with_direction(LayoutDirection::Horizontal);

        manager.add_child(LayoutNode::new("left".to_string()).with_weight(1.0));
        manager.add_child(LayoutNode::new("right".to_string()).with_weight(1.0));

        let total_area = Rect::new(0, 0, 100, 50);
        let layout = manager.calculate_layout(total_area);

        let left_area = layout.get("left").unwrap();
        let right_area = layout.get("right").unwrap();

        assert_eq!(left_area.width, 50);
        assert_eq!(right_area.width, 50);
        assert_eq!(left_area.height, 50);
        assert_eq!(right_area.height, 50);
    }

    #[test]
    fn test_layout_with_spacing() {
        let mut manager = LayoutManager::new().with_spacing(5);

        manager.add_child(LayoutNode::new("child1".to_string()));
        manager.add_child(LayoutNode::new("child2".to_string()));

        let total_area = Rect::new(0, 0, 100, 100);
        let layout = manager.calculate_layout(total_area);

        let area1 = layout.get("child1").unwrap();
        let area2 = layout.get("child2").unwrap();

        // Check that spacing is applied
        assert_eq!(area2.y, area1.y + area1.height + 5);
    }

    #[test]
    fn test_layout_constraints() {
        let mut manager = LayoutManager::new();

        manager.add_child(
            LayoutNode::new("constrained".to_string())
                .with_weight(1.0)
                .with_min_size(10, 30),
        );

        let small_area = Rect::new(0, 0, 100, 20);
        let layout = manager.calculate_layout(small_area);

        let area = layout.get("constrained").unwrap();
        assert!(area.height >= 30); // Should respect minimum constraint
    }

    #[test]
    fn test_min_size_calculation() {
        let mut manager = LayoutManager::new().with_spacing(5);

        manager.add_child(LayoutNode::new("child1".to_string()).with_min_size(20, 10));
        manager.add_child(LayoutNode::new("child2".to_string()).with_min_size(30, 15));

        let (min_width, min_height) = manager.get_total_min_size();
        assert_eq!(min_width, 30); // Max of child widths
        assert_eq!(min_height, 30); // Sum of child heights + spacing (10 + 15 + 5)
    }

    #[test]
    fn test_empty_layout() {
        let manager = LayoutManager::new();
        let area = Rect::new(0, 0, 100, 100);

        let layout = manager.calculate_layout(area);
        assert!(layout.is_empty());

        let (min_width, min_height) = manager.get_total_min_size();
        assert_eq!(min_width, 0);
        assert_eq!(min_height, 0);
    }
}

// ============================================================================
// Event System Tests
// ============================================================================

#[cfg(test)]
mod event_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum KeyCode {
        Char(char),
        Enter,
        Escape,
        Tab,
        Backspace,
        Delete,
        Up,
        Down,
        Left,
        Right,
        F(u8),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct KeyEvent {
        pub code: KeyCode,
        pub ctrl: bool,
        pub alt: bool,
        pub shift: bool,
    }

    impl KeyEvent {
        pub fn new(code: KeyCode) -> Self {
            Self {
                code,
                ctrl: false,
                alt: false,
                shift: false,
            }
        }

        pub fn with_ctrl(mut self) -> Self {
            self.ctrl = true;
            self
        }

        pub fn with_alt(mut self) -> Self {
            self.alt = true;
            self
        }

        pub fn with_shift(mut self) -> Self {
            self.shift = true;
            self
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Action {
        None,
        Quit,
        Refresh,
        Navigate(String),
        ExecuteWorkflow(String),
        ShowHelp,
        ToggleTheme,
        FocusNext,
        FocusPrevious,
        Custom(String),
    }

    pub struct KeyBinding {
        pub key: KeyEvent,
        pub action: Action,
        pub description: String,
    }

    impl KeyBinding {
        pub fn new(key: KeyEvent, action: Action, description: String) -> Self {
            Self {
                key,
                action,
                description,
            }
        }
    }

    pub struct EventHandler {
        bindings: Vec<KeyBinding>,
    }

    impl EventHandler {
        pub fn new() -> Self {
            Self {
                bindings: Vec::new(),
            }
        }

        pub fn add_binding(&mut self, binding: KeyBinding) {
            self.bindings.push(binding);
        }

        pub fn handle_key(&self, key: KeyEvent) -> Action {
            for binding in &self.bindings {
                if binding.key == key {
                    return binding.action.clone();
                }
            }
            Action::None
        }

        pub fn get_bindings_for_action(&self, action: &Action) -> Vec<&KeyBinding> {
            self.bindings
                .iter()
                .filter(|b| &b.action == action)
                .collect()
        }

        pub fn remove_binding(&mut self, key: &KeyEvent) -> bool {
            if let Some(pos) = self.bindings.iter().position(|b| &b.key == key) {
                self.bindings.remove(pos);
                true
            } else {
                false
            }
        }
    }

    pub fn create_default_bindings() -> EventHandler {
        let mut handler = EventHandler::new();

        handler.add_binding(KeyBinding::new(
            KeyEvent::new(KeyCode::Char('q')).with_ctrl(),
            Action::Quit,
            "Quit application".to_string(),
        ));

        handler.add_binding(KeyBinding::new(
            KeyEvent::new(KeyCode::F(5)),
            Action::Refresh,
            "Refresh data".to_string(),
        ));

        handler.add_binding(KeyBinding::new(
            KeyEvent::new(KeyCode::F(1)),
            Action::ShowHelp,
            "Show help".to_string(),
        ));

        handler.add_binding(KeyBinding::new(
            KeyEvent::new(KeyCode::Tab),
            Action::FocusNext,
            "Focus next widget".to_string(),
        ));

        handler.add_binding(KeyBinding::new(
            KeyEvent::new(KeyCode::Tab).with_shift(),
            Action::FocusPrevious,
            "Focus previous widget".to_string(),
        ));

        handler
    }

    #[test]
    fn test_key_event_creation() {
        let key = KeyEvent::new(KeyCode::Char('q')).with_ctrl().with_shift();

        assert_eq!(key.code, KeyCode::Char('q'));
        assert!(key.ctrl);
        assert!(key.shift);
        assert!(!key.alt);
    }

    #[test]
    fn test_event_handler_basic() {
        let mut handler = EventHandler::new();

        let quit_key = KeyEvent::new(KeyCode::Char('q')).with_ctrl();
        handler.add_binding(KeyBinding::new(
            quit_key.clone(),
            Action::Quit,
            "Quit".to_string(),
        ));

        assert_eq!(handler.handle_key(quit_key), Action::Quit);
        assert_eq!(
            handler.handle_key(KeyEvent::new(KeyCode::Char('x'))),
            Action::None
        );
    }

    #[test]
    fn test_default_bindings() {
        let handler = create_default_bindings();

        assert_eq!(
            handler.handle_key(KeyEvent::new(KeyCode::Char('q')).with_ctrl()),
            Action::Quit
        );

        assert_eq!(
            handler.handle_key(KeyEvent::new(KeyCode::F(5))),
            Action::Refresh
        );

        assert_eq!(
            handler.handle_key(KeyEvent::new(KeyCode::Tab)),
            Action::FocusNext
        );

        assert_eq!(
            handler.handle_key(KeyEvent::new(KeyCode::Tab).with_shift()),
            Action::FocusPrevious
        );
    }

    #[test]
    fn test_binding_management() {
        let mut handler = EventHandler::new();

        let key = KeyEvent::new(KeyCode::Char('h'));
        handler.add_binding(KeyBinding::new(
            key.clone(),
            Action::ShowHelp,
            "Help".to_string(),
        ));

        assert_eq!(handler.handle_key(key.clone()), Action::ShowHelp);

        assert!(handler.remove_binding(&key));
        assert_eq!(handler.handle_key(key), Action::None);

        assert!(!handler.remove_binding(&KeyEvent::new(KeyCode::Char('x'))));
    }

    #[test]
    fn test_action_lookup() {
        let handler = create_default_bindings();

        let quit_bindings = handler.get_bindings_for_action(&Action::Quit);
        assert_eq!(quit_bindings.len(), 1);
        assert_eq!(quit_bindings[0].key.code, KeyCode::Char('q'));
        assert!(quit_bindings[0].key.ctrl);

        let help_bindings = handler.get_bindings_for_action(&Action::ShowHelp);
        assert_eq!(help_bindings.len(), 1);
        assert_eq!(help_bindings[0].key.code, KeyCode::F(1));
    }
}

// ============================================================================
// Action Dispatcher Tests
// ============================================================================

#[cfg(test)]
mod action_dispatch_tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DispatchAction {
        None,
        Quit,
        Refresh,
        Navigate(String),
        ExecuteWorkflow(String),
        Custom(String),
    }

    impl std::fmt::Display for DispatchAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DispatchAction::None => write!(f, "None"),
                DispatchAction::Quit => write!(f, "Quit"),
                DispatchAction::Refresh => write!(f, "Refresh"),
                DispatchAction::Navigate(view) => write!(f, "Navigate({})", view),
                DispatchAction::ExecuteWorkflow(name) => write!(f, "ExecuteWorkflow({})", name),
                DispatchAction::Custom(name) => write!(f, "Custom({})", name),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum DispatchResult {
        Success,
        Error(String),
        Ignored,
    }

    pub struct ActionDispatcher {
        actions: Vec<DispatchAction>,
        handlers: HashMap<String, Box<dyn ActionHandler>>,
    }

    pub trait ActionHandler {
        fn handle(&mut self, action: &DispatchAction) -> DispatchResult;
        fn can_handle(&self, action: &DispatchAction) -> bool;
        fn name(&self) -> &str;
    }

    impl ActionDispatcher {
        pub fn new() -> Self {
            Self {
                actions: Vec::new(),
                handlers: HashMap::new(),
            }
        }

        pub fn register_handler(&mut self, name: String, handler: Box<dyn ActionHandler>) {
            self.handlers.insert(name, handler);
        }

        pub fn dispatch(&mut self, action: DispatchAction) {
            self.actions.push(action);
        }

        pub fn process_next(&mut self) -> Option<DispatchResult> {
            if let Some(action) = self.actions.pop() {
                for handler in self.handlers.values_mut() {
                    if handler.can_handle(&action) {
                        return Some(handler.handle(&action));
                    }
                }
                Some(DispatchResult::Error("No handler found".to_string()))
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
        can_handle_actions: Vec<DispatchAction>,
    }

    impl MockActionHandler {
        pub fn new(name: String, can_handle_actions: Vec<DispatchAction>) -> Self {
            Self {
                name,
                can_handle_actions,
            }
        }
    }

    impl ActionHandler for MockActionHandler {
        fn handle(&mut self, _action: &DispatchAction) -> DispatchResult {
            DispatchResult::Success
        }

        fn can_handle(&self, action: &DispatchAction) -> bool {
            self.can_handle_actions.contains(action)
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_action_dispatcher_creation() {
        let dispatcher = ActionDispatcher::new();
        assert!(!dispatcher.has_pending_actions());
        assert_eq!(dispatcher.handlers.len(), 0);
    }

    #[test]
    fn test_action_handler_registration() {
        let mut dispatcher = ActionDispatcher::new();
        let handler = MockActionHandler::new("test_handler".to_string(), vec![DispatchAction::Quit]);

        dispatcher.register_handler("test_handler".to_string(), Box::new(handler));
        assert_eq!(dispatcher.handlers.len(), 1);
    }

    #[test]
    fn test_action_dispatch_and_process() {
        let mut dispatcher = ActionDispatcher::new();
        let handler = MockActionHandler::new("test_handler".to_string(), vec![DispatchAction::Quit]);

        dispatcher.register_handler("test_handler".to_string(), Box::new(handler));

        // Dispatch an action
        dispatcher.dispatch(DispatchAction::Quit);
        assert!(dispatcher.has_pending_actions());

        // Process the action
        let result = dispatcher.process_next();
        assert!(result.is_some());
        match result.unwrap() {
            DispatchResult::Success => {}
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn test_action_display() {
        assert_eq!(format!("{}", DispatchAction::Quit), "Quit");
        assert_eq!(format!("{}", DispatchAction::Refresh), "Refresh");
    }

    #[test]
    fn test_unhandled_action() {
        let mut dispatcher = ActionDispatcher::new();

        // Dispatch action with no handlers
        dispatcher.dispatch(DispatchAction::Quit);

        let result = dispatcher.process_next();
        assert!(result.is_some());

        // Should return error since no handler can process it
        match result.unwrap() {
            DispatchResult::Error(_) => {} // Expected
            other => panic!("Expected Error, got {:?}", other),
        }
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[cfg(test)]
mod error_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ErrorSeverity {
        Low,
        Medium,
        High,
        Critical,
    }

    #[derive(Debug, Clone)]
    pub struct TuiError {
        pub message: String,
        pub severity: ErrorSeverity,
        pub component: String,
        pub timestamp: Instant,
        pub recoverable: bool,
    }

    impl TuiError {
        pub fn new(message: String, severity: ErrorSeverity, component: String) -> Self {
            Self {
                message,
                severity,
                component,
                timestamp: Instant::now(),
                recoverable: true,
            }
        }

        pub fn critical(message: String, component: String) -> Self {
            Self {
                message,
                severity: ErrorSeverity::Critical,
                component,
                timestamp: Instant::now(),
                recoverable: false,
            }
        }

        pub fn is_critical(&self) -> bool {
            matches!(self.severity, ErrorSeverity::Critical)
        }
    }

    pub struct ErrorManager {
        errors: Vec<TuiError>,
        max_errors: usize,
        error_counts: HashMap<ErrorSeverity, usize>,
    }

    impl ErrorManager {
        pub fn new() -> Self {
            Self {
                errors: Vec::new(),
                max_errors: 100,
                error_counts: HashMap::new(),
            }
        }

        pub fn with_max_errors(mut self, max_errors: usize) -> Self {
            self.max_errors = max_errors;
            self
        }

        pub fn report_error(&mut self, error: TuiError) {
            *self.error_counts.entry(error.severity.clone()).or_insert(0) += 1;

            self.errors.push(error);

            if self.errors.len() > self.max_errors {
                let removed = self.errors.remove(0);
                if let Some(count) = self.error_counts.get_mut(&removed.severity) {
                    *count = count.saturating_sub(1);
                }
            }
        }

        pub fn get_recent_errors(&self, count: usize) -> Vec<&TuiError> {
            let start = if self.errors.len() > count {
                self.errors.len() - count
            } else {
                0
            };
            self.errors[start..].iter().collect()
        }

        pub fn get_errors_by_severity(&self, severity: ErrorSeverity) -> Vec<&TuiError> {
            self.errors
                .iter()
                .filter(|e| e.severity == severity)
                .collect()
        }

        pub fn get_errors_by_component(&self, component: &str) -> Vec<&TuiError> {
            self.errors
                .iter()
                .filter(|e| e.component == component)
                .collect()
        }

        pub fn clear_errors(&mut self) {
            self.errors.clear();
            self.error_counts.clear();
        }

        pub fn error_count(&self) -> usize {
            self.errors.len()
        }

        pub fn get_error_count_by_severity(&self, severity: &ErrorSeverity) -> usize {
            self.error_counts.get(severity).copied().unwrap_or(0)
        }

        pub fn has_critical_errors(&self) -> bool {
            self.error_counts
                .get(&ErrorSeverity::Critical)
                .copied()
                .unwrap_or(0)
                > 0
        }
    }

    #[test]
    fn test_error_creation() {
        let error = TuiError::new(
            "Test error".to_string(),
            ErrorSeverity::Medium,
            "test_component".to_string(),
        );

        assert_eq!(error.message, "Test error");
        assert_eq!(error.severity, ErrorSeverity::Medium);
        assert_eq!(error.component, "test_component");
        assert!(error.recoverable);
        assert!(!error.is_critical());

        let critical_error = TuiError::critical(
            "Critical error".to_string(),
            "critical_component".to_string(),
        );

        assert!(critical_error.is_critical());
        assert!(!critical_error.recoverable);
    }

    #[test]
    fn test_error_manager_basic() {
        let mut manager = ErrorManager::new();

        assert_eq!(manager.error_count(), 0);
        assert!(!manager.has_critical_errors());

        let error = TuiError::new(
            "Test error".to_string(),
            ErrorSeverity::Medium,
            "test".to_string(),
        );

        manager.report_error(error);
        assert_eq!(manager.error_count(), 1);
        assert_eq!(
            manager.get_error_count_by_severity(&ErrorSeverity::Medium),
            1
        );
    }

    #[test]
    fn test_error_filtering() {
        let mut manager = ErrorManager::new();

        manager.report_error(TuiError::new(
            "Low error".to_string(),
            ErrorSeverity::Low,
            "comp1".to_string(),
        ));

        manager.report_error(TuiError::new(
            "High error".to_string(),
            ErrorSeverity::High,
            "comp2".to_string(),
        ));

        manager.report_error(TuiError::critical(
            "Critical error".to_string(),
            "comp1".to_string(),
        ));

        let high_errors = manager.get_errors_by_severity(ErrorSeverity::High);
        assert_eq!(high_errors.len(), 1);
        assert_eq!(high_errors[0].message, "High error");

        let comp1_errors = manager.get_errors_by_component("comp1");
        assert_eq!(comp1_errors.len(), 2);

        assert!(manager.has_critical_errors());
    }

    #[test]
    fn test_error_limit() {
        let mut manager = ErrorManager::new().with_max_errors(3);

        for i in 0..5 {
            manager.report_error(TuiError::new(
                format!("Error {}", i),
                ErrorSeverity::Low,
                "test".to_string(),
            ));
        }

        assert_eq!(manager.error_count(), 3);

        let recent_errors = manager.get_recent_errors(3);
        assert_eq!(recent_errors[0].message, "Error 2");
        assert_eq!(recent_errors[1].message, "Error 3");
        assert_eq!(recent_errors[2].message, "Error 4");
    }

    #[test]
    fn test_error_counts() {
        let mut manager = ErrorManager::new();

        manager.report_error(TuiError::new(
            "Low 1".to_string(),
            ErrorSeverity::Low,
            "test".to_string(),
        ));
        manager.report_error(TuiError::new(
            "Low 2".to_string(),
            ErrorSeverity::Low,
            "test".to_string(),
        ));
        manager.report_error(TuiError::new(
            "High 1".to_string(),
            ErrorSeverity::High,
            "test".to_string(),
        ));

        assert_eq!(manager.get_error_count_by_severity(&ErrorSeverity::Low), 2);
        assert_eq!(manager.get_error_count_by_severity(&ErrorSeverity::High), 1);
        assert_eq!(
            manager.get_error_count_by_severity(&ErrorSeverity::Critical),
            0
        );

        manager.clear_errors();
        assert_eq!(manager.error_count(), 0);
        assert_eq!(manager.get_error_count_by_severity(&ErrorSeverity::Low), 0);
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_tui_workflow() {
        // Create all components
        let mut theme_manager = theme_tests::ThemeManager::new();
        let mut layout_manager = layout_tests::LayoutManager::new();
        let mut widget =
            widget_tests::TestWidget::new("main_widget".to_string(), "Main Widget".to_string());
        let event_handler = event_tests::create_default_bindings();
        let mut error_manager = error_tests::ErrorManager::new();

        // Set up theme
        assert!(theme_manager.set_theme("Dark").is_ok());
        let theme = theme_manager.current_theme();
        assert_eq!(theme.name, "Dark");

        // Set up layout
        let node = layout_tests::LayoutNode::new("main_widget".to_string()).with_weight(1.0);
        layout_manager.add_child(node);

        // Initialize widget
        widget.initialize();
        widget.activate();
        assert!(widget.context().is_active());

        // Calculate layout
        let total_area = widget_tests::Rect::new(0, 0, 80, 24);
        let layout = layout_manager.calculate_layout(total_area);
        assert!(layout.contains_key("main_widget"));

        // Resize widget to fit layout
        let widget_area = layout.get("main_widget").unwrap();
        widget.resize(*widget_area);
        assert_eq!(widget.context().area, Some(*widget_area));

        // Test event handling
        let quit_key = event_tests::KeyEvent::new(event_tests::KeyCode::Char('q')).with_ctrl();
        let action = event_handler.handle_key(quit_key);
        assert_eq!(action, event_tests::Action::Quit);

        // Test theme styling
        let status_style = theme.get_status_style("running");
        assert_eq!(status_style.fg, Some(theme_tests::Color::Green));

        // Verify no errors occurred
        assert_eq!(error_manager.error_count(), 0);
        assert!(!error_manager.has_critical_errors());
    }

    #[test]
    fn test_responsive_layout_adaptation() {
        let mut layout_manager = layout_tests::LayoutManager::new();

        // Add widgets with different constraints
        layout_manager.add_child(
            layout_tests::LayoutNode::new("header".to_string())
                .with_weight(0.2)
                .with_min_size(0, 3)
                .with_max_size(u16::MAX, 5),
        );

        layout_manager.add_child(
            layout_tests::LayoutNode::new("content".to_string())
                .with_weight(1.0)
                .with_min_size(0, 10),
        );

        layout_manager.add_child(
            layout_tests::LayoutNode::new("footer".to_string())
                .with_weight(0.1)
                .with_min_size(0, 2)
                .with_max_size(u16::MAX, 3),
        );

        // Test large screen
        let large_area = widget_tests::Rect::new(0, 0, 120, 40);
        let large_layout = layout_manager.calculate_layout(large_area);

        let header_area = large_layout.get("header").unwrap();
        let content_area = large_layout.get("content").unwrap();
        let footer_area = large_layout.get("footer").unwrap();

        assert!(header_area.height >= 3);
        assert!(header_area.height <= 5);
        assert!(content_area.height >= 10);
        assert!(footer_area.height >= 2);
        assert!(footer_area.height <= 3);

        // Test small screen
        let small_area = widget_tests::Rect::new(0, 0, 80, 20);
        let small_layout = layout_manager.calculate_layout(small_area);

        let small_header = small_layout.get("header").unwrap();
        let small_content = small_layout.get("content").unwrap();
        let small_footer = small_layout.get("footer").unwrap();

        // Constraints should still be respected
        assert!(small_header.height >= 3);
        assert!(small_content.height >= 10);
        assert!(small_footer.height >= 2);
    }

    #[test]
    fn test_error_recovery_workflow() {
        let mut error_manager = error_tests::ErrorManager::new();
        let mut widget = widget_tests::TestWidget::new(
            "error_test".to_string(),
            "Error Test Widget".to_string(),
        );

        // Simulate a recoverable error
        let recoverable_error = error_tests::TuiError::new(
            "Network timeout".to_string(),
            error_tests::ErrorSeverity::Medium,
            "network".to_string(),
        );

        error_manager.report_error(recoverable_error);
        assert_eq!(error_manager.error_count(), 1);
        assert!(!error_manager.has_critical_errors());

        // Widget should still be functional
        widget.initialize();
        widget.activate();
        assert!(widget.context().is_active());

        // Simulate a critical error
        let critical_error =
            error_tests::TuiError::critical("Memory corruption".to_string(), "memory".to_string());

        error_manager.report_error(critical_error);
        assert!(error_manager.has_critical_errors());

        // In a real application, critical errors would trigger shutdown
        let critical_errors =
            error_manager.get_errors_by_severity(error_tests::ErrorSeverity::Critical);
        assert_eq!(critical_errors.len(), 1);
        assert!(!critical_errors[0].recoverable);
    }
}
