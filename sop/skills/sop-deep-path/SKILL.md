---
name: "sop-deep-path"
description: "Deep path workflow for complex changes. Invoke when task is triaged as deep path (cross-file, new feature, refactor, API change)."
version: v2.12.0
updated: 2026-02-25
layer: "路径宏"
load_policy:
  tier: 3
  auto_load_states: ["[ROUTE_DEEP]"]
  depends_on: ["sop-requirement-analyst", "sop-architecture-design", "sop-architecture-reviewer", "sop-implementation-designer", "sop-progress-supervisor", "sop-code-implementation", "sop-code-review", "sop-document-sync", "sop-test-design-csv", "sop-test-implementation"]
---

# Deep Path Workflow

## 侧重点

- 仅当任务为跨文件/新功能/重构/API 变更等复杂变更时 → 必须调用本 Skill
- 仅当存在路径选择冲突或输入缺口影响后续阶段时 → 必须进入 `[USER_DECISION]`

## 触发条件

- 仅当任务为跨文件/新功能/重构/API 变更等复杂变更时 → 必须调用本 Skill
- 仅当存在路径选择冲突或输入缺口影响后续阶段时 → 必须进入 `[USER_DECISION]`

## Input

- Task: [desc]
- Context: type/scope/constraints

## Workflow Steps

### Step 1: Requirement Analysis

**Purpose**: Clarify requirements

CMD: `REQ_ANALYZE(input) -> [WAITING_FOR_REQUIREMENTS]`

**Stop Point**: `[WAITING_FOR_REQUIREMENTS]`

### Step 2: Code Audit (Optional)

**Purpose**: Impact assessment

CMD: `AUDIT(scope)` / `LIST_DESIGN_MD(root)`

### Step 3: Architecture Design

**Purpose**: Technology-agnostic design

CMD: `ARCH_DESIGN(prd) -> [WAITING_FOR_ARCHITECTURE]`

**Stop Point**: `[WAITING_FOR_ARCHITECTURE]`

### Step 4: Architecture Review

**Purpose**: Quality assurance

CMD: `ARCH_REVIEW(l2) -> [ARCHITECTURE_PASSED] | [USER_DECISION]`

**Max**: 3 rounds

**Stop Points**:
- Pass: `[ARCHITECTURE_PASSED]`
- Deadlock: `[USER_DECISION]`

### Step 5: Implementation Design (Directory-based)

**Purpose**: Project-specific design

CMD: `IMPL_DESIGN(l2, dir) -> [WAITING_FOR_DESIGN]`

**Stop Point**: `[WAITING_FOR_DESIGN]`

### Step 6: Directory Scheduling

**Purpose**: Create parallel execution plan

CMD: `SCHEDULE_DIRS(design_list) -> [SCHEDULING]`

**Stop Point**: `[SCHEDULING]`

### Step 7: Code Implementation (Parallel)

**Purpose**: Physical coding with directory-based parallel execution

CMD: `RUN_DIR_BATCH(depth_desc) -> IMPLEMENT(dir, design) -> [WAITING_FOR_CODE_REVIEW]`

**Stop Point**: `[WAITING_FOR_CODE_REVIEW]`

### Step 9: Code Review

**Purpose**: Validate changes against design docs and common practices

CMD: `CODE_REVIEW(diff, design_refs) -> [DIFF_APPROVAL]`

### Step 10: Test Design (Optional)

**Purpose**: Design test cases for acceptance criteria

**Condition**: TDD路径或需要测试资产

CMD: `TEST_DESIGN_CSV(design, criteria) -> [WAITING_FOR_TEST_DESIGN]`

**Stop Point**: `[WAITING_FOR_TEST_DESIGN]`

### Step 11: Test Implementation (Optional)

**Purpose**: Implement test code from test cases

**Condition**: 有CSV测试用例

CMD: `TEST_IMPLEMENT(csv, design_refs) -> [WAITING_FOR_TEST_IMPLEMENTATION]`

**Stop Point**: `[WAITING_FOR_TEST_IMPLEMENTATION]`

### Step 12: Document Maintenance

**Purpose**: Sync docs

CMD: `DOC_SYNC(scope) -> [已完成]`

## 来源与依赖准则

- 各阶段产物必须包含“来源与依赖声明”（标准：04_reference/review_standards/source_dependency.standard.md），并优先用 `TRACE_SOURCES(inputs)` 固化
- 当任一阶段出现关键来源/依赖缺口时必须中断：进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录

## Output

- 状态：`[已完成]`
- 产物：PRD / L2 / design.md / code / indexes
- 参考：05_constraints/command_dictionary.md

## Stop Points

- `[WAITING_FOR_REQUIREMENTS]`: 需求文档已落盘，等待确认
- `[WAITING_FOR_ARCHITECTURE]`: 架构设计已落盘，等待确认/进入审查
- `[ARCHITECTURE_PASSED]`: 架构审查通过，可进入实现设计
- `[WAITING_FOR_DESIGN]`: 目录级实现设计已落盘，等待确认
- `[SCHEDULING]`: 目录调度计划已生成，等待启动批次
- `[WAITING_FOR_CODE_REVIEW]`: 代码变更已就绪，等待代码审查
- `[WAITING_FOR_TEST_DESIGN]`: 测试设计已就绪，等待确认
- `[WAITING_FOR_TEST_IMPLEMENTATION]`: 测试代码已就绪，等待审查
- `[DIFF_APPROVAL]`: 审查通过，等待用户确认变更
- `[USER_DECISION]`: 输入不足/冲突/依赖缺口影响后续阶段，必须中断等待决策

## Constraints

- Must follow all stages
- Must pass reviews
- 3-strike rule applies
- Document must sync

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）

## 3-Strike Rule

| Strike | Condition | Action |
|--------|-----------|--------|
| 1 | Implementation fails | Auto-fix |
| 2 | Fails again | Audit + redesign |
| 3 | Fails again | **Break**, user decision |
