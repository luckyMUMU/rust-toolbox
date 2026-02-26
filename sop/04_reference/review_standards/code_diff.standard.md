---
version: v2.4.0
updated: 2026-02-22
artifact: Code Diff
---

# 代码 Diff 审查标准

## Scope

- 适用产物：代码变更 Diff（含实现代码与测试代码）
- 审查者：`sop-code-review`
- 实现者：`sop-code-implementation`（功能代码）/ `sop-test-implementation`（测试代码，按范围）

## SSOT

- 实现边界：`04_reference/design_directory_strategy.md`
- 状态/命令：`05_constraints/state_dictionary.md`、`05_constraints/command_dictionary.md`
- 安全与供应链红线：`05_constraints/security_supply_chain.md`
- 分层验收标准：`05_constraints/acceptance_criteria.md`
- 编码原则：`05_constraints/coding_principles.md`

## 严重等级与通过规则

- 🔴：违背设计契约/边界违规/安全红线/测试不可运行或缺失，必须修复
- 🟡：可维护性/性能风险/可观测性缺失，建议修复
- 🟢：优化建议，可选采纳
- 通过门槛：🔴=0 且“必检项”全部满足

## 必检项（Hard requirements）

- 设计一致性：与 L2/L3 设计一致（接口/行为/错误语义），任何偏离必须有 ADR 或设计变更记录
- 边界合规：不跨 design.md 目录边界；若存在跨目录诉求，必须使用规定的依赖/请求机制
- 测试与验收：必要层级测试已补齐，且审查范围内可证明可运行
- 安全与供应链：不引入密钥泄露/权限扩大/未校验输入/危险依赖等红线问题
- 回归风险：变更影响面可解释，关键行为变更有测试/验收覆盖
- 抽象层级一致性：同一方法内调用应处于统一抽象层；对表操作优先复用 CRUD/Repository/DAO，避免业务方法混写持久化细节

## 推荐项（Soft requirements）

- 可读性：复杂逻辑有清晰结构，避免不必要的技巧性实现
- 可观测性：关键路径有合理的错误处理与可定位信息
- 性能：避免显而易见的热点与无界增长；必要时给出证据或保护

## 证据要求（Evidence）

- 每个 🔴 问题必须附带：文件位置、设计依据（章节/段落）、具体修复建议
- 对“测试通过”的声明必须给出可复核证据（日志/命令输出/报告文件路径，按项目能力）

## 项目可配置项（Project knobs）

- 必须满足的测试层级集合（默认：与变更风险匹配，不低于 L1/L2）
- 安全审查强度（默认：红线强制 + 高风险点额外检查）
- 允许的跨目录例外与审批流程
