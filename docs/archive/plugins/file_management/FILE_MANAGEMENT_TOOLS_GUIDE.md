# 文件管理工具用户指南

本综合指南涵盖工作流工具包系统的文件管理工具插件，通过可重用的工作流组件提供智能文件和文件夹管理能力。

## 目录

1. [概述](#概述)
2. [快速开始](#快速开始)
3. [工具参考](#工具参考)
4. [工作流模板](#工作流模板)
5. [配置指南](#配置指南)
6. [人工决策集成](#人工决策集成)
7. [最佳实践](#最佳实践)
8. [故障排除](#故障排除)
9. [高级用法](#高级用法)

## 概述

文件管理工具插件提供了一套全面的工具，用于智能文件和文件夹组织，包括：

- **智能分类**：基于可配置规则对文件夹进行分类，支持中文文本
- **批处理**：并行处理多个文件/文件夹，带进度跟踪
- **文件夹合并**：智能合并重复文件夹，带冲突解决
- **人工决策集成**：针对模糊场景的交互式决策
- **实验模式**：执行前预览操作并生成详细报告
- **文本处理**：高级文本规范化和中文/拼音转换
- **模式匹配**：使用AC自动机进行高效多模式字符串匹配

### 主要特性

✅ **工作流集成**：与工作流工具包的插件系统无缝集成  
✅ **人机协作**：针对复杂场景的交互式决策  
✅ **实验模式**：执行前安全预览和确认  
✅ **中文文本支持**：完整的Unicode支持，带拼音转换  
✅ **批处理操作**：可配置并发度的并行处理  
✅ **冲突解决**：多种处理文件冲突的策略  
✅ **进度跟踪**：实时进度报告和性能指标  
✅ **错误恢复**：全面的错误处理，带回滚能力  

### 架构与实现

文件管理工具使用Rust实现，并在`workflow-toolkit`生态系统中注册为插件。工作流YAML中的每个工具对应一个特定的Rust执行器：

- **YAML工具映射**：工作流节点使用`tool_name`在`FileManagementToolRegistry`中查找Rust执行器。
- **安全执行**：所有操作都支持`experimental_mode`进行安全试运行。
- **详细映射**：有关YAML到Rust映射的完整列表，请参阅[API参考 - 工作流集成](FILE_MANAGEMENT_API_REFERENCE.md#workflow-integration--execution-mechanism)。

## 快速开始

### 安装

文件管理工具作为插件包含在工作流工具包系统中：

```bash
# 构建包含文件管理工具的项目
cargo build --features file-management

# 验证插件可用
cargo run -- plugin list | grep file-management
```

### 基本用法

#### 1. 简单文件夹分类

```bash
# 创建基本分类规则
cat > basic-rules.json << 'EOF'
{
  "categories": {
    "documents": {
      "keywords": [
        {"pattern": "doc", "weight": 1.0},
        {"pattern": "pdf", "weight": 1.0}
      ],
      "target_directory": "Documents"
    },
    "images": {
      "keywords": [
        {"pattern": "photo", "weight": 1.0},
        {"pattern": "image", "weight": 1.0}
      ],
      "target_directory": "Images"
    }
  }
}
EOF

# 运行分类工作流
cargo run -- workflow execute workflows/templates/interactive/classification.yaml \
  --param source_directory="/path/to/messy/folders" \
  --param output_directory="/path/to/organized" \
  --param classification_rules="basic-rules.json" \
  --param experimental_mode=true
```

#### 2. 文件夹合并

```bash
# 跨位置合并重复文件夹
cargo run -- workflow execute workflows/templates/interactive/merge.yaml \
  --param source_directories='["/home/user/Downloads", "/home/user/Desktop"]' \
  --param target_directory="/home/user/Organized" \
  --param merge_strategy="UserDecision" \
  --param experimental_mode=true
```

#### 3. 批处理文件

```bash
# 批量处理文件
cargo run -- workflow execute workflows/templates/interactive/batch-processing.yaml \
  --param source_directory="/data/incoming" \
  --param target_directory="/data/processed" \
  --param operation_type="move" \
  --param batch_size=20 \
  --param experimental_mode=true
```

## 工具参考

### 核心工具

#### 1. 文件夹分类器 (`folder-classifier`)

*注意：在高级工作流中，分类过程通常被分解为细粒度步骤（加载规则、构建自动机、并行匹配）以获得更好的性能和控制。有关这些底层工具的详细信息，请参阅[API参考](FILE_MANAGEMENT_API_REFERENCE.md#granular-classification-tools-api)。*

基于可配置规则智能地对文件夹进行分类。

**参数：**
- `folder_path` (字符串)：要分类的文件夹路径
- `classification_rules` (对象/字符串)：分类规则（JSON对象或文件路径）
- `enable_user_interaction` (布尔值)：启用人工决策
- `experimental_mode` (布尔值)：以模拟模式运行
- `confidence_threshold` (数字)：自动分类的最低置信度

**返回：**
- `status`：分类状态（classified, unclassified, pending, error）
- `category`：分配的类别（如果已分类）
- `candidates`：所有类别候选及其分数
- `score`：分类的置信度分数
- `processing_time_ms`：分类所用时间

**示例：**
```yaml
- name: "classify_folder"
  tool: "folder-classifier"
  params:
    folder_path: "/path/to/folder"
    classification_rules: "rules.json"
    confidence_threshold: 0.8
    experimental_mode: true
```

#### 2. 文件移动器 (`file-mover`)

安全的文件和文件夹操作，带冲突解决。

**参数：**
- `operations` (数组)：移动操作列表
- `conflict_resolution` (字符串)：如何处理冲突（Skip, Overwrite, Rename, Fail）
- `check_disk_space` (布尔值)：操作前验证磁盘空间
- `create_directories` (布尔值)：按需创建目标目录

**返回：**
- `operations_completed`：成功操作的数量
- `operations_failed`：失败操作的数量
- `total_bytes_moved`：传输的总数据量
- `duration_ms`：操作持续时间
- `errors`：详细的错误信息

**示例：**
```yaml
- name: "move_files"
  tool: "file-mover"
  params:
    operations:
      - source: "/source/file.txt"
        destination: "/target/file.txt"
        operation_type: "Move"
    conflict_resolution: "Rename"
    check_disk_space: true
```

#### 3. 文件夹合并器 (`folder-merger`)

智能合并文件夹，带重复处理。

**参数：**
- `source_directories` (数组)：要扫描可合并文件夹的目录
- `target_directory` (字符串)：合并文件夹的目标目录
- `merge_strategy` (字符串)：合并策略（SmallerToLarger, LargerToSmaller, UserDecision, TargetDirectory）
- `duplicate_handling` (字符串)：如何处理重复项
- `minimum_folder_size` (数字)：要考虑的最小文件夹大小

**返回：**
- `merge_operations`：计划的合并操作列表
- `space_savings`：估计的空间节省
- `conflicts_detected`：发现的冲突数量
- `merge_summary`：合并分析摘要

**示例：**
```yaml
- name: "merge_folders"
  tool: "folder-merger"
  params:
    source_directories: ["/dir1", "/dir2"]
    merge_strategy: "SmallerToLarger"
    duplicate_handling: "Rename"
```

#### 4. 批处理器 (`batch-processor`)

在工作流中并行处理多个项目。

**参数：**
- `source_directory` (字符串)：包含要处理项目的目录
- `target_directory` (字符串)：处理后项目的目标目录
- `operation_type` (字符串)：操作类型（move, copy, classify, merge, custom）
- `batch_size` (数字)：每批项目数
- `max_concurrent_batches` (数字)：最大并发批次数

**返回：**
- `batches_completed`：完成的批次数
- `total_items_processed`：处理的总项目数
- `processing_time`：总处理时间
- `throughput`：每秒处理的项目数
- `error_summary`：任何错误的摘要

**示例：**
```yaml
- name: "batch_process"
  tool: "batch-processor"
  params:
    source_directory: "/data/input"
    target_directory: "/data/output"
    operation_type: "move"
    batch_size: 25
    max_concurrent_batches: 4
```

#### 5. 人工决策 (`human-decision`)

针对模糊场景的交互式决策。支持单项目和批量决策模式。

**参数：**
- `decision_type` (字符串)：决策类型（Classification, FileConflict, MergeStrategy, Custom）
- `context` (对象)：决策上下文，包含标题、描述和元数据
- `options` (数组)：可用的决策选项（单模式）
- `items` (数组)：要决策的项目列表（批量模式）
- `timeout_seconds` (数字)：决策超时
- `default_choice` (数字)：超时时的默认选项

**返回：**
- `selected_option`：所选选项的ID（单模式）
- `decisions`：决策结果列表（批量模式）
- `decision_time_ms`：决策所用时间
- `was_timeout`：决策是否超时
- `user_input`：额外的用户输入（如果有）

**示例（单模式）：**
```yaml
- name: "user_decision"
  tool: "human-decision"
  params:
    decision_type: "Classification"
    context:
      title: "文件夹分类决策"
      description: "发现多个类别"
    options:
      - id: "documents"
        label: "文档"
        recommended: true
      - id: "projects"
        label: "项目"
    timeout_seconds: 300
```

### 实用工具

#### 6. 文本处理器 (`text-processor`)

高级文本处理，包括中文和拼音转换。

**参数：**
- `text` (字符串)：要处理的文本
- `operations` (数组)：要应用的处理操作
- `chinese_processing` (对象)：中文特定的处理选项

**返回：**
- `original`：原始文本
- `processed`：处理后的文本
- `pinyin_variants`：生成的拼音变体
- `combinations`：关键词组合
- `metadata`：处理元数据

#### 7. AC匹配器 (`ac-matcher`)

使用AC自动机进行高效多模式字符串匹配。

**参数：**
- `text` (字符串)：要搜索的文本
- `patterns` (数组)：要搜索的模式
- `case_sensitive` (布尔值)：区分大小写匹配
- `find_overlapping` (布尔值)：查找重叠匹配

**返回：**
- `matches`：找到的模式匹配
- `total_matches`：匹配总数
- `categories_found`：有匹配的类别

## 工作流模板

文件管理工具包括三个综合工作流模板：

### 1. 交互式分类工作流

**文件：** `workflows/templates/interactive/classification.yaml`

**目的：** 带人工决策支持和实验模式的智能文件夹分类。

**主要特性：**
- 可配置大小的批处理
- 针对模糊分类的人工决策
- 带确认步骤的实验模式
- 中文文本处理支持
- 自动清理空目录

**用法：**
```bash
cargo run -- workflow execute workflows/templates/interactive/classification.yaml \
  --param source_directory="/path/to/folders" \
  --param output_directory="/path/to/organized" \
  --param classification_rules="rules.json"
```

### 2. 交互式合并工作流

**文件：** `workflows/templates/interactive/merge.yaml`

**目的：** 带用户决策和多种策略的智能文件夹合并。

**主要特性：**
- 多种合并策略（SmallerToLarger, LargerToSmaller, UserDecision, TargetDirectory）
- 针对合并策略和冲突的人工决策
- 带详细预览的实验模式
- 操作前创建备份
- 全面的验证和报告

**用法：**
```bash
cargo run -- workflow execute workflows/templates/interactive/merge.yaml \
  --param source_directories='["/dir1", "/dir2"]' \
  --param merge_strategy="UserDecision"
```

### 3. 交互式批处理工作流

**文件：** `workflows/templates/interactive/batch-processing.yaml`

**目的：** 带人工监督的通用批处理文件操作工作流。

**主要特性：**
- 任何操作类型的通用批处理
- 可配置的批次大小和并发度
- 针对冲突的人工决策
- 全面的错误处理和恢复
- 灵活的过滤条件

**用法：**
```bash
cargo run -- workflow execute workflows/templates/interactive/batch-processing.yaml \
  --param source_directory="/data/input" \
  --param target_directory="/data/output" \
  --param operation_type="move"
```

## 配置指南

### 分类规则格式

分类规则定义如何对文件夹进行分类：

```json
{
  "version": "1.0",
  "categories": {
    "documents": {
      "description": "文档和文本文件",
      "keywords": [
        {"pattern": "doc", "weight": 1.0, "case_sensitive": false},
        {"pattern": "pdf", "weight": 1.0, "case_sensitive": false}
      ],
      "target_directory": "Documents",
      "confidence_threshold": 0.7
    }
  },
  "settings": {
    "minimum_score_threshold": 0.5,
    "case_sensitive": false,
    "enable_chinese_processing": false
  }
}
```

### 中文文本处理

为多语言环境启用中文文本处理：

```json
{
  "settings": {
    "enable_chinese_processing": true,
    "enable_pinyin_conversion": true,
    "normalize_traditional_chinese": true
  },
  "categories": {
    "documents": {
      "keywords": [
        {"pattern": "文档", "weight": 1.0, "language": "zh"},
        {"pattern": "document", "weight": 1.0, "language": "en"},
        {"pattern": "wendang", "weight": 0.9, "type": "pinyin"}
      ],
      "target_directory": "文档_Documents"
    }
  }
}
```

### 性能配置

针对不同场景优化性能：

```yaml
# 对于大数据集
performance_config:
  batch_size: 100
  max_concurrent_batches: 8
  streaming_mode: true
  memory_limit: "4GB"

# 对于交互式使用
interactive_config:
  batch_size: 10
  enable_user_interaction: true
  decision_timeout: 300
  detailed_progress: true
```

## 人工决策集成

### 决策类型

系统支持几种类型的决策：

#### 1. 分类决策

当多个类别具有相似的置信度分数时：
- 显示所有候选类别及其分数
- 显示文件夹上下文和分析
- 允许用户选择正确的类别
- 从决策中学习以改进未来的分类

#### 2. 冲突解决

当检测到文件冲突时：
- 显示冲突文件的详细信息（大小、日期、校验和）
- 提供解决选项（保留较新的、重命名、跳过）
- 允许对类似冲突进行批量解决
- 保持操作安全性和数据完整性

#### 3. 合并策略选择

当合并重复文件夹时：
- 显示文件夹大小和位置
- 提供合并策略选项
- 显示估计的空间节省
- 允许每组策略决策

### 决策上下文

为知情决策提供丰富的上下文：

```yaml
decision_context:
  title: "文件夹分类决策"
  description: "文件夹'Project Documents'发现多个类别"
  metadata:
    folder_name: "Project Documents"
    folder_size: "2.3 GB"
    file_count: 156
    confidence_scores:
      documents: 0.75
      projects: 0.72
```

### 超时和升级

为不同场景配置超时和升级：

```yaml
timeout_config:
  classification: 120      # 2分钟
  conflict_resolution: 300 # 5分钟
  merge_strategy: 600      # 10分钟
  
escalation_config:
  escalate_on_timeout: true
  escalation_hierarchy:
    - "team_lead"
    - "supervisor"
```

## 最佳实践

### 1. 从实验模式开始

始终从实验模式开始预览操作：

```yaml
experimental_mode: true
enable_user_interaction: true
```

### 2. 使用适当的批次大小

根据您的系统和用例选择批次大小：

```yaml
# 对于交互式使用
batch_size: 5-10

# 对于自动处理
batch_size: 25-50

# 对于高性能系统
batch_size: 100+
```

### 3. 配置合理的超时

根据决策复杂度设置超时：

```yaml
# 简单决策
decision_timeout: 60

# 复杂决策
decision_timeout: 300

# 关键决策
decision_timeout: 600
```

### 4. 测试分类规则

首先在小数据集上测试规则：

```bash
# 使用小子集测试
cargo run -- workflow execute template.yaml \
  --param source_directory="/test/small_dataset" \
  --param experimental_mode=true
```

### 5. 监控性能

启用性能监控以进行优化：

```yaml
performance_monitoring:
  enable_metrics: true
  track_processing_time: true
  monitor_memory_usage: true
  report_bottlenecks: true
```

### 6. 备份重要数据

操作前始终创建备份：

```yaml
backup_config:
  create_backup: true
  backup_directory: "/backups"
  verify_backup: true
```

## 故障排除

### 常见问题

#### 1. 分类规则未加载

**症状：** 分类失败，显示"Invalid rules"错误

**解决方案：**
- 使用 `jq '.' rules.json` 验证JSON语法
- 检查文件权限和可访问性
- 根据模式验证规则结构
- 确保所有必填字段都存在

#### 2. 人工决策超时

**症状：** 工作流停止，显示"Decision timeout"错误

**解决方案：**
- 增加 `decision_timeout` 参数
- 检查终端/UI响应性
- 验证用户交互是否启用
- 考虑批量决策模式

#### 3. 文件操作失败

**症状：** "Permission denied"或"Disk full"错误

**解决方案：**
- 检查磁盘空间：`df -h /target/directory`
- 验证权限：`ls -ld /target/directory`
- 查看冲突解决设置
- 确保目标目录存在

#### 4. 性能问题

**症状：** 处理缓慢，内存使用高

**解决方案：**
- 减少批次大小
- 限制并发操作
- 启用流式模式
- 监控系统资源

#### 5. 中文文本处理错误

**症状：** 拼音转换不正确，编码问题

**解决方案：**
- 验证终端中的Unicode支持
- 检查输入文本编码（UTF-8）
- 启用正确的中文处理设置
- 使用简化示例测试

### 调试模式

启用调试日志进行故障排除：

```bash
# 启用调试日志
RUST_LOG=debug cargo run -- workflow execute template.yaml

# 启用跟踪日志进行详细分析
RUST_LOG=trace cargo run -- workflow execute template.yaml

# 记录到文件进行分析
RUST_LOG=debug cargo run -- workflow execute template.yaml 2> debug.log
```

### 性能分析

分析性能以识别瓶颈：

```bash
# 启用性能分析
cargo run -- workflow execute template.yaml \
  --param enable_profiling=true \
  --param profile_output="profile.json"
```

## 高级用法

### 自定义分类规则

创建复杂的分类规则：

```json
{
  "categories": {
    "work_projects": {
      "keywords": [
        {"pattern": "project", "weight": 1.0},
        {"pattern": "work", "weight": 0.9},
        {"pattern": "client", "weight": 1.1}
      ],
      "target_directory": "Work/Projects/{year}",
      "conditions": {
        "min_folder_size": 1048576,
        "exclude_patterns": ["temp", "cache"]
      },
      "metadata_extraction": true
    }
  }
}
```

### 工作流组合

在复杂工作流中组合多个工具：

```yaml
steps:
  - name: "preprocess_text"
    tool: "text-processor"
    params:
      operations: ["NormalizeCase", "ConvertTraditional"]
  
  - name: "classify_folders"
    tool: "folder-classifier"
    params:
      preprocessed_text: "{{ preprocess_text.processed }}"
  
  - name: "human_review"
    tool: "human-decision"
    condition: "{{ classify_folders.ambiguous_count > 0 }}"
  
  - name: "execute_operations"
    tool: "file-mover"
    params:
      operations: "{{ human_review.final_operations }}"
```

### 与外部系统集成

与数据库和外部API集成：

```yaml
integration_config:
  database:
    track_operations: true
    store_metadata: true
    connection: "postgresql://localhost/filemanagement"
  
  webhooks:
    notify_completion: true
    webhook_url: "https://api.example.com/file-operations"
  
  monitoring:
    metrics_endpoint: "http://localhost:8080/metrics"
    alert_on_errors: true
```

### 企业特性

配置企业级特性：

```yaml
enterprise_config:
  compliance:
    audit_all_operations: true
    data_classification: true
    retention_policies: true
  
  security:
    encrypt_sensitive_data: true
    access_control: true
    dual_approval: true
  
  governance:
    approval_workflows: true
    escalation_hierarchy: true
    compliance_reporting: true
```

## 获取帮助

### 文档资源

- **API参考**：所有工具的详细API文档
- **配置指南**：全面的配置选项
- **使用示例**：常见场景的工作示例
- **最佳实践**：推荐的模式和实践

### 社区支持

- **GitHub Issues**：报告错误和请求功能
- **Discussions**：提问和分享经验
- **Examples Repository**：社区贡献的示例
- **Wiki**：社区维护的文档

### 专业支持

- **企业支持**：适用于企业部署
- **定制开发**：针对特定需求的定制解决方案
- **培训**：研讨会和培训课程
- **咨询**：架构和实施指导

---

*本指南持续更新。有关最新信息和额外资源，请参阅各个文档和社区资源。*
