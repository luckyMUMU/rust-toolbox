---
name: "sop-architecture-reviewer"
description: "Architecture review workflow with multi-round iteration. Invoke when architecture design is complete and needs quality review."
version: v2.12.0
updated: 2026-02-25
layer: "设计审查"
load_policy:
  tier: 2
  auto_load_states: ["[WAITING_FOR_ARCHITECTURE]"]
  depends_on: ["sop-architecture-design"]
---

# Architecture Review Workflow

**位置**: `sop/skills/sop-architecture-reviewer/SKILL.md`

## 侧重点

- 只输出审查报告；禁止直接修改架构文档
- 审查结论必须绑定证据（设计章节/验收标准/红线条款）

## 触发条件

- 架构设计已完成，需要进行质量审查并给出可执行的修复清单
- 进入多轮审查迭代（最多 3 轮），直至通过或进入用户决策

## Input

- L2 架构文档（link 或内容）
- 目录结构（可选）
- 当前轮次 N（1-3）
- 上轮回复摘要（可选）

## Review Standards

- 标准：04_reference/review_standards/architecture_design.standard.md
- 来源与依赖：04_reference/review_standards/source_dependency.standard.md
- 报告质量：04_reference/review_standards/review_report.standard.md
- 项目可覆写（可选）：04_reference/review_standards/profiles/<project>.md（模板：04_reference/review_standards/_project_profile.md）

## Workflow Steps

### Step 1: Dimension Review

**Purpose**: Check 6 dimensions

维度：完整性/一致性/可行性/性能/安全/可扩展

### Step 2: Directory Structure Review

**Purpose**: Review directory-based design

**Actions**:
1. Check directory boundaries are clear
2. Verify cross-directory interfaces
3. Review dependency directions
4. Check for circular dependencies

### Step 3: Issue Identification

**Purpose**: Find problems

**Severity**:
- 🔴 Critical: Must fix
- 🟡 Warning: Should fix
- 🟢 Suggestion: Nice to have

### Step 4: Iteration

**Purpose**: Resolve issues

**Max**: 3 rounds

**Flow**:
```
Round 1: Identify issues → sop-architecture-design fixes
Round 2: Verify fixes → New issues?
Round 3: Final check → Pass or deadlock
```

## 来源与依赖准则

- 审查报告必须包含“来源与依赖声明”（标准：04_reference/review_standards/source_dependency.standard.md），并优先用 `TRACE_SOURCES(inputs)` 固化
- 当审查依据缺失或冲突无法消解时，必须进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录

## Output

- 交付物（模板）：04_reference/interaction_formats/design_review.md
- 交付物（落盘）：`temp/architecture_review.md`
- CMD: `ARCH_REVIEW(l2)`（post: `[ARCHITECTURE_PASSED]` / `[USER_DECISION]`）
- **审查确认**：审查结论须通过对用户的明确提问完成确认；输出须包含可操作确认项（如“是否通过”“是否采纳某条修订”“选 A/B/C”），使用 `ASK_USER_DECISION(topic, options)` 或等价形式，待用户回复后再进入下一状态

## Stop Points

- `[ARCHITECTURE_PASSED]`: 审查通过
- `[USER_DECISION]`: 审查僵局/冲突无法在 3 轮内收敛

## Constraints

- 6 dimensions
- Max 3 rounds
- Constructive feedback
- Clear severity levels
- Must reference SSOT when using states/commands: 05_constraints/state_dictionary.md, 05_constraints/command_dictionary.md
- **Directory structure review**
- **Cross-directory dependency check**

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）

## Failure Handling

- 当达到最大轮次仍存在 Critical 且无法形成一致修复方案时，必须进入 `[USER_DECISION]`
