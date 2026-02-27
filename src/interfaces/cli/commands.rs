//! Command line argument definitions using clap

use crate::core::ExecutionStatus;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Main CLI application structure
#[derive(Parser)]
#[command(name = "workflow-toolkit")]
#[command(about = "A multi-interface workflow execution system built with Rust")]
#[command(version)]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(long, global = true, help = "Path to configuration file")]
    pub config: Option<PathBuf>,

    /// Log level
    #[arg(long, global = true, default_value = "info", help = "Set log level")]
    pub log_level: String,

    /// Output format
    #[arg(long, global = true, default_value = "table", help = "Output format")]
    pub output: OutputFormat,

    /// Verbose output
    #[arg(short, long, global = true, help = "Enable verbose output")]
    pub verbose: bool,

    /// Quiet mode (suppress non-error output)
    #[arg(short, long, global = true, help = "Suppress non-error output")]
    pub quiet: bool,
}

/// Output format options
#[derive(Debug, Clone, ValueEnum, Default)]
pub enum OutputFormat {
    /// Human-readable table format
    #[default]
    Table,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Plain text format
    Text,
}

/// Top-level commands
#[derive(Subcommand)]
pub enum Commands {
    /// Workflow management commands
    Workflow {
        #[command(subcommand)]
        action: WorkflowAction,
    },
    /// Tool management commands
    Tool {
        #[command(subcommand)]
        action: ToolAction,
    },
    /// Plugin management commands
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// Batch execution commands
    Batch {
        #[command(subcommand)]
        action: BatchAction,
    },
    /// Start TUI interface
    Tui,
    /// Start MCP server
    Server {
        /// HTTP port for MCP server
        #[arg(long, default_value = "8080", help = "HTTP port for MCP server")]
        http_port: u16,
        /// WebSocket port for MCP server
        #[arg(long, default_value = "8081", help = "WebSocket port for MCP server")]
        ws_port: u16,
        /// Enable authentication
        #[arg(long, help = "Enable authentication")]
        auth: bool,
    },
    /// Generate shell completion scripts
    Completion {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Shell types for completion
#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

/// Workflow management actions
#[derive(Subcommand)]
pub enum WorkflowAction {
    /// Create a new workflow from definition file
    Create {
        /// Path to workflow definition file (YAML)
        #[arg(help = "Path to workflow definition file")]
        definition_file: PathBuf,
        /// Validate only, don't save
        #[arg(long, help = "Validate definition without saving")]
        validate_only: bool,
        /// Force overwrite if workflow exists
        #[arg(long, help = "Force overwrite existing workflow")]
        force: bool,
    },
    /// Execute a workflow
    Execute {
        /// Workflow name or path to definition file
        #[arg(help = "Workflow name or path to definition file")]
        workflow_name: String,
        /// Parameters file (JSON/YAML)
        #[arg(long, help = "Path to parameters file (JSON or YAML)")]
        params: Option<PathBuf>,
        /// Parameters as JSON string
        #[arg(long, help = "Parameters as JSON string")]
        params_json: Option<String>,
        /// Execute in background
        #[arg(long, help = "Execute in background and return immediately")]
        background: bool,
        /// Wait for completion and show progress
        #[arg(long, help = "Wait for completion and show progress")]
        wait: bool,
        /// Timeout in seconds
        #[arg(long, help = "Execution timeout in seconds")]
        timeout: Option<u64>,
    },
    /// Get workflow execution status
    Status {
        /// Workflow execution ID
        #[arg(help = "Workflow execution ID")]
        workflow_id: String,
        /// Show detailed node status
        #[arg(long, help = "Show detailed status for each node")]
        detailed: bool,
        /// Follow status updates
        #[arg(long, help = "Follow status updates in real-time")]
        follow: bool,
        /// Refresh interval in seconds (for follow mode)
        #[arg(long, default_value = "2", help = "Refresh interval in seconds")]
        interval: u64,
    },
    /// Pause a running workflow
    Pause {
        /// Workflow execution ID
        #[arg(help = "Workflow execution ID to pause")]
        workflow_id: String,
    },
    /// Resume a paused workflow
    Resume {
        /// Workflow execution ID
        #[arg(help = "Workflow execution ID to resume")]
        workflow_id: String,
    },
    /// Stop a workflow
    Stop {
        /// Workflow execution ID
        #[arg(help = "Workflow execution ID to stop")]
        workflow_id: String,
        /// Force stop without graceful shutdown
        #[arg(long, help = "Force stop without graceful shutdown")]
        force: bool,
    },
    /// List workflows
    List {
        /// Filter by status
        #[arg(long, help = "Filter by execution status")]
        status: Option<ExecutionStatus>,
        /// Show only recent executions
        #[arg(long, help = "Show only recent executions (last 24 hours)")]
        recent: bool,
        /// Maximum number of results
        #[arg(long, default_value = "50", help = "Maximum number of results")]
        limit: usize,
    },
}

/// Tool management actions
#[derive(Subcommand)]
pub enum ToolAction {
    /// List available tools
    List {
        /// Filter by category
        #[arg(long, help = "Filter tools by category")]
        category: Option<String>,
        /// Filter by tag
        #[arg(long, help = "Filter tools by tag")]
        tag: Option<String>,
        /// Search pattern
        #[arg(long, help = "Search tools by name or description")]
        search: Option<String>,
        /// Show detailed information
        #[arg(long, help = "Show detailed tool information")]
        detailed: bool,
    },
    /// Execute a tool
    Execute {
        /// Tool name
        #[arg(help = "Name of the tool to execute")]
        tool_name: String,
        /// Tool parameters (JSON format)
        #[arg(long, help = "Tool parameters as JSON string")]
        params: Option<String>,
        /// Parameters file (JSON/YAML)
        #[arg(long, help = "Path to parameters file")]
        params_file: Option<PathBuf>,
        /// Timeout in seconds
        #[arg(long, help = "Execution timeout in seconds")]
        timeout: Option<u64>,
        /// Dry run (validate parameters only)
        #[arg(long, help = "Validate parameters without executing")]
        dry_run: bool,
    },
    /// Show tool information
    Info {
        /// Tool name
        #[arg(help = "Name of the tool")]
        tool_name: String,
    },
}

/// Plugin management actions
#[derive(Subcommand)]
pub enum PluginAction {
    /// Install a plugin
    Install {
        /// Plugin path or URL
        #[arg(help = "Path to plugin or URL")]
        plugin_path: String,
        /// Plugin type
        #[arg(long, help = "Plugin type (auto-detected if not specified)")]
        plugin_type: Option<String>,
        /// Force reinstall if plugin exists
        #[arg(long, help = "Force reinstall if plugin already exists")]
        force: bool,
    },
    /// List installed plugins
    List {
        /// Show detailed information
        #[arg(long, help = "Show detailed plugin information")]
        detailed: bool,
        /// Filter by type
        #[arg(long, help = "Filter plugins by type")]
        plugin_type: Option<String>,
    },
    /// Reload a plugin
    Reload {
        /// Plugin name
        #[arg(help = "Name of the plugin to reload")]
        plugin_name: String,
    },
    /// Uninstall a plugin
    Uninstall {
        /// Plugin name
        #[arg(help = "Name of the plugin to uninstall")]
        plugin_name: String,
        /// Force uninstall without confirmation
        #[arg(long, help = "Force uninstall without confirmation")]
        force: bool,
    },
    /// Show plugin information
    Info {
        /// Plugin name
        #[arg(help = "Name of the plugin")]
        plugin_name: String,
    },
}

/// Batch execution actions
#[derive(Subcommand)]
pub enum BatchAction {
    /// Execute multiple workflows
    Execute {
        /// File containing list of workflows to execute
        #[arg(help = "Path to file containing workflow list")]
        workflow_list_file: PathBuf,
        /// Maximum parallel executions
        #[arg(
            long,
            default_value = "4",
            help = "Maximum number of parallel executions"
        )]
        parallel: usize,
        /// Continue on failure
        #[arg(long, help = "Continue execution even if some workflows fail")]
        continue_on_failure: bool,
        /// Output directory for results
        #[arg(long, help = "Directory to save execution results")]
        output_dir: Option<PathBuf>,
        /// Timeout for each workflow in seconds
        #[arg(long, help = "Timeout for each workflow in seconds")]
        timeout: Option<u64>,
    },
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Parse command line arguments from iterator
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        clap::Parser::parse_from(args)
    }

    /// Validate command arguments
    pub fn validate(&self) -> Result<(), crate::interfaces::cli::CliError> {
        // Add validation logic here if needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        // Test basic command parsing
        let cli = Cli::parse_from(&["workflow-toolkit", "workflow", "list"]);
        match cli.command {
            Commands::Workflow {
                action: WorkflowAction::List { .. },
            } => {}
            _ => panic!("Expected workflow list command"),
        }
    }

    #[test]
    fn test_workflow_execute_command() {
        let cli = Cli::parse_from(&[
            "workflow-toolkit",
            "workflow",
            "execute",
            "test-workflow",
            "--params-json",
            r#"{"key": "value"}"#,
            "--wait",
        ]);

        match cli.command {
            Commands::Workflow {
                action:
                    WorkflowAction::Execute {
                        workflow_name,
                        params_json,
                        wait,
                        ..
                    },
            } => {
                assert_eq!(workflow_name, "test-workflow");
                assert_eq!(params_json, Some(r#"{"key": "value"}"#.to_string()));
                assert!(wait);
            }
            _ => panic!("Expected workflow execute command"),
        }
    }

    #[test]
    fn test_tool_list_command() {
        let cli = Cli::parse_from(&[
            "workflow-toolkit",
            "tool",
            "list",
            "--category",
            "data-processing",
            "--detailed",
        ]);

        match cli.command {
            Commands::Tool {
                action:
                    ToolAction::List {
                        category, detailed, ..
                    },
            } => {
                assert_eq!(category, Some("data-processing".to_string()));
                assert!(detailed);
            }
            _ => panic!("Expected tool list command"),
        }
    }

    #[test]
    fn test_batch_execute_command() {
        let cli = Cli::parse_from(&[
            "workflow-toolkit",
            "batch",
            "execute",
            "workflows.yaml",
            "--parallel",
            "8",
            "--continue-on-failure",
        ]);

        match cli.command {
            Commands::Batch {
                action:
                    BatchAction::Execute {
                        workflow_list_file,
                        parallel,
                        continue_on_failure,
                        ..
                    },
            } => {
                assert_eq!(workflow_list_file, PathBuf::from("workflows.yaml"));
                assert_eq!(parallel, 8);
                assert!(continue_on_failure);
            }
            _ => panic!("Expected batch execute command"),
        }
    }

    #[test]
    fn test_global_options() {
        let cli = Cli::parse_from(&[
            "workflow-toolkit",
            "--config",
            "/path/to/config.toml",
            "--log-level",
            "debug",
            "--output",
            "json",
            "--verbose",
            "workflow",
            "list",
        ]);

        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.toml")));
        assert_eq!(cli.log_level, "debug");
        assert!(matches!(cli.output, OutputFormat::Json));
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_help() {
        // Test that help can be generated without panicking
        let _help = Cli::command().render_help();
    }

    #[test]
    fn test_output_format_parsing() {
        assert!(matches!(
            OutputFormat::from_str("table", true),
            Ok(OutputFormat::Table)
        ));
        assert!(matches!(
            OutputFormat::from_str("json", true),
            Ok(OutputFormat::Json)
        ));
        assert!(matches!(
            OutputFormat::from_str("yaml", true),
            Ok(OutputFormat::Yaml)
        ));
        assert!(matches!(
            OutputFormat::from_str("text", true),
            Ok(OutputFormat::Text)
        ));
    }
}
