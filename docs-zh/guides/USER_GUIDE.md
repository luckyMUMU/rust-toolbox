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

工作流使用 YAML 格式定义：

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
      input: "${nodes.greet.message}"
```
