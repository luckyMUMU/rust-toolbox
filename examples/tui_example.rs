//! TUI Example
//! 
//! This example demonstrates the basic TUI framework functionality.

use workflow_toolkit::interfaces::tui::{BasicTuiInterface, TuiApp};
use workflow_toolkit::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("Starting TUI Example...");
    
    // Create and initialize TUI interface
    let mut tui_interface = BasicTuiInterface::new();
    tui_interface.initialize().await?;
    
    println!("TUI interface initialized successfully!");
    println!("Press Ctrl+C to exit");
    
    // Run the TUI (this would normally run the interactive interface)
    // For now, we'll just demonstrate that it can be created and initialized
    println!("TUI framework is ready!");
    
    Ok(())
}