//! Example demonstrating Docker plugin usage

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use workflow_toolkit::core::{ExecutionContext, PluginInfo, PluginType, ToolInfo};
use workflow_toolkit::plugins::{
    DockerPluginBuilder, DockerToolConfig, DockerMount, DockerMountType,
    DockerResourceLimits, DockerNetworkConfig, PluginConfig, Plugin,
};
use workflow_toolkit::error::Result;
use chrono::Utc;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Docker Plugin Example");
    println!("====================");

    // Create a Docker plugin for image processing
    let plugin_info = PluginInfo {
        name: "image-processor".to_string(),
        version: "1.0.0".to_string(),
        description: "Docker-based image processing tools".to_string(),
        plugin_type: PluginType::Docker,
        author: Some("Workflow Toolkit Team".to_string()),
        license: Some("MIT".to_string()),
        repository: None,
        dependencies: vec![],
        tools: vec!["resize_image".to_string(), "convert_format".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create Docker tool configurations
    let resize_tool_config = DockerToolConfig {
        image: "alpine/imagemagick:latest".to_string(),
        command: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            r#"
            # Read input parameters from environment
            INPUT_FILE="${WORKFLOW_PARAMS}" || echo '{"input": "/tmp/input.jpg", "width": 800, "height": 600}'
            
            # Parse JSON parameters (simplified for example)
            echo "Processing image with parameters: $INPUT_FILE"
            
            # Simulate image processing
            echo '{"success": true, "result": {"output_file": "/tmp/output.jpg", "width": 800, "height": 600}}' > /tmp/result.json
            cat /tmp/result.json
            "#.to_string(),
        ]),
        entrypoint: None,
        working_dir: Some("/tmp".to_string()),
        environment: {
            let mut env = HashMap::new();
            env.insert("MAGICK_MEMORY_LIMIT".to_string(), "256MB".to_string());
            env
        },
        mounts: vec![
            DockerMount {
                source: PathBuf::from("/tmp/workflow-data"),
                target: "/tmp/data".to_string(),
                mount_type: DockerMountType::Bind,
                read_only: false,
            }
        ],
        ports: HashMap::new(),
        resource_limits: Some(DockerResourceLimits {
            memory: Some(512 * 1024 * 1024), // 512MB
            memory_swap: Some(1024 * 1024 * 1024), // 1GB
            nano_cpus: Some(500_000_000), // 0.5 CPU
            cpu_shares: None,
            pids_limit: Some(100),
        }),
        network_config: Some(DockerNetworkConfig {
            network_mode: "none".to_string(), // No network access for security
            networks: vec![],
            dns: vec![],
            dns_search: vec![],
        }),
        labels: {
            let mut labels = HashMap::new();
            labels.insert("tool.type".to_string(), "image-processor".to_string());
            labels.insert("tool.function".to_string(), "resize".to_string());
            labels
        },
        user: Some("1000:1000".to_string()), // Run as non-root user
        privileged: false,
    };

    let convert_tool_config = DockerToolConfig {
        image: "alpine/imagemagick:latest".to_string(),
        command: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            r#"
            # Read input parameters from environment
            INPUT_FILE="${WORKFLOW_PARAMS}" || echo '{"input": "/tmp/input.jpg", "format": "png"}'
            
            # Parse JSON parameters (simplified for example)
            echo "Converting image format with parameters: $INPUT_FILE"
            
            # Simulate format conversion
            echo '{"success": true, "result": {"output_file": "/tmp/output.png", "format": "png"}}' > /tmp/result.json
            cat /tmp/result.json
            "#.to_string(),
        ]),
        entrypoint: None,
        working_dir: Some("/tmp".to_string()),
        environment: HashMap::new(),
        mounts: vec![
            DockerMount {
                source: PathBuf::from("/tmp/workflow-data"),
                target: "/tmp/data".to_string(),
                mount_type: DockerMountType::Bind,
                read_only: false,
            }
        ],
        ports: HashMap::new(),
        resource_limits: None, // Use default limits
        network_config: None, // Use default network config
        labels: {
            let mut labels = HashMap::new();
            labels.insert("tool.type".to_string(), "image-processor".to_string());
            labels.insert("tool.function".to_string(), "convert".to_string());
            labels
        },
        user: Some("1000:1000".to_string()),
        privileged: false,
    };

    // Create tool info
    let resize_tool_info = ToolInfo {
        name: "resize_image".to_string(),
        version: "1.0.0".to_string(),
        description: "Resize images using ImageMagick in Docker".to_string(),
        category: Some("image-processing".to_string()),
        tags: vec!["image".to_string(), "resize".to_string(), "docker".to_string()],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "input": {"type": "string", "description": "Input image path"},
                "width": {"type": "integer", "description": "Target width"},
                "height": {"type": "integer", "description": "Target height"}
            },
            "required": ["input", "width", "height"]
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "result": {
                    "type": "object",
                    "properties": {
                        "output_file": {"type": "string"},
                        "width": {"type": "integer"},
                        "height": {"type": "integer"}
                    }
                }
            }
        }),
        plugin_name: Some("image-processor".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let convert_tool_info = ToolInfo {
        name: "convert_format".to_string(),
        version: "1.0.0".to_string(),
        description: "Convert image formats using ImageMagick in Docker".to_string(),
        category: Some("image-processing".to_string()),
        tags: vec!["image".to_string(), "convert".to_string(), "docker".to_string()],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "input": {"type": "string", "description": "Input image path"},
                "format": {"type": "string", "description": "Target format (jpg, png, gif, etc.)"}
            },
            "required": ["input", "format"]
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "result": {
                    "type": "object",
                    "properties": {
                        "output_file": {"type": "string"},
                        "format": {"type": "string"}
                    }
                }
            }
        }),
        plugin_name: Some("image-processor".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Build the Docker plugin
    println!("Building Docker plugin...");
    let mut plugin = DockerPluginBuilder::new()
        .name("image-processor")
        .version("1.0.0")
        .description("Docker-based image processing tools")
        .default_image("alpine/imagemagick:latest")
        .default_working_dir("/tmp")
        .add_default_environment_variable("MAGICK_THREAD_LIMIT", "2")
        .add_default_mount(
            PathBuf::from("/tmp/workflow-data"),
            "/tmp/data".to_string(),
            DockerMountType::Bind,
            false,
        )
        .resource_limits(DockerResourceLimits {
            memory: Some(1024 * 1024 * 1024), // 1GB default
            memory_swap: Some(2 * 1024 * 1024 * 1024), // 2GB
            nano_cpus: Some(1_000_000_000), // 1 CPU
            cpu_shares: None,
            pids_limit: Some(512),
        })
        .auto_remove(true)
        .execution_timeout(Duration::from_secs(300)) // 5 minutes
        .add_tool(resize_tool_info, resize_tool_config, Some(Duration::from_secs(120)))
        .add_tool(convert_tool_info, convert_tool_config, Some(Duration::from_secs(60)))
        .build()
        .await?;

    println!("Docker plugin built successfully!");

    // Initialize the plugin
    let plugin_config = PluginConfig::new("image-processor".to_string(), PluginType::Docker);
    plugin.initialize(plugin_config)?;

    // Initialize async components
    plugin.initialize_async().await?;

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
        println!("  - {} v{}: {}", tool.name(), tool.version(), tool.get_info().description);
    }

    // Example: Execute the resize tool (this would require Docker to be running)
    if let Some(resize_tool) = tools.iter().find(|t| t.name() == "resize_image") {
        println!("\nTesting resize tool (simulation)...");
        
        let context = ExecutionContext::new()
            .with_user_id("example-user");
        
        let params = json!({
            "input": "/tmp/data/input.jpg",
            "width": 800,
            "height": 600
        });

        // Note: This would actually try to run Docker, so we'll just validate parameters
        match resize_tool.validate_parameters(&params) {
            Ok(()) => println!("✓ Parameters are valid for resize tool"),
            Err(e) => println!("✗ Parameter validation failed: {}", e),
        }

        // In a real scenario, you would call:
        // let result = resize_tool.execute(params, context).await?;
        // println!("Resize result: {}", serde_json::to_string_pretty(&result)?);
    }

    // Example: Execute the convert tool (simulation)
    if let Some(convert_tool) = tools.iter().find(|t| t.name() == "convert_format") {
        println!("\nTesting convert tool (simulation)...");
        
        let context = ExecutionContext::new()
            .with_user_id("example-user");
        
        let params = json!({
            "input": "/tmp/data/input.jpg",
            "format": "png"
        });

        match convert_tool.validate_parameters(&params) {
            Ok(()) => println!("✓ Parameters are valid for convert tool"),
            Err(e) => println!("✗ Parameter validation failed: {}", e),
        }
    }

    // Demonstrate Docker environment capabilities
    let environment = plugin.environment();
    let env_guard = environment.lock().await;
    
    if env_guard.is_initialized() {
        println!("\n✓ Docker environment is initialized and ready");
        
        // Get environment info (this would require Docker to be running)
        // let env_info = env_guard.get_environment_info().await?;
        // println!("Docker environment info: {}", serde_json::to_string_pretty(&env_info)?);
    } else {
        println!("\n✗ Docker environment is not initialized");
    }

    drop(env_guard);

    // Shutdown the plugin
    plugin.shutdown()?;
    println!("\nDocker plugin shut down successfully!");

    println!("\nDocker Plugin Example completed!");
    println!("Note: This example demonstrates the Docker plugin API.");
    println!("To actually execute containers, ensure Docker is installed and running.");

    Ok(())
}