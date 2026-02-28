---
version: v3.0.0
updated: 2026-02-28
skill_type: specification
---

# 规范类 Skill

> **职责**: 生成规范文档

---

## 概述

规范类 Skill 负责将需求转换为规范文档，是 Spec-first 架构的核心。

---

## Skill 列表

### sop-requirement-analyst

**职责**: 需求分析，生成 P1/P2 级规范

**输入契约**:
```yaml
required_inputs:
  - name: "requirement_description"
    type: text
    validation: "长度 >= 10 字符"
```

**输出契约**:
```yaml
required_outputs:
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
    format: "Markdown，包含 Gherkin 场景"
```

**行为契约**:
```yaml
preconditions:
  - "需求描述清晰无歧义"
postconditions:
  - "规范文档包含完整的 BDD 场景"
  - "规范文档通过 P0 级约束验证"
invariants:
  - "规范必须使用领域语言"
```

---

### sop-architecture-design

**职责**: 架构设计，生成 P1 级规范

**输入契约**:
```yaml
required_inputs:
  - name: "system_requirements"
    type: file
    path: "specs/{name}-spec.md"
  - name: "constitution_docs"
    type: files
    path: "01_constitution/"
```

**输出契约**:
```yaml
required_outputs:
  - name: "architecture_document"
    type: file
    path: "docs/02_logical_workflow/{name}-architecture.md"
    format: "Markdown，包含架构图、接口定义"
```

**行为契约**:
```yaml
preconditions:
  - "系统需求已确认"
  - "工程宪章已存在"
postconditions:
  - "架构文档符合 P0 级约束"
  - "架构文档包含分层设计"
invariants:
  - "架构设计必须遵循 DDD 原则"
```

---

### sop-implementation-designer

**职责**: 实现设计，生成 P2/P3 级规范

**输入契约**:
```yaml
required_inputs:
  - name: "architecture_document"
    type: file
    path: "docs/02_logical_workflow/{name}-architecture.md"
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
```

**输出契约**:
```yaml
required_outputs:
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
    format: "Markdown，包含实现设计"
```

**行为契约**:
```yaml
preconditions:
  - "架构文档已通过审查"
postconditions:
  - "设计文档包含聚合根定义"
  - "设计文档包含仓储接口定义"
invariants:
  - "设计必须可测试"
```

---

## 与其他 Skill 的协作

```
规范类 Skill
    │
    ├── 输出规范文档
    │       │
    │       ▼
    │   实现类 Skill（读取规范，生成代码）
    │       │
    │       ▼
    │   验证类 Skill（验证规范是否被满足）
    │
    └── 与编排类 Skill 协作（管理规范版本）
```

---

## 相关文档

- [Skill 索引](index.md)
- [实现类 Skill](../implementation/)
- [验证类 Skill](../verification/)

---

**文档所有者**: Skill 团队  
**最后审核**: 2026-02-28
