---
version: v3.0.2
---

# 约束索引

core_principle: 分层约束，P0不可违背

## 约束层级

```yaml
P0:
  name: 工程宪章约束
  strength: 不可违背，违反即熔断
  content: [安全红线, 质量红线, 架构红线]
  approval: 技术委员会
  doc: p0-constraints.md

P1:
  name: 系统约束
  strength: 跨模块约束
  content: 系统级质量要求
  approval: 技术负责人
  doc: p1-constraints.md

P2:
  name: 模块约束
  strength: 单模块约束
  content: 模块级质量要求
  approval: 模块负责人
  doc: p2-constraints.md

P3:
  name: 实现约束
  strength: 实现细节
  content: [编码规范, 测试规范]
  approval: 自动化工具
  doc: p3-constraints.md
```

## 约束验证

```yaml
verification:
  P0:
    timing: 每次提交
    method: 静态分析 + CI/CD
    handle: 构建失败，必须修复
  P1:
    timing: 每次合并
    method: 自动化测试
    handle: 构建失败，必须修复
  P2:
    timing: 每次构建
    method: 代码审查
    handle: 警告，建议修复
  P3:
    timing: 实时
    method: IDE插件
    handle: 提示，可选修复
```

## 违反处理流程

```yaml
P0_violation:
  steps: [检测违反, 构建失败, 记录详情, 通知责任人, 修复违规, 重新验证, 构建通过]

P1_P3_violation:
  steps: [检测违反, 记录详情, 生成报告, 通知责任人, 评估影响, 决定处理]
```

## 目录结构

```yaml
files:
  - index.md: 本文件
  - p0-constraints.md: P0级约束
  - p1-constraints.md: P1级约束
  - p2-constraints.md: P2级约束
  - p3-constraints.md: P3级约束
  - state-dictionary.md: 状态字典
  - command-dictionary.md: 命令字典
```

## 相关文档

- ../01_constitution/: P0级规范
- ../03_workflow/: 5阶段流程
- ../04_skills/: 规范驱动Skill
