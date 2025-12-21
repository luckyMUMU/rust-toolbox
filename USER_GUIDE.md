# 用户指南 (User Guide)

## 1. 简介
Rust Toolbox (简称 `rt-box`) 是一个强大的工具流编排平台。

## 1.1 相关文档

- [设计文档](DESIGN.md): 项目的整体设计文档，包括技术选型和核心原则
- [架构设计文档](ARCHITECTURE_DESIGN.md): 详细描述项目的架构设计、核心组件和部署架构
- [插件开发指南](PLUGIN_GUIDE.md): 插件开发的规范和指南
- [AI工作规范](AI_WORK_PROTOCOL.md): AI辅助开发的工作规范
- [变更日志](CHANGELOG.md): 项目的变更历史

## 2. 核心概念
- **工具 (Tool)**: 执行单一任务的原子单元。
- **工作流 (Workflow)**: 串联执行的工具序列。

## 3. 工具库 (Tool Library)

### 3.1 文件操作 (File Operations)

#### 📂 移动文件夹 (`file.move_folder`)
移动或重命名指定的文件夹。

#### 🔍 重复文件查找 (`file.duplicates`)
在指定目录中查找重复文件。

#### 🖼️ 相似图片查找 (`file.similar_images`)
查找视觉上相似的图片。

#### 📁 空目录清理 (`file.empty_directories`)
查找并清理指定的空目录。

#### 🧹 临时文件清理 (`file.temporary_files`)
查找系统或应用产生的临时文件。

#### 🔗 损坏链接检查 (`file.broken_symlinks`)
查找并清理损坏的符号链接。

### 3.2 文本操作 (Text Operations)

#### 🔤 中文转拼音 (`text.pinyin`)
将中文文本转换为带声调或不带声调的拼音。

**行为说明 (Behavior):**
1. **拼音转换**: 将中文文本转换为对应的拼音。
2. **声调控制**: 可选择是否保留声调。
3. **混合文本**: 支持中英文混合文本，只转换中文部分。

**输入参数 (Input):**
```json
{
  "text": "你好世界",      // 要转换的中文文本 (必填)
  "tone": true             // 是否包含声调 (可选, 默认 true)
}
```

**输出 (Output):**
```json
{
  "pinyin": "nǐ hǎo shì jiè" // 转换后的拼音
}
```

#### 🔣 简繁体转换 (`text.convert_chinese`)
转换简体和繁体中文。

**行为说明 (Behavior):**
1. **模式多样**: 支持简体转繁体 (s2t)、繁体转简体 (t2s) 等多种模式。
2. **地区适配**: 支持台湾、香港地区的习惯用词转换。

**输入参数 (Input):**
```json
{
  "text": "简体中文", // 要转换的文本 (必填)
  "mode": "s2t"      // 转换模式 (必填)
}
```

**输出 (Output):**
```json
{
  "converted": "繁體中文"
}
```

#### 🧠 AC 自动机 (`text.ac_automaton`)
高效的多模式匹配工具，适用于关键词过滤、敏感词检测等场景。

**行为说明 (Behavior):**
1. **多模式匹配**: 一次遍历文本即可找到所有匹配的模式串。
2. **状态管理**: 支持添加、删除模式串，并能动态重建。
3. **性能**: 支持并行处理，适合大规模文本匹配。

**输入参数 (Input):**
```json
{
  "action": "match",                  // 操作类型 (必填: add, remove, list, match, save, load)
  "patterns": ["关键词1", "关键词2"], // 模式串列表
  "texts": ["待匹配的长文本..."],     // 待匹配文本列表
  "ignore_case": true,                // 是否忽略大小写 (可选, 默认 false)
  "parallel": false                   // 是否并行匹配 (可选, 默认 false)
}
```

**输出 (Output):**
```json
{
  "success": true,
  "results": [
    {
      "pattern": "关键词1",
      "start": 10,
      "end": 13
    }
  ],
  "elapsed_ms": 15
}
```

### 3.3 媒体操作 (Media Operations)

#### 📹 YouTube 下载器 (`media.ytdlp`)
从 YouTube 和其他支持的网站下载视频和音频内容。

**行为说明 (Behavior):**
1. **单个视频**: 支持下载单个视频。
2. **播放列表**: 支持下载整个播放列表。
3. **格式选择**: 支持多种格式选择。
4. **字幕下载**: 支持字幕下载（包括自动生成字幕）。
5. **自定义输出**: 支持自定义输出目录和文件名。

**输入参数 (Input):**
```json
{
  "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ", // 要下载的视频或播放列表的URL (必填)
  "format": "best",                                  // 要下载的格式 (可选, 默认 best)
  "playlist": false,                                  // 是否下载整个播放列表 (可选, 默认 false)
  "subtitles": false,                                 // 是否下载字幕 (可选, 默认 false)
  "output_dir": ".",                                  // 保存下载文件的目录 (可选, 默认当前目录)
  "filename_template": "%(title)s.%(ext)s"            // 输出文件名的模板 (可选, 默认 %(title)s.%(ext)s)
}
```

**输出 (Output):**
```json
{
  "success": true,
  "files": [
    {
      "path": "./Rick Astley - Never Gonna Give You Up (Official Music Video).mp4",
      "size": 123456789
    }
  ],
  "message": "成功下载了 1 个文件"
}
```


## 4. 使用方式 (Usage)

### 4.1 命令行 (CLI) - `rt-cli`

#### 列出所有工具
```powershell
cargo run --bin rt-cli -- list
```

#### 运行工具
通过 `--input` 参数直接传递 JSON 字符串来运行工具。

**示例 1: 运行 `file.move_folder` 工具**
```powershell
cargo run --bin rt-cli -- run file.move_folder --input '{"source": "./tmp/a", "destination": "./tmp/b"}'
```
*注意：在 PowerShell 中输入 JSON 字符串时，建议使用单引号包裹，避免转义问题。*

**示例 2: 运行 `text.pinyin` 工具**
```powershell
cargo run --bin rt-cli -- run text.pinyin --input '{"text": "你好世界", "tone": true}'
```

#### 工作流管理

##### 运行工作流
```powershell
cargo run --bin rt-cli -- workflow run ./my_workflow.json
```

##### 查看工作流状态
```powershell
cargo run --bin rt-cli -- workflow status <instance_id>
```

##### 暂停工作流
```powershell
cargo run --bin rt-cli -- workflow pause <instance_id>
```

##### 停止工作流
```powershell
cargo run --bin rt-cli -- workflow stop <instance_id>
```

### 4.2 工作流定义

工作流定义采用 JSON 格式，包含节点、边和元数据。每个节点代表一个工具调用，边定义了节点之间的依赖关系。

#### 工作流定义示例
```json
{
  "id": "example_workflow",
  "name": "示例工作流",
  "description": "一个示例工作流，展示了工具链的编排",
  "nodes": [
    {
      "id": "task1",
      "tool_name": "text.pinyin",
      "label": "中文转拼音",
      "input_mappings": {},
      "static_inputs": {
        "text": "你好世界",
        "tone": false
      }
    },
    {
      "id": "task2",
      "tool_name": "file.move_folder",
      "label": "移动文件夹",
      "input_mappings": {
        "source": "{{ task1.output.pinyin }}"
      },
      "static_inputs": {
        "destination": "./tmp/destination",
        "overwrite": true
      }
    }
  ],
  "edges": [
    {
      "from": "task1",
      "to": "task2"
    }
  ]
}
```

#### 工作流定义字段说明

| 字段 | 类型 | 描述 |
|------|------|------|
| `id` | 字符串 | 工作流唯一标识符 |
| `name` | 字符串 | 工作流名称 |
| `description` | 字符串 | 工作流描述 |
| `nodes` | 数组 | 工作流节点列表 |
| `edges` | 数组 | 工作流边列表，定义节点间依赖关系 |

#### 节点字段说明

| 字段 | 类型 | 描述 |
|------|------|------|
| `id` | 字符串 | 节点唯一标识符 |
| `tool_name` | 字符串 | 要调用的工具名称（如 `text.pinyin`） |
| `label` | 字符串 | 节点显示标签（可选） |
| `input_mappings` | 对象 | 输入字段映射，键为输入字段名，值为表达式（如 `{{ task1.output.pinyin }}`） |
| `static_inputs` | 对象 | 静态输入参数，直接传递给工具 |

#### 边字段说明

| 字段 | 类型 | 描述 |
|------|------|------|
| `from` | 字符串 | 源节点 ID |
| `to` | 字符串 | 目标节点 ID |

### 4.3 数据传递与表达式

工作流引擎支持通过表达式在节点间传递数据。表达式使用 `{{ node_id.output.field_path }}` 格式，其中：
- `node_id` 是源节点的 ID
- `field_path` 是源节点输出 JSON 中的字段路径

#### 示例：使用表达式传递数据

```json
{
  "nodes": [
    {
      "id": "http_get",
      "tool_name": "http.get",
      "static_inputs": {
        "url": "https://api.example.com/user/123"
      }
    },
    {
      "id": "file_write",
      "tool_name": "file.write",
      "input_mappings": {
        "content": "{{ http_get.output.body.name }}"
      },
      "static_inputs": {
        "path": "./user_name.txt"
      }
    }
  ],
  "edges": [
    {
      "from": "http_get",
      "to": "file_write"
    }
  ]
}
```

在这个示例中：
1. `http_get` 节点调用 `http.get` 工具获取用户信息
2. `file_write` 节点使用表达式 `{{ http_get.output.body.name }}` 从 `http_get` 节点的输出中提取用户名
3. 最后将用户名写入文件

### 4.4 工作流最佳实践

1. **原子性**：每个节点只做一件事，便于调试和复用
2. **清晰命名**：为节点和工作流使用清晰、描述性的名称
3. **错误处理**：考虑添加错误处理节点，处理可能的失败情况
4. **模块化**：将复杂工作流拆分为多个简单工作流
5. **测试**：在生产环境中使用前，先在测试环境中验证工作流

### 4.5 图形界面 (GUI) - `rt-gui`

#### 启动界面
```powershell
cargo run --bin rt-gui
```

#### 界面操作
1. **左侧列表**: 点击选择要使用的工具（如 `file.move_folder`）。
2. **中间面板**: 
   - 在 "Input (JSON)" 文本框中输入参数。例如：
     ```json
     {
       "source": "D:/tmp/test_src",
       "destination": "D:/tmp/test_dst",
       "overwrite": true
     }
     ```
3. **运行**: 点击 "Run" 按钮。
4. **查看结果**: 底部面板将显示工具执行结果或错误信息。

#### 工作流设计器
1. **创建工作流**: 在 GUI 中打开工作流设计器
2. **添加节点**: 从左侧工具箱拖拽工具到画布上
3. **配置节点**: 点击节点，在右侧属性面板中配置输入参数和映射
4. **连接节点**: 拖动节点间的连线，定义依赖关系
5. **保存工作流**: 点击保存按钮，将工作流保存为 JSON 文件
6. **运行工作流**: 点击运行按钮，启动工作流执行
7. **监控执行**: 在监控面板中查看工作流执行状态和日志

#### 工作流监控
- **实时状态**: 显示工作流的当前状态（运行中、已完成、失败等）
- **节点状态**: 显示每个节点的状态（待执行、运行中、成功、失败）
- **执行日志**: 显示工作流执行过程中的详细日志
- **结果查看**: 点击节点可查看其输入输出数据

### 4.6 工作流示例

#### 示例 1: 备份并转换文件

```json
{
  "id": "backup_and_convert",
  "name": "备份并转换文件",
  "description": "备份文件并转换为不同格式",
  "nodes": [
    {
      "id": "check_file",
      "tool_name": "file.exists",
      "label": "检查文件是否存在",
      "static_inputs": {
        "path": "./source.txt"
      }
    },
    {
      "id": "backup",
      "tool_name": "file.copy",
      "label": "备份文件",
      "input_mappings": {
        "source": "{{ check_file.output.path }}"
      },
      "static_inputs": {
        "destination": "./backup.txt",
        "overwrite": true
      }
    },
    {
      "id": "convert",
      "tool_name": "text.convert_case",
      "label": "转换文件内容",
      "input_mappings": {
        "text": "{{ backup.output.content }}"
      },
      "static_inputs": {
        "case": "uppercase"
      }
    },
    {
      "id": "write_result",
      "tool_name": "file.write",
      "label": "写入结果",
      "input_mappings": {
        "content": "{{ convert.output.converted }}"
      },
      "static_inputs": {
        "path": "./result.txt",
        "overwrite": true
      }
    }
  ],
  "edges": [
    {
      "from": "check_file",
      "to": "backup"
    },
    {
      "from": "backup",
      "to": "convert"
    },
    {
      "from": "convert",
      "to": "write_result"
    }
  ]
}
```

#### 示例 2: 批量下载并处理视频

```json
{
  "id": "video_processing",
  "name": "视频处理工作流",
  "description": "批量下载 YouTube 视频并转换格式",
  "nodes": [
    {
      "id": "download_list",
      "tool_name": "http.get",
      "label": "获取视频列表",
      "static_inputs": {
        "url": "https://api.example.com/videos"
      }
    },
    {
      "id": "download_video",
      "tool_name": "media.ytdlp",
      "label": "下载视频",
      "input_mappings": {
        "url": "{{ download_list.output.body.videos[0].url }}"
      },
      "static_inputs": {
        "output_dir": "./videos",
        "format": "best"
      }
    },
    {
      "id": "convert_format",
      "tool_name": "media.ffmpeg",
      "label": "转换视频格式",
      "input_mappings": {
        "input_path": "{{ download_video.output.files[0].path }}"
      },
      "static_inputs": {
        "output_path": "./converted.mp4",
        "output_format": "mp4"
      }
    }
  ],
  "edges": [
    {
      "from": "download_list",
      "to": "download_video"
    },
    {
      "from": "download_video",
      "to": "convert_format"
    }
  ]
}
```

### 4.7 工作流执行流程

1. **解析定义**: 引擎解析工作流 JSON 定义，验证其完整性和正确性
2. **构建依赖图**: 根据边定义构建有向无环图 (DAG)
3. **初始化状态**: 创建工作流实例，初始化节点状态
4. **执行节点**: 按照依赖顺序执行节点：
   - 找出所有无依赖的节点，并行执行
   - 当节点完成后，更新状态并触发依赖节点的执行
   - 重复直到所有节点执行完成或某个节点失败
5. **更新状态**: 更新工作流和节点状态
6. **生成结果**: 收集所有节点的输出，生成最终结果

### 4.8 工作流状态

| 状态 | 描述 |
|------|------|
| `Pending` | 工作流已创建，但尚未开始执行 |
| `Running` | 工作流正在执行中 |
| `Paused` | 工作流已暂停，可通过命令恢复执行 |
| `Completed` | 工作流已成功完成 |
| `Failed` | 工作流执行失败，包含失败原因 |

### 4.9 节点状态

| 状态 | 描述 |
|------|------|
| `Pending` | 节点已准备好执行，但依赖节点尚未完成 |
| `Running` | 节点正在执行中 |
| `Completed` | 节点执行成功 |
| `Failed` | 节点执行失败，包含失败原因 |
| `Skipped` | 节点被跳过执行 |

## 5. Model Context Protocol (MCP) 支持

Rust Toolbox 实现了 Model Context Protocol (MCP)，支持外部系统通过标准化接口调用工具和工作流。

### 5.1 MCP 核心概念
- **MCP 上下文**: 包含执行状态、历史记录和环境信息的上下文对象
- **MCP 请求**: 标准化的工具调用格式
- **MCP 响应**: 标准化的执行结果格式
- **MCP 服务器**: 提供 REST API 和 WebSocket 端点

### 5.2 启动 MCP 服务器

#### 命令行方式
```powershell
cargo run --bin rt-cli -- mcp-server start --address 127.0.0.1 --port 8000
```

#### 配置选项
- `--address`: 服务器监听地址（默认：127.0.0.1）
- `--port`: 服务器监听端口（默认：8000）
- `--max-request-size`: 最大请求大小（默认：10MB）
- `--enable-websocket`: 启用 WebSocket 支持（默认：true）

### 5.3 REST API 端点

#### 健康检查
```
GET /health
```
**响应示例**:
```json
{ "status": "ok", "service": "mcp-server" }
```

#### 获取工具列表
```
GET /tools
```
**响应示例**:
```json
[
  {
    "name": "text.pinyin",
    "display_name": "中文转拼音",
    "description": "将中文文本转换为拼音",
    "mcp_supported": true,
    "type": "core"
  }
]
```

#### 获取 MCP 支持的工具列表
```
GET /tools/mcp
```
**响应示例**:
```json
[
  {
    "name": "text.pinyin",
    "display_name": "中文转拼音",
    "description": "将中文文本转换为拼音",
    "mcp_supported": true,
    "type": "core"
  }
]
```

#### 调用工具
```
POST /tools/{name}/call
Content-Type: application/json
```
**请求示例**:
```json
{
  "text": "你好世界",
  "tone": true
}
```
**响应示例**:
```json
{
  "success": true,
  "data": { "pinyin": "nǐ hǎo shì jiè" }
}
```

#### MCP 调用端点
```
POST /mcp/call
Content-Type: application/json
```
**请求示例**:
```json
{
  "id": "req-12345",
  "component_type": "Tool",
  "component_name": "text.pinyin",
  "method": "run",
  "params": {
    "text": "你好世界",
    "tone": true
  },
  "context": {
    "id": "ctx-12345",
    "parent_id": null,
    "model_state": {},
    "execution_history": [],
    "environment_info": {},
    "metadata": {}
  },
  "service_context": {
    "caller_id": "test-caller",
    "caller_type": "User",
    "permission_level": "Standard",
    "extra": {}
  }
}
```
**响应示例**:
```json
{
  "id": "resp-67890",
  "request_id": "req-12345",
  "status": "Success",
  "data": { "pinyin": "nǐ hǎo shì jiè" },
  "error": null,
  "context": {
    "id": "ctx-12345",
    "parent_id": null,
    "model_state": {},
    "execution_history": [
      {
        "id": "exec-54321",
        "timestamp": "2025-12-14T08:00:00Z",
        "component_type": "Tool",
        "component_name": "text.pinyin",
        "method": "run",
        "input": { "text": "你好世界", "tone": true },
        "output": { "pinyin": "nǐ hǎo shì jiè" },
        "status": "Success",
        "error": null,
        "duration_ms": 123
      }
    ],
    "environment_info": {},
    "metadata": {}
  },
  "duration_ms": 123
}
```

### 5.4 WebSocket 支持

#### 连接 WebSocket
```
ws://localhost:8000/ws/mcp
```

#### WebSocket 消息格式
- **请求消息**: 与 `/mcp/call` 端点的请求格式相同
- **响应消息**: 与 `/mcp/call` 端点的响应格式相同

#### WebSocket 示例
```javascript
// 使用 JavaScript 连接 WebSocket
const socket = new WebSocket('ws://localhost:8000/ws/mcp');

socket.onopen = () => {
  console.log('WebSocket connected');
  
  // 发送 MCP 请求
  const request = {
    "id": "req-12345",
    "component_type": "Tool",
    "component_name": "text.pinyin",
    "method": "run",
    "params": {
      "text": "你好世界",
      "tone": true
    },
    "context": {
      "id": "ctx-12345",
      "parent_id": null,
      "model_state": {},
      "execution_history": [],
      "environment_info": {},
      "metadata": {}
    },
    "service_context": {
      "caller_id": "js-client",
      "caller_type": "User",
      "permission_level": "Standard",
      "extra": {}
    }
  };
  
  socket.send(JSON.stringify(request));
};

socket.onmessage = (event) => {
  const response = JSON.parse(event.data);
  console.log('WebSocket response:', response);
};
```

### 5.5 MCP 客户端示例

#### 使用 curl 调用 MCP API
```bash
curl -X POST http://localhost:8000/mcp/call \
  -H "Content-Type: application/json" \
  -d '{"id":"req-123","component_type":"Tool","component_name":"text.pinyin","method":"run","params":{"text":"你好世界","tone":true},"context":{"id":"ctx-123","parent_id":null,"model_state":{},"execution_history":[],"environment_info":{},"metadata":{}},"service_context":{"caller_id":"curl-client","caller_type":"User","permission_level":"Standard","extra":{}}}'
```

#### 使用 Python 调用 MCP API
```python
import requests
import json

url = "http://localhost:8000/mcp/call"
headers = {"Content-Type": "application/json"}

request_data = {
    "id": "req-123",
    "component_type": "Tool",
    "component_name": "text.pinyin",
    "method": "run",
    "params": {
        "text": "你好世界",
        "tone": True
    },
    "context": {
        "id": "ctx-123",
        "parent_id": None,
        "model_state": {},
        "execution_history": [],
        "environment_info": {},
        "metadata": {}
    },
    "service_context": {
        "caller_id": "python-client",
        "caller_type": "User",
        "permission_level": "Standard",
        "extra": {}
    }
}

response = requests.post(url, headers=headers, data=json.dumps(request_data))
print(response.json())
```

## 6. 插件管理 (Plugin Management)

Rust Toolbox 支持通过外部插件扩展功能。

### 6.1 安装插件
1.  获取插件的可执行文件（例如 `rt-plugin-custom.exe`）。
2.  在 `rt-cli` 或 `rt-gui` 的同级目录下创建一个名为 `plugins` 的文件夹。
3.  将插件可执行文件放入 `plugins` 文件夹中。
4.  重启 `rt-cli` 或 `rt-gui`，工具将自动扫描并加载以 `rt-plugin-` 开头的插件。

### 6.2 验证安装
使用 `list` 命令查看已加载的工具：
```powershell
cargo run --bin rt-cli -- list
```
如果插件加载成功，您将在列表中看到插件提供的工具。
