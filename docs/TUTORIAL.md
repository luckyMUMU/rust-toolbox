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
# 检查版本
workflow-toolkit --version

# 查看帮助
workflow-toolkit --help

# 验证配置
workflow-toolkit config validate
```

### 第一次运行

```bash
# 启动MCP服务器
workflow-toolkit server start

# 在另一个终端中，列出可用工具
workflow-toolkit tool list

# 执行简单工具
workflow-toolkit tool execute echo --params '{"message": "Hello, World!"}'
```

## 基础概念

### 核心组件

#### 1. 工作流 (Workflow)
工作流是一系列按特定顺序执行的任务集合，支持条件分支、循环和并行处理。

```yaml
# 基本工作流结构
name: "my-workflow"
version: "1.0.0"
description: "我的第一个工作流"

nodes:
  - id: "start"
    type: "start"
  - id: "task1"
    type: "tool"
    tool_name: "echo"
    parameters:
      message: "Hello"
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "task1"
  - from: "task1"
    to: "end"
```

#### 2. 工具 (Tools)
工具是执行具体任务的可重用组件，可以是内置工具或通过插件提供的外部工具。

#### 3. 插件 (Plugins)
插件扩展系统功能，支持多种语言和运行时环境：
- Native (Rust)
- Python
- Node.js
- Docker
- WebAssembly

#### 4. 节点类型 (Node Types)
- **Start**: 工作流起始节点
- **End**: 工作流结束节点
- **Tool**: 执行具体工具的节点
- **Condition**: 条件判断节点
- **Loop**: 循环节点
- **Parallel**: 并行执行节点

### 数据流和变量

工作流中的数据通过变量在节点间传递：

```yaml
# 变量引用示例
nodes:
  - id: "load_data"
    type: "tool"
    tool_name: "file_reader"
    parameters:
      path: "./data.json"
  
  - id: "process_data"
    type: "tool"
    tool_name: "data_processor"
    parameters:
      input: "${load_data.content}"  # 引用前一个节点的输出
      operation: "filter"
```

## 创建第一个工作流

### 步骤1: 创建工作流定义文件

创建 `hello-world.yaml`:

```yaml
name: "hello-world"
version: "1.0.0"
description: "Hello World 示例工作流"

# 全局配置
global_config:
  timeout: "5m"
  variables:
    greeting: "Hello"
    target: "World"

# 工作流节点
nodes:
  - id: "start"
    type: "start"
    
  - id: "generate_message"
    type: "tool"
    tool_name: "string_formatter"
    parameters:
      template: "${greeting}, ${target}!"
      variables:
        greeting: "${greeting}"
        target: "${target}"
    timeout: "30s"
    
  - id: "display_message"
    type: "tool"
    tool_name: "echo"
    parameters:
      message: "${generate_message.result}"
    
  - id: "save_message"
    type: "tool"
    tool_name: "file_writer"
    parameters:
      path: "./output/hello.txt"
      content: "${generate_message.result}"
      
  - id: "end"
    type: "end"

# 节点连接
edges:
  - from: "start"
    to: "generate_message"
  - from: "generate_message"
    to: "display_message"
  - from: "display_message"
    to: "save_message"
  - from: "save_message"
    to: "end"
```

### 步骤2: 验证工作流

```bash
# 验证工作流定义
workflow-toolkit workflow create --validate hello-world.yaml
```

### 步骤3: 创建工作流

```bash
# 创建工作流
workflow-toolkit workflow create hello-world.yaml
```

### 步骤4: 执行工作流

```bash
# 执行工作流
workflow-toolkit workflow execute hello-world

# 监控执行状态
workflow-toolkit workflow status <execution-id> --watch
```

### 步骤5: 查看结果

```bash
# 列出所有执行记录
workflow-toolkit workflow list

# 查看输出文件
cat ./output/hello.txt
```

## 工具开发

### 内置工具使用

查看所有可用工具：

```bash
workflow-toolkit tool list --show-schema
```

常用内置工具：
- `echo`: 输出消息
- `file_reader`: 读取文件
- `file_writer`: 写入文件
- `http_client`: HTTP请求
- `json_processor`: JSON数据处理

### 自定义工具开发

#### Rust工具示例

```rust
use workflow_toolkit::{
    tools::{BasicTool, AsyncFunctionExecutor},
    ExecutionContext, Result,
};
use serde_json::{json, Value};
use std::sync::Arc;

// 创建自定义工具
fn create_text_analyzer_tool() -> Result<BasicTool> {
    let executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            let text = params.get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| workflow_toolkit::WorkflowError::tool_execution(
                    "Missing 'text' parameter"
                ))?;
            
            // 分析文本
            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            let line_count = text.lines().count();
            
            // 简单情感分析
            let positive_words = ["good", "great", "excellent", "amazing", "wonderful"];
            let negative_words = ["bad", "terrible", "awful", "horrible", "disappointing"];
            
            let positive_score = positive_words.iter()
                .map(|word| text.to_lowercase().matches(word).count())
                .sum::<usize>();
            
            let negative_score = negative_words.iter()
                .map(|word| text.to_lowercase().matches(word).count())
                .sum::<usize>();
            
            let sentiment = if positive_score > negative_score {
                "positive"
            } else if negative_score > positive_score {
                "negative"
            } else {
                "neutral"
            };
            
            Ok(json!({
                "word_count": word_count,
                "char_count": char_count,
                "line_count": line_count,
                "sentiment": sentiment,
                "positive_score": positive_score,
                "negative_score": negative_score,
                "analysis_timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    ));
    
    BasicTool::builder()
        .name("text_analyzer")
        .version("1.0.0")
        .description("Text analysis tool with sentiment detection")
        .executor(executor)
        .build()
        .map_err(|e| workflow_toolkit::WorkflowError::tool_execution(&e.to_string()))
}

// 注册工具
async fn register_custom_tools() -> Result<()> {
    let mut registry = workflow_toolkit::tools::BasicToolRegistry::new();
    
    let text_analyzer = create_text_analyzer_tool()?;
    registry.register_tool(Arc::new(text_analyzer))?;
    
    println!("Custom tools registered successfully!");
    Ok(())
}
```

#### 工具配置Schema

```json
{
  "name": "text_analyzer",
  "version": "1.0.0",
  "description": "Text analysis tool with sentiment detection",
  "parameters_schema": {
    "type": "object",
    "properties": {
      "text": {
        "type": "string",
        "description": "Text to analyze"
      },
      "include_sentiment": {
        "type": "boolean",
        "description": "Whether to include sentiment analysis",
        "default": true
      }
    },
    "required": ["text"]
  },
  "return_schema": {
    "type": "object",
    "properties": {
      "word_count": {"type": "integer"},
      "char_count": {"type": "integer"},
      "line_count": {"type": "integer"},
      "sentiment": {"type": "string"},
      "positive_score": {"type": "integer"},
      "negative_score": {"type": "integer"},
      "analysis_timestamp": {"type": "string"}
    }
  }
}
```

## 插件开发

### Python插件开发

#### 1. 创建插件目录结构

```
my_python_plugin/
├── main.py
├── requirements.txt
├── config.yaml
└── tools/
    ├── __init__.py
    ├── data_processor.py
    └── web_scraper.py
```

#### 2. 实现插件主文件

```python
# main.py
import json
import asyncio
from typing import Dict, Any, List
from tools.data_processor import DataProcessor
from tools.web_scraper import WebScraper

class MyPythonPlugin:
    def __init__(self):
        self.name = "my_python_plugin"
        self.version = "1.0.0"
        self.data_processor = DataProcessor()
        self.web_scraper = WebScraper()
    
    def get_plugin_info(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "version": self.version,
            "description": "My custom Python plugin",
            "author": "Your Name",
            "tools": ["csv_processor", "json_transformer", "web_scraper"]
        }
    
    def get_tools(self) -> Dict[str, callable]:
        return {
            "csv_processor": self.data_processor.process_csv,
            "json_transformer": self.data_processor.transform_json,
            "web_scraper": self.web_scraper.scrape_website
        }
    
    async def initialize(self, config: Dict[str, Any]) -> bool:
        """初始化插件"""
        try:
            # 执行初始化逻辑
            await self.data_processor.initialize(config.get("data_processor", {}))
            await self.web_scraper.initialize(config.get("web_scraper", {}))
            return True
        except Exception as e:
            print(f"Plugin initialization failed: {e}")
            return False
    
    async def shutdown(self) -> bool:
        """关闭插件"""
        try:
            await self.data_processor.shutdown()
            await self.web_scraper.shutdown()
            return True
        except Exception as e:
            print(f"Plugin shutdown failed: {e}")
            return False

# 插件入口点
plugin_instance = MyPythonPlugin()

def get_plugin_info():
    return plugin_instance.get_plugin_info()

def get_tools():
    return plugin_instance.get_tools()

async def initialize(config):
    return await plugin_instance.initialize(config)

async def shutdown():
    return await plugin_instance.shutdown()
```

#### 3. 实现具体工具

```python
# tools/data_processor.py
import pandas as pd
import json
from typing import Dict, Any, List

class DataProcessor:
    def __init__(self):
        self.initialized = False
    
    async def initialize(self, config: Dict[str, Any]):
        self.config = config
        self.initialized = True
    
    async def shutdown(self):
        self.initialized = False
    
    def process_csv(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """处理CSV文件"""
        try:
            file_path = params.get("file_path")
            operations = params.get("operations", [])
            
            if not file_path:
                raise ValueError("Missing file_path parameter")
            
            # 读取CSV
            df = pd.read_csv(file_path)
            
            # 执行操作
            for operation in operations:
                op_type = operation.get("type")
                
                if op_type == "filter":
                    condition = operation.get("condition")
                    df = df.query(condition)
                
                elif op_type == "sort":
                    column = operation.get("column")
                    ascending = operation.get("ascending", True)
                    df = df.sort_values(column, ascending=ascending)
                
                elif op_type == "group":
                    group_by = operation.get("group_by")
                    agg_func = operation.get("aggregation", "sum")
                    df = df.groupby(group_by).agg(agg_func).reset_index()
                
                elif op_type == "transform":
                    column = operation.get("column")
                    transformation = operation.get("transformation")
                    
                    if transformation == "uppercase":
                        df[column] = df[column].str.upper()
                    elif transformation == "lowercase":
                        df[column] = df[column].str.lower()
            
            # 保存结果
            output_path = params.get("output_path")
            if output_path:
                df.to_csv(output_path, index=False)
            
            return {
                "success": True,
                "record_count": len(df),
                "columns": list(df.columns),
                "output_path": output_path,
                "operations_applied": len(operations)
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    def transform_json(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """转换JSON数据"""
        try:
            data = params.get("data")
            transformations = params.get("transformations", [])
            
            if isinstance(data, str):
                data = json.loads(data)
            
            result = data.copy() if isinstance(data, dict) else data[:]
            
            for transform in transformations:
                transform_type = transform.get("type")
                
                if transform_type == "rename_field":
                    old_name = transform.get("old_name")
                    new_name = transform.get("new_name")
                    if isinstance(result, dict) and old_name in result:
                        result[new_name] = result.pop(old_name)
                
                elif transform_type == "add_field":
                    field_name = transform.get("field_name")
                    field_value = transform.get("field_value")
                    if isinstance(result, dict):
                        result[field_name] = field_value
                
                elif transform_type == "remove_field":
                    field_name = transform.get("field_name")
                    if isinstance(result, dict) and field_name in result:
                        del result[field_name]
            
            return {
                "success": True,
                "result": result,
                "transformations_applied": len(transformations)
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
```

#### 4. 插件配置文件

```yaml
# config.yaml
plugin:
  name: "my_python_plugin"
  version: "1.0.0"
  description: "My custom Python plugin for data processing"
  
environment:
  python_version: "3.8+"
  virtual_env: "./venv"
  
dependencies:
  - pandas>=1.3.0
  - requests>=2.25.0
  - beautifulsoup4>=4.9.0
  
tools:
  csv_processor:
    description: "Process CSV files with various operations"
    timeout: 300
    memory_limit: "512MB"
    
  json_transformer:
    description: "Transform JSON data structures"
    timeout: 60
    memory_limit: "256MB"
    
  web_scraper:
    description: "Scrape web pages and extract data"
    timeout: 120
    memory_limit: "256MB"
    
security:
  sandbox: true
  network_access: true
  file_system_access: "restricted"
  allowed_domains:
    - "api.example.com"
    - "data.example.org"
```

### Node.js插件开发

#### 1. 创建package.json

```json
{
  "name": "my-nodejs-plugin",
  "version": "1.0.0",
  "description": "My custom Node.js plugin",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "test": "jest"
  },
  "dependencies": {
    "axios": "^0.27.0",
    "cheerio": "^1.0.0-rc.12",
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "jest": "^28.0.0"
  },
  "keywords": ["workflow", "plugin", "automation"],
  "author": "Your Name",
  "license": "MIT"
}
```

#### 2. 实现插件主文件

```javascript
// index.js
const axios = require('axios');
const cheerio = require('cheerio');
const _ = require('lodash');

class MyNodeJSPlugin {
    constructor() {
        this.name = 'my_nodejs_plugin';
        this.version = '1.0.0';
        this.initialized = false;
    }
    
    async initialize(config = {}) {
        try {
            this.config = config;
            this.httpClient = axios.create({
                timeout: config.timeout || 30000,
                headers: config.headers || {}
            });
            this.initialized = true;
            return { success: true };
        } catch (error) {
            return { success: false, error: error.message };
        }
    }
    
    async shutdown() {
        this.initialized = false;
        return { success: true };
    }
    
    getPluginInfo() {
        return {
            name: this.name,
            version: this.version,
            description: 'My custom Node.js plugin for web operations',
            author: 'Your Name',
            tools: ['web_scraper', 'api_client', 'data_transformer']
        };
    }
    
    async webScraper(params) {
        try {
            const { url, selectors, options = {} } = params;
            
            const response = await this.httpClient.get(url, {
                headers: options.headers || {}
            });
            
            const $ = cheerio.load(response.data);
            const results = {};
            
            for (const [key, selector] of Object.entries(selectors)) {
                if (selector.multiple) {
                    results[key] = [];
                    $(selector.css).each((i, elem) => {
                        const text = $(elem).text().trim();
                        const href = $(elem).attr('href');
                        results[key].push({
                            text,
                            href: href ? new URL(href, url).href : null
                        });
                    });
                } else {
                    const elem = $(selector.css).first();
                    results[key] = {
                        text: elem.text().trim(),
                        href: elem.attr('href') ? new URL(elem.attr('href'), url).href : null
                    };
                }
            }
            
            return {
                success: true,
                url,
                data: results,
                scraped_at: new Date().toISOString()
            };
            
        } catch (error) {
            return {
                success: false,
                error: error.message,
                url: params.url
            };
        }
    }
    
    async apiClient(params) {
        try {
            const { method, url, data, headers = {}, timeout } = params;
            
            const config = {
                method: method.toLowerCase(),
                url,
                headers: { ...this.config.headers, ...headers },
                timeout: timeout || this.config.timeout || 30000
            };
            
            if (data && ['post', 'put', 'patch'].includes(config.method)) {
                config.data = data;
            }
            
            const response = await axios(config);
            
            return {
                success: true,
                status: response.status,
                statusText: response.statusText,
                headers: response.headers,
                data: response.data
            };
            
        } catch (error) {
            return {
                success: false,
                error: error.message,
                status: error.response?.status,
                statusText: error.response?.statusText,
                data: error.response?.data
            };
        }
    }
    
    async dataTransformer(params) {
        try {
            const { data, transformations } = params;
            let result = _.cloneDeep(data);
            
            for (const transform of transformations) {
                switch (transform.type) {
                    case 'map':
                        if (Array.isArray(result)) {
                            result = result.map(item => {
                                const mapped = {};
                                for (const [newKey, oldKey] of Object.entries(transform.mapping)) {
                                    mapped[newKey] = _.get(item, oldKey);
                                }
                                return mapped;
                            });
                        }
                        break;
                        
                    case 'filter':
                        if (Array.isArray(result)) {
                            result = result.filter(item => {
                                return eval(transform.condition.replace(/\$\{(\w+)\}/g, 'item.$1'));
                            });
                        }
                        break;
                        
                    case 'sort':
                        if (Array.isArray(result)) {
                            result = _.orderBy(result, transform.field, transform.order || 'asc');
                        }
                        break;
                        
                    case 'group':
                        if (Array.isArray(result)) {
                            result = _.groupBy(result, transform.field);
                        }
                        break;
                }
            }
            
            return {
                success: true,
                result,
                transformations_applied: transformations.length
            };
            
        } catch (error) {
            return {
                success: false,
                error: error.message
            };
        }
    }
}

// 创建插件实例
const plugin = new MyNodeJSPlugin();

// 导出插件接口
module.exports = {
    getPluginInfo: () => plugin.getPluginInfo(),
    initialize: (config) => plugin.initialize(config),
    shutdown: () => plugin.shutdown(),
    
    // 工具函数
    webScraper: (params) => plugin.webScraper(params),
    apiClient: (params) => plugin.apiClient(params),
    dataTransformer: (params) => plugin.dataTransformer(params)
};
```

### Docker插件开发

#### 1. 创建Dockerfile

```dockerfile
FROM node:16-alpine

WORKDIR /app

# 安装依赖
COPY package*.json ./
RUN npm ci --only=production

# 复制应用代码
COPY . .

# 创建非root用户
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001

# 设置权限
RUN chown -R nodejs:nodejs /app
USER nodejs

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# 启动应用
CMD ["npm", "start"]
```

#### 2. 创建插件服务器

```javascript
// server.js
const express = require('express');
const bodyParser = require('body-parser');
const { execSync } = require('child_process');

const app = express();
app.use(bodyParser.json());

// 插件信息
const pluginInfo = {
    name: 'docker_tools_plugin',
    version: '1.0.0',
    description: 'Docker-based tools plugin',
    tools: ['system_info', 'file_processor', 'image_converter']
};

// 健康检查端点
app.get('/health', (req, res) => {
    res.json({ status: 'healthy', timestamp: new Date().toISOString() });
});

// 插件信息端点
app.get('/plugin/info', (req, res) => {
    res.json(pluginInfo);
});

// 工具执行端点
app.post('/tools/:toolName/execute', async (req, res) => {
    const { toolName } = req.params;
    const { parameters } = req.body;
    
    try {
        let result;
        
        switch (toolName) {
            case 'system_info':
                result = await getSystemInfo(parameters);
                break;
            case 'file_processor':
                result = await processFile(parameters);
                break;
            case 'image_converter':
                result = await convertImage(parameters);
                break;
            default:
                return res.status(404).json({
                    success: false,
                    error: `Tool '${toolName}' not found`
                });
        }
        
        res.json(result);
    } catch (error) {
        res.status(500).json({
            success: false,
            error: error.message
        });
    }
});

// 工具实现
async function getSystemInfo(params) {
    const info = {
        hostname: execSync('hostname').toString().trim(),
        uptime: execSync('uptime').toString().trim(),
        memory: execSync('free -h').toString().trim(),
        disk: execSync('df -h').toString().trim(),
        processes: execSync('ps aux --sort=-%cpu | head -10').toString().trim()
    };
    
    return {
        success: true,
        system_info: info,
        collected_at: new Date().toISOString()
    };
}

async function processFile(params) {
    const { input_path, output_path, operation } = params;
    
    try {
        switch (operation) {
            case 'compress':
                execSync(`gzip -c ${input_path} > ${output_path}`);
                break;
            case 'decompress':
                execSync(`gunzip -c ${input_path} > ${output_path}`);
                break;
            case 'convert_encoding':
                const { from_encoding, to_encoding } = params;
                execSync(`iconv -f ${from_encoding} -t ${to_encoding} ${input_path} > ${output_path}`);
                break;
            default:
                throw new Error(`Unknown operation: ${operation}`);
        }
        
        return {
            success: true,
            input_path,
            output_path,
            operation,
            processed_at: new Date().toISOString()
        };
    } catch (error) {
        return {
            success: false,
            error: error.message
        };
    }
}

async function convertImage(params) {
    const { input_path, output_path, format, quality } = params;
    
    try {
        let command = `convert ${input_path}`;
        
        if (quality) {
            command += ` -quality ${quality}`;
        }
        
        command += ` ${output_path}`;
        
        execSync(command);
        
        return {
            success: true,
            input_path,
            output_path,
            format,
            quality,
            converted_at: new Date().toISOString()
        };
    } catch (error) {
        return {
            success: false,
            error: error.message
        };
    }
}

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
    console.log(`Docker plugin server running on port ${PORT}`);
});
```

## 高级特性

### 条件执行

```yaml
# 条件工作流示例
nodes:
  - id: "check_file_size"
    type: "tool"
    tool_name: "file_info"
    parameters:
      path: "${input_file}"
  
  - id: "size_condition"
    type: "condition"
    condition: "${check_file_size.size} > 1048576"  # 1MB
  
  - id: "compress_large_file"
    type: "tool"
    tool_name: "file_compressor"
    parameters:
      input: "${input_file}"
      output: "${input_file}.gz"
  
  - id: "process_small_file"
    type: "tool"
    tool_name: "direct_processor"
    parameters:
      input: "${input_file}"

edges:
  - from: "check_file_size"
    to: "size_condition"
  - from: "size_condition"
    to: "compress_large_file"
    condition: "true"
  - from: "size_condition"
    to: "process_small_file"
    condition: "false"
```

### 循环处理

```yaml
# 循环工作流示例
nodes:
  - id: "load_batch"
    type: "tool"
    tool_name: "batch_loader"
    parameters:
      source: "${data_source}"
      batch_size: 100
  
  - id: "process_loop"
    type: "loop"
    parameters:
      items: "${load_batch.batches}"
      max_iterations: 50
      parallel: true
      max_concurrency: 4
  
  - id: "process_batch"
    type: "tool"
    tool_name: "batch_processor"
    parameters:
      batch: "${current_item}"
      
edges:
  - from: "load_batch"
    to: "process_loop"
  - from: "process_loop"
    to: "process_batch"
```

### 错误处理和重试

```yaml
# 错误处理示例
nodes:
  - id: "unreliable_task"
    type: "tool"
    tool_name: "external_api_call"
    parameters:
      url: "https://api.example.com/data"
    retry_policy:
      max_attempts: 5
      delay: "2s"
      backoff: "exponential"
      max_delay: "30s"
    timeout: "60s"
    
  - id: "error_handler"
    type: "tool"
    tool_name: "error_logger"
    parameters:
      error: "${unreliable_task.error}"
      context: "API call failed"
      
edges:
  - from: "unreliable_task"
    to: "next_task"
    condition: "${unreliable_task.success} == true"
  - from: "unreliable_task"
    to: "error_handler"
    condition: "${unreliable_task.success} == false"
```

### 并行处理优化

```yaml
# 高性能并行处理
global_config:
  parallel_config:
    max_global_concurrency: 16
    resource_limits:
      memory: "8GB"
      cpu: "4.0"

nodes:
  - id: "parallel_group"
    type: "parallel"
    parameters:
      max_concurrency: 8
      load_balancing: "round_robin"
      timeout: "30m"
      
  - id: "cpu_intensive_task_1"
    type: "tool"
    tool_name: "data_analyzer"
    parameters:
      dataset: "${datasets[0]}"
    resource_requirements:
      cpu: "1.0"
      memory: "2GB"
      
  - id: "cpu_intensive_task_2"
    type: "tool"
    tool_name: "data_analyzer"
    parameters:
      dataset: "${datasets[1]}"
    resource_requirements:
      cpu: "1.0"
      memory: "2GB"
```

## 最佳实践

### 1. 工作流设计原则

#### 模块化设计
```yaml
# 好的做法：模块化工作流
name: "data-pipeline"
nodes:
  - id: "extract"
    type: "tool"
    tool_name: "data_extractor"
    
  - id: "transform"
    type: "tool"
    tool_name: "data_transformer"
    
  - id: "load"
    type: "tool"
    tool_name: "data_loader"
```

#### 错误处理
```yaml
# 为关键节点添加错误处理
nodes:
  - id: "critical_task"
    type: "tool"
    tool_name: "important_processor"
    retry_policy:
      max_attempts: 3
      delay: "5s"
      backoff: "exponential"
    timeout: "10m"
    
  - id: "fallback_task"
    type: "tool"
    tool_name: "fallback_processor"
    
edges:
  - from: "critical_task"
    to: "next_task"
    condition: "${critical_task.success} == true"
  - from: "critical_task"
    to: "fallback_task"
    condition: "${critical_task.success} == false"
```

### 2. 性能优化

#### 缓存策略
```rust
// 实现工具级缓存
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct CachedTool {
    cache: Arc<RwLock<HashMap<String, (Value, std::time::Instant)>>>,
    ttl: std::time::Duration,
}

impl CachedTool {
    async fn execute_with_cache(&self, params: Value) -> Result<Value> {
        let cache_key = self.generate_cache_key(&params);
        
        // 检查缓存
        if let Some(cached_result) = self.get_from_cache(&cache_key).await {
            return Ok(cached_result);
        }
        
        // 执行实际计算
        let result = self.execute_actual(params).await?;
        
        // 存储到缓存
        self.store_in_cache(cache_key, result.clone()).await;
        
        Ok(result)
    }
}
```

#### 资源管理
```yaml
# 配置资源限制
global_config:
  resource_limits:
    max_memory_per_node: "1GB"
    max_cpu_per_node: "2.0"
    max_execution_time: "1h"
    
nodes:
  - id: "memory_intensive_task"
    type: "tool"
    tool_name: "large_data_processor"
    resource_requirements:
      memory: "4GB"
      cpu: "1.0"
      temporary_disk: "10GB"
```

### 3. 监控和日志

#### 结构化日志
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(params))]
async fn execute_tool(name: &str, params: Value) -> Result<Value> {
    info!(tool_name = name, "开始执行工具");
    
    let start_time = std::time::Instant::now();
    
    match tool_registry.execute_tool(name, params).await {
        Ok(result) => {
            info!(
                tool_name = name,
                execution_time_ms = start_time.elapsed().as_millis(),
                "工具执行成功"
            );
            Ok(result)
        }
        Err(e) => {
            error!(
                tool_name = name,
                error = %e,
                execution_time_ms = start_time.elapsed().as_millis(),
                "工具执行失败"
            );
            Err(e)
        }
    }
}
```

#### 指标收集
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref WORKFLOW_EXECUTIONS: Counter = register_counter!(
        "workflow_executions_total",
        "Total number of workflow executions"
    ).unwrap();
    
    static ref WORKFLOW_DURATION: Histogram = register_histogram!(
        "workflow_duration_seconds",
        "Workflow execution duration in seconds"
    ).unwrap();
}

async fn execute_workflow_with_metrics(workflow: WorkflowDefinition) -> Result<()> {
    WORKFLOW_EXECUTIONS.inc();
    let timer = WORKFLOW_DURATION.start_timer();
    
    let result = execute_workflow(workflow).await;
    timer.observe_duration();
    
    result
}
```

### 4. 安全最佳实践

#### 输入验证
```rust
use serde_json::Value;
use jsonschema::{JSONSchema, ValidationError};

fn validate_tool_parameters(params: &Value, schema: &Value) -> Result<()> {
    let compiled_schema = JSONSchema::compile(schema)
        .map_err(|e| WorkflowError::validation(&format!("Invalid schema: {}", e)))?;
    
    if let Err(errors) = compiled_schema.validate(params) {
        let error_messages: Vec<String> = errors
            .map(|e| format!("Validation error: {}", e))
            .collect();
        return Err(WorkflowError::validation(&error_messages.join(", ")));
    }
    
    Ok(())
}
```

#### 权限控制
```yaml
# 插件安全配置
plugins:
  - name: "untrusted_plugin"
    security:
      sandbox: true
      network_access: false
      file_system_access: "read_only"
      allowed_paths:
        - "/tmp"
        - "/data/input"
      resource_limits:
        memory: "256MB"
        cpu: "0.5"
        execution_time: "5m"
```

## 故障排除

### 常见问题

#### 1. 工作流执行卡住

**症状**: 工作流状态长时间保持在"Running"

**排查步骤**:
```bash
# 检查工作流状态
workflow-toolkit workflow status <execution-id> --format json

# 查看详细日志
workflow-toolkit workflow logs <execution-id>

# 检查系统资源
workflow-toolkit system status
```

**可能原因和解决方案**:
- 节点超时设置过长 → 调整timeout参数
- 工具执行死锁 → 检查工具实现
- 资源不足 → 增加系统资源或优化工具

#### 2. 插件加载失败

**症状**: 插件状态显示为"Error"

**排查步骤**:
```bash
# 检查插件状态
workflow-toolkit plugin list --status error

# 查看插件日志
workflow-toolkit plugin logs <plugin-name>

# 验证插件配置
workflow-toolkit plugin validate <plugin-path>
```

**解决方案**:
- 检查依赖安装
- 验证配置文件格式
- 确认权限设置

#### 3. 性能问题

**症状**: 工作流执行缓慢

**性能分析**:
```bash
# 启用性能分析
workflow-toolkit workflow execute <workflow> --profile

# 查看性能报告
workflow-toolkit workflow profile <execution-id>

# 监控资源使用
workflow-toolkit system monitor --duration 5m
```

**优化建议**:
- 启用结果缓存
- 调整并发设置
- 优化工具实现
- 使用更快的存储后端

### 调试技巧

#### 1. 启用详细日志
```bash
# 设置日志级别
export RUST_LOG=debug
workflow-toolkit workflow execute <workflow>

# 或在配置文件中设置
[logging]
level = "debug"
```

#### 2. 使用干运行模式
```bash
# 验证工作流而不实际执行
workflow-toolkit workflow execute <workflow> --dry-run
```

#### 3. 分步调试
```bash
# 单独测试工具
workflow-toolkit tool execute <tool-name> --params <params>

# 验证工作流定义
workflow-toolkit workflow validate <workflow-file>
```

### 获取帮助

- 查看文档: [API参考](API_REFERENCE.md)
- 提交Issue: GitHub Issues
- 社区讨论: GitHub Discussions
- 邮件支持: support@example.com

---

本教程涵盖了工作流工具包的主要功能和使用方法。随着您对系统的深入了解，可以探索更多高级特性和自定义选项。