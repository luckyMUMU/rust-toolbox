/// Unit tests for Node.js plugin functionality

use super::*;
use crate::core::{ExecutionContext, PluginType, ToolInfo};
use crate::plugins::types::{Plugin, PluginConfig};
use chrono::Utc;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio;

/// Create a temporary Node.js script for testing
async fn create_test_script(temp_dir: &TempDir, script_name: &str, script_content: &str) -> PathBuf {
    let script_path = temp_dir.path().join(script_name);
    tokio::fs::write(&script_path, script_content).await.unwrap();
    script_path
}

/// Simple test script that echoes input
const ECHO_SCRIPT: &str = r#"
const readline = require('readline');
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

let inputData = '';
rl.on('line', (line) => {
    inputData += line + '\n';
});

rl.on('close', () => {
    try {
        const input = JSON.parse(inputData.trim());
        const result = {
            success: true,
            echo: input.params,
            timestamp: new Date().toISOString()
        };
        console.log(JSON.stringify(result));
        process.exit(0);
    } catch (error) {
        const errorOutput = {
            success: false,
            error: error.message
        };
        console.log(JSON.stringify(errorOutput));
        process.exit(1);
    }
});
"#;

/// Test script that performs simple math
const MATH_SCRIPT: &str = r#"
const readline = require('readline');
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

let inputData = '';
rl.on('line', (line) => {
    inputData += line + '\n';
});

rl.on('close', () => {
    try {
        const input = JSON.parse(inputData.trim());
        const params = input.params || {};
        
        if (typeof params.a !== 'number' || typeof params.b !== 'number') {
            throw new Error('Parameters a and b must be numbers');
        }
        
        const result = {
            success: true,
            sum: params.a + params.b,
            product: params.a * params.b,
            timestamp: new Date().toISOString()
        };
        console.log(JSON.stringify(result));
        process.exit(0);
    } catch (error) {
        const errorOutput = {
            success: false,
            error: error.message
        };
        console.log(JSON.stringify(errorOutput));
        process.exit(1);
    }
});
"#;

#[tokio::test]
async fn test_nodejs_runtime_config_default() {
    let config = NodeJsRuntimeConfig::default();
    assert_eq!(config.node_executable, "node");
    assert_eq!(config.npm_executable, "npm");
    assert!(config.auto_install_dependencies);
    assert!(config.environment_variables.is_empty());
    assert!(config.module_paths.is_empty());
}

#[tokio::test]
async fn test_nodejs_environment_creation() {
    let config = NodeJsRuntimeConfig::default();
    let env = NodeJsEnvironment::new(config);
    
    assert!(!env.is_initialized());
    assert_eq!(env.node_executable(), PathBuf::from("node"));
    assert_eq!(env.npm_executable(), PathBuf::from("npm"));
}

#[tokio::test]
async fn test_nodejs_plugin_builder() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_test_script(&temp_dir, "test.js", ECHO_SCRIPT).await;

    let tool_info = ToolInfo {
        name: "test_tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Test tool".to_string(),
        category: Some("test".to_string()),
        tags: vec!["test".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("test-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let plugin: NodeJsPlugin = NodeJsPluginBuilder::new()
        .name("test-plugin")
        .version("1.0.0")
        .description("Test Node.js plugin")
        .working_directory(temp_dir.path())
        .auto_install_dependencies(false)
        .add_tool(tool_info, script_path.file_name().unwrap().into(), None)
        .build()
        .await
        .unwrap();

    assert_eq!(plugin.info().name, "test-plugin");
    assert_eq!(plugin.info().version, "1.0.0");
    assert_eq!(plugin.info().plugin_type, PluginType::NodeJs);
    assert_eq!(plugin.get_tools().len(), 1);
}

#[tokio::test]
async fn test_nodejs_plugin_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_test_script(&temp_dir, "test.js", ECHO_SCRIPT).await;

    let tool_info = ToolInfo {
        name: "test_tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Test tool".to_string(),
        category: Some("test".to_string()),
        tags: vec!["test".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("test-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let mut plugin: NodeJsPlugin = NodeJsPluginBuilder::new()
        .name("test-plugin")
        .version("1.0.0")
        .working_directory(temp_dir.path())
        .auto_install_dependencies(false)
        .add_tool(tool_info, script_path.file_name().unwrap().into(), None)
        .build()
        .await
        .unwrap();

    // Test initialization
    let config = PluginConfig::new("test-plugin".to_string(), crate::core::PluginType::NodeJs);
    plugin.initialize(config).unwrap();
    
    // Skip async initialization in tests that don't have Node.js available
    // plugin.initialize_async().await.unwrap();
    
    assert!(plugin.is_initialized());
}

#[tokio::test]
async fn test_nodejs_tool_execution() {
    // Skip this test if Node.js is not available
    if !is_nodejs_available().await {
        println!("Skipping Node.js execution test - Node.js not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let script_path = create_test_script(&temp_dir, "echo.js", ECHO_SCRIPT).await;

    let tool_info = ToolInfo {
        name: "echo_tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Echo tool".to_string(),
        category: Some("test".to_string()),
        tags: vec!["test".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("test-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let mut plugin: NodeJsPlugin = NodeJsPluginBuilder::new()
        .name("test-plugin")
        .version("1.0.0")
        .working_directory(temp_dir.path())
        .auto_install_dependencies(false)
        .add_tool(tool_info, script_path.file_name().unwrap().into(), Some(Duration::from_secs(10)))
        .build()
        .await
        .unwrap();

    let config = PluginConfig::new("test-plugin".to_string(), crate::core::PluginType::NodeJs);
    plugin.initialize(config).unwrap();
    plugin.initialize_async().await.unwrap();

    let tools = plugin.get_tools();
    let tool = tools.first().unwrap();

    let context = ExecutionContext::new();
    let params = json!({
        "message": "Hello, World!",
        "number": 42
    });

    let result: Value = tool.execute(params.clone(), context).await.unwrap();
    
    assert!(result.get("success").and_then(|v: &Value| v.as_bool()).unwrap_or(false));
    assert_eq!(result.get("echo"), Some(&params));
}

#[tokio::test]
async fn test_nodejs_tool_math_execution() {
    // Skip this test if Node.js is not available
    if !is_nodejs_available().await {
        println!("Skipping Node.js math test - Node.js not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let script_path = create_test_script(&temp_dir, "math.js", MATH_SCRIPT).await;

    let tool_info = ToolInfo {
        name: "math_tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Math tool".to_string(),
        category: Some("math".to_string()),
        tags: vec!["math".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("test-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let mut plugin: NodeJsPlugin = NodeJsPluginBuilder::new()
        .name("test-plugin")
        .version("1.0.0")
        .working_directory(temp_dir.path())
        .auto_install_dependencies(false)
        .add_tool(tool_info, script_path.file_name().unwrap().into(), Some(Duration::from_secs(10)))
        .build()
        .await
        .unwrap();

    let config = PluginConfig::new("test-plugin".to_string(), crate::core::PluginType::NodeJs);
    plugin.initialize(config).unwrap();
    plugin.initialize_async().await.unwrap();

    let tools = plugin.get_tools();
    let tool = tools.first().unwrap();

    let context = ExecutionContext::new();
    let params = json!({
        "a": 10,
        "b": 5
    });

    let result: Value = tool.execute(params, context).await.unwrap();
    
    assert!(result.get("success").and_then(|v: &Value| v.as_bool()).unwrap_or(false));
    assert_eq!(result.get("sum").and_then(|v: &Value| v.as_i64()), Some(15));
    assert_eq!(result.get("product").and_then(|v: &Value| v.as_i64()), Some(50));
}

#[tokio::test]
async fn test_nodejs_tool_error_handling() {
    // Skip this test if Node.js is not available
    if !is_nodejs_available().await {
        println!("Skipping Node.js error test - Node.js not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let script_path = create_test_script(&temp_dir, "math.js", MATH_SCRIPT).await;

    let tool_info = ToolInfo {
        name: "math_tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Math tool".to_string(),
        category: Some("math".to_string()),
        tags: vec!["math".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("test-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let mut plugin: NodeJsPlugin = NodeJsPluginBuilder::new()
        .name("test-plugin")
        .version("1.0.0")
        .working_directory(temp_dir.path())
        .auto_install_dependencies(false)
        .add_tool(tool_info, script_path.file_name().unwrap().into(), Some(Duration::from_secs(10)))
        .build()
        .await
        .unwrap();

    let config = PluginConfig::new("test-plugin".to_string(), crate::core::PluginType::NodeJs);
    plugin.initialize(config).unwrap();
    plugin.initialize_async().await.unwrap();

    let tools = plugin.get_tools();
    let tool = tools.first().unwrap();

    let context = ExecutionContext::new();
    
    // Test with invalid parameters (missing 'b')
    let params = json!({
        "a": 10
        // Missing 'b' parameter
    });

    let result = tool.execute(params, context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_package_json_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let package_json_path = temp_dir.path().join("package.json");
    
    let package_content = json!({
        "name": "test-package",
        "version": "1.0.0",
        "description": "Test package",
        "main": "index.js",
        "dependencies": {
            "lodash": "^4.17.21"
        },
        "devDependencies": {
            "jest": "^29.0.0"
        }
    });

    tokio::fs::write(&package_json_path, serde_json::to_string_pretty(&package_content).unwrap())
        .await
        .unwrap();

    let package_json: PackageJson = PackageJson::load_from_file(&package_json_path).await.unwrap();
    
    assert_eq!(package_json.name, "test-package");
    assert_eq!(package_json.version, "1.0.0");
    assert_eq!(package_json.description, Some("Test package".to_string()));
    assert_eq!(package_json.main, Some("index.js".to_string()));
    
    let all_deps = package_json.get_all_dependencies();
    assert_eq!(all_deps.len(), 2);
    assert_eq!(all_deps.get("lodash"), Some(&"^4.17.21".to_string()));
    assert_eq!(all_deps.get("jest"), Some(&"^29.0.0".to_string()));
}

#[tokio::test]
async fn test_nodejs_plugin_shutdown() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = create_test_script(&temp_dir, "test.js", ECHO_SCRIPT).await;

    let tool_info = ToolInfo {
        name: "test_tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Test tool".to_string(),
        category: Some("test".to_string()),
        tags: vec!["test".to_string()],
        parameters_schema: json!({"type": "object"}),
        return_schema: json!({"type": "object"}),
        plugin_name: Some("test-plugin".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let mut plugin: NodeJsPlugin = NodeJsPluginBuilder::new()
        .name("test-plugin")
        .version("1.0.0")
        .working_directory(temp_dir.path())
        .auto_install_dependencies(false)
        .add_tool(tool_info, script_path.file_name().unwrap().into(), None)
        .build()
        .await
        .unwrap();

    let config = PluginConfig::new("test-plugin".to_string(), crate::core::PluginType::NodeJs);
    plugin.initialize(config).unwrap();
    
    assert!(plugin.is_initialized());
    assert!(!plugin.get_tools().is_empty());

    plugin.shutdown().unwrap();
    
    assert!(!plugin.is_initialized());
    assert!(plugin.get_tools().is_empty());
}

/// Helper function to check if Node.js is available
async fn is_nodejs_available() -> bool {
    tokio::process::Command::new("node")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}