---
version: v2.9.0
updated: 2026-02-24
---

# 版本同步检查清单

---

## 目的

本文件定义 SOP 版本更新时的**强制检查清单**，确保所有文件版本号与 CHANGELOG.md 保持一致。

---

## 触发条件

| 触发场景 | 检查范围 | 强制性 |
|----------|----------|--------|
| 主版本更新（v2.x → v3.x） | 全部文件 | 必须 |
| 次版本更新（v2.8.x → v2.9.x） | 全部文件 | 必须 |
| 修订版本更新（v2.8.0 → v2.8.1） | 变更文件 | 建议 |

---

## 检查流程

### 步骤 1：确认版本基线

```
动作：读取 sop/CHANGELOG.md
输出：目标主版本号、次版本号
```

### 步骤 2：核对核心文档版本

| 文件 | 检查项 | 通过条件 |
|------|--------|----------|
| `02_skill_matrix/index.md` | 主/次版本 | 与 CHANGELOG 一致 |
| `05_constraints/index.md` | 主/次版本 | 与 CHANGELOG 一致 |
| `05_constraints/state_dictionary.md` | 主/次版本 | 与 CHANGELOG 一致 |
| `05_constraints/command_dictionary.md` | 主/次版本 | 与 CHANGELOG 一致 |
| `05_constraints/constraint_matrix.md` | 主/次版本 | 与 CHANGELOG 一致 |
| `04_reference/index.md` | 主/次版本 | 与 CHANGELOG 一致 |

### 步骤 3：核对 Skill 合约版本

| 检查项 | 通过条件 |
|--------|----------|
| 所有 `skills/*/SKILL.md` 主/次版本 | 与 CHANGELOG 一致 |

**Skill 清单**（共 17 个）：
- sop-workflow-orchestrator
- sop-fast-path
- sop-deep-path
- sop-code-explorer
- sop-requirement-analyst
- sop-architecture-design
- sop-architecture-reviewer
- sop-implementation-designer
- sop-design-placement
- sop-progress-supervisor
- sop-code-implementation
- sop-code-review
- sop-document-sync
- sop-capability-reuse
- sop-tdd-workflow
- sop-test-design-csv
- sop-test-implementation

### 步骤 4：核对参考文档版本

| 目录 | 检查项 |
|------|--------|
| `03_workflow/` | 所有文件主/次版本 |
| `04_reference/` | 所有文件主/次版本 |
| `04_context_reference/` | 所有文件主/次版本 |

### 步骤 5：更新版本号

仅当发现不一致时执行：

```
动作：将不一致文件的版本号更新为目标版本
输出：更新文件列表
```

### 步骤 6：更新日期

```
动作：将所有更新文件的 updated 字段改为当前日期
输出：更新日期列表
```

---

## 检查结果模板

```markdown
# 版本同步检查结果

## 元信息
- 检查日期: YYYY-MM-DD
- 目标版本: vX.Y.Z
- 检查人: [AI Agent / 用户]

## 检查结果

| 文件 | 当前版本 | 目标版本 | 状态 | 修复动作 |
|------|----------|----------|------|----------|
| [path] | vX.Y.Z | vX.Y.Z | ✅/❌ | [动作] |

## 统计
- 检查文件数: N
- 通过数: N
- 失败数: N
- 已修复数: N

## 结论
- [ ] 全部通过
- [ ] 存在失败项，需修复后重新检查
```

---

## 版本超前处理

当发现文件版本超前于 CHANGELOG 时：

1. **审查变更内容**：确认超前版本的变更是否为预期变更
2. **决策**：
   - 如为预期变更 → 更新 CHANGELOG 到超前版本
   - 如为非预期变更 → 回退文件版本到 CHANGELOG 版本
3. **记录决策**：在 ADR 中记录版本超前处理决策

---

## 相关文档

- [CHANGELOG.md](../CHANGELOG.md) - 版本历史
- [sop_GUIDE.md](../sop_GUIDE.md) - 审查指南
- [state_dictionary.md](state_dictionary.md) - 状态字典
- [command_dictionary.md](command_dictionary.md) - 命令字典
