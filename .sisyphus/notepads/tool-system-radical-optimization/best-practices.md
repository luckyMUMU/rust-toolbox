# 工具系统最佳实践指南

## 概述

本文档提供工具系统激进优化后的最佳实践建议，帮助开发者编写高效、可维护的工具代码。

**版本**: 0.2.0-alpha  
**最后更新**: 2026-02-01  

---

## 工具设计最佳实践

### 1. 单一职责原则

**原则**: 每个工具只做一件事，做好一件事。

**推荐**:
```rust
// ✅ 好 - 单一职责
let read_file_tool = NativeToolBuilder::new()
    .name("read_file")
    .description("Reads a file and returns its contents")
    .executor(|input, _ctx| async move {
        let path = input.params["path"].as_str().unwrap();
        let content = tokio::fs::read_to_string(path).await?;
        Ok(ToolOutput::success(json!({"content": content})))
    })
    .build()?;

let parse_json_tool = NativeToolBuilder::new()
    .name("parse_json")
    .description("Parses JSON string into object")
    .executor(|input, _ctx| async move {
        let json_str = input.params["json"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(json_str)?;
        Ok(ToolOutput::success(parsed))
    })
    .build()?;
```

**避免**:
```rust
// ❌ 不好 - 职责过多
let file_processor_tool = NativeToolBuilder::new()
    .name("file_processor")
    .description("Reads file, parses JSON, validates, transforms, and saves")
    .executor(|input, _ctx| async move {
        // 做了太多事情...
        // 1. 读取文件
        // 2. 解析JSON
        // 3. 验证数据
        // 4. 转换格式
        // 5. 保存结果
    })
    .build()?;
```

---

### 2. 使用强类型参数

**原则**: 使用`#[derive(ToolInput)]`宏定义强类型参数。

**推荐**:
```rust
use workflow_toolkit::macros::ToolInput;
use serde::{Serialize, Deserialize};

#[derive(ToolInput, Serialize, Deserialize, Debug)]
pub struct FileReadInput {
    #[tool_input(description = "File path to read", required = true)]
    pub path: String,
    
    #[tool_input(description = "Maximum bytes to read", default = 1024)]
    pub max_bytes: usize,
    
    #[tool_input(description = "Encoding", default = "utf-8")]
    pub encoding: String,
}

// 使用
let tool = NativeToolBuilder::new()
    .name("read_file")
    .executor(|input, _ctx| async move {
        let file_input = FileReadInput::from_tool_input(&input)?;
        file_input.validate()?; // 自动验证
        
        // 类型安全的访问
        let path = file_input.path;
        let max_bytes = file_input.max_bytes;
        
        // ...
    })
    .build()?;
```

**优势**:
- 编译时类型检查
- 自动参数验证
- IDE自动完成支持
- 自文档化

---

### 3. 合理的错误处理

**原则**: 提供清晰、可操作的错误信息。

**推荐**:
```rust
.executor(|input, _ctx| async move {
    let path = input.params["path"]
        .as_str()
        .ok_or_else(|| WorkflowError::ValidationError(
            "Parameter 'path' must be a string".to_string()
        ))?;
    
    // 检查文件存在
    if !std::path::Path::new(path).exists() {
        return Err(WorkflowError::NotFound {
            resource: format!("File '{}'", path),
        });
    }
    
    // 读取文件
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| WorkflowError::IoError {
            message: format!("Failed to read file '{}': {}", path, e),
        })?;
    
    Ok(ToolOutput::success(json!({"content": content})))
})
```

**错误信息最佳实践**:
- 指明哪个参数/操作出错
- 提供上下文信息
- 建议解决方案

---

### 4. 使用中间件增强功能

**原则**: 通过中间件添加横切关注点，保持工具逻辑纯净。

**推荐**:
```rust
use workflow_toolkit::tools::{
    MiddlewareStack, LoggingMiddleware, TimingMiddleware,
    RetryMiddleware, TimeoutMiddleware
};

// 创建工具
let tool = NativeToolBuilder::new()
    .name("api_call")
    .executor(|input, _ctx| async move {
        // 纯净的业务逻辑
        call_api(input.params).await
    })
    .build()?;

// 添加中间件
let mut stack = MiddlewareStack::new();
stack.add(Arc::new(LoggingMiddleware::new()));
stack.add(Arc::new(TimingMiddleware::new()));
stack.add(Arc::new(RetryMiddleware::new(3)));
stack.add(Arc::new(TimeoutMiddleware::seconds(30)));

let tool = tool.with_middleware(stack);
```

**中间件使用场景**:
- **LoggingMiddleware**: 记录执行日志
- **TimingMiddleware**: 性能监控
- **RetryMiddleware**: 网络请求重试
- **TimeoutMiddleware**: 防止长时间阻塞
- **CircuitBreakerMiddleware**: 故障保护
- **MetricsMiddleware**: 指标收集

---

## 注册表使用最佳实践

### 1. 批量注册工具

**原则**: 批量注册时使用并行处理。

**推荐**:
```rust
use tokio::join;

let tools = vec![
    create_echo_tool()?,
    create_calculator_tool()?,
    create_file_tool()?,
];

// 并行注册
let handles: Vec<_> = tools.into_iter().map(|tool| {
    let registry = registry.clone();
    tokio::spawn(async move {
        registry.register(tool)
    })
}).collect();

let ids: Vec<_> = join_all(handles).await
    .into_iter()
    .filter_map(|r| r.ok())
    .collect();
```

---

### 2. 工具命名规范

**原则**: 使用清晰、一致的命名规范。

**推荐**:
```rust
// ✅ 好 - 清晰、一致
let tool = NativeToolBuilder::new()
    .name("file_read")           // 动词_名词
    .name("json_parse")          // 格式_操作
    .name("data_transform")      // 数据_操作
    .name("http_get")            // 协议_操作
    .build()?;

// ❌ 避免
let tool = NativeToolBuilder::new()
    .name("tool1")               // 无意义
    .name("DoSomething")         // 驼峰命名
    .name("file-utils")          // 包含连字符
    .build()?;
```

**命名规范**:
- 使用`snake_case`
- 动词_名词格式
- 避免缩写
- 保持唯一性

---

### 3. 工具分类和标签

**原则**: 使用分类和标签组织工具。

**推荐**:
```rust
let tool = NativeToolBuilder::new()
    .name("file_read")
    .category("filesystem")           // 大分类
    .tags(vec![                       // 细粒度标签
        "file".to_string(),
        "read".to_string(),
        "io".to_string(),
    ])
    .build()?;
```

**分类建议**:
- `filesystem` - 文件系统操作
- `network` - 网络请求
- `data` - 数据处理
- `math` - 数学运算
- `text` - 文本处理
- `utility` - 通用工具

---

## 性能优化最佳实践

### 1. 避免不必要的克隆

**原则**: 最小化数据克隆。

**推荐**:
```rust
// ✅ 好 - 使用引用
.executor(|input, _ctx| async move {
    let data = &input.params;  // 借用，不克隆
    process_data(data).await
})

// ❌ 避免 - 不必要的克隆
.executor(|input, _ctx| async move {
    let data = input.params.clone();  // 克隆数据
    process_data(&data).await
})
```

---

### 2. 使用缓存

**原则**: 缓存昂贵的计算结果。

**推荐**:
```rust
use moka::future::Cache;

// 创建缓存
let cache: Cache<String, Value> = Cache::new(10_000);

let tool = NativeToolBuilder::new()
    .name("expensive_operation")
    .executor(move |input, _ctx| {
        let cache = cache.clone();
        async move {
            let key = compute_cache_key(&input);
            
            // 尝试从缓存获取
            if let Some(result) = cache.get(&key).await {
                return Ok(ToolOutput::success(result));
            }
            
            // 执行昂贵操作
            let result = expensive_computation(input.params).await?;
            
            // 存入缓存
            cache.insert(key, result.clone()).await;
            
            Ok(ToolOutput::success(result))
        }
    })
    .build()?;
```

---

### 3. 限制并发

**原则**: 使用信号量限制并发数。

**推荐**:
```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10)); // 最多10个并发

let tool = NativeToolBuilder::new()
    .name("api_call")
    .executor(move |input, _ctx| {
        let semaphore = semaphore.clone();
        async move {
            let _permit = semaphore.acquire().await?;
            call_api(input.params).await
        }
    })
    .build()?;
```

---

### 4. 异步I/O

**原则**: 始终使用异步I/O操作。

**推荐**:
```rust
// ✅ 好 - 异步I/O
.executor(|input, _ctx| async move {
    let content = tokio::fs::read_to_string(path).await?;
    let response = reqwest::get(url).await?;
    // ...
})

// ❌ 避免 - 阻塞I/O
.executor(|input, _ctx| async move {
    let content = std::fs::read_to_string(path)?;  // 阻塞！
    // ...
})
```

---

## 测试最佳实践

### 1. 单元测试

**原则**: 为每个工具编写单元测试。

**推荐**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_echo_tool() {
        let tool = create_echo_tool().unwrap();
        let input = ToolInput::new(json!({
            "message": "Hello"
        }));
        
        let output = tool.execute(input, ExecutionContext::default()).await.unwrap();
        
        assert!(output.success);
        assert_eq!(output.result["message"], "Hello");
    }
    
    #[tokio::test]
    async fn test_echo_tool_validation() {
        let tool = create_echo_tool().unwrap();
        let input = ToolInput::new(json!({})); // 缺少必需参数
        
        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
}
```

---

### 2. 使用Mock

**原则**: 使用mock隔离测试。

**推荐**:
```rust
#[tokio::test]
async fn test_with_mock() {
    // 创建mock注册表
    let registry = ToolRegistry::new();
    
    // 注册mock工具
    let mock_tool = NativeToolBuilder::new()
        .name("mock_api")
        .executor(|_input, _ctx| async move {
            Ok(ToolOutput::success(json!({
                "status": "ok",
                "data": "mocked"
            })))
        })
        .build()
        .unwrap();
    
    registry.register(Tool::Native(Arc::new(mock_tool)));
    
    // 测试使用mock工具的业务逻辑
    // ...
}
```

---

### 3. 集成测试

**原则**: 测试工具链和组合。

**推荐**:
```rust
#[tokio::test]
async fn test_tool_chain() {
    let registry = ToolRegistry::new();
    
    // 注册多个工具
    registry.register(create_read_file_tool().unwrap());
    registry.register(create_parse_json_tool().unwrap());
    registry.register(create_transform_tool().unwrap());
    
    // 执行工具链
    let file_content = registry.execute("read_file", input1).await.unwrap();
    let parsed = registry.execute("parse_json", file_content.into()).await.unwrap();
    let transformed = registry.execute("transform", parsed.into()).await.unwrap();
    
    // 验证结果
    assert!(transformed.success);
}
```

---

## 安全最佳实践

### 1. 输入验证

**原则**: 始终验证输入参数。

**推荐**:
```rust
.executor(|input, _ctx| async move {
    // 验证路径不包含遍历
    let path = input.params["path"].as_str().unwrap();
    if path.contains("..") || path.starts_with("/") {
        return Err(WorkflowError::ValidationError(
            "Invalid path: directory traversal not allowed".to_string()
        ));
    }
    
    // 验证文件类型
    if !path.ends_with(".txt") && !path.ends_with(".json") {
        return Err(WorkflowError::ValidationError(
            "Only .txt and .json files are allowed".to_string()
        ));
    }
    
    // ...
})
```

---

### 2. 资源限制

**原则**: 限制资源使用。

**推荐**:
```rust
.executor(|input, _ctx| async move {
    // 限制文件大小
    let max_size = 10 * 1024 * 1024; // 10MB
    let metadata = tokio::fs::metadata(path).await?;
    if metadata.len() > max_size {
        return Err(WorkflowError::ValidationError(
            format!("File too large: {} > {} bytes", metadata.len(), max_size)
        ));
    }
    
    // 限制执行时间
    let timeout = Duration::from_secs(30);
    let result = tokio::time::timeout(timeout, process_data()).await?;
    
    // ...
})
```

---

## 文档最佳实践

### 1. 工具描述

**原则**: 提供清晰、详细的工具描述。

**推荐**:
```rust
let tool = NativeToolBuilder::new()
    .name("csv_to_json")
    .description("Converts CSV file to JSON format")
    .description(r#"
        Converts a CSV file to JSON format.
        
        ## Parameters
        - `input_path`: Path to the CSV file
        - `output_path`: Path for the output JSON file
        - `delimiter`: CSV delimiter (default: ",")
        
        ## Example
        ```json
        {
            "input_path": "data.csv",
            "output_path": "data.json",
            "delimiter": ";"
        }
        ```
        
        ## Returns
        JSON object with conversion statistics.
    "#)
    .build()?;
```

---

### 2. 使用示例

**原则**: 提供使用示例。

**推荐**:
```rust
let metadata = ToolMetadata {
    examples: vec![
        ToolExample {
            title: "Basic usage".to_string(),
            description: "Convert a simple CSV file".to_string(),
            input: json!({
                "input_path": "users.csv",
                "output_path": "users.json"
            }),
            expected_output: json!({
                "status": "success",
                "records_converted": 100
            }),
        },
        ToolExample {
            title: "Custom delimiter".to_string(),
            description: "Use semicolon as delimiter".to_string(),
            input: json!({
                "input_path": "data.csv",
                "output_path": "data.json",
                "delimiter": ";"
            }),
            expected_output: json!({
                "status": "success",
                "records_converted": 50
            }),
        },
    ],
    // ...
};
```

---

## 版本管理最佳实践

### 1. 语义化版本

**原则**: 使用语义化版本（SemVer）。

**推荐**:
```rust
let tool = NativeToolBuilder::new()
    .name("data_processor")
    .version("1.2.3")  // MAJOR.MINOR.PATCH
    .build()?;
```

**版本规则**:
- **MAJOR**: 不兼容的API更改
- **MINOR**: 向后兼容的功能添加
- **PATCH**: 向后兼容的问题修复

---

### 2. 版本兼容性

**原则**: 维护版本兼容性。

**推荐**:
```rust
// 添加新参数时提供默认值
#[derive(ToolInput)]
pub struct ProcessInput {
    pub data: String,
    
    #[tool_input(default = false)]  // 新参数，默认false
    pub verbose: bool,
}
```

---

## 监控和日志最佳实践

### 1. 结构化日志

**原则**: 使用结构化日志。

**推荐**:
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(input), fields(tool_name = "file_processor"))]
.executor(|input, _ctx| async move {
    info!(path = %input.params["path"], "Starting file processing");
    
    match process_file(&input).await {
        Ok(result) => {
            info!(records = result.len(), "File processed successfully");
            Ok(ToolOutput::success(result))
        }
        Err(e) => {
            error!(error = %e, "File processing failed");
            Err(e)
        }
    }
})
```

---

### 2. 指标收集

**原则**: 收集关键指标。

**推荐**:
```rust
use workflow_toolkit::tools::MetricsMiddleware;

let mut stack = MiddlewareStack::new();
stack.add(Arc::new(MetricsMiddleware::new()
    .with_prefix("tool")));

// 自动收集指标:
// - tool_{name}_success_duration_ms
// - tool_{name}_failure_duration_ms
// - tool_{name}_total_calls
```

---

## 总结

### 关键要点

1. **设计**: 单一职责，强类型参数
2. **实现**: 纯净逻辑，中间件增强
3. **性能**: 避免克隆，使用缓存，限制并发
4. **测试**: 单元测试，mock隔离，集成测试
5. **安全**: 输入验证，资源限制
6. **文档**: 清晰描述，使用示例
7. **版本**: 语义化版本，向后兼容
8. **监控**: 结构化日志，指标收集

### 检查清单

工具开发检查清单:
- [ ] 单一职责
- [ ] 强类型参数
- [ ] 错误处理完善
- [ ] 单元测试覆盖
- [ ] 文档完整
- [ ] 性能优化
- [ ] 安全检查
- [ ] 版本管理

---

## 参考资源

- [API参考](./api-reference.md) - 完整API文档
- [迁移指南](./migration-guide.md) - 从旧系统迁移
- [故障排除](./troubleshooting.md) - 常见问题解决
- [新旧对比](./comparison.md) - 架构对比

---

**文档版本**: 1.0  
**最后更新**: 2026-02-01  
**维护者**: Atlas Orchestrator
