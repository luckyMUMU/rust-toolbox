# 用户使用手册

## 概述

欢迎使用工作流工具包！本手册将帮助您快速上手，学会创建和管理工作流，掌握常见使用场景，并解决可能遇到的问题。

## 快速开始指南

### 1. 安装和配置

#### 系统要求

- 操作系统: Linux, macOS, Windows
- 内存: 最少2GB，推荐4GB+
- 磁盘空间: 至少1GB可用空间
- 网络: 用于下载插件和依赖

#### 安装步骤

```bash
# 1. 下载并解压工作流工具包
wget https://github.com/example/workflow-toolkit/releases/latest/download/workflow-toolkit.tar.gz
tar -xzf workflow-toolkit.tar.gz
cd workflow-toolkit

# 2. 运行安装脚本
./install.sh

# 3. 验证安装
workflow-toolkit --version
```

#### 初始配置

```bash
# 1. 创建配置目录
mkdir -p ~/.workflow-toolkit

# 2. 复制默认配置
cp config/default.toml ~/.workflow-toolkit/config.toml

# 3. 编辑配置文件（可选）
nano ~/.workflow-toolkit/config.toml
```

### 2. 第一个工作流

让我们创建一个简单的"Hello World"工作流来熟悉基本操作。

#### 创建工作流定义文件

创建文件 `hello-world.yaml`:

```yaml
name: "hello-world"
version: "1.0.0"
description: "我的第一个工作流"

nodes:
  - id: "start"
    type: "start"
    
  - id: "say_hello"
    type: "tool"
    tool_name: "echo"
    parameters:
      message: "Hello, World!"
      
  - id: "get_time"
    type: "tool"
    tool_name: "datetime"
    parameters:
      format: "ISO8601"
      
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "say_hello"
  - from: "say_hello"
    to: "get_time"
  - from: "get_time"
    to: "end"
```

#### 创建和执行工作流

```bash
# 1. 创建工作流
workflow-toolkit workflow create hello-world.yaml

# 2. 列出工作流
workflow-toolkit workflow list

# 3. 执行工作流
workflow-toolkit workflow execute hello-world

# 4. 查看执行状态
workflow-toolkit workflow status <execution-id>
```

### 3. 使用CLI界面

#### 基本命令结构

```bash
workflow-toolkit <命令组> <子命令> [选项] [参数]
```

#### 常用命令

```bash
# 查看帮助
workflow-toolkit --help
workflow-toolkit workflow --help

# 查看版本
workflow-toolkit --version

# 列出所有工作流
workflow-toolkit workflow list

# 列出所有工具
workflow-toolkit tool list

# 查看配置
workflow-toolkit config show
```

### 4. 使用TUI界面

启动交互式终端界面：

```bash
workflow-toolkit tui
```

界面操作：
- `Tab`: 切换面板
- `↑/↓`: 选择项目
- `Enter`: 确认选择
- `Esc`: 返回上级
- `q`: 退出程序

## 工作流定义教程

### 1. 工作流基本结构

每个工作流定义包含以下主要部分：

```yaml
# 基本信息
name: "工作流名称"
version: "版本号"
description: "描述信息"

# 元数据（可选）
metadata:
  author: "作者"
  tags: ["标签1", "标签2"]

# 全局配置（可选）
global_config:
  timeout: "30m"
  variables:
    var1: "value1"

# 节点定义
nodes:
  - id: "节点ID"
    type: "节点类型"
    # 其他节点配置

# 连接定义
edges:
  - from: "源节点ID"
    to: "目标节点ID"
```

### 2. 节点类型详解

#### Start节点（开始节点）

```yaml
- id: "start"
  type: "start"
```

#### End节点（结束节点）

```yaml
- id: "end"
  type: "end"
```

#### Tool节点（工具节点）

```yaml
- id: "process_data"
  type: "tool"
  tool_name: "data_processor"
  parameters:
    input_file: "data.csv"
    operation: "clean"
  timeout: "10m"
  retry_policy:
    max_attempts: 3
    delay: "5s"
```

#### Condition节点（条件节点）

```yaml
- id: "check_result"
  type: "condition"
  condition: "${process_data.success} == true"
```

#### Parallel节点（并行节点）

```yaml
- id: "parallel_tasks"
  type: "parallel"
  # 并行执行多个后续节点
```

#### Loop节点（循环节点）

```yaml
- id: "process_files"
  type: "loop"
  condition: "${file_index} < ${total_files}"
  max_iterations: 100
```

### 3. 参数和变量

#### 全局变量

```yaml
global_config:
  variables:
    input_dir: "/data/input"
    output_dir: "/data/output"
    batch_size: 100
```

#### 环境变量

```yaml
parameters:
  api_key: "${env.API_KEY}"
  database_url: "${env.DATABASE_URL}"
```

#### 节点输出引用

```yaml
parameters:
  input_data: "${previous_node.output}"
  processed_count: "${data_processor.metadata.count}"
```

#### 条件表达式

```yaml
condition: "${data_quality.score} > 0.8 && ${record_count} > 0"
```

### 4. 错误处理和重试

#### 重试策略

```yaml
retry_policy:
  max_attempts: 3
  delay: "5s"
  backoff: "exponential"  # linear, exponential, fixed
  max_delay: "60s"
```

#### 错误处理

```yaml
error_handling:
  on_failure: "continue"  # stop, continue, retry
  fallback_node: "error_handler"
```

### 5. 高级特性

#### 条件分支

```yaml
nodes:
  - id: "check_file_size"
    type: "condition"
    condition: "${file_size} > 1000000"
    
edges:
  - from: "check_file_size"
    to: "large_file_processor"
    condition: "true"
    
  - from: "check_file_size"
    to: "small_file_processor"
    condition: "false"
```

#### 循环处理

```yaml
nodes:
  - id: "process_batch"
    type: "loop"
    condition: "${batch_index} < ${total_batches}"
    max_iterations: 50
    
  - id: "process_single_batch"
    type: "tool"
    tool_name: "batch_processor"
    parameters:
      batch_data: "${batches[${batch_index}]}"
```

#### 并行执行

```yaml
nodes:
  - id: "parallel_start"
    type: "parallel"
    
  - id: "task_a"
    type: "tool"
    tool_name: "processor_a"
    
  - id: "task_b"
    type: "tool"
    tool_name: "processor_b"
    
  - id: "task_c"
    type: "tool"
    tool_name: "processor_c"
    
  - id: "merge_results"
    type: "tool"
    tool_name: "result_merger"
    parameters:
      results: ["${task_a.output}", "${task_b.output}", "${task_c.output}"]

edges:
  - from: "parallel_start"
    to: "task_a"
  - from: "parallel_start"
    to: "task_b"
  - from: "parallel_start"
    to: "task_c"
  - from: "task_a"
    to: "merge_results"
  - from: "task_b"
    to: "merge_results"
  - from: "task_c"
    to: "merge_results"
```

## 常见使用场景示例

### 1. 数据处理管道

这是一个典型的ETL（提取、转换、加载）数据处理工作流。

```yaml
name: "etl-pipeline"
version: "1.0.0"
description: "数据ETL处理管道"

global_config:
  variables:
    source_db: "postgresql://source-db:5432/data"
    target_db: "postgresql://target-db:5432/warehouse"
    batch_size: 1000

nodes:
  - id: "start"
    type: "start"
    
  - id: "extract_data"
    type: "tool"
    tool_name: "database_extractor"
    parameters:
      connection: "${source_db}"
      query: "SELECT * FROM transactions WHERE created_at >= '${start_date}'"
      batch_size: "${batch_size}"
    timeout: "30m"
    
  - id: "validate_data"
    type: "tool"
    tool_name: "data_validator"
    parameters:
      data: "${extract_data.output}"
      schema_file: "schemas/transaction_schema.json"
      
  - id: "check_validation"
    type: "condition"
    condition: "${validate_data.valid_records} > 0"
    
  - id: "transform_data"
    type: "tool"
    tool_name: "data_transformer"
    parameters:
      input: "${validate_data.valid_records}"
      transformations:
        - field: "amount"
          operation: "normalize_currency"
        - field: "timestamp"
          operation: "convert_timezone"
          timezone: "UTC"
          
  - id: "load_data"
    type: "tool"
    tool_name: "database_loader"
    parameters:
      connection: "${target_db}"
      table: "processed_transactions"
      data: "${transform_data.output}"
      mode: "upsert"
      
  - id: "generate_report"
    type: "tool"
    tool_name: "report_generator"
    parameters:
      template: "etl_summary"
      data:
        extracted_count: "${extract_data.record_count}"
        valid_count: "${validate_data.valid_records}"
        loaded_count: "${load_data.inserted_count}"
        
  - id: "send_notification"
    type: "tool"
    tool_name: "email_sender"
    parameters:
      to: ["data-team@company.com"]
      subject: "ETL Pipeline Completed"
      body: "${generate_report.output}"
      
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "extract_data"
  - from: "extract_data"
    to: "validate_data"
  - from: "validate_data"
    to: "check_validation"
  - from: "check_validation"
    to: "transform_data"
    condition: "true"
  - from: "transform_data"
    to: "load_data"
  - from: "load_data"
    to: "generate_report"
  - from: "generate_report"
    to: "send_notification"
  - from: "send_notification"
    to: "end"
  - from: "check_validation"
    to: "end"
    condition: "false"
```

### 2. API集成工作流

自动化API数据同步和处理。

```yaml
name: "api-sync-workflow"
version: "1.0.0"
description: "API数据同步工作流"

global_config:
  variables:
    api_base_url: "https://api.example.com"
    api_key: "${env.API_KEY}"
    sync_interval: "1h"

nodes:
  - id: "start"
    type: "start"
    
  - id: "fetch_users"
    type: "tool"
    tool_name: "http_client"
    parameters:
      method: "GET"
      url: "${api_base_url}/users"
      headers:
        Authorization: "Bearer ${api_key}"
        Content-Type: "application/json"
      timeout: "30s"
      
  - id: "fetch_orders"
    type: "tool"
    tool_name: "http_client"
    parameters:
      method: "GET"
      url: "${api_base_url}/orders"
      headers:
        Authorization: "Bearer ${api_key}"
      query_params:
        since: "${last_sync_time}"
        limit: "100"
        
  - id: "merge_data"
    type: "tool"
    tool_name: "data_merger"
    parameters:
      users: "${fetch_users.response.body}"
      orders: "${fetch_orders.response.body}"
      join_key: "user_id"
      
  - id: "transform_records"
    type: "tool"
    tool_name: "json_transformer"
    parameters:
      input: "${merge_data.output}"
      transformations:
        - field: "created_at"
          operation: "parse_iso_date"
        - field: "total_amount"
          operation: "to_decimal"
          precision: 2
          
  - id: "save_to_database"
    type: "tool"
    tool_name: "database_writer"
    parameters:
      connection: "postgresql://localhost/analytics"
      table: "user_orders"
      data: "${transform_records.output}"
      conflict_resolution: "update"
      
  - id: "update_sync_timestamp"
    type: "tool"
    tool_name: "config_updater"
    parameters:
      key: "last_sync_time"
      value: "${workflow.start_time}"
      
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "fetch_users"
  - from: "start"
    to: "fetch_orders"
  - from: "fetch_users"
    to: "merge_data"
  - from: "fetch_orders"
    to: "merge_data"
  - from: "merge_data"
    to: "transform_records"
  - from: "transform_records"
    to: "save_to_database"
  - from: "save_to_database"
    to: "update_sync_timestamp"
  - from: "update_sync_timestamp"
    to: "end"
```

### 3. 文件处理工作流

批量处理文件和文档。

```yaml
name: "file-processing-pipeline"
version: "1.0.0"
description: "文件批量处理管道"

global_config:
  variables:
    input_dir: "/data/input"
    output_dir: "/data/output"
    archive_dir: "/data/archive"

nodes:
  - id: "start"
    type: "start"
    
  - id: "scan_directory"
    type: "tool"
    tool_name: "file_scanner"
    parameters:
      directory: "${input_dir}"
      pattern: "*.pdf"
      recursive: true
      
  - id: "check_files_exist"
    type: "condition"
    condition: "${scan_directory.file_count} > 0"
    
  - id: "process_files"
    type: "loop"
    condition: "${file_index} < ${scan_directory.file_count}"
    max_iterations: 1000
    
  - id: "extract_text"
    type: "tool"
    tool_name: "pdf_text_extractor"
    parameters:
      file_path: "${scan_directory.files[${file_index}]}"
      
  - id: "analyze_content"
    type: "tool"
    tool_name: "text_analyzer"
    parameters:
      text: "${extract_text.content}"
      analysis_types: ["sentiment", "keywords", "entities"]
      
  - id: "generate_summary"
    type: "tool"
    tool_name: "text_summarizer"
    parameters:
      text: "${extract_text.content}"
      max_length: 200
      
  - id: "save_results"
    type: "tool"
    tool_name: "json_writer"
    parameters:
      file_path: "${output_dir}/${extract_text.filename}.json"
      data:
        original_file: "${scan_directory.files[${file_index}]}"
        extracted_text: "${extract_text.content}"
        analysis: "${analyze_content.results}"
        summary: "${generate_summary.text}"
        processed_at: "${current_timestamp}"
        
  - id: "archive_file"
    type: "tool"
    tool_name: "file_mover"
    parameters:
      source: "${scan_directory.files[${file_index}]}"
      destination: "${archive_dir}/"
      
  - id: "generate_report"
    type: "tool"
    tool_name: "processing_report"
    parameters:
      total_files: "${scan_directory.file_count}"
      processed_files: "${file_index + 1}"
      output_directory: "${output_dir}"
      
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "scan_directory"
  - from: "scan_directory"
    to: "check_files_exist"
  - from: "check_files_exist"
    to: "process_files"
    condition: "true"
  - from: "process_files"
    to: "extract_text"
  - from: "extract_text"
    to: "analyze_content"
  - from: "analyze_content"
    to: "generate_summary"
  - from: "generate_summary"
    to: "save_results"
  - from: "save_results"
    to: "archive_file"
  - from: "archive_file"
    to: "process_files"  # 循环回到处理下一个文件
  - from: "process_files"
    to: "generate_report"  # 循环结束
  - from: "generate_report"
    to: "end"
  - from: "check_files_exist"
    to: "end"
    condition: "false"
```

### 4. 监控和告警工作流

系统监控和自动告警。

```yaml
name: "system-monitoring"
version: "1.0.0"
description: "系统监控和告警工作流"

global_config:
  variables:
    check_interval: "5m"
    alert_threshold: 80
    notification_channels: ["email", "slack"]

nodes:
  - id: "start"
    type: "start"
    
  - id: "check_cpu_usage"
    type: "tool"
    tool_name: "system_monitor"
    parameters:
      metric: "cpu_usage"
      duration: "5m"
      
  - id: "check_memory_usage"
    type: "tool"
    tool_name: "system_monitor"
    parameters:
      metric: "memory_usage"
      duration: "5m"
      
  - id: "check_disk_usage"
    type: "tool"
    tool_name: "system_monitor"
    parameters:
      metric: "disk_usage"
      path: "/"
      
  - id: "check_service_health"
    type: "tool"
    tool_name: "health_checker"
    parameters:
      services: ["nginx", "postgresql", "redis"]
      
  - id: "evaluate_alerts"
    type: "tool"
    tool_name: "alert_evaluator"
    parameters:
      metrics:
        cpu: "${check_cpu_usage.average}"
        memory: "${check_memory_usage.average}"
        disk: "${check_disk_usage.percentage}"
      service_status: "${check_service_health.results}"
      thresholds:
        cpu_warning: 70
        cpu_critical: 90
        memory_warning: 80
        memory_critical: 95
        disk_warning: 85
        disk_critical: 95
        
  - id: "check_alerts"
    type: "condition"
    condition: "${evaluate_alerts.alert_count} > 0"
    
  - id: "send_email_alert"
    type: "tool"
    tool_name: "email_sender"
    parameters:
      to: ["ops-team@company.com"]
      subject: "System Alert - ${evaluate_alerts.severity} Level"
      template: "system_alert"
      data: "${evaluate_alerts.alerts}"
      
  - id: "send_slack_alert"
    type: "tool"
    tool_name: "slack_sender"
    parameters:
      channel: "#alerts"
      message: "🚨 System Alert: ${evaluate_alerts.summary}"
      attachments: "${evaluate_alerts.details}"
      
  - id: "log_metrics"
    type: "tool"
    tool_name: "metrics_logger"
    parameters:
      timestamp: "${current_timestamp}"
      metrics:
        cpu_usage: "${check_cpu_usage.average}"
        memory_usage: "${check_memory_usage.average}"
        disk_usage: "${check_disk_usage.percentage}"
        service_health: "${check_service_health.healthy_count}"
        alert_count: "${evaluate_alerts.alert_count}"
        
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "check_cpu_usage"
  - from: "start"
    to: "check_memory_usage"
  - from: "start"
    to: "check_disk_usage"
  - from: "start"
    to: "check_service_health"
  - from: "check_cpu_usage"
    to: "evaluate_alerts"
  - from: "check_memory_usage"
    to: "evaluate_alerts"
  - from: "check_disk_usage"
    to: "evaluate_alerts"
  - from: "check_service_health"
    to: "evaluate_alerts"
  - from: "evaluate_alerts"
    to: "check_alerts"
  - from: "check_alerts"
    to: "send_email_alert"
    condition: "true"
  - from: "check_alerts"
    to: "send_slack_alert"
    condition: "true"
  - from: "send_email_alert"
    to: "log_metrics"
  - from: "send_slack_alert"
    to: "log_metrics"
  - from: "check_alerts"
    to: "log_metrics"
    condition: "false"
  - from: "log_metrics"
    to: "end"
```

### 5. 机器学习训练管道

自动化机器学习模型训练和部署。

```yaml
name: "ml-training-pipeline"
version: "1.0.0"
description: "机器学习模型训练管道"

global_config:
  variables:
    data_path: "/data/ml"
    model_path: "/models"
    experiment_name: "customer_churn_v1"

nodes:
  - id: "start"
    type: "start"
    
  - id: "load_data"
    type: "tool"
    tool_name: "data_loader"
    parameters:
      source: "${data_path}/customer_data.csv"
      format: "csv"
      
  - id: "preprocess_data"
    type: "tool"
    tool_name: "data_preprocessor"
    parameters:
      data: "${load_data.output}"
      operations:
        - "remove_duplicates"
        - "handle_missing_values"
        - "encode_categorical"
        - "normalize_numerical"
      target_column: "churn"
      
  - id: "split_data"
    type: "tool"
    tool_name: "data_splitter"
    parameters:
      data: "${preprocess_data.output}"
      train_ratio: 0.7
      validation_ratio: 0.15
      test_ratio: 0.15
      random_seed: 42
      
  - id: "train_model"
    type: "tool"
    tool_name: "ml_trainer"
    parameters:
      algorithm: "random_forest"
      train_data: "${split_data.train}"
      validation_data: "${split_data.validation}"
      hyperparameters:
        n_estimators: 100
        max_depth: 10
        min_samples_split: 5
      early_stopping: true
      
  - id: "evaluate_model"
    type: "tool"
    tool_name: "model_evaluator"
    parameters:
      model: "${train_model.model}"
      test_data: "${split_data.test}"
      metrics: ["accuracy", "precision", "recall", "f1", "auc"]
      
  - id: "check_performance"
    type: "condition"
    condition: "${evaluate_model.metrics.auc} > 0.85"
    
  - id: "save_model"
    type: "tool"
    tool_name: "model_saver"
    parameters:
      model: "${train_model.model}"
      path: "${model_path}/${experiment_name}"
      metadata:
        version: "${workflow.execution_id}"
        performance: "${evaluate_model.metrics}"
        training_date: "${current_timestamp}"
        
  - id: "deploy_model"
    type: "tool"
    tool_name: "model_deployer"
    parameters:
      model_path: "${save_model.path}"
      deployment_target: "production"
      health_check: true
      
  - id: "send_success_notification"
    type: "tool"
    tool_name: "notification_sender"
    parameters:
      message: "Model training completed successfully. AUC: ${evaluate_model.metrics.auc}"
      channels: ["email", "slack"]
      
  - id: "send_failure_notification"
    type: "tool"
    tool_name: "notification_sender"
    parameters:
      message: "Model training failed. Performance below threshold. AUC: ${evaluate_model.metrics.auc}"
      channels: ["email"]
      
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "load_data"
  - from: "load_data"
    to: "preprocess_data"
  - from: "preprocess_data"
    to: "split_data"
  - from: "split_data"
    to: "train_model"
  - from: "train_model"
    to: "evaluate_model"
  - from: "evaluate_model"
    to: "check_performance"
  - from: "check_performance"
    to: "save_model"
    condition: "true"
  - from: "save_model"
    to: "deploy_model"
  - from: "deploy_model"
    to: "send_success_notification"
  - from: "send_success_notification"
    to: "end"
  - from: "check_performance"
    to: "send_failure_notification"
    condition: "false"
  - from: "send_failure_notification"
    to: "end"
```

## 故障排除指南

### 1. 常见问题和解决方案

#### 问题：工作流创建失败

**错误信息**: `Invalid workflow definition: missing required field 'name'`

**解决方案**:
1. 检查YAML格式是否正确
2. 确认必需字段是否存在
3. 验证工作流定义

```bash
# 验证YAML格式
workflow-toolkit workflow create --validate my-workflow.yaml

# 查看详细错误信息
workflow-toolkit workflow create my-workflow.yaml --verbose
```

#### 问题：工作流执行卡住

**症状**: 工作流状态长时间保持"Running"，没有进展

**排查步骤**:
1. 检查当前执行状态
```bash
workflow-toolkit workflow status <execution-id> --format json
```

2. 查看节点详细信息
```bash
workflow-toolkit workflow status <execution-id> --watch
```

3. 检查系统资源
```bash
# 检查CPU和内存使用
top
# 检查磁盘空间
df -h
```

4. 检查工具可用性
```bash
workflow-toolkit tool list
workflow-toolkit tool execute <tool-name> --params '{}'
```

#### 问题：插件加载失败

**错误信息**: `Plugin load failed: No such file or directory`

**解决方案**:
1. 检查插件路径
```bash
ls -la /path/to/plugin
```

2. 验证插件配置
```bash
workflow-toolkit config show --section plugins
```

3. 检查依赖安装
```bash
# Python插件
pip list
# Node.js插件
npm list
# Docker插件
docker images
```

4. 重新安装插件
```bash
workflow-toolkit plugin uninstall <plugin-name>
workflow-toolkit plugin install <plugin-source>
```

#### 问题：工具执行超时

**错误信息**: `Tool execution timeout after 30s`

**解决方案**:
1. 增加超时时间
```yaml
nodes:
  - id: "slow_task"
    type: "tool"
    tool_name: "heavy_processor"
    timeout: "60m"  # 增加到60分钟
```

2. 优化工具性能
3. 检查网络连接（对于API调用）
4. 分解大任务为小任务

#### 问题：参数验证失败

**错误信息**: `Parameter validation failed: expected string, got number`

**解决方案**:
1. 查看工具schema
```bash
workflow-toolkit tool schema <tool-name>
```

2. 修正参数类型
```yaml
parameters:
  count: "10"      # 字符串
  # 而不是
  count: 10        # 数字
```

3. 使用参数转换
```yaml
parameters:
  count: "${string(10)}"
```

### 2. 调试技巧

#### 启用详细日志

```bash
# 设置日志级别
export RUST_LOG=debug
workflow-toolkit workflow execute my-workflow

# 或者使用命令行选项
workflow-toolkit --log-level debug workflow execute my-workflow
```

#### 使用干运行模式

```bash
# 模拟执行，不实际运行
workflow-toolkit workflow execute --dry-run my-workflow
```

#### 分步调试

```bash
# 单独测试工具
workflow-toolkit tool execute data_processor --params-file test-params.json

# 验证工作流定义
workflow-toolkit workflow create --validate my-workflow.yaml
```

#### 监控执行过程

```bash
# 实时监控
workflow-toolkit workflow status <execution-id> --watch

# 查看执行历史
workflow-toolkit workflow list --status completed
```

### 3. 性能优化

#### 并行执行优化

```yaml
# 使用并行节点提高效率
nodes:
  - id: "parallel_start"
    type: "parallel"
    
  - id: "task_1"
    type: "tool"
    tool_name: "processor_1"
    
  - id: "task_2"
    type: "tool"
    tool_name: "processor_2"
```

#### 缓存配置优化

```toml
[cache]
max_capacity = 2000      # 增加缓存容量
ttl = "2h"              # 延长缓存时间
```

#### 资源限制调整

```toml
[workflow]
max_concurrent_workflows = 5  # 减少并发数
default_timeout = "60m"       # 增加默认超时
```

### 4. 日志分析

#### 查看系统日志

```bash
# 查看应用日志
tail -f ~/.workflow-toolkit/logs/workflow-toolkit.log

# 查看特定工作流日志
grep "workflow_id:abc123" ~/.workflow-toolkit/logs/workflow-toolkit.log
```

#### 日志格式说明

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "target": "workflow_engine",
  "message": "Workflow execution started",
  "workflow_id": "abc123",
  "execution_id": "exec_456",
  "node_id": "process_data"
}
```

### 5. 备份和恢复

#### 数据备份

```bash
# 备份工作流定义
workflow-toolkit workflow list --format json > workflows_backup.json

# 备份配置
cp ~/.workflow-toolkit/config.toml config_backup.toml

# 备份数据库
cp -r ~/.workflow-toolkit/data data_backup/
```

#### 数据恢复

```bash
# 恢复工作流定义
cat workflows_backup.json | jq -r '.[] | .name' | while read name; do
  workflow-toolkit workflow create "${name}.yaml"
done

# 恢复配置
cp config_backup.toml ~/.workflow-toolkit/config.toml

# 恢复数据库
cp -r data_backup/ ~/.workflow-toolkit/data/
```

### 6. 获取帮助

#### 内置帮助

```bash
# 查看命令帮助
workflow-toolkit --help
workflow-toolkit workflow --help
workflow-toolkit tool execute --help
```

#### 社区支持

- 官方文档: https://workflow-toolkit.example.com/docs
- GitHub Issues: https://github.com/example/workflow-toolkit/issues
- 社区论坛: https://community.workflow-toolkit.example.com
- 邮件列表: support@workflow-toolkit.example.com

#### 报告问题

提交问题时请包含：
1. 工作流定义文件
2. 错误信息和日志
3. 系统环境信息
4. 复现步骤

```bash
# 收集系统信息
workflow-toolkit --version
workflow-toolkit config show
workflow-toolkit plugin list
```

## 最佳实践

### 1. 工作流设计原则

- **单一职责**: 每个节点只做一件事
- **幂等性**: 重复执行应该产生相同结果
- **错误处理**: 为关键节点设置重试和错误处理
- **可观测性**: 添加适当的日志和监控
- **文档化**: 为工作流添加清晰的描述和注释

### 2. 性能优化建议

- **并行化**: 识别可以并行执行的任务
- **缓存**: 缓存昂贵的计算结果
- **批处理**: 合并小任务减少开销
- **资源管理**: 合理设置超时和重试策略
- **监控**: 监控执行时间和资源使用

### 3. 安全考虑

- **敏感数据**: 使用环境变量存储密钥
- **权限控制**: 最小权限原则
- **输入验证**: 验证所有外部输入
- **审计日志**: 记录重要操作
- **网络安全**: 使用HTTPS和VPN

### 4. 维护和运维

- **版本控制**: 使用Git管理工作流定义
- **测试**: 在生产环境前充分测试
- **监控**: 设置告警和监控
- **备份**: 定期备份重要数据
- **文档**: 保持文档更新

---

*本手册持续更新，如有问题请查阅API参考文档或联系技术支持。*