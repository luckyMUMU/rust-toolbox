---
name: sop-requirement-analyst
description: 分析需求并生成 P1/P2 级规范文档，将业务需求转换为可执行的规范定义
---

# sop-requirement-analyst

## 描述

需求分析 Skill 负责将用户的业务需求转换为结构化的规范文档。该 Skill 是 Spec-First 架构的核心，确保需求在实现前被完整定义和验证。

主要职责：
- 解析业务需求描述
- 识别功能边界和约束条件
- 生成包含 BDD 场景的规范文档
- 确保规范符合 P0 级约束

## 使用场景

触发此 Skill 的条件：

1. **新需求提出**：用户提出新的功能需求或业务规则
2. **需求澄清**：需要将模糊的业务描述转换为清晰的规范定义
3. **规范创建**：需要创建新的 P1/P2 级规范文档
4. **验收标准定义**：需要定义功能验收的 BDD 场景

## 指令

### 步骤 1: 收集需求信息

1. 读取用户提供的原始需求描述
2. 识别关键业务实体和关系
3. 确认功能边界和约束条件
4. 记录不明确的点，准备澄清问题

### 步骤 2: 分析需求结构

1. 将需求分解为功能点
2. 识别跨模块依赖
3. 确定规范层级（P1 系统级或 P2 模块级）
4. 评估对现有系统的影响

### 步骤 3: 编写规范文档

1. 使用规范模板创建文档结构
2. 编写功能描述和业务规则
3. 定义 BDD 场景（Given-When-Then）
4. 添加约束条件和验收标准

### 步骤 4: 验证规范质量

1. 检查规范是否完整覆盖需求
2. 验证 BDD 场景的可测试性
3. 确认符合 P0 级约束（无硬编码密钥、无强制解包等）
4. 生成规范文档到 `specs/{name}-spec.md`

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "requirement_description"
    type: text
    validation: "长度 >= 10 字符"
    description: "用户提供的原始需求描述"
  
optional_inputs:
  - name: "existing_specs"
    type: files
    path: "specs/"
    description: "现有规范文档，用于参考和对齐"
```

### 输出契约

```yaml
required_outputs:
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
    format: "Markdown，包含 Gherkin 场景"
    guarantees:
      - "规范文档包含完整的 BDD 场景"
      - "规范文档通过 P0 级约束验证"
      - "规范使用领域语言编写"
```

### 行为契约

```yaml
preconditions:
  - "需求描述清晰无歧义"
  - "需求描述长度 >= 10 字符"

postconditions:
  - "规范文档包含完整的 BDD 场景"
  - "规范文档通过 P0 级约束验证"
  - "规范文档保存在 specs/ 目录"

invariants:
  - "规范必须使用领域语言"
  - "规范必须可测试"
  - "规范必须符合 P0 级约束"
```

## 示例

### 输入示例

```
需求描述：
用户需要一个订单管理系统，支持创建订单、查询订单状态、取消订单。
订单需要记录商品信息、数量、总价。
用户只能取消未发货的订单。
```

### 输出示例

```markdown
# 订单管理规范

## 功能描述

订单管理系统支持创建、查询和取消订单。

## 业务规则

1. 订单必须包含至少一个商品
2. 订单总价 = 商品单价 × 数量
3. 只有未发货的订单可以取消

## BDD 场景

### 场景 1: 创建订单

**Given** 用户已登录
**When** 用户提交订单，包含商品 A（单价 100 元，数量 2）
**Then** 系统创建订单，总价为 200 元
**And** 订单状态为"待支付"

### 场景 2: 取消订单

**Given** 订单状态为"待发货"
**When** 用户取消订单
**Then** 订单状态变更为"已取消"

### 场景 3: 取消已发货订单失败

**Given** 订单状态为"已发货"
**When** 用户尝试取消订单
**Then** 系统拒绝操作
**And** 提示"已发货订单无法取消"
```

## 相关文档

- [Skill 索引](../index.md)
- [架构设计 Skill](../architecture-design/SKILL.md)
- [实现设计 Skill](../implementation-designer/SKILL.md)
- [工程宪章](../../../01_constitution/)