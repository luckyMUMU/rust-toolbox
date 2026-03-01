# Workflow Toolkit API 契约规范 (API Contract Specification)

## 文档元数据

- **版本**: v1.0.0
- **规范级别**: P2 (模块规范)
- **最后更新**: 2026-03-01
- **状态**: 已批准

## 1. API 概述

### 1.1 API 类型

Workflow Toolkit 提供以下 API 类型：
- **CLI API**: 命令行接口
- **TUI API**: 终端用户界面 API
- **MCP API**: Model Context Protocol API（AI 助手集成）

### 1.2 API 设计原则

1. **一致性**: 所有 API 遵循统一的设计风格
2. **可发现性**: API 提供清晰的帮助和文档
3. **错误处理**: 统一的错误响应格式
4. **版本化**: API 支持版本管理

## 2. CLI API

### 2.1 全局选项

```
workflow-toolkit [OPTIONS] <COMMAND>

选项:
  -c, --config <FILE>     配置文件路径
  -v, --verbose           详细输出
  -q, --quiet             静默模式
  -o, --output <FORMAT>   输出格式 (json|yaml|table)
  -h, --help              显示帮助信息
  -V, --version           显示版本信息
```

### 2.2 工作流命令

#### 执行工作流

```
workflow-toolkit workflow execute [OPTIONS] <WORKFLOW>

参数:
  <WORKFLOW>  工作流文件路径或名称

选项:
  --input <JSON>          输入参数 (JSON 格式)
  --input-file <FILE>     输入参数文件
  --dry-run               试运行，不实际执行
  --timeout <SECONDS>     执行超时时间
  --checkpoint <DIR>      检查点目录
  --resume <ID>           从指定执行 ID 恢复

输出:
  执行结果 (根据 --output 格式化)
```

**示例**:
```bash
# 执行工作流
workflow-toolkit workflow execute basic/hello-world.yaml

# 带参数执行
workflow-toolkit workflow execute --input '{"name": "World"}' basic/hello-world.yaml

# 从文件读取参数
workflow-toolkit workflow execute --input-file params.json basic/hello-world.yaml

# 恢复执行
workflow-toolkit workflow execute --resume exec-123 basic/hello-world.yaml
```

#### 列出工作流

```
workflow-toolkit workflow list [OPTIONS]

选项:
  --filter <PATTERN>     过滤模式
  --format <FORMAT>      输出格式

输出:
  工作流列表
```

#### 验证工作流

```
workflow-toolkit workflow validate <WORKFLOW>

参数:
  <WORKFLOW>  工作流文件路径

输出:
  验证结果
```

#### 查看工作流状态

```
workflow-toolkit workflow status <EXECUTION_ID>

参数:
  <EXECUTION_ID>  执行 ID

输出:
  执行状态详情
```

### 2.3 工具命令

#### 列出工具

```
workflow-toolkit tool list [OPTIONS]

选项:
  --category <CAT>       按类别过滤
  --tag <TAG>            按标签过滤
  --kind <KIND>          按类型过滤
  --format <FORMAT>      输出格式

输出:
  工具列表
```

#### 执行工具

```
workflow-toolkit tool execute [OPTIONS] <TOOL_NAME>

参数:
  <TOOL_NAME>  工具名称

选项:
  --input <JSON>         输入参数 (JSON 格式)
  --input-file <FILE>    输入参数文件
  --timeout <SECONDS>    执行超时时间

输出:
  工具执行结果
```

**示例**:
```bash
# 执行工具
workflow-toolkit tool execute --input '{"text": "hello"}' text_processor

# 查看工具信息
workflow-toolkit tool info text_processor
```

#### 查看工具信息

```
workflow-toolkit tool info <TOOL_NAME>

参数:
  <TOOL_NAME>  工具名称

输出:
  工具详细信息（包括 Schema）
```

### 2.4 插件命令

#### 列出插件

```
workflow-toolkit plugin list [OPTIONS]

选项:
  --status <STATUS>      按状态过滤
  --type <TYPE>          按类型过滤
  --format <FORMAT>      输出格式

输出:
  插件列表
```

#### 加载插件

```
workflow-toolkit plugin load <PLUGIN_PATH>

参数:
  <PLUGIN_PATH>  插件路径

输出:
  加载结果
```

#### 卸载插件

```
workflow-toolkit plugin unload <PLUGIN_ID>

参数:
  <PLUGIN_ID>  插件 ID

输出:
  卸载结果
```

#### 重载插件

```
workflow-toolkit plugin reload <PLUGIN_ID>

参数:
  <PLUGIN_ID>  插件 ID

输出:
  重载结果
```

### 2.5 批处理命令

#### 执行批处理

```
workflow-toolkit batch execute [OPTIONS] <CONFIG>

参数:
  <CONFIG>  批处理配置文件

选项:
  --parallel <N>         并行数
  --dry-run              试运行
  --continue-on-error    出错时继续

输出:
  批处理结果
```

## 3. TUI API

### 3.1 启动 TUI

```
workflow-toolkit tui [OPTIONS]

选项:
  --theme <THEME>        主题名称
  --layout <LAYOUT>      布局配置
```

### 3.2 快捷键

| 快捷键 | 功能 |
|--------|------|
| `q` | 退出 |
| `h` | 帮助 |
| `Tab` | 切换面板 |
| `Enter` | 确认/执行 |
| `Esc` | 取消/返回 |
| `r` | 刷新 |
| `p` | 暂停/恢复 |
| `s` | 停止 |
| `?` | 显示帮助 |

### 3.3 面板

#### 工作流列表面板

- 显示所有可用工作流
- 支持过滤和搜索
- 支持执行和编辑

#### 执行监控面板

- 实时显示执行状态
- 节点执行进度
- 日志输出
- 错误信息

#### 工具管理面板

- 工具列表
- 工具详情
- 工具执行

#### 插件管理面板

- 插件列表
- 插件状态
- 插件操作（加载/卸载/重载）

#### 系统状态面板

- 内存使用
- CPU 使用
- 并发任务数
- 缓存命中率

## 4. MCP API

### 4.1 协议版本

- **版本**: 1.0
- **传输**: stdio, TCP

### 4.2 工具定义

#### workflow_execute

执行工作流

**输入 Schema**:
```json
{
  "type": "object",
  "properties": {
    "workflow": {
      "type": "string",
      "description": "工作流文件路径或名称"
    },
    "input": {
      "type": "object",
      "description": "输入参数"
    },
    "timeout": {
      "type": "integer",
      "description": "超时时间（秒）"
    }
  },
  "required": ["workflow"]
}
```

**输出 Schema**:
```json
{
  "type": "object",
  "properties": {
    "execution_id": {
      "type": "string",
      "description": "执行 ID"
    },
    "status": {
      "type": "string",
      "enum": ["success", "failed", "timeout"],
      "description": "执行状态"
    },
    "result": {
      "type": "object",
      "description": "执行结果"
    },
    "error": {
      "type": "string",
      "description": "错误信息"
    }
  }
}
```

#### tool_execute

执行工具

**输入 Schema**:
```json
{
  "type": "object",
  "properties": {
    "tool_name": {
      "type": "string",
      "description": "工具名称"
    },
    "input": {
      "type": "object",
      "description": "输入参数"
    },
    "timeout": {
      "type": "integer",
      "description": "超时时间（秒）"
    }
  },
  "required": ["tool_name"]
}
```

**输出 Schema**:
```json
{
  "type": "object",
  "properties": {
    "status": {
      "type": "string",
      "enum": ["success", "failed", "timeout"],
      "description": "执行状态"
    },
    "result": {
      "type": "object",
      "description": "执行结果"
    },
    "error": {
      "type": "string",
      "description": "错误信息"
    }
  }
}
```

#### workflow_list

列出工作流

**输入 Schema**:
```json
{
  "type": "object",
  "properties": {
    "filter": {
      "type": "string",
      "description": "过滤模式"
    }
  }
}
```

**输出 Schema**:
```json
{
  "type": "object",
  "properties": {
    "workflows": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "path": { "type": "string" },
          "description": { "type": "string" }
        }
      }
    }
  }
}
```

#### tool_list

列出工具

**输入 Schema**:
```json
{
  "type": "object",
  "properties": {
    "category": {
      "type": "string",
      "description": "按类别过滤"
    },
    "tag": {
      "type": "string",
      "description": "按标签过滤"
    }
  }
}
```

**输出 Schema**:
```json
{
  "type": "object",
  "properties": {
    "tools": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "kind": { "type": "string" },
          "description": { "type": "string" },
          "input_schema": { "type": "object" },
          "output_schema": { "type": "object" }
        }
      }
    }
  }
}
```

#### execution_status

查询执行状态

**输入 Schema**:
```json
{
  "type": "object",
  "properties": {
    "execution_id": {
      "type": "string",
      "description": "执行 ID"
    }
  },
  "required": ["execution_id"]
}
```

**输出 Schema**:
```json
{
  "type": "object",
  "properties": {
    "execution_id": { "type": "string" },
    "status": { "type": "string" },
    "progress": { "type": "number" },
    "started_at": { "type": "string" },
    "ended_at": { "type": "string" },
    "result": { "type": "object" },
    "error": { "type": "string" }
  }
}
```

## 5. 错误响应格式

### 5.1 CLI 错误格式

**JSON 格式**:
```json
{
  "error": {
    "code": "WORKFLOW_NOT_FOUND",
    "message": "工作流不存在",
    "details": {
      "workflow": "basic/unknown.yaml"
    }
  }
}
```

**YAML 格式**:
```yaml
error:
  code: WORKFLOW_NOT_FOUND
  message: 工作流不存在
  details:
    workflow: basic/unknown.yaml
```

**Table 格式**:
```
Error: WORKFLOW_NOT_FOUND
Message: 工作流不存在
Details:
  workflow: basic/unknown.yaml
```

### 5.2 MCP 错误格式

```json
{
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": {
      "field": "workflow",
      "reason": "required field missing"
    }
  }
}
```

### 5.3 错误代码

| 代码 | 描述 |
|------|------|
| `WORKFLOW_NOT_FOUND` | 工作流不存在 |
| `WORKFLOW_VALIDATION_FAILED` | 工作流验证失败 |
| `WORKFLOW_EXECUTION_FAILED` | 工作流执行失败 |
| `TOOL_NOT_FOUND` | 工具不存在 |
| `TOOL_EXECUTION_FAILED` | 工具执行失败 |
| `TOOL_TIMEOUT` | 工具执行超时 |
| `PLUGIN_NOT_FOUND` | 插件不存在 |
| `PLUGIN_LOAD_FAILED` | 插件加载失败 |
| `INVALID_INPUT` | 输入参数无效 |
| `PERMISSION_DENIED` | 权限不足 |
| `INTERNAL_ERROR` | 内部错误 |

## 6. 版本管理

### 6.1 API 版本

- CLI API: 遵循语义化版本
- MCP API: 使用 `jsonrpc` 版本字段

### 6.2 兼容性保证

- **MAJOR**: 可能破坏向后兼容性
- **MINOR**: 新增功能，向后兼容
- **PATCH**: Bug 修复，向后兼容

## 7. 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0.0 | 2026-03-01 | 初始版本 | Architecture Team |
