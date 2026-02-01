# 故障排除指南

## 概述

本文档提供工具系统激进优化后的常见问题和解决方案。

**版本**: 0.2.0-alpha  
**最后更新**: 2026-02-01  

---

## 编译错误

### 错误1: "no `ToolNode` in `tools`"

**症状**:
```
error[E0432]: unresolved import `crate::tools::ToolNode`
  --> src/.../file.rs:10:5
   |
10 | use crate::tools::ToolNode;
   |     ^^^^^^^^^^^^^^^^^^^^^^ no `ToolNode` in `tools`
```

**原因**: `ToolNode` trait已被移除，替换为`Tool`枚举。

**解决方案**:
1. 使用兼容性层（临时）:
```rust
use workflow_toolkit::tools::compat::ToolNode;
```

2. 迁移到新API（推荐）:
```rust
// 旧代码
use workflow_toolkit::tools::ToolNode;

// 新代码
use workflow_toolkit::tools::Tool;
```

---

### 错误2: "expected an `Fn` closure, found `Arc<...>`"

**症状**:
```
error[E0631]: type mismatch in closure arguments
  --> src/.../file.rs:397:23
   |
397 |             .executor(executor)
   |              -------- ^^^^^^^^ expected an `Fn(ToolInput, core::ExecutionContext)` closure, found `Arc<...>`
```

**原因**: `BasicToolBuilder::executor()`现在接受闭包而非`Arc<dyn ToolExecutor>`。

**解决方案**:
使用`executor_arc()`方法或改用闭包:
```rust
// 方案1: 使用executor_arc（兼容性）
.executor_arc(executor)

// 方案2: 改用闭包（推荐）
.executor(|input, _ctx| async move {
    // 执行逻辑
    Ok(ToolOutput::success(input.params))
})
```

---

### 错误3: "method `list_tools` not found"

**症状**:
```
error[E0599]: no method named `list_tools` found for struct `ToolRegistry`
```

**原因**: 新注册表使用`list_names()`或`list_tools()`（返回`Vec<ToolInfo>`）。

**解决方案**:
```rust
// 获取工具名称列表
let names = registry.list_names();

// 获取工具信息列表
let tools = registry.list_tools();
```

---

### 错误4: "the trait bound `BasicToolRegistry: ToolRegistry` is not satisfied"

**症状**:
```
error[E0277]: the trait bound `BasicToolRegistry: compat::ToolRegistry` is not satisfied
```

**原因**: `BasicToolRegistry`现在是一个空结构体，仅用于兼容性。

**解决方案**:
使用新的`ToolRegistry`结构体:
```rust
// 旧代码
let registry = BasicToolRegistry::new();

// 新代码
let registry = ToolRegistry::new();
```

---

### 错误5: "cannot find trait `ToolExecutor`"

**症状**:
```
error[E0405]: cannot find trait `ToolExecutor` in this scope
```

**原因**: `ToolExecutor` trait已被移除。

**解决方案**:
1. 使用兼容性层:
```rust
use workflow_toolkit::tools::compat::ToolExecutor;
```

2. 改用闭包（推荐）:
```rust
// 不再需要ToolExecutor trait
// 直接使用闭包
.executor(|input, ctx| async move {
    // 执行逻辑
})
```

---

## 运行时错误

### 错误6: 工具执行超时

**症状**:
```
WorkflowError::Timeout { duration: ... }
```

**原因**: 工具执行时间超过限制。

**解决方案**:
1. 增加超时时间:
```rust
use workflow_toolkit::tools::TimeoutMiddleware;

let mut stack = MiddlewareStack::new();
stack.add(Arc::new(TimeoutMiddleware::seconds(30))); // 增加超时
```

2. 优化工具性能
3. 使用异步操作避免阻塞

---

### 错误7: 工具未找到

**症状**:
```
WorkflowError::ToolNotFound { name: "tool_name" }
```

**原因**: 尝试执行未注册的工具。

**解决方案**:
1. 检查工具名称拼写
2. 确认工具已注册:
```rust
if !registry.contains("tool_name") {
    println!("Tool not registered!");
}
```

3. 列出所有可用工具:
```rust
for name in registry.list_names() {
    println!("Available: {}", name);
}
```

---

### 错误8: 参数验证失败

**症状**:
```
WorkflowError::ValidationError("Field 'xxx' is required")
```

**原因**: 输入参数不符合要求。

**解决方案**:
1. 使用强类型参数:
```rust
#[derive(ToolInput)]
pub struct MyInput {
    #[tool_input(required = true)]
    pub field: String,
}

let input = MyInput::from_tool_input(&tool_input)?;
input.validate()?;
```

2. 提供默认值:
```rust
#[tool_input(default = 10)]
pub count: u32,
```

3. 检查参数类型:
```rust
if !input.params.is_object() {
    return Err(WorkflowError::validation_error("Params must be object"));
}
```

---

### 错误9: 中间件链错误

**症状**:
```
WorkflowError::MiddlewareError { middleware: "...", message: "..." }
```

**原因**: 中间件执行失败。

**解决方案**:
1. 检查中间件配置:
```rust
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(LoggingMiddleware::new()));
// 确保中间件顺序正确
```

2. 禁用问题中间件:
```rust
// 暂时移除中间件进行测试
// stack.add(Arc::new(ProblematicMiddleware::new()));
```

3. 查看日志:
```rust
// 启用日志中间件查看执行流程
stack.add(Arc::new(LoggingMiddleware::new()));
```

---

## 性能问题

### 问题10: 工具查找慢

**症状**: 工具查找耗时超过预期。

**原因**: 可能使用了旧API或大量工具。

**解决方案**:
1. 使用新注册表API:
```rust
// 快 - O(1)
let tool = registry.get("name");

// 避免 - 遍历查找
for tool in registry.list_tools() {
    if tool.name == "name" { ... }
}
```

2. 使用ID直接查找:
```rust
let tool = registry.get_by_id(tool_id); // 更快
```

3. 检查工具数量:
```rust
println!("Registered tools: {}", registry.len());
```

---

### 问题11: 内存使用过高

**症状**: 应用程序内存占用持续增长。

**原因**: 可能缓存未清理或工具未释放。

**解决方案**:
1. 清理注册表:
```rust
registry.clear(); // 清除所有工具
```

2. 移除单个工具:
```rust
registry.remove("tool_name");
```

3. 检查元数据缓存:
```rust
// 注册表自动管理缓存，无需手动清理
```

---

### 问题12: 并发性能差

**症状**: 并发执行工具时性能不佳。

**原因**: 可能使用了锁或同步操作。

**解决方案**:
1. 使用DashMap的无锁并发:
```rust
// registry已经是线程安全的
// 无需额外锁
```

2. 并行执行:
```rust
use tokio::join;

let results = join_all(tools.iter().map(|tool| {
    tool.execute(input.clone(), ctx.clone())
})).await;
```

3. 限制并发数:
```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10)); // 限制10个并发
```

---

## 迁移问题

### 问题13: 旧插件无法编译

**症状**: 现有插件代码编译失败。

**原因**: API已更改。

**解决方案**:
1. 使用兼容性层（快速修复）:
```rust
use workflow_toolkit::tools::compat::*;
```

2. 逐步迁移:
```rust
// 步骤1: 保持旧代码运行
use workflow_toolkit::tools::compat::ToolNode;

// 步骤2: 新代码使用新API
use workflow_toolkit::tools::Tool;

// 步骤3: 逐步替换旧代码
```

3. 参考迁移指南: [migration-guide.md](./migration-guide.md)

---

### 问题14: 组合工具无法工作

**症状**: `ToolChain`, `ConditionalTool`等无法使用。

**原因**: 组合工具API已更改。

**解决方案**:
1. 使用新的`ComposedTool`:
```rust
use workflow_toolkit::tools::{ComposedTool, CompositionType};

let composed = ComposedTool {
    id: ToolId::new(),
    metadata: Arc::new(metadata),
    composition_type: CompositionType::Chain(vec![tool1_id, tool2_id]),
    tools: vec![tool1_id, tool2_id],
    middleware_stack: None,
};
```

2. 参考示例: [strongly_typed_tools.rs](../../../examples/strongly_typed_tools.rs)

---

## 调试技巧

### 技巧1: 启用日志

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_env_filter("workflow_toolkit=debug")
    .init();
```

### 技巧2: 使用日志中间件

```rust
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(LoggingMiddleware::new()));
```

### 技巧3: 检查工具注册

```rust
println!("Registered tools:");
for name in registry.list_names() {
    println!("  - {}", name);
}
```

### 技巧4: 验证输入输出

```rust
// 打印输入
println!("Input: {:?}", input.params);

// 打印输出
println!("Output: {:?}", output.result);
```

### 技巧5: 使用计时中间件

```rust
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(TimingMiddleware::new()));
```

---

## 常见问题 (FAQ)

### Q1: 如何回滚到旧版本？

**A**: 使用Git回滚:
```bash
git checkout 00164aa^ -- src/tools/
```

或锁定版本:
```toml
[dependencies]
workflow-toolkit = "=0.1.0"
```

---

### Q2: 可以同时使用新旧API吗？

**A**: 可以，通过兼容性层:
```rust
use workflow_toolkit::tools::Tool; // 新API
use workflow_toolkit::tools::compat::ToolNode; // 旧API
```

---

### Q3: 性能提升不明显？

**A**: 检查:
1. 是否使用了新API（非兼容性层）
2. 是否启用了release模式编译
3. 是否有其他性能瓶颈

```bash
cargo build --release
cargo test --release
```

---

### Q4: 如何贡献新中间件？

**A**: 实现`Middleware` trait:
```rust
#[async_trait]
impl Middleware for MyMiddleware {
    async fn process(&self, ctx: &mut MiddlewareContext, next: Next<'_>) -> Result<ToolOutput> {
        // 前置处理
        let result = next.run(ctx).await;
        // 后置处理
        result
    }
}
```

---

### Q5: 强类型参数有什么好处？

**A**:
1. 编译时类型检查
2. 自动验证
3. IDE支持（自动完成）
4. 文档生成

---

## 获取帮助

### 资源

1. **API文档**: [api-reference.md](./api-reference.md)
2. **迁移指南**: [migration-guide.md](./migration-guide.md)
3. **示例代码**: [examples/](../../../examples/)
4. **测试用例**: [tests/](../../../tests/)

### 报告问题

如果遇到未记录的问题:

1. 收集错误信息:
```bash
cargo build 2>&1 | tee build.log
```

2. 创建最小复现:
```rust
// 最小代码示例
```

3. 提交Issue:
- 描述问题
- 提供复现步骤
- 附上错误日志

---

## 更新记录

| 日期 | 更新内容 |
|------|----------|
| 2026-02-01 | 创建故障排除指南 |
| 2026-02-01 | 添加编译错误解决方案 |
| 2026-02-01 | 添加运行时错误解决方案 |
| 2026-02-01 | 添加性能问题解决方案 |

---

**文档版本**: 1.0  
**最后更新**: 2026-02-01  
**维护者**: Atlas Orchestrator
