---
version: v3.0.2
---

# 工作流规范

core_principle: 规范驱动，契约协作，分层约束

## 5阶段工作流

```yaml
stages:
  - id: 0
    name: 规范重量选择
    actions: [评估需求复杂度, 推荐规范重量]
    output: 规范重量决策JSON
  - id: 1
    name: 理解与设计
    actions: [多轮次多维度提问, 需求分析BDD, 架构设计DDD, 实现设计]
    output: [clarification.md, spec.md, architecture-design.md, design.md]
  - id: 2
    name: 实现与验证
    actions: [TDD循环, 约束验证P0-P3, 代码审查]
    output: [代码变更, 审查报告, 测试报告]
  - id: 3
    name: 交付与同步
    actions: [文档同步, 索引更新]
    output: [完成通知, 变更摘要]
  - id: 4
    name: 归档与演化
    actions: [创建归档记录, 升级评估, 更新CHANGELOG]
    output: [归档记录, CHANGELOG]
```

## 质量门控

```yaml
quality_gates:
  stage_0:
    checks: [需求描述清晰, 复杂度评估合理]
    pass: 全部通过
    fail: 返回需求澄清
  stage_1:
    checks: [需求澄清完成, 长期短期区分, 用户意图确认, 设计文档完整, 符合P0约束, 通过审查]
    pass: 全部通过
    fail: 返回需求澄清或设计修正
  stage_2:
    checks: [代码规范, 测试通过, 约束验证通过]
    pass: 全部通过
    fail: 返回实现修正
  stage_3:
    checks: [文档同步完成, 索引更新正确]
    pass: 全部通过
    fail: 返回同步修正
  stage_4:
    checks: [归档记录完整, CHANGELOG更新]
    pass: 全部通过
    fail: 返回归档修正

gate_failure:
  - 每次失败需用户决策
  - 可选: 修复后重试/回滚/终止
```

## 契约式协作

```yaml
principles:
  - 各环节独立上下文
  - 仅通过契约传递(JSON/YAML文件)
  - 禁止共享内存/状态/缓存
  - 禁止隐式依赖
  - 上下文版本化

contract_structure:
  precondition: 输入必须满足的条件
  logic: 独立的上下文空间
  postcondition: 输出必须满足的条件
  invariant: 环节内部必须保持的性质
```

## 路径选择

```yaml
heavy:
  scenarios: [从0到1核心系统, 跨团队大型功能, 安全金融高风险模块, 长期演进基础设施]
  flow: stage_0 -> stage_1(完整设计) -> stage_2 -> stage_3 -> stage_4
  outputs: [工程宪章文档, 系统规范文档, 约束矩阵]

light:
  scenarios: [小功能增量需求, UI接口配置改动, 试验性功能, Bug修复, 性能优化]
  flow: stage_0 -> stage_1(简化设计) -> stage_2 -> stage_3 -> stage_4
  outputs: [proposal.md, confirmation.md, archive.md]

fast:
  scenarios: [单文件, <30行, 无逻辑变更]
  flow: stage_2(跳过设计) -> stage_3
```

## 阶段文档索引

```yaml
stage_docs:
  - stage: 0
    doc: stage-0-weight.md
    desc: 规范重量选择
  - stage: 1
    doc: stage-1-design.md
    desc: 理解与设计
  - stage: 2
    doc: stage-2-implement.md
    desc: 实现与验证
  - stage: 3
    doc: stage-3-deliver.md
    desc: 交付与同步
  - stage: 4
    doc: stage-4-archive.md
    desc: 归档与演化
```

## 契约模板索引

```yaml
contract_templates:
  - stage: 0
    file: contracts/stage-0-contract.yaml
    desc: 规范重量决策契约
  - stage: 1
    file: contracts/stage-1-contract.yaml
    desc: 设计输出契约
  - stage: 2
    file: contracts/stage-2-contract.yaml
    desc: 实现输出契约
  - stage: 3
    file: contracts/stage-3-contract.yaml
    desc: 交付输出契约
  - stage: 4
    file: contracts/stage-4-contract.yaml
    desc: 归档输出契约
```

## 相关文档

- ../01_constitution/: P0级规范
- ../02_specifications/: P1-P2级规范
- ../05_constraints/: P0-P3约束
- ../04_skills/: 规范驱动Skill
