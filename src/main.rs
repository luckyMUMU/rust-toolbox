use workflow_toolkit::{init_logging, Result};
use workflow_toolkit::config::{Config, ConfigManager};
use workflow_toolkit::interfaces::cli::{CliApp, Cli};
use workflow_toolkit::storage::{StateManager, SimpleMemoryCache, FileStorage};
use workflow_toolkit::tools::{BasicToolRegistry, ToolRegistry};
use workflow_toolkit::workflow::engine::DefaultWorkflowEngine;
use std::sync::Arc;

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
        workflow_toolkit::WorkflowError::workflow_execution(&format!("Failed to create temp dir: {}", e))
    })?;
    
    // Create storage components
    let storage = Arc::new(FileStorage::new(&temp_dir.join("storage")).map_err(|e| {
        workflow_toolkit::WorkflowError::workflow_execution(&format!("Failed to create storage: {}", e))
    })?);
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));
    
    // Create tool registry with some basic tools
    let mut tool_registry = BasicToolRegistry::new();
    
    // Add a simple echo tool for testing
    use workflow_toolkit::tools::{BasicTool, AsyncFunctionExecutor};
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
        .map_err(|e| workflow_toolkit::WorkflowError::workflow_execution(&format!("Failed to create echo tool: {}", e)))?;
    
    tool_registry.register_tool(Arc::new(echo_tool))
        .map_err(|e| workflow_toolkit::WorkflowError::workflow_execution(&format!("Failed to register echo tool: {}", e)))?;
    
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
    );
    
    // Start configuration hot reload monitoring for the app
    app.start_config_hot_reload().await?;
    
    let args: Vec<String> = std::env::args().collect();
    app.run(args).await
}