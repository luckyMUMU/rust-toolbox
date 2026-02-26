---
name: "sop-implementation-designer"
description: "Implementation design workflow for creating detailed technical designs. Invoke when architecture is approved and ready for implementation design."
version: v2.12.0
updated: 2026-02-25
layer: "实现设计"
load_policy:
  tier: 2
  auto_load_states: ["[ARCHITECTURE_PASSED]"]
  depends_on: ["sop-architecture-design", "sop-architecture-reviewer", "sop-code-explorer"]
---

# Implementation Design Workflow

**位置**: `sop/skills/sop-implementation-designer/SKILL.md`

## 侧重点

- 目录级设计必须包含接口契约与跨目录依赖声明
- 当设计依据（L2/ADR）存在冲突 → 必须进入 `[USER_DECISION]` 并落盘决策记录

## 质量门控检查

> 完成实现设计后，必须执行以下门控检查：

| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 任务覆盖完整 | 所有L2原子操作已映射到L3实现 | [ ] |
| 依赖无循环 | 目录依赖图中无循环依赖 | [ ] |
| 任务可独立验证 | 每个任务有明确的验收标准 | [ ] |

**门控失败处理**：若任一检查项未通过，应记录失败原因并返回修正。

## 输入/输出契约定义要求

> 每个目录的design.md应包含以下契约定义：

### 输入契约

| 契约项 | 说明 |
|--------|------|
| 前置依赖 | L2架构设计文档、需求文档(FRD/MRD) |
| 输入数据 | 业务需求、技术约束、现有代码上下文 |
| 环境依赖 | 开发环境、技术栈版本、外部服务 |

### 输出契约

| 契约项 | 说明 |
|--------|------|
| 输出数据 | design.md文档、接口定义、数据模型 |
| 交付物 | 可执行的任务清单、验收标准 |
| 验收标准 | 门控检查项全部通过、用户确认 |

## 触发条件

- 架构已通过审查（通常已具备 `[ARCHITECTURE_PASSED]`）
- 需要为一个或多个目录生成可执行的实现设计（design.md）

## Input

- L2 架构文档（link 或内容）
- `sop-code-explorer` 审计报告（可选）
- ADR/RAG 参考（可选）
- 目录范围（dir）

## Workflow Steps

### Step 1: Directory-based Module Design

**Purpose**: Design implementation for each directory with design.md

**Actions**:
1. Review directory structure
2. Identify each module directory (where design.md will be placed)
3. Design module boundaries per directory
4. Define directory-level responsibilities

**Directory Module Map**:
输出：dir→module→responsibility（写入 design.md）

### Step 2: Tech Mapping with ADR Reference

**Purpose**: Map architecture to tech stack per directory, referencing ADRs

**Actions**:
1. **Review existing ADRs**:
   - Check `docs/04_context_reference/adr_*.md`（参见 04_reference/document_directory_mapping.md）
   - Note technology decisions from L2
   - Reference ADR in design.md

2. **Check RAG for tech references**:
   - Review `docs/04_context_reference/rag/external/tech_docs/`（参见 04_reference/document_directory_mapping.md）
   - Check for relevant technology documentation
   - Mark `[USER_DECISION]` if conflict with ADR

3. **Identify tech options per directory**:
   - Based on ADR decisions
   - Consider RAG references
   - Select best fit for each directory

4. **Document tech choices**:
   - Link to ADR if exists
   - Reference RAG if used
   - Record rationale

### Step 3: Directory-level Interface Contract

**Purpose**: Define interfaces between directories

**Actions**:
1. Define input/output for each directory
2. Specify data structures
3. Document error handling
4. **Define cross-directory interface contracts**

**Interface Contract Template**:
写入位置：design.md 的“Interface Contract / Directory Dependencies”章节

### Step 4: Cross-Directory Dependency Design

**Purpose**: Design how directories interact

**Actions**:
1. Identify dependencies between directories
2. Design interface contracts
3. Define dependency direction
4. Avoid circular dependencies

**Dependency Design Output**:
输出：cross_dir_deps（写入 design.md）

### Step 5: Task Decomposition (Per Directory)

**Purpose**: Create executable tasks per directory

**Actions**:
1. Break down work per directory
2. Define dependencies between directory tasks
3. Estimate effort per directory
4. Identify parallelizable tasks

**Task List Template**:
输出：task_list（写入 design.md）

### Step 6: Test Strategy (Per Directory)

**Purpose**: Define testing approach per directory

**Actions**:
1. Unit test scope per directory
2. Integration test scope for cross-directory
3. Coverage targets per directory

### Step 7: Reference Documentation

**Purpose**: Link to ADRs and RAG references

**Actions**:
1. **List referenced ADRs**:
   - Link to `docs/04_context_reference/adr_*.md`
   - Note which decisions affect this directory

2. **List RAG references**:
   - Link to `docs/04_context_reference/rag/` files
   - Document external knowledge used

3. **Conflict check**:
   - Compare design with ADR decisions
   - Compare with RAG references
   - Mark `[USER_DECISION]` if conflict found

参考：04_reference/knowledge_management.md

### Step 8: Gate Check

**Purpose**: Execute quality gate check for implementation design phase

**Actions**:
CMD: `GATE_CHECK(design_doc, gate='GATE_DESIGN')`

**Gate Check Items**:
| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 任务覆盖完整 | 所有L2原子操作已映射到L3实现 | [ ] |
| 依赖无循环 | 目录依赖图中无循环依赖 | [ ] |
| 任务可独立验证 | 每个任务有明确的验收标准 | [ ] |

**State Transition**:
- 通过 → `[WAITING_FOR_DESIGN]`
- 失败 → `[GATE_FAILED]` → 用户决策

## 来源与依赖准则

- 必须声明输入来源与依赖（L2/审计报告/ADR/RAG等），并优先用 `TRACE_SOURCES(inputs)` 固化“来源与依赖声明”
- 当关键来源缺失或冲突无法消解时，必须进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录
- 标准：04_reference/review_standards/source_dependency.standard.md

## Output

- 交付物：`<module_dir>/design.md`（每个独立目录必须产出）
- 交付物（模板）：04_reference/document_templates/implementation_design.md
- Stop: `[WAITING_FOR_DESIGN]`
- CMD: `IMPL_DESIGN(l2, dir)`

## Stop Points

- `[WAITING_FOR_DESIGN]`: 设计已完成，等待用户确认
- `[USER_DECISION]`: ADR 冲突/技术选型冲突影响实现路线

## design.md Rules (Per Directory)

**Directory-based**:
- Create design.md in each module directory
- One design.md per independent module directory
- Include cross-directory interface contracts

**Complexity-based**:
| Complexity | Lines | Action |
|------------|-------|--------|
| Low | <100 | 仅在非目录调度/快速路径可跳过；目录调度下使用极简 design.md |
| Medium | 100-500 | Brief design.md + interface contracts |
| High | >500 | Full design.md + detailed contracts |

**Required**: Interface contract with cross-directory dependencies

极简 design.md 模板（目录调度下 Low complexity 允许）：
```markdown
# [Module/Directory] Design

## Scope
- Directory: [path]
- Goal: [one sentence]

## Interfaces
- Inputs: [list]
- Outputs: [list]
- Errors: [list]

## Dependencies
- [dep_dir]: [purpose]
```

## Constraints

- Project-specific
- Traceable to architecture
- Clear interfaces
- Actionable tasks
- Must reference SSOT when using states/commands: 05_constraints/state_dictionary.md, 05_constraints/command_dictionary.md
- **Directory-level design**
- **Cross-directory interface contracts**
- **Dependency direction must be clear**

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）

## Failure Handling

- 当接口契约/依赖方向不清晰导致无法拆分任务时，必须停止推进并补齐 design.md 相关章节
