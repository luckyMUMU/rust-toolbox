---
version: v3.0.2
stage: 3
---

# 阶段3: 交付与同步

goal: 文档同步，索引更新，完成交付

## 输入契约

```yaml
preconditions:
  required_inputs:
    - name: code_changes
      type: git_diff
      path: git commit
      validation: 代码变更必须存在且已提交
    - name: code_review_report
      type: json
      path: contracts/stage-2-code-review.json
      validation: 审查状态必须为passed
    - name: test_report
      type: json
      path: contracts/stage-2-test-report.json
      validation: 测试状态必须为passed
  constraints:
    - 所有审查和测试必须通过
    - 代码必须已合并到目标分支
```

## 处理流程

```yaml
steps:
  - id: 1
    name: 文档同步
    actions: [同步README.md, 同步API文档, 同步CHANGELOG.md]
    output: 更新的文档列表
  - id: 2
    name: 索引更新
    actions: [更新文档索引, 更新导航链接, 验证链接有效性]
    output: 更新的索引文件
  - id: 3
    name: 交付确认
    actions: [验证交付物完整性, 生成变更摘要, 发送交付通知]
    output: 完成通知 + 变更摘要
```

## 文档同步规范

```yaml
sync_rules:
  README.md:
    condition: 功能变更
    content: [功能描述, 使用说明]
  API文档:
    condition: 接口变更
    content: [接口定义, 参数说明]
  CHANGELOG.md:
    condition: 所有变更
    content: 变更记录
  design.md:
    condition: 设计变更
    content: 设计更新

checklist:
  - README.md已更新
  - API文档已更新(如有接口变更)
  - CHANGELOG.md已更新
  - 所有链接有效
  - 版本号已更新
```

## 索引更新规范

```yaml
index_files:
  - file: sop/index.md
    desc: SOP入口索引
    trigger: 结构变更
  - file: docs/index.md
    desc: 文档索引
    trigger: 文档变更
  - file: src/index.md
    desc: 源码索引
    trigger: 模块变更

link_rules:
  - 所有内部链接必须有效
  - 从入口到任意文档最短跳数<=3
  - 无孤立文档(无入链的文档)
```

## 输出契约

```yaml
stage_id: stage-3-deliver-sync
version: "1.0.0"

postconditions:
  required_outputs:
    - name: delivery_notification
      type: json
      path: contracts/stage-3-delivery.json
      format:
        delivery_status: completed
        delivered_at: 时间戳
        summary: 交付摘要
        changes: [变更清单]
    - name: documentation_updates
      type: files
      path: docs/或相关文档路径
      format: 更新的文档列表

invariants:
  - 交付物必须完整(代码+文档+测试)
  - 所有变更必须可追溯(Git Commit Hash)
  - 文档必须与代码同步更新
  - 用户可见变更必须有通知
```

## 质量门控

```yaml
quality_gates:
  - check: 需求实现
    pass: 所有需求已实现
    fail: 返回实现阶段
  - check: 验收满足
    pass: 所有验收条件满足
    fail: 返回验证阶段
  - check: 质量达标
    pass: 所有质量指标达标
    fail: 返回相应阶段
  - check: 文档同步
    pass: 文档与代码一致
    fail: 返回文档同步
  - check: 索引更新
    pass: 索引已更新
    fail: 返回索引更新
  - check: 链接有效
    pass: 所有链接有效
    fail: 返回链接修复
```

## 状态定义

```yaml
states:
  STAGE_3_STARTED:
    trigger: 阶段2通过
    action: 执行文档同步
  STAGE_3_SYNCING:
    trigger: 文档同步
    action: 等待同步完成
  STAGE_3_INDEXING:
    trigger: 索引更新
    action: 等待更新完成
  STAGE_3_WAITING_CONFIRM:
    trigger: 同步完成
    action: 用户确认后进入阶段4
  STAGE_3_PASSED:
    trigger: 用户确认
    action: 进入阶段4
```

## 相关文档

- stage-2-implement.md: 阶段2实现与验证
- stage-4-archive.md: 阶段4归档与演化
- contracts/stage-3-contract.yaml: 契约模板
