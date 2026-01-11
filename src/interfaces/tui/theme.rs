//! Theme system for TUI interface
//! 
//! This module provides theme management and styling for the TUI components.

use ratatui::style::{Color, Modifier, Style};
use std::path::{Path, PathBuf};
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
    config_path: Option<PathBuf>,
    auto_save: bool,
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
            config_path: None,
            auto_save: false,
        }
    }
    
    /// Create theme manager with configuration file path
    pub fn with_config_path<P: AsRef<Path>>(config_path: P) -> Result<Self, String> {
        let mut manager = Self::new();
        manager.config_path = Some(config_path.as_ref().to_path_buf());
        manager.auto_save = true;
        
        // Load themes from config file if it exists
        if config_path.as_ref().exists() {
            manager.load_themes_from_file(config_path.as_ref())?;
        } else {
            // Save default themes to file
            manager.save_themes_to_file(config_path.as_ref())?;
        }
        
        Ok(manager)
    }
    
    /// Add a theme to the manager
    pub fn add_theme(&mut self, theme: Theme) -> Result<(), String> {
        theme.validate()?;
        self.themes.insert(theme.name.clone(), theme);
        
        // Auto-save if enabled
        if self.auto_save {
            if let Some(ref path) = self.config_path {
                self.save_themes_to_file(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Remove a theme from the manager
    pub fn remove_theme(&mut self, name: &str) -> Option<Theme> {
        if name == self.current_theme {
            return None; // Cannot remove current theme
        }
        
        let removed = self.themes.remove(name);
        
        // Auto-save if enabled
        if self.auto_save && removed.is_some() {
            if let Some(ref path) = self.config_path {
                let _ = self.save_themes_to_file(path);
            }
        }
        
        removed
    }
    
    /// Get the current theme
    pub fn current_theme(&self) -> Option<&Theme> {
        self.themes.get(&self.current_theme)
    }
    
    /// Set the current theme
    pub fn set_current_theme(&mut self, name: &str) -> Result<(), String> {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
            
            // Auto-save if enabled
            if self.auto_save {
                if let Some(ref path) = self.config_path {
                    self.save_current_theme_to_file(path)?;
                }
            }
            
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
    
    /// Get a mutable theme by name
    pub fn get_theme_mut(&mut self, name: &str) -> Option<&mut Theme> {
        self.themes.get_mut(name)
    }
    
    /// Load themes from configuration
    pub fn load_themes(&mut self, configs: Vec<ThemeConfig>) -> Result<(), String> {
        for config in configs {
            let theme = Theme::from_config(config)?;
            self.themes.insert(theme.name.clone(), theme);
        }
        Ok(())
    }
    
    /// Save themes to configuration format
    pub fn save_themes(&self) -> Vec<ThemeConfig> {
        self.themes.values().map(|theme| theme.to_config()).collect()
    }
    
    /// Load themes from file
    pub fn load_themes_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read theme file: {}", e))?;
        
        let theme_data: ThemeFileData = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse theme file: {}", e))?;
        
        // Load themes
        for config in theme_data.themes {
            let theme = Theme::from_config(config)?;
            self.themes.insert(theme.name.clone(), theme);
        }
        
        // Set current theme if specified
        if let Some(current) = theme_data.current_theme {
            if self.themes.contains_key(&current) {
                self.current_theme = current;
            }
        }
        
        Ok(())
    }
    
    /// Save themes to file
    pub fn save_themes_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        // Ensure directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create theme directory: {}", e))?;
        }
        
        let theme_data = ThemeFileData {
            current_theme: Some(self.current_theme.clone()),
            themes: self.save_themes(),
        };
        
        let content = toml::to_string_pretty(&theme_data)
            .map_err(|e| format!("Failed to serialize themes: {}", e))?;
        
        std::fs::write(path.as_ref(), content)
            .map_err(|e| format!("Failed to write theme file: {}", e))?;
        
        Ok(())
    }
    
    /// Save only current theme selection to file
    fn save_current_theme_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        // For now, save the entire theme data
        // In a more sophisticated implementation, we might have separate files
        self.save_themes_to_file(path)
    }
    
    /// Import theme from file
    pub fn import_theme<P: AsRef<Path>>(&mut self, path: P) -> Result<String, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read theme file: {}", e))?;
        
        let theme_config: ThemeConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse theme file: {}", e))?;
        
        let theme = Theme::from_config(theme_config)?;
        let theme_name = theme.name.clone();
        
        self.add_theme(theme)?;
        
        Ok(theme_name)
    }
    
    /// Export theme to file
    pub fn export_theme<P: AsRef<Path>>(&self, theme_name: &str, path: P) -> Result<(), String> {
        let theme = self.get_theme(theme_name)
            .ok_or_else(|| format!("Theme not found: {}", theme_name))?;
        
        let theme_config = theme.to_config();
        let content = toml::to_string_pretty(&theme_config)
            .map_err(|e| format!("Failed to serialize theme: {}", e))?;
        
        std::fs::write(path.as_ref(), content)
            .map_err(|e| format!("Failed to write theme file: {}", e))?;
        
        Ok(())
    }
    
    /// Create a custom theme based on an existing theme
    pub fn create_custom_theme(&mut self, base_theme_name: &str, new_name: String) -> Result<(), String> {
        let base_theme = self.get_theme(base_theme_name)
            .ok_or_else(|| format!("Base theme not found: {}", base_theme_name))?;
        
        let mut new_theme = base_theme.clone();
        new_theme.name = new_name.clone();
        new_theme.set_metadata("author".to_string(), "User".to_string());
        new_theme.set_metadata("version".to_string(), "1.0.0".to_string());
        new_theme.set_metadata("description".to_string(), format!("Custom theme based on {}", base_theme_name));
        new_theme.set_metadata("base_theme".to_string(), base_theme_name.to_string());
        
        self.add_theme(new_theme)?;
        Ok(())
    }
    
    /// Modify a theme's colors
    pub fn modify_theme_colors(&mut self, theme_name: &str, color_modifications: HashMap<String, String>) -> Result<(), String> {
        let theme = self.get_theme_mut(theme_name)
            .ok_or_else(|| format!("Theme not found: {}", theme_name))?;
        
        for (color_key, color_value) in color_modifications {
            match color_key.as_str() {
                "primary" => theme.colors.primary = parse_color(&color_value)?,
                "secondary" => theme.colors.secondary = parse_color(&color_value)?,
                "accent" => theme.colors.accent = parse_color(&color_value)?,
                "background" => theme.colors.background = parse_color(&color_value)?,
                "surface" => theme.colors.surface = parse_color(&color_value)?,
                "text_primary" => theme.colors.text_primary = parse_color(&color_value)?,
                "text_secondary" => theme.colors.text_secondary = parse_color(&color_value)?,
                "success" => theme.colors.success = parse_color(&color_value)?,
                "warning" => theme.colors.warning = parse_color(&color_value)?,
                "error" => theme.colors.error = parse_color(&color_value)?,
                "info" => theme.colors.info = parse_color(&color_value)?,
                _ => return Err(format!("Unknown color key: {}", color_key)),
            }
        }
        
        // Regenerate styles based on new colors
        theme.styles = StyleScheme::from_colors(&theme.colors);
        
        // Auto-save if enabled
        if self.auto_save {
            if let Some(ref path) = self.config_path {
                self.save_themes_to_file(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Get theme statistics
    pub fn get_theme_stats(&self) -> ThemeStats {
        let total_themes = self.themes.len();
        let custom_themes = self.themes.values()
            .filter(|theme| theme.get_metadata("base_theme").is_some())
            .count();
        let builtin_themes = total_themes - custom_themes;
        
        ThemeStats {
            total_themes,
            builtin_themes,
            custom_themes,
            current_theme: self.current_theme.clone(),
        }
    }
    
    /// Validate all themes
    pub fn validate_all_themes(&self) -> Vec<(String, String)> {
        let mut errors = Vec::new();
        
        for (name, theme) in &self.themes {
            if let Err(error) = theme.validate() {
                errors.push((name.clone(), error));
            }
        }
        
        errors
    }
    
    /// Reset to default themes
    pub fn reset_to_defaults(&mut self) -> Result<(), String> {
        self.themes.clear();
        
        let dark_theme = Theme::dark();
        let light_theme = Theme::light();
        let high_contrast_theme = Theme::high_contrast();
        
        self.themes.insert(dark_theme.name.clone(), dark_theme);
        self.themes.insert(light_theme.name.clone(), light_theme);
        self.themes.insert(high_contrast_theme.name.clone(), high_contrast_theme);
        
        self.current_theme = "Dark".to_string();
        
        // Auto-save if enabled
        if self.auto_save {
            if let Some(ref path) = self.config_path {
                self.save_themes_to_file(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Enable or disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }
    
    /// Check if auto-save is enabled
    pub fn is_auto_save_enabled(&self) -> bool {
        self.auto_save
    }
    
    /// Get config file path
    pub fn config_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Theme file data structure for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFileData {
    pub current_theme: Option<String>,
    pub themes: Vec<ThemeConfig>,
}

/// Theme statistics
#[derive(Debug, Clone)]
pub struct ThemeStats {
    pub total_themes: usize,
    pub builtin_themes: usize,
    pub custom_themes: usize,
    pub current_theme: String,
}

/// Theme builder for creating custom themes
pub struct ThemeBuilder {
    name: String,
    colors: ColorScheme,
    metadata: HashMap<String, String>,
}

impl ThemeBuilder {
    /// Create a new theme builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            colors: ColorScheme {
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
            },
            metadata: HashMap::new(),
        }
    }
    
    /// Create theme builder from existing theme
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            name: theme.name.clone(),
            colors: theme.colors.clone(),
            metadata: theme.metadata.clone(),
        }
    }
    
    /// Set primary color
    pub fn primary(mut self, color: Color) -> Self {
        self.colors.primary = color;
        self
    }
    
    /// Set secondary color
    pub fn secondary(mut self, color: Color) -> Self {
        self.colors.secondary = color;
        self
    }
    
    /// Set accent color
    pub fn accent(mut self, color: Color) -> Self {
        self.colors.accent = color;
        self
    }
    
    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.colors.background = color;
        self
    }
    
    /// Set surface color
    pub fn surface(mut self, color: Color) -> Self {
        self.colors.surface = color;
        self
    }
    
    /// Set text colors
    pub fn text_colors(mut self, primary: Color, secondary: Color, disabled: Color) -> Self {
        self.colors.text_primary = primary;
        self.colors.text_secondary = secondary;
        self.colors.text_disabled = disabled;
        self
    }
    
    /// Set status colors
    pub fn status_colors(mut self, success: Color, warning: Color, error: Color, info: Color) -> Self {
        self.colors.success = success;
        self.colors.warning = warning;
        self.colors.error = error;
        self.colors.info = info;
        self
    }
    
    /// Set border colors
    pub fn border_colors(mut self, normal: Color, focused: Color) -> Self {
        self.colors.border = normal;
        self.colors.border_focused = focused;
        self
    }
    
    /// Set metadata
    pub fn metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Set author metadata
    pub fn author(mut self, author: String) -> Self {
        self.metadata.insert("author".to_string(), author);
        self
    }
    
    /// Set description metadata
    pub fn description(mut self, description: String) -> Self {
        self.metadata.insert("description".to_string(), description);
        self
    }
    
    /// Set version metadata
    pub fn version(mut self, version: String) -> Self {
        self.metadata.insert("version".to_string(), version);
        self
    }
    
    /// Build the theme
    pub fn build(self) -> Theme {
        let styles = StyleScheme::from_colors(&self.colors);
        
        Theme {
            name: self.name,
            colors: self.colors,
            styles,
            metadata: self.metadata,
        }
    }
}

/// Theme preset manager for common theme configurations
pub struct ThemePresetManager {
    presets: HashMap<String, ThemeBuilder>,
}

impl ThemePresetManager {
    /// Create a new preset manager with default presets
    pub fn new() -> Self {
        let mut presets = HashMap::new();
        
        // Solarized Dark preset
        let solarized_dark = ThemeBuilder::new("SolarizedDark".to_string())
            .background(Color::Rgb(0, 43, 54))
            .surface(Color::Rgb(7, 54, 66))
            .primary(Color::Rgb(38, 139, 210))
            .secondary(Color::Rgb(42, 161, 152))
            .accent(Color::Rgb(211, 54, 130))
            .text_colors(
                Color::Rgb(147, 161, 161),
                Color::Rgb(88, 110, 117),
                Color::Rgb(101, 123, 131),
            )
            .status_colors(
                Color::Rgb(133, 153, 0),
                Color::Rgb(181, 137, 0),
                Color::Rgb(220, 50, 47),
                Color::Rgb(38, 139, 210),
            )
            .author("Ethan Schoonover".to_string())
            .description("Solarized Dark theme".to_string());
        
        presets.insert("solarized_dark".to_string(), solarized_dark);
        
        // Monokai preset
        let monokai = ThemeBuilder::new("Monokai".to_string())
            .background(Color::Rgb(39, 40, 34))
            .surface(Color::Rgb(73, 72, 62))
            .primary(Color::Rgb(102, 217, 239))
            .secondary(Color::Rgb(166, 226, 46))
            .accent(Color::Rgb(249, 38, 114))
            .text_colors(
                Color::Rgb(248, 248, 242),
                Color::Rgb(117, 113, 94),
                Color::Rgb(117, 113, 94),
            )
            .status_colors(
                Color::Rgb(166, 226, 46),
                Color::Rgb(230, 219, 116),
                Color::Rgb(249, 38, 114),
                Color::Rgb(102, 217, 239),
            )
            .author("Wimer Hazenberg".to_string())
            .description("Monokai theme".to_string());
        
        presets.insert("monokai".to_string(), monokai);
        
        // Dracula preset
        let dracula = ThemeBuilder::new("Dracula".to_string())
            .background(Color::Rgb(40, 42, 54))
            .surface(Color::Rgb(68, 71, 90))
            .primary(Color::Rgb(139, 233, 253))
            .secondary(Color::Rgb(80, 250, 123))
            .accent(Color::Rgb(255, 121, 198))
            .text_colors(
                Color::Rgb(248, 248, 242),
                Color::Rgb(98, 114, 164),
                Color::Rgb(98, 114, 164),
            )
            .status_colors(
                Color::Rgb(80, 250, 123),
                Color::Rgb(241, 250, 140),
                Color::Rgb(255, 85, 85),
                Color::Rgb(139, 233, 253),
            )
            .author("Zeno Rocha".to_string())
            .description("Dracula theme".to_string());
        
        presets.insert("dracula".to_string(), dracula);
        
        Self { presets }
    }
    
    /// Get a preset by name
    pub fn get_preset(&self, name: &str) -> Option<&ThemeBuilder> {
        self.presets.get(name)
    }
    
    /// Get all preset names
    pub fn preset_names(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }
    
    /// Build a theme from a preset
    pub fn build_preset(&self, name: &str) -> Option<Theme> {
        self.presets.get(name).map(|builder| builder.clone().build())
    }
    
    /// Add a custom preset
    pub fn add_preset(&mut self, name: String, builder: ThemeBuilder) {
        self.presets.insert(name, builder);
    }
}

impl Default for ThemePresetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Theme validator for checking theme consistency and accessibility
pub struct ThemeValidator;

impl ThemeValidator {
    /// Validate a theme for accessibility and consistency
    pub fn validate_theme(theme: &Theme) -> Vec<ThemeValidationIssue> {
        let mut issues = Vec::new();
        
        // Check contrast ratios
        issues.extend(Self::check_contrast_ratios(theme));
        
        // Check color consistency
        issues.extend(Self::check_color_consistency(theme));
        
        // Check accessibility
        issues.extend(Self::check_accessibility(theme));
        
        issues
    }
    
    /// Check contrast ratios between text and background colors
    fn check_contrast_ratios(theme: &Theme) -> Vec<ThemeValidationIssue> {
        let mut issues = Vec::new();
        
        // This is a simplified contrast check
        // In a real implementation, you would calculate actual contrast ratios
        
        if Self::colors_too_similar(&theme.colors.text_primary, &theme.colors.background) {
            issues.push(ThemeValidationIssue {
                severity: ValidationSeverity::Error,
                message: "Primary text and background colors have insufficient contrast".to_string(),
                suggestion: "Choose colors with higher contrast ratio".to_string(),
            });
        }
        
        if Self::colors_too_similar(&theme.colors.text_secondary, &theme.colors.surface) {
            issues.push(ThemeValidationIssue {
                severity: ValidationSeverity::Warning,
                message: "Secondary text and surface colors may have low contrast".to_string(),
                suggestion: "Consider adjusting secondary text color".to_string(),
            });
        }
        
        issues
    }
    
    /// Check color consistency across the theme
    fn check_color_consistency(theme: &Theme) -> Vec<ThemeValidationIssue> {
        let mut issues = Vec::new();
        
        // Check if status colors are distinct
        let status_colors = [
            &theme.colors.success,
            &theme.colors.warning,
            &theme.colors.error,
            &theme.colors.info,
        ];
        
        for (i, color1) in status_colors.iter().enumerate() {
            for (j, color2) in status_colors.iter().enumerate() {
                if i != j && Self::colors_too_similar(color1, color2) {
                    issues.push(ThemeValidationIssue {
                        severity: ValidationSeverity::Warning,
                        message: "Status colors are too similar and may be confusing".to_string(),
                        suggestion: "Use more distinct colors for different status types".to_string(),
                    });
                    break;
                }
            }
        }
        
        issues
    }
    
    /// Check accessibility features
    fn check_accessibility(theme: &Theme) -> Vec<ThemeValidationIssue> {
        let mut issues = Vec::new();
        
        // Check if theme is suitable for color-blind users
        if !Self::is_colorblind_friendly(theme) {
            issues.push(ThemeValidationIssue {
                severity: ValidationSeverity::Info,
                message: "Theme may not be suitable for color-blind users".to_string(),
                suggestion: "Consider using patterns or shapes in addition to colors".to_string(),
            });
        }
        
        issues
    }
    
    /// Simple color similarity check (placeholder implementation)
    fn colors_too_similar(color1: &Color, color2: &Color) -> bool {
        // This is a very simplified check
        // In a real implementation, you would convert to a perceptual color space
        // and calculate actual distance
        match (color1, color2) {
            (Color::Black, Color::DarkGray) => true,
            (Color::White, Color::Gray) => true,
            _ => false,
        }
    }
    
    /// Check if theme is colorblind-friendly (placeholder implementation)
    fn is_colorblind_friendly(_theme: &Theme) -> bool {
        // This would involve more sophisticated color analysis
        // For now, return true as a placeholder
        true
    }
}

/// Theme validation issue
#[derive(Debug, Clone)]
pub struct ThemeValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
    pub suggestion: String,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}