use std::sync::Arc;
use workflow_toolkit::config::Config;
use workflow_toolkit::core::PluginType;
use workflow_toolkit::interfaces::cli::{Cli, CliApp};
use workflow_toolkit::plugins::{FileManagementPlugin, Plugin, PluginConfig};
use workflow_toolkit::storage::{FileStorage, SimpleMemoryCache, StateManager};
use workflow_toolkit::tools::{BasicToolRegistry, ToolRegistry};
use workflow_toolkit::workflow::engine::DefaultWorkflowEngine;
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
        workflow_toolkit::WorkflowError::workflow_execution(&format!(
            "Failed to create temp dir: {}",
            e
        ))
    })?;

    // Create storage components
    let storage = Arc::new(FileStorage::new(&temp_dir.join("storage")).map_err(|e| {
        workflow_toolkit::WorkflowError::workflow_execution(&format!(
            "Failed to create storage: {}",
            e
        ))
    })?);
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));

    // Create tool registry with some basic tools
    let mut tool_registry = BasicToolRegistry::new();

    // Add a simple echo tool for testing
    use workflow_toolkit::tools::{AsyncFunctionExecutor, BasicTool, DataCacheTool, DataTransformTool};
    let echo_executor = Arc::new(AsyncFunctionExecutor::new(|params, _context| async move {
        if let Some(message) = params.get("message") {
            Ok(serde_json::json!({
                "output": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        } else {
            Ok(serde_json::json!({
                "output": "Hello, World!",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    }));

    let echo_tool = BasicTool::builder()
        .name("echo")
        .version("1.0.0")
        .description("A simple echo tool for testing")
        .executor(echo_executor)
        .build()
        .map_err(|e| {
            workflow_toolkit::WorkflowError::workflow_execution(&format!(
                "Failed to create echo tool: {}",
                e
            ))
        })?;

    tool_registry
        .register_tool(Arc::new(echo_tool))
        .map_err(|e| {
            workflow_toolkit::WorkflowError::workflow_execution(&format!(
                "Failed to register echo tool: {}",
                e
            ))
        })?;

    // Register System Tools (DataCache, DataTransform)
    tool_registry.register_tool(Arc::new(DataCacheTool)).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to register data-cache tool: {}", e);
    });
    tool_registry.register_tool(Arc::new(DataTransformTool)).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to register data-transform tool: {}", e);
    });

    // Register File Management Plugin Tools
    let mut fm_plugin = FileManagementPlugin::new();
    let fm_config = PluginConfig::new("file-management".to_string(), PluginType::Native);
    
    // Initialize the plugin
    if let Err(e) = fm_plugin.initialize(fm_config) {
        eprintln!("Warning: Failed to initialize file management plugin: {}", e);
    } else {
        // Register all plugin tools
        let tools = fm_plugin.get_tools();
        // println!("Registering {} file management tools...", tools.len());
        
        for tool in tools {
            if let Err(e) = tool_registry.register_tool(tool) {
                 eprintln!("Warning: Failed to register file management tool: {}", e);
            }
        }
    }

    let tool_registry = Arc::new(tool_registry);

    // Create workflow engine
    let workflow_engine = Arc::new(DefaultWorkflowEngine::new(
        state_manager.clone(),
        tool_registry.clone(),
        4, // max parallel workflows
    ));

    // Create and run CLI application with all components
    let app = CliApp::with_components(
        config_manager,
        workflow_engine,
        tool_registry,
        state_manager,
    )
    .await;

    // Start configuration hot reload monitoring for the app
    app.start_config_hot_reload().await?;

    let args: Vec<String> = std::env::args().collect();
    app.run(args).await
}
