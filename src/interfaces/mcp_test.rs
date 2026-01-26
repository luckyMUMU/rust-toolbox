//! Test for MCP server interface integration

#[cfg(test)]
mod tests {
    use super::super::mcp::{McpServer, McpServerConfig, McpServerInterface};
    use crate::core::{AuthConfig, RateLimitConfig};
    use crate::tools::BasicToolRegistry;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let server = McpServer::new();
        // We can't access private fields, so just test that creation works
        let tools = server.list_tools().await.unwrap();
        assert_eq!(tools.len(), 0); // No tools registered yet
    }

    #[tokio::test]
    async fn test_mcp_server_tool_registration() {
        let mut server = McpServer::new();
        let tool_registry = Arc::new(BasicToolRegistry::new());

        // Register tools should work
        let result = server.register_tools(tool_registry).await;
        assert!(result.is_ok());

        // Should have default MCP tools registered
        let tools = server.list_tools().await.unwrap();
        assert!(tools.len() > 0);

        // Check that default tools are present
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"execute_workflow".to_string()));
        assert!(tool_names.contains(&"get_workflow_status".to_string()));
        assert!(tool_names.contains(&"install_plugin".to_string()));
    }

    #[tokio::test]
    async fn test_mcp_server_config_creation() {
        let config = McpServerConfig {
            http_port: 8080,
            ws_port: 8081,
            auth: AuthConfig {
                enabled: true,
                token: None,
                jwt_secret: Some("test_secret".to_string()),
                token_expiry: std::time::Duration::from_secs(3600),
                allowed_origins: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                burst_size: 10,
                enabled: true,
                max_requests: 1000,
                window_ms: 60000,
            },
            cors_origins: vec!["localhost".to_string(), "127.0.0.1".to_string()],
        };

        assert_eq!(config.http_port, 8080);
        assert_eq!(config.ws_port, 8081);
        assert!(config.auth.enabled);
    }

    #[tokio::test]
    async fn test_mcp_server_stub_operations() {
        let mut server = McpServer::new();
        let tool_registry = Arc::new(BasicToolRegistry::new());
        server.register_tools(tool_registry).await.unwrap();

        // Test stub implementations
        let config = McpServerConfig {
            http_port: 8080,
            ws_port: 8081,
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            cors_origins: vec!["*".to_string()],
        };

        // These should not fail (stub implementations)
        assert!(server.start(config).await.is_ok());
        assert!(server.stop().await.is_ok());

        // Test workflow operations (stubs)
        let workflow_request = crate::interfaces::mcp::McpWorkflowRequest {
            workflow_name: "test_workflow".to_string(),
            parameters: None,
        };
        let workflow_id = server.execute_workflow(workflow_request).await.unwrap();
        assert!(!workflow_id.is_empty());

        let status = server.get_workflow_status(&workflow_id).await.unwrap();
        assert_eq!(status.workflow_id, workflow_id);

        // Test tool execution (stub)
        let tool_request = crate::interfaces::mcp::McpToolRequest {
            name: "execute_workflow".to_string(),
            arguments: serde_json::json!({"workflow_name": "test"}),
        };
        let response = server.execute_tool(tool_request).await.unwrap();
        assert!(!response.content.is_empty());
    }
}
