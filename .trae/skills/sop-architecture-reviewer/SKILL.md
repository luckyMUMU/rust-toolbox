---
name: sop-architecture-reviewer
description: 审查架构设计，验证实现是否符合 P1 级规范和工程宪章
---

# sop-architecture-reviewer

## 描述

架构审查 Skill 负责验证系统架构是否符合 P0/P1 级规范。该 Skill 是质量保障的核心，确保架构设计符合工程宪章要求。

主要职责：
- 验证架构分层
- 检查模块边界
- 验证依赖方向
- 检查 P0 级约束

## 使用场景

触发此 Skill 的条件：

1. **架构设计完成**：架构文档已完成，需要审查
2. **代码变更审查**：重大代码变更可能影响架构
3. **定期架构审查**：定期检查系统架构健康度
4. **架构重构验证**：验证架构重构是否符合预期

## 指令

### 步骤 1: 准备审查材料

1. 读取架构设计文档
2. 读取工程宪章（P0 级约束）
3. 读取代码变更记录
4. 确定审查范围

### 步骤 2: 验证架构分层

1. 检查分层是否清晰
2. 验证各层职责是否明确
3. 检查层间依赖方向
4. 记录分层问题

### 步骤 3: 验证模块边界

1. 检查模块划分是否合理
2. 验证模块间接口定义
3. 检查模块间依赖关系
4. 记录边界问题

### 步骤 4: 验证 P0 级约束

1. 检查是否禁止硬编码密钥
2. 检查是否禁止强制解包
3. 检查是否禁止循环依赖
4. 检查核心模块覆盖率是否 100%

### 步骤 5: 验证 P1 级约束

1. 检查 API 响应时间
2. 检查库使用是否优先项目已有库
3. 检查跨模块约束
4. 记录 P1 级警告

### 步骤 6: 生成审查报告

1. 汇总审查结果
2. 分类问题（P0 违反/P1 警告/建议）
3. 提供改进建议
4. 生成报告到 `contracts/architecture-review.json`

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "architecture_document"
    type: file
    path: "docs/02_logical_workflow/{name}-architecture.md"
    description: "架构设计文档"
  
  - name: "code_changes"
    type: git_diff
    path: "git commit"
    description: "代码变更记录"
  
  - name: "p0_constraints"
    type: file
    path: "01_constitution/architecture-principles.md"
    description: "P0 级架构约束"

optional_inputs:
  - name: "previous_review"
    type: json
    path: "contracts/architecture-review.json"
    description: "上次审查报告"
```

### 输出契约

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
    guarantees:
      - "审查报告包含完整的约束验证结果"
      - "P0 级违反必须报告"
```

### 行为契约

```yaml
preconditions:
  - "架构文档存在"
  - "代码变更已提交"
  - "工程宪章存在"

postconditions:
  - "审查报告包含完整的约束验证结果"
  - "P0 级违反必须报告"
  - "审查报告保存在 contracts/"

invariants:
  - "P0 级违反必须报告"
  - "审查必须客观准确"
  - "审查必须覆盖所有约束"
```

## 示例

### 输入示例

```
架构设计：
- 分层：表现层、应用层、领域层、基础设施层
- 模块：订单模块、用户模块、商品模块
```

### 输出示例

```json
{
  "review_date": "2026-03-01T10:00:00Z",
  "review_status": "failed",
  "p0_violations": [
    {
      "constraint": "P0-禁止循环依赖",
      "location": "src/order/OrderService.ts",
      "severity": "严重",
      "description": "OrderService 依赖 UserService，UserService 依赖 OrderService"
    }
  ],
  "p1_warnings": [
    {
      "constraint": "P1-API响应时间",
      "location": "src/order/OrderRepository.ts",
      "severity": "警告",
      "description": "订单查询接口响应时间 600ms，超过 500ms 限制"
    }
  ],
  "recommendations": [
    "引入事件机制解耦 OrderService 和 UserService",
    "为订单查询添加缓存层"
  ]
}
```

## 相关文档

- [Skill 索引](../index.md)
- [架构设计 Skill](../../specification/architecture-design/SKILL.md)
- [代码审查 Skill](../code-review/SKILL.md)
- [工程宪章](../../../01_constitution/)