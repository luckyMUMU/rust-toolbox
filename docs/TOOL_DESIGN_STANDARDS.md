# 工具设计标准与架构指南

> **Workflow Toolkit 工具开发规范**  
> **版本**: v0.1.0  
> *最后更新：2026-02-05*

---

## 1. 核心原则：单一职责原则 (SRP)

`workflow-toolkit` 生态系统中的每个工具都必须遵循**单一职责原则**。一个工具应该只做一件事，把它做好，并且做完整。

### 1.1 什么定义了一个工具？
*   **原子操作**：工具执行单一的逻辑工作单元（例如，"读取文件"、"解析JSON"、"发送HTTP请求"）。
*   **无业务逻辑**：工具**不得**包含实现业务规则的复杂分支逻辑（`if/else`）。业务逻辑属于**工作流**编排层。
*   **无状态性**：工具应该是纯函数，即 `输出 = f(输入)`。避免在多次执行之间持久化的内部状态，除非明确设计为存储类工具（如 `DataCacheTool`）。

### 1.2 命名规范
工具名称应清晰描述其动作和目标。

*   **格式**：`名词-动词` 或 `动词-名词`（ID使用kebab-case，结构体使用PascalCase）。
*   **示例**：
    *   ✅ `file-reader` / `FileReader`
    *   ✅ `json-transformer` / `JsonTransformer`
    *   ✅ `directory-scanner` / `DirectoryScanner`
    *   ❌ `process-manager`（过于模糊）
    *   ❌ `file-handler`（它是读取？写入？还是删除？）

## 2. 工具实现指南

### 2.1 `ToolNode` Trait
所有工具都必须实现 `ToolNode` trait。

```rust
#[async_trait]
impl ToolNode for MyTool {
    fn name(&self) -> String { "my-tool".to_string() }
    
    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // 需要严格的schema验证
    }
    
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        // 原子执行逻辑
    }
}
```

### 2.2 错误处理
*   返回特定的 `WorkflowError` 类型。
*   不要吞掉错误；将它们传播给引擎。
*   让工作流引擎处理重试和错误策略。

## 3. 工作流编排

复杂行为通过**组合**简单工具来实现，而不是让工具变得复杂。

### 3.1 条件逻辑
*   **反模式**：工具接收一个布尔值 `compress_files` 参数并在内部分支。
*   **最佳实践**：
    *   节点 A：`FileScanner`
    *   节点 B：`Condition`（检查文件大小）
    *   节点 C：`FileCompressor`（仅在节点 B 为 true 时执行）
    *   节点 D：`FileMover`

### 3.2 数据流
*   工具通过 `parameters` 接收数据。
*   工具通过 `Result<Value>` 返回数据。
*   使用 `DataCacheTool` 在非相邻节点间共享数据。
*   使用 `DataTransformTool` 将工具 A 的输出映射为工具 B 的输入。

## 4. 系统工具

以下标准工具可用于流程控制和数据管理：

*   **`data-cache`**：在执行上下文中存储/检索临时数据。
*   **`data-transform`**：对JSON数据应用类似JQ的转换或模板渲染。

## 5. 审查清单

在提交新工具之前，问自己：
1.  我能否用一句话描述这个工具的功能而不使用"和"？
2.  这个工具是否对*接下来做什么*做出决策？（如果是，将该逻辑移到工作流）。
3.  名称是否足够具体？
