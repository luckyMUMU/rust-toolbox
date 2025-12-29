//! Execution logging and audit tracking for workflows

use crate::core::{ExecutionContext, WorkflowId};
use crate::error::Result;
use crate::storage::StateManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Audit event types for tracking workflow operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditEventType {
    /// Workflow lifecycle events
    WorkflowCreated,
    WorkflowStarted,
    WorkflowPaused,
    WorkflowResumed,
    WorkflowStopped,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowCancelled,
    
    /// Node execution events
    NodeStarted,
    NodeCompleted,
    NodeFailed,
    NodeRetried,
    NodeSkipped,
    
    /// System events
    CheckpointCreated,
    StateRecovered,
    ErrorOccurred,
    
    /// Security events
    AccessGranted,
    AccessDenied,
    AuthenticationFailed,
    
    /// Configuration events
    ConfigurationChanged,
    PluginLoaded,
    PluginUnloaded,
}

/// Severity levels for audit events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Detailed audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: String,
    /// Event type
    pub event_type: AuditEventType,
    /// Event severity
    pub severity: AuditSeverity,
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    /// Workflow ID (if applicable)
    pub workflow_id: Option<WorkflowId>,
    /// Node ID (if applicable)
    pub node_id: Option<String>,
    /// User ID who triggered the event
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Event description
    pub description: String,
    /// Additional event data
    pub metadata: HashMap<String, Value>,
    /// Source component that generated the event
    pub source: String,
    /// Duration of the operation (if applicable)
    pub duration: Option<chrono::Duration>,
    /// Error details (if applicable)
    pub error_details: Option<ErrorDetails>,
}

/// Error details for audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub error_code: Option<String>,
    pub retry_count: Option<u32>,
}

/// Execution log entry for detailed operation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogEntry {
    /// Log entry ID
    pub log_id: String,
    /// Workflow execution ID
    pub execution_id: String,
    /// Workflow ID
    pub workflow_id: WorkflowId,
    /// Node ID (if applicable)
    pub node_id: Option<String>,
    /// Log level
    pub level: LogLevel,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Log message
    pub message: String,
    /// Structured data
    pub data: HashMap<String, Value>,
    /// Source location (file, line, etc.)
    pub source_location: Option<String>,
}

/// Log levels for execution logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Audit and logging manager
pub struct AuditLogger {
    state_manager: Arc<StateManager>,
    compliance_mode: bool,
    retention_days: u32,
    enable_detailed_logging: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(
        state_manager: Arc<StateManager>,
        compliance_mode: bool,
        retention_days: u32,
    ) -> Self {
        Self {
            state_manager,
            compliance_mode,
            retention_days,
            enable_detailed_logging: true,
        }
    }

    /// Log an audit event
    pub async fn log_audit_event(&self, event: AuditEvent) -> Result<()> {
        // Log to structured logging system
        match event.severity {
            AuditSeverity::Debug => debug!(
                event_id = %event.event_id,
                event_type = ?event.event_type,
                workflow_id = ?event.workflow_id,
                node_id = ?event.node_id,
                user_id = ?event.user_id,
                "{}",
                event.description
            ),
            AuditSeverity::Info => info!(
                event_id = %event.event_id,
                event_type = ?event.event_type,
                workflow_id = ?event.workflow_id,
                node_id = ?event.node_id,
                user_id = ?event.user_id,
                "{}",
                event.description
            ),
            AuditSeverity::Warning => warn!(
                event_id = %event.event_id,
                event_type = ?event.event_type,
                workflow_id = ?event.workflow_id,
                node_id = ?event.node_id,
                user_id = ?event.user_id,
                error_details = ?event.error_details,
                "{}",
                event.description
            ),
            AuditSeverity::Error | AuditSeverity::Critical => error!(
                event_id = %event.event_id,
                event_type = ?event.event_type,
                workflow_id = ?event.workflow_id,
                node_id = ?event.node_id,
                user_id = ?event.user_id,
                error_details = ?event.error_details,
                "{}",
                event.description
            ),
        }

        // Store audit event for compliance and querying
        let key = format!("audit:event:{}:{}", event.timestamp.format("%Y%m%d"), event.event_id);
        let value = serde_json::to_vec(&event)?;
        self.state_manager.storage.save(&key, &value).await?;

        // If in compliance mode, also store in immutable audit log
        if self.compliance_mode {
            self.store_compliance_record(&event).await?;
        }

        Ok(())
    }

    /// Log an execution log entry
    pub async fn log_execution(&self, entry: ExecutionLogEntry) -> Result<()> {
        if !self.enable_detailed_logging {
            return Ok(());
        }

        // Log to structured logging system
        match entry.level {
            LogLevel::Trace => tracing::trace!(
                log_id = %entry.log_id,
                execution_id = %entry.execution_id,
                workflow_id = %entry.workflow_id,
                node_id = ?entry.node_id,
                data = ?entry.data,
                "{}",
                entry.message
            ),
            LogLevel::Debug => debug!(
                log_id = %entry.log_id,
                execution_id = %entry.execution_id,
                workflow_id = %entry.workflow_id,
                node_id = ?entry.node_id,
                data = ?entry.data,
                "{}",
                entry.message
            ),
            LogLevel::Info => info!(
                log_id = %entry.log_id,
                execution_id = %entry.execution_id,
                workflow_id = %entry.workflow_id,
                node_id = ?entry.node_id,
                data = ?entry.data,
                "{}",
                entry.message
            ),
            LogLevel::Warn => warn!(
                log_id = %entry.log_id,
                execution_id = %entry.execution_id,
                workflow_id = %entry.workflow_id,
                node_id = ?entry.node_id,
                data = ?entry.data,
                "{}",
                entry.message
            ),
            LogLevel::Error => error!(
                log_id = %entry.log_id,
                execution_id = %entry.execution_id,
                workflow_id = %entry.workflow_id,
                node_id = ?entry.node_id,
                data = ?entry.data,
                "{}",
                entry.message
            ),
        }

        // Store execution log entry
        let key = format!(
            "execution:log:{}:{}:{}",
            entry.workflow_id,
            entry.execution_id,
            entry.log_id
        );
        let value = serde_json::to_vec(&entry)?;
        self.state_manager.storage.save(&key, &value).await?;

        Ok(())
    }

    /// Create audit event for workflow lifecycle
    pub fn create_workflow_event(
        &self,
        event_type: AuditEventType,
        workflow_id: WorkflowId,
        workflow_name: &str,
        context: &ExecutionContext,
        description: Option<String>,
    ) -> AuditEvent {
        let description = description.unwrap_or_else(|| {
            format!("Workflow '{}' {:?}", workflow_name, event_type)
        });

        let severity = match event_type {
            AuditEventType::WorkflowFailed | AuditEventType::WorkflowCancelled => AuditSeverity::Error,
            AuditEventType::WorkflowStarted | AuditEventType::WorkflowCompleted => AuditSeverity::Info,
            _ => AuditSeverity::Debug,
        };

        let mut metadata = HashMap::new();
        metadata.insert("workflow_name".to_string(), Value::String(workflow_name.to_string()));
        metadata.insert("execution_id".to_string(), Value::String(context.execution_id.clone()));

        AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            severity,
            timestamp: Utc::now(),
            workflow_id: Some(workflow_id),
            node_id: None,
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            description,
            metadata,
            source: "workflow_engine".to_string(),
            duration: None,
            error_details: None,
        }
    }

    /// Create audit event for node execution
    pub fn create_node_event(
        &self,
        event_type: AuditEventType,
        workflow_id: WorkflowId,
        node_id: &str,
        context: &ExecutionContext,
        duration: Option<chrono::Duration>,
        error_details: Option<ErrorDetails>,
    ) -> AuditEvent {
        let description = format!("Node '{}' {:?}", node_id, event_type);

        let severity = match event_type {
            AuditEventType::NodeFailed => AuditSeverity::Error,
            AuditEventType::NodeRetried => AuditSeverity::Warning,
            AuditEventType::NodeStarted | AuditEventType::NodeCompleted => AuditSeverity::Info,
            _ => AuditSeverity::Debug,
        };

        let mut metadata = HashMap::new();
        metadata.insert("execution_id".to_string(), Value::String(context.execution_id.clone()));
        if let Some(duration) = duration {
            metadata.insert("duration_ms".to_string(), Value::Number(
                serde_json::Number::from(duration.num_milliseconds())
            ));
        }

        AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            severity,
            timestamp: Utc::now(),
            workflow_id: Some(workflow_id),
            node_id: Some(node_id.to_string()),
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            description,
            metadata,
            source: "node_executor".to_string(),
            duration,
            error_details,
        }
    }

    /// Create execution log entry
    pub fn create_execution_log(
        &self,
        level: LogLevel,
        workflow_id: WorkflowId,
        execution_id: &str,
        node_id: Option<&str>,
        message: &str,
        data: HashMap<String, Value>,
    ) -> ExecutionLogEntry {
        ExecutionLogEntry {
            log_id: Uuid::new_v4().to_string(),
            execution_id: execution_id.to_string(),
            workflow_id,
            node_id: node_id.map(|s| s.to_string()),
            level,
            timestamp: Utc::now(),
            message: message.to_string(),
            data,
            source_location: None,
        }
    }

    /// Query audit events by criteria
    pub async fn query_audit_events(
        &self,
        criteria: AuditQueryCriteria,
    ) -> Result<Vec<AuditEvent>> {
        let mut events = Vec::new();
        
        // Build key prefix based on criteria
        let prefix = if let Some(date) = criteria.date {
            format!("audit:event:{}", date.format("%Y%m%d"))
        } else {
            "audit:event:".to_string()
        };

        let keys = self.state_manager.storage.list_keys(&prefix).await?;
        let values = self.state_manager.storage.batch_load(keys).await?;

        for value_opt in values {
            if let Some(value) = value_opt {
                if let Ok(event) = serde_json::from_slice::<AuditEvent>(&value) {
                    if self.matches_criteria(&event, &criteria) {
                        events.push(event);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = criteria.limit {
            events.truncate(limit);
        }

        Ok(events)
    }

    /// Query execution logs by criteria
    pub async fn query_execution_logs(
        &self,
        workflow_id: WorkflowId,
        execution_id: Option<&str>,
        level_filter: Option<LogLevel>,
        limit: Option<usize>,
    ) -> Result<Vec<ExecutionLogEntry>> {
        let mut logs = Vec::new();
        
        let prefix = if let Some(exec_id) = execution_id {
            format!("execution:log:{}:{}", workflow_id, exec_id)
        } else {
            format!("execution:log:{}", workflow_id)
        };

        let keys = self.state_manager.storage.list_keys(&prefix).await?;
        let values = self.state_manager.storage.batch_load(keys).await?;

        for value_opt in values {
            if let Some(value) = value_opt {
                if let Ok(log_entry) = serde_json::from_slice::<ExecutionLogEntry>(&value) {
                    if let Some(filter_level) = &level_filter {
                        if log_entry.level < *filter_level {
                            continue;
                        }
                    }
                    logs.push(log_entry);
                }
            }
        }

        // Sort by timestamp (newest first)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        Ok(logs)
    }

    /// Generate audit report for compliance
    pub async fn generate_audit_report(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<AuditReport> {
        let criteria = AuditQueryCriteria {
            workflow_id: None,
            event_types: None,
            severity_filter: None,
            user_id: None,
            date: None,
            start_time: Some(start_date),
            end_time: Some(end_date),
            limit: None,
        };

        let events = self.query_audit_events(criteria).await?;
        
        let mut report = AuditReport {
            report_id: Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            start_date,
            end_date,
            total_events: events.len(),
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            workflows_affected: std::collections::HashSet::new(),
            users_involved: std::collections::HashSet::new(),
            error_summary: Vec::new(),
        };

        // Analyze events
        for event in &events {
            // Count by type
            *report.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
            
            // Count by severity
            *report.events_by_severity.entry(event.severity.clone()).or_insert(0) += 1;
            
            // Track workflows and users
            if let Some(workflow_id) = event.workflow_id {
                report.workflows_affected.insert(workflow_id);
            }
            if let Some(user_id) = &event.user_id {
                report.users_involved.insert(user_id.clone());
            }
            
            // Collect error summaries
            if event.severity >= AuditSeverity::Error {
                if let Some(error_details) = &event.error_details {
                    report.error_summary.push(ErrorSummary {
                        timestamp: event.timestamp,
                        workflow_id: event.workflow_id,
                        node_id: event.node_id.clone(),
                        error_type: error_details.error_type.clone(),
                        error_message: error_details.error_message.clone(),
                    });
                }
            }
        }

        Ok(report)
    }

    /// Clean up old audit records based on retention policy
    pub async fn cleanup_old_records(&self) -> Result<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.retention_days as i64);
        let mut deleted_count = 0;

        // Clean up audit events
        let audit_keys = self.state_manager.storage.list_keys("audit:event:").await?;
        for key in audit_keys {
            if let Some(value) = self.state_manager.storage.load(&key).await? {
                if let Ok(event) = serde_json::from_slice::<AuditEvent>(&value) {
                    if event.timestamp < cutoff_date {
                        self.state_manager.storage.delete(&key).await?;
                        deleted_count += 1;
                    }
                }
            }
        }

        // Clean up execution logs
        let log_keys = self.state_manager.storage.list_keys("execution:log:").await?;
        for key in log_keys {
            if let Some(value) = self.state_manager.storage.load(&key).await? {
                if let Ok(log_entry) = serde_json::from_slice::<ExecutionLogEntry>(&value) {
                    if log_entry.timestamp < cutoff_date {
                        self.state_manager.storage.delete(&key).await?;
                        deleted_count += 1;
                    }
                }
            }
        }

        info!("Cleaned up {} old audit and log records", deleted_count);
        Ok(deleted_count)
    }

    /// Store compliance record (immutable audit trail)
    async fn store_compliance_record(&self, event: &AuditEvent) -> Result<()> {
        // In a real implementation, this would store to an immutable audit log
        // such as a blockchain, write-only database, or secure log service
        let compliance_key = format!(
            "compliance:audit:{}:{}",
            event.timestamp.format("%Y%m%d%H%M%S"),
            event.event_id
        );
        
        // Create compliance record with hash for integrity
        let compliance_record = ComplianceRecord {
            event: event.clone(),
            hash: self.calculate_event_hash(event),
            stored_at: Utc::now(),
        };
        
        let value = serde_json::to_vec(&compliance_record)?;
        self.state_manager.storage.save(&compliance_key, &value).await?;
        
        Ok(())
    }

    /// Calculate hash for event integrity
    fn calculate_event_hash(&self, event: &AuditEvent) -> String {
        // Simple hash implementation - in production, use cryptographic hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        event.event_id.hash(&mut hasher);
        event.timestamp.hash(&mut hasher);
        event.description.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }

    /// Check if event matches query criteria
    fn matches_criteria(&self, event: &AuditEvent, criteria: &AuditQueryCriteria) -> bool {
        if let Some(workflow_id) = criteria.workflow_id {
            if event.workflow_id != Some(workflow_id) {
                return false;
            }
        }

        if let Some(event_types) = &criteria.event_types {
            if !event_types.contains(&event.event_type) {
                return false;
            }
        }

        if let Some(severity_filter) = &criteria.severity_filter {
            if event.severity < *severity_filter {
                return false;
            }
        }

        if let Some(user_id) = &criteria.user_id {
            if event.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        if let Some(start_time) = criteria.start_time {
            if event.timestamp < start_time {
                return false;
            }
        }

        if let Some(end_time) = criteria.end_time {
            if event.timestamp > end_time {
                return false;
            }
        }

        true
    }
}

/// Query criteria for audit events
#[derive(Debug, Clone)]
pub struct AuditQueryCriteria {
    pub workflow_id: Option<WorkflowId>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub severity_filter: Option<AuditSeverity>,
    pub user_id: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// Audit report for compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_events: usize,
    pub events_by_type: HashMap<AuditEventType, usize>,
    pub events_by_severity: HashMap<AuditSeverity, usize>,
    pub workflows_affected: std::collections::HashSet<WorkflowId>,
    pub users_involved: std::collections::HashSet<String>,
    pub error_summary: Vec<ErrorSummary>,
}

/// Error summary for audit reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub timestamp: DateTime<Utc>,
    pub workflow_id: Option<WorkflowId>,
    pub node_id: Option<String>,
    pub error_type: String,
    pub error_message: String,
}

/// Compliance record for immutable audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplianceRecord {
    pub event: AuditEvent,
    pub hash: String,
    pub stored_at: DateTime<Utc>,
}

/// Convenience macros for logging
#[macro_export]
macro_rules! audit_info {
    ($logger:expr, $event_type:expr, $workflow_id:expr, $context:expr, $($arg:tt)*) => {
        {
            let event = $logger.create_workflow_event(
                $event_type,
                $workflow_id,
                "workflow",
                $context,
                Some(format!($($arg)*))
            );
            let _ = $logger.log_audit_event(event).await;
        }
    };
}

#[macro_export]
macro_rules! audit_error {
    ($logger:expr, $event_type:expr, $workflow_id:expr, $context:expr, $error:expr, $($arg:tt)*) => {
        {
            let error_details = Some(crate::workflow::audit::ErrorDetails {
                error_type: std::any::type_name_of_val(&$error).to_string(),
                error_message: $error.to_string(),
                stack_trace: None,
                error_code: None,
                retry_count: None,
            });
            
            let mut event = $logger.create_workflow_event(
                $event_type,
                $workflow_id,
                "workflow",
                $context,
                Some(format!($($arg)*))
            );
            event.severity = crate::workflow::audit::AuditSeverity::Error;
            event.error_details = error_details;
            
            let _ = $logger.log_audit_event(event).await;
        }
    };
}

#[macro_export]
macro_rules! execution_log {
    ($logger:expr, $level:expr, $workflow_id:expr, $execution_id:expr, $node_id:expr, $($arg:tt)*) => {
        {
            let log_entry = $logger.create_execution_log(
                $level,
                $workflow_id,
                $execution_id,
                $node_id,
                &format!($($arg)*),
                std::collections::HashMap::new()
            );
            let _ = $logger.log_execution(log_entry).await;
        }
    };
}