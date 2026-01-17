# Workflow Toolkit - 快速参考卡

## 🚀 快速开始

```bash
# 构建
cargo build --release

# 基本命令
workflow-toolkit --help
workflow-toolkit workflow execute file.yaml
workflow-toolkit tui
```

## 📋 核心命令

### 工作流
```bash
workflow-toolkit workflow create <file>          # 创建
workflow-toolkit workflow execute <name>         # 执行
workflow-toolkit workflow list                   # 列表
workflow-toolkit workflow status <id>            # 状态
workflow-toolkit workflow pause <id>             # 暂停
workflow-toolkit workflow resume <id>            # 恢复
workflow-toolkit workflow stop <id>              # 停止
```

### 工具
```bash
workflow-toolkit tool list                       # 列出工具
workflow-toolkit tool execute <name> --params '{}'  # 执行
```

### 插件
```bash
workflow-toolkit plugin list                     # 列出插件
workflow-toolkit plugin install <path>           # 安装
workflow-toolkit plugin reload <name>            # 重载
```

### 批量
```bash
workflow-toolkit batch execute <list.json>       # 批量执行
```

### TUI
```bash
workflow-toolkit tui                             # 启动TUI
```
