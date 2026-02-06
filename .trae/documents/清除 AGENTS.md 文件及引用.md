## 目标
清除项目中所有 AGENTS.md 文件及其在代码和文档中的引用

## 步骤

### 步骤 1：删除 AGENTS.md 文件（10个）
- tests/AGENTS.md
- src/storage/AGENTS.md
- src/plugins/file_management/AGENTS.md
- src/plugins/AGENTS.md
- src/performance/AGENTS.md
- src/interfaces/tui/widgets/AGENTS.md
- src/interfaces/tui/AGENTS.md
- src/interfaces/cli/AGENTS.md
- src/interfaces/AGENTS.md
- examples/templates/AGENTS.md

### 步骤 2：清理 .rs 文件中的引用（10个）
删除类似 `//! See [AGENTS.md](AGENTS.md) for detailed documentation.` 的注释行：
- src/lib.rs
- src/tools/mod.rs
- src/plugins/mod.rs
- src/plugins/file_management/mod.rs
- src/workflow/mod.rs
- src/interfaces/mod.rs
- src/storage/mod.rs
- src/performance/mod.rs
- src/interfaces/tui/widgets/mod.rs
- src/interfaces/cli/mod.rs

### 步骤 3：清理 .md 文件中的引用（6个）
- README.md
- README_CN.md
- docs/PRODUCT_DESIGN.md
- docs/guides/USER_GUIDE.md
- docs/guides/CHEATSHEET.md
- docs/dev/DEVELOPMENT_GUIDE.md

### 步骤 4：处理脚本文件
- docs/scripts/update_agents_md.py（建议删除，因为不再需要更新 AGENTS.md）

## 预期结果
- 所有 AGENTS.md 文件被删除
- 所有对 AGENTS.md 的引用被清理
- 代码和文档保持一致