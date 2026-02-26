---
name: "sop-requirement-analyst"
description: "Requirement analysis workflow for multi-level requirements (L1 PRD / L2 MRD / L3 FRD). Invoke when starting a new project, module, or feature, or when requirements need clarification."
version: v2.12.0
updated: 2026-02-25
layer: "需求"
load_policy:
  tier: 2
  auto_load_states: ["[ROUTE_DEEP]"]
  depends_on: ["sop-code-explorer"]
---

# Requirement Analysis Workflow

## 侧重点

- 需求分层必须可落盘、可验收、可追溯
- 仅当存在不确定/冲突需求会影响设计或实现 → 进入 `[WAITING_FOR_REQUIREMENTS]` 或 `[USER_DECISION]`

## 质量门控检查

> 完成需求分析后，必须执行以下门控检查：

| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 需求边界清晰 | 需求范围明确，无歧义 | [ ] |
| 技术方案对齐 | 技术约束与现有架构一致 | [ ] |
| 验收标准具体 | 每个需求有可测试的验收标准 | [ ] |
| 关键假设确认 | 所有关键假设已与用户确认 | [ ] |

**门控失败处理**：若任一检查项未通过，应记录失败原因并返回修正。

## 中断恢复状态保存

> 当需求阶段中断时，应保存以下状态到continuation_request：

- 当前需求分析进度
- 已确认的需求点
- 待澄清的问题列表
- 用户决策记录

## 触发条件

- 仅当开始新项目/新模块/新功能，或需求存在缺口需要澄清时 → 必须调用本 Skill
- 仅当输入不足或存在冲突且影响后续设计/实现时 → 必须进入 `[WAITING_FOR_REQUIREMENTS]` 或 `[USER_DECISION]`

## Input

- user_request
- background/constraints/related_docs
- project_type（new project/module/feature）

## Workflow Steps

### Step 1: Requirement Level Identification

**Determine level**:
| Level | Condition | Output |
|-------|-----------|--------|
| L1 | New project | Project PRD |
| L2 | New module | Module MRD |
| L3 | New feature | Feature FRD |

### Step 2: Dialogue Elicitation

**Purpose**: Clarify user needs

Rounds: core → clarify → confirm

### Step 3: Multi-Level PRD Generation
模板：04_reference/document_templates/{project_prd.md,module_mrd.md,feature_frd.md,prototype_guide.md}
目录：04_reference/document_directory_mapping.md

### Step 4: Multi-Dimension Analysis
维度：Business/User/Function/Tech/Risk/Acceptance（每项输出：结论+风险）

**Tech 维度额外要求**:
1. 针对关键技术点做候选方案调研（>=2 个可选项，允许保持“非绑定”）
2. 形成对比表：适用场景/约束/风险/成本/可维护性
3. 外部参考若用于结论或边界，必须沉淀到 RAG 并在文档中引用（参见 04_reference/knowledge_management.md）

### Step 5: User Confirmation

**Purpose**: Validate requirements

**Checklist**:
- [ ] Requirements correct?
- [ ] Scope accurate?
- [ ] Priority reasonable?
- [ ] Any missing?

**Stop Point**: `[WAITING_FOR_REQUIREMENTS]`

### Step 6: Gate Check

**Purpose**: Execute quality gate check for requirements phase

**Actions**:
CMD: `GATE_CHECK(requirements_doc, gate='GATE_REQUIREMENTS')`

**Gate Check Items**:
| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 需求边界清晰 | 需求范围明确，无歧义 | [ ] |
| 技术方案对齐 | 技术约束与现有架构一致 | [ ] |
| 验收标准具体 | 每个需求有可测试的验收标准 | [ ] |
| 关键假设确认 | 所有关键假设已与用户确认 | [ ] |

**State Transition**:
- 通过 → `[WAITING_FOR_REQUIREMENTS]`
- 失败 → `[GATE_FAILED]` → 用户决策

## 来源与依赖准则

- 必须声明需求来源与依赖（用户输入/历史文档/代码现状/外部参考等），并优先用 `TRACE_SOURCES(inputs)` 固化“来源与依赖声明”
- 当关键来源缺失或冲突无法消解时，必须进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录
- 标准：04_reference/review_standards/source_dependency.standard.md

## Output

- Stop: `[WAITING_FOR_REQUIREMENTS]`
- CMD: `REQ_ANALYZE(input)`
- **审查确认**：需求确认须通过对用户的明确提问完成（如“是否接受当前范围”“是否补充某条”“选 A/B/C”），输出须包含可操作确认项，使用 `ASK_USER_DECISION(topic, options)` 或等价形式，待用户回复后再进入下一阶段

## Stop Points

- `[WAITING_FOR_REQUIREMENTS]`: 需求已落盘，等待确认
- `[USER_DECISION]`: 关键需求冲突不可消解或缺口影响后续设计/实现

## Constraints

- Must have multi-round dialogue
- Must cover 6 dimensions
- Tech dimension must include option survey and RAG references when applicable
- Must get user confirmation
- Must define clear boundaries
- Must support multi-level requirements
- Must create prototypes for UI projects

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）
