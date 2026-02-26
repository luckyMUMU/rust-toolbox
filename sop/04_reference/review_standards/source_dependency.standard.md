---
version: v2.4.0
updated: 2026-02-22
artifact: Sources & Dependencies
---

# 来源与依赖合规审查标准

## Scope

- 适用产物：需求分析（PRD/MRD/FRD）、L2 架构、L3 design.md、测试设计、测试代码、代码审查报告、架构审查报告
- 目的：确保后续流程严格依赖前置产出；缺口必须中断并形成可追溯决策

## SSOT

- 状态：`05_constraints/state_dictionary.md`
- 命令：`05_constraints/command_dictionary.md`
- 模板：`04_reference/interaction_formats/source_dependency.md`

## 严重等级与通过规则

- 🔴：缺少来源/依赖声明、或缺口未触发用户决策、或决策未落盘
- 🟡：来源声明存在但证据不足/不可复核
- 🟢：结构与表达优化建议
- 通过门槛：🔴=0 且“必检项”全部满足

## 必检项（Hard requirements）

- 产物必须包含“来源与依赖声明”（按模板字段）
  - Inputs：必须包含前置阶段产物路径与摘要
  - Dependencies：必须列出关键依赖项
  - Gaps：必须列出缺口（若无缺口也要显式写“无”）
- 当无法定位来源或依赖时：必须进入 `[USER_DECISION]` 并提供选项
  - 必须在产物中记录用户选择
  - 必须将决策落盘为可引用文件，并在后续产物引用

## 推荐项（Soft requirements）

- External Sources 必须可复核（优先沉淀到项目 RAG/决策记录）
  - 引用外部规范时说明采用/不采用的理由
  - 对关键假设给出验证路径

## 项目可配置项（Project knobs）

- 必须声明的依赖类型集合（默认：接口契约/安全红线/验收标准/跨目录依赖）
  - 可扩展：合规/隐私/性能预算等
- 决策落盘目录（默认：`docs/04_context_reference/decisions/`）
