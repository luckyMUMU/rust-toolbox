//! MCP server interface implementation

use crate::core::{AuthConfig, RateLimitConfig};
use crate::error::Result;
use crate::tools::ToolRegistry;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// MCP server configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub http_port: u16,
    pub ws_port: u16,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub cors_origins: Vec<String>,
}

/// MCP tool definition following the MCP protocol specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolRequest {
    pub name: String,
    pub arguments: Value,
}

/// MCP tool execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResponse {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// MCP content item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// MCP workflow execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWorkflowRequest {
    pub workflow_name: String,
    pub parameters: Option<Value>,
}

/// MCP workflow status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWorkflowStatusResponse {
    pub workflow_id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub current_node: Option<String>,
    pub progress: Option<f64>,
}

/// MCP plugin installation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPluginRequest {
    pub plugin_source: String,
    pub plugin_type: String,
    pub config: Option<Value>,
}

/// MCP server interface trait
#[async_trait]
pub trait McpServerInterface: Send + Sync {
    /// Start the MCP server with the given configuration
    async fn start(&self, config: McpServerConfig) -> Result<()>;
    
    /// Stop the MCP server
    async fn stop(&self) -> Result<()>;
    
    /// Register MCP tools from the tool registry
    async fn register_tools(&mut self, tool_registry: Arc<dyn ToolRegistry>) -> Result<()>;
    
    /// Get list of available MCP tools
    async fn list_tools(&self) -> Result<Vec<McpToolDefinition>>;
    
    /// Execute an MCP tool
    async fn execute_tool(&self, request: McpToolRequest) -> Result<McpToolResponse>;
    
    /// Execute a workflow via MCP
    async fn execute_workflow(&self, request: McpWorkflowRequest) -> Result<String>;
    
    /// Get workflow status via MCP
    async fn get_workflow_status(&self, workflow_id: &str) -> Result<McpWorkflowStatusResponse>;
    
    /// Install a plugin via MCP
    async fn install_plugin(&self, request: McpPluginRequest) -> Result<String>;
    
    /// Pause a workflow via MCP
    async fn pause_workflow(&self, workflow_id: &str) -> Result<()>;
    
    /// Resume a workflow via MCP
    async fn resume_workflow(&self, workflow_id: &str) -> Result<()>;
    
    /// Stop a workflow via MCP
    async fn stop_workflow(&self, workflow_id: &str) -> Result<()>;
}

/// Basic MCP server implementation (stub)
/// Note: Full MCP implementation will be added in later tasks
pub struct McpServer {
    /// Registered MCP tools
    tools: HashMap<String, McpToolDefinition>,
    /// Reference to tool registry for tool execution
    tool_registry: Option<Arc<dyn ToolRegistry>>,
    /// Server running state
    is_running: bool,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            tool_registry: None,
            is_running: false,
        }
    }
    
    /// Create default MCP tool definitions based on the design specification
    fn create_default_tools() -> Vec<McpToolDefinition> {
        vec![
            McpToolDefinition {
                name: "execute_workflow".to_string(),
                description: "执行指定的工作流".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_name": {"type": "string"},
                        "parameters": {"type": "object"}
                    },
                    "required": ["workflow_name"]
                }),
            },
            McpToolDefinition {
                name: "get_workflow_status".to_string(),
                description: "获取工作流执行状态".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {"type": "string"}
                    },
                    "required": ["workflow_id"]
                }),
            },
            McpToolDefinition {
                name: "install_plugin".to_string(),
                description: "安装新的插件".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plugin_source": {"type": "string"},
                        "plugin_type": {
                            "type": "string", 
                            "enum": ["python", "nodejs", "docker", "wasm", "native"]
                        }
                    },
                    "required": ["plugin_source", "plugin_type"]
                }),
            },
            McpToolDefinition {
                name: "pause_workflow".to_string(),
                description: "暂停指定的工作流".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {"type": "string"}
                    },
                    "required": ["workflow_id"]
                }),
            },
            McpToolDefinition {
                name: "resume_workflow".to_string(),
                description: "恢复指定的工作流".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {"type": "string"}
                    },
                    "required": ["workflow_id"]
                }),
            },
            McpToolDefinition {
                name: "stop_workflow".to_string(),
                description: "停止指定的工作流".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": {"type": "string"}
                    },
                    "required": ["workflow_id"]
                }),
            },
        ]
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpServerInterface for McpServer {
    async fn start(&self, _config: McpServerConfig) -> Result<()> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP server start requested - implementation pending");
        println!("Available MCP tools: {:?}", self.tools.keys().collect::<Vec<_>>());
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP server stop requested - implementation pending");
        Ok(())
    }
    
    async fn register_tools(&mut self, tool_registry: Arc<dyn ToolRegistry>) -> Result<()> {
        // Store reference to tool registry
        self.tool_registry = Some(tool_registry.clone());
        
        // Register default MCP tools
        let default_tools = Self::create_default_tools();
        for tool in default_tools {
            self.tools.insert(tool.name.clone(), tool);
        }
        
        // Register tools from the tool registry as MCP tools
        let tool_infos = tool_registry.list_tools();
        for tool_info in tool_infos {
            let mcp_tool = McpToolDefinition {
                name: format!("tool_{}", tool_info.name),
                description: if tool_info.description.is_empty() {
                    format!("Execute tool: {}", tool_info.name)
                } else {
                    tool_info.description
                },
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "parameters": {"type": "object"}
                    }
                }),
            };
            self.tools.insert(mcp_tool.name.clone(), mcp_tool);
        }
        
        println!("Registered {} MCP tools", self.tools.len());
        Ok(())
    }
    
    async fn list_tools(&self) -> Result<Vec<McpToolDefinition>> {
        Ok(self.tools.values().cloned().collect())
    }
    
    async fn execute_tool(&self, request: McpToolRequest) -> Result<McpToolResponse> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP tool execution requested: {} with args: {:?}", request.name, request.arguments);
        
        // Check if tool exists
        if !self.tools.contains_key(&request.name) {
            return Ok(McpToolResponse {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: format!("Tool '{}' not found", request.name),
                }],
                is_error: Some(true),
            });
        }
        
        // For now, return a stub response
        Ok(McpToolResponse {
            content: vec![McpContent {
                content_type: "text".to_string(),
                text: format!("Tool '{}' execution requested - implementation pending", request.name),
            }],
            is_error: None,
        })
    }
    
    async fn execute_workflow(&self, request: McpWorkflowRequest) -> Result<String> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP workflow execution requested: {} with params: {:?}", request.workflow_name, request.parameters);
        Ok("workflow_execution_stub_id".to_string())
    }
    
    async fn get_workflow_status(&self, workflow_id: &str) -> Result<McpWorkflowStatusResponse> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP workflow status requested for: {}", workflow_id);
        Ok(McpWorkflowStatusResponse {
            workflow_id: workflow_id.to_string(),
            status: "pending".to_string(),
            started_at: None,
            completed_at: None,
            current_node: None,
            progress: None,
        })
    }
    
    async fn install_plugin(&self, request: McpPluginRequest) -> Result<String> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP plugin installation requested: {} ({})", request.plugin_source, request.plugin_type);
        Ok("plugin_installation_stub_id".to_string())
    }
    
    async fn pause_workflow(&self, workflow_id: &str) -> Result<()> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP workflow pause requested for: {}", workflow_id);
        Ok(())
    }
    
    async fn resume_workflow(&self, workflow_id: &str) -> Result<()> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP workflow resume requested for: {}", workflow_id);
        Ok(())
    }
    
    async fn stop_workflow(&self, workflow_id: &str) -> Result<()> {
        // Stub implementation - will be implemented in later tasks
        println!("MCP workflow stop requested for: {}", workflow_id);
        Ok(())
    }
}