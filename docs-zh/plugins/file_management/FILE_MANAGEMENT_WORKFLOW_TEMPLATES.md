# 文件管理流程模板指南

本指南为文件管理工具（File Management Tools）的流程模板提供全面文档，包括详细的使用说明、配置选项以及创建交互式文件管理流程的最佳实践。

## 目录

1. [模板概览](#模板概览)  
2. [交互式分类流程](#交互式分类流程)  
3. [交互式合并流程](#交互式合并流程)  
4. [交互式批处理流程](#交互式批处理流程)  
5. [配置参考](#配置参考)  
6. [使用示例](#使用示例)  
7. [高级模式](#高级模式)  
8. [故障排除](#故障排除)

## 模板概览

文件管理工具包含三个完整的流程模板，适用于不同的文件管理场景：

| 模板 | 用途 | 核心特性 |
|----------|---------|--------------|
| **交互式分类** | 智能文件夹整理 | 人工决策、实验模式、中文支持 |
| **交互式合并** | 重复文件夹整合 | 多种策略、冲突解决、备份功能 |
| **交互式批处理** | 通用批量操作 | 并行处理、错误恢复、过滤机制 |

### 公共特性

所有模板均具备以下核心能力：

✅ **人工决策集成**：在模糊场景中进行交互式决策  
✅ **实验模式**：执行前预览操作并生成详细报告  
✅ **错误处理**：全面的错误恢复与回滚能力  
✅ **进度追踪**：实时进度更新与性能指标  
✅ **可配置批处理**：可调节的批次大小和并发级别  
✅ **冲突解决**：多种文件冲突处理策略  
✅ **审计追踪**：完整记录并报告所有操作  

## 交互式分类流程

**文件路径：** `examples/templates/interactive-classification-workflow.yaml`

### 用途

基于可配置规则智能分类和整理文件夹，支持人工决策和实验模式预览。

### 流程步骤

1. **📁 扫描文件夹**：发现源目录中的所有文件夹  
2. **✅ 验证规则**：检查分类规则是否有效且完整  
3. **🔄 批量分类**：以可配置批次并行处理文件夹  
4. **📊 审查结果**：显示分类统计数据和置信度评分  
5. **❓ 实验模式检查**：根据实验模式设置分支执行  
6. **👤 人工确认**：获取用户对操作的批准（若启用实验模式）  
7. **⚡ 执行操作**：将文件夹移至分类位置，并处理冲突  
8. **📋 生成报告**：创建包含统计信息的详细操作报告  
9. **🧹 清理**：删除空目录并执行最终清理  

### 参数

#### 必需参数

| 参数 | 类型 | 描述 |
|-----------|------|-------------|
| `source_directory` | 字符串 | 包含待分类文件夹的目录 |
| `output_directory` | 字符串 | 整理后文件夹的根目录 |
| `classification_rules` | 对象/字符串 | 分类规则（JSON对象或文件路径） |

#### 可选参数

| 参数 | 类型 | 默认值 | 描述 |
|-----------|------|---------|-------------|
| `experimental_mode` | 布尔值 | `true` | 启用实验模式（模拟操作） |
| `enable_user_interaction` | 布尔值 | `true` | 启用人工决策 |
| `decision_timeout` | 整数 | `300` | 人工决策超时时间（秒） |
| `batch_size` | 整数 | `10` | 每批次处理的文件夹数量 |
| `confidence_threshold` | 数字 | `0.8` | 自动分类所需的最低置信度 |
| `enable_chinese_processing` | 布尔值 | `false` | 启用中文文本处理和拼音支持 |
| `cleanup_empty_directories` | 布尔值 | `true` | 操作完成后删除空目录 |

### 使用示例

#### 基础分类

```bash
# 使用基础规则进行简单文件夹分类
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/Downloads" \
  --param output_directory="/home/user/Organized" \
  --param classification_rules="basic-rules.json" \
  --param experimental_mode=true
```

#### 生产环境分类

```bash
# 生产模式自动执行
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/data/incoming" \
  --param output_directory="/data/organized" \
  --param classification_rules="production-rules.json" \
  --param experimental_mode=false \
  --param enable_user_interaction=false \
  --param confidence_threshold=0.9
```

#### 中文文本处理

```bash
# 支持中文文本的分类
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/中文文件夹" \
  --param output_directory="/home/user/整理后" \
  --param classification_rules="chinese-rules.json" \
  --param enable_chinese_processing=true \
  --param experimental_mode=true
```

### 分类规则格式

```json
{
  "version": "1.0",
  "metadata": {
    "name": "标准分类规则",
    "description": "通用文件夹分类",
    "author": "系统管理员"
  },
  "categories": {
    "documents": {
      "description": "文档和文本文件",
      "keywords": [
        {"pattern": "doc", "weight": 1.0, "case_sensitive": false},
        {"pattern": "pdf", "weight": 1.0, "case_sensitive": false},
        {"pattern": "report", "weight": 1.2, "case_sensitive": false}
      ],
      "target_directory": "Documents",
      "confidence_threshold": 0.7,
      "priority": 1
    },
    "projects": {
      "description": "项目相关文件夹",
      "keywords": [
        {"pattern": "project", "weight": 1.0},
        {"pattern": "work", "weight": 0.8},
        {"pattern": "client", "weight": 1.1}
      ],
      "target_directory": "Projects/{year}",
      "confidence_threshold": 0.8,
      "priority": 2
    }
  },
  "settings": {
    "minimum_score_threshold": 0.5,
    "case_sensitive": false,
    "enable_chinese_processing": false,
    "enable_regex_patterns": true
  }
}
```

### 节点与实现映射

`interactive-classification-workflow.yaml` 由多个节点组成，每个节点对应特定的 Rust 实现。

| 节点 ID | `tool_name` | Rust 实现 | 在流程中的角色 |
| :--- | :--- | :--- | :--- |
| `scan_folders` | `directory-scanner` | `PlaceholderExecutor`（测试中模拟） | 发现源文件夹 |
| `validate_rules` | `rule-validator` | `PlaceholderExecutor`（测试中模拟） | 确保规则格式正确 |
| `batch_classify` | `batch-processor` | `BatchProcessorTool` | 编排批量分类 |
| `review_results` | `result-reviewer` | `ResultReviewTool` | 汇总并显示分类统计 |
| `human_decision` | `human-decision` | `HumanDecisionTool` | 获取用户批准/覆盖 |
| `execute_moves` | `file-mover` | `FileMoverExecutor` | 执行物理文件/文件夹移动 |
| `generate_report` | `operation-reporter`| `PlaceholderExecutor` | 创建操作总结报告 |
| `cleanup` | `folder-cleanup` | `PlaceholderExecutor` | 删除空目录 |

---

## 交互式合并流程

**文件路径：** `examples/templates/interactive-merge-workflow.yaml`

### 用途

智能合并多个位置的重复文件夹，支持用户引导的策略选择和全面的冲突解决。

### 流程步骤

1. **📁 扫描目录**：发现源目录中的所有文件夹  
2. **🔍 查找公共文件夹**：识别跨位置同名文件夹  
3. **📊 分析合并候选**：计算大小、检测冲突并评估复杂度  
4. **📋 审查分析**：显示合并分析、统计数据和建议  
5. **🎯 确定策略**：选择合并策略（自动或用户引导）  
6. **👤 决策策略**：获取用户对合并策略的决策（如需要）  
7. **📝 创建合并计划**：生成包含所有操作的详细执行计划  
8. **⚠️ 处理冲突**：通过用户输入解决重复文件冲突  
9. **✅ 完成计划**：完成包含所有冲突解决方案的合并计划  
10. **📊 审查最终计划**：显示完整的最终合并计划  
11. **❓ 实验模式检查**：根据实验模式设置分支执行  
12. **👤 确认执行**：获取用户确认（若启用实验模式）  
13. **💾 创建备份**：在合并操作前创建备份（若启用）  
14. **⚡ 执行合并**：执行文件夹合并操作并跟踪进度  
15. **🔍 验证结果**：验证合并操作成功完成  
16. **📋 生成报告**：创建包含统计信息的综合合并报告  
17. **🧹 清理**：成功合并后删除空目录  

### 参数

#### 必需参数

| 参数 | 类型 | 描述 |
|-----------|------|-------------|
| `source_directories` | 数组 | 待扫描合并文件夹的目录列表 |

#### 可选参数

| 参数 | 类型 | 默认值 | 描述 |
|-----------|------|---------|-------------|
| `target_directory` | 字符串 | `null` | 合并后文件夹的目标目录（未指定则使用第一个源目录） |
| `merge_strategy` | 字符串 | `"UserDecision"` | 合并策略：SmallerToLarger, LargerToSmaller, UserDecision, TargetDirectory |
| `experimental_mode` | 布尔值 | `true` | 启用实验模式（模拟操作） |
| `enable_user_interaction` | 布尔值 | `true` | 启用人工决策 |
| `decision_timeout` | 整数 | `300` | 人工决策超时时间（秒） |
| `duplicate_handling` | 字符串 | `"UserDecision"` | 重复文件处理方式：Skip, Rename, Overwrite, UserDecision |
| `minimum_folder_size` | 整数 | `0` | 考虑合并的最小文件夹大小（字节） |
| `max_merge_depth` | 整数 | `3` | 递归合并的最大目录深度 |
| `enable_size_analysis` | 布尔值 | `true` | 合并前执行详细大小分析 |
| `backup_before_merge` | 布尔值 | `false` | 执行合并操作前创建备份 |

### 使用示例

#### 基础合并（用户决策）

```bash
# 交互式合并，用户引导策略选择
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/home/user/Downloads", "/home/user/Desktop", "/home/user/Documents/Temp"]' \
  --param target_directory="/home/user/Organized" \
  --param merge_strategy="UserDecision" \
  --param experimental_mode=true
```

#### 自动合并策略

```bash
# 使用“小合并到大”策略自动合并
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/data/dir1", "/data/dir2"]' \
  --param merge_strategy="SmallerToLarger" \
  --param duplicate_handling="Rename" \
  --param experimental_mode=false
```

#### 生产环境合并（带备份）

```bash
# 生产环境合并，创建备份并验证
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/enterprise/shared1", "/enterprise/shared2"]' \
  --param target_directory="/enterprise/consolidated" \
  --param merge_strategy="TargetDirectory" \
  --param backup_before_merge=true \
  --param enable_size_analysis=true \
  --param experimental_mode=false
```

### 合并策略

| 策略 | 描述 | 使用场景 |
|----------|-------------|----------|
| **SmallerToLarger** | 将较小文件夹合并到最大的一个 | 整合分散的文件 |
| **LargerToSmaller** | 将较大文件夹合并到最小的一个 | 保持现有结构 |
| **UserDecision** | 用户为每组决定策略 | 需要判断的复杂场景 |
| **TargetDirectory** | 将所有文件夹合并到指定目标 | 集中式组织 |

### 冲突解决选项

| 选项 | 描述 | 安全级别 |
|--------|-------------|--------------|
| **Skip** | 跳过冲突文件，保留原始文件 | 高 |
| **Rename** | 重命名冲突文件并添加后缀 | 高 |
| **Overwrite** | 用较新/较大的文件替换现有文件 | 中 |
| **UserDecision** | 每个冲突都询问用户 | 最高 |

## 交互式批处理流程

**文件路径：** `examples/templates/interactive-batch-processing-workflow.yaml`

### 用途

通用批量文件操作流程，具备人工监督、全面错误处理和灵活的操作类型。

### 流程步骤

1. **📁 扫描源项**：发现并过滤源目录中的项目  
2. **✅ 验证配置**：验证操作配置和目标路径  
3. **📊 分析项目**：创建批处理计划并检测潜在冲突  
4. **📋 审查计划**：显示处理计划、统计数据和估算  
5. **⚠️ 处理冲突**：通过用户决策或自动策略解决冲突  
6. **✅ 完成计划**：完成包含所有冲突解决方案的处理计划  
7. **📊 审查最终计划**：显示完整的最终处理计划  
8. **❓ 实验模式检查**：根据实验模式设置分支执行  
9. **👤 确认执行**：获取用户确认（若启用实验模式）  
10. **💾 创建备份**：在操作前创建备份（若启用）  
11. **⚡ 执行批次**：执行批处理操作，并行处理并跟踪进度  
12. **🔍 验证结果**：验证批处理操作成功完成  
13. **🚨 处理失败**：处理任何失败，提供用户决策和恢复选项  
14. **📋 生成报告**：创建综合批处理报告  
15. **🧹 清理**：清理临时文件并执行最终操作  
16. **📊 最终摘要**：生成包含完整统计信息的最终状态摘要  

### 参数

#### 必需参数

| 参数 | 类型 | 描述 |
|-----------|------|-------------|
| `source_directory` | 字符串 | 包含待处理文件/文件夹的目录 |
| `target_directory` | 字符串 | 处理后项目的目录 |

#### 可选参数

| 参数 | 类型 | 默认值 | 描述 |
|-----------|------|---------|-------------|
| `operation_type` | 字符串 | `"move"` | 操作类型：move, copy, classify, merge, custom |
| `operation_config` | 对象 | `{}` | 操作类型特定的配置 |
| `experimental_mode` | 布尔值 | `true` | 启用实验模式（模拟操作） |
| `enable_user_interaction` | 布尔值 | `true` | 在模糊场景中启用人工决策 |
| `decision_timeout` | 整数 | `300` | 人工决策超时时间（秒） |
| `batch_size` | 整数 | `20` | 每批次处理的项目数量 |
| `max_concurrent_batches` | 整数 | `3` | 最大并发批次数 |
| `conflict_resolution` | 字符串 | `"UserDecision"` | 冲突处理方式：Skip, Rename, Overwrite, UserDecision |
| `progress_reporting` | 布尔值 | `true` | 启用详细进度报告 |
| `create_backup` | 布尔值 | `false` | 操作前创建备份 |
| `filter_criteria` | 对象 | `{}` | 过滤待处理项目的条件 |

### 使用示例

#### 基础批量移动操作

```bash
# 批量移动文件并启用用户交互
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/data/incoming" \
  --param target_directory="/data/processed" \
  --param operation_type="move" \
  --param experimental_mode=true \
  --param batch_size=25
```

#### 批量复制（带冲突解决）

```bash
# 复制文件并自动解决冲突
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/source/documents" \
  --param target_directory="/backup/documents" \
  --param operation_type="copy" \
  --param conflict_resolution="Rename" \
  --param create_backup=false \
  --param experimental_mode=false
```

#### 自定义批处理（带过滤）

```bash
# 自定义操作并过滤文件
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/media/raw" \
  --param target_directory="/media/processed" \
  --param operation_type="custom" \
  --param operation_config='{"custom_tool":"media-processor","resize_images":true}' \
  --param filter_criteria='{"file_extensions":[".jpg",".png",".mp4"],"min_size_bytes":10240}' \
  --param batch_size=10 \
  --param max_concurrent_batches=2
```

### 操作类型

| 类型 | 描述 | 配置选项 |
|------|-------------|----------------------|
| **move** | 移动文件/文件夹 | `preserve_structure`, `verify_moves` |
| **copy** | 复制文件/文件夹 | `preserve_timestamps`, `verify_checksums` |
| **classify** | 分类和整理 | `classification_rules`, `confidence_threshold` |
| **merge** | 合并重复文件夹 | `merge_strategy`, `duplicate_handling` |
| **custom** | 自定义工具执行 | `custom_tool`, `custom_params` |

### 过滤条件选项

```json
{
  "include_hidden": false,
  "min_size_bytes": 1024,
  "max_size_bytes": 1073741824,
  "file_extensions": [".pdf", ".docx", ".txt"],
  "exclude_patterns": ["*.tmp", "*.lock", ".*"],
  "modified_after": "2024-01-01",
  "modified_before": "2024-12-31"
}
```

## 配置参考

### 全局配置

```yaml
# 全局工作流配置
global_config:
  variables:
    temp_directory: "/tmp/workflow-toolkit"
    max_memory_usage: "4GB"
    default_timeout: "30m"
  
  error_handling:
    continue_on_error: false
    save_state_on_error: true
    max_consecutive_errors: 5
  
  retry_policy:
    max_attempts: 3
    initial_delay: "5s"
    backoff_strategy: "exponential"
```

### 性能配置

```yaml
# 性能优化设置
performance_config:
  # 内存管理
  memory_limit: "4GB"
  streaming_mode: true
  cache_strategy: "LRU"
  
  # 并发控制
  max_concurrent_operations: 8
  thread_pool_size: "auto"
  async_io: true
  
  # 批处理优化
  adaptive_batch_sizing: true
  batch_size_min: 5
  batch_size_max: 100
  
  # I/O 优化
  buffer_size: "64KB"
  read_ahead: true
  write_behind: true
```

### 安全配置

```yaml
# 安全和保障设置
security_config:
  # 访问控制
  restrict_to_user_directories: true
  validate_path_traversal: true
  check_permissions: true
  
  # 操作安全
  mandatory_experimental_mode: false
  require_confirmation: true
  create_backups: true
  
  # 审计和合规
  audit_all_operations: true
  log_user_actions: true
  compliance_mode: false
```

## 使用示例

### 示例 1：个人桌面清理

整理混合文件类型的杂乱桌面：

```bash
# 为个人文件创建分类规则
cat > personal-rules.json << 'EOF'
{
  "categories": {
    "documents": {
      "keywords": [
        {"pattern": "doc", "weight": 1.0},
        {"pattern": "pdf", "weight": 1.0},
        {"pattern": "resume", "weight": 1.2}
      ],
      "target_directory": "Documents"
    },
    "images": {
      "keywords": [
        {"pattern": "photo", "weight": 1.0},
        {"pattern": "screenshot", "weight": 1.1},
        {"pattern": "image", "weight": 0.9}
      ],
      "target_directory": "Pictures"
    },
    "downloads": {
      "keywords": [
        {"pattern": "download", "weight": 1.0},
        {"pattern": "installer", "weight": 1.1}
      ],
      "target_directory": "Downloads/Organized"
    }
  }
}
EOF

# 运行分类工作流
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/Desktop" \
  --param output_directory="/home/user/Organized" \
  --param classification_rules="personal-rules.json" \
  --param experimental_mode=true \
  --param batch_size=5 \
  --param decision_timeout=180
```

### 示例 2：企业文档管理

处理具有合规要求的企业文档：

```bash
# 创建企业分类规则
cat > enterprise-rules.json << 'EOF'
{
  "categories": {
    "financial": {
      "keywords": [
        {"pattern": "financial", "weight": 1.0},
        {"pattern": "budget", "weight": 1.1},
        {"pattern": "invoice", "weight": 1.2}
      ],
      "target_directory": "Financial/{year}",
      "confidence_threshold": 0.9
    },
    "legal": {
      "keywords": [
        {"pattern": "contract", "weight": 1.2},
        {"pattern": "legal", "weight": 1.0},
        {"pattern": "agreement", "weight": 1.1}
      ],
      "target_directory": "Legal/{year}",
      "confidence_threshold": 0.95
    },
    "hr": {
      "keywords": [
        {"pattern": "employee", "weight": 1.0},
        {"pattern": "hr", "weight": 1.1},
        {"pattern": "personnel", "weight": 0.9}
      ],
      "target_directory": "HR/{department}",
      "confidence_threshold": 0.85
    }
  },
  "settings": {
    "minimum_score_threshold": 0.7,
    "enable_audit_logging": true,
    "require_approval": true
  }
}
EOF

# 使用企业设置运行
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/enterprise/incoming" \
  --param output_directory="/enterprise/classified" \
  --param classification_rules="enterprise-rules.json" \
  --param experimental_mode=true \
  --param enable_user_interaction=true \
  --param confidence_threshold=0.9 \
  --param decision_timeout=600
```

### 示例 3：媒体库整理

基于元数据整理大型媒体收藏：

```bash
# 为媒体文件运行批处理
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/media/unsorted" \
  --param target_directory="/media/organized" \
  --param operation_type="custom" \
  --param operation_config='{
    "custom_tool": "media-organizer",
    "extract_metadata": true,
    "organize_by_date": true,
    "create_thumbnails": true
  }' \
  --param filter_criteria='{
    "file_extensions": [".jpg", ".png", ".mp4", ".mov", ".avi"],
    "min_size_bytes": 10240,
    "exclude_patterns": ["*.tmp", ".*"]
  }' \
  --param batch_size=20 \
  --param max_concurrent_batches=4 \
  --param experimental_mode=true
```

### 示例 4：开发项目清理

整理开发项目和代码仓库：

```bash
# 创建开发专用规则
cat > dev-rules.json << 'EOF'
{
  "categories": {
    "active_projects": {
      "keywords": [
        {"pattern": "src", "weight": 1.2},
        {"pattern": "project", "weight": 1.0},
        {"pattern": "git", "weight": 1.1}
      ],
      "target_directory": "Development/Active/{language}",
      "conditions": {
        "has_git_repo": true,
        "recent_activity": "30d"
      }
    },
    "archived_projects": {
      "keywords": [
        {"pattern": "old", "weight": 1.0},
        {"pattern": "archive", "weight": 1.2},
        {"pattern": "backup", "weight": 1.1}
      ],
      "target_directory": "Development/Archive/{year}"
    },
    "documentation": {
      "keywords": [
        {"pattern": "doc", "weight": 1.0},
        {"pattern": "readme", "weight": 1.2},
        {"pattern": "wiki", "weight": 1.1}
      ],
      "target_directory": "Development/Documentation"
    }
  }
}
EOF

# 使用开发设置运行分类
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/developer/workspace" \
  --param output_directory="/home/developer/organized" \
  --param classification_rules="dev-rules.json" \
  --param experimental_mode=true \
  --param enable_user_interaction=true \
  --param batch_size=10
```

## 高级模式

### 模式 1：多阶段分类

组合多个分类阶段处理复杂场景：

```yaml
name: "multi_stage_classification"
description: "多阶段分类，包含预处理和细化"

steps:
  # 阶段 1：预处理文件夹名称（概念上）
  - name: "preprocess_text"
    tool: "text-processor"
    params:
      text: "${input.folder_name}"
      operations: ["NormalizeCase", "ConvertTraditional", "GeneratePinyin"]

  # 阶段 2：初始分类
  - name: "initial_classification"
    tool: "folder-classifier"
    params:
      folder_path: "${input.folder_path}"
      classification_rules: "${input.rules_file}"
      enable_user_interaction: false
      output_format: "Detailed"

  # 阶段 3：人工审查模糊案例
  - name: "human_review"
    tool: "human-decision"
    condition: "${nodes.initial_classification.status} == 'Ambiguous'"
    params:
      decision_type: "Classification"
      context:
        title: "模糊分类"
        description: "请为 ${input.folder_name} 选择正确的类别"
        metadata: "${nodes.initial_classification.metadata}"
      options: "${nodes.initial_classification.candidates}"

  # 阶段 4：执行操作
  - name: "execute_operations"
    tool: "file-mover"
    params:
      operations:
        - source: "${input.folder_path}"
          destination: "${nodes.human_review.selected_category}/${input.folder_name}"
          operation_type: "Move"
```

### 模式 2：条件处理

使用条件创建自适应工作流：

```yaml
name: "conditional_processing"
description: "基于文件夹特征的自适应处理"

steps:
  - name: "analyze_folders"
    tool: "folder-analyzer" # 分析工具占位符
    params:
      source_directory: "${input.source_directory}"

  # 不同处理大型文件夹
  - name: "process_large_folders"
    tool: "batch-processor"
    condition: "${nodes.analyze_folders.large_folder_count} > 0"
    params:
      tool_name: "large-folder-handler"
      batch_items: "${nodes.analyze_folders.large_folders}"
      max_concurrency: 2
      processing_mode: "Sequential"

  # 小型文件夹批量处理
  - name: "process_small_folders"
    tool: "batch-processor"
    condition: "${nodes.analyze_folders.small_folder_count} > 0"
    params:
      tool_name: "small-folder-handler"
      batch_items: "${nodes.analyze_folders.small_folders}"
      max_concurrency: 8
      processing_mode: "Parallel"
```

### 模式 3：错误恢复与重试

实现带重试逻辑的健壮错误处理：

```yaml
name: "robust_processing"
description: "带全面错误处理的处理流程"

steps:
  - name: "initial_processing"
    tool: "batch-processor"
    params:
      tool_name: "file-processor"
      batch_items: "${input.items}"
      continue_on_error: true
      retry_failed_items: false

  # 通过用户决策处理失败
  - name: "handle_failures"
    tool: "human-decision"
    condition: "${nodes.initial_processing.error_summary.total_errors} > 0"
    params:
      decision_type: "Custom"
      context:
        title: "检测到处理失败"
        description: "某些操作失败。请选择恢复操作。"
      options:
        - id: "retry"
          label: "重试失败操作"
        - id: "skip"
          label: "跳过失败操作"
        - id: "manual"
          label: "需要手动审查"

  # 使用不同设置重试失败操作
  - name: "retry_operations"
    tool: "batch-processor"
    condition: "${nodes.handle_failures.selected_option} == 'retry'"
    params:
      tool_name: "file-processor"
      batch_items: "${nodes.initial_processing.error_summary.failed_items}"
      max_concurrency: 1
      retry_failed_items: true
      max_retries: 3
```

## 故障排除

### 常见问题与解决方案

#### 1. 分类规则无法加载

**症状：**
- 工作流因"无效规则"错误而失败
- 分类返回无结果

**解决方案：**
```bash
# 验证 JSON 语法
jq '.' classification-rules.json

# 检查文件权限
ls -la classification-rules.json

# 使用最小规则测试
echo '{"categories":{"test":{"keywords":[{"pattern":"test","weight":1.0}],"target_directory":"Test"}}}' > test-rules.json
```

#### 2. 人工决策超时

**症状：**
- 工作流因"决策超时"错误停止
- 未出现用户交互提示

**解决方案：**
```yaml
# 为复杂决策增加超时时间
decision_timeout: 600  # 10分钟

# 启用批量决策提高效率
batch_decision_mode: true
group_similar_decisions: true

# 提供默认选择
default_choice: 0
enable_auto_fallback: true
```

#### 3. 大数据集性能问题

**症状：**
- 处理速度慢
- 内存使用率高
- 系统无响应

**解决方案：**
```yaml
# 优化批次大小
batch_size: 10          # 在内存受限系统上减少
max_concurrent_batches: 2

# 启用流式模式
streaming_mode: true
memory_limit: "2GB"

# 使用渐进式加载
lazy_loading: true
preload_metadata: false
```

#### 4. 文件操作失败

**症状：**
- "权限被拒绝"错误
- "磁盘空间不足"错误
- 操作静默失败

**解决方案：**
```bash
# 检查磁盘空间
df -h /target/directory

# 验证权限
ls -ld /target/directory
chmod 755 /target/directory

# 测试写入权限
touch /target/directory/test_file && rm /target/directory/test_file

# 检查文件锁
lsof /path/to/locked/file
```

### 调试模式配置

启用全面调试：

```yaml
debug_config:
  # 日志配置
  log_level: "debug"
  log_file: "workflow_debug.log"
  include_stack_traces: true
  
  # 跟踪配置
  enable_operation_tracing: true
  trace_decision_flow: true
  capture_intermediate_results: true
  
  # 性能监控
  enable_performance_profiling: true
  memory_usage_tracking: true
  bottleneck_detection: true
```

### 性能监控

监控工作流性能：

```bash
# 启用性能分析
RUST_LOG=debug cargo run -- workflow execute template.yaml \
  --param enable_profiling=true \
  --param profile_output="performance_profile.json"

# 监控系统资源
top -p $(pgrep -f workflow-toolkit)
htop -p $(pgrep -f workflow-toolkit)

# 检查 I/O 性能
iotop -p $(pgrep -f workflow-toolkit)
```

### 恢复程序

#### 工作流状态恢复

```bash
# 错误时保存工作流状态
cargo run -- workflow execute template.yaml \
  --param save_state_on_error=true \
  --param state_file="workflow_state.json"

# 从保存状态恢复
cargo run -- workflow resume workflow_state.json

# 回滚到先前检查点
cargo run -- workflow rollback --checkpoint checkpoint_001
```

#### 数据恢复

```bash
# 操作前创建备份
cargo run -- workflow execute template.yaml \
  --param create_backup=true \
  --param backup_directory="/backups"

# 如需从备份恢复
cp -r /backups/original_data /restored_data
```

---

*本指南持续更新新特性和改进。欲了解最新信息，请参阅官方文档和社区资源。*