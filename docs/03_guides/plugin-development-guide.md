# Workflow Toolkit 插件开发指南

## 文档元数据

- **版本**: v1.0.0
- **最后更新**: 2026-03-01
- **状态**: 已批准

## 1. 插件概述

### 1.1 什么是插件

插件是 Workflow Toolkit 的扩展机制，用于：
- 添加新的工具功能
- 扩展工作流能力
- 集成外部系统

### 1.2 插件类型

| 类型 | 描述 | 适用场景 |
|------|------|----------|
| Native | Rust 动态库 | 高性能、系统级操作 |
| Python | Python 脚本 | 快速开发、数据处理 |
| Node.js | Node.js 脚本 | JavaScript 生态集成 |
| Docker | Docker 容器 | 隔离环境、复杂依赖 |
| WASM | WebAssembly 模块 | 安全沙箱、跨平台 |

## 2. 插件结构

### 2.1 目录结构

```
my-plugin/
├── plugin.yaml         # 插件清单
├── libmy_plugin.so     # Native 插件（Linux）
├── my_plugin.dll       # Native 插件（Windows）
├── main.py             # Python 插件入口
├── index.js            # Node.js 插件入口
├── Dockerfile          # Docker 插件
├── module.wasm         # WASM 插件
└── README.md           # 插件文档
```

### 2.2 插件清单 (plugin.yaml)

```yaml
apiVersion: plugin.kit/v1
kind: Plugin
metadata:
  name: my-plugin
  version: "1.0.0"
  description: 我的自定义插件
  author: Your Name
  tags:
    - custom
    - example
spec:
  type: native|python|nodejs|docker|wasm
  entry: libmy_plugin.so|main.py|index.js|Dockerfile|module.wasm
  runtime:
    python_version: "3.10"
    node_version: "18"
  tools:
    - name: my_tool
      description: 我的工具
      kind: processor
      input_schema:
        type: object
        properties:
          input:
            type: string
            description: 输入参数
        required: [input]
      output_schema:
        type: object
        properties:
          result:
            type: string
            description: 输出结果
  security:
    level: basic
    file_permissions:
      read_paths:
        - /data/input
      write_paths:
        - /data/output
    network_permissions:
      enabled: false
  resources:
    max_memory: 256MB
    max_cpu: 0.5
    max_execution_time: 60s
```

## 3. Native 插件开发

### 3.1 项目结构

```
my-native-plugin/
├── Cargo.toml
├── src/
│   └── lib.rs
└── plugin.yaml
```

### 3.2 Cargo.toml

```toml
[package]
name = "my-native-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
workflow-toolkit-plugin = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["rt-multi-thread"] }
```

### 3.3 插件实现

```rust
use workflow_toolkit_plugin::*;
use serde::{Deserialize, Serialize};

// 插件结构体
pub struct MyPlugin {
    info: PluginInfo,
}

// 输入参数
#[derive(Deserialize)]
struct MyToolInput {
    input: String,
}

// 输出结果
#[derive(Serialize)]
struct MyToolOutput {
    result: String,
}

// 实现 Plugin trait
#[async_trait]
impl Plugin for MyPlugin {
    async fn initialize(&mut self, _config: &PluginConfig) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_tools(&self) -> Vec<Tool> {
        vec![
            Tool::native("my_tool")
                .description("我的工具")
                .input_schema(json!({
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": "输入参数"
                        }
                    },
                    "required": ["input"]
                }))
                .output_schema(json!({
                    "type": "object",
                    "properties": {
                        "result": {
                            "type": "string",
                            "description": "输出结果"
                        }
                    }
                }))
                .handler(|input: MyToolInput| {
                    Ok(MyToolOutput {
                        result: format!("处理结果: {}", input.input),
                    })
                })
                .build(),
        ]
    }

    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn status(&self) -> PluginStatus {
        PluginStatus::Loaded
    }
}

// 插件入口
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    let plugin = Box::new(MyPlugin {
        info: PluginInfo {
            id: PluginId::new("my-plugin"),
            name: "my-plugin".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Native,
        },
    });
    Box::into_raw(plugin) as *mut dyn Plugin
}
```

### 3.4 编译

```bash
# Linux
cargo build --release
cp target/release/libmy_native_plugin.so .

# Windows
cargo build --release
cp target/release/my_native_plugin.dll .
```

## 4. Python 插件开发

### 4.1 项目结构

```
my-python-plugin/
├── main.py
├── requirements.txt
└── plugin.yaml
```

### 4.2 插件实现

```python
#!/usr/bin/env python3
import json
import sys
from typing import Dict, Any

class MyPlugin:
    def __init__(self):
        self.info = {
            "id": "my-python-plugin",
            "name": "my-python-plugin",
            "version": "1.0.0",
            "type": "python"
        }
    
    def initialize(self, config: Dict[str, Any]) -> bool:
        return True
    
    def shutdown(self) -> bool:
        return True
    
    def get_tools(self) -> list:
        return [
            {
                "name": "my_tool",
                "description": "我的工具",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": "输入参数"
                        }
                    },
                    "required": ["input"]
                },
                "output_schema": {
                    "type": "object",
                    "properties": {
                        "result": {
                            "type": "string",
                            "description": "输出结果"
                        }
                    }
                }
            }
        ]
    
    def execute(self, tool_name: str, input_data: Dict[str, Any]) -> Dict[str, Any]:
        if tool_name == "my_tool":
            return {
                "result": f"处理结果: {input_data.get('input', '')}"
            }
        raise ValueError(f"Unknown tool: {tool_name}")

def main():
    plugin = MyPlugin()
    
    for line in sys.stdin:
        try:
            request = json.loads(line)
            command = request.get("command")
            
            if command == "initialize":
                result = plugin.initialize(request.get("config", {}))
            elif command == "shutdown":
                result = plugin.shutdown()
            elif command == "get_tools":
                result = plugin.get_tools()
            elif command == "execute":
                result = plugin.execute(
                    request.get("tool_name"),
                    request.get("input", {})
                )
            else:
                result = {"error": f"Unknown command: {command}"}
            
            print(json.dumps(result), flush=True)
        except Exception as e:
            print(json.dumps({"error": str(e)}), flush=True)

if __name__ == "__main__":
    main()
```

### 4.3 requirements.txt

```
# 依赖列表
```

## 5. Node.js 插件开发

### 5.1 项目结构

```
my-nodejs-plugin/
├── index.js
├── package.json
└── plugin.yaml
```

### 5.2 插件实现

```javascript
const readline = require('readline');

class MyPlugin {
    constructor() {
        this.info = {
            id: 'my-nodejs-plugin',
            name: 'my-nodejs-plugin',
            version: '1.0.0',
            type: 'nodejs'
        };
    }
    
    initialize(config) {
        return true;
    }
    
    shutdown() {
        return true;
    }
    
    getTools() {
        return [
            {
                name: 'my_tool',
                description: '我的工具',
                input_schema: {
                    type: 'object',
                    properties: {
                        input: {
                            type: 'string',
                            description: '输入参数'
                        }
                    },
                    required: ['input']
                },
                output_schema: {
                    type: 'object',
                    properties: {
                        result: {
                            type: 'string',
                            description: '输出结果'
                        }
                    }
                }
            }
        ];
    }
    
    execute(toolName, input) {
        if (toolName === 'my_tool') {
            return {
                result: `处理结果: ${input.input || ''}`
            };
        }
        throw new Error(`Unknown tool: ${toolName}`);
    }
}

const plugin = new MyPlugin();

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
});

rl.on('line', (line) => {
    try {
        const request = JSON.parse(line);
        const command = request.command;
        
        let result;
        if (command === 'initialize') {
            result = plugin.initialize(request.config || {});
        } else if (command === 'shutdown') {
            result = plugin.shutdown();
        } else if (command === 'get_tools') {
            result = plugin.getTools();
        } else if (command === 'execute') {
            result = plugin.execute(request.tool_name, request.input || {});
        } else {
            result = { error: `Unknown command: ${command}` };
        }
        
        console.log(JSON.stringify(result));
    } catch (e) {
        console.log(JSON.stringify({ error: e.message }));
    }
});
```

### 5.3 package.json

```json
{
    "name": "my-nodejs-plugin",
    "version": "1.0.0",
    "main": "index.js"
}
```

## 6. Docker 插件开发

### 6.1 项目结构

```
my-docker-plugin/
├── Dockerfile
├── main.py
└── plugin.yaml
```

### 6.2 Dockerfile

```dockerfile
FROM python:3.10-slim

WORKDIR /app

COPY main.py .
COPY plugin.yaml .

ENTRYPOINT ["python", "main.py"]
```

### 6.3 插件实现

与 Python 插件类似，但需要处理 Docker 特定的输入输出格式。

## 7. WASM 插件开发

### 7.1 项目结构

```
my-wasm-plugin/
├── src/
│   └── lib.rs
├── Cargo.toml
└── plugin.yaml
```

### 7.2 Cargo.toml

```toml
[package]
name = "my-wasm-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 7.3 插件实现

```rust
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct MyToolInput {
    input: String,
}

#[derive(Serialize)]
struct MyToolOutput {
    result: String,
}

#[wasm_bindgen]
pub fn execute(input: &str) -> String {
    let input: MyToolInput = serde_json::from_str(input).unwrap();
    let output = MyToolOutput {
        result: format!("处理结果: {}", input.input),
    };
    serde_json::to_string(&output).unwrap()
}
```

### 7.4 编译

```bash
wasm-pack build --target nodejs
```

## 8. 安全配置

### 8.1 安全级别

| 级别 | 描述 | 限制 |
|------|------|------|
| Unrestricted | 无限制 | 仅用于信任模块 |
| Basic | 基础隔离 | 限制文件系统和网络 |
| Strict | 严格隔离 | 禁止所有外部访问 |
| Maximum | 最大隔离 | 额外限制 CPU 和内存 |

### 8.2 文件权限配置

```yaml
security:
  level: basic
  file_permissions:
    read_paths:
      - /data/input
    write_paths:
      - /data/output
    allow_temp: true
    allow_cwd: false
```

### 8.3 网络权限配置

```yaml
security:
  network_permissions:
    enabled: true
    allowed_hosts:
      - api.example.com
    allowed_ports:
      - 443
    allow_dns: true
    allow_http: false
    allow_https: true
```

## 9. 测试插件

### 9.1 加载插件

```bash
workflow-toolkit plugin load ./my-plugin
```

### 9.2 查看插件状态

```bash
workflow-toolkit plugin list
```

### 9.3 测试工具

```bash
workflow-toolkit tool execute --input '{"input": "test"}' my_tool
```

### 9.4 卸载插件

```bash
workflow-toolkit plugin unload my-plugin
```

## 10. 最佳实践

### 10.1 错误处理

```rust
// 提供清晰的错误消息
fn execute(&self, input: MyToolInput) -> Result<MyToolOutput> {
    if input.input.is_empty() {
        return Err(Error::validation("输入参数不能为空"));
    }
    // ...
}
```

### 10.2 日志记录

```rust
// 使用 tracing 记录日志
use tracing::{info, warn, error};

fn execute(&self, input: MyToolInput) -> Result<MyToolOutput> {
    info!("开始处理: {}", input.input);
    // ...
    info!("处理完成");
}
```

### 10.3 性能优化

```rust
// 使用缓存
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct MyPlugin {
    cache: Arc<RwLock<HashMap<String, String>>>,
}
```

## 11. 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0.0 | 2026-03-01 | 初始版本 | Architecture Team |
