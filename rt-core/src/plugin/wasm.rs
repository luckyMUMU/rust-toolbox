use std::path::PathBuf;
use async_trait::async_trait;
use serde_json::Value;
use crate::{Tool, Locale, Result, CoreError};
use super::manifest::PluginMetadata;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, sync::WasiCtxBuilder};
use wasi_common::pipe::{ReadPipe, WritePipe};

struct WasmState {
    wasi: WasiCtx,
}

#[derive(Debug)]
pub struct WasmPlugin {
    path: PathBuf,
    metadata: PluginMetadata,
}

impl WasmPlugin {
    pub async fn new(path: PathBuf) -> Result<Self> {
        // Run spec
        let metadata_val = Self::run_wasm(&path, vec!["spec".to_string()], None).await?;
        // If null (empty output), fail
        if metadata_val == Value::Null {
            return Err(CoreError::ToolFailure(format!("Plugin spec returned empty output for {:?}", path)));
        }
        
        // 首先尝试解析为单个对象
        match serde_json::from_value::<PluginMetadata>(metadata_val.clone()) {
            Ok(metadata) => {
                // 单个工具插件
                Ok(Self { path, metadata })
            },
            Err(e) => {
                // 如果单个对象解析失败，尝试解析为数组
                let error_msg = format!("Failed to parse plugin metadata: {}", e);
                Err(CoreError::ToolFailure(error_msg))
            },
        }
    }
    
    /// 从插件输出中提取所有工具元数据
    pub async fn extract_all_metadata(path: &PathBuf) -> Result<Vec<PluginMetadata>> {
        // Run spec
        let metadata_val = Self::run_wasm(path, vec!["spec".to_string()], None).await?;
        // If null (empty output), fail
        if metadata_val == Value::Null {
            return Err(CoreError::ToolFailure(format!("Plugin spec returned empty output for {:?}", path)));
        }
        
        // 首先尝试解析为数组
        match serde_json::from_value::<Vec<PluginMetadata>>(metadata_val.clone()) {
            Ok(metadata_list) => {
                // 多个工具插件
                Ok(metadata_list)
            },
            Err(_) => {
                // 如果数组解析失败，尝试解析为单个对象
                let metadata: PluginMetadata = serde_json::from_value(metadata_val)
                    .map_err(|e| CoreError::ToolFailure(format!("Failed to parse plugin metadata: {}", e)))?;
                // 返回包含单个工具的列表
                Ok(vec![metadata])
            },
        }
    }
    
    /// 根据元数据和插件路径创建插件实例
    pub fn from_metadata(metadata: PluginMetadata, path: PathBuf) -> Self {
        Self {
            path,
            metadata,
        }
    }

    async fn run_wasm(path: &PathBuf, args: Vec<String>, input: Option<String>) -> Result<Value> {
        let path = path.clone();
        tokio::task::spawn_blocking(move || {
            let engine = Engine::default();
            let mut linker: Linker<WasmState> = Linker::new(&engine);
            
            wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasmState| &mut s.wasi)
                .map_err(|e| CoreError::ToolFailure(format!("Failed to link WASI: {}", e)))?;

            let stdout = WritePipe::new_in_memory();
            let stdin = if let Some(s) = input {
                ReadPipe::from(s)
            } else {
                ReadPipe::from("")
            };

            let wasi = WasiCtxBuilder::new()
                .inherit_stderr()
                .stdin(Box::new(stdin))
                .stdout(Box::new(stdout.clone()))
                .args(&args)
                .map_err(|e| CoreError::ToolFailure(format!("Failed to set args: {}", e)))?
                .build();

            let state = WasmState { wasi };
            let mut store = Store::new(&engine, state);
            
            let module = Module::from_file(&engine, &path)
                .map_err(|e| CoreError::ToolFailure(format!("Failed to load Wasm module: {}", e)))?;
            
            linker.module(&mut store, "", &module)
                .map_err(|e| CoreError::ToolFailure(format!("Failed to link module: {}", e)))?;
            
            let instance = linker.instantiate(&mut store, &module)
                .map_err(|e| CoreError::ToolFailure(format!("Failed to instantiate module: {}", e)))?;
            
            let func = instance.get_typed_func::<(), ()>(&mut store, "_start")
                .map_err(|e| CoreError::ToolFailure(format!("Failed to get _start function: {}", e)))?;
            
            func.call(&mut store, ())
                .map_err(|e| CoreError::ToolFailure(format!("Wasm runtime error: {}", e)))?;

            drop(store);
            
            let output = stdout.try_into_inner()
                .expect("Sole owner of stdout pipe")
                .into_inner();

            if output.is_empty() {
                 return Ok(Value::Null);
            }

            serde_json::from_slice(&output)
                .map_err(|e| CoreError::ToolFailure(format!("Failed to parse output JSON: {}", e)))
        }).await.map_err(|e| CoreError::ToolFailure(format!("Wasm execution failed: {}", e)))?
    }
}

#[async_trait]
impl Tool for WasmPlugin {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn display_name(&self, locale: Locale) -> String {
        self.metadata.display_name.get(locale).to_string()
    }

    fn description(&self, locale: Locale) -> String {
        self.metadata.description.get(locale).to_string()
    }

    fn user_guide(&self, locale: Locale) -> String {
        self.metadata.user_guide.get(locale).to_string()
    }

    fn input_schema(&self, locale: Locale) -> Value {
        let mut schema = self.metadata.input_schema.clone();
        
        if let Some(fields) = &self.metadata.input_fields {
             if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
                 for (field_name, localized_title) in fields {
                     if let Some(field_schema) = props.get_mut(field_name) {
                         if let Some(field_obj) = field_schema.as_object_mut() {
                             field_obj.insert("title".to_string(), Value::String(localized_title.get(locale).to_string()));
                         }
                     }
                 }
             }
        }
        
        schema
    }

    fn output_schema(&self, locale: Locale) -> Value {
        let mut schema = self.metadata.output_schema.clone().unwrap_or_else(|| serde_json::json!({ "type": "object" }));

        if let Some(fields) = &self.metadata.output_fields {
             if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
                 for (field_name, localized_title) in fields {
                     if let Some(field_schema) = props.get_mut(field_name) {
                         if let Some(field_obj) = field_schema.as_object_mut() {
                             field_obj.insert("title".to_string(), Value::String(localized_title.get(locale).to_string()));
                         }
                     }
                 }
             }
        }
        
        schema
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let input_str = serde_json::to_string(&input)
             .map_err(|e| CoreError::ToolFailure(format!("Serialization error: {}", e)))?;
        Self::run_wasm(&self.path, vec!["run".to_string()], Some(input_str)).await
    }
    
    fn mcp_supported(&self) -> bool {
        self.metadata.mcp_supported
    }
    
    async fn run_with_context(&self, request: crate::mcp::McpRequest) -> Result<crate::mcp::McpResponse> {
        if self.metadata.mcp_supported {
            // 使用 MCP 模式运行 Wasm 插件
            let input_str = serde_json::to_string(&request)
                .map_err(|e| CoreError::ToolFailure(format!("Failed to serialize MCP request: {}", e)))?;
            let result = Self::run_wasm(&self.path, vec!["run-with-context".to_string()], Some(input_str)).await?;
            
            let mcp_response: crate::mcp::McpResponse = serde_json::from_value(result)
                .map_err(|e| CoreError::ToolFailure(format!("Failed to parse MCP response: {}", e)))?;
            
            Ok(mcp_response)
        } else {
            // 回退到默认实现 - 修复借用问题，先克隆需要的字段
            let params = request.params.clone();
            let request_copy = request.clone();
            let context = request.context;
            
            let result = self.run(params).await?;
            
            Ok(crate::mcp::McpResponse::success_from_request(
                &request_copy,
                result,
                context,
                None,
            ))
        }
    }
}
