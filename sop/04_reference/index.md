---
version: v2.12.0
updated: 2026-02-25
---

# 参考文档

## 渐进式披露层级 (L1-L4)

参见 [document_directory_mapping.md](document_directory_mapping.md)（逻辑目录 → 项目实际目录映射）。

| 层级 | 目录 | 内容 | 格式 | 创建者 |
|------|------|------|------|--------|
| L1 | `01_concept_overview.md` | 核心概念 | Markdown | - |
| L2 | `docs/02_logical_workflow/` | 逻辑工作流 | `.md` | sop-architecture-design |
| L3 | `docs/03_technical_spec/` / `src/**/design.md` | 技术规格 | Markdown/YAML | sop-implementation-designer |
| L4 | `docs/04_context_reference/` | 决策参考 | `adr_*.md` | sop-architecture-design / sop-implementation-designer |

---

## 文档放置规则

| 目录 | 用途 | 权限 |
|------|------|------|
| `/docs` | 项目设计文档 | 动态创建更新 |
| `/docs/参考/` | SOP参考文档 | **非指定不变更** |
| `/docs/01_requirements/` | **需求文档 (L1-L3)** | **sop-requirement-analyst 产出** |
| `/docs/01_requirements/modules/` | **模块/功能需求 (L2-L3)** | **sop-requirement-analyst 产出** |
| `/docs/01_requirements/prototypes/` | **原型设计 (L3)** | **sop-requirement-analyst 产出** |
| `/docs/02_logical_workflow/` | 架构设计 (L2) | sop-architecture-design 产出 |
| `/docs/03_technical_spec/` | 技术规格 (L3) | sop-implementation-designer 产出 |
| `/docs/03_technical_spec/test_cases/` | **测试用例** | **sop-test-design-csv 产出** |
| `/docs/04_context_reference/` | 决策参考 (L4) | sop-architecture-design / sop-implementation-designer 产出 |
| `src/**/design.md` | 实现设计 (L3) | sop-implementation-designer 产出 |
| `tests/` | **测试代码** | **sop-test-implementation 产出** |

---

## L1-L3: 需求分层（sop-requirement-analyst）

### 需求层级

| 层级 | 文档 | 位置 | 内容 | 创建者 |
|------|------|------|------|--------|
| L1 | **Project PRD** | `01_requirements/project_prd.md` | 项目愿景、模块清单 | sop-requirement-analyst |
| L2 | **Module MRD** | `01_requirements/modules/[module]_mrd.md` | 模块功能、边界 | sop-requirement-analyst |
| L3 | **Feature FRD** | `01_requirements/modules/[module]/[feature]_frd.md` | 功能详情、交互 | sop-requirement-analyst |
| L3 | **Prototype** | `01_requirements/prototypes/[module]/` | 界面原型 | sop-requirement-analyst |

### 需求文档结构

```
docs/01_requirements/
├── project_prd.md              # L1: 项目级
├── modules/
│   ├── [module]_mrd.md         # L2: 模块级
│   └── [module]/
│       └── [feature]_frd.md    # L3: 功能级
└── prototypes/                 # L3: 原型设计
    └── [module]/
        ├── [feature]_wireframe.drawio
        ├── [feature]_mockup.fig
        └── [feature]_interaction.md
```

### 原型文件格式

| 类型 | 格式 | 工具 | 用途 |
|------|------|------|------|
| 线框图 | `.drawio`, `.png` | Draw.io, Figma | 低保真，快速迭代 |
| 高保真 | `.fig`, `.sketch`, `.xd` | Figma, Sketch, Adobe XD | 最终设计稿 |
| 交互说明 | `.md` | Markdown | 补充交互细节 |

---

## L2: 逻辑工作流 (`.md`)

**产出 Skill**: sop-architecture-design  
**规范**: 技术无关逻辑设计（Markdown 文档，伪代码用 `pseudo` 代码块）

### 伪代码规范
- **原子操作**: `UPPER_SNAKE_CASE` (例: `VALIDATE_INPUT`)
- **函数**: `lower_snake_case` (例: `process_data`)
- **缩进**: 4空格
- **注释**: 说明"为什么"

---

## L3: 技术规格

**产出 Skill**: sop-implementation-designer  
**规范**: 将L2逻辑流程映射为具体技术实现

### design.md规则
| 复杂度 | 行数 | 要求 |
|--------|------|------|
| 低 | <100 | 省略，代码注释 |
| 中 | 100-500 | 简要+接口契约 |
| 高 | >500 | 完整+详细契约 |

---

## L4: 决策参考 (ADR)

**产出 Skill**: sop-architecture-design / sop-implementation-designer  
**规范**: 记录关键决策的背景和理由

### ADR编号
```
ADR-[模块]-[序号]: [标题]
```

---

## 参考资料与知识沉淀

用户输入与外部知识沉淀规范请参考：[knowledge_management.md](knowledge_management.md)

---

## TDD测试文档

**产出 Skill**: sop-test-design-csv / sop-test-implementation  
**规范**: CSV格式测试用例，便于人工审核

### 测试用例位置
```
docs/03_technical_spec/test_cases/
├── [module]_test_cases.csv      # 测试用例 (sop-test-design-csv 产出)
└── [module]_test_plan.md        # 测试计划 (可选)
```

---

## 模板

| 模板 | 层级 | 用途 | Skill |
|------|------|------|------|
| **[项目PRD](document_templates/project_prd.md)** | **L1** | **项目级需求** | **sop-requirement-analyst** |
| [PRD（旧：单功能）](document_templates/prd.md) | L3 | 单功能PRD（不推荐） | sop-requirement-analyst |
| **[模块MRD](document_templates/module_mrd.md)** | **L2** | **模块级需求** | **sop-requirement-analyst** |
| **[功能FRD](document_templates/feature_frd.md)** | **L3** | **功能级需求** | **sop-requirement-analyst** |
| **[原型规范](document_templates/prototype_guide.md)** | **L3** | **原型设计** | **sop-requirement-analyst** |
| [架构设计](document_templates/architecture_design.md) | L2 | 逻辑工作流 | sop-architecture-design | 质量门控检查清单、架构图建议 |
| [实现设计](document_templates/implementation_design.md) | L3 | 技术规格 | sop-implementation-designer | 质量门控检查清单、输入/输出契约定义 |
| [ADR](document_templates/adr.md) | L4 | 决策参考 | sop-architecture-design / sop-implementation-designer |

---

## 规范与策略

| 文档 | 用途 |
|------|------|
| [设计指南](design_guide.md) | 设计决策的硬门槛与记录要求、Scope/边界与 design.md 落点策略 |
| [交互规范指南](spec_interactive_guide.md) | 规格文档交互式编写规范与问答策略 |
| [Prompt Pack 规范](prompt_pack.standard.md) | Prompt Pack 的组织形式与覆盖机制 |
| [编码原则](05_constraints/coding_principles.md) | 六大设计原则 + 方法逻辑层级一致性 + CRUD 分层复用 |
| [知识沉淀规范](knowledge_management.md) | 外部知识/用户输入/RAG 的沉淀与更新规则 |
| [RAG 目录说明](rag/README.md) | RAG 子目录与维护规则 |

---

## 交互格式

| 格式 | 用途 | Skill |
|------|------|------|
| [Progress Supervisor报告](interaction_formats/supervisor_report.md) | 进度/熔断/决策 | sop-progress-supervisor |
| [分诊输出](interaction_formats/router_triage.md) | 路径选择/调用链编排 | sop-workflow-orchestrator |
| [代码审计报告](interaction_formats/code_audit_report.md) | 只读检索/定位/证据输出 | sop-code-explorer |
| [设计审查报告](interaction_formats/design_review.md) | 架构审查结论 | sop-architecture-reviewer |
| [测试用例CSV格式](interaction_formats/test_case_csv.md) | CSV 用例格式与字段 | sop-test-design-csv |
| [执行结果](interaction_formats/worker_execution_result.md) | 执行变更汇总与落盘 | sop-code-implementation / sop-document-sync |
| [代码审查报告](interaction_formats/code_review.md) | 基于证据的审查输出 | sop-code-review |
| [来源与依赖声明](interaction_formats/source_dependency.md) | 来源/依赖/Gaps/决策声明 | sop-capability-reuse |
| [手动模式模板](interaction_formats/manual_mode_templates.md) | 手动触发路径/模式的请求模板 | sop-workflow-orchestrator |
| [续跑与恢复请求](interaction_formats/continuation_request.md) | 未完成/依赖等待/验证失败的续跑交接 | sop-workflow-orchestrator |
| [SSOT 漂移检测报告](interaction_formats/ssot_drift_detection_report.md) | SSOT 引用一致性检测 | 审查时 |

---

## 审查标准

> 全量审查标准见 [review_standards/index.md](review_standards/index.md)

| 标准 | 用途 |
|------|------|
| [上下文压缩与交接标准](review_standards/context_handoff.standard.md) | 压缩/交接后可恢复执行与证据可追溯 |
| [标准索引](review_standards/index.md) | 全量审查标准入口 |
| [来源与依赖合规标准](review_standards/source_dependency.standard.md) | 来源/依赖/Gaps 与决策闭环门槛 |
| [测试代码审查标准](review_standards/test_code.standard.md) | 测试实现质量门槛 |

---

## 状态标记

- SSOT: 05_constraints/state_dictionary.md + 05_constraints/command_dictionary.md
