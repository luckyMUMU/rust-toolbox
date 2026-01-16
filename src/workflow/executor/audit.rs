//! Audit executor implementation.
//!
//! Wraps component execution with audit logging for compliance
//! and debugging purposes.

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::workflow::component::{Component, ComponentOutput, ComponentStatus};
use crate::workflow::context::DataContext;
use crate::workflow::executor::{BoxedExecutor, Executor};
use crate::workflow::{AuditEventType, AuditLogger, ErrorDetails, LogLevel};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

/// Audit executor that logs component execution events.
pub struct AuditExecutor {
    inner: BoxedExecutor,
    audit_logger: Arc<AuditLogger>,
}

impl AuditExecutor {
    /// Create a new audit executor.
    pub fn new(inner: BoxedExecutor, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            inner,
            audit_logger,
        }
    }

    /// Log a component start event.
    async fn log_start(
        &self,
        component: &dyn Component,
        execution_ctx: &ExecutionContext,
    ) -> Result<()> {
        let workflow_id = execution_ctx.workflow_id.unwrap_or_else(uuid::Uuid::nil);
        let event = self.audit_logger.create_node_event(
            AuditEventType::NodeStarted,
            workflow_id,
            component.id(),
            execution_ctx,
            None,
            None,
        );
        self.audit_logger.log_audit_event(event).await
    }

    /// Log a component completion event.
    async fn log_completion(
        &self,
        component: &dyn Component,
        execution_ctx: &ExecutionContext,
        output: &ComponentOutput,
        duration: chrono::Duration,
    ) -> Result<()> {
        let event_type = match &output.status {
            ComponentStatus::Success => AuditEventType::NodeCompleted,
            ComponentStatus::Failure(_) => AuditEventType::NodeFailed,
            ComponentStatus::Skip => AuditEventType::NodeSkipped,
            ComponentStatus::Break | ComponentStatus::Continue => AuditEventType::NodeCompleted,
        };

        let error_details = match &output.status {
            ComponentStatus::Failure(msg) => Some(ErrorDetails {
                error_type: "ComponentFailure".to_string(),
                error_message: msg.clone(),
                stack_trace: None,
                error_code: None,
                retry_count: None,
            }),
            _ => None,
        };

        let workflow_id = execution_ctx.workflow_id.unwrap_or_else(uuid::Uuid::nil);
        let event = self.audit_logger.create_node_event(
            event_type,
            workflow_id,
            component.id(),
            execution_ctx,
            Some(duration),
            error_details,
        );
        self.audit_logger.log_audit_event(event).await
    }

    /// Log an execution error.
    async fn log_error(
        &self,
        component: &dyn Component,
        execution_ctx: &ExecutionContext,
        error: &crate::error::WorkflowError,
        duration: chrono::Duration,
    ) -> Result<()> {
        let error_details = ErrorDetails {
            error_type: "ExecutionError".to_string(),
            error_message: error.to_string(),
            stack_trace: None,
            error_code: None,
            retry_count: None,
        };

        let workflow_id = execution_ctx.workflow_id.unwrap_or_else(uuid::Uuid::nil);
        let event = self.audit_logger.create_node_event(
            AuditEventType::NodeFailed,
            workflow_id,
            component.id(),
            execution_ctx,
            Some(duration),
            Some(error_details),
        );
        self.audit_logger.log_audit_event(event).await
    }
}

#[async_trait]
impl Executor for AuditExecutor {
    async fn execute(
        &self,
        component: &dyn Component,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput> {
        let start_time = Utc::now();

        // Log start event
        if let Err(e) = self.log_start(component, execution_ctx).await {
            tracing::warn!(
                component_id = component.id(),
                error = %e,
                "Failed to log component start event"
            );
        }

        // Execute the component
        let result = self.inner.execute(component, context, execution_ctx).await;

        let duration = Utc::now().signed_duration_since(start_time);

        // Log completion or error
        match &result {
            Ok(output) => {
                if let Err(e) = self
                    .log_completion(component, execution_ctx, output, duration)
                    .await
                {
                    tracing::warn!(
                        component_id = component.id(),
                        error = %e,
                        "Failed to log component completion event"
                    );
                }
            }
            Err(error) => {
                if let Err(e) = self
                    .log_error(component, execution_ctx, error, duration)
                    .await
                {
                    tracing::warn!(
                        component_id = component.id(),
                        error = %e,
                        "Failed to log component error event"
                    );
                }
            }
        }

        // Log execution details
        let log_level = match &result {
            Ok(output) if output.status.is_success() => LogLevel::Info,
            Ok(_) => LogLevel::Warn,
            Err(_) => LogLevel::Error,
        };

        let message = match &result {
            Ok(output) => format!(
                "Component '{}' completed with status {:?} in {:?}",
                component.id(),
                output.status,
                duration
            ),
            Err(e) => format!(
                "Component '{}' failed with error: {} in {:?}",
                component.id(),
                e,
                duration
            ),
        };

        let workflow_id = execution_ctx.workflow_id.unwrap_or_else(uuid::Uuid::nil);
        let log_entry = self.audit_logger.create_execution_log(
            log_level,
            workflow_id,
            &execution_ctx.execution_id,
            Some(component.id()),
            &message,
            std::collections::HashMap::new(),
        );

        if let Err(e) = self.audit_logger.log_execution(log_entry).await {
            tracing::warn!(
                component_id = component.id(),
                error = %e,
                "Failed to log execution details"
            );
        }

        result
    }

    fn name(&self) -> &str {
        "AuditExecutor"
    }
}
