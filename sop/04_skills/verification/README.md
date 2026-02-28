---
version: v3.0.0
updated: 2026-02-28
skill_type: verification
---

# 验证类 Skill

> **职责**: 验证规范是否被满足

---

## 概述

验证类 Skill 负责验证实现是否符合规范，是质量保障的核心。

---

## Skill 列表

### sop-architecture-reviewer

**职责**: 架构审查，验证实现是否符合 P1 级规范

**输入契约**:
```yaml
required_inputs:
  - name: "architecture_document"
    type: file
    path: "docs/02_logical_workflow/{name}-architecture.md"
  - name: "code_changes"
    type: git_diff
    path: "git commit"
  - name: "p0_constraints"
    type: file
    path: "01_constitution/architecture-principles.md"
```

**输出契约**:
```yaml
required_outputs:
  - name: "architecture_review_report"
    type: json
    path: "contracts/architecture-review.json"
    format:
      review_status: "passed|failed"
      p0_violations: ["P0 级违反"]
      p1_warnings: ["P1 级警告"]
      recommendations: ["改进建议"]
```

**行为契约**:
```yaml
preconditions:
  - "架构文档存在"
  - "代码变更已提交"
postconditions:
  - "审查报告包含完整的约束验证结果"
invariants:
  - "P0 级违反必须报告"
```

---

### sop-code-review

**职责**: 代码审查，验证实现是否符合 P2/P3 级规范

**输入契约**:
```yaml
required_inputs:
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
  - name: "code_changes"
    type: git_diff
    path: "git commit"
  - name: "test_report"
    type: json
    path: "contracts/stage-2-test-report.json"
```

**输出契约**:
```yaml
required_outputs:
  - name: "code_review_report"
    type: json
    path: "contracts/code-review.json"
    format:
      review_status: "passed|failed"
      coverage: 0-100
      issues: ["问题清单"]
      security_scan: "安全扫描结果"
```

**行为契约**:
```yaml
preconditions:
  - "设计文档存在"
  - "代码变更已提交"
  - "测试已通过"
postconditions:
  - "审查报告包含完整的质量检查结果"
invariants:
  - "核心模块覆盖率必须 >= 100%"
```

---

## 验证流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    验证流程                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  步骤 1: P0 级约束验证                                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ • 禁止硬编码密钥                                                      │   │
│  │ • 核心模块覆盖率 100%                                                 │   │
│  │ • 禁止强制解包                                                        │   │
│  │ • 禁止循环依赖                                                        │   │
│  │ 结果：零违反                                                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                   │                                        │
│                                   ▼                                        │
│  步骤 2: P1 级约束验证                                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ • API 响应时间 < 500ms                                                │   │
│  │ • 优先使用项目已有库                                                  │   │
│  │ 结果：警告可接受                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                   │                                        │
│                                   ▼                                        │
│  步骤 3: P2/P3 级约束验证                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ • 遵循命名约定                                                        │   │
│  │ • 公共 API 必须注释                                                   │   │
│  │ 结果：自动化验证                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 验证工具

| 约束类型 | 工具 | 集成方式 |
|----------|------|----------|
| 密钥检测 | git-secrets、truffleHog | pre-commit hook |
| 漏洞扫描 | npm audit、Snyk | CI/CD |
| 覆盖率检查 | istanbul、coverage.py | npm test |
| 架构约束 | ArchUnit、dependency-cruiser | CI/CD |
| 代码风格 | ESLint、Pylint | pre-commit hook |

---

## 相关文档

- [Skill 索引](../index.md)
- [规范类 Skill](../specification/)
- [实现类 Skill](../implementation/)
- [约束规范](../../05_constraints/)

---

**文档所有者**: Skill 团队  
**最后审核**: 2026-02-28
