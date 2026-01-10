//! Complete TUI Application Example
//! 
//! This example demonstrates the complete TUI application with all widgets
//! integrated and working together.

use workflow_toolkit::interfaces::tui::{MainTuiInterface, TuiInterface};
use workflow_toolkit::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();
    
    println!("Starting Complete TUI Application Example");
    println!("=========================================");
    println!();
    println!("This example demonstrates the complete TUI interface with:");
    println!("- F1: 工作流列表 (Workflow List)");
    println!("- F2: 执行监控 (Execution Monitor)");
    println!("- F3: 工具管理 (Tool Manager)");
    println!("- F4: 插件管理 (Plugin Manager)");
    println!("- F5: 系统状态 (System Status)");
    println!("- F6: 日志查看器 (Log Viewer)");
    println!();
    println!("Use F1-F6 to switch between views");
    println!("Use Ctrl+Q to quit");
    println!();
    println!("Press Enter to start the TUI application...");
    
    // Wait for user input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    
    // Create and initialize TUI interface
    let mut tui_interface = MainTuiInterface::new();
    
    println!("Initializing TUI application...");
    if let Err(e) = tui_interface.initialize().await {
        eprintln!("Failed to initialize TUI: {}", e);
        return Err(e);
    }
    
    println!("TUI application initialized successfully!");
    println!("Starting TUI interface...");
    
    // Start the TUI application
    match tui_interface.start().await {
        Ok(()) => {
            println!("TUI application completed successfully");
        }
        Err(e) => {
            eprintln!("TUI application error: {}", e);
            return Err(e);
        }
    }
    
    println!("TUI application example completed");
    Ok(())
}