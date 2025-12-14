# Workflow Engine Design Document

## 1. 系统架构设计 (System Architecture)

基于 Tokio 异步运行时的轻量级工作流引擎，旨在实现工具的编排与自动化执行。

### 1.1 架构图 (Architecture Diagram)

```mermaid
graph TD
    subgraph Frontend [前端交互层]
        CLI[rt-cli]
        GUI[rt-gui]
    end

    subgraph Core [rt-core]
        Engine[Workflow Engine]
        State[Execution State Manager]
        Scheduler[Task Scheduler (Tokio)]
        Context[Data Context]
    end

    subgraph Tools [工具层]
        Registry[Tool Registry]
        Native[Native Tools]
        Plugin[Plugin Adapter]
    end

    CLI -->|Command| Engine
    GUI -->|Event| Engine
    Engine -->|Schedule| Scheduler
    Engine -->|Query| Registry
    Scheduler -->|Execute| Native
    Scheduler -->|Execute| Plugin
    Native -->|Result| Context
    Plugin -->|Result| Context
    Context -->|Data Map| Scheduler
    State -->|Monitor| GUI
```

### 1.2 核心模块
1.  **Workflow Engine**: 负责解析工作流定义，管理生命周期（启动、暂停、停止）。
2.  **Task Scheduler**: 基于 `Tokio` 的并发调度器，负责依赖分析和任务派发。
3.  **Data Context**: 负责节点间的数据传递，支持 JSON Path 变量替换。
4.  **Execution State Manager**: 维护运行时状态，提供监控接口。

---

## 2. 关键数据结构 (Data Structures)

### 2.1 工作流定义 (Workflow Definition)
```rust
/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

/// 工作流节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub tool_name: String, // 引用已注册的工具 (e.g., "file.move_folder")
    pub label: Option<String>,
    pub input_mappings: HashMap<String, String>, // 输入字段 -> 表达式 (e.g., "{{ node1.output.path }}")
    pub static_inputs: Value, // 静态配置值
}

/// 节点连接 (依赖关系)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub from: String, // Node ID
    pub to: String,   // Node ID
}
```

### 2.2 运行时状态 (Runtime State)
```rust
/// 执行实例
pub struct WorkflowInstance {
    pub id: String, // UUID
    pub def: WorkflowDefinition,
    pub status: WorkflowStatus,
    pub node_states: HashMap<String, NodeExecutionState>,
    pub context: HashMap<String, Value>, // 存储所有节点的输出
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct NodeExecutionState {
    pub status: NodeStatus,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}
```

---

## 3. 接口规范 (Interface Specifications)

### 3.1 引擎接口 (Engine API)
```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 验证工作流定义
    fn validate(&self, def: &WorkflowDefinition) -> Result<(), CoreError>;

    /// 启动工作流
    async fn start_workflow(&self, def: WorkflowDefinition) -> Result<String>;

    /// 获取工作流状态
    async fn get_status(&self, instance_id: &str) -> Result<WorkflowInstance>;

    /// 暂停/停止
    async fn pause_workflow(&self, instance_id: &str) -> Result<()>;
    async fn stop_workflow(&self, instance_id: &str) -> Result<()>;
    
    /// 获取执行日志
    async fn get_logs(&self, instance_id: &str) -> Result<Vec<LogEntry>>;
}
```

### 3.2 数据传递规范
- **引用语法**: `{{ node_id.output.json_path }}`
- **解析逻辑**: 
  1. 在节点执行前，引擎解析 `input_mappings`。
  2. 从 `context` 中查找对应 `node_id` 的 `output`。
  3. 使用 JSON Path 提取值。
  4. 将提取值注入到工具的 `input` 中。

---

## 4. 插件开发指南草案 (Plugin Development Guide)

### 4.1 插件集成原理
工作流引擎通过 `rt-core::Tool` trait 与工具交互。插件只需符合标准插件协议（JSON Input/Output），即可被工作流引擎调用，**无需任何额外修改**。

### 4.2 MCP 支持
工作流引擎支持 Model Context Protocol (MCP)，允许插件与工具通过标准化协议交互。插件只需实现 `rt-core::Tool` trait 并遵循标准协议，即可无缝集成到工作流中。

### 4.3 最佳实践
1.  **原子性**: 插件应只做一件事，便于在工作流中组合。
2.  **结构化输出**: 输出必须是扁平或层级清晰的 JSON，便于后续节点通过 JSON Path 引用。
3.  **错误处理**: 插件失败应返回非零退出码，引擎会自动捕获并标记节点失败。
4.  **MCP 兼容性**: 对于支持 MCP 的插件，确保输出格式符合 MCP 规范，便于上下文传递。

### 4.3 示例
假设有一个 `http.get` 插件：
```json
// Output
{
  "status": 200,
  "body": { "user_id": 123 }
}
```
后续节点可引用: `{{ http_node.output.body.user_id }}`

---

## 5. 界面原型设计 (UI Prototype)

### 5.1 CLI 界面
```bash
# 运行工作流
$ rt-cli workflow run ./my_flow.json
[INFO] Workflow 'Backup & Convert' started (ID: 550e8400...)
[INFO] Step 1: 'Check File' (file.exists) ... DONE (5ms)
[INFO] Step 2: 'Backup' (file.copy) ... RUNNING
[INFO] Step 2: 'Backup' (file.copy) ... DONE (120ms)
[INFO] Workflow Completed Successfully.

# 查看状态
$ rt-cli workflow status 550e8400...
Status: Completed
Nodes:
  - Check File: Success
  - Backup: Success
```

### 5.2 GUI 界面 (rt-gui)
**多标签页设计**:
- **Tab 1: Workflow Designer (画布)**
  - **左侧**: 工具箱 (Toolbox)，列出所有可用工具 (Native & Plugins)。
  - **中间**: 无限画布。拖拽工具创建节点，连线定义依赖。
  - **右侧**: 属性面板 (Properties)。配置选中节点的 `static_inputs` 和 `input_mappings`。
  - **顶部**: 工具栏 (Run, Save, Load)。

- **Tab 2: Execution Monitor (监控)**
  - **左侧**: 历史记录列表。
  - **中间**: 运行时 DAG 可视化。
    - 灰色: Pending
    - 蓝色: Running (带动画)
    - 绿色: Success
    - 红色: Failed
  - **底部**: 实时日志控制台。
  - **点击节点**: 弹出侧边栏显示该节点的详细 Input/Output JSON。

### 5.3 扩展性设计
- **API 预留**: `WorkflowEngine` trait 设计为异步，未来可替换为 gRPC 客户端以支持分布式执行。
- **状态存储**: 目前内存存储，未来可扩展为 SQLite/Redis 持久化。
