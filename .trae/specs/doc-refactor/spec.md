# 文档体系重构 Spec

## Why

当前项目文档体系与 SOP v3.0.0 规范不一致，存在以下问题：
- 需求文档（PRD）格式不符合 SOP 规范的需求分析标准
- 设计文档分层与 SOP 的 P1/P2/P3 级规范分层不对应
- ADR 文档格式与 SOP 模板不一致
- 文档索引混乱，缺乏统一的导航体系

需要将现有文档重构为符合 SOP v3.0.0 标准的规范体系。

## What Changes

- **重构需求文档**：将 `docs/01_requirements/workflow_toolkit_prd.md` 转换为 SOP 标准的需求规范文档
- **重构设计文档**：将 `design.md` 和各层 `design.md` 转换为 SOP 标准的实现设计文档（P2/P3 级）
- **重构 ADR 文档**：将 `docs/04_context_reference/architecture_decision.md` 和 `adr/` 下的文档转换为 SOP 标准格式
- **清理重复文档**：删除或归档与 SOP 冲突的旧文档
- **更新文档索引**：建立符合 SOP 标准的文档导航体系

## Impact

- **Affected specs**: 
  - P1 级系统规范（原 PRD 转换）
  - P2 级模块规范（各子系统设计）
  - P3 级实现规范（编码、测试规范）
- **Affected code**: 无代码变更，仅文档重构
- **Affected docs**: 
  - `docs/` 目录结构将大幅调整
  - `sop/02_specifications/` 将包含转换后的系统规范

## ADDED Requirements

### Requirement: 需求规范文档
系统 SHALL 提供符合 SOP 标准的需求规范文档，包含：
- BDD 场景描述（Gherkin 语法）
- 用户故事地图
- 功能需求清单（优先级标注）
- 非功能需求定义

#### Scenario: 需求规范转换成功
- **WHEN** 将 PRD 转换为 SOP 需求规范
- **THEN** 包含所有原始 PRD 的核心功能需求
- **AND** 使用 Gherkin 语法描述关键场景
- **AND** 需求优先级与 SOP 规范一致（P0/P1/P2）

### Requirement: 设计文档分层
系统 SHALL 提供分层设计文档，包含：
- P1 级架构设计（系统架构、技术选型）
- P2 级模块设计（模块边界、接口定义）
- P3 级实现设计（类设计、算法设计）

#### Scenario: 设计文档符合分层规范
- **WHEN** 读取设计文档
- **THEN** 可以清晰识别 P1/P2/P3 级内容
- **AND** 每层文档都有明确的版本号和审批记录

### Requirement: ADR 文档标准化
系统 SHALL 提供标准格式的架构决策记录，包含：
- 决策状态（Proposed/Accepted/Deprecated）
- 决策背景和理由
- 替代方案分析
- 决策后果

#### Scenario: ADR 文档格式正确
- **WHEN** 读取任意 ADR 文档
- **THEN** 符合 SOP 模板格式
- **AND** 包含所有必需字段
- **AND** 有唯一的 ADR 编号

## MODIFIED Requirements

### Requirement: 系统规范（原 PRD）
**原需求**：PRD 文档包含产品需求、用户分析、功能需求

**修改后**：
```
# 系统规范（P1 级）

## 系统功能边界
- 核心功能清单（表格形式）
- 功能边界定义（包含/不包含）

## 核心业务流程
- 5 阶段工作流映射
- 规范驱动流程

## 外部接口定义
- 输入接口（CLI/TUI/MCP）
- 输出接口
- 契约接口格式

## 性能指标
- 响应时间目标
- 吞吐量目标
- 可用性目标
```

### Requirement: 设计文档（原 design.md）
**原需求**：设计文档包含架构分层、技术决策、状态记录

**修改后**：
```
# 实现设计（P2/P3 级）

## 设计概述
- 设计目标
- 设计约束（追溯到 P0/P1 规范）

## 领域模型设计
- 聚合根定义
- 值对象定义
- 领域服务定义

## 模块设计
- 模块边界
- 接口定义
- 依赖关系

## 实现细节
- 关键算法
- 数据结构
- 错误处理策略
```

## REMOVED Requirements

### Requirement: 旧 PRD 格式
**Reason**: PRD 格式不符合 SOP 规范的需求分析标准，缺少 BDD 场景描述和规范追溯

**Migration**: 
- 核心功能需求 → 系统规范（P1 级）
- 用户故事 → 需求规范（BDD 场景）
- 非功能需求 → 质量约束（P1/P2 级）

### Requirement: 旧 ADR 格式
**Reason**: ADR 格式与 SOP 模板不一致，缺少状态追踪和后果分析

**Migration**: 
- 保留所有决策内容
- 重新格式化为 SOP 标准模板
- 添加状态标记和版本历史

### Requirement: 旧文档索引
**Reason**: 文档索引混乱，与 SOP 导航体系不一致

**Migration**: 
- 建立 SOP 标准文档索引（sop/02_specifications/index.md）
- 旧文档索引移动到归档目录
- 更新所有交叉引用
