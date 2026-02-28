---
version: v3.0.0
updated: 2026-02-28
---

# Skill 索引

> **核心理念**: 规范驱动 Skill，Skill 是规范的执行工具

---

## 概述

本目录定义所有 Skill，每个 Skill 都是规范的"翻译器"，将规范转换为代码、测试、文档。

---

## Skill 分类

### 规范类 Skill

**职责**: 生成规范文档

| Skill | 职责 | 输入 | 输出 |
|-------|------|------|------|
| `sop-requirement-analyst` | 需求分析 | 需求描述 | P1/P2 级规范 |
| `sop-architecture-design` | 架构设计 | 系统需求 | P1 级规范 |
| `sop-implementation-designer` | 实现设计 | 架构设计 | P2/P3 级规范 |

**目录**: [specification/](specification/)

### 实现类 Skill

**职责**: 将规范翻译为代码

| Skill | 职责 | 输入 | 输出 |
|-------|------|------|------|
| `sop-code-explorer` | 探索代码 | 规范文档 | 代码分析报告 |
| `sop-code-implementation` | 代码实现 | P2/P3 级规范 | 代码变更 |
| `sop-test-implementation` | 测试实现 | BDD 场景 | 测试代码 |

**目录**: [implementation/](implementation/)

### 验证类 Skill

**职责**: 验证规范是否被满足

| Skill | 职责 | 输入 | 输出 |
|-------|------|------|------|
| `sop-architecture-reviewer` | 架构审查 | 代码变更 | P1 级验证报告 |
| `sop-code-review` | 代码审查 | 代码变更 | P2/P3 级验证报告 |

**目录**: [verification/](verification/)

### 编排类 Skill

**职责**: 管理规范版本和流程编排

| Skill | 职责 | 输入 | 输出 |
|-------|------|------|------|
| `sop-workflow-orchestrator` | 流程编排 | 规范文档 | 工作流状态 |
| `sop-document-sync` | 文档同步 | 代码变更 | 文档更新 |
| `sop-progress-supervisor` | 进度监管 | 工作流状态 | 进度报告 |

**目录**: [orchestration/](orchestration/)

---

## 规范驱动 Skill 的工作机制

### 核心原则

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    规范驱动 Skill 工作机制                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                   │
│  │   规范文档   │────▶│    Skill    │────▶│    产物     │                   │
│  │  (Spec)     │     │  (执行工具)  │     │(Deliverable)│                   │
│  └─────────────┘     └─────────────┘     └─────────────┘                   │
│         │                   │                   │                          │
│         │                   │                   │                          │
│         ▼                   ▼                   ▼                          │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                   │
│  │ 定义"做什么" │     │ 读取规范    │     │ 代码        │                   │
│  │ 定义约束条件 │     │ 执行任务    │     │ 测试        │                   │
│  │ 定义验收标准 │     │ 生成产物    │     │ 文档        │                   │
│  │             │     │ 验证约束    │     │             │                   │
│  └─────────────┘     └─────────────┘     └─────────────┘                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Skill 执行流程

```
1. 读取规范
   Skill 从规范文档中读取：
   - 功能定义（做什么）
   - 约束条件（不能做什么）
   - 验收标准（如何验证）

2. 执行任务
   Skill 根据规范执行任务：
   - 规范类 Skill：生成规范文档
   - 实现类 Skill：生成代码/测试
   - 验证类 Skill：验证规范是否被满足
   - 编排类 Skill：管理流程和版本

3. 生成产物
   Skill 生成规范要求的产物：
   - 代码变更
   - 测试代码
   - 文档更新
   - 审查报告

4. 验证约束
   Skill 验证产物是否满足规范约束：
   - P0 级约束：零违反
   - P1 级约束：警告可接受
   - P2/P3 级约束：自动化验证
```

---

## Skill 与规范层级映射

| 规范层级 | 相关 Skill | 验证 Skill |
|----------|-----------|------------|
| P0 级（工程宪章） | `sop-architecture-design` | `sop-architecture-reviewer` |
| P1 级（系统规范） | `sop-requirement-analyst` | `sop-architecture-reviewer` |
| P2 级（模块规范） | `sop-implementation-designer` | `sop-code-review` |
| P3 级（实现规范） | `sop-code-implementation` | `sop-code-review` |

---

## Skill 合约规范

### 合约模板

```yaml
skill_id: "skill-name"
skill_type: "specification|implementation|verification|orchestration"
version: "1.0.0"

input_contract:
  required_inputs:
    - name: "input_name"
      type: "file|data"
      validation: "验证规则"
  constraints:
    - "输入约束"

output_contract:
  required_outputs:
    - name: "output_name"
      type: "file|data"
      format: "格式定义"
  guarantees:
    - "输出保证"

behavior_contract:
  preconditions:
    - "前置条件"
  postconditions:
    - "后置条件"
  invariants:
    - "不变式"
```

---

## 目录结构

```
04_skills/
├── index.md               # 本文件
├── specification/         # 规范类 Skill
│   └── README.md
├── implementation/        # 实现类 Skill
│   └── README.md
├── verification/          # 验证类 Skill
│   └── README.md
└── orchestration/         # 编排类 Skill
    └── README.md
```

---

## 相关文档

- [工程宪章](../01_constitution/) - P0 级规范
- [系统规范](../02_specifications/) - P1-P2 级规范
- [工作流程](../03_workflow/) - 5 阶段流程
- [约束规范](../05_constraints/) - P0-P3 约束

---

**文档所有者**: Skill 团队  
**最后审核**: 2026-02-28  
**下次审核**: 2026-06-30
