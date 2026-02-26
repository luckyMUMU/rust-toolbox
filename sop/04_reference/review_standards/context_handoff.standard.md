---
version: v2.4.0
updated: 2026-02-22
artifact: Context Handoff
---

# 上下文压缩与交接合规标准

## Scope

- 适用场景：上下文不足、会话压缩、多人/多轮交接、长任务拆分执行
- 适用产物：任何进入停止点（如 `[USER_DECISION]`、`[DIR_WAITING_DEP]`、`[WAITING_FOR_*]`）的阶段输出
- 目的：在上下文变短/人员切换后，仍能基于证据恢复执行并保持 SSOT 一致

## SSOT

- 状态：`05_constraints/state_dictionary.md`
- 命令：`05_constraints/command_dictionary.md`
- 交接模板：`04_reference/interaction_formats/continuation_request.md`

## 严重等级与通过规则

- 🔴：缺少交接快照，导致无法复现已完成/未完成/当前停止点
- 🟡：快照存在但缺少证据（文件路径/命令输出/可复核引用）
- 🟢：结构与表达优化建议
- 通过门槛：🔴=0 且“必检项”全部满足

## 必检项（Hard requirements）

- 必须输出“交接快照”，且内容完整覆盖：
  - 当前状态（使用 SSOT 中的状态标记）
  - 已完成清单（每条必须有可复核证据）
  - 未完成 TODO（每条必须说明阻塞原因与解除条件）
  - 下一步推荐 Skill（必须给出输入与预期输出）
- 仅当发现信息不足/冲突/依赖缺口时：
  - 必须进入 `[USER_DECISION]`
  - 必须给出可选项（至少 2 个）与各自影响
  - 必须要求决策落盘（后续产物引用）

## 推荐项（Soft requirements）

- 快照应保持精简，但必须可执行恢复
- 证据引用优先使用“文件路径 + 行号范围”或“命令输出摘要”
- 将跨目录依赖明确归类为 `[DIR_WAITING_DEP]`，避免隐式等待

