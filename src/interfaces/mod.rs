//! # Interface Layer
//!
//! This module contains the different user interfaces for the workflow toolkit:
//!
//! - **CLI**: Command-line interface for batch processing and automation.
//! - **TUI**: Terminal user interface for interactive monitoring and management.
//! - **MCP**: Model Context Protocol server for AI assistant integration.
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.
pub mod cli;
pub mod mcp;
#[cfg(feature = "mcp")]
pub mod mcp_server;
pub mod tui;

#[cfg(test)]
mod mcp_test;

pub use cli::CliInterface;
pub use mcp::McpServer;
#[cfg(feature = "mcp")]
pub use mcp_server::{WorkflowMcpServer, WorkflowMcpServerBuilder, McpServerConfig};
pub use tui::{LayoutManager, MainTuiInterface, Theme, ThemeManager, Widget, WidgetRegistry};
