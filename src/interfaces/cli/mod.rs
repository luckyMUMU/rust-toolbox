//! # Command Line Interface (CLI)
//!
//! The CLI provides a standard way to interact with the workflow toolkit from the terminal.
//! It supports:
//! - Workflow execution (single or batch)
//! - Tool management (list, inspect)
//! - Plugin management
//! - MCP server execution
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.
pub mod app;
pub mod commands;
pub mod error;
pub mod output;

pub use app::CliApp;
pub use commands::{BatchAction, Cli, Commands, PluginAction, ToolAction, WorkflowAction};
pub use error::CliError;
pub use output::{JsonFormatter, OutputFormat, OutputFormatter, TableFormatter, YamlFormatter};

/// CLI interface for the workflow toolkit
pub struct CliInterface {
    app: CliApp,
}

impl CliInterface {
    pub fn new(app: CliApp) -> Self {
        Self { app }
    }

    pub async fn run(&self, args: Vec<String>) -> crate::Result<()> {
        self.app.run(args).await
    }
}

impl Default for CliInterface {
    fn default() -> Self {
        Self::new(CliApp::default())
    }
}
