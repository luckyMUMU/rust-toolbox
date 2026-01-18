//! Main CLI application implementation

use crate::config::{CliConfigOverrides, Config, ConfigManager};
use crate::core::ExecutionContext;
use crate::interfaces::cli::output::{create_formatter, OutputFormatter};
use crate::interfaces::cli::{
    BatchAction, Cli, CliError, Commands, PluginAction, ToolAction, WorkflowAction,
};
use crate::interfaces::mcp::{McpServer, McpServerConfig, McpServerInterface};
use crate::interfaces::tui::EnhancedTuiInterface;
use crate::plugins::manager::PluginManager;
use crate::plugins::types::PluginConfig;
use crate::storage::StateManager;
use crate::tools::ToolRegistry;
use crate::workflow::{
    RefactoredWorkflowEngine, WorkflowConverter, DataContext, ExecutionTracker,
};
use crate::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Batch configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub workflows: Vec<WorkflowSpec>,
}

/// Individual workflow specification in batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {
    pub name: String,
    pub file: String,
    pub parameters: serde_json::Value,
}

/// Result of batch workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub workflow_name: String,
    pub status: crate::core::ExecutionStatus,
    pub duration: std::time::Duration,
    pub error: Option<String>,
}

/// Summary of batch execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub total_workflows: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_duration: std::time::Duration,
    pub average_duration: std::time::Duration,
}

impl BatchSummary {
    pub fn from_results(results: &[BatchResult]) -> Self {
        let total_workflows = results.len();
        let successful = results
            .iter()
            .filter(|r| r.status == crate::core::ExecutionStatus::Completed)
            .count();
        let failed = total_workflows - successful;
        let total_duration = results.iter().map(|r| r.duration).sum();
        let average_duration = if total_workflows > 0 {
            total_duration / total_workflows as u32
        } else {
            std::time::Duration::from_secs(0)
        };

        Self {
            total_workflows,
            successful,
            failed,
            total_duration,
            average_duration,
        }
    }
}

/// Main CLI application.
///
/// Orchestrates the execution of commands, initialization of components (workflow engine,
/// tool registry, etc.), and handling of application state.
pub struct CliApp {
    config_manager: Option<Arc<ConfigManager>>,
    workflow_engine: Option<Arc<RefactoredWorkflowEngine>>,
    tool_registry: Option<Arc<dyn ToolRegistry>>,
    state_manager: Option<Arc<StateManager>>,
    mcp_server: Option<Arc<dyn McpServerInterface>>,
    plugin_manager: Option<Arc<PluginManager>>,
}

impl CliApp {
    /// Create a new CLI application with configuration manager
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self {
            config_manager: Some(config_manager),
            workflow_engine: None,
            tool_registry: None,
            state_manager: None,
            mcp_server: None,
            plugin_manager: None,
        }
    }

    /// Create CLI application from config (legacy support)
    pub fn from_config(config: Config) -> Self {
        let config_manager = Arc::new(ConfigManager::new(config));
        Self::new(config_manager)
    }

    /// Create CLI application with all components
    pub async fn with_components(
        config_manager: Arc<ConfigManager>,
        workflow_engine: Arc<RefactoredWorkflowEngine>,
        tool_registry: Arc<dyn ToolRegistry>,
        state_manager: Arc<StateManager>,
    ) -> Self {
        // Create plugin manager
        let plugin_manager = Arc::new(PluginManager::new());

        // Create MCP server and register tools
        let mut mcp_server = McpServer::new();
        if let Err(e) = mcp_server.register_tools(tool_registry.clone()).await {
            warn!("Failed to register tools with MCP server: {}", e);
        }

        Self {
            config_manager: Some(config_manager),
            workflow_engine: Some(workflow_engine),
            tool_registry: Some(tool_registry),
            state_manager: Some(state_manager),
            mcp_server: Some(Arc::new(mcp_server)),
            plugin_manager: Some(plugin_manager),
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> Config {
        self.config_manager
            .as_ref()
            .map(|cm| cm.get_config())
            .unwrap_or_default()
    }

    /// Start configuration hot reload monitoring
    pub async fn start_config_hot_reload(&self) -> Result<()> {
        if let Some(config_manager) = &self.config_manager {
            config_manager.start_hot_reload().await?;
            info!("Configuration hot reload monitoring started");
        }
        Ok(())
    }

    /// Run the CLI application with given arguments
    pub async fn run(&self, args: Vec<String>) -> Result<()> {
        let cli = if args.is_empty() {
            Cli::parse()
        } else {
            Cli::parse_from(args)
        };

        // Validate arguments
        cli.validate().map_err(|e| crate::WorkflowError::from(e))?;

        // Set up logging based on CLI options
        self.setup_logging(&cli)?;

        // Load configuration if specified
        self.load_config(&cli).await?;

        // Create output formatter
        let formatter = create_formatter(&cli.output);

        // Handle the command
        match &cli.command {
            Commands::Workflow { action } => {
                self.handle_workflow_command(action, &formatter, &cli).await
            }
            Commands::Tool { action } => self.handle_tool_command(action, &formatter, &cli).await,
            Commands::Plugin { action } => {
                self.handle_plugin_command(action, &formatter, &cli).await
            }
            Commands::Batch { action } => self.handle_batch_command(action, &formatter, &cli).await,
            Commands::Tui => self.handle_tui_command(&cli).await,
            Commands::Server {
                http_port,
                ws_port,
                auth,
            } => {
                self.handle_server_command(*http_port, *ws_port, *auth, &cli)
                    .await
            }
            Commands::Completion { shell } => self.handle_completion_command(shell, &cli).await,
        }
    }

    /// Setup logging based on CLI options
    fn setup_logging(&self, cli: &Cli) -> Result<()> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        // Check if logging is already initialized
        if tracing::dispatcher::has_been_set() {
            return Ok(());
        }

        let log_level = if cli.verbose {
            "debug"
        } else if cli.quiet {
            "error"
        } else {
            &cli.log_level
        };

        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| format!("workflow_toolkit={}", log_level).into());

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();

        debug!("Logging initialized with level: {}", log_level);
        Ok(())
    }

    /// Load configuration if specified
    async fn load_config(&self, cli: &Cli) -> Result<()> {
        if let Some(config_manager) = &self.config_manager {
            // Apply command line overrides
            let mut cli_overrides = CliConfigOverrides::from_cli(cli);

            // If server command, add server-specific overrides
            if let Commands::Server {
                http_port, ws_port, ..
            } = &cli.command
            {
                cli_overrides = cli_overrides.with_server_options(*http_port, *ws_port);
            }

            config_manager.apply_command_line_overrides(&cli_overrides)?;

            // If a specific config file was provided, reload from that file
            if let Some(config_path) = &cli.config {
                debug!("Loading configuration from: {:?}", config_path);
                // Create a new config manager with the specified file
                let new_config_manager = Config::load_from_path_with_priority(config_path)?;
                // Apply the same CLI overrides
                new_config_manager.apply_command_line_overrides(&cli_overrides)?;
                // Note: In a real implementation, we'd need to update the config_manager reference
                // For now, we'll just log that a different config was requested
                info!("Configuration loaded from: {:?}", config_path);
            }
        }
        Ok(())
    }

    /// Handle workflow commands
    async fn handle_workflow_command(
        &self,
        action: &WorkflowAction,
        formatter: &Box<dyn OutputFormatter>,
        _cli: &Cli,
    ) -> Result<()> {
        let engine = self.workflow_engine.as_ref().ok_or_else(|| {
            crate::WorkflowError::workflow_execution("Workflow engine not initialized")
        })?;

        match action {
            WorkflowAction::Create {
                definition_file,
                validate_only,
                force,
            } => {
                info!("Creating workflow from: {:?}", definition_file);

                // Load workflow definition
                let definition = self.load_workflow_definition(definition_file).await?;

                // Validate the definition
                definition.validate()?;

                if *validate_only {
                    println!(
                        "{}",
                        formatter.format_success("Workflow definition is valid")
                    );
                    return Ok(());
                }

                println!(
                    "{}",
                    formatter.format_success(&format!(
                        "Workflow '{}' created successfully{}",
                        definition.name,
                        if *force { " (forced)" } else { "" }
                    ))
                );
            }

            WorkflowAction::Execute {
                workflow_name,
                params,
                params_json,
                background,
                wait: _,
                timeout: _,
            } => {
                info!("Executing workflow: {}", workflow_name);

                // Load workflow definition
                let definition = if std::path::Path::new(workflow_name).exists() {
                    self.load_workflow_definition(&PathBuf::from(workflow_name))
                        .await?
                } else {
                    return Err(crate::WorkflowError::NotFound {
                        resource: format!("workflow '{}'", workflow_name),
                    }
                    .into());
                };

                // Convert to FlowNode using WorkflowConverter
                let flow = WorkflowConverter::convert(&definition)?;

                // Prepare data context
                let mut context = DataContext::new();
                let workflow_params = self
                    .parse_workflow_params(params.clone(), params_json.clone())
                    .await?;
                context.set_input_params(workflow_params)?;

                // Create execution tracker
                let workflow_id = Uuid::new_v4();
                let tracker = Arc::new(ExecutionTracker::new(workflow_id, &definition.name));
                tracker.mark_running().await;

                println!(
                    "{}",
                    formatter.format_success(&format!("Workflow started with ID: {}", workflow_id))
                );

                if *background {
                    let engine = engine.clone();
                    let flow = flow.clone();
                    let mut context = context.clone();
                    let tracker = tracker.clone();

                    tokio::spawn(async move {
                        if let Err(e) = engine.execute_flow(&flow, &mut context, tracker.clone()).await {
                            tracing::error!("Background execution failed: {}", e);
                            let _ = tracker.mark_node_failed("root", &e.to_string());
                        } else {
                            tracker.mark_completed().await;
                        }
                    });
                } else {
                    match engine.execute_flow(&flow, &mut context, tracker.clone()).await {
                        Ok(_) => {
                            tracker.mark_completed().await;
                            println!("{}", formatter.format_success("Workflow completed successfully"));
                        }
                        Err(e) => {
                            let _ = tracker.mark_node_failed("root", &e.to_string());
                            println!("{}", formatter.format_error(&format!("Workflow failed: {}", e)));
                        }
                    }
                }
            }

            WorkflowAction::Status { .. } | 
            WorkflowAction::Pause { .. } | 
            WorkflowAction::Resume { .. } | 
            WorkflowAction::Stop { .. } | 
            WorkflowAction::List { .. } => {
                warn!("This command is not yet supported in the v2 engine");
                println!("Command not supported in v2 engine yet.");
            }
        }

        Ok(())
    }

    /// Handle tool commands
    async fn handle_tool_command(
        &self,
        action: &ToolAction,
        formatter: &Box<dyn OutputFormatter>,
        _cli: &Cli,
    ) -> Result<()> {
        let registry = self.tool_registry.as_ref().ok_or_else(|| {
            crate::WorkflowError::workflow_execution("Tool registry not initialized")
        })?;

        match action {
            ToolAction::List {
                category,
                tag,
                search,
                detailed,
            } => {
                info!(
                    "Listing tools (category: {:?}, tag: {:?}, search: {:?})",
                    category, tag, search
                );

                let mut tools = registry.list_tools();

                // Also include tools from plugins
                if let Some(plugin_manager) = &self.plugin_manager {
                    if let Ok(plugin_tools) = plugin_manager.get_all_tools() {
                        for tool in plugin_tools {
                            // Convert plugin tool to ToolInfo
                            let tool_info = tool.get_info();
                            tools.push(tool_info);
                        }
                    }
                }

                // Apply filters
                if let Some(cat) = category {
                    tools.retain(|tool| tool.category.as_ref().map_or(false, |c| c == cat));
                }

                if let Some(tag_filter) = tag {
                    tools.retain(|tool| tool.tags.contains(tag_filter));
                }

                if let Some(search_term) = search {
                    let search_lower = search_term.to_lowercase();
                    tools.retain(|tool| {
                        tool.name.to_lowercase().contains(&search_lower)
                            || tool.description.to_lowercase().contains(&search_lower)
                    });
                }

                if *detailed {
                    for tool in &tools {
                        println!("{}", formatter.format_tool_info(tool));
                        println!(); // Add spacing between tools
                    }
                } else {
                    println!("{}", formatter.format_tool_list(&tools));
                }
            }

            ToolAction::Execute {
                tool_name,
                params,
                params_file,
                timeout,
                dry_run,
            } => {
                info!("Executing tool: {} (dry_run: {})", tool_name, dry_run);

                // Parse parameters
                let tool_params = self
                    .parse_tool_params(params.clone(), params_file.clone())
                    .await?;

                if *dry_run {
                    // Validate parameters only
                    registry.validate_tool_params(tool_name, &tool_params)?;
                    println!("{}", formatter.format_success("Tool parameters are valid"));
                    return Ok(());
                }

                // Create execution context
                let context = ExecutionContext::new();

                // Try to execute from registry first, then from plugins
                let result = match registry
                    .execute_tool(tool_name, tool_params.clone(), context.clone())
                    .await
                {
                    Ok(result) => result,
                    Err(_) => {
                        // Try to find and execute from plugins
                        if let Some(plugin_manager) = &self.plugin_manager {
                            let plugin_tools = plugin_manager.get_all_tools()?;
                            let tool = plugin_tools
                                .iter()
                                .find(|t| t.name() == tool_name)
                                .ok_or_else(|| crate::WorkflowError::NotFound {
                                    resource: format!("tool '{}'", tool_name),
                                })?;

                            // Execute tool with timeout if specified
                            if let Some(timeout_secs) = timeout {
                                debug!("Tool execution timeout set to {} seconds", timeout_secs);
                                // TODO: Implement timeout wrapper
                                tool.execute(tool_params, context).await?
                            } else {
                                tool.execute(tool_params, context).await?
                            }
                        } else {
                            return Err(crate::WorkflowError::NotFound {
                                resource: format!("tool '{}'", tool_name),
                            }
                            .into());
                        }
                    }
                };

                // Format and display result
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                );
            }

            ToolAction::Info { tool_name } => {
                info!("Getting info for tool: {}", tool_name);

                let mut tools = registry.list_tools();

                // Also check plugin tools
                if let Some(plugin_manager) = &self.plugin_manager {
                    if let Ok(plugin_tools) = plugin_manager.get_all_tools() {
                        for tool in plugin_tools {
                            if tool.name() == *tool_name {
                                let tool_info = tool.get_info();
                                tools.push(tool_info);
                                break;
                            }
                        }
                    }
                }

                let tool = tools
                    .iter()
                    .find(|t| t.name == *tool_name)
                    .ok_or_else(|| CliError::ToolNotFound(tool_name.clone()))?;

                println!("{}", formatter.format_tool_info(tool));
            }
        }

        Ok(())
    }

    /// Handle plugin commands
    async fn handle_plugin_command(
        &self,
        action: &PluginAction,
        formatter: &Box<dyn OutputFormatter>,
        _cli: &Cli,
    ) -> Result<()> {
        let plugin_manager = self.plugin_manager.as_ref().ok_or_else(|| {
            crate::WorkflowError::workflow_execution("Plugin manager not initialized")
        })?;

        match action {
            PluginAction::Install {
                plugin_path,
                plugin_type,
                force,
            } => {
                info!(
                    "Installing plugin from: {} (type: {:?}, force: {})",
                    plugin_path, plugin_type, force
                );

                // Determine plugin type if not specified
                let detected_type = if let Some(ptype) = plugin_type {
                    self.parse_plugin_type(ptype)?
                } else {
                    self.detect_plugin_type(plugin_path)?
                };

                // Create plugin configuration
                let plugin_name = self.extract_plugin_name(plugin_path);
                let config = PluginConfig::new(plugin_name.clone(), detected_type.clone());

                // Check if plugin already exists
                if !force {
                    if let Ok(plugins) = plugin_manager.list_plugins() {
                        if plugins.iter().any(|p| p.name == plugin_name) {
                            return Err(crate::WorkflowError::ValidationError(format!(
                                "Plugin '{}' already exists. Use --force to reinstall.",
                                plugin_name
                            ))
                            .into());
                        }
                    }
                }

                // Create and load the plugin based on type
                match detected_type {
                    crate::core::PluginType::Native => {
                        let plugin_info = crate::core::PluginInfo {
                            name: plugin_name.clone(),
                            version: "1.0.0".to_string(),
                            plugin_type: crate::core::PluginType::Native,
                            description: Some(format!("Native plugin from {}", plugin_path)),
                            author: None,
                            homepage: None,
                            metadata: std::collections::HashMap::new(),
                        };

                        let native_plugin = crate::plugins::types::NativePlugin::new(
                            plugin_info,
                            PathBuf::from(plugin_path),
                        );

                        plugin_manager.load_plugin(Box::new(native_plugin), config)?;
                    }
                    crate::core::PluginType::Python => {
                        let plugin_info = crate::core::PluginInfo {
                            name: plugin_name.clone(),
                            version: "1.0.0".to_string(),
                            plugin_type: crate::core::PluginType::Python,
                            description: Some(format!("Python plugin from {}", plugin_path)),
                            author: None,
                            homepage: None,
                            metadata: std::collections::HashMap::new(),
                        };

                        let runtime_config = crate::plugins::python::PythonRuntimeConfig {
                            python_executable: "python3".to_string(),
                            virtual_env_path: None,
                            requirements_file: Some(
                                PathBuf::from(plugin_path).join("requirements.txt"),
                            ),
                            python_paths: vec![],
                            environment_variables: std::collections::HashMap::new(),
                            working_directory: Some(PathBuf::from(plugin_path)),
                        };

                        let python_plugin =
                            crate::plugins::types::PythonPlugin::new(plugin_info, runtime_config);

                        plugin_manager.load_plugin(Box::new(python_plugin), config)?;
                    }
                    crate::core::PluginType::NodeJs => {
                        let plugin_info = crate::core::PluginInfo {
                            name: plugin_name.clone(),
                            version: "1.0.0".to_string(),
                            plugin_type: crate::core::PluginType::NodeJs,
                            description: Some(format!("Node.js plugin from {}", plugin_path)),
                            author: None,
                            homepage: None,
                            metadata: std::collections::HashMap::new(),
                        };

                        let runtime_config = crate::plugins::nodejs::NodeJsRuntimeConfig {
                            node_executable: "node".to_string(),
                            npm_executable: "npm".to_string(),
                            project_directory: Some(PathBuf::from(plugin_path)),
                            node_modules_path: None,
                            module_paths: vec![],
                            package_json: Some(PathBuf::from(plugin_path).join("package.json")),
                            environment_variables: std::collections::HashMap::new(),
                            working_directory: Some(PathBuf::from(plugin_path)),
                            auto_install_dependencies: true,
                        };

                        let nodejs_plugin =
                            crate::plugins::types::NodeJsPlugin::new(plugin_info, runtime_config);

                        plugin_manager.load_plugin(Box::new(nodejs_plugin), config)?;
                    }
                    crate::core::PluginType::Docker => {
                        let plugin_info = crate::core::PluginInfo {
                            name: plugin_name.clone(),
                            version: "1.0.0".to_string(),
                            plugin_type: crate::core::PluginType::Docker,
                            description: Some(format!("Docker plugin from {}", plugin_path)),
                            author: None,
                            homepage: None,
                            metadata: std::collections::HashMap::new(),
                        };

                        let runtime_config = crate::plugins::docker::DockerRuntimeConfig::default();

                        let docker_plugin =
                            crate::plugins::types::DockerPlugin::new(plugin_info, runtime_config)?;

                        plugin_manager.load_plugin(Box::new(docker_plugin), config)?;
                    }
                    crate::core::PluginType::Wasm => {
                        // WASM plugin support temporarily disabled
                        return Err(crate::WorkflowError::ValidationError(
                            "WASM plugin support is temporarily disabled".to_string(),
                        )
                        .into());
                        /*
                        let plugin_info = crate::core::PluginInfo {
                            name: plugin_name.clone(),
                            version: "1.0.0".to_string(),
                            plugin_type: crate::core::PluginType::Wasm,
                            description: Some(format!("WASM plugin from {}", plugin_path)),
                            author: None,
                            metadata: std::collections::HashMap::new(),
                        };

                        let runtime_config = crate::plugins::wasm::WasmRuntimeConfig {
                            runtime_type: crate::plugins::wasm::WasmRuntimeType::Wasmtime,
                            module_path: PathBuf::from(plugin_path),
                            memory_limit: 64 * 1024 * 1024, // 64MB
                            timeout: std::time::Duration::from_secs(30),
                            fuel_limit: Some(1_000_000),
                            allow_wasi: false,
                            allowed_imports: vec![],
                            entry_points: {
                                let mut map = std::collections::HashMap::new();
                                map.insert("main".to_string(), "main".to_string());
                                map
                            },
                            extism_config: None,
                        };

                        let wasm_plugin = crate::plugins::types::WasmPlugin::new(
                            plugin_info,
                            runtime_config
                        );

                        plugin_manager.load_plugin(Box::new(wasm_plugin), config)?;
                        */
                    }
                    crate::core::PluginType::Go => {
                        return Err(crate::WorkflowError::ValidationError(
                            "Go plugin support is not yet implemented".to_string(),
                        )
                        .into());
                    }
                }

                // Register plugin tools with tool registry
                // Note: This requires mutable access to the tool registry, which is not
                // available through Arc<dyn ToolRegistry>. This would need to be redesigned
                // to use interior mutability or a different approach.
                if let Some(_tool_registry) = &self.tool_registry {
                    let plugin_tools = plugin_manager.get_plugin_tools(&plugin_name)?;
                    debug!(
                        "Plugin '{}' provides {} tools",
                        plugin_name,
                        plugin_tools.len()
                    );
                    // TODO: Implement tool registration with proper thread-safe design
                }

                println!(
                    "{}",
                    formatter.format_success(&format!(
                        "Plugin '{}' installed successfully from {} (type: {:?}){}",
                        plugin_name,
                        plugin_path,
                        detected_type,
                        if *force { " (forced)" } else { "" }
                    ))
                );
            }

            PluginAction::List {
                detailed,
                plugin_type,
            } => {
                info!(
                    "Listing plugins (detailed: {}, type: {:?})",
                    detailed, plugin_type
                );

                let plugins = plugin_manager.list_plugins()?;

                // Filter by type if specified
                let filtered_plugins: Vec<_> = if let Some(ptype) = plugin_type {
                    let filter_type = self.parse_plugin_type(ptype)?;
                    plugins
                        .into_iter()
                        .filter(|p| {
                            // Filter by plugin type
                            p.plugin_type == filter_type
                        })
                        .collect()
                } else {
                    plugins
                };

                if filtered_plugins.is_empty() {
                    println!("No plugins installed.");
                } else if *detailed {
                    for plugin in &filtered_plugins {
                        println!("Plugin: {}", plugin.name);
                        println!("  Version: {}", plugin.version);
                        println!("  Type: {:?}", plugin.plugin_type);
                        if let Some(description) = &plugin.description {
                            println!("  Description: {}", description);
                        }
                        if let Some(author) = &plugin.author {
                            println!("  Author: {}", author);
                        }

                        // Get plugin status
                        if let Ok(Some(status)) = plugin_manager.get_plugin_status(&plugin.name) {
                            println!("  Status: {:?}", status);
                        }

                        // List plugin tools
                        if let Ok(tools) = plugin_manager.get_plugin_tools(&plugin.name) {
                            if !tools.is_empty() {
                                println!("  Tools:");
                                for tool in tools {
                                    println!(
                                        "    - {}: {}",
                                        tool.name(),
                                        tool.get_info().description
                                    );
                                }
                            }
                        }

                        println!(); // Add spacing between plugins
                    }
                } else {
                    println!(
                        "{:<20} {:<10} {:<15} {}",
                        "Name", "Version", "Type", "Description"
                    );
                    println!("{}", "-".repeat(80));
                    for plugin in &filtered_plugins {
                        println!(
                            "{:<20} {:<10} {:<15} {}",
                            plugin.name,
                            plugin.version,
                            format!("{:?}", plugin.plugin_type),
                            plugin.description.as_deref().unwrap_or("No description")
                        );
                    }
                }
            }

            PluginAction::Reload { plugin_name } => {
                info!("Reloading plugin: {}", plugin_name);

                plugin_manager.reload_plugin(plugin_name)?;

                // Re-register plugin tools with tool registry
                // Note: This requires mutable access to the tool registry, which is not
                // available through Arc<dyn ToolRegistry>. This would need to be redesigned
                // to use interior mutability or a different approach.
                if let Some(_tool_registry) = &self.tool_registry {
                    let plugin_tools = plugin_manager.get_plugin_tools(plugin_name)?;
                    debug!(
                        "Plugin '{}' provides {} tools after reload",
                        plugin_name,
                        plugin_tools.len()
                    );
                    // TODO: Implement tool registration with proper thread-safe design
                }

                println!(
                    "{}",
                    formatter
                        .format_success(&format!("Plugin '{}' reloaded successfully", plugin_name))
                );
            }

            PluginAction::Uninstall { plugin_name, force } => {
                info!("Uninstalling plugin: {} (force: {})", plugin_name, force);

                // Check if plugin exists
                let plugins = plugin_manager.list_plugins()?;
                if !plugins.iter().any(|p| p.name == *plugin_name) {
                    return Err(crate::WorkflowError::NotFound {
                        resource: format!("plugin '{}'", plugin_name),
                    }
                    .into());
                }

                // Confirm uninstallation if not forced
                if !force {
                    println!(
                        "Are you sure you want to uninstall plugin '{}'? (y/N)",
                        plugin_name
                    );
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if !input.trim().to_lowercase().starts_with('y') {
                        println!("Uninstallation cancelled.");
                        return Ok(());
                    }
                }

                // Remove plugin tools from tool registry
                // Note: This requires mutable access to the tool registry, which is not
                // available through Arc<dyn ToolRegistry>. This would need to be redesigned
                // to use interior mutability or a different approach.
                if let Some(_tool_registry) = &self.tool_registry {
                    if let Ok(plugin_tools) = plugin_manager.get_plugin_tools(plugin_name) {
                        debug!(
                            "Would unregister {} tools from plugin '{}'",
                            plugin_tools.len(),
                            plugin_name
                        );
                        // TODO: Implement tool unregistration with proper thread-safe design
                    }
                }

                // Unload the plugin
                plugin_manager.unload_plugin(plugin_name)?;

                println!(
                    "{}",
                    formatter.format_success(&format!(
                        "Plugin '{}' uninstalled successfully{}",
                        plugin_name,
                        if *force { " (forced)" } else { "" }
                    ))
                );
            }

            PluginAction::Info { plugin_name } => {
                info!("Getting info for plugin: {}", plugin_name);

                let plugins = plugin_manager.list_plugins()?;
                let plugin = plugins
                    .iter()
                    .find(|p| p.name == *plugin_name)
                    .ok_or_else(|| crate::WorkflowError::NotFound {
                        resource: format!("plugin '{}'", plugin_name),
                    })?;

                println!("Plugin Information:");
                println!("  Name: {}", plugin.name);
                println!("  Version: {}", plugin.version);
                println!("  Type: {:?}", plugin.plugin_type);

                if let Some(description) = &plugin.description {
                    println!("  Description: {}", description);
                }

                if let Some(author) = &plugin.author {
                    println!("  Author: {}", author);
                }

                // Get plugin status
                if let Ok(Some(status)) = plugin_manager.get_plugin_status(plugin_name) {
                    println!("  Status: {:?}", status);
                }

                // List plugin tools
                if let Ok(tools) = plugin_manager.get_plugin_tools(plugin_name) {
                    if !tools.is_empty() {
                        println!("  Tools ({}):", tools.len());
                        for tool in tools {
                            println!("    - {}: {}", tool.name(), tool.get_info().description);
                        }
                    } else {
                        println!("  Tools: None");
                    }
                } else {
                    println!("  Tools: Unable to retrieve");
                }
            }
        }

        Ok(())
    }

    /// Handle batch commands
    async fn handle_batch_command(
        &self,
        action: &BatchAction,
        formatter: &Box<dyn OutputFormatter>,
        _cli: &Cli,
    ) -> Result<()> {
        match action {
            BatchAction::Execute {
                workflow_list_file,
                parallel,
                continue_on_failure,
                output_dir,
                timeout,
            } => {
                info!(
                    "Executing batch workflows from: {:?} (parallel: {}, continue_on_failure: {})",
                    workflow_list_file, parallel, continue_on_failure
                );

                // Load batch configuration
                let batch_config = self.load_batch_config(workflow_list_file).await?;

                // Execute workflows in parallel
                let results = self
                    .execute_batch_workflows(
                        batch_config,
                        *parallel,
                        *continue_on_failure,
                        timeout.map(|t| std::time::Duration::from_secs(t)),
                    )
                    .await?;

                // Display results
                println!(
                    "{}",
                    formatter.format_success(&format!(
                        "Batch execution completed: {} workflows processed",
                        results.len()
                    ))
                );

                // Save results if output directory specified
                if let Some(dir) = output_dir {
                    self.save_batch_results(&results, dir).await?;
                    println!(
                        "{}",
                        formatter.format_success(&format!("Results saved to: {:?}", dir))
                    );
                }
            }
        }

        Ok(())
    }

    /// Handle TUI command
    async fn handle_tui_command(&self, _cli: &Cli) -> Result<()> {
        info!("Starting TUI interface");

        // Create and start the TUI interface
        let tui_interface = crate::interfaces::tui::BasicTuiInterface::new();

        println!("Starting TUI interface...");
        println!("Note: This is a stub implementation. Full TUI functionality will be implemented later.");
        println!("Press Ctrl+C to exit.");

        // Start the TUI interface (stub implementation)
        tui_interface.start()?;

        // In a real implementation, this would run the event loop
        // For now, we just simulate a basic interface
        println!("TUI interface started successfully (stub mode)");
        println!("Available views:");
        println!("  F1 - Workflow List");
        println!("  F2 - Execution Monitor");
        println!("  F3 - Tool Manager");
        println!("  F4 - System Status");
        println!("  F5 - Log Viewer");
        println!("  Q  - Quit");

        // Wait for user input to exit (simplified for stub)
        println!("\nPress Enter to exit TUI interface...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input)?;

        tui_interface.stop()?;
        println!("TUI interface stopped");

        Ok(())
    }

    /// Handle server command
    async fn handle_server_command(
        &self,
        http_port: u16,
        ws_port: u16,
        auth: bool,
        _cli: &Cli,
    ) -> Result<()> {
        info!(
            "Starting MCP server (HTTP: {}, WS: {}, auth: {})",
            http_port, ws_port, auth
        );

        if let Some(mcp_server) = &self.mcp_server {
            let config = McpServerConfig {
                http_port,
                ws_port,
                auth: crate::core::AuthConfig {
                    enabled: auth,
                    token: None,
                    jwt_secret: if auth {
                        Some("default_secret_key".to_string())
                    } else {
                        None
                    },
                    token_expiry: std::time::Duration::from_secs(3600),
                    allowed_origins: vec!["*".to_string()],
                },
                rate_limit: crate::core::RateLimitConfig {
                    requests_per_minute: 60,
                    burst_size: 10,
                    enabled: true,
                    max_requests: 1000,
                    window_ms: 60000,
                },
                cors_origins: vec!["*".to_string()], // TODO: Configure properly
            };

            // List available tools
            if let Ok(tools) = mcp_server.list_tools().await {
                info!("MCP server registered {} tools:", tools.len());
                for tool in &tools {
                    info!("  - {}: {}", tool.name, tool.description);
                }
            }

            mcp_server.start(config).await?;

            // Keep the server running
            info!("MCP server started successfully. Press Ctrl+C to stop.");
            tokio::signal::ctrl_c().await.map_err(|e| {
                crate::WorkflowError::workflow_execution(&format!(
                    "Failed to wait for Ctrl+C: {}",
                    e
                ))
            })?;

            info!("Shutting down MCP server...");
            mcp_server.stop().await?;
            info!("MCP server stopped.");
        } else {
            return Err(crate::WorkflowError::workflow_execution(
                "MCP server not initialized",
            ));
        }

        Ok(())
    }

    /// Handle completion command
    async fn handle_completion_command(
        &self,
        shell: &crate::interfaces::cli::commands::Shell,
        _cli: &Cli,
    ) -> Result<()> {
        info!("Generating completion for shell: {:?}", shell);

        use clap::CommandFactory;
        use clap_complete::{generate, Shell as CompletionShell};

        let mut cmd = Cli::command();
        let shell_type = match shell {
            crate::interfaces::cli::commands::Shell::Bash => CompletionShell::Bash,
            crate::interfaces::cli::commands::Shell::Zsh => CompletionShell::Zsh,
            crate::interfaces::cli::commands::Shell::Fish => CompletionShell::Fish,
            crate::interfaces::cli::commands::Shell::PowerShell => CompletionShell::PowerShell,
        };

        generate(
            shell_type,
            &mut cmd,
            "workflow-toolkit",
            &mut std::io::stdout(),
        );
        Ok(())
    }

    /// Load workflow definition from file
    async fn load_workflow_definition(
        &self,
        path: &PathBuf,
    ) -> Result<crate::workflow::WorkflowDefinition> {
        use tokio::fs;

        if !path.exists() {
            return Err(CliError::FileNotFound(path.clone()).into());
        }

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| CliError::IoError(e))?;

        debug!("Loaded file content: {}", content);

        // Try to parse as YAML first, then JSON
        if let Ok(definition) =
            serde_yaml::from_str::<crate::workflow::WorkflowDefinition>(&content)
        {
            debug!("Successfully parsed as YAML");
            Ok(definition)
        } else if let Ok(definition) =
            serde_json::from_str::<crate::workflow::WorkflowDefinition>(&content)
        {
            debug!("Successfully parsed as JSON");
            Ok(definition)
        } else {
            // Try to get more specific error information
            if let Err(yaml_err) =
                serde_yaml::from_str::<crate::workflow::WorkflowDefinition>(&content)
            {
                debug!("YAML parsing error: {}", yaml_err);
            }
            if let Err(json_err) =
                serde_json::from_str::<crate::workflow::WorkflowDefinition>(&content)
            {
                debug!("JSON parsing error: {}", json_err);
            }
            Err(CliError::InvalidFileFormat("File must be valid YAML or JSON".to_string()).into())
        }
    }

    /// Parse workflow parameters from various sources
    async fn parse_workflow_params(
        &self,
        params_file: Option<PathBuf>,
        params_json: Option<String>,
    ) -> Result<serde_json::Value> {
        if let Some(json_str) = params_json {
            serde_json::from_str(&json_str).map_err(|e| CliError::JsonError(e).into())
        } else if let Some(file_path) = params_file {
            use tokio::fs;

            let content = fs::read_to_string(&file_path)
                .await
                .map_err(|e| CliError::IoError(e))?;

            // Try JSON first, then YAML
            if let Ok(value) = serde_json::from_str(&content) {
                Ok(value)
            } else {
                serde_yaml::from_str(&content).map_err(|e| CliError::YamlError(e).into())
            }
        } else {
            Ok(serde_json::Value::Null)
        }
    }

    /// Parse tool parameters from various sources
    async fn parse_tool_params(
        &self,
        params_json: Option<String>,
        params_file: Option<PathBuf>,
    ) -> Result<serde_json::Value> {
        if let Some(json_str) = params_json {
            serde_json::from_str(&json_str).map_err(|e| CliError::JsonError(e).into())
        } else if let Some(file_path) = params_file {
            use tokio::fs;

            let content = fs::read_to_string(&file_path)
                .await
                .map_err(|e| CliError::IoError(e))?;

            // Try JSON first, then YAML
            if let Ok(value) = serde_json::from_str(&content) {
                Ok(value)
            } else {
                serde_yaml::from_str(&content).map_err(|e| CliError::YamlError(e).into())
            }
        } else {
            Ok(serde_json::Value::Null)
        }
    }

    /// Load batch configuration from file
    async fn load_batch_config(&self, path: &PathBuf) -> Result<BatchConfig> {
        use tokio::fs;

        if !path.exists() {
            return Err(CliError::FileNotFound(path.clone()).into());
        }

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| CliError::IoError(e))?;

        // Try to parse as YAML first, then JSON
        if let Ok(config) = serde_yaml::from_str::<BatchConfig>(&content) {
            Ok(config)
        } else if let Ok(config) = serde_json::from_str::<BatchConfig>(&content) {
            Ok(config)
        } else {
            Err(
                CliError::InvalidFileFormat("Batch config must be valid YAML or JSON".to_string())
                    .into(),
            )
        }
    }

    /// Execute workflows in batch
    async fn execute_batch_workflows(
        &self,
        batch_config: BatchConfig,
        parallel_limit: usize,
        continue_on_failure: bool,
        timeout: Option<std::time::Duration>,
    ) -> Result<Vec<BatchResult>> {
        use futures::future::join_all;
        use tokio::sync::Semaphore;

        let engine = self.workflow_engine.as_ref().ok_or_else(|| {
            crate::WorkflowError::workflow_execution("Workflow engine not initialized")
        })?;

        let semaphore = Arc::new(Semaphore::new(parallel_limit));
        let mut tasks = Vec::new();

        for workflow_spec in batch_config.workflows {
            let engine = engine.clone();
            let semaphore = semaphore.clone();
            let timeout = timeout;

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                let start_time = std::time::Instant::now();

                // Load workflow definition
                let definition_result = if std::path::Path::new(&workflow_spec.file).exists() {
                    // Load from file
                    use tokio::fs;
                    let content = fs::read_to_string(&workflow_spec.file).await.map_err(|e| {
                        crate::WorkflowError::workflow_execution(&format!(
                            "Failed to read file: {}",
                            e
                        ))
                    })?;

                    if let Ok(def) =
                        serde_yaml::from_str::<crate::workflow::WorkflowDefinition>(&content)
                    {
                        Ok(def)
                    } else if let Ok(def) =
                        serde_json::from_str::<crate::workflow::WorkflowDefinition>(&content)
                    {
                        Ok(def)
                    } else {
                        Err(crate::WorkflowError::workflow_execution(
                            "Invalid workflow file format",
                        ))
                    }
                } else {
                    Err(crate::WorkflowError::NotFound {
                        resource: format!("workflow file '{}'", workflow_spec.file),
                    })
                };

                let result = match definition_result {
                    Ok(definition) => {
                        // Convert parameters to HashMap
                        let params: std::collections::HashMap<String, serde_json::Value> = match workflow_spec.parameters.as_object() {
                            Some(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                            None => std::collections::HashMap::new(),
                        };

                        // Execute workflow with timeout if specified
                        let execution_result = if let Some(timeout_duration) = timeout {
                            tokio::time::timeout(
                                timeout_duration,
                                engine.execute(definition, params.clone()),
                            )
                            .await
                            .map_err(|_| {
                                crate::WorkflowError::Timeout {
                                    duration: timeout_duration,
                                }
                            })?
                        } else {
                            engine.execute(definition, params).await
                        };

                        match execution_result {
                            Ok(execution) => BatchResult {
                                workflow_name: workflow_spec.name,
                                status: execution.status,
                                duration: start_time.elapsed(),
                                error: None,
                            },
                            Err(e) => BatchResult {
                                workflow_name: workflow_spec.name,
                                status: crate::core::ExecutionStatus::Failed,
                                duration: start_time.elapsed(),
                                error: Some(e.to_string()),
                            },
                        }
                    }
                    Err(e) => BatchResult {
                        workflow_name: workflow_spec.name,
                        status: crate::core::ExecutionStatus::Failed,
                        duration: start_time.elapsed(),
                        error: Some(e.to_string()),
                    },
                };

                Ok::<BatchResult, crate::WorkflowError>(result)
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results = join_all(tasks).await;
        let mut batch_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(batch_result)) => {
                    batch_results.push(batch_result);
                }
                Ok(Err(e)) => {
                    if !continue_on_failure {
                        return Err(e.into());
                    }
                    // Create a failed result
                    batch_results.push(BatchResult {
                        workflow_name: "unknown".to_string(),
                        status: crate::core::ExecutionStatus::Failed,
                        duration: std::time::Duration::from_secs(0),
                        error: Some(e.to_string()),
                    });
                }
                Err(e) => {
                    if !continue_on_failure {
                        return Err(crate::WorkflowError::workflow_execution(&format!(
                            "Task join error: {}",
                            e
                        ))
                        .into());
                    }
                    batch_results.push(BatchResult {
                        workflow_name: "unknown".to_string(),
                        status: crate::core::ExecutionStatus::Failed,
                        duration: std::time::Duration::from_secs(0),
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(batch_results)
    }

    /// Parse plugin type from string
    fn parse_plugin_type(&self, plugin_type: &str) -> Result<crate::core::PluginType> {
        match plugin_type.to_lowercase().as_str() {
            "native" => Ok(crate::core::PluginType::Native),
            "python" => Ok(crate::core::PluginType::Python),
            "nodejs" | "node" => Ok(crate::core::PluginType::NodeJs),
            "go" => Ok(crate::core::PluginType::Go),
            "docker" => Ok(crate::core::PluginType::Docker),
            "wasm" | "webassembly" => Ok(crate::core::PluginType::Wasm),
            _ => Err(crate::WorkflowError::ValidationError(
                format!("Unsupported plugin type: {}. Supported types: native, python, nodejs, go, docker, wasm", plugin_type)
            ).into()),
        }
    }

    /// Detect plugin type from path
    fn detect_plugin_type(&self, plugin_path: &str) -> Result<crate::core::PluginType> {
        let path = std::path::Path::new(plugin_path);

        // Check for specific files that indicate plugin type
        if path.join("requirements.txt").exists()
            || path.join("setup.py").exists()
            || path.join("pyproject.toml").exists()
        {
            Ok(crate::core::PluginType::Python)
        } else if path.join("package.json").exists() {
            Ok(crate::core::PluginType::NodeJs)
        } else if path.join("Dockerfile").exists() {
            Ok(crate::core::PluginType::Docker)
        } else if path
            .extension()
            .map_or(false, |ext| ext == "wasm" || ext == "wat")
        {
            Ok(crate::core::PluginType::Wasm)
        } else if path
            .extension()
            .map_or(false, |ext| ext == "so" || ext == "dll" || ext == "dylib")
        {
            Ok(crate::core::PluginType::Native)
        } else {
            // Default to native if we can't detect
            warn!(
                "Could not detect plugin type for {}, defaulting to native",
                plugin_path
            );
            Ok(crate::core::PluginType::Native)
        }
    }

    /// Extract plugin name from path
    fn extract_plugin_name(&self, plugin_path: &str) -> String {
        let path = std::path::Path::new(plugin_path);

        // Use the directory name or file stem as plugin name
        if path.is_dir() {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string()
        } else {
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string()
        }
    }

    /// Save batch results to output directory
    async fn save_batch_results(
        &self,
        results: &[BatchResult],
        output_dir: &PathBuf,
    ) -> Result<()> {
        use tokio::fs;

        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir)
            .await
            .map_err(|e| CliError::IoError(e))?;

        // Save results as JSON
        let results_json =
            serde_json::to_string_pretty(results).map_err(|e| CliError::JsonError(e))?;

        let results_file = output_dir.join("batch_results.json");
        fs::write(&results_file, results_json)
            .await
            .map_err(|e| CliError::IoError(e))?;

        // Save summary
        let summary = BatchSummary::from_results(results);
        let summary_json =
            serde_json::to_string_pretty(&summary).map_err(|e| CliError::JsonError(e))?;

        let summary_file = output_dir.join("batch_summary.json");
        fs::write(&summary_file, summary_json)
            .await
            .map_err(|e| CliError::IoError(e))?;

        Ok(())
    }
}

impl Default for CliApp {
    fn default() -> Self {
        let config_manager = Arc::new(ConfigManager::new(Config::default()));
        Self::new(config_manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_app_creation() {
        let config = Config::default();
        let config_manager = Arc::new(ConfigManager::new(config));
        let app = CliApp::new(config_manager);

        assert!(app.workflow_engine.is_none());
        assert!(app.tool_registry.is_none());
        assert!(app.state_manager.is_none());
    }

    #[tokio::test]
    async fn test_parse_workflow_params_json() {
        let app = CliApp::default();

        let result = app
            .parse_workflow_params(None, Some(r#"{"key": "value"}"#.to_string()))
            .await;

        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params["key"], "value");
    }

    #[tokio::test]
    async fn test_parse_workflow_params_empty() {
        let app = CliApp::default();

        let result = app.parse_workflow_params(None, None).await;

        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params, serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_parse_tool_params_invalid_json() {
        let app = CliApp::default();

        let result = app
            .parse_tool_params(Some("invalid json".to_string()), None)
            .await;

        assert!(result.is_err());
    }
}
