---
name: "sop-workflow-orchestrator"
description: "Workflow orchestration for task triage and path selection. Invoke on new task to select path (fast/deep/TDD) and produce a Skill call chain with persisted artifacts."
version: v2.12.0
updated: 2026-02-25
layer: "编排"
load_policy:
  tier: 1
  auto_load_states: []
  depends_on: []
---

# Workflow Orchestration（Skill-first）

**位置**: `sop/skills/sop-workflow-orchestrator/SKILL.md`

## 侧重点

- 优先最小可行调用链；仅当风险/范围不清时才升级深度路径
- 所有结论必须绑定"来源与依赖声明"；缺口直接触发 `[USER_DECISION]`

## 触发条件

- 接收到新任务，需要选择路径（fast/deep/TDD）并输出 Skill 调用链
- 任务范围/风险不明确，需要先做复杂度判断与依赖判断

## Input

- user_request
- context(project_type/related_files/urgency/constraints)

## Workflow Steps

### Step 1: Analyze Task Complexity
CMD: `FAST_PATH_CHECK(change) -> allow|upgrade`
CMD: `TDD_CHECK(scope) -> on|off`

### Step 2: Select Path

**Fast Path**（必须同时满足）：
- Single file + <30 lines + no logic change

**Deep Path**（满足任一）：
- Cross-file / new feature / refactor / API change / architecture

**TDD Deep Path**（Deep Path 且满足任一）：
- Core business / complex logic / high coverage requirement

### Step 3: Directory Structure Analysis (Deep Path)

**Purpose**: Prepare for directory-based parallel execution

CMD: `LIST_DESIGN_MD(root) -> design_list`

### Step 4: Compose Skill Call Chain

**Fast Path Flow**:
```
sop-code-explorer → sop-code-implementation → sop-code-review → sop-document-sync
```

**Deep Path Flow (Directory-based)**:
```
New project:
sop-requirement-analyst → sop-architecture-design → sop-architecture-reviewer
→ sop-implementation-designer → sop-progress-supervisor → sop-code-implementation
→ sop-code-review → sop-document-sync

Feature:
sop-requirement-analyst → sop-implementation-designer
→ sop-progress-supervisor → sop-code-implementation → sop-code-review → sop-document-sync
```

**TDD Deep Path Flow (Directory-based)**:
```
... deep path ...
→ sop-test-design-csv
→ sop-test-implementation
→ sop-code-implementation（运行验收 + 修正代码）
```

### Step 5: Parallel Execution Plan

**Purpose**: Define directory-based parallel execution strategy

CMD: `SCHEDULE_DIRS(design_list) -> dir_map`

## 来源与依赖准则

- 必须声明分诊/编排依据来源与依赖（变更范围/风险/目录结构/约束等），并优先用 `TRACE_SOURCES(inputs)` 固化“来源与依赖声明”
- 当无法判断或存在冲突时必须中断：进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录
- 标准：04_reference/review_standards/source_dependency.standard.md

## Output

- 交付物（模板）：04_reference/interaction_formats/router_triage.md
- CMD: `ROUTE(task)`（必须输出：路径、Skill 调用链、下一步命令式指令）

## Stop Points

- `[USER_DECISION]`: 路径选择存在冲突/不确定且影响后续执行

## Failure Handling

- 当输入信息不足导致无法判定路径时，必须进入 `[USER_DECISION]` 并给出选项与代价对比

## Constraints

- Must accurately judge complexity
- Must consider dependencies
- Must provide clear next steps
- Must check TDD conditions
- Must reference SSOT when using states/commands: 05_constraints/state_dictionary.md, 05_constraints/command_dictionary.md
- Must persist output via the router triage template (artifact)
- **Must analyze directory structure for parallel execution**
- **Must create directory-based execution plan**
- **Must assign directory scope to implementation skills**

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）
