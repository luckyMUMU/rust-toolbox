---
name: "sop-test-implementation"
description: "Acceptance test implementation workflow from CSV test cases. Invoke after CSV test design is ready to generate L1-L4 test code and verification instructions."
version: v2.12.0
updated: 2026-02-25
layer: "测试"
load_policy:
  tier: 2
  auto_load_states: ["[WAITING_FOR_TEST_DESIGN]"]
  depends_on: ["sop-test-design-csv"]
---

# Test Implementation Workflow

## 侧重点

- 仅实现测试代码；禁止修改 CSV 用例
- 测试代码必须满足分层验收门禁（L1-L4）并可重复运行

## 触发条件

- 仅当存在已落盘的 CSV 测试用例且需要实现测试代码时 → 必须调用本 Skill
- 仅当 CSV 存在缺口/歧义影响实现时 → 必须进入 `[USER_DECISION]`

## Input

- CSV 测试用例：`docs/03_technical_spec/test_cases/*.csv`
- 接口信息（只读）：L3 design.md / 代码接口签名（只用于对齐调用方式）
- 运行约束：`05_constraints/acceptance_criteria.md`

## Workflow Steps

### Step 1: Validate CSV Contract

- CMD: `TRACE_SOURCES(inputs) -> source_dependency_block`
- 必须校验 CSV 是否满足模板字段（参见 `04_reference/interaction_formats/test_case_csv.md`）
- 仅当字段缺失/冲突 → 必须进入 `[USER_DECISION]`

### Step 2: Implement Tests

- 按 L1-L4 分层实现（路径与组织可按项目覆写，但必须可复现）
- 仅能修改测试代码；禁止修改 CSV

### Step 3: Persist & Document How-To-Run

- 必须输出可执行命令契约（如何运行测试/如何定位失败）
- 审查标准：`04_reference/review_standards/test_code.standard.md`

## 来源与依赖准则

- 测试代码产出必须声明来源与依赖（CSV/设计依据/验收门禁）
- 当依据缺失或冲突无法消解时 → 必须进入 `[USER_DECISION]` 并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录

## Output

- 交付物（落盘）：`tests/**`（项目约定路径）
- Stop: `[WAITING_FOR_TEST_IMPLEMENTATION]`
- CMD: `TEST_IMPLEMENT(csv, design_refs) -> test_paths`

## Stop Points

- `[WAITING_FOR_TEST_IMPLEMENTATION]`: 测试代码已落盘，等待审查/继续实现
- `[USER_DECISION]`: CSV/接口信息不足导致无法实现可运行测试

## Failure Handling

- 当测试无法稳定复现（flaky/非确定性）时，必须停下并给出稳定化方案或进入 `[USER_DECISION]`

## Constraints

- 禁止修改 CSV：本 Skill 只读 CSV
- 测试必须可复现：必须提供运行指令与失败定位信息
- SSOT：状态/命令引用 `05_constraints/state_dictionary.md`、`05_constraints/command_dictionary.md`

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）
