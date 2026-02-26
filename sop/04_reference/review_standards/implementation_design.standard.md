---
version: v2.4.0
updated: 2026-02-22
artifact: L3 Implementation Design (design.md)
---

# L3 实现设计审查标准（design.md）

## Scope

- 适用产物：目录内 `design.md`（`04_reference/document_templates/implementation_design.md` 的实例）
- 设计者：`sop-implementation-designer`
- 复核者：`sop-code-review`（按需要）

## SSOT

- 目录边界策略：`04_reference/design_directory_strategy.md`
- 工作流：`03_workflow/index.md`
- 状态/命令：`05_constraints/state_dictionary.md`、`05_constraints/command_dictionary.md`

## 严重等级与通过规则

- 🔴：边界违规/契约缺失/不可实现/与 L2 冲突/验收无法闭环，必须修复
- 🟡：可维护性/一致性/复杂度风险，建议修复
- 🟢：优化建议，可选采纳
- 通过门槛：🔴=0 且所有“必检项”满足

## 必检项（Hard requirements）

- 边界声明：本目录包含/不包含的内容明确，且符合目录策略
- 对外契约：对外接口、输入输出、错误语义、权限语义明确
- 依赖方向：跨目录依赖方向与约束明确，不产生循环依赖
- 失败路径：关键失败场景有处理策略（重试/补偿/降级/错误码）
- 测试接口：验收测试所需的可观测/可注入点明确（不泄露敏感信息）
- 变更影响：对现有调用方/数据/兼容性影响可判断

## 推荐项（Soft requirements）

- 数据模型与约束：关键字段、校验规则、幂等与一致性策略明确
- 性能：可能的热点与上界风险已识别
- 文档可导航：小节结构清晰，可被 `sop-code-review` 作为设计依据引用

## 证据要求（Evidence）

- 与 L2 的对应关系必须可追溯（章节映射或 ADR 引用）
- 任一跨目录交互必须给出契约描述与依赖理由

## 项目可配置项（Project knobs）

- 目录边界允许的例外类型与审批流程
- 必须覆盖的失败场景集合
- 是否强制给出 L1-L4 测试映射表
