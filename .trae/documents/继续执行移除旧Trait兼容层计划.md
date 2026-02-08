## 执行计划：继续移除旧 Trait 兼容层

### 当前状态
- Wave 1 (基础设施): ✅ 已完成
- Wave 3 (ToolRegistry修复): ✅ 已完成  
- Wave 2/4/5/6: ⏸️ 待执行

### 编译错误统计
- 14个 `impl ToolNode for` 待移除
- 3个 `impl ToolExecutor for` 待移除
- 多处 `Vec<Arc<dyn ToolNode>>` 类型待修改

### 执行步骤

#### Phase 1: 修改 docker.rs
1. 删除第905-947行 `impl ToolNode for DockerToolNode`
2. 删除第971-995行 `impl ToolExecutor for DockerToolExecutor`
3. 修改第1005行字段类型 `Vec<Arc<dyn ToolNode>>` → `Vec<Tool>`
4. 重写 `get_tools()` 方法，使用 `Tool::Docker(DockerTool::new(...))`

#### Phase 2: 修改 python.rs
1. 删除第497-552行 `impl ToolNode for PythonToolNode`
2. 删除第576-608行 `impl ToolExecutor for PythonToolExecutor`
3. 修改字段类型和 `get_tools()` 方法
4. 使用 `Tool::Python(PythonTool::new(...))`

#### Phase 3: 修改 nodejs.rs
1. 删除第674-732行 `impl ToolNode for NodeJsToolNode`
2. 删除第756-788行 `impl ToolExecutor for NodeJsToolExecutor`
3. 修改字段类型和 `get_tools()` 方法
4. 使用 `Tool::NodeJs(NodeJsTool::new(...))`

#### Phase 4: 修改 classification_flow.rs
1. 删除11个 `impl ToolNode for` 代码块
2. 将每个工具改为创建函数模式
3. 使用 `Tool::Native(NativeTool::new(...))`

#### Phase 5: 编译验证
1. `cargo check` 验证无错误
2. `cargo build --release` 完整构建
3. `cargo test` 运行测试

### 迁移模式
```rust
// 旧模式
impl ToolNode for XxxTool { ... }
fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> { ... }

// 新模式
fn get_tools(&self) -> Vec<Tool> {
    let tool = XxxTool::new(...);
    vec![Tool::Xxx(Arc::new(tool))]
}
```

### 成功标准
- [ ] 0个 `impl ToolNode for` 残留
- [ ] 0个 `dyn ToolNode` 使用
- [ ] `cargo build` 0错误
- [ ] `cargo test` 全部通过