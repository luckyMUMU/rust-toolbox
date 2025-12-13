use clap::{Parser, Subcommand};
use rt_core::{Tool, WorkflowEngine, InMemoryWorkflowEngine, WorkflowDefinition, WorkflowStatus};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "rt-cli")]
#[command(about = "Rust Toolbox CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available tools
    List,
    /// Run a specific tool by name
    Run {
        /// Name of the tool to run
        tool_name: String,
        /// Input arguments as JSON string
        #[arg(long, default_value = "{}")]
        input: String,
    },
    /// Manage workflows
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// Run a workflow from a file
    Run {
        /// Path to workflow definition JSON file
        file: String,
    },
}

async fn register_tools() -> HashMap<String, Box<dyn Tool>> {
    let mut tools: HashMap<String, Box<dyn Tool>> = HashMap::new();
    
    // Built-in tools
    for tool in rt_tools::get_all_tools() {
        tools.insert(tool.name().to_string(), tool);
    }

    // Plugins
    let plugin_dir = std::path::Path::new("plugins");
    if plugin_dir.exists() {
         let plugins = rt_core::plugin::load_plugins(plugin_dir).await;
         for tool in plugins {
             tools.insert(tool.name().to_string(), tool);
         }
    }

    tools
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    // For workflow engine, we need ownership of tools map, but register_tools returns it.
    // However, Cli::Run also needs tools.
    // We can call register_tools() once.
    let tools_map = register_tools().await;
    // We need to clone keys or wrap tools in Arc if we want to share.
    // Tool trait is Send+Sync.
    // But Box<dyn Tool> is not Clone.
    // So we can't easily share the SAME map between direct Run and WorkflowEngine if both consume it.
    // But here we are in a CLI, we execute ONE command.
    
    match &cli.command {
        Commands::List => {
            println!("Available Tools:");
            for tool in tools_map.values() {
                println!("  - {}: {}", tool.name(), tool.description(rt_core::Locale::En));
            }
        }
        Commands::Run { tool_name, input } => {
            if let Some(tool) = tools_map.get(tool_name) {
                let input_value: serde_json::Value = serde_json::from_str(input)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON input: {}", e))?;
                
                match tool.run(input_value).await {
                    Ok(result) => {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    }
                    Err(e) => {
                        eprintln!("Tool execution failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: Tool '{}' not found.", tool_name);
                std::process::exit(1);
            }
        }
        Commands::Workflow { command } => {
            match command {
                WorkflowCommands::Run { file } => {
                    let content = std::fs::read_to_string(file)
                        .map_err(|e| anyhow::anyhow!("Failed to read workflow file: {}", e))?;
                    let def: WorkflowDefinition = serde_json::from_str(&content)
                        .map_err(|e| anyhow::anyhow!("Invalid workflow definition: {}", e))?;
                    
                    let engine = InMemoryWorkflowEngine::new(tools_map);
                    
                    println!("Starting workflow: {} ({})", def.name, def.id);
                    let instance_id = engine.start_workflow(def).await?;
                    println!("Instance ID: {}", instance_id);
                    
                    // Poll status
                    loop {
                        let status = engine.get_status(&instance_id).await?;
                        match status.status {
                            WorkflowStatus::Completed => {
                                println!("Workflow completed successfully.");
                                println!("Context: {}", serde_json::to_string_pretty(&status.context)?);
                                break;
                            }
                            WorkflowStatus::Failed(e) => {
                                eprintln!("Workflow failed: {}", e);
                                std::process::exit(1);
                            }
                            WorkflowStatus::Paused => {
                                println!("Workflow paused.");
                                break;
                            }
                            _ => {
                                // Pending or Running
                                tokio::time::sleep(Duration::from_millis(500)).await;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
