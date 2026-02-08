# 工作计划：彻底移除旧 Trait 兼容层，迁移到纯 Enum 工具系统

## TL;DR

**目标**: 完全删除工具系统的旧 trait 兼容层（`compat` 模块和 `node` 模块），将所有代码迁移到新 enum 系统（`Tool` enum 和 `NativeTool`）。

**范围**: 涉及 15+ 文件的重大重构，包括插件系统、工作流组件、测试文件。

**策略**: 
1. 不保留任何兼容层（激进迁移）
2. 直接删除 `compat.rs` 和 `node.rs`
3. 更新所有使用旧 trait 的代码
4. 修改 domain 层 port 定义

**预期工作量**: 大型任务（Large），预计 2-3 天

---

## Context

### 当前状态

代码库中工具系统正在从 trait-based 向 enum-based 迁移：

**旧系统（待删除）**:
- `src/tools/compat.rs` (346 行) - 兼容层，包含 `ToolNode`, `ToolExecutor`, `ToolRegistry` trait
- `src/tools/node.rs` (314 行) - `BasicTool`, `BasicToolBuilder` 等旧实现
- `src/domain/port/tool_registry.rs` - domain 层旧 trait 定义

**新系统（保留）**:
- `src/tools/types.rs` - `Tool` enum, `NativeTool`, `ToolInput`, `ToolOutput`
- `src/tools/registry.rs` - 新的 `ToolRegistry` 结构体
- `src/tools/middleware.rs` - 中间件系统

### 影响范围分析

根据代码分析，以下文件需要修改：

| 文件 | 影响内容 | 修改类型 |
|------|----------|----------|
| `src/tools/mod.rs` | 移除 compat 导出 | 删除导出 |
| `src/tools/compat.rs` | 整个文件删除 | 删除文件 |
| `src/tools/node.rs` | 整个文件删除 | 删除文件 |
| `src/domain/port/tool_registry.rs` | 移除 ToolNode/ToolRegistry trait | 删除代码 |
| `src/domain/port/plugin_manager.rs` | 更新 get_tools 返回类型 | 修改签名 |
| `src/plugins/python.rs` | 迁移 BasicTool → NativeTool | 重写 |
| `src/plugins/nodejs.rs` | 迁移 BasicTool → NativeTool | 重写 |
| `src/plugins/docker.rs` | 迁移 BasicTool → NativeTool | 重写 |
| `src/plugins/native.rs` | 迁移 BasicTool → NativeTool | 重写 |
| `src/plugins/manager.rs` | 移除 tool_to_trait_object 使用 | 修改 |
| `src/plugins/file_management/plugin.rs` | 迁移到 enum 系统 | 重写 |
| `src/plugins/file_management/utils/registry.rs` | 迁移所有工具注册 | 重写 |
| `src/plugins/file_management/batch/batch_processor.rs` | 更新使用方式 | 修改 |
| `src/plugins/file_management/classification/*.rs` | 迁移 ToolNode 实现 | 重写 |
| `src/plugins/file_management/ui/human_decision_tool.rs` | 迁移 BasicTool 使用 | 重写 |
| `src/workflow/component/tool.rs` | 更新工具组件 | 修改 |
| `src/workflow/retry_tests.rs` | 更新测试 | 修改 |
| `src/main.rs` | 更新 BasicToolRegistry 使用 | 修改 |
| `src/lib.rs` | 更新导出 | 修改 |
| `tests/contract/mod.rs` | 更新测试 | 修改 |

---

## 工作分解

### Wave 1: 核心 Trait 和 Domain 层清理

**目标**: 删除核心 trait 定义，清理 domain 层

**任务 1.1**: 删除 `src/tools/compat.rs` 文件
- 删除整个文件
- 这是 346 行的兼容层代码

**任务 1.2**: 删除 `src/tools/node.rs` 文件
- 删除整个文件
- 包含 `BasicTool`, `BasicToolBuilder`, `AsyncFunctionExecutor`, `FunctionExecutor`

**任务 1.3**: 更新 `src/tools/mod.rs` 导出
- 移除 `pub mod compat;`
- 移除 `pub mod node;`（如果单独存在）
- 移除所有 compat 导出：`ToolNode`, `ToolExecutor`, `ToolRegistry`, `BasicToolRegistry`, `ComposableTool`
- 移除 `BasicTool`, `BasicToolBuilder` 导出
- 移除兼容性类型别名：`ToolEnum`, `ToolInputStruct`, `ToolOutputStruct`

**任务 1.4**: 清理 `src/domain/port/tool_registry.rs`
- 删除 `ToolNode` trait 定义
- 删除 `ToolRegistry` trait 定义
- 如果 domain 层需要保留接口，改为使用新系统的类型

**任务 1.5**: 更新 `src/domain/port/plugin_manager.rs`
- 修改 `get_tools()` 签名：返回 `Vec<Tool>` 而不是 `Vec<Arc<dyn ToolNode>>`
- 移除对旧 trait 的依赖

---

### Wave 2: 插件系统迁移

**目标**: 将所有插件从 BasicTool 迁移到 NativeTool

**任务 2.1**: 迁移 `src/plugins/python.rs`
- 移除 `BasicTool`, `ToolExecutor`, `ToolNode` 导入
- 移除 `compat::tool_node_to_enum` 使用
- 修改 `PythonToolNode`：不再实现 `ToolNode` trait
- 修改 `get_tools()`：直接创建 `Tool::Native(NativeTool::new(...))`
- 更新工具创建逻辑：使用 `NativeToolBuilder` 替代 `BasicTool::from_executor`

**任务 2.2**: 迁移 `src/plugins/nodejs.rs`
- 同 Python 插件的迁移步骤
- 修改 `NodeJsToolNode` 实现
- 更新 `get_tools()` 方法

**任务 2.3**: 迁移 `src/plugins/docker.rs`
- 同上述步骤
- 修改 `DockerToolNode` 实现

**任务 2.4**: 迁移 `src/plugins/native.rs`
- 移除 `BasicTool` 使用
- 修改工具创建逻辑

**任务 2.5**: 更新 `src/plugins/manager.rs`
- 移除 `tool_to_trait_object` 使用
- 更新工具管理逻辑

**任务 2.6**: 更新 `src/plugins/types.rs`
- 修改 `Plugin` trait 的 `get_tools()` 方法签名
- 更新 `NativePlugin`, `PythonPlugin` 等的内部存储

---

### Wave 3: 文件管理插件迁移

**目标**: 迁移最复杂的 file_management 模块

**任务 3.1**: 迁移 `src/plugins/file_management/utils/registry.rs`
- 这是一个工具注册表，有大量 `BasicTool` 使用
- 将所有工具创建改为 `NativeToolBuilder`
- 更新 `registered_tools` 存储类型

**任务 3.2**: 迁移 `src/plugins/file_management/plugin.rs`
- 更新 `get_tools()` 实现
- 移除 `ToolNode` 依赖

**任务 3.3**: 迁移分类工具 (`classification/*.rs`)
- 每个文件都实现了 `ToolNode` trait
- 改为创建返回 `Tool` 的函数
- 示例转换：
  ```rust
  // 旧代码
  impl ToolNode for RuleLoaderTool { ... }
  
  // 新代码
  pub fn create_rule_loader_tool() -> Tool {
      Tool::Native(Arc::new(NativeTool::new(...)))
  }
  ```

**任务 3.4**: 迁移 `src/plugins/file_management/ui/human_decision_tool.rs`
- 移除 `compat::ToolNode` 使用
- 迁移 `BasicTool` 创建

**任务 3.5**: 更新 `src/plugins/file_management/batch/batch_processor.rs`
- 更新对 `ToolNode`, `ToolRegistry` 的引用

---

### Wave 4: 工作流和组件层更新

**目标**: 更新工作流执行系统

**任务 4.1**: 更新 `src/workflow/component/tool.rs`
- 移除 `BasicTool`, `BasicToolRegistry` 使用
- 更新为使用新的 `Tool` 和 `ToolRegistry`

**任务 4.2**: 更新 `src/workflow/retry_tests.rs`
- 更新测试代码中的 `ToolNode` 使用

**任务 4.3**: 更新 `src/tools/registry.rs`（如果需要）
- 移除 `compat::ToolRegistry` trait 实现

---

### Wave 5: 应用程序入口更新

**目标**: 更新主程序和库导出

**任务 5.1**: 更新 `src/main.rs`
- 替换 `BasicToolRegistry` 使用
- 更新工具注册逻辑

**任务 5.2**: 更新 `src/lib.rs`
- 移除 `ToolNode`, `ToolRegistry`（旧 trait）导出
- 更新文档注释
- 确保只导出新的 enum 系统

**任务 5.3**: 更新 `tests/contract/mod.rs`
- 更新测试代码

---

## 关键技术决策

### 1. Tool 创建方式

**统一使用 NativeToolBuilder**:
```rust
let tool = NativeToolBuilder::new()
    .name("my_tool")
    .version("1.0.0")
    .description("Tool description")
    .executor(|input, ctx| async move {
        // 执行逻辑
        Ok(ToolOutput::success(result))
    })
    .build()?;

// 注册到 registry
tool_registry.register("my_tool", Tool::Native(Arc::new(tool)));
```

### 2. 插件 get_tools() 实现模式

**新模式**:
```rust
impl Plugin for MyPlugin {
    fn get_tools(&self) -> Vec<Tool> {
        vec![
            self.create_tool_1(),
            self.create_tool_2(),
            // ...
        ]
    }
}

impl MyPlugin {
    fn create_tool_1(&self) -> Tool {
        let native_tool = NativeToolBuilder::new()
            .name("tool_1")
            .version("1.0.0")
            .executor(|input, ctx| async move { ... })
            .build()
            .unwrap();
        Tool::Native(Arc::new(native_tool))
    }
}
```

### 3. 中间件使用

如果需要中间件，使用 `with_middleware`:
```rust
let tool = NativeToolBuilder::new()
    .name("my_tool")
    .executor(|input, ctx| async move { ... })
    .build()?
    .with_middleware(create_middleware_stack());
```

---

## 验证策略

### 编译验证
每个 Wave 完成后必须能够通过编译：
```bash
cargo check
cargo build
```

### 测试验证
全部完成后运行测试：
```bash
cargo test
```

### 关键检查点
1. 无 `compat::` 导入残留
2. 无 `ToolNode` trait 实现
3. 无 `BasicTool` 使用
4. 所有 `get_tools()` 返回 `Vec<Tool>`
5. `ToolRegistry` 使用新的结构体而非 trait

---

## 成功标准

- [ ] `src/tools/compat.rs` 文件被删除
- [ ] `src/tools/node.rs` 文件被删除
- [ ] 代码库中无 `compat::` 导入
- [ ] 代码库中无 `ToolNode` trait 实现
- [ ] 代码库中无 `BasicTool` 使用
- [ ] 所有插件 `get_tools()` 返回 `Vec<Tool>`
- [ ] 编译通过：`cargo build` 成功
- [ ] 测试通过：`cargo test` 通过
- [ ] `src/lib.rs` 只导出新的 enum 系统

---

## Commit 策略

建议每个 Wave 完成后提交一次：

```
Wave 1: refactor(tools): remove compat and node modules, clean domain layer
Wave 2: refactor(plugins): migrate all plugins to enum-based Tool system
Wave 3: refactor(file-management): migrate file management tools to new system
Wave 4: refactor(workflow): update workflow components to use new Tool system
Wave 5: refactor(app): update main entry points and library exports
Final: test(tools): verify all tests pass after migration
```

---

## 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| file_management 模块改动量大 | 高 | 中 | 逐个文件迁移，充分测试 |
| 测试覆盖不足 | 中 | 高 | 手动验证关键路径 |
| 运行时行为变化 | 低 | 高 | 代码审查，集成测试 |
| 编译错误难以解决 | 中 | 中 | 按 Wave 渐进式修改 |

---

**计划生成时间**: 2026-02-08  
**建议执行方式**: `/start-work` 启动 Sisyphus 执行  
**预期完成时间**: 2-3 天
