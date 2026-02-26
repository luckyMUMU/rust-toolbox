---
version: v2.4.0
updated: 2026-02-22
artifact: Test Design
---

# 测试设计审查标准

## Scope

- 适用产物：CSV 测试用例、分层验收测试设计文档（L1-L4）
- 设计者：`sop-test-design-csv`
- 复核者：`sop-code-review`（或项目在 Profile 中指定）

## SSOT

- 分层验收标准：`05_constraints/acceptance_criteria.md`
- 状态/命令：`05_constraints/state_dictionary.md`、`05_constraints/command_dictionary.md`
- 测试资产隔离与边界：`02_skill_matrix/index.md`

## 严重等级与通过规则

- 🔴：覆盖缺失导致关键需求无法验证/用例不可执行/与需求或设计冲突，必须修复
- 🟡：覆盖不足或表达不清导致实现歧义，建议修复
- 🟢：优化建议，可选采纳
- 通过门槛：🔴=0 且“必检项”全部满足

## 必检项（Hard requirements）

- 可追踪：每个测试场景可追踪到需求/设计依据（FRD/L3/接口契约）
- 覆盖矩阵：主流程、边界、异常、权限/安全相关场景覆盖明确
- 可执行：输入、前置条件、预期输出可被实现为可运行测试
- 版本与变更：测试资产版本可识别；变更影响可解释
- 分层合理：L1-L4 的职责边界明确，避免层级混用

## 推荐项（Soft requirements）

- 数据最小化：用例数据最小且可复用，避免脆弱依赖
- 非功能：必要时包含性能/可靠性/幂等与一致性场景

## 证据要求（Evidence）

- 对“覆盖充分”的结论必须引用覆盖矩阵或用例列表
- 与设计冲突的地方必须提出明确问题并记录处理决策

## 项目可配置项（Project knobs）

- 必须覆盖的场景集合（按行业/合规调整）
- 必须输出的覆盖矩阵格式（按项目工具链调整）
- 必须设计的验收层级集合（L1-L4 子集）
