# 工具系统 (Tool System)

> **版本**: v1.0  
> **创建日期**: 2026-01-15  
> **最后更新**: 2026-02-27  
> **维护者**: Workflow Toolkit Team

## 1. 核心定义 (Stable)

### 1.1 模块职责

工具系统负责工具的注册、发现、组合和执行。采用 Enum-based 架构（已重构自 trait-based），提供更好的性能和类型安全。

### 1.2 模块结构

```
tools/
├── types.rs        # 工具类型定义（Enum-based）
├── registry.rs     # 工具注册表
├── node.rs         # 工具节点实现
├── composable.rs   # 可组合工具
├── middleware.rs   # 中间件系统
├── template.rs     # 模板系统
├── version.rs      # 版本管理
├── compat.rs       # 兼容层（旧 trait）
└── algo/           # 算法模块
    └── mod.rs
```

### 1.3 核心类型

#### 工具 Enum

```rust
/// 工具 Enum（新架构）
pub enum Tool {
    /// 原生 Rust 实现
    Native(NativeTool),
    /// Python 脚本
    Python(PythonTool),
    /// Node.js 模块
    NodeJs(NodeJsTool),
    /// Docker 容器
    Docker(DockerTool),
    /// WebAssembly
    Wasm(WasmTool),
    /// 组合工具
    Composed(ComposedTool),
}

impl Tool {
    /// 执行工具
    pub async fn execute(&self, input: ToolInput) -> Result<ToolOutput>;
    
    /// 获取元数据
    pub fn metadata(&self) -> &ToolMetadata;
    
    /// 获取输入 Schema
    pub fn input_schema(&self) -> &InputSchema;
    
    /// 获取输出 Schema
    pub fn output_schema(&self) -> &OutputSchema;
}
```

#### 工具元数据

```rust
/// 工具元数据
pub struct ToolMetadata {
    pub id: ToolId,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 工具 ID
pub struct ToolId(pub Uuid);

/// 工具种类
pub enum ToolKind {
    Native,
    Python,
    NodeJs,
    Docker,
    Wasm,
    Composed,
}
```

#### 输入输出

```rust
/// 工具输入
pub struct ToolInput {
    pub params: Value,
    pub context: Option<ExecutionContext>,
}

/// 工具输出
pub struct ToolOutput {
    pub result: Value,
    pub metadata: OutputMetadata,
}

/// 输出元数据
pub struct OutputMetadata {
    pub execution_time_ms: u64,
    pub cached: bool,
}
```

### 1.4 工具注册表

```rust
/// 工具注册表
pub struct ToolRegistry {
    tools: DashMap<ToolId, Tool>,
    name_index: DashMap<String, ToolId>,
}

impl ToolRegistry {
    /// 注册工具
    pub fn register(&self, tool: Tool) -> Result<ToolId>;
    
    /// 注销工具
    pub fn unregister(&self, id: &ToolId) -> Result<()>;
    
    /// 通过 ID 获取工具
    pub fn get(&self, id: &ToolId) -> Option<Tool>;
    
    /// 通过名称获取工具
    pub fn get_by_name(&self, name: &str) -> Option<Tool>;
    
    /// 列出所有工具
    pub fn list_all(&self) -> Vec<ToolMetadata>;
    
    /// 按类别筛选
    pub fn filter_by_category(&self, category: &str) -> Vec<ToolMetadata>;
}

/// 注册表构建器
pub struct ToolRegistryBuilder;
```

### 1.5 中间件系统

```rust
/// 中间件（Middleware）trait
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, ctx: MiddlewareContext, next: Next<'_>) -> Result<ToolOutput>;
}

/// 中间件上下文
pub struct MiddlewareContext {
    pub tool_id: ToolId,
    pub input: ToolInput,
    pub execution_metadata: ExecutionMetadata,
}

/// 中间件栈
pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareStack {
    pub fn builder() -> MiddlewareStackBuilder;
}

/// 内置中间件
pub struct RetryMiddleware;
pub struct CacheMiddleware;
pub struct TimeoutMiddleware;
pub struct LoggingMiddleware;
pub struct MetricsMiddleware;
pub struct CircuitBreakerMiddleware;
pub struct TimingMiddleware;
```

### 1.6 可组合工具

```rust
/// 工具组合器
pub struct ToolComposer;

/// 组合工具
pub struct ComposedTool {
    pub composition_type: CompositionType,
    pub tools: Vec<ToolId>,
}

pub enum CompositionType {
    /// 顺序执行
    Sequence,
    /// 并行执行
    Parallel,
    /// 条件执行
    Conditional { condition: Box<dyn Fn(&Value) -> bool> },
    /// 分支执行
    Branch { selector: Box<dyn Fn(&Value) -> usize> },
}

/// 工具链
pub struct ToolChain {
    tools: Vec<ToolId>,
    data_flow: HashMap<String, String>, // 输出到输入的映射
}

/// 并行工具
pub struct ParallelTools {
    tools: Vec<ToolId>,
    merge_strategy: MergeStrategy,
}
```

### 1.7 模板系统

```rust
/// 模板引擎（Template Engine）
pub struct TemplateEngine;

/// 参数模板
pub struct ParameterTemplate {
    pub template: String,
    pub variables: Vec<String>,
}

/// 模板上下文
pub struct TemplateContext {
    pub variables: HashMap<String, Value>,
    pub execution_context: Option<ExecutionContext>,
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-T001: Enum vs Trait
- **决策**: 从 trait-based 重构为 Enum-based
- **理由**: 更好的性能（避免虚表调用），更简单的序列化
- **风险**: 扩展性稍差，新增类型需要修改 Enum

#### ADR-T002: 中间件模式
- **决策**: 采用洋葱模型（Onion Model）中间件链
- **理由**: 支持横切关注点（日志、缓存、重试）的复用
- **风险**: 调用栈深度增加

#### ADR-T003: 组合工具设计
- **决策**: 组合也是工具（ComposedTool 是 Tool 的变体）
- **理由**: 统一接口，支持嵌套组合
- **风险**: 调试复杂度增加

### 2.2 任务清单

- [x] Task 0: 渐进式重构（修复旧逻辑/格式）
- [x] Task 1: Tool Enum 实现
- [x] Task 2: 新注册表实现
- [x] Task 3: 工具节点实现
- [x] Task 4: 中间件系统
- [x] Task 5: 组合工具完善
- [ ] Task 6: 版本管理实现

### 2.3 接口契约

```rust
/// 工具执行配置
pub struct ToolExecutionConfig {
    pub timeout: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
    pub use_cache: bool,
    pub cache_ttl: Option<Duration>,
}

/// 工具执行结果
pub struct ToolExecutionResult {
    pub success: bool,
    pub output: Option<ToolOutput>,
    pub error: Option<ToolError>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
}

/// 工具错误
pub enum ToolError {
    NotFound { tool_id: ToolId },
    InvalidInput { message: String },
    ExecutionFailed { message: String },
    Timeout { duration: Duration },
    Cancelled,
}
```

### 2.4 测试策略

- **单元测试（Unit Test）**: 各工具类型独立测试
- **集成测试（Integration Test）**: 工具注册/执行/组合完整流程
- **性能测试（Performance Test）**: Enum vs Trait 性能对比
- **兼容性测试（Compatibility Test）**: 旧 trait 兼容层测试

## 3. 状态记录

- `[已完成]` | 组合工具完善 | 2026-02-07
- `[已完成]` | 中间件系统 | 2026-02-01
- `[已完成]` | Enum 重构 | 2026-01-28
- `[已完成]` | 新注册表 | 2026-01-25

## 4. 架构演进

### 旧架构（Trait-based）
```rust
// 已废弃，移至 compat 模块
trait Tool { async fn execute(&self, input: Value) -> Result<Value>; }
```

### 新架构（Enum-based）
```rust
// 当前实现
enum Tool { Native(...), Python(...), ... }
impl Tool { async fn execute(&self, ...) -> ... }
```

## 5. 使用示例

```rust
// 注册工具
let registry = ToolRegistry::new();
let tool = Tool::Native(NativeTool::new("calculator", calc_fn));
let id = registry.register(tool)?;

// 执行工具
let input = ToolInput::new(json!({"a": 1, "b": 2}));
let output = registry.get(&id).unwrap().execute(input).await?;

// 组合工具
let chain = ToolChain::builder()
    .add(tool_a)
    .add(tool_b)
    .map_output("result", "input")
    .build();
```

---

## 6. 工具开发指南

### 6.1 自定义工具开发规范

#### 工具接口要求

所有自定义工具必须满足以下要求：

1. **实现 ToolNode trait** 或 **包装为 Tool Enum 变体**
2. **提供完整的 Schema 定义**（输入/输出）
3. **支持参数验证**
4. **正确处理错误**

#### 开发原生工具（Native Tool）

```rust
use workflow_toolkit::{
    tools::{ToolNode, ToolDefinition, ToolInfo},
    ExecutionContext, WorkflowError,
};
use serde_json::{json, Value};
use async_trait::async_trait;

/// 自定义计算器工具
pub struct CalculatorTool {
    info: ToolInfo,
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            info: ToolInfo {
                name: "calculator".to_string(),
                version: "1.0.0".to_string(),
                description: "基础数学计算工具".to_string(),
                parameters_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["add", "subtract", "multiply", "divide"]
                        },
                        "a": { "type": "number" },
                        "b": { "type": "number" }
                    },
                    "required": ["operation", "a", "b"]
                }),
                return_schema: json!({
                    "type": "object",
                    "properties": {
                        "result": { "type": "number" }
                    }
                }),
                category: Some("math".to_string()),
                tags: vec!["calculation".to_string()],
                dependencies: vec![],
                plugin_name: None,
            },
        }
    }
}

#[async_trait]
impl ToolNode for CalculatorTool {
    fn name(&self) -> &str { &self.info.name }
    fn version(&self) -> &str { &self.info.version }
    
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value, WorkflowError> {
        let operation = params["operation"].as_str()
            .ok_or_else(|| WorkflowError::invalid_input("缺少 operation 参数"))?;
        let a = params["a"].as_f64()
            .ok_or_else(|| WorkflowError::invalid_input("缺少 a 参数"))?;
        let b = params["b"].as_f64()
            .ok_or_else(|| WorkflowError::invalid_input("缺少 b 参数"))?;
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(WorkflowError::tool_execution("除零错误"));
                }
                a / b
            }
            _ => return Err(WorkflowError::invalid_input(&format!("未知操作: {}", operation))),
        };
        
        Ok(json!({ "result": result }))
    }
    
    fn validate_parameters(&self, params: &Value) -> Result<(), WorkflowError> {
        // 使用 JSON Schema 验证
        if params["operation"].as_str().is_none() {
            return Err(WorkflowError::invalid_input("缺少 operation 参数"));
        }
        if params["a"].as_f64().is_none() {
            return Err(WorkflowError::invalid_input("缺少 a 参数或类型错误"));
        }
        if params["b"].as_f64().is_none() {
            return Err(WorkflowError::invalid_input("缺少 b 参数或类型错误"));
        }
        Ok(())
    }
    
    fn get_schema(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.info.name.clone(),
            version: self.info.version.clone(),
            description: self.info.description.clone(),
            parameters_schema: self.info.parameters_schema.clone(),
            return_schema: self.info.return_schema.clone(),
        }
    }
    
    fn get_plugin_info(&self) -> Option<&PluginInfo> { None }
}
```

### 6.2 工具注册流程

```rust
// 方式一：直接注册到注册表
let mut registry = ToolRegistry::new();
let calculator = CalculatorTool::new();
registry.register_tool(Arc::new(calculator))?;

// 方式二：使用 Tool Enum
let tool = Tool::Native(NativeTool::from_fn("my_tool", |params, ctx| async move {
    // 工具逻辑
    Ok(json!({"status": "success"}))
}));
registry.register(tool)?;
```

### 6.3 工具配置规范

```yaml
# tool-config.yaml
name: my-custom-tool
version: 1.0.0
description: 自定义工具描述
category: utility
tags:
  - custom
  - utility

# 输入 Schema (JSON Schema 格式)
input_schema:
  type: object
  properties:
    input_path:
      type: string
      description: 输入文件路径
    output_path:
      type: string
      description: 输出文件路径
  required:
    - input_path

# 输出 Schema
output_schema:
  type: object
  properties:
    success:
      type: boolean
    message:
      type: string

# 执行配置
execution:
  timeout_seconds: 30
  retry_count: 3
  retry_delay_ms: 1000
  cache_enabled: true
  cache_ttl_seconds: 300
```

### 6.4 工具测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_calculator_add() {
        let tool = CalculatorTool::new();
        let params = json!({"operation": "add", "a": 2, "b": 3});
        
        let result = tool.execute(params, ExecutionContext::default()).await.unwrap();
        
        assert_eq!(result["result"], 5.0);
    }
    
    #[test]
    fn test_parameter_validation() {
        let tool = CalculatorTool::new();
        
        // 缺少必需参数
        let result = tool.validate_parameters(&json!({"a": 1}));
        assert!(result.is_err());
        
        // 参数类型错误
        let result = tool.validate_parameters(&json!({"operation": "add", "a": "not_a_number", "b": 2}));
        assert!(result.is_err());
        
        // 正确参数
        let result = tool.validate_parameters(&json!({"operation": "add", "a": 1, "b": 2}));
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_divide_by_zero() {
        let tool = CalculatorTool::new();
        let params = json!({"operation": "divide", "a": 1, "b": 0});
        
        let result = tool.execute(params, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
}
```

### 6.5 工具扩展点

| 扩展点 | 接口 | 用途 |
|--------|------|------|
| 自定义执行器 | `AsyncFunctionExecutor` | 包装异步函数为工具 |
| 中间件 | `Middleware` trait | 添加横切关注点（日志、缓存、重试） |
| 参数验证器 | `ParameterValidator` trait | 自定义参数验证逻辑 |
| 结果处理器 | `ResultProcessor` trait | 后处理工具输出 |

### 6.6 工具版本管理

```rust
/// 工具版本兼容性检查
pub fn check_compatibility(tool_version: &str, required_version: &str) -> bool {
    // 使用语义化版本检查
    let tool_parts: Vec<u32> = tool_version.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let required_parts: Vec<u32> = required_version.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    // 主版本号必须匹配
    if tool_parts.get(0) != required_parts.get(0) {
        return false;
    }
    
    // 工具次版本号 >= 要求次版本号
    tool_parts.get(1).unwrap_or(&0) >= required_parts.get(1).unwrap_or(&0)
}
```
