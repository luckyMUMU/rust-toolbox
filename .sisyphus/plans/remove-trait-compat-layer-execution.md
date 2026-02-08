# 工作计划：移除旧 Trait 兼容层 - 执行阶段

## TL;DR

**目标**: 完成工具系统从 trait-based 到 enum-based 的迁移，移除所有剩余的 ToolNode trait 实现和旧 API 使用。

**范围**: 涉及 17+ 文件的代码重构，包括：
- 移除 14 个 ToolNode trait 实现
- 修复 16 处 ToolRegistry 使用模式  
- 修复 18 处 BasicTool 使用

**状态**: Wave 1 已完成（基础设施清理），当前执行 Wave 2-6

**预期工作量**: 中等（Medium），预计 8-12 小时

**并行执行**: YES - 各 Wave 内任务可并行，Wave 间需顺序执行

**关键路径**: 
Wave 2 (移除主要插件 ToolNode) → Wave 3 (修复 ToolRegistry) → Wave 4 (修复 BasicTool) → Wave 5 (移除分类工具 ToolNode) → Wave 6 (验证)

---

## Context

### 已完成工作（Wave 1）

**基础设施清理（100% 完成）**:
- ✅ 删除 `src/tools/compat.rs` (345行代码)
- ✅ 删除 `src/tools/node.rs` (313行代码)
- ✅ 删除重复的 `src/error.rs`

**Domain 层更新（100% 完成）**:
- ✅ `src/domain/port/tool_registry.rs` - 改为新系统 re-export
- ✅ `src/domain/port/plugin_manager.rs` - `get_tools()` 返回 `Vec<Tool>`
- ✅ `src/tools/mod.rs` - 清理所有 compat 导出

**导入修复（约 75% 完成）**:
- ✅ 已修复 16 个文件的导入错误

### 当前代码状态

**编译错误类型**:
1. 14 个 `impl ToolNode for` 需要移除
2. 16 处 `Arc<dyn ToolRegistry>` 需要改为 `Arc<ToolRegistry>`
3. 18 处 `BasicTool` 使用需要改为 `NativeToolBuilder`

### 技术背景

**旧模式（待移除）**:
```rust
// ToolNode trait 实现
#[async_trait]
impl ToolNode for MyTool {
    fn name(&self) -> &str { &self.info.name }
    async fn execute(&self, params: Value, ctx: ExecutionContext) -> Result<Value> { ... }
}

// get_tools 返回 trait object
fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
    vec![Arc::new(MyTool::new(...))]
}

// BasicTool 使用
let tool = BasicTool::from_executor(tool_info, executor, Some(plugin_info))?;
```

**新模式（目标）**:
```rust
// 直接创建 Tool enum
fn get_tools(&self) -> Vec<Tool> {
    let native_tool = NativeToolBuilder::new()
        .name("tool_name")
        .version("1.0.0")
        .executor(|input: ToolInput, ctx| async move {
            // 执行逻辑
            Ok(ToolOutput::success(result))
        })
        .build()
        .unwrap();
    vec![Tool::Native(Arc::new(native_tool))]
}

// ToolRegistry 使用
Arc<ToolRegistry>  // 不是 Arc<dyn ToolRegistry>
```

---

## Work Objectives

### Core Objective
完成工具系统的 trait-to-enum 迁移，确保：
1. 代码库中无任何 `ToolNode` trait 实现
2. 所有 `ToolRegistry` 使用新的 struct 模式
3. 所有工具创建使用 `NativeToolBuilder`
4. 编译通过，测试通过

### Concrete Deliverables
- [ ] 修改 3 个主要插件文件（docker.rs, python.rs, nodejs.rs）
- [ ] 修改 6 个 ToolRegistry 使用文件
- [ ] 修改 8 个 BasicTool 使用文件
- [ ] 修改 classification_flow.rs（11 个 ToolNode 实现）
- [ ] 编译通过：`cargo build` 成功
- [ ] 测试通过：`cargo test` 通过

### Definition of Done
```bash
# 验证无任何旧 trait 残留
grep -r "impl ToolNode for" src/ && echo "FAIL" || echo "PASS"
grep -r "dyn ToolRegistry" src/ && echo "FAIL" || echo "PASS"
grep -r "BasicTool::" src/ && echo "FAIL" || echo "PASS"

# 编译和测试
cargo build --release 2>&1 | grep -q "error" && echo "FAIL" || echo "PASS"
cargo test 2>&1 | grep -q "test result: FAILED" && echo "FAIL" || echo "PASS"
```

### Must Have
- 完全移除 14 个 ToolNode trait 实现
- 修复所有 ToolRegistry 和 BasicTool 使用
- 保持现有功能不变（仅迁移，不修改逻辑）
- 每次修改后验证编译状态

### Must NOT Have (Guardrails)
- 不添加新功能
- 不修改工具执行逻辑（仅改变创建方式）
- 不保留任何 backward compatibility 代码
- 不使用 unsafe 代码

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (Cargo built-in test)
- **Automated tests**: Tests-after (每 Wave 完成后运行测试)
- **Framework**: `cargo test`

### Agent-Executed QA Scenarios

**编译验证场景**:
```
Scenario: 每文件修改后验证编译
  Tool: Bash (cargo)
  Preconditions: 修改单个文件
  Steps:
    1. cargo check 2>&1 | tee /tmp/check_output.txt
    2. grep "error\[E" /tmp/check_output.txt | wc -l
  Expected Result: 错误数不增加（理想为0）
  Evidence: /tmp/check_output.txt
```

**最终验证场景**:
```
Scenario: 完整编译和测试
  Tool: Bash (cargo)
  Preconditions: 所有修改完成
  Steps:
    1. cargo clean
    2. cargo build --release 2>&1 | tee /tmp/build_output.txt
    3. cargo test 2>&1 | tee /tmp/test_output.txt
  Expected Result: 
    - build: 0 errors
    - test: all passed
  Evidence: /tmp/build_output.txt, /tmp/test_output.txt
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 2 (3个文件，可并行):
├── Task 1: 修复 docker.rs
├── Task 2: 修复 python.rs
└── Task 3: 修复 nodejs.rs

Wave 3 (6个文件，可并行):
├── Task 4: 修复 interfaces/mcp.rs
├── Task 5: 修复 interfaces/cli/app.rs
├── Task 6: 修复 workflow/retry_tests.rs
├── Task 7: 修复 interfaces/tui/widgets/tool_manager.rs
├── Task 8: 修复 plugins/manager.rs
└── Task 9: 修复 workflow/component/tool.rs

Wave 4 (8个文件，可并行):
├── Task 10: 修复 main.rs
├── Task 11-18: 修复各插件文件 BasicTool 使用

Wave 5 (1个文件):
└── Task 19: 修复 classification_flow.rs (11个实现)

Wave 6 (验证):
├── Task 20: 编译验证
└── Task 21: 测试验证
```

### Critical Path
Task 1-3 (Wave 2) → Task 4-9 (Wave 3) → Task 10-18 (Wave 4) → Task 19 (Wave 5) → Task 20-21 (Wave 6)

---

## TODOs

### Wave 2: 移除主要插件的 ToolNode 实现

#### Task 1: 移除 docker.rs ToolNode 实现

**What to do**:
1. 删除第 905-947 行的 `impl ToolNode for DockerToolNode` 代码块
2. 修改 `get_tools()` 方法（第 1123 行），改为返回 `Vec<Tool>`
3. 将工具创建逻辑从 `BasicTool::from_executor` 改为 `NativeToolBuilder`

**Must NOT do**:
- 不要修改 Docker 容器的实际执行逻辑
- 不要删除 `DockerToolExecutor` 结构体（保留用于内部使用）
- 不要改变方法签名（除了 get_tools 的返回类型）

**Recommended Agent Profile**:
- **Category**: `quick`（单个文件的确定性修改）
- **Skills**: []
- **Reason**: 这是模式化的重构，有明确的目标模式

**References**:
- 迁移模式见 Context 部分
- 新系统类型: `src/tools/types.rs`

**Acceptance Criteria**:
```bash
# 验证 ToolNode 实现已移除
grep -n "impl ToolNode for DockerToolNode" src/plugins/docker.rs && exit 1 || echo "PASS"

# 验证编译
cargo check 2>&1 | grep "error\[E" | head -5
```

**Commit**: YES
- Message: `refactor(plugins): migrate docker plugin to enum-based Tool system`
- Files: `src/plugins/docker.rs`

---

#### Task 2: 移除 python.rs ToolNode 实现

**What to do**:
1. 删除第 497-539 行的 `impl ToolNode for PythonToolNode` 代码块
2. 修改 `get_tools()` 方法（第 675 行附近），改为返回 `Vec<Tool>`
3. 将 `add_basic_tool` 方法中的 `BasicTool::from_executor` 改为 `NativeToolBuilder`

**Must NOT do**:
- 不要修改 Python 脚本的实际执行逻辑
- 不要改变 `PythonToolExecutor` 的实现

**Recommended Agent Profile**:
- **Category**: `quick`
- **Skills**: []

**Acceptance Criteria**:
```bash
grep -n "impl ToolNode for PythonToolNode" src/plugins/python.rs && exit 1 || echo "PASS"
cargo check 2>&1 | grep "error\[E" | head -5
```

**Commit**: YES
- Message: `refactor(plugins): migrate python plugin to enum-based Tool system`
- Files: `src/plugins/python.rs`

---

#### Task 3: 移除 nodejs.rs ToolNode 实现

**What to do**:
1. 删除第 674-716 行的 `impl ToolNode for NodeJsToolNode` 代码块
2. 修改 `get_tools()` 方法（第 858 行附近），改为返回 `Vec<Tool>`
3. 将 `add_basic_tool` 方法中的 `BasicTool::from_executor` 改为 `NativeToolBuilder`

**Must NOT do**:
- 不要修改 Node.js 脚本的实际执行逻辑

**Recommended Agent Profile**:
- **Category**: `quick`
- **Skills**: []

**Acceptance Criteria**:
```bash
grep -n "impl ToolNode for NodeJsToolNode" src/plugins/nodejs.rs && exit 1 || echo "PASS"
cargo check 2>&1 | grep "error\[E" | head -5
```

**Commit**: YES
- Message: `refactor(plugins): migrate nodejs plugin to enum-based Tool system`
- Files: `src/plugins/nodejs.rs`

---

### Wave 3: 修复 ToolRegistry 使用模式

#### Task 4: 修复 interfaces/mcp.rs

**What to do**:
- 将 3 处 `Arc<dyn ToolRegistry>` 改为 `Arc<ToolRegistry>`（第 103, 151, 312 行）

**Acceptance Criteria**:
```bash
grep -n "dyn ToolRegistry" src/interfaces/mcp.rs && exit 1 || echo "PASS"
```

**Commit**: YES
- Message: `refactor(interfaces): update mcp.rs to use struct ToolRegistry`
- Files: `src/interfaces/mcp.rs`

---

#### Task 5: 修复 interfaces/cli/app.rs

**What to do**:
- 将 3 处 `Arc<dyn ToolRegistry>` 改为 `Arc<ToolRegistry>`（第 88, 115 行）
- 更新相关注释中的说明（第 735, 844, 890 行）

**Acceptance Criteria**:
```bash
grep -n "dyn ToolRegistry" src/interfaces/cli/app.rs && exit 1 || echo "PASS"
```

**Commit**: YES
- Message: `refactor(interfaces): update cli/app.rs to use struct ToolRegistry`
- Files: `src/interfaces/cli/app.rs`

---

#### Task 6: 修复 workflow/retry_tests.rs

**What to do**:
- 将第 162 行 `Arc<dyn ToolRegistry>` 改为 `Arc<ToolRegistry>`

**Acceptance Criteria**:
```bash
grep -n "dyn ToolRegistry" src/workflow/retry_tests.rs && exit 1 || echo "PASS"
```

**Commit**: YES (可合并到 Task 9)
- Message: `refactor(workflow): update retry_tests.rs to use struct ToolRegistry`
- Files: `src/workflow/retry_tests.rs`

---

#### Task 7: 修复 interfaces/tui/widgets/tool_manager.rs

**What to do**:
- 将 2 处 `Arc<dyn ToolRegistry>` 改为 `Arc<ToolRegistry>`（第 20, 40 行）

**Acceptance Criteria**:
```bash
grep -n "dyn ToolRegistry" src/interfaces/tui/widgets/tool_manager.rs && exit 1 || echo "PASS"
```

**Commit**: YES
- Message: `refactor(interfaces): update tool_manager.rs to use struct ToolRegistry`
- Files: `src/interfaces/tui/widgets/tool_manager.rs`

---

#### Task 8: 修复 plugins/manager.rs

**What to do**:
- 将 3 处 `Arc<RwLock<dyn ToolRegistry>>` 改为 `Arc<RwLock<ToolRegistry>>`（第 20, 35, 45 行）

**Acceptance Criteria**:
```bash
grep -n "dyn ToolRegistry" src/plugins/manager.rs && exit 1 || echo "PASS"
```

**Commit**: YES
- Message: `refactor(plugins): update manager.rs to use struct ToolRegistry`
- Files: `src/plugins/manager.rs`

---

#### Task 9: 修复 workflow/component/tool.rs

**What to do**:
1. 将 2 处 `Arc<dyn ToolRegistry>` 改为 `Arc<ToolRegistry>`（第 23, 37 行）
2. 移除第 189 行的 `BasicTool, BasicToolRegistry` 导入

**Acceptance Criteria**:
```bash
grep -n "dyn ToolRegistry" src/workflow/component/tool.rs && exit 1 || echo "PASS"
grep -n "BasicTool" src/workflow/component/tool.rs && exit 1 || echo "PASS"
```

**Commit**: YES (可合并到 Task 6)
- Message: `refactor(workflow): update tool.rs to use struct ToolRegistry`
- Files: `src/workflow/component/tool.rs`

---

### Wave 4: 修复 BasicTool 使用

#### Task 10: 修复 main.rs

**What to do**:
1. 修改第 5 行导入：`use workflow_toolkit::tools::ToolRegistry;`
2. 修改第 52 行：`let mut tool_registry = ToolRegistry::new();`

**Acceptance Criteria**:
```bash
grep -n "BasicToolRegistry" src/main.rs && exit 1 || echo "PASS"
```

**Commit**: YES
- Message: `refactor(main): update main.rs to use new ToolRegistry`
- Files: `src/main.rs`

---

#### Task 11: 修复 plugins/native.rs BasicTool 使用

**What to do**:
- 将第 264 行的 `BasicTool::from_executor` 改为 `NativeToolBuilder` 模式

**Acceptance Criteria**:
```bash
grep -n "BasicTool" src/plugins/native.rs && exit 1 || echo "PASS"
```

**Commit**: YES (可合并到 Wave 2 的提交)

---

#### Task 12: 修复 plugins/docker.rs BasicTool 使用

**What to do**:
- 将第 1079 行的 `BasicTool::from_executor` 改为 `NativeToolBuilder` 模式
- 更新相关注释（第 949, 1066 行）

**Acceptance Criteria**:
```bash
grep -n "BasicTool::" src/plugins/docker.rs && exit 1 || echo "PASS"
```

**Commit**: YES (可合并到 Task 1)

---

#### Task 13: 修复 plugins/python.rs BasicTool 使用

**What to do**:
- 将第 701 行的 `BasicTool::from_executor` 改为 `NativeToolBuilder` 模式
- 更新相关注释（第 554, 685 行）

**Acceptance Criteria**:
```bash
grep -n "BasicTool::" src/plugins/python.rs && exit 1 || echo "PASS"
```

**Commit**: YES (可合并到 Task 2)

---

#### Task 14: 修复 plugins/nodejs.rs BasicTool 使用

**What to do**:
- 将第 884 行的 `BasicTool::from_executor` 改为 `NativeToolBuilder` 模式
- 更新相关注释（第 734, 868 行）

**Acceptance Criteria**:
```bash
grep -n "BasicTool::" src/plugins/nodejs.rs && exit 1 || echo "PASS"
```

**Commit**: YES (可合并到 Task 3)

---

#### Task 15: 修复 plugins/file_management/utils/registry.rs

**What to do**:
- 将第 263, 350, 459 行的 `BasicTool::builder()` 改为 `NativeToolBuilder::new()` 模式

**Acceptance Criteria**:
```bash
grep -n "BasicTool" src/plugins/file_management/utils/registry.rs && exit 1 || echo "PASS"
```

**Commit**: YES
- Message: `refactor(file-management): migrate registry.rs to NativeToolBuilder`
- Files: `src/plugins/file_management/utils/registry.rs`

---

#### Task 16: 修复 plugins/file_management/ui/human_decision_tool.rs

**What to do**:
- 将第 555 行的返回类型从 `BasicTool` 改为 `Tool`
- 将第 638 行的 `BasicTool::builder()` 改为 `NativeToolBuilder::new()` 模式

**Acceptance Criteria**:
```bash
grep -n "BasicTool" src/plugins/file_management/ui/human_decision_tool.rs && exit 1 || echo "PASS"
```

**Commit**: YES
- Message: `refactor(file-management): migrate human_decision_tool.rs to NativeToolBuilder`
- Files: `src/plugins/file_management/ui/human_decision_tool.rs`

---

### Wave 5: 移除 classification_flow.rs 的 ToolNode 实现

#### Task 17: 移除 classification_flow.rs 所有 ToolNode 实现

**What to do**:
- 删除 11 个 `impl ToolNode for` 代码块：
  - RuleLoaderTool (第 43 行开始)
  - RulePreprocessorTool (第 175 行)
  - AutomatonBuilderTool (第 251 行)
  - DirectoryScannerTool (第 295 行)
  - FolderNamePreprocessorTool (第 360 行)
  - ParallelMatcherTool (第 414 行)
  - ScoreCalculatorTool (第 501 行)
  - AmbiguityDetectorTool (第 574 行)
  - ResultMergerTool (第 658 行)
  - ExperimentalCheckTool (第 721 行)
  - ReportGeneratorTool (第 752 行)

**Must NOT do**:
- 保留工具结构体本身（仅删除 trait 实现）
- 保留工具的业务逻辑方法

**Recommended Agent Profile**:
- **Category**: `unspecified-high`（文件较大，修改点多）
- **Skills**: []

**Acceptance Criteria**:
```bash
grep -n "impl ToolNode for" src/plugins/file_management/classification/classification_flow.rs && exit 1 || echo "PASS"
```

**Commit**: YES
- Message: `refactor(file-management): remove ToolNode impls from classification_flow.rs`
- Files: `src/plugins/file_management/classification/classification_flow.rs`

---

### Wave 6: 验证

#### Task 18: 编译验证

**What to do**:
1. 运行 `cargo clean`
2. 运行 `cargo build --release`
3. 确认 0 错误

**Acceptance Criteria**:
```bash
cargo clean
cargo build --release 2>&1 | tee /tmp/build.log
if grep -q "error\[E" /tmp/build.log; then
    echo "BUILD FAILED"
    grep "error\[E" /tmp/build.log | head -10
    exit 1
else
    echo "BUILD SUCCESS"
fi
```

**Commit**: NO（验证任务）

---

#### Task 19: 测试验证

**What to do**:
1. 运行 `cargo test`
2. 确认所有测试通过

**Acceptance Criteria**:
```bash
cargo test 2>&1 | tee /tmp/test.log
if grep -q "test result: FAILED" /tmp/test.log; then
    echo "TESTS FAILED"
    grep "test result:" /tmp/test.log
    exit 1
else
    echo "TESTS PASSED"
    grep "test result:" /tmp/test.log
fi
```

**Commit**: NO（验证任务）

---

#### Task 20: 最终验证 - 无旧 API 残留

**What to do**:
1. 验证无任何 `ToolNode` trait 实现
2. 验证无任何 `dyn ToolRegistry`
3. 验证无任何 `BasicTool` 使用

**Acceptance Criteria**:
```bash
#!/bin/bash
echo "Checking for old API remnants..."

ERRORS=0

if grep -r "impl ToolNode for" src/; then
    echo "❌ Found ToolNode implementations"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No ToolNode implementations found"
fi

if grep -r "dyn ToolRegistry" src/; then
    echo "❌ Found dyn ToolRegistry usage"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No dyn ToolRegistry usage found"
fi

if grep -r "BasicTool::" src/ || grep -r "BasicTool " src/ | grep -v "// " | grep -v "BasicToolRegistry"; then
    echo "❌ Found BasicTool usage"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ No BasicTool usage found"
fi

if [ $ERRORS -eq 0 ]; then
    echo "✅ All old APIs removed successfully!"
    exit 0
else
    echo "❌ Found $ERRORS old API remnants"
    exit 1
fi
```

**Commit**: NO（验证任务）

---

## Commit Strategy

| Wave | Commit Message | Files |
|------|---------------|-------|
| Wave 2 | `refactor(plugins): migrate docker/python/nodejs plugins to enum-based Tool system` | docker.rs, python.rs, nodejs.rs, native.rs |
| Wave 3 | `refactor: update ToolRegistry usage from trait to struct` | mcp.rs, cli/app.rs, retry_tests.rs, tool_manager.rs, manager.rs, tool.rs |
| Wave 4 | `refactor: migrate BasicTool usage to NativeToolBuilder` | main.rs, registry.rs, human_decision_tool.rs |
| Wave 5 | `refactor(file-management): remove ToolNode trait implementations` | classification_flow.rs |
| Final | `chore: verify no old API remnants` | - |

---

## Success Criteria

### Verification Commands
```bash
# 1. 验证无旧 trait 实现
grep -r "impl ToolNode for" src/ && exit 1 || echo "✅ No ToolNode implementations"

# 2. 验证无 dyn ToolRegistry
grep -r "dyn ToolRegistry" src/ && exit 1 || echo "✅ No dyn ToolRegistry usage"

# 3. 验证无 BasicTool
grep -r "BasicTool::" src/ && exit 1 || echo "✅ No BasicTool usage"

# 4. 编译通过
cargo build --release 2>&1 | grep -q "error\[E" && exit 1 || echo "✅ Build successful"

# 5. 测试通过
cargo test 2>&1 | grep -q "test result: FAILED" && exit 1 || echo "✅ All tests passed"
```

### Final Checklist
- [ ] 0 个 `impl ToolNode for` 残留
- [ ] 0 个 `dyn ToolRegistry` 使用
- [ ] 0 个 `BasicTool` 使用
- [ ] `cargo build` 0 错误
- [ ] `cargo test` 全部通过
- [ ] `src/tools/compat.rs` 已删除
- [ ] `src/tools/node.rs` 已删除

---

## Notes

### 关键文件映射

| 原使用 | 新使用 | 文件位置 |
|--------|--------|----------|
| `impl ToolNode for X` | `NativeToolBuilder::new()` | 所有插件文件 |
| `Arc<dyn ToolRegistry>` | `Arc<ToolRegistry>` | 6个接口/工作流文件 |
| `BasicTool::from_executor` | `NativeToolBuilder::new()` | 8个插件文件 |
| `BasicTool::builder()` | `NativeToolBuilder::new()` | registry.rs, human_decision_tool.rs |

### 常见错误模式

**错误 1**: 忘记更新 `get_tools()` 返回类型
```rust
// 错误
fn get_tools(&self) -> Vec<Arc<dyn ToolNode>>

// 正确
fn get_tools(&self) -> Vec<Tool>
```

**错误 2**: 使用 `BasicTool` 而不是 `NativeToolBuilder`
```rust
// 错误
let tool = BasicTool::from_executor(...)?;

// 正确
let native_tool = NativeToolBuilder::new().executor(...).build()?;
let tool = Tool::Native(Arc::new(native_tool));
```

**错误 3**: 忘记移除 trait 导入
```rust
// 删除这行
use crate::tools::ToolNode;
```

### 应急回滚

如果需要回滚：
```bash
# 查看提交历史
git log --oneline -10

# 回滚到 Wave 1 完成状态
git reset --hard <wave1-commit-hash>
```

---

**计划生成时间**: 2026-02-08  
**计划状态**: Ready for Execution  
**建议执行**: `/start-work` 启动 Sisyphus 执行
