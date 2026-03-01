---
name: sop-code-review
description: 审查代码实现，验证是否符合 P2/P3 级规范和设计文档
version: v3.0.0
skill_type: verification
---

# sop-code-review

## 描述

代码审查 Skill 负责验证代码实现是否符合 P2/P3 级规范。该 Skill 是质量保障的关键环节，确保代码质量和测试覆盖率。

主要职责：
- 验证代码规范
- 检查测试覆盖率
- 验证设计遵循
- 检查安全问题

## 使用场景

触发此 Skill 的条件：

1. **代码提交**：代码变更已提交，需要审查
2. **合并请求**：分支合并前需要代码审查
3. **定期审查**：定期检查代码质量
4. **问题排查**：发现问题时需要审查相关代码

## 指令

### 步骤 1: 准备审查材料

1. 读取设计文档
2. 读取代码变更记录
3. 读取测试报告
4. 确定审查范围

### 步骤 2: 验证代码规范

1. 检查命名规范
2. 检查代码格式
3. 检查注释完整性
4. 检查代码复杂度

### 步骤 3: 验证设计遵循

1. 检查类结构是否符合设计
2. 检查方法签名是否符合设计
3. 检查数据结构是否符合设计
4. 记录设计偏差

### 步骤 4: 验证测试覆盖

1. 检查单元测试覆盖率
2. 检查核心模块覆盖率（必须 100%）
3. 检查测试质量
4. 记录覆盖不足的模块

### 步骤 5: 验证安全问题

1. 检查是否硬编码密钥
2. 检查是否使用不安全的依赖
3. 检查是否暴露敏感信息
4. 运行安全扫描工具

### 步骤 6: 生成审查报告

1. 汇总审查结果
2. 分类问题（严重/警告/建议）
3. 提供改进建议
4. 生成报告到 `contracts/code-review.json`

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
    description: "设计文档"
  
  - name: "code_changes"
    type: git_diff
    path: "git commit"
    description: "代码变更记录"
  
  - name: "test_report"
    type: json
    path: "contracts/stage-2-test-report.json"
    description: "测试报告"

optional_inputs:
  - name: "previous_review"
    type: json
    path: "contracts/code-review.json"
    description: "上次审查报告"
```

### 输出契约

```yaml
required_outputs:
  - name: "code_review_report"
    type: json
    path: "contracts/code-review.json"
    format:
      review_status: "passed|failed"
      coverage: 0-100
      issues: ["问题清单"]
      security_scan: "安全扫描结果"
    guarantees:
      - "审查报告包含完整的质量检查结果"
      - "核心模块覆盖率必须 >= 100%"
```

### 行为契约

```yaml
preconditions:
  - "设计文档存在"
  - "代码变更已提交"
  - "测试已通过"

postconditions:
  - "审查报告包含完整的质量检查结果"
  - "核心模块覆盖率必须 >= 100%"
  - "审查报告保存在 contracts/"

invariants:
  - "核心模块覆盖率必须 >= 100%"
  - "审查必须客观准确"
  - "安全问题必须报告"
```

## 示例

### 输入示例

```
代码变更：
- 新增 Order.cancel() 方法
- 新增 OrderItem 类
- 新增单元测试
```

### 输出示例

```json
{
  "review_date": "2026-03-01T10:00:00Z",
  "review_status": "passed",
  "coverage": {
    "overall": 95,
    "core_modules": 100,
    "details": {
      "Order": 100,
      "OrderItem": 100,
      "OrderRepository": 90
    }
  },
  "issues": [
    {
      "severity": "警告",
      "location": "src/order/OrderRepository.ts:45",
      "description": "缺少错误日志记录"
    }
  ],
  "security_scan": {
    "status": "passed",
    "vulnerabilities": []
  }
}
```

## 相关文档

- [Skill 索引](../index.md)
- [实现设计 Skill](../../specification/implementation-designer/SKILL.md)
- [代码实现 Skill](../../implementation/code-implementation/SKILL.md)
- [测试实现 Skill](../../implementation/test-implementation/SKILL.md)
