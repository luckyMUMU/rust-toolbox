---
version: v3.0.2
stage: 1
---

# 阶段1: 理解与设计

goal: 理解需求，完成设计(BDD场景 + DDD战术映射)

## 输入契约

```yaml
preconditions:
  required_inputs:
    - name: weight_decision
      type: json
      path: contracts/stage-0-decision.json
      validation: 必须包含recommended_weight字段
    - name: requirement_description
      type: text
      description: 需求描述文本
  constraints:
    - 输入必须通过阶段0的质量门控
```

## 处理流程

```yaml
steps:
  - id: 1
    name: 需求澄清
    rounds:
      - name: 业务背景与目标
        questions:
          - q: 这个需求要解决什么业务问题？
            purpose: 理解问题本质
          - q: 谁会使用这个功能？
            purpose: 明确用户画像
          - q: 如何衡量这个功能的成功？
            purpose: 定义验收标准
          - q: 这个需求的优先级如何？
            purpose: 评估优先级
      - name: 长期目标与短期实现
        questions:
          - q: 这个功能最终要达到什么状态？
            purpose: 理解终极目标
          - q: 第一期需要交付哪些核心能力？
            purpose: 定义MVP范围
          - q: 从短期到长期的演进路径是什么？
            purpose: 规划迭代路线
          - q: 短期实现是否可以接受技术债务？
            purpose: 评估技术风险
      - name: 边界与约束
        questions:
          - q: 这个功能不包含什么？
            purpose: 避免范围蔓延
          - q: 是否有技术栈、性能、安全方面的约束？
            purpose: 识别技术限制
          - q: 是否需要兼容现有系统？
            purpose: 评估集成影响
          - q: 团队规模、时间预算、技术能力如何？
            purpose: 评估可行性
      - name: 风险与假设
        questions:
          - q: 这个需求最大的风险是什么？
            purpose: 提前规避风险
          - q: 我们做了哪些假设？
            purpose: 暴露隐藏假设
          - q: 如果失败，可能的失败模式是什么？
            purpose: 设计降级方案
          - q: 如果上线后出现问题，如何回滚？
            purpose: 制定应急预案
    output: specs/{name}-clarification.md
  - id: 2
    name: 需求分析
    actions:
      - 使用Gherkin语法编写场景
      - 覆盖正常流程和异常流程
      - 与业务专家达成共识
    output: specs/{name}-spec.md
  - id: 3
    name: 架构设计
    actions:
      - 定义限界上下文
      - 建立统一语言
      - 制定架构原则与技术栈
    output: docs/02_logical_workflow/{name}-architecture.md
  - id: 4
    name: 实现设计
    actions:
      - 定义聚合根、值对象、领域服务
      - 明确仓储模式和领域事件
      - 映射到模块规范
    output: src/{module}/design.md
  - id: 5
    name: 设计审查
    actions:
      - 架构审查
      - 约束验证P0/P1级
      - 用户确认
    output: contracts/stage-1-review.json
```

## 澄清记录模板

```markdown
# 需求澄清记录

## 业务背景与目标
- 业务问题：
- 目标用户：
- 成功标准：
- 优先级：

## 长期目标与短期实现
- 长期愿景：
- 短期目标(MVP)：
- 演进路径：
- 技术债务策略：

## 边界与约束
- 功能边界(不包含)：
- 技术约束：
- 兼容性要求：
- 资源限制：

## 风险与假设
- 已识别风险：
- 关键假设：
- 失败场景：
- 回滚策略：

## 用户确认
- 确认人：
- 确认时间：
- 确认内容：
```

## BDD场景规范

```yaml
gherkin_syntax:
  feature: Feature: [功能名称]
  scenario: Scenario: [场景名称]
  keywords: [Given, When, Then, And]

coverage:
  normal_flow: 100%
  exception_flow: 100%
  boundary: 关键边界
```

## DDD战术映射

```yaml
aggregate_root:
  - 唯一入口: 聚合根是访问聚合内实体的唯一入口
  - 边界保护: 聚合外不能直接修改聚合内实体
  - 事务边界: 一个聚合 = 一个事务

value_object:
  - 不可变: 值对象创建后不可修改
  - 相等性: 值对象通过属性值判断相等
  - 自验证: 值对象构造时验证自身有效性

domain_service:
  - 无状态: 领域服务不应持有状态
  - 跨实体: 封装跨实体的业务逻辑
  - 领域语言: 方法名应使用领域语言

repository:
  - 接口在领域层: 仓储接口定义在领域层
  - 实现在基础设施层: 仓储实现在基础设施层
  - 隐藏实现细节: 不暴露数据库细节
```

## 输出契约

```yaml
stage_id: stage-1-understand-design
version: "1.2.0"

postconditions:
  required_outputs:
    - name: clarification_document
      type: file
      path: specs/{name}-clarification.md
      format: Markdown，包含4个维度的澄清记录
      guarantees:
        - 澄清记录包含长期目标与短期实现
        - 澄清记录已由用户确认
    - name: spec_document
      type: file
      path: specs/{name}-spec.md
      format: Markdown，包含Gherkin场景
    - name: architecture_document
      type: file
      path: docs/02_logical_workflow/{name}-architecture.md
      format: Markdown，包含架构图、接口定义
    - name: design_document
      type: file
      path: src/{module}/design.md
      format: Markdown，包含实现设计
    - name: design_review_report
      type: json
      path: contracts/stage-1-review.json
      format:
        review_status: passed|failed
        reviewers: [审查者列表]
        issues: [问题清单]
        approvals: [批准者签名]

invariants:
  - 设计必须符合P0级约束(质量红线)
  - 设计必须符合P1级约束(架构原则)
  - 设计文档必须包含可测试性设计
  - 所有接口必须有明确的输入输出定义
  - 需求澄清记录必须包含长期目标与短期实现
```

## 质量门控

```yaml
quality_gates:
  - check: 需求澄清完成
    pass: 完成4轮提问，记录完整
    fail: 返回需求澄清
  - check: 长期短期区分
    pass: 明确长期目标与短期实现
    fail: 返回需求澄清
  - check: 用户意图确认
    pass: 用户确认需求澄清记录
    fail: 返回需求澄清
  - check: 需求边界清晰
    pass: 功能范围明确，无歧义
    fail: 返回需求分析
  - check: 技术方案对齐
    pass: 技术选型合理，与现有系统兼容
    fail: 返回架构设计
  - check: 验收标准具体
    pass: 每个功能都有明确的验收条件
    fail: 返回需求分析
  - check: 关键假设确认
    pass: 所有关键假设已与用户确认
    fail: 返回需求分析
  - check: 架构图清晰
    pass: 架构图完整，模块边界明确
    fail: 返回架构设计
  - check: 接口定义完整
    pass: 所有接口都有明确的输入输出定义
    fail: 返回实现设计
  - check: 与现有系统无冲突
    pass: 不与现有架构/接口冲突
    fail: 返回架构设计
  - check: 设计可行
    pass: 技术方案可实现
    fail: 返回实现设计
```

## 状态定义

```yaml
states:
  STAGE_1_STARTED:
    trigger: 阶段0通过
    action: 执行需求澄清
  STAGE_1_CLARIFYING:
    trigger: 需求澄清
    action: 等待澄清完成
  STAGE_1_ANALYZING:
    trigger: 需求分析
    action: 等待分析完成
  STAGE_1_DESIGNING:
    trigger: 设计阶段
    action: 等待设计完成
  STAGE_1_REVIEWING:
    trigger: 设计审查
    action: 等待审查完成
  STAGE_1_WAITING_CONFIRM:
    trigger: 审查完成
    action: 用户确认后进入阶段2
  STAGE_1_PASSED:
    trigger: 用户确认
    action: 进入阶段2
  STAGE_1_FAILED:
    trigger: 审查失败
    action: 返回设计修正
```

## 相关文档

- stage-0-weight.md: 阶段0规范重量选择
- stage-2-implement.md: 阶段2实现与验证
- contracts/stage-1-contract.yaml: 契约模板
