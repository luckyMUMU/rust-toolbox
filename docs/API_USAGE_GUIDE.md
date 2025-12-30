# API使用指南

本指南提供工作流工具包API的详细使用示例和最佳实践。

## 快速开始

### 1. 基本工作流创建和执行

```rust
use workflow_toolkit::{
    WorkflowDefinition, WorkflowEngine, DefaultWorkflowEngine,
    ToolRegistry, BasicToolRegistry, StateManager,
    storage::{FileStorage, SimpleMemoryCache},
    config::ConfigManager, Config,
};
use std::sync::Arc;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化组件
    let config = Arc::new(ConfigManager::new(Config::default()));
    let storage = Arc::new(FileStorage::new("./data")?);
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));
    let tool_registry = Arc::new(BasicToolRegistry::new());
    
    // 2. 创建工作流引擎
    let engine = Arc::new(DefaultWorkflowEngine::new(
        state_manager,
        tool_registry,
        4, // 最大并发数
    ));
    
    // 3. 定义工作流
    let workflow = WorkflowDefinition {
        name: "hello-world".to_string(),
        version: "1.0.0".to_string(),
        description: Some("简单的Hello World工作流".to_string()),
        metadata: std::collections::HashMap::new(),
        nodes: vec![
            workflow_toolkit::workflow::WorkflowNode {
                id: "start".to_string(),
                node_type: workflow_toolkit::workflow::NodeType::Start,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: Vec::new(),
                metadata: std::collections::HashMap::new(),
            },
            workflow_toolkit::workflow::WorkflowNode {
                id: "hello".to_string(),
                node_type: workflow_toolkit::workflow::NodeType::Tool,
                tool_name: Some("echo".to_string()),
                parameters: json!({
                    "message": "Hello, World!"
                }),
                retry_policy: None,
                timeout: Some(std::time::Duration::from_secs(30)),
                depends_on: vec!["start".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            workflow_toolkit::workflow::WorkflowNode {
                id: "end".to_string(),
                node_type: workflow_toolkit::workflow::NodeType::End,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["hello".to_string()],
                metadata: std::collections::HashMap::new(),
            },
        ],
        edges: vec![
            workflow_toolkit::workflow::WorkflowEdge {
                from: "start".to_string(),
                to: "hello".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            workflow_toolkit::workflow::WorkflowEdge {
                from: "hello".to_string(),
                to: "end".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
        ],
        global_config: workflow_toolkit::WorkflowConfig::default(),
    };
    
    // 4. 执行工作流
    let execution = engine.execute_workflow(workflow).await?;
    println!("工作流执行ID: {}", execution.id);
    
    // 5. 监控执行状态
    loop {
        let status = engine.get_workflow_status(execution.id).await?;
        println!("当前状态: {:?}", status);
        
        if status.is_terminal() {
            break;
        }
        
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    
    Ok(())
}
```

### 2. 工具注册和使用

```rust
use workflow_toolkit::{
    tools::{BasicTool, AsyncFunctionExecutor, ToolRegistry, BasicToolRegistry},
    ExecutionContext,
};
use serde_json::{json, Value};
use std::sync::Arc;

async fn register_custom_tools() -> Result<Arc<BasicToolRegistry>, Box<dyn std::error::Error>> {
    let mut registry = BasicToolRegistry::new();
    
    // 注册计算器工具
    let calculator_executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            let a = params["a"].as_f64().unwrap_or(0.0);
            let b = params["b"].as_f64().unwrap_or(0.0);
            let operation = params["operation"].as_str().unwrap_or("add");
            
            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b != 0.0 {
                        a / b
                    } else {
                        return Err(workflow_toolkit::WorkflowError::tool_execution(
                            "Division by zero"
                        ));
                    }
                }
                _ => return Err(workflow_toolkit::WorkflowError::tool_execution(
                    &format!("Unknown operation: {}", operation)
                )),
            };
            
            Ok(json!({
                "result": result,
                "operation": operation,
                "operands": [a, b]
            }))
        }
    ));
    
    let calculator_tool = BasicTool::builder()
        .name("calculator")
        .version("1.0.0")
        .description("基本计算器工具")
        .executor(calculator_executor)
        .build()?;
    
    registry.register_tool(Arc::new(calculator_tool))?;
    
    // 注册文件处理工具
    let file_processor_executor = Arc::new(AsyncFunctionExecutor::new(
        |params: Value, _context: ExecutionContext| async move {
            let file_path = params["path"].as_str()
                .ok_or_else(|| workflow_toolkit::WorkflowError::tool_execution("Missing 'path' parameter"))?;
            let operation = params["operation"].as_str().unwrap_or("read");
            
            match operation {
                "read" => {
                    let content = tokio::fs::read_to_string(file_path).await
                        .map_err(|e| workflow_toolkit::WorkflowError::tool_execution(&e.to_string()))?;
                    Ok(json!({
                        "content": content,
                        "size": content.len(),
                        "path": file_path
                    }))
                }
                "write" => {
                    let content = params["content"].as_str()
                        .ok_or_else(|| workflow_toolkit::WorkflowError::tool_execution("Missing 'content' parameter"))?;
                    tokio::fs::write(file_path, content).await
                        .map_err(|e| workflow_toolkit::WorkflowError::tool_execution(&e.to_string()))?;
                    Ok(json!({
                        "success": true,
                        "path": file_path,
                        "bytes_written": content.len()
                    }))
                }
                _ => Err(workflow_toolkit::WorkflowError::tool_execution(
                    &format!("Unknown operation: {}", operation)
                ))
            }
        }
    ));
    
    let file_tool = BasicTool::builder()
        .name("file_processor")
        .version("1.0.0")
        .description("文件处理工具")
        .executor(file_processor_executor)
        .build()?;
    
    registry.register_tool(Arc::new(file_tool))?;
    
    Ok(Arc::new(registry))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = register_custom_tools().await?;
    
    // 测试计算器工具
    let calc_result = registry.execute_tool(
        "calculator",
        json!({
            "a": 10,
            "b": 5,
            "operation": "multiply"
        }),
        ExecutionContext::new(),
    ).await?;
    
    println!("计算结果: {}", calc_result);
    
    // 测试文件工具
    let file_result = registry.execute_tool(
        "file_processor",
        json!({
            "path": "./test.txt",
            "operation": "write",
            "content": "Hello, World!"
        }),
        ExecutionContext::new(),
    ).await?;
    
    println!("文件操作结果: {}", file_result);
    
    Ok(())
}
```

## 高级用法

### 1. 条件和循环工作流

```yaml
# conditional-workflow.yaml
name: "conditional-processing"
version: "1.0.0"
description: "带条件判断的数据处理工作流"

global_config:
  variables:
    threshold: 100
    max_retries: 3

nodes:
  - id: "start"
    type: "start"
    
  - id: "load_data"
    type: "tool"
    tool_name: "data_loader"
    parameters:
      source: "${env.DATA_SOURCE}"
      format: "json"
    
  - id: "validate_data"
    type: "condition"
    condition: "${load_data.record_count} > ${threshold}"
    
  - id: "process_large_dataset"
    type: "tool"
    tool_name: "batch_processor"
    parameters:
      data: "${load_data.output}"
      batch_size: 1000
      parallel: true
    
  - id: "process_small_dataset"
    type: "tool"
    tool_name: "simple_processor"
    parameters:
      data: "${load_data.output}"
      
  - id: "quality_check"
    type: "tool"
    tool_name: "quality_checker"
    parameters:
      data: "${process_large_dataset.output || process_small_dataset.output}"
      
  - id: "retry_check"
    type: "condition"
    condition: "${quality_check.score} < 0.8 && ${context.retry_count} < ${max_retries}"
    
  - id: "save_results"
    type: "tool"
    tool_name: "data_saver"
    parameters:
      data: "${quality_check.output}"
      destination: "${env.OUTPUT_PATH}"
      
  - id: "send_alert"
    type: "tool"
    tool_name: "alert_sender"
    parameters:
      message: "数据质量检查失败，已达到最大重试次数"
      severity: "error"
      
  - id: "end"
    type: "end"

edges:
  - from: "start"
    to: "load_data"
    
  - from: "load_data"
    to: "validate_data"
    
  - from: "validate_data"
    to: "process_large_dataset"
    condition: "true"
    
  - from: "validate_data"
    to: "process_small_dataset"
    condition: "false"
    
  - from: "process_large_dataset"
    to: "quality_check"
    
  - from: "process_small_dataset"
    to: "quality_check"
    
  - from: "quality_check"
    to: "retry_check"
    
  - from: "retry_check"
    to: "load_data"
    condition: "true"
    
  - from: "retry_check"
    to: "send_alert"
    condition: "false"
    
  - from: "quality_check"
    to: "save_results"
    condition: "${quality_check.score} >= 0.8"
    
  - from: "save_results"
    to: "end"
    
  - from: "send_alert"
    to: "end"
```

### 2. 并行处理工作流

```rust
use workflow_toolkit::{WorkflowDefinition, workflow::*};
use serde_json::json;

fn create_parallel_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        name: "parallel-data-processing".to_string(),
        version: "1.0.0".to_string(),
        description: Some("并行数据处理工作流".to_string()),
        metadata: std::collections::HashMap::new(),
        nodes: vec![
            WorkflowNode {
                id: "start".to_string(),
                node_type: NodeType::Start,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: Vec::new(),
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "split_data".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("data_splitter".to_string()),
                parameters: json!({
                    "input_file": "${env.INPUT_FILE}",
                    "chunk_size": 1000,
                    "output_dir": "./chunks"
                }),
                retry_policy: None,
                timeout: Some(std::time::Duration::from_secs(300)),
                depends_on: vec!["start".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "parallel_processing".to_string(),
                node_type: NodeType::Parallel,
                tool_name: None,
                parameters: json!({
                    "max_concurrency": 4,
                    "timeout": "30m"
                }),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["split_data".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "process_chunk_1".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("chunk_processor".to_string()),
                parameters: json!({
                    "chunk_file": "${split_data.chunks[0]}",
                    "algorithm": "fast"
                }),
                retry_policy: Some(RetryPolicy {
                    max_attempts: 3,
                    delay: std::time::Duration::from_secs(5),
                    backoff: BackoffStrategy::Exponential,
                }),
                timeout: Some(std::time::Duration::from_secs(600)),
                depends_on: vec!["parallel_processing".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "process_chunk_2".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("chunk_processor".to_string()),
                parameters: json!({
                    "chunk_file": "${split_data.chunks[1]}",
                    "algorithm": "fast"
                }),
                retry_policy: Some(RetryPolicy {
                    max_attempts: 3,
                    delay: std::time::Duration::from_secs(5),
                    backoff: BackoffStrategy::Exponential,
                }),
                timeout: Some(std::time::Duration::from_secs(600)),
                depends_on: vec!["parallel_processing".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "merge_results".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("result_merger".to_string()),
                parameters: json!({
                    "inputs": [
                        "${process_chunk_1.output}",
                        "${process_chunk_2.output}"
                    ],
                    "output_file": "${env.OUTPUT_FILE}"
                }),
                retry_policy: None,
                timeout: Some(std::time::Duration::from_secs(300)),
                depends_on: vec!["process_chunk_1".to_string(), "process_chunk_2".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            WorkflowNode {
                id: "end".to_string(),
                node_type: NodeType::End,
                tool_name: None,
                parameters: json!({}),
                retry_policy: None,
                timeout: None,
                depends_on: vec!["merge_results".to_string()],
                metadata: std::collections::HashMap::new(),
            },
        ],
        edges: vec![
            WorkflowEdge {
                from: "start".to_string(),
                to: "split_data".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "split_data".to_string(),
                to: "parallel_processing".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "parallel_processing".to_string(),
                to: "process_chunk_1".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "parallel_processing".to_string(),
                to: "process_chunk_2".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "process_chunk_1".to_string(),
                to: "merge_results".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "process_chunk_2".to_string(),
                to: "merge_results".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
            WorkflowEdge {
                from: "merge_results".to_string(),
                to: "end".to_string(),
                condition: None,
                metadata: std::collections::HashMap::new(),
            },
        ],
        global_config: WorkflowConfig::default(),
    }
}
```

### 3. 插件开发示例

#### Python插件开发

```python
# plugins/data_tools/main.py
import json
import pandas as pd
from typing import Dict, Any

class DataProcessor:
    """数据处理工具集合"""
    
    def __init__(self):
        self.name = "data_processor"
        self.version = "1.0.0"
    
    def process_csv(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """处理CSV文件"""
        try:
            file_path = params.get('file_path')
            operations = params.get('operations', [])
            
            # 读取CSV文件
            df = pd.read_csv(file_path)
            
            # 执行操作
            for op in operations:
                if op['type'] == 'filter':
                    df = df.query(op['condition'])
                elif op['type'] == 'sort':
                    df = df.sort_values(op['column'], ascending=op.get('ascending', True))
                elif op['type'] == 'group':
                    df = df.groupby(op['column']).agg(op['aggregation'])
            
            # 保存结果
            output_path = params.get('output_path', 'output.csv')
            df.to_csv(output_path, index=False)
            
            return {
                'success': True,
                'output_path': output_path,
                'record_count': len(df),
                'columns': list(df.columns)
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def validate_data(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """数据验证"""
        try:
            data = params.get('data')
            schema = params.get('schema')
            
            # 简单的数据验证逻辑
            errors = []
            
            if isinstance(data, list):
                for i, record in enumerate(data):
                    for field, rules in schema.items():
                        if field not in record:
                            if rules.get('required', False):
                                errors.append(f"Record {i}: Missing required field '{field}'")
                        else:
                            value = record[field]
                            if 'type' in rules and not isinstance(value, rules['type']):
                                errors.append(f"Record {i}: Field '{field}' has wrong type")
            
            return {
                'valid': len(errors) == 0,
                'errors': errors,
                'record_count': len(data) if isinstance(data, list) else 1
            }
            
        except Exception as e:
            return {
                'valid': False,
                'errors': [str(e)]
            }

# 插件入口点
def get_tools():
    """返回插件提供的工具列表"""
    processor = DataProcessor()
    return {
        'csv_processor': processor.process_csv,
        'data_validator': processor.validate_data
    }

def get_plugin_info():
    """返回插件信息"""
    return {
        'name': 'data_tools',
        'version': '1.0.0',
        'description': '数据处理工具集合',
        'author': '开发团队',
        'tools': ['csv_processor', 'data_validator']
    }
```

#### Node.js插件开发

```javascript
// plugins/web_tools/index.js
const axios = require('axios');
const cheerio = require('cheerio');

class WebTools {
    constructor() {
        this.name = 'web_tools';
        this.version = '1.0.0';
    }
    
    async scrapeWebsite(params) {
        try {
            const { url, selectors } = params;
            
            // 获取网页内容
            const response = await axios.get(url, {
                timeout: params.timeout || 30000,
                headers: params.headers || {}
            });
            
            const $ = cheerio.load(response.data);
            const results = {};
            
            // 提取数据
            for (const [key, selector] of Object.entries(selectors)) {
                if (selector.multiple) {
                    results[key] = [];
                    $(selector.css).each((i, elem) => {
                        results[key].push($(elem).text().trim());
                    });
                } else {
                    results[key] = $(selector.css).first().text().trim();
                }
            }
            
            return {
                success: true,
                url: url,
                data: results,
                timestamp: new Date().toISOString()
            };
            
        } catch (error) {
            return {
                success: false,
                error: error.message,
                url: params.url
            };
        }
    }
    
    async httpRequest(params) {
        try {
            const { method, url, headers, data, timeout } = params;
            
            const config = {
                method: method || 'GET',
                url: url,
                headers: headers || {},
                timeout: timeout || 30000
            };
            
            if (data && ['POST', 'PUT', 'PATCH'].includes(method?.toUpperCase())) {
                config.data = data;
            }
            
            const response = await axios(config);
            
            return {
                success: true,
                status: response.status,
                headers: response.headers,
                data: response.data
            };
            
        } catch (error) {
            return {
                success: false,
                error: error.message,
                status: error.response?.status,
                data: error.response?.data
            };
        }
    }
}

// 导出工具函数
const webTools = new WebTools();

module.exports = {
    scrapeWebsite: webTools.scrapeWebsite.bind(webTools),
    httpRequest: webTools.httpRequest.bind(webTools),
    
    getPluginInfo: () => ({
        name: 'web_tools',
        version: '1.0.0',
        description: 'Web scraping and HTTP request tools',
        author: 'Development Team',
        tools: ['scrapeWebsite', 'httpRequest']
    })
};
```

## 最佳实践

### 1. 错误处理

```rust
use workflow_toolkit::{WorkflowError, Result};

// 自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum CustomToolError {
    #[error("参数验证失败: {message}")]
    ValidationError { message: String },
    
    #[error("外部服务错误: {service} - {error}")]
    ExternalServiceError { service: String, error: String },
    
    #[error("资源不足: {resource}")]
    ResourceExhausted { resource: String },
}

impl From<CustomToolError> for WorkflowError {
    fn from(error: CustomToolError) -> Self {
        WorkflowError::tool_execution(&error.to_string())
    }
}

// 工具实现中的错误处理
async fn robust_tool_execution(params: Value, _context: ExecutionContext) -> Result<Value> {
    // 参数验证
    let input = params.get("input")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CustomToolError::ValidationError {
            message: "Missing or invalid 'input' parameter".to_string()
        })?;
    
    // 外部服务调用
    let result = match external_service_call(input).await {
        Ok(data) => data,
        Err(e) => return Err(CustomToolError::ExternalServiceError {
            service: "data_api".to_string(),
            error: e.to_string(),
        }.into()),
    };
    
    // 资源检查
    if result.len() > 1_000_000 {
        return Err(CustomToolError::ResourceExhausted {
            resource: "memory".to_string(),
        }.into());
    }
    
    Ok(json!({
        "result": result,
        "processed_at": chrono::Utc::now().to_rfc3339()
    }))
}
```

### 2. 性能优化

```rust
use workflow_toolkit::{tools::*, storage::*};
use std::sync::Arc;
use tokio::sync::RwLock;

// 使用缓存提高性能
pub struct CachedTool {
    name: String,
    cache: Arc<RwLock<std::collections::HashMap<String, (Value, std::time::Instant)>>>,
    cache_ttl: std::time::Duration,
}

impl CachedTool {
    pub fn new(name: String, cache_ttl: std::time::Duration) -> Self {
        Self {
            name,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            cache_ttl,
        }
    }
    
    async fn get_cached_result(&self, key: &str) -> Option<Value> {
        let cache = self.cache.read().await;
        if let Some((value, timestamp)) = cache.get(key) {
            if timestamp.elapsed() < self.cache_ttl {
                return Some(value.clone());
            }
        }
        None
    }
    
    async fn set_cached_result(&self, key: String, value: Value) {
        let mut cache = self.cache.write().await;
        cache.insert(key, (value, std::time::Instant::now()));
        
        // 清理过期缓存
        let now = std::time::Instant::now();
        cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.cache_ttl);
    }
}

#[async_trait::async_trait]
impl ToolNode for CachedTool {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        // 生成缓存键
        let cache_key = format!("{:x}", md5::compute(params.to_string().as_bytes()));
        
        // 检查缓存
        if let Some(cached_result) = self.get_cached_result(&cache_key).await {
            return Ok(cached_result);
        }
        
        // 执行实际计算
        let result = self.expensive_computation(params).await?;
        
        // 缓存结果
        self.set_cached_result(cache_key, result.clone()).await;
        
        Ok(result)
    }
    
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
    
    fn get_schema(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            description: "Cached computation tool".to_string(),
            parameters_schema: json!({}),
            return_schema: json!({}),
            dependencies: Vec::new(),
            metadata: std::collections::HashMap::new(),
            plugin_info: None,
        }
    }
    
    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        None
    }
}

impl CachedTool {
    async fn expensive_computation(&self, params: Value) -> Result<Value> {
        // 模拟耗时计算
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        Ok(json!({
            "result": "computed_value",
            "input": params,
            "computed_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}
```

### 3. 监控和日志

```rust
use tracing::{info, warn, error, debug, instrument};
use workflow_toolkit::*;

#[instrument(skip(engine))]
async fn execute_workflow_with_monitoring(
    engine: Arc<dyn WorkflowEngine>,
    definition: WorkflowDefinition,
) -> Result<WorkflowExecution> {
    let start_time = std::time::Instant::now();
    
    info!(
        workflow_name = %definition.name,
        workflow_version = %definition.version,
        node_count = definition.nodes.len(),
        "开始执行工作流"
    );
    
    // 执行工作流
    let execution = match engine.execute_workflow(definition).await {
        Ok(exec) => {
            info!(
                execution_id = %exec.id,
                elapsed_ms = start_time.elapsed().as_millis(),
                "工作流启动成功"
            );
            exec
        }
        Err(e) => {
            error!(
                error = %e,
                elapsed_ms = start_time.elapsed().as_millis(),
                "工作流启动失败"
            );
            return Err(e);
        }
    };
    
    // 监控执行状态
    let monitoring_task = {
        let engine = engine.clone();
        let execution_id = execution.id;
        
        tokio::spawn(async move {
            let mut last_status = ExecutionStatus::Pending;
            let mut check_count = 0;
            
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                check_count += 1;
                
                match engine.get_workflow_status(execution_id).await {
                    Ok(status) => {
                        if status != last_status {
                            info!(
                                execution_id = %execution_id,
                                old_status = ?last_status,
                                new_status = ?status,
                                check_count = check_count,
                                "工作流状态变更"
                            );
                            last_status = status.clone();
                        }
                        
                        if status.is_terminal() {
                            info!(
                                execution_id = %execution_id,
                                final_status = ?status,
                                total_checks = check_count,
                                "工作流执行完成"
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        warn!(
                            execution_id = %execution_id,
                            error = %e,
                            check_count = check_count,
                            "获取工作流状态失败"
                        );
                    }
                }
            }
        })
    };
    
    // 等待监控任务完成
    let _ = monitoring_task.await;
    
    Ok(execution)
}

// 性能指标收集
pub struct PerformanceMetrics {
    pub execution_count: std::sync::atomic::AtomicU64,
    pub success_count: std::sync::atomic::AtomicU64,
    pub failure_count: std::sync::atomic::AtomicU64,
    pub total_execution_time: std::sync::atomic::AtomicU64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            execution_count: std::sync::atomic::AtomicU64::new(0),
            success_count: std::sync::atomic::AtomicU64::new(0),
            failure_count: std::sync::atomic::AtomicU64::new(0),
            total_execution_time: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    pub fn record_execution(&self, duration: std::time::Duration, success: bool) {
        use std::sync::atomic::Ordering;
        
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        
        if success {
            self.success_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    pub fn get_stats(&self) -> (u64, u64, u64, f64) {
        use std::sync::atomic::Ordering;
        
        let executions = self.execution_count.load(Ordering::Relaxed);
        let successes = self.success_count.load(Ordering::Relaxed);
        let failures = self.failure_count.load(Ordering::Relaxed);
        let total_time = self.total_execution_time.load(Ordering::Relaxed);
        
        let avg_time = if executions > 0 {
            total_time as f64 / executions as f64
        } else {
            0.0
        };
        
        (executions, successes, failures, avg_time)
    }
}
```

## 故障排除

### 常见问题和解决方案

1. **工作流执行卡住**
   - 检查节点依赖关系是否正确
   - 确认所有依赖的工具都已注册
   - 检查是否存在死锁或循环依赖

2. **工具执行失败**
   - 验证工具参数格式和类型
   - 检查工具的依赖是否满足
   - 查看详细的错误日志

3. **性能问题**
   - 启用结果缓存
   - 优化并发设置
   - 监控资源使用情况

4. **插件加载失败**
   - 检查插件路径和权限
   - 确认插件依赖已安装
   - 查看插件初始化日志

更多详细信息请参考[API参考文档](API_REFERENCE.md)和[用户手册](USER_MANUAL.md)。