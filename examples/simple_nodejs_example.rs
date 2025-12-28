//! Simple Node.js Plugin Example (No npm dependencies required)
//! 
//! This example demonstrates basic Node.js plugin functionality without requiring npm

use std::path::PathBuf;
use std::time::Duration;
use tokio;
use workflow_toolkit::{
    core::{ExecutionContext, ToolInfo},
    plugins::{NodeJsPluginBuilder, PluginConfig, Plugin},
    error::Result,
};
use serde_json::json;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Simple Node.js Plugin Example (No npm required)");
    println!("==================================================");

    // Create tool info for simple test
    let test_tool_info = ToolInfo {
        name: "simple_test".to_string(),
        version: "1.0.0".to_string(),
        description: "A simple test tool implemented in Node.js".to_string(),
        category: Some("test".to_string()),
        tags: vec!["test".to_string(), "nodejs".to_string()],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "message": {"type": "string"}
            }
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "message": {"type": "string"}
            }
        }),
        plugin_name: Some("simple-nodejs-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Build the plugin without npm dependencies
    let mut plugin = NodeJsPluginBuilder::new()
        .name("simple-nodejs-plugin")
        .version("1.0.0")
        .description("Simple Node.js plugin without npm dependencies")
        .working_directory("examples/nodejs_tools")
        .auto_install_dependencies(false) // Disable npm install
        .add_tool(
            test_tool_info,
            PathBuf::from("simple_test.js"),
            Some(Duration::from_secs(10))
        )
        .build()
        .await?;

    println!("✅ Plugin created: {}", plugin.info().name);

    // Initialize the plugin
    let config = PluginConfig::new("simple-nodejs-plugin".to_string(), workflow_toolkit::core::PluginType::NodeJs);
    plugin.initialize(config)?;
    
    // Initialize async components (without npm install)
    plugin.initialize_async().await?;
    
    println!("✅ Plugin initialized with status: {:?}", plugin.status());

    // Test the simple tool
    println!("🧪 Testing simple Node.js tool...");
    
    let tools = plugin.get_tools();
    if let Some(test_tool) = tools.first() {
        let context = ExecutionContext::new();
        
        // Test with parameters
        let params = json!({
            "message": "Hello from Rust!"
        });
        
        let result = test_tool.execute(params, context).await?;
        println!("📊 Test result: {}", serde_json::to_string_pretty(&result)?);
    }

    // Shutdown the plugin
    plugin.shutdown()?;
    println!("✅ Plugin shut down successfully");

    println!("\n✅ Simple Node.js plugin example completed successfully!");
    Ok(())
}