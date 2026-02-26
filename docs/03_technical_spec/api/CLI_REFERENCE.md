# CLI参考

> **工作流工具包命令行接口完整参考**  
> *最后更新：2026-02-26*

## 概述

工作流工具包CLI（`workflow-toolkit`）为管理工作流、工具、插件和系统服务提供统一接口。

## 命令结构

```bash
workflow-toolkit [全局选项] <命令> [子命令] [参数]
```

## 全局选项

| 选项 | 描述 | 默认值 |
|--------|-------------|---------|
| `--config <文件>` | 配置文件路径 | `config/default.toml` |
| `--log-level <级别>` | 设置日志级别（trace, debug, info, warn, error） | `info` |
| `--output <格式>` | 输出格式（table, json, yaml, text） | `table` |
| `--verbose`, `-v` | 启用详细输出 | `false` |
| `--quiet`, `-q` | 抑制非错误输出 | `false` |
| `--help`, `-h` | 显示帮助信息 | - |
| `--version`, `-V` | 显示版本信息 | - |

---

## 工作流管理（`workflow`）

管理工作流定义和执行。

### `workflow create`
创建或验证工作流定义。

```bash
workflow-toolkit workflow create [选项] <定义文件>
```

**选项：**
- `--validate-only`：仅验证定义而不保存
- `--force`：强制覆盖现有工作流

**示例：**
```bash
# 验证工作流文件
workflow-toolkit workflow create --validate-only workflows/basic/hello-world.yaml

# 创建/导入工作流
workflow-toolkit workflow create examples/my-workflow.yaml
```

### `workflow execute`
执行工作流。

```bash
workflow-toolkit workflow execute [选项] <工作流名称>
```

**选项：**
- `--params <文件>`：参数文件路径（JSON/YAML）
- `--params-json <JSON>`：JSON字符串形式的参数
- `--background`：后台执行
- `--wait`：等待完成并显示进度
- `--timeout <秒>`：执行超时时间

**示例：**
```bash
# 使用默认参数执行
workflow-toolkit workflow execute hello-world

# 使用JSON参数执行
workflow-toolkit workflow execute hello-world --params-json '{"name": "Alice"}' --wait
```

### `workflow status`
获取工作流执行状态。

```bash
workflow-toolkit workflow status [选项] <工作流ID>
```

**选项：**
- `--detailed`：显示详细的节点状态
- `--follow`：实时跟踪状态更新
- `--interval <秒>`：跟踪模式的刷新间隔（默认：2）

**示例：**
```bash
# 检查状态
workflow-toolkit workflow status 1234-5678

# 监视状态更新
workflow-toolkit workflow status 1234-5678 --follow --detailed
```

### `workflow pause` / `resume` / `stop`
控制工作流执行。

```bash
workflow-toolkit workflow pause <工作流ID>
workflow-toolkit workflow resume <工作流ID>
workflow-toolkit workflow stop <工作流ID> [--force]
```

**示例：**
```bash
# 暂停正在执行的工作流
workflow-toolkit workflow pause 1234-5678-abcd

# 恢复暂停的工作流
workflow-toolkit workflow resume 1234-5678-abcd

# 强制停止工作流（立即终止）
workflow-toolkit workflow stop 1234-5678-abcd --force

# 正常停止工作流（等待当前节点完成）
workflow-toolkit workflow stop 1234-5678-abcd
```

### `workflow list`
列出工作流。

```bash
workflow-toolkit workflow list [选项]
```

**选项：**
- `--status <状态>`：按执行状态过滤（Pending, Running等）
- `--recent`：仅显示最近的执行
- `--limit <N>`：最大结果数量（默认：50）

**示例：**
```bash
# 列出所有工作流执行记录
workflow-toolkit workflow list

# 仅显示最近10条记录
workflow-toolkit workflow list --recent --limit 10

# 筛选正在执行的工作流
workflow-toolkit workflow list --status Running

# 筛选失败的工作流
workflow-toolkit workflow list --status Failed --limit 20
```

---

## 工具管理（`tool`）

检查和执行单个工具。

### `tool list`
列出可用工具。

```bash
workflow-toolkit tool list [选项]
```

**选项：**
- `--category <类别>`：按类别过滤
- `--tag <标签>`：按标签过滤
- `--search <查询>`：按名称或描述搜索
- `--detailed`：显示详细信息

**示例：**
```bash
# 列出所有工具
workflow-toolkit tool list

# 搜索文件工具
workflow-toolkit tool list --search "file"
```

### `tool execute`
直接执行特定工具。

```bash
workflow-toolkit tool execute [选项] <工具名称>
```

**选项：**
- `--params <JSON>`：JSON字符串形式的工具参数
- `--params-file <文件>`：参数文件路径
- `--timeout <秒>`：执行超时时间
- `--dry-run`：验证参数而不执行

**示例：**
```bash
# 执行echo工具
workflow-toolkit tool execute echo --params '{"message": "Hello"}'

# 执行文件读取
workflow-toolkit tool execute file_read --params '{"path": "data.txt"}'
```

### `tool info`
显示工具的详细信息。

```bash
workflow-toolkit tool info <工具名称>
```

---

## 插件管理（`plugin`）

管理扩展和插件。

### `plugin install`
安装新插件。

```bash
workflow-toolkit plugin install [选项] <插件路径>
```

**选项：**
- `--plugin-type <类型>`：显式设置插件类型（native, python, nodejs, docker, wasm）
- `--force`：如果存在则重新安装

**示例：**
```bash
# 安装本地Python插件
workflow-toolkit plugin install ./plugins/my-python-plugin

# 从URL安装（如果支持）
workflow-toolkit plugin install https://example.com/plugins/my-plugin.zip
```

### `plugin list`
列出已安装的插件。

```bash
workflow-toolkit plugin list [选项]
```

**选项：**
- `--detailed`：显示详细信息
- `--plugin-type <类型>`：按类型过滤

**示例：**
```bash
# 列出所有插件
workflow-toolkit plugin list

# 显示详细信息
workflow-toolkit plugin list --detailed

# 仅列出 Python 插件
workflow-toolkit plugin list --plugin-type python

# 以 JSON 格式输出
workflow-toolkit plugin list --detailed --output json
```

### `plugin reload` / `uninstall`
管理插件生命周期。

```bash
workflow-toolkit plugin reload <插件名称>
workflow-toolkit plugin uninstall <插件名称> [--force]
```

**示例：**
```bash
# 重新加载插件（更新代码后）
workflow-toolkit plugin reload my-python-plugin

# 卸载插件
workflow-toolkit plugin uninstall old-plugin

# 强制卸载（即使有依赖）
workflow-toolkit plugin uninstall stubborn-plugin --force
```

### `plugin info`
显示插件的详细信息。

```bash
workflow-toolkit plugin info <插件名称>
```

**示例：**
```bash
# 查看插件详细信息
workflow-toolkit plugin info file-management

# 以 YAML 格式输出
workflow-toolkit plugin info file-management --output yaml
```

---

## 批处理操作（`batch`）

批量执行多个工作流。

### `batch execute`
从列表文件执行工作流。

```bash
workflow-toolkit batch execute [选项] <工作流列表文件>
```

**选项：**
- `--parallel <N>`：最大并行执行数（默认：4）
- `--continue-on-failure`：即使某些工作流失败也继续执行
- `--output-dir <目录>`：保存结果的目录
- `--timeout <秒>`：每个工作流的超时时间

**示例：**
```bash
# 使用8个并行工作器运行批处理
workflow-toolkit batch execute batch-jobs.yaml --parallel 8
```

---

## 其他命令

### `tui`
启动终端用户界面。

```bash
workflow-toolkit tui
```

**示例：**
```bash
# 启动 TUI 界面
workflow-toolkit tui

# 使用特定配置文件启动
workflow-toolkit tui --config custom-config.toml
```

### `server`
启动MCP（模型上下文协议）服务器。

```bash
workflow-toolkit server [选项]
```

**选项：**
- `--http-port <端口>`：HTTP端口（默认：8080）
- `--ws-port <端口>`：WebSocket端口（默认：8081）
- `--auth`：启用认证

**示例：**
```bash
# 使用默认端口启动服务器
workflow-toolkit server

# 自定义端口启动
workflow-toolkit server --http-port 9000 --ws-port 9001

# 启用认证启动
workflow-toolkit server --auth

# 后台运行（结合 nohup）
nohup workflow-toolkit server > server.log 2>&1 &
```

### `completion`
生成shell补全脚本。

```bash
workflow-toolkit completion <SHELL>
```

**支持的Shell：** bash, zsh, fish, powershell

**示例：**
```bash
# 生成PowerShell补全
workflow-toolkit completion powershell > completion.ps1
```
