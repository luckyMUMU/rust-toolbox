# 完整重构执行计划

## 🎯 目标
彻底移除工具系统的旧trait兼容层，完成向纯Enum工具系统的迁移

## 📊 当前状态
- **编译状态**: ✅ 0错误，60警告
- **迁移进度**: ~70%
- **预计工作量**: 5-8小时
- **风险等级**: 低（内部实现变更，不影响公共API）

---

## 🌊 Wave 1: 插件内部结构重命名（2-3小时）

### 任务 1.1: Python插件重构
**文件**: `src/plugins/python.rs`

**修改内容**:
```rust
// 重命名 struct
pub struct PythonToolNode { ... }  
// 改为
pub struct PythonPluginImpl { ... }

// 更新所有impl块
impl PythonToolNode { ... }
// 改为
impl PythonPluginImpl { ... }

// 更新get_tools()实现
fn get_tools(&self) -> Vec<Tool> {
    // 确保使用NativeToolBuilder创建工具
    // 返回Tool::Native(Arc::new(native_tool))
}
```

**验证**: `cargo check --lib` 通过

---

### 任务 1.2: Docker插件重构
**文件**: `src/plugins/docker.rs`

**修改内容**:
```rust
pub struct DockerToolNode { ... }
// 改为
pub struct DockerPluginImpl { ... }
```

**验证**: `cargo check --lib` 通过

---

### 任务 1.3: Node.js插件重构
**文件**: `src/plugins/nodejs.rs`

**修改内容**:
```rust
pub struct NodeJsToolNode { ... }
// 改为
pub struct NodeJsPluginImpl { ... }
```

**验证**: `cargo check --lib` 通过

---

### 任务 1.4: Native插件重构
**文件**: `src/plugins/native.rs`

**修改内容**:
```rust
pub struct NativeToolNode { ... }
// 改为
pub struct NativePluginImpl { ... }
```

**验证**: `cargo check --lib` 通过

---

### 任务 1.5: 更新PluginType枚举
**文件**: `src/plugins/types.rs`

**修改内容**:
```rust
// 更新内部类型引用
inner: Option<crate::plugins::python::PythonPlugin>,
// 确保指向正确的类型
```

**验证**: `cargo check --lib` 通过

---

## 🌊 Wave 2: 依赖文件更新（1-2小时）

### 任务 2.1: 更新插件管理器
**文件**: `src/plugins/manager.rs`

**检查点**:
- 更新对插件工具获取的调用
- 确保与新的struct名称兼容

**验证**: `cargo check --lib` 通过

---

### 任务 2.2: 更新集成模块
**文件**: `src/plugins/integration.rs`

**检查点**:
- 更新集成逻辑中的类型引用

**验证**: `cargo check --lib` 通过

---

### 任务 2.3: 更新运行时模块
**文件**: `src/plugins/runtime.rs`

**检查点**:
- 检查是否有遗留的旧类型引用

**验证**: `cargo check --lib` 通过

---

## 🌊 Wave 3: 文档清理（1小时）

### 任务 3.1: 更新库文档
**文件**: `src/lib.rs`

**修改内容**:
```rust
// 更新架构概述注释
/// ### Tool System
/// - **ToolRegistry**: Registry for all available tools
/// - **Tool**: Enum-based tool types (Native, Python, Node.js, Docker, WASM)
/// - **Template System**: Reusable tool configurations
```

---

### 任务 3.2: 更新工具设计文档
**文件**: `src/tools/design.md`

**修改内容**:
- 移除ToolNode相关设计
- 更新为Enum-based架构描述
- 添加迁移说明

---

### 任务 3.3: 更新工作流设计文档
**文件**: `src/workflow/design.md`

**修改内容**:
- 更新工具执行描述
- 移除trait相关设计

---

### 任务 3.4: 清理模块级注释
**文件**: 多个文件

**检查清单**:
- [ ] `src/tools/mod.rs`
- [ ] `src/plugins/mod.rs`
- [ ] `src/workflow/mod.rs`

**验证**: 无"ToolNode"、"BasicTool"、"compat"残留

---

## 🌊 Wave 4: 测试验证（1-2小时）

### 任务 4.1: 编译验证
```bash
cargo build --lib
cargo build --bins
cargo build --examples
```

**成功标准**: 0错误

---

### 任务 4.2: 测试运行
```bash
cargo test --lib
cargo test --integration
```

**成功标准**: 所有测试通过

---

### 任务 4.3: 旧引用检查
```bash
# 检查是否还有旧引用
grep -r "ToolNode" src/ || echo "✅ No ToolNode references"
grep -r "BasicTool" src/ || echo "✅ No BasicTool references"
grep -r "compat::" src/ || echo "✅ No compat references"
```

**成功标准**: 无残留

---

### 任务 4.4: Clippy检查
```bash
cargo clippy -- -D warnings
```

**成功标准**: 无警告（或接受现有警告）

---

## 📁 文件修改清单

| Wave | 文件 | 修改类型 | 预计时间 |
|------|------|----------|----------|
| 1 | src/plugins/python.rs | 重命名+更新 | 30-45min |
| 1 | src/plugins/docker.rs | 重命名+更新 | 30-45min |
| 1 | src/plugins/nodejs.rs | 重命名+更新 | 30-45min |
| 1 | src/plugins/native.rs | 重命名+更新 | 30-45min |
| 1 | src/plugins/types.rs | 更新引用 | 15min |
| 2 | src/plugins/manager.rs | 检查更新 | 20min |
| 2 | src/plugins/integration.rs | 检查更新 | 20min |
| 2 | src/plugins/runtime.rs | 检查更新 | 20min |
| 3 | src/lib.rs | 文档更新 | 15min |
| 3 | src/tools/design.md | 文档更新 | 15min |
| 3 | src/workflow/design.md | 文档更新 | 15min |
| 4 | 测试验证 | 全面检查 | 1-2hour |

---

## ⚠️ 风险评估与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 编译失败 | 低 | 中 | 每个Wave后验证编译 |
| 测试失败 | 中 | 中 | 保留原代码备份 |
| 命名冲突 | 低 | 低 | 使用Impl后缀 |
| 文档不一致 | 中 | 低 | Wave 3专门处理 |

---

## ✅ 成功标准

- [ ] `cargo build` 0错误
- [ ] `cargo test` 100%通过
- [ ] 无`ToolNode`残留
- [ ] 无`BasicTool`残留
- [ ] 无`compat::`残留
- [ ] 文档更新完成
- [ ] Clippy检查通过

---

## 🔄 回滚策略

每个Wave完成后执行：
```bash
git add .
git commit -m "Wave X: 描述"
```

如出现问题可快速回滚：
```bash
git revert HEAD
```

---

## 📝 Commit建议

```
Wave 1: refactor(plugins): rename *ToolNode to *PluginImpl
Wave 2: refactor(plugins): update dependent modules
Wave 3: docs: update architecture documentation
Wave 4: test: verify all tests pass after migration
Final: chore: cleanup remaining references
```

---

**计划生成时间**: 2026-02-08
**建议启动命令**: `/start-work` 或确认后开始执行