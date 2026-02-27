//! Layout management system for TUI widgets
//!
//! This module provides responsive layout calculation and widget size constraint handling.

use crate::error::WorkflowError;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

use super::widget::{SizeConstraints, WidgetId};

/// Layout-specific error type
#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("Terminal too small: minimum {min_width}x{min_height}, current {current_width}x{current_height}")]
    TerminalTooSmall {
        min_width: u16,
        min_height: u16,
        current_width: u16,
        current_height: u16,
    },
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    #[error("Layout calculation error: {message}")]
    CalculationError { message: String },
}

impl From<LayoutError> for WorkflowError {
    fn from(err: LayoutError) -> Self {
        WorkflowError::validation(format!("Layout error: {}", err))
    }
}

/// Layout direction for arranging widgets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

/// Layout constraints for responsive design
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutConstraints {
    /// Fixed size in characters
    Length(u16),
    /// Percentage of available space
    Percentage(u16),
    /// Minimum size in characters
    Min(u16),
    /// Maximum size in characters
    Max(u16),
    /// Fill remaining space equally
    Fill,
    /// Ratio of available space
    Ratio(u32, u32), // (numerator, denominator)
}

/// Rectangle with layout information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRect {
    pub rect: Rect,
    pub constraints: LayoutConstraints,
    pub widget_id: Option<WidgetId>,
    pub is_visible: bool,
    pub z_index: i32,
}

/// Layout node in the layout tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutNode {
    pub id: String,
    pub direction: LayoutDirection,
    pub constraints: Vec<LayoutConstraints>,
    pub children: Vec<LayoutNode>,
    pub widget_id: Option<WidgetId>,
    pub size_constraints: Option<SizeConstraints>,
    pub advanced_constraints: Option<AdvancedSizeConstraints>,
    pub margin: Margin,
    pub padding: Padding,
    pub is_visible: bool,
    pub z_index: i32,
    pub priority: i32,
    pub is_collapsible: bool,
    pub is_collapsed: bool,
    pub animation_state: AnimationState,
}

/// Margin configuration
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

/// Padding configuration
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Padding {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

/// Layout calculation result
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub widget_areas: HashMap<WidgetId, Rect>,
    pub total_area: Rect,
    pub overflow_widgets: Vec<WidgetId>,
    pub hidden_widgets: Vec<WidgetId>,
}

/// Layout manager for handling responsive layouts
pub struct LayoutManager {
    root_node: Option<LayoutNode>,
    terminal_size: (u16, u16),
    min_terminal_size: (u16, u16),
    layout_cache: HashMap<String, LayoutResult>,
    responsive_breakpoints: Vec<ResponsiveBreakpoint>,
    layout_config: LayoutConfig,
    dynamic_adjustments: Vec<DynamicAdjustment>,
    performance_metrics: LayoutPerformanceMetrics,
    compact_mode: bool,
    fullscreen_widget: Option<WidgetId>,
    layout_priority_manager: LayoutPriorityManager,
    config_manager: Option<LayoutConfigManager>,
    focus_manager: FocusManager,
}

/// Responsive breakpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveBreakpoint {
    pub name: String,
    pub min_width: u16,
    pub min_height: u16,
    pub layout_adjustments: Vec<LayoutAdjustment>,
}

/// Layout adjustment for responsive design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutAdjustment {
    /// Hide specific widgets
    HideWidgets(Vec<WidgetId>),
    /// Change layout direction
    ChangeDirection(String, LayoutDirection), // (node_id, new_direction)
    /// Modify constraints
    ModifyConstraints(String, Vec<LayoutConstraints>), // (node_id, new_constraints)
    /// Switch to compact mode
    CompactMode(bool),
    /// Adjust margins and padding
    AdjustSpacing(String, Margin, Padding), // (node_id, margin, padding)
    /// Change widget priority
    SetPriority(WidgetId, i32),
    /// Enable/disable scrolling
    SetScrollable(WidgetId, bool),
}

/// Advanced layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Enable responsive layout adjustments
    pub responsive_enabled: bool,
    /// Cache layout calculations
    pub cache_enabled: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Layout calculation timeout in milliseconds
    pub calculation_timeout_ms: u64,
    /// Enable performance monitoring
    pub performance_monitoring: bool,
    /// Minimum widget size before hiding
    pub min_widget_size: (u16, u16),
    /// Default spacing between widgets
    pub default_spacing: u16,
    /// Enable smooth transitions
    pub smooth_transitions: bool,
    /// Transition duration in milliseconds
    pub transition_duration_ms: u64,
    /// Compact mode threshold (width in characters)
    pub compact_mode_threshold: u16,
    /// Enable automatic compact mode switching
    pub auto_compact_mode: bool,
    /// Fullscreen mode enabled
    pub fullscreen_enabled: bool,
    /// Compact mode spacing reduction factor
    pub compact_spacing_factor: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            responsive_enabled: true,
            cache_enabled: true,
            max_cache_size: 100,
            calculation_timeout_ms: 100,
            performance_monitoring: true,
            min_widget_size: (10, 3),
            default_spacing: 1,
            smooth_transitions: false,
            transition_duration_ms: 200,
            compact_mode_threshold: 80,
            auto_compact_mode: true,
            fullscreen_enabled: true,
            compact_spacing_factor: 0.5,
        }
    }
}

/// Dynamic layout adjustment based on runtime conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAdjustment {
    pub id: String,
    pub condition: AdjustmentCondition,
    pub adjustment: LayoutAdjustment,
    pub priority: i32,
    pub enabled: bool,
}

/// Condition for dynamic layout adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentCondition {
    /// Terminal size condition
    TerminalSize {
        min_width: u16,
        max_width: u16,
        min_height: u16,
        max_height: u16,
    },
    /// Widget count condition
    WidgetCount { min_count: usize, max_count: usize },
    /// Performance condition (CPU usage, memory, etc.)
    Performance {
        max_cpu_percent: f64,
        max_memory_mb: u64,
    },
    /// Time-based condition
    TimeOfDay { start_hour: u8, end_hour: u8 },
    /// Custom condition (evaluated by callback)
    Custom(String), // Condition name for lookup
}

/// Layout performance metrics
#[derive(Debug, Clone, Default)]
pub struct LayoutPerformanceMetrics {
    pub calculation_time_ms: f64,
    pub cache_hit_rate: f64,
    pub total_calculations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub last_calculation_time: Option<std::time::Instant>,
    pub average_calculation_time_ms: f64,
}

/// Advanced size constraints with more options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedSizeConstraints {
    /// Basic size constraints
    pub basic: SizeConstraints,
    /// Aspect ratio constraints (width:height)
    pub aspect_ratio: Option<(u16, u16)>,
    /// Maintain aspect ratio when resizing
    pub maintain_aspect_ratio: bool,
    /// Flexible sizing options
    pub flexible: bool,
    /// Priority for space allocation (higher = more space)
    pub priority: i32,
    /// Can be hidden when space is limited
    pub hideable: bool,
    /// Can be collapsed to minimum size
    pub collapsible: bool,
    /// Minimum size when collapsed
    pub collapsed_size: Option<(u16, u16)>,
}

impl Default for AdvancedSizeConstraints {
    fn default() -> Self {
        Self {
            basic: SizeConstraints::default(),
            aspect_ratio: None,
            maintain_aspect_ratio: false,
            flexible: true,
            priority: 0,
            hideable: false,
            collapsible: false,
            collapsed_size: None,
        }
    }
}

impl AdvancedSizeConstraints {
    /// Create new advanced constraints from basic constraints
    pub fn from_basic(basic: SizeConstraints) -> Self {
        Self {
            basic,
            ..Default::default()
        }
    }

    /// Set aspect ratio
    pub fn with_aspect_ratio(mut self, width: u16, height: u16) -> Self {
        self.aspect_ratio = Some((width, height));
        self.maintain_aspect_ratio = true;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Make hideable
    pub fn hideable(mut self) -> Self {
        self.hideable = true;
        self
    }

    /// Make collapsible
    pub fn collapsible(mut self, collapsed_size: (u16, u16)) -> Self {
        self.collapsible = true;
        self.collapsed_size = Some(collapsed_size);
        self
    }

    /// Calculate size respecting aspect ratio
    pub fn calculate_size_with_aspect_ratio(
        &self,
        available_width: u16,
        available_height: u16,
    ) -> (u16, u16) {
        if !self.maintain_aspect_ratio || self.aspect_ratio.is_none() {
            return self.basic.clamp(available_width, available_height);
        }

        let (aspect_w, aspect_h) = self.aspect_ratio.unwrap();
        let aspect_ratio = aspect_w as f64 / aspect_h as f64;

        // Try to fit within available space while maintaining aspect ratio
        let width_by_height = (available_height as f64 * aspect_ratio) as u16;
        let height_by_width = (available_width as f64 / aspect_ratio) as u16;

        let (final_width, final_height) = if width_by_height <= available_width {
            (width_by_height, available_height)
        } else {
            (available_width, height_by_width)
        };

        self.basic.clamp(final_width, final_height)
    }
}

/// Animation state for smooth transitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AnimationState {
    /// No animation
    #[default]
    None,
    /// Expanding from collapsed state
    Expanding {
        progress: f64,
        target_size: (u16, u16),
    },
    /// Collapsing to minimum size
    Collapsing {
        progress: f64,
        target_size: (u16, u16),
    },
    /// Fading in
    FadingIn { progress: f64 },
    /// Fading out
    FadingOut { progress: f64 },
}

/// Layout priority manager for handling widget priorities and space allocation
#[derive(Debug, Clone)]
pub struct LayoutPriorityManager {
    /// Widget priority mappings
    widget_priorities: HashMap<WidgetId, i32>,
    /// Priority groups for batch operations
    priority_groups: HashMap<String, Vec<WidgetId>>,
    /// Default priority for new widgets
    default_priority: i32,
    /// Enable priority-based layout
    enabled: bool,
}

impl Default for LayoutPriorityManager {
    fn default() -> Self {
        Self {
            widget_priorities: HashMap::new(),
            priority_groups: HashMap::new(),
            default_priority: 0,
            enabled: true,
        }
    }
}

impl LayoutPriorityManager {
    /// Create a new priority manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set widget priority
    pub fn set_widget_priority(&mut self, widget_id: WidgetId, priority: i32) {
        self.widget_priorities.insert(widget_id, priority);
    }

    /// Get widget priority
    pub fn get_widget_priority(&self, widget_id: &WidgetId) -> i32 {
        self.widget_priorities
            .get(widget_id)
            .copied()
            .unwrap_or(self.default_priority)
    }

    /// Create a priority group
    pub fn create_priority_group(&mut self, group_name: String, widget_ids: Vec<WidgetId>) {
        self.priority_groups.insert(group_name, widget_ids);
    }

    /// Set priority for all widgets in a group
    pub fn set_group_priority(&mut self, group_name: &str, priority: i32) {
        if let Some(widget_ids) = self.priority_groups.get(group_name).cloned() {
            for widget_id in widget_ids {
                self.set_widget_priority(widget_id, priority);
            }
        }
    }

    /// Get sorted widgets by priority (highest first)
    pub fn get_sorted_widgets(&self, widget_ids: &[WidgetId]) -> Vec<(WidgetId, i32)> {
        let mut widgets: Vec<(WidgetId, i32)> = widget_ids
            .iter()
            .map(|id| (id.clone(), self.get_widget_priority(id)))
            .collect();

        widgets.sort_by(|a, b| b.1.cmp(&a.1)); // Descending order
        widgets
    }

    /// Enable or disable priority-based layout
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if priority-based layout is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Compact layout mode configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactMode {
    /// Normal layout mode
    Normal,
    /// Compact mode for small screens
    Compact,
    /// Ultra-compact mode for very small screens
    UltraCompact,
    /// Auto mode - switches based on terminal size
    Auto,
}

/// Fullscreen mode configuration
#[derive(Debug, Clone)]
pub struct FullscreenMode {
    /// Currently fullscreen widget
    pub active_widget: Option<WidgetId>,
    /// Previous layout state before fullscreen
    pub previous_layout: Option<LayoutNode>,
    /// Fullscreen transition animation
    pub transition_state: FullscreenTransition,
    /// Allow fullscreen mode
    pub enabled: bool,
}

impl Default for FullscreenMode {
    fn default() -> Self {
        Self {
            active_widget: None,
            previous_layout: None,
            transition_state: FullscreenTransition::None,
            enabled: true,
        }
    }
}

/// Fullscreen transition states
#[derive(Debug, Clone, PartialEq)]
pub enum FullscreenTransition {
    /// No transition
    None,
    /// Entering fullscreen mode
    Entering { progress: f64 },
    /// Exiting fullscreen mode
    Exiting { progress: f64 },
}

/// Layout configuration template for saving and loading layouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Root layout node
    pub root_node: LayoutNode,
    /// Layout configuration
    pub layout_config: LayoutConfig,
    /// Responsive breakpoints
    pub breakpoints: Vec<ResponsiveBreakpoint>,
    /// Dynamic adjustments
    pub dynamic_adjustments: Vec<DynamicAdjustment>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Template version
    pub version: String,
    /// Template tags for categorization
    pub tags: Vec<String>,
}

impl LayoutTemplate {
    /// Create a new layout template
    pub fn new(name: String, description: String, root_node: LayoutNode) -> Self {
        let now = chrono::Utc::now();
        Self {
            name,
            description,
            root_node,
            layout_config: LayoutConfig::default(),
            breakpoints: Vec::new(),
            dynamic_adjustments: Vec::new(),
            created_at: now,
            modified_at: now,
            version: "1.0.0".to_string(),
            tags: Vec::new(),
        }
    }

    /// Update the template with new data
    pub fn update(&mut self, root_node: LayoutNode, layout_config: LayoutConfig) {
        self.root_node = root_node;
        self.layout_config = layout_config;
        self.modified_at = chrono::Utc::now();
    }

    /// Add a tag to the template
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.modified_at = chrono::Utc::now();
        }
    }

    /// Remove a tag from the template
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let initial_len = self.tags.len();
        self.tags.retain(|t| t != tag);
        let removed = self.tags.len() < initial_len;

        if removed {
            self.modified_at = chrono::Utc::now();
        }

        removed
    }
}

/// Layout configuration manager for saving and loading layouts
#[derive(Debug, Clone)]
pub struct LayoutConfigManager {
    /// Configuration directory path
    config_dir: PathBuf,
    /// Available layout templates
    templates: HashMap<String, LayoutTemplate>,
    /// Current active template name
    active_template: Option<String>,
    /// Auto-save enabled
    auto_save: bool,
    /// Configuration file format
    format: ConfigFormat,
}

/// Configuration file format
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConfigFormat {
    /// JSON format
    #[default]
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
}

impl ConfigFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::Json => "json",
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Toml => "toml",
        }
    }

    /// Serialize template to string
    pub fn serialize<T: Serialize>(&self, data: &T) -> std::result::Result<String, LayoutError> {
        match self {
            ConfigFormat::Json => {
                serde_json::to_string_pretty(data).map_err(|e| LayoutError::ConfigError {
                    message: format!("JSON serialization error: {}", e),
                })
            }
            ConfigFormat::Yaml => {
                serde_yaml::to_string(data).map_err(|e| LayoutError::ConfigError {
                    message: format!("YAML serialization error: {}", e),
                })
            }
            ConfigFormat::Toml => {
                toml::to_string_pretty(data).map_err(|e| LayoutError::ConfigError {
                    message: format!("TOML serialization error: {}", e),
                })
            }
        }
    }

    /// Deserialize template from string
    pub fn deserialize<T: for<'de> Deserialize<'de>>(
        &self,
        content: &str,
    ) -> std::result::Result<T, LayoutError> {
        match self {
            ConfigFormat::Json => {
                serde_json::from_str(content).map_err(|e| LayoutError::ConfigError {
                    message: format!("JSON deserialization error: {}", e),
                })
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(content).map_err(|e| LayoutError::ConfigError {
                    message: format!("YAML deserialization error: {}", e),
                })
            }
            ConfigFormat::Toml => toml::from_str(content).map_err(|e| LayoutError::ConfigError {
                message: format!("TOML deserialization error: {}", e),
            }),
        }
    }
}

impl Default for LayoutConfigManager {
    fn default() -> Self {
        Self::new(PathBuf::from(".kiro/layouts"))
    }
}

impl LayoutConfigManager {
    /// Create a new layout configuration manager
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            templates: HashMap::new(),
            active_template: None,
            auto_save: true,
            format: ConfigFormat::default(),
        }
    }

    /// Set configuration format
    pub fn set_format(&mut self, format: ConfigFormat) {
        self.format = format;
    }

    /// Get configuration format
    pub fn format(&self) -> &ConfigFormat {
        &self.format
    }

    /// Enable or disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }

    /// Check if auto-save is enabled
    pub fn is_auto_save(&self) -> bool {
        self.auto_save
    }

    /// Initialize the configuration manager
    pub async fn initialize(&mut self) -> std::result::Result<(), LayoutError> {
        // Create config directory if it doesn't exist
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .await
                .map_err(|e| LayoutError::ConfigError {
                    message: format!("Failed to create config directory: {}", e),
                })?;
        }

        // Load existing templates
        self.load_all_templates().await?;

        tracing::debug!(
            "Layout configuration manager initialized with {} templates",
            self.templates.len()
        );
        Ok(())
    }

    /// Load all templates from the configuration directory
    pub async fn load_all_templates(&mut self) -> std::result::Result<(), LayoutError> {
        let mut entries =
            fs::read_dir(&self.config_dir)
                .await
                .map_err(|e| LayoutError::ConfigError {
                    message: format!("Failed to read config directory: {}", e),
                })?;

        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| LayoutError::ConfigError {
                    message: format!("Failed to read directory entry: {}", e),
                })?
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == self.format.extension() {
                        if let Err(e) = self.load_template_from_file(&path).await {
                            tracing::warn!("Failed to load template from {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a template from a file
    async fn load_template_from_file(
        &mut self,
        path: &Path,
    ) -> std::result::Result<(), LayoutError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| LayoutError::ConfigError {
                message: format!("Failed to read template file: {}", e),
            })?;

        let template: LayoutTemplate = self.format.deserialize(&content)?;
        self.templates.insert(template.name.clone(), template);

        Ok(())
    }

    /// Save a template to a file
    async fn save_template_to_file(
        &self,
        template: &LayoutTemplate,
    ) -> std::result::Result<(), LayoutError> {
        let filename = format!("{}.{}", template.name, self.format.extension());
        let path = self.config_dir.join(filename);

        let content = self.format.serialize(template)?;

        fs::write(&path, content)
            .await
            .map_err(|e| LayoutError::ConfigError {
                message: format!("Failed to write template file: {}", e),
            })?;

        tracing::debug!("Saved template '{}' to {:?}", template.name, path);
        Ok(())
    }

    /// Save a layout template
    pub async fn save_template(
        &mut self,
        template: LayoutTemplate,
    ) -> std::result::Result<(), LayoutError> {
        let name = template.name.clone();

        // Save to file if auto-save is enabled
        if self.auto_save {
            self.save_template_to_file(&template).await?;
        }

        // Store in memory
        self.templates.insert(name, template);

        Ok(())
    }

    /// Load a layout template by name
    pub fn get_template(&self, name: &str) -> Option<&LayoutTemplate> {
        self.templates.get(name)
    }

    /// Get all template names
    pub fn get_template_names(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Get templates by tag
    pub fn get_templates_by_tag(&self, tag: &str) -> Vec<&LayoutTemplate> {
        self.templates
            .values()
            .filter(|template| template.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Delete a template
    pub async fn delete_template(&mut self, name: &str) -> std::result::Result<bool, LayoutError> {
        let removed = self.templates.remove(name).is_some();

        if removed && self.auto_save {
            // Delete file
            let filename = format!("{}.{}", name, self.format.extension());
            let path = self.config_dir.join(filename);

            if path.exists() {
                fs::remove_file(&path)
                    .await
                    .map_err(|e| LayoutError::ConfigError {
                        message: format!("Failed to delete template file: {}", e),
                    })?;
            }

            // Clear active template if it was deleted
            if self.active_template.as_ref() == Some(&name.to_string()) {
                self.active_template = None;
            }
        }

        Ok(removed)
    }

    /// Set the active template
    pub fn set_active_template(
        &mut self,
        name: Option<String>,
    ) -> std::result::Result<(), LayoutError> {
        if let Some(ref template_name) = name {
            if !self.templates.contains_key(template_name) {
                return Err(LayoutError::ConfigError {
                    message: format!("Template '{}' not found", template_name),
                });
            }
        }

        self.active_template = name;
        Ok(())
    }

    /// Get the active template
    pub fn get_active_template(&self) -> Option<&LayoutTemplate> {
        self.active_template
            .as_ref()
            .and_then(|name| self.templates.get(name))
    }

    /// Get the active template name
    pub fn get_active_template_name(&self) -> Option<&String> {
        self.active_template.as_ref()
    }

    /// Create a template from current layout manager state
    pub fn create_template_from_manager(
        &self,
        name: String,
        description: String,
        layout_manager: &LayoutManager,
    ) -> std::result::Result<LayoutTemplate, LayoutError> {
        let root_node = layout_manager
            .root_node()
            .ok_or_else(|| LayoutError::ConfigError {
                message: "No root layout node available".to_string(),
            })?
            .clone();

        let mut template = LayoutTemplate::new(name, description, root_node);
        template.layout_config = layout_manager.config().clone();
        template.breakpoints = layout_manager.responsive_breakpoints.clone();
        template.dynamic_adjustments = layout_manager.dynamic_adjustments.clone();

        Ok(template)
    }

    /// Apply a template to a layout manager
    pub fn apply_template_to_manager(
        &self,
        template_name: &str,
        layout_manager: &mut LayoutManager,
    ) -> std::result::Result<(), LayoutError> {
        let template =
            self.get_template(template_name)
                .ok_or_else(|| LayoutError::ConfigError {
                    message: format!("Template '{}' not found", template_name),
                })?;

        // Apply template to layout manager
        layout_manager.set_root_node(template.root_node.clone())?;
        layout_manager.set_config(template.layout_config.clone());

        // Clear and set breakpoints
        layout_manager.responsive_breakpoints.clear();
        for breakpoint in &template.breakpoints {
            layout_manager
                .responsive_breakpoints
                .push(breakpoint.clone());
        }

        // Clear and set dynamic adjustments
        layout_manager.dynamic_adjustments.clear();
        for adjustment in &template.dynamic_adjustments {
            layout_manager.dynamic_adjustments.push(adjustment.clone());
        }

        layout_manager.clear_cache();

        tracing::info!(
            "Applied layout template '{}' to layout manager",
            template_name
        );
        Ok(())
    }

    /// Export all templates to a single file
    pub async fn export_all_templates(&self, path: &Path) -> std::result::Result<(), LayoutError> {
        let export_data = LayoutTemplateExport {
            templates: self.templates.values().cloned().collect(),
            exported_at: chrono::Utc::now(),
            format_version: "1.0.0".to_string(),
        };

        let content = self.format.serialize(&export_data)?;

        fs::write(path, content)
            .await
            .map_err(|e| LayoutError::ConfigError {
                message: format!("Failed to export templates: {}", e),
            })?;

        tracing::info!("Exported {} templates to {:?}", self.templates.len(), path);
        Ok(())
    }

    /// Import templates from a file
    pub async fn import_templates(
        &mut self,
        path: &Path,
        overwrite: bool,
    ) -> std::result::Result<usize, LayoutError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| LayoutError::ConfigError {
                message: format!("Failed to read import file: {}", e),
            })?;

        let import_data: LayoutTemplateExport = self.format.deserialize(&content)?;

        let mut imported_count = 0;
        for template in import_data.templates {
            let should_import = overwrite || !self.templates.contains_key(&template.name);

            if should_import {
                self.save_template(template).await?;
                imported_count += 1;
            }
        }

        tracing::info!("Imported {} templates from {:?}", imported_count, path);
        Ok(imported_count)
    }
}

/// Layout template export/import structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LayoutTemplateExport {
    templates: Vec<LayoutTemplate>,
    exported_at: chrono::DateTime<chrono::Utc>,
    format_version: String,
}

/// Focus management system for layout widgets
#[derive(Debug, Clone)]
pub struct FocusManager {
    /// Currently focused widget
    focused_widget: Option<WidgetId>,
    /// Focus history for navigation
    focus_history: Vec<WidgetId>,
    /// Maximum focus history size
    max_history_size: usize,
    /// Focus preservation during layout changes
    preserve_focus: bool,
    /// Focus visual indicators enabled
    visual_indicators: bool,
    /// Focus ring configuration
    focus_ring: FocusRing,
    /// Widget focus states
    widget_focus_states: HashMap<WidgetId, FocusState>,
}

/// Focus ring visual configuration
#[derive(Debug, Clone)]
pub struct FocusRing {
    /// Focus ring enabled
    pub enabled: bool,
    /// Focus ring style
    pub style: FocusRingStyle,
    /// Focus ring color
    pub color: FocusRingColor,
    /// Focus ring thickness
    pub thickness: u16,
    /// Focus ring animation
    pub animation: bool,
}

/// Focus ring visual style
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusRingStyle {
    /// Solid border
    Solid,
    /// Dashed border
    Dashed,
    /// Dotted border
    Dotted,
    /// Double border
    Double,
    /// Rounded corners
    Rounded,
}

/// Focus ring color configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusRingColor {
    /// Default theme color
    Default,
    /// Primary accent color
    Primary,
    /// Secondary accent color
    Secondary,
    /// Custom RGB color
    Custom(u8, u8, u8),
    /// Adaptive color based on content
    Adaptive,
}

/// Widget focus state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusState {
    /// Widget can receive focus
    pub focusable: bool,
    /// Widget is currently focused
    pub focused: bool,
    /// Focus priority (higher = more likely to receive focus)
    pub priority: i32,
    /// Focus group for navigation
    pub group: Option<String>,
    /// Focus position within group
    pub position: Option<usize>,
    /// Last focused timestamp
    pub last_focused: Option<chrono::DateTime<chrono::Utc>>,
    /// Focus preservation across layout changes
    pub preserve_on_layout_change: bool,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self {
            focused_widget: None,
            focus_history: Vec::new(),
            max_history_size: 10,
            preserve_focus: true,
            visual_indicators: true,
            focus_ring: FocusRing::default(),
            widget_focus_states: HashMap::new(),
        }
    }
}

impl Default for FocusRing {
    fn default() -> Self {
        Self {
            enabled: true,
            style: FocusRingStyle::Solid,
            color: FocusRingColor::Primary,
            thickness: 1,
            animation: false,
        }
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            focusable: true,
            focused: false,
            priority: 0,
            group: None,
            position: None,
            last_focused: None,
            preserve_on_layout_change: true,
        }
    }
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Get currently focused widget
    pub fn focused_widget(&self) -> Option<&WidgetId> {
        self.focused_widget.as_ref()
    }

    /// Set focus to a widget
    pub fn set_focus(&mut self, widget_id: WidgetId) -> bool {
        // Check if widget is focusable
        if let Some(state) = self.widget_focus_states.get(&widget_id) {
            if !state.focusable {
                return false;
            }
        }

        // Clear previous focus
        if let Some(ref current_focused) = self.focused_widget {
            if let Some(state) = self.widget_focus_states.get_mut(current_focused) {
                state.focused = false;
            }

            // Add to history if different widget
            if current_focused != &widget_id {
                self.add_to_history(current_focused.clone());
            }
        }

        // Set new focus
        self.focused_widget = Some(widget_id.clone());

        // Update widget state
        let state = self.widget_focus_states.entry(widget_id).or_default();
        state.focused = true;
        state.last_focused = Some(chrono::Utc::now());

        tracing::debug!("Focus set to widget: {:?}", self.focused_widget);
        true
    }

    /// Clear focus
    pub fn clear_focus(&mut self) {
        if let Some(ref focused) = self.focused_widget {
            if let Some(state) = self.widget_focus_states.get_mut(focused) {
                state.focused = false;
            }
            self.add_to_history(focused.clone());
        }

        self.focused_widget = None;
        tracing::debug!("Focus cleared");
    }

    /// Move focus to next focusable widget
    pub fn focus_next(&mut self, available_widgets: &[WidgetId]) -> bool {
        let focusable_widgets = self.get_focusable_widgets(available_widgets);

        if focusable_widgets.is_empty() {
            return false;
        }

        let next_widget = if let Some(ref current) = self.focused_widget {
            // Find current widget position and move to next
            if let Some(current_pos) = focusable_widgets.iter().position(|w| w == current) {
                let next_pos = (current_pos + 1) % focusable_widgets.len();
                focusable_widgets[next_pos].clone()
            } else {
                // Current widget not in list, focus first
                focusable_widgets[0].clone()
            }
        } else {
            // No current focus, focus first
            focusable_widgets[0].clone()
        };

        self.set_focus(next_widget)
    }

    /// Move focus to previous focusable widget
    pub fn focus_previous(&mut self, available_widgets: &[WidgetId]) -> bool {
        let focusable_widgets = self.get_focusable_widgets(available_widgets);

        if focusable_widgets.is_empty() {
            return false;
        }

        let prev_widget = if let Some(ref current) = self.focused_widget {
            // Find current widget position and move to previous
            if let Some(current_pos) = focusable_widgets.iter().position(|w| w == current) {
                let prev_pos = if current_pos == 0 {
                    focusable_widgets.len() - 1
                } else {
                    current_pos - 1
                };
                focusable_widgets[prev_pos].clone()
            } else {
                // Current widget not in list, focus last
                focusable_widgets[focusable_widgets.len() - 1].clone()
            }
        } else {
            // No current focus, focus last
            focusable_widgets[focusable_widgets.len() - 1].clone()
        };

        self.set_focus(prev_widget)
    }

    /// Go back to previous focused widget
    pub fn focus_back(&mut self) -> bool {
        if let Some(previous) = self.focus_history.pop() {
            self.set_focus(previous)
        } else {
            false
        }
    }

    /// Register a widget for focus management
    pub fn register_widget(&mut self, widget_id: WidgetId, state: FocusState) {
        self.widget_focus_states.insert(widget_id, state);
    }

    /// Unregister a widget from focus management
    pub fn unregister_widget(&mut self, widget_id: &WidgetId) {
        self.widget_focus_states.remove(widget_id);

        // Clear focus if this widget was focused
        if self.focused_widget.as_ref() == Some(widget_id) {
            self.focused_widget = None;
        }

        // Remove from history
        self.focus_history.retain(|w| w != widget_id);
    }

    /// Get widget focus state
    pub fn get_widget_state(&self, widget_id: &WidgetId) -> Option<&FocusState> {
        self.widget_focus_states.get(widget_id)
    }

    /// Update widget focus state
    pub fn update_widget_state(&mut self, widget_id: &WidgetId, state: FocusState) {
        self.widget_focus_states.insert(widget_id.clone(), state);
    }

    /// Check if a widget is focused
    pub fn is_focused(&self, widget_id: &WidgetId) -> bool {
        self.focused_widget.as_ref() == Some(widget_id)
    }

    /// Check if a widget is focusable
    pub fn is_focusable(&self, widget_id: &WidgetId) -> bool {
        self.widget_focus_states
            .get(widget_id)
            .is_none_or(|state| state.focusable)
    }

    /// Set widget focusable state
    pub fn set_focusable(&mut self, widget_id: &WidgetId, focusable: bool) {
        let state = self
            .widget_focus_states
            .entry(widget_id.clone())
            .or_default();
        state.focusable = focusable;

        // Clear focus if widget becomes unfocusable
        if !focusable && self.is_focused(widget_id) {
            self.clear_focus();
        }
    }

    /// Handle layout change and preserve focus if needed
    pub fn handle_layout_change(&mut self, available_widgets: &[WidgetId]) {
        if !self.preserve_focus {
            return;
        }

        // Check if currently focused widget is still available
        if let Some(ref focused) = self.focused_widget.clone() {
            if !available_widgets.contains(focused) {
                // Try to restore from history
                while let Some(historical) = self.focus_history.pop() {
                    if available_widgets.contains(&historical) {
                        self.set_focus(historical);
                        return;
                    }
                }

                // No historical widget available, focus first available
                if let Some(first_available) = available_widgets.first() {
                    self.set_focus(first_available.clone());
                } else {
                    self.clear_focus();
                }
            }
        } else if !available_widgets.is_empty() {
            // No current focus, try to restore from history
            while let Some(historical) = self.focus_history.pop() {
                if available_widgets.contains(&historical) {
                    self.set_focus(historical);
                    return;
                }
            }
        }
    }

    /// Get focus ring configuration
    pub fn focus_ring(&self) -> &FocusRing {
        &self.focus_ring
    }

    /// Set focus ring configuration
    pub fn set_focus_ring(&mut self, focus_ring: FocusRing) {
        self.focus_ring = focus_ring;
    }

    /// Check if visual indicators are enabled
    pub fn visual_indicators_enabled(&self) -> bool {
        self.visual_indicators
    }

    /// Enable or disable visual indicators
    pub fn set_visual_indicators(&mut self, enabled: bool) {
        self.visual_indicators = enabled;
    }

    /// Get focus preservation setting
    pub fn preserve_focus_enabled(&self) -> bool {
        self.preserve_focus
    }

    /// Enable or disable focus preservation
    pub fn set_preserve_focus(&mut self, enabled: bool) {
        self.preserve_focus = enabled;
    }

    /// Get focusable widgets sorted by priority
    fn get_focusable_widgets(&self, available_widgets: &[WidgetId]) -> Vec<WidgetId> {
        let mut focusable: Vec<(WidgetId, i32)> = available_widgets
            .iter()
            .filter_map(|widget_id| {
                let state = self.widget_focus_states.get(widget_id)?;
                if state.focusable {
                    Some((widget_id.clone(), state.priority))
                } else {
                    None
                }
            })
            .collect();

        // Sort by priority (highest first), then by position if in same group
        focusable.sort_by(|a, b| {
            b.1.cmp(&a.1) // Priority descending
        });

        focusable
            .into_iter()
            .map(|(widget_id, _)| widget_id)
            .collect()
    }

    /// Add widget to focus history
    fn add_to_history(&mut self, widget_id: WidgetId) {
        // Remove if already in history
        self.focus_history.retain(|w| w != &widget_id);

        // Add to end
        self.focus_history.push(widget_id);

        // Limit history size
        if self.focus_history.len() > self.max_history_size {
            self.focus_history.remove(0);
        }
    }

    /// Get focus statistics
    pub fn get_focus_stats(&self) -> FocusStats {
        FocusStats {
            focused_widget: self.focused_widget.clone(),
            total_widgets: self.widget_focus_states.len(),
            focusable_widgets: self
                .widget_focus_states
                .values()
                .filter(|state| state.focusable)
                .count(),
            history_size: self.focus_history.len(),
            visual_indicators: self.visual_indicators,
            preserve_focus: self.preserve_focus,
        }
    }
}

/// Focus management statistics
#[derive(Debug, Clone)]
pub struct FocusStats {
    pub focused_widget: Option<WidgetId>,
    pub total_widgets: usize,
    pub focusable_widgets: usize,
    pub history_size: usize,
    pub visual_indicators: bool,
    pub preserve_focus: bool,
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new() -> Self {
        Self {
            root_node: None,
            terminal_size: (80, 24),     // Default terminal size
            min_terminal_size: (40, 10), // Minimum usable size
            layout_cache: HashMap::new(),
            responsive_breakpoints: Self::default_breakpoints(),
            layout_config: LayoutConfig::default(),
            dynamic_adjustments: Vec::new(),
            performance_metrics: LayoutPerformanceMetrics::default(),
            compact_mode: false,
            fullscreen_widget: None,
            layout_priority_manager: LayoutPriorityManager::new(),
            config_manager: None,
            focus_manager: FocusManager::new(),
        }
    }

    /// Create layout manager with configuration management
    pub fn with_config_manager(config_dir: PathBuf) -> Self {
        let mut manager = Self::new();
        manager.config_manager = Some(LayoutConfigManager::new(config_dir));
        manager
    }

    /// Initialize the layout manager (including config manager if present)
    pub async fn initialize(&mut self) -> std::result::Result<(), LayoutError> {
        if let Some(ref mut config_manager) = self.config_manager {
            config_manager.initialize().await?;

            // Try to load default template if available
            if let Some(template) = config_manager.get_template("default") {
                let template_clone = template.clone();
                let _ = config_manager; // Release the mutable borrow
                self.apply_template(&template_clone)?;
                tracing::info!("Loaded default layout template");
            }
        }

        Ok(())
    }

    /// Get the configuration manager
    pub fn config_manager(&self) -> Option<&LayoutConfigManager> {
        self.config_manager.as_ref()
    }

    /// Get mutable configuration manager
    pub fn config_manager_mut(&mut self) -> Option<&mut LayoutConfigManager> {
        self.config_manager.as_mut()
    }

    /// Save current layout as a template
    pub async fn save_as_template(
        &mut self,
        name: String,
        description: String,
    ) -> std::result::Result<(), LayoutError> {
        // Extract the template creation logic to avoid borrowing conflicts
        let template = if let Some(ref config_manager) = self.config_manager {
            config_manager.create_template_from_manager(name, description, self)?
        } else {
            return Err(LayoutError::ConfigError {
                message: "Configuration manager not available".to_string(),
            });
        };

        // Now save the template
        if let Some(ref mut config_manager) = self.config_manager {
            config_manager.save_template(template).await?;
            Ok(())
        } else {
            Err(LayoutError::ConfigError {
                message: "Configuration manager not available".to_string(),
            })
        }
    }

    /// Load and apply a template by name
    pub async fn load_template(
        &mut self,
        template_name: &str,
    ) -> std::result::Result<(), LayoutError> {
        // First apply the template
        if let Some(config_manager) = self.config_manager.take() {
            let result = config_manager.apply_template_to_manager(template_name, self);
            self.config_manager = Some(config_manager);
            result?;
        } else {
            return Err(LayoutError::ConfigError {
                message: "Configuration manager not available".to_string(),
            });
        }

        // Then set the active template
        if let Some(ref mut config_manager) = self.config_manager {
            config_manager.set_active_template(Some(template_name.to_string()))?;
            Ok(())
        } else {
            Err(LayoutError::ConfigError {
                message: "Configuration manager not available".to_string(),
            })
        }
    }

    /// Apply a template directly
    fn apply_template(
        &mut self,
        template: &LayoutTemplate,
    ) -> std::result::Result<(), LayoutError> {
        self.set_root_node(template.root_node.clone())?;
        self.set_config(template.layout_config.clone());

        // Apply breakpoints
        self.responsive_breakpoints = template.breakpoints.clone();

        // Apply dynamic adjustments
        self.dynamic_adjustments = template.dynamic_adjustments.clone();

        self.clear_cache();
        Ok(())
    }

    /// Get available template names
    pub fn get_available_templates(&self) -> Vec<String> {
        self.config_manager
            .as_ref()
            .map(|cm| cm.get_template_names())
            .unwrap_or_default()
    }

    /// Get current active template name
    pub fn get_active_template_name(&self) -> Option<String> {
        self.config_manager
            .as_ref()
            .and_then(|cm| cm.get_active_template_name().cloned())
    }

    /// Delete a template
    pub async fn delete_template(&mut self, name: &str) -> std::result::Result<bool, LayoutError> {
        if let Some(ref mut config_manager) = self.config_manager {
            config_manager.delete_template(name).await
        } else {
            Err(LayoutError::ConfigError {
                message: "Configuration manager not available".to_string(),
            })
        }
    }

    /// Export all templates
    pub async fn export_templates(&self, path: &Path) -> std::result::Result<(), LayoutError> {
        if let Some(ref config_manager) = self.config_manager {
            config_manager.export_all_templates(path).await
        } else {
            Err(LayoutError::ConfigError {
                message: "Configuration manager not available".to_string(),
            })
        }
    }

    /// Import templates
    pub async fn import_templates(
        &mut self,
        path: &Path,
        overwrite: bool,
    ) -> std::result::Result<usize, LayoutError> {
        if let Some(ref mut config_manager) = self.config_manager {
            config_manager.import_templates(path, overwrite).await
        } else {
            Err(LayoutError::ConfigError {
                message: "Configuration manager not available".to_string(),
            })
        }
    }

    /// Create a predefined layout template
    pub fn create_predefined_template(
        name: &str,
    ) -> std::result::Result<LayoutTemplate, LayoutError> {
        match name {
            "default" => {
                let root_node = LayoutNode::new("root".to_string(), LayoutDirection::Vertical)
                    .add_child(
                        LayoutNode::widget("header".to_string(), WidgetId("header".to_string())),
                        LayoutConstraints::Length(3),
                    )
                    .add_child(
                        LayoutNode::new("main".to_string(), LayoutDirection::Horizontal)
                            .add_child(
                                LayoutNode::widget(
                                    "sidebar".to_string(),
                                    WidgetId("sidebar".to_string()),
                                ),
                                LayoutConstraints::Percentage(25),
                            )
                            .add_child(
                                LayoutNode::widget(
                                    "content".to_string(),
                                    WidgetId("content".to_string()),
                                ),
                                LayoutConstraints::Fill,
                            ),
                        LayoutConstraints::Fill,
                    )
                    .add_child(
                        LayoutNode::widget("footer".to_string(), WidgetId("footer".to_string())),
                        LayoutConstraints::Length(1),
                    );

                Ok(LayoutTemplate::new(
                    "default".to_string(),
                    "Default three-panel layout with header and footer".to_string(),
                    root_node,
                ))
            }
            "compact" => {
                let root_node = LayoutNode::new("root".to_string(), LayoutDirection::Vertical)
                    .add_child(
                        LayoutNode::widget("main".to_string(), WidgetId("main".to_string())),
                        LayoutConstraints::Fill,
                    )
                    .add_child(
                        LayoutNode::widget("status".to_string(), WidgetId("status".to_string())),
                        LayoutConstraints::Length(1),
                    );

                let mut template = LayoutTemplate::new(
                    "compact".to_string(),
                    "Compact layout for small screens".to_string(),
                    root_node,
                );
                template.add_tag("compact".to_string());
                template.add_tag("mobile".to_string());

                Ok(template)
            }
            "fullwidth" => {
                let root_node =
                    LayoutNode::widget("main".to_string(), WidgetId("main".to_string()));

                let mut template = LayoutTemplate::new(
                    "fullwidth".to_string(),
                    "Full-width single panel layout".to_string(),
                    root_node,
                );
                template.add_tag("simple".to_string());
                template.add_tag("fullscreen".to_string());

                Ok(template)
            }
            _ => Err(LayoutError::ConfigError {
                message: format!("Unknown predefined template: {}", name),
            }),
        }
    }

    /// Install predefined templates
    pub async fn install_predefined_templates(
        &mut self,
    ) -> std::result::Result<usize, LayoutError> {
        if let Some(ref mut config_manager) = self.config_manager {
            let templates = vec!["default", "compact", "fullwidth"];
            let mut installed = 0;

            for template_name in templates {
                let template = Self::create_predefined_template(template_name)?;
                config_manager.save_template(template).await?;
                installed += 1;
            }

            // Set default as active if no active template
            if config_manager.get_active_template_name().is_none() {
                config_manager.set_active_template(Some("default".to_string()))?;
            }

            tracing::info!("Installed {} predefined layout templates", installed);
            Ok(installed)
        } else {
            Err(LayoutError::ConfigError {
                message: "Configuration manager not available".to_string(),
            })
        }
    }

    /// Create layout manager with custom configuration
    pub fn with_config(config: LayoutConfig) -> Self {
        let mut manager = Self::new();
        manager.layout_config = config;
        manager
    }

    /// Get layout configuration
    pub fn config(&self) -> &LayoutConfig {
        &self.layout_config
    }

    /// Update layout configuration
    pub fn set_config(&mut self, config: LayoutConfig) {
        self.layout_config = config;
        self.clear_cache();
    }

    /// Get layout priority manager
    pub fn priority_manager(&self) -> &LayoutPriorityManager {
        &self.layout_priority_manager
    }

    /// Get mutable layout priority manager
    pub fn priority_manager_mut(&mut self) -> &mut LayoutPriorityManager {
        &mut self.layout_priority_manager
    }

    /// Check if compact mode is active
    pub fn is_compact_mode(&self) -> bool {
        self.compact_mode
    }

    /// Set compact mode manually
    pub fn set_compact_mode(&mut self, enabled: bool) {
        if self.compact_mode != enabled {
            self.compact_mode = enabled;
            self.clear_cache();
            tracing::debug!(
                "Compact mode {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    /// Toggle compact mode
    pub fn toggle_compact_mode(&mut self) {
        self.set_compact_mode(!self.compact_mode);
    }

    /// Check if a widget is in fullscreen mode
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen_widget.is_some()
    }

    /// Get the current fullscreen widget
    pub fn fullscreen_widget(&self) -> Option<&WidgetId> {
        self.fullscreen_widget.as_ref()
    }

    /// Enter fullscreen mode for a widget
    pub fn enter_fullscreen(
        &mut self,
        widget_id: WidgetId,
    ) -> std::result::Result<(), LayoutError> {
        if !self.layout_config.fullscreen_enabled {
            return Err(LayoutError::ConfigError {
                message: "Fullscreen mode is disabled".to_string(),
            });
        }

        if self.fullscreen_widget.is_some() {
            return Err(LayoutError::ConfigError {
                message: "Another widget is already in fullscreen mode".to_string(),
            });
        }

        self.fullscreen_widget = Some(widget_id.clone());
        self.clear_cache();
        tracing::debug!("Entered fullscreen mode for widget: {:?}", widget_id);
        Ok(())
    }

    /// Exit fullscreen mode
    pub fn exit_fullscreen(&mut self) -> std::result::Result<(), LayoutError> {
        if let Some(widget_id) = self.fullscreen_widget.take() {
            self.clear_cache();
            tracing::debug!("Exited fullscreen mode for widget: {:?}", widget_id);
            Ok(())
        } else {
            Err(LayoutError::ConfigError {
                message: "No widget is currently in fullscreen mode".to_string(),
            })
        }
    }

    /// Toggle fullscreen mode for a widget
    pub fn toggle_fullscreen(
        &mut self,
        widget_id: WidgetId,
    ) -> std::result::Result<(), LayoutError> {
        if let Some(current_widget) = &self.fullscreen_widget {
            if current_widget == &widget_id {
                self.exit_fullscreen()
            } else {
                self.exit_fullscreen()?;
                self.enter_fullscreen(widget_id)
            }
        } else {
            self.enter_fullscreen(widget_id)
        }
    }

    /// Add a dynamic adjustment
    pub fn add_dynamic_adjustment(&mut self, adjustment: DynamicAdjustment) {
        self.dynamic_adjustments.push(adjustment);
        self.clear_cache();
    }

    /// Remove a dynamic adjustment
    pub fn remove_dynamic_adjustment(&mut self, id: &str) -> bool {
        let initial_len = self.dynamic_adjustments.len();
        self.dynamic_adjustments.retain(|adj| adj.id != id);
        let removed = self.dynamic_adjustments.len() < initial_len;

        if removed {
            self.clear_cache();
        }

        removed
    }

    /// Get performance metrics
    pub fn performance_metrics(&self) -> &LayoutPerformanceMetrics {
        &self.performance_metrics
    }

    /// Set the root layout node
    pub fn set_root_node(&mut self, node: LayoutNode) -> std::result::Result<(), LayoutError> {
        self.validate_layout_node(&node)?;
        self.root_node = Some(node);
        self.clear_cache();
        Ok(())
    }

    /// Get the root layout node
    pub fn root_node(&self) -> Option<&LayoutNode> {
        self.root_node.as_ref()
    }

    /// Update terminal size and recalculate layouts
    pub fn update_terminal_size(
        &mut self,
        width: u16,
        height: u16,
    ) -> std::result::Result<(), LayoutError> {
        if width < self.min_terminal_size.0 || height < self.min_terminal_size.1 {
            return Err(LayoutError::TerminalTooSmall {
                min_width: self.min_terminal_size.0,
                min_height: self.min_terminal_size.1,
                current_width: width,
                current_height: height,
            });
        }

        let old_size = self.terminal_size;
        self.terminal_size = (width, height);

        // Auto-switch compact mode based on terminal size
        if self.layout_config.auto_compact_mode {
            let should_be_compact = width < self.layout_config.compact_mode_threshold;
            if self.compact_mode != should_be_compact {
                self.compact_mode = should_be_compact;
                tracing::debug!("Auto-switched compact mode to: {}", should_be_compact);
            }
        }

        self.clear_cache();
        tracing::debug!(
            "Terminal size updated from {}x{} to {}x{}",
            old_size.0,
            old_size.1,
            width,
            height
        );
        Ok(())
    }

    /// Get current terminal size
    pub fn terminal_size(&self) -> (u16, u16) {
        self.terminal_size
    }

    /// Calculate layout for the current terminal size
    pub fn calculate_layout(&mut self) -> std::result::Result<LayoutResult, LayoutError> {
        let start_time = std::time::Instant::now();

        // Handle fullscreen mode
        if let Some(ref fullscreen_widget) = self.fullscreen_widget.clone() {
            return self.calculate_fullscreen_layout(fullscreen_widget.clone());
        }

        let cache_key = format!(
            "{}x{}_compact:{}",
            self.terminal_size.0, self.terminal_size.1, self.compact_mode
        );

        // Check cache first
        if self.layout_config.cache_enabled {
            if let Some(cached_result) = self.layout_cache.get(&cache_key).cloned() {
                self.performance_metrics.cache_hits += 1;
                self.performance_metrics.total_calculations += 1;
                self.update_cache_hit_rate();
                return Ok(cached_result);
            }
        }

        self.performance_metrics.cache_misses += 1;
        self.performance_metrics.total_calculations += 1;

        let root_node = self
            .root_node
            .as_ref()
            .ok_or_else(|| LayoutError::ConfigError {
                message: "No root layout node configured".to_string(),
            })?;

        let terminal_rect = Rect {
            x: 0,
            y: 0,
            width: self.terminal_size.0,
            height: self.terminal_size.1,
        };

        // Apply responsive adjustments
        let mut adjusted_node = self.apply_responsive_adjustments(root_node.clone())?;

        // Apply compact mode adjustments
        if self.compact_mode {
            self.apply_compact_mode_adjustments(&mut adjusted_node);
        }

        // Apply dynamic adjustments
        self.apply_dynamic_adjustments(&mut adjusted_node)?;

        // Calculate layout recursively
        let mut widget_areas = HashMap::new();
        let mut overflow_widgets = Vec::new();
        let mut hidden_widgets = Vec::new();

        self.calculate_node_layout(
            &adjusted_node,
            terminal_rect,
            &mut widget_areas,
            &mut overflow_widgets,
            &mut hidden_widgets,
        )?;

        let result = LayoutResult {
            widget_areas,
            total_area: terminal_rect,
            overflow_widgets,
            hidden_widgets,
        };

        // Cache the result if caching is enabled
        if self.layout_config.cache_enabled {
            // Limit cache size
            if self.layout_cache.len() >= self.layout_config.max_cache_size {
                // Remove oldest entry (simple FIFO)
                if let Some(oldest_key) = self.layout_cache.keys().next().cloned() {
                    self.layout_cache.remove(&oldest_key);
                }
            }
            self.layout_cache.insert(cache_key, result.clone());
        }

        // Update performance metrics
        let calculation_time = start_time.elapsed();
        self.performance_metrics.calculation_time_ms = calculation_time.as_millis() as f64;
        self.performance_metrics.last_calculation_time = Some(start_time);
        self.update_average_calculation_time();
        self.update_cache_hit_rate();

        Ok(result)
    }

    /// Calculate layout for fullscreen mode
    fn calculate_fullscreen_layout(
        &mut self,
        widget_id: WidgetId,
    ) -> std::result::Result<LayoutResult, LayoutError> {
        let terminal_rect = Rect {
            x: 0,
            y: 0,
            width: self.terminal_size.0,
            height: self.terminal_size.1,
        };

        let mut widget_areas = HashMap::new();
        widget_areas.insert(widget_id, terminal_rect);

        Ok(LayoutResult {
            widget_areas,
            total_area: terminal_rect,
            overflow_widgets: Vec::new(),
            hidden_widgets: Vec::new(),
        })
    }

    /// Apply compact mode adjustments to a layout node
    fn apply_compact_mode_adjustments(&self, node: &mut LayoutNode) {
        // Reduce margins and padding more aggressively than regular compact mode
        let spacing_factor = self.layout_config.compact_spacing_factor;

        node.margin = Margin {
            top: ((node.margin.top as f64) * spacing_factor) as u16,
            right: ((node.margin.right as f64) * spacing_factor) as u16,
            bottom: ((node.margin.bottom as f64) * spacing_factor) as u16,
            left: ((node.margin.left as f64) * spacing_factor) as u16,
        };

        node.padding = Padding {
            top: ((node.padding.top as f64) * spacing_factor) as u16,
            right: ((node.padding.right as f64) * spacing_factor) as u16,
            bottom: ((node.padding.bottom as f64) * spacing_factor) as u16,
            left: ((node.padding.left as f64) * spacing_factor) as u16,
        };

        // Adjust advanced constraints for compact mode
        if let Some(ref mut advanced_constraints) = node.advanced_constraints {
            // Make widgets more flexible in compact mode
            advanced_constraints.flexible = true;

            // Reduce minimum sizes slightly
            if let Some(min_width) = advanced_constraints.basic.min_width {
                advanced_constraints.basic.min_width = Some((min_width as f64 * 0.8) as u16);
            }
            if let Some(min_height) = advanced_constraints.basic.min_height {
                advanced_constraints.basic.min_height = Some((min_height as f64 * 0.8) as u16);
            }

            // Enable hiding for low-priority widgets
            if advanced_constraints.priority < 0 {
                advanced_constraints.hideable = true;
            }
        }

        // Apply to children recursively
        for child in &mut node.children {
            self.apply_compact_mode_adjustments(child);
        }
    }

    /// Update cache hit rate
    fn update_cache_hit_rate(&mut self) {
        if self.performance_metrics.total_calculations > 0 {
            self.performance_metrics.cache_hit_rate = self.performance_metrics.cache_hits as f64
                / self.performance_metrics.total_calculations as f64;
        }
    }

    /// Update average calculation time
    fn update_average_calculation_time(&mut self) {
        if self.performance_metrics.total_calculations > 0 {
            let total_time = self.performance_metrics.average_calculation_time_ms
                * (self.performance_metrics.total_calculations - 1) as f64;
            self.performance_metrics.average_calculation_time_ms = (total_time
                + self.performance_metrics.calculation_time_ms)
                / self.performance_metrics.total_calculations as f64;
        } else {
            self.performance_metrics.average_calculation_time_ms =
                self.performance_metrics.calculation_time_ms;
        }
    }

    /// Apply dynamic adjustments based on current conditions
    fn apply_dynamic_adjustments(
        &self,
        node: &mut LayoutNode,
    ) -> std::result::Result<(), LayoutError> {
        for adjustment in &self.dynamic_adjustments {
            if !adjustment.enabled {
                continue;
            }

            if self.evaluate_adjustment_condition(&adjustment.condition)? {
                self.apply_layout_adjustment(node, &adjustment.adjustment)?;
            }
        }
        Ok(())
    }

    /// Evaluate an adjustment condition
    fn evaluate_adjustment_condition(
        &self,
        condition: &AdjustmentCondition,
    ) -> std::result::Result<bool, LayoutError> {
        match condition {
            AdjustmentCondition::TerminalSize {
                min_width,
                max_width,
                min_height,
                max_height,
            } => {
                let (width, height) = self.terminal_size;
                Ok(width >= *min_width
                    && width <= *max_width
                    && height >= *min_height
                    && height <= *max_height)
            }
            AdjustmentCondition::WidgetCount {
                min_count,
                max_count,
            } => {
                let widget_count = self.count_widgets_in_node(self.root_node.as_ref().unwrap());
                Ok(widget_count >= *min_count && widget_count <= *max_count)
            }
            AdjustmentCondition::Performance {
                max_cpu_percent: _,
                max_memory_mb: _,
            } => {
                // TODO: Implement performance monitoring
                Ok(false)
            }
            AdjustmentCondition::TimeOfDay {
                start_hour: _,
                end_hour: _,
            } => {
                // TODO: Implement time-based conditions
                Ok(false)
            }
            AdjustmentCondition::Custom(_name) => {
                // TODO: Implement custom condition evaluation
                Ok(false)
            }
        }
    }

    /// Count widgets in a node tree
    fn count_widgets_in_node(&self, node: &LayoutNode) -> usize {
        let mut count = if node.widget_id.is_some() { 1 } else { 0 };
        for child in &node.children {
            count += self.count_widgets_in_node(child);
        }
        count
    }

    /// Calculate layout for a specific node
    fn calculate_node_layout(
        &self,
        node: &LayoutNode,
        area: Rect,
        widget_areas: &mut HashMap<WidgetId, Rect>,
        overflow_widgets: &mut Vec<WidgetId>,
        hidden_widgets: &mut Vec<WidgetId>,
    ) -> std::result::Result<(), LayoutError> {
        if !node.is_visible {
            if let Some(ref widget_id) = node.widget_id {
                hidden_widgets.push(widget_id.clone());
            }
            return Ok(());
        }

        // Check if node is collapsed
        if node.is_collapsed && node.is_collapsible {
            if let Some(ref widget_id) = node.widget_id {
                // Use collapsed size if available
                if let Some(ref advanced_constraints) = node.advanced_constraints {
                    if let Some((collapsed_width, collapsed_height)) =
                        advanced_constraints.collapsed_size
                    {
                        let collapsed_area = Rect {
                            x: area.x,
                            y: area.y,
                            width: collapsed_width.min(area.width),
                            height: collapsed_height.min(area.height),
                        };
                        widget_areas.insert(widget_id.clone(), collapsed_area);
                        return Ok(());
                    }
                }
            }
        }

        // Apply margin to the available area
        let content_area = self.apply_margin(area, &node.margin);

        if node.children.is_empty() {
            // Leaf node - assign area to widget
            if let Some(ref widget_id) = node.widget_id {
                let widget_area = self.apply_padding(content_area, &node.padding);

                // Use advanced constraints if available
                let final_area = if let Some(ref advanced_constraints) = node.advanced_constraints {
                    self.apply_advanced_constraints(widget_area, advanced_constraints)?
                } else if let Some(ref constraints) = node.size_constraints {
                    self.apply_basic_constraints(widget_area, constraints)?
                } else {
                    widget_area
                };

                // Check if widget should be hidden due to size constraints
                if final_area.width < self.layout_config.min_widget_size.0
                    || final_area.height < self.layout_config.min_widget_size.1
                {
                    // Check if widget can be hidden
                    if let Some(ref advanced_constraints) = node.advanced_constraints {
                        if advanced_constraints.hideable {
                            hidden_widgets.push(widget_id.clone());
                            return Ok(());
                        }
                    }

                    overflow_widgets.push(widget_id.clone());
                    return Ok(());
                }

                widget_areas.insert(widget_id.clone(), final_area);
            }
        } else {
            // Container node - layout children with priority-based allocation
            let child_areas = self.calculate_child_areas_with_priority(node, content_area)?;

            for (child, child_area) in node.children.iter().zip(child_areas.iter()) {
                self.calculate_node_layout(
                    child,
                    *child_area,
                    widget_areas,
                    overflow_widgets,
                    hidden_widgets,
                )?;
            }
        }

        Ok(())
    }

    /// Apply advanced size constraints
    fn apply_advanced_constraints(
        &self,
        area: Rect,
        constraints: &AdvancedSizeConstraints,
    ) -> std::result::Result<Rect, LayoutError> {
        // Start with basic constraints
        let (width, height) = constraints.basic.clamp(area.width, area.height);

        // Apply aspect ratio if needed
        let (final_width, final_height) = if constraints.maintain_aspect_ratio {
            constraints.calculate_size_with_aspect_ratio(width, height)
        } else {
            (width, height)
        };

        Ok(Rect {
            x: area.x,
            y: area.y,
            width: final_width,
            height: final_height,
        })
    }

    /// Apply basic size constraints
    fn apply_basic_constraints(
        &self,
        area: Rect,
        constraints: &SizeConstraints,
    ) -> std::result::Result<Rect, LayoutError> {
        let (width, height) = constraints.clamp(area.width, area.height);

        Ok(Rect {
            x: area.x,
            y: area.y,
            width,
            height,
        })
    }

    /// Calculate areas for child nodes with priority-based allocation
    fn calculate_child_areas_with_priority(
        &self,
        node: &LayoutNode,
        area: Rect,
    ) -> std::result::Result<Vec<Rect>, LayoutError> {
        if node.children.is_empty() {
            return Ok(vec![]);
        }

        // Check if priority manager is enabled and any children have priority-based advanced constraints
        let has_priority_children = self.layout_priority_manager.is_enabled()
            && node.children.iter().any(|child| {
                child
                    .advanced_constraints
                    .as_ref()
                    .is_some_and(|ac| ac.priority != 0)
                    || child.widget_id.as_ref().is_some_and(|wid| {
                        self.layout_priority_manager.get_widget_priority(wid) != 0
                    })
            });

        if has_priority_children {
            self.calculate_priority_based_layout(node, area)
        } else {
            self.calculate_child_areas(node, area)
        }
    }

    /// Calculate layout with priority-based space allocation
    fn calculate_priority_based_layout(
        &self,
        node: &LayoutNode,
        area: Rect,
    ) -> std::result::Result<Vec<Rect>, LayoutError> {
        let mut areas = vec![Rect::default(); node.children.len()];
        let mut remaining_area = area;

        // Create indexed children with their priorities
        let mut indexed_children: Vec<(usize, &LayoutNode, i32)> = node
            .children
            .iter()
            .enumerate()
            .map(|(i, child)| {
                let priority = if let Some(ref widget_id) = child.widget_id {
                    self.layout_priority_manager.get_widget_priority(widget_id)
                } else {
                    child
                        .advanced_constraints
                        .as_ref()
                        .map_or(0, |ac| ac.priority)
                };
                (i, child, priority)
            })
            .collect();

        // Sort by priority (highest first)
        indexed_children.sort_by(|(_, _, a), (_, _, b)| b.cmp(a));

        // Allocate space based on priority
        for (original_index, child, priority) in &indexed_children {
            if remaining_area.width == 0 || remaining_area.height == 0 {
                // No more space available
                areas[*original_index] = Rect::default();
                continue;
            }

            let allocated_area = if let Some(ref advanced_constraints) = child.advanced_constraints
            {
                // Calculate preferred size for this child
                let preferred_width = advanced_constraints
                    .basic
                    .preferred_width
                    .unwrap_or(remaining_area.width / 2);
                let preferred_height = advanced_constraints
                    .basic
                    .preferred_height
                    .unwrap_or(remaining_area.height / 2);

                // Apply priority multiplier for high-priority widgets
                let priority_multiplier = if *priority > 0 {
                    1.0 + (*priority as f64 * 0.1) // 10% more space per priority point
                } else {
                    1.0 + (*priority as f64 * 0.05) // 5% less space per negative priority point
                };

                let allocated_width = ((preferred_width as f64) * priority_multiplier) as u16;
                let allocated_height = ((preferred_height as f64) * priority_multiplier) as u16;

                (
                    allocated_width.min(remaining_area.width),
                    allocated_height.min(remaining_area.height),
                )
            } else {
                // No advanced constraints, use equal distribution of remaining space
                let current_position = indexed_children
                    .iter()
                    .position(|(i, _, _)| i == original_index)
                    .unwrap();
                let remaining_children = indexed_children.len() - current_position;

                (
                    remaining_area.width / remaining_children as u16,
                    remaining_area.height / remaining_children as u16,
                )
            };

            areas[*original_index] = match node.direction {
                LayoutDirection::Horizontal => {
                    let child_area = Rect {
                        x: remaining_area.x,
                        y: remaining_area.y,
                        width: allocated_area.0,
                        height: remaining_area.height,
                    };
                    remaining_area.x += allocated_area.0;
                    remaining_area.width = remaining_area.width.saturating_sub(allocated_area.0);
                    child_area
                }
                LayoutDirection::Vertical => {
                    let child_area = Rect {
                        x: remaining_area.x,
                        y: remaining_area.y,
                        width: remaining_area.width,
                        height: allocated_area.1,
                    };
                    remaining_area.y += allocated_area.1;
                    remaining_area.height = remaining_area.height.saturating_sub(allocated_area.1);
                    child_area
                }
            };
        }

        Ok(areas)
    }

    /// Calculate areas for child nodes
    fn calculate_child_areas(
        &self,
        node: &LayoutNode,
        area: Rect,
    ) -> std::result::Result<Vec<Rect>, LayoutError> {
        if node.children.is_empty() {
            return Ok(vec![]);
        }

        // Convert layout constraints to ratatui constraints
        let constraints: Vec<Constraint> = node
            .constraints
            .iter()
            .map(|c| self.convert_layout_constraint(c, area))
            .collect();

        // Use ratatui's layout system
        let layout = Layout::default()
            .direction(match node.direction {
                LayoutDirection::Horizontal => Direction::Horizontal,
                LayoutDirection::Vertical => Direction::Vertical,
            })
            .constraints(constraints);

        let areas = layout.split(area);

        if areas.len() != node.children.len() {
            return Err(LayoutError::CalculationError {
                message: format!(
                    "Layout calculation mismatch: expected {} areas, got {}",
                    node.children.len(),
                    areas.len()
                ),
            });
        }

        Ok(areas.to_vec())
    }

    /// Convert layout constraint to ratatui constraint
    fn convert_layout_constraint(&self, constraint: &LayoutConstraints, _area: Rect) -> Constraint {
        match constraint {
            LayoutConstraints::Length(len) => Constraint::Length(*len),
            LayoutConstraints::Percentage(pct) => Constraint::Percentage(*pct),
            LayoutConstraints::Min(min) => Constraint::Min(*min),
            LayoutConstraints::Max(max) => Constraint::Max(*max),
            LayoutConstraints::Fill => Constraint::Fill(1),
            LayoutConstraints::Ratio(num, den) => Constraint::Ratio(*num, *den),
        }
    }

    /// Apply margin to an area
    fn apply_margin(&self, area: Rect, margin: &Margin) -> Rect {
        let x = area.x + margin.left;
        let y = area.y + margin.top;
        let width = area.width.saturating_sub(margin.left + margin.right);
        let height = area.height.saturating_sub(margin.top + margin.bottom);

        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Apply padding to an area
    fn apply_padding(&self, area: Rect, padding: &Padding) -> Rect {
        let x = area.x + padding.left;
        let y = area.y + padding.top;
        let width = area.width.saturating_sub(padding.left + padding.right);
        let height = area.height.saturating_sub(padding.top + padding.bottom);

        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Apply responsive adjustments to a layout node
    fn apply_responsive_adjustments(
        &self,
        mut node: LayoutNode,
    ) -> std::result::Result<LayoutNode, LayoutError> {
        // Find the appropriate breakpoint
        let breakpoint = self.find_active_breakpoint();

        if let Some(bp) = breakpoint {
            for adjustment in &bp.layout_adjustments {
                self.apply_layout_adjustment(&mut node, adjustment)?;
            }
        }

        Ok(node)
    }

    /// Find the active responsive breakpoint
    fn find_active_breakpoint(&self) -> Option<&ResponsiveBreakpoint> {
        let (width, height) = self.terminal_size;

        // Find the largest breakpoint that fits
        self.responsive_breakpoints
            .iter()
            .filter(|bp| width >= bp.min_width && height >= bp.min_height)
            .max_by_key(|bp| bp.min_width * bp.min_height)
    }

    /// Apply a layout adjustment to a node
    fn apply_layout_adjustment(
        &self,
        node: &mut LayoutNode,
        adjustment: &LayoutAdjustment,
    ) -> std::result::Result<(), LayoutError> {
        match adjustment {
            LayoutAdjustment::HideWidgets(widget_ids) => {
                self.hide_widgets_in_node(node, widget_ids);
            }
            LayoutAdjustment::ChangeDirection(node_id, new_direction) => {
                if let Some(target_node) = self.find_node_mut(node, node_id) {
                    target_node.direction = new_direction.clone();
                }
            }
            LayoutAdjustment::ModifyConstraints(node_id, new_constraints) => {
                if let Some(target_node) = self.find_node_mut(node, node_id) {
                    target_node.constraints = new_constraints.clone();
                }
            }
            LayoutAdjustment::CompactMode(enabled) => {
                if *enabled {
                    self.apply_compact_mode(node);
                }
            }
            LayoutAdjustment::AdjustSpacing(node_id, margin, padding) => {
                if let Some(target_node) = self.find_node_mut(node, node_id) {
                    target_node.margin = margin.clone();
                    target_node.padding = padding.clone();
                }
            }
            LayoutAdjustment::SetPriority(widget_id, priority) => {
                if let Some(target_node) = self.find_widget_node_mut(node, widget_id) {
                    target_node.priority = *priority;
                }
            }
            LayoutAdjustment::SetScrollable(_widget_id, _scrollable) => {
                // TODO: Implement scrollable widget support
                tracing::debug!("SetScrollable adjustment not yet implemented");
            }
        }

        Ok(())
    }

    /// Hide widgets in a node tree
    fn hide_widgets_in_node(&self, node: &mut LayoutNode, widget_ids: &[WidgetId]) {
        if let Some(ref widget_id) = node.widget_id {
            if widget_ids.contains(widget_id) {
                node.is_visible = false;
            }
        }

        for child in &mut node.children {
            self.hide_widgets_in_node(child, widget_ids);
        }
    }

    /// Find a node by ID (mutable)
    fn find_node_mut<'a>(
        &self,
        node: &'a mut LayoutNode,
        node_id: &str,
    ) -> Option<&'a mut LayoutNode> {
        if node.id == node_id {
            return Some(node);
        }

        for child in &mut node.children {
            if let Some(found) = self.find_node_mut(child, node_id) {
                return Some(found);
            }
        }

        None
    }

    /// Find a node by widget ID (mutable)
    fn find_widget_node_mut<'a>(
        &self,
        node: &'a mut LayoutNode,
        widget_id: &WidgetId,
    ) -> Option<&'a mut LayoutNode> {
        if let Some(ref node_widget_id) = node.widget_id {
            if node_widget_id == widget_id {
                return Some(node);
            }
        }

        for child in &mut node.children {
            if let Some(found) = self.find_widget_node_mut(child, widget_id) {
                return Some(found);
            }
        }

        None
    }

    /// Apply compact mode adjustments
    fn apply_compact_mode(&self, node: &mut LayoutNode) {
        // Reduce margins and padding
        node.margin = Margin {
            top: node.margin.top.min(1),
            right: node.margin.right.min(1),
            bottom: node.margin.bottom.min(1),
            left: node.margin.left.min(1),
        };

        node.padding = Padding {
            top: node.padding.top.min(1),
            right: node.padding.right.min(1),
            bottom: node.padding.bottom.min(1),
            left: node.padding.left.min(1),
        };

        // Apply to children recursively
        for child in &mut node.children {
            self.apply_compact_mode(child);
        }
    }

    /// Validate a layout node configuration
    fn validate_layout_node(&self, node: &LayoutNode) -> std::result::Result<(), LayoutError> {
        // Check that constraints match children count
        if !node.children.is_empty() && node.constraints.len() != node.children.len() {
            return Err(LayoutError::ConfigError {
                message: format!(
                    "Node '{}' has {} children but {} constraints",
                    node.id,
                    node.children.len(),
                    node.constraints.len()
                ),
            });
        }

        // Validate children recursively
        for child in &node.children {
            self.validate_layout_node(child)?;
        }

        Ok(())
    }

    /// Clear the layout cache
    fn clear_cache(&mut self) {
        self.layout_cache.clear();
        tracing::debug!("Layout cache cleared");
    }

    /// Get layout information for a specific widget
    pub fn get_widget_area(
        &mut self,
        widget_id: &WidgetId,
    ) -> std::result::Result<Option<Rect>, LayoutError> {
        let layout_result = self.calculate_layout()?;
        Ok(layout_result.widget_areas.get(widget_id).copied())
    }

    /// Check if a widget is currently visible
    pub fn is_widget_visible(
        &mut self,
        widget_id: &WidgetId,
    ) -> std::result::Result<bool, LayoutError> {
        let layout_result = self.calculate_layout()?;
        Ok(!layout_result.hidden_widgets.contains(widget_id)
            && !layout_result.overflow_widgets.contains(widget_id))
    }

    /// Get all visible widgets
    pub fn get_visible_widgets(&mut self) -> std::result::Result<Vec<WidgetId>, LayoutError> {
        let layout_result = self.calculate_layout()?;
        Ok(layout_result.widget_areas.keys().cloned().collect())
    }

    /// Get overflow widgets (widgets that don't fit)
    pub fn get_overflow_widgets(&mut self) -> std::result::Result<Vec<WidgetId>, LayoutError> {
        let layout_result = self.calculate_layout()?;
        Ok(layout_result.overflow_widgets)
    }

    /// Get hidden widgets (widgets that are explicitly hidden)
    pub fn get_hidden_widgets(&mut self) -> std::result::Result<Vec<WidgetId>, LayoutError> {
        let layout_result = self.calculate_layout()?;
        Ok(layout_result.hidden_widgets)
    }

    /// Set minimum terminal size
    pub fn set_min_terminal_size(&mut self, width: u16, height: u16) {
        self.min_terminal_size = (width, height);
        tracing::debug!("Minimum terminal size set to {}x{}", width, height);
    }

    /// Add a responsive breakpoint
    pub fn add_breakpoint(&mut self, breakpoint: ResponsiveBreakpoint) {
        self.responsive_breakpoints.push(breakpoint);

        // Sort breakpoints by size (smallest first)
        self.responsive_breakpoints
            .sort_by_key(|bp| bp.min_width * bp.min_height);

        self.clear_cache();
    }

    /// Remove a responsive breakpoint
    pub fn remove_breakpoint(&mut self, name: &str) -> bool {
        let initial_len = self.responsive_breakpoints.len();
        self.responsive_breakpoints.retain(|bp| bp.name != name);
        let removed = self.responsive_breakpoints.len() < initial_len;

        if removed {
            self.clear_cache();
        }

        removed
    }

    /// Get default responsive breakpoints
    fn default_breakpoints() -> Vec<ResponsiveBreakpoint> {
        vec![
            ResponsiveBreakpoint {
                name: "ultra-compact".to_string(),
                min_width: 40,
                min_height: 10,
                layout_adjustments: vec![
                    LayoutAdjustment::CompactMode(true),
                    LayoutAdjustment::AdjustSpacing(
                        "root".to_string(),
                        Margin::uniform(0),
                        Padding::uniform(0),
                    ),
                ],
            },
            ResponsiveBreakpoint {
                name: "compact".to_string(),
                min_width: 60,
                min_height: 15,
                layout_adjustments: vec![
                    LayoutAdjustment::CompactMode(true),
                    LayoutAdjustment::AdjustSpacing(
                        "root".to_string(),
                        Margin::uniform(1),
                        Padding::uniform(1),
                    ),
                ],
            },
            ResponsiveBreakpoint {
                name: "small".to_string(),
                min_width: 80,
                min_height: 24,
                layout_adjustments: vec![],
            },
            ResponsiveBreakpoint {
                name: "medium".to_string(),
                min_width: 120,
                min_height: 30,
                layout_adjustments: vec![],
            },
            ResponsiveBreakpoint {
                name: "large".to_string(),
                min_width: 160,
                min_height: 40,
                layout_adjustments: vec![],
            },
        ]
    }

    /// Get layout statistics
    pub fn get_layout_stats(&mut self) -> std::result::Result<LayoutStats, LayoutError> {
        let layout_result = self.calculate_layout()?;

        Ok(LayoutStats {
            total_widgets: layout_result.widget_areas.len(),
            visible_widgets: layout_result.widget_areas.len(),
            hidden_widgets: layout_result.hidden_widgets.len(),
            overflow_widgets: layout_result.overflow_widgets.len(),
            terminal_size: self.terminal_size,
            active_breakpoint: self.find_active_breakpoint().map(|bp| bp.name.clone()),
            cache_size: self.layout_cache.len(),
            compact_mode: self.compact_mode,
            fullscreen_widget: self.fullscreen_widget.clone(),
        })
    }

    /// Get compact mode threshold
    pub fn compact_mode_threshold(&self) -> u16 {
        self.layout_config.compact_mode_threshold
    }

    /// Set compact mode threshold
    pub fn set_compact_mode_threshold(&mut self, threshold: u16) {
        self.layout_config.compact_mode_threshold = threshold;

        // Re-evaluate compact mode if auto mode is enabled
        if self.layout_config.auto_compact_mode {
            let should_be_compact = self.terminal_size.0 < threshold;
            if self.compact_mode != should_be_compact {
                self.set_compact_mode(should_be_compact);
            }
        }
    }

    /// Check if auto compact mode is enabled
    pub fn is_auto_compact_mode(&self) -> bool {
        self.layout_config.auto_compact_mode
    }

    /// Set auto compact mode
    pub fn set_auto_compact_mode(&mut self, enabled: bool) {
        self.layout_config.auto_compact_mode = enabled;

        if enabled {
            // Apply current terminal size logic
            let should_be_compact =
                self.terminal_size.0 < self.layout_config.compact_mode_threshold;
            if self.compact_mode != should_be_compact {
                self.set_compact_mode(should_be_compact);
            }
        }
    }

    /// Get all widgets that can be made fullscreen
    pub fn get_fullscreen_capable_widgets(&self) -> Vec<WidgetId> {
        let mut widgets = Vec::new();

        if let Some(ref root_node) = self.root_node {
            self.collect_fullscreen_widgets(root_node, &mut widgets);
        }

        widgets
    }

    /// Collect widgets that can be made fullscreen
    fn collect_fullscreen_widgets(&self, node: &LayoutNode, widgets: &mut Vec<WidgetId>) {
        if let Some(ref widget_id) = node.widget_id {
            // Check if widget can be made fullscreen
            let can_fullscreen = node
                .advanced_constraints
                .as_ref()
                .is_none_or(|ac| !ac.hideable); // Don't allow hideable widgets to go fullscreen

            if can_fullscreen {
                widgets.push(widget_id.clone());
            }
        }

        for child in &node.children {
            self.collect_fullscreen_widgets(child, widgets);
        }
    }

    /// Get compact mode spacing factor
    pub fn compact_spacing_factor(&self) -> f64 {
        self.layout_config.compact_spacing_factor
    }

    /// Set compact mode spacing factor
    pub fn set_compact_spacing_factor(&mut self, factor: f64) {
        self.layout_config.compact_spacing_factor = factor.clamp(0.0, 1.0);
        if self.compact_mode {
            self.clear_cache();
        }
    }

    // === Focus Management Methods ===

    /// Get the focus manager
    pub fn focus_manager(&self) -> &FocusManager {
        &self.focus_manager
    }

    /// Get mutable focus manager
    pub fn focus_manager_mut(&mut self) -> &mut FocusManager {
        &mut self.focus_manager
    }

    /// Set focus to a widget
    pub fn set_focus(&mut self, widget_id: WidgetId) -> bool {
        self.focus_manager.set_focus(widget_id)
    }

    /// Clear focus
    pub fn clear_focus(&mut self) {
        self.focus_manager.clear_focus();
    }

    /// Move focus to next focusable widget
    pub fn focus_next(&mut self) -> bool {
        let available_widgets = self.get_all_widget_ids();
        self.focus_manager.focus_next(&available_widgets)
    }

    /// Move focus to previous focusable widget
    pub fn focus_previous(&mut self) -> bool {
        let available_widgets = self.get_all_widget_ids();
        self.focus_manager.focus_previous(&available_widgets)
    }

    /// Go back to previous focused widget
    pub fn focus_back(&mut self) -> bool {
        self.focus_manager.focus_back()
    }

    /// Register a widget for focus management
    pub fn register_widget_for_focus(&mut self, widget_id: WidgetId, state: FocusState) {
        self.focus_manager.register_widget(widget_id, state);
    }

    /// Unregister a widget from focus management
    pub fn unregister_widget_from_focus(&mut self, widget_id: &WidgetId) {
        self.focus_manager.unregister_widget(widget_id);
    }

    /// Check if a widget is focused
    pub fn is_widget_focused(&self, widget_id: &WidgetId) -> bool {
        self.focus_manager.is_focused(widget_id)
    }

    /// Check if a widget is focusable
    pub fn is_widget_focusable(&self, widget_id: &WidgetId) -> bool {
        self.focus_manager.is_focusable(widget_id)
    }

    /// Set widget focusable state
    pub fn set_widget_focusable(&mut self, widget_id: &WidgetId, focusable: bool) {
        self.focus_manager.set_focusable(widget_id, focusable);
    }

    /// Get currently focused widget
    pub fn get_focused_widget(&self) -> Option<&WidgetId> {
        self.focus_manager.focused_widget()
    }

    /// Handle layout change and preserve focus if needed
    pub fn handle_focus_on_layout_change(&mut self) {
        let available_widgets = self.get_all_widget_ids();
        self.focus_manager.handle_layout_change(&available_widgets);
    }

    /// Get focus statistics
    pub fn get_focus_stats(&self) -> FocusStats {
        self.focus_manager.get_focus_stats()
    }

    /// Enable or disable focus preservation during layout changes
    pub fn set_focus_preservation(&mut self, enabled: bool) {
        self.focus_manager.set_preserve_focus(enabled);
    }

    /// Enable or disable visual focus indicators
    pub fn set_focus_visual_indicators(&mut self, enabled: bool) {
        self.focus_manager.set_visual_indicators(enabled);
    }

    /// Get focus ring configuration
    pub fn get_focus_ring(&self) -> &FocusRing {
        self.focus_manager.focus_ring()
    }

    /// Set focus ring configuration
    pub fn set_focus_ring(&mut self, focus_ring: FocusRing) {
        self.focus_manager.set_focus_ring(focus_ring);
    }

    /// Get all widget IDs from the layout tree
    fn get_all_widget_ids(&self) -> Vec<WidgetId> {
        let mut widget_ids = Vec::new();
        if let Some(ref root_node) = self.root_node {
            self.collect_widget_ids(root_node, &mut widget_ids);
        }
        widget_ids
    }

    /// Collect widget IDs from a node tree
    fn collect_widget_ids(&self, node: &LayoutNode, widget_ids: &mut Vec<WidgetId>) {
        if let Some(ref widget_id) = node.widget_id {
            widget_ids.push(widget_id.clone());
        }

        for child in &node.children {
            self.collect_widget_ids(child, widget_ids);
        }
    }

    /// Update terminal size and handle focus changes
    pub fn update_terminal_size_with_focus(
        &mut self,
        width: u16,
        height: u16,
    ) -> std::result::Result<(), LayoutError> {
        // Store old widget list
        let old_widgets = self.get_all_widget_ids();

        // Update terminal size
        self.update_terminal_size(width, height)?;

        // Handle focus changes due to layout adjustments
        let new_widgets = self.get_all_widget_ids();
        if old_widgets != new_widgets {
            self.handle_focus_on_layout_change();
        }

        Ok(())
    }
}

/// Layout statistics
#[derive(Debug, Clone)]
pub struct LayoutStats {
    pub total_widgets: usize,
    pub visible_widgets: usize,
    pub hidden_widgets: usize,
    pub overflow_widgets: usize,
    pub terminal_size: (u16, u16),
    pub active_breakpoint: Option<String>,
    pub cache_size: usize,
    pub compact_mode: bool,
    pub fullscreen_widget: Option<WidgetId>,
}

impl LayoutNode {
    /// Create a new layout node
    pub fn new(id: String, direction: LayoutDirection) -> Self {
        Self {
            id,
            direction,
            constraints: Vec::new(),
            children: Vec::new(),
            widget_id: None,
            size_constraints: None,
            advanced_constraints: None,
            margin: Margin::default(),
            padding: Padding::default(),
            is_visible: true,
            z_index: 0,
            priority: 0,
            is_collapsible: false,
            is_collapsed: false,
            animation_state: AnimationState::default(),
        }
    }

    /// Create a leaf node for a widget
    pub fn widget(id: String, widget_id: WidgetId) -> Self {
        Self {
            id,
            direction: LayoutDirection::Vertical,
            constraints: Vec::new(),
            children: Vec::new(),
            widget_id: Some(widget_id),
            size_constraints: None,
            advanced_constraints: None,
            margin: Margin::default(),
            padding: Padding::default(),
            is_visible: true,
            z_index: 0,
            priority: 0,
            is_collapsible: false,
            is_collapsed: false,
            animation_state: AnimationState::default(),
        }
    }

    /// Add a child node
    pub fn add_child(mut self, child: LayoutNode, constraint: LayoutConstraints) -> Self {
        self.children.push(child);
        self.constraints.push(constraint);
        self
    }

    /// Set size constraints
    pub fn with_size_constraints(mut self, constraints: SizeConstraints) -> Self {
        self.size_constraints = Some(constraints);
        self
    }

    /// Set advanced size constraints
    pub fn with_advanced_constraints(mut self, constraints: AdvancedSizeConstraints) -> Self {
        self.advanced_constraints = Some(constraints);
        self
    }

    /// Set margin
    pub fn with_margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    /// Set padding
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Set visibility
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.is_visible = visible;
        self
    }

    /// Set z-index
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Make collapsible
    pub fn collapsible(mut self) -> Self {
        self.is_collapsible = true;
        self
    }

    /// Set collapsed state
    pub fn with_collapsed(mut self, collapsed: bool) -> Self {
        self.is_collapsed = collapsed;
        self
    }

    /// Set animation state
    pub fn with_animation(mut self, animation: AnimationState) -> Self {
        self.animation_state = animation;
        self
    }

    /// Toggle collapsed state
    pub fn toggle_collapsed(&mut self) {
        if self.is_collapsible {
            self.is_collapsed = !self.is_collapsed;
        }
    }

    /// Expand the node (if collapsible)
    pub fn expand(&mut self) {
        if self.is_collapsible {
            self.is_collapsed = false;
        }
    }

    /// Collapse the node (if collapsible)
    pub fn collapse(&mut self) {
        if self.is_collapsible {
            self.is_collapsed = true;
        }
    }
}

impl Margin {
    /// Create uniform margin
    pub fn uniform(size: u16) -> Self {
        Self {
            top: size,
            right: size,
            bottom: size,
            left: size,
        }
    }

    /// Create horizontal and vertical margin
    pub fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create margin with individual values
    pub fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Padding {
    /// Create uniform padding
    pub fn uniform(size: u16) -> Self {
        Self {
            top: size,
            right: size,
            bottom: size,
            left: size,
        }
    }

    /// Create horizontal and vertical padding
    pub fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create padding with individual values
    pub fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Direction> for LayoutDirection {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Horizontal => LayoutDirection::Horizontal,
            Direction::Vertical => LayoutDirection::Vertical,
        }
    }
}

impl From<LayoutDirection> for Direction {
    fn from(direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Horizontal => Direction::Horizontal,
            LayoutDirection::Vertical => Direction::Vertical,
        }
    }
}
