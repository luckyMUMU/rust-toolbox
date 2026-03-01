---
name: sop-code-implementation
description: 根据规范和设计文档实现代码，生成符合项目规范的代码变更
version: v3.0.0
skill_type: implementation
---

# sop-code-implementation

## 描述

代码实现 Skill 负责将设计文档转换为实际的代码实现。该 Skill 是实现阶段的核心，确保代码符合规范要求和项目编码标准。

主要职责：
- 实现类和方法
- 编写业务逻辑
- 确保代码质量
- 遵循编码规范

## 使用场景

触发此 Skill 的条件：

1. **设计完成**：设计文档已通过审查，可以开始实现
2. **功能开发**：需要实现新的功能模块
3. **Bug 修复**：需要修复代码缺陷
4. **代码重构**：需要改进现有代码结构

## 指令

### 步骤 1: 准备实现环境

1. 读取设计文档和规范文档
2. 理解实现要求和约束条件
3. 确认技术栈和框架
4. 检查依赖项是否已安装

### 步骤 2: 实现类结构

1. 创建类文件
2. 定义类属性和字段
3. 实现构造函数
4. 添加必要的导入

### 步骤 3: 实现业务逻辑

1. 实现方法签名
2. 编写业务逻辑代码
3. 添加错误处理
4. 实现数据验证

### 步骤 4: 编写单元测试

1. 创建测试文件
2. 编写测试用例
3. 覆盖正常和异常场景
4. 确保测试通过

### 步骤 5: 代码质量检查

1. 运行 lint 检查
2. 运行类型检查
3. 运行单元测试
4. 修复所有问题

### 步骤 6: 提交代码变更

1. 检查代码变更范围
2. 编写提交信息
3. 确保符合 P2/P3 级规范
4. 提交代码

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
    description: "设计文档"
  
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
    description: "规范文档"

optional_inputs:
  - name: "existing_code"
    type: file
    path: "src/{module}/"
    description: "现有代码，用于参考或修改"
```

### 输出契约

```yaml
required_outputs:
  - name: "code_changes"
    type: git_diff
    path: "git commit"
    format: "符合项目代码规范的代码变更"
    guarantees:
      - "代码符合 P2/P3 级规范"
      - "代码通过 lint 检查"
      - "代码通过类型检查"
      - "单元测试通过"
```

### 行为契约

```yaml
preconditions:
  - "设计文档已通过审查"
  - "规范文档已确认"
  - "技术栈已确定"

postconditions:
  - "代码符合 P2/P3 级规范"
  - "代码通过 lint 检查"
  - "代码通过类型检查"
  - "单元测试通过"

invariants:
  - "禁止强制解包（unwrap/expect）"
  - "禁止硬编码密钥"
  - "禁止循环依赖"
  - "公共 API 必须注释"
```

## 示例

### 输入示例

```
设计文档：
class Order {
  addItem(product: Product, quantity: number): void;
  cancel(): Result<void, OrderError>;
}
```

### 输出示例

```typescript
// src/order/Order.ts

export class Order {
  private items: OrderItem[] = [];

  addItem(product: Product, quantity: number): void {
    if (quantity <= 0) {
      throw new Error('数量必须大于 0');
    }
    
    const existingItem = this.items.find(
      item => item.productId === product.id
    );
    
    if (existingItem) {
      existingItem.quantity += quantity;
    } else {
      this.items.push(new OrderItem(product, quantity));
    }
    
    this.calculateTotal();
  }

  cancel(): Result<void, OrderError> {
    if (this.status === OrderStatus.SHIPPED) {
      return Err({ type: 'ORDER_ALREADY_SHIPPED' });
    }
    
    this.status = OrderStatus.CANCELLED;
    return Ok(undefined);
  }
}
```

## 相关文档

- [Skill 索引](../index.md)
- [实现设计 Skill](../../specification/implementation-designer/SKILL.md)
- [测试实现 Skill](../test-implementation/SKILL.md)
- [代码审查 Skill](../../verification/code-review/SKILL.md)
