//! Layout management system for TUI widgets
//! 
//! This module provides responsive layout calculation and widget size constraint handling.

use crate::error::{Result, WorkflowError};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use super::widget::{WidgetId, SizeConstraints};

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
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub direction: LayoutDirection,
    pub constraints: Vec<LayoutConstraints>,
    pub children: Vec<LayoutNode>,
    pub widget_id: Option<WidgetId>,
    pub size_constraints: Option<SizeConstraints>,
    pub margin: Margin,
    pub padding: Padding,
    pub is_visible: bool,
    pub z_index: i32,
}

/// Margin configuration
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

/// Padding configuration
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
}

/// Responsive breakpoint configuration
#[derive(Debug, Clone)]
pub struct ResponsiveBreakpoint {
    pub name: String,
    pub min_width: u16,
    pub min_height: u16,
    pub layout_adjustments: Vec<LayoutAdjustment>,
}

/// Layout adjustment for responsive design
#[derive(Debug, Clone)]
pub enum LayoutAdjustment {
    /// Hide specific widgets
    HideWidgets(Vec<WidgetId>),
    /// Change layout direction
    ChangeDirection(String, LayoutDirection), // (node_id, new_direction)
    /// Modify constraints
    ModifyConstraints(String, Vec<LayoutConstraints>), // (node_id, new_constraints)
    /// Switch to compact mode
    CompactMode(bool),
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new() -> Self {
        Self {
            root_node: None,
            terminal_size: (80, 24), // Default terminal size
            min_terminal_size: (40, 10), // Minimum usable size
            layout_cache: HashMap::new(),
            responsive_breakpoints: Self::default_breakpoints(),
        }
    }
    
    /// Set the root layout node
    pub fn set_root_node(&mut self, node: LayoutNode) -> Result<(), LayoutError> {
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
    pub fn update_terminal_size(&mut self, width: u16, height: u16) -> Result<(), LayoutError> {
        if width < self.min_terminal_size.0 || height < self.min_terminal_size.1 {
            return Err(LayoutError::TerminalTooSmall {
                min_width: self.min_terminal_size.0,
                min_height: self.min_terminal_size.1,
                current_width: width,
                current_height: height,
            });
        }
        
        self.terminal_size = (width, height);
        self.clear_cache();
        tracing::debug!("Terminal size updated to {}x{}", width, height);
        Ok(())
    }
    
    /// Get current terminal size
    pub fn terminal_size(&self) -> (u16, u16) {
        self.terminal_size
    }
    
    /// Calculate layout for the current terminal size
    pub fn calculate_layout(&mut self) -> Result<LayoutResult, LayoutError> {
        let cache_key = format!("{}x{}", self.terminal_size.0, self.terminal_size.1);
        
        // Check cache first
        if let Some(cached_result) = self.layout_cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }
        
        let root_node = self.root_node.as_ref()
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
        let adjusted_node = self.apply_responsive_adjustments(root_node.clone())?;
        
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
        
        // Cache the result
        self.layout_cache.insert(cache_key, result.clone());
        
        Ok(result)
    }
    
    /// Calculate layout for a specific node
    fn calculate_node_layout(
        &self,
        node: &LayoutNode,
        area: Rect,
        widget_areas: &mut HashMap<WidgetId, Rect>,
        overflow_widgets: &mut Vec<WidgetId>,
        hidden_widgets: &mut Vec<WidgetId>,
    ) -> Result<(), LayoutError> {
        if !node.is_visible {
            if let Some(ref widget_id) = node.widget_id {
                hidden_widgets.push(widget_id.clone());
            }
            return Ok(());
        }
        
        // Apply margin to the available area
        let content_area = self.apply_margin(area, &node.margin);
        
        if node.children.is_empty() {
            // Leaf node - assign area to widget
            if let Some(ref widget_id) = node.widget_id {
                let widget_area = self.apply_padding(content_area, &node.padding);
                
                // Check size constraints
                if let Some(ref constraints) = node.size_constraints {
                    if !constraints.satisfies(widget_area.width, widget_area.height) {
                        let (clamped_width, clamped_height) = constraints.clamp(widget_area.width, widget_area.height);
                        
                        if clamped_width != widget_area.width || clamped_height != widget_area.height {
                            tracing::warn!(
                                "Widget {} size clamped from {}x{} to {}x{}",
                                widget_id,
                                widget_area.width,
                                widget_area.height,
                                clamped_width,
                                clamped_height
                            );
                            
                            if clamped_width == 0 || clamped_height == 0 {
                                overflow_widgets.push(widget_id.clone());
                                return Ok(());
                            }
                        }
                        
                        let clamped_area = Rect {
                            x: widget_area.x,
                            y: widget_area.y,
                            width: clamped_width,
                            height: clamped_height,
                        };
                        
                        widget_areas.insert(widget_id.clone(), clamped_area);
                    } else {
                        widget_areas.insert(widget_id.clone(), widget_area);
                    }
                } else {
                    widget_areas.insert(widget_id.clone(), widget_area);
                }
            }
        } else {
            // Container node - layout children
            let child_areas = self.calculate_child_areas(node, content_area)?;
            
            for (child, child_area) in node.children.iter().zip(child_areas.iter()) {
                self.calculate_node_layout(child, *child_area, widget_areas, overflow_widgets, hidden_widgets)?;
            }
        }
        
        Ok(())
    }
    
    /// Calculate areas for child nodes
    fn calculate_child_areas(&self, node: &LayoutNode, area: Rect) -> std::result::Result<Vec<Rect>, LayoutError> {
        if node.children.is_empty() {
            return Ok(vec![]);
        }
        
        // Convert layout constraints to ratatui constraints
        let constraints: Vec<Constraint> = node.constraints.iter()
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
    fn convert_layout_constraint(&self, constraint: &LayoutConstraints, area: Rect) -> Constraint {
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
        
        Rect { x, y, width, height }
    }
    
    /// Apply padding to an area
    fn apply_padding(&self, area: Rect, padding: &Padding) -> Rect {
        let x = area.x + padding.left;
        let y = area.y + padding.top;
        let width = area.width.saturating_sub(padding.left + padding.right);
        let height = area.height.saturating_sub(padding.top + padding.bottom);
        
        Rect { x, y, width, height }
    }
    
    /// Apply responsive adjustments to a layout node
    fn apply_responsive_adjustments(&self, mut node: LayoutNode) -> std::result::Result<LayoutNode, LayoutError> {
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
    fn apply_layout_adjustment(&self, node: &mut LayoutNode, adjustment: &LayoutAdjustment) -> std::result::Result<(), LayoutError> {
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
    fn find_node_mut<'a>(&self, node: &'a mut LayoutNode, node_id: &str) -> Option<&'a mut LayoutNode> {
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
    pub fn get_widget_area(&mut self, widget_id: &WidgetId) -> std::result::Result<Option<Rect>, LayoutError> {
        let layout_result = self.calculate_layout()?;
        Ok(layout_result.widget_areas.get(widget_id).copied())
    }
    
    /// Check if a widget is currently visible
    pub fn is_widget_visible(&mut self, widget_id: &WidgetId) -> std::result::Result<bool, LayoutError> {
        let layout_result = self.calculate_layout()?;
        Ok(!layout_result.hidden_widgets.contains(widget_id) && 
           !layout_result.overflow_widgets.contains(widget_id))
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
        self.responsive_breakpoints.sort_by_key(|bp| bp.min_width * bp.min_height);
        
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
                name: "compact".to_string(),
                min_width: 40,
                min_height: 10,
                layout_adjustments: vec![
                    LayoutAdjustment::CompactMode(true),
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
        })
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
            margin: Margin::default(),
            padding: Padding::default(),
            is_visible: true,
            z_index: 0,
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
            margin: Margin::default(),
            padding: Padding::default(),
            is_visible: true,
            z_index: 0,
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
        Self { top, right, bottom, left }
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
        Self { top, right, bottom, left }
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