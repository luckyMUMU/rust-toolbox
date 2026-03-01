---
version: v3.0.2
stage: 2
---

# 阶段2: 实现与验证

goal: TDD循环执行，约束验证，代码审查

## 输入契约

```yaml
preconditions:
  required_inputs:
    - name: design_document
      type: file
      path: src/{module}/design.md
      validation: 设计文档必须存在且通过审查
    - name: design_review_report
      type: json
      path: contracts/stage-1-review.json
      validation: 审查状态必须为passed
    - name: constraint_matrix
      type: file
      path: 05_constraints/constraint-matrix.md
      validation: 约束矩阵必须存在
  constraints:
    - 设计文档必须通过审查
    - 必须明确适用的P0/P1级约束
```

## 处理流程

```yaml
steps:
  - id: 1
    name: 测试设计(TDD红阶段)
    actions:
      - 根据规范中的BDD场景生成单元测试
      - 验证测试因功能未实现而失败
      - 测试代码必须符合P3级测试规范
    output: 测试代码(失败状态)
  - id: 2
    name: 代码实现(TDD绿阶段)
    actions:
      - 编写满足测试的最小代码增量
      - 确保测试通过
      - 代码必须符合P2/P3级规范
    output: 实现代码
  - id: 3
    name: 代码重构(TDD重构阶段)
    actions:
      - 在测试保护下优化代码结构
      - 引入DDD战术模式(聚合根、值对象等)
      - 确保不引入新的失败
    output: 优化后的代码
  - id: 4
    name: 约束验证
    actions:
      - P0级约束验证(零违反)
      - P1级约束验证(警告可接受)
      - P2级约束验证
    output: 约束验证报告
  - id: 5
    name: 代码审查
    actions:
      - 设计一致性审查
      - 代码质量审查
      - 安全审查
    output: 审查报告
```

## TDD循环规范

```yaml
red:
  name: 红阶段
  actions:
    - 根据BDD场景生成单元测试
    - 验证测试因功能未实现而失败
  spec_source: BDD场景(Gherkin)

green:
  name: 绿阶段
  actions:
    - 编写满足测试的最小代码增量
    - 确保测试通过
  spec_source: 模块规范(P2级)

refactor:
  name: 重构阶段
  actions:
    - 在测试保护下优化代码结构
    - 引入DDD战术模式
    - 确保不引入新的失败
  spec_source: DDD战术模式
```

## 约束验证规范

```yaml
P0_verification:
  - constraint: 禁止硬编码密钥
    method: 静态分析
    pass: 零违反
  - constraint: 核心模块覆盖率100%
    method: 覆盖率工具
    pass: ">=100%"
  - constraint: 禁止强制解包
    method: 静态分析
    pass: 零违反
  - constraint: 禁止循环依赖
    method: 依赖分析
    pass: 零违反

P1_verification:
  - constraint: API响应时间<500ms
    method: 性能测试
    pass: 平均<500ms
  - constraint: 优先使用项目已有库
    method: 依赖审查
    pass: 无新依赖或已审批

P2_verification:
  - constraint: 遵循命名约定
    method: 代码风格检查
    pass: 无违规命名
  - constraint: 公共API必须注释
    method: 文档检查
    pass: 无缺失注释
```

## 输出契约

```yaml
stage_id: stage-2-implement-verify
version: "1.0.0"

postconditions:
  required_outputs:
    - name: code_changes
      type: git_diff
      path: git commit
      format: 符合项目代码规范的代码变更
    - name: code_review_report
      type: json
      path: contracts/stage-2-code-review.json
      format:
        review_status: passed|failed
        coverage: 0-100
        issues: [问题清单]
        security_scan: 安全扫描结果
    - name: test_report
      type: json
      path: contracts/stage-2-test-report.json
      format:
        test_status: passed|failed
        total_tests: 数字
        passed_tests: 数字
        coverage: 0-100
    - name: constraint_report
      type: json
      path: contracts/stage-2-constraint-report.json
      format:
        p0_violations: 0
        p1_warnings: 数字
        p2_warnings: 数字

invariants:
  - 代码必须通过P0级约束验证(零违反)
  - 代码必须通过P1级约束验证(警告可接受)
  - 核心模块测试覆盖率必须>=100%
  - 禁止强制解包(unwrap/expect)
  - 禁止硬编码密钥等敏感信息
```

## 质量门控

```yaml
quality_gates:
  - check: 代码规范
    pass: 通过lint检查
    fail: 返回代码修正
  - check: 测试通过
    pass: 所有测试通过
    fail: 返回测试修正
  - check: 文档同步
    pass: 文档与代码一致
    fail: 返回文档修正
  - check: P0约束
    pass: 零违反
    fail: 返回代码修正
  - check: P1约束
    pass: 无阻断性问题
    fail: 返回代码修正
  - check: 代码审查
    pass: 审查通过
    fail: 返回代码修正
```

## 状态定义

```yaml
states:
  STAGE_2_STARTED:
    trigger: 阶段1通过
    action: 执行TDD循环
  STAGE_2_TESTING:
    trigger: TDD红阶段
    action: 等待测试完成
  STAGE_2_IMPLEMENTING:
    trigger: TDD绿阶段
    action: 等待实现完成
  STAGE_2_REFACTORING:
    trigger: TDD重构阶段
    action: 等待重构完成
  STAGE_2_VERIFYING:
    trigger: 约束验证
    action: 等待验证完成
  STAGE_2_REVIEWING:
    trigger: 代码审查
    action: 等待审查完成
  STAGE_2_WAITING_CONFIRM:
    trigger: 审查完成
    action: 用户确认后进入阶段3
  STAGE_2_PASSED:
    trigger: 用户确认
    action: 进入阶段3
  STAGE_2_FAILED:
    trigger: 审查失败
    action: 返回代码修正
```

## 相关文档

- stage-1-design.md: 阶段1理解与设计
- stage-3-deliver.md: 阶段3交付与同步
- contracts/stage-2-contract.yaml: 契约模板
