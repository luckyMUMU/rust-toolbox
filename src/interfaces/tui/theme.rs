//! Theme system for TUI interface
//! 
//! This module provides theme management and styling for the TUI components.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Theme configuration for the TUI interface
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub styles: StyleScheme,
    pub metadata: HashMap<String, String>,
}

/// Color scheme definition
#[derive(Debug, Clone)]
pub struct ColorScheme {
    // Primary colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    
    // Background colors
    pub background: Color,
    pub surface: Color,
    pub overlay: Color,
    
    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    
    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // UI element colors
    pub border: Color,
    pub border_focused: Color,
    pub highlight: Color,
    pub selection: Color,
    
    // Progress and status indicators
    pub progress_bar: Color,
    pub progress_background: Color,
    pub status_running: Color,
    pub status_completed: Color,
    pub status_failed: Color,
    pub status_paused: Color,
}

/// Style scheme definition
#[derive(Debug, Clone)]
pub struct StyleScheme {
    // Application-level styles
    pub header: Style,
    pub footer: Style,
    pub status_bar: Style,
    
    // Widget styles
    pub widget_border: Style,
    pub widget_border_focused: Style,
    pub widget_title: Style,
    pub widget_content: Style,
    
    // Text styles
    pub text_normal: Style,
    pub text_bold: Style,
    pub text_italic: Style,
    pub text_underline: Style,
    pub text_dimmed: Style,
    
    // Status styles
    pub success: Style,
    pub warning: Style,
    pub error: Style,
    pub info: Style,
    
    // Interactive element styles
    pub button: Style,
    pub button_focused: Style,
    pub button_pressed: Style,
    pub input: Style,
    pub input_focused: Style,
    
    // List styles
    pub list_item: Style,
    pub list_item_selected: Style,
    pub list_item_focused: Style,
    
    // Progress styles
    pub progress_bar: Style,
    pub progress_text: Style,
    
    // Log styles
    pub log_error: Style,
    pub log_warn: Style,
    pub log_info: Style,
    pub log_debug: Style,
    pub log_trace: Style,
    
    // Search and filter styles
    pub search_highlight: Style,
    pub filter_active: Style,
}

impl Theme {
    /// Create a dark theme
    pub fn dark() -> Self {
        let colors = ColorScheme {
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            
            background: Color::Black,
            surface: Color::DarkGray,
            overlay: Color::Gray,
            
            text_primary: Color::White,
            text_secondary: Color::LightBlue,
            text_disabled: Color::DarkGray,
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            
            border: Color::Gray,
            border_focused: Color::Blue,
            highlight: Color::Blue,
            selection: Color::DarkGray,
            
            progress_bar: Color::Green,
            progress_background: Color::DarkGray,
            status_running: Color::Green,
            status_completed: Color::Blue,
            status_failed: Color::Red,
            status_paused: Color::Yellow,
        };
        
        let styles = StyleScheme::from_colors(&colors);
        
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "Workflow Toolkit".to_string());
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("description".to_string(), "Default dark theme".to_string());
        
        Self {
            name: "Dark".to_string(),
            colors,
            styles,
            metadata,
        }
    }
    
    /// Create a light theme
    pub fn light() -> Self {
        let colors = ColorScheme {
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            
            background: Color::White,
            surface: Color::LightBlue,
            overlay: Color::Gray,
            
            text_primary: Color::Black,
            text_secondary: Color::Blue,
            text_disabled: Color::Gray,
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            
            border: Color::DarkGray,
            border_focused: Color::Blue,
            highlight: Color::Blue,
            selection: Color::Gray,
            
            progress_bar: Color::Green,
            progress_background: Color::Gray,
            status_running: Color::Green,
            status_completed: Color::Blue,
            status_failed: Color::Red,
            status_paused: Color::Yellow,
        };
        
        let styles = StyleScheme::from_colors(&colors);
        
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "Workflow Toolkit".to_string());
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("description".to_string(), "Default light theme".to_string());
        
        Self {
            name: "Light".to_string(),
            colors,
            styles,
            metadata,
        }
    }
    
    /// Create a high contrast theme
    pub fn high_contrast() -> Self {
        let colors = ColorScheme {
            primary: Color::White,
            secondary: Color::Yellow,
            accent: Color::Magenta,
            
            background: Color::Black,
            surface: Color::Black,
            overlay: Color::DarkGray,
            
            text_primary: Color::White,
            text_secondary: Color::Yellow,
            text_disabled: Color::Gray,
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
            
            border: Color::White,
            border_focused: Color::Yellow,
            highlight: Color::Yellow,
            selection: Color::White,
            
            progress_bar: Color::Green,
            progress_background: Color::White,
            status_running: Color::Green,
            status_completed: Color::Cyan,
            status_failed: Color::Red,
            status_paused: Color::Yellow,
        };
        
        let styles = StyleScheme::from_colors(&colors);
        
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "Workflow Toolkit".to_string());
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("description".to_string(), "High contrast theme for accessibility".to_string());
        
        Self {
            name: "HighContrast".to_string(),
            colors,
            styles,
            metadata,
        }
    }
    
    /// Get a style for a specific status
    pub fn status_style(&self, status: &str) -> Style {
        match status.to_lowercase().as_str() {
            "running" | "active" => Style::default().fg(self.colors.status_running),
            "completed" | "success" => Style::default().fg(self.colors.status_completed),
            "failed" | "error" => Style::default().fg(self.colors.status_failed),
            "paused" | "warning" => Style::default().fg(self.colors.status_paused),
            _ => self.styles.text_normal,
        }
    }
    
    /// Get a style for a specific log level
    pub fn log_level_style(&self, level: &str) -> Style {
        match level.to_lowercase().as_str() {
            "error" => self.styles.log_error,
            "warn" | "warning" => self.styles.log_warn,
            "info" => self.styles.log_info,
            "debug" => self.styles.log_debug,
            "trace" => self.styles.log_trace,
            _ => self.styles.text_normal,
        }
    }
    
    /// Check if this is a dark theme
    pub fn is_dark(&self) -> bool {
        // Simple heuristic: if background is darker than text, it's a dark theme
        matches!(self.colors.background, Color::Black | Color::DarkGray)
    }
    
    /// Check if this is a high contrast theme
    pub fn is_high_contrast(&self) -> bool {
        self.name.to_lowercase().contains("contrast")
    }
    
    /// Get theme metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Set theme metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Validate the theme configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }
        
        // Add more validation as needed
        Ok(())
    }
    
    /// Create a custom theme from a configuration
    pub fn from_config(config: ThemeConfig) -> Result<Self, String> {
        let colors = ColorScheme::from_config(&config.colors)?;
        let styles = StyleScheme::from_colors(&colors);
        
        Ok(Self {
            name: config.name,
            colors,
            styles,
            metadata: config.metadata.unwrap_or_default(),
        })
    }
    
    /// Convert theme to configuration format
    pub fn to_config(&self) -> ThemeConfig {
        ThemeConfig {
            name: self.name.clone(),
            colors: self.colors.to_config(),
            metadata: Some(self.metadata.clone()),
        }
    }
}

impl StyleScheme {
    /// Create styles from a color scheme
    pub fn from_colors(colors: &ColorScheme) -> Self {
        Self {
            // Application-level styles
            header: Style::default()
                .fg(colors.text_primary)
                .bg(colors.primary)
                .add_modifier(Modifier::BOLD),
            footer: Style::default()
                .fg(colors.text_secondary)
                .bg(colors.surface),
            status_bar: Style::default()
                .fg(colors.text_secondary)
                .bg(colors.surface),
            
            // Widget styles
            widget_border: Style::default().fg(colors.border),
            widget_border_focused: Style::default()
                .fg(colors.border_focused)
                .add_modifier(Modifier::BOLD),
            widget_title: Style::default()
                .fg(colors.text_primary)
                .add_modifier(Modifier::BOLD),
            widget_content: Style::default().fg(colors.text_primary),
            
            // Text styles
            text_normal: Style::default().fg(colors.text_primary),
            text_bold: Style::default()
                .fg(colors.text_primary)
                .add_modifier(Modifier::BOLD),
            text_italic: Style::default()
                .fg(colors.text_primary)
                .add_modifier(Modifier::ITALIC),
            text_underline: Style::default()
                .fg(colors.text_primary)
                .add_modifier(Modifier::UNDERLINED),
            text_dimmed: Style::default().fg(colors.text_disabled),
            
            // Status styles
            success: Style::default()
                .fg(colors.success)
                .add_modifier(Modifier::BOLD),
            warning: Style::default()
                .fg(colors.warning)
                .add_modifier(Modifier::BOLD),
            error: Style::default()
                .fg(colors.error)
                .add_modifier(Modifier::BOLD),
            info: Style::default()
                .fg(colors.info)
                .add_modifier(Modifier::BOLD),
            
            // Interactive element styles
            button: Style::default()
                .fg(colors.text_primary)
                .bg(colors.surface),
            button_focused: Style::default()
                .fg(colors.text_primary)
                .bg(colors.highlight)
                .add_modifier(Modifier::BOLD),
            button_pressed: Style::default()
                .fg(colors.background)
                .bg(colors.text_primary),
            input: Style::default()
                .fg(colors.text_primary)
                .bg(colors.surface),
            input_focused: Style::default()
                .fg(colors.text_primary)
                .bg(colors.surface)
                .add_modifier(Modifier::UNDERLINED),
            
            // List styles
            list_item: Style::default().fg(colors.text_primary),
            list_item_selected: Style::default()
                .fg(colors.text_primary)
                .bg(colors.selection),
            list_item_focused: Style::default()
                .fg(colors.text_primary)
                .bg(colors.highlight)
                .add_modifier(Modifier::BOLD),
            
            // Progress styles
            progress_bar: Style::default()
                .fg(colors.progress_bar)
                .bg(colors.progress_background),
            progress_text: Style::default().fg(colors.text_secondary),
            
            // Log styles
            log_error: Style::default()
                .fg(colors.error)
                .add_modifier(Modifier::BOLD),
            log_warn: Style::default()
                .fg(colors.warning),
            log_info: Style::default()
                .fg(colors.info),
            log_debug: Style::default()
                .fg(colors.text_secondary),
            log_trace: Style::default()
                .fg(colors.text_disabled),
            
            // Search and filter styles
            search_highlight: Style::default()
                .fg(colors.background)
                .bg(colors.accent)
                .add_modifier(Modifier::BOLD),
            filter_active: Style::default()
                .fg(colors.accent)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl ColorScheme {
    /// Create color scheme from configuration
    pub fn from_config(config: &ColorConfig) -> Result<Self, String> {
        Ok(Self {
            primary: parse_color(&config.primary)?,
            secondary: parse_color(&config.secondary)?,
            accent: parse_color(&config.accent)?,
            
            background: parse_color(&config.background)?,
            surface: parse_color(&config.surface)?,
            overlay: parse_color(&config.overlay)?,
            
            text_primary: parse_color(&config.text_primary)?,
            text_secondary: parse_color(&config.text_secondary)?,
            text_disabled: parse_color(&config.text_disabled)?,
            
            success: parse_color(&config.success)?,
            warning: parse_color(&config.warning)?,
            error: parse_color(&config.error)?,
            info: parse_color(&config.info)?,
            
            border: parse_color(&config.border)?,
            border_focused: parse_color(&config.border_focused)?,
            highlight: parse_color(&config.highlight)?,
            selection: parse_color(&config.selection)?,
            
            progress_bar: parse_color(&config.progress_bar)?,
            progress_background: parse_color(&config.progress_background)?,
            status_running: parse_color(&config.status_running)?,
            status_completed: parse_color(&config.status_completed)?,
            status_failed: parse_color(&config.status_failed)?,
            status_paused: parse_color(&config.status_paused)?,
        })
    }
    
    /// Convert color scheme to configuration format
    pub fn to_config(&self) -> ColorConfig {
        ColorConfig {
            primary: color_to_string(&self.primary),
            secondary: color_to_string(&self.secondary),
            accent: color_to_string(&self.accent),
            
            background: color_to_string(&self.background),
            surface: color_to_string(&self.surface),
            overlay: color_to_string(&self.overlay),
            
            text_primary: color_to_string(&self.text_primary),
            text_secondary: color_to_string(&self.text_secondary),
            text_disabled: color_to_string(&self.text_disabled),
            
            success: color_to_string(&self.success),
            warning: color_to_string(&self.warning),
            error: color_to_string(&self.error),
            info: color_to_string(&self.info),
            
            border: color_to_string(&self.border),
            border_focused: color_to_string(&self.border_focused),
            highlight: color_to_string(&self.highlight),
            selection: color_to_string(&self.selection),
            
            progress_bar: color_to_string(&self.progress_bar),
            progress_background: color_to_string(&self.progress_background),
            status_running: color_to_string(&self.status_running),
            status_completed: color_to_string(&self.status_completed),
            status_failed: color_to_string(&self.status_failed),
            status_paused: color_to_string(&self.status_paused),
        }
    }
}

/// Theme configuration for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub colors: ColorConfig,
    pub metadata: Option<HashMap<String, String>>,
}

/// Color configuration for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    
    pub background: String,
    pub surface: String,
    pub overlay: String,
    
    pub text_primary: String,
    pub text_secondary: String,
    pub text_disabled: String,
    
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    
    pub border: String,
    pub border_focused: String,
    pub highlight: String,
    pub selection: String,
    
    pub progress_bar: String,
    pub progress_background: String,
    pub status_running: String,
    pub status_completed: String,
    pub status_failed: String,
    pub status_paused: String,
}

/// Parse a color from string representation
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
            // Try to parse as RGB hex color
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

/// Convert a color to string representation
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

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Theme manager for handling multiple themes
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    current_theme: String,
}

impl ThemeManager {
    /// Create a new theme manager with default themes
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        
        let dark_theme = Theme::dark();
        let light_theme = Theme::light();
        let high_contrast_theme = Theme::high_contrast();
        
        themes.insert(dark_theme.name.clone(), dark_theme);
        themes.insert(light_theme.name.clone(), light_theme);
        themes.insert(high_contrast_theme.name.clone(), high_contrast_theme);
        
        Self {
            themes,
            current_theme: "Dark".to_string(),
        }
    }
    
    /// Add a theme to the manager
    pub fn add_theme(&mut self, theme: Theme) -> Result<(), String> {
        theme.validate()?;
        self.themes.insert(theme.name.clone(), theme);
        Ok(())
    }
    
    /// Remove a theme from the manager
    pub fn remove_theme(&mut self, name: &str) -> Option<Theme> {
        if name == self.current_theme {
            return None; // Cannot remove current theme
        }
        self.themes.remove(name)
    }
    
    /// Get the current theme
    pub fn current_theme(&self) -> Option<&Theme> {
        self.themes.get(&self.current_theme)
    }
    
    /// Set the current theme
    pub fn set_current_theme(&mut self, name: &str) -> Result<(), String> {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
            Ok(())
        } else {
            Err(format!("Theme not found: {}", name))
        }
    }
    
    /// Get all available theme names
    pub fn theme_names(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }
    
    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }
    
    /// Load themes from configuration
    pub fn load_themes(&mut self, configs: Vec<ThemeConfig>) -> Result<(), String> {
        for config in configs {
            let theme = Theme::from_config(config)?;
            self.add_theme(theme)?;
        }
        Ok(())
    }
    
    /// Save themes to configuration format
    pub fn save_themes(&self) -> Vec<ThemeConfig> {
        self.themes.values().map(|theme| theme.to_config()).collect()
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}