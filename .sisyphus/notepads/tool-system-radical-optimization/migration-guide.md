# 迁移指南 - 从Trait系统到Enum系统

## 概述

**目标**: 帮助开发者将代码从旧trait系统迁移到新enum系统  
**适用版本**: workflow-toolkit v0.2.0+  
**难度**: 中等（有兼容性层支持平滑迁移）  
**预计时间**: 1-2小时/模块  

## 迁移策略

### 策略1: 使用兼容性层（推荐短期）
**适用场景**: 
- 需要快速恢复编译
- 不想立即重写大量代码
- 等待新系统稳定

**方法**: 
- 继续使用旧trait API
- 兼容性层会自动适配
- 逐步迁移到新API

### 策略2: 直接迁移（推荐长期）
**适用场景**:
- 新模块开发
- 重构现有模块
- 追求最佳性能

**方法**:
- 直接使用新enum API
- 获得完整性能提升
- 更简洁的代码

---

## 详细迁移步骤

### 1. 工具定义迁移

#### 旧代码 (Trait系统)
```rust
use workflow_toolkit::tools::{ToolNode, ToolExecutor, BasicTool};
use std::sync::Arc;

// 定义执行器
pub struct MyToolExecutor;

#[async_trait]
impl ToolExecutor for MyToolExecutor {
    async fn execute(&self, params: Value, ctx: ExecutionContext) -> Result<Value> {
        // 执行逻辑
        Ok(params)
    }
}

// 创建工具
let executor = Arc::new(MyToolExecutor);
let tool = BasicTool::new(tool_info, executor, None)?;
```

#### 新代码 (Enum系统)
```rust
use workflow_toolkit::tools::{NativeToolBuilder, ToolInput, ToolOutput};

// 直接使用Builder创建工具
let tool = NativeToolBuilder::new()
    .name("my_tool")
    .version("1.0.0")
    .description("My tool description")
    .executor(|input: ToolInput, _ctx| async move {
        // 执行逻辑
        Ok(ToolOutput::success(input.params))
    })
    .build()?;
```

**关键变化**:
- ❌ 不再需要单独的Executor结构体
- ❌ 不再需要`#[async_trait]`
- ✅ 使用闭包定义执行逻辑
- ✅ 更简洁的代码

### 2. 工具注册迁移

#### 旧代码
```rust
use workflow_toolkit::tools::{ToolRegistry, ToolNode};
use std::sync::Arc;

// 创建注册表
trait MyToolRegistry: ToolRegistry {}

// 注册工具
registry.register_tool(Arc::new(tool)).await?;

// 获取工具
let tool: Arc<dyn ToolNode> = registry.get_tool("tool_name").await.unwrap();
```

#### 新代码
```rust
use workflow_toolkit::tools::{ToolRegistry, Tool, ToolId};

// 创建注册表（具体类型，非trait）
let registry = ToolRegistry::new();

// 注册工具
let tool_id: ToolId = registry.register(tool);

// 获取工具（返回Tool枚举，非Arc<dyn>）
let tool: Tool = registry.get("tool_name").unwrap();

// 执行
let output = tool.execute(input, ctx).await?;
```

**关键变化**:
- ❌ 不再使用`Arc<dyn ToolNode>`
- ❌ 不再使用async注册
- ✅ 使用`Tool`枚举（静态分发）
- ✅ O(1)查找性能

### 3. 工具组合迁移

#### 旧代码
```rust
use workflow_toolkit::tools::{ToolChain, ComposableTool};

// 创建工具链
let chain = ToolChain::new("my_chain", "Description")
    .add_step("step1", Arc::new(tool1))
    .add_step("step2", Arc::new(tool2));

// 执行
let result = chain.execute(params, ctx).await?;
```

#### 新代码
```rust
use workflow_toolkit::tools::{ComposedTool, CompositionType, ToolId};

// 创建组合工具
let composed = ComposedTool {
    id: ToolId::new(),
    metadata: Arc::new(metadata),
    composition_type: CompositionType::Chain(vec![tool1_id, tool2_id]),
    tools: vec![tool1_id, tool2_id],
    middleware_stack: None,
};

// 包装为Tool枚举
let tool = Tool::Composed(Arc::new(composed));

// 执行
let output = tool.execute(input, ctx).await?;
```

**关键变化**:
- ❌ `ToolChain`结构体已弃用
- ❌ 不再直接存储工具引用
- ✅ 使用`ToolId`引用
- ✅ 通过注册表解析工具

### 4. 中间件添加

#### 新功能（旧系统不支持）
```rust
use workflow_toolkit::tools::{
    MiddlewareStack, LoggingMiddleware, TimingMiddleware
};

// 创建中间件栈
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(LoggingMiddleware::new()));
stack.add(Arc::new(TimingMiddleware::new()));

// 为工具添加中间件
let tool = tool.with_middleware(stack);
```

**说明**: 中间件系统是新功能，旧系统没有对应概念。

---

## 常见模式迁移

### 模式1: 简单工具

**旧代码**:
```rust
let tool = BasicTool::new(info, Arc::new(executor), None)?;
```

**新代码**:
```rust
let tool = NativeToolBuilder::new()
    .name("tool_name")
    .version("1.0.0")
    .executor(|input, _ctx| async move {
        Ok(ToolOutput::success(input.params))
    })
    .build()?;
```

### 模式2: 带验证的工具

**旧代码**:
```rust
impl ToolExecutor for MyExecutor {
    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // 验证逻辑
        Ok(())
    }
    
    async fn execute(&self, params: Value, ctx: ExecutionContext) -> Result<Value> {
        self.validate_parameters(&params)?;
        // 执行
    }
}
```

**新代码**:
```rust
let tool = NativeToolBuilder::new()
    .name("tool_name")
    .executor(|input, _ctx| async move {
        // 在闭包内验证
        if !input.params.is_object() {
            return Err(WorkflowError::ValidationError(
                "Params must be object".to_string()
            ));
        }
        // 执行
        Ok(ToolOutput::success(input.params))
    })
    .build()?;
```

### 模式3: 插件工具注册

**旧代码**:
```rust
impl Plugin for MyPlugin {
    async fn register_tools(&self, registry: Arc<dyn ToolRegistry>) -> Result<()> {
        for tool in self.tools {
            registry.register_tool(Arc::new(tool)).await?;
        }
        Ok(())
    }
}
```

**新代码**:
```rust
impl Plugin for MyPlugin {
    fn register_tools(&self, registry: &mut ToolRegistry) -> Result<()> {
        for tool in self.tools {
            registry.register(tool); // 非async
        }
        Ok(())
    }
}
```

---

## 兼容性层使用

### 何时使用兼容性层

**使用场景**:
- 旧代码需要快速恢复编译
- 第三方插件暂时无法更新
- 过渡期间保持功能可用

**如何使用**:
```rust
// 旧trait仍然可用（通过compat模块）
use workflow_toolkit::tools::compat::{ToolNode, ToolExecutor, ToolRegistry};

// 继续使用旧API
#[async_trait]
impl ToolNode for MyTool { ... }
```

**限制**:
- 性能不如新系统
- 部分新功能不可用
- 计划在未来版本移除

---

## 性能优化建议

### 1. 使用枚举匹配而非动态分发

**优化前**:
```rust
let tool: Arc<dyn ToolNode> = registry.get_tool(name).unwrap();
tool.execute(params, ctx).await?; // 虚表调用
```

**优化后**:
```rust
let tool: Tool = registry.get(name).unwrap();
tool.execute(input, ctx).await?; // 枚举匹配
```

### 2. 批量操作使用并行

```rust
use tokio::join;

let results = join_all(tools.iter().map(|tool| {
    let tool = tool.clone();
    async move {
        tool.execute(input.clone(), ctx.clone()).await
    }
})).await;
```

### 3. 缓存工具元数据

```rust
// 注册表已内置元数据缓存
let metadata = registry.get_metadata("tool_name"); // O(1)
```

---

## 调试技巧

### 1. 启用日志中间件

```rust
let tool = tool.with_middleware(
    MiddlewareStack::new()
        .with(Arc::new(LoggingMiddleware::new()))
);
```

### 2. 使用计时中间件

```rust
let tool = tool.with_middleware(
    MiddlewareStack::new()
        .with(Arc::new(TimingMiddleware::new()))
);
```

### 3. 检查工具注册

```rust
// 列出所有注册的工具
for name in registry.list_names() {
    println!("Registered tool: {}", name);
}
```

---

## 常见问题 (FAQ)

### Q1: 旧代码还能编译吗？
**A**: 可以，通过兼容性层。建议逐步迁移。

### Q2: 性能提升多少？
**A**: 预期30-50%，主要来自：
- O(1)查找（vs O(n)）
- 静态分发（vs 虚表）
- 无锁并发（vs Mutex）

### Q3: 需要重写所有代码吗？
**A**: 不需要。可以：
1. 使用兼容性层保持旧代码运行
2. 新代码使用新API
3. 逐步迁移旧代码

### Q4: 中间件系统必须使用吗？
**A**: 不是必须的。工具默认没有中间件，可以按需添加。

### Q5: 组合工具（Chain/Conditional/Parallel）还能用吗？
**A**: 可以，但API有变化：
- 旧：直接存储工具引用
- 新：通过ToolId引用，从注册表解析

---

## 迁移检查清单

### 准备阶段
- [ ] 阅读本迁移指南
- [ ] 备份现有代码
- [ ] 确定迁移策略（兼容层或直接迁移）

### 迁移阶段
- [ ] 更新Cargo.toml（如有需要）
- [ ] 修改工具定义代码
- [ ] 修改工具注册代码
- [ ] 修改工具组合代码
- [ ] 添加中间件（可选）

### 验证阶段
- [ ] 编译通过
- [ ] 单元测试通过
- [ ] 集成测试通过
- [ ] 性能测试达标

### 清理阶段
- [ ] 移除兼容性层代码（如使用直接迁移）
- [ ] 更新文档
- [ ] 代码审查

---

## 示例项目

### 完整迁移示例

见 `examples/tool_migration_example.rs`（待创建）

### 最小可运行示例

```rust
use workflow_toolkit::tools::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建注册表
    let registry = ToolRegistry::new();
    
    // 2. 创建工具
    let tool = NativeToolBuilder::new()
        .name("echo")
        .version("1.0.0")
        .executor(|input, _ctx| async move {
            Ok(ToolOutput::success(input.params))
        })
        .build()?;
    
    // 3. 注册工具
    registry.register(tool);
    
    // 4. 执行
    let input = ToolInput::new(json!({"message": "Hello"}));
    let output = registry.execute("echo", input).await?;
    
    println!("Result: {:?}", output.result);
    Ok(())
}
```

---

## 获取帮助

### 资源
- [API文档](docs/api.md)
- [示例代码](examples/)
- [测试用例](tests/)

### 支持
- 提交Issue: [GitHub Issues](https://github.com/...)
- 讨论区: [GitHub Discussions](https://github.com/...)

---

**文档版本**: 1.0  
**最后更新**: 2026-02-01  
**作者**: Atlas Orchestrator
