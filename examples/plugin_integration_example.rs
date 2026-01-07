//! Example demonstrating improved plugin registration system
//! 
//! This example shows how the file management plugin integrates with the
//! workflow-toolkit tool registry, satisfying requirements 7.1 and 7.2.

use workflow_toolkit::{
    core::{ExecutionContext, PluginType},
    error::Result,
    plugins::{
        IntegratedPluginSystem, IntegratedPluginSystemBuilder,
        FileManagementPlugin, FileManagementConfig,
        types::{PluginConfig, SecurityPolicy, ResourceLimits},
    },
};
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting plugin integration example");

    // Create a temporary directory for the plugin
    let temp_dir = TempDir::new().map_err(|e| {
        workflow_toolkit::error::WorkflowError::Io(e)
    })?;

    // Create an integrated plugin system
    let mut system = IntegratedPluginSystem::new();

    info!("Created integrated plugin system");

    // Create a file management plugin
    let plugin = FileManagementPlugin::builder()
        .temp_directory(temp_dir.path())
        .max_threads(4)
        .enable_chinese_processing(true)
        .default_experimental_mode(false)
        .build()?;

    // Create plugin configuration
    let config = PluginConfig {
        name: "file-management".to_string(),
        plugin_type: PluginType::Native,
        enabled: true,
        config: serde_json::to_value(&FileManagementConfig {
            max_threads: 4,
            temp_directory: temp_dir.path().to_path_buf(),
            default_encoding: "utf-8".to_string(),
            enable_chinese_processing: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_batch_size: 1000,
            default_experimental_mode: false,
            human_decision_timeout: Some(300),
        })?,
        security_policy: SecurityPolicy {
            allow_file_system_access: true,
            allow_network_access: false,
            allowed_paths: vec![temp_dir.path().to_path_buf()],
            environment_variables: HashMap::new(),
            sandbox_enabled: false,
        },
        resource_limits: ResourceLimits {
            max_memory: Some(2 * 1024 * 1024 * 1024), // 2GB
            max_cpu_time: None,
            max_execution_time: Some(std::time::Duration::from_secs(3600)), // 1 hour
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_network_connections: Some(0),
        },
        dependencies: Vec::new(),
        metadata: HashMap::new(),
    };

    // Load the plugin - this will automatically register all tools with the main tool registry
    info!("Loading file management plugin...");
    system.load_plugin(Box::new(plugin), config)?;

    // Demonstrate requirement 7.1: Tools are registered with the workflow-toolkit tool registry
    info!("=== Requirement 7.1: Tool Registration ===");
    let tools = system.list_all_tools()?;
    info!("Total tools registered: {}", tools.len());
    
    for tool in &tools {
        info!("  - {} v{}: {}", tool.name, tool.version, tool.description);
        if let Some(category) = &tool.category {
            info!("    Category: {}", category);
        }
        if !tool.tags.is_empty() {
            info!("    Tags: {}", tool.tags.join(", "));
        }
    }

    // Demonstrate requirement 7.2: Tools accept parameters through standard workflow parameter system
    info!("\n=== Requirement 7.2: Standard Parameter System ===");
    
    // Test the text processor tool
    if system.has_tool("text-processor")? {
        info!("Testing text-processor tool with standard parameters...");
        
        let params = json!({
            "text": "Hello World 测试",
            "operations": ["normalize_case", "remove_punctuation"],
            "experimental_mode": true
        });
        
        // Validate parameters using standard validation
        match system.validate_tool_params("text-processor", &params) {
            Ok(()) => info!("✓ Parameter validation passed"),
            Err(e) => info!("✗ Parameter validation failed: {}", e),
        }
        
        // Execute tool using standard execution context
        let context = ExecutionContext::new();
        match system.execute_tool("text-processor", params, context).await {
            Ok(result) => {
                info!("✓ Tool execution successful");
                info!("Result: {}", serde_json::to_string_pretty(&result)?);
            }
            Err(e) => info!("✗ Tool execution failed: {}", e),
        }
    }

    // Test the AC matcher tool
    if system.has_tool("ac-matcher")? {
        info!("\nTesting ac-matcher tool with standard parameters...");
        
        let params = json!({
            "text": "hello world test hello",
            "patterns": [
                {"pattern": "hello", "category": "greeting", "score": 1.0},
                {"pattern": "world", "category": "noun", "score": 0.8},
                {"pattern": "test", "category": "action", "score": 1.2}
            ],
            "case_sensitive": false,
            "find_overlapping": false,
            "experimental_mode": true
        });
        
        // Validate parameters
        match system.validate_tool_params("ac-matcher", &params) {
            Ok(()) => info!("✓ Parameter validation passed"),
            Err(e) => info!("✗ Parameter validation failed: {}", e),
        }
        
        // Execute tool
        let context = ExecutionContext::new();
        match system.execute_tool("ac-matcher", params, context).await {
            Ok(result) => {
                info!("✓ Tool execution successful");
                if let Some(total_matches) = result.get("total_matches") {
                    info!("Found {} matches", total_matches);
                }
                if let Some(categories) = result.get("categories_found") {
                    info!("Categories found: {}", categories);
                }
            }
            Err(e) => info!("✗ Tool execution failed: {}", e),
        }
    }

    // Test the folder classifier tool
    if system.has_tool("folder-classifier")? {
        info!("\nTesting folder-classifier tool with standard parameters...");
        
        let params = json!({
            "folder_paths": ["/tmp/test1", "/tmp/test2"],
            "classification_rules": {
                "categories": {
                    "documents": {
                        "keywords": ["doc", "pdf", "text"],
                        "weight": 1.0
                    },
                    "media": {
                        "keywords": ["video", "audio", "image"],
                        "weight": 1.0
                    }
                }
            },
            "experimental_mode": true
        });
        
        // Validate parameters
        match system.validate_tool_params("folder-classifier", &params) {
            Ok(()) => info!("✓ Parameter validation passed"),
            Err(e) => info!("✗ Parameter validation failed: {}", e),
        }
    }

    // Demonstrate plugin lifecycle management
    info!("\n=== Plugin Lifecycle Management ===");
    let plugins = system.list_plugins()?;
    for plugin_info in plugins {
        info!("Plugin: {} v{}", plugin_info.name, plugin_info.version);
        if let Some(description) = plugin_info.description {
            info!("  Description: {}", description);
        }
        
        let status = system.get_plugin_status(&plugin_info.name)?;
        if let Some(status) = status {
            info!("  Status: {:?}", status);
        }
    }

    // Demonstrate tool registry integration
    info!("\n=== Tool Registry Integration ===");
    info!("Total tools in registry: {}", system.tool_count()?);
    
    // Test tool existence checks
    let test_tools = ["text-processor", "ac-matcher", "folder-classifier", "file-mover", "batch-processor"];
    for tool_name in test_tools {
        let exists = system.has_tool(tool_name)?;
        info!("Tool '{}': {}", tool_name, if exists { "✓ Available" } else { "✗ Not found" });
    }

    // Shutdown the system
    info!("\n=== Shutdown ===");
    system.shutdown_all()?;
    info!("System shut down successfully");
    
    // Verify tools are unregistered
    let final_tool_count = system.tool_count()?;
    info!("Tools remaining after shutdown: {}", final_tool_count);

    info!("Plugin integration example completed successfully");
    Ok(())
}