---
name: sop-code-explorer
description: |
  Use when:
    - 需要验证现有代码是否符合规范要求
    - 在进行代码审查前，需要了解代码结构
    - 需要分析代码库以制定重构计划
    - 需要评估新需求对现有代码的影响
  Don't use when:
    - 需要修改代码 → 使用 sop-code-implementation
    - 需要审查代码质量 → 使用 sop-code-review
    - 需要创建规范文档 → 使用 sop-requirement-analyst
    - 需要同步文档 → 使用 sop-document-sync
  Inputs:
    - spec_document: 规范文档
    - codebase: 代码库目录（src/）
    - design_document: 设计文档（可选）
  Outputs:
    - contracts/code-analysis.json: 代码分析报告
  Success criteria:
    - 分析报告包含完整的实现状态
    - 分析报告包含约束验证结果
    - 分析报告保存在 contracts/
---

# sop-code-explorer

## 描述

代码探索 Skill 负责分析现有代码库，验证实现是否符合规范要求。该 Skill 是只读操作，不会修改任何代码。

主要职责：
- 探索代码库结构
- 分析现有实现
- 验证规范遵循情况
- 生成代码分析报告

## 使用场景

触发此 Skill 的条件：

1. **规范验证**：需要验证现有代码是否符合规范要求
2. **代码审查准备**：在进行代码审查前，需要了解代码结构
3. **重构分析**：需要分析代码库以制定重构计划
4. **影响分析**：需要评估新需求对现有代码的影响

## 指令

### 步骤 1: 读取规范文档

1. 读取规范文档（P1/P2/P3 级）
2. 理解规范要求和约束条件
3. 识别需要验证的功能点
4. 确定验证标准

### 步骤 2: 探索代码库

1. 扫描代码库目录结构
2. 识别关键模块和文件
3. 分析代码组织方式
4. 记录代码库状态

### 步骤 3: 分析现有实现

1. 查找相关代码实现
2. 分析代码逻辑和结构
3. 对比规范要求
4. 记录实现差异

### 步骤 4: 验证约束遵循

1. 检查 P0 级约束（无硬编码密钥、无强制解包等）
2. 检查 P1 级约束（API 响应时间、库使用等）
3. 检查 P2/P3 级约束（命名规范、注释等）
4. 记录约束违反情况

### 步骤 5: 生成分析报告

1. 汇总已实现的功能
2. 列出未实现的功能
3. 记录约束违反情况
4. 提供改进建议
5. 生成报告到 `contracts/code-analysis.json`

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "spec_document"
    type: file
    path: "specs/{name}-spec.md"
    description: "规范文档"
  
  - name: "codebase"
    type: directory
    path: "src/"
    description: "代码库目录"

optional_inputs:
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
    description: "设计文档，用于参考"
```

### 输出契约

```yaml
required_outputs:
  - name: "code_analysis_report"
    type: json
    path: "contracts/code-analysis.json"
    format:
      existing_implementations: ["已实现的功能"]
      missing_implementations: ["未实现的功能"]
      constraint_violations: ["约束违反"]
    guarantees:
      - "分析报告包含完整的实现状态"
      - "分析报告包含约束验证结果"
```

### 行为契约

```yaml
preconditions:
  - "规范文档存在"
  - "代码库可访问"

postconditions:
  - "分析报告包含完整的实现状态"
  - "分析报告包含约束验证结果"
  - "分析报告保存在 contracts/"

invariants:
  - "不修改任何代码"
  - "只读操作"
  - "报告必须准确客观"
```

## 常见坑

### 坑 1: 分析范围过大

- **现象**: 分析报告过于冗长，包含大量与规范无关的代码细节。
- **原因**: 未根据规范文档限定分析范围，对整个代码库进行了全面扫描。
- **解决**: 先识别规范中涉及的功能点和模块，仅分析与这些功能相关的代码文件。

### 坑 2: 误判实现状态

- **现象**: 报告显示功能已实现，但实际运行时功能不可用。
- **原因**: 仅检查代码是否存在，未验证代码是否被正确调用和集成。
- **解决**: 不仅检查代码实现，还要验证代码的调用链路和集成状态，确保功能端到端可用。

### 坑 3: 约束检查遗漏

- **现象**: 分析报告未发现已存在的约束违反问题。
- **原因**: 仅检查了部分约束项，未使用完整的约束检查清单。
- **解决**: 使用标准化的约束检查清单，逐项验证 P0/P1/P2/P3 级约束，确保无遗漏。

## 示例

### 输入示例

```
规范要求：
- 订单创建功能
- 订单取消功能（仅限未发货订单）
- 订单查询功能
```

### 输出示例

```json
{
  "analysis_date": "2026-03-01T10:00:00Z",
  "spec_document": "specs/order-spec.md",
  "existing_implementations": [
    {
      "feature": "订单创建",
      "location": "src/order/OrderAppService.ts",
      "status": "完整实现",
      "notes": "符合规范要求"
    },
    {
      "feature": "订单查询",
      "location": "src/order/OrderAppService.ts",
      "status": "完整实现",
      "notes": "支持按用户ID查询"
    }
  ],
  "missing_implementations": [
    {
      "feature": "订单取消",
      "reason": "未实现",
      "priority": "高"
    }
  ],
  "constraint_violations": [
    {
      "constraint": "P0-禁止硬编码密钥",
      "location": "src/order/OrderRepository.ts:15",
      "severity": "严重",
      "description": "发现硬编码的数据库连接字符串"
    }
  ]
}
```

## 相关文档

- [Skill 索引](../index.md)
- [代码实现 Skill](../code-implementation/SKILL.md)
- [代码审查 Skill](../../verification/code-review/SKILL.md)