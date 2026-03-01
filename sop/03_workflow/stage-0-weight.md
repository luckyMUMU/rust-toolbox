---
version: v3.0.2
stage: 0
---

# 阶段0: 规范重量选择

goal: 评估需求复杂度，推荐规范重量(重规范/轻规范/快速路径)

## 输入契约

```yaml
requirement_description:
  type: text
  validation: 长度>=10字符
```

## 处理流程

```yaml
steps:
  - id: 1
    name: 需求分析
    actions: [解析需求描述, 识别功能范围, 识别技术影响]
  - id: 2
    name: 复杂度评估
    actions: [评估功能复杂度1-10, 评估技术复杂度1-10, 评估影响范围模块数, 计算综合复杂度分数]
  - id: 3
    name: 规范重量推荐
    actions: [根据复杂度分数推荐规范重量, 列出必需文档清单, 评估工作量]
  - id: 4
    name: 用户确认
    actions: [展示推荐结果, 等待用户确认或调整, 输出规范重量决策]
```

## 复杂度评估标准

```yaml
complexity:
  functional:
    1-3: 简单功能(配置修改、文案调整)
    4-6: 中等功能(新增接口、数据表)
    7-10: 复杂功能(新模块、新系统)
  technical:
    1-3: 无技术影响(纯前端修改)
    4-6: 中等技术影响(新增依赖、接口变更)
    7-10: 重大技术影响(架构变更、技术栈变更)
  scope:
    1-3: 单模块(单个服务/模块)
    4-6: 多模块(2-5个模块)
    7-10: 跨系统(跨多个系统)

calculation: 综合复杂度 = (功能复杂度 + 技术复杂度 + 影响范围) / 3

recommendation:
  - score: ">=7"
    path: heavy
  - score: "4-6"
    path: light
  - score: "<=3"
    path: fast
```

## 规范重量选择标准

```yaml
heavy:
  triggers:
    - 从0到1的核心系统开发
    - 跨团队协作的大型功能
    - 安全/金融等高风险模块
    - 需要长期演进的基础设施
    - 架构级变更
    - 核心接口变更
  required_docs:
    - 工程宪章(4个文档)
    - 系统规范
    - API契约
    - 数据模型
    - 约束矩阵

light:
  triggers:
    - 小功能/增量需求
    - 不涉及架构变更
    - 不涉及核心接口变更
    - 影响范围可控
  required_docs:
    - proposal.md(需求提案)
    - confirmation.md(技术确认)
    - archive.md(归档记录)

fast:
  triggers:
    - 单文件修改
    - 行数变化<30行
    - 无逻辑变更
    - 无测试依赖
    - 无文档依赖
  skip_stages: [stage_1]
```

## 输出契约

```yaml
stage_id: stage-0-weight-selection
version: "1.0.0"

decision:
  recommended_weight: heavy|light|fast
  reason: 推荐原因
  complexity_score: 1-10
  complexity_breakdown:
    functional: 1-10
    technical: 1-10
    scope: 1-10

required_documents:
  - name: 文档名称
    required: true|false
    priority: P0|P1|P2|P3

estimated_effort:
  development: X人天
  testing: X人天
  documentation: X人天

next_stage: stage-1-design
```

## 质量门控

```yaml
quality_gates:
  - check: 需求描述清晰
    pass: 长度>=10字符，无歧义
    fail: 返回需求澄清
  - check: 复杂度评估合理
    pass: 分数在1-10范围内
    fail: 重新评估
  - check: 推荐结果明确
    pass: 明确推荐重规范/轻规范/快速路径
    fail: 重新推荐
```

## 状态定义

```yaml
states:
  STAGE_0_STARTED:
    trigger: 工作流入口
    action: 执行需求分析
  STAGE_0_EVALUATING:
    trigger: 复杂度评估
    action: 等待评估完成
  STAGE_0_WAITING_CONFIRM:
    trigger: 评估完成
    action: 用户确认后进入阶段1
  STAGE_0_PASSED:
    trigger: 用户确认
    action: 进入阶段1
```

## 相关文档

- index.md: 工作流入口
- stage-1-design.md: 阶段1理解与设计
- contracts/stage-0-contract.yaml: 契约模板
