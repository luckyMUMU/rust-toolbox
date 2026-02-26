---
name: "sop-document-sync"
description: "Document synchronization workflow for index updates, progressive disclosure, and task archiving. Invoke when documents need synchronization, status updates, or task archiving."
version: v2.12.0
updated: 2026-02-25
layer: "文档"
load_policy:
  tier: 2
  auto_load_states: ["[ALL_COMPLETED]"]
  depends_on: ["sop-code-review", "sop-progress-supervisor"]
---

# Document Synchronization Workflow

**位置**: `sop/skills/sop-document-sync/SKILL.md`

## 侧重点

- 仅同步文档与索引；不得改动 `docs/参考/` 下的 SSOT（除非重构任务明确包含）
- 变更必须包含版本一致性检查与链接检查结果

## 质量门控检查

> 完成文档同步后，必须执行以下门控检查：

| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 需求实现 | 所有需求已实现 | [ ] |
| 验收满足 | 验收标准全部满足 | [ ] |
| 质量达标 | 文档完整准确 | [ ] |

**门控失败处理**：若任一检查项未通过，应记录失败原因并返回相应阶段修正。

## 评估阶段产出建议

> 当任务全部完成时，建议产出以下评估文档：

### 项目总结报告（可追加到spec.md）

- 任务完成情况汇总
- 质量指标达成情况
- 遗留问题清单
- 经验教训总结

### 待办事项清单（可追加到tasks.md）

- 未完成事项
- 后续优化建议
- 配置缺失项
- 需要支持的事项

## 触发条件

- 任意文档发生新增/更新/状态变更/归档，需要同步父级索引与交叉引用
- 发现链接断裂、索引缺失、状态标记不一致，需要执行文档修复与同步

## Input

- type(add/update/status/archive)
- target(path)
- change(brief)
- parent/related(paths)

## Workflow Steps

### Step 1: Content Update

**Purpose**: Update document

**Actions**:
1. Apply changes
2. Update status
3. 完成同步后标记 `[已完成]`

### Step 2: Parent Index Update

**Purpose**: Maintain hierarchy

**Actions**:
1. Update parent summary
2. Add/update links
3. Ensure progressive disclosure

### Step 3: Cross-Reference Sync

**Purpose**: Maintain consistency

**Actions**:
1. Update related docs
2. Fix broken links
3. Sync status marks

### Step 4: Validation

**Purpose**: Check quality

**Actions**:
1. Validate links
2. Check structure
3. Verify format

### Step 5: Task Archive

**Purpose**: Archive completed tasks when directory is completed

**触发条件**: 目录状态从 `[DIR_WORKING]` 变为 `[DIR_COMPLETED]`

**Actions**:
1. Read design.md in completed directory
2. Identify tasks with status `[x]` (completed)
3. Move completed tasks to "归档任务" section in design.md
4. Update archive metadata (date, reason: "目录归档同步")
5. Clear completed tasks from "活跃任务" section
6. Update document index

CMD: `TASK_ARCHIVE(dir) -> archived_tasks`

**归档规则**:
| 规则 | 说明 |
|------|------|
| 仅归档已完成任务 | 状态为 `[x]` 的任务移入归档 |
| 保留未完成任务 | 状态为 `[ ]`、`[-]`、`[!]` 的任务保留在活跃清单 |
| 记录归档元数据 | 归档日期、归档原因（目录归档同步） |

### Step 6: Gate Check

**Purpose**: Execute quality gate check for document sync phase

**Actions**:
CMD: `GATE_CHECK(sync_result, gate='GATE_SYNC')`

**Gate Check Items**:
| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 需求实现 | 所有需求已实现 | [ ] |
| 验收满足 | 验收标准全部满足 | [ ] |
| 质量达标 | 文档完整准确 | [ ] |

**State Transition**:
- 通过 → `[已完成]`
- 失败 → `[GATE_FAILED]` → 用户决策

## 来源与依赖准则

- 必须声明同步依据来源与依赖（变更文件列表/引用关系/目录映射规则等），并优先用 `TRACE_SOURCES(inputs)` 固化“来源与依赖声明”
- 当存在结构/命名冲突无法消解时必须中断：进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录
- 标准：04_reference/review_standards/source_dependency.standard.md

## Output

- 交付物：目标文档内容更新（落盘至 target/path）
- 交付物：父级索引与相关文档链接/状态更新（落盘至 parent/related paths）
- 状态：`[已完成]`
- CMD: `DOC_SYNC(scope)`

## Stop Points

- `[已完成]`: 本次同步结束
- `[USER_DECISION]`: 同步导致的结构/命名冲突需要人工选择

## Constraints

- Parent docs: summary + links only
- Progressive disclosure
- Valid links required
- 状态标记必须以 SSOT 为准：05_constraints/state_dictionary.md；本 Skill 仅使用 `[已完成]` / `[USER_DECISION]`

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）

## Failure Handling

- 发现断链/缺失引用时必须修复并复查，禁止只记录不修复

## Document Levels (L1-L4)

参见 04_reference/document_directory_mapping.md（逻辑目录 → 项目实际目录映射）。

| Level | Path | Content | Creator |
|-------|------|---------|---------|
| L1 | `docs/01_requirements/` | PRD/MRD/FRD/Prototype | sop-requirement-analyst |
| L2 | `docs/02_logical_workflow/` | Architecture (.md) | sop-architecture-design |
| L3 | `docs/03_technical_spec/` + `src/**/design.md` | Implementation + Test Cases | sop-implementation-designer + sop-test-design-csv |
| L4 | `docs/04_context_reference/` | ADR + Context | sop-architecture-design / sop-implementation-designer |

## Document Types

| Type | Location | Creator |
|------|----------|---------|
| Project PRD | `docs/01_requirements/project_prd.md` | sop-requirement-analyst |
| Module MRD | `docs/01_requirements/modules/[module]_mrd.md` | sop-requirement-analyst |
| Feature FRD | `docs/01_requirements/modules/[module]/[feature]_frd.md` | sop-requirement-analyst |
| Prototype | `docs/01_requirements/prototypes/[module]/` | sop-requirement-analyst |
| Architecture | `docs/02_logical_workflow/*.md` | sop-architecture-design |
| Implementation | `src/**/design.md` | sop-implementation-designer |
| Test Cases | `docs/03_technical_spec/test_cases/*.csv` | sop-test-design-csv |
| Test Code | `tests/*.test.[ext]` | sop-test-implementation |
| ADR | `docs/04_context_reference/adr_*.md` | sop-architecture-design / sop-implementation-designer |
