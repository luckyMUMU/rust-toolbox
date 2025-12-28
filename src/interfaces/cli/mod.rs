//! CLI interface implementation for the workflow toolkit

pub mod app;
pub mod commands;
pub mod output;
pub mod error;

pub use app::CliApp;
pub use commands::{Cli, Commands, WorkflowAction, ToolAction, PluginAction, BatchAction};
pub use output::{OutputFormat, OutputFormatter, TableFormatter, JsonFormatter, YamlFormatter};
pub use error::CliError;

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