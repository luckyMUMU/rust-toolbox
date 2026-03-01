# Tasks

- [x] Task 1: 创建归档目录结构
  - [x] 创建 `.temp/docs-archive/` 目录
  - [x] 创建 `.temp/design-archive/` 目录
  - [x] 创建 `.temp/ADR/` 目录

- [x] Task 2: 归档现有 docs 文档
  - [x] 移动 `docs/01_constitution/` 到 `.temp/docs-archive/`
  - [x] 移动 `docs/05_constraints/` 到 `.temp/docs-archive/`
  - [x] 移动 `docs/design/` 到 `.temp/docs-archive/`
  - [x] 移动 `docs/specs/` 到 `.temp/docs-archive/`
  - [x] 移动 `docs/参考/` 到 `.temp/docs-archive/`
  - [x] 移动 `docs/INDEX.md` 到 `.temp/docs-archive/`

- [x] Task 3: 归档所有 design.md 文档
  - [x] 移动根目录 `design.md` 到 `.temp/design-archive/`
  - [x] 移动 `src/domain/design.md` 到 `.temp/design-archive/src/domain/`
  - [x] 移动 `src/application/design.md` 到 `.temp/design-archive/src/application/`
  - [x] 移动 `src/infrastructure/design.md` 到 `.temp/design-archive/src/infrastructure/`
  - [x] 移动 `src/interfaces/design.md` 到 `.temp/design-archive/src/interfaces/`
  - [x] 移动 `src/workflow/design.md` 到 `.temp/design-archive/src/workflow/`
  - [x] 移动 `src/workflow/component/design.md` 到 `.temp/design-archive/src/workflow/component/`
  - [x] 移动 `src/workflow/executor/design.md` 到 `.temp/design-archive/src/workflow/executor/`
  - [x] 移动 `src/workflow/state/design.md` 到 `.temp/design-archive/src/workflow/state/`
  - [x] 移动 `src/workflow/context/design.md` 到 `.temp/design-archive/src/workflow/context/`
  - [x] 移动 `src/plugins/design.md` 到 `.temp/design-archive/src/plugins/`
  - [x] 移动 `src/plugins/file_management/design.md` 到 `.temp/design-archive/src/plugins/file_management/`
  - [x] 移动 `src/storage/design.md` 到 `.temp/design-archive/src/storage/`
  - [x] 移动 `src/tools/design.md` 到 `.temp/design-archive/src/tools/`
  - [x] 移动 `src/performance/design.md` 到 `.temp/design-archive/src/performance/`

- [x] Task 4: 提取 ADR 文档
  - [x] 复制 `.temp/docs-archive/design/adr/*` 到 `.temp/ADR/`
  - [x] 分析并整理 ADR 文档内容
  - [x] 提取核心设计决策记录

- [x] Task 5: 分析代码实现现状
  - [x] 分析 `src/domain/` 模块设计
  - [x] 分析 `src/application/` 模块设计
  - [x] 分析 `src/infrastructure/` 模块设计
  - [x] 分析 `src/interfaces/` 模块设计
  - [x] 分析 `src/workflow/` 模块设计
  - [x] 分析 `src/plugins/` 模块设计
  - [x] 分析 `src/storage/` 模块设计
  - [x] 分析 `src/tools/` 模块设计

- [x] Task 6: 根据 SOP 创建 P0 级工程宪章文档
  - [x] 复制 `sop/01_constitution/*` 到 `docs/01_constitution/`
  - [x] 验证文档符合 P0 约束

- [x] Task 7: 创建 P1 级系统规范文档
  - [x] 基于代码分析创建 `docs/02_specifications/system-spec.md`
  - [x] 定义系统边界和模块职责
  - [x] 定义跨模块接口规范

- [x] Task 8: 创建 P2 级模块规范文档
  - [x] 创建 `docs/02_specifications/domain-model.md`
  - [x] 创建 `docs/02_specifications/api-contract.md`
  - [x] 创建 `docs/02_specifications/data-model.md`

- [x] Task 9: 创建设计文档
  - [x] 创建 `docs/02_logical_workflow/architecture-design.md`
  - [x] 创建 `docs/02_logical_workflow/adr-index.md`
  - [x] 创建 ADR 文档目录 `docs/02_logical_workflow/adr/`
  - [x] 创建 `docs/02_logical_workflow/adr/adr-di-001-di-container.md`
  - [x] 创建 `docs/02_logical_workflow/adr/adr-workflow-002-joinset.md`
  - [x] 创建 `docs/02_logical_workflow/adr/adr-plugin-003-wasm-sandbox.md`
  - [x] 创建 `docs/02_logical_workflow/adr/adr-tools-004-schema-validation.md`

- [x] Task 10: 创建指南文档
  - [x] 创建 `docs/03_guides/module-design-guide.md`
  - [x] 创建 `docs/03_guides/plugin-development-guide.md`
  - [x] 创建 `docs/03_guides/workflow-writing-guide.md`

- [x] Task 11: 用户决策与清理
  - [x] 清理归档文档（.temp/docs-archive/ 和 .temp/design-archive/）
  - [x] 在 docs 目录下新建 ADR 文档
  - [x] 更新文档索引

# Task Dependencies
- [Task 2] depends on [Task 1]
- [Task 3] depends on [Task 2]
- [Task 4] depends on [Task 3]
- [Task 5] depends on [Task 4]
- [Task 6] depends on [Task 5]
- [Task 7] depends on [Task 6]
- [Task 8] depends on [Task 7]
- [Task 9] depends on [Task 8]
- [Task 10] depends on [Task 9]
- [Task 11] depends on [Task 10]
