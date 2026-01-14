# Workflow Toolkit - 用户指南

## 快速开始

### 安装

```bash
# 从源码构建
cargo build --release

# 或使用预构建二进制 (如果可用)
# workflow-toolkit --version
```

### 基本使用

```bash
# 查看帮助
workflow-toolkit --help

# 查看版本
workflow-toolkit --version

# 查看可用命令
workflow-toolkit workflow --help
workflow-toolkit tool --help
```

## 工作流管理

### 1. 创建工作流

工作流使用YAML格式定义：

```yaml
# hello-world.yaml
name: hello-world
version: 1.0.0
description: 简单的示例工作流

nodes:
  - id: greet
    tool: echo
    params:
      message: "Hello, World!"
    
  - id: process
    tool: transform
    params:
      input: "{{greet.output}}"
      operation: uppercase

edges:
  - from: greet
    to: process
```

### 2. 执行工作流

```bash
# 方式1: 直接执行定义文件
workflow-toolkit workflow execute hello-world.yaml

# 方式2: 先创建，再执行
workflow-toolkit workflow create hello-world.yaml
workflow-toolkit workflow execute hello-world

# 方式3: 带参数执行
workflow-toolkit workflow execute hello-world.yaml --params params.json

# 方式4: 后台执行
workflow-toolkit workflow execute hello-world.yaml --background

# 方式5: 等待完成并显示进度
workflow-toolkit workflow execute hello-world.yaml --wait
```

### 3. 查看状态

```bash
# 列出所有工作流
workflow-toolkit workflow list

# 查看特定执行状态
workflow-toolkit workflow status <workflow-id>

# 详细状态
workflow-toolkit workflow status <workflow-id> --detailed

# 实时跟踪
workflow-toolkit workflow status <workflow-id> --follow
```

### 4. 控制执行

```bash
# 暂停
workflow-toolkit workflow pause <workflow-id>

# 恢复
workflow-toolkit workflow resume <workflow-id>

# 停止
workflow-toolkit workflow stop <workflow-id>

# 强制停止
workflow-toolkit workflow stop <workflow-id> --force
```

### 5. 批量执行

```bash
# 创建批量执行列表 (batch-list.json)
{
  "workflows": [
    {"name": "workflow1", "file": "wf1.yaml", "params": {"param1": "value1"}},
    {"name": "workflow2", "file": "wf2.yaml", "params": {"param2": "value2"}}
  ]
}

# 执行批量任务
workflow-toolkit batch execute batch-list.json --parallel 4 --continue-on-failure

# 查看结果
ls output/
```

## 工具管理

### 1. 列出工具

```bash
# 所有工具
workflow-toolkit tool list

# 按分类过滤
workflow-toolkit tool list --category data-processing

# 按标签过滤
workflow-toolkit tool list --tag ai

# 搜索工具
workflow-toolkit tool list --search "transform"
```

### 2. 执行工具

```bash
# 方式1: JSON参数
workflow-toolkit tool execute echo --params '{"message": "Hello"}'

# 方式2: 参数文件
workflow-toolkit tool execute transform --params-file params.json

# 方式3: 带超时
workflow-toolkit tool execute long-running --timeout 300
```

### 3. 工具类型

#### Echo工具 (测试用)
```bash
workflow-toolkit tool execute echo --params '{"message": "Test"}'
```

#### Transform工具 (数据转换)
```bash
# 大写转换
workflow-toolkit tool execute transform --params '{
  "input": "hello",
  "operation": "uppercase"
}'

# 小写转换
workflow-toolkit tool execute transform --params '{
  "input": "HELLO",
  "operation": "lowercase"
}'
```

## 插件管理

### 1. 插件类型

| 类型 | 用途 | 示例 |
|------|------|------|
| **Native** | 高性能Rust代码 | 数据处理库 |
| **Python** | ML/数据科学 | 机器学习模型 |
| **Node.js** | JavaScript工具 | Web API调用 |
| **Docker** | 隔离环境 | 复杂工具链 |
| **WASM** | 便携代码 (暂不可用) | 沙箱计算 |

### 2. 加载插件

```bash
# 方式1: 配置文件自动加载
# config/default.toml
[plugins]
plugin_dir = "./plugins"
auto_load = true

# 方式2: 手动加载
workflow-toolkit plugin install ./plugins/my-plugin.so

# 方式3: 重新加载
workflow-toolkit plugin reload my-plugin
```

### 3. 管理插件

```bash
# 列出已加载插件
workflow-toolkit plugin list

# 查看插件详情
workflow-toolkit plugin list --detailed

# 卸载插件
workflow-toolkit plugin unload my-plugin
```

### 4. Python插件示例

```python
# plugins/my_ml_plugin.py
def process_data(params, context):
    """处理数据的工具函数"""
    input_data = params.get("input")
    # ML处理逻辑
    result = {"output": processed}
    return result

# 插件元数据
PLUGIN_INFO = {
    "name": "ml-processor",
    "version": "1.0.0",
    "tools": [
        {
            "name": "ml_predict",
            "description": "机器学习预测",
            "parameters": {
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }
        }
    ]
}
```

## TUI界面

### 启动TUI

```bash
workflow-toolkit tui
```

### TUI导航

```
┌─────────────────────────────────────────────────┐
│  Workflow Toolkit v0.1.0                        │
├─────────────────────────────────────────────────┤
│  [Tab] 切换视图  [↑/↓] 导航  [Enter] 选择       │
│  [Esc] 返回     [q] 退出    [h] 帮助            │
├─────────────────────────────────────────────────┤
│  ▼ Workflows                                    │
│  ▶ Tools                                       │
│  ▶ Plugins                                     │
│  ▶ System Status                               │
│  ▶ Logs                                        │
└─────────────────────────────────────────────────┘
```

### TUI视图

#### 工作流视图
- 浏览所有工作流
- 查看执行状态
- 启动/暂停/停止工作流
- 查看执行历史

#### 工具视图
- 列出可用工具
- 查看工具详情
- 直接执行工具

#### 系统状态视图
- CPU使用率
- 内存使用
- 磁盘空间
- 网络状态

#### 日志视图
- 实时日志流
- 按级别过滤
- 搜索功能

### TUI快捷键

| 按键 | 功能 |
|------|------|
| `Tab` | 切换视图 |
| `↑/↓` | 导航 |
| `Enter` | 选择/执行 |
| `Esc` | 返回/取消 |
| `q` | 退出 |
| `r` | 刷新 |
| `h` | 帮助 |
| `m` | 维护模式 |

## 配置管理

### 配置文件位置

1. **系统级**: `/etc/workflow-toolkit/config.toml`
2. **用户级**: `~/.config/workflow-toolkit/config.toml`
3. **项目级**: `./workflow-toolkit.toml`
4. **命令行**: `--config path/to/config.toml`

### 配置优先级

命令行参数 > 环境变量 > 项目配置 > 用户配置 > 系统配置 > 默认值

### 配置示例

```toml
# config/default.toml

[server]
http_port = 8080
ws_port = 8081
bind_address = "127.0.0.1"

[storage]
database_path = "./data/workflow.db"
cache_size = 104857600  # 100MB
backup_enabled = true
backup_interval = 86400  # 24小时
retention_days = 30

[logging]
level = "info"
format = "Pretty"
file_path = "./logs/workflow-toolkit.log"

[plugins]
plugin_dir = "./plugins"
auto_load = true
sandbox_enabled = true
timeout = 300
memory_limit = 1073741824  # 1GB

[workflow]
max_concurrent_workflows = 5
default_timeout = 1800  # 30分钟
checkpoint_enabled = true
checkpoint_interval = 300  # 5分钟

[file_management]
temp_dir = "./tmp"
max_file_size = "100MB"
classification_confidence_threshold = 0.8
batch_size = 100

[performance]
enable_profiling = false
metrics_collection = true
cache_ttl = 3600  # 1小时

[tui]
theme = "dark"
refresh_rate = 60
enable_mouse = true
enable_virtualization = true
max_log_entries = 10000

[system_monitoring]
cpu_threshold_warning = 70.0
cpu_threshold_critical = 90.0
memory_threshold_warning = 80.0
memory_threshold_critical = 95.0
disk_threshold_warning = 85.0
disk_threshold_critical = 95.0
update_interval = 5

[maintenance]
auto_cleanup_enabled = true
cleanup_interval = 3600  # 1小时
max_alert_age = 86400  # 24小时
memory_cleanup_threshold = 90.0
disk_cleanup_threshold = 90.0
```

### 环境变量

```bash
# 服务器配置
export WORKFLOW_TOOLKIT_SERVER__HTTP_PORT=8080
export WORKFLOW_TOOLKIT_SERVER__WS_PORT=8081

# 存储配置
export WORKFLOW_TOOLKIT_STORAGE__DATABASE_PATH="./data/workflow.db"

# 插件配置
export WORKFLOW_TOOLKIT_PLUGINS__PLUGIN_DIR="./plugins"

# 日志配置
export WORKFLOW_TOOLKIT_LOGGING__LEVEL="debug"

# TUI配置
export WORKFLOW_TOOLKIT_TUI__THEME="dark"
export WORKFLOW_TOOLKIT_TUI__REFRESH_RATE=60

# 系统监控配置
export WORKFLOW_TOOLKIT_SYSTEM_MONITORING__CPU_THRESHOLD_WARNING=70.0
export WORKFLOW_TOOLKIT_SYSTEM_MONITORING__UPDATE_INTERVAL="5s"

# 维护配置
export WORKFLOW_TOOLKIT_MAINTENANCE__AUTO_CLEANUP_ENABLED=true
```

## 文件管理插件

### 1. 文件分类

```bash
# 交互式分类
workflow-toolkit tool execute file_classification \
  --params '{
    "directory": "./data",
    "rules_file": "classification-rules.json",
    "interactive": true
  }'
```

**分类规则文件 (classification-rules.json):**
```json
{
  "rules": [
    {
      "pattern": "*.pdf",
      "category": "documents",
      "action": "move",
      "destination": "./documents/pdf"
    },
    {
      "pattern": "*.jpg|*.png|*.gif",
      "category": "images",
      "action": "copy",
      "destination": "./media/images"
    },
    {
      "pattern": "*.csv|*.xlsx",
      "category": "data",
      "action": "move",
      "destination": "./data/spreadsheets"
    }
  ]
}
```

### 2. 批量处理

```bash
# 批量重命名
workflow-toolkit tool execute batch_processor \
  --params '{
    "input_dir": "./input",
    "output_dir": "./output",
    "operation": "rename",
    "pattern": "prefix_{index}_{original}",
    "batch_size": 100
  }'
```

### 3. 文本处理

```bash
# 文本分析
workflow-toolkit tool execute text_processor \
  --params '{
    "input_file": "document.txt",
    "operations": ["extract_keywords", "count_words", "sentiment_analysis"],
    "output_format": "json"
  }'
```

## MCP服务器

### 启动服务器

```bash
# 默认端口
workflow-toolkit server

# 自定义端口
workflow-toolkit server --http-port 8080 --ws-port 8081

# 带认证
export WORKFLOW_TOOLKIT_AUTH__JWT_SECRET="your-secret-key"
workflow-toolkit server
```

### MCP客户端连接

```bash
# 使用MCP客户端连接
mcp-client connect http://localhost:8080

# 执行工作流
mcp-client call execute_workflow --params '{"name": "hello-world"}'

# 列出工具
mcp-client call list_tools
```

## 高级功能

### 1. 检查点恢复

工作流自动保存检查点，支持从失败点恢复：

```bash
# 执行工作流 (自动启用检查点)
workflow-toolkit workflow execute long-workflow.yaml

# 如果失败，重新执行会自动恢复
workflow-toolkit workflow execute long-workflow.yaml
```

### 2. 错误恢复策略

在工作流定义中配置：

```yaml
name: resilient-workflow
config:
  retry_policy:
    strategy: exponential
    max_attempts: 3
    base_delay: 1
    max_delay: 60
    backoff_multiplier: 2.0
  
  error_handling:
    on_node_failure: retry  # retry | skip | stop | pause
    on_workflow_failure: pause  # stop | pause | continue
```

### 3. 并行执行

```yaml
nodes:
  - id: parallel_task_1
    tool: data_processor
    params: {input: "data1"}
    
  - id: parallel_task_2
    tool: data_processor
    params: {input: "data2"}
    
  - id: parallel_task_3
    tool: data_processor
    params: {input: "data3"}

edges:
  - from: start
    to: parallel_task_1
  - from: start
    to: parallel_task_2
  - from: start
    to: parallel_task_3
  - from: parallel_task_1
    to: merge
  - from: parallel_task_2
    to: merge
  - from: parallel_task_3
    to: merge
```

### 4. 条件执行

```yaml
nodes:
  - id: check_condition
    tool: condition_checker
    params: {threshold: 100}
    
  - id: process_if_true
    tool: data_processor
    params: {mode: "advanced"}
    
  - id: process_if_false
    tool: data_processor
    params: {mode: "basic"}

edges:
  - from: check_condition
    to: process_if_true
    condition: "output.result == true"
  - from: check_condition
    to: process_if_false
    condition: "output.result == false"
```

### 5. 循环执行

```yaml
nodes:
  - id: loop_counter
    tool: counter
    params: {start: 0, end: 10}
    
  - id: process_item
    tool: item_processor
    params: {item: "{{loop_counter.current}}"}

edges:
  - from: loop_counter
    to: process_item
    condition: "counter.current < counter.end"
  - from: process_item
    to: loop_counter
    condition: "counter.next"
```

## 监控和维护

### 1. 系统监控

```bash
# 实时监控
workflow-toolkit system monitor --watch

# 健康检查
workflow-toolkit system health

# 性能分析
workflow-toolkit system performance --analyze
```

### 2. 维护操作

```bash
# 查看维护建议
workflow-toolkit system maintenance --recommendations

# 清理缓存
workflow-toolkit system maintenance --cleanup-cache

# 优化数据库
workflow-toolkit system maintenance --optimize

# 备份数据
workflow-toolkit system maintenance --backup
```

### 3. 日志管理

```bash
# 查看日志
tail -f logs/workflow-toolkit.log

# 按级别过滤
grep "ERROR" logs/workflow-toolkit.log

# 按时间过滤
grep "2024-01-15" logs/workflow-toolkit.log
```

## 故障排除

### 常见问题

#### 1. 工作流验证失败
```bash
# 检查工作流定义
workflow-toolkit workflow create file.yaml --validate-only

# 查看详细错误
RUST_LOG=debug workflow-toolkit workflow create file.yaml
```

#### 2. 工具未找到
```bash
# 列出所有工具
workflow-toolkit tool list

# 检查插件是否加载
workflow-toolkit plugin list
```

#### 3. 插件加载失败
```bash
# 启用详细日志
RUST_LOG=debug workflow-toolkit plugin install plugin.so

# 检查权限
ls -l plugin.so
chmod +x plugin.so
```

#### 4. 内存不足
```bash
# 检查内存使用
workflow-toolkit system monitor

# 调整配置
# config.toml
[plugins]
memory_limit = 536870912  # 512MB
```

#### 5. 性能问题
```bash
# 启用性能分析
export WORKFLOW_TOOLKIT_PERFORMANCE__ENABLE_PROFILING=true

# 查看热点
cargo run --release --example performance_monitoring_example
```

## 最佳实践

### 1. 工作流设计

- **保持简单**: 每个工作流只做一件事
- **模块化**: 重用工具和子工作流
- **错误处理**: 配置适当的重试策略
- **测试**: 先在小数据集上测试
- **文档**: 添加清晰的描述和注释

### 2. 工具选择

- **优先原生工具**: 性能最佳
- **插件用于特殊需求**: ML、外部API等
- **参数验证**: 始终验证输入参数
- **资源限制**: 设置合理的超时和内存限制

### 3. 性能优化

- **批量操作**: 减少I/O次数
- **缓存结果**: 重用计算结果
- **并行执行**: 利用多核CPU
- **异步操作**: 避免阻塞

### 4. 安全考虑

- **插件沙箱**: 启用沙箱模式
- **资源限制**: 设置内存和CPU限制
- **权限控制**: 限制文件系统访问
- **审计日志**: 记录所有操作

### 5. 备份和恢复

- **定期备份**: 配置自动备份
- **版本控制**: 工作流定义使用Git
- **测试恢复**: 定期测试恢复流程
- **多副本**: 重要数据多副本存储

## 示例工作流

### 1. 数据处理管道

```yaml
# data-pipeline.yaml
name: data-processing-pipeline
version: 1.0.0
description: 从CSV到报告的数据处理管道

nodes:
  - id: load_data
    tool: csv_loader
    params:
      file: "./data/input.csv"
      delimiter: ","
      
  - id: validate_data
    tool: data_validator
    params:
      rules: "./config/validation-rules.json"
      
  - id: transform_data
    tool: data_transformer
    params:
      operations: ["clean", "normalize", "aggregate"]
      
  - id: ml_predict
    plugin: ml-processor
    tool: predict
    params:
      model: "./models/predictor.pkl"
      
  - id: generate_report
    tool: report_generator
    params:
      format: "pdf"
      output: "./reports/final-report.pdf"

edges:
  - from: load_data
    to: validate_data
  - from: validate_data
    to: transform_data
  - from: transform_data
    to: ml_predict
  - from: ml_predict
    to: generate_report
```

执行:
```bash
workflow-toolkit workflow execute data-pipeline.yaml --wait
```

### 2. 文件整理器

```yaml
# file-organizer.yaml
name: file-organizer
version: 1.0.0
description: 自动整理下载文件夹

nodes:
  - id: scan_downloads
    tool: folder_scanner
    params:
      directory: "~/Downloads"
      recursive: false
      
  - id: classify_files
    plugin: file-management
    tool: file_classification
    params:
      rules_file: "./config/file-rules.json"
      interactive: false
      
  - id: organize
    plugin: file-management
    tool: batch_processor
    params:
      operation: "move"
      batch_size: 50

edges:
  - from: scan_downloads
    to: classify_files
  - from: classify_files
    to: organize
```

执行:
```bash
workflow-toolkit workflow execute file-organizer.yaml
```

### 3. API数据同步

```yaml
# api-sync.yaml
name: api-data-sync
version: 1.0.0
description: 从API同步数据到本地

nodes:
  - id: fetch_data
    tool: http_client
    params:
      url: "https://api.example.com/data"
      method: "GET"
      headers:
        Authorization: "Bearer {{env.API_TOKEN}}"
      
  - id: parse_response
    tool: json_parser
    params:
      path: "$.results[*]"
      
  - id: store_data
    tool: database_writer
    params:
      table: "synced_data"
      batch_size: 1000
      
  - id: send_notification
    tool: email_sender
    params:
      to: "admin@example.com"
      subject: "数据同步完成"
      body: "成功同步 {{store_data.count}} 条记录"

edges:
  - from: fetch_data
    to: parse_response
  - from: parse_response
    to: store_data
  - from: store_data
    to: send_notification
```

执行:
```bash
# 设置环境变量
export API_TOKEN="your-api-token"

# 执行
workflow-toolkit workflow execute api-sync.yaml --wait
```

## 总结

本指南涵盖了Workflow Toolkit的所有核心功能。关键要点：

1. **工作流**: 使用YAML定义，支持DAG结构
2. **工具**: 可扩展的工具系统，支持多种类型
3. **插件**: 灵活的插件架构，支持多语言
4. **接口**: CLI、TUI、MCP三种交互方式
5. **配置**: 分层配置，环境变量支持
6. **监控**: 实时监控和维护工具

**下一步**:
- 查看 `DEVELOPMENT_GUIDE.md` 了解开发细节
- 阅读 `AGENTS.md` 文件获取模块特定指导
- 探索 `examples/` 目录中的示例
- 加入社区讨论和贡献代码

**获取帮助**:
- 使用 `--help` 查看命令帮助
- 查看日志获取详细信息: `RUST_LOG=debug`
- 阅读设计文档了解架构细节
