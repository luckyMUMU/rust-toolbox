//! Example demonstrating Python plugin usage

use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use workflow_toolkit::{
    core::{ExecutionContext, PluginType, ToolInfo},
    plugins::{Plugin, PythonPluginBuilder, PythonPluginImpl, PythonRuntimeConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Python Plugin Example");
    println!("====================");

    // Create Python runtime configuration
    let python_executable = if cfg!(windows) { "python" } else { "python3" };
    let runtime_config = PythonRuntimeConfig {
        python_executable: python_executable.to_string(),
        virtual_env_path: None, // Use system Python for this example
        requirements_file: Some(PathBuf::from("examples/python_tools/requirements.txt")),
        python_paths: vec![PathBuf::from("examples/python_tools")],
        environment_variables: [("PYTHONUNBUFFERED".to_string(), "1".to_string())]
            .into_iter()
            .collect(),
        working_directory: Some(PathBuf::from("examples/python_tools")),
    };

    // Create tool info for calculator
    let calculator_tool_info = ToolInfo {
        name: "calculator".to_string(),
        version: "1.0.0".to_string(),
        description: "Simple calculator tool".to_string(),
        category: Some("math".to_string()),
        tags: vec!["calculator".to_string(), "math".to_string()],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide", "power"]
                },
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "result": {"type": "number"},
                "operation": {"type": "string"},
                "operands": {"type": "object"}
            }
        }),
        plugin_name: Some("python-tools".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create tool info for data processor
    let processor_tool_info = ToolInfo {
        name: "data_processor".to_string(),
        version: "1.0.0".to_string(),
        description: "Data processing tool".to_string(),
        category: Some("data".to_string()),
        tags: vec!["data".to_string(), "processing".to_string()],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string",
                    "enum": ["numbers", "text", "list"]
                },
                "operation": {"type": "string"},
                "data": {}
            },
            "required": ["type", "operation", "data"]
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "result": {},
                "operation": {"type": "string"}
            }
        }),
        plugin_name: Some("python-tools".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Build the Python plugin
    println!("Building Python plugin...");
    let mut plugin: PythonPluginImpl = PythonPluginBuilder::new()
        .name("python-tools")
        .version("1.0.0")
        .description("Python tools plugin for demonstration")
        .python_executable(python_executable)
        .working_directory("examples/python_tools")
        .add_tool(
            calculator_tool_info,
            PathBuf::from("simple_calculator.py"), // Relative to working directory
            Some(Duration::from_secs(30)),
        )
        .add_tool(
            processor_tool_info,
            PathBuf::from("data_processor.py"), // Relative to working directory
            Some(Duration::from_secs(60)),
        )
        .build()
        .await?;

    // Initialize the plugin
    println!("Initializing Python plugin...");
    let plugin_config = workflow_toolkit::plugins::PluginConfig::new(
        "python-tools".to_string(),
        PluginType::Python,
    );
    plugin.initialize(plugin_config)?;
    plugin.initialize_async().await?;

    // Get tools from the plugin
    let tools = plugin.get_tools();
    println!("Plugin loaded with {} tools:", tools.len());
    for tool in &tools {
        println!("  - {} ({})", tool.name(), tool.version());
    }

    // Test the calculator tool
    println!("\nTesting calculator tool...");
    if let Some(calculator_tool) = tools.iter().find(|t| t.name() == "calculator") {
        let context = ExecutionContext::new();
        let params = json!({
            "operation": "add",
            "a": 10,
            "b": 5
        });

        match calculator_tool.execute(params, context).await {
            Ok(result) => {
                println!("Calculator result: {}", serde_json::to_string_pretty(&result)?);
            }
            Err(e) => {
                println!("Calculator error: {}", e);
            }
        }
    }

    // Test the data processor tool
    println!("\nTesting data processor tool...");
    if let Some(processor_tool) = tools.iter().find(|t| t.name() == "data_processor") {
        let context = ExecutionContext::new();
        let params = json!({
            "type": "numbers",
            "operation": "mean",
            "data": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        });

        match processor_tool.execute(params, context).await {
            Ok(result) => {
                println!("Data processor result: {}", serde_json::to_string_pretty(&result)?);
            }
            Err(e) => {
                println!("Data processor error: {}", e);
            }
        }
    }

    // Test text processing
    println!("\nTesting text processing...");
    if let Some(processor_tool) = tools.iter().find(|t| t.name() == "data_processor") {
        let context = ExecutionContext::new();
        let params = json!({
            "type": "text",
            "operation": "word_count",
            "data": "Hello world! This is a test of the Python plugin system."
        });

        match processor_tool.execute(params, context).await {
            Ok(result) => {
                println!("Text processing result: {}", serde_json::to_string_pretty(&result)?);
            }
            Err(e) => {
                println!("Text processing error: {}", e);
            }
        }
    }

    // Test error handling
    println!("\nTesting error handling...");
    if let Some(calculator_tool) = tools.iter().find(|t| t.name() == "calculator") {
        let context = ExecutionContext::new();
        let params = json!({
            "operation": "divide",
            "a": 10,
            "b": 0  // This should cause a division by zero error
        });

        match calculator_tool.execute(params, context).await {
            Ok(result) => {
                println!("Unexpected success: {}", serde_json::to_string_pretty(&result)?);
            }
            Err(e) => {
                println!("Expected error caught: {}", e);
            }
        }
    }

    println!("\nPython plugin example completed successfully!");
    Ok(())
}