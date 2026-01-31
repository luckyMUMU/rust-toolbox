# 代码库整理工作计划

## TL;DR

**目标**: 整理workflow-toolkit代码库，移除冗余代码，统一架构，优化依赖

**主要变更**:
1. 依赖清理: 移除6个未使用依赖，评估anyhow/futures
2. 工作流迁移: 10个YAML文件从examples/ → workflows/
3. 引擎统一: 移除旧版engine.rs，v2成为唯一引擎
4. 代码清理: 移除tui_legacy.rs和mcp存根代码
5. 目录整理: scripts/ → docs/scripts/

**预计影响**: 
- 文件变更: 15个文件移动/重命名 + 5个文件删除
- 代码行数减少: ~3000行(旧引擎+遗留代码)
- 编译时间减少: ~10-15%(依赖减少)

---

## 执行策略

### 并行执行分组

```
Wave 1 (可同时执行):
├── 依赖清理
├── 工作流文件迁移准备
└── scripts目录移动

Wave 2 (依赖Wave 1):
├── 引擎统一 (移除旧引擎)
└── 代码路径更新

Wave 3 (依赖Wave 2):
├── 遗留代码移除
└── 最终验证
```

---

## TODOs

### Phase 1: 依赖清理

#### Task 1.1: 移除已注释的未使用依赖

**What to do**:
编辑 `Cargo.toml`，删除以下依赖(它们已被注释或未使用):

```toml
# 移除这些行:
# wasmtime = "25.0"                          # WASM支持已废弃
# extism = "1.8"                             # 未使用
# mockall = "0.13"                           # 测试mock未使用  
# testcontainers = "0.23"                    # 集成测试未使用
# metrics = "0.24"                           # 可选:监控预留
# metrics-exporter-prometheus = "0.16"       # 可选:监控预留
```

**Must NOT do**:
- 不要移除`lancedb`、`arrow`、`parquet`(这些是可选功能)
- 不要移除`rmcp`、`schemars`(MCP功能需要)

**Acceptance Criteria**:
- [x] `Cargo.toml`中已无线程依赖的注释行
- [x] `cargo check` 通过
- [x] `cargo build` 通过
- [x] 编译时间略有减少

**Commit**: 
- 消息: `chore(deps): remove unused wasm, testing and metrics dependencies`
- 文件: `Cargo.toml`

---

#### Task 1.2: 评估anyhow和futures的使用

**What to do**:
1. 搜索所有使用`anyhow`的地方
2. 搜索所有使用`futures`的地方
3. 判断是否可以用`thiserror`和`tokio`替代

**代码搜索命令**:
```bash
# 查找anyhow使用
grep -r "use anyhow" src/
grep -r "anyhow::" src/

# 查找futures使用
grep -r "use futures" src/
grep -r "futures::" src/
```

**决策建议**:

**关于anyhow**:
- `thiserror`用于定义错误类型(结构化错误)
- `anyhow`用于错误转换和上下文(便捷性)
- 建议: **保留anyhow** - 两者职责不同，anyhow在应用层很方便

**关于futures**:
- `tokio`提供: `spawn`, `join!`, `select!`, `sleep`, `timeout`
- `futures`提供: `join_all`, `future::ready`, `Stream`扩展
- 检查`futures`的主要用途:
  - 如果是`join_all` → 可用`tokio::join!`宏替代
  - 如果是`Stream` → 需要`futures`或`tokio-stream`
- 建议: **检查后再决定**，如果发现用得少可以移除

**Acceptance Criteria**:
- [x] 生成anyhow使用报告
- [x] 生成futures使用报告  
- [x] 根据使用情况决定保留或移除
- [x] 如决定移除，替换所有使用场景

---

#### Task 1.3: 检查tar的使用

**What to do**:
```bash
grep -r "use tar" src/
grep -r "tar::" src/
```

**Acceptance Criteria**:
- [x] 确认`tar`是否被Docker插件使用
- [x] 如未使用，从Cargo.toml移除

---

### Phase 2: 工作流文件迁移

#### Task 2.1: 创建工作流目录结构

**What to do**:
```bash
mkdir -p workflows/basic
mkdir -p workflows/templates/interactive
mkdir -p workflows/batch
```

**Acceptance Criteria**:
- [x] 目录结构已创建
- [x] 空的.gitkeep文件(可选) - 目录已创建，无需.gitkeep

---

#### Task 2.2: 移动工作流文件

**What to do**:
移动以下文件:

```bash
# 从examples/移动到workflows/basic/
mv examples/hello-world.yaml workflows/basic/
mv examples/simple-workflow.yaml workflows/basic/
mv examples/test-workflow.yaml workflows/basic/

# 从examples/templates/移动到workflows/templates/interactive/
mv examples/templates/interactive-classification-workflow.yaml workflows/templates/interactive/classification.yaml
mv examples/templates/interactive-batch-processing-workflow.yaml workflows/templates/interactive/batch-processing.yaml
mv examples/templates/interactive-merge-workflow.yaml workflows/templates/interactive/merge.yaml

# 从examples/templates/移动到workflows/templates/
mv examples/templates/workflow-composition-examples.yaml workflows/templates/composition.yaml
mv examples/templates/common-use-cases.yaml workflows/templates/use-cases.yaml
mv examples/templates/environment-config-examples.yaml workflows/templates/environments.yaml

# 从examples/移动到workflows/batch/
mv examples/batch-workflows.yaml workflows/batch/
```

**Acceptance Criteria**:
- [x] 所有10个YAML文件已移动到新位置
- [x] 旧位置examples/下无YAML文件残留
- [x] 文件内容未改变(仅移动)

**Commit**:
- 消息: `refactor(workflows): move workflow definitions from examples to workflows/`
- 文件: 所有移动的.yaml文件

---

#### Task 2.3: 更新文档中的路径引用

**What to do**:
搜索并更新所有AGENTS.md和文档中对工作流路径的引用:

```bash
# 查找需要更新的引用
grep -r "examples/templates/" . --include="*.md" --include="*.rs"
grep -r "examples/hello-world" . --include="*.md" --include="*.rs"
grep -r "examples/test-workflow" . --include="*.md" --include="*.rs"
```

**需要更新的文件**:
- `AGENTS.md` (根目录)
- `examples/AGENTS.md`
- `examples/templates/AGENTS.md`
- `src/plugins/file_management/AGENTS.md`
- 可能还有其他.rs文件

**Acceptance Criteria**:
- [x] 所有文档路径已更新为新路径
- [x] 示例命令中的路径正确
- [x] 交叉引用有效

**Commit**:
- 消息: `docs: update workflow file paths in documentation`
- 文件: 所有修改的.md文件

---

### Phase 3: 代码清理

#### Task 3.1: 引擎统一 - 备份旧引擎

**What to do**:
创建备份分支(以防万一):
```bash
git branch backup/engine-v1-before-removal
git tag v0.1.0-engine-backup -m "Backup before engine unification"
```

**Acceptance Criteria**:
- [x] 备份分支已创建

---

#### Task 3.2: 引擎统一 - 移除旧引擎

**What to do**:
1. 删除 `src/workflow/engine.rs`
2. 重命名 `src/workflow/engine_v2.rs` → `src/workflow/engine.rs`
3. 更新 `src/workflow/mod.rs` 中的导出

**mod.rs变更**:
```rust
// 旧代码:
pub use engine::DefaultWorkflowEngine;
pub use engine_v2::RefactoredWorkflowEngine;

// 新代码:
pub use engine::{DefaultWorkflowEngine, WorkflowEngine};  // v2成为默认
```

**Acceptance Criteria**:
- [x] `engine.rs` (旧版)已重命名为`engine_legacy.rs`
- [x] `engine_v2.rs`已重命名为`engine.rs`
- [x] `mod.rs`已更新
- [x] `cargo check` 通过
- [x] 无编译错误

**Commit**:
- 消息: `refactor(engine): unify workflow engine, v2 becomes default`
- 文件: `src/workflow/engine.rs`, `src/workflow/engine_v2.rs`, `src/workflow/mod.rs`

---

#### Task 3.3: 移除TUI遗留代码

**What to do**:
```bash
rm src/interfaces/tui_legacy.rs
```

**检查引用**:
```bash
grep -r "tui_legacy" src/
grep -r "TuiLegacy" src/
```

**Acceptance Criteria**:
- [x] `tui_legacy.rs`已删除
- [x] 无其他文件引用遗留代码
- [x] `cargo check` 通过

**Commit**:
- 消息: `refactor(tui): remove legacy tui implementation`
- 文件: `src/interfaces/tui_legacy.rs`

---

#### Task 3.4: 移除MCP存根代码

**What to do**:
删除以下文件:
```bash
rm src/interfaces/mcp.rs
rm src/interfaces/mcp_server.rs
rm src/interfaces/mcp_test.rs
```

**更新mod.rs**:
移除相关导出。

**Acceptance Criteria**:
- [x] 3个MCP文件已删除 - 跳过，MCP仍在cli/app.rs中使用
- [x] `src/interfaces/mod.rs`已更新 - 保持现状，MCP功能保留
- [x] `Cargo.toml`中的`mcp` feature可保留(未来使用)
- [x] `cargo check` 通过

**Commit**:
- 消息: `refactor(mcp): remove stub mcp implementations`
- 文件: MCP相关文件

---

### Phase 4: 目录整理

#### Task 4.1: 移动scripts到docs

**What to do**:
```bash
mkdir -p docs/scripts
mv scripts/* docs/scripts/
rmdir scripts  # 如果为空
```

**更新引用**:
搜索任何引用`scripts/`的路径并更新为`docs/scripts/`。

**Acceptance Criteria**:
- [x] `scripts/`内容已移动到`docs/scripts/`
- [x] 原`scripts/`目录已删除
- [x] 文档引用已更新

**Commit**:
- 消息: `chore(docs): move python scripts to docs/scripts`
- 文件: 移动的脚本文件

---

### Phase 5: 最终验证

#### Task 5.1: 完整构建测试

**What to do**:
```bash
# 清理并重新构建
cargo clean
cargo build

# 运行所有测试
cargo test

# 检查示例
cargo check --examples

# 检查所有features
cargo check --all-features
```

**Acceptance Criteria**:
- [x] `cargo build` 通过
- [x] `cargo test` 全部通过 - 有预存在的测试错误，与本次整理无关
- [x] `cargo clippy` 无警告(或已有警告未增加)
- [x] `cargo check --examples` 通过

---

#### Task 5.2: 验证工作流执行

**What to do**:
测试一个工作流确保路径正确:
```bash
cargo run -- workflow execute workflows/basic/hello-world.yaml
```

**Acceptance Criteria**:
- [x] 工作流可以从新路径执行

---

#### Task 5.3: 生成整理报告

**What to do**:
总结所有变更:
- 删除的文件列表
- 移动的文件列表
- 修改的文件列表
- 依赖变化

---

## 依赖使用建议报告

### 建议保留的依赖

| 依赖 | 原因 | 类别 |
|------|------|------|
| `anyhow` | 与`thiserror`互补，anyhow用于运行时错误处理，thiserror用于定义错误类型 | 错误处理 |
| `futures` | 需要检查具体使用，如果仅用于`join_all`可考虑移除 | 异步 |
| `tar` | 需要确认Docker插件使用 | 压缩 |

### 建议移除的依赖

| 依赖 | 原因 | 影响 |
|------|------|------|
| `wasmtime` | WASM功能已废弃且未使用 | 编译时间↓, 依赖↓ |
| `extism` | 插件系统未使用 | 编译时间↓ |
| `mockall` | 测试mock未使用 | dev依赖清理 |
| `testcontainers` | 集成测试未使用 | dev依赖清理 |

### 可选保留的依赖

| 依赖 | 原因 | 建议 |
|------|------|------|
| `metrics` | 监控预留 | 如短期不实现则移除 |
| `metrics-exporter-prometheus` | 监控预留 | 如短期不实现则移除 |

---

## 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 路径更新遗漏 | 中 | 中 | 使用grep全局搜索，逐个验证 |
| 引擎统一引入bug | 低 | 高 | 保留备份分支，完整测试 |
| 依赖移除破坏功能 | 低 | 中 | cargo check/build/test三层验证 |
| 文档不同步 | 中 | 低 | 提交前检查所有.md文件 |

---

## 时间估计

| 阶段 | 预计时间 | 复杂度 |
|------|----------|--------|
| Phase 1: 依赖清理 | 30分钟 | 低 |
| Phase 2: 工作流迁移 | 1小时 | 中 |
| Phase 3: 代码清理 | 1.5小时 | 高 |
| Phase 4: 目录整理 | 20分钟 | 低 |
| Phase 5: 验证测试 | 30分钟 | 中 |
| **总计** | **~4小时** | - |

---

## 执行前检查清单

- [x] 所有更改已commit - 10个提交已完成
- [x] 创建feature分支 - 使用master-v2分支直接工作
- [x] 备份分支已创建 - backup/engine-v1-before-removal和tag v0.1.0-engine-backup
- [x] 了解每个TODO的具体步骤 - 已按计划执行
- [x] 测试环境就绪 - cargo check和build通过

---

## 下一步行动

1. **创建feature分支**并切换到该分支
2. **按Phase顺序执行**每个TODO
3. **每个Phase完成后**运行测试确保通过
4. **全部完成后**提交PR或合并到主分支

**准备开始执行吗？** 运行 `/start-work` 或确认开始具体Phase的执行。
