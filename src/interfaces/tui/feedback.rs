//! User Feedback System for TUI
//!
//! This module provides comprehensive user feedback mechanisms including
//! progress indicators, confirmation dialogs, notifications, and status messages.

use crate::error::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, RwLock};
use tracing::warn;

/// Progress indicator for long-running operations
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    pub id: String,
    pub title: String,
    pub current: u64,
    pub total: u64,
    pub status: ProgressStatus,
    pub message: String,
    pub started_at: Instant,
    pub estimated_completion: Option<Instant>,
    pub show_percentage: bool,
    pub show_eta: bool,
    pub show_rate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

impl fmt::Display for ProgressStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgressStatus::NotStarted => write!(f, "未开始"),
            ProgressStatus::InProgress => write!(f, "进行中"),
            ProgressStatus::Completed => write!(f, "已完成"),
            ProgressStatus::Failed => write!(f, "失败"),
            ProgressStatus::Cancelled => write!(f, "已取消"),
            ProgressStatus::Paused => write!(f, "已暂停"),
        }
    }
}

impl ProgressStatus {
    pub fn color(&self) -> Color {
        match self {
            ProgressStatus::NotStarted => Color::Gray,
            ProgressStatus::InProgress => Color::Blue,
            ProgressStatus::Completed => Color::Green,
            ProgressStatus::Failed => Color::Red,
            ProgressStatus::Cancelled => Color::Yellow,
            ProgressStatus::Paused => Color::Magenta,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ProgressStatus::NotStarted => "○",
            ProgressStatus::InProgress => "●",
            ProgressStatus::Completed => "✓",
            ProgressStatus::Failed => "✗",
            ProgressStatus::Cancelled => "⊘",
            ProgressStatus::Paused => "⏸",
        }
    }
}

impl ProgressIndicator {
    pub fn new<S: Into<String>>(id: S, title: S, total: u64) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            current: 0,
            total,
            status: ProgressStatus::NotStarted,
            message: String::new(),
            started_at: Instant::now(),
            estimated_completion: None,
            show_percentage: true,
            show_eta: true,
            show_rate: false,
        }
    }

    pub fn start(&mut self) {
        self.status = ProgressStatus::InProgress;
        self.started_at = Instant::now();
    }

    pub fn update(&mut self, current: u64, message: Option<String>) {
        self.current = current.min(self.total);
        if let Some(msg) = message {
            self.message = msg;
        }

        // Update ETA calculation
        if self.current > 0 && self.status == ProgressStatus::InProgress {
            let elapsed = self.started_at.elapsed();
            let rate = self.current as f64 / elapsed.as_secs_f64();
            if rate > 0.0 {
                let remaining = (self.total - self.current) as f64 / rate;
                self.estimated_completion =
                    Some(Instant::now() + Duration::from_secs_f64(remaining));
            }
        }

        // Auto-complete when reaching total
        if self.current >= self.total && self.status == ProgressStatus::InProgress {
            self.status = ProgressStatus::Completed;
        }
    }

    pub fn complete(&mut self) {
        self.current = self.total;
        self.status = ProgressStatus::Completed;
        self.message = "操作完成".to_string();
    }

    pub fn fail<S: Into<String>>(&mut self, message: S) {
        self.status = ProgressStatus::Failed;
        self.message = message.into();
    }

    pub fn cancel(&mut self) {
        self.status = ProgressStatus::Cancelled;
        self.message = "操作已取消".to_string();
    }

    pub fn pause(&mut self) {
        self.status = ProgressStatus::Paused;
    }

    pub fn resume(&mut self) {
        if self.status == ProgressStatus::Paused {
            self.status = ProgressStatus::InProgress;
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn eta(&self) -> Option<Duration> {
        self.estimated_completion.map(|eta| {
            if eta > Instant::now() {
                eta.duration_since(Instant::now())
            } else {
                Duration::from_secs(0)
            }
        })
    }

    pub fn rate(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.current as f64 / elapsed
        } else {
            0.0
        }
    }
}

/// Confirmation dialog for user decisions
#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    pub title: String,
    pub message: String,
    pub options: Vec<ConfirmationOption>,
    pub default_option: usize,
    pub selected_option: usize,
    pub dialog_type: DialogType,
    pub show_details: bool,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConfirmationOption {
    pub label: String,
    pub key: char,
    pub action: ConfirmationAction,
    pub style: OptionStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmationAction {
    Confirm,
    Cancel,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionStyle {
    Default,
    Primary,
    Danger,
    Warning,
}

impl OptionStyle {
    pub fn color(&self) -> Color {
        match self {
            OptionStyle::Default => Color::White,
            OptionStyle::Primary => Color::Blue,
            OptionStyle::Danger => Color::Red,
            OptionStyle::Warning => Color::Yellow,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogType {
    Info,
    Warning,
    Error,
    Question,
    Confirmation,
}

impl DialogType {
    pub fn color(&self) -> Color {
        match self {
            DialogType::Info => Color::Blue,
            DialogType::Warning => Color::Yellow,
            DialogType::Error => Color::Red,
            DialogType::Question => Color::Cyan,
            DialogType::Confirmation => Color::Green,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            DialogType::Info => "ℹ",
            DialogType::Warning => "⚠",
            DialogType::Error => "✗",
            DialogType::Question => "?",
            DialogType::Confirmation => "✓",
        }
    }
}

impl ConfirmationDialog {
    pub fn new<S: Into<String>>(title: S, message: S, dialog_type: DialogType) -> Self {
        let default_options = match dialog_type {
            DialogType::Confirmation => vec![
                ConfirmationOption {
                    label: "确认".to_string(),
                    key: 'y',
                    action: ConfirmationAction::Confirm,
                    style: OptionStyle::Primary,
                },
                ConfirmationOption {
                    label: "取消".to_string(),
                    key: 'n',
                    action: ConfirmationAction::Cancel,
                    style: OptionStyle::Default,
                },
            ],
            DialogType::Error => vec![ConfirmationOption {
                label: "确定".to_string(),
                key: 'o',
                action: ConfirmationAction::Confirm,
                style: OptionStyle::Default,
            }],
            _ => vec![
                ConfirmationOption {
                    label: "确定".to_string(),
                    key: 'o',
                    action: ConfirmationAction::Confirm,
                    style: OptionStyle::Default,
                },
                ConfirmationOption {
                    label: "取消".to_string(),
                    key: 'c',
                    action: ConfirmationAction::Cancel,
                    style: OptionStyle::Default,
                },
            ],
        };

        Self {
            title: title.into(),
            message: message.into(),
            options: default_options,
            default_option: 0,
            selected_option: 0,
            dialog_type,
            show_details: false,
            details: None,
        }
    }

    pub fn with_options(mut self, options: Vec<ConfirmationOption>) -> Self {
        self.options = options;
        self
    }

    pub fn with_details<S: Into<String>>(mut self, details: S) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn select_next(&mut self) {
        if self.selected_option < self.options.len().saturating_sub(1) {
            self.selected_option += 1;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_option > 0 {
            self.selected_option -= 1;
        }
    }

    pub fn select_by_key(&mut self, key: char) -> Option<&ConfirmationAction> {
        for (i, option) in self.options.iter().enumerate() {
            if option.key == key {
                self.selected_option = i;
                return Some(&option.action);
            }
        }
        None
    }

    pub fn get_selected_action(&self) -> Option<&ConfirmationAction> {
        self.options
            .get(self.selected_option)
            .map(|opt| &opt.action)
    }

    pub fn toggle_details(&mut self) {
        if self.details.is_some() {
            self.show_details = !self.show_details;
        }
    }
}

/// Notification system for non-blocking messages
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Option<Duration>,
    pub dismissible: bool,
    pub actions: Vec<NotificationAction>,
}

#[derive(Debug, Clone)]
pub struct NotificationAction {
    pub label: String,
    pub key: char,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationType {
    pub fn color(&self) -> Color {
        match self {
            NotificationType::Info => Color::Blue,
            NotificationType::Success => Color::Green,
            NotificationType::Warning => Color::Yellow,
            NotificationType::Error => Color::Red,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NotificationType::Info => "ℹ",
            NotificationType::Success => "✓",
            NotificationType::Warning => "⚠",
            NotificationType::Error => "✗",
        }
    }
}

impl Notification {
    pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        id: S1,
        title: S2,
        message: S3,
        notification_type: NotificationType,
    ) -> Self {
        let duration = match notification_type {
            NotificationType::Info => Some(Duration::from_secs(3)),
            NotificationType::Success => Some(Duration::from_secs(2)),
            NotificationType::Warning => Some(Duration::from_secs(5)),
            NotificationType::Error => None, // Errors don't auto-dismiss
        };

        Self {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            notification_type,
            created_at: Instant::now(),
            duration,
            dismissible: true,
            actions: Vec::new(),
        }
    }

    pub fn with_duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_actions(mut self, actions: Vec<NotificationAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(duration) = self.duration {
            self.created_at.elapsed() >= duration
        } else {
            false
        }
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Status message for temporary feedback
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub message: String,
    pub status_type: StatusType,
    pub created_at: Instant,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
    Loading,
}

impl StatusType {
    pub fn color(&self) -> Color {
        match self {
            StatusType::Info => Color::Blue,
            StatusType::Success => Color::Green,
            StatusType::Warning => Color::Yellow,
            StatusType::Error => Color::Red,
            StatusType::Loading => Color::Cyan,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            StatusType::Info => "ℹ",
            StatusType::Success => "✓",
            StatusType::Warning => "⚠",
            StatusType::Error => "✗",
            StatusType::Loading => "⟳",
        }
    }
}

impl StatusMessage {
    pub fn new<S: Into<String>>(message: S, status_type: StatusType) -> Self {
        let duration = match status_type {
            StatusType::Loading => Duration::from_secs(30), // Longer for loading
            StatusType::Error => Duration::from_secs(10),   // Longer for errors
            _ => Duration::from_secs(3),
        };

        Self {
            message: message.into(),
            status_type,
            created_at: Instant::now(),
            duration,
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    pub fn remaining_time(&self) -> Duration {
        self.duration.saturating_sub(self.created_at.elapsed())
    }
}

/// Feedback manager for coordinating all user feedback
pub struct FeedbackManager {
    progress_indicators: Arc<RwLock<Vec<ProgressIndicator>>>,
    notifications: Arc<RwLock<VecDeque<Notification>>>,
    current_dialog: Arc<RwLock<Option<ConfirmationDialog>>>,
    status_message: Arc<RwLock<Option<StatusMessage>>>,
    feedback_sender: mpsc::UnboundedSender<FeedbackEvent>,
    feedback_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<FeedbackEvent>>>>,
    max_notifications: usize,
}

#[derive(Debug, Clone)]
pub enum FeedbackEvent {
    ShowProgress(ProgressIndicator),
    UpdateProgress {
        id: String,
        current: u64,
        message: Option<String>,
    },
    CompleteProgress(String),
    FailProgress {
        id: String,
        message: String,
    },
    CancelProgress(String),
    ShowNotification(Notification),
    DismissNotification(String),
    ShowDialog(ConfirmationDialog),
    DismissDialog,
    ShowStatus(StatusMessage),
    ClearStatus,
}

impl FeedbackManager {
    pub fn new() -> Self {
        let (feedback_sender, feedback_receiver) = mpsc::unbounded_channel();

        Self {
            progress_indicators: Arc::new(RwLock::new(Vec::new())),
            notifications: Arc::new(RwLock::new(VecDeque::new())),
            current_dialog: Arc::new(RwLock::new(None)),
            status_message: Arc::new(RwLock::new(None)),
            feedback_sender,
            feedback_receiver: Arc::new(RwLock::new(Some(feedback_receiver))),
            max_notifications: 5,
        }
    }

    pub async fn start_processing(&self) -> Result<()> {
        let mut receiver_guard = self.feedback_receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            let manager = self.clone_for_processing().await;

            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    if let Err(e) = manager.handle_feedback_event(event).await {
                        warn!("Failed to handle feedback event: {}", e);
                    }
                }
            });
        }
        Ok(())
    }

    async fn clone_for_processing(&self) -> Self {
        Self {
            progress_indicators: Arc::clone(&self.progress_indicators),
            notifications: Arc::clone(&self.notifications),
            current_dialog: Arc::clone(&self.current_dialog),
            status_message: Arc::clone(&self.status_message),
            feedback_sender: self.feedback_sender.clone(),
            feedback_receiver: Arc::new(RwLock::new(None)),
            max_notifications: self.max_notifications,
        }
    }

    async fn handle_feedback_event(&self, event: FeedbackEvent) -> Result<()> {
        match event {
            FeedbackEvent::ShowProgress(progress) => {
                let mut indicators = self.progress_indicators.write().await;
                // Remove existing progress with same ID
                indicators.retain(|p| p.id != progress.id);
                indicators.push(progress);
            }
            FeedbackEvent::UpdateProgress {
                id,
                current,
                message,
            } => {
                let mut indicators = self.progress_indicators.write().await;
                if let Some(progress) = indicators.iter_mut().find(|p| p.id == id) {
                    progress.update(current, message);
                }
            }
            FeedbackEvent::CompleteProgress(id) => {
                let mut indicators = self.progress_indicators.write().await;
                if let Some(progress) = indicators.iter_mut().find(|p| p.id == id) {
                    progress.complete();
                }
            }
            FeedbackEvent::FailProgress { id, message } => {
                let mut indicators = self.progress_indicators.write().await;
                if let Some(progress) = indicators.iter_mut().find(|p| p.id == id) {
                    progress.fail(message);
                }
            }
            FeedbackEvent::CancelProgress(id) => {
                let mut indicators = self.progress_indicators.write().await;
                if let Some(progress) = indicators.iter_mut().find(|p| p.id == id) {
                    progress.cancel();
                }
            }
            FeedbackEvent::ShowNotification(notification) => {
                let mut notifications = self.notifications.write().await;
                notifications.push_back(notification);

                // Maintain max notifications
                while notifications.len() > self.max_notifications {
                    notifications.pop_front();
                }
            }
            FeedbackEvent::DismissNotification(id) => {
                let mut notifications = self.notifications.write().await;
                notifications.retain(|n| n.id != id);
            }
            FeedbackEvent::ShowDialog(dialog) => {
                let mut current_dialog = self.current_dialog.write().await;
                *current_dialog = Some(dialog);
            }
            FeedbackEvent::DismissDialog => {
                let mut current_dialog = self.current_dialog.write().await;
                *current_dialog = None;
            }
            FeedbackEvent::ShowStatus(status) => {
                let mut status_message = self.status_message.write().await;
                *status_message = Some(status);
            }
            FeedbackEvent::ClearStatus => {
                let mut status_message = self.status_message.write().await;
                *status_message = None;
            }
        }
        Ok(())
    }

    // Public API methods
    pub async fn show_progress<S: Into<String>>(&self, id: S, title: S, total: u64) -> Result<()> {
        let progress = ProgressIndicator::new(id, title, total);
        self.feedback_sender
            .send(FeedbackEvent::ShowProgress(progress))
            .map_err(|e| {
                crate::error::WorkflowError::ValidationError(format!(
                    "Failed to show progress: {}",
                    e
                ))
            })?;
        Ok(())
    }

    pub async fn update_progress<S: Into<String>>(
        &self,
        id: S,
        current: u64,
        message: Option<String>,
    ) -> Result<()> {
        self.feedback_sender
            .send(FeedbackEvent::UpdateProgress {
                id: id.into(),
                current,
                message,
            })
            .map_err(|e| {
                crate::error::WorkflowError::ValidationError(format!(
                    "Failed to update progress: {}",
                    e
                ))
            })?;
        Ok(())
    }

    pub async fn complete_progress<S: Into<String>>(&self, id: S) -> Result<()> {
        self.feedback_sender
            .send(FeedbackEvent::CompleteProgress(id.into()))
            .map_err(|e| {
                crate::error::WorkflowError::ValidationError(format!(
                    "Failed to complete progress: {}",
                    e
                ))
            })?;
        Ok(())
    }

    pub async fn show_notification<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        &self,
        id: S1,
        title: S2,
        message: S3,
        notification_type: NotificationType,
    ) -> Result<()> {
        let notification =
            Notification::new(id.into(), title.into(), message.into(), notification_type);
        self.feedback_sender
            .send(FeedbackEvent::ShowNotification(notification))
            .map_err(|e| {
                crate::error::WorkflowError::ValidationError(format!(
                    "Failed to show notification: {}",
                    e
                ))
            })?;
        Ok(())
    }

    pub async fn show_confirmation<S: Into<String>>(
        &self,
        title: S,
        message: S,
        dialog_type: DialogType,
    ) -> Result<()> {
        let dialog = ConfirmationDialog::new(title, message, dialog_type);
        self.feedback_sender
            .send(FeedbackEvent::ShowDialog(dialog))
            .map_err(|e| {
                crate::error::WorkflowError::ValidationError(format!(
                    "Failed to show dialog: {}",
                    e
                ))
            })?;
        Ok(())
    }

    pub async fn show_status<S: Into<String>>(
        &self,
        message: S,
        status_type: StatusType,
    ) -> Result<()> {
        let status = StatusMessage::new(message, status_type);
        self.feedback_sender
            .send(FeedbackEvent::ShowStatus(status))
            .map_err(|e| {
                crate::error::WorkflowError::ValidationError(format!(
                    "Failed to show status: {}",
                    e
                ))
            })?;
        Ok(())
    }

    pub async fn dismiss_dialog(&self) -> Result<()> {
        self.feedback_sender
            .send(FeedbackEvent::DismissDialog)
            .map_err(|e| {
                crate::error::WorkflowError::ValidationError(format!(
                    "Failed to dismiss dialog: {}",
                    e
                ))
            })?;
        Ok(())
    }

    // Getters for rendering
    pub async fn get_progress_indicators(&self) -> Vec<ProgressIndicator> {
        self.progress_indicators.read().await.clone()
    }

    pub async fn get_notifications(&self) -> Vec<Notification> {
        let mut notifications = self.notifications.write().await;

        // Remove expired notifications
        notifications.retain(|n| !n.is_expired());

        notifications.iter().cloned().collect()
    }

    pub async fn get_current_dialog(&self) -> Option<ConfirmationDialog> {
        self.current_dialog.read().await.clone()
    }

    pub async fn get_status_message(&self) -> Option<StatusMessage> {
        let mut status_guard = self.status_message.write().await;

        // Remove expired status
        if let Some(ref status) = *status_guard {
            if status.is_expired() {
                *status_guard = None;
                return None;
            }
        }

        status_guard.clone()
    }

    pub async fn cleanup_completed_progress(&self) {
        let mut indicators = self.progress_indicators.write().await;
        indicators.retain(|p| {
            match p.status {
                ProgressStatus::Completed | ProgressStatus::Failed | ProgressStatus::Cancelled => {
                    // Keep completed items for a short time for user feedback
                    p.started_at.elapsed() < Duration::from_secs(5)
                }
                _ => true,
            }
        });
    }
}

impl Default for FeedbackManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget for rendering feedback elements
pub struct FeedbackWidget {
    feedback_manager: Arc<FeedbackManager>,
    show_progress: bool,
    show_notifications: bool,
    compact_mode: bool,
}

impl FeedbackWidget {
    pub fn new(feedback_manager: Arc<FeedbackManager>) -> Self {
        Self {
            feedback_manager,
            show_progress: true,
            show_notifications: true,
            compact_mode: false,
        }
    }

    pub fn with_compact_mode(mut self, compact: bool) -> Self {
        self.compact_mode = compact;
        self
    }

    pub async fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        // Render dialog first (modal)
        if let Some(dialog) = self.feedback_manager.get_current_dialog().await {
            self.render_confirmation_dialog(frame, area, &dialog);
            return; // Dialog is modal, don't render other elements
        }

        // Calculate layout for other elements
        let mut constraints = Vec::new();
        let mut _render_areas: Vec<Rect> = Vec::new();

        // Status message area (top)
        if self.feedback_manager.get_status_message().await.is_some() {
            constraints.push(Constraint::Length(1));
        }

        // Progress indicators area
        let progress_indicators = self.feedback_manager.get_progress_indicators().await;
        if self.show_progress && !progress_indicators.is_empty() {
            let progress_height = if self.compact_mode {
                progress_indicators.len().min(3) as u16
            } else {
                progress_indicators.len().min(5) as u16
            };
            constraints.push(Constraint::Length(progress_height * 2));
        }

        // Notifications area (bottom)
        let notifications = self.feedback_manager.get_notifications().await;
        if self.show_notifications && !notifications.is_empty() {
            let notification_height = if self.compact_mode {
                notifications.len().min(2) as u16 * 2
            } else {
                notifications.len().min(3) as u16 * 3
            };
            constraints.push(Constraint::Length(notification_height));
        }

        // Main content area
        constraints.push(Constraint::Min(0));

        if constraints.len() > 1 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            let mut chunk_index = 0;

            // Render status message
            if let Some(status) = self.feedback_manager.get_status_message().await {
                self.render_status_message(frame, chunks[chunk_index], &status);
                chunk_index += 1;
            }

            // Render progress indicators
            if self.show_progress && !progress_indicators.is_empty() {
                self.render_progress_indicators(frame, chunks[chunk_index], &progress_indicators);
                chunk_index += 1;
            }

            // Render notifications
            if self.show_notifications && !notifications.is_empty() {
                self.render_notifications(frame, chunks[chunk_index], &notifications);
            }
        }
    }

    fn render_status_message(&self, frame: &mut Frame, area: Rect, status: &StatusMessage) {
        let text = format!("{} {}", status.status_type.icon(), status.message);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(status.status_type.color()))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn render_progress_indicators(
        &self,
        frame: &mut Frame,
        area: Rect,
        indicators: &[ProgressIndicator],
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2); indicators.len().min(5)])
            .split(area);

        for (i, progress) in indicators.iter().take(5).enumerate() {
            let progress_area = chunks[i];

            // Split into label and progress bar
            let progress_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1)])
                .split(progress_area);

            // Render label
            let label_text = if progress.message.is_empty() {
                format!("{} {}", progress.status.icon(), progress.title)
            } else {
                format!(
                    "{} {} - {}",
                    progress.status.icon(),
                    progress.title,
                    progress.message
                )
            };

            let label =
                Paragraph::new(label_text).style(Style::default().fg(progress.status.color()));
            frame.render_widget(label, progress_chunks[0]);

            // Render progress bar
            let percentage = progress.percentage();
            let gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(progress.status.color()))
                .percent(percentage as u16)
                .label(format!("{:.1}%", percentage));

            frame.render_widget(gauge, progress_chunks[1]);
        }
    }

    fn render_notifications(&self, frame: &mut Frame, area: Rect, notifications: &[Notification]) {
        let items: Vec<ListItem> = notifications
            .iter()
            .take(3)
            .map(|notification| {
                let age = notification.age().as_secs();
                let age_text = if age < 60 {
                    format!("{}s", age)
                } else {
                    format!("{}m", age / 60)
                };

                let content = vec![
                    Line::from(vec![
                        Span::styled(
                            notification.notification_type.icon(),
                            Style::default().fg(notification.notification_type.color()),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            &notification.title,
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(age_text, Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(Span::raw(&notification.message)),
                ];

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("通知"));

        frame.render_widget(list, area);
    }

    fn render_confirmation_dialog(
        &self,
        frame: &mut Frame,
        area: Rect,
        dialog: &ConfirmationDialog,
    ) {
        // Calculate dialog size
        let dialog_width = area.width.min(60);
        let dialog_height = area.height.min(15);

        let dialog_area = Rect {
            x: (area.width.saturating_sub(dialog_width)) / 2,
            y: (area.height.saturating_sub(dialog_height)) / 2,
            width: dialog_width,
            height: dialog_height,
        };

        // Clear background
        frame.render_widget(Clear, dialog_area);

        // Create dialog block
        let title = format!("{} {}", dialog.dialog_type.icon(), dialog.title);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(dialog.dialog_type.color()));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout dialog content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Message
                Constraint::Length(3), // Options
            ])
            .split(inner_area);

        // Render message
        let message_text = Text::from(dialog.message.clone());
        let message_paragraph = Paragraph::new(message_text)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);
        frame.render_widget(message_paragraph, chunks[0]);

        // Render options
        let option_text: Vec<Span> = dialog
            .options
            .iter()
            .enumerate()
            .flat_map(|(i, option)| {
                let style = if i == dialog.selected_option {
                    Style::default()
                        .fg(option.style.color())
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().fg(option.style.color())
                };

                vec![
                    Span::styled(format!("[{}] {}", option.key, option.label), style),
                    Span::raw("  "),
                ]
            })
            .collect();

        let options_paragraph =
            Paragraph::new(Line::from(option_text)).alignment(Alignment::Center);
        frame.render_widget(options_paragraph, chunks[1]);
    }
}

/// Convenience functions for common feedback patterns
impl FeedbackManager {
    pub async fn show_info<S: Into<String>>(&self, title: S, message: S) -> Result<()> {
        self.show_notification(
            "info",
            &title.into(),
            &message.into(),
            NotificationType::Info,
        )
        .await
    }

    pub async fn show_success<S: Into<String>>(&self, title: S, message: S) -> Result<()> {
        self.show_notification(
            "success",
            title.into(),
            message.into(),
            NotificationType::Success,
        )
        .await
    }

    pub async fn show_warning<S: Into<String>>(&self, title: S, message: S) -> Result<()> {
        self.show_notification(
            "warning",
            title.into(),
            message.into(),
            NotificationType::Warning,
        )
        .await
    }

    pub async fn show_error<S: Into<String>>(&self, title: S, message: S) -> Result<()> {
        self.show_notification(
            "error",
            title.into(),
            message.into(),
            NotificationType::Error,
        )
        .await
    }

    pub async fn confirm_action<S: Into<String>>(&self, title: S, message: S) -> Result<()> {
        self.show_confirmation(title, message, DialogType::Confirmation)
            .await
    }

    pub async fn show_loading<S: Into<String>>(&self, message: S) -> Result<()> {
        self.show_status(message, StatusType::Loading).await
    }
}
