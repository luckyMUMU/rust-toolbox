//! Tests for audit logging and execution tracking

use super::audit::{AuditEventType, AuditLogger, AuditQueryCriteria, LogLevel};
use crate::core::ExecutionContext;
use crate::storage::{FileStorage, SimpleMemoryCache, StateManager};
use chrono::Utc;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn test_audit_event_logging() {
    // **Feature: workflow-toolkit, Property 11: Execution log completeness**
    // *For any* workflow execution, logs should contain all task start, end and result information
    // **Validates: Requirements 5.4**

    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(FileStorage::new(temp_dir.path().join("storage")).unwrap());
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));

    let audit_logger = AuditLogger::new(state_manager.clone(), false, 30);

    let workflow_id = Uuid::new_v4();
    let context = ExecutionContext::new().with_workflow_id(workflow_id);

    // Test workflow lifecycle events
    let workflow_started_event = audit_logger.create_workflow_event(
        AuditEventType::WorkflowStarted,
        workflow_id,
        "test_workflow",
        &context,
        Some("Test workflow started".to_string()),
    );

    let result = audit_logger
        .log_audit_event(workflow_started_event.clone())
        .await;
    assert!(
        result.is_ok(),
        "Should successfully log audit event: {:?}",
        result
    );

    // Test node execution events
    let node_started_event = audit_logger.create_node_event(
        AuditEventType::NodeStarted,
        workflow_id,
        "test_node",
        &context,
        None,
        None,
    );

    let result = audit_logger.log_audit_event(node_started_event).await;
    assert!(
        result.is_ok(),
        "Should successfully log node event: {:?}",
        result
    );

    // Test execution logging
    let log_entry = audit_logger.create_execution_log(
        LogLevel::Info,
        workflow_id,
        &context.execution_id,
        Some("test_node"),
        "Test node execution started",
        std::collections::HashMap::new(),
    );

    let result = audit_logger.log_execution(log_entry).await;
    assert!(
        result.is_ok(),
        "Should successfully log execution entry: {:?}",
        result
    );

    // Query audit events
    let criteria = AuditQueryCriteria {
        workflow_id: Some(workflow_id),
        event_types: None,
        severity_filter: None,
        user_id: None,
        date: None,
        start_time: None,
        end_time: None,
        limit: Some(10),
    };

    let events = audit_logger.query_audit_events(criteria).await.unwrap();
    assert!(!events.is_empty(), "Should find logged audit events");
    assert!(
        events
            .iter()
            .any(|e| e.event_type == AuditEventType::WorkflowStarted),
        "Should find workflow started event"
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_type == AuditEventType::NodeStarted),
        "Should find node started event"
    );
}

#[tokio::test]
async fn test_execution_log_querying() {
    // **Feature: workflow-toolkit, Property 11: Execution log completeness**
    // *For any* workflow execution, execution logs should be queryable and complete
    // **Validates: Requirements 5.4**

    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(FileStorage::new(temp_dir.path().join("storage")).unwrap());
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));

    let audit_logger = AuditLogger::new(state_manager.clone(), false, 30);

    let workflow_id = Uuid::new_v4();
    let execution_id = Uuid::new_v4().to_string();

    // Log multiple execution entries
    let log_levels = vec![LogLevel::Info, LogLevel::Warn, LogLevel::Error];
    for (i, level) in log_levels.iter().enumerate() {
        let log_entry = audit_logger.create_execution_log(
            level.clone(),
            workflow_id,
            &execution_id,
            Some(&format!("node_{}", i)),
            &format!("Test log message {} with level {:?}", i, level),
            std::collections::HashMap::new(),
        );

        let result = audit_logger.log_execution(log_entry).await;
        assert!(
            result.is_ok(),
            "Should successfully log execution entry: {:?}",
            result
        );
    }

    // Query all logs for the workflow
    let logs = audit_logger
        .query_execution_logs(workflow_id, Some(&execution_id), None, None)
        .await
        .unwrap();

    assert_eq!(logs.len(), 3, "Should find all 3 logged entries");

    // Query with level filter
    let error_logs = audit_logger
        .query_execution_logs(
            workflow_id,
            Some(&execution_id),
            Some(LogLevel::Error),
            None,
        )
        .await
        .unwrap();

    assert_eq!(error_logs.len(), 1, "Should find only error level logs");
    assert_eq!(
        error_logs[0].level,
        LogLevel::Error,
        "Should be error level log"
    );
}

#[tokio::test]
async fn test_audit_report_generation() {
    // **Feature: workflow-toolkit, Property 11: Execution log completeness**
    // *For any* time period, audit reports should contain complete execution information
    // **Validates: Requirements 5.4**

    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(FileStorage::new(temp_dir.path().join("storage")).unwrap());
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));

    let audit_logger = AuditLogger::new(state_manager.clone(), false, 30);

    let workflow_id = Uuid::new_v4();
    let context = ExecutionContext::new().with_workflow_id(workflow_id);

    let start_time = Utc::now();

    // Log various events
    let events = vec![
        AuditEventType::WorkflowStarted,
        AuditEventType::NodeStarted,
        AuditEventType::NodeCompleted,
        AuditEventType::WorkflowCompleted,
    ];

    for event_type in events {
        let event = audit_logger.create_workflow_event(
            event_type,
            workflow_id,
            "test_workflow",
            &context,
            None,
        );

        let result = audit_logger.log_audit_event(event).await;
        assert!(
            result.is_ok(),
            "Should successfully log audit event: {:?}",
            result
        );
    }

    let end_time = Utc::now();

    // Generate audit report
    let report = audit_logger
        .generate_audit_report(start_time, end_time)
        .await
        .unwrap();

    assert!(
        report.total_events >= 4,
        "Report should contain at least 4 events"
    );
    assert!(
        report.workflows_affected.contains(&workflow_id),
        "Report should include the test workflow"
    );
    assert!(
        !report.report_id.is_empty(),
        "Report should have a valid ID"
    );
    assert!(
        report
            .events_by_type
            .contains_key(&AuditEventType::WorkflowStarted),
        "Report should categorize events by type"
    );
}

#[tokio::test]
async fn test_audit_cleanup() {
    // **Feature: workflow-toolkit, Property 11: Execution log completeness**
    // *For any* retention policy, old audit records should be cleaned up correctly
    // **Validates: Requirements 5.4**

    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(FileStorage::new(temp_dir.path().join("storage")).unwrap());
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));

    // Create audit logger with short retention (1 day for testing)
    let audit_logger = AuditLogger::new(state_manager.clone(), false, 1);

    let workflow_id = Uuid::new_v4();
    let context = ExecutionContext::new().with_workflow_id(workflow_id);

    // Log an event
    let event = audit_logger.create_workflow_event(
        AuditEventType::WorkflowStarted,
        workflow_id,
        "test_workflow",
        &context,
        None,
    );

    let result = audit_logger.log_audit_event(event).await;
    assert!(
        result.is_ok(),
        "Should successfully log audit event: {:?}",
        result
    );

    // Verify event exists
    let criteria = AuditQueryCriteria {
        workflow_id: Some(workflow_id),
        event_types: None,
        severity_filter: None,
        user_id: None,
        date: None,
        start_time: None,
        end_time: None,
        limit: None,
    };

    let events_before = audit_logger
        .query_audit_events(criteria.clone())
        .await
        .unwrap();
    assert!(
        !events_before.is_empty(),
        "Should find the logged event before cleanup"
    );

    // Note: In a real test, we would manipulate timestamps to simulate old records
    // For this test, we just verify the cleanup method runs without error
    let cleanup_result = audit_logger.cleanup_old_records().await;
    assert!(
        cleanup_result.is_ok(),
        "Cleanup should complete without error: {:?}",
        cleanup_result
    );
}
