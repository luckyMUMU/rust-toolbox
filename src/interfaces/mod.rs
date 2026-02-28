//! # Interface Layer
//!
//! This module contains the different user interfaces for the workflow toolkit:
//!
//! - **CLI**: Command-line interface for batch processing and automation.
//! - **TUI**: Terminal user interface for interactive monitoring and management.
//! - **MCP**: Model Context Protocol server for AI assistant integration.
//! - **DTO**: Data Transfer Objects for interface layer data exchange.
pub mod cli;
pub mod dto;
pub mod mcp;
#[cfg(feature = "mcp")]
pub mod mcp_server;
pub mod tui;

#[cfg(test)]
mod mcp_test;

pub use cli::CliInterface;
pub use dto::*;
pub use mcp::McpServer;
#[cfg(feature = "mcp")]
pub use mcp_server::{McpServerConfig, WorkflowMcpServer, WorkflowMcpServerBuilder};
pub use tui::{LayoutManager, MainTuiInterface, Theme, ThemeManager, Widget, WidgetRegistry};
