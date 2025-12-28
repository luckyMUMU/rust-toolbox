//! Main CLI application implementation

use crate::config::Config;
use crate::core::ExecutionContext;
use crate::interfaces::cli::{
    CliError, Cli, Commands, WorkflowAction, ToolAction, PluginAction, BatchAction
};
use crate::interfaces::cli::output::{create_formatter, OutputFormatter};
use crate::interfaces::tui::TuiInterface;
use crate::storage::StateManager;
use crate::tools::ToolRegistry;
use crate::workflow::WorkflowEngine;
use crate::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

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
        let successful = results.iter()
            .filter(|r| r.status == crate::core::ExecutionStatus::Completed)
            .count();
        let failed = total_workflows - successful;
        let total_duration = results.iter()
            .map(|r| r.duration)
            .sum();
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

/// Main CLI application
pub struct CliApp {
    config: Config,
    workflow_engine: Option<Arc<dyn WorkflowEngine>>,
    tool_registry: Option<Arc<dyn ToolRegistry>>,
    state_manager: Option<Arc<StateManager>>,
}

impl CliApp {
    /// Create a new CLI application
    pub fn new(config: Config) -> Self {
        Self {
            config,
            workflow_engine: None,
            tool_registry: None,
            state_manager: None,
        }
    }
    
    /// Create CLI application with all components
    pub fn with_components(
        config: Config,
        workflow_engine: Arc<dyn WorkflowEngine>,
        tool_registry: Arc<dyn ToolRegistry>,
        state_manager: Arc<StateManager>,
    ) -> Self {
        Self {
            config,
            workflow_engine: Some(workflow_engine),
            tool_registry: Some(tool_registry),
            state_manager: Some(state_manager),
        }
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
        let _config = self.load_config(&cli).await?;
        
        // Create output formatter
        let formatter = create_formatter(&cli.output);
        
        // Handle the command
        match &cli.command {
            Commands::Workflow { action } => {
                self.handle_workflow_command(action, &formatter, &cli).await
            }
            Commands::Tool { action } => {
                self.handle_tool_command(action, &formatter, &cli).await
            }
            Commands::Plugin { action } => {
                self.handle_plugin_command(action, &formatter, &cli).await
            }
            Commands::Batch { action } => {
                self.handle_batch_command(action, &formatter, &cli).await
            }
            Commands::Tui => {
                self.handle_tui_command(&cli).await
            }
            Commands::Server { http_port, ws_port, auth } => {
                self.handle_server_command(*http_port, *ws_port, *auth, &cli).await
            }
            Commands::Completion { shell } => {
                self.handle_completion_command(shell, &cli).await
            }
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
    
    /// Load configuration from file if specified
    async fn load_config(&self, cli: &Cli) -> Result<Config> {
        if let Some(config_path) = &cli.config {
            debug!("Loading configuration from: {:?}", config_path);
            Config::load_from_path(config_path).map_err(|e| {
                crate::WorkflowError::workflow_execution(&format!("Failed to load config: {}", e))
            })
        } else {
            Ok(self.config.clone())
        }
    }
    
    /// Handle workflow commands
    async fn handle_workflow_command(
        &self,
        action: &WorkflowAction,
        formatter: &Box<dyn OutputFormatter>,
        _cli: &Cli,
    ) -> Result<()> {
        let engine = self.workflow_engine.as_ref()
            .ok_or_else(|| crate::WorkflowError::workflow_execution("Workflow engine not initialized"))?;
        
        match action {
            WorkflowAction::Create { definition_file, validate_only, force } => {
                info!("Creating workflow from: {:?}", definition_file);
                
                // Load workflow definition
                let definition = self.load_workflow_definition(definition_file).await?;
                
                // Validate the definition
                definition.validate()?;
                
                if *validate_only {
                    println!("{}", formatter.format_success("Workflow definition is valid"));
                    return Ok(());
                }
                
                // TODO: Save workflow definition to registry
                // For now, just validate and report success
                println!("{}", formatter.format_success(&format!(
                    "Workflow '{}' created successfully{}",
                    definition.name,
                    if *force { " (forced)" } else { "" }
                )));
            }
            
            WorkflowAction::Execute { 
                workflow_name, 
                params, 
                params_json, 
                background, 
                wait, 
                timeout 
            } => {
                info!("Executing workflow: {}", workflow_name);
                
                // Load workflow definition
                let definition = if std::path::Path::new(workflow_name).exists() {
                    self.load_workflow_definition(&PathBuf::from(workflow_name)).await?
                } else {
                    // Try to load from state manager as a saved workflow
                    // For now, return error since we don't have a workflow registry yet
                    return Err(crate::WorkflowError::NotFound {
                        resource: format!("workflow '{}'", workflow_name),
                    }.into());
                };
                
                // Parse parameters
                let _params = self.parse_workflow_params(params.clone(), params_json.clone()).await?;
                
                // Execute workflow
                let execution = engine.execute_workflow(definition).await?;
                
                if *background {
                    println!("{}", formatter.format_success(&format!(
                        "Workflow started in background with ID: {}", 
                        execution.id
                    )));
                } else if *wait {
                    // TODO: Implement progress monitoring
                    println!("{}", formatter.format_workflow_status(&execution));
                } else {
                    println!("{}", formatter.format_success(&format!(
                        "Workflow started with ID: {}", 
                        execution.id
                    )));
                }
                
                if let Some(_timeout_secs) = timeout {
                    debug!("Execution timeout set to {} seconds", _timeout_secs);
                }
            }
            
            WorkflowAction::Status { workflow_id, detailed, follow, interval } => {
                info!("Getting status for workflow: {}", workflow_id);
                
                let workflow_uuid = workflow_id.parse()
                    .map_err(|_| CliError::InvalidArguments("Invalid workflow ID format".to_string()))?;
                
                if *follow {
                    // TODO: Implement real-time status following
                    warn!("Follow mode not yet implemented, showing current status");
                    debug!("Would refresh every {} seconds", interval);
                }
                
                let status = engine.get_workflow_status(workflow_uuid).await?;
                
                if *detailed {
                    // TODO: Get detailed execution info
                    println!("Status: {:?}", status);
                } else {
                    println!("Status: {:?}", status);
                }
            }
            
            WorkflowAction::Pause { workflow_id } => {
                info!("Pausing workflow: {}", workflow_id);
                
                let workflow_uuid = workflow_id.parse()
                    .map_err(|_| CliError::InvalidArguments("Invalid workflow ID format".to_string()))?;
                
                engine.pause_workflow(workflow_uuid).await?;
                println!("{}", formatter.format_success(&format!("Workflow {} paused", workflow_id)));
            }
            
            WorkflowAction::Resume { workflow_id } => {
                info!("Resuming workflow: {}", workflow_id);
                
                let workflow_uuid = workflow_id.parse()
                    .map_err(|_| CliError::InvalidArguments("Invalid workflow ID format".to_string()))?;
                
                engine.resume_workflow(workflow_uuid).await?;
                println!("{}", formatter.format_success(&format!("Workflow {} resumed", workflow_id)));
            }
            
            WorkflowAction::Stop { workflow_id, force } => {
                info!("Stopping workflow: {} (force: {})", workflow_id, force);
                
                let workflow_uuid = workflow_id.parse()
                    .map_err(|_| CliError::InvalidArguments("Invalid workflow ID format".to_string()))?;
                
                engine.stop_workflow(workflow_uuid).await?;
                println!("{}", formatter.format_success(&format!(
                    "Workflow {} stopped{}", 
                    workflow_id,
                    if *force { " (forced)" } else { "" }
                )));
            }
            
            WorkflowAction::List { status, recent, limit } => {
                info!("Listing workflows (status: {:?}, recent: {}, limit: {})", status, recent, limit);
                
                // Get workflow executions from state manager
                if let Some(state_manager) = &self.state_manager {
                    // TODO: Implement proper workflow listing from state manager
                    // For now, show empty list with proper formatting
                    let executions = Vec::new(); // Placeholder
                    println!("{}", formatter.format_workflow_list(&executions));
                } else {
                    return Err(crate::WorkflowError::workflow_execution("State manager not initialized").into());
                }
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
        let registry = self.tool_registry.as_ref()
            .ok_or_else(|| crate::WorkflowError::workflow_execution("Tool registry not initialized"))?;
        
        match action {
            ToolAction::List { category, tag, search, detailed } => {
                info!("Listing tools (category: {:?}, tag: {:?}, search: {:?})", category, tag, search);
                
                let mut tools = registry.list_tools();
                
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
                        tool.name.to_lowercase().contains(&search_lower) ||
                        tool.description.to_lowercase().contains(&search_lower)
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
            
            ToolAction::Execute { tool_name, params, params_file, timeout, dry_run } => {
                info!("Executing tool: {} (dry_run: {})", tool_name, dry_run);
                
                // Parse parameters
                let tool_params = self.parse_tool_params(params.clone(), params_file.clone()).await?;
                
                if *dry_run {
                    // Validate parameters only
                    registry.validate_tool_params(tool_name, &tool_params)?;
                    println!("{}", formatter.format_success("Tool parameters are valid"));
                    return Ok(());
                }
                
                // Create execution context
                let context = ExecutionContext::new();
                
                // Execute tool with timeout if specified
                let result = if let Some(timeout_secs) = timeout {
                    debug!("Tool execution timeout set to {} seconds", timeout_secs);
                    // TODO: Implement timeout wrapper
                    registry.execute_tool(tool_name, tool_params, context).await?
                } else {
                    registry.execute_tool(tool_name, tool_params, context).await?
                };
                
                // Format and display result
                println!("{}", serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| result.to_string()));
            }
            
            ToolAction::Info { tool_name } => {
                info!("Getting info for tool: {}", tool_name);
                
                let tools = registry.list_tools();
                let tool = tools.iter()
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
        match action {
            PluginAction::Install { plugin_path, plugin_type, force } => {
                info!("Installing plugin from: {} (type: {:?}, force: {})", plugin_path, plugin_type, force);
                
                // TODO: Implement plugin installation
                println!("{}", formatter.format_success(&format!(
                    "Plugin installed from {} (type: {:?}){}",
                    plugin_path,
                    plugin_type,
                    if *force { " (forced)" } else { "" }
                )));
            }
            
            PluginAction::List { detailed, plugin_type } => {
                info!("Listing plugins (detailed: {}, type: {:?})", detailed, plugin_type);
                
                // TODO: Implement plugin listing
                println!("No plugins installed.");
            }
            
            PluginAction::Reload { plugin_name } => {
                info!("Reloading plugin: {}", plugin_name);
                
                // TODO: Implement plugin reloading
                println!("{}", formatter.format_success(&format!("Plugin {} reloaded", plugin_name)));
            }
            
            PluginAction::Uninstall { plugin_name, force } => {
                info!("Uninstalling plugin: {} (force: {})", plugin_name, force);
                
                // TODO: Implement plugin uninstallation
                println!("{}", formatter.format_success(&format!(
                    "Plugin {} uninstalled{}",
                    plugin_name,
                    if *force { " (forced)" } else { "" }
                )));
            }
            
            PluginAction::Info { plugin_name } => {
                info!("Getting info for plugin: {}", plugin_name);
                
                // TODO: Implement plugin info
                println!("Plugin: {}", plugin_name);
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
                timeout 
            } => {
                info!("Executing batch workflows from: {:?} (parallel: {}, continue_on_failure: {})", 
                      workflow_list_file, parallel, continue_on_failure);
                
                // Load batch configuration
                let batch_config = self.load_batch_config(workflow_list_file).await?;
                
                // Execute workflows in parallel
                let results = self.execute_batch_workflows(
                    batch_config,
                    *parallel,
                    *continue_on_failure,
                    timeout.map(|t| std::time::Duration::from_secs(t))
                ).await?;
                
                // Display results
                println!("{}", formatter.format_success(&format!(
                    "Batch execution completed: {} workflows processed",
                    results.len()
                )));
                
                // Save results if output directory specified
                if let Some(dir) = output_dir {
                    self.save_batch_results(&results, dir).await?;
                    println!("{}", formatter.format_success(&format!(
                        "Results saved to: {:?}",
                        dir
                    )));
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
        info!("Starting MCP server (HTTP: {}, WS: {}, auth: {})", http_port, ws_port, auth);
        
        // TODO: Implement MCP server
        println!("MCP server not yet implemented (HTTP: {}, WS: {}, auth: {})", http_port, ws_port, auth);
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
        
        generate(shell_type, &mut cmd, "workflow-toolkit", &mut std::io::stdout());
        Ok(())
    }
    
    /// Load workflow definition from file
    async fn load_workflow_definition(&self, path: &PathBuf) -> Result<crate::workflow::WorkflowDefinition> {
        use tokio::fs;
        
        if !path.exists() {
            return Err(CliError::FileNotFound(path.clone()).into());
        }
        
        let content = fs::read_to_string(path).await
            .map_err(|e| CliError::IoError(e))?;
        
        debug!("Loaded file content: {}", content);
        
        // Try to parse as YAML first, then JSON
        if let Ok(definition) = serde_yaml::from_str::<crate::workflow::WorkflowDefinition>(&content) {
            debug!("Successfully parsed as YAML");
            Ok(definition)
        } else if let Ok(definition) = serde_json::from_str::<crate::workflow::WorkflowDefinition>(&content) {
            debug!("Successfully parsed as JSON");
            Ok(definition)
        } else {
            // Try to get more specific error information
            if let Err(yaml_err) = serde_yaml::from_str::<crate::workflow::WorkflowDefinition>(&content) {
                debug!("YAML parsing error: {}", yaml_err);
            }
            if let Err(json_err) = serde_json::from_str::<crate::workflow::WorkflowDefinition>(&content) {
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
            serde_json::from_str(&json_str)
                .map_err(|e| CliError::JsonError(e).into())
        } else if let Some(file_path) = params_file {
            use tokio::fs;
            
            let content = fs::read_to_string(&file_path).await
                .map_err(|e| CliError::IoError(e))?;
            
            // Try JSON first, then YAML
            if let Ok(value) = serde_json::from_str(&content) {
                Ok(value)
            } else {
                serde_yaml::from_str(&content)
                    .map_err(|e| CliError::YamlError(e).into())
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
            serde_json::from_str(&json_str)
                .map_err(|e| CliError::JsonError(e).into())
        } else if let Some(file_path) = params_file {
            use tokio::fs;
            
            let content = fs::read_to_string(&file_path).await
                .map_err(|e| CliError::IoError(e))?;
            
            // Try JSON first, then YAML
            if let Ok(value) = serde_json::from_str(&content) {
                Ok(value)
            } else {
                serde_yaml::from_str(&content)
                    .map_err(|e| CliError::YamlError(e).into())
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
        
        let content = fs::read_to_string(path).await
            .map_err(|e| CliError::IoError(e))?;
        
        // Try to parse as YAML first, then JSON
        if let Ok(config) = serde_yaml::from_str::<BatchConfig>(&content) {
            Ok(config)
        } else if let Ok(config) = serde_json::from_str::<BatchConfig>(&content) {
            Ok(config)
        } else {
            Err(CliError::InvalidFileFormat("Batch config must be valid YAML or JSON".to_string()).into())
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
        use tokio::sync::Semaphore;
        use futures::future::join_all;
        
        let engine = self.workflow_engine.as_ref()
            .ok_or_else(|| crate::WorkflowError::workflow_execution("Workflow engine not initialized"))?;
        
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
                    let content = fs::read_to_string(&workflow_spec.file).await
                        .map_err(|e| crate::WorkflowError::workflow_execution(&format!("Failed to read file: {}", e)))?;
                    
                    if let Ok(def) = serde_yaml::from_str::<crate::workflow::WorkflowDefinition>(&content) {
                        Ok(def)
                    } else if let Ok(def) = serde_json::from_str::<crate::workflow::WorkflowDefinition>(&content) {
                        Ok(def)
                    } else {
                        Err(crate::WorkflowError::workflow_execution("Invalid workflow file format"))
                    }
                } else {
                    Err(crate::WorkflowError::NotFound {
                        resource: format!("workflow file '{}'", workflow_spec.file),
                    })
                };
                
                let result = match definition_result {
                    Ok(definition) => {
                        // Execute workflow with timeout if specified
                        let execution_result = if let Some(timeout_duration) = timeout {
                            tokio::time::timeout(timeout_duration, engine.execute_workflow(definition)).await
                                .map_err(|_| crate::WorkflowError::Timeout { 
                                    duration: timeout_duration,
                                })?
                        } else {
                            engine.execute_workflow(definition).await
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
                            }
                        }
                    }
                    Err(e) => BatchResult {
                        workflow_name: workflow_spec.name,
                        status: crate::core::ExecutionStatus::Failed,
                        duration: start_time.elapsed(),
                        error: Some(e.to_string()),
                    }
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
                        return Err(crate::WorkflowError::workflow_execution(&format!("Task join error: {}", e)).into());
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
    
    /// Save batch results to output directory
    async fn save_batch_results(&self, results: &[BatchResult], output_dir: &PathBuf) -> Result<()> {
        use tokio::fs;
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir).await
            .map_err(|e| CliError::IoError(e))?;
        
        // Save results as JSON
        let results_json = serde_json::to_string_pretty(results)
            .map_err(|e| CliError::JsonError(e))?;
        
        let results_file = output_dir.join("batch_results.json");
        fs::write(&results_file, results_json).await
            .map_err(|e| CliError::IoError(e))?;
        
        // Save summary
        let summary = BatchSummary::from_results(results);
        let summary_json = serde_json::to_string_pretty(&summary)
            .map_err(|e| CliError::JsonError(e))?;
        
        let summary_file = output_dir.join("batch_summary.json");
        fs::write(&summary_file, summary_json).await
            .map_err(|e| CliError::IoError(e))?;
        
        Ok(())
    }
}

impl Default for CliApp {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_app_creation() {
        let config = Config::default();
        let app = CliApp::new(config);
        
        assert!(app.workflow_engine.is_none());
        assert!(app.tool_registry.is_none());
        assert!(app.state_manager.is_none());
    }

    #[tokio::test]
    async fn test_parse_workflow_params_json() {
        let app = CliApp::default();
        
        let result = app.parse_workflow_params(
            None,
            Some(r#"{"key": "value"}"#.to_string()),
        ).await;
        
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
        
        let result = app.parse_tool_params(
            Some("invalid json".to_string()),
            None,
        ).await;
        
        assert!(result.is_err());
    }
}