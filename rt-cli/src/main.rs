use clap::{Parser, Subcommand};
use rt_core::Tool;
use std::collections::HashMap;
use std::sync:: Arc;

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
}

fn register_tools() -> HashMap<String, Box<dyn Tool>> {
    let mut tools: HashMap<String, Box<dyn Tool>> = HashMap::new();
    
    // Helper to register tool
    let mut register = |tool: Box<dyn Tool>| {
        tools.insert(tool.name().to_string(), tool);
    };

    // Register rt-tools
    register(Box::new(rt_tools::MoveFolder));

    tools
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let tools = Arc::new(register_tools());

    match &cli.command {
        Commands::List => {
            println!("Available Tools:");
            for tool in tools.values() {
                println!("  - {}: {}", tool.name(), tool.description(rt_core::Locale::En));
            }
        }
        Commands::Run { tool_name, input } => {
            if let Some(tool) = tools.get(tool_name) {
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
    }

    Ok(())
}
