//! Tool Manager Widget Implementation
//! 
//! This widget displays and manages available tools with filtering, sorting, and execution capabilities.

use crate::interfaces::tui::{
    Widget, WidgetId, WidgetContext, Theme
};
use crate::interfaces::tui::widget::{WidgetCapabilities, SizeConstraints, UpdateFrequency, WidgetError};
use crate::interfaces::tui::action::{Action, SortOrder, ViewType};
use crate::core::ToolInfo;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Rect, Layout, Direction, Constraint},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Multi-dimensional filter set
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterSet {
    pub status_filter: Option<ToolStatus>,
    pub category_filter: Option<String>,
    pub source_filter: Option<String>,
    pub tag_filters: Vec<String>,
    pub version_filter: Option<String>,
    pub execution_count_range: Option<(u32, u32)>,
    pub success_rate_range: Option<(f64, f64)>,
    pub text_search: String,
}

impl FilterSet {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn is_empty(&self) -> bool {
        self.status_filter.is_none() &&
        self.category_filter.is_none() &&
        self.source_filter.is_none() &&
        self.tag_filters.is_empty() &&
        self.version_filter.is_none() &&
        self.execution_count_range.is_none() &&
        self.success_rate_range.is_none() &&
        self.text_search.is_empty()
    }
    
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    
    pub fn to_display_string(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(status) = &self.status_filter {
            parts.push(format!("status:{}", status.description()));
        }
        if let Some(category) = &self.category_filter {
            parts.push(format!("category:{}", category));
        }
        if let Some(source) = &self.source_filter {
            parts.push(format!("source:{}", source));
        }
        if !self.tag_filters.is_empty() {
            parts.push(format!("tags:{}", self.tag_filters.join(",")));
        }
        if let Some(version) = &self.version_filter {
            parts.push(format!("version:{}", version));
        }
        if let Some((min, max)) = self.execution_count_range {
            parts.push(format!("executions:{}..{}", min, max));
        }
        if let Some((min, max)) = self.success_rate_range {
            parts.push(format!("success:{:.1}%..{:.1}%", min * 100.0, max * 100.0));
        }
        if !self.text_search.is_empty() {
            parts.push(self.text_search.clone());
        }
        
        parts.join(" ")
    }
}

/// Tool execution record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRecord {
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub success: bool,
    pub error_message: Option<String>,
    pub parameters: serde_json::Value,
}

/// Tool performance metrics
#[derive(Debug, Clone)]
pub struct ToolPerformanceMetrics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub success_rate: f64,
    pub recent_success_rate: f64,
    pub average_duration: Option<Duration>,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub last_execution: Option<DateTime<Utc>>,
    pub executions_per_hour: f64,
}

/// Tool execution statistics
#[derive(Debug, Clone)]
pub struct ToolExecutionStats {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub success_rate: f64,
    pub average_duration: Option<Duration>,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub last_execution: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
}

/// Tool execution status for display
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolStatus {
    Available,
    Running,
    Error,
    Disabled,
}

impl ToolStatus {
    pub fn symbol(&self) -> &str {
        match self {
            ToolStatus::Available => "○",
            ToolStatus::Running => "●",
            ToolStatus::Error => "✗",
            ToolStatus::Disabled => "⊘",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            ToolStatus::Available => Color::White,
            ToolStatus::Running => Color::Green,
            ToolStatus::Error => Color::Red,
            ToolStatus::Disabled => Color::DarkGray,
        }
    }
    
    pub fn description(&self) -> &str {
        match self {
            ToolStatus::Available => "可用",
            ToolStatus::Running => "运行中",
            ToolStatus::Error => "错误",
            ToolStatus::Disabled => "已禁用",
        }
    }
}

impl PartialOrd for ToolStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ToolStatus {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use ToolStatus::*;
        match (self, other) {
            (Running, Running) => std::cmp::Ordering::Equal,
            (Running, _) => std::cmp::Ordering::Less,
            (_, Running) => std::cmp::Ordering::Greater,
            (Error, Error) => std::cmp::Ordering::Equal,
            (Error, _) => std::cmp::Ordering::Less,
            (_, Error) => std::cmp::Ordering::Greater,
            (Available, Available) => std::cmp::Ordering::Equal,
            (Available, _) => std::cmp::Ordering::Less,
            (_, Available) => std::cmp::Ordering::Greater,
            (Disabled, Disabled) => std::cmp::Ordering::Equal,
        }
    }
}

/// Enhanced tool information for display in the TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDisplayInfo {
    pub info: ToolInfo,
    pub status: ToolStatus,
    pub last_execution: Option<DateTime<Utc>>,
    pub execution_count: u32,
    pub average_duration: Option<Duration>,
    pub success_rate: Option<f64>,
    pub source: String, // Plugin name or "Built-in"
}

impl ToolDisplayInfo {
    /// Create a new ToolDisplayInfo from a ToolInfo
    pub fn from_tool_info(info: ToolInfo) -> Self {
        let source = info.plugin_name.clone().unwrap_or_else(|| "Built-in".to_string());
        
        Self {
            info,
            status: ToolStatus::Available,
            last_execution: None,
            execution_count: 0,
            average_duration: None,
            success_rate: None,
            source,
        }
    }
    
    /// Check if the tool matches a filter string (legacy method)
    pub fn matches_filter(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        
        let filter_lower = filter.to_lowercase();
        
        // Support advanced filter syntax
        if filter.starts_with("status:") {
            let status_filter = filter[7..].trim().to_lowercase();
            return self.status.description().to_lowercase().contains(&status_filter);
        }
        
        if filter.starts_with("category:") {
            let category_filter = filter[9..].trim().to_lowercase();
            return self.info.category.as_ref()
                .map_or(false, |cat| cat.to_lowercase().contains(&category_filter));
        }
        
        if filter.starts_with("tag:") {
            let tag_filter = filter[4..].trim().to_lowercase();
            return self.info.tags.iter().any(|tag| tag.to_lowercase().contains(&tag_filter));
        }
        
        if filter.starts_with("source:") {
            let source_filter = filter[7..].trim().to_lowercase();
            return self.source.to_lowercase().contains(&source_filter);
        }
        
        if filter.starts_with("version:") {
            let version_filter = filter[8..].trim().to_lowercase();
            return self.info.version.to_lowercase().contains(&version_filter);
        }
        
        // Check name
        if self.info.name.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        // Check description
        if self.info.description.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        // Check category
        if let Some(category) = &self.info.category {
            if category.to_lowercase().contains(&filter_lower) {
                return true;
            }
        }
        
        // Check tags
        if self.info.tags.iter().any(|tag| tag.to_lowercase().contains(&filter_lower)) {
            return true;
        }
        
        // Check source
        if self.source.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        // Check version
        if self.info.version.to_lowercase().contains(&filter_lower) {
            return true;
        }
        
        false
    }
    
    /// Check if the tool matches a FilterSet (new advanced method)
    pub fn matches_filter_set(&self, filter_set: &FilterSet) -> bool {
        // Status filter
        if let Some(status_filter) = &filter_set.status_filter {
            if &self.status != status_filter {
                return false;
            }
        }
        
        // Category filter
        if let Some(category_filter) = &filter_set.category_filter {
            match &self.info.category {
                Some(category) => {
                    if !category.to_lowercase().contains(&category_filter.to_lowercase()) {
                        return false;
                    }
                }
                None => return false,
            }
        }
        
        // Source filter
        if let Some(source_filter) = &filter_set.source_filter {
            if !self.source.to_lowercase().contains(&source_filter.to_lowercase()) {
                return false;
            }
        }
        
        // Tag filters (all must match)
        for tag_filter in &filter_set.tag_filters {
            if !self.info.tags.iter().any(|tag| tag.to_lowercase().contains(&tag_filter.to_lowercase())) {
                return false;
            }
        }
        
        // Version filter
        if let Some(version_filter) = &filter_set.version_filter {
            if !self.info.version.to_lowercase().contains(&version_filter.to_lowercase()) {
                return false;
            }
        }
        
        // Execution count range
        if let Some((min, max)) = filter_set.execution_count_range {
            if self.execution_count < min || self.execution_count > max {
                return false;
            }
        }
        
        // Success rate range
        if let Some((min, max)) = filter_set.success_rate_range {
            match self.success_rate {
                Some(rate) => {
                    if rate < min || rate > max {
                        return false;
                    }
                }
                None => return false,
            }
        }
        
        // Text search (searches in name, description, tags)
        if !filter_set.text_search.is_empty() {
            let search_lower = filter_set.text_search.to_lowercase();
            let matches_text = self.info.name.to_lowercase().contains(&search_lower) ||
                self.info.description.to_lowercase().contains(&search_lower) ||
                self.info.tags.iter().any(|tag| tag.to_lowercase().contains(&search_lower)) ||
                self.source.to_lowercase().contains(&search_lower);
            
            if !matches_text {
                return false;
            }
        }
        
        true
    }
}

/// Tool Manager Widget for displaying and managing tools
pub struct ToolManagerWidget {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
    
    // Widget state
    tools: Vec<ToolDisplayInfo>,
    filtered_tools: Vec<usize>, // Indices into tools vec
    selected_index: usize,
    scroll_state: ScrollbarState,
    list_state: ListState,
    
    // Pagination and virtual scrolling
    page_size: usize,
    current_page: usize,
    scroll_offset: usize,
    visible_range: (usize, usize), // (start, end) indices for virtual scrolling
    
    // Filtering and sorting
    filter: String,
    sort_order: SortOrder,
    search_mode: bool,
    filter_history: Vec<String>, // Recent filter history
    saved_filters: std::collections::HashMap<String, String>, // Named saved filters
    active_filters: FilterSet, // Multi-dimensional filters
    
    // Display options
    show_details: bool,
    show_help: bool,
    show_status_indicators: bool,
    show_filter_panel: bool,
    
    // Performance tracking
    last_update: Option<std::time::Instant>,
    update_count: u64,
    
    // Tool execution tracking
    executing_tools: std::collections::HashSet<String>,
    tool_execution_history: std::collections::HashMap<String, Vec<ToolExecutionRecord>>,
}

impl ToolManagerWidget {
    /// Create a new ToolManagerWidget
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        let capabilities = WidgetCapabilities {
            keyboard_input: true,
            mouse_input: false,
            focusable: true,
            resizable: true,
            scrollable: true,
            themeable: true,
            configurable: true,
        };
        
        let size_constraints = SizeConstraints::new()
            .min_size(50, 10)
            .preferred_size(90, 30);
        
        Self {
            context: WidgetContext::new(WidgetId::from("tool_manager")),
            capabilities,
            size_constraints,
            tools: Vec::new(),
            filtered_tools: Vec::new(),
            selected_index: 0,
            scroll_state: ScrollbarState::default(),
            list_state,
            page_size: 20, // Default page size
            current_page: 0,
            scroll_offset: 0,
            visible_range: (0, 0),
            filter: String::new(),
            sort_order: SortOrder::NameAsc,
            search_mode: false,
            filter_history: Vec::new(),
            saved_filters: std::collections::HashMap::new(),
            active_filters: FilterSet::new(),
            show_details: false,
            show_help: false,
            show_status_indicators: true,
            show_filter_panel: false,
            last_update: None,
            update_count: 0,
            executing_tools: std::collections::HashSet::new(),
            tool_execution_history: std::collections::HashMap::new(),
        }
    }
    
    /// Set the tools to display
    pub fn set_tools(&mut self, tools: Vec<ToolDisplayInfo>) {
        self.tools = tools;
        self.apply_filter_and_sort();
        self.update_selection();
        self.last_update = Some(std::time::Instant::now());
        self.update_count += 1;
    }
    
    /// Add a tool to the list
    pub fn add_tool(&mut self, tool: ToolDisplayInfo) {
        self.tools.push(tool);
        self.apply_filter_and_sort();
        self.update_selection();
        self.update_count += 1;
    }
    
    /// Remove a tool by name
    pub fn remove_tool(&mut self, name: &str) -> bool {
        if let Some(pos) = self.tools.iter().position(|t| t.info.name == name) {
            self.tools.remove(pos);
            self.apply_filter_and_sort();
            self.update_selection();
            self.update_count += 1;
            true
        } else {
            false
        }
    }
    
    /// Update a tool's status
    pub fn update_tool_status(&mut self, name: &str, status: ToolStatus) -> bool {
        if let Some(tool) = self.tools.iter_mut().find(|t| t.info.name == name) {
            tool.status = status;
            self.apply_filter_and_sort();
            self.update_selection();
            self.update_count += 1;
            true
        } else {
            false
        }
    }
    
    /// Get the currently selected tool
    pub fn selected_tool(&self) -> Option<&ToolDisplayInfo> {
        self.filtered_tools
            .get(self.selected_index)
            .and_then(|&index| self.tools.get(index))
    }
    
    /// Get the currently selected tool name
    pub fn selected_tool_name(&self) -> Option<String> {
        self.selected_tool().map(|t| t.info.name.clone())
    }
    
    /// Set the filter string
    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Clear the filter
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Set the sort order
    pub fn set_sort_order(&mut self, sort_order: SortOrder) {
        self.sort_order = sort_order;
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Toggle details view
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
    
    /// Toggle help view
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
    }
    
    /// Exit search mode
    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
    }
    
    /// Move selection up
    pub fn select_previous(&mut self) {
        if !self.filtered_tools.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_tools.len() - 1
            } else {
                self.selected_index - 1
            };
            self.update_selection();
        }
    }
    
    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.filtered_tools.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_tools.len();
            self.update_selection();
        }
    }
    
    /// Move to first item
    pub fn select_first(&mut self) {
        if !self.filtered_tools.is_empty() {
            self.selected_index = 0;
            self.update_selection();
        }
    }
    
    /// Move to last item
    pub fn select_last(&mut self) {
        if !self.filtered_tools.is_empty() {
            self.selected_index = self.filtered_tools.len() - 1;
            self.update_selection();
        }
    }
    
    /// Move to next page
    pub fn next_page(&mut self) {
        let total_pages = self.total_pages();
        if self.current_page < total_pages.saturating_sub(1) {
            self.current_page += 1;
            self.selected_index = self.current_page * self.page_size;
            self.update_selection();
        }
    }
    
    /// Move to previous page
    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = self.current_page * self.page_size;
            self.update_selection();
        }
    }
    
    /// Get total number of pages
    pub fn total_pages(&self) -> usize {
        if self.page_size == 0 {
            1
        } else {
            (self.filtered_tools.len() + self.page_size - 1) / self.page_size
        }
    }
    
    /// Update visible range for virtual scrolling
    fn update_visible_range(&mut self, area_height: usize) {
        let visible_items = area_height.saturating_sub(2); // Account for borders
        let start = self.scroll_offset;
        let end = (start + visible_items).min(self.filtered_tools.len());
        self.visible_range = (start, end);
    }
    
    /// Scroll up by one item
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
    
    /// Scroll down by one item
    pub fn scroll_down(&mut self) {
        let max_scroll = self.filtered_tools.len().saturating_sub(1);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }
    
    /// Toggle status indicators display
    pub fn toggle_status_indicators(&mut self) {
        self.show_status_indicators = !self.show_status_indicators;
    }
    
    /// Mark a tool as executing
    pub fn mark_tool_executing(&mut self, tool_name: &str) {
        self.executing_tools.insert(tool_name.to_string());
        // Update tool status
        if let Some(tool) = self.tools.iter_mut().find(|t| t.info.name == tool_name) {
            tool.status = ToolStatus::Running;
        }
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Mark a tool as finished executing
    pub fn mark_tool_finished(&mut self, tool_name: &str, success: bool, error: Option<String>) {
        self.executing_tools.remove(tool_name);
        // Update tool status
        if let Some(tool) = self.tools.iter_mut().find(|t| t.info.name == tool_name) {
            tool.status = if success { ToolStatus::Available } else { ToolStatus::Error };
            tool.execution_count += 1;
            tool.last_execution = Some(Utc::now());
            
            // Update success rate
            let history = self.tool_execution_history.entry(tool_name.to_string()).or_insert_with(Vec::new);
            let successful_executions = history.iter().filter(|r| r.success).count() + if success { 1 } else { 0 };
            let total_executions = history.len() + 1;
            tool.success_rate = Some(successful_executions as f64 / total_executions as f64);
        }
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Add execution record to history
    pub fn add_execution_record(&mut self, tool_name: &str, record: ToolExecutionRecord) {
        let history = self.tool_execution_history.entry(tool_name.to_string()).or_insert_with(Vec::new);
        history.push(record);
        
        // Keep only last 100 records per tool
        if history.len() > 100 {
            history.remove(0);
        }
        
        // Update average duration
        if let Some(tool) = self.tools.iter_mut().find(|t| t.info.name == tool_name) {
            let durations: Vec<_> = history.iter()
                .filter_map(|r| r.duration)
                .collect();
            
            if !durations.is_empty() {
                let total_ms: u64 = durations.iter().map(|d| d.as_millis() as u64).sum();
                tool.average_duration = Some(Duration::from_millis(total_ms / durations.len() as u64));
            }
        }
    }
    
    /// Get execution history for a tool
    pub fn get_execution_history(&self, tool_name: &str) -> Option<&Vec<ToolExecutionRecord>> {
        self.tool_execution_history.get(tool_name)
    }
    
    /// Check if a tool is currently executing
    pub fn is_tool_executing(&self, tool_name: &str) -> bool {
        self.executing_tools.contains(tool_name)
    }
    
    /// Toggle filter panel display
    pub fn toggle_filter_panel(&mut self) {
        self.show_filter_panel = !self.show_filter_panel;
    }
    
    /// Save current filter with a name
    pub fn save_filter(&mut self, name: String) {
        let filter_string = if self.active_filters.is_empty() {
            self.filter.clone()
        } else {
            self.active_filters.to_display_string()
        };
        
        if !filter_string.is_empty() {
            self.saved_filters.insert(name, filter_string);
        }
    }
    
    /// Load a saved filter
    pub fn load_saved_filter(&mut self, name: &str) -> bool {
        if let Some(filter_string) = self.saved_filters.get(name) {
            self.parse_and_apply_filter(filter_string.clone());
            true
        } else {
            false
        }
    }
    
    /// Get list of saved filter names
    pub fn get_saved_filter_names(&self) -> Vec<String> {
        self.saved_filters.keys().cloned().collect()
    }
    
    /// Add filter to history
    fn add_to_filter_history(&mut self, filter: String) {
        if !filter.is_empty() && !self.filter_history.contains(&filter) {
            self.filter_history.push(filter);
            // Keep only last 20 filters
            if self.filter_history.len() > 20 {
                self.filter_history.remove(0);
            }
        }
    }
    
    /// Parse and apply a complex filter string
    pub fn parse_and_apply_filter(&mut self, filter_string: String) {
        self.add_to_filter_history(filter_string.clone());
        
        // Reset filters
        self.active_filters.clear();
        self.filter.clear();
        
        // Parse the filter string
        let parts: Vec<&str> = filter_string.split_whitespace().collect();
        let mut text_parts = Vec::new();
        
        for part in parts {
            if part.starts_with("status:") {
                let status_str = &part[7..];
                self.active_filters.status_filter = match status_str.to_lowercase().as_str() {
                    "available" | "可用" => Some(ToolStatus::Available),
                    "running" | "运行中" => Some(ToolStatus::Running),
                    "error" | "错误" => Some(ToolStatus::Error),
                    "disabled" | "已禁用" => Some(ToolStatus::Disabled),
                    _ => None,
                };
            } else if part.starts_with("category:") {
                self.active_filters.category_filter = Some(part[9..].to_string());
            } else if part.starts_with("source:") {
                self.active_filters.source_filter = Some(part[7..].to_string());
            } else if part.starts_with("tag:") {
                self.active_filters.tag_filters.push(part[4..].to_string());
            } else if part.starts_with("version:") {
                self.active_filters.version_filter = Some(part[8..].to_string());
            } else if part.starts_with("executions:") {
                if let Some(range_str) = part.strip_prefix("executions:") {
                    if let Some((min_str, max_str)) = range_str.split_once("..") {
                        if let (Ok(min), Ok(max)) = (min_str.parse::<u32>(), max_str.parse::<u32>()) {
                            self.active_filters.execution_count_range = Some((min, max));
                        }
                    }
                }
            } else if part.starts_with("success:") {
                if let Some(range_str) = part.strip_prefix("success:") {
                    if let Some((min_str, max_str)) = range_str.split_once("..") {
                        let min_str = min_str.trim_end_matches('%');
                        let max_str = max_str.trim_end_matches('%');
                        if let (Ok(min), Ok(max)) = (min_str.parse::<f64>(), max_str.parse::<f64>()) {
                            self.active_filters.success_rate_range = Some((min / 100.0, max / 100.0));
                        }
                    }
                }
            } else {
                // Regular text search
                text_parts.push(part);
            }
        }
        
        // Combine remaining parts as text search
        if !text_parts.is_empty() {
            self.active_filters.text_search = text_parts.join(" ");
        }
        
        // Update legacy filter for display
        self.filter = filter_string;
        
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Get available categories from current tools
    pub fn get_available_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.tools
            .iter()
            .filter_map(|tool| tool.info.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }
    
    /// Get available sources from current tools
    pub fn get_available_sources(&self) -> Vec<String> {
        let mut sources: Vec<String> = self.tools
            .iter()
            .map(|tool| tool.source.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        sources.sort();
        sources
    }
    
    /// Get available tags from current tools
    pub fn get_available_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.tools
            .iter()
            .flat_map(|tool| tool.info.tags.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        tags.sort();
        tags
    }
    
    /// Quick filter by category
    pub fn quick_filter_by_category(&mut self, category: String) {
        self.active_filters.clear();
        self.active_filters.category_filter = Some(category.clone());
        self.filter = format!("category:{}", category);
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Quick filter by source
    pub fn quick_filter_by_source(&mut self, source: String) {
        self.active_filters.clear();
        self.active_filters.source_filter = Some(source.clone());
        self.filter = format!("source:{}", source);
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Quick filter by status
    pub fn quick_filter_by_status(&mut self, status: ToolStatus) {
        self.active_filters.clear();
        self.active_filters.status_filter = Some(status.clone());
        self.filter = format!("status:{}", status.description());
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Get tool execution parameters from user input
    pub fn get_tool_execution_parameters(&self, tool: &ToolDisplayInfo) -> Option<serde_json::Value> {
        // In a real implementation, this would open a parameter input dialog
        // For now, return empty parameters or default values based on schema
        if tool.info.parameters_schema.is_null() {
            Some(serde_json::Value::Object(serde_json::Map::new()))
        } else {
            // Generate default parameters based on schema
            self.generate_default_parameters(&tool.info.parameters_schema)
        }
    }
    
    /// Generate default parameters based on schema
    fn generate_default_parameters(&self, schema: &serde_json::Value) -> Option<serde_json::Value> {
        if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
            let mut params = serde_json::Map::new();
            
            for (param_name, param_schema) in properties {
                let default_value = match param_schema.get("type").and_then(|v| v.as_str()) {
                    Some("string") => {
                        if let Some(default) = param_schema.get("default") {
                            default.clone()
                        } else {
                            serde_json::Value::String("".to_string())
                        }
                    }
                    Some("number") | Some("integer") => {
                        if let Some(default) = param_schema.get("default") {
                            default.clone()
                        } else {
                            serde_json::Value::Number(serde_json::Number::from(0))
                        }
                    }
                    Some("boolean") => {
                        if let Some(default) = param_schema.get("default") {
                            default.clone()
                        } else {
                            serde_json::Value::Bool(false)
                        }
                    }
                    Some("array") => serde_json::Value::Array(vec![]),
                    Some("object") => serde_json::Value::Object(serde_json::Map::new()),
                    _ => serde_json::Value::Null,
                };
                
                params.insert(param_name.clone(), default_value);
            }
            
            Some(serde_json::Value::Object(params))
        } else {
            Some(serde_json::Value::Object(serde_json::Map::new()))
        }
    }
    
    /// Start tool execution with parameters
    pub fn start_tool_execution(&mut self, tool_name: &str, parameters: serde_json::Value) -> Result<(), String> {
        // Mark tool as executing
        self.mark_tool_executing(tool_name);
        
        // Create execution record
        let execution_record = ToolExecutionRecord {
            started_at: Utc::now(),
            completed_at: None,
            duration: None,
            success: false,
            error_message: None,
            parameters,
        };
        
        // Add to history (will be updated when execution completes)
        let history = self.tool_execution_history.entry(tool_name.to_string()).or_insert_with(Vec::new);
        history.push(execution_record);
        
        // In a real implementation, this would trigger the actual tool execution
        // For now, we just simulate the start
        Ok(())
    }
    
    /// Complete tool execution with result
    pub fn complete_tool_execution(&mut self, tool_name: &str, success: bool, result: Option<serde_json::Value>, error: Option<String>) {
        let completion_time = Utc::now();
        
        // Update execution record
        if let Some(history) = self.tool_execution_history.get_mut(tool_name) {
            if let Some(record) = history.last_mut() {
                record.completed_at = Some(completion_time);
                record.duration = Some(completion_time.signed_duration_since(record.started_at).to_std().unwrap_or(Duration::from_secs(0)));
                record.success = success;
                record.error_message = error.clone();
            }
        }
        
        // Mark tool as finished
        self.mark_tool_finished(tool_name, success, error);
    }
    
    /// Retry failed tool execution
    pub fn retry_tool_execution(&mut self, tool_name: &str) -> Result<(), String> {
        // Get the last execution parameters
        if let Some(history) = self.tool_execution_history.get(tool_name) {
            if let Some(last_record) = history.last() {
                let parameters = last_record.parameters.clone();
                return self.start_tool_execution(tool_name, parameters);
            }
        }
        
        Err("No previous execution found to retry".to_string())
    }
    
    /// Get tool execution statistics
    pub fn get_tool_execution_stats(&self, tool_name: &str) -> Option<ToolExecutionStats> {
        if let Some(history) = self.tool_execution_history.get(tool_name) {
            if history.is_empty() {
                return None;
            }
            
            let total_executions = history.len();
            let successful_executions = history.iter().filter(|r| r.success).count();
            let failed_executions = total_executions - successful_executions;
            
            let success_rate = successful_executions as f64 / total_executions as f64;
            
            let durations: Vec<Duration> = history.iter()
                .filter_map(|r| r.duration)
                .collect();
            
            let average_duration = if !durations.is_empty() {
                let total_ms: u64 = durations.iter().map(|d| d.as_millis() as u64).sum();
                Some(Duration::from_millis(total_ms / durations.len() as u64))
            } else {
                None
            };
            
            let min_duration = durations.iter().min().cloned();
            let max_duration = durations.iter().max().cloned();
            
            let last_execution = history.last().map(|r| r.started_at);
            let last_success = history.iter().rev().find(|r| r.success).map(|r| r.started_at);
            let last_failure = history.iter().rev().find(|r| !r.success).map(|r| r.started_at);
            
            Some(ToolExecutionStats {
                total_executions,
                successful_executions,
                failed_executions,
                success_rate,
                average_duration,
                min_duration,
                max_duration,
                last_execution,
                last_success,
                last_failure,
            })
        } else {
            None
        }
    }
    
    /// Get recent execution errors for a tool
    pub fn get_recent_execution_errors(&self, tool_name: &str, limit: usize) -> Vec<&ToolExecutionRecord> {
        if let Some(history) = self.tool_execution_history.get(tool_name) {
            history.iter()
                .rev()
                .filter(|r| !r.success && r.error_message.is_some())
                .take(limit)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Clear execution history for a tool
    pub fn clear_tool_execution_history(&mut self, tool_name: &str) -> bool {
        if let Some(history) = self.tool_execution_history.get_mut(tool_name) {
            history.clear();
            
            // Reset tool statistics
            if let Some(tool) = self.tools.iter_mut().find(|t| t.info.name == tool_name) {
                tool.execution_count = 0;
                tool.last_execution = None;
                tool.average_duration = None;
                tool.success_rate = None;
            }
            
            true
        } else {
            false
        }
    }
    
    /// Export execution history for a tool
    pub fn export_tool_execution_history(&self, tool_name: &str) -> Option<String> {
        if let Some(history) = self.tool_execution_history.get(tool_name) {
            match serde_json::to_string_pretty(history) {
                Ok(json) => Some(json),
                Err(_) => None,
            }
        } else {
            None
        }
    }
    
    /// Apply current filter and sort order
    fn apply_filter_and_sort(&mut self) {
        // Apply filter
        self.filtered_tools = self.tools
            .iter()
            .enumerate()
            .filter(|(_, tool)| {
                // Use new FilterSet if it has any filters, otherwise use legacy filter
                if !self.active_filters.is_empty() {
                    tool.matches_filter_set(&self.active_filters)
                } else {
                    tool.matches_filter(&self.filter)
                }
            })
            .map(|(i, _)| i)
            .collect();
        
        // Apply sort
        self.filtered_tools.sort_by(|&a, &b| {
            let tool_a = &self.tools[a];
            let tool_b = &self.tools[b];
            
            match self.sort_order {
                SortOrder::NameAsc => tool_a.info.name.cmp(&tool_b.info.name),
                SortOrder::NameDesc => tool_b.info.name.cmp(&tool_a.info.name),
                SortOrder::DateAsc => tool_a.last_execution.cmp(&tool_b.last_execution),
                SortOrder::DateDesc => tool_b.last_execution.cmp(&tool_a.last_execution),
                SortOrder::StatusAsc => tool_a.status.cmp(&tool_b.status),
                SortOrder::StatusDesc => tool_b.status.cmp(&tool_a.status),
                SortOrder::Custom(ref custom) => {
                    match custom.as_str() {
                        "category_asc" => tool_a.info.category.cmp(&tool_b.info.category),
                        "category_desc" => tool_b.info.category.cmp(&tool_a.info.category),
                        "source_asc" => tool_a.source.cmp(&tool_b.source),
                        "source_desc" => tool_b.source.cmp(&tool_a.source),
                        "execution_count_asc" => tool_a.execution_count.cmp(&tool_b.execution_count),
                        "execution_count_desc" => tool_b.execution_count.cmp(&tool_a.execution_count),
                        "success_rate_asc" => tool_a.success_rate.partial_cmp(&tool_b.success_rate).unwrap_or(std::cmp::Ordering::Equal),
                        "success_rate_desc" => tool_b.success_rate.partial_cmp(&tool_a.success_rate).unwrap_or(std::cmp::Ordering::Equal),
                        "version_asc" => tool_a.info.version.cmp(&tool_b.info.version),
                        "version_desc" => tool_b.info.version.cmp(&tool_a.info.version),
                        _ => std::cmp::Ordering::Equal,
                    }
                }
            }
        });
    }
    
    /// Update selection state
    fn update_selection(&mut self) {
        if self.selected_index >= self.filtered_tools.len() && !self.filtered_tools.is_empty() {
            self.selected_index = self.filtered_tools.len() - 1;
        }
        
        // Update current page based on selection
        if self.page_size > 0 {
            self.current_page = self.selected_index / self.page_size;
        }
        
        // Update scroll offset to keep selection visible
        let visible_items = 20; // Default visible items, will be updated in render
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_index.saturating_sub(visible_items - 1);
        }
        
        self.list_state.select(if self.filtered_tools.is_empty() {
            None
        } else {
            Some(self.selected_index.saturating_sub(self.scroll_offset))
        });
        
        // Update scrollbar state
        self.scroll_state = self.scroll_state.content_length(self.filtered_tools.len());
        self.scroll_state = self.scroll_state.position(self.selected_index);
    }
}

#[async_trait]
impl Widget for ToolManagerWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
    }
    
    fn title(&self) -> &str {
        "工具管理器"
    }
    
    fn description(&self) -> Option<&str> {
        Some("显示和管理可用的工具")
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
        UpdateFrequency::Interval(Duration::from_secs(10)) // Update every 10 seconds
    }
    
    async fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) -> std::result::Result<(), WidgetError> {
        if self.show_help {
            // Show help overlay
            let help_area = Rect {
                x: area.x + area.width / 4,
                y: area.y + area.height / 4,
                width: area.width / 2,
                height: area.height / 2,
            };
            
            // Clear the help area
            frame.render_widget(Clear, help_area);
            self.render_help(frame, help_area, theme);
            return Ok(());
        }
        
        if self.show_filter_panel {
            // Show filter panel overlay
            let filter_area = Rect {
                x: area.x + area.width / 6,
                y: area.y + area.height / 6,
                width: area.width * 2 / 3,
                height: area.height * 2 / 3,
            };
            
            // Clear the filter area
            frame.render_widget(Clear, filter_area);
            // TODO: Implement render_filter_panel method
            // self.render_filter_panel(frame, filter_area, theme);
            return Ok(());
        }
        
        if self.show_details {
            // Split view: list on left, details on right
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);
            
            self.render_tool_list(frame, chunks[0], theme);
            self.render_tool_details(frame, chunks[1], theme);
        } else {
            // Full width list
            self.render_tool_list(frame, area, theme);
        }
        
        Ok(())
    }
    
    async fn handle_event(&mut self, event: Event) -> std::result::Result<Option<Action>, WidgetError> {
        if let Event::Key(key) = event {
            return self.handle_key_event(key).await;
        }
        Ok(None)
    }
    
    async fn update(&mut self) -> std::result::Result<(), WidgetError> {
        self.context.mark_updated();
        // In a real implementation, this would fetch updated tool data
        // For now, we just track that an update occurred
        self.update_count += 1;
        Ok(())
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("↑/k", "上一个工具"),
            ("↓/j", "下一个工具"),
            ("Home/g", "第一个工具"),
            ("End/G", "最后一个工具"),
            ("PgUp", "上一页"),
            ("PgDn", "下一页"),
            ("Ctrl+U", "向上滚动"),
            ("Ctrl+D", "向下滚动"),
            ("Enter", "执行工具"),
            ("R", "重试上次执行"),
            ("Space", "切换详情"),
            ("h", "显示执行历史"),
            ("C", "清除执行历史"),
            ("E", "导出执行历史"),
            ("/", "搜索/过滤"),
            ("f", "快速过滤"),
            ("F", "过滤器面板"),
            ("A", "高级过滤"),
            ("s", "循环排序"),
            ("c", "清除过滤"),
            ("i", "切换状态指示器"),
            ("1-3", "快速状态过滤"),
            ("r", "刷新"),
            ("?", "帮助"),
            ("Esc", "返回"),
            ("Ctrl+Q", "退出"),
        ]
    }
}

impl Default for ToolManagerWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManagerWidget {
    /// Render the tool list
    fn render_tool_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Update visible range for virtual scrolling
        self.update_visible_range(area.height as usize);
        
        // Collect the filter and other needed values before creating the closure
        let filter = self.filter.clone();
        let search_mode = self.search_mode;
        let show_status_indicators = self.show_status_indicators;
        
        // Get visible items for virtual scrolling
        let visible_start = self.scroll_offset;
        let visible_end = (visible_start + area.height as usize - 2).min(self.filtered_tools.len());
        let visible_indices = if visible_start < self.filtered_tools.len() {
            &self.filtered_tools[visible_start..visible_end]
        } else {
            &[]
        };
        
        let items: Vec<ListItem> = visible_indices
            .iter()
            .enumerate()
            .map(|(display_index, &tool_index)| {
                let tool = &self.tools[tool_index];
                
                let status_style = Style::default().fg(tool.status.color());
                
                let last_exec = tool.last_execution
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "从未执行".to_string());
                
                // Highlight search terms in the tool name
                let name_spans = if !filter.is_empty() && !filter.starts_with("status:") && !filter.starts_with("category:") && !filter.starts_with("tag:") && !filter.starts_with("source:") && !filter.starts_with("version:") {
                    Self::highlight_text_static(&tool.info.name, &filter, Style::default().add_modifier(Modifier::BOLD))
                } else {
                    vec![Span::styled(&tool.info.name, Style::default().add_modifier(Modifier::BOLD))]
                };
                
                let mut first_line = vec![];
                
                // Add status indicator if enabled
                if show_status_indicators {
                    first_line.push(Span::styled(tool.status.symbol(), status_style));
                    first_line.push(Span::raw(" "));
                }
                
                first_line.extend(name_spans);
                first_line.extend(vec![
                    Span::raw(" v"),
                    Span::styled(&tool.info.version, Style::default().fg(Color::Gray)),
                    Span::raw(" ("),
                    Span::styled(&tool.source, Style::default().fg(Color::Cyan)),
                    Span::raw(")"),
                ]);
                
                // Add execution indicator for running tools
                if self.executing_tools.contains(&tool.info.name) {
                    first_line.push(Span::raw(" "));
                    first_line.push(Span::styled("⚡", Style::default().fg(Color::Yellow)));
                }
                
                let second_line = vec![
                    Span::raw("  "),
                    Span::styled(&tool.info.description, Style::default().fg(Color::Gray)),
                ];
                
                let mut third_line = vec![
                    Span::raw("  最后执行: "),
                    Span::styled(last_exec, Style::default().fg(Color::Yellow)),
                ];
                
                if tool.execution_count > 0 {
                    third_line.push(Span::styled(format!(" | 执行次数: {}", tool.execution_count), Style::default().fg(Color::Magenta)));
                }
                
                if let Some(rate) = tool.success_rate {
                    let rate_color = if rate >= 0.9 { Color::Green } else if rate >= 0.7 { Color::Yellow } else { Color::Red };
                    third_line.push(Span::styled(format!(" | 成功率: {:.1}%", rate * 100.0), Style::default().fg(rate_color)));
                }
                
                if let Some(duration) = tool.average_duration {
                    third_line.push(Span::styled(format!(" | 平均耗时: {}ms", duration.as_millis()), Style::default().fg(Color::Blue)));
                }
                
                let content = vec![
                    Line::from(first_line),
                    Line::from(second_line),
                    Line::from(third_line),
                ];
                
                ListItem::new(content)
            })
            .collect();
        
        let title = if search_mode {
            format!("工具管理器 - 搜索: {}", filter)
        } else if !filter.is_empty() {
            format!("工具管理器 - 过滤: {} ({}/{})", filter, self.filtered_tools.len(), self.tools.len())
        } else {
            format!("工具管理器 ({} 工具) - 页面 {}/{}", self.tools.len(), self.current_page + 1, self.total_pages().max(1))
        };
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if self.context.has_focus {
                    theme.styles.widget_border_focused
                } else {
                    theme.styles.widget_border
                }))
            .highlight_style(theme.styles.list_item_selected)
            .highlight_symbol("► ");
        
        frame.render_stateful_widget(list, area, &mut self.list_state);
        
        // Render scrollbar if needed
        if self.filtered_tools.len() > area.height as usize - 2 {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            
            let scrollbar_area = Rect {
                x: area.right() - 1,
                y: area.y + 1,
                width: 1,
                height: area.height - 2,
            };
            
            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut self.scroll_state);
        }
        
        // Render pagination info if multiple pages
        if self.total_pages() > 1 {
            self.render_pagination_info(frame, area, theme);
        }
        
        // Render search/filter status
        if search_mode || !filter.is_empty() {
            self.render_search_status(frame, area, theme);
        }
    }
    
    /// Render search/filter status
    fn render_search_status(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let status_text = if self.search_mode {
            format!("搜索: {} (按 Enter 确认, Esc 取消)", self.filter)
        } else {
            format!("过滤: {} ({} 结果)", self.filter, self.filtered_tools.len())
        };
        
        let status_area = Rect {
            x: area.x + 2,
            y: area.bottom().saturating_sub(1),
            width: area.width.saturating_sub(4),
            height: 1,
        };
        
        let status = Paragraph::new(status_text)
            .style(if self.search_mode {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            });
        
        frame.render_widget(status, status_area);
    }
    
    /// Render pagination information
    fn render_pagination_info(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let pagination_text = format!(
            "页面 {}/{} | 显示 {}-{} / {} 工具 | PgUp/PgDn 翻页",
            self.current_page + 1,
            self.total_pages().max(1),
            self.scroll_offset + 1,
            (self.scroll_offset + area.height as usize - 2).min(self.filtered_tools.len()),
            self.filtered_tools.len()
        );
        
        let pagination_area = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(4),
            height: 1,
        };
        
        let pagination = Paragraph::new(pagination_text)
            .style(Style::default().fg(Color::DarkGray));
        
        frame.render_widget(pagination, pagination_area);
    }
    
    /// Render tool details panel
    fn render_tool_details(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(tool) = self.selected_tool() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(10), // Basic info
                    Constraint::Length(8),  // Parameters schema
                    Constraint::Length(8),  // Execution statistics and history
                    Constraint::Min(0),     // Documentation and examples
                ])
                .split(area);
            
            // Basic information
            self.render_tool_info(frame, chunks[0], tool, theme);
            
            // Parameters schema
            self.render_tool_parameters(frame, chunks[1], tool, theme);
            
            // Execution statistics and history
            self.render_tool_execution_info(frame, chunks[2], tool, theme);
            
            // Documentation and examples
            self.render_tool_documentation(frame, chunks[3], tool, theme);
        } else {
            let no_selection = Paragraph::new("未选择工具")
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("工具详情")
                    .border_style(theme.styles.widget_border))
                .style(theme.styles.info);
            
            frame.render_widget(no_selection, area);
        }
    }
    
    /// Render tool parameters schema
    fn render_tool_parameters(&self, frame: &mut Frame, area: Rect, tool: &ToolDisplayInfo, theme: &Theme) {
        let mut param_lines = vec![
            Line::from(vec![
                Span::styled("参数配置:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
        ];
        
        if tool.info.parameters_schema.is_null() {
            param_lines.push(Line::from("  无参数"));
        } else {
            if let Some(properties) = tool.info.parameters_schema.get("properties").and_then(|v| v.as_object()) {
                for (param_name, param_schema) in properties.iter().take(5) { // Show first 5 parameters
                    let param_type = param_schema.get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    
                    let required = tool.info.parameters_schema
                        .get("required")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().any(|v| v.as_str() == Some(param_name)))
                        .unwrap_or(false);
                    
                    let description = param_schema.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("无描述");
                    
                    let default_value = param_schema.get("default")
                        .map(|v| format!(" (默认: {})", v))
                        .unwrap_or_default();
                    
                    param_lines.push(Line::from(vec![
                        Span::raw("  • "),
                        Span::styled(param_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        if required {
                            Span::styled(" *", Style::default().fg(Color::Red))
                        } else {
                            Span::raw("")
                        },
                        Span::raw(" ("),
                        Span::styled(param_type, Style::default().fg(Color::Yellow)),
                        Span::raw(")"),
                        Span::styled(default_value, Style::default().fg(Color::Gray)),
                    ]));
                    
                    if description.len() > 50 {
                        param_lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(&description[..47], Style::default().fg(Color::Gray)),
                            Span::styled("...", Style::default().fg(Color::Gray)),
                        ]));
                    } else {
                        param_lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(description, Style::default().fg(Color::Gray)),
                        ]));
                    }
                }
                
                if properties.len() > 5 {
                    param_lines.push(Line::from(format!("  ... 还有 {} 个参数", properties.len() - 5)));
                }
            } else {
                param_lines.push(Line::from("  参数配置格式错误"));
            }
        }
        
        // Show execution controls
        param_lines.push(Line::from(""));
        param_lines.push(Line::from(vec![
            Span::styled("执行控制:", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        
        if self.is_tool_executing(&tool.info.name) {
            param_lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("⚡ 正在执行中...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            param_lines.push(Line::from("  按 Ctrl+C 取消执行"));
        } else {
            param_lines.push(Line::from("  按 Enter 执行工具"));
            if self.get_execution_history(&tool.info.name).is_some() {
                param_lines.push(Line::from("  按 R 重试上次执行"));
            }
        }
        
        let params_paragraph = Paragraph::new(param_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("参数和执行")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(params_paragraph, area);
    }
    
    /// Render tool basic information
    fn render_tool_info(&self, frame: &mut Frame, area: Rect, tool: &ToolDisplayInfo, theme: &Theme) {
        let info_lines = vec![
            Line::from(vec![
                Span::raw("名称: "),
                Span::styled(&tool.info.name, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("版本: "),
                Span::styled(&tool.info.version, Style::default().fg(Color::Cyan)),
                Span::raw(" | 创建时间: "),
                Span::styled(tool.info.created_at.format("%Y-%m-%d").to_string(), Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::raw("状态: "),
                Span::styled(tool.status.symbol(), Style::default().fg(tool.status.color())),
                Span::raw(" "),
                Span::styled(tool.status.description(), Style::default().fg(tool.status.color())),
                if self.executing_tools.contains(&tool.info.name) {
                    Span::styled(" (正在执行)", Style::default().fg(Color::Yellow))
                } else {
                    Span::raw("")
                },
            ]),
            Line::from(vec![
                Span::raw("来源: "),
                Span::styled(&tool.source, Style::default().fg(Color::Magenta)),
                Span::raw(" | 分类: "),
                Span::styled(
                    tool.info.category.as_deref().unwrap_or("未分类"), 
                    Style::default().fg(Color::Yellow)
                ),
            ]),
            Line::from(vec![
                Span::raw("标签: "),
                Span::styled(
                    if tool.info.tags.is_empty() {
                        "无".to_string()
                    } else {
                        tool.info.tags.join(", ")
                    },
                    Style::default().fg(Color::Green)
                ),
            ]),
            Line::from(vec![
                Span::raw("更新时间: "),
                Span::styled(tool.info.updated_at.format("%Y-%m-%d %H:%M").to_string(), Style::default().fg(Color::Gray)),
            ]),
            // Performance metrics
            if tool.execution_count > 0 {
                Line::from(vec![
                    Span::raw("执行统计: "),
                    Span::styled(format!("{}次", tool.execution_count), Style::default().fg(Color::Magenta)),
                    if let Some(rate) = tool.success_rate {
                        let rate_color = if rate >= 0.9 { Color::Green } else if rate >= 0.7 { Color::Yellow } else { Color::Red };
                        Span::styled(format!(" | 成功率: {:.1}%", rate * 100.0), Style::default().fg(rate_color))
                    } else {
                        Span::raw("")
                    },
                    if let Some(duration) = tool.average_duration {
                        Span::styled(format!(" | 平均耗时: {}ms", duration.as_millis()), Style::default().fg(Color::Blue))
                    } else {
                        Span::raw("")
                    },
                ])
            } else {
                Line::from(vec![
                    Span::raw("执行统计: "),
                    Span::styled("无执行记录", Style::default().fg(Color::Gray)),
                ])
            },
        ];
        
        let info_paragraph = Paragraph::new(info_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("基本信息")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(info_paragraph, area);
    }
    
    /// Render tool execution statistics and history
    fn render_tool_execution_info(&self, frame: &mut Frame, area: Rect, tool: &ToolDisplayInfo, theme: &Theme) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("执行统计:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
        ];
        
        if let Some(stats) = self.get_tool_execution_stats(&tool.info.name) {
            lines.push(Line::from(vec![
                Span::raw("  总执行次数: "),
                Span::styled(stats.total_executions.to_string(), Style::default().fg(Color::Cyan)),
                Span::raw(" | 成功: "),
                Span::styled(stats.successful_executions.to_string(), Style::default().fg(Color::Green)),
                Span::raw(" | 失败: "),
                Span::styled(stats.failed_executions.to_string(), Style::default().fg(Color::Red)),
            ]));
            
            let success_rate_color = if stats.success_rate >= 0.9 { 
                Color::Green 
            } else if stats.success_rate >= 0.7 { 
                Color::Yellow 
            } else { 
                Color::Red 
            };
            
            lines.push(Line::from(vec![
                Span::raw("  成功率: "),
                Span::styled(format!("{:.1}%", stats.success_rate * 100.0), Style::default().fg(success_rate_color)),
            ]));
            
            if let Some(avg_duration) = stats.average_duration {
                lines.push(Line::from(vec![
                    Span::raw("  平均耗时: "),
                    Span::styled(format!("{}ms", avg_duration.as_millis()), Style::default().fg(Color::Blue)),
                ]));
                
                if let (Some(min), Some(max)) = (stats.min_duration, stats.max_duration) {
                    lines.push(Line::from(vec![
                        Span::raw("  耗时范围: "),
                        Span::styled(format!("{}ms", min.as_millis()), Style::default().fg(Color::Gray)),
                        Span::raw(" - "),
                        Span::styled(format!("{}ms", max.as_millis()), Style::default().fg(Color::Gray)),
                    ]));
                }
            }
            
            if let Some(last_exec) = stats.last_execution {
                lines.push(Line::from(vec![
                    Span::raw("  最后执行: "),
                    Span::styled(last_exec.format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().fg(Color::Yellow)),
                ]));
            }
            
            // Show recent errors if any
            let recent_errors = self.get_recent_execution_errors(&tool.info.name, 2);
            if !recent_errors.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("最近错误:", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
                ]));
                
                for (i, error_record) in recent_errors.iter().enumerate() {
                    if i >= 2 { break; } // Limit to 2 recent errors
                    
                    let error_msg = error_record.error_message.as_deref().unwrap_or("未知错误");
                    let error_time = error_record.started_at.format("%m-%d %H:%M").to_string();
                    
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(error_time, Style::default().fg(Color::Gray)),
                        Span::raw(": "),
                        Span::styled(
                            if error_msg.len() > 40 {
                                format!("{}...", &error_msg[..37])
                            } else {
                                error_msg.to_string()
                            },
                            Style::default().fg(Color::Red)
                        ),
                    ]));
                }
            }
        } else {
            lines.push(Line::from("  无执行记录"));
        }
        
        // Show current execution status
        if self.is_tool_executing(&tool.info.name) {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("⚡ 正在执行中...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        }
        
        let execution_info = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("执行信息")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(execution_info, area);
    }
    
    /// Render tool dependencies and version info
    fn render_tool_dependencies(&self, frame: &mut Frame, area: Rect, tool: &ToolDisplayInfo, theme: &Theme) {
        let mut dep_lines = vec![
            Line::from(vec![
                Span::styled("依赖关系:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
        ];
        
        if tool.info.dependencies.is_empty() {
            dep_lines.push(Line::from("  无依赖"));
        } else {
            for dep in &tool.info.dependencies {
                let version_req = tool.info.version_requirements.get(dep)
                    .map(|v| format!(" ({})", v))
                    .unwrap_or_default();
                
                dep_lines.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(dep, Style::default().fg(Color::Cyan)),
                    Span::styled(version_req, Style::default().fg(Color::Gray)),
                ]));
            }
        }
        
        dep_lines.push(Line::from(""));
        dep_lines.push(Line::from(vec![
            Span::styled("版本信息:", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        
        if let Some(history) = self.get_execution_history(&tool.info.name) {
            let recent_executions = history.len().min(5);
            dep_lines.push(Line::from(vec![
                Span::raw("  最近执行: "),
                Span::styled(format!("{} 次", recent_executions), Style::default().fg(Color::Yellow)),
            ]));
            
            if let Some(last_execution) = history.last() {
                let status_text = if last_execution.success { "成功" } else { "失败" };
                let status_color = if last_execution.success { Color::Green } else { Color::Red };
                
                dep_lines.push(Line::from(vec![
                    Span::raw("  上次结果: "),
                    Span::styled(status_text, Style::default().fg(status_color)),
                    if let Some(duration) = last_execution.duration {
                        Span::styled(format!(" ({}ms)", duration.as_millis()), Style::default().fg(Color::Blue))
                    } else {
                        Span::raw("")
                    },
                ]));
            }
        } else {
            dep_lines.push(Line::from("  无执行历史"));
        }
        
        let deps_paragraph = Paragraph::new(dep_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("依赖和版本")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(deps_paragraph, area);
    }
    /// Render tool documentation
    fn render_tool_documentation(&self, frame: &mut Frame, area: Rect, tool: &ToolDisplayInfo, theme: &Theme) {
        let doc_lines = vec![
            Line::from(vec![
                Span::styled("描述:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(tool.info.description.clone()),
            Line::from(""),
            Line::from(vec![
                Span::styled("使用示例:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from("基本用法:"),
            Line::from(format!("  {}", tool.info.name)),
            Line::from(""),
            if !tool.info.parameters_schema.is_null() {
                Line::from("参数示例:")
            } else {
                Line::from("")
            },
            if !tool.info.parameters_schema.is_null() {
                if let Some(properties) = tool.info.parameters_schema.get("properties").and_then(|v| v.as_object()) {
                    let example_params: Vec<String> = properties.iter()
                        .take(2) // Show first 2 parameters as example
                        .map(|(name, schema)| {
                            let example_value = match schema.get("type").and_then(|v| v.as_str()) {
                                Some("string") => "\"example\"".to_string(),
                                Some("number") | Some("integer") => "42".to_string(),
                                Some("boolean") => "true".to_string(),
                                Some("array") => "[]".to_string(),
                                Some("object") => "{}".to_string(),
                                _ => "null".to_string(),
                            };
                            format!("    \"{}\": {}", name, example_value)
                        })
                        .collect();
                    
                    if !example_params.is_empty() {
                        Line::from(format!("  {{\n{}\n  }}", example_params.join(",\n")))
                    } else {
                        Line::from("  {}")
                    }
                } else {
                    Line::from("  {}")
                }
            } else {
                Line::from("")
            },
            Line::from(""),
            Line::from(vec![
                Span::styled("最佳实践:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from("• 确保提供所有必需参数"),
            Line::from("• 检查参数类型和格式"),
            Line::from("• 处理可能的错误返回值"),
            if tool.execution_count > 0 {
                Line::from("• 参考执行历史优化参数")
            } else {
                Line::from("")
            },
        ];
        
        let doc_paragraph = Paragraph::new(doc_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("文档和示例")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(doc_paragraph, area);
    }
    
    /// Render help overlay
    fn render_help(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let help_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("工具管理器 - 帮助", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from("导航:"),
            Line::from("  ↑/k        上一个工具"),
            Line::from("  ↓/j        下一个工具"),
            Line::from("  Home/g     第一个工具"),
            Line::from("  End/G      最后一个工具"),
            Line::from("  PgUp       上一页"),
            Line::from("  PgDn       下一页"),
            Line::from("  Ctrl+U     向上滚动"),
            Line::from("  Ctrl+D     向下滚动"),
            Line::from(""),
            Line::from("工具执行:"),
            Line::from("  Enter      执行选中的工具"),
            Line::from("  R          重试上次失败的执行"),
            Line::from("  h          显示执行历史"),
            Line::from("  C          清除执行历史"),
            Line::from("  E          导出执行历史"),
            Line::from(""),
            Line::from("显示操作:"),
            Line::from("  Space      切换详情面板"),
            Line::from("  r/F5       刷新工具列表"),
            Line::from("  i          切换状态指示器"),
            Line::from(""),
            Line::from("过滤和搜索:"),
            Line::from("  /          进入搜索模式"),
            Line::from("  f          快速过滤"),
            Line::from("  c          清除过滤"),
            Line::from("  s          循环排序方式"),
            Line::from("  F          过滤器面板"),
            Line::from("  A          高级过滤模式"),
            Line::from(""),
            Line::from("高级过滤语法:"),
            Line::from("  status:available    按状态过滤"),
            Line::from("  category:data       按分类过滤"),
            Line::from("  tag:utility         按标签过滤"),
            Line::from("  source:plugin       按来源过滤"),
            Line::from("  version:1.0         按版本过滤"),
            Line::from("  executions:10..50   按执行次数过滤"),
            Line::from("  success:80%..100%   按成功率过滤"),
            Line::from(""),
            Line::from("快速过滤:"),
            Line::from("  1          可用工具"),
            Line::from("  2          运行中工具"),
            Line::from("  3          错误工具"),
            Line::from(""),
            Line::from("状态指示器:"),
            Line::from("  ○          可用"),
            Line::from("  ●          运行中"),
            Line::from("  ✗          错误"),
            Line::from("  ⊘          已禁用"),
            Line::from("  ⚡          正在执行"),
            Line::from(""),
            Line::from("其他:"),
            Line::from("  ?          显示/隐藏帮助"),
            Line::from("  Esc        返回"),
            Line::from("  Ctrl+Q     退出"),
            Line::from(""),
            Line::from("按任意键关闭帮助"),
        ];
        
        let help = Paragraph::new(help_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("帮助")
                .border_style(Style::default().fg(Color::Yellow))
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(help, area);
    }
    
    /// Render filter panel overlay
    fn render_filter_panel(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(8),  // Active filters
                Constraint::Length(6),  // Quick filters
                Constraint::Length(6),  // Saved filters
                Constraint::Min(0),     // Filter history
            ])
            .split(area);
        
        // Title
        let title = Paragraph::new("高级过滤器")
            .block(Block::default()
                .borders(Borders::ALL)
                .title("过滤器面板")
                .border_style(Style::default().fg(Color::Yellow))
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        frame.render_widget(title, chunks[0]);
        
        // Active filters
        self.render_active_filters(frame, chunks[1], theme);
        
        // Quick filters
        self.render_quick_filters(frame, chunks[2], theme);
        
        // Saved filters
        self.render_saved_filters(frame, chunks[3], theme);
        
        // Filter history
        self.render_filter_history(frame, chunks[4], theme);
    }
    
    /// Render active filters section
    fn render_active_filters(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut lines = vec![
            Line::from("当前活动过滤器:"),
        ];
        
        if self.active_filters.is_empty() && self.filter.is_empty() {
            lines.push(Line::from("  无活动过滤器"));
        } else {
            if let Some(status) = &self.active_filters.status_filter {
                lines.push(Line::from(vec![
                    Span::raw("  状态: "),
                    Span::styled(status.description(), Style::default().fg(Color::Cyan)),
                ]));
            }
            
            if let Some(category) = &self.active_filters.category_filter {
                lines.push(Line::from(vec![
                    Span::raw("  分类: "),
                    Span::styled(category, Style::default().fg(Color::Green)),
                ]));
            }
            
            if let Some(source) = &self.active_filters.source_filter {
                lines.push(Line::from(vec![
                    Span::raw("  来源: "),
                    Span::styled(source, Style::default().fg(Color::Magenta)),
                ]));
            }
            
            if !self.active_filters.tag_filters.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("  标签: "),
                    Span::styled(self.active_filters.tag_filters.join(", "), Style::default().fg(Color::Yellow)),
                ]));
            }
            
            if !self.active_filters.text_search.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("  文本: "),
                    Span::styled(&self.active_filters.text_search, Style::default().fg(Color::White)),
                ]));
            }
        }
        
        let active_filters = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("活动过滤器")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info);
        
        frame.render_widget(active_filters, area);
    }
    
    /// Render quick filters section
    fn render_quick_filters(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let lines = vec![
            Line::from("快速过滤器 (按数字键选择):"),
            Line::from("  1. 可用工具    2. 运行中工具    3. 错误工具"),
            Line::from("  4. 按分类      5. 按来源        6. 按标签"),
            Line::from("  7. 高执行次数  8. 高成功率      9. 最近更新"),
        ];
        
        let quick_filters = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("快速过滤")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info);
        
        frame.render_widget(quick_filters, area);
    }
    
    /// Render saved filters section
    fn render_saved_filters(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut lines = vec![
            Line::from("已保存的过滤器:"),
        ];
        
        if self.saved_filters.is_empty() {
            lines.push(Line::from("  无已保存的过滤器"));
        } else {
            for (name, filter) in self.saved_filters.iter().take(3) {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(": "),
                    Span::styled(filter, Style::default().fg(Color::Gray)),
                ]));
            }
            
            if self.saved_filters.len() > 3 {
                lines.push(Line::from(format!("  ... 还有 {} 个", self.saved_filters.len() - 3)));
            }
        }
        
        let saved_filters = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("已保存过滤器")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info);
        
        frame.render_widget(saved_filters, area);
    }
    
    /// Render filter history section
    fn render_filter_history(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut lines = vec![
            Line::from("过滤器历史:"),
        ];
        
        if self.filter_history.is_empty() {
            lines.push(Line::from("  无历史记录"));
        } else {
            for (i, filter) in self.filter_history.iter().rev().take(5).enumerate() {
                lines.push(Line::from(vec![
                    Span::raw(format!("  {}. ", i + 1)),
                    Span::styled(filter, Style::default().fg(Color::Gray)),
                ]));
            }
        }
        
        let filter_history = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("历史记录")
                .border_style(theme.styles.widget_border))
            .style(theme.styles.info)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(filter_history, area);
    }
    
    /// Handle key events
    async fn handle_key_event(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        if self.show_help {
            // Any key closes help
            self.show_help = false;
            return Ok(None);
        }
        
        if self.search_mode {
            return self.handle_search_key_event(key).await;
        }
        
        match key.code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                Ok(None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Ok(None)
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.select_first();
                Ok(None)
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.select_last();
                Ok(None)
            }
            
            // Pagination
            KeyCode::PageUp => {
                self.previous_page();
                Ok(None)
            }
            KeyCode::PageDown => {
                self.next_page();
                Ok(None)
            }
            
            // Scrolling
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                for _ in 0..5 {
                    self.scroll_up();
                }
                Ok(None)
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                for _ in 0..5 {
                    self.scroll_down();
                }
                Ok(None)
            }
            
            // Actions
            KeyCode::Enter => {
                if let Some(tool_name) = self.selected_tool_name() {
                    // Get tool for parameter input
                    if let Some(tool) = self.selected_tool() {
                        // Get execution parameters (in a real implementation, this would open a parameter input dialog)
                        if let Some(parameters) = self.get_tool_execution_parameters(tool) {
                            // Start tool execution
                            match self.start_tool_execution(&tool_name, parameters) {
                                Ok(()) => {
                                    // Return action to actually execute the tool
                                    Ok(Some(Action::ExecuteTool(tool_name)))
                                }
                                Err(error) => {
                                    // In a real implementation, show error dialog
                                    tracing::error!("Failed to start tool execution: {}", error);
                                    Ok(None)
                                }
                            }
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
            KeyCode::Char(' ') => {
                self.toggle_details();
                Ok(None)
            }
            KeyCode::Char('r') | KeyCode::F(5) => {
                Ok(Some(Action::RefreshTools))
            }
            
            // Tool execution management
            KeyCode::Char('R') => {
                // Retry last failed execution
                if let Some(tool_name) = self.selected_tool_name() {
                    match self.retry_tool_execution(&tool_name) {
                        Ok(()) => {
                            Ok(Some(Action::ExecuteTool(tool_name)))
                        }
                        Err(error) => {
                            tracing::error!("Failed to retry tool execution: {}", error);
                            Ok(None)
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('h') => {
                // Show execution history for selected tool
                if let Some(tool_name) = self.selected_tool_name() {
                    if let Some(_history) = self.get_execution_history(&tool_name) {
                        // In a real implementation, this would open a history dialog
                        // For now, just toggle details to show history in the details panel
                        if !self.show_details {
                            self.toggle_details();
                        }
                    }
                }
                Ok(None)
            }
            KeyCode::Char('C') => {
                // Clear execution history for selected tool
                if let Some(tool_name) = self.selected_tool_name() {
                    self.clear_tool_execution_history(&tool_name);
                }
                Ok(None)
            }
            KeyCode::Char('E') => {
                // Export execution history for selected tool
                if let Some(tool_name) = self.selected_tool_name() {
                    if let Some(_json) = self.export_tool_execution_history(&tool_name) {
                        // In a real implementation, this would save to file or show export dialog
                        tracing::info!("Execution history exported for tool: {}", tool_name);
                    }
                }
                Ok(None)
            }
            
            // Filtering and searching
            KeyCode::Char('/') => {
                self.enter_search_mode();
                Ok(None)
            }
            KeyCode::Char('f') => {
                // Quick filter mode - could be enhanced
                self.enter_search_mode();
                Ok(None)
            }
            KeyCode::Char('c') => {
                self.clear_filter();
                Ok(None)
            }
            KeyCode::Char('s') => {
                // Cycle through sort orders
                self.cycle_sort_order();
                Ok(None)
            }
            
            // Display options
            KeyCode::Char('i') => {
                self.toggle_status_indicators();
                Ok(None)
            }
            KeyCode::Char('F') => {
                self.toggle_filter_panel();
                Ok(None)
            }
            
            // Quick filters (number keys)
            KeyCode::Char('1') => {
                self.quick_filter_by_status(ToolStatus::Available);
                Ok(None)
            }
            KeyCode::Char('2') => {
                self.quick_filter_by_status(ToolStatus::Running);
                Ok(None)
            }
            KeyCode::Char('3') => {
                self.quick_filter_by_status(ToolStatus::Error);
                Ok(None)
            }
            
            // Advanced filter parsing
            KeyCode::Char('A') => {
                // Advanced filter mode - could open a text input for complex filters
                self.enter_search_mode();
                Ok(None)
            }
            
            // Help and navigation
            KeyCode::Char('?') => {
                self.toggle_help();
                Ok(None)
            }
            KeyCode::Esc => {
                Ok(Some(Action::Navigate(ViewType::WorkflowList)))
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Ok(Some(Action::Quit))
            }
            
            _ => Ok(None),
        }
    }
    
    /// Handle key events in search mode
    async fn handle_search_key_event(&mut self, key: KeyEvent) -> std::result::Result<Option<Action>, WidgetError> {
        match key.code {
            KeyCode::Enter => {
                // Parse and apply the filter
                let filter_text = self.filter.clone();
                self.parse_and_apply_filter(filter_text);
                self.exit_search_mode();
                Ok(None)
            }
            KeyCode::Esc => {
                self.filter.clear();
                self.active_filters.clear();
                self.apply_filter_and_sort();
                self.update_selection();
                self.exit_search_mode();
                Ok(None)
            }
            KeyCode::Backspace => {
                self.filter.pop();
                // Real-time filtering as user types
                let filter_text = self.filter.clone();
                self.parse_and_apply_filter(filter_text);
                Ok(None)
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                // Real-time filtering as user types
                let filter_text = self.filter.clone();
                self.parse_and_apply_filter(filter_text);
                Ok(None)
            }
            _ => Ok(None),
        }
    }
    
    /// Show tool execution parameter input dialog
    pub async fn show_parameter_input_dialog(&mut self, tool: &ToolDisplayInfo) -> Option<serde_json::Value> {
        // In a real implementation, this would show an interactive parameter input dialog
        // For now, we'll generate default parameters or use a simple input mechanism
        
        if tool.info.parameters_schema.is_null() {
            // No parameters needed
            return Some(serde_json::Value::Object(serde_json::Map::new()));
        }
        
        // Generate parameters based on schema with some intelligent defaults
        self.generate_execution_parameters(&tool.info.parameters_schema)
    }
    
    /// Generate execution parameters with intelligent defaults
    fn generate_execution_parameters(&self, schema: &serde_json::Value) -> Option<serde_json::Value> {
        if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
            let mut params = serde_json::Map::new();
            
            for (param_name, param_schema) in properties {
                let param_value = match param_schema.get("type").and_then(|v| v.as_str()) {
                    Some("string") => {
                        // Use default if available, otherwise use intelligent defaults based on name
                        if let Some(default) = param_schema.get("default") {
                            default.clone()
                        } else {
                            match param_name.to_lowercase().as_str() {
                                name if name.contains("path") || name.contains("file") => {
                                    serde_json::Value::String("./".to_string())
                                }
                                name if name.contains("url") || name.contains("uri") => {
                                    serde_json::Value::String("https://example.com".to_string())
                                }
                                name if name.contains("name") => {
                                    serde_json::Value::String("example".to_string())
                                }
                                name if name.contains("message") || name.contains("text") => {
                                    serde_json::Value::String("Hello, World!".to_string())
                                }
                                _ => serde_json::Value::String("".to_string())
                            }
                        }
                    }
                    Some("number") | Some("integer") => {
                        if let Some(default) = param_schema.get("default") {
                            default.clone()
                        } else {
                            // Use minimum if available, otherwise 0
                            param_schema.get("minimum")
                                .and_then(|v| v.as_i64())
                                .map(|n| serde_json::Value::Number(serde_json::Number::from(n)))
                                .unwrap_or_else(|| serde_json::Value::Number(serde_json::Number::from(0)))
                        }
                    }
                    Some("boolean") => {
                        if let Some(default) = param_schema.get("default") {
                            default.clone()
                        } else {
                            serde_json::Value::Bool(false)
                        }
                    }
                    Some("array") => {
                        if let Some(default) = param_schema.get("default") {
                            default.clone()
                        } else {
                            serde_json::Value::Array(vec![])
                        }
                    }
                    Some("object") => {
                        if let Some(default) = param_schema.get("default") {
                            default.clone()
                        } else {
                            serde_json::Value::Object(serde_json::Map::new())
                        }
                    }
                    _ => serde_json::Value::Null,
                };
                
                params.insert(param_name.clone(), param_value);
            }
            
            Some(serde_json::Value::Object(params))
        } else {
            Some(serde_json::Value::Object(serde_json::Map::new()))
        }
    }
    
    /// Execute tool with performance monitoring
    pub async fn execute_tool_with_monitoring(&mut self, tool_name: &str, parameters: serde_json::Value) -> Result<serde_json::Value, String> {
        let start_time = Utc::now();
        
        // Mark tool as executing
        self.mark_tool_executing(tool_name);
        
        // Create execution record
        let mut execution_record = ToolExecutionRecord {
            started_at: start_time,
            completed_at: None,
            duration: None,
            success: false,
            error_message: None,
            parameters: parameters.clone(),
        };
        
        // Simulate tool execution (in a real implementation, this would call the actual tool)
        let result = self.simulate_tool_execution(tool_name, &parameters).await;
        
        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time).to_std().unwrap_or(Duration::from_secs(0));
        
        // Update execution record
        execution_record.completed_at = Some(end_time);
        execution_record.duration = Some(duration);
        
        match result {
            Ok(output) => {
                execution_record.success = true;
                
                // Mark tool as finished successfully
                self.mark_tool_finished(tool_name, true, None);
                
                // Add to execution history
                self.add_execution_record(tool_name, execution_record);
                
                Ok(output)
            }
            Err(error) => {
                execution_record.success = false;
                execution_record.error_message = Some(error.clone());
                
                // Mark tool as finished with error
                self.mark_tool_finished(tool_name, false, Some(error.clone()));
                
                // Add to execution history
                self.add_execution_record(tool_name, execution_record);
                
                Err(error)
            }
        }
    }
    
    /// Simulate tool execution (placeholder for actual implementation)
    async fn simulate_tool_execution(&self, tool_name: &str, parameters: &serde_json::Value) -> Result<serde_json::Value, String> {
        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(100 + (tool_name.len() * 10) as u64)).await;
        
        // Simulate different outcomes based on tool name and parameters
        match tool_name {
            name if name.contains("error") || name.contains("fail") => {
                Err(format!("Simulated error in tool: {}", name))
            }
            name if name.contains("slow") => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(serde_json::json!({
                    "status": "completed",
                    "message": format!("Slow tool {} completed successfully", name),
                    "parameters": parameters,
                    "execution_time": "2000ms"
                }))
            }
            _ => {
                Ok(serde_json::json!({
                    "status": "success",
                    "message": format!("Tool {} executed successfully", tool_name),
                    "parameters": parameters,
                    "timestamp": Utc::now().to_rfc3339()
                }))
            }
        }
    }
    
    /// Cancel tool execution
    pub fn cancel_tool_execution(&mut self, tool_name: &str) -> bool {
        if self.is_tool_executing(tool_name) {
            // Mark tool as finished with cancellation
            self.mark_tool_finished(tool_name, false, Some("Execution cancelled by user".to_string()));
            true
        } else {
            false
        }
    }
    
    /// Get tool performance metrics
    pub fn get_tool_performance_metrics(&self, tool_name: &str) -> Option<ToolPerformanceMetrics> {
        if let Some(history) = self.get_execution_history(tool_name) {
            if history.is_empty() {
                return None;
            }
            
            let total_executions = history.len();
            let successful_executions = history.iter().filter(|r| r.success).count();
            let failed_executions = total_executions - successful_executions;
            
            let durations: Vec<Duration> = history.iter()
                .filter_map(|r| r.duration)
                .collect();
            
            let average_duration = if !durations.is_empty() {
                let total_ms: u64 = durations.iter().map(|d| d.as_millis() as u64).sum();
                Some(Duration::from_millis(total_ms / durations.len() as u64))
            } else {
                None
            };
            
            let min_duration = durations.iter().min().cloned();
            let max_duration = durations.iter().max().cloned();
            
            // Calculate success rate trend (last 10 executions)
            let recent_executions = history.iter().rev().take(10).collect::<Vec<_>>();
            let recent_success_rate = if !recent_executions.is_empty() {
                recent_executions.iter().filter(|r| r.success).count() as f64 / recent_executions.len() as f64
            } else {
                0.0
            };
            
            Some(ToolPerformanceMetrics {
                total_executions,
                successful_executions,
                failed_executions,
                success_rate: successful_executions as f64 / total_executions as f64,
                recent_success_rate,
                average_duration,
                min_duration,
                max_duration,
                last_execution: history.last().map(|r| r.started_at),
                executions_per_hour: self.calculate_executions_per_hour(history),
            })
        } else {
            None
        }
    }
    
    /// Calculate executions per hour based on history
    fn calculate_executions_per_hour(&self, history: &[ToolExecutionRecord]) -> f64 {
        if history.len() < 2 {
            return 0.0;
        }
        
        let first_execution = history.first().unwrap().started_at;
        let last_execution = history.last().unwrap().started_at;
        let duration_hours = last_execution.signed_duration_since(first_execution).num_seconds() as f64 / 3600.0;
        
        if duration_hours > 0.0 {
            history.len() as f64 / duration_hours
        } else {
            0.0
        }
    }
    
    /// Show tool execution result dialog
    pub fn show_execution_result(&self, tool_name: &str, result: &Result<serde_json::Value, String>) -> String {
        match result {
            Ok(output) => {
                format!(
                    "✅ 工具 '{}' 执行成功\n\n结果:\n{}",
                    tool_name,
                    serde_json::to_string_pretty(output).unwrap_or_else(|_| "无法格式化结果".to_string())
                )
            }
            Err(error) => {
                format!(
                    "❌ 工具 '{}' 执行失败\n\n错误信息:\n{}",
                    tool_name,
                    error
                )
            }
        }
    }
    
    /// Cycle through sort orders
    fn cycle_sort_order(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::NameAsc => SortOrder::NameDesc,
            SortOrder::NameDesc => SortOrder::StatusAsc,
            SortOrder::StatusAsc => SortOrder::StatusDesc,
            SortOrder::StatusDesc => SortOrder::DateAsc,
            SortOrder::DateAsc => SortOrder::DateDesc,
            SortOrder::DateDesc => SortOrder::Custom("category_asc".to_string()),
            SortOrder::Custom(ref custom) => match custom.as_str() {
                "category_asc" => SortOrder::Custom("category_desc".to_string()),
                "category_desc" => SortOrder::Custom("source_asc".to_string()),
                "source_asc" => SortOrder::Custom("source_desc".to_string()),
                "source_desc" => SortOrder::Custom("execution_count_desc".to_string()),
                "execution_count_desc" => SortOrder::Custom("success_rate_desc".to_string()),
                "success_rate_desc" => SortOrder::NameAsc,
                _ => SortOrder::NameAsc,
            },
        };
        
        self.apply_filter_and_sort();
        self.update_selection();
    }
    
    /// Highlight text with search terms (static version for use in closures)
    fn highlight_text_static<'a>(text: &'a str, search: &'a str, highlight_style: Style) -> Vec<Span<'a>> {
        if search.is_empty() {
            return vec![Span::styled(text, Style::default())];
        }
        
        let search_lower = search.to_lowercase();
        let text_lower = text.to_lowercase();
        
        let mut spans = Vec::new();
        let mut last_end = 0;
        
        for (start, _) in text_lower.match_indices(&search_lower) {
            // Add text before match
            if start > last_end {
                spans.push(Span::styled(&text[last_end..start], Style::default()));
            }
            
            // Add highlighted match
            let end = start + search.len();
            spans.push(Span::styled(&text[start..end], highlight_style));
            last_end = end;
        }
        
        // Add remaining text
        if last_end < text.len() {
            spans.push(Span::styled(&text[last_end..], Style::default()));
        }
        
        if spans.is_empty() {
            vec![Span::styled(text, Style::default())]
        } else {
            spans
        }
    }
}