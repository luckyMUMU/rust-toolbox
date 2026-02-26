---
version: v2.12.0
updated: 2026-02-25
---

# Skill 矩阵（SSOT）

---

## 1. Skill 总览

| Skill | 层级 | 职责摘要 | 典型输入 | 典型输出 | 停止点 | 必须落盘交付物 | 质量门控 |
|------|------|----------|----------|----------|--------|----------------|----------|
| sop-workflow-orchestrator | 编排 | 分诊、路径选择、调用链编排 | user_request + context | 路径+调用链+下一步命令式指令 | `[USER_DECISION]` | `04_reference/interaction_formats/router_triage.md` | - |
| sop-code-explorer | 编排 | 代码库检索/审计/上下文提取 | scope + targets | 审计摘要/定位结果 | `[USER_DECISION]` | `04_reference/interaction_formats/code_audit_report.md` | - |
| sop-requirement-analyst | 需求 | L1-L3 需求分层与落盘 | user_request + constraints | PRD/MRD/FRD/原型指引 | `[WAITING_FOR_REQUIREMENTS]` / `[GATE_FAILED]` / `[USER_DECISION]` | `04_reference/document_templates/*prd*.md` | 需求边界清晰、技术方案对齐、验收标准具体、关键假设确认 |
| sop-architecture-design | 设计 | L2 架构设计与技术选型 | PRD/MRD + constraints | 架构设计文档 | `[WAITING_FOR_ARCHITECTURE]` / `[GATE_FAILED]` / `[USER_DECISION]` | `04_reference/document_templates/architecture_design.md` | 架构图清晰、接口定义完整、与现有系统无冲突、设计可行 |
| sop-architecture-reviewer | 设计审查 | L2 架构审查与结论落盘 | 架构设计文档 | 审查报告（Pass/Fail） | `[ARCHITECTURE_PASSED]` / `[ARCHITECTURE_FAILED]` / `[USER_DECISION]` | `04_reference/interaction_formats/design_review.md` | - |
| sop-implementation-designer | 实现设计 | 目录级 L3 design.md 设计 | L2 设计 + 目标目录 | design.md | `[WAITING_FOR_DESIGN]` / `[GATE_FAILED]` / `[USER_DECISION]` | `04_reference/document_templates/implementation_design.md` | 任务覆盖完整、依赖无循环、每个任务可独立验证 |
| sop-test-design-csv | 测试设计 | 基于 L2/L3 生成 CSV 用例 | L2/L3 设计 + 验收标准 | CSV 测试用例 | `[WAITING_FOR_TEST_DESIGN]` / `[USER_DECISION]` | `04_reference/interaction_formats/test_case_csv.md` | - |
| sop-test-implementation | 测试实现 | 基于 CSV 实现验收测试代码 | CSV 用例 + 接口信息 | L1-L4 测试代码 | `[WAITING_FOR_TEST_IMPLEMENTATION]` / `[USER_DECISION]` | `04_reference/review_standards/test_code.standard.md` | - |
| sop-code-implementation | 实现 | 按 design.md 目录边界改代码 | design.md + dir_scope | 代码 Diff + 变更说明 | `[WAITING_FOR_CODE_REVIEW]` / `[DIR_WAITING_DEP]` / `[DIR_COMPLETED]` / `[GATE_FAILED]` / `[USER_DECISION]` | `04_reference/interaction_formats/worker_execution_result.md` | 代码规范、测试通过、文档同步 |
| sop-code-review | 质量 | 基于证据的代码审查（只输出报告） | Diff + 设计依据 | 审查报告 | `[WAITING_FOR_CODE_REVIEW]` / `[DIFF_APPROVAL]` / `[REVIEW_CONFLICT]` / `[USER_DECISION]` | `04_reference/interaction_formats/code_review.md` | - |
| sop-progress-supervisor | 监管 | 目录并行调度、等待/唤醒、熔断 | dir_map + statuses | 调度指令/熔断报告 | `[SCHEDULING]` / `[PARALLEL_EXECUTING]` / `[WAITING_DEPENDENCY]` / `[ALL_COMPLETED]` / `[FUSION_TRIGGERED]` / `[CYCLE_DETECTED]` / `[USER_DECISION]` | `04_reference/interaction_formats/supervisor_report.md` | - |
| sop-document-sync | 文档 | 更新索引/导航/CHANGELOG 同步 | change_set + artifacts | 文档同步变更 | `[已完成]` / `[GATE_FAILED]` / `[USER_DECISION]` | `04_reference/interaction_formats/worker_execution_result.md` | 需求实现、验收满足、质量达标 |
| sop-fast-path | 路径宏 | 快速路径调用链（编排宏） | change_request | 调用链与门禁 | `[WAITING_FOR_CODE_REVIEW]` / `[DIFF_APPROVAL]` / `[USER_DECISION]` | `03_workflow/fast_path.md` | - |
| sop-deep-path | 路径宏 | 深度路径调用链（编排宏） | change_request | 调用链与门禁 | `[WAITING_FOR_REQUIREMENTS]` / `[WAITING_FOR_ARCHITECTURE]` / `[ARCHITECTURE_PASSED]` / `[WAITING_FOR_DESIGN]` / `[SCHEDULING]` / `[WAITING_FOR_CODE_REVIEW]` / `[WAITING_FOR_TEST_DESIGN]` / `[WAITING_FOR_TEST_IMPLEMENTATION]` / `[DIFF_APPROVAL]` / `[USER_DECISION]` | `03_workflow/deep_path.md` | - |
| sop-tdd-workflow | 路径宏 | TDD 增强调用链（编排宏） | deep_path_context | 调用链与门禁 | `[WAITING_FOR_TEST_DESIGN]` / `[WAITING_FOR_TEST_IMPLEMENTATION]` / `[WAITING_FOR_ACCEPTANCE_REVIEW]` / `[USER_DECISION]` | `05_constraints/acceptance_criteria.md` | - |
| sop-capability-reuse | 复用 | 先复用→改进→新建→清理 | target + constraints | 复用方案/改造方案 | `[USER_DECISION]` | `04_reference/knowledge_management.md` | - |
| sop-design-placement | 规约 | design.md 落点与目录策略 | scope + targets | 落点决策与路径 | `[USER_DECISION]` | `04_reference/design_directory_strategy.md` | - |

---

## 2. 工作范围（Scope）规则

### 2.1 目录边界（实现类 Skill）

- 仅当存在 `src/**/design.md` 时 → `sop-code-implementation` 与 `sop-test-implementation` **必须**以该 `design.md` 所在目录作为工作边界。
- 禁止跨越边界直接修改其他目录；仅当发现跨目录依赖时 → **必须**进入 `[DIR_WAITING_DEP]` 并交由 `sop-progress-supervisor` 调度。
- 目录边界算法 SSOT：`04_reference/design_directory_strategy.md`。

### 2.2 测试隔离（CSV 与测试代码）

- 仅当处于 TDD/分层验收路径且需要测试资产变更时 → **只能**通过 `sop-test-design-csv` 变更 CSV。
- `sop-test-implementation` **禁止**修改 CSV；只能读取 CSV 并产出测试代码。
- `sop-code-implementation` **禁止**创建/修改 CSV；只能运行测试并根据失败结果修正代码。

---

## 3. 技能边界说明（Skill Boundaries）

### 3.1 sop-code-explorer vs sop-requirement-analyst

| 方面 | sop-code-explorer | sop-requirement-analyst |
|------|-------------------|-------------------------|
| **核心职责** | 代码库检索、上下文提取、依赖分析 | 需求整理与文档化、需求澄清 |
| **禁止行为** | 不做需求分析、不产出PRD/MRD/FRD | 不做代码检索（由explorer负责） |
| **协作场景** | 当需要分析现有代码以理解需求时：先调用 sop-code-explorer 提取上下文，再调用 sop-requirement-analyst 进行需求分析 |

### 3.2 sop-architecture-design vs sop-implementation-designer

| 方面 | sop-architecture-design | sop-implementation-designer |
|------|-------------------------|------------------------------|
| **设计层级** | L2（项目级架构设计） | L3（目录级实现设计） |
| **设计范围** | 跨模块、跨目录的技术决策 | 单目录内的详细实现设计 |
| **产出物** | 架构设计文档、ADR | design.md |
| **依赖关系** | 依赖需求文档 | 依赖架构设计文档 |

### 3.3 sop-code-review vs sop-architecture-reviewer

| 方面 | sop-code-review | sop-architecture-reviewer |
|------|-----------------|---------------------------|
| **审查对象** | 代码Diff | 架构设计文档 |
| **审查时机** | 实现阶段 | 设计阶段 |
| **审查重点** | 设计一致性、正确性、测试覆盖 | 架构合理性、技术选型、扩展性 |
| **输出** | 审查报告（通过/需修改/僵局） | 审查报告（Pass/Fail） |

### 3.4 路径宏Skills说明

| Skill | 类型 | 组成 |
|-------|------|------|
| sop-fast-path | 编排宏 | sop-code-explorer → sop-code-implementation → sop-code-review → sop-document-sync |
| sop-deep-path | 编排宏 | sop-requirement-analyst → sop-architecture-design → sop-architecture-reviewer → sop-implementation-designer → sop-progress-supervisor → sop-code-implementation → sop-code-review → sop-document-sync |
| sop-tdd-workflow | 编排宏 | sop-deep-path + sop-test-design-csv + sop-test-implementation |

**编排宏特点**：
- 不是独立技能，而是多个Skills的有序组合
- 通过预定义的调用链简化常见工作流
- 可根据实际需求动态调整调用链

---

## 4. 版本依赖声明

每个SKILL.md应包含版本依赖声明：

```yaml
---
name: "sop-code-implementation"
version: "v2.12.0"
depends_on:
  sop_components:
    state_dictionary: ">=v2.12.0"
    command_dictionary: ">=v2.12.0"
  skills:
    sop-code-explorer: ">=v2.12.0"
    sop-implementation-designer: ">=v2.12.0"
---
```

**版本兼容性检查**：
- CMD: `CHECK_DEPENDENCIES()` 验证所有依赖版本
- 报告不兼容项并提示升级方案
