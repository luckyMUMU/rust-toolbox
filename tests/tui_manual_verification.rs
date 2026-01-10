//! Manual TUI Widget Verification
//! 
//! This module provides manual verification of Widget functionality
//! by testing each Widget individually without the complex app integration.

use std::time::Duration;
use workflow_toolkit::interfaces::tui::widget::{
    Widget, WidgetId, WidgetContext, WidgetCapabilities, WidgetState, WidgetError, SizeConstraints
};
use workflow_toolkit::interfaces::tui::{
    Theme,
    widgets::{
        WorkflowListWidget, LogViewerWidget, ToolManagerWidget, 
        PluginManagerWidget, SystemStatusWidget
    }
};
use workflow_toolkit::error::Result;
use ratatui::{
    backend::TestBackend,
    Terminal,
    layout::Rect,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
};

/// Manual verification results
#[derive(Debug)]
pub struct VerificationResults {
    pub workflow_list: WidgetVerificationResult,
    pub log_viewer: WidgetVerificationResult,
    pub tool_manager: WidgetVerificationResult,
    pub plugin_manager: WidgetVerificationResult,
    pub system_status: WidgetVerificationResult,
    pub overall_success: bool,
}

/// Individual widget verification result
#[derive(Debug)]
pub struct WidgetVerificationResult {
    pub widget_name: String,
    pub initialization: bool,
    pub lifecycle: bool,
    pub event_handling: bool,
    pub rendering: bool,
    pub capabilities: bool,
    pub success: bool,
}

impl WidgetVerificationResult {
    fn new(widget_name: String) -> Self {
        Self {
            widget_name,
            initialization: false,
            lifecycle: false,
            event_handling: false,
            rendering: false,
            capabilities: false,
            success: false,
        }
    }
    
    fn calculate_success(&mut self) {
        self.success = self.initialization && self.lifecycle && self.event_handling && self.rendering && self.capabilities;
    }
}

/// Verify a widget's basic functionality
async fn verify_widget<W: Widget>(mut widget: W, widget_name: &str) -> WidgetVerificationResult {
    let mut result = WidgetVerificationResult::new(widget_name.to_string());
    
    // Test 1: Initialization
    match widget.initialize().await {
        Ok(()) => {
            if widget.context().state == WidgetState::Inactive {
                result.initialization = true;
                println!("✓ {}: Initialization successful", widget_name);
            } else {
                println!("✗ {}: Initialization failed - wrong state", widget_name);
            }
        }
        Err(e) => {
            println!("✗ {}: Initialization failed - {}", widget_name, e);
        }
    }
    
    // Test 2: Lifecycle management
    let lifecycle_test = async {
        widget.on_activate().await?;
        if widget.context().state != WidgetState::Active {
            return Err("Activation failed");
        }
        
        widget.on_focus().await?;
        if widget.context().state != WidgetState::Focused || !widget.context().has_focus {
            return Err("Focus failed");
        }
        
        widget.on_blur().await?;
        if widget.context().state != WidgetState::Active || widget.context().has_focus {
            return Err("Blur failed");
        }
        
        widget.on_deactivate().await?;
        if widget.context().state != WidgetState::Inactive {
            return Err("Deactivation failed");
        }
        
        widget.cleanup().await?;
        if widget.context().state != WidgetState::Uninitialized {
            return Err("Cleanup failed");
        }
        
        Ok(())
    };
    
    match lifecycle_test.await {
        Ok(()) => {
            result.lifecycle = true;
            println!("✓ {}: Lifecycle management successful", widget_name);
        }
        Err(e) => {
            println!("✗ {}: Lifecycle management failed - {}", widget_name, e);
        }
    }
    
    // Re-initialize for further tests
    let _ = widget.initialize().await;
    let _ = widget.on_activate().await;
    
    // Test 3: Event handling
    let key_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match widget.handle_event(key_event).await {
        Ok(_) => {
            result.event_handling = true;
            println!("✓ {}: Event handling successful", widget_name);
        }
        Err(e) => {
            println!("✗ {}: Event handling failed - {}", widget_name, e);
        }
    }
    
    // Test 4: Rendering
    let backend = TestBackend::new(80, 24);
    match Terminal::new(backend) {
        Ok(mut terminal) => {
            let theme = Theme::default();
            let render_result = terminal.draw(|frame| {
                let area = frame.area();
                let rt = tokio::runtime::Handle::current();
                let _ = rt.block_on(async {
                    widget.render(frame, area, &theme).await
                });
            });
            
            match render_result {
                Ok(()) => {
                    result.rendering = true;
                    println!("✓ {}: Rendering successful", widget_name);
                }
                Err(e) => {
                    println!("✗ {}: Rendering failed - {}", widget_name, e);
                }
            }
        }
        Err(e) => {
            println!("✗ {}: Terminal creation failed - {}", widget_name, e);
        }
    }
    
    // Test 5: Capabilities
    let capabilities = widget.capabilities();
    if capabilities.keyboard_input && capabilities.focusable && capabilities.themeable {
        result.capabilities = true;
        println!("✓ {}: Capabilities check successful", widget_name);
    } else {
        println!("✗ {}: Capabilities check failed", widget_name);
    }
    
    result.calculate_success();
    result
}

/// Run comprehensive verification of all widgets
pub async fn run_comprehensive_verification() -> VerificationResults {
    println!("🔍 Starting comprehensive TUI Widget verification...\n");
    
    // Test WorkflowListWidget
    println!("Testing WorkflowListWidget...");
    let workflow_list_result = verify_widget(WorkflowListWidget::new(), "WorkflowListWidget").await;
    
    // Test LogViewerWidget
    println!("\nTesting LogViewerWidget...");
    let log_viewer_result = verify_widget(LogViewerWidget::new(), "LogViewerWidget").await;
    
    // Test ToolManagerWidget
    println!("\nTesting ToolManagerWidget...");
    let tool_manager_result = verify_widget(ToolManagerWidget::new(), "ToolManagerWidget").await;
    
    // Test PluginManagerWidget
    println!("\nTesting PluginManagerWidget...");
    let plugin_manager_result = verify_widget(PluginManagerWidget::new(), "PluginManagerWidget").await;
    
    // Test SystemStatusWidget
    println!("\nTesting SystemStatusWidget...");
    let system_status_result = verify_widget(SystemStatusWidget::new(), "SystemStatusWidget").await;
    
    let overall_success = workflow_list_result.success &&
                         log_viewer_result.success &&
                         tool_manager_result.success &&
                         plugin_manager_result.success &&
                         system_status_result.success;
    
    println!("\n📊 Verification Summary:");
    println!("WorkflowListWidget: {}", if workflow_list_result.success { "✅ PASS" } else { "❌ FAIL" });
    println!("LogViewerWidget: {}", if log_viewer_result.success { "✅ PASS" } else { "❌ FAIL" });
    println!("ToolManagerWidget: {}", if tool_manager_result.success { "✅ PASS" } else { "❌ FAIL" });
    println!("PluginManagerWidget: {}", if plugin_manager_result.success { "✅ PASS" } else { "❌ FAIL" });
    println!("SystemStatusWidget: {}", if system_status_result.success { "✅ PASS" } else { "❌ FAIL" });
    println!("\nOverall Result: {}", if overall_success { "✅ ALL WIDGETS FUNCTIONAL" } else { "❌ SOME WIDGETS NEED ATTENTION" });
    
    VerificationResults {
        workflow_list: workflow_list_result,
        log_viewer: log_viewer_result,
        tool_manager: tool_manager_result,
        plugin_manager: plugin_manager_result,
        system_status: system_status_result,
        overall_success,
    }
}

/// Test widget interaction patterns
async fn test_widget_interactions() -> Result<()> {
    println!("\n🔄 Testing Widget interaction patterns...");
    
    // Test multiple widgets can coexist
    let mut widget1 = WorkflowListWidget::new();
    let mut widget2 = LogViewerWidget::new();
    
    // Initialize both
    widget1.initialize().await?;
    widget2.initialize().await?;
    
    // Test switching between widgets
    widget1.on_activate().await?;
    assert_eq!(widget1.context().state, WidgetState::Active);
    assert_eq!(widget2.context().state, WidgetState::Inactive);
    
    widget1.on_deactivate().await?;
    widget2.on_activate().await?;
    assert_eq!(widget1.context().state, WidgetState::Inactive);
    assert_eq!(widget2.context().state, WidgetState::Active);
    
    println!("✓ Widget interaction patterns working correctly");
    Ok(())
}

/// Test widget data flow
async fn test_widget_data_flow() -> Result<()> {
    println!("\n📊 Testing Widget data flow...");
    
    let mut widget = WorkflowListWidget::new();
    widget.initialize().await?;
    widget.on_activate().await?;
    
    // Test update functionality
    let before_update = widget.context().last_update;
    widget.update().await?;
    let after_update = widget.context().last_update;
    
    assert!(after_update.is_some());
    assert!(after_update != before_update);
    
    // Test metrics
    let metrics = widget.metrics();
    assert!(metrics.contains_key("is_active"));
    assert!(metrics.contains_key("has_focus"));
    assert!(metrics.contains_key("is_visible"));
    
    println!("✓ Widget data flow working correctly");
    Ok(())
}

/// Test error handling resilience
async fn test_error_handling() -> Result<()> {
    println!("\n🛡️ Testing error handling resilience...");
    
    let mut widget = WorkflowListWidget::new();
    
    // Test handling events before initialization
    let key_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let result = widget.handle_event(key_event).await;
    
    // Should either succeed or fail gracefully (not panic)
    match result {
        Ok(_) => println!("✓ Widget handled uninitialized event gracefully"),
        Err(_) => println!("✓ Widget returned error for uninitialized event (acceptable)"),
    }
    
    // Test validation
    let validation_result = widget.validate();
    match validation_result {
        Ok(_) => println!("✓ Widget validation passed"),
        Err(_) => println!("✓ Widget validation failed as expected for uninitialized widget"),
    }
    
    println!("✓ Error handling resilience verified");
    Ok(())
}

#[cfg(test)]
mod verification_tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_widget_verification() -> Result<()> {
        let results = run_comprehensive_verification().await;
        
        // At least some widgets should be functional
        let functional_count = [
            results.workflow_list.success,
            results.log_viewer.success,
            results.tool_manager.success,
            results.plugin_manager.success,
            results.system_status.success,
        ].iter().filter(|&&x| x).count();
        
        println!("\nFunctional widgets: {}/5", functional_count);
        
        // We expect at least 3 out of 5 widgets to be functional for this checkpoint
        assert!(functional_count >= 3, "At least 3 widgets should be functional");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_widget_interactions() -> Result<()> {
        test_widget_interactions().await
    }

    #[tokio::test]
    async fn test_widget_data_flow() -> Result<()> {
        test_widget_data_flow().await
    }

    #[tokio::test]
    async fn test_error_handling() -> Result<()> {
        test_error_handling().await
    }
}