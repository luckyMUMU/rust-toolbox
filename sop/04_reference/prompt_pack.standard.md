---
version: v2.9.0
updated: 2026-02-24
status: deprecated
---

# Prompt Pack 规范（已废弃）

> **⚠️ 本文档已废弃**
> 
> 自 v2.8.0 起，`prompts/` 目录已删除。所有执行不变量已合并到：
> - 全局不变量 → `05_constraints/constraint_matrix.md`
> - 编排规则 → `03_workflow/index.md`
> - 侧重点 → 各 `skills/*/SKILL.md`
>
> 本文档仅作历史参考保留。

---

## 历史定义

**Prompt Pack** 曾是一组可组合的提示词模块，用于驱动一组 Skill 的稳定执行。Prompt Pack 的职责是：

- 固化执行不变量（命令式表达、停止点、落盘交付物、SSOT 引用）
- 为每个 Skill 提供可覆写的"执行风格/偏好/侧重点"
- 允许按项目或用户偏好切换 pack，而不改 Skill 合约（SKILL.md）

---

## 迁移映射

| 原位置 | 新位置 |
|--------|--------|
| `prompts/packs/default/00_system.md` | `05_constraints/constraint_matrix.md`（全局不变量章节） |
| `prompts/packs/default/01_operator.md` | `03_workflow/index.md`（编排入口与能力选择协议章节） |
| `prompts/packs/default/skills/*.md` | 各 `skills/*/SKILL.md`（侧重点章节） |

---

## 手动模式模板

手动模式模板仍然有效，位于：
- 模式与路径模板：`04_reference/interaction_formats/manual_mode_templates.md`
- 续跑与恢复请求：`04_reference/interaction_formats/continuation_request.md`
