# 工作流分类系统增强 Spec

## Why
当前系统已实现分类工具和合并工具，但缺少工作流编排能力和分类规则的持久化管理。需要支持类似 `run-classify.bat` 的完整工作流，并能够导入、持久化和更新 `classfy.json` 格式的分类规则。

## What Changes
- 添加分类规则持久化存储（支持导入 `classfy.json` 格式）
- 创建分类工作流定义（Set文件夹合并 → 分类 → 多目录合并）
- 修复现有测试代码中的编译错误
- 添加工作流执行入口和 CLI 支持

## Impact
- Affected specs: `classification`, `merge`, `workflow`
- Affected code: 
  - `src/plugins/file_management/classification/rule_config.rs` - 规则持久化
  - `src/plugins/file_management/merge/` - 合并工作流
  - `src/workflow/` - 工作流定义
  - 测试文件 - 修复编译错误

## ADDED Requirements

### Requirement: 分类规则持久化存储
系统应提供分类规则的持久化存储能力，支持从 `classfy.json` 格式导入、更新和导出规则。

#### Scenario: 导入 classfy.json 规则
- **GIVEN** 一个符合 `classfy.json` 格式的配置文件
- **WHEN** 用户调用规则导入接口
- **THEN** 系统解析配置并持久化到存储后端
- **AND** 返回导入的规则数量和状态

#### Scenario: 更新分类规则
- **GIVEN** 已存在的分类规则
- **WHEN** 用户更新规则的关键词或优先级
- **THEN** 系统更新存储中的规则
- **AND** 保留规则的版本历史

#### Scenario: 导出分类规则
- **GIVEN** 存储中的分类规则
- **WHEN** 用户请求导出规则
- **THEN** 系统生成 `classfy.json` 格式的配置文件

### Requirement: 分类工作流定义
系统应提供预定义的分类工作流，实现类似 `run-classify.bat` 的完整流程。

#### Scenario: 执行完整分类工作流
- **GIVEN** 目标目录和分类规则配置
- **WHEN** 用户启动分类工作流
- **THEN** 系统按顺序执行：
  1. Set 文件夹合并预处理
  2. 文件夹分类
  3. 多目录合并
- **AND** 每个步骤完成后记录进度

#### Scenario: 工作流步骤失败处理
- **GIVEN** 正在执行的工作流
- **WHEN** 某步骤执行失败
- **THEN** 系统记录失败原因
- **AND** 支持从失败点恢复执行

### Requirement: CLI 支持
系统应提供命令行接口支持工作流执行。

#### Scenario: 命令行执行分类工作流
- **GIVEN** 配置好的工作流参数
- **WHEN** 用户通过 CLI 启动工作流
- **THEN** 系统执行工作流并输出进度
- **AND** 支持日志输出模式选择

## MODIFIED Requirements

### Requirement: 测试代码修复
修复现有测试代码中的编译错误，确保测试能够正常运行。

#### Scenario: 测试编译通过
- **GIVEN** 项目测试代码
- **WHEN** 执行 `cargo test --lib`
- **THEN** 所有测试编译通过
- **AND** 测试可以执行

## REMOVED Requirements
无移除的需求。
