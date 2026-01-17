//! TUI-specific configuration management
//!
//! This module provides configuration management specifically for the TUI interface,
//! including user preferences, layout settings, and hot reload capabilities.

use crate::config::{Config, ConfigManager};
use crate::error::{Result, WorkflowError};
use crate::interfaces::tui::{
    theme::ThemeConfig, FocusManager, LayoutManager, NavigationConfig,
    PlatformManager, ThemeManager,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, RwLock};
use tracing::{debug, error, info, warn};

/// TUI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    pub interface: InterfaceConfig,
    pub themes: ThemeManagerConfig,
    pub layout: LayoutConfig,
    pub navigation: NavigationConfig,
    pub performance: PerformanceConfig,
    pub accessibility: AccessibilityConfig,
    pub keybindings: KeybindingsConfig,
    pub user_preferences: UserPreferences,
}

/// Interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    pub default_view: String,
    pub show_help_on_startup: bool,
    pub confirm_quit: bool,
    pub auto_refresh_interval: Option<u64>, // seconds
    pub status_bar_format: String,
    pub header_format: String,
    pub enable_mouse: bool,
    pub enable_unicode: bool,
    pub frame_rate: u16, // FPS
}

/// Theme manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManagerConfig {
    pub default_theme: String,
    pub auto_detect_theme: bool,
    pub custom_themes: Vec<ThemeConfig>,
    pub theme_switching_enabled: bool,
    pub follow_system_theme: bool,
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub default_layout: String,
    pub custom_layouts: HashMap<String, LayoutDefinition>,
    pub responsive_breakpoints: ResponsiveBreakpoints,
    pub widget_spacing: u16,
    pub border_style: String,
    pub enable_animations: bool,
}

/// Layout definition for custom layouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDefinition {
    pub name: String,
    pub description: String,
    pub widgets: Vec<WidgetLayout>,
    pub constraints: Vec<LayoutConstraint>,
}

/// Widget layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetLayout {
    pub widget_type: String,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub visible: bool,
    pub priority: u8,
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u16,
    pub y: u16,
    pub anchor: String, // "top-left", "center", etc.
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: SizeConstraint,
    pub height: SizeConstraint,
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
}

/// Size constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizeConstraint {
    Fixed(u16),
    Percentage(u8),
    Flexible(u16), // weight
    Auto,
}

/// Layout constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConstraint {
    pub constraint_type: String,
    pub value: u16,
}

/// Responsive breakpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveBreakpoints {
    pub small: u16,  // < 80 columns
    pub medium: u16, // 80-120 columns
    pub large: u16,  // > 120 columns
    pub enable_compact_mode: bool,
    pub compact_threshold: u16,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub target_fps: u16,
    pub enable_vsync: bool,
    pub buffer_size: usize,
    pub lazy_rendering: bool,
    pub cache_rendered_content: bool,
    pub max_cache_size: usize,
    pub gc_interval: u64, // seconds
}

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    pub high_contrast_mode: bool,
    pub large_text_mode: bool,
    pub screen_reader_support: bool,
    pub keyboard_only_navigation: bool,
    pub focus_indicators: bool,
    pub animation_reduction: bool,
    pub color_blind_support: bool,
}

/// Keybindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub global_bindings: HashMap<String, String>,
    pub view_specific_bindings: HashMap<String, HashMap<String, String>>,
    pub custom_bindings: HashMap<String, String>,
    pub enable_vim_mode: bool,
    pub enable_emacs_mode: bool,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub last_used_view: Option<String>,
    pub window_size: Option<(u16, u16)>,
    pub recent_workflows: Vec<String>,
    pub favorite_tools: Vec<String>,
    pub hidden_widgets: Vec<String>,
    pub custom_shortcuts: HashMap<String, String>,
    pub notification_settings: NotificationSettings,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub enable_notifications: bool,
    pub notification_duration: u64, // seconds
    pub notification_position: String,
    pub sound_enabled: bool,
    pub priority_filter: String,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            interface: InterfaceConfig::default(),
            themes: ThemeManagerConfig::default(),
            layout: LayoutConfig::default(),
            navigation: NavigationConfig::default(),
            performance: PerformanceConfig::default(),
            accessibility: AccessibilityConfig::default(),
            keybindings: KeybindingsConfig::default(),
            user_preferences: UserPreferences::default(),
        }
    }
}

impl Default for InterfaceConfig {
    fn default() -> Self {
        Self {
            default_view: "WorkflowList".to_string(),
            show_help_on_startup: false,
            confirm_quit: true,
            auto_refresh_interval: Some(30),
            status_bar_format: "{view} | {shortcuts} | {status}".to_string(),
            header_format: "工作流工具包 TUI v{version} - {view}".to_string(),
            enable_mouse: true,
            enable_unicode: true,
            frame_rate: 60,
        }
    }
}

impl Default for ThemeManagerConfig {
    fn default() -> Self {
        Self {
            default_theme: "Dark".to_string(),
            auto_detect_theme: true,
            custom_themes: Vec::new(),
            theme_switching_enabled: true,
            follow_system_theme: false,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            default_layout: "standard".to_string(),
            custom_layouts: HashMap::new(),
            responsive_breakpoints: ResponsiveBreakpoints::default(),
            widget_spacing: 1,
            border_style: "rounded".to_string(),
            enable_animations: true,
        }
    }
}

impl Default for ResponsiveBreakpoints {
    fn default() -> Self {
        Self {
            small: 80,
            medium: 120,
            large: 160,
            enable_compact_mode: true,
            compact_threshold: 60,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            enable_vsync: true,
            buffer_size: 8192,
            lazy_rendering: true,
            cache_rendered_content: true,
            max_cache_size: 1024 * 1024, // 1MB
            gc_interval: 300,            // 5 minutes
        }
    }
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            high_contrast_mode: false,
            large_text_mode: false,
            screen_reader_support: false,
            keyboard_only_navigation: false,
            focus_indicators: true,
            animation_reduction: false,
            color_blind_support: false,
        }
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        let mut global_bindings = HashMap::new();
        global_bindings.insert("quit".to_string(), "Ctrl+q".to_string());
        global_bindings.insert("help".to_string(), "?".to_string());
        global_bindings.insert("refresh".to_string(), "F12".to_string());
        global_bindings.insert("navigate_workflow_list".to_string(), "F1".to_string());
        global_bindings.insert("navigate_execution_monitor".to_string(), "F2".to_string());
        global_bindings.insert("navigate_tool_manager".to_string(), "F3".to_string());
        global_bindings.insert("navigate_plugin_manager".to_string(), "F4".to_string());
        global_bindings.insert("navigate_system_status".to_string(), "F5".to_string());
        global_bindings.insert("navigate_log_viewer".to_string(), "F6".to_string());

        Self {
            global_bindings,
            view_specific_bindings: HashMap::new(),
            custom_bindings: HashMap::new(),
            enable_vim_mode: false,
            enable_emacs_mode: false,
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            last_used_view: None,
            window_size: None,
            recent_workflows: Vec::new(),
            favorite_tools: Vec::new(),
            hidden_widgets: Vec::new(),
            custom_shortcuts: HashMap::new(),
            notification_settings: NotificationSettings::default(),
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            notification_duration: 5,
            notification_position: "top-right".to_string(),
            sound_enabled: false,
            priority_filter: "info".to_string(),
        }
    }
}

/// TUI configuration manager with hot reload support
pub struct TuiConfigManager {
    config: Arc<RwLock<TuiConfig>>,
    base_config_manager: Arc<ConfigManager>,
    config_path: PathBuf,
    watch_sender: watch::Sender<TuiConfig>,
    watch_receiver: watch::Receiver<TuiConfig>,
    theme_manager: Arc<RwLock<ThemeManager>>,
    platform_manager: Arc<RwLock<PlatformManager>>,
}

impl TuiConfigManager {
    /// Create a new TUI configuration manager
    pub async fn new(base_config_manager: Arc<ConfigManager>, config_dir: PathBuf) -> Result<Self> {
        let config_path = config_dir.join("tui.toml");

        // Load TUI configuration
        let tui_config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            let default_config = TuiConfig::default();
            Self::save_to_file(&config_path, &default_config)?;
            default_config
        };

        let (watch_sender, watch_receiver) = watch::channel(tui_config.clone());

        // Initialize theme manager with custom themes
        let mut theme_manager = ThemeManager::new();
        if let Err(e) = theme_manager.load_themes(tui_config.themes.custom_themes.clone()) {
            warn!("Failed to load custom themes: {}", e);
        }

        // Set default theme
        if let Err(e) = theme_manager.set_current_theme(&tui_config.themes.default_theme) {
            warn!("Failed to set default theme: {}, using fallback", e);
        }

        // Initialize platform manager
        let platform_manager = PlatformManager::new()?;

        Ok(Self {
            config: Arc::new(RwLock::new(tui_config)),
            base_config_manager,
            config_path,
            watch_sender,
            watch_receiver,
            theme_manager: Arc::new(RwLock::new(theme_manager)),
            platform_manager: Arc::new(RwLock::new(platform_manager)),
        })
    }

    /// Get current TUI configuration
    pub async fn get_config(&self) -> TuiConfig {
        self.config.read().await.clone()
    }

    /// Get configuration watch receiver for hot reload notifications
    pub fn get_watch_receiver(&self) -> watch::Receiver<TuiConfig> {
        self.watch_receiver.clone()
    }

    /// Get theme manager
    pub fn theme_manager(&self) -> Arc<RwLock<ThemeManager>> {
        Arc::clone(&self.theme_manager)
    }

    /// Get platform manager
    pub fn platform_manager(&self) -> Arc<RwLock<PlatformManager>> {
        Arc::clone(&self.platform_manager)
    }

    /// Update TUI configuration
    pub async fn update_config(&self, new_config: TuiConfig) -> Result<()> {
        // Validate configuration
        self.validate_config(&new_config)?;

        // Update configuration
        {
            let mut config = self.config.write().await;
            *config = new_config.clone();
        }

        // Save to file
        Self::save_to_file(&self.config_path, &new_config)?;

        // Update theme manager if themes changed
        {
            let mut theme_manager = self.theme_manager.write().await;
            if let Err(e) = theme_manager.load_themes(new_config.themes.custom_themes.clone()) {
                warn!("Failed to reload custom themes: {}", e);
            }

            if let Err(e) = theme_manager.set_current_theme(&new_config.themes.default_theme) {
                warn!("Failed to update default theme: {}", e);
            }
        }

        // Notify watchers
        if let Err(e) = self.watch_sender.send(new_config) {
            warn!("Failed to notify configuration watchers: {}", e);
        }

        info!("TUI configuration updated");
        Ok(())
    }

    /// Update user preferences
    pub async fn update_user_preferences(&self, preferences: UserPreferences) -> Result<()> {
        let mut config = self.get_config().await;
        config.user_preferences = preferences;
        self.update_config(config).await
    }

    /// Update theme configuration
    pub async fn update_theme_config(&self, theme_config: ThemeManagerConfig) -> Result<()> {
        let mut config = self.get_config().await;
        config.themes = theme_config;
        self.update_config(config).await
    }

    /// Update layout configuration
    pub async fn update_layout_config(&self, layout_config: LayoutConfig) -> Result<()> {
        let mut config = self.get_config().await;
        config.layout = layout_config;
        self.update_config(config).await
    }

    /// Update keybindings configuration
    pub async fn update_keybindings(&self, keybindings: KeybindingsConfig) -> Result<()> {
        let mut config = self.get_config().await;
        config.keybindings = keybindings;
        self.update_config(config).await
    }

    /// Start hot reload monitoring
    pub async fn start_hot_reload(&self) -> Result<()> {
        let config_path = self.config_path.clone();
        let config_manager = self.clone_for_hot_reload();

        tokio::spawn(async move {
            config_manager.hot_reload_loop(config_path).await;
        });

        info!("Started TUI configuration hot reload monitoring");
        Ok(())
    }

    /// Clone for hot reload (only necessary parts)
    fn clone_for_hot_reload(&self) -> TuiConfigManagerForHotReload {
        TuiConfigManagerForHotReload {
            config: Arc::clone(&self.config),
            watch_sender: self.watch_sender.clone(),
            theme_manager: Arc::clone(&self.theme_manager),
        }
    }

    /// Load configuration from file
    fn load_from_file(path: &Path) -> Result<TuiConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: TuiConfig = toml::from_str(&content).map_err(|e| {
            WorkflowError::Config(config::ConfigError::Message(format!(
                "Failed to parse TUI config: {}",
                e
            )))
        })?;
        Ok(config)
    }

    /// Save configuration to file
    fn save_to_file(path: &Path, config: &TuiConfig) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content =
            toml::to_string_pretty(config).map_err(|e| WorkflowError::Generic(e.into()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration
    fn validate_config(&self, config: &TuiConfig) -> Result<()> {
        // Validate frame rate
        if config.performance.target_fps == 0 || config.performance.target_fps > 120 {
            return Err(WorkflowError::ValidationError(
                "Frame rate must be between 1 and 120 FPS".to_string(),
            ));
        }

        // Validate buffer size
        if config.performance.buffer_size < 1024 {
            return Err(WorkflowError::ValidationError(
                "Buffer size must be at least 1024 bytes".to_string(),
            ));
        }

        // Validate responsive breakpoints
        let breakpoints = &config.layout.responsive_breakpoints;
        if breakpoints.small >= breakpoints.medium || breakpoints.medium >= breakpoints.large {
            return Err(WorkflowError::ValidationError(
                "Responsive breakpoints must be in ascending order".to_string(),
            ));
        }

        // Validate notification duration
        if config
            .user_preferences
            .notification_settings
            .notification_duration
            == 0
        {
            return Err(WorkflowError::ValidationError(
                "Notification duration must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Apply configuration to components
    pub async fn apply_to_components(
        &self,
        _layout_manager: &mut LayoutManager,
        _focus_manager: &mut FocusManager,
    ) -> Result<()> {
        let _config = self.get_config().await;

        // Apply layout configuration
        // This would involve updating the layout manager with new settings
        debug!("Applying layout configuration");

        // Apply navigation configuration
        // This would involve updating the focus manager with new settings
        debug!("Applying navigation configuration");

        // Apply performance configuration
        // This would involve updating performance settings
        debug!("Applying performance configuration");

        info!("Applied TUI configuration to components");
        Ok(())
    }

    /// Get merged configuration with base configuration
    pub async fn get_merged_config(&self) -> (Config, TuiConfig) {
        let base_config = self.base_config_manager.get_config();
        let tui_config = self.get_config().await;
        (base_config, tui_config)
    }

    /// Export configuration to file
    pub async fn export_config(&self, path: &Path) -> Result<()> {
        let config = self.get_config().await;
        Self::save_to_file(path, &config)?;
        info!("Exported TUI configuration to: {:?}", path);
        Ok(())
    }

    /// Import configuration from file
    pub async fn import_config(&self, path: &Path) -> Result<()> {
        let config = Self::load_from_file(path)?;
        self.update_config(config).await?;
        info!("Imported TUI configuration from: {:?}", path);
        Ok(())
    }

    /// Reset configuration to defaults
    pub async fn reset_to_defaults(&self) -> Result<()> {
        let default_config = TuiConfig::default();
        self.update_config(default_config).await?;
        info!("Reset TUI configuration to defaults");
        Ok(())
    }

    /// Get configuration schema for validation
    pub fn get_config_schema() -> serde_json::Value {
        // This would return a JSON schema for the TUI configuration
        // For now, return a placeholder
        serde_json::json!({
            "type": "object",
            "properties": {
                "interface": {
                    "type": "object",
                    "properties": {
                        "frame_rate": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 120
                        }
                    }
                }
            }
        })
    }
}

/// Helper struct for hot reload functionality
struct TuiConfigManagerForHotReload {
    config: Arc<RwLock<TuiConfig>>,
    watch_sender: watch::Sender<TuiConfig>,
    theme_manager: Arc<RwLock<ThemeManager>>,
}

impl TuiConfigManagerForHotReload {
    /// Hot reload monitoring loop
    async fn hot_reload_loop(&self, config_path: PathBuf) {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        let mut last_modified = self.get_file_modified_time(&config_path).await;

        loop {
            interval.tick().await;

            if let Some(current_modified) = self.get_file_modified_time(&config_path).await {
                if Some(current_modified) != last_modified {
                    debug!("TUI configuration file changed, reloading...");

                    match self.reload_config_file(&config_path).await {
                        Ok(()) => {
                            last_modified = Some(current_modified);
                            info!("TUI configuration hot reloaded successfully");
                        }
                        Err(e) => {
                            error!("Failed to hot reload TUI configuration: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Get file modification time
    async fn get_file_modified_time(&self, path: &Path) -> Option<std::time::SystemTime> {
        tokio::fs::metadata(path)
            .await
            .ok()
            .and_then(|metadata| metadata.modified().ok())
    }

    /// Reload configuration from file
    async fn reload_config_file(&self, config_path: &Path) -> Result<()> {
        let new_config = TuiConfigManager::load_from_file(config_path)?;

        // Update configuration
        {
            let mut config = self.config.write().await;
            *config = new_config.clone();
        }

        // Update theme manager
        {
            let mut theme_manager = self.theme_manager.write().await;
            if let Err(e) = theme_manager.load_themes(new_config.themes.custom_themes.clone()) {
                warn!("Failed to reload custom themes during hot reload: {}", e);
            }

            if let Err(e) = theme_manager.set_current_theme(&new_config.themes.default_theme) {
                warn!("Failed to update default theme during hot reload: {}", e);
            }
        }

        // Notify watchers
        if let Err(e) = self.watch_sender.send(new_config) {
            warn!(
                "Failed to notify configuration watchers during hot reload: {}",
                e
            );
        }

        Ok(())
    }
}

/// Configuration preset manager
pub struct ConfigPresetManager {
    presets: HashMap<String, TuiConfig>,
}

impl ConfigPresetManager {
    /// Create a new preset manager with default presets
    pub fn new() -> Self {
        let mut presets = HashMap::new();

        // Default preset
        presets.insert("default".to_string(), TuiConfig::default());

        // Performance preset
        let mut performance_config = TuiConfig::default();
        performance_config.performance.target_fps = 30;
        performance_config.performance.lazy_rendering = true;
        performance_config.performance.cache_rendered_content = true;
        performance_config.interface.enable_mouse = false;
        performance_config.layout.enable_animations = false;
        presets.insert("performance".to_string(), performance_config);

        // Accessibility preset
        let mut accessibility_config = TuiConfig::default();
        accessibility_config.accessibility.high_contrast_mode = true;
        accessibility_config.accessibility.large_text_mode = true;
        accessibility_config.accessibility.keyboard_only_navigation = true;
        accessibility_config.accessibility.focus_indicators = true;
        accessibility_config.accessibility.animation_reduction = true;
        accessibility_config.themes.default_theme = "HighContrast".to_string();
        presets.insert("accessibility".to_string(), accessibility_config);

        // Minimal preset
        let mut minimal_config = TuiConfig::default();
        minimal_config.interface.show_help_on_startup = false;
        minimal_config.interface.enable_mouse = false;
        minimal_config.interface.enable_unicode = false;
        minimal_config.layout.enable_animations = false;
        minimal_config.performance.target_fps = 15;
        presets.insert("minimal".to_string(), minimal_config);

        Self { presets }
    }

    /// Get a preset by name
    pub fn get_preset(&self, name: &str) -> Option<&TuiConfig> {
        self.presets.get(name)
    }

    /// Add a custom preset
    pub fn add_preset(&mut self, name: String, config: TuiConfig) {
        self.presets.insert(name, config);
    }

    /// Remove a preset
    pub fn remove_preset(&mut self, name: &str) -> Option<TuiConfig> {
        self.presets.remove(name)
    }

    /// List all preset names
    pub fn list_presets(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }

    /// Save presets to file
    pub fn save_presets(&self, path: &Path) -> Result<()> {
        let content =
            toml::to_string_pretty(&self.presets).map_err(|e| WorkflowError::Generic(e.into()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load presets from file
    pub fn load_presets(&mut self, path: &Path) -> Result<()> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let presets: HashMap<String, TuiConfig> = toml::from_str(&content).map_err(|e| {
                WorkflowError::Config(config::ConfigError::Message(format!(
                    "Failed to parse presets: {}",
                    e
                )))
            })?;
            self.presets.extend(presets);
        }
        Ok(())
    }
}

impl Default for ConfigPresetManager {
    fn default() -> Self {
        Self::new()
    }
}
