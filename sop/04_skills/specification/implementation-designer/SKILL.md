---
name: sop-implementation-designer
description: 进行实现设计，生成 P2/P3 级设计文档，定义类结构、方法签名和实现细节
version: v3.0.0
skill_type: specification
---

# sop-implementation-designer

## 描述

实现设计 Skill 负责将架构设计转换为详细的实现设计文档。该 Skill 在架构设计和代码实现之间建立桥梁，确保实现符合架构约束。

主要职责：
- 设计类结构和继承关系
- 定义方法签名和参数
- 设计数据结构和算法
- 定义错误处理策略
- 确保设计可测试

## 使用场景

触发此 Skill 的条件：

1. **架构设计完成**：架构文档已通过审查，需要详细设计
2. **模块实现准备**：准备开始编码前，需要明确实现细节
3. **接口细化**：需要定义具体的类接口和方法签名
4. **技术方案设计**：需要设计具体的技术实现方案

## 指令

### 步骤 1: 分析架构设计

1. 读取架构设计文档
2. 理解模块职责和边界
3. 识别关键类和接口
4. 确认技术栈和框架

### 步骤 2: 设计类结构

1. 定义类名和职责
2. 设计属性和字段
3. 定义方法签名
4. 设计继承和组合关系

### 步骤 3: 设计数据结构

1. 定义数据模型
2. 设计数据验证规则
3. 定义数据转换逻辑
4. 设计数据存储方案

### 步骤 4: 设计错误处理

1. 定义错误类型
2. 设计错误传播策略
3. 定义错误恢复机制
4. 设计日志记录方案

### 步骤 5: 编写设计文档

1. 绘制类图和时序图
2. 编写类和方法的详细说明
3. 定义测试策略
4. 记录设计决策和约束

### 步骤 6: 验证设计质量

1. 检查是否符合架构约束
2. 验证设计的可测试性
3. 确认符合 P2/P3 级规范
4. 生成设计文档到 `src/{module}/design.md`

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "architecture_document"
    type: file
    path: "docs/02_logical_workflow/{name}-architecture.md"
    description: "架构设计文档"
  
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
    description: "需求规范文档"

optional_inputs:
  - name: "existing_design"
    type: file
    path: "src/{module}/design.md"
    description: "现有设计文档，用于参考和对齐"
```

### 输出契约

```yaml
required_outputs:
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
    format: "Markdown，包含实现设计"
    guarantees:
      - "设计文档包含聚合根定义"
      - "设计文档包含仓储接口定义"
      - "设计文档包含类图和时序图"
```

### 行为契约

```yaml
preconditions:
  - "架构文档已通过审查"
  - "需求规范文档已确认"
  - "技术栈已确定"

postconditions:
  - "设计文档包含聚合根定义"
  - "设计文档包含仓储接口定义"
  - "设计文档包含类图和时序图"
  - "设计文档保存在 src/{module}/"

invariants:
  - "设计必须可测试"
  - "设计必须符合架构约束"
  - "设计必须遵循项目编码规范"
```

## 示例

### 输入示例

```
架构设计：
- 聚合根: Order
- 仓储接口: OrderRepository
- 应用服务: OrderAppService
```

### 输出示例

```markdown
# 订单模块实现设计

## 类结构

### Order (聚合根)

```typescript
class Order {
  private orderId: OrderId;
  private userId: UserId;
  private status: OrderStatus;
  private items: OrderItem[];
  private totalPrice: Money;
  private createdAt: Date;

  constructor(orderId: OrderId, userId: UserId): Order;
  
  addItem(product: Product, quantity: number): void;
  cancel(): Result<void, OrderError>;
  calculateTotal(): Money;
  
  private validateCanCancel(): boolean;
}
```

### OrderRepository (仓储接口)

```typescript
interface OrderRepository {
  save(order: Order): Promise<void>;
  findById(orderId: OrderId): Promise<Option<Order>>;
  findByUserId(userId: UserId): Promise<Order[]>;
}
```

## 错误处理

### 错误类型

```typescript
type OrderError = 
  | { type: 'ORDER_ALREADY_SHIPPED' }
  | { type: 'ORDER_ITEM_NOT_FOUND', productId: ProductId }
  | { type: 'INVALID_QUANTITY', quantity: number };
```

## 测试策略

- 单元测试：覆盖 Order 聚合根的所有业务逻辑
- 集成测试：覆盖 OrderRepository 的持久化逻辑
- E2E 测试：覆盖完整的订单创建和取消流程
```

## 相关文档

- [Skill 索引](../index.md)
- [架构设计 Skill](../architecture-design/SKILL.md)
- [代码实现 Skill](../../implementation/code-implementation/SKILL.md)
- [代码审查 Skill](../../verification/code-review/SKILL.md)
