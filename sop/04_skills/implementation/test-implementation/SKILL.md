---
name: sop-test-implementation
description: |
  Use when:
    - 在实现前编写失败的测试（TDD 红阶段）
    - 需要验证实现是否符合 BDD 场景
    - 需要增加测试覆盖率
    - 需要编写回归测试用例
  Don't use when:
    - BDD 场景尚未定义 → 使用 sop-requirement-analyst 先定义场景
    - 需要实现代码而非测试 → 使用 sop-code-implementation
    - 需要审查测试质量 → 使用 sop-code-review
    - 需要探索现有代码 → 使用 sop-code-explorer
  Inputs:
    - spec_document: 规范文档，包含 BDD 场景
    - bdd_scenarios: BDD 场景定义
    - implementation_code: 实现代码（可选）
  Outputs:
    - tests/{module}/{name}-test.{ext}: 测试代码
  Success criteria:
    - 测试覆盖所有 BDD 场景
    - 测试代码符合 P3 级测试规范
    - 测试可重复执行
---

# sop-test-implementation

## 描述

测试实现 Skill 负责将 BDD 场景转换为测试代码。该 Skill 支持 TDD（测试驱动开发）流程，确保测试覆盖所有业务场景。

主要职责：
- 编写单元测试
- 编写集成测试
- 实现 BDD 场景
- 确保测试可重复执行

## 使用场景

触发此 Skill 的条件：

1. **TDD 红阶段**：在实现前编写失败的测试
2. **规范验证**：需要验证实现是否符合 BDD 场景
3. **测试覆盖**：需要增加测试覆盖率
4. **回归测试**：需要编写回归测试用例

## 指令

### 步骤 1: 分析 BDD 场景

1. 读取规范文档中的 BDD 场景
2. 理解 Given-When-Then 结构
3. 识别测试数据和预期结果
4. 确定测试类型（单元/集成/E2E）

### 步骤 2: 设计测试用例

1. 设计测试数据
2. 设计测试步骤
3. 设计断言条件
4. 设计测试隔离策略

### 步骤 3: 编写测试代码

1. 创建测试文件
2. 编写测试设置（Given）
3. 编写测试执行（When）
4. 编写测试验证（Then）

### 步骤 4: 编写测试辅助代码

1. 创建测试数据构建器
2. 创建 Mock 对象
3. 创建测试工具函数
4. 创建测试固件

### 步骤 5: 验证测试质量

1. 运行测试确保可执行
2. 检查测试覆盖率
3. 验证测试隔离性
4. 确保测试可重复执行

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
    description: "规范文档，包含 BDD 场景"
  
  - name: "bdd_scenarios"
    type: section
    path: "spec_document.bdd_scenarios"
    description: "BDD 场景定义"

optional_inputs:
  - name: "implementation_code"
    type: file
    path: "src/{module}/"
    description: "实现代码，用于参考"
```

### 输出契约

```yaml
required_outputs:
  - name: "test_code"
    type: file
    path: "tests/{module}/{name}-test.{ext}"
    format: "符合测试规范的测试代码"
    guarantees:
      - "测试覆盖所有 BDD 场景"
      - "测试代码符合 P3 级测试规范"
      - "测试可重复执行"
```

### 行为契约

```yaml
preconditions:
  - "BDD 场景已定义"
  - "测试框架已确定"

postconditions:
  - "测试覆盖所有 BDD 场景"
  - "测试代码符合 P3 级测试规范"
  - "测试可重复执行"
  - "测试文件保存在 tests/"

invariants:
  - "测试必须可重复执行"
  - "测试必须隔离"
  - "测试必须快速"
```

## 常见坑

### 坑 1: 测试相互依赖

- **现象**: 测试用例之间存在执行顺序依赖，单独运行某个测试会失败。
- **原因**: 测试共享了可变状态，前一个测试修改的状态影响了后一个测试。
- **解决**: 每个测试用例独立准备测试数据，测试完成后清理状态，确保测试隔离性。

### 坑 2: 测试断言不充分

- **现象**: 测试执行通过，但未真正验证业务逻辑的正确性。
- **原因**: 断言仅检查返回值不为空，未验证具体值的正确性。
- **解决**: 断言必须验证具体的业务规则，如"总价应等于单价乘以数量"，而非仅检查"总价不为空"。

### 坑 3: Mock 滥用

- **现象**: 过度使用 Mock 导致测试与实现细节耦合，重构时测试大量失败。
- **原因**: 对所有依赖都进行 Mock，包括稳定的值对象和工具类。
- **解决**: 仅对外部依赖（数据库、网络服务）使用 Mock，对于稳定的内部类使用真实实现。

## 示例

### 输入示例

```
BDD 场景：创建订单

Given 用户已登录
When 用户提交订单，包含商品 A（单价 100 元，数量 2）
Then 系统创建订单，总价为 200 元
And 订单状态为"待支付"
```

### 输出示例

```typescript
// tests/order/order-test.ts

describe('Order', () => {
  describe('创建订单', () => {
    it('应该创建订单并计算总价', () => {
      // Given
      const user = UserBuilder.create().build();
      const product = ProductBuilder.create()
        .withName('商品 A')
        .withPrice(Money.fromYuan(100))
        .build();
      
      // When
      const order = Order.create(user.id);
      order.addItem(product, 2);
      
      // Then
      expect(order.totalPrice).toEqual(Money.fromYuan(200));
      expect(order.status).toEqual(OrderStatus.PENDING_PAYMENT);
    });
  });
});
```

## 相关文档

- [Skill 索引](../index.md)
- [需求分析 Skill](../../specification/requirement-analyst/SKILL.md)
- [代码实现 Skill](../code-implementation/SKILL.md)
- [代码审查 Skill](../../verification/code-review/SKILL.md)