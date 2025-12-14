use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use serde_json::Value;
use crate::{Tool, Result, CoreError};
use crate::mcp::{McpRequest, McpResponse, ContextManager};
use crate::plugin::PluginManager;
use futures_util::{StreamExt, SinkExt};
use warp::ws::Message;

/// MCP 服务器配置
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// 服务器地址
    pub address: String,
    /// 服务器端口
    pub port: u16,
    /// 是否启用 WebSocket
    pub enable_websocket: bool,
    /// 是否启用 REST API
    pub enable_rest: bool,
    /// 最大请求大小（字节）
    pub max_request_size: usize,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 8080,
            enable_websocket: true,
            enable_rest: true,
            max_request_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// MCP 服务器，处理 MCP 协议请求
pub struct McpServer {
    /// 配置
    config: McpServerConfig,
    /// 工具集合
    tools: Arc<RwLock<std::collections::HashMap<String, Arc<dyn Tool>>>>,
    /// 插件管理器
    plugin_manager: Arc<PluginManager>,
    /// 上下文管理器
    context_manager: Arc<ContextManager>,
}

// 手动实现 Debug trait，避免 dyn Tool 没有 Debug 实现的问题
impl std::fmt::Debug for McpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 对于异步 RwLock，我们不能直接在 Debug 中获取读锁，所以使用占位值
        f.debug_struct("McpServer")
            .field("config", &self.config)
            .field("tools_count", &"<async>")
            .field("plugin_manager", &self.plugin_manager)
            .field("context_manager", &self.context_manager)
            .finish()
    }
}

impl McpServer {
    /// 创建一个新的 MCP 服务器
    pub fn new(
        config: McpServerConfig,
        tools: std::collections::HashMap<String, Arc<dyn Tool>>,
        plugin_manager: Arc<PluginManager>,
    ) -> Self {
        Self {
            config,
            tools: Arc::new(RwLock::new(tools)),
            plugin_manager,
            context_manager: Arc::new(ContextManager::new()),
        }
    }
    
    /// 启动 MCP 服务器
    pub async fn start(&self) -> Result<()> {
        // 这里我们只返回 Result<()>，让外部处理错误
        let api_filters = self.build_api_filters();
        let ws_filters = self.build_websocket_filters();
        
        let routes = api_filters.or(ws_filters);
        
        use std::net::SocketAddr;
        
        let address = format!("{}:{}", self.config.address, self.config.port);
        println!("MCP Server starting on {}", address);
        
        let socket_addr: SocketAddr = address.parse().map_err(|e| CoreError::ConfigError(format!("Invalid address: {}", e)))?;
        
        warp::serve(routes)
            .run(socket_addr)
            .await;
        
        Ok(())
    }
    
    /// 构建 API 过滤器
    fn build_api_filters(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        // 健康检查端点
        let health = warp::path!("health")
            .and(warp::get())
            .map(|| {
                warp::reply::json(&serde_json::json!({ "status": "ok", "service": "mcp-server" }))
            });
        
        // 获取工具列表
        let list_tools_tools = self.tools.clone();
        let list_tools_plugin_manager = self.plugin_manager.clone();
        let list_tools = warp::path!("tools")
            .and(warp::get())
            .and(warp::any().map(move || list_tools_tools.clone()))
            .and(warp::any().map(move || list_tools_plugin_manager.clone()))
            .and_then(Self::handle_list_tools);
        
        // 获取 MCP 支持的工具列表
        let list_mcp_tools_tools = self.tools.clone();
        let list_mcp_tools_plugin_manager = self.plugin_manager.clone();
        let list_mcp_tools = warp::path!("tools" / "mcp")
            .and(warp::get())
            .and(warp::any().map(move || list_mcp_tools_tools.clone()))
            .and(warp::any().map(move || list_mcp_tools_plugin_manager.clone()))
            .and_then(Self::handle_list_mcp_tools);
        
        // 调用工具
        let call_tool_tools = self.tools.clone();
        let call_tool_plugin_manager = self.plugin_manager.clone();
        let call_tool_context_manager = self.context_manager.clone();
        let call_tool = warp::path!("tools" / String / "call")
            .and(warp::post())
            .and(warp::body::content_length_limit(self.config.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::any().map(move || call_tool_tools.clone()))
            .and(warp::any().map(move || call_tool_plugin_manager.clone()))
            .and(warp::any().map(move || call_tool_context_manager.clone()))
            .and_then(Self::handle_call_tool);
        
        // MCP 调用端点
        let mcp_call_tools = self.tools.clone();
        let mcp_call_plugin_manager = self.plugin_manager.clone();
        let mcp_call_context_manager = self.context_manager.clone();
        let mcp_call = warp::path!("mcp" / "call")
            .and(warp::post())
            .and(warp::body::content_length_limit(self.config.max_request_size as u64))
            .and(warp::body::json())
            .and(warp::any().map(move || mcp_call_tools.clone()))
            .and(warp::any().map(move || mcp_call_plugin_manager.clone()))
            .and(warp::any().map(move || mcp_call_context_manager.clone()))
            .and_then(Self::handle_mcp_call);
        
        // 根路径
        let root = warp::path::end()
            .map(|| {
                warp::reply::json(&serde_json::json!({ "service": "mcp-server", "version": "1.0.0" }))
            });
        
        // 组合所有路由
        health.or(list_tools).or(list_mcp_tools).or(call_tool).or(mcp_call).or(root)
            .with(warp::cors().allow_any_origin())
    }
    
    /// 构建 WebSocket 过滤器
    fn build_websocket_filters(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let tools = self.tools.clone();
        let plugin_manager = self.plugin_manager.clone();
        let context_manager = self.context_manager.clone();
        
        // WebSocket 端点
        let ws = warp::path!("ws" / "mcp")
            .and(warp::ws())
            .and(warp::any().map(move || tools.clone()))
            .and(warp::any().map(move || plugin_manager.clone()))
            .and(warp::any().map(move || context_manager.clone()))
            .map(|ws: warp::ws::Ws, tools, plugin_manager, context_manager| {
                ws.on_upgrade(move |socket| {
                    Self::handle_websocket_connection(socket, tools, plugin_manager, context_manager)
                })
            });
        
        ws
    }
    
    /// 处理工具列表请求
    async fn handle_list_tools(
        tools: Arc<RwLock<std::collections::HashMap<String, Arc<dyn Tool>>>>,
        plugin_manager: Arc<PluginManager>,
    ) -> std::result::Result<impl warp::Reply, warp::Rejection> {
        let mut tool_list = Vec::new();
        
        // 获取核心工具
        let core_tools = tools.read().await;
        for (name, tool) in core_tools.iter() {
            tool_list.push(serde_json::json!({ 
                "name": name,
                "display_name": tool.display_name(crate::Locale::En),
                "description": tool.description(crate::Locale::En),
                "mcp_supported": tool.mcp_supported(),
                "type": "core"
            }));
        }
        drop(core_tools);
        
        // 获取插件工具
        let plugin_tools = plugin_manager.list_tools().await;
        for tool in plugin_tools {
            tool_list.push(serde_json::json!({ 
                "name": tool.name(),
                "display_name": tool.display_name(crate::Locale::En),
                "description": tool.description(crate::Locale::En),
                "mcp_supported": tool.mcp_supported(),
                "type": "plugin"
            }));
        }
        
        Ok(warp::reply::json(&tool_list))
    }
    
    /// 处理 MCP 工具列表请求
    async fn handle_list_mcp_tools(
        tools: Arc<RwLock<std::collections::HashMap<String, Arc<dyn Tool>>>>,
        plugin_manager: Arc<PluginManager>,
    ) -> std::result::Result<impl warp::Reply, warp::Rejection> {
        let mut tool_list = Vec::new();
        
        // 获取核心 MCP 工具
        let core_tools = tools.read().await;
        for (name, tool) in core_tools.iter() {
            if tool.mcp_supported() {
                tool_list.push(serde_json::json!({ 
                    "name": name,
                    "display_name": tool.display_name(crate::Locale::En),
                    "description": tool.description(crate::Locale::En),
                    "mcp_supported": true,
                    "type": "core"
                }));
            }
        }
        drop(core_tools);
        
        // 获取插件 MCP 工具
        let plugin_tools = plugin_manager.list_mcp_tools().await;
        for tool in plugin_tools {
            tool_list.push(serde_json::json!({ 
                "name": tool.name(),
                "display_name": tool.display_name(crate::Locale::En),
                "description": tool.description(crate::Locale::En),
                "mcp_supported": true,
                "type": "plugin"
            }));
        }
        
        Ok(warp::reply::json(&tool_list))
    }
    
    /// 处理工具调用请求
    async fn handle_call_tool(
        tool_name: String,
        input: Value,
        tools: Arc<RwLock<std::collections::HashMap<String, Arc<dyn Tool>>>>,
        plugin_manager: Arc<PluginManager>,
        _context_manager: Arc<ContextManager>,
    ) -> std::result::Result<impl warp::Reply, warp::Rejection> {
        // 查找工具
        let tool = {
            let core_tools = tools.read().await;
            if let Some(tool) = core_tools.get(&tool_name) {
                Some(tool.clone())
            } else {
                drop(core_tools);
                plugin_manager.get_tool(&tool_name).await
            }
        };
        
        if let Some(tool) = tool {
            // 执行工具
            let result = tool.run(input).await;
            match result {
                Ok(output) => Ok(warp::reply::json(&serde_json::json!({ "success": true, "data": output }))),
                Err(e) => Ok(warp::reply::json(&serde_json::json!({ "success": false, "error": e.to_string() }))),
            }
        } else {
            Ok(warp::reply::json(&serde_json::json!({ "success": false, "error": format!("Tool not found: {}", tool_name) })))
        }
    }
    
    /// 处理 MCP 调用请求
    async fn handle_mcp_call(
        request: McpRequest,
        tools: Arc<RwLock<std::collections::HashMap<String, Arc<dyn Tool>>>>,
        plugin_manager: Arc<PluginManager>,
        _context_manager: Arc<ContextManager>,
    ) -> std::result::Result<impl warp::Reply, warp::Rejection> {
        // 先克隆 request，避免部分移动问题
        let request_copy = request.clone();
        let component_name = request.component_name;
        let context = request.context;
        
        // 查找工具
        let tool = {
            let core_tools = tools.read().await;
            if let Some(tool) = core_tools.get(&component_name) {
                Some(tool.clone())
            } else {
                drop(core_tools);
                plugin_manager.get_tool(&component_name).await
            }
        };
        
        if let Some(tool) = tool {
            if !tool.mcp_supported() {
                let response = McpResponse::failure_from_request(
                    &request_copy,
                    format!("Tool {} does not support MCP", component_name),
                    Some("MCP_NOT_SUPPORTED".to_string()),
                    context.clone(),
                    None,
                );
                return Ok(warp::reply::json(&response));
            }
            
            // 执行 MCP 工具调用
            let result = tool.run_with_context(request_copy.clone()).await;
            match result {
                Ok(response) => Ok(warp::reply::json(&response)),
                Err(e) => {
                    let response = McpResponse::failure_from_request(
                        &request_copy,
                        e.to_string(),
                        Some("EXECUTION_FAILED".to_string()),
                        context.clone(),
                        None,
                    );
                    Ok(warp::reply::json(&response))
                },
            }
        } else {
            let response = McpResponse::failure_from_request(
                &request_copy,
                format!("Tool not found: {}", component_name),
                Some("COMPONENT_NOT_FOUND".to_string()),
                context,
                None,
            );
            Ok(warp::reply::json(&response))
        }
    }
    
    /// 处理 WebSocket 连接
    async fn handle_websocket_connection(
        socket: warp::ws::WebSocket,
        _tools: Arc<RwLock<std::collections::HashMap<String, Arc<dyn Tool>>>>,
        _plugin_manager: Arc<PluginManager>,
        _context_manager: Arc<ContextManager>,
    ) {
        // WebSocket 处理逻辑
        // 目前仅作为占位符，实际实现将在后续添加
        println!("WebSocket connection established");
        let (mut tx, mut rx) = socket.split::<Message>();
        
        // 简单的回显逻辑
        while let Some(result) = rx.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        println!("WebSocket message: {}", text);
                        if let Err(e) = tx.send(Message::text(format!("Echo: {}", text))).await {
                            eprintln!("WebSocket send error: {}", e);
                            break;
                        }
                    }
                },
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                },
            }
        }
        
        println!("WebSocket connection closed");
    }
}
