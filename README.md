# Rust Toolbox (rt-box)

Rust Toolbox 是一个模块化的工具集合项目，旨在通过统一的接口和工作流引擎，提供可扩展的工具链支持。

## 相关文档

- [架构设计文档](ARCHITECTURE_DESIGN.md): 详细描述项目的架构设计、核心组件和部署架构
- [设计文档](DESIGN.md): 项目的整体设计文档，包括技术选型和核心原则
- [用户指南](USER_GUIDE.md): 详细的用户使用指南，包括工具库和使用方式
- [插件开发指南](PLUGIN_GUIDE.md): 插件开发的规范和指南
- [AI工作规范](AI_WORK_PROTOCOL.md): AI辅助开发的工作规范
- [变更日志](CHANGELOG.md): 项目的变更历史

## 项目结构 (Project Structure)

本项目采用 Cargo Workspace 结构，包含以下核心 Crate：

- **`rt-core`**: 核心库。定义了 `Tool` Trait 以及通用数据结构、插件系统和持久化管理。
- **`rt-tools`**: 内置工具集。包含具体业务逻辑工具。
- **`rt-cli`**: 命令行入口。提供基于命令行的工具列出与运行功能。
- **`rt-gui`**: 图形界面入口。提供可视化的工具配置与运行界面。
- **`rt-plugin-pinyin`**: 插件。提供中文转拼音功能。
- **`rt-plugin-ytdlp`**: 插件。提供视频下载功能。

## 可用工具 (Available Tools)

### File Operations (`file`)
- **`file.move_folder`**: 移动或重命名文件夹（支持移动到现有目录内部）。

### Text Operations (`text`)
- **`text.pinyin`**: 将中文文本转换为带声调或不带声调的拼音。

### Media Operations (`media`)
- **`media.ytdlp`**: 从 YouTube 和其他支持的网站下载视频和音频内容（通过插件实现）。

## 快速开始 (Getting Started)

### 构建项目
```powershell
cargo build
```

### 运行测试
```powershell
cargo test
```

### 运行 CLI
```powershell
cargo run --bin rt-cli -- list
```

### 运行 GUI
```powershell
cargo run --bin rt-gui
```

### 打包 GUI 为可执行文件 (Build EXE)
若要生成独立的 `.exe` 文件以便分发，请使用 release 模式进行构建：

```powershell
# 构建发布版本
cargo build --release --bin rt-gui
```

构建完成后，可执行文件位于：
`target/release/rt-gui.exe`

> **注意**：由于字体文件已嵌入到程序中，生成的 exe 是完全独立的单文件，无需附带 `assets` 目录即可在其他 Windows 机器上运行。

## 开发规范
请参考 [AI_WORK_PROTOCOL.md](AI_WORK_PROTOCOL.md)。

## 许可证
	GNU Affero General Public License v3
