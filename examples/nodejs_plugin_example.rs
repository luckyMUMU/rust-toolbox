//! Node.js Plugin Example
//!
//! This example demonstrates how to:
//! - Create and configure a Node.js plugin
//! - Load Node.js tools from scripts
//! - Execute Node.js tools with parameters
//! - Handle npm dependency management
//! - Process results from Node.js tools

use chrono::Utc;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;
use tokio;
use workflow_toolkit::{
    core::{ExecutionContext, ToolInfo},
    error::Result,
    plugins::{
        NodeJsPluginBuilder, NodeJsRuntimeConfig, Plugin, PluginConfig, PluginStatus,
        PluginType as PluginTypeEnum,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Node.js Plugin Example");
    println!("========================");

    // Example 1: Basic Node.js Plugin Setup
    println!("\n📦 Example 1: Basic Node.js Plugin Setup");
    basic_plugin_setup().await?;

    // Example 2: Plugin with package.json Dependencies
    println!("\n📦 Example 2: Plugin with package.json Dependencies");
    plugin_with_dependencies().await?;

    // Example 3: Multiple Tools in One Plugin
    println!("\n📦 Example 3: Multiple Tools in One Plugin");
    multiple_tools_plugin().await?;

    // Example 4: Error Handling and Validation
    println!("\n📦 Example 4: Error Handling and Validation");
    error_handling_example().await?;

    println!("\n✅ All Node.js plugin examples completed successfully!");
    Ok(())
}

async fn basic_plugin_setup() -> Result<()> {
    println!("Creating a basic Node.js plugin...");

    // Create tool info for simple calculator
    let calculator_info = ToolInfo {
        name: "simple_calculator".to_string(),
        version: "1.0.0".to_string(),
        description: "A simple calculator tool implemented in Node.js".to_string(),
        category: Some("math".to_string()),
        tags: vec![
            "calculator".to_string(),
            "math".to_string(),
            "nodejs".to_string(),
        ],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide", "power", "modulo"]
                },
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "result": {"type": "number"},
                "operation": {"type": "string"},
                "operands": {"type": "object"}
            }
        }),
        plugin_name: Some("nodejs-example".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Build the plugin
    let mut plugin = NodeJsPluginBuilder::new()
        .name("nodejs-example")
        .version("1.0.0")
        .description("Example Node.js plugin with calculator tool")
        .working_directory("examples/nodejs_tools")
        .add_tool(
            calculator_info,
            PathBuf::from("simple_calculator.js"),
            Some(Duration::from_secs(30)),
        )
        .build()
        .await?;

    println!("✅ Plugin created: {}", plugin.info().name);

    // Initialize the plugin
    let config = PluginConfig::new(
        "nodejs-example".to_string(),
        workflow_toolkit::core::PluginType::NodeJs,
    );
    plugin.initialize(config)?;

    // Initialize async components
    plugin.initialize_async().await?;

    println!("✅ Plugin initialized with status: {:?}", plugin.status());

    // Test the calculator tool
    println!("🧮 Testing calculator tool...");

    let tools = plugin.get_tools();
    if let Some(calculator_tool) = tools.first() {
        let context = ExecutionContext::new();

        // Test addition
        let params = json!({
            "operation": "add",
            "a": 15,
            "b": 25
        });

        let result = calculator_tool.execute(params, context.clone()).await?;
        println!(
            "📊 Addition result: {}",
            serde_json::to_string_pretty(&result)?
        );

        // Test division
        let params = json!({
            "operation": "divide",
            "a": 100,
            "b": 4
        });

        let result = calculator_tool.execute(params, context).await?;
        println!(
            "📊 Division result: {}",
            serde_json::to_string_pretty(&result)?
        );
    }

    // Shutdown the plugin
    plugin.shutdown()?;
    println!("✅ Plugin shut down successfully");

    Ok(())
}

async fn plugin_with_dependencies() -> Result<()> {
    println!("Creating Node.js plugin with npm dependencies...");

    // Create tool info for data processor
    let processor_info = ToolInfo {
        name: "data_processor".to_string(),
        version: "1.0.0".to_string(),
        description: "A data processing tool using lodash".to_string(),
        category: Some("data".to_string()),
        tags: vec![
            "data".to_string(),
            "processing".to_string(),
            "lodash".to_string(),
        ],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["filter", "map", "group_by", "sort", "unique", "aggregate", "transform"]
                },
                "data": {"type": "array"},
                "options": {"type": "object"}
            },
            "required": ["operation", "data"]
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "result": {},
                "operation": {"type": "string"}
            }
        }),
        plugin_name: Some("nodejs-data-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Build the plugin with package.json
    let mut plugin = NodeJsPluginBuilder::new()
        .name("nodejs-data-plugin")
        .version("1.0.0")
        .description("Node.js plugin with lodash dependency")
        .package_json("examples/nodejs_tools/package.json")
        .working_directory("examples/nodejs_tools")
        .auto_install_dependencies(true)
        .add_tool(
            processor_info,
            PathBuf::from("data_processor.js"),
            Some(Duration::from_secs(60)),
        )
        .build()
        .await?;

    println!(
        "✅ Plugin with dependencies created: {}",
        plugin.info().name
    );

    // Initialize the plugin
    let config = PluginConfig::new(
        "nodejs-data-plugin".to_string(),
        workflow_toolkit::core::PluginType::NodeJs,
    );
    plugin.initialize(config)?;

    // Initialize async components (this will install npm dependencies)
    println!("📦 Installing npm dependencies...");
    plugin.initialize_async().await?;

    println!("✅ Plugin initialized with dependencies");

    // Test the data processor tool
    println!("🔄 Testing data processor tool...");

    let tools = plugin.get_tools();
    if let Some(processor_tool) = tools.first() {
        let context = ExecutionContext::new();

        // Test filtering
        let params = json!({
            "operation": "filter",
            "data": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            "options": {
                "predicate": {
                    "type": "greater_than",
                    "value": 5
                }
            }
        });

        let result = processor_tool.execute(params, context.clone()).await?;
        println!(
            "📊 Filter result: {}",
            serde_json::to_string_pretty(&result)?
        );

        // Test aggregation
        let params = json!({
            "operation": "aggregate",
            "data": [10, 20, 30, 40, 50],
            "options": {
                "aggregation": {
                    "type": "sum"
                }
            }
        });

        let result = processor_tool.execute(params, context).await?;
        println!(
            "📊 Aggregation result: {}",
            serde_json::to_string_pretty(&result)?
        );
    }

    // Shutdown the plugin
    plugin.shutdown()?;
    println!("✅ Plugin shut down successfully");

    Ok(())
}

async fn multiple_tools_plugin() -> Result<()> {
    println!("Creating Node.js plugin with multiple tools...");

    // Create tool infos
    let calculator_info = ToolInfo {
        name: "calculator".to_string(),
        version: "1.0.0".to_string(),
        description: "Calculator tool".to_string(),
        category: Some("math".to_string()),
        tags: vec!["calculator".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("multi-tool-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let processor_info = ToolInfo {
        name: "processor".to_string(),
        version: "1.0.0".to_string(),
        description: "Data processor tool".to_string(),
        category: Some("data".to_string()),
        tags: vec!["processor".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("multi-tool-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Build the plugin with multiple tools
    let mut plugin = NodeJsPluginBuilder::new()
        .name("multi-tool-plugin")
        .version("1.0.0")
        .description("Node.js plugin with multiple tools")
        .package_json("examples/nodejs_tools/package.json")
        .working_directory("examples/nodejs_tools")
        .add_tool(
            calculator_info,
            PathBuf::from("simple_calculator.js"),
            Some(Duration::from_secs(30)),
        )
        .add_tool(
            processor_info,
            PathBuf::from("data_processor.js"),
            Some(Duration::from_secs(60)),
        )
        .build()
        .await?;

    println!(
        "✅ Multi-tool plugin created with {} tools",
        plugin.get_tools().len()
    );

    // Initialize the plugin
    let config = PluginConfig::new(
        "multi-tool-plugin".to_string(),
        workflow_toolkit::core::PluginType::NodeJs,
    );
    plugin.initialize(config)?;
    plugin.initialize_async().await?;

    // Test both tools
    let tools = plugin.get_tools();
    let context = ExecutionContext::new();

    for (i, tool) in tools.iter().enumerate() {
        println!("🔧 Testing tool {}: {}", i + 1, tool.name());

        if tool.name() == "calculator" {
            let params = json!({
                "operation": "multiply",
                "a": 7,
                "b": 8
            });
            let result = tool.execute(params, context.clone()).await?;
            println!(
                "📊 Calculator result: {}",
                serde_json::to_string_pretty(&result)?
            );
        } else if tool.name() == "processor" {
            let params = json!({
                "operation": "sort",
                "data": [3, 1, 4, 1, 5, 9, 2, 6],
                "options": {
                    "order": "desc"
                }
            });
            let result = tool.execute(params, context.clone()).await?;
            println!(
                "📊 Processor result: {}",
                serde_json::to_string_pretty(&result)?
            );
        }
    }

    plugin.shutdown()?;
    println!("✅ Multi-tool plugin shut down successfully");

    Ok(())
}

async fn error_handling_example() -> Result<()> {
    println!("Testing error handling and validation...");

    // Create a simple plugin for error testing
    let tool_info = ToolInfo {
        name: "error_test_calculator".to_string(),
        version: "1.0.0".to_string(),
        description: "Calculator for error testing".to_string(),
        category: Some("test".to_string()),
        tags: vec!["test".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("error-test-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let mut plugin = NodeJsPluginBuilder::new()
        .name("error-test-plugin")
        .version("1.0.0")
        .description("Plugin for error testing")
        .working_directory("examples/nodejs_tools")
        .add_tool(
            tool_info,
            PathBuf::from("simple_calculator.js"),
            Some(Duration::from_secs(10)),
        )
        .build()
        .await?;

    let config = PluginConfig::new(
        "error-test-plugin".to_string(),
        workflow_toolkit::core::PluginType::NodeJs,
    );
    plugin.initialize(config)?;
    plugin.initialize_async().await?;

    let tools = plugin.get_tools();
    if let Some(tool) = tools.first() {
        let context = ExecutionContext::new();

        // Test 1: Division by zero
        println!("🧪 Test 1: Division by zero");
        let params = json!({
            "operation": "divide",
            "a": 10,
            "b": 0
        });

        match tool.execute(params, context.clone()).await {
            Ok(result) => {
                println!(
                    "📊 Unexpected success: {}",
                    serde_json::to_string_pretty(&result)?
                );
            }
            Err(e) => {
                println!("✅ Expected error caught: {}", e);
            }
        }

        // Test 2: Invalid operation
        println!("🧪 Test 2: Invalid operation");
        let params = json!({
            "operation": "invalid_op",
            "a": 5,
            "b": 3
        });

        match tool.execute(params, context.clone()).await {
            Ok(result) => {
                println!(
                    "📊 Unexpected success: {}",
                    serde_json::to_string_pretty(&result)?
                );
            }
            Err(e) => {
                println!("✅ Expected error caught: {}", e);
            }
        }

        // Test 3: Missing parameters
        println!("🧪 Test 3: Missing parameters");
        let params = json!({
            "operation": "add",
            "a": 5
            // Missing 'b' parameter
        });

        match tool.execute(params, context.clone()).await {
            Ok(result) => {
                println!(
                    "📊 Unexpected success: {}",
                    serde_json::to_string_pretty(&result)?
                );
            }
            Err(e) => {
                println!("✅ Expected error caught: {}", e);
            }
        }

        // Test 4: Invalid parameter types
        println!("🧪 Test 4: Invalid parameter types");
        let params = json!({
            "operation": "add",
            "a": "not_a_number",
            "b": 5
        });

        match tool.execute(params, context).await {
            Ok(result) => {
                println!(
                    "📊 Unexpected success: {}",
                    serde_json::to_string_pretty(&result)?
                );
            }
            Err(e) => {
                println!("✅ Expected error caught: {}", e);
            }
        }
    }

    plugin.shutdown()?;
    println!("✅ Error handling tests completed");

    Ok(())
}
