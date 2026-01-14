//! # Interface Layer
//!
//! This module contains the different user interfaces for the workflow toolkit:
//!
//! - **CLI**: Command-line interface for batch processing and automation.
//! - **TUI**: Terminal user interface for interactive monitoring and management.
//! - **MCP**: Model Context Protocol server for AI assistant integration.
pub mod cli;
pub mod mcp;
pub mod tui;

#[cfg(test)]
mod mcp_test;

pub use cli::CliInterface;
pub use mcp::McpServer;
pub use tui::{LayoutManager, MainTuiInterface, Theme, ThemeManager, Widget, WidgetRegistry};
