# Workflow Toolkit 数据模型规范 (Data Model Specification)

## 文档元数据

- **版本**: v1.0.0
- **规范级别**: P2 (模块规范)
- **最后更新**: 2026-03-01
- **状态**: 已批准

## 1. 数据模型概述

### 1.1 数据存储类型

Workflow Toolkit 使用以下数据存储类型：
- **工作流定义**: YAML/JSON 文件
- **执行状态**: 文件系统存储 + 内存缓存
- **插件信息**: 文件系统存储 + 内存缓存
- **检查点**: 文件系统存储
- **备份**: 文件系统存储

### 1.2 数据格式

- **配置文件**: YAML 格式
- **数据交换**: JSON 格式
- **持久化**: 二进制格式（可选）

## 2. 工作流定义数据模型

### 2.1 工作流文件格式

**文件扩展名**: `.yaml` 或 `.json`

**YAML 格式示例**:
```yaml
apiVersion: workflow.kit/v1
kind: Workflow
metadata:
  name: hello-world
  description: 简单的 Hello World 工作流
  version: "1.0.0"
  tags:
    - example
    - basic
spec:
  concurrency:
    max_parallel_nodes: 10
    max_parallel_tools: 5
  retry:
    max_retries: 3
    backoff: exponential
  timeout: 3600s
  checkpoint:
    enabled: true
    interval: 60s
  nodes:
    - id: start
      name: 开始
      type: tool
      config:
        tool_name: echo
        parameters:
          message: "Hello, ${input.name}!"
    - id: process
      name: 处理
      type: tool
      config:
        tool_name: text_processor
        parameters:
          text: "${start.result}"
          operation: uppercase
    - id: end
      name: 结束
      type: tool
      config:
        tool_name: echo
        parameters:
          message: "${process.result}"
  edges:
    - source: start
      target: process
    - source: process
      target: end
```

### 2.2 节点配置

#### 工具节点

```yaml
type: tool
config:
  tool_name: string          # 工具名称
  tool_version: string       # 工具版本（可选）
  parameters:                # 参数
    key: value
  timeout: duration          # 超时时间（可选）
  retry:                     # 重试配置（可选）
    max_retries: integer
    backoff: linear|exponential
```

#### 条件节点

```yaml
type: condition
config:
  expression: string         # 条件表达式
  true_branch: string        # true 分支目标节点
  false_branch: string       # false 分支目标节点
```

#### 循环节点

```yaml
type: loop
config:
  items: string              # 循环项表达式
  item_var: string           # 循环变量名
  index_var: string          # 索引变量名（可选）
  body:                      # 循环体节点
    - node_id
  parallel: boolean          # 是否并行执行
  max_parallel: integer      # 最大并行数
```

#### 并行节点

```yaml
type: parallel
config:
  branches:                  # 并行分支
    - name: branch1
      nodes:
        - node_id
    - name: branch2
      nodes:
        - node_id
  wait_strategy: all|any|n   # 等待策略
  wait_count: integer        # 等待数量（n 策略）
```

#### 切换节点

```yaml
type: switch
config:
  expression: string         # 切换表达式
  cases:                     # 分支情况
    - value: "value1"
      target: node_id1
    - value: "value2"
      target: node_id2
  default: node_id           # 默认分支（可选）
```

#### 检查点节点

```yaml
type: checkpoint
config:
  name: string               # 检查点名称
  save_context: boolean      # 是否保存上下文
```

### 2.3 边配置

```yaml
source: string               # 源节点 ID
target: string               # 目标节点 ID
condition: string            # 条件表达式（可选）
weight: integer              # 权重（可选）
```

## 3. 执行状态数据模型

### 3.1 执行状态文件格式

**存储路径**: `{checkpoint_dir}/{execution_id}/state.json`

```json
{
  "execution_id": "exec-123",
  "workflow_id": "hello-world",
  "status": "running",
  "started_at": "2026-03-01T10:00:00Z",
  "ended_at": null,
  "progress": 0.5,
  "current_node": "process",
  "node_states": {
    "start": {
      "node_id": "start",
      "status": "completed",
      "started_at": "2026-03-01T10:00:00Z",
      "ended_at": "2026-03-01T10:00:01Z",
      "result": "Hello, World!",
      "error": null,
      "retry_count": 0
    },
    "process": {
      "node_id": "process",
      "status": "running",
      "started_at": "2026-03-01T10:00:01Z",
      "ended_at": null,
      "result": null,
      "error": null,
      "retry_count": 0
    }
  },
  "context": {
    "global": {
      "input": {
        "name": "World"
      }
    },
    "slots": {
      "start.result": "Hello, World!"
    }
  },
  "checkpoints": [
    {
      "id": "cp-1",
      "name": "after_start",
      "created_at": "2026-03-01T10:00:01Z",
      "node_id": "start",
      "context_snapshot": {}
    }
  ]
}
```

### 3.2 执行历史记录

**存储路径**: `{data_dir}/history/{execution_id}.json`

```json
{
  "execution_id": "exec-123",
  "workflow_id": "hello-world",
  "status": "completed",
  "started_at": "2026-03-01T10:00:00Z",
  "ended_at": "2026-03-01T10:00:05Z",
  "duration_ms": 5000,
  "node_count": 3,
  "success_count": 3,
  "failure_count": 0,
  "result": {
    "message": "HELLO, WORLD!"
  },
  "metrics": {
    "total_tools_executed": 3,
    "total_retries": 0,
    "cache_hits": 0,
    "cache_misses": 3
  }
}
```

## 4. 插件数据模型

### 4.1 插件清单文件

**文件名**: `plugin.yaml` 或 `plugin.json`

```yaml
apiVersion: plugin.kit/v1
kind: Plugin
metadata:
  name: file-management
  version: "1.0.0"
  description: 文件管理插件
  author: Workflow Toolkit Team
  tags:
    - file
    - management
spec:
  type: native|python|nodejs|docker|wasm
  entry: libfile_management.so|main.py|index.js|Dockerfile|module.wasm
  runtime:
    python_version: "3.10"
    node_version: "18"
  tools:
    - name: text_processor
      description: 文本处理工具
      kind: processor
      input_schema:
        type: object
        properties:
          text:
            type: string
            description: 输入文本
          operation:
            type: string
            enum: [uppercase, lowercase, reverse]
        required: [text, operation]
      output_schema:
        type: object
        properties:
          result:
            type: string
      examples:
        - input:
            text: "hello"
            operation: "uppercase"
          output:
            result: "HELLO"
  security:
    level: basic
    file_permissions:
      read_paths:
        - /data/input
      write_paths:
        - /data/output
    network_permissions:
      enabled: false
  resources:
    max_memory: 256MB
    max_cpu: 0.5
    max_execution_time: 60s
```

### 4.2 插件状态文件

**存储路径**: `{data_dir}/plugins/{plugin_id}.json`

```json
{
  "plugin_id": "file-management",
  "status": "loaded",
  "loaded_at": "2026-03-01T09:00:00Z",
  "tool_count": 5,
  "tools": [
    "text_processor",
    "batch_processor",
    "classifier",
    "human_decision",
    "result_confirmation"
  ],
  "metrics": {
    "total_executions": 100,
    "success_count": 98,
    "failure_count": 2,
    "average_execution_time_ms": 150
  }
}
```

## 5. 工具数据模型

### 5.1 工具注册信息

```json
{
  "id": {
    "name": "text_processor",
    "version": "1.0.0"
  },
  "kind": "processor",
  "description": "文本处理工具",
  "plugin_id": "file-management",
  "input_schema": {
    "type": "object",
    "properties": {
      "text": {
        "type": "string",
        "description": "输入文本"
      },
      "operation": {
        "type": "string",
        "enum": ["uppercase", "lowercase", "reverse"],
        "description": "操作类型"
      }
    },
    "required": ["text", "operation"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "result": {
        "type": "string",
        "description": "处理结果"
      }
    }
  },
  "metadata": {
    "category": "text",
    "tags": ["text", "process"],
    "author": "Workflow Toolkit Team",
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-03-01T00:00:00Z"
  },
  "config": {
    "timeout": 30,
    "retryable": true,
    "cacheable": true,
    "cache_ttl": 3600
  }
}
```

### 5.2 工具组合定义

```json
{
  "id": {
    "name": "text_pipeline",
    "version": "1.0.0"
  },
  "kind": "composed",
  "composition_type": "chain",
  "tools": [
    {
      "tool_name": "text_processor",
      "input_mapping": {
        "text": "$.input.text",
        "operation": "uppercase"
      },
      "output_mapping": {
        "$.result": "$.step1.result"
      }
    },
    {
      "tool_name": "text_processor",
      "input_mapping": {
        "text": "$.step1.result",
        "operation": "reverse"
      },
      "output_mapping": {
        "$.result": "$.output.result"
      }
    }
  ],
  "error_propagation": "fail_fast"
}
```

## 6. 检查点数据模型

### 6.1 检查点文件格式

**存储路径**: `{checkpoint_dir}/{execution_id}/checkpoints/{checkpoint_id}.json`

```json
{
  "checkpoint_id": "cp-1",
  "execution_id": "exec-123",
  "name": "after_start",
  "created_at": "2026-03-01T10:00:01Z",
  "node_id": "start",
  "node_status": "completed",
  "context_snapshot": {
    "global": {
      "input": {
        "name": "World"
      }
    },
    "slots": {
      "start.result": "Hello, World!"
    }
  },
  "pending_nodes": ["process", "end"],
  "completed_nodes": ["start"],
  "metadata": {
    "workflow_id": "hello-world",
    "workflow_version": "1.0.0"
  }
}
```

## 7. 备份数据模型

### 7.1 备份清单文件

**存储路径**: `{backup_dir}/backup-{timestamp}/manifest.json`

```json
{
  "backup_id": "backup-20260301-100000",
  "created_at": "2026-03-01T10:00:00Z",
  "type": "full|incremental",
  "base_backup": null,
  "size_bytes": 1048576,
  "items": [
    {
      "type": "workflow",
      "id": "hello-world",
      "path": "workflows/hello-world.yaml",
      "checksum": "sha256:abc123"
    },
    {
      "type": "execution",
      "id": "exec-123",
      "path": "executions/exec-123.json",
      "checksum": "sha256:def456"
    },
    {
      "type": "plugin",
      "id": "file-management",
      "path": "plugins/file-management.json",
      "checksum": "sha256:ghi789"
    }
  ],
  "statistics": {
    "total_items": 10,
    "total_size_bytes": 1048576,
    "workflows": 3,
    "executions": 5,
    "plugins": 2
  },
  "verification": {
    "verified_at": "2026-03-01T10:01:00Z",
    "status": "passed",
    "errors": []
  }
}
```

## 8. 配置数据模型

### 8.1 全局配置文件

**文件名**: `config.toml` 或 `config.yaml`

```toml
[general]
name = "workflow-toolkit"
version = "1.0.0"
log_level = "info"

[storage]
data_dir = "./data"
checkpoint_dir = "./checkpoints"
backup_dir = "./backups"
cache_enabled = true
cache_ttl = 3600

[workflow]
default_timeout = 3600
max_concurrent_workflows = 10
checkpoint_interval = 60

[plugin]
plugin_dir = "./plugins"
auto_load = true
security_level = "basic"

[performance]
max_memory = "1GB"
max_cpu = 2.0
metrics_enabled = true
tracing_enabled = false

[interface]
default_output = "table"
theme = "default"
```

### 8.2 环境变量配置

| 环境变量 | 描述 | 默认值 |
|----------|------|--------|
| `WT_CONFIG` | 配置文件路径 | `./config.toml` |
| `WT_DATA_DIR` | 数据目录 | `./data` |
| `WT_LOG_LEVEL` | 日志级别 | `info` |
| `WT_CACHE_ENABLED` | 是否启用缓存 | `true` |
| `WT_SECURITY_LEVEL` | 安全级别 | `basic` |

## 9. 数据迁移

### 9.1 版本兼容性

| 数据类型 | 向后兼容 | 迁移策略 |
|----------|----------|----------|
| 工作流定义 | MAJOR 版本 | 自动迁移 |
| 执行状态 | MINOR 版本 | 自动迁移 |
| 插件清单 | MAJOR 版本 | 手动迁移 |
| 检查点 | PATCH 版本 | 自动迁移 |

### 9.2 迁移脚本

```bash
# 迁移工作流定义
workflow-toolkit migrate workflows --from v0.9 --to v1.0

# 迁移执行状态
workflow-toolkit migrate executions --from v0.9 --to v1.0

# 迁移插件配置
workflow-toolkit migrate plugins --from v0.9 --to v1.0
```

## 10. 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0.0 | 2026-03-01 | 初始版本 | Architecture Team |
