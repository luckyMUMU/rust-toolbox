# YouTube Downloader Plugin

## 概述

YouTube Downloader 插件是一个基于 Rust Toolbox 框架的插件，它集成了 `yt-dlp` 的核心功能，允许用户从 YouTube 和其他支持的网站下载视频和音频内容。

## 功能特性

- ✅ 下载单个视频
- ✅ 下载整个播放列表
- ✅ 支持多种格式选择
- ✅ 支持字幕下载（包括自动生成字幕）
- ✅ 自定义输出目录和文件名
- ✅ 完整的多语言支持（英文和简体中文）
- ✅ 结构化的输入输出
- ✅ 详细的日志记录

## 安装要求

1. **Rust Toolbox**：需要安装 Rust Toolbox 宿主程序（`rt-cli` 或 `rt-gui`）
2. **yt-dlp**：需要在系统路径中安装 `yt-dlp` 工具
   - 安装方法：`pip install yt-dlp`（需要 Python 3.7+）
3. **FFmpeg**（可选）：用于视频格式转换和合并

## 安装步骤

1. **克隆仓库**：
   ```bash
git clone https://github.com/your-username/rust-tool.git
cd rust-tool
   ```

2. **构建插件**：
   ```bash
cargo build --release --package rt-plugin-ytdlp
   ```

3. **部署插件**：
   将编译好的插件可执行文件复制到 `plugins` 目录：
   ```bash
cp target/release/rt-plugin-ytdlp.exe plugins/
   ```

## 使用方法

### 通过命令行界面（CLI）

1. **查看插件规范**：
   ```bash
rt-cli plugin spec --name media.ytdlp
   ```

2. **运行插件**：
   ```bash
rt-cli plugin run --name media.ytdlp --input-file input.json
   ```

### 通过图形用户界面（GUI）

1. 启动 `rt-gui`
2. 在插件列表中找到 "YouTube下载器"
3. 填写输入参数
4. 点击 "运行" 按钮开始下载

## 输入参数

| 参数名 | 类型 | 默认值 | 描述 |
|--------|------|--------|------|
| **URL** | 字符串 | 必填 | 要下载的视频或播放列表的URL |
| **格式** | 字符串 | `best` | 要下载的格式（例如：`best`, `bestvideo+bestaudio`, `bestaudio`） |
| **播放列表** | 布尔值 | `false` | 是否下载整个播放列表 |
| **字幕** | 布尔值 | `false` | 是否下载字幕 |
| **输出目录** | 字符串 | `.` | 保存下载文件的目录 |
| **文件名模板** | 字符串 | `%(title)s.%(ext)s` | 输出文件名的模板 |

## 输出结果

| 字段名 | 类型 | 描述 |
|--------|------|------|
| **success** | 布尔值 | 下载是否成功 |
| **files** | 数组 | 下载的文件列表，包含文件路径和大小 |
| **message** | 字符串 | 描述结果的消息 |

## 示例

### 输入示例（input.json）

```json
{
  "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
  "format": "best",
  "playlist": false,
  "subtitles": false,
  "output_dir": "./downloads",
  "filename_template": "%(title)s.%(ext)s"
}
```

### 输出示例

```json
{
  "success": true,
  "files": [
    {
      "path": "./downloads/Rick Astley - Never Gonna Give You Up (Official Music Video).mp4",
      "size": 123456789
    }
  ],
  "message": "成功下载了 1 个文件"
}
```

## 配置

### 日志配置

插件使用 `log4rs` 进行日志记录。如果在当前目录下存在 `log4rs.yml` 文件，将使用该文件进行配置。否则，将使用默认的日志配置。

## 开发

### 目录结构

```
rt-plugin-ytdlp/
├── Cargo.toml              # 插件依赖配置
├── src/
│   ├── main.rs            # 插件核心逻辑
│   └── i18n.rs            # 多语言支持
├── locales/
│   ├── tool.en.json       # 英文翻译
│   └── tool.zh.json       # 中文翻译
├── README.md              # 插件说明文档
└── input.json             # 测试用输入示例
```

### 测试

运行测试：
```bash
cargo test --package rt-plugin-ytdlp
```

### 构建调试版本

```bash
cargo build --package rt-plugin-ytdlp
```

### 构建发布版本

```bash
cargo build --release --package rt-plugin-ytdlp
```

## 许可证

本插件采用 MIT 许可证。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 支持的网站

该插件支持 `yt-dlp` 支持的所有网站，包括但不限于：

- YouTube
- Bilibili
- 抖音
- 优酷
- 腾讯视频
- 爱奇艺
- SoundCloud
- Twitter

完整列表请查看 [yt-dlp 支持的网站](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md)。

## 故障排除

1. **yt-dlp 未找到**：确保 `yt-dlp` 已安装并添加到系统路径中
2. **下载失败**：检查 URL 是否有效，或者尝试使用不同的格式
3. **权限错误**：确保输出目录存在且有写入权限
4. **日志查看**：查看日志文件以获取详细的错误信息

## 联系方式

如有问题或建议，请通过以下方式联系：

- 提交 GitHub Issue
- 发送邮件到：your-email@example.com

## 更新日志

### v0.1.0 (2025-12-13)

- 初始版本发布
- 支持基本视频下载
- 支持播放列表下载
- 支持格式选择
- 支持字幕下载
- 完整的多语言支持

## 未来计划

- [ ] 支持更多 yt-dlp 高级功能
- [ ] 提供更详细的错误处理
- [ ] 支持进度跟踪
- [ ] 支持批量下载
- [ ] 添加更多测试用例
- [ ] 支持更多输出格式

---

**Enjoy downloading! 🎉**