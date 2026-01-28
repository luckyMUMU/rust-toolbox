# Phase 1.2: McpServer 核心服务 - 详细设计与执行计划

## 1. 现状分析

### 1.1 已完成的 MCP 协议依赖
- `rmcp` 0.1.5 已添加到 Cargo.toml
- `schemars` 0.8.22 已添加
- feature flag `mcp` 已配置

### 1.2 遇到的问题
`rmcp` 0.1.5 的 API 与预期有显著差异:
- 模块路径不匹配
- Trait 方法签名复杂
- 类型系统要求严格

## 2. 技术选型决策

### 方案对比

| 方案 | 优点 | 缺点 | 推荐度 |
|------|------|------|--------|
| **A. 升级 rmcp 到 0.14.0** | 可能修复 API 问题 | API 可能有 Breaking Changes | ⭐⭐⭐ |
| **B. 使用 mcp-protocol-server** | 功能完整 | 依赖较重 | ⭐⭐ |
| **C. 自研 MCP 协议实现** | 完全可控 | 工作量大 | ⭐ |
| **D. 使用官方 SDK** | 官方支持 | 可能没有 Rust SDK | ⭐⭐ |

### 推荐方案: A (升级 rmcp)

理由:
1. 0.1.5 到 0.14.0 跨越多个版本，API 已稳定
2. 保持技术栈一致性
3. 社区活跃度较高

## 3. 详细设计

### 3.1 升级 rmcp 到 0.14.0

```toml
# Cargo.toml 修改
[dependencies]
rmcp = { version = "0.14", features = ["server", "macros"], optional = true }
```

### 3.2 核心架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    WorkflowMcpServer                         │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ ToolRegistry │  │   Server     │  │ ResourceManager  │  │
│  │   Adapter    │  │  (rmcp)      │  │                  │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  list_tools  │  │  call_tool   │  │ read_resource    │  │
│  │              │  │              │  │                  │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 核心组件设计

#### 3.3.1 WorkflowMcpServer

```rust
#[derive(Clone)]
pub struct WorkflowMcpServer {
    config: McpServerConfig,
    tool_registry: Arc<dyn ToolRegistry>,
    resource_manager: Arc<ResourceManager>,
}

#[async_trait]
impl ServerHandler for WorkflowMcpServer {
    // 实现 MCP 协议方法
}
```

#### 3.3.2 ToolRegistry Adapter

```rust
/// 将内部 ToolRegistry 适配为 MCP Tool 格式
pub struct ToolRegistryAdapter {
    registry: Arc<dyn ToolRegistry>,
}

impl ToolRegistryAdapter {
    /// 列出所有可用工具
    pub fn list_tools(&self) -> Vec<Tool> {
        // 转换 ToolInfo -> MCP Tool
    }
    
    /// 执行工具
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value> {
        // 查找并执行工具
    }
}
```

#### 3.3.3 ResourceManager

```rust
/// 管理 MCP Resources
pub struct ResourceManager {
    /// 上下文数据存储
    contexts: DashMap<String, ExecutionContext>,
    /// 日志存储
    logs: DashMap<String, Vec<String>>,
}

impl ResourceManager {
    /// 注册执行上下文
    pub fn register_context(&self, trace_id: String, context: ExecutionContext);
    
    /// 读取资源
    pub fn read_resource(&self, uri: &str) -> Option<String>;
}
```

### 3.4 传输层设计

支持两种传输方式:

1. **StdioTransport**: 用于本地进程通信
2. **SseTransport**: 用于 HTTP 服务器推送

```rust
pub enum McpTransport {
    Stdio(StdioTransport),
    Sse(SseTransport),
}

impl McpTransport {
    pub async fn start<H: ServerHandler>(self, handler: H) -> Result<()>;
}
```

## 4. 执行计划

### 任务 1: 升级 rmcp 依赖 (预计 30 分钟)

**目标**: 将 rmcp 从 0.1.5 升级到 0.14.0

**步骤**:
1. 修改 Cargo.toml
2. 运行 cargo check 验证依赖
3. 解决可能的依赖冲突

**验收标准**:
- [ ] `cargo check --features mcp` 通过
- [ ] 无依赖冲突

### 任务 2: 调研 rmcp 0.14.0 API (预计 1 小时)

**目标**: 理解新版本 API 结构

**步骤**:
1. 阅读 rmcp 0.14.0 文档
2. 查看官方示例代码
3. 记录关键 API 变化

**关键调研点**:
- [ ] `ServerHandler` trait 定义
- [ ] `Server` 创建方式
- [ ] Transport 配置方法
- [ ] Tool/Resource 类型定义

### 任务 3: 实现核心 ServerHandler (预计 2 小时)

**目标**: 实现 `ServerHandler` trait

**步骤**:
1. 定义 `WorkflowMcpServer` 结构体
2. 实现 `ServerHandler` trait
3. 实现工具发现和执行方法

**代码结构**:
```rust
// src/interfaces/mcp_server.rs

use rmcp::{
    handler::server::ServerHandler,
    model::*,
    // ... 其他导入
};

#[derive(Clone)]
pub struct WorkflowMcpServer {
    // ... 字段
}

#[async_trait]
impl ServerHandler for WorkflowMcpServer {
    async fn list_tools(&self, ...) -> Result<ListToolsResult>;
    async fn call_tool(&self, ...) -> Result<CallToolResult>;
    async fn list_resources(&self, ...) -> Result<ListResourcesResult>;
    async fn read_resource(&self, ...) -> Result<ReadResourceResult>;
}
```

**验收标准**:
- [ ] 结构体正确定义
- [ ] Trait 方法实现完整
- [ ] 编译通过

### 任务 4: 实现 ToolRegistry 适配器 (预计 1.5 小时)

**目标**: 桥接内部 ToolRegistry 和 MCP Tool 格式

**步骤**:
1. 创建 `ToolRegistryAdapter`
2. 实现 ToolInfo -> MCP Tool 转换
3. 实现工具执行逻辑

**关键代码**:
```rust
impl ToolRegistryAdapter {
    pub fn to_mcp_tool(&self, tool_info: &ToolInfo) -> Tool {
        Tool::new(
            &tool_info.name,
            &tool_info.description,
            tool_info.parameters_schema.clone(),
        )
    }
}
```

**验收标准**:
- [ ] 工具列表正确转换
- [ ] 工具执行正常

### 任务 5: 实现 ResourceManager (预计 1 小时)

**目标**: 支持 Resources 协议

**步骤**:
1. 定义 `ResourceManager` 结构体
2. 实现 URI 解析
3. 实现资源读取

**URI 规范**:
- `flow://{trace_id}/context` - 执行上下文
- `flow://{trace_id}/logs` - 执行日志
- `flow://{trace_id}/result` - 执行结果

**验收标准**:
- [ ] URI 解析正确
- [ ] 资源读取正常

### 任务 6: 实现传输层启动 (预计 1 小时)

**目标**: 支持 stdio 和 SSE 传输

**步骤**:
1. 实现 stdio 传输启动
2. (可选) 实现 SSE 传输
3. 添加配置支持

**代码示例**:
```rust
impl WorkflowMcpServer {
    pub async fn start_stdio(self) -> Result<()> {
        let transport = StdioTransport::new();
        let server = Server::new(transport, self).await?;
        server.waiting().await?;
        Ok(())
    }
}
```

**验收标准**:
- [ ] stdio 传输正常
- [ ] 服务器可启动和停止

### 任务 7: 集成测试 (预计 1 小时)

**目标**: 验证 MCP Server 功能

**测试用例**:
1. 工具列表获取
2. 工具执行
3. 资源读取
4. 错误处理

**验收标准**:
- [ ] 所有测试用例通过
- [ ] 代码覆盖率 > 80%

## 5. 时间表

| 任务 | 预计时间 | 依赖 |
|------|----------|------|
| 1. 升级 rmcp | 30 分钟 | 无 |
| 2. 调研 API | 1 小时 | 任务 1 |
| 3. 实现 ServerHandler | 2 小时 | 任务 2 |
| 4. ToolRegistry 适配器 | 1.5 小时 | 任务 3 |
| 5. ResourceManager | 1 小时 | 任务 3 |
| 6. 传输层启动 | 1 小时 | 任务 3-5 |
| 7. 集成测试 | 1 小时 | 任务 6 |
| **总计** | **8 小时** | - |

## 6. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| rmcp 0.14.0 API 仍不稳定 | 高 | 准备回退到 0.1.x 或换库 |
| ToolNode trait 方法缺失 | 中 | 添加必要的方法到 trait |
| 类型转换复杂 | 低 | 编写转换辅助函数 |
| 生命周期问题 | 中 | 使用 Arc 和 Clone 规避 |

## 7. 下一步行动

1. **立即执行**: 任务 1 (升级 rmcp)
2. **并行准备**: 调研 MCP 协议规范
3. **后续跟进**: 按顺序执行任务 2-7

---

*本计划应根据实际开发进度调整*
