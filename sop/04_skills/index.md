---
version: v3.0.0
updated: 2026-03-01
---

# Skill 索引

> **核心理念**: 规范驱动 Skill，Skill 是规范的执行工具

---

## 概述

本目录定义所有 Skill，每个 Skill 都是规范的"翻译器"，将规范转换为代码、测试、文档。

**符合 Trae Skill 标准**：每个 Skill 使用独立的 `SKILL.md` 文件定义，支持按需加载。

---

## 目录结构

```
04_skills/
├── index.md                          # 本文件
├── specification/                    # 规范类 Skill
│   ├── requirement-analyst/
│   │   └── SKILL.md                 # 需求分析
│   ├── architecture-design/
│   │   └── SKILL.md                 # 架构设计
│   └── implementation-designer/
│       └── SKILL.md                 # 实现设计
├── implementation/                   # 实现类 Skill
│   ├── code-explorer/
│   │   └── SKILL.md                 # 代码探索
│   ├── code-implementation/
│   │   └── SKILL.md                 # 代码实现
│   └── test-implementation/
│       └── SKILL.md                 # 测试实现
├── verification/                     # 验证类 Skill
│   ├── architecture-reviewer/
│   │   └── SKILL.md                 # 架构审查
│   └── code-review/
│       └── SKILL.md                 # 代码审查
└── orchestration/                    # 编排类 Skill
    ├── workflow-orchestrator/
    │   └── SKILL.md                 # 工作流编排
    ├── document-sync/
    │   └── SKILL.md                 # 文档同步
    └── progress-supervisor/
        └── SKILL.md                 # 进度监管
```

---

## Skill 分类

### 规范类 Skill

**职责**: 生成规范文档

| Skill | 描述 | 文件 |
|-------|------|------|
| `sop-requirement-analyst` | 分析需求并生成 P1/P2 级规范文档 | [SKILL.md](specification/requirement-analyst/SKILL.md) |
| `sop-architecture-design` | 进行系统架构设计，生成 P1 级架构文档 | [SKILL.md](specification/architecture-design/SKILL.md) |
| `sop-implementation-designer` | 进行实现设计，生成 P2/P3 级设计文档 | [SKILL.md](specification/implementation-designer/SKILL.md) |

### 实现类 Skill

**职责**: 将规范翻译为代码

| Skill | 描述 | 文件 |
|-------|------|------|
| `sop-code-explorer` | 探索代码库并生成分析报告 | [SKILL.md](implementation/code-explorer/SKILL.md) |
| `sop-code-implementation` | 根据规范和设计文档实现代码 | [SKILL.md](implementation/code-implementation/SKILL.md) |
| `sop-test-implementation` | 根据 BDD 场景编写测试代码 | [SKILL.md](implementation/test-implementation/SKILL.md) |

### 验证类 Skill

**职责**: 验证规范是否被满足

| Skill | 描述 | 文件 |
|-------|------|------|
| `sop-architecture-reviewer` | 审查架构设计，验证是否符合 P1 级规范 | [SKILL.md](verification/architecture-reviewer/SKILL.md) |
| `sop-code-review` | 审查代码实现，验证是否符合 P2/P3 级规范 | [SKILL.md](verification/code-review/SKILL.md) |

### 编排类 Skill

**职责**: 管理规范版本和流程编排

| Skill | 描述 | 文件 |
|-------|------|------|
| `sop-workflow-orchestrator` | 编排工作流程，管理规范版本和工作流状态 | [SKILL.md](orchestration/workflow-orchestrator/SKILL.md) |
| `sop-document-sync` | 同步文档与代码，确保规范文档与实现保持一致 | [SKILL.md](orchestration/document-sync/SKILL.md) |
| `sop-progress-supervisor` | 监管工作流进度，生成进度报告 | [SKILL.md](orchestration/progress-supervisor/SKILL.md) |

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
| P0 级（工程宪章） | [sop-architecture-design](specification/architecture-design/SKILL.md) | [sop-architecture-reviewer](verification/architecture-reviewer/SKILL.md) |
| P1 级（系统规范） | [sop-requirement-analyst](specification/requirement-analyst/SKILL.md) | [sop-architecture-reviewer](verification/architecture-reviewer/SKILL.md) |
| P2 级（模块规范） | [sop-implementation-designer](specification/implementation-designer/SKILL.md) | [sop-code-review](verification/code-review/SKILL.md) |
| P3 级（实现规范） | [sop-code-implementation](implementation/code-implementation/SKILL.md) | [sop-code-review](verification/code-review/SKILL.md) |

---

## SKILL.md 文件格式

每个 SKILL.md 文件遵循 Trae 标准格式：

```yaml
---
name: skill-id
description: 简要描述这个技能的功能和使用场景（一句话）
version: v3.0.0
skill_type: specification|implementation|verification|orchestration
---

# skill-id

## 描述
详细说明 Skill 的作用。

## 使用场景
触发这个 Skill 的条件。

## 指令
清晰的分步说明，告诉智能体具体怎么做。

## 契约
输入/输出契约定义。

## 示例
输入/输出示例。
```

---

## 相关文档

- [工程宪章](../01_constitution/) - P0 级规范
- [系统规范](../02_specifications/) - P1-P2 级规范
- [工作流程](../03_workflow/) - 5 阶段流程
- [约束规范](../05_constraints/) - P0-P3 约束

---

**文档所有者**: Skill 团队  
**最后审核**: 2026-03-01  
**下次审核**: 2026-06-30
