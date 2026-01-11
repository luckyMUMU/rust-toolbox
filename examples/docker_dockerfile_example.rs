//! Example demonstrating Docker plugin with custom Dockerfile

use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use workflow_toolkit::core::{ExecutionContext, PluginInfo, PluginType, ToolInfo};
use workflow_toolkit::error::Result;
use workflow_toolkit::plugins::{
    DockerMount, DockerMountType, DockerPluginBuilder, DockerResourceLimits, DockerToolConfig,
    Plugin, PluginConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Docker Plugin with Dockerfile Example");
    println!("====================================");

    // Create a Docker plugin that builds from a Dockerfile
    let plugin_info = PluginInfo {
        name: "custom-processor".to_string(),
        version: "1.0.0".to_string(),
        description: "Custom Docker-based text processing tools".to_string(),
        plugin_type: PluginType::Docker,
        author: Some("Workflow Toolkit Team".to_string()),
        license: Some("MIT".to_string()),
        repository: None,
        dependencies: vec![],
        tools: vec!["text_processor".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create Docker tool configuration using our custom image
    let processor_tool_config = DockerToolConfig {
        image: "workflow-toolkit/simple-processor:latest".to_string(),
        command: None, // Use default CMD from Dockerfile
        entrypoint: None,
        working_dir: Some("/app".to_string()),
        environment: {
            let mut env = HashMap::new();
            env.insert("TOOL_NAME".to_string(), "text_processor".to_string());
            env.insert("TOOL_VERSION".to_string(), "1.0.0".to_string());
            env
        },
        mounts: vec![
            DockerMount {
                source: PathBuf::from("/tmp/workflow-input"),
                target: "/app/input".to_string(),
                mount_type: DockerMountType::Bind,
                read_only: true,
            },
            DockerMount {
                source: PathBuf::from("/tmp/workflow-output"),
                target: "/app/output".to_string(),
                mount_type: DockerMountType::Bind,
                read_only: false,
            },
        ],
        ports: HashMap::new(),
        resource_limits: Some(DockerResourceLimits {
            memory: Some(256 * 1024 * 1024),      // 256MB
            memory_swap: Some(512 * 1024 * 1024), // 512MB
            nano_cpus: Some(250_000_000),         // 0.25 CPU
            cpu_shares: None,
            pids_limit: Some(50),
        }),
        network_config: None, // Use default network
        labels: {
            let mut labels = HashMap::new();
            labels.insert("tool.category".to_string(), "text-processing".to_string());
            labels.insert("tool.language".to_string(), "bash".to_string());
            labels
        },
        user: Some("1000:1000".to_string()), // Run as non-root
        privileged: false,
    };

    // Create tool info
    let processor_tool_info = ToolInfo {
        name: "text_processor".to_string(),
        version: "1.0.0".to_string(),
        description: "Process text using various operations in a Docker container".to_string(),
        category: Some("text-processing".to_string()),
        tags: vec![
            "text".to_string(),
            "processing".to_string(),
            "docker".to_string(),
        ],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["echo", "uppercase", "lowercase", "reverse", "length", "base64_encode", "base64_decode", "json_pretty", "word_count", "line_count"],
                    "description": "Text processing operation to perform"
                },
                "input": {
                    "type": "string",
                    "description": "Input text to process"
                },
                "delay": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 60,
                    "description": "Optional delay in seconds before processing"
                }
            },
            "required": ["operation", "input"]
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "result": {
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string"},
                        "input": {"type": "string"},
                        "output": {"type": "string"},
                        "processed_at": {"type": "string"},
                        "container_hostname": {"type": "string"},
                        "container_user": {"type": "string"}
                    }
                },
                "metadata": {
                    "type": "object",
                    "properties": {
                        "execution_time_seconds": {"type": "integer"},
                        "tool_version": {"type": "string"},
                        "container_os": {"type": "string"},
                        "container_arch": {"type": "string"}
                    }
                }
            }
        }),
        plugin_name: Some("custom-processor".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Build the Docker plugin
    println!("Building Docker plugin...");
    let mut plugin = DockerPluginBuilder::new()
        .name("custom-processor")
        .version("1.0.0")
        .description("Custom Docker-based text processing tools")
        .default_working_dir("/app")
        .add_default_environment_variable("LANG", "C.UTF-8")
        .add_default_environment_variable("LC_ALL", "C.UTF-8")
        .resource_limits(DockerResourceLimits {
            memory: Some(512 * 1024 * 1024),       // 512MB default
            memory_swap: Some(1024 * 1024 * 1024), // 1GB
            nano_cpus: Some(500_000_000),          // 0.5 CPU
            cpu_shares: None,
            pids_limit: Some(100),
        })
        .auto_remove(true)
        .execution_timeout(Duration::from_secs(120)) // 2 minutes
        .add_tool(
            processor_tool_info,
            processor_tool_config,
            Some(Duration::from_secs(90)),
        )
        .build()
        .await?;

    println!("Docker plugin built successfully!");

    // Initialize the plugin
    let plugin_config = PluginConfig::new("custom-processor".to_string(), PluginType::Docker);
    plugin.initialize(plugin_config)?;

    // Note: In a real scenario, you would build the Docker image first:
    // let dockerfile_path = PathBuf::from("examples/docker_tools/Dockerfile");
    // plugin.build_image(&dockerfile_path, "workflow-toolkit/simple-processor:latest").await?;

    println!("Docker plugin initialized successfully!");

    // Get plugin information
    let info = plugin.info();
    println!("Plugin: {} v{}", info.name, info.version);
    println!("Description: {}", info.description);
    println!("Tools: {:?}", info.tools);

    // Get available tools
    let tools = plugin.get_tools();
    println!("Available tools: {}", tools.len());

    for tool in &tools {
        println!(
            "  - {} v{}: {}",
            tool.name(),
            tool.version(),
            tool.get_info().description
        );
    }

    // Demonstrate various text processing operations
    if let Some(processor_tool) = tools.iter().find(|t| t.name() == "text_processor") {
        println!("\nTesting text processor tool (parameter validation)...");

        let test_cases = vec![
            ("echo", "Hello, Docker!", None),
            ("uppercase", "hello world", None),
            ("lowercase", "HELLO WORLD", None),
            ("reverse", "Hello", None),
            ("length", "Hello, World!", None),
            ("base64_encode", "Hello, World!", None),
            ("word_count", "The quick brown fox jumps", None),
            ("line_count", "Line 1\nLine 2\nLine 3", None),
            ("json_pretty", r#"{"name":"test","value":123}"#, None),
            ("echo", "Delayed processing", Some(2)),
        ];

        for (operation, input, delay) in test_cases {
            let mut params = json!({
                "operation": operation,
                "input": input
            });

            if let Some(delay_val) = delay {
                params["delay"] = json!(delay_val);
            }

            match processor_tool.validate_parameters(&params) {
                Ok(()) => println!("✓ Valid parameters for {} operation: {}", operation, input),
                Err(e) => println!("✗ Invalid parameters for {} operation: {}", operation, e),
            }

            // In a real scenario with Docker running, you would execute:
            // let context = ExecutionContext::new().with_user_id("example-user");
            // let result = processor_tool.execute(params, context).await?;
            // println!("Result: {}", serde_json::to_string_pretty(&result)?);
        }
    }

    // Test invalid parameters
    println!("\nTesting invalid parameters...");
    if let Some(processor_tool) = tools.iter().find(|t| t.name() == "text_processor") {
        let invalid_cases = vec![
            json!({"operation": "invalid_op", "input": "test"}),
            json!({"input": "test"}),     // missing operation
            json!({"operation": "echo"}), // missing input
            json!({"operation": "echo", "input": "test", "delay": -1}), // invalid delay
            json!({"operation": "echo", "input": "test", "delay": 100}), // delay too large
        ];

        for params in invalid_cases {
            match processor_tool.validate_parameters(&params) {
                Ok(()) => println!("✗ Unexpectedly valid parameters: {}", params),
                Err(e) => println!("✓ Correctly rejected invalid parameters: {}", e),
            }
        }
    }

    // Demonstrate Docker environment capabilities
    let environment = plugin.environment();
    let env_guard = environment.lock().await;

    if env_guard.is_initialized() {
        println!("\n✓ Docker environment is initialized and ready");
    } else {
        println!("\n✗ Docker environment is not initialized");
    }

    drop(env_guard);

    // Shutdown the plugin
    plugin.shutdown()?;
    println!("\nDocker plugin shut down successfully!");

    println!("\nDocker Plugin with Dockerfile Example completed!");
    println!("\nTo actually run this example:");
    println!("1. Ensure Docker is installed and running");
    println!("2. Build the Docker image:");
    println!("   cd examples/docker_tools");
    println!("   docker build -t workflow-toolkit/simple-processor:latest .");
    println!("3. Create input/output directories:");
    println!("   mkdir -p /tmp/workflow-input /tmp/workflow-output");
    println!("4. Run the example again");

    Ok(())
}
