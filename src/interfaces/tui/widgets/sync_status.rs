use crate::error::{Result, WorkflowError};
use crate::interfaces::tui::sync::{DataSyncManager, SyncMetrics, SyncStatus};
use crate::interfaces::tui::widget::{Widget, WidgetContext, WidgetId, WidgetCapabilities, SizeConstraints, WidgetError};
use crate::interfaces::tui::action::Action;
use crate::interfaces::tui::event::TuiEvent;
use crate::interfaces::tui::theme::Theme;
use async_trait::async_trait;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    text::Line,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Sync Status Widget - displays synchronization status and metrics
pub struct SyncStatusWidget {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
    sync_manager: Option<Arc<RwLock<DataSyncManager>>>,
    metrics: SyncMetrics,
}

impl SyncStatusWidget {
    pub fn new() -> Self {
        let capabilities = WidgetCapabilities {
            keyboard_input: true,
            mouse_input: false,
            focusable: true,
            resizable: true,
            scrollable: false,
            themeable: true,
            configurable: false,
        };
        
        let size_constraints = SizeConstraints::new()
            .min_size(30, 5)
            .preferred_size(60, 10);
        
        Self {
            context: WidgetContext::new(WidgetId::from("sync_status")),
            capabilities,
            size_constraints,
            sync_manager: None,
            metrics: SyncMetrics::default(),
        }
    }
    
    pub fn set_sync_manager(&mut self, sync_manager: Arc<RwLock<DataSyncManager>>) {
        self.sync_manager = Some(sync_manager);
    }
}

#[async_trait]
impl Widget for SyncStatusWidget {
    fn id(&self) -> &WidgetId {
        &self.context.id
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
    
    async fn initialize(&mut self) -> std::result::Result<(), WidgetError> {
        Ok(())
    }
    
    async fn cleanup(&mut self) -> std::result::Result<(), WidgetError> {
        Ok(())
    }
    
    async fn update(&mut self) -> std::result::Result<(), WidgetError> {
        if let Some(sync_manager) = &self.sync_manager {
            let manager = sync_manager.read().await;
            self.metrics = manager.get_metrics().await;
        }
        Ok(())
    }
    
    async fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) -> std::result::Result<(), WidgetError> {
        let status_text = match &self.metrics.current_status {
            SyncStatus::Idle => "空闲",
            SyncStatus::Syncing => "同步中...",
            SyncStatus::Success => "同步成功",
            SyncStatus::Failed(_) => "同步失败",
            SyncStatus::Offline => "离线模式",
            SyncStatus::Retrying { .. } => "重试中",
        };
        
        let content = vec![
            Line::from(format!("同步状态: {}", status_text)),
            Line::from(format!("成功次数: {}", self.metrics.successful_syncs)),
            Line::from(format!("失败次数: {}", self.metrics.failed_syncs)),
            Line::from(format!("缓存命中率: {:.1}%", self.metrics.cache_hit_rate * 100.0)),
            Line::from(format!("数据新鲜度: {:.1}%", self.metrics.data_freshness * 100.0)),
        ];
        
        let paragraph = Paragraph::new(content)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("同步状态"));
        
        frame.render_widget(paragraph, area);
        Ok(())
    }
    
    async fn handle_event(&mut self, event: ratatui::crossterm::event::Event) -> std::result::Result<Option<Action>, WidgetError> {
        Ok(None)
    }
    
    fn title(&self) -> &str {
        "同步状态"
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![("显示数据同步状态和缓存信息", "")]
    }
}