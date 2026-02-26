---
name: "sop-code-review"
description: "Code review workflow for validating changes against design docs and common engineering practices. Invoke after implementation diff is ready and before user approval."
version: v2.12.0
updated: 2026-02-25
layer: "质量"
load_policy:
  tier: 1
  auto_load_states: []
  depends_on: []
---

# Code Review Workflow

## 侧重点

- 只输出审查报告；禁止直接修改代码
- 结论必须绑定证据（设计章节/验收标准/红线条款/RAG 引用）

## 触发条件

- 仅当已产出可审查 Diff 且需要放行/返工判定时 → 必须调用本 Skill
- 仅当审查依据（设计/验收/红线）缺失或冲突无法消解时 → 必须进入 `[USER_DECISION]`

## Input

- Diff（link 或内容摘要）
- 设计依据（至少一项）：
  - L2 架构文档（docs/02_logical_workflow/*.md）
  - L3 实现设计（src/**/design.md）
  - 测试设计/验收标准（docs/03_technical_spec/test_cases/*.csv / 05_constraints/acceptance_criteria.md）
- 约束依据：
  - 安全与供应链红线（05_constraints/security_supply_chain.md）
  - 目录边界与跨目录协作规则（03_workflow/deep_path.md + 05_constraints/state_dictionary.md）

## Review Standards

- 代码 Diff：04_reference/review_standards/code_diff.standard.md
- 测试代码：04_reference/review_standards/test_code.standard.md
- 来源与依赖：04_reference/review_standards/source_dependency.standard.md
- 报告质量：04_reference/review_standards/review_report.standard.md
- 项目可覆写（可选）：04_reference/review_standards/profiles/<project>.md（模板：04_reference/review_standards/_project_profile.md）

## Workflow Steps

### Step 1: Scope & Evidence Collection

**Purpose**: Make review checkable and traceable

**Actions**:
1. Identify affected files/interfaces
2. Map changes to design sections (L2/L3)
3. Identify required tests/quality gates (lint/typecheck/acceptance)
4. Record any external references used for review into RAG when they are not already captured（参见 04_reference/knowledge_management.md）

### Step 2: Dimension Review

**Purpose**: Review with a stable checklist

**Dimensions**:
1. **设计一致性**: 接口/行为/错误码/边界与设计一致
2. **正确性**: 边界条件、异常路径、并发/幂等等关键点
3. **测试与验收**: 覆盖与受影响范围匹配；分层验收门禁满足
4. **安全与供应链**: 密钥/权限/输入校验/依赖治理满足红线
5. **可维护性**: 复杂度、可读性、重复、命名、结构清晰
6. **可观测性**: 日志/错误信息/可追踪性不泄露敏感信息
7. **性能风险**: 明显的 O(N^2)、无界循环、无超时重试策略等

### Step 3: Issue Identification & Severity

**Severity**:
- 🔴 Critical: Correctness/Security/Boundary/Contract breaking
- 🟡 Warning: Quality/Maintainability/Perf risk
- 🟢 Suggestion: Nice to have

### Step 4: Iteration

**Max**: 3 rounds

**Flow**:
```
Round 1: Identify issues → sop-code-implementation fixes
Round 2: Verify fixes → New issues?
Round 3: Final check → Pass or deadlock
```

When deadlock happens:
- Mark `[USER_DECISION]` and provide options

## 来源与依赖准则

- 审查报告必须包含“来源与依赖声明”（标准：04_reference/review_standards/source_dependency.standard.md），并优先用 `TRACE_SOURCES(inputs)` 固化
- 当审查依据缺失或冲突无法消解时，必须进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录

## Output

- 模板：04_reference/interaction_formats/code_review.md
- CMD: `CODE_REVIEW(diff, design_refs)`（pre: `[WAITING_FOR_CODE_REVIEW]`；post: `[DIFF_APPROVAL]` / `[DIR_WORKING]` / `[USER_DECISION]`）
- **审查确认**：审查结论须通过对用户的明确提问完成确认；输出须包含可操作确认项（如“是否放行”“是否返工并采纳某条”“选 A/B/C”），使用 `ASK_USER_DECISION(topic, options)` 或等价形式，待用户回复后再进入下一状态

## Constraints

- 只审查不修改：`sop-code-review` 不改代码，只输出审查结论与建议
- 证据优先：结论必须绑定到设计章节/验收标准/红线条款或 RAG 引用
- 目录边界合规：禁止建议跨越 design.md 边界的直接修改路径
- 外部规范引用必须沉淀：行业规范/最佳实践若用于决策或阻塞项，需落到 RAG 并在报告中引用

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）

## Stop Points

- `[DIFF_APPROVAL]`: 审查通过，- `[DIR_WORKING]`: 需修改（Strike 1/2）
- `[USER_DECISION]`: 审查依据缺失/冲突，或 3 轮迭代无法收敛

## 审查失败处理路径

### 单审模式
```
[WAITING_FOR_CODE_REVIEW] → REVIEW() → 
  ├── 通过 → [DIFF_APPROVAL]
  ├── 需修改(Strike 1) → [DIR_WORKING] (修复)
  ├── 需修改(Strike 2) → [DIR_WORKING] (修复)
  └── 需修改(Strike 3) → [USER_DECISION] (僵局处理)
```

### 僵局处理选项
```
[USER_DECISION] → ASK_USER_DECISION("审查僵局", [
  "采纳审查意见并修复",
  "跳过当前审查点（记录分歧）",
  "启动第二意见审查",
  "终止任务"
])
→ 用户选择后执行：
  - 修复 → [DIR_WORKING]
  - 跳过 → [DIFF_APPROVAL] (记录分歧)
  - 第二意见 → PARALLEL_REVIEW()
  - 终止 → [已完成]
```

### 第二意见审查触发条件
- Strike 3 自动触发
- 用户明确要求
- 涉及安全/性能敏感模块

## Failure Handling

- 当 Diff 无法映射到任何设计依据且影响放行结论时，必须进入 `[USER_DECISION]` 并给出补全证据清单
