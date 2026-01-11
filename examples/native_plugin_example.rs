//! Example of creating and using a native plugin

use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use workflow_toolkit::core::{PluginInfo, PluginType};
use workflow_toolkit::plugins::{
    NativePluginBuilder, Plugin, PluginConfig, ResourceLimits, SecurityPolicy,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Native Plugin Example");
    println!("====================");

    // Create plugin info
    let plugin_info = PluginInfo {
        name: "example-native-plugin".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::Native,
        description: Some("Example native plugin for demonstration".to_string()),
        author: Some("Workflow Toolkit Team".to_string()),
        metadata: std::collections::HashMap::new(),
    };

    // Create plugin configuration
    let plugin_config = PluginConfig {
        name: "example-native-plugin".to_string(),
        plugin_type: PluginType::Native,
        enabled: true,
        config: json!({
            "debug": true,
            "timeout": 30
        }),
        security_policy: SecurityPolicy {
            allow_network_access: false,
            allow_file_system_access: true,
            allowed_paths: vec![PathBuf::from("/tmp")],
            environment_variables: HashMap::new(),
            sandbox_enabled: true,
        },
        resource_limits: ResourceLimits::default(),
        dependencies: vec![],
        metadata: HashMap::new(),
    };

    // Create native plugin
    let library_path = PathBuf::from("./target/debug/libexample_plugin.so");

    println!("Creating native plugin...");
    let mut plugin = NativePluginBuilder::new()
        .info(plugin_info)
        .library_path(library_path.clone())
        .build()?;

    println!("Plugin created: {}", plugin.info().name);
    println!("Status: {:?}", plugin.status());

    // Note: This example won't actually work without a real dynamic library
    // In a real scenario, you would:
    // 1. Build a dynamic library with the required plugin API functions
    // 2. Initialize the plugin with the library path
    // 3. Use the plugin's tools

    if library_path.exists() {
        println!("Initializing plugin...");
        match plugin.initialize(plugin_config) {
            Ok(()) => {
                println!("Plugin initialized successfully!");
                println!("Status: {:?}", plugin.status());
                println!("Available tools: {}", plugin.get_tools().len());

                // List tools
                for tool in plugin.get_tools() {
                    println!("  - {} v{}", tool.name(), tool.version());
                }

                // Shutdown plugin
                println!("Shutting down plugin...");
                plugin.shutdown()?;
                println!("Plugin shutdown complete. Status: {:?}", plugin.status());
            }
            Err(e) => {
                println!("Failed to initialize plugin: {}", e);
                println!("This is expected since we don't have a real dynamic library");
            }
        }
    } else {
        println!("Library file not found: {:?}", library_path);
        println!("This is expected in the example - you would need to build a real plugin library");
    }

    println!("\nNative Plugin API Requirements:");
    println!("==============================");
    println!("A native plugin dynamic library must export these C functions:");
    println!("- plugin_info() -> *const c_char");
    println!("- plugin_init(*const c_char) -> *mut c_void");
    println!("- plugin_get_tools(*mut c_void) -> *const ToolDescriptor");
    println!("- plugin_execute_tool(*mut c_void, *const c_char, *const c_char, *const c_char) -> *const c_char");
    println!("- plugin_shutdown(*mut c_void)");

    Ok(())
}
