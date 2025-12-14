use super::*;
use serde_json::json;
use crate::service::{ServiceContext, CallerType, PermissionLevel};

#[tokio::test]
async fn test_mcp_context_creation() {
    // 创建新的 MCP 上下文
    let context = McpContext::new();
    assert!(!context.id.is_empty(), "Context ID should not be empty");
    assert_eq!(context.parent_id, None, "Parent ID should be None for new context");
    assert_eq!(context.execution_history.len(), 0, "Execution history should be empty for new context");
}

#[tokio::test]
async fn test_mcp_context_child_creation() {
    // 创建父上下文
    let parent = McpContext::new();
    let parent_id = parent.id.clone();
    
    // 创建子上下文
    let child = parent.create_child();
    assert!(!child.id.is_empty(), "Child context ID should not be empty");
    assert_eq!(child.parent_id, Some(parent_id), "Child context should have parent ID");
    assert_eq!(child.model_state, parent.model_state, "Child context should inherit model state");
}

#[tokio::test]
async fn test_mcp_context_execution_record() {
    // 创建上下文
    let mut context = McpContext::new();
    
    // 创建执行记录
    let record = ExecutionRecord {
        id: "test-record-1".to_string(),
        timestamp: chrono::Utc::now(),
        component_type: ComponentType::Tool,
        component_name: "test-tool".to_string(),
        method: "run".to_string(),
        input: json!("test-input"),
        output: Some(json!("test-output")),
        status: ExecutionStatus::Success,
        error: None,
        duration_ms: Some(100),
    };
    
    // 添加执行记录
    context.add_execution_record(record.clone());
    assert_eq!(context.execution_history.len(), 1, "Execution history should have one record");
    assert_eq!(context.execution_history[0].id, record.id, "Execution record ID should match");
    assert_eq!(context.execution_history[0].component_name, record.component_name, "Execution record component name should match");
}

#[tokio::test]
async fn test_mcp_request_creation() {
    // 创建 MCP 上下文
    let context = McpContext::new();
    
    // 创建服务上下文
    let service_context = ServiceContext {
        caller_id: "test-caller".to_string(),
        caller_type: CallerType::User,
        permission_level: PermissionLevel::Standard,
        extra: json!({ "test": "extra" }),
    };
    
    // 创建 MCP 请求
    let request = McpRequest::new_tool_call(
        "test-tool".to_string(),
        json!("test-input"),
        context.clone(),
        McpServiceContext {
            caller_id: service_context.caller_id.clone(),
            caller_type: service_context.caller_type,
            permission_level: service_context.permission_level,
            extra: service_context.extra,
        }
    );
    
    assert!(!request.id.is_empty(), "Request ID should not be empty");
    assert_eq!(request.component_type, ComponentType::Tool, "Component type should be Tool");
    assert_eq!(request.component_name, "test-tool", "Component name should match");
    assert_eq!(request.method, "run", "Method should be run");
    assert_eq!(request.params, json!("test-input"), "Params should match");
    assert_eq!(request.context.id, context.id, "Context ID should match");
}

#[tokio::test]
async fn test_mcp_response_creation() {
    // 创建 MCP 上下文
    let context = McpContext::new();
    
    // 创建服务上下文
    let service_context = ServiceContext {
        caller_id: "test-caller".to_string(),
        caller_type: CallerType::User,
        permission_level: PermissionLevel::Standard,
        extra: json!({ "test": "extra" }),
    };
    
    // 创建 MCP 请求
    let request = McpRequest::new_tool_call(
        "test-tool".to_string(),
        json!("test-input"),
        context.clone(),
        McpServiceContext {
            caller_id: service_context.caller_id.clone(),
            caller_type: service_context.caller_type,
            permission_level: service_context.permission_level,
            extra: service_context.extra,
        }
    );
    
    // 创建成功响应
    let response = McpResponse::success_from_request(
        &request,
        json!("test-output"),
        context.clone(),
        Some(100),
    );
    
    assert!(!response.id.is_empty(), "Response ID should not be empty");
    assert_eq!(response.request_id, request.id, "Response should reference the correct request");
    assert_eq!(response.status, ResponseStatus::Success, "Status should be Success");
    assert_eq!(response.data, Some(json!("test-output")), "Data should match");
    assert_eq!(response.error, None, "Error should be None for success response");
    assert_eq!(response.duration_ms, Some(100), "Duration should match");
    assert_eq!(response.context.id, context.id, "Context ID should match");
}

#[tokio::test]
async fn test_context_manager() {
    // 创建上下文管理器
    let manager = ContextManager::new();
    
    // 创建上下文
    let context = manager.create_context().await;
    let context_id = context.id.clone();
    
    // 获取上下文
    let retrieved = manager.get_context(&context_id).await.unwrap();
    assert_eq!(retrieved.id, context_id, "Retrieved context should match created context");
    
    // 创建子上下文
    let child = manager.create_child_context(&context_id).await.unwrap();
    assert_eq!(child.parent_id, Some(context_id), "Child context should have correct parent ID");
    
    // 删除上下文
    manager.delete_context(&context_id).await.unwrap();
    let result = manager.get_context(&context_id).await;
    assert!(result.is_err(), "Should not be able to retrieve deleted context");
}

#[tokio::test]
async fn test_mcp_node_from_base() {
    // 创建基础工作流节点
    let base_node = crate::workflow::WorkflowNode {
        id: "test-node".to_string(),
        tool_name: "test-tool".to_string(),
        label: Some("Test Node".to_string()),
        input_mappings: std::collections::HashMap::new(),
        static_inputs: json!({ "test": "value" }),
    };
    
    // 从基础节点创建 MCP 节点
    let mcp_node = McpNode::from_base(base_node);
    assert_eq!(mcp_node.base.id, "test-node", "Node ID should match");
    assert_eq!(mcp_node.base.tool_name, "test-tool", "Tool name should match");
    assert_eq!(mcp_node.mcp_config.mcp_supported, false, "MCP supported should be false by default");
    assert_eq!(mcp_node.context_mappings.len(), 0, "Context mappings should be empty by default");
    assert_eq!(mcp_node.output_context_updates.len(), 0, "Output context updates should be empty by default");
}

#[tokio::test]
async fn test_mcp_workflow_from_base() {
    // 创建基础工作流定义
    let base_workflow = crate::workflow::WorkflowDefinition {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        description: "Test workflow description".to_string(),
        nodes: vec![
            crate::workflow::WorkflowNode {
                id: "node1".to_string(),
                tool_name: "tool1".to_string(),
                label: None,
                input_mappings: std::collections::HashMap::new(),
                static_inputs: json!({}),
            }
        ],
        edges: vec![],
    };
    
    // 从基础工作流创建 MCP 工作流
    let mcp_workflow = McpWorkflow::from_base(base_workflow);
    assert_eq!(mcp_workflow.base.id, "test-workflow", "Workflow ID should match");
    assert_eq!(mcp_workflow.mcp_config.mcp_supported, false, "MCP supported should be false by default");
    assert_eq!(mcp_workflow.mcp_nodes.len(), 1, "MCP nodes should match base workflow nodes");
    assert_eq!(mcp_workflow.mcp_nodes[0].base.id, "node1", "MCP node ID should match");
}

#[tokio::test]
async fn test_mcp_workflow_set_mcp_supported() {
    // 创建基础工作流定义
    let base_workflow = crate::workflow::WorkflowDefinition {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        description: "Test workflow description".to_string(),
        nodes: vec![
            crate::workflow::WorkflowNode {
                id: "node1".to_string(),
                tool_name: "tool1".to_string(),
                label: None,
                input_mappings: std::collections::HashMap::new(),
                static_inputs: json!({}),
            }
        ],
        edges: vec![],
    };
    
    // 从基础工作流创建 MCP 工作流并设置为支持 MCP
    let mcp_workflow = McpWorkflow::from_base(base_workflow).set_mcp_supported(true);
    assert_eq!(mcp_workflow.mcp_config.mcp_supported, true, "MCP supported should be true");
    assert_eq!(mcp_workflow.mcp_nodes[0].mcp_config.mcp_supported, true, "All MCP nodes should be MCP supported");
}

// 测试工具
struct TestMcpTool {
    name: String,
    mcp_supported: bool,
    return_value: serde_json::Value,
}

#[async_trait::async_trait]
impl crate::Tool for TestMcpTool {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self, _locale: crate::locale::Locale) -> String {
        format!("Test MCP tool: {}", self.name)
    }
    
    fn user_guide(&self, _locale: crate::locale::Locale) -> String {
        "Test MCP tool guide".to_string()
    }
    
    fn input_schema(&self, _locale: crate::locale::Locale) -> serde_json::Value {
        serde_json::json!({ "type": "object" })
    }
    
    async fn run(&self, _input: serde_json::Value) -> crate::error::Result<serde_json::Value> {
        Ok(self.return_value.clone())
    }
    
    fn mcp_supported(&self) -> bool {
        self.mcp_supported
    }
    
    async fn run_with_context(&self, request: McpRequest) -> crate::error::Result<McpResponse> {
        // 自定义实现：返回请求中的上下文 ID
        let context_id = request.context.id.clone();
        let response_data = json!({ 
            "tool_result": self.return_value.clone(),
            "context_id": context_id 
        });
        
        Ok(McpResponse::success_from_request(
            &request,
            response_data,
            request.context.clone(),
            Some(123),
        ))
    }
}

#[tokio::test]
async fn test_mcp_tool_execution() {
    // 创建测试 MCP 工具
    let test_tool = TestMcpTool {
        name: "test-mcp-tool".to_string(),
        mcp_supported: true,
        return_value: json!("test-result"),
    };
    
    // 创建 MCP 上下文
    let context = McpContext::new();
    let context_id = context.id.clone();
    
    // 创建服务上下文
    let service_context = McpServiceContext {
        caller_id: "test-caller".to_string(),
        caller_type: crate::service::CallerType::User,
        permission_level: crate::service::PermissionLevel::Standard,
        extra: json!({}),
    };
    
    // 创建 MCP 请求
    let request = McpRequest::new_tool_call(
        "test-mcp-tool".to_string(),
        json!("test-input"),
        context.clone(),
        service_context
    );
    
    // 执行工具
    let response = test_tool.run_with_context(request).await.unwrap();
    
    // 验证结果
    assert_eq!(response.status, ResponseStatus::Success, "Status should be Success");
    assert_eq!(response.duration_ms, Some(123), "Duration should be 123");
    assert_eq!(response.context.id, context_id, "Context ID should match");
    
    // 验证返回的数据包含上下文 ID
    let data = response.data.unwrap();
    assert_eq!(data["tool_result"], json!("test-result"), "Tool result should match");
    assert_eq!(data["context_id"], context_id, "Context ID should be returned in result");
}

#[tokio::test]
async fn test_plugin_manager_mcp_tools() {
    // 创建临时目录
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();
    
    // 创建插件管理器
    let manager = crate::plugin::PluginManager::new(temp_path.to_path_buf());
    
    // 创建测试工具
    let test_tool1 = TestMcpTool {
        name: "mcp-tool-1".to_string(),
        mcp_supported: true,
        return_value: json!("result1"),
    };
    
    let test_tool2 = TestMcpTool {
        name: "non-mcp-tool".to_string(),
        mcp_supported: false,
        return_value: json!("result2"),
    };
    
    // 手动添加工具到插件管理器
    { 
        let mut plugins = manager.plugins.write().await;
        plugins.insert(test_tool1.name().to_string(), Arc::from(test_tool1));
        plugins.insert(test_tool2.name().to_string(), Arc::from(test_tool2));
    }
    
    // 测试获取所有工具
    let all_tools = manager.list_tools().await;
    assert_eq!(all_tools.len(), 2, "Should have 2 tools");
    
    // 测试获取支持 MCP 的工具
    let mcp_tools = manager.list_mcp_tools().await;
    assert_eq!(mcp_tools.len(), 1, "Should have 1 MCP tool");
    assert_eq!(mcp_tools[0].name(), "mcp-tool-1", "MCP tool name should match");
    
    // 测试获取单个 MCP 工具
    let mcp_tool = manager.get_mcp_tool("mcp-tool-1").await;
    assert!(mcp_tool.is_some(), "Should be able to get MCP tool");
    assert_eq!(mcp_tool.unwrap().name(), "mcp-tool-1", "MCP tool name should match");
    
    // 测试获取非 MCP 工具作为 MCP 工具（应该失败）
    let non_mcp_tool = manager.get_mcp_tool("non-mcp-tool").await;
    assert!(non_mcp_tool.is_none(), "Should not be able to get non-MCP tool as MCP tool");
}
