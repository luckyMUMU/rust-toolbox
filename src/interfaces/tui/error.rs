//! TUI Error Handling System
//! 
//! This module provides comprehensive error handling for the TUI interface,
//! including error classification, recovery strategies, and user-friendly error display.

use crate::error::{Result, WorkflowError};
use async_trait::async_trait;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

/// TUI-specific error types with recovery strategies
#[derive(Error, Debug, Clone)]
pub enum TuiError {
    #[error("渲染错误: {message}")]
    RenderError { message: String },
    
    #[error("输入处理错误: {message}")]
    InputError { message: String },
    
    #[error("数据同步错误: {message}")]
    DataError { message: String },
    
    #[error("配置错误: {message}")]
    ConfigError { message: String },
    
    #[error("终端操作错误: {message}")]
    TerminalError { message: String },
    
    #[error("Widget错误: {widget_id} - {message}")]
    WidgetError { widget_id: String, message: String },
    
    #[error("布局错误: {message}")]
    LayoutError { message: String },
    
    #[error("主题错误: {message}")]
    ThemeError { message: String },
    
    #[error("网络连接错误: {message}")]
    NetworkError { message: String },
    
    #[error("权限错误: {message}")]
    PermissionError { message: String },
}

impl From<WorkflowError> for TuiError {
    fn from(err: WorkflowError) -> Self {
        match err {
            WorkflowError::Io(io_err) => TuiError::TerminalError {
                message: format!("IO错误: {}", io_err),
            },
            WorkflowError::Config(config_err) => TuiError::ConfigError {
                message: format!("配置错误: {}", config_err),
            },
            WorkflowError::ValidationError(msg) => TuiError::InputError {
                message: format!("验证错误: {}", msg),
            },
            WorkflowError::Storage { message } => TuiError::DataError {
                message: format!("存储错误: {}", message),
            },
            WorkflowError::PermissionDenied { message } => TuiError::PermissionError {
                message,
            },
            _ => TuiError::DataError {
                message: format!("系统错误: {}", err),
            },
        }
    }
}

/// Error severity levels for prioritization and display
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "信息"),
            ErrorSeverity::Warning => write!(f, "警告"),
            ErrorSeverity::Error => write!(f, "错误"),
            ErrorSeverity::Critical => write!(f, "严重"),
        }
    }
}

impl ErrorSeverity {
    pub fn color(&self) -> Color {
        match self {
            ErrorSeverity::Info => Color::Blue,
            ErrorSeverity::Warning => Color::Yellow,
            ErrorSeverity::Error => Color::Red,
            ErrorSeverity::Critical => Color::Magenta,
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            ErrorSeverity::Info => "ℹ",
            ErrorSeverity::Warning => "⚠",
            ErrorSeverity::Error => "✗",
            ErrorSeverity::Critical => "🔥",
        }
    }
}

/// Recovery strategies for different types of errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff
    Retry { max_attempts: u32, delay_ms: u64 },
    /// Use a default/fallback value
    UseDefault,
    /// Skip the operation and continue
    Skip,
    /// Restart the component
    RestartComponent,
    /// Show error dialog and wait for user action
    ShowDialog,
    /// Graceful degradation - disable feature
    Degrade,
    /// No recovery possible - requires manual intervention
    Manual,
}

/// Error context with recovery information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error: TuiError,
    pub severity: ErrorSeverity,
    pub timestamp: Instant,
    pub component: String,
    pub recovery_strategy: RecoveryStrategy,
    pub user_message: String,
    pub technical_details: String,
    pub recovery_suggestions: Vec<String>,
    pub retry_count: u32,
}

impl ErrorContext {
    pub fn new(error: TuiError, component: String) -> Self {
        let (severity, recovery_strategy, user_message, suggestions) = match &error {
            TuiError::RenderError { .. } => (
                ErrorSeverity::Warning,
                RecoveryStrategy::Retry { max_attempts: 3, delay_ms: 100 },
                "界面渲染出现问题，正在尝试恢复".to_string(),
                vec!["检查终端大小".to_string(), "重启应用程序".to_string()],
            ),
            TuiError::InputError { .. } => (
                ErrorSeverity::Warning,
                RecoveryStrategy::Skip,
                "输入处理失败，请重试".to_string(),
                vec!["检查输入格式".to_string(), "使用默认值".to_string()],
            ),
            TuiError::DataError { .. } => (
                ErrorSeverity::Error,
                RecoveryStrategy::Retry { max_attempts: 5, delay_ms: 1000 },
                "数据同步失败，正在重试".to_string(),
                vec!["检查网络连接".to_string(), "刷新数据".to_string()],
            ),
            TuiError::ConfigError { .. } => (
                ErrorSeverity::Error,
                RecoveryStrategy::UseDefault,
                "配置加载失败，使用默认配置".to_string(),
                vec!["检查配置文件".to_string(), "重置配置".to_string()],
            ),
            TuiError::TerminalError { .. } => (
                ErrorSeverity::Critical,
                RecoveryStrategy::RestartComponent,
                "终端操作失败，需要重启".to_string(),
                vec!["重启应用程序".to_string(), "检查终端兼容性".to_string()],
            ),
            TuiError::WidgetError { .. } => (
                ErrorSeverity::Warning,
                RecoveryStrategy::Degrade,
                "组件出现问题，已禁用相关功能".to_string(),
                vec!["刷新界面".to_string(), "重启应用程序".to_string()],
            ),
            TuiError::NetworkError { .. } => (
                ErrorSeverity::Error,
                RecoveryStrategy::Retry { max_attempts: 3, delay_ms: 2000 },
                "网络连接失败，正在重试".to_string(),
                vec!["检查网络连接".to_string(), "切换到离线模式".to_string()],
            ),
            TuiError::PermissionError { .. } => (
                ErrorSeverity::Error,
                RecoveryStrategy::Manual,
                "权限不足，请检查访问权限".to_string(),
                vec!["联系管理员".to_string(), "检查文件权限".to_string()],
            ),
            _ => (
                ErrorSeverity::Error,
                RecoveryStrategy::ShowDialog,
                "发生未知错误".to_string(),
                vec!["重试操作".to_string(), "重启应用程序".to_string()],
            ),
        };
        
        Self {
            technical_details: format!("{:?}", error),
            user_message,
            recovery_suggestions: suggestions,
            error,
            severity,
            timestamp: Instant::now(),
            component,
            recovery_strategy,
            retry_count: 0,
        }
    }
    
    pub fn with_user_message(mut self, message: String) -> Self {
        self.user_message = message;
        self
    }
    
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.recovery_suggestions = suggestions;
        self
    }
    
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
    
    pub fn should_retry(&self) -> bool {
        match &self.recovery_strategy {
            RecoveryStrategy::Retry { max_attempts, .. } => self.retry_count < *max_attempts,
            _ => false,
        }
    }
    
    pub fn retry_delay(&self) -> Duration {
        match &self.recovery_strategy {
            RecoveryStrategy::Retry { delay_ms, .. } => {
                // Exponential backoff
                let base_delay = *delay_ms;
                let backoff_delay = base_delay * (2_u64.pow(self.retry_count.min(10)));
                Duration::from_millis(backoff_delay.min(30000)) // Max 30 seconds
            }
            _ => Duration::from_millis(0),
        }
    }
}

/// Error recovery handler trait
#[async_trait]
pub trait ErrorRecoveryHandler: Send + Sync {
    async fn handle_error(&self, context: &mut ErrorContext) -> Result<bool>;
    fn can_handle(&self, error: &TuiError) -> bool;
    fn priority(&self) -> u32;
}

/// Default error recovery handler
pub struct DefaultRecoveryHandler;

#[async_trait]
impl ErrorRecoveryHandler for DefaultRecoveryHandler {
    async fn handle_error(&self, context: &mut ErrorContext) -> Result<bool> {
        match &context.recovery_strategy {
            RecoveryStrategy::Retry { .. } => {
                if context.should_retry() {
                    context.increment_retry();
                    let delay = context.retry_delay();
                    info!(
                        "Retrying operation for component '{}' (attempt {}) after {:?}",
                        context.component, context.retry_count, delay
                    );
                    tokio::time::sleep(delay).await;
                    Ok(true) // Indicate retry should happen
                } else {
                    warn!(
                        "Max retry attempts reached for component '{}', giving up",
                        context.component
                    );
                    Ok(false)
                }
            }
            RecoveryStrategy::UseDefault => {
                info!("Using default recovery for component '{}'", context.component);
                Ok(true)
            }
            RecoveryStrategy::Skip => {
                info!("Skipping failed operation for component '{}'", context.component);
                Ok(true)
            }
            RecoveryStrategy::Degrade => {
                warn!("Degrading functionality for component '{}'", context.component);
                Ok(true)
            }
            _ => Ok(false), // Other strategies need specialized handlers
        }
    }
    
    fn can_handle(&self, _error: &TuiError) -> bool {
        true // Default handler can handle any error
    }
    
    fn priority(&self) -> u32 {
        0 // Lowest priority
    }
}

/// Error manager for centralized error handling
pub struct ErrorManager {
    handlers: Vec<Arc<dyn ErrorRecoveryHandler>>,
    error_history: Arc<RwLock<VecDeque<ErrorContext>>>,
    error_sender: mpsc::UnboundedSender<ErrorContext>,
    error_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ErrorContext>>>>,
    max_history: usize,
}

impl ErrorManager {
    pub fn new() -> Self {
        let (error_sender, error_receiver) = mpsc::unbounded_channel();
        
        Self {
            handlers: vec![Arc::new(DefaultRecoveryHandler)],
            error_history: Arc::new(RwLock::new(VecDeque::new())),
            error_sender,
            error_receiver: Arc::new(RwLock::new(Some(error_receiver))),
            max_history: 100,
        }
    }
    
    pub fn add_handler(&mut self, handler: Arc<dyn ErrorRecoveryHandler>) {
        self.handlers.push(handler);
        // Sort by priority (higher priority first)
        self.handlers.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }
    
    pub async fn handle_error(&self, mut context: ErrorContext) -> Result<bool> {
        // Log the error
        match context.severity {
            ErrorSeverity::Info => info!("TUI Info: {} - {}", context.component, context.user_message),
            ErrorSeverity::Warning => warn!("TUI Warning: {} - {}", context.component, context.user_message),
            ErrorSeverity::Error => error!("TUI Error: {} - {}", context.component, context.user_message),
            ErrorSeverity::Critical => error!("TUI Critical: {} - {}", context.component, context.user_message),
        }
        
        // Try recovery handlers in priority order
        for handler in &self.handlers {
            if handler.can_handle(&context.error) {
                match handler.handle_error(&mut context).await {
                    Ok(true) => {
                        // Recovery successful
                        self.add_to_history(context).await;
                        return Ok(true);
                    }
                    Ok(false) => {
                        // Handler couldn't recover, try next
                        continue;
                    }
                    Err(e) => {
                        error!("Recovery handler failed: {}", e);
                        continue;
                    }
                }
            }
        }
        
        // No handler could recover
        self.add_to_history(context).await;
        Ok(false)
    }
    
    pub async fn report_error(&self, error: TuiError, component: String) -> Result<()> {
        let context = ErrorContext::new(error, component);
        self.error_sender.send(context).map_err(|e| {
            WorkflowError::ValidationError(format!("Failed to report error: {}", e))
        })?;
        Ok(())
    }
    
    pub async fn start_error_processing(&self) -> Result<()> {
        let mut receiver_guard = self.error_receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            let error_manager = self.clone_for_processing().await;
            
            tokio::spawn(async move {
                while let Some(context) = receiver.recv().await {
                    if let Err(e) = error_manager.handle_error(context).await {
                        error!("Error processing failed: {}", e);
                    }
                }
            });
        }
        Ok(())
    }
    
    async fn clone_for_processing(&self) -> Self {
        Self {
            handlers: self.handlers.clone(),
            error_history: Arc::clone(&self.error_history),
            error_sender: self.error_sender.clone(),
            error_receiver: Arc::new(RwLock::new(None)), // Don't clone receiver
            max_history: self.max_history,
        }
    }
    
    async fn add_to_history(&self, context: ErrorContext) {
        let mut history = self.error_history.write().await;
        history.push_back(context);
        
        // Maintain max history size
        while history.len() > self.max_history {
            history.pop_front();
        }
    }
    
    pub async fn get_recent_errors(&self, count: usize) -> Vec<ErrorContext> {
        let history = self.error_history.read().await;
        history.iter().rev().take(count).cloned().collect()
    }
    
    pub async fn get_errors_by_severity(&self, severity: ErrorSeverity) -> Vec<ErrorContext> {
        let history = self.error_history.read().await;
        history
            .iter()
            .filter(|ctx| ctx.severity == severity)
            .cloned()
            .collect()
    }
    
    pub async fn clear_history(&self) {
        let mut history = self.error_history.write().await;
        history.clear();
    }
}

/// Error display widget for showing errors to users
pub struct ErrorDisplayWidget {
    current_error: Option<ErrorContext>,
    show_details: bool,
    auto_dismiss_timer: Option<Instant>,
    auto_dismiss_duration: Duration,
}

impl ErrorDisplayWidget {
    pub fn new() -> Self {
        Self {
            current_error: None,
            show_details: false,
            auto_dismiss_timer: None,
            auto_dismiss_duration: Duration::from_secs(5),
        }
    }
    
    pub fn show_error(&mut self, context: ErrorContext) {
        // Auto-dismiss for info and warnings
        self.auto_dismiss_timer = match context.severity {
            ErrorSeverity::Info | ErrorSeverity::Warning => Some(Instant::now()),
            _ => None,
        };
        
        self.current_error = Some(context);
        self.show_details = false;
    }
    
    pub fn dismiss_error(&mut self) {
        self.current_error = None;
        self.auto_dismiss_timer = None;
        self.show_details = false;
    }
    
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
    
    pub fn should_auto_dismiss(&self) -> bool {
        if let Some(timer) = self.auto_dismiss_timer {
            timer.elapsed() >= self.auto_dismiss_duration
        } else {
            false
        }
    }
    
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Check for auto-dismiss
        if self.should_auto_dismiss() {
            self.dismiss_error();
            return;
        }
        
        if let Some(ref context) = self.current_error {
            self.render_error_dialog(frame, area, context);
        }
    }
    
    fn render_error_dialog(&self, frame: &mut Frame, area: Rect, context: &ErrorContext) {
        // Calculate dialog size
        let dialog_width = area.width.min(80);
        let dialog_height = if self.show_details { 
            area.height.min(20) 
        } else { 
            area.height.min(10) 
        };
        
        let dialog_area = Rect {
            x: (area.width.saturating_sub(dialog_width)) / 2,
            y: (area.height.saturating_sub(dialog_height)) / 2,
            width: dialog_width,
            height: dialog_height,
        };
        
        // Clear background
        frame.render_widget(Clear, dialog_area);
        
        // Create dialog content
        let title = format!("{} {}", context.severity.icon(), context.severity);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(context.severity.color()));
        
        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);
        
        // Split content area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Message
                Constraint::Min(0),    // Details/Suggestions
                Constraint::Length(2), // Controls
            ])
            .split(inner_area);
        
        // Render message
        let message_text = Text::from(vec![
            Line::from(vec![
                Span::styled("组件: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&context.component),
            ]),
            Line::from(vec![
                Span::styled("消息: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&context.user_message),
            ]),
        ]);
        
        let message_paragraph = Paragraph::new(message_text)
            .wrap(Wrap { trim: true });
        frame.render_widget(message_paragraph, chunks[0]);
        
        // Render details or suggestions
        if self.show_details {
            let details_text = Text::from(vec![
                Line::from(vec![
                    Span::styled("技术详情:", Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Line::from(Span::raw(&context.technical_details)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("重试次数:", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!(" {}", context.retry_count)),
                ]),
            ]);
            
            let details_paragraph = Paragraph::new(details_text)
                .wrap(Wrap { trim: true });
            frame.render_widget(details_paragraph, chunks[1]);
        } else if !context.recovery_suggestions.is_empty() {
            let mut suggestion_lines = vec![
                Line::from(vec![
                    Span::styled("建议操作:", Style::default().add_modifier(Modifier::BOLD)),
                ]),
            ];
            
            for (i, suggestion) in context.recovery_suggestions.iter().enumerate() {
                suggestion_lines.push(Line::from(format!("{}. {}", i + 1, suggestion)));
            }
            
            let suggestions_text = Text::from(suggestion_lines);
            let suggestions_paragraph = Paragraph::new(suggestions_text)
                .wrap(Wrap { trim: true });
            frame.render_widget(suggestions_paragraph, chunks[1]);
        }
        
        // Render controls
        let controls_text = if self.show_details {
            "按 'd' 隐藏详情 | 按 Esc 关闭"
        } else {
            "按 'd' 显示详情 | 按 Esc 关闭"
        };
        
        let controls_paragraph = Paragraph::new(controls_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(controls_paragraph, chunks[2]);
    }
    
    pub fn has_error(&self) -> bool {
        self.current_error.is_some()
    }
}

impl Default for ErrorDisplayWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macros for error reporting
#[macro_export]
macro_rules! tui_error {
    ($manager:expr, $component:expr, $error:expr) => {
        if let Err(e) = $manager.report_error($error, $component.to_string()).await {
            tracing::error!("Failed to report TUI error: {}", e);
        }
    };
}

#[macro_export]
macro_rules! tui_render_error {
    ($manager:expr, $component:expr, $message:expr) => {
        tui_error!(
            $manager,
            $component,
            $crate::interfaces::tui::error::TuiError::RenderError {
                message: $message.to_string()
            }
        )
    };
}

#[macro_export]
macro_rules! tui_data_error {
    ($manager:expr, $component:expr, $message:expr) => {
        tui_error!(
            $manager,
            $component,
            $crate::interfaces::tui::error::TuiError::DataError {
                message: $message.to_string()
            }
        )
    };
}

/// Helper functions for creating common error types
impl TuiError {
    pub fn render_error<S: Into<String>>(message: S) -> Self {
        Self::RenderError {
            message: message.into(),
        }
    }
    
    pub fn input_error<S: Into<String>>(message: S) -> Self {
        Self::InputError {
            message: message.into(),
        }
    }
    
    pub fn data_error<S: Into<String>>(message: S) -> Self {
        Self::DataError {
            message: message.into(),
        }
    }
    
    pub fn widget_error<S: Into<String>>(widget_id: S, message: S) -> Self {
        Self::WidgetError {
            widget_id: widget_id.into(),
            message: message.into(),
        }
    }
    
    pub fn network_error<S: Into<String>>(message: S) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }
}