# 快速开始指南

## 概述

本文档帮助您快速上手工具系统激进优化后的新版本（0.2.0-alpha）。

**预计时间**: 15分钟  
**难度**: 初级  
**前提**: 熟悉Rust基础

---

## 安装

### 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
workflow-toolkit = { path = "path/to/workflow-toolkit", features = ["macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 启用宏功能

确保启用 `macros` 特性以使用派生宏：

```toml
[dependencies]
workflow-toolkit = { version = "0.2.0-alpha", features = ["macros"] }
```

---

## 5分钟快速入门

### 1. 创建简单工具（2分钟）

```rust
use workflow_toolkit::tools::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建注册表
    let registry = ToolRegistry::new();
    
    // 创建工具
    let tool = NativeToolBuilder::new()
        .name("echo")
        .version("1.0.0")
        .description("Echoes the input")
        .executor(|input, _ctx| async move {
            Ok(ToolOutput::success(input.params))
        })
        .build()?;
    
    // 注册工具
    registry.register(Tool::Native(Arc::new(tool)));
    
    // 执行工具
    let input = ToolInput::new(json!("Hello, World!"));
    let output = registry.execute("echo", input).await?;
    
    println!("Result: {:?}", output.result);
    Ok(())
}
```

**运行**:
```bash
cargo run
```

**输出**:
```
Result: String("Hello, World!")
```

---

### 2. 使用强类型参数（2分钟）

```rust
use workflow_toolkit::tools::*;
use workflow_toolkit::macros::ToolInput;
use serde::{Serialize, Deserialize};
use serde_json::json;

// 定义输入结构
#[derive(ToolInput, Serialize, Deserialize, Debug)]
pub struct GreetInput {
    #[tool_input(description = "Name to greet", required = true)]
    pub name: String,
    
    #[tool_input(description = "Greeting message", default = "Hello")]
    pub greeting: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let registry = ToolRegistry::new();
    
    let tool = NativeToolBuilder::new()
        .name("greet")
        .version("1.0.0")
        .executor(|input, _ctx| async move {
            // 转换为强类型
            let greet_input = GreetInput::from_tool_input(&input)?;
            
            // 验证输入
            greet_input.validate()?;
            
            // 处理
            let message = format!("{}, {}!", greet_input.greeting, greet_input.name);
            
            Ok(ToolOutput::success(json!({"message": message})))
        })
        .build()?;
    
    registry.register(Tool::Native(Arc::new(tool)));
    
    // 使用强类型输入
    let input = GreetInput {
        name: "Alice".to_string(),
        greeting: "Hi".to_string(),
    };
    
    let output = registry.execute("greet", input.into_tool_input()).await?;
    println!("{:?}", output.result);
    
    Ok(())
}
```

**输出**:
```
Object({"message": String("Hi, Alice!")})
```

---

### 3. 添加中间件（1分钟）

```rust
use workflow_toolkit::tools::*;
use workflow_toolkit::tools::{
    MiddlewareStack, LoggingMiddleware, TimingMiddleware
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let registry = ToolRegistry::new();
    
    // 创建中间件栈
    let mut stack = MiddlewareStack::new();
    stack.add(Arc::new(LoggingMiddleware::new()));
    stack.add(Arc::new(TimingMiddleware::new()));
    
    // 创建带中间件的工具
    let tool = NativeToolBuilder::new()
        .name("processed_echo")
        .version("1.0.0")
        .executor(|input, _ctx| async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(ToolOutput::success(input.params))
        })
        .with_middleware(stack)  // 添加中间件
        .build()?;
    
    registry.register(Tool::Native(Arc::new(tool)));
    
    let input = ToolInput::new(json!("test"));
    let output = registry.execute("processed_echo", input).await?;
    
    println!("Result: {:?}", output.result);
    Ok(())
}
```

**输出**:
```
[INFO] Starting execution of tool: processed_echo
[INFO] Tool processed_echo completed successfully in 102ms
Result: String("test")
```

---

## 完整示例（10分钟）

### 构建一个文件处理工作流

```rust
use workflow_toolkit::tools::*;
use workflow_toolkit::macros::ToolInput;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::sync::Arc;

// 定义输入输出类型
#[derive(ToolInput, Serialize, Deserialize)]
pub struct ReadFileInput {
    #[tool_input(description = "File path", required = true)]
    pub path: String,
}

#[derive(ToolOutput, Serialize, Deserialize)]
pub struct ReadFileOutput {
    pub content: String,
    pub lines: usize,
}

#[derive(ToolInput, Serialize, Deserialize)]
pub struct ProcessInput {
    #[tool_input(description = "Content to process", required = true)]
    pub content: String,
    
    #[tool_input(description = "Convert to uppercase", default = false)]
    pub uppercase: bool,
}

#[derive(ToolOutput, Serialize, Deserialize)]
pub struct ProcessOutput {
    pub processed: String,
    pub word_count: usize,
}

// 创建工具
fn create_tools() -> Result<(Tool, Tool)> {
    // 1. 文件读取工具
    let read_tool = NativeToolBuilder::new()
        .name("read_file")
        .version("1.0.0")
        .description("Reads a file and returns its content")
        .category("filesystem")
        .tags(vec!["file".to_string(), "read".to_string()])
        .executor(|input, _ctx| async move {
            let file_input = ReadFileInput::from_tool_input(&input)?;
            
            let content = tokio::fs::read_to_string(&file_input.path)
                .await
                .map_err(|e| WorkflowError::IoError {
                    message: format!("Failed to read file: {}", e),
                })?;
            
            let lines = content.lines().count();
            
            let output = ReadFileOutput {
                content,
                lines,
            };
            
            Ok(output.into_tool_output())
        })
        .build()?;
    
    // 2. 文本处理工具
    let process_tool = NativeToolBuilder::new()
        .name("process_text")
        .version("1.0.0")
        .description("Processes text content")
        .category("text")
        .tags(vec!["text".to_string(), "process".to_string()])
        .executor(|input, _ctx| async move {
            let process_input = ProcessInput::from_tool_input(&input)?;
            
            let processed = if process_input.uppercase {
                process_input.content.to_uppercase()
            } else {
                process_input.content
            };
            
            let word_count = processed.split_whitespace().count();
            
            let output = ProcessOutput {
                processed,
                word_count,
            };
            
            Ok(output.into_tool_output())
        })
        .build()?;
    
    Ok((
        Tool::Native(Arc::new(read_tool)),
        Tool::Native(Arc::new(process_tool)),
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建注册表
    let registry = ToolRegistry::new();
    
    // 创建并注册工具
    let (read_tool, process_tool) = create_tools()?;
    registry.register(read_tool);
    registry.register(process_tool);
    
    println!("Registered tools: {:?}", registry.list_names());
    
    // 执行工作流
    // 步骤1: 读取文件
    let read_input = ReadFileInput {
        path: "example.txt".to_string(),
    };
    
    println!("\nStep 1: Reading file...");
    let read_output = registry.execute("read_file", read_input.into_tool_input()).await?;
    let file_result = ReadFileOutput::from_tool_output(&read_output)?;
    
    println!("File has {} lines", file_result.lines);
    
    // 步骤2: 处理内容
    let process_input = ProcessInput {
        content: file_result.content,
        uppercase: true,
    };
    
    println!("\nStep 2: Processing text...");
    let process_output = registry.execute("process_text", process_input.into_tool_input()).await?;
    let process_result = ProcessOutput::from_tool_output(&process_output)?;
    
    println!("Word count: {}", process_result.word_count);
    println!("Processed (first 100 chars): {}", &process_result.processed[..100.min(process_result.processed.len())]);
    
    Ok(())
}
```

---

## 常用模式

### 模式1: 错误处理

```rust
.executor(|input, _ctx| async move {
    // 验证参数
    let path = input.params["path"]
        .as_str()
        .ok_or_else(|| WorkflowError::ValidationError(
            "Parameter 'path' is required".to_string()
        ))?;
    
    // 执行业务逻辑
    match do_something(path).await {
        Ok(result) => Ok(ToolOutput::success(result)),
        Err(e) => Err(WorkflowError::tool_execution(format!("Failed: {}", e))),
    }
})
```

### 模式2: 超时控制

```rust
use workflow_toolkit::tools::TimeoutMiddleware;
use std::time::Duration;

let mut stack = MiddlewareStack::new();
stack.add(Arc::new(TimeoutMiddleware::new(Duration::from_secs(30))));

let tool = tool.with_middleware(stack);
```

### 模式3: 重试机制

```rust
use workflow_toolkit::tools::RetryMiddleware;

let mut stack = MiddlewareStack::new();
stack.add(Arc::new(RetryMiddleware::new(3))); // 重试3次

let tool = tool.with_middleware(stack);
```

### 模式4: 批量执行

```rust
use tokio::join;

let inputs = vec![input1, input2, input3];

let futures: Vec<_> = inputs.into_iter().map(|input| {
    registry.execute("tool_name", input)
}).collect();

let results = join_all(futures).await;
```

---

## 下一步

### 学习更多

- 📖 [API参考文档](./api-reference.md) - 完整API文档
- 🎯 [最佳实践指南](./best-practices.md) - 开发最佳实践
- 🔄 [迁移指南](./migration-guide.md) - 从旧版本迁移
- 📊 [新旧对比](./comparison.md) - 架构对比

### 查看示例

- [examples/strongly_typed_tools.rs](../../../examples/strongly_typed_tools.rs) - 强类型工具示例
- [examples/](../../../examples/) - 更多示例

### 获取帮助

- 🔧 [故障排除指南](./troubleshooting.md) - 常见问题解决
- 📚 [学习记录](./learnings.md) - 设计决策和经验

---

## 常见问题

### Q: 如何调试工具？

**A**: 使用日志中间件：
```rust
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(LoggingMiddleware::new()));
```

### Q: 如何处理异步操作？

**A**: 执行器已经是异步的：
```rust
.executor(|input, _ctx| async move {
    let result = some_async_function().await?;
    Ok(ToolOutput::success(result))
})
```

### Q: 如何共享状态？

**A**: 使用闭包捕获：
```rust
let shared_data = Arc::new(Mutex::new(data));

.executor(move |input, _ctx| {
    let shared_data = shared_data.clone();
    async move {
        let data = shared_data.lock().await;
        // 使用数据
    }
})
```

### Q: 旧代码还能用吗？

**A**: 可以，使用兼容性层：
```rust
use workflow_toolkit::tools::compat::*;
```

建议逐步迁移到新API。

---

## 总结

恭喜！您已经掌握了工具系统的基本用法：

✅ 创建简单工具  
✅ 使用强类型参数  
✅ 添加中间件  
✅ 构建工作流  

现在您可以开始构建自己的工作流系统了！

---

**文档版本**: 1.0  
**最后更新**: 2026-02-01  
**维护者**: Atlas Orchestrator
