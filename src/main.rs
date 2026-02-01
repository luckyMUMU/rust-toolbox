use std::sync::Arc;
use workflow_toolkit::config::Config;
use workflow_toolkit::interfaces::cli::{Cli, CliApp};
use workflow_toolkit::storage::{FileStorage, SimpleMemoryCache, StateManager};
use workflow_toolkit::tools::{BasicToolRegistry, ToolRegistry};
use workflow_toolkit::workflow::RefactoredWorkflowEngine;
use workflow_toolkit::{init_logging, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let _cli = Cli::parse_args();

    // Initialize logging early but don't fail if already initialized
    let _ = init_logging();

    // Load configuration with priority handling
    let config_manager = if let Some(config_path) = &_cli.config {
        Arc::new(Config::load_from_path_with_priority(config_path)?)
    } else {
        Arc::new(Config::load_with_priority()?)
    };

    // Start configuration hot reload monitoring
    let config_manager_clone = config_manager.clone();
    tokio::spawn(async move {
        if let Err(e) = config_manager_clone.start_hot_reload().await {
            eprintln!("Failed to start configuration hot reload: {}", e);
        }
    });

    // Initialize components
    let temp_dir = std::env::temp_dir().join("workflow-toolkit");
    std::fs::create_dir_all(&temp_dir).map_err(|e| {
        workflow_toolkit::WorkflowError::workflow_execution(format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    // Create storage components
    let storage = Arc::new(FileStorage::new(temp_dir.join("storage")).map_err(|e| {
        workflow_toolkit::WorkflowError::workflow_execution(format!(
            "Failed to create storage: {}",
            e
        ))
    })?);
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));

    // Create tool registry with some basic tools
    let mut tool_registry = BasicToolRegistry::new();

    // Register File Management Plugin Tools
    use std::collections::HashMap;
    use workflow_toolkit::core::{PluginInfo, PluginType};
    use workflow_toolkit::plugins::file_management::{
        FileManagementConfig, FileManagementToolRegistry,
    };

    let fm_config = FileManagementConfig::default();
    let fm_plugin_info = PluginInfo {
        name: "file-management".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::Native,
        description: Some("File management tools".to_string()),
        author: Some("System".to_string()),
        homepage: None,
        metadata: HashMap::new(),
    };

    let mut fm_registry = FileManagementToolRegistry::new(fm_config, fm_plugin_info);
    match fm_registry.register_all_tools() {
        Ok(tools) => {
            for tool in tools {
                if let Err(e) = tool_registry.register_tool(tool.clone()) {
                    eprintln!("Failed to register tool {}: {}", tool.name(), e);
                }
            }
            println!("Registered file management tools successfully");
        }
        Err(e) => {
            eprintln!("Failed to register file management tools: {}", e);
        }
    }

    // Add a simple echo tool for testing
    use workflow_toolkit::tools::{NativeToolBuilder, ToolInput, ToolOutput, Tool};
    let echo_tool = NativeToolBuilder::new()
        .name("echo")
        .version("1.0.0")
        .description("A simple echo tool for testing")
        .executor(|input: ToolInput, _ctx| async move {
            let params = &input.params;
            let result = if let Some(message) = params.get("message") {
                serde_json::json!({
                    "output": message,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            } else {
                serde_json::json!({
                    "output": "Hello, World!",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            };
            Ok(ToolOutput::success(result))
        })
        .build()
        .map_err(|e| {
            workflow_toolkit::WorkflowError::workflow_execution(format!(
                "Failed to create echo tool: {}",
                e
            ))
        })?;

    tool_registry
        .register("echo", Tool::Native(Arc::new(echo_tool)));

    let tool_registry = Arc::new(tool_registry);

    // Create workflow engine
    let workflow_engine = Arc::new(RefactoredWorkflowEngine::new(
        state_manager.clone(),
        tool_registry.clone(),
        4, // max parallel workflows
    ));

    // Create and run CLI application with all components
    let app = CliApp::with_components(config_manager, workflow_engine, tool_registry).await;

    // Start configuration hot reload monitoring for the app
    app.start_config_hot_reload().await?;

    let args: Vec<String> = std::env::args().collect();
    app.run(args).await
}
