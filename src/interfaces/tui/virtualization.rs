//! TUI Data Virtualization Module
//!
//! This module provides virtualization capabilities for displaying large datasets
//! efficiently in the TUI interface, including virtual scrolling, pagination,
//! and lazy loading mechanisms.

use crate::error::Result;
use async_trait::async_trait;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState,
        TableState,
    },
    Frame,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Virtualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualizationConfig {
    /// Page size for pagination
    pub page_size: usize,

    /// Buffer size for virtual scrolling (items to keep in memory)
    pub buffer_size: usize,

    /// Prefetch size (items to load ahead)
    pub prefetch_size: usize,

    /// Enable lazy loading
    pub enable_lazy_loading: bool,

    /// Lazy loading threshold (distance from viewport)
    pub lazy_loading_threshold: usize,

    /// Cache size for loaded items
    pub cache_size: usize,

    /// Cache TTL for items
    pub cache_ttl: Duration,

    /// Enable virtual scrolling
    pub enable_virtual_scrolling: bool,

    /// Minimum item height for virtual scrolling
    pub min_item_height: u16,

    /// Maximum item height for virtual scrolling
    pub max_item_height: u16,

    /// Enable smooth scrolling
    pub enable_smooth_scrolling: bool,

    /// Smooth scrolling animation duration
    pub scroll_animation_duration: Duration,

    /// Enable data compression for large items
    pub enable_compression: bool,

    /// Compression threshold (item size in bytes)
    pub compression_threshold: usize,
}

impl Default for VirtualizationConfig {
    fn default() -> Self {
        Self {
            page_size: 100,
            buffer_size: 500,
            prefetch_size: 50,
            enable_lazy_loading: true,
            lazy_loading_threshold: 20,
            cache_size: 1000,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            enable_virtual_scrolling: true,
            min_item_height: 1,
            max_item_height: 10,
            enable_smooth_scrolling: true,
            scroll_animation_duration: Duration::from_millis(200),
            enable_compression: true,
            compression_threshold: 1024, // 1KB
        }
    }
}

/// Virtual item data
#[derive(Debug, Clone)]
pub struct VirtualItem<T> {
    pub index: usize,
    pub data: Option<T>,
    pub height: u16,
    pub cached_at: Option<Instant>,
    pub loading: bool,
    pub error: Option<String>,
}

impl<T> VirtualItem<T> {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            data: None,
            height: 1,
            cached_at: None,
            loading: false,
            error: None,
        }
    }

    pub fn with_data(index: usize, data: T, height: u16) -> Self {
        Self {
            index,
            data: Some(data),
            height,
            cached_at: Some(Instant::now()),
            loading: false,
            error: None,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.data.is_some() && !self.loading
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.cached_at
            .map(|cached_at| cached_at.elapsed() > ttl)
            .unwrap_or(true)
    }
}

/// Data provider trait for virtualized data
#[async_trait]
pub trait VirtualDataProvider<T>: Send + Sync {
    /// Get the total number of items
    async fn total_count(&self) -> Result<usize>;

    /// Load a range of items
    async fn load_range(&self, start: usize, count: usize) -> Result<Vec<T>>;

    /// Get item height (for variable height items)
    async fn item_height(&self, _index: usize) -> Result<u16> {
        Ok(1) // Default height
    }

    /// Search items (optional)
    async fn search(&self, _query: &str, _start: usize, _count: usize) -> Result<Vec<(usize, T)>> {
        // Default implementation: no search support
        Ok(Vec::new())
    }

    /// Filter items (optional)
    async fn filter(
        &self,
        _predicate: Box<dyn Fn(&T) -> bool + Send + Sync>,
        _start: usize,
        _count: usize,
    ) -> Result<Vec<(usize, T)>> {
        // Default implementation: no filter support
        Ok(Vec::new())
    }
}

/// Virtual list widget for large datasets
pub struct VirtualListWidget<T> {
    config: VirtualizationConfig,
    data_provider: Arc<dyn VirtualDataProvider<T>>,

    // Virtual scrolling state
    viewport_start: usize,
    viewport_size: usize,
    total_items: usize,

    // Item cache
    item_cache: HashMap<usize, VirtualItem<T>>,
    cache_order: VecDeque<usize>, // LRU cache order

    // Rendering state
    list_state: ListState,
    scrollbar_state: ScrollbarState,

    // Loading state
    loading_ranges: HashMap<usize, Instant>, // start_index -> loading_started_at

    // Search and filter
    search_query: Option<String>,
    filter_active: bool,
    filtered_indices: Vec<usize>,

    // Performance metrics
    load_times: VecDeque<Duration>,
    cache_hits: u64,
    cache_misses: u64,

    // Smooth scrolling
    scroll_animation: Option<ScrollAnimation>,
}

#[derive(Debug, Clone)]
struct ScrollAnimation {
    start_position: f64,
    target_position: f64,
    start_time: Instant,
    duration: Duration,
}

impl<T> VirtualListWidget<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new virtual list widget
    pub fn new(
        config: VirtualizationConfig,
        data_provider: Arc<dyn VirtualDataProvider<T>>,
    ) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            config,
            data_provider,
            viewport_start: 0,
            viewport_size: 0,
            total_items: 0,
            item_cache: HashMap::new(),
            cache_order: VecDeque::new(),
            list_state,
            scrollbar_state: ScrollbarState::default(),
            loading_ranges: HashMap::new(),
            search_query: None,
            filter_active: false,
            filtered_indices: Vec::new(),
            load_times: VecDeque::new(),
            cache_hits: 0,
            cache_misses: 0,
            scroll_animation: None,
        }
    }

    /// Initialize the widget
    pub async fn initialize(&mut self) -> Result<()> {
        self.total_items = self.data_provider.total_count().await?;
        self.scrollbar_state = self.scrollbar_state.content_length(self.total_items);

        // Load initial viewport
        self.load_viewport(0, self.config.page_size).await?;

        tracing::debug!(
            "Virtual list initialized with {} total items",
            self.total_items
        );
        Ok(())
    }

    /// Update viewport size based on available area
    pub fn update_viewport_size(&mut self, area: Rect) {
        let available_height = area.height.saturating_sub(2) as usize; // Account for borders
        self.viewport_size = available_height;

        // Update scrollbar
        self.scrollbar_state = self
            .scrollbar_state
            .viewport_content_length(available_height);
    }

    /// Scroll to a specific position
    pub async fn scroll_to(&mut self, position: usize) -> Result<()> {
        let new_start = position.min(self.total_items.saturating_sub(self.viewport_size));

        if self.config.enable_smooth_scrolling {
            self.start_scroll_animation(new_start).await?;
        } else {
            self.set_viewport_start(new_start).await?;
        }

        Ok(())
    }

    /// Scroll by a relative amount
    pub async fn scroll_by(&mut self, delta: i32) -> Result<()> {
        let current_pos = self.viewport_start as i32;
        let new_pos = (current_pos + delta).max(0) as usize;
        self.scroll_to(new_pos).await
    }

    /// Set search query
    pub async fn set_search_query(&mut self, query: Option<String>) -> Result<()> {
        self.search_query = query;

        if let Some(ref query) = self.search_query {
            // Perform search
            let search_results = self
                .data_provider
                .search(query, 0, self.config.cache_size)
                .await?;

            self.filtered_indices = search_results.into_iter().map(|(index, _)| index).collect();
            self.filter_active = true;

            // Update total items for filtered view
            self.scrollbar_state = self
                .scrollbar_state
                .content_length(self.filtered_indices.len());
        } else {
            self.filter_active = false;
            self.filtered_indices.clear();
            self.scrollbar_state = self.scrollbar_state.content_length(self.total_items);
        }

        // Reset viewport
        self.set_viewport_start(0).await?;

        Ok(())
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.item_cache.clear();
        self.cache_order.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
        tracing::debug!("Virtual list cache cleared");
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> VirtualizationMetrics {
        let avg_load_time = if !self.load_times.is_empty() {
            self.load_times.iter().sum::<Duration>() / self.load_times.len() as u32
        } else {
            Duration::ZERO
        };

        let cache_hit_rate = if self.cache_hits + self.cache_misses > 0 {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        } else {
            0.0
        };

        VirtualizationMetrics {
            total_items: self.total_items,
            cached_items: self.item_cache.len(),
            viewport_size: self.viewport_size,
            viewport_start: self.viewport_start,
            cache_hit_rate,
            average_load_time: avg_load_time,
            active_loading_ranges: self.loading_ranges.len(),
            search_active: self.search_query.is_some(),
            filter_active: self.filter_active,
            filtered_item_count: self.filtered_indices.len(),
        }
    }

    /// Render the virtual list
    pub async fn render<F>(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        item_renderer: F,
    ) -> Result<()>
    where
        F: for<'a> Fn(&'a T, bool, Style) -> ListItem<'a> + Send + Sync,
    {
        self.update_viewport_size(area);

        // Update scroll animation
        if let Some(animation) = &self.scroll_animation {
            if animation.start_time.elapsed() >= animation.duration {
                // Animation complete
                let target = animation.target_position as usize;
                self.set_viewport_start(target).await?;
                self.scroll_animation = None;
            } else {
                // Update animated position
                let progress =
                    animation.start_time.elapsed().as_secs_f64() / animation.duration.as_secs_f64();
                let eased_progress = Self::ease_in_out_cubic(progress);
                let current_pos = animation.start_position
                    + (animation.target_position - animation.start_position) * eased_progress;

                // Don't actually change viewport_start during animation, just update visual position
                self.scrollbar_state = self.scrollbar_state.position(current_pos as usize);
            }
        }

        // Ensure viewport is loaded
        self.ensure_viewport_loaded().await?;

        // Create list items for current viewport
        let mut list_items = Vec::new();
        let effective_total = if self.filter_active {
            self.filtered_indices.len()
        } else {
            self.total_items
        };

        for i in 0..self
            .viewport_size
            .min(effective_total.saturating_sub(self.viewport_start))
        {
            let item_index = if self.filter_active {
                self.filtered_indices
                    .get(self.viewport_start + i)
                    .copied()
                    .unwrap_or(0)
            } else {
                self.viewport_start + i
            };

            if let Some(virtual_item) = self.item_cache.get(&item_index) {
                if let Some(ref data) = virtual_item.data {
                    let is_selected = self.list_state.selected() == Some(i);
                    let style = if is_selected {
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    list_items.push(item_renderer(data, is_selected, style));
                } else if virtual_item.loading {
                    list_items.push(
                        ListItem::new("Loading...").style(Style::default().fg(Color::Yellow)),
                    );
                } else if let Some(ref error) = virtual_item.error {
                    list_items.push(
                        ListItem::new(format!("Error: {}", error))
                            .style(Style::default().fg(Color::Red)),
                    );
                } else {
                    list_items
                        .push(ListItem::new("No data").style(Style::default().fg(Color::Gray)));
                }
            } else {
                list_items
                    .push(ListItem::new("Loading...").style(Style::default().fg(Color::Yellow)));
            }
        }

        // Create the list widget
        let title = if let Some(ref query) = self.search_query {
            format!(
                "Search: {} ({} results)",
                query,
                self.filtered_indices.len()
            )
        } else {
            format!("Items ({} total)", self.total_items)
        };

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        frame.render_stateful_widget(list, area, &mut self.list_state);

        // Render scrollbar if needed
        if effective_total > self.viewport_size {
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

            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut self.scrollbar_state);
        }

        Ok(())
    }

    /// Handle key events
    pub async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    } else if self.viewport_start > 0 {
                        self.scroll_by(-1).await?;
                    }
                }
                Ok(true)
            }
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected() {
                    let effective_total = if self.filter_active {
                        self.filtered_indices.len()
                    } else {
                        self.total_items
                    };

                    if selected < self.viewport_size - 1
                        && self.viewport_start + selected + 1 < effective_total
                    {
                        self.list_state.select(Some(selected + 1));
                    } else if self.viewport_start + self.viewport_size < effective_total {
                        self.scroll_by(1).await?;
                    }
                }
                Ok(true)
            }
            KeyCode::PageUp => {
                self.scroll_by(-(self.viewport_size as i32)).await?;
                Ok(true)
            }
            KeyCode::PageDown => {
                self.scroll_by(self.viewport_size as i32).await?;
                Ok(true)
            }
            KeyCode::Home => {
                self.scroll_to(0).await?;
                self.list_state.select(Some(0));
                Ok(true)
            }
            KeyCode::End => {
                let effective_total = if self.filter_active {
                    self.filtered_indices.len()
                } else {
                    self.total_items
                };

                if effective_total > 0 {
                    self.scroll_to(effective_total.saturating_sub(self.viewport_size))
                        .await?;
                    self.list_state
                        .select(Some(self.viewport_size.min(effective_total) - 1));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Get currently selected item
    pub fn get_selected_item(&self) -> Option<&T> {
        if let Some(selected) = self.list_state.selected() {
            let item_index = if self.filter_active {
                self.filtered_indices
                    .get(self.viewport_start + selected)
                    .copied()?
            } else {
                self.viewport_start + selected
            };

            self.item_cache.get(&item_index)?.data.as_ref()
        } else {
            None
        }
    }

    /// Get selected item index
    pub fn get_selected_index(&self) -> Option<usize> {
        if let Some(selected) = self.list_state.selected() {
            if self.filter_active {
                self.filtered_indices
                    .get(self.viewport_start + selected)
                    .copied()
            } else {
                Some(self.viewport_start + selected)
            }
        } else {
            None
        }
    }

    // Private methods

    async fn set_viewport_start(&mut self, start: usize) -> Result<()> {
        let effective_total = if self.filter_active {
            self.filtered_indices.len()
        } else {
            self.total_items
        };

        self.viewport_start = start.min(effective_total.saturating_sub(self.viewport_size));
        self.scrollbar_state = self.scrollbar_state.position(self.viewport_start);

        // Load viewport data
        self.ensure_viewport_loaded().await?;

        Ok(())
    }

    async fn start_scroll_animation(&mut self, target: usize) -> Result<()> {
        self.scroll_animation = Some(ScrollAnimation {
            start_position: self.viewport_start as f64,
            target_position: target as f64,
            start_time: Instant::now(),
            duration: self.config.scroll_animation_duration,
        });

        Ok(())
    }

    async fn ensure_viewport_loaded(&mut self) -> Result<()> {
        let load_start = self
            .viewport_start
            .saturating_sub(self.config.prefetch_size);
        let load_end = (self.viewport_start + self.viewport_size + self.config.prefetch_size).min(
            if self.filter_active {
                self.filtered_indices.len()
            } else {
                self.total_items
            },
        );

        // Check what ranges need to be loaded
        let mut ranges_to_load = Vec::new();
        let mut current_range_start = None;

        for i in load_start..load_end {
            let item_index = if self.filter_active {
                self.filtered_indices.get(i).copied().unwrap_or(0)
            } else {
                i
            };

            let needs_loading = !self.item_cache.contains_key(&item_index)
                || self
                    .item_cache
                    .get(&item_index)
                    .unwrap()
                    .is_expired(self.config.cache_ttl);

            if needs_loading {
                if current_range_start.is_none() {
                    current_range_start = Some(i);
                }
            } else if let Some(range_start) = current_range_start {
                ranges_to_load.push((range_start, i));
                current_range_start = None;
            }
        }

        // Add final range if needed
        if let Some(range_start) = current_range_start {
            ranges_to_load.push((range_start, load_end));
        }

        // Load missing ranges
        for (start, end) in ranges_to_load {
            self.load_range(start, end - start).await?;
        }

        Ok(())
    }

    async fn load_range(&mut self, start: usize, count: usize) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        // Check if already loading
        if self.loading_ranges.contains_key(&start) {
            return Ok(());
        }

        self.loading_ranges.insert(start, Instant::now());

        // Mark items as loading
        for i in start..start + count {
            let item_index = if self.filter_active {
                self.filtered_indices.get(i).copied().unwrap_or(0)
            } else {
                i
            };

            let item = self
                .item_cache
                .entry(item_index)
                .or_insert_with(|| VirtualItem::new(item_index));
            item.loading = true;
            item.error = None;
        }

        let load_start_time = Instant::now();

        // Load data
        match self.data_provider.load_range(start, count).await {
            Ok(data) => {
                // Store loaded data
                for (i, item_data) in data.into_iter().enumerate() {
                    let item_index = if self.filter_active {
                        self.filtered_indices.get(start + i).copied().unwrap_or(0)
                    } else {
                        start + i
                    };

                    let height = self
                        .data_provider
                        .item_height(item_index)
                        .await
                        .unwrap_or(1);
                    let virtual_item = VirtualItem::with_data(item_index, item_data, height);

                    self.item_cache.insert(item_index, virtual_item);
                    self.update_cache_order(item_index);
                }

                self.cache_hits += count as u64;
            }
            Err(e) => {
                // Mark items as error
                for i in start..start + count {
                    let item_index = if self.filter_active {
                        self.filtered_indices.get(i).copied().unwrap_or(0)
                    } else {
                        i
                    };

                    if let Some(item) = self.item_cache.get_mut(&item_index) {
                        item.loading = false;
                        item.error = Some(e.to_string());
                    }
                }

                self.cache_misses += count as u64;
                tracing::error!("Failed to load range {}-{}: {}", start, start + count, e);
            }
        }

        self.loading_ranges.remove(&start);

        // Record load time
        let load_time = load_start_time.elapsed();
        self.load_times.push_back(load_time);
        if self.load_times.len() > 100 {
            self.load_times.pop_front();
        }

        // Cleanup cache if needed
        self.cleanup_cache();

        Ok(())
    }

    fn update_cache_order(&mut self, index: usize) {
        // Remove from current position
        if let Some(pos) = self.cache_order.iter().position(|&x| x == index) {
            self.cache_order.remove(pos);
        }

        // Add to front (most recently used)
        self.cache_order.push_front(index);
    }

    fn cleanup_cache(&mut self) {
        while self.item_cache.len() > self.config.cache_size {
            if let Some(oldest_index) = self.cache_order.pop_back() {
                self.item_cache.remove(&oldest_index);
            } else {
                break;
            }
        }
    }

    fn ease_in_out_cubic(t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    async fn load_viewport(&mut self, start: usize, count: usize) -> Result<()> {
        self.load_range(start, count).await
    }
}

/// Virtualization performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualizationMetrics {
    pub total_items: usize,
    pub cached_items: usize,
    pub viewport_size: usize,
    pub viewport_start: usize,
    pub cache_hit_rate: f64,
    pub average_load_time: Duration,
    pub active_loading_ranges: usize,
    pub search_active: bool,
    pub filter_active: bool,
    pub filtered_item_count: usize,
}

/// Virtual table widget for tabular data
pub struct VirtualTableWidget<T> {
    #[allow(dead_code)]
    config: VirtualizationConfig,
    #[allow(dead_code)]
    data_provider: Arc<dyn VirtualDataProvider<T>>,

    // Table state
    table_state: TableState,
    #[allow(dead_code)]
    column_widths: Vec<u16>,
    #[allow(dead_code)]
    column_headers: Vec<String>,

    // Virtual scrolling (reuse from list)
    list_widget: VirtualListWidget<T>,
}

impl<T> VirtualTableWidget<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new virtual table widget
    pub fn new(
        config: VirtualizationConfig,
        data_provider: Arc<dyn VirtualDataProvider<T>>,
        column_headers: Vec<String>,
        column_widths: Vec<u16>,
    ) -> Self {
        let list_widget = VirtualListWidget::new(config.clone(), data_provider.clone());

        Self {
            config,
            data_provider,
            table_state: TableState::default(),
            column_widths,
            column_headers,
            list_widget,
        }
    }

    /// Initialize the table widget
    pub async fn initialize(&mut self) -> Result<()> {
        self.list_widget.initialize().await?;
        self.table_state.select(Some(0));
        Ok(())
    }

    /// Render the virtual table
    pub async fn render<F>(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        row_renderer: F,
    ) -> Result<()>
    where
        F: Fn(&T, usize) -> Vec<String> + Send + Sync,
    {
        // For now, delegate to the list widget with a custom renderer
        // In a full implementation, this would use ratatui's Table widget
        self.list_widget
            .render(frame, area, |data, _is_selected, style| {
                let row_data = row_renderer(data, 0);
                let content = row_data.join(" | ");
                ListItem::new(content).style(style)
            })
            .await
    }

    /// Handle key events
    pub async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        self.list_widget.handle_key_event(key).await
    }

    /// Get selected item
    pub fn get_selected_item(&self) -> Option<&T> {
        self.list_widget.get_selected_item()
    }
}

/// Example data provider implementation for testing
pub struct MockDataProvider {
    total_count: usize,
    item_size: usize,
}

impl MockDataProvider {
    pub fn new(total_count: usize, item_size: usize) -> Self {
        Self {
            total_count,
            item_size,
        }
    }
}

#[async_trait]
impl VirtualDataProvider<String> for MockDataProvider {
    async fn total_count(&self) -> Result<usize> {
        Ok(self.total_count)
    }

    async fn load_range(&self, start: usize, count: usize) -> Result<Vec<String>> {
        // Simulate loading delay
        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut items = Vec::new();
        for i in start..start + count {
            if i < self.total_count {
                let content = "x".repeat(self.item_size);
                items.push(format!("Item {} - {}", i, content));
            }
        }

        Ok(items)
    }

    async fn search(
        &self,
        query: &str,
        start: usize,
        count: usize,
    ) -> Result<Vec<(usize, String)>> {
        let mut results = Vec::new();
        let mut found = 0;

        for i in 0..self.total_count {
            if found >= count {
                break;
            }

            let item = format!("Item {}", i);
            if item.contains(query) {
                if found >= start {
                    results.push((i, item));
                }
                found += 1;
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_virtual_list_creation() {
        let config = VirtualizationConfig::default();
        let provider = Arc::new(MockDataProvider::new(1000, 50));
        let mut list = VirtualListWidget::new(config, provider);

        list.initialize().await.unwrap();
        assert_eq!(list.total_items, 1000);
    }

    #[tokio::test]
    async fn test_virtual_scrolling() {
        let config = VirtualizationConfig::default();
        let provider = Arc::new(MockDataProvider::new(1000, 50));
        let mut list = VirtualListWidget::new(config, provider);

        list.initialize().await.unwrap();

        // Test scrolling
        list.scroll_to(100).await.unwrap();
        assert_eq!(list.viewport_start, 100);

        list.scroll_by(50).await.unwrap();
        assert_eq!(list.viewport_start, 150);

        list.scroll_by(-25).await.unwrap();
        assert_eq!(list.viewport_start, 125);
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let config = VirtualizationConfig::default();
        let provider = Arc::new(MockDataProvider::new(1000, 50));
        let mut list = VirtualListWidget::new(config, provider);

        list.initialize().await.unwrap();

        // Test search
        list.set_search_query(Some("Item 1".to_string()))
            .await
            .unwrap();
        assert!(list.filter_active);
        assert!(!list.filtered_indices.is_empty());

        // Clear search
        list.set_search_query(None).await.unwrap();
        assert!(!list.filter_active);
        assert!(list.filtered_indices.is_empty());
    }

    #[tokio::test]
    async fn test_cache_management() {
        let mut config = VirtualizationConfig::default();
        config.cache_size = 10; // Small cache for testing

        let provider = Arc::new(MockDataProvider::new(1000, 50));
        let mut list = VirtualListWidget::new(config, provider);

        list.initialize().await.unwrap();

        // Load more items than cache size
        list.scroll_to(0).await.unwrap();
        list.scroll_to(50).await.unwrap();
        list.scroll_to(100).await.unwrap();

        // Cache should be limited
        assert!(list.item_cache.len() <= 10);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let config = VirtualizationConfig::default();
        let provider = Arc::new(MockDataProvider::new(1000, 50));
        let mut list = VirtualListWidget::new(config, provider);

        list.initialize().await.unwrap();

        let metrics = list.get_metrics();
        assert_eq!(metrics.total_items, 1000);
        assert!(metrics.cached_items > 0);
        assert!(!metrics.search_active);
        assert!(!metrics.filter_active);
    }
}
