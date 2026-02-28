---
version: v3.0.0
updated: 2026-02-28
skill_type: implementation
---

# 实现类 Skill

> **职责**: 将规范翻译为代码

---

## 概述

实现类 Skill 负责将规范文档转换为代码和测试，是规范的具体实现。

---

## Skill 列表

### sop-code-explorer

**职责**: 探索代码，验证规范是否被遵循

**输入契约**:
```yaml
required_inputs:
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
  - name: "codebase"
    type: directory
    path: "src/"
```

**输出契约**:
```yaml
required_outputs:
  - name: "code_analysis_report"
    type: json
    path: "contracts/code-analysis.json"
    format:
      existing_implementations: ["已实现的功能"]
      missing_implementations: ["未实现的功能"]
      constraint_violations: ["约束违反"]
```

**行为契约**:
```yaml
preconditions:
  - "规范文档存在"
  - "代码库可访问"
postconditions:
  - "分析报告包含完整的实现状态"
invariants:
  - "不修改任何代码"
```

---

### sop-code-implementation

**职责**: 代码实现，将规范翻译为代码

**输入契约**:
```yaml
required_inputs:
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
```

**输出契约**:
```yaml
required_outputs:
  - name: "code_changes"
    type: git_diff
    path: "git commit"
    format: "符合项目代码规范的代码变更"
```

**行为契约**:
```yaml
preconditions:
  - "设计文档已通过审查"
  - "规范文档已确认"
postconditions:
  - "代码符合 P2/P3 级规范"
  - "代码通过 lint 检查"
invariants:
  - "禁止强制解包"
  - "禁止硬编码密钥"
```

---

### sop-test-implementation

**职责**: 测试实现，将 BDD 场景翻译为测试代码

**输入契约**:
```yaml
required_inputs:
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
  - name: "bdd_scenarios"
    type: section
    path: "spec_document.bdd_scenarios"
```

**输出契约**:
```yaml
required_outputs:
  - name: "test_code"
    type: file
    path: "tests/{module}/{name}-test.{ext}"
    format: "符合测试规范的测试代码"
```

**行为契约**:
```yaml
preconditions:
  - "BDD 场景已定义"
postconditions:
  - "测试覆盖所有 BDD 场景"
  - "测试代码符合 P3 级测试规范"
invariants:
  - "测试必须可重复执行"
```

---

## TDD 循环执行

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TDD 循环执行                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐                                                           │
│  │ sop-test-   │                                                           │
│  │ implementation│ ──▶ 红阶段：编写失败测试                                 │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ sop-code-   │                                                           │
│  │ implementation│ ──▶ 绿阶段：编写实现代码                                 │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐                                                           │
│  │ sop-code-   │                                                           │
│  │ implementation│ ──▶ 重构阶段：优化代码结构                               │
│  └─────────────┘                                                           │
│         │                                                                   │
│         ▼                                                                   │
│      重复循环                                                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 相关文档

- [Skill 索引](../index.md)
- [规范类 Skill](../specification/)
- [验证类 Skill](../verification/)

---

**文档所有者**: Skill 团队  
**最后审核**: 2026-02-28
