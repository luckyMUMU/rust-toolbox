---
version: v3.0.2
sop_path: sop/
docs_output_path: docs/
protected_directories: {sop: true, docs/参考: true}
allow_modify_protected: false
---

# SOP (Spec-first)

core_principle: 规范是核心，Skill是实现方式
sop_path: sop/

## 目录保护

protected:
  - path: sop/
    protected: true
    reason: SOP规范文件
  - path: docs/参考/
    protected: true
    reason: 参考文档

allow_modify_protected: false
change_requires_user_auth: true

## 文档创建位置

output_paths:
  spec: docs/specs/{name}-spec.md
  clarification: docs/specs/{name}-clarification.md
  design: docs/design/{name}-design.md
  archive: docs/archive/{name}-archive.md
  contract: docs/contracts/{stage}-contract.yaml

## 规范分层

layers:
  P0:
    name: 工程宪章
    docs: [project-charter, quality-redlines, architecture-principles, security-baseline]
    path: 01_constitution/
    constraint: 不可违背，违反即熔断
  P1:
    name: 系统规范
    docs: [system-spec]
    path: 02_specifications/
    constraint: 跨模块约束，技术负责人审批
  P2:
    name: 模块规范
    docs: [api-contract, data-model, domain-model]
    path: 02_specifications/
    constraint: 单模块约束，模块负责人审批
  P3:
    name: 实现规范
    docs: [coding-standards, testing-standards]
    path: 05_constraints/p3-constraints.md
    constraint: 自动化工具验证

## 5阶段工作流

workflow:
  stage_0:
    name: 规范重量选择
    actions: [评估需求复杂度, 推荐规范重量]
    output: 规范重量决策JSON
    doc: 03_workflow/stage-0-weight.md
  stage_1:
    name: 理解与设计
    actions: [多轮次多维度提问, 需求分析BDD, 架构设计DDD, 实现设计]
    output: [clarification.md, spec.md, architecture-design.md, design.md]
    doc: 03_workflow/stage-1-design.md
  stage_2:
    name: 实现与验证
    actions: [TDD循环, 约束验证P0-P3, 代码审查]
    output: [代码变更, 审查报告, 测试报告]
    doc: 03_workflow/stage-2-implement.md
  stage_3:
    name: 交付与同步
    actions: [文档同步, 索引更新]
    output: [完成通知, 变更摘要]
    doc: 03_workflow/stage-3-deliver.md
  stage_4:
    name: 归档与演化
    actions: [创建归档记录, 升级评估, 更新CHANGELOG]
    output: [归档记录, CHANGELOG]
    doc: 03_workflow/stage-4-archive.md

## 核心约束

constraints:
  spec_first:
    - 先写规范后写代码
    - 规范是唯一真理源
    - P0级需技术委员会审批，P1级需技术负责人审批
  contract:
    - 各环节独立上下文
    - 契约显式声明
    - 契约版本化
  layer:
    - P0级不可违背
    - P1级警告可接受
    - P2-P3级自动化验证
  verification:
    - 验证规范是否被满足
    - 审查须用户确认
    - 无出处不决断
  directory_protection:
    - 保护目录禁止修改
    - 文档创建在docs/目录
    - 变更需用户授权

## 质量门控

quality_gates:
  stage_0:
    checks: [需求描述清晰, 复杂度评估合理]
    pass: 全部通过
    fail: 返回需求澄清
  stage_1:
    checks: [需求澄清完成, 长期短期区分, 用户意图确认, 设计文档完整, 符合P0约束, 通过审查]
    pass: 全部通过
    fail: 返回需求澄清或设计修正
  stage_2:
    checks: [代码规范, 测试通过, 约束验证通过]
    pass: 全部通过
    fail: 返回实现修正
  stage_3:
    checks: [文档同步完成, 索引更新正确]
    pass: 全部通过
    fail: 返回同步修正
  stage_4:
    checks: [归档记录完整, CHANGELOG更新]
    pass: 全部通过
    fail: 返回归档修正

gate_failure:
  - 每次失败需用户决策
  - 可选: 修复后重试/回滚/终止

## 路径选择

paths:
  heavy:
    scenarios: [从0到1核心系统, 跨团队大型功能, 安全金融高风险模块]
    flow: stage_0 -> stage_1(完整设计) -> stage_2 -> stage_3 -> stage_4
  light:
    scenarios: [小功能增量需求, UI接口配置改动, Bug修复]
    flow: stage_0 -> stage_1(简化设计) -> stage_2 -> stage_3 -> stage_4
  fast:
    scenarios: [单文件, <30行, 无逻辑变更]
    flow: stage_2(跳过设计) -> stage_3

## Skill分类

skills:
  specification:
    duty: 生成规范文档
    examples: [sop-requirement-analyst, sop-architecture-design]
  implementation:
    duty: 将规范翻译为代码
    examples: [sop-code-implementation, sop-test-implementation]
  verification:
    duty: 验证规范是否被满足
    examples: [sop-architecture-reviewer, sop-code-review]
  orchestration:
    duty: 管理规范版本和流程
    examples: [sop-workflow-orchestrator, sop-document-sync]

## 命令速查

commands:
  /start: 启动工作流
  /spec-weight: 评估规范重量
  /propose: 创建需求提案
  /design: 生成设计文档
  /implement: 开始实现
  /review: 请求代码审查
  /approve: 批准变更
  /archive: 创建归档记录

## 状态速查

states:
  WORKFLOW_STARTED: 工作流启动
  STAGE_N_PASSED: 阶段N通过
  STAGE_N_FAILED: 阶段N失败
  CONSTRAINT_P0_VIOLATED: P0级约束违反
  SPEC_APPROVED: 规范已批准

## 目录结构

structure:
  AGENT_SOP.md: 唯一入口
  01_constitution/: P0级工程宪章
  02_specifications/: P1-P2级系统模块规范
  03_workflow/: 工作流5阶段
  04_skills/: Skill定义
  05_constraints/: 约束定义P0-P3
  06_templates/: 模板与格式
  07_reference/: 参考资料
  CHANGELOG.md: 版本历史
