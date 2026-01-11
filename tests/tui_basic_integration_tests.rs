//! Basic TUI Integration Tests
//!
//! This module contains basic integration tests for the TUI system components
//! that can be tested without the full TUI application running.

use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use workflow_toolkit::error::Result;
use workflow_toolkit::interfaces::tui::{
    action::LogLevel,
    state::{ConnectionStatus, NetworkStatus, StateChangeEvent, SystemHealth, SystemStatus},
    widgets::log_viewer::LogEntry,
    widgets::workflow_list::{WorkflowInfo, WorkflowStatus},
    SharedAppState,
};

/// Test fixture for basic TUI integration testing
struct BasicTuiFixture {
    shared_state: Arc<SharedAppState>,
}

impl BasicTuiFixture {
    /// Create a new basic integration test fixture
    async fn new() -> Result<Self> {
        let shared_state = Arc::new(SharedAppState::new());

        Ok(Self { shared_state })
    }

    /// Populate test data
    async fn populate_test_data(&self) -> Result<()> {
        // Add test workflows
        let workflows = vec![
            WorkflowInfo {
                name: "Integration Test Workflow 1".to_string(),
                version: "1.0.0".to_string(),
                description: Some("First integration test workflow".to_string()),
                status: WorkflowStatus::Available,
                last_execution: Some(Utc::now() - chrono::Duration::hours(1)),
                execution_count: 5,
                tags: vec!["test".to_string(), "integration".to_string()],
                node_count: 3,
                estimated_duration: Some(Duration::from_secs(60)),
                success_rate: Some(0.95),
            },
            WorkflowInfo {
                name: "Integration Test Workflow 2".to_string(),
                version: "2.0.0".to_string(),
                description: Some("Second integration test workflow".to_string()),
                status: WorkflowStatus::Running,
                last_execution: Some(Utc::now() - chrono::Duration::minutes(30)),
                execution_count: 12,
                tags: vec!["test".to_string(), "active".to_string()],
                node_count: 5,
                estimated_duration: Some(Duration::from_secs(120)),
                success_rate: Some(0.88),
            },
        ];

        self.shared_state.set_workflows(workflows).await?;

        // Add test log entries
        let logs = vec![
            LogEntry {
                id: "integration-log-1".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(5),
                level: LogLevel::Info,
                message: "Basic integration test started".to_string(),
                source: Some("test_suite".to_string()),
                execution_id: None,
                workflow_id: None,
                node_id: None,
            },
            LogEntry {
                id: "integration-log-2".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(3),
                level: LogLevel::Debug,
                message: "Test workflow execution initiated".to_string(),
                source: Some("workflow_engine".to_string()),
                execution_id: Some("test-exec-1".to_string()),
                workflow_id: Some("test-workflow-1".to_string()),
                node_id: Some("node-1".to_string()),
            },
            LogEntry {
                id: "integration-log-3".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(1),
                level: LogLevel::Warn,
                message: "Test warning message for integration".to_string(),
                source: Some("test_monitor".to_string()),
                execution_id: Some("test-exec-1".to_string()),
                workflow_id: Some("test-workflow-1".to_string()),
                node_id: Some("node-2".to_string()),
            },
        ];

        for log in logs {
            self.shared_state.add_log_entry(log).await?;
        }

        // Set test system status
        let system_status = SystemStatus {
            cpu_usage: 45.2,
            memory_usage: 62.8,
            memory_total: 16_000_000_000,
            memory_used: 10_048_000_000,
            disk_usage: 75.5,
            disk_total: 1_000_000_000_000,
            disk_used: 755_000_000_000,
            active_workflows: 2,
            system_health: SystemHealth::Warning,
            uptime: Duration::from_secs(3600 * 24 * 7), // 1 week
            network_status: NetworkStatus::Connected,
            load_average: [2.1, 1.8, 1.5],
            process_count: 234,
            thread_count: 1456,
        };

        self.shared_state.set_system_status(system_status).await?;

        Ok(())
    }
}

/// Test suite for shared state integration
#[cfg(test)]
mod shared_state_tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_shared_state_basic_operations() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;
        fixture.populate_test_data().await?;

        // Test workflow operations
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), 2);
        assert_eq!(workflows[0].name, "Integration Test Workflow 1");
        assert_eq!(workflows[1].name, "Integration Test Workflow 2");

        // Test log operations
        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].message, "Basic integration test started");

        // Test system status
        let system_status = fixture.shared_state.get_system_status().await;
        assert_eq!(system_status.active_workflows, 2);
        assert_eq!(system_status.system_health, SystemHealth::Warning);

        Ok(())
    }

    #[tokio::test]
    async fn test_state_change_events() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Subscribe to state changes
        let mut state_receiver = fixture.shared_state.subscribe();

        // Add a workflow and verify event is broadcast
        let new_workflow = WorkflowInfo {
            name: "Event Test Workflow".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Workflow for testing events".to_string()),
            status: WorkflowStatus::Available,
            last_execution: None,
            execution_count: 0,
            tags: vec!["event".to_string(), "test".to_string()],
            node_count: 2,
            estimated_duration: Some(Duration::from_secs(30)),
            success_rate: None,
        };

        fixture
            .shared_state
            .add_workflow(new_workflow.clone())
            .await?;

        // Verify event was broadcast
        let event = timeout(Duration::from_millis(100), state_receiver.recv()).await;
        assert!(event.is_ok());

        if let Ok(Ok(StateChangeEvent::WorkflowAdded { workflow })) = event {
            assert_eq!(workflow.name, "Event Test Workflow");
        } else {
            panic!("Expected WorkflowAdded event");
        }

        // Update workflow status and verify event
        fixture
            .shared_state
            .update_workflow_status("Event Test Workflow", WorkflowStatus::Running)
            .await?;

        let status_event = timeout(Duration::from_millis(100), state_receiver.recv()).await;
        assert!(status_event.is_ok());

        if let Ok(Ok(StateChangeEvent::WorkflowStatusChanged { name, status })) = status_event {
            assert_eq!(name, "Event Test Workflow");
            assert_eq!(status, WorkflowStatus::Running);
        } else {
            panic!("Expected WorkflowStatusChanged event");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_state_access() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test concurrent access to shared state
        let shared_state = Arc::clone(&fixture.shared_state);

        // Spawn multiple tasks that access state concurrently
        let mut handles = Vec::new();

        for i in 0..5 {
            let state = Arc::clone(&shared_state);
            let handle = tokio::spawn(async move {
                // Add workflow
                let workflow = WorkflowInfo {
                    name: format!("Concurrent Workflow {}", i),
                    version: "1.0.0".to_string(),
                    description: Some(format!("Workflow created by task {}", i)),
                    status: WorkflowStatus::Available,
                    last_execution: None,
                    execution_count: 0,
                    tags: vec![format!("task-{}", i)],
                    node_count: 1,
                    estimated_duration: Some(Duration::from_secs(30)),
                    success_rate: None,
                };

                state.add_workflow(workflow).await?;

                // Add log entry
                let log = LogEntry {
                    id: format!("concurrent-log-{}", i),
                    timestamp: Utc::now(),
                    level: LogLevel::Info,
                    message: format!("Concurrent log from task {}", i),
                    source: Some("concurrent_test".to_string()),
                    execution_id: None,
                    workflow_id: None,
                    node_id: None,
                };

                state.add_log_entry(log).await?;

                Ok::<(), workflow_toolkit::error::WorkflowError>(())
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.map_err(|e| {
                workflow_toolkit::error::WorkflowError::ValidationError(format!(
                    "Task join error: {}",
                    e
                ))
            })??;
        }

        // Verify all data was added correctly
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), 5);

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 5);

        // Verify data integrity
        for i in 0..5 {
            assert!(workflows
                .iter()
                .any(|w| w.name == format!("Concurrent Workflow {}", i)));
            assert!(logs.iter().any(|l| l.id == format!("concurrent-log-{}", i)));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_data_persistence_and_retrieval() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;
        fixture.populate_test_data().await?;

        // Test workflow data persistence
        let initial_workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(initial_workflows.len(), 2);

        // Add more workflows
        for i in 0..3 {
            let workflow = WorkflowInfo {
                name: format!("Persistence Test Workflow {}", i),
                version: "1.0.0".to_string(),
                description: Some(format!("Workflow for persistence test {}", i)),
                status: WorkflowStatus::Available,
                last_execution: None,
                execution_count: 0,
                tags: vec!["persistence".to_string(), "test".to_string()],
                node_count: 1,
                estimated_duration: Some(Duration::from_secs(60)),
                success_rate: None,
            };

            fixture.shared_state.add_workflow(workflow).await?;
        }

        // Verify persistence
        let updated_workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(updated_workflows.len(), 5);

        // Test workflow removal
        fixture
            .shared_state
            .remove_workflow("Persistence Test Workflow 1")
            .await?;

        let final_workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(final_workflows.len(), 4);
        assert!(!final_workflows
            .iter()
            .any(|w| w.name == "Persistence Test Workflow 1"));

        // Test log data persistence
        let initial_logs = fixture.shared_state.get_logs().await;
        assert_eq!(initial_logs.len(), 3);

        // Add more logs
        for i in 0..10 {
            let log = LogEntry {
                id: format!("persistence-log-{}", i),
                timestamp: Utc::now(),
                level: LogLevel::Info,
                message: format!("Persistence test log {}", i),
                source: Some("persistence_test".to_string()),
                execution_id: None,
                workflow_id: None,
                node_id: None,
            };

            fixture.shared_state.add_log_entry(log).await?;
        }

        let updated_logs = fixture.shared_state.get_logs().await;
        assert_eq!(updated_logs.len(), 13);

        Ok(())
    }

    #[tokio::test]
    async fn test_connection_status_management() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test initial connection status
        let initial_status = fixture.shared_state.get_connection_status().await;
        assert_eq!(initial_status, ConnectionStatus::Disconnected);

        let status_string = fixture.shared_state.connection_status_string().await;
        assert_eq!(status_string, "未连接");

        // Test status changes
        fixture
            .shared_state
            .set_connection_status(ConnectionStatus::Connecting)
            .await?;
        let connecting_status = fixture.shared_state.get_connection_status().await;
        assert_eq!(connecting_status, ConnectionStatus::Connecting);

        let connecting_string = fixture.shared_state.connection_status_string().await;
        assert_eq!(connecting_string, "连接中...");

        fixture
            .shared_state
            .set_connection_status(ConnectionStatus::Connected)
            .await?;
        let connected_status = fixture.shared_state.get_connection_status().await;
        assert_eq!(connected_status, ConnectionStatus::Connected);

        let connected_string = fixture.shared_state.connection_status_string().await;
        assert_eq!(connected_string, "已连接");

        // Test error status
        fixture
            .shared_state
            .set_connection_status(ConnectionStatus::Error("Test error".to_string()))
            .await?;
        let error_status = fixture.shared_state.get_connection_status().await;
        assert_eq!(
            error_status,
            ConnectionStatus::Error("Test error".to_string())
        );

        let error_string = fixture.shared_state.connection_status_string().await;
        assert_eq!(error_string, "错误: Test error");

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_metrics_tracking() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test initial metrics
        let initial_metrics = fixture.shared_state.get_metrics().await;
        assert_eq!(initial_metrics.widget_update_count.len(), 0);

        // Record some widget updates
        fixture
            .shared_state
            .record_widget_update("workflow_list")
            .await?;
        fixture
            .shared_state
            .record_widget_update("log_viewer")
            .await?;
        fixture
            .shared_state
            .record_widget_update("workflow_list")
            .await?;
        fixture
            .shared_state
            .record_widget_update("system_status")
            .await?;

        let updated_metrics = fixture.shared_state.get_metrics().await;
        assert_eq!(
            updated_metrics.widget_update_count.get("workflow_list"),
            Some(&2)
        );
        assert_eq!(
            updated_metrics.widget_update_count.get("log_viewer"),
            Some(&1)
        );
        assert_eq!(
            updated_metrics.widget_update_count.get("system_status"),
            Some(&1)
        );

        // Test metrics update
        use workflow_toolkit::interfaces::tui::state::PerformanceMetrics;
        let new_metrics = PerformanceMetrics {
            render_time_ms: 16.7,
            update_time_ms: 2.3,
            event_processing_time_ms: 0.8,
            memory_usage_mb: 45.2,
            fps: 60.0,
            widget_update_count: updated_metrics.widget_update_count.clone(),
            last_measurement: Utc::now(),
        };

        fixture
            .shared_state
            .update_metrics(new_metrics.clone())
            .await?;

        let final_metrics = fixture.shared_state.get_metrics().await;
        assert_eq!(final_metrics.render_time_ms, 16.7);
        assert_eq!(final_metrics.fps, 60.0);
        assert_eq!(final_metrics.memory_usage_mb, 45.2);

        Ok(())
    }

    #[tokio::test]
    async fn test_data_staleness_detection() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test data staleness detection
        let initial_update = fixture.shared_state.last_update().await;

        // Data should not be stale initially
        let is_stale = fixture
            .shared_state
            .is_data_stale(chrono::Duration::seconds(1))
            .await;
        assert!(!is_stale);

        // Wait a bit and check again
        tokio::time::sleep(Duration::from_millis(10)).await;

        let time_since_update = fixture.shared_state.time_since_last_update().await;
        assert!(time_since_update.num_milliseconds() >= 10);

        // Check staleness with very short threshold
        let is_stale_short = fixture
            .shared_state
            .is_data_stale(chrono::Duration::milliseconds(5))
            .await;
        assert!(is_stale_short);

        // Refresh data
        fixture.shared_state.refresh_all().await?;

        let updated_time = fixture.shared_state.last_update().await;
        assert!(updated_time > initial_update);

        // Should not be stale after refresh
        let is_stale_after_refresh = fixture
            .shared_state
            .is_data_stale(chrono::Duration::seconds(1))
            .await;
        assert!(!is_stale_after_refresh);

        Ok(())
    }
}

/// Test suite for error handling and edge cases
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_operations() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test operations on non-existent data

        // Try to update non-existent workflow
        let result = fixture
            .shared_state
            .update_workflow_status("NonExistent", WorkflowStatus::Running)
            .await;
        assert!(result.is_err());

        // Try to remove non-existent workflow
        let result = fixture.shared_state.remove_workflow("NonExistent").await;
        assert!(result.is_err());

        // Try to remove non-existent tool
        let result = fixture.shared_state.remove_tool("NonExistent").await;
        assert!(result.is_err());

        // Try to remove non-existent plugin
        let result = fixture.shared_state.remove_plugin("NonExistent").await;
        assert!(result.is_err());

        // Verify that failed operations don't corrupt state
        let workflows = fixture.shared_state.get_workflows().await;
        let tools = fixture.shared_state.get_tools().await;
        let plugins = fixture.shared_state.get_plugins().await;

        // State should remain consistent
        assert_eq!(workflows.len(), 0); // No workflows added yet
        assert_eq!(tools.len(), 0);
        assert_eq!(plugins.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_data_handling() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test operations with empty state
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), 0);

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 0);

        let tools = fixture.shared_state.get_tools().await;
        assert_eq!(tools.len(), 0);

        let plugins = fixture.shared_state.get_plugins().await;
        assert_eq!(plugins.len(), 0);

        // Test that operations work correctly with empty state
        fixture.shared_state.refresh_all().await?;

        let connection_status = fixture.shared_state.get_connection_status().await;
        assert_eq!(connection_status, ConnectionStatus::Disconnected);

        let system_status = fixture.shared_state.get_system_status().await;
        assert_eq!(system_status.active_workflows, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_unicode_data_handling() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test handling of Unicode and special characters
        let unicode_workflow = WorkflowInfo {
            name: "测试工作流 🚀".to_string(),
            version: "1.0.0-α".to_string(),
            description: Some("包含Unicode字符的测试工作流 ✨🔧⚡".to_string()),
            status: WorkflowStatus::Available,
            last_execution: None,
            execution_count: 0,
            tags: vec!["测试".to_string(), "Unicode".to_string(), "🏷️".to_string()],
            node_count: 1,
            estimated_duration: Some(Duration::from_secs(60)),
            success_rate: None,
        };

        fixture.shared_state.add_workflow(unicode_workflow).await?;

        let unicode_log = LogEntry {
            id: "unicode-log".to_string(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Unicode测试消息 with émojis 🎉 and spëcial chars àáâãäå".to_string(),
            source: Some("unicode_test".to_string()),
            execution_id: None,
            workflow_id: None,
            node_id: None,
        };

        fixture.shared_state.add_log_entry(unicode_log).await?;

        // Verify data was stored correctly
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "测试工作流 🚀");
        assert!(workflows[0]
            .description
            .as_ref()
            .unwrap()
            .contains("✨🔧⚡"));
        assert!(workflows[0].tags.contains(&"🏷️".to_string()));

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 1);
        assert!(logs[0].message.contains("🎉"));
        assert!(logs[0].message.contains("àáâãäå"));

        Ok(())
    }
}

/// Test suite for performance and scalability
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_large_dataset_performance() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test performance with larger datasets
        let workflow_count = 100;
        let log_count = 500;

        // Measure workflow addition performance
        let start_time = Instant::now();

        for i in 0..workflow_count {
            let workflow = WorkflowInfo {
                name: format!("Performance Test Workflow {}", i),
                version: "1.0.0".to_string(),
                description: Some(format!("Performance test workflow {}", i)),
                status: if i % 3 == 0 {
                    WorkflowStatus::Running
                } else {
                    WorkflowStatus::Available
                },
                last_execution: Some(Utc::now() - chrono::Duration::hours(i as i64 % 24)),
                execution_count: i as u32,
                tags: vec![format!("perf-{}", i % 10)],
                node_count: (i % 5 + 1) as u32,
                estimated_duration: Some(Duration::from_secs((i % 300 + 30) as u64)),
                success_rate: Some(0.8 + (i as f64 % 100.0) / 500.0),
            };

            fixture.shared_state.add_workflow(workflow).await?;
        }

        let workflow_time = start_time.elapsed();
        println!("Added {} workflows in {:?}", workflow_count, workflow_time);

        // Measure log addition performance
        let log_start_time = Instant::now();

        for i in 0..log_count {
            let log = LogEntry {
                id: format!("perf-log-{}", i),
                timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
                level: match i % 4 {
                    0 => LogLevel::Debug,
                    1 => LogLevel::Info,
                    2 => LogLevel::Warn,
                    3 => LogLevel::Error,
                    _ => LogLevel::Info,
                },
                message: format!("Performance test log entry {}", i),
                source: Some(format!("perf_test_{}", i % 10)),
                execution_id: if i % 5 == 0 {
                    Some(format!("exec-{}", i / 5))
                } else {
                    None
                },
                workflow_id: if i % 3 == 0 {
                    Some(format!("workflow-{}", i / 3))
                } else {
                    None
                },
                node_id: if i % 7 == 0 {
                    Some(format!("node-{}", i / 7))
                } else {
                    None
                },
            };

            fixture.shared_state.add_log_entry(log).await?;
        }

        let log_time = log_start_time.elapsed();
        println!("Added {} log entries in {:?}", log_count, log_time);

        // Verify data integrity
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), workflow_count);

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), log_count);

        // Performance assertions (should be reasonable for integration tests)
        assert!(workflow_time < Duration::from_secs(2)); // Should complete in under 2 seconds
        assert!(log_time < Duration::from_secs(3)); // Should complete in under 3 seconds

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_access_performance() -> Result<()> {
        let fixture = BasicTuiFixture::new().await?;

        // Test concurrent access performance
        let task_count = 10;
        let operations_per_task = 20;

        let start_time = Instant::now();
        let mut handles = Vec::new();

        for task_id in 0..task_count {
            let state = Arc::clone(&fixture.shared_state);
            let handle = tokio::spawn(async move {
                for i in 0..operations_per_task {
                    // Add workflow
                    let workflow = WorkflowInfo {
                        name: format!("Concurrent Workflow {}-{}", task_id, i),
                        version: "1.0.0".to_string(),
                        description: Some(format!(
                            "Workflow from task {} operation {}",
                            task_id, i
                        )),
                        status: WorkflowStatus::Available,
                        last_execution: None,
                        execution_count: 0,
                        tags: vec![format!("task-{}", task_id), format!("op-{}", i)],
                        node_count: 1,
                        estimated_duration: Some(Duration::from_secs(30)),
                        success_rate: None,
                    };

                    state.add_workflow(workflow).await?;

                    // Add log entry
                    let log = LogEntry {
                        id: format!("concurrent-log-{}-{}", task_id, i),
                        timestamp: Utc::now(),
                        level: LogLevel::Info,
                        message: format!("Concurrent log from task {} operation {}", task_id, i),
                        source: Some("concurrent_test".to_string()),
                        execution_id: None,
                        workflow_id: None,
                        node_id: None,
                    };

                    state.add_log_entry(log).await?;
                }

                Ok::<(), workflow_toolkit::error::WorkflowError>(())
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.map_err(|e| {
                workflow_toolkit::error::WorkflowError::ValidationError(format!(
                    "Task join error: {}",
                    e
                ))
            })??;
        }

        let elapsed = start_time.elapsed();
        let total_operations = task_count * operations_per_task * 2; // 2 operations per iteration

        println!(
            "Completed {} concurrent operations in {:?}",
            total_operations, elapsed
        );

        // Verify all data was added correctly
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), task_count * operations_per_task);

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), task_count * operations_per_task);

        // Performance assertion
        assert!(elapsed < Duration::from_secs(5)); // Should complete in under 5 seconds

        Ok(())
    }
}
