# Tasks

## Phase 1: 修复测试编译错误
- [x] Task 1: 修复 ToolMetadata 字段问题
  - [x] SubTask 1.1: 修复 `src/tools/composed_executor.rs` 测试中的 ToolMetadata 字段
  - [x] SubTask 1.2: 修复 `src/workflow/component/tool.rs` 测试中的 ToolMetadata 字段
- [x] Task 2: 修复 RetryPolicy 和 WorkflowConfig 字段问题
  - [x] SubTask 2.1: 修复 `src/workflow/config_validator.rs` 测试中的 RetryPolicy 字段
  - [x] SubTask 2.2: 修复 `src/workflow/config_validator.rs` 测试中的 WorkflowConfig 字段
- [x] Task 3: 修复错误处理相关测试
  - [x] SubTask 3.1: 修复 `src/workflow/error_handler.rs` 测试中的 String 类型错误
- [x] Task 4: 修复 CheckpointType 和 StateManager 测试
  - [x] SubTask 4.1: 为 CheckpointType 添加 PartialEq derive
  - [x] SubTask 4.2: 修复 StateManager 测试中的 default 方法调用
- [x] Task 5: 修复其他测试编译错误
  - [x] SubTask 5.1: 修复 `src/application/port/unit_of_work.rs` 生命周期问题
  - [x] SubTask 5.2: 修复 `src/workflow/component/tool.rs` boxed 方法问题

## Phase 2: 分类规则持久化
- [x] Task 6: 创建分类规则存储服务
  - [x] SubTask 6.1: 定义 RuleStorageService 接口
  - [x] SubTask 6.2: 实现基于文件系统的规则存储
  - [x] SubTask 6.3: 添加规则版本管理
- [x] Task 7: 实现 classfy.json 格式支持
  - [x] SubTask 7.1: 定义 classfy.json 的 Rust 数据结构
  - [x] SubTask 7.2: 实现格式转换器（classfy.json ↔ ClassificationRules）
  - [x] SubTask 7.3: 添加导入/导出 API

## Phase 3: 分类工作流定义
- [x] Task 8: 创建分类工作流组件
  - [x] SubTask 8.1: 定义 SetFolderMergeComponent（Set文件夹合并）
  - [x] SubTask 8.2: 定义 ClassificationComponent（文件夹分类）
  - [x] SubTask 8.3: 定义 MultiFolderMergeComponent（多目录合并）
- [x] Task 9: 创建预定义工作流模板
  - [x] SubTask 9.1: 定义 ClassificationWorkflowDefinition
  - [x] SubTask 9.2: 实现工作流构建器
  - [x] SubTask 9.3: 添加工作流参数配置

## Phase 4: CLI 支持
- [x] Task 10: 添加 CLI 入口
  - [x] SubTask 10.1: 定义 CLI 参数结构
  - [x] SubTask 10.2: 实现工作流执行命令
  - [x] SubTask 10.3: 添加进度输出和日志模式

# Task Dependencies
- [Task 2] depends on [Task 1]
- [Task 3] depends on [Task 1]
- [Task 4] depends on [Task 1]
- [Task 5] depends on [Task 1, Task 2, Task 3, Task 4]
- [Task 6] can run in parallel with [Task 1-5]
- [Task 7] depends on [Task 6]
- [Task 8] depends on [Task 7]
- [Task 9] depends on [Task 8]
- [Task 10] depends on [Task 9]
