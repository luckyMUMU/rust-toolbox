# 文件管理插件 - file-management-plugin

## 概述

这是一个独立的 crate，包含所有文件管理相关的功能，包括：
- AI 驱动的文件夹分类
- 批量文件处理
- 文本处理（支持中文）
- 人工决策集成
- 文件夹合并

## 文件列表

- **batch_confirmation_tool.rs** - 批量确认工具
- **batch_processor.rs** - 批处理引擎
- **batch_processor_tool.rs** - 批处理工具实现
- **classification_flow.rs** - 分类流程管道
- **classification_tool.rs** - 文件夹分类工具
- **error.rs** - 错误类型定义
- **error_recovery.rs** - 错误恢复策略
- **human_decision_tool.rs** - 人工决策工具
- **monitoring.rs** - 监控和指标
- **performance.rs** - 性能优化
- **plugin.rs** - 插件入口
- **progress_tracker.rs** - 进度跟踪
- **registry.rs** - 工具注册表
- **result_confirmation_tool.rs** - 结果确认
- **result_review_tool.rs** - 结果审查
- **rule_config.rs** - 分类规则配置
- **text_processor_tool.rs** - 文本处理
- **utils.rs** - 通用工具

## 使用方法

```rust
use file_management_plugin::FileManagementPlugin;

let plugin = FileManagementPlugin::builder()
    .config(FileManagementConfig::default())
    .build()?;
```
