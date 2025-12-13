# Rust Toolbox 插件开发指南

本文档详细说明了如何为 Rust Toolbox 开发外部插件。

## 1. 插件系统简介

Rust Toolbox 采用基于**独立进程**的插件系统。这意味着插件可以是任何语言编写的可执行文件（二进制文件、Shell 脚本等），只要它遵循特定的输入/输出协议。

- **发现机制**：系统启动时会自动扫描项目根目录下的 `plugins/` 文件夹。
- **命名规则**：插件的可执行文件名必须以 `rt-plugin-` 开头（例如 `rt-plugin-echo.exe` 或 `rt-plugin-calculator`）。

## 2. 通信协议

插件必须支持两个子命令：`spec` 和 `run`。

### 2.1 `spec` 命令 (元数据)

系统在加载插件时会执行 `<plugin_executable> spec`。插件必须向 **STDOUT** 输出包含元数据的 JSON 字符串。

**JSON 结构 (`PluginMetadata`)**:

```json
{
  "name": "category.tool_name",
  "display_name": {
    "en": "English Name",
    "zh": "中文名称"
  },
  "description": {
    "en": "Description in English",
    "zh": "中文描述"
  },
  "user_guide": {
    "en": "Markdown guide...",
    "zh": "Markdown 指南..."
  },
  "input_schema": {
    "type": "object",
    "properties": {
      "field1": { "type": "string" }
    }
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "result": { "type": "string" }
    }
  }
}
```

### 2.2 `run` 命令 (执行)

当用户调用插件时，系统会执行 `<plugin_executable> run`。

- **输入 (STDIN)**: JSON 格式的参数对象（符合 `input_schema`）。
- **输出 (STDOUT)**: JSON 格式的执行结果（符合 `output_schema`）。
- **日志/错误 (STDERR)**: 插件的日志或错误信息应输出到 STDERR，这些信息会被主程序捕获并显示在控制台，不影响 JSON 解析。
- **退出码**: 0 表示成功，非 0 表示失败。

## 3. 开发示例 (Rust)

### 3.1 创建项目

```bash
cargo new --bin rt-plugin-demo
```

### 3.2 实现代码 (`src/main.rs`)

```rust
use serde_json::{json, Value};
use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rt-plugin-demo <spec|run>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "spec" => print_spec(),
        "run" => run_tool(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn print_spec() {
    let spec = json!({
        "name": "demo.echo",
        "display_name": {
            "en": "Echo Tool",
            "zh": "回声工具"
        },
        "description": {
            "en": "Repeats the input text",
            "zh": "重复输入的文本"
        },
        "user_guide": {
            "en": "# Echo Tool\n\nReturns the input text exactly as is.",
            "zh": "# 回声工具\n\n原样返回输入的文本。"
        },
        "input_schema": {
            "type": "object",
            "properties": {
                "text": { "type": "string", "title": "Text to echo" }
            },
            "required": ["text"]
        },
        "output_schema": {
            "type": "object",
            "properties": {
                "echo": { "type": "string" }
            }
        }
    });
    println!("{}", spec.to_string());
}

fn run_tool() {
    // 读取 STDIN
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).expect("Failed to read stdin");
    
    // 解析输入
    let input: Value = serde_json::from_str(&buffer).expect("Invalid JSON input");
    
    // 业务逻辑
    let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
    
    // 输出结果到 STDOUT
    let output = json!({
        "echo": text
    });
    println!("{}", output.to_string());
}
```

## 4. 开发示例 (Python)

如果是脚本语言，需要确保可执行文件能直接运行（在 Windows 上可能需要封装成 `.exe` 或 `.bat`）。

如果使用 Python，可以使用 PyInstaller 打包：

```python
# main.py
import sys
import json

def spec():
    print(json.dumps({
        "name": "python.adder",
        "display_name": {"en": "Adder", "zh": "加法器"},
        "description": {"en": "Adds two numbers", "zh": "计算两个数字之和"},
        "user_guide": {"en": "Input a and b", "zh": "输入 a 和 b"},
        "input_schema": {
            "type": "object",
            "properties": {
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["a", "b"]
        },
        "output_schema": {"type": "number"}
    }))

def run():
    # 从 stdin 读取
    input_str = sys.stdin.read()
    if not input_str:
        return
        
    input_data = json.loads(input_str)
    a = input_data.get("a", 0)
    b = input_data.get("b", 0)
    
    # 输出结果到 stdout
    print(json.dumps(a + b))

if __name__ == "__main__":
    if len(sys.argv) > 1:
        cmd = sys.argv[1]
        if cmd == "spec":
            spec()
        elif cmd == "run":
            run()
```

打包命令：`pyinstaller --onefile --name rt-plugin-python-demo main.py`

## 5. 部署与测试

1.  在项目根目录创建 `plugins/` 文件夹。
2.  将编译好的可执行文件（如 `rt-plugin-demo.exe`）放入该目录。
3.  运行 `rt-cli list` 查看是否成功加载插件。
4.  运行 `rt-cli run demo.echo -i '{"text": "Hello"}'` 测试运行。
