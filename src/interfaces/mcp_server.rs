//! MCP Server implementation - Simplified version using JSON-RPC
//!
//! This module provides a complete MCP (Model Context Protocol) server implementation
//! that allows AI assistants to discover and execute tools from the workflow toolkit.
//! Uses a simplified JSON-RPC approach without complex macro dependencies.

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::tools::{ToolNode, ToolRegistry};
use crate::workflow::RefactoredWorkflowEngine;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::Arc;
use tracing::{debug, error, info};

/// MCP Server configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: "workflow-toolkit-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Tool registration info
#[derive(Clone)]
struct RegisteredTool {
    name: String,
    description: String,
    input_schema: serde_json::Map<String, Value>,
    tool: Arc<dyn ToolNode>,
}

impl std::fmt::Debug for RegisteredTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

/// MCP Server implementation using simplified JSON-RPC
#[derive(Clone)]
pub struct WorkflowMcpServer {
    /// Server configuration
    config: McpServerConfig,
    /// Registered tools
    tools: Arc<std::sync::RwLock<HashMap<String, RegisteredTool>>>,
    /// Tool registry reference
    tool_registry: Option<Arc<dyn ToolRegistry>>,
    /// Workflow engine reference
    workflow_engine: Option<Arc<RefactoredWorkflowEngine>>,
}

impl std::fmt::Debug for WorkflowMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowMcpServer")
            .field("config", &self.config)
            .field(
                "tools_count",
                &self.tools.read().map(|t| t.len()).unwrap_or(0),
            )
            .finish_non_exhaustive()
    }
}

impl WorkflowMcpServer {
    /// Create a new MCP server with the given configuration
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            tools: Arc::new(std::sync::RwLock::new(HashMap::new())),
            tool_registry: None,
            workflow_engine: None,
        }
    }

    /// Set the tool registry
    pub fn with_tool_registry(mut self, registry: Arc<dyn ToolRegistry>) -> Self {
        self.tool_registry = Some(registry.clone());
        self
    }

    /// Set the workflow engine
    pub fn with_workflow_engine(mut self, engine: Arc<RefactoredWorkflowEngine>) -> Self {
        self.workflow_engine = Some(engine);
        self
    }

    /// Register a tool with the MCP server
    pub fn register_tool(&self, tool: Arc<dyn ToolNode>) -> Result<()> {
        let name = tool.name().to_string();
        let description = tool.description();

        // Convert Value to Map for input schema
        let input_schema = match &tool.definition().parameters_schema {
            Value::Object(map) => map.clone(),
            _ => serde_json::Map::new(),
        };

        let registered_tool = RegisteredTool {
            name: name.clone(),
            description,
            input_schema,
            tool,
        };

        let mut tools = self
            .tools
            .write()
            .map_err(|e| WorkflowError::ConcurrentAccess {
                message: format!("Failed to lock tools: {}", e),
            })?;

        tools.insert(name.clone(), registered_tool);
        info!("Registered MCP tool: {}", name);

        Ok(())
    }

    /// Register all tools from the tool registry
    pub fn register_all_tools(&self) -> Result<()> {
        if let Some(ref registry) = self.tool_registry {
            let tool_infos = registry.list_tools();
            for tool_info in tool_infos {
                if let Some(tool) = registry.get_tool(&tool_info.name) {
                    self.register_tool(tool)?;
                }
            }
        }
        Ok(())
    }

    /// Get tool by name
    fn get_tool(&self, name: &str) -> Option<RegisteredTool> {
        self.tools.read().ok()?.get(name).cloned()
    }

    /// Start the MCP server with stdio transport
    pub async fn start_stdio(self) -> Result<()> {
        info!("Starting MCP server with stdio transport");

        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();

        // Send initialization message
        let init_response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": null,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {}
                },
                "serverInfo": {
                    "name": self.config.name,
                    "version": self.config.version
                }
            }
        });

        Self::send_message(&mut stdout_lock, &init_response)?;

        // Process incoming messages
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    match self.handle_request(&line).await {
                        Ok(response) => {
                            if let Err(e) = Self::send_message(&mut stdout_lock, &response) {
                                error!("Failed to send response: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to handle request: {}", e);
                            let error_response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": null,
                                "error": {
                                    "code": -32603,
                                    "message": format!("Internal error: {}", e)
                                }
                            });
                            let _ = Self::send_message(&mut stdout_lock, &error_response);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read line: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Send a JSON-RPC message
    fn send_message(writer: &mut dyn Write, message: &Value) -> Result<()> {
        let msg_str = serde_json::to_string(message)?;
        writeln!(writer, "{}", msg_str)?;
        writer.flush()?;
        Ok(())
    }

    /// Handle an incoming JSON-RPC request
    async fn handle_request(&self, request_str: &str) -> Result<Value> {
        let request: JsonRpcRequest = serde_json::from_str(request_str)
            .map_err(|e| WorkflowError::workflow_execution(format!("Invalid JSON: {}", e)))?;

        debug!("Received request: method={}", request.method);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(),
            "tools/list" => self.handle_list_tools(),
            "tools/call" => self.handle_call_tool(request.params).await,
            "resources/list" => self.handle_list_resources(),
            "resources/read" => self.handle_read_resource(request.params),
            _ => Err(WorkflowError::workflow_execution(format!(
                "Unknown method: {}",
                request.method
            ))),
        };

        match result {
            Ok(result) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": result
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32603,
                    "message": e.to_string()
                }
            })),
        }
    }

    /// Handle initialize request
    fn handle_initialize(&self) -> Result<Value> {
        Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": self.config.name,
                "version": self.config.version
            }
        }))
    }

    /// Handle tools/list request
    fn handle_list_tools(&self) -> Result<Value> {
        let tools = self
            .tools
            .read()
            .map_err(|e| WorkflowError::ConcurrentAccess {
                message: format!("Failed to lock tools: {}", e),
            })?;

        let tool_list: Vec<Value> = tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema
                })
            })
            .collect();

        Ok(serde_json::json!({
            "tools": tool_list
        }))
    }

    /// Handle tools/call request
    async fn handle_call_tool(&self, params: Option<Value>) -> Result<Value> {
        let params = params
            .ok_or_else(|| WorkflowError::workflow_execution("Missing params".to_string()))?;

        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkflowError::workflow_execution("Missing tool name".to_string()))?;

        let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

        info!("Executing tool: {} with args: {:?}", tool_name, arguments);

        // Find the tool
        let tool = self.get_tool(tool_name).ok_or_else(|| {
            WorkflowError::workflow_execution(format!("Tool '{}' not found", tool_name))
        })?;

        // Create execution context
        let exec_context = ExecutionContext::new();

        // Execute the tool
        match tool.tool.execute(arguments, exec_context).await {
            Ok(result) => {
                debug!("Tool {} executed successfully", tool_name);
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": result.to_string()
                        }
                    ]
                }))
            }
            Err(e) => {
                error!("Tool {} execution failed: {}", tool_name, e);
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Error: {}", e)
                        }
                    ],
                    "isError": true
                }))
            }
        }
    }

    /// Handle resources/list request
    fn handle_list_resources(&self) -> Result<Value> {
        // For now, return empty list
        Ok(serde_json::json!({
            "resources": []
        }))
    }

    /// Handle resources/read request
    fn handle_read_resource(&self, params: Option<Value>) -> Result<Value> {
        let params = params
            .ok_or_else(|| WorkflowError::workflow_execution("Missing params".to_string()))?;

        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkflowError::workflow_execution("Missing URI".to_string()))?;

        debug!("Reading resource: {}", uri);

        // Parse URI: flow://{trace_id}/context/{key}
        if let Some(resource_path) = uri.strip_prefix("flow://") {
            let parts: Vec<&str> = resource_path.split('/').collect();

            if parts.len() >= 2 {
                let _trace_id = parts[0];
                let resource_type = parts[1];

                let content = match resource_type {
                    "context" => format!("Context data for trace {}", _trace_id),
                    "logs" => format!("Logs for trace {}", _trace_id),
                    _ => format!("Unknown resource type: {}", resource_type),
                };

                return Ok(serde_json::json!({
                    "contents": [
                        {
                            "uri": uri,
                            "mimeType": "text/plain",
                            "text": content
                        }
                    ]
                }));
            }
        }

        Err(WorkflowError::workflow_execution(format!(
            "Invalid resource URI: {}",
            uri
        )))
    }
}

/// JSON-RPC request structure
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

/// Builder for WorkflowMcpServer
pub struct WorkflowMcpServerBuilder {
    config: McpServerConfig,
    tool_registry: Option<Arc<dyn ToolRegistry>>,
    workflow_engine: Option<Arc<RefactoredWorkflowEngine>>,
}

impl WorkflowMcpServerBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: McpServerConfig::default(),
            tool_registry: None,
            workflow_engine: None,
        }
    }

    /// Set server name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set server version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.config.version = version.into();
        self
    }

    /// Set tool registry
    pub fn with_tool_registry(mut self, registry: Arc<dyn ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    /// Set workflow engine
    pub fn with_workflow_engine(mut self, engine: Arc<RefactoredWorkflowEngine>) -> Self {
        self.workflow_engine = Some(engine);
        self
    }

    /// Build the MCP server
    pub fn build(self) -> WorkflowMcpServer {
        let mut server = WorkflowMcpServer::new(self.config);

        if let Some(registry) = self.tool_registry {
            server.tool_registry = Some(registry);
        }

        if let Some(engine) = self.workflow_engine {
            server.workflow_engine = Some(engine);
        }

        server
    }
}

impl Default for WorkflowMcpServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_builder() {
        let server = WorkflowMcpServerBuilder::new()
            .with_name("test-server")
            .with_version("1.0.0")
            .build();

        assert_eq!(server.config.name, "test-server");
        assert_eq!(server.config.version, "1.0.0");
    }

    #[test]
    fn test_resource_uri_parsing() {
        let uri = "flow://trace-123/context/user_id";
        let parts: Vec<&str> = uri.strip_prefix("flow://").unwrap().split('/').collect();

        assert_eq!(parts[0], "trace-123");
        assert_eq!(parts[1], "context");
        assert_eq!(parts[2], "user_id");
    }
}
