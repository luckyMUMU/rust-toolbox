# 开发者指南

## 概述

本文档是工具系统激进优化后的完整开发者指南，涵盖从入门到精通的全部内容。

**版本**: 0.2.0-alpha  
**难度**: 初级到高级  
**预计学习时间**: 2-4小时  
**最后更新**: 2026-02-01  

---

## 目录

1. [快速入门](#快速入门) - 15分钟上手
2. [核心概念](#核心概念) - 理解架构
3. [基础教程](#基础教程) - 逐步学习
4. [高级主题](#高级主题) - 深入掌握
5. [实战案例](#实战案例) - 真实场景
6. [调试与优化](#调试与优化) - 性能调优
7. [参考资源](#参考资源) - 延伸阅读

---

## 快速入门

### 5分钟创建第一个工具

```rust
use workflow_toolkit::tools::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建注册表
    let registry = ToolRegistry::new();
    
    // 2. 创建工具
    let tool = NativeToolBuilder::new()
        .name("hello")
        .version("1.0.0")
        .executor(|input, _ctx| async move {
            let name = input.params["name"].as_str().unwrap_or("World");
            Ok(ToolOutput::success(json!({
                "message": format!("Hello, {}!", name)
            })))
        })
        .build()?;
    
    // 3. 注册并执行
    registry.register(Tool::Native(Arc::new(tool)));
    let output = registry.execute("hello", ToolInput::new(json!({"name": "Alice"}))).await?;
    
    println!("{}", output.result["message"]);
    Ok(())
}
```

**输出**:
```
Hello, Alice!
```

---

## 核心概念

### 1. 工具（Tool）

工具是执行特定任务的基本单元。

```rust
pub enum Tool {
    Native(Arc<NativeTool>),    // Rust原生工具
    Python(Arc<PythonTool>),    // Python脚本工具
    NodeJs(Arc<NodeJsTool>),    // Node.js工具
    Docker(Arc<DockerTool>),    // Docker容器工具
    Wasm(Arc<WasmTool>),        // WASM工具
    Composed(Arc<ComposedTool>), // 组合工具
}
```

**关键特性**:
- **静态分发**: 编译时确定类型，零运行时开销
- **类型安全**: 编译器检查所有可能的工具类型
- **内存高效**: 8字节枚举标签 vs 16字节胖指针

### 2. 注册表（ToolRegistry）

注册表是工具的管理中心。

```rust
pub struct ToolRegistry {
    tools: DashMap<ToolId, Tool>,           // ID -> 工具
    name_index: DashMap<String, ToolId>,    // 名称 -> ID
    metadata_cache: DashMap<ToolId, Arc<ToolMetadata>>,
}
```

**关键特性**:
- **O(1)查找**: 双重索引实现常数时间查找
- **无锁并发**: DashMap支持高并发访问
- **元数据缓存**: 加速工具信息查询

### 3. 中间件（Middleware）

中间件是横切关注点的实现机制。

```rust
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn process(
        &self,
        ctx: &mut MiddlewareContext,
        next: Next<'_>,
    ) -> Result<ToolOutput>;
}
```

**执行流程**:
```
Request → Middleware1 → Middleware2 → ... → Tool Execution
            ↓                ↓
        Pre-process      Pre-process
            ↓                ↓
        Post-process     Post-process
            ↓                ↓
Response ← Response ← Response ← ... ← Result
```

### 4. 强类型参数

强类型参数提供编译时类型安全。

```rust
#[derive(ToolInput, Serialize, Deserialize)]
pub struct ProcessInput {
    #[tool_input(description = "Data to process", required = true)]
    pub data: String,
    
    #[tool_input(description = "Processing mode", default = "normal")]
    pub mode: String,
}
```

**优势**:
- 编译时类型检查
- 自动参数验证
- IDE自动完成
- 自文档化

---

## 基础教程

### 教程1: 创建简单工具

**目标**: 创建一个文件读取工具

**步骤**:

1. **定义工具**:
```rust
let read_file_tool = NativeToolBuilder::new()
    .name("read_file")
    .version("1.0.0")
    .description("Reads a file and returns its content")
    .category("filesystem")
    .tags(vec!["file".to_string(), "read".to_string()])
    .executor(|input, _ctx| async move {
        // 获取参数
        let path = input.params["path"]
            .as_str()
            .ok_or_else(|| WorkflowError::ValidationError(
                "Parameter 'path' is required".to_string()
            ))?;
        
        // 读取文件
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| WorkflowError::IoError {
                message: format!("Failed to read file: {}", e),
            })?;
        
        // 返回结果
        Ok(ToolOutput::success(json!({
            "content": content,
            "size": content.len()
        })))
    })
    .build()?;
```

2. **注册工具**:
```rust
let registry = ToolRegistry::new();
registry.register(Tool::Native(Arc::new(read_file_tool)));
```

3. **执行工具**:
```rust
let input = ToolInput::new(json!({
    "path": "example.txt"
}));

let output = registry.execute("read_file", input).await?;
println!("Content: {}", output.result["content"]);
```

**完整代码**:
```rust
use workflow_toolkit::tools::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let registry = ToolRegistry::new();
    
    let tool = NativeToolBuilder::new()
        .name("read_file")
        .version("1.0.0")
        .executor(|input, _ctx| async move {
            let path = input.params["path"].as_str().unwrap();
            let content = tokio::fs::read_to_string(path).await?;
            Ok(ToolOutput::success(json!({"content": content})))
        })
        .build()?;
    
    registry.register(Tool::Native(Arc::new(tool)));
    
    let input = ToolInput::new(json!({"path": "test.txt"}));
    let output = registry.execute("read_file", input).await?;
    
    println!("{}", output.result["content"]);
    Ok(())
}
```

---

### 教程2: 使用强类型参数

**目标**: 使用强类型参数创建计算器工具

**步骤**:

1. **定义输入结构**:
```rust
#[derive(ToolInput, Serialize, Deserialize, Debug)]
pub struct CalculatorInput {
    #[tool_input(description = "First operand", required = true)]
    pub a: f64,
    
    #[tool_input(description = "Second operand", required = true)]
    pub b: f64,
    
    #[tool_input(description = "Operation (add, subtract, multiply, divide)", required = true)]
    pub operation: String,
}

#[derive(ToolOutput, Serialize, Deserialize, Debug)]
pub struct CalculatorOutput {
    pub result: f64,
    pub expression: String,
}
```

2. **实现验证逻辑**:
```rust
impl ToolInputConvert for CalculatorInput {
    fn validate(&self) -> Result<(), WorkflowError> {
        // 验证操作类型
        let valid_ops = ["add", "subtract", "multiply", "divide"];
        if !valid_ops.contains(&self.operation.as_str()) {
            return Err(WorkflowError::ValidationError(
                format!("Invalid operation: {}", self.operation)
            ));
        }
        
        // 验证除数
        if self.operation == "divide" && self.b == 0.0 {
            return Err(WorkflowError::ValidationError(
                "Cannot divide by zero".to_string()
            ));
        }
        
        Ok(())
    }
    
    // ... 其他方法
}
```

3. **创建工具**:
```rust
let calculator_tool = NativeToolBuilder::new()
    .name("calculator")
    .version("1.0.0")
    .executor(|input, _ctx| async move {
        // 解析输入
        let calc_input = CalculatorInput::from_tool_input(&input)?;
        
        // 验证
        calc_input.validate()?;
        
        // 计算
        let result = match calc_input.operation.as_str() {
            "add" => calc_input.a + calc_input.b,
            "subtract" => calc_input.a - calc_input.b,
            "multiply" => calc_input.a * calc_input.b,
            "divide" => calc_input.a / calc_input.b,
            _ => unreachable!(),
        };
        
        // 构建输出
        let output = CalculatorOutput {
            result,
            expression: format!("{} {} {} = {}",
                calc_input.a,
                match calc_input.operation.as_str() {
                    "add" => "+",
                    "subtract" => "-",
                    "multiply" => "*",
                    "divide" => "/",
                    _ => "?",
                },
                calc_input.b,
                result
            ),
        };
        
        Ok(output.into_tool_output())
    })
    .build()?;
```

4. **使用工具**:
```rust
// 创建强类型输入
let calc_input = CalculatorInput {
    a: 10.0,
    b: 5.0,
    operation: "multiply".to_string(),
};

// 执行
let output = registry.execute("calculator", calc_input.into_tool_input()).await?;
let result = CalculatorOutput::from_tool_output(&output)?;

println!("{} = {}", result.expression, result.result);
// 输出: 10 * 5 = 50
```

---

### 教程3: 添加中间件

**目标**: 为工具添加日志和计时中间件

**步骤**:

1. **创建中间件栈**:
```rust
use workflow_toolkit::tools::{
    MiddlewareStack, LoggingMiddleware, TimingMiddleware
};

let mut stack = MiddlewareStack::new();
stack.add(Arc::new(LoggingMiddleware::new()));
stack.add(Arc::new(TimingMiddleware::new()));
```

2. **应用到工具**:
```rust
let tool = NativeToolBuilder::new()
    .name("processed_tool")
    .executor(|input, _ctx| async move {
        // 业务逻辑
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(ToolOutput::success(input.params))
    })
    .with_middleware(stack)
    .build()?;
```

3. **查看输出**:
```
[INFO] Starting execution of tool: processed_tool
[INFO] Tool processed_tool completed successfully in 102ms
```

**完整示例**:
```rust
use workflow_toolkit::tools::*;
use workflow_toolkit::tools::{
    MiddlewareStack, LoggingMiddleware, TimingMiddleware, RetryMiddleware
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let registry = ToolRegistry::new();
    
    // 创建中间件栈
    let mut stack = MiddlewareStack::new();
    stack.add(Arc::new(LoggingMiddleware::new()));
    stack.add(Arc::new(TimingMiddleware::new()));
    stack.add(Arc::new(RetryMiddleware::new(3)));
    
    // 创建工具
    let tool = NativeToolBuilder::new()
        .name("api_call")
        .executor(|input, _ctx| async move {
            // 模拟API调用
            call_external_api(input.params).await
        })
        .with_middleware(stack)
        .build()?;
    
    registry.register(Tool::Native(Arc::new(tool)));
    
    // 执行
    let output = registry.execute("api_call", input).await?;
    
    Ok(())
}
```

---

## 高级主题

### 主题1: 自定义中间件

**目标**: 创建自定义中间件

**实现**:
```rust
use workflow_toolkit::tools::*;
use tracing::{info, warn};

// 自定义中间件：记录输入输出大小
pub struct SizeLoggingMiddleware;

#[async_trait]
impl Middleware for SizeLoggingMiddleware {
    async fn process(
        &self,
        ctx: &mut MiddlewareContext,
        next: Next<'_>,
    ) -> Result<ToolOutput> {
        let input_size = serde_json::to_string(&ctx.input.params)
            .unwrap_or_default()
            .len();
        
        info!("Input size: {} bytes", input_size);
        
        let start = Instant::now();
        let result = next.run(ctx).await;
        let elapsed = start.elapsed();
        
        match &result {
            Ok(output) => {
                let output_size = serde_json::to_string(&output.result)
                    .unwrap_or_default()
                    .len();
                info!(
                    "Output size: {} bytes, elapsed: {:?}",
                    output_size, elapsed
                );
            }
            Err(e) => {
                warn!("Execution failed after {:?}: {}", elapsed, e);
            }
        }
        
        result
    }
}
```

**使用**:
```rust
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(SizeLoggingMiddleware));
```

---

### 主题2: 组合工具

**目标**: 创建工具链

**实现**:
```rust
use workflow_toolkit::tools::*;

// 创建组合工具
async fn execute_tool_chain(
    registry: &ToolRegistry,
    tool_ids: Vec<ToolId>,
    initial_input: ToolInput,
) -> Result<ToolOutput> {
    let mut current_input = initial_input;
    
    for tool_id in tool_ids {
        let tool = registry.get_by_id(tool_id)
            .ok_or_else(|| WorkflowError::NotFound {
                resource: format!("tool id {:?}", tool_id),
            })?;
        
        let ctx = ExecutionContext::default();
        let output = tool.execute(current_input, ctx).await?;
        
        // 将输出转换为下一次的输入
        current_input = ToolInput::new(output.result);
    }
    
    Ok(ToolOutput::success(current_input.params))
}

// 使用
let tool_ids = vec![
    registry.get("read_file").unwrap().id(),
    registry.get("parse_json").unwrap().id(),
    registry.get("transform").unwrap().id(),
];

let result = execute_tool_chain(&registry, tool_ids, initial_input).await?;
```

---

### 主题3: 并发执行

**目标**: 并行执行多个工具

**实现**:
```rust
use tokio::join;

async fn execute_parallel(
    registry: &ToolRegistry,
    tasks: Vec<(String, ToolInput)>,
) -> Vec<Result<ToolOutput>> {
    let futures: Vec<_> = tasks.into_iter().map(|(name, input)| {
        let registry = registry.clone();
        async move {
            registry.execute(&name, input).await
        }
    }).collect();
    
    join_all(futures).await
}

// 使用
let tasks = vec![
    ("tool1".to_string(), input1),
    ("tool2".to_string(), input2),
    ("tool3".to_string(), input3),
];

let results = execute_parallel(&registry, tasks).await;

for (i, result) in results.iter().enumerate() {
    match result {
        Ok(output) => println!("Task {} succeeded: {:?}", i, output.result),
        Err(e) => println!("Task {} failed: {}", i, e),
    }
}
```

---

## 实战案例

### 案例1: 数据处理管道

**场景**: 构建一个ETL（提取-转换-加载）管道

**组件**:
1. **Extract**: 从API获取数据
2. **Transform**: 清洗和转换数据
3. **Load**: 保存到数据库

**实现**:
```rust
pub struct EtlPipeline {
    registry: ToolRegistry,
}

impl EtlPipeline {
    pub async fn run(&self, source: &str, destination: &str) -> Result<EtlResult> {
        // Step 1: Extract
        let extract_input = ToolInput::new(json!({"url": source}));
        let extract_output = self.registry.execute("extract", extract_input).await?;
        
        // Step 2: Transform
        let transform_input = ToolInput::new(extract_output.result);
        let transform_output = self.registry.execute("transform", transform_input).await?;
        
        // Step 3: Load
        let load_input = ToolInput::new(json!({
            "data": transform_output.result,
            "destination": destination
        }));
        let load_output = self.registry.execute("load", load_input).await?;
        
        Ok(EtlResult {
            records_processed: load_output.result["count"].as_u64().unwrap_or(0),
            duration: load_output.metadata.as_ref()
                .and_then(|m| m["duration_ms"].as_u64())
                .unwrap_or(0),
        })
    }
}
```

---

### 案例2: 工作流编排

**场景**: 实现一个审批工作流

**流程**:
1. 提交申请
2. 自动审核
3. 人工审批（如果需要）
4. 发送通知

**实现**:
```rust
pub struct ApprovalWorkflow {
    registry: ToolRegistry,
}

impl ApprovalWorkflow {
    pub async fn submit(&self, application: Application) -> Result<WorkflowId> {
        // 保存申请
        let save_input = ToolInput::new(json!({"application": application}));
        let save_output = self.registry.execute("save_application", save_input).await?;
        let workflow_id = save_output.result["id"].as_str().unwrap();
        
        // 自动审核
        let auto_review_input = ToolInput::new(json!({"application": application}));
        let auto_review_output = self.registry.execute("auto_review", auto_review_input).await?;
        
        let decision = auto_review_output.result["decision"].as_str().unwrap();
        
        match decision {
            "approved" => {
                // 自动通过
                self.approve(workflow_id).await?;
            }
            "rejected" => {
                // 自动拒绝
                self.reject(workflow_id, "Auto review failed").await?;
            }
            "manual_review" => {
                // 需要人工审批
                self.request_manual_review(workflow_id).await?;
            }
            _ => {}
        }
        
        Ok(WorkflowId::new())
    }
    
    async fn approve(&self, workflow_id: &str) -> Result<()> {
        let input = ToolInput::new(json!({"id": workflow_id, "action": "approve"}));
        self.registry.execute("update_status", input).await?;
        
        // 发送通知
        let notify_input = ToolInput::new(json!({
            "workflow_id": workflow_id,
            "status": "approved"
        }));
        self.registry.execute("send_notification", notify_input).await?;
        
        Ok(())
    }
    
    // ... 其他方法
}
```

---

## 调试与优化

### 性能分析

**1. 使用计时中间件**:
```rust
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(TimingMiddleware::new()));
```

**2. 自定义性能指标**:
```rust
pub struct PerformanceMetricsMiddleware {
    metrics: Arc<Mutex<Metrics>>,
}

#[async_trait]
impl Middleware for PerformanceMetricsMiddleware {
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        let start = Instant::now();
        let result = next.run(ctx).await;
        let elapsed = start.elapsed();
        
        let mut metrics = self.metrics.lock().await;
        metrics.record(ctx.metadata.tool_name.clone(), elapsed, result.is_ok());
        
        result
    }
}
```

**3. 分析瓶颈**:
```rust
// 打印性能报告
for (tool_name, stats) in metrics.iter() {
    println!(
        "{}: avg={:?}, min={:?}, max={:?}, calls={}",
        tool_name, stats.avg, stats.min, stats.max, stats.count
    );
}
```

---

### 内存优化

**1. 避免不必要的克隆**:
```rust
// ✅ 好 - 使用引用
let data = &input.params;

// ❌ 避免 - 不必要的克隆
let data = input.params.clone();
```

**2. 使用流式处理**:
```rust
// 处理大文件时使用流
let mut stream = tokio::fs::read_dir(path).await?;
while let Some(entry) = stream.next_entry().await? {
    // 处理每个条目
}
```

**3. 限制缓存大小**:
```rust
use moka::future::Cache;

let cache: Cache<String, Value> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(3600))
    .build();
```

---

### 调试技巧

**1. 启用详细日志**:
```rust
tracing_subscriber::fmt()
    .with_env_filter("workflow_toolkit=debug")
    .with_target(true)
    .with_thread_ids(true)
    .init();
```

**2. 使用日志中间件**:
```rust
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(LoggingMiddleware::new()));
```

**3. 检查工具注册**:
```rust
println!("Registered tools:");
for name in registry.list_names() {
    let metadata = registry.get_metadata(&name).unwrap();
    println!("  - {} (v{}): {}",
        name,
        metadata.version,
        metadata.info.description
    );
}
```

---

## 参考资源

### 文档

- [API参考](./api-reference.md) - 完整API文档
- [最佳实践](./best-practices.md) - 开发最佳实践
- [故障排除](./troubleshooting.md) - 常见问题解决
- [新旧对比](./comparison.md) - 架构对比
- [迁移指南](./migration-guide.md) - 从旧版本迁移

### 示例

- [强类型工具示例](../../../examples/strongly_typed_tools.rs)
- [更多示例](../../../examples/)

### 工具

- [测试计划](./test-plan.md) - 测试策略
- [学习记录](./learnings.md) - 设计决策

---

## 总结

恭喜！您已经完成了开发者指南的学习。

### 掌握的技能

✅ 创建简单和复杂工具  
✅ 使用强类型参数  
✅ 添加和自定义中间件  
✅ 构建工具链和工作流  
✅ 性能优化和调试  

### 下一步

1. **实践**: 尝试构建自己的工具
2. **深入**: 阅读[最佳实践](./best-practices.md)
3. **探索**: 查看[API参考](./api-reference.md)
4. **贡献**: 分享您的经验和工具

---

**文档版本**: 1.0  
**最后更新**: 2026-02-01  
**维护者**: Atlas Orchestrator
