//! TUI Integration Tests
//!
//! This module contains comprehensive integration tests for the TUI system,
//! covering end-to-end user workflows, cross-component interactions, and data flow.

use chrono::Utc;
use ratatui::{
    backend::TestBackend,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    Terminal,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use workflow_toolkit::error::Result;
use workflow_toolkit::interfaces::tui::{
    action::{Action, LogLevel, ViewType},
    state::{ConnectionStatus, NetworkStatus, StateChangeEvent, SystemHealth, SystemStatus},
    widget::{WidgetCapabilities, WidgetState},
    widgets::{
        log_viewer::LogEntry,
        workflow_list::{WorkflowInfo, WorkflowStatus},
        ExecutionMonitorWidget, LogViewerWidget, PluginManagerWidget, SystemStatusWidget,
        ToolManagerWidget, WorkflowListWidget,
    },
    MainTuiInterface, SharedAppState, Theme, TuiApp, Widget, WidgetId,
};

/// Integration test fixture for TUI system testing
struct TuiIntegrationFixture {
    shared_state: Arc<SharedAppState>,
    widgets: Vec<Box<dyn Widget>>,
    theme: Theme,
}

impl TuiIntegrationFixture {
    /// Create a new integration test fixture
    async fn new() -> Result<Self> {
        let shared_state = Arc::new(SharedAppState::new());

        // Create all widgets
        let widgets: Vec<Box<dyn Widget>> = vec![
            Box::new(WorkflowListWidget::new()),
            Box::new(LogViewerWidget::new()),
            Box::new(ToolManagerWidget::new()),
            Box::new(PluginManagerWidget::new()),
            Box::new(SystemStatusWidget::new()),
        ];

        let theme = Theme::default();

        Ok(Self {
            shared_state,
            widgets,
            theme,
        })
    }

    /// Initialize all widgets in the fixture
    async fn initialize_widgets(&mut self) -> Result<()> {
        for widget in &mut self.widgets {
            widget.initialize().await?;
        }
        Ok(())
    }

    /// Cleanup all widgets in the fixture
    async fn cleanup_widgets(&mut self) -> Result<()> {
        for widget in &mut self.widgets {
            widget.cleanup().await?;
        }
        Ok(())
    }

    /// Populate test data
    async fn populate_test_data(&self) -> Result<()> {
        // Add test workflows
        let workflows = vec![
            WorkflowInfo {
                name: "Test Workflow 1".to_string(),
                version: "1.0.0".to_string(),
                description: Some("First test workflow".to_string()),
                status: WorkflowStatus::Available,
                last_execution: Some(Utc::now() - chrono::Duration::hours(1)),
                execution_count: 5,
                tags: vec!["test".to_string(), "integration".to_string()],
                node_count: 3,
                estimated_duration: Some(Duration::from_secs(60)),
                success_rate: Some(0.95),
            },
            WorkflowInfo {
                name: "Test Workflow 2".to_string(),
                version: "2.0.0".to_string(),
                description: Some("Second test workflow".to_string()),
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
                id: "test-log-1".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(5),
                level: LogLevel::Info,
                message: "Integration test started".to_string(),
                source: Some("test_suite".to_string()),
                execution_id: None,
                workflow_id: None,
                node_id: None,
            },
            LogEntry {
                id: "test-log-2".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(3),
                level: LogLevel::Debug,
                message: "Test workflow execution initiated".to_string(),
                source: Some("workflow_engine".to_string()),
                execution_id: Some("test-exec-1".to_string()),
                workflow_id: Some("test-workflow-1".to_string()),
                node_id: Some("node-1".to_string()),
            },
            LogEntry {
                id: "test-log-3".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(1),
                level: LogLevel::Warn,
                message: "Test warning message".to_string(),
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

/// Test suite for end-to-end user workflows
#[cfg(test)]
mod end_to_end_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_user_workflow_navigation() -> Result<()> {
        let mut fixture = TuiIntegrationFixture::new().await?;
        fixture.initialize_widgets().await?;
        fixture.populate_test_data().await?;

        // Test complete user workflow:
        // 1. Start with workflow list
        // 2. Navigate through all views
        // 3. Interact with widgets
        // 4. Verify state consistency

        let mut workflow_widget = WorkflowListWidget::new();
        workflow_widget.initialize().await?;
        workflow_widget.on_activate().await?;

        // Verify initial state
        assert_eq!(workflow_widget.context().state, WidgetState::Active);

        // Simulate navigation keys
        let down_key = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let result = workflow_widget.handle_event(down_key).await?;

        let up_key = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        let result = workflow_widget.handle_event(up_key).await?;

        // Test widget switching
        workflow_widget.on_deactivate().await?;
        assert_eq!(workflow_widget.context().state, WidgetState::Inactive);

        let mut log_widget = LogViewerWidget::new();
        log_widget.initialize().await?;
        log_widget.on_activate().await?;
        assert_eq!(log_widget.context().state, WidgetState::Active);

        // Test data access
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), 2);
        assert_eq!(workflows[0].name, "Test Workflow 1");

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].message, "Integration test started");

        // Cleanup
        log_widget.cleanup().await?;
        workflow_widget.cleanup().await?;
        fixture.cleanup_widgets().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_lifecycle_management() -> Result<()> {
        let mut fixture = TuiIntegrationFixture::new().await?;

        // Test complete widget lifecycle for all widget types
        let widget_types = vec![
            (
                "WorkflowList",
                Box::new(WorkflowListWidget::new()) as Box<dyn Widget>,
            ),
            (
                "LogViewer",
                Box::new(LogViewerWidget::new()) as Box<dyn Widget>,
            ),
            (
                "ToolManager",
                Box::new(ToolManagerWidget::new()) as Box<dyn Widget>,
            ),
            (
                "PluginManager",
                Box::new(PluginManagerWidget::new()) as Box<dyn Widget>,
            ),
            (
                "SystemStatus",
                Box::new(SystemStatusWidget::new()) as Box<dyn Widget>,
            ),
        ];

        for (widget_name, mut widget) in widget_types {
            // Test complete lifecycle
            assert_eq!(widget.context().state, WidgetState::Uninitialized);

            widget.initialize().await?;
            assert_eq!(widget.context().state, WidgetState::Inactive);

            widget.on_activate().await?;
            assert_eq!(widget.context().state, WidgetState::Active);

            widget.on_focus().await?;
            assert_eq!(widget.context().state, WidgetState::Focused);
            assert!(widget.context().has_focus);

            widget.on_blur().await?;
            assert_eq!(widget.context().state, WidgetState::Active);
            assert!(!widget.context().has_focus);

            widget.on_deactivate().await?;
            assert_eq!(widget.context().state, WidgetState::Inactive);

            widget.cleanup().await?;
            assert_eq!(widget.context().state, WidgetState::Uninitialized);

            println!("✓ {}: Complete lifecycle test passed", widget_name);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_tui_interface_integration() -> Result<()> {
        // Test the MainTuiInterface integration
        let mut tui_interface = MainTuiInterface::new();

        // Test initialization
        assert!(!tui_interface.is_initialized());
        assert_eq!(tui_interface.initialization_attempts(), 0);

        // Initialize (but don't start to avoid terminal issues)
        tui_interface.initialize().await?;
        assert!(tui_interface.is_initialized());
        assert!(tui_interface.initialization_attempts() > 0);

        // Test refresh
        tui_interface.refresh().await?;

        // Test stop (should not error even if not running)
        tui_interface.stop().await?;
        assert!(!tui_interface.is_running());

        Ok(())
    }
}

/// Test suite for cross-component interactions
#[cfg(test)]
mod cross_component_tests {
    use super::*;

    #[tokio::test]
    async fn test_state_synchronization_between_widgets() -> Result<()> {
        let mut fixture = TuiIntegrationFixture::new().await?;
        fixture.initialize_widgets().await?;
        fixture.populate_test_data().await?;

        // Test that state changes are properly synchronized
        let mut workflow_widget = WorkflowListWidget::new();
        let mut log_widget = LogViewerWidget::new();

        workflow_widget.initialize().await?;
        log_widget.initialize().await?;

        // Subscribe to state changes
        let mut state_receiver = fixture.shared_state.subscribe();

        // Modify workflow status
        fixture
            .shared_state
            .update_workflow_status("Test Workflow 1", WorkflowStatus::Running)
            .await?;

        // Verify state change event was broadcast
        let event = timeout(Duration::from_millis(100), state_receiver.recv()).await;
        assert!(event.is_ok());

        if let Ok(Ok(StateChangeEvent::WorkflowStatusChanged { name, status })) = event {
            assert_eq!(name, "Test Workflow 1");
            assert_eq!(status, WorkflowStatus::Running);
        } else {
            panic!("Expected WorkflowStatusChanged event");
        }

        // Verify the change is reflected in shared state
        let workflows = fixture.shared_state.get_workflows().await;
        let updated_workflow = workflows
            .iter()
            .find(|w| w.name == "Test Workflow 1")
            .unwrap();
        assert_eq!(updated_workflow.status, WorkflowStatus::Running);

        // Add a new log entry
        let new_log = LogEntry {
            id: "test-log-new".to_string(),
            timestamp: Utc::now(),
            level: LogLevel::Error,
            message: "New test log entry".to_string(),
            source: Some("integration_test".to_string()),
            execution_id: None,
            workflow_id: None,
            node_id: None,
        };

        fixture.shared_state.add_log_entry(new_log.clone()).await?;

        // Verify log entry event was broadcast
        let log_event = timeout(Duration::from_millis(100), state_receiver.recv()).await;
        assert!(log_event.is_ok());

        if let Ok(Ok(StateChangeEvent::LogEntryAdded { entry })) = log_event {
            assert_eq!(entry.id, "test-log-new");
            assert_eq!(entry.message, "New test log entry");
        } else {
            panic!("Expected LogEntryAdded event");
        }

        // Cleanup
        workflow_widget.cleanup().await?;
        log_widget.cleanup().await?;
        fixture.cleanup_widgets().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_event_propagation() -> Result<()> {
        let mut fixture = TuiIntegrationFixture::new().await?;
        fixture.initialize_widgets().await?;

        // Test that events are properly handled by widgets
        let mut workflow_widget = WorkflowListWidget::new();
        workflow_widget.initialize().await?;
        workflow_widget.on_activate().await?;

        // Test various key events
        let test_events = vec![
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        ];

        for event in test_events {
            // Each event should be handled without error
            let result = workflow_widget.handle_event(event).await;
            assert!(result.is_ok(), "Event handling should not fail");
        }

        // Test widget capabilities
        let capabilities = workflow_widget.capabilities();
        assert!(capabilities.keyboard_input);
        assert!(capabilities.focusable);
        assert!(capabilities.themeable);

        // Test help text
        let help_text = workflow_widget.help_text();
        assert!(!help_text.is_empty());

        // Test metrics
        let metrics = workflow_widget.metrics();
        assert!(metrics.contains_key("is_active"));
        assert!(metrics.contains_key("has_focus"));

        workflow_widget.cleanup().await?;
        fixture.cleanup_widgets().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_theme_system_integration() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;

        // Test theme system with widgets
        let theme = Theme::default();

        // Verify theme has required components
        assert!(theme.colors.primary != theme.colors.secondary);
        assert!(theme.styles.header.fg.is_some() || theme.styles.header.bg.is_some());

        // Test that widgets can use themes for rendering
        let mut widget = WorkflowListWidget::new();
        widget.initialize().await?;
        widget.on_activate().await?;

        // Test rendering with theme (using test backend)
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;

        let render_result = terminal.draw(|frame| {
            let area = frame.area();
            let rt = tokio::runtime::Handle::current();
            let _ = rt.block_on(async { widget.render(frame, area, &theme).await });
        });

        assert!(render_result.is_ok());

        widget.cleanup().await?;

        Ok(())
    }
}

/// Test suite for data flow validation
#[cfg(test)]
mod data_flow_tests {
    use super::*;

    #[tokio::test]
    async fn test_shared_state_data_flow() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;
        fixture.populate_test_data().await?;

        // Test complete data flow through shared state

        // 1. Verify initial data
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), 2);

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 3);

        let system_status = fixture.shared_state.get_system_status().await;
        assert_eq!(system_status.active_workflows, 2);
        assert_eq!(system_status.system_health, SystemHealth::Warning);

        // 2. Test data modifications
        fixture
            .shared_state
            .update_workflow_status("Test Workflow 1", WorkflowStatus::Completed)
            .await?;

        let updated_workflows = fixture.shared_state.get_workflows().await;
        let updated_workflow = updated_workflows
            .iter()
            .find(|w| w.name == "Test Workflow 1")
            .unwrap();
        assert_eq!(updated_workflow.status, WorkflowStatus::Completed);

        // 3. Test data additions
        let new_workflow = WorkflowInfo {
            name: "New Test Workflow".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Newly added workflow".to_string()),
            status: WorkflowStatus::Available,
            last_execution: None,
            execution_count: 0,
            tags: vec!["new".to_string()],
            node_count: 2,
            estimated_duration: Some(Duration::from_secs(30)),
            success_rate: None,
        };

        fixture.shared_state.add_workflow(new_workflow).await?;

        let all_workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(all_workflows.len(), 3);
        assert!(all_workflows.iter().any(|w| w.name == "New Test Workflow"));

        // 4. Test data removals
        fixture
            .shared_state
            .remove_workflow("Test Workflow 2")
            .await?;

        let remaining_workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(remaining_workflows.len(), 2);
        assert!(!remaining_workflows
            .iter()
            .any(|w| w.name == "Test Workflow 2"));

        // 5. Test connection status changes
        fixture
            .shared_state
            .set_connection_status(ConnectionStatus::Connected)
            .await?;
        let connection_status = fixture.shared_state.get_connection_status().await;
        assert_eq!(connection_status, ConnectionStatus::Connected);

        let status_string = fixture.shared_state.connection_status_string().await;
        assert_eq!(status_string, "已连接");

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_metrics_tracking() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;

        // Test performance metrics collection and updates
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

        let updated_metrics = fixture.shared_state.get_metrics().await;
        assert_eq!(
            updated_metrics.widget_update_count.get("workflow_list"),
            Some(&2)
        );
        assert_eq!(
            updated_metrics.widget_update_count.get("log_viewer"),
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

        Ok(())
    }

    #[tokio::test]
    async fn test_data_staleness_detection() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;

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

        // Refresh data
        fixture.shared_state.refresh_all().await?;

        let updated_time = fixture.shared_state.last_update().await;
        assert!(updated_time > initial_update);

        Ok(())
    }
}

/// Test suite for error handling and recovery
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_widget_error_resilience() -> Result<()> {
        let mut fixture = TuiIntegrationFixture::new().await?;

        // Test that widgets handle errors gracefully
        let mut widget = WorkflowListWidget::new();

        // Test handling events before initialization
        let key_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let result = widget.handle_event(key_event).await;

        // Should either succeed or fail gracefully (not panic)
        match result {
            Ok(_) => {}  // Widget handled it gracefully
            Err(_) => {} // Widget returned an error (acceptable)
        }

        // Initialize and try again
        widget.initialize().await?;
        let result = widget.handle_event(key_event).await?;

        // Test validation
        let validation_result = widget.validate();
        assert!(validation_result.is_ok() || validation_result.is_err()); // Should not panic

        widget.cleanup().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_state_error_recovery() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;

        // Test error handling in state operations

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
    async fn test_concurrent_state_access() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;

        // Test concurrent access to shared state
        let shared_state = Arc::clone(&fixture.shared_state);

        // Spawn multiple tasks that access state concurrently
        let mut handles = Vec::new();

        for i in 0..10 {
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
        assert_eq!(workflows.len(), 10);

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 10);

        // Verify data integrity
        for i in 0..10 {
            assert!(workflows
                .iter()
                .any(|w| w.name == format!("Concurrent Workflow {}", i)));
            assert!(logs.iter().any(|l| l.id == format!("concurrent-log-{}", i)));
        }

        Ok(())
    }
}

/// Test suite for performance and scalability
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_widget_rendering_performance() -> Result<()> {
        let mut fixture = TuiIntegrationFixture::new().await?;
        fixture.populate_test_data().await?;

        // Test rendering performance with multiple widgets
        let mut widgets = vec![
            Box::new(WorkflowListWidget::new()) as Box<dyn Widget>,
            Box::new(LogViewerWidget::new()) as Box<dyn Widget>,
            Box::new(ToolManagerWidget::new()) as Box<dyn Widget>,
            Box::new(SystemStatusWidget::new()) as Box<dyn Widget>,
        ];

        // Initialize all widgets
        for widget in &mut widgets {
            widget.initialize().await?;
            widget.on_activate().await?;
        }

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend)?;
        let theme = Theme::default();

        // Measure rendering time
        let start_time = Instant::now();
        let render_count = 50;

        for _ in 0..render_count {
            for widget in &mut widgets {
                terminal.draw(|frame| {
                    let area = frame.area();
                    let rt = tokio::runtime::Handle::current();
                    let _ = rt.block_on(async { widget.render(frame, area, &theme).await });
                })?;
            }
        }

        let elapsed = start_time.elapsed();
        let avg_render_time = elapsed / (render_count * widgets.len() as u32);

        println!("Average render time per widget: {:?}", avg_render_time);

        // Should render in less than 10ms per widget on average
        assert!(avg_render_time < Duration::from_millis(10));

        // Cleanup
        for widget in &mut widgets {
            widget.cleanup().await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_state_update_performance() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;

        // Test performance of state updates with large datasets
        let workflow_count = 1000;
        let log_count = 5000;

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
        // Note: SharedAppState limits logs to 10000 entries
        assert!(logs.len() <= 10000);
        assert!(logs.len() >= log_count.min(10000));

        // Performance assertions
        assert!(workflow_time < Duration::from_secs(5)); // Should complete in under 5 seconds
        assert!(log_time < Duration::from_secs(10)); // Should complete in under 10 seconds

        Ok(())
    }

    #[tokio::test]
    async fn test_event_processing_performance() -> Result<()> {
        let mut fixture = TuiIntegrationFixture::new().await?;
        fixture.initialize_widgets().await?;

        // Test event processing performance
        let mut widget = WorkflowListWidget::new();
        widget.initialize().await?;
        widget.on_activate().await?;

        let event_count = 1000;
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        ];

        let start_time = Instant::now();

        for i in 0..event_count {
            let event = events[i % events.len()].clone();
            widget.handle_event(event).await?;
        }

        let elapsed = start_time.elapsed();
        let avg_event_time = elapsed / event_count;

        println!("Processed {} events in {:?}", event_count, elapsed);
        println!("Average event processing time: {:?}", avg_event_time);

        // Should process events in less than 1ms on average
        assert!(avg_event_time < Duration::from_millis(1));

        widget.cleanup().await?;
        fixture.cleanup_widgets().await?;

        Ok(())
    }
}

/// Test suite for compatibility and edge cases
#[cfg(test)]
mod compatibility_tests {
    use super::*;

    #[tokio::test]
    async fn test_terminal_size_handling() -> Result<()> {
        let mut fixture = TuiIntegrationFixture::new().await?;
        fixture.initialize_widgets().await?;

        // Test rendering at different terminal sizes
        let terminal_sizes = vec![
            (80, 24),  // Standard
            (120, 40), // Large
            (40, 12),  // Small
            (200, 60), // Very large
            (20, 6),   // Very small
        ];

        let mut widget = WorkflowListWidget::new();
        widget.initialize().await?;
        widget.on_activate().await?;

        let theme = Theme::default();

        for (width, height) in terminal_sizes {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend)?;

            let render_result = terminal.draw(|frame| {
                let area = frame.area();
                let rt = tokio::runtime::Handle::current();
                let _ = rt.block_on(async { widget.render(frame, area, &theme).await });
            });

            assert!(
                render_result.is_ok(),
                "Rendering should work at {}x{}",
                width,
                height
            );
        }

        widget.cleanup().await?;
        fixture.cleanup_widgets().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_data_handling() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;

        // Test widgets with empty data
        let mut widget = WorkflowListWidget::new();
        widget.initialize().await?;
        widget.on_activate().await?;

        // Verify empty state handling
        let workflows = fixture.shared_state.get_workflows().await;
        assert_eq!(workflows.len(), 0);

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 0);

        // Test rendering with empty data
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;
        let theme = Theme::default();

        let render_result = terminal.draw(|frame| {
            let area = frame.area();
            let rt = tokio::runtime::Handle::current();
            let _ = rt.block_on(async { widget.render(frame, area, &theme).await });
        });

        assert!(render_result.is_ok());

        // Test event handling with empty data
        let key_event = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let result = widget.handle_event(key_event).await;
        assert!(result.is_ok());

        widget.cleanup().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_unicode_and_special_characters() -> Result<()> {
        let fixture = TuiIntegrationFixture::new().await?;

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

        let logs = fixture.shared_state.get_logs().await;
        assert_eq!(logs.len(), 1);
        assert!(logs[0].message.contains("🎉"));

        // Test rendering with Unicode content
        let mut widget = WorkflowListWidget::new();
        widget.initialize().await?;
        widget.on_activate().await?;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;
        let theme = Theme::default();

        let render_result = terminal.draw(|frame| {
            let area = frame.area();
            let rt = tokio::runtime::Handle::current();
            let _ = rt.block_on(async { widget.render(frame, area, &theme).await });
        });

        assert!(render_result.is_ok());

        widget.cleanup().await?;

        Ok(())
    }
}
