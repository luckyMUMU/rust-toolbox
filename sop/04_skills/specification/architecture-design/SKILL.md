---
name: sop-architecture-design
description: |
  Use when:
    - 需要为新系统设计整体架构
    - 现有系统需要架构层面的重构
    - 需要定义新的模块边界和职责
    - 需要定义跨模块的接口规范
  Don't use when:
    - 需求规范尚未确认 → 使用 sop-requirement-analyst 先完成需求分析
    - 架构设计已完成，需要审查 → 使用 sop-architecture-reviewer
    - 架构已确定，需要设计实现细节 → 使用 sop-implementation-designer
    - 需要修改代码而非设计架构 → 使用 sop-code-implementation
  Inputs:
    - system_requirements: 系统需求规范文档（specs/{name}-spec.md）
    - constitution_docs: 工程宪章文档（01_constitution/）
    - existing_architecture: 现有架构文档（可选）
  Outputs:
    - docs/02_logical_workflow/{name}-architecture.md: 架构设计文档
  Success criteria:
    - 架构文档符合 P0 级约束
    - 架构文档包含分层设计
    - 架构文档包含领域模型定义
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

## 常见坑

### 坑 1: 分层依赖方向错误

- **现象**: 领域层直接依赖基础设施层，导致业务逻辑与具体实现耦合。
- **原因**: 未遵循依赖倒置原则，领域层直接引用具体的技术实现类。
- **解决**: 使用仓储接口（Repository Interface）在领域层定义抽象，由基础设施层实现具体类，确保依赖方向由外向内。

### 坑 2: 聚合边界划分过大

- **现象**: 单个聚合包含过多实体，导致性能问题和并发冲突频繁。
- **原因**: 将业务关联的所有实体都放入同一聚合，未考虑聚合的一致性边界。
- **解决**: 遵循"小聚合"原则，仅将必须保持强一致性的实体划入同一聚合，其他实体通过领域事件实现最终一致性。

### 坑 3: 忽视非功能性需求

- **现象**: 架构设计仅关注功能实现，上线后出现性能、安全等问题。
- **原因**: 架构设计阶段未充分分析非功能性需求（性能、安全、可扩展性）。
- **解决**: 在步骤 1 中明确列出非功能性需求，并在架构文档中单独章节说明应对方案。

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