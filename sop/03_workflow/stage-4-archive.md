---
version: v3.0.2
stage: 4
---

# 阶段4: 归档与演化

goal: 创建归档记录，评估升级，更新重规范

## 输入契约

```yaml
preconditions:
  required_inputs:
    - name: delivery_notification
      type: json
      path: contracts/stage-3-delivery.json
      validation: 交付状态必须为completed
    - name: all_stage_contracts
      type: files
      path: contracts/stage-*.json
      validation: 所有阶段的契约文件必须存在
  constraints:
    - 所有阶段必须已完成
    - 所有契约文件必须完整
```

## 处理流程

```yaml
steps:
  - id: 1
    name: 创建归档记录
    actions: [记录实现摘要, 记录变更内容, 记录经验教训]
    output: archive.md
  - id: 2
    name: 升级评估
    actions: [评估是否影响核心架构, 评估是否需要长期维护, 评估是否跨模块影响, 评估是否涉及安全/合规]
    output: 升级评估报告
  - id: 3
    name: 更新重规范
    actions: [更新工程宪章P0级变更, 更新系统规范P1级变更, 更新模块规范P2级变更]
    output: 更新的重规范文档
    condition: 如需要
  - id: 4
    name: 更新CHANGELOG
    actions: [记录变更类型, 记录变更内容, 记录影响范围]
    output: CHANGELOG.md
```

## 归档记录模板

```markdown
# 归档记录：[功能名称]

## 基本信息
- 需求提案：[proposal.md链接]
- 技术确认：[confirmation.md链接]
- 完成时间：[YYYY-MM-DD]
- 实际耗时：[X人天]
- 负责人：[姓名]

## 实现摘要
- [实现要点1]
- [实现要点2]

## 变更记录
- [变更1：描述及原因]

## 升级评估
- 是否需要升级为重规范：是/否
- 升级原因：[原因描述]
- 目标文档：
  - [ ] [文档1](链接)

## 重规范更新内容
### 更新文档1：[文档名称]
- 新增内容：[描述]
- 修改内容：[描述]

## 经验教训
- [经验1]
- [改进建议]

归档人：[姓名]
归档时间：[YYYY-MM-DD]
```

## 升级评估规范

```yaml
must_upgrade:
  UPG-001:
    condition: 影响核心架构原则
    target: P0级架构原则文档
  UPG-002:
    condition: 新增安全要求或发现安全漏洞
    target: P0级安全基线文档
  UPG-003:
    condition: 新增核心业务流程
    target: P1级系统规范文档
  UPG-004:
    condition: 新增/修改核心API
    target: P2级API契约文档
  UPG-005:
    condition: 新增/修改核心数据模型
    target: P2级数据模型文档
  UPG-006:
    condition: 新增核心领域概念
    target: P2级领域模型文档

optional_upgrade:
  - 功能稳定运行超过3个月
  - 多个轻规范涉及同一模块，可合并升级
  - 团队决定需要长期维护的功能
```

## CHANGELOG更新规范

```yaml
change_types:
  Added: 新功能
  Changed: 功能变更
  Fixed: Bug修复
  Deprecated: 即将移除
  Removed: 已移除
  Security: 安全相关

format: |
  ## [版本号] - YYYY-MM-DD

  ### 新增
  - [新增内容1]

  ### 变更
  - [变更内容1]

  ### 修复
  - [修复内容1]

  ### 安全
  - [安全相关内容]
```

## 输出契约

```yaml
stage_id: stage-4-archive-evolve
version: "1.0.0"

postconditions:
  required_outputs:
    - name: archive_record
      type: file
      path: archives/{name}-archive.md
      format: 归档记录(Markdown)
    - name: constitution_updates
      type: files
      path: 01_constitution/或02_specifications/
      format: 更新的重规范文档(如需要)
    - name: changelog_entry
      type: file
      path: CHANGELOG.md
      format: 变更日志条目

invariants:
  - 归档记录必须包含完整决策链
  - 必须明确是否需要更新重规范
  - 变更日志必须更新
  - 经验教训必须记录
```

## 质量门控

```yaml
quality_gates:
  - check: 归档记录完整
    pass: 包含所有必需字段
    fail: 返回归档记录
  - check: 升级评估明确
    pass: 明确是否升级及原因
    fail: 返回升级评估
  - check: CHANGELOG更新
    pass: 变更日志已更新
    fail: 返回CHANGELOG更新
  - check: 经验教训记录
    pass: 记录了经验教训
    fail: 返回经验教训记录
```

## 状态定义

```yaml
states:
  STAGE_4_STARTED:
    trigger: 阶段3通过
    action: 执行归档记录
  STAGE_4_ARCHIVING:
    trigger: 归档记录
    action: 等待归档完成
  STAGE_4_EVALUATING:
    trigger: 升级评估
    action: 等待评估完成
  STAGE_4_UPDATING:
    trigger: 更新重规范
    action: 等待更新完成
  STAGE_4_COMPLETED:
    trigger: 归档完成
    action: 流程结束
```

## 相关文档

- stage-3-deliver.md: 阶段3交付与同步
- stage-0-weight.md: 阶段0规范重量选择
- contracts/stage-4-contract.yaml: 契约模板
