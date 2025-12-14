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

#### 📂 移动文件夹 (`file.move_folder`)
移动或重命名指定的文件夹。

**行为说明 (Behavior):**
1. **重命名/移动**: 如果 `destination` 不存在，源文件夹将被重命名或移动到该路径。
2. **移动到内部**: 如果 `destination` 是一个已存在的目录，源文件夹将被移动到该目录**内部**。

**输入参数 (Input):**
```json
{
  "source": "path/to/source_folder",      // 源路径 (必填)
  "destination": "path/to/target_folder", // 目标路径 (必填)
  "overwrite": false                      // 是否覆盖 (可选, 默认 false)。
                                          // 如果为 true 且目标路径(计算后)已存在，将先删除目标再移动。
}
```

**输出 (Output):**
```json
{
  "success": true,
  "moved_files": 0 // 移动的文件/项目数量
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
cargo run --bin rt-cli -- run file.move_folder --input '{\"source\": \"./tmp/a\", \"destination\": \"./tmp/b\"}'
```
*注意：在 PowerShell 中输入 JSON 字符串时，建议使用单引号包裹，避免转义问题。*

**示例 2: 运行 `text.pinyin` 工具**
```powershell
cargo run --bin rt-cli -- run text.pinyin --input '{\"text\": \"你好世界\", \"tone\": true}'
```

#### 运行工作流
```powershell
cargo run --bin rt-cli -- workflow run ./my_workflow.json
```

**工作流示例 (my_workflow.json):**
```json
{
  "name": "example_workflow",
  "description": "一个示例工作流，展示了工具链的编排",
  "tasks": {
    "task1": {
      "tool": "text.pinyin",
      "input": {
        "text": "你好世界",
        "tone": false
      }
    },
    "task2": {
      "tool": "file.move_folder",
      "input": {
        "source": "./tmp/source",
        "destination": "./tmp/destination",
        "overwrite": true
      }
    }
  },
  "dependencies": {
    "task2": ["task1"]
  }
}
```

这个工作流定义了两个任务：
1. `task1`: 使用 `text.pinyin` 工具将中文文本转换为拼音
2. `task2`: 使用 `file.move_folder` 工具移动文件夹

依赖关系 `task2: ["task1"]` 表示 `task2` 将在 `task1` 完成后执行。

### 4.2 图形界面 (GUI) - `rt-gui`

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

## 5. 插件管理 (Plugin Management)

Rust Toolbox 支持通过外部插件扩展功能。

### 5.1 安装插件
1.  获取插件的可执行文件（例如 `rt-plugin-custom.exe`）。
2.  在 `rt-cli` 或 `rt-gui` 的同级目录下创建一个名为 `plugins` 的文件夹。
3.  将插件可执行文件放入 `plugins` 文件夹中。
4.  重启 `rt-cli` 或 `rt-gui`，工具将自动扫描并加载以 `rt-plugin-` 开头的插件。

### 5.2 验证安装
使用 `list` 命令查看已加载的工具：
```powershell
cargo run --bin rt-cli -- list
```
如果插件加载成功，您将在列表中看到插件提供的工具。
