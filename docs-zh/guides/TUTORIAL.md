# 工作流工具包教程

本教程将引导您从基础到高级使用工作流工具包，包括安装、配置、创建工作流、开发插件等各个方面。

## 目录

1. [快速开始](#快速开始)
2. [基础概念](#基础概念)
3. [创建第一个工作流](#创建第一个工作流)
4. [工具开发](#工具开发)
5. [插件开发](#插件开发)
6. [高级特性](#高级特性)
7. [最佳实践](#最佳实践)
8. [故障排除](#故障排除)

## 快速开始

### 安装

#### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/your-org/workflow-toolkit.git
cd workflow-toolkit

# 构建项目
cargo build --release

# 运行测试
cargo test

# 安装到系统
cargo install --path .
```

#### 使用预编译二进制

```bash
# 下载最新版本
wget https://github.com/your-org/workflow-toolkit/releases/latest/download/workflow-toolkit-linux-x64.tar.gz

# 解压并安装
tar -xzf workflow-toolkit-linux-x64.tar.gz
sudo mv workflow-toolkit /usr/local/bin/
```

### 验证安装

```bash
workflow-toolkit --version
```
