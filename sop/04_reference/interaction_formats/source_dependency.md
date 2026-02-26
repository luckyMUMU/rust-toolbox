---
version: v2.4.0
updated: 2026-02-22
---

# 来源与依赖声明格式

**使用**: 任意 Skill（产出需可追溯时）

---

## 产物头（必填）

```markdown
## 来源与依赖声明

### 产物
- 类型: [PRD/MRD/FRD | L2架构 | design.md | 测试设计 | 测试代码 | 代码Diff | 审查报告]
- 路径: [path]
- 版本/日期: [vX.Y.Z / YYYY-MM-DD]
```

---

## Inputs（前置产出依赖，必填）

```markdown
### Inputs
| 输入 | 来源产物 | 路径 | 摘要 | 是否必需 |
|------|----------|------|------|----------|
| 需求 | 用户输入 | (对话/链接/文件) | [摘要] | 必需 |
| 架构 | L2文档 | docs/02_logical_workflow/... | [摘要] | 可选/必需 |
| 实现设计 | design.md | src/**/design.md | [摘要] | 可选/必需 |
| 测试设计 | CSV/文档 | docs/03_technical_spec/test_cases/... | [摘要] | 可选/必需 |
```

规则：
- 除“需求主要来源于用户”外，后续阶段必须声明其依赖的前置产出（路径 + 摘要）。
```

---

## External Sources（外部来源，可选但需可复核）

```markdown
### External Sources
| 来源 | 链接/引用 | 用途 | 可复核性说明 |
|------|----------|------|--------------|
| 规范/文档 | [URL或RAG条目] | [用途] | [为何可信/如何复核] |
```
```

---

## Dependencies（依赖清单，必填）

```markdown
### Dependencies
| 依赖项 | 类型 | 依赖来源 | 影响 | 风险 |
|--------|------|----------|------|------|
| [接口/模块/约束/命令/状态] | [内部/外部] | [path/链接] | [影响] | [风险] |
```
```

---

## Gaps（缺口与不确定性，必填）

```markdown
### Gaps
| 缺口 | 原因 | 影响 | 建议选项 |
|------|------|------|----------|
| [缺少来源/缺少依赖/信息冲突] | [原因] | [影响] | A/B/C |
```
```

---

## User Decision（当缺口阻塞时必填）

```markdown
### User Decision
- 触发: [USER_DECISION]
- 主题: [SOURCE_MISSING | DEPENDENCY_MISSING | CONFLICT]
- 选项:
  - A: ...
  - B: ...
  - C: ...
- 用户选择: [A/B/C/自定义]
- 决策记录落盘: docs/04_context_reference/decisions/YYYY-MM-DD_<topic>.md
```
