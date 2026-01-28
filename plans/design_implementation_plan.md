# Workflow Toolkit 设计实现计划

> **目标**: 将当前实现与设计文档对齐，完成工业级数字骨架转型
> **预计周期**: 5 个阶段
> **最后更新**: 2026-01-28

---

## 执行摘要

当前代码库实现度约 **70%**，核心引擎和基础架构已基本完成。本计划聚焦于实现 MCP Server、EL 表达式编排、工具原子化、WASM 插件和 LanceDB 深度集成五大关键模块。

---

## Phase 1: MCP Server 完整实现 (高优先级)

### 目标
实现完整的 Model Context Protocol 服务器，使 AI 助手 (Cursor/Claude) 能够直接调用编排好的复杂工具。

### 任务清单

#### 1.1 添加 MCP 协议依赖和基础架构
- **文件**: `Cargo.toml`
- **工作**:
  - 添加 `mcp-protocol-server` 或 `rmcp` 依赖
  - 添加 `jsonrpc-core` 和 `jsonrpc-http-server` 依赖
  - 更新 feature flags
- **验收标准**: 项目能够编译通过，依赖无冲突

#### 1.2 实现 McpServer 核心服务
- **文件**: `src/interfaces/mcp_server.rs` (新建)
- **工作**:
  - 实现 `McpServer` struct
  - 实现 `ServerHandler` trait
  - 支持 JSON-RPC 2.0 协议
  - 实现生命周期管理 (initialize/initialized/shutdown)
- **核心代码结构**:
```rust
pub struct McpServer {
    tool_registry: Arc<dyn ToolRegistry>,
    workflow_engine: Arc<RefactoredWorkflowEngine>,
    config: McpServerConfig,
}

#[async_trait]
impl ServerHandler for McpServer {
    async fn list_tools(&self) -> Result<Vec<Tool>>;
    async fn call_tool(&self, name: &str, args: Value) -> Result<Vec<Content>>;
    async fn list_resources(&self) -> Result<Vec<Resource>>;
    async fn read_resource(&self, uri: &str) -> Result<String>;
}
```

#### 1.3 实现 Tool 自动注册机制
- **文件**: `src/interfaces/mcp/tool_registry.rs` (新建)
- **工作**:
  - 实现 `Tool` 到 MCP `Tool` 的自动转换
  - 从 `ToolInfo` 自动生成 `inputSchema`
  - 支持动态工具发现
- **验收标准**: 新工具注册后自动暴露给 MCP

#### 1.4 实现 Resources URI 路由
- **文件**: `src/interfaces/mcp/resources.rs` (新建)
- **工作**:
  - 实现 URI 解析器 (`flow://{trace_id}/data/{key}`)
  - 实现资源读取处理器
  - 支持上下文数据、执行日志、审计记录
- **URI 规范**:
  - `flow://{trace_id}/context/{key}` - 上下文数据
  - `flow://{trace_id}/logs` - 执行日志
  - `flow://{trace_id}/audit` - 审计记录

#### 1.5 添加 MCP 配置和启动集成
- **文件**: 
  - `src/config.rs` (更新)
  - `src/interfaces/cli/commands.rs` (更新)
- **工作**:
  - 添加 MCP 服务器配置段
  - 添加 `mcp` 子命令 (`workflow-toolkit mcp start`)
  - 集成到 CliApp
- **配置示例**:
```toml
[mcp]
enabled = true
http_port = 3001
ws_port = 3002
cors_origins = ["*"]
```

### 依赖项
- `rmcp` 或 `mcp-protocol-server` crate
- `jsonrpc-core` 18.0
- `jsonrpc-http-server` 18.0

### 验收标准
- [ ] MCP Server 可以独立启动
- [ ] AI 助手可以通过 MCP 协议发现工具
- [ ] AI 助手可以调用工具并获取结果
- [ ] Resources URI 可以访问上下文数据

---

## Phase 2: EL 表达式编排引擎 (高优先级)

### 目标
实现声明式编排语言 (EL)，支持 THEN/WHEN/SWITCH/FINALLY 等语义，让业务逻辑通过 YAML 定义而非硬编码。

### 任务清单

#### 2.1 设计 EL 表达式语法规范
- **文件**: `docs/el_spec.md` (新建)
- **工作**:
  - 定义完整 EL 语法
  - 定义表达式求值规则
  - 设计错误处理机制
- **语法草案**:
```yaml
# THEN - 串行
THEN:
  - node: scanner
  - node: processor

# WHEN - 并行
WHEN:
  - node: ocr
  - node: metadata

# SWITCH - 条件分支
SWITCH:
  on: "${context.confidence > 0.8}"
  cases:
    - value: true
      node: archiver
    - value: false
      node: reviewer

# FINALLY - 兜底
FINALLY:
  node: cleanup
```

#### 2.2 实现表达式解析器
- **文件**: `src/workflow/el/` (新建目录)
  - `src/workflow/el/parser.rs`
  - `src/workflow/el/lexer.rs`
  - `src/workflow/el/ast.rs`
- **工作**:
  - 实现 `${context.xxx}` 变量引用
  - 实现 `${env.XXX}` 环境变量
  - 实现 `${input.xxx}` 输入参数
- **核心结构**:
```rust
pub enum Expr {
    Literal(Value),
    Variable(String),           // ${context.xxx}
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),    // 函数调用
}
```

#### 2.3 实现条件表达式支持
- **文件**: `src/workflow/el/evaluator.rs`
- **工作**:
  - 实现比较运算符 (`>`, `<`, `>=`, `<=`, `==`, `!=`)
  - 实现逻辑运算符 (`&&`, `||`, `!`)
  - 实现算术运算符 (`+`, `-`, `*`, `/`)
  - 实现类型转换
- **示例**:
```yaml
on: "${context.count > 10 && context.type == 'pdf'}"
```

#### 2.4 实现 YAML/JSON 到 FlowNode 转换器
- **文件**: `src/workflow/el/converter.rs`
- **工作**:
  - 实现 YAML 解析器
  - 映射到 `FlowNode` 枚举
  - 支持循环引用检测
  - 支持验证和错误报告
- **转换示例**:
```rust
impl ElConverter {
    pub fn from_yaml(yaml: &str) -> Result<FlowNode>;
    pub fn from_json(json: &str) -> Result<FlowNode>;
}
```

#### 2.5 添加编排验证和调试工具
- **文件**: `src/workflow/el/validator.rs`
- **工作**:
  - 验证节点引用存在性
  - 验证表达式语法
  - 检测循环依赖
  - 生成执行计划可视化
- **CLI 集成**:
```bash
workflow-toolkit workflow validate workflow.yaml
workflow-toolkit workflow visualize workflow.yaml --output graph.png
```

### 依赖项
- `pest` 或 `nom` 用于语法解析
- `serde_yaml` 已存在

### 验收标准
- [ ] 完整的 EL 语法解析器
- [ ] YAML 工作流定义可以转换为 FlowNode
- [ ] 表达式可以在上下文中正确求值
- [ ] 验证工具可以检测常见错误

---

## Phase 3: 工具原子化重构 (中优先级)

### 目标
将现有粗粒度工具拆分为原子化简单工具，支持通过编排组合成复杂工具。

### 任务清单

#### 3.1 拆分 ClassificationTool 为原子组件
- **文件**: 
  - `src/plugins/file_management/ac_manager.rs` (新建)
  - `src/plugins/file_management/ac_pattern_pusher.rs` (新建)
  - `src/plugins/file_management/ac_matcher.rs` (新建)
- **工作**:
  - `ac-manager`: 自动机生命周期管理
  - `ac-pattern-pusher`: 模式串动态添加/删除
  - `ac-matcher`: 执行多模式匹配
- **原子工具设计**:
```rust
// ac-manager
pub struct AcManagerTool;
impl ToolNode for AcManagerTool {
    fn id(&self) -> &str { "ac-manager" }
    async fn process(&self, ctx: &ExecutionContext) -> Result<()> {
        let automaton = AhoCorasickMatcher::new();
        ctx.set("ac_automaton", automaton).await;
    }
}

// ac-pattern-pusher
pub struct AcPatternPusherTool;
impl ToolNode for AcPatternPusherTool {
    fn id(&self) -> &str { "ac-pattern-pusher" }
    async fn process(&self, ctx: &ExecutionContext) -> Result<()> {
        let mut automaton: AhoCorasickMatcher = ctx.get("ac_automaton").await?;
        let patterns: Vec<Pattern> = ctx.get("patterns").await?;
        automaton.add_patterns(patterns);
        ctx.set("ac_automaton", automaton).await;
    }
}

// ac-matcher
pub struct AcMatcherTool;
impl ToolNode for AcMatcherTool {
    fn id(&self) -> &str { "ac-matcher" }
    async fn process(&self, ctx: &ExecutionContext) -> Result<()> {
        let automaton: AhoCorasickMatcher = ctx.get("ac_automaton").await?;
        let text: String = ctx.get("text").await?;
        let matches = automaton.find_matches(&text);
        ctx.set("matches", matches).await;
    }
}
```

#### 3.2 拆分 BatchProcessorTool 为原子组件
- **文件**:
  - `src/plugins/file_management/batch_scanner.rs` (新建)
  - `src/plugins/file_management/batch_classifier.rs` (新建)
  - `src/plugins/file_management/batch_executor.rs` (新建)
- **工作**:
  - `batch-scanner`: 扫描目录生成文件列表
  - `batch-classifier`: 计算每个文件的目标路径
  - `batch-executor`: 执行物理移动/复制操作

#### 3.3 设计复杂工具编排 DSL
- **文件**: `src/workflow/complex_tool.rs` (新建)
- **工作**:
  - 定义复杂工具结构
  - 支持内嵌 FlowNode
  - 支持参数映射
- **示例**:
```yaml
id: "sensitive-word-filter"
name: "敏感词过滤器"
type: "complex"
composed_of:
  - THEN:
      - node: ac-manager
      - node: ac-pattern-pusher
      - node: ac-matcher
```

#### 3.4 实现工具组合注册机制
- **文件**: `src/tools/composite.rs` (新建)
- **工作**:
  - 实现 `CompositeTool` struct
  - 支持从 YAML 加载复杂工具
  - 自动注册到 ToolRegistry

#### 3.5 更新文档和示例
- **文件**:
  - `docs/tool_design_v2.1.md` (更新)
  - `examples/complex_tool_example.yaml` (新建)
- **工作**:
  - 更新工具设计规范
  - 添加原子化工具示例
  - 添加复杂工具编排示例

### 验收标准
- [ ] ClassificationTool 拆分为 3 个原子工具
- [ ] BatchProcessorTool 拆分为 3 个原子工具
- [ ] 可以通过 YAML 定义复杂工具
- [ ] 复杂工具可以像简单工具一样被调用

---

## Phase 4: WASM 插件恢复 (中优先级)

### 目标
恢复 WASM 插件支持，实现完全沙箱化的插件执行环境。

### 任务清单

#### 4.1 调研 wasmtime/wasmer 兼容性
- **工作**:
  - 测试 `wasmtime` 最新版本兼容性
  - 调研 `wasmer` 作为替代方案
  - 评估 `extism` 框架
- **决策点**: 选择 `wasmtime` 或 `wasmer`

#### 4.2 更新依赖和构建配置
- **文件**: `Cargo.toml`
- **工作**:
  - 取消注释 `wasmtime` 依赖
  - 或添加 `wasmer` 依赖
  - 更新 feature flags
  - 修复编译错误

#### 4.3 实现 WASM 沙箱执行器
- **文件**: `src/plugins/wasm_executor.rs` (更新)
- **工作**:
  - 实现 WASM 模块加载
  - 实现内存限制
  - 实现函数导出/导入
  - 实现错误处理
- **核心结构**:
```rust
pub struct WasmExecutor {
    engine: wasmtime::Engine,
    store: wasmtime::Store<WasmState>,
    instance: wasmtime::Instance,
}

impl WasmExecutor {
    pub fn new(module_path: &Path, limits: ResourceLimits) -> Result<Self>;
    pub async fn call(&mut self, func: &str, args: Vec<Value>) -> Result<Value>;
}
```

#### 4.4 添加 WASM 插件示例
- **文件**:
  - `examples/wasm_tools/simple_calculator.rs` (新建)
  - `examples/wasm_tools/README.md` (更新)
- **工作**:
  - 创建 Rust WASM 插件示例
  - 创建 WIT 接口定义
  - 添加编译和使用说明

### 依赖项
- `wasmtime` 25.0+ 或 `wasmer` 4.0+
- `wit-bindgen` (用于 WIT 接口)

### 验收标准
- [ ] WASM 插件可以编译和加载
- [ ] WASM 插件可以注册工具
- [ ] WASM 插件可以执行并返回结果
- [ ] 资源限制生效

---

## Phase 5: LanceDB 深度集成 (低优先级)

### 目标
实现上下文快照自动保存和"时间旅行"查询能力。

### 任务清单

#### 5.1 设计上下文快照存储方案
- **文件**: `docs/lancedb_integration.md` (新建)
- **工作**:
  - 设计快照数据结构
  - 设计存储策略（何时保存、保存什么）
  - 设计版本化管理
- **快照结构**:
```rust
pub struct ContextSnapshot {
    pub trace_id: String,
    pub timestamp: DateTime<Utc>,
    pub node_id: String,
    pub global_slots: HashMap<String, Value>,
    pub execution_state: ExecutionState,
}
```

#### 5.2 实现自动快照保存机制
- **文件**: `src/storage/snapshot.rs` (新建)
- **工作**:
  - 实现 `SnapshotManager` struct
  - 集成到 Executor Chain
  - 支持配置化保存策略
- **集成点**:
```rust
// 在 AuditExecutor 或新增 SnapshotExecutor 中
async fn execute(&self, component: &dyn Component, ...) -> Result<ComponentOutput> {
    let output = self.inner.execute(component, context, execution_ctx).await?;
    // 保存快照
    self.snapshot_manager.save(ContextSnapshot {
        trace_id: execution_ctx.trace_id.clone(),
        node_id: component.id().to_string(),
        global_slots: context.global_slots().await,
        ...
    }).await?;
    Ok(output)
}
```

#### 5.3 实现"时间旅行"查询 API
- **文件**: `src/storage/time_travel.rs` (新建)
- **工作**:
  - 实现按时间戳查询
  - 实现按节点 ID 查询
  - 实现状态恢复
- **API 设计**:
```rust
pub struct TimeTravelQuery {
    pub trace_id: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub node_id: Option<String>,
}

impl SnapshotManager {
    pub async fn query(&self, query: TimeTravelQuery) -> Result<Vec<ContextSnapshot>>;
    pub async fn restore(&self, snapshot_id: &str) -> Result<DataContext>;
}
```

#### 5.4 添加向量检索能力
- **文件**: `src/storage/vector_store.rs` (新建)
- **工作**:
  - 集成 LanceDB 向量存储
  - 实现文本向量化
  - 实现相似度搜索
- **使用场景**:
  - 相似工作流查找
  - 历史执行模式分析

#### 5.5 性能优化和测试
- **工作**:
  - 快照压缩
  - 异步保存
  - 定期清理
  - 性能基准测试

### 依赖项
- `lancedb` 0.20+ (已存在，可选功能)
- `arrow` 54.0+ (已存在)

### 验收标准
- [ ] 上下文快照自动保存
- [ ] 可以按时间戳查询历史状态
- [ ] 可以从快照恢复执行上下文
- [ ] 向量检索功能可用

---

## 附录 A: 优先级矩阵

| Phase | 优先级 | 业务价值 | 技术难度 | 建议启动时间 |
|-------|--------|----------|----------|--------------|
| Phase 1: MCP Server | P0 | 高 (AI 集成) | 中 | 立即 |
| Phase 2: EL 表达式 | P0 | 高 (编排能力) | 中 | Phase 1 并行 |
| Phase 3: 工具原子化 | P1 | 中 (可维护性) | 中 | Phase 2 完成后 |
| Phase 4: WASM 插件 | P1 | 中 (扩展性) | 高 | Phase 3 完成后 |
| Phase 5: LanceDB | P2 | 低 (高级功能) | 中 | 资源充足时 |

## 附录 B: 技术选型建议

### MCP 协议库选择
| 方案 | 优点 | 缺点 | 推荐 |
|------|------|------|------|
| `rmcp` | 纯 Rust，现代化设计 | 较新，社区小 | ✅ 推荐 |
| `mcp-protocol-server` | 功能完整 | 依赖较重 | 备选 |
| 自研 | 完全可控 | 工作量大 | 不推荐 |

### 表达式解析器选择
| 方案 | 优点 | 缺点 | 推荐 |
|------|------|------|------|
| `pest` | 语法清晰，文档好 | 性能一般 | ✅ 推荐 |
| `nom` | 高性能 | 学习曲线陡 | 备选 |
| `serde_json` | 简单 | 功能有限 | 简单场景 |

### WASM 运行时选择
| 方案 | 优点 | 缺点 | 推荐 |
|------|------|------|------|
| `wasmtime` | 成熟，性能高 | 依赖问题 | 调研后决定 |
| `wasmer` | 易用 | 社区较小 | 备选 |
| `extism` | 专为插件设计 | 额外抽象层 | 考虑中 |

## 附录 C: 风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| MCP 协议变更 | 高 | 中 | 关注官方动态，封装抽象层 |
| WASM 依赖冲突 | 中 | 高 | 使用 Docker 隔离，或换 wasmer |
| EL 语法设计缺陷 | 中 | 中 | 充分原型验证，向后兼容 |
| 性能不达标 | 中 | 低 | 早期基准测试，持续监控 |

---

*本计划应根据实际开发进度和反馈持续调整*
