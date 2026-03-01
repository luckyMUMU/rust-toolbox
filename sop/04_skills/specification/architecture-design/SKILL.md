---
name: sop-architecture-design
description: 进行系统架构设计，生成 P1 级架构文档，定义系统分层、模块边界和接口规范
version: v3.0.0
skill_type: specification
---

# sop-architecture-design

## 描述

架构设计 Skill 负责将系统需求转换为架构设计文档。该 Skill 确保系统架构符合工程宪章（P0 级约束），并遵循 DDD（领域驱动设计）原则。

主要职责：
- 设计系统分层架构
- 定义模块边界和职责
- 设计领域模型和聚合根
- 定义仓储接口和应用服务
- 确保架构符合 P0 级约束

## 使用场景

触发此 Skill 的条件：

1. **新系统设计**：需要为新系统设计整体架构
2. **架构重构**：现有系统需要架构层面的重构
3. **模块拆分**：需要定义新的模块边界和职责
4. **接口定义**：需要定义跨模块的接口规范

## 指令

### 步骤 1: 分析系统需求

1. 读取系统需求规范文档
2. 识别核心业务领域
3. 确定系统边界和外部依赖
4. 分析非功能性需求（性能、安全、可扩展性）

### 步骤 2: 设计分层架构

1. 确定架构分层（表现层、应用层、领域层、基础设施层）
2. 定义各层职责和依赖关系
3. 设计模块划分方案
4. 确保依赖方向正确（外层依赖内层）

### 步骤 3: 设计领域模型

1. 识别领域实体和值对象
2. 定义聚合根和聚合边界
3. 设计领域服务和领域事件
4. 定义仓储接口

### 步骤 4: 编写架构文档

1. 绘制架构图（分层图、模块图、领域模型图）
2. 编写架构决策说明
3. 定义接口规范
4. 记录约束条件和设计原则

### 步骤 5: 验证架构质量

1. 检查是否符合 P0 级约束（无循环依赖、分层清晰）
2. 验证是否符合 DDD 原则
3. 确认架构可测试性
4. 生成架构文档到 `docs/02_logical_workflow/{name}-architecture.md`

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "system_requirements"
    type: file
    path: "specs/{name}-spec.md"
    description: "系统需求规范文档"
  
  - name: "constitution_docs"
    type: files
    path: "01_constitution/"
    description: "工程宪章文档，包含 P0 级约束"

optional_inputs:
  - name: "existing_architecture"
    type: file
    path: "docs/02_logical_workflow/"
    description: "现有架构文档，用于参考和对齐"
```

### 输出契约

```yaml
required_outputs:
  - name: "architecture_document"
    type: file
    path: "docs/02_logical_workflow/{name}-architecture.md"
    format: "Markdown，包含架构图、接口定义"
    guarantees:
      - "架构文档符合 P0 级约束"
      - "架构文档包含分层设计"
      - "架构文档包含领域模型定义"
```

### 行为契约

```yaml
preconditions:
  - "系统需求已确认"
  - "工程宪章已存在"
  - "需求规范文档已通过审查"

postconditions:
  - "架构文档符合 P0 级约束"
  - "架构文档包含分层设计"
  - "架构文档包含领域模型定义"
  - "架构文档保存在 docs/02_logical_workflow/"

invariants:
  - "架构设计必须遵循 DDD 原则"
  - "架构必须分层清晰"
  - "依赖方向必须正确（外层依赖内层）"
```

## 示例

### 输入示例

```
系统需求：订单管理系统
- 支持创建、查询、取消订单
- 订单包含商品、数量、总价
- 用户只能取消未发货订单
```

### 输出示例

```markdown
# 订单系统架构设计

## 分层架构

┌─────────────────────────────────────┐
│         表现层 (Presentation)        │
│   REST API / GraphQL / gRPC         │
└─────────────────────────────────────┘
                  ▼
┌─────────────────────────────────────┐
│         应用层 (Application)         │
│   OrderAppService / QueryService    │
└─────────────────────────────────────┘
                  ▼
┌─────────────────────────────────────┐
│          领域层 (Domain)             │
│   Order / OrderItem / OrderService  │
└─────────────────────────────────────┘
                  ▼
┌─────────────────────────────────────┐
│       基础设施层 (Infrastructure)     │
│   OrderRepository / MessageQueue    │
└─────────────────────────────────────┘

## 领域模型

### 聚合根: Order

- 属性: orderId, userId, status, totalPrice, items
- 行为: create(), addItem(), cancel(), calculateTotal()

### 实体: OrderItem

- 属性: productId, quantity, unitPrice, subtotal

## 仓储接口

interface OrderRepository {
  save(order: Order): void
  findById(orderId: OrderId): Order
  findByUserId(userId: UserId): List<Order>
}
```

## 相关文档

- [Skill 索引](../index.md)
- [需求分析 Skill](../requirement-analyst/SKILL.md)
- [实现设计 Skill](../implementation-designer/SKILL.md)
- [架构审查 Skill](../../verification/architecture-reviewer/SKILL.md)
