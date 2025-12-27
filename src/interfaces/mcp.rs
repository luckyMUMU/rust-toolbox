//! MCP server interface implementation

use crate::core::{AuthConfig, RateLimitConfig};
use crate::error::Result;

/// MCP server configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub http_port: u16,
    pub ws_port: u16,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
}

/// MCP server interface trait
pub trait McpServerInterface: Send + Sync {
    fn start(&self, config: McpServerConfig) -> Result<()>;
    fn stop(&self) -> Result<()>;
}

/// Basic MCP server implementation (stub)
/// Note: Full MCP implementation will be added in later tasks
pub struct McpServer {
    // Implementation will be added later when MCP dependencies are available
}

impl McpServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServerInterface for McpServer {
    fn start(&self, _config: McpServerConfig) -> Result<()> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP server start requested - implementation pending");
        Ok(())
    }
    
    fn stop(&self) -> Result<()> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP server stop requested - implementation pending");
        Ok(())
    }
}