---
name: sop-code-review
description: |
  Use when:
    - 代码变更已提交，需要审查
    - 分支合并前需要代码审查
    - 定期检查代码质量
    - 发现问题时需要审查相关代码
  Don't use when:
    - 代码尚未实现 → 使用 sop-code-implementation
    - 需要审查架构而非代码 → 使用 sop-architecture-reviewer
    - 需要探索代码结构 → 使用 sop-code-explorer
    - 测试尚未通过 → 先使用 sop-test-implementation 完成测试
  Inputs:
    - design_document: 设计文档
    - code_changes: 代码变更记录（git diff）
    - test_report: 测试报告
    - previous_review: 上次审查报告（可选）
  Outputs:
    - contracts/code-review.json: 代码审查报告
  Success criteria:
    - 审查报告包含完整的质量检查结果
    - 核心模块覆盖率必须 >= 100%
    - 审查报告保存在 contracts/
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

## 常见坑

### 坑 1: 审查流于形式

- **现象**: 代码审查仅检查格式和命名，未深入分析业务逻辑正确性。
- **原因**: 审查者未充分理解设计文档和业务需求，只关注表面问题。
- **解决**: 审查前先阅读设计文档和规范文档，对照设计验证实现的业务逻辑是否正确。

### 坑 2: 忽视安全审查

- **现象**: 审查报告未发现 SQL 注入、XSS 等安全漏洞。
- **原因**: 审查重点放在功能和性能上，忽视了安全相关的代码检查。
- **解决**: 在审查清单中明确安全检查项，包括输入验证、输出编码、敏感信息处理等，必要时使用安全扫描工具。

### 坑 3: 覆盖率数据误读

- **现象**: 整体覆盖率达到要求，但核心模块覆盖率不足。
- **原因**: 仅关注整体覆盖率数字，未细分各模块的覆盖情况。
- **解决**: 审查报告中必须列出各模块的覆盖率详情，核心模块覆盖率必须达到 100%。

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