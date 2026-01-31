## Phase 2 Task 2.3: 文档路径更新清单

### 需要更新的文件和变更

#### 1. 基础工作流路径更新
**从**: `examples/hello-world.yaml`
**到**: `workflows/basic/hello-world.yaml`

**影响文件**:
- `docs/api/CLI_REFERENCE.md` (1处)
- `docs-zh/dev/DOCUMENTATION_STANDARDS.md` (4处)
- `docs/guides/TROUBLESHOOTING.md` (12处)
- `docs/dev/DOCUMENTATION_STANDARDS.md` (4处)
- `docs-zh/api/CLI_REFERENCE.md` (1处)
- `src/interfaces/cli/AGENTS.md` (1处)

#### 2. 模板工作流路径更新
**从**: `examples/templates/interactive-classification-workflow.yaml`
**到**: `workflows/templates/interactive/classification.yaml`

**从**: `examples/templates/interactive-merge-workflow.yaml`
**到**: `workflows/templates/interactive/merge.yaml`

**从**: `examples/templates/interactive-batch-processing-workflow.yaml`
**到**: `workflows/templates/interactive/batch-processing.yaml`

**影响文件**:
- `examples/AGENTS.md` (3处)
- `docs-zh/plugins/file_management/FILE_MANAGEMENT_WORKFLOW_TEMPLATES.md` (15处)
- `examples/templates/AGENTS.md` (15处)

#### 3. AGENTS.md链接更新
**从**: `examples/templates/AGENTS.md`
**到**: `workflows/templates/AGENTS.md` (需要新建或更新)

**影响文件**:
- `AGENTS.md` (多处引用)
- `QUICK_REFERENCE.md`
- `README_CN.md`
- `src/plugins/file_management/AGENTS.md`

---

### 更新策略

由于涉及大量文档更新，建议：
1. 先更新代码示例中的路径（TROUBLESHOOTING.md, CLI_REFERENCE.md等）
2. 再更新AGENTS.md中的交叉引用
3. 最后更新中文文档

### 注意事项
- 保持文档的其他内容不变
- 只替换路径部分
- 确保新路径正确无误
