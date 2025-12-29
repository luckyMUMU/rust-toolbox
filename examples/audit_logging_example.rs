//! Example demonstrating audit logging and execution tracking capabilities

use std::sync::Arc;
use tempfile::TempDir;
use tokio;
use uuid::Uuid;

use workflow_toolkit::core::ExecutionContext;
use workflow_toolkit::storage::{StateManager, SimpleMemoryCache, FileStorage};
use workflow_toolkit::workflow::audit::{
    AuditLogger, AuditEventType, LogLevel, AuditQueryCriteria, ErrorDetails
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt::init();

    println!("🔍 Workflow Audit Logging Example");
    println!("==================================");

    // Set up storage and audit logger
    let temp_dir = TempDir::new()?;
    let storage = Arc::new(FileStorage::new(temp_dir.path().join("audit_storage"))?);
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));
    
    // Create audit logger with compliance mode enabled
    let audit_logger = AuditLogger::new(state_manager.clone(), true, 90); // 90 days retention
    
    println!("\n📋 Creating sample workflow execution...");
    
    // Simulate a workflow execution with comprehensive logging
    let workflow_id = Uuid::new_v4();
    let execution_id = Uuid::new_v4().to_string();
    let context = ExecutionContext::new()
        .with_workflow_id(workflow_id)
        .with_user_id("user123");

    // 1. Log workflow creation and start
    println!("  ✅ Logging workflow lifecycle events...");
    
    let workflow_created_event = audit_logger.create_workflow_event(
        AuditEventType::WorkflowCreated,
        workflow_id,
        "data-processing-pipeline",
        &context,
        Some("Data processing pipeline created with 5 nodes".to_string()),
    );
    audit_logger.log_audit_event(workflow_created_event).await?;

    let workflow_started_event = audit_logger.create_workflow_event(
        AuditEventType::WorkflowStarted,
        workflow_id,
        "data-processing-pipeline",
        &context,
        None,
    );
    audit_logger.log_audit_event(workflow_started_event).await?;

    // 2. Log detailed execution steps
    println!("  📝 Logging detailed execution steps...");
    
    let nodes = vec!["data-ingestion", "data-validation", "data-transformation", "data-analysis", "data-export"];
    
    for (i, node_id) in nodes.iter().enumerate() {
        // Log node start
        let node_started_event = audit_logger.create_node_event(
            AuditEventType::NodeStarted,
            workflow_id,
            node_id,
            &context,
            None,
            None,
        );
        audit_logger.log_audit_event(node_started_event).await?;

        // Log execution details
        let log_entry = audit_logger.create_execution_log(
            LogLevel::Info,
            workflow_id,
            &execution_id,
            Some(node_id),
            &format!("Starting execution of {} node (step {} of {})", node_id, i + 1, nodes.len()),
            std::collections::HashMap::new(),
        );
        audit_logger.log_execution(log_entry).await?;

        // Simulate some processing time and different outcomes
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        if node_id == &"data-validation" {
            // Simulate a retry scenario
            let retry_event = audit_logger.create_node_event(
                AuditEventType::NodeRetried,
                workflow_id,
                node_id,
                &context,
                None,
                Some(ErrorDetails {
                    error_type: "ValidationError".to_string(),
                    error_message: "Invalid data format detected, retrying with fallback parser".to_string(),
                    stack_trace: None,
                    error_code: Some("VAL_001".to_string()),
                    retry_count: Some(1),
                }),
            );
            audit_logger.log_audit_event(retry_event).await?;

            let retry_log = audit_logger.create_execution_log(
                LogLevel::Warn,
                workflow_id,
                &execution_id,
                Some(node_id),
                "Data validation failed, retrying with fallback parser",
                std::collections::HashMap::new(),
            );
            audit_logger.log_execution(retry_log).await?;
        }

        // Log successful completion
        let duration = chrono::Duration::milliseconds(100 + (i as i64 * 50));
        let node_completed_event = audit_logger.create_node_event(
            AuditEventType::NodeCompleted,
            workflow_id,
            node_id,
            &context,
            Some(duration),
            None,
        );
        audit_logger.log_audit_event(node_completed_event).await?;

        let completion_log = audit_logger.create_execution_log(
            LogLevel::Info,
            workflow_id,
            &execution_id,
            Some(node_id),
            &format!("Node {} completed successfully in {:?}", node_id, duration),
            std::collections::HashMap::new(),
        );
        audit_logger.log_execution(completion_log).await?;
    }

    // 3. Log workflow completion
    println!("  🎉 Logging workflow completion...");
    
    let workflow_completed_event = audit_logger.create_workflow_event(
        AuditEventType::WorkflowCompleted,
        workflow_id,
        "data-processing-pipeline",
        &context,
        Some("Data processing pipeline completed successfully".to_string()),
    );
    audit_logger.log_audit_event(workflow_completed_event).await?;

    let final_log = audit_logger.create_execution_log(
        LogLevel::Info,
        workflow_id,
        &execution_id,
        None,
        "Workflow execution completed successfully - all 5 nodes processed",
        std::collections::HashMap::new(),
    );
    audit_logger.log_execution(final_log).await?;

    // 4. Query and display audit events
    println!("\n🔍 Querying audit events...");
    
    let criteria = AuditQueryCriteria {
        workflow_id: Some(workflow_id),
        event_types: None,
        severity_filter: None,
        user_id: None,
        date: None,
        start_time: None,
        end_time: None,
        limit: Some(20),
    };
    
    let events = audit_logger.query_audit_events(criteria).await?;
    println!("  📊 Found {} audit events for workflow {}", events.len(), workflow_id);
    
    for event in &events {
        println!("    • {} - {:?} - {} ({})", 
                event.timestamp.format("%H:%M:%S%.3f"),
                event.event_type,
                event.description,
                event.severity as u8);
    }

    // 5. Query execution logs
    println!("\n📋 Querying execution logs...");
    
    let logs = audit_logger.query_execution_logs(
        workflow_id,
        Some(&execution_id),
        None,
        Some(10),
    ).await?;
    
    println!("  📝 Found {} execution log entries", logs.len());
    
    for log in &logs {
        println!("    • {} - {:?} - {} - {}", 
                log.timestamp.format("%H:%M:%S%.3f"),
                log.level,
                log.node_id.as_deref().unwrap_or("workflow"),
                log.message);
    }

    // 6. Generate audit report
    println!("\n📊 Generating audit report...");
    
    let start_time = chrono::Utc::now() - chrono::Duration::hours(1);
    let end_time = chrono::Utc::now();
    
    let report = audit_logger.generate_audit_report(start_time, end_time).await?;
    
    println!("  📈 Audit Report Summary:");
    println!("    • Report ID: {}", report.report_id);
    println!("    • Total Events: {}", report.total_events);
    println!("    • Workflows Affected: {}", report.workflows_affected.len());
    println!("    • Users Involved: {}", report.users_involved.len());
    
    println!("    • Events by Type:");
    for (event_type, count) in &report.events_by_type {
        println!("      - {:?}: {}", event_type, count);
    }
    
    println!("    • Events by Severity:");
    for (severity, count) in &report.events_by_severity {
        println!("      - {:?}: {}", severity, count);
    }

    // 7. Demonstrate compliance features
    println!("\n🔒 Compliance and Security Features:");
    println!("  • All events are immutably stored with timestamps");
    println!("  • User actions are tracked and attributed");
    println!("  • Error details are captured for forensic analysis");
    println!("  • Audit trail supports regulatory compliance");
    
    // 8. Cleanup demonstration
    println!("\n🧹 Cleanup capabilities:");
    let cleanup_count = audit_logger.cleanup_old_records().await?;
    println!("  • Cleaned up {} old records (retention policy: 90 days)", cleanup_count);

    println!("\n✅ Audit logging example completed successfully!");
    println!("   All workflow execution details have been logged and are available for:");
    println!("   • Compliance reporting");
    println!("   • Performance analysis");
    println!("   • Error investigation");
    println!("   • Security auditing");

    Ok(())
}