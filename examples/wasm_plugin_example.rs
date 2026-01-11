//! Example demonstrating WASM plugin usage

use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;
use workflow_toolkit::core::{ExecutionContext, PluginInfo, PluginType};
use workflow_toolkit::error::Result;
use workflow_toolkit::plugins::{
    ExtismConfig, Plugin, PluginConfig, ResourceLimits, SecurityPolicy, WasmPlugin,
    WasmPluginBuilder, WasmRuntimeConfig, WasmRuntimeType,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 WASM Plugin Example");
    println!("======================");

    // Example 1: Create a WASM plugin using wasmtime runtime
    println!("\n📦 Creating WASM plugin with Wasmtime runtime...");

    let mut wasmtime_plugin = WasmPluginBuilder::new(
        "simple-calculator-wasmtime".to_string(),
        "1.0.0".to_string(),
    )
    .with_description("Simple calculator WASM plugin using Wasmtime".to_string())
    .with_author("Workflow Toolkit Team".to_string())
    .with_module_path(PathBuf::from("examples/wasm_tools/simple_calculator.wasm"))
    .with_memory_limit(32 * 1024 * 1024) // 32MB
    .with_timeout(Duration::from_secs(10))
    .with_fuel_limit(100_000)
    .with_runtime_type(WasmRuntimeType::Wasmtime)
    .add_entry_point("calculate".to_string(), "calculate".to_string())
    .build();

    println!(
        "✅ Wasmtime plugin created: {}",
        wasmtime_plugin.info().name
    );

    // Example 2: Create a WASM plugin using extism runtime for multi-language support
    println!("\n📦 Creating WASM plugin with Extism runtime...");

    let extism_config = ExtismConfig {
        allowed_hosts: vec!["httpbin.org".to_string()],
        allowed_paths: vec![PathBuf::from("/tmp")],
        config_data: [
            ("api_key".to_string(), "demo-key".to_string()),
            ("timeout".to_string(), "30".to_string()),
        ]
        .iter()
        .cloned()
        .collect(),
        memory_pages: Some(2),           // 128KB
        max_var_bytes: Some(512 * 1024), // 512KB
    };

    let mut extism_plugin =
        WasmPluginBuilder::new("multi-lang-processor".to_string(), "1.0.0".to_string())
            .with_description("Multi-language WASM plugin using Extism".to_string())
            .with_author("Workflow Toolkit Team".to_string())
            .with_module_path(PathBuf::from(
                "examples/wasm_tools/multi_lang_processor.wasm",
            ))
            .with_memory_limit(16 * 1024 * 1024) // 16MB
            .with_timeout(Duration::from_secs(15))
            .with_extism_config(extism_config)
            .add_entry_point("process_text".to_string(), "process_text".to_string())
            .add_entry_point("transform_data".to_string(), "transform_data".to_string())
            .build();

    println!("✅ Extism plugin created: {}", extism_plugin.info().name);

    // Example 3: Configure and initialize the wasmtime plugin
    println!("\n⚙️  Configuring Wasmtime plugin...");

    let wasmtime_config = PluginConfig {
        name: "simple-calculator-wasmtime".to_string(),
        plugin_type: PluginType::Wasm,
        enabled: true,
        config: json!({
            "module_path": "examples/wasm_tools/simple_calculator.wasm",
            "runtime_type": "Wasmtime",
            "entry_points": {
                "calculate": "calculate"
            }
        }),
        security_policy: SecurityPolicy {
            allow_network_access: false,
            allow_file_system_access: false,
            allowed_paths: vec![],
            environment_variables: std::collections::HashMap::new(),
            sandbox_enabled: true,
        },
        resource_limits: ResourceLimits {
            max_memory: Some(32 * 1024 * 1024),
            max_cpu_time: Some(Duration::from_secs(10)),
            max_execution_time: Some(Duration::from_secs(30)),
            max_file_size: None,
            max_network_connections: None,
        },
        dependencies: vec![],
        metadata: std::collections::HashMap::new(),
    };

    // Example 4: Configure extism plugin for multi-language support
    println!("\n⚙️  Configuring Extism plugin...");

    let extism_config = PluginConfig {
        name: "multi-lang-processor".to_string(),
        plugin_type: PluginType::Wasm,
        enabled: true,
        config: json!({
            "module_path": "examples/wasm_tools/multi_lang_processor.wasm",
            "runtime_type": "Extism",
            "entry_points": {
                "process_text": "process_text",
                "transform_data": "transform_data"
            },
            "extism_config": {
                "allowed_hosts": ["httpbin.org"],
                "config_data": {
                    "api_key": "demo-key",
                    "timeout": "30"
                }
            }
        }),
        security_policy: SecurityPolicy {
            allow_network_access: true, // Extism can handle network access securely
            allow_file_system_access: false,
            allowed_paths: vec![],
            environment_variables: std::collections::HashMap::new(),
            sandbox_enabled: true,
        },
        resource_limits: ResourceLimits {
            max_memory: Some(16 * 1024 * 1024),
            max_cpu_time: Some(Duration::from_secs(15)),
            max_execution_time: Some(Duration::from_secs(45)),
            max_file_size: None,
            max_network_connections: Some(5),
        },
        dependencies: vec![],
        metadata: std::collections::HashMap::new(),
    };

    // Try to initialize both plugins (will fail if WASM files don't exist)
    for (mut plugin, config, name) in [
        (
            Box::new(wasmtime_plugin) as Box<dyn Plugin>,
            wasmtime_config,
            "Wasmtime",
        ),
        (
            Box::new(extism_plugin) as Box<dyn Plugin>,
            extism_config,
            "Extism",
        ),
    ] {
        println!("\n🔧 Testing {} plugin...", name);

        match plugin.initialize(config) {
            Ok(_) => {
                println!("✅ {} plugin initialized successfully", name);

                // Get available tools
                println!("\n🔧 Available tools in {} plugin:", name);
                let tools = plugin.get_tools();
                for tool in &tools {
                    println!(
                        "  - {} (v{}): {}",
                        tool.name(),
                        tool.version(),
                        tool.get_info().description
                    );
                }

                // Execute a tool if available
                if let Some(tool) = tools.first() {
                    println!("\n🏃 Executing {} tool: {}", name, tool.name());

                    let params = json!({
                        "operation": "add",
                        "operands": [10, 20]
                    });

                    let context = ExecutionContext::new();

                    match tool.execute(params, context).await {
                        Ok(result) => {
                            println!("✅ {} tool execution result: {}", name, result);
                        }
                        Err(e) => {
                            println!("❌ {} tool execution failed: {}", name, e);
                        }
                    }
                }

                // Shutdown the plugin
                println!("\n🛑 Shutting down {} plugin...", name);
                plugin.shutdown().unwrap();
                println!("✅ {} plugin shut down successfully", name);
            }
            Err(e) => {
                println!("❌ {} plugin initialization failed: {}", name, e);
                println!("💡 This is expected if the WASM file doesn't exist");
                if name == "Wasmtime" {
                    println!(
                        "   To create a Wasmtime-compatible WASM module, compile the .wat file:"
                    );
                    println!("   wat2wasm examples/wasm_tools/simple_calculator.wat");
                } else {
                    println!("   To create an Extism-compatible WASM module:");
                    println!(
                        "   Use any language that compiles to WASM (Rust, Go, JavaScript, etc.)"
                    );
                }
            }
        }
    }

    // Example 5: Demonstrate WASM runtime configuration options
    println!("\n⚙️  WASM Runtime Configuration Options:");

    // Wasmtime configuration
    let wasmtime_config = WasmRuntimeConfig {
        module_path: PathBuf::from("examples/wasm_tools/simple_calculator.wasm"),
        memory_limit: 64 * 1024 * 1024, // 64MB
        timeout: Duration::from_secs(30),
        fuel_limit: Some(1_000_000), // Limit computational resources
        allow_wasi: false,           // Disable WASI for security
        allowed_imports: vec!["env.log".to_string()], // Only allow specific imports
        entry_points: [
            ("calculate".to_string(), "calculate".to_string()),
            ("validate".to_string(), "validate_input".to_string()),
        ]
        .iter()
        .cloned()
        .collect(),
        runtime_type: WasmRuntimeType::Wasmtime,
        extism_config: None,
    };

    println!("📋 Wasmtime Configuration:");
    println!(
        "  - Memory limit: {} MB",
        wasmtime_config.memory_limit / (1024 * 1024)
    );
    println!("  - Timeout: {:?}", wasmtime_config.timeout);
    println!("  - Fuel limit: {:?}", wasmtime_config.fuel_limit);
    println!("  - WASI enabled: {}", wasmtime_config.allow_wasi);
    println!("  - Entry points: {:?}", wasmtime_config.entry_points);

    // Extism configuration
    let extism_runtime_config = WasmRuntimeConfig {
        module_path: PathBuf::from("examples/wasm_tools/multi_lang_processor.wasm"),
        memory_limit: 32 * 1024 * 1024, // 32MB
        timeout: Duration::from_secs(45),
        fuel_limit: None,        // Extism handles resource limits differently
        allow_wasi: true,        // Extism can safely handle WASI
        allowed_imports: vec![], // Extism manages imports
        entry_points: [
            ("process_text".to_string(), "process_text".to_string()),
            ("transform_data".to_string(), "transform_data".to_string()),
            ("fetch_data".to_string(), "fetch_remote_data".to_string()),
        ]
        .iter()
        .cloned()
        .collect(),
        runtime_type: WasmRuntimeType::Extism,
        extism_config: Some(ExtismConfig {
            allowed_hosts: vec!["api.example.com".to_string(), "httpbin.org".to_string()],
            allowed_paths: vec![PathBuf::from("/tmp"), PathBuf::from("/var/data")],
            config_data: [
                ("api_key".to_string(), "your-api-key".to_string()),
                ("max_retries".to_string(), "3".to_string()),
                ("cache_ttl".to_string(), "300".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            memory_pages: Some(4),                // 256KB
            max_var_bytes: Some(2 * 1024 * 1024), // 2MB
        }),
    };

    println!("\n📋 Extism Configuration:");
    println!(
        "  - Memory limit: {} MB",
        extism_runtime_config.memory_limit / (1024 * 1024)
    );
    println!("  - Timeout: {:?}", extism_runtime_config.timeout);
    println!("  - WASI enabled: {}", extism_runtime_config.allow_wasi);
    println!("  - Entry points: {:?}", extism_runtime_config.entry_points);
    if let Some(ref extism_config) = extism_runtime_config.extism_config {
        println!("  - Allowed hosts: {:?}", extism_config.allowed_hosts);
        println!(
            "  - Config data keys: {:?}",
            extism_config.config_data.keys().collect::<Vec<_>>()
        );
        println!("  - Memory pages: {:?}", extism_config.memory_pages);
    }

    println!("\n🎉 WASM plugin example completed!");

    Ok(())
}

/// Helper function to demonstrate WASM plugin creation patterns
fn create_wasm_plugin_examples() -> Vec<Box<dyn Plugin>> {
    vec![
        // Example 1: Simple calculator using Wasmtime
        Box::new(
            WasmPluginBuilder::new("calculator-wasmtime".to_string(), "1.0.0".to_string())
                .with_description("Mathematical calculator using Wasmtime".to_string())
                .with_module_path(PathBuf::from("plugins/calculator.wasm"))
                .with_runtime_type(WasmRuntimeType::Wasmtime)
                .add_entry_point("add".to_string(), "add_numbers".to_string())
                .add_entry_point("multiply".to_string(), "multiply_numbers".to_string())
                .build(),
        ),
        // Example 2: Text processor using Extism for multi-language support
        Box::new(
            WasmPluginBuilder::new("text-processor-extism".to_string(), "2.1.0".to_string())
                .with_description("Text processing utilities using Extism".to_string())
                .with_author("Text Team".to_string())
                .with_module_path(PathBuf::from("plugins/text_processor.wasm"))
                .with_memory_limit(16 * 1024 * 1024) // 16MB
                .with_timeout(Duration::from_secs(5))
                .with_runtime_type(WasmRuntimeType::Extism)
                .add_config_data("language".to_string(), "en".to_string())
                .add_config_data("max_length".to_string(), "10000".to_string())
                .add_entry_point("uppercase".to_string(), "to_uppercase".to_string())
                .add_entry_point("lowercase".to_string(), "to_lowercase".to_string())
                .add_entry_point("reverse".to_string(), "reverse_text".to_string())
                .build(),
        ),
        // Example 3: Data validator with strict limits using Wasmtime
        Box::new(
            WasmPluginBuilder::new("validator-wasmtime".to_string(), "1.5.0".to_string())
                .with_description("Data validation engine using Wasmtime".to_string())
                .with_module_path(PathBuf::from("plugins/validator.wasm"))
                .with_memory_limit(8 * 1024 * 1024) // 8MB - strict limit
                .with_timeout(Duration::from_secs(2)) // Fast timeout
                .with_fuel_limit(50_000) // Limited computation
                .with_runtime_type(WasmRuntimeType::Wasmtime)
                .add_entry_point(
                    "validate_json".to_string(),
                    "validate_json_schema".to_string(),
                )
                .add_entry_point(
                    "validate_email".to_string(),
                    "validate_email_format".to_string(),
                )
                .build(),
        ),
        // Example 4: Network-enabled processor using Extism
        Box::new(
            WasmPluginBuilder::new("network-processor".to_string(), "3.0.0".to_string())
                .with_description("Network-enabled data processor using Extism".to_string())
                .with_module_path(PathBuf::from("plugins/network_processor.wasm"))
                .with_memory_limit(64 * 1024 * 1024) // 64MB
                .with_timeout(Duration::from_secs(30))
                .with_runtime_type(WasmRuntimeType::Extism)
                .add_allowed_host("api.example.com".to_string())
                .add_allowed_host("httpbin.org".to_string())
                .add_config_data("api_key".to_string(), "demo-key".to_string())
                .add_config_data("timeout".to_string(), "10".to_string())
                .add_entry_point("fetch_data".to_string(), "fetch_remote_data".to_string())
                .add_entry_point(
                    "process_response".to_string(),
                    "process_api_response".to_string(),
                )
                .build(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_plugin_creation() {
        let plugins = create_wasm_plugin_examples();
        assert_eq!(plugins.len(), 4);

        // Test different runtime types
        let wasmtime_plugin =
            WasmPluginBuilder::new("test-wasmtime".to_string(), "1.0.0".to_string())
                .with_runtime_type(WasmRuntimeType::Wasmtime)
                .build();
        assert_eq!(wasmtime_plugin.info().name, "test-wasmtime");
        assert_eq!(wasmtime_plugin.info().plugin_type, PluginType::Wasm);

        let extism_plugin = WasmPluginBuilder::new("test-extism".to_string(), "1.0.0".to_string())
            .with_runtime_type(WasmRuntimeType::Extism)
            .build();
        assert_eq!(extism_plugin.info().name, "test-extism");
        assert_eq!(extism_plugin.info().plugin_type, PluginType::Wasm);
    }

    #[tokio::test]
    async fn test_wasm_runtime_config() {
        let wasmtime_config = WasmRuntimeConfig {
            runtime_type: WasmRuntimeType::Wasmtime,
            ..WasmRuntimeConfig::default()
        };
        assert_eq!(wasmtime_config.memory_limit, 64 * 1024 * 1024);
        assert_eq!(wasmtime_config.timeout, Duration::from_secs(30));
        assert!(matches!(
            wasmtime_config.runtime_type,
            WasmRuntimeType::Wasmtime
        ));

        let extism_config = WasmRuntimeConfig {
            runtime_type: WasmRuntimeType::Extism,
            extism_config: Some(ExtismConfig::default()),
            ..WasmRuntimeConfig::default()
        };
        assert!(matches!(
            extism_config.runtime_type,
            WasmRuntimeType::Extism
        ));
        assert!(extism_config.extism_config.is_some());
    }

    #[test]
    fn test_extism_config() {
        let config = ExtismConfig {
            allowed_hosts: vec!["example.com".to_string()],
            config_data: [("key".to_string(), "value".to_string())]
                .iter()
                .cloned()
                .collect(),
            memory_pages: Some(2),
            max_var_bytes: Some(1024),
            ..ExtismConfig::default()
        };

        assert_eq!(config.allowed_hosts.len(), 1);
        assert_eq!(config.config_data.get("key"), Some(&"value".to_string()));
        assert_eq!(config.memory_pages, Some(2));
        assert_eq!(config.max_var_bytes, Some(1024));
    }
}
