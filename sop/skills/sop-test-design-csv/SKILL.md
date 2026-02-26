---
name: "sop-test-design-csv"
description: "Test design workflow for generating and maintaining CSV test cases from L2/L3 designs. Invoke when TDD/layered acceptance is enabled and test cases must be created or updated."
version: v2.12.0
updated: 2026-02-25
layer: "测试"
load_policy:
  tier: 2
  auto_load_states: ["[ARCHITECTURE_PASSED]", "[WAITING_FOR_DESIGN]"]
  depends_on: ["sop-architecture-design", "sop-implementation-designer"]
---

# CSV Test Design Workflow

## 侧重点

- 用例仅基于设计文档与验收标准；不从代码推导用例
- CSV 变更必须带版本与变更说明（落盘在 CSV 头部）

## 触发条件

- 仅当启用 TDD/分层验收且需要新增/更新测试用例时 → 必须调用本 Skill
- 仅当测试用例来源缺失/冲突会影响后续实现时 → 必须进入 `[USER_DECISION]`

## Input

- 设计依据（至少一项）：
  - L2 架构/逻辑文档（`docs/02_logical_workflow/*.md`）
  - L3 实现设计（`src/**/design.md`）
- 验收门禁：`05_constraints/acceptance_criteria.md`
- 目标 CSV 路径（或模块标识）

## Workflow Steps

### Step 1: Evidence & Scope

CMD: `TRACE_SOURCES(inputs) -> source_dependency_block`

### Step 2: Case Derivation (Design-only)

- 仅能基于设计与验收标准推导用例；禁止从代码实现倒推
- 必须覆盖：正常路径、边界条件、异常路径、关键状态流转

### Step 3: Persist CSV

- 模板：`04_reference/interaction_formats/test_case_csv.md`
- 必须写入：用例ID、输入、步骤、期望输出、层级（L1-L4）、版本与变更说明

## 来源与依赖准则

- CSV 必须包含“来源与依赖声明”（标准：`04_reference/review_standards/source_dependency.standard.md`）
- 当依据缺失或冲突无法消解时 → 必须进入 `[USER_DECISION]` 并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录

## Output

- 交付物（模板）：`04_reference/interaction_formats/test_case_csv.md`
- 交付物（落盘）：`docs/03_technical_spec/test_cases/*.csv`
- Stop: `[WAITING_FOR_TEST_DESIGN]`
- CMD: `TEST_DESIGN_CSV(design_refs, criteria) -> csv_path`
- **审查确认**：测试设计产出后须通过对用户的明确提问完成确认（如“是否接受当前用例集”“是否补充某类场景”“选 A/B/C”），输出须包含可操作确认项，待用户回复后再进入下一阶段

## Stop Points

- `[WAITING_FOR_TEST_DESIGN]`: 测试设计已落盘，等待确认/继续实现
- `[USER_DECISION]`: 设计依据不足/冲突导致无法生成可验证用例

## Failure Handling

- 当无法为关键设计章节生成可验证用例时，必须列出缺口并进入 `[USER_DECISION]`

## Constraints

- 只改 CSV：本 Skill 仅维护测试用例 CSV，不得修改代码
- 设计优先：用例必须映射到设计章节/验收条款
- SSOT：状态/命令引用 `05_constraints/state_dictionary.md`、`05_constraints/command_dictionary.md`

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）
