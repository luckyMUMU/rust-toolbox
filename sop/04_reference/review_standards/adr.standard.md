---
version: v2.4.0
updated: 2026-02-22
artifact: ADR
---

# ADR 审查标准

## Scope

- 适用产物：ADR（`04_reference/document_templates/adr.md` 的实例）
- 审查者：`sop-architecture-reviewer` / `sop-code-review`（按决策类型）
- 设计者：`sop-architecture-design` / `sop-implementation-designer`

## SSOT

- 工作流与命令：`03_workflow/index.md`、`05_constraints/command_dictionary.md`
- 安全与供应链红线：`05_constraints/security_supply_chain.md`

## 严重等级与通过规则

- 🔴：缺失关键要素导致无法复核/不可执行/风险不可控，必须补齐
- 🟡：证据不足或影响面描述不足，建议补齐
- 🟢：表达/结构优化，可选采纳
- 通过门槛：🔴=0 且所有“必检项”满足

## 必检项（Hard requirements）

- 决策问题定义明确：要解决什么、边界是什么、非目标是什么
- 至少 2 个备选方案：含对比维度与放弃理由
- 证据可追溯：外部资料必须可复查（RAG/引用），内部假设必须显式
- 影响面明确：对接口/数据/成本/安全/性能/团队流程的影响可判断
- 迁移与回滚：若涉及替换/升级，必须说明迁移路径与回滚策略
- 安全与合规：不违反红线；若引入新依赖，必须说明治理策略

## 推荐项（Soft requirements）

- 决策有效期：何时需要重新评估（触发条件）
- 风险清单：按严重程度列出并给出缓解策略
- 监控信号：用于验证决策是否正确的观测指标

## 证据要求（Evidence）

- 每个备选方案必须至少包含：优点/缺点/风险/实施成本/对现有系统影响

## 项目可配置项（Project knobs）

- 备选方案数量下限（默认：2）
- 需要强制 ADR 的决策类型集合（默认：引入关键依赖/变更核心接口/变更数据模型/变更安全边界）
