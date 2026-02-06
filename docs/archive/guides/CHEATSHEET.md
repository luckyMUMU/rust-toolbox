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

### 服务器
```bash
workflow-toolkit server --http-port 8080         # 启动MCP服务器
```

### 系统
```bash
workflow-toolkit system monitor --watch          # 监控
workflow-toolkit system health                   # 健康检查
workflow-toolkit system maintenance --recommendations  # 维护建议
```

## 📝 工作流定义 (YAML)

```yaml
name: my-workflow
version: 1.0.0
description: 工作流描述

config:
  retry_policy:
    strategy: exponential
    max_attempts: 3
    base_delay: 1

nodes:
  - id: step1
    tool: echo
    params:
      message: "Hello"
      
  - id: step2
    tool: transform
    params:
      input: "${nodes.step1.output}"
      operation: uppercase

edges:
  - from: step1
    to: step2
```

## 🔧 工具定义

```rust
// Rust原生工具
pub struct MyTool {
    info: ToolInfo,
    executor: Arc<dyn ToolExecutor>,
}

#[async_trait]
impl ToolNode for MyTool {
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        // 执行逻辑
        Ok(result)
    }
}
```

## 🔌 插件定义

```python
# Python插件
def process(params, context):
    return {"output": result}

PLUGIN_INFO = {
    "name": "my-plugin",
    "version": "1.0.0",
    "tools": [{"name": "my_tool", "description": "..."}]
}
```

## ⚙️ 配置

### 文件位置
- `/etc/workflow-toolkit/config.toml` (系统)
- `~/.config/workflow-toolkit/config.toml` (用户)
- `./workflow-toolkit.toml` (项目)

### 环境变量
```bash
export WORKFLOW_TOOLKIT_LOGGING__LEVEL=debug
export WORKFLOW_TOOLKIT_SERVER__HTTP_PORT=8080
export WORKFLOW_TOOLKIT_STORAGE__DATABASE_PATH="./data.db"
```

### 配置示例
```toml
[server]
http_port = 8080
ws_port = 8081

[storage]
database_path = "./data/workflow.db"
cache_size = 104857600

[logging]
level = "info"
format = "Pretty"

[plugins]
plugin_dir = "./plugins"
sandbox_enabled = true

[workflow]
max_concurrent_workflows = 5
default_timeout = 1800
```

## 🎨 TUI 快捷键

| 按键 | 功能 |
|------|------|
| `Tab` | 切换视图 |
| `↑/↓` | 导航 |
| `Enter` | 选择/执行 |
| `Esc` | 返回/取消 |
| `q` | 退出 |
| `r` | 刷新 |
| `h` | 帮助 |
| `m` | 维护模式 |

## 📊 监控命令

```bash
# 实时监控
RUST_LOG=info workflow-toolkit system monitor --watch

# 性能分析
export WORKFLOW_TOOLKIT_PERFORMANCE__ENABLE_PROFILING=true

# 查看日志
tail -f logs/workflow-toolkit.log
```

## 🔍 故障排除

### 编译错误
```bash
cargo check                    # 快速检查
cargo fmt                      # 格式化
cargo clippy -- -D warnings    # Lint检查
```

### 运行时错误
```bash
RUST_LOG=debug workflow-toolkit ...  # 详细日志
workflow-toolkit workflow create --validate-only  # 验证
```

### 测试失败
```bash
cargo test -- --test-threads=1  # 单线程测试
cargo test module::tests        # 特定模块
```

## ✅ 代码规范

### 导入顺序
```rust
use std::...;          // 1. 标准库
use tokio::...;        // 2. 外部依赖
use crate::...;        // 3. 内部模块
```

### 错误处理
```rust
// ✅ 正确
let value = operation().map_err(|e| {
    WorkflowError::workflow_execution(&format!("Failed: {}", e))
})?;

// ❌ 错误
let value = operation().unwrap();
```

### 异步代码
```rust
#[async_trait]
pub trait MyTrait: Send + Sync {
    async fn do_something(&self) -> Result<()>;
}
```

### 测试
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_sync() { }
    
    #[tokio::test]
    async fn test_async() { }
}
```

## 📚 文档索引

| 文档 | 用途 |
|------|------|
| `README.md` | 项目概述 |
| `PROJECT_OVERVIEW.md` | 完整概览 |
| `USER_GUIDE.md` | 用户指南 |
| `DEVELOPMENT_GUIDE.md` | 开发指南 |
| `design.md` | 架构设计 |
| `IMPLEMENTATION_VERIFICATION.md` | 验证报告 |

## 🎯 常见任务

### 添加新工具
1. 实现 `ToolNode` trait
2. 注册到 `ToolRegistry`
3. 在工作流中使用

### 添加新插件
1. 选择插件类型
2. 实现插件接口
3. 配置加载路径
4. 重启或重载

### 调试工作流
1. 启用详细日志: `RUST_LOG=debug`
2. 验证定义: `--validate-only`
3. 查看状态: `workflow status`
4. 检查日志: `tail -f logs/...`

### 性能优化
1. 使用并发执行
2. 启用缓存
3. 批量操作
4. 监控资源使用

## 🔗 链接

- **构建**: `cargo build --release`
- **测试**: `cargo test`
- **文档**: `cargo doc --open`
- **示例**: `cargo run --example ...`

---

**提示**: 使用 `--help` 查看任何命令的详细帮助！
