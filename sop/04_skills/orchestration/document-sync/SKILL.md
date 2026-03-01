---
name: sop-document-sync
description: 同步文档与代码，确保规范文档与实现保持一致
version: v3.0.0
skill_type: orchestration
---

# sop-document-sync

## 描述

文档同步 Skill 负责确保规范文档与代码实现保持同步。该 Skill 在代码变更后自动更新相关文档。

主要职责：
- 检测代码变更
- 更新相关文档
- 同步 API 文档
- 更新设计文档

## 使用场景

触发此 Skill 的条件：

1. **代码提交**：代码变更已提交，需要同步文档
2. **API 变更**：API 接口发生变化，需要更新 API 文档
3. **设计变更**：实现设计发生变化，需要更新设计文档
4. **定期同步**：定期检查文档与代码的一致性

## 指令

### 步骤 1: 检测代码变更

1. 读取代码变更记录（git diff）
2. 识别变更的文件和模块
3. 分析变更类型（新增/修改/删除）
4. 确定影响范围

### 步骤 2: 识别相关文档

1. 查找与变更代码相关的文档
2. 识别需要更新的文档类型
3. 确定文档位置
4. 记录文档清单

### 步骤 3: 更新文档内容

1. 读取现有文档内容
2. 分析需要更新的部分
3. 生成更新内容
4. 应用文档更新

### 步骤 4: 验证文档质量

1. 检查文档格式
2. 验证文档完整性
3. 检查文档准确性
4. 记录更新日志

### 步骤 5: 提交文档变更

1. 检查文档变更范围
2. 编写提交信息
3. 提交文档变更
4. 通知相关人员

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "code_changes"
    type: git_diff
    path: "git commit"
    description: "代码变更记录"
  
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
    description: "设计文档"

optional_inputs:
  - name: "api_spec"
    type: file
    path: "docs/api/"
    description: "API 规范文档"
```

### 输出契约

```yaml
required_outputs:
  - name: "documentation_updates"
    type: files
    path: "docs/ 或相关文档路径"
    format: "更新的文档列表"
    guarantees:
      - "文档与代码同步更新"
      - "文档格式正确"
```

### 行为契约

```yaml
preconditions:
  - "代码变更已提交"
  - "相关文档存在"

postconditions:
  - "文档与代码同步更新"
  - "文档格式正确"
  - "文档变更已提交"

invariants:
  - "文档必须反映最新实现"
  - "文档更新必须准确"
  - "文档格式必须符合规范"
```

## 示例

### 输入示例

```
代码变更：
- 新增 Order.cancel() 方法
- 修改 OrderItem 类
```

### 输出示例

```
文档更新：
1. src/order/design.md
   - 新增 Order.cancel() 方法说明
   - 更新 OrderItem 类图

2. docs/api/order-api.md
   - 新增 POST /orders/{id}/cancel 接口文档

3. docs/changelog.md
   - 记录订单取消功能新增
```

## 相关文档

- [Skill 索引](../index.md)
- [工作流编排 Skill](../workflow-orchestrator/SKILL.md)
- [进度监管 Skill](../progress-supervisor/SKILL.md)
