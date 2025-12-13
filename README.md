# Rust Toolbox (rt-box)

Rust Toolbox 是一个模块化的工具集合项目，旨在通过统一的接口和工作流引擎，提供可扩展的工具链支持。

## 项目结构 (Project Structure)

本项目采用 Cargo Workspace 结构，包含以下核心 Crate：

- **`rt-core`**: 核心库。定义了 `Tool` Trait 以及通用数据结构。
- **`rt-tools`**: 内置工具集。包含具体业务逻辑工具。
- **`rt-cli`**: 命令行入口。提供基于命令行的工具列出与运行功能。
- **`rt-gui`**: 图形界面入口。提供可视化的工具配置与运行界面。

## 可用工具 (Available Tools)

### File Operations (`file`)
- **`file.move_folder`**: 移动或重命名文件夹（支持移动到现有目录内部）。

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

## 开发规范
请参考 [AI_WORK_PROTOCOL.md](AI_WORK_PROTOCOL.md)。

## 许可证
MIT License
