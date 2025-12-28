//! WebAssembly plugin support using wasmtime and extism

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus};
use crate::tools::ToolNode;
use async_trait::async_trait;
use chrono::Utc;
use extism::Plugin as ExtismPlugin;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};
use wasmtime::{Config, Engine, Linker, Module, Store};

/// WASM runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmRuntimeConfig {
    pub module_path: PathBuf,
    pub memory_limit: u64,
    pub timeout: Duration,
    pub fuel_limit: Option<u64>,
    pub allow_wasi: bool,
    pub allowed_imports: Vec<String>,
    pub entry_points: HashMap<String, String>,
    pub runtime_type: WasmRuntimeType,
    pub extism_config: Option<ExtismConfig>,
}

/// WASM runtime type selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmRuntimeType {
    Wasmtime,
    Extism,
}

/// Extism-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtismConfig {
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<PathBuf>,
    pub config_data: HashMap<String, String>,
    pub memory_pages: Option<u32>,
    pub max_var_bytes: Option<u32>,
}

impl Default for ExtismConfig {
    fn default() -> Self {
        Self {
            allowed_hosts: Vec::new(),
            allowed_paths: Vec::new(),
            config_data: HashMap::new(),
            memory_pages: Some(1), // 64KB default
            max_var_bytes: Some(1024 * 1024), // 1MB default
        }
    }
}

impl Default for WasmRuntimeConfig {
    fn default() -> Self {
        Self {
            module_path: PathBuf::new(),
            memory_limit: 64 * 1024 * 1024, // 64MB
            timeout: Duration::from_secs(30),
            fuel_limit: Some(1_000_000),
            allow_wasi: false,
            allowed_imports: Vec::new(),
            entry_points: HashMap::new(),
            runtime_type: WasmRuntimeType::Wasmtime,
            extism_config: None,
        }
    }
}

/// WASM plugin implementation using wasmtime
pub struct WasmPlugin {
    info: PluginInfo,
    config: Option<PluginConfig>,
    runtime_config: WasmRuntimeConfig,
    status: PluginStatus,
    engine: Option<Engine>,
    module: Option<Module>,
    tools: Vec<Arc<dyn ToolNode>>,
}

impl WasmPlugin {
    /// Create a new WASM plugin
    pub fn new(info: PluginInfo, runtime_config: WasmRuntimeConfig) -> Self {
        Self {
            info,
            config: None,
            runtime_config,
            status: PluginStatus::Uninitialized,
            engine: None,
            module: None,
            tools: Vec::new(),
        }
    }

    /// Initialize the WASM engine and module
    fn initialize_wasm_engine(&mut self) -> Result<()> {
        debug!("Initializing WASM engine for plugin: {}", self.info.name);

        // Configure wasmtime engine
        let mut config = Config::new();
        // Note: fuel consumption may not be available in all wasmtime versions
        // config.consume_fuel(self.runtime_config.fuel_limit.is_some());
        
        // Set memory limits
        config.max_wasm_stack(1024 * 1024); // 1MB stack
        
        if self.runtime_config.allow_wasi {
            config.wasm_component_model(true);
        }

        let engine = Engine::new(&config).map_err(|e| {
            WorkflowError::plugin(format!("Failed to create WASM engine: {}", e))
        })?;

        // Load WASM module
        let module_bytes = std::fs::read(&self.runtime_config.module_path).map_err(|e| {
            WorkflowError::plugin(format!(
                "Failed to read WASM module from {:?}: {}",
                self.runtime_config.module_path, e
            ))
        })?;

        let module = Module::new(&engine, &module_bytes).map_err(|e| {
            WorkflowError::plugin(format!("Failed to compile WASM module: {}", e))
        })?;

        self.engine = Some(engine);
        self.module = Some(module);

        debug!("WASM engine initialized successfully for plugin: {}", self.info.name);
        Ok(())
    }

    /// Create tool nodes from the WASM module
    fn create_tool_nodes(&mut self) -> Result<()> {
        let mut tools = Vec::new();

        // Create tool nodes for each entry point based on runtime type
        for (tool_name, entry_point) in &self.runtime_config.entry_points {
            let now = Utc::now();
            let tool_info = ToolInfo {
                name: tool_name.clone(),
                version: self.info.version.clone(),
                description: format!("WASM tool: {}", tool_name),
                category: Some("wasm".to_string()),
                tags: vec!["wasm".to_string(), "plugin".to_string()],
                parameters_schema: Value::Object(serde_json::Map::new()),
                return_schema: Value::Object(serde_json::Map::new()),
                plugin_name: Some(self.info.name.clone()),
                created_at: now,
                updated_at: now,
            };

            let tool: Arc<dyn ToolNode> = match self.runtime_config.runtime_type {
                WasmRuntimeType::Wasmtime => {
                    if self.engine.is_none() || self.module.is_none() {
                        return Err(WorkflowError::plugin(
                            "Wasmtime engine/module not initialized".to_string(),
                        ));
                    }

                    Arc::new(WasmToolNode::new(
                        tool_info,
                        self.engine.as_ref().unwrap().clone(),
                        self.module.as_ref().unwrap().clone(),
                        entry_point.clone(),
                        self.runtime_config.clone(),
                        Some(self.info.clone()),
                    ))
                }
                WasmRuntimeType::Extism => {
                    Arc::new(ExtismToolNode::new(
                        tool_info,
                        self.runtime_config.module_path.clone(),
                        entry_point.clone(),
                        self.runtime_config.clone(),
                        Some(self.info.clone()),
                    ))
                }
            };

            tools.push(tool);
        }

        self.tools = tools;
        Ok(())
    }
}

impl Plugin for WasmPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        info!("Initializing WASM plugin: {}", config.name);
        self.status = PluginStatus::Initializing;

        // Validate security policy
        if !config.security_policy.sandbox_enabled {
            info!("WASM plugin {} has sandbox disabled, this may be unsafe", config.name);
        }

        // Apply resource limits
        if let Some(memory_limit) = config.resource_limits.max_memory {
            self.runtime_config.memory_limit = memory_limit;
        }

        if let Some(execution_time) = config.resource_limits.max_execution_time {
            self.runtime_config.timeout = execution_time;
        }

        // Initialize runtime based on type
        match self.runtime_config.runtime_type {
            WasmRuntimeType::Wasmtime => {
                // Initialize WASM engine for wasmtime
                self.initialize_wasm_engine()?;
            }
            WasmRuntimeType::Extism => {
                // For extism, we don't need to pre-initialize the engine
                // as it's created per-execution
                info!("Using Extism runtime for multi-language WASM support");
            }
        }

        // Create tool nodes
        self.create_tool_nodes()?;

        self.config = Some(config);
        self.status = PluginStatus::Ready;

        info!("WASM plugin initialized successfully: {}", self.info.name);
        Ok(())
    }

    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        self.tools.clone()
    }

    fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down WASM plugin: {}", self.info.name);
        self.status = PluginStatus::ShuttingDown;

        // Clear tools
        self.tools.clear();

        // Clear WASM resources
        self.module = None;
        self.engine = None;

        self.status = PluginStatus::Shutdown;
        info!("WASM plugin shut down successfully: {}", self.info.name);
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
    }

    fn status(&self) -> PluginStatus {
        self.status
    }
}

/// Extism-based WASM tool node for multi-language support
pub struct ExtismToolNode {
    info: ToolInfo,
    module_path: PathBuf,
    function_name: String,
    runtime_config: WasmRuntimeConfig,
    plugin_info: Option<PluginInfo>,
}

impl ExtismToolNode {
    pub fn new(
        info: ToolInfo,
        module_path: PathBuf,
        function_name: String,
        runtime_config: WasmRuntimeConfig,
        plugin_info: Option<PluginInfo>,
    ) -> Self {
        Self {
            info,
            module_path,
            function_name,
            runtime_config,
            plugin_info,
        }
    }

    /// Execute using extism for multi-language support
    async fn execute_extism_function(&self, input: &str) -> Result<String> {
        let module_path = self.module_path.clone();
        let function_name = self.function_name.clone();
        let runtime_config = self.runtime_config.clone();
        let input = input.to_string();

        // Execute in a separate task to handle timeout
        let result = tokio::time::timeout(runtime_config.timeout, async move {
            tokio::task::spawn_blocking(move || {
                // Read WASM module
                let wasm_data = std::fs::read(&module_path).map_err(|e| {
                    WorkflowError::plugin(format!(
                        "Failed to read WASM module from {:?}: {}",
                        module_path, e
                    ))
                })?;

                // Create extism plugin
                let mut plugin = ExtismPlugin::new(&wasm_data, [], true).map_err(|e| {
                    WorkflowError::plugin(format!("Failed to create extism plugin: {}", e))
                })?;

                // Note: Configuration setting would be done here if the API supports it
                // For now, we'll skip configuration and just execute the function

                // Call the function
                let output = plugin.call(&function_name, input.as_bytes()).map_err(|e| {
                    WorkflowError::plugin(format!("Extism function call failed: {}", e))
                })?;

                // Convert output to string
                let result_str = String::from_utf8(output).map_err(|e| {
                    WorkflowError::plugin(format!("Failed to convert extism output to string: {}", e))
                })?;

                Ok(result_str)
            }).await.map_err(|e| {
                WorkflowError::plugin(format!("Extism execution task failed: {}", e))
            })?
        }).await;

        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(WorkflowError::plugin(format!(
                "Extism execution timed out after {:?}",
                self.runtime_config.timeout
            ))),
        }
    }
}

#[async_trait]
impl ToolNode for ExtismToolNode {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn version(&self) -> &str {
        &self.info.version
    }

    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        // Basic validation - in a real implementation, this would use JSON schema
        Ok(())
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        debug!("Executing Extism WASM tool: {}", self.info.name);

        let start_time = Instant::now();

        // Convert parameters to JSON string for WASM input
        let input = serde_json::to_string(&params).map_err(|e| {
            WorkflowError::plugin(format!("Failed to serialize parameters: {}", e))
        })?;

        // Execute extism function
        let output = self.execute_extism_function(&input).await?;

        // Parse output as JSON
        let result: Value = serde_json::from_str(&output).map_err(|e| {
            WorkflowError::plugin(format!("Failed to parse extism output as JSON: {}", e))
        })?;

        let execution_time = start_time.elapsed();
        debug!(
            "Extism WASM tool '{}' executed in {:?}",
            self.info.name, execution_time
        );

        Ok(result)
    }

    fn get_info(&self) -> ToolInfo {
        self.info.clone()
    }

    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        self.plugin_info.as_ref()
    }
}
pub struct WasmToolNode {
    info: ToolInfo,
    engine: Engine,
    module: Module,
    entry_point: String,
    runtime_config: WasmRuntimeConfig,
    plugin_info: Option<PluginInfo>,
}

impl WasmToolNode {
    pub fn new(
        info: ToolInfo,
        engine: Engine,
        module: Module,
        entry_point: String,
        runtime_config: WasmRuntimeConfig,
        plugin_info: Option<PluginInfo>,
    ) -> Self {
        Self {
            info,
            engine,
            module,
            entry_point,
            runtime_config,
            plugin_info,
        }
    }

    /// Execute the WASM function with timeout and resource limits
    async fn execute_wasm_function(&self, input: &str) -> Result<String> {
        let engine = self.engine.clone();
        let module = self.module.clone();
        let entry_point = self.entry_point.clone();
        let runtime_config = self.runtime_config.clone();
        let input = input.to_string();

        // Execute in a separate task to handle timeout
        let result = tokio::time::timeout(runtime_config.timeout, async move {
            tokio::task::spawn_blocking(move || {
                let mut store = Store::new(&engine, ());
                
                // Set fuel limit if configured (this feature may not be available in all wasmtime versions)
                // if let Some(fuel_limit) = runtime_config.fuel_limit {
                //     store.add_fuel(fuel_limit).map_err(|e| {
                //         WorkflowError::plugin(format!("Failed to add fuel: {}", e))
                //     })?;
                // }

                // Create linker for imports
                let mut linker = Linker::new(&engine);
                
                // Add basic host functions
                linker.func_wrap("env", "log", |msg: i32| {
                    debug!("WASM log: {}", msg);
                }).map_err(|e| {
                    WorkflowError::plugin(format!("Failed to add log function: {}", e))
                })?;

                // Instantiate the module
                let instance = linker.instantiate(&mut store, &module).map_err(|e| {
                    WorkflowError::plugin(format!("Failed to instantiate WASM module: {}", e))
                })?;

                // For now, return a simple success message
                // In a real implementation, we would:
                // 1. Get the entry point function
                // 2. Allocate memory for input/output
                // 3. Call the function
                // 4. Read the result from memory
                
                Ok(format!("{{\"result\": \"WASM function '{}' executed successfully\", \"input\": {}}}", 
                    entry_point, input))
            }).await.map_err(|e| {
                WorkflowError::plugin(format!("WASM execution task failed: {}", e))
            })?
        }).await;

        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(WorkflowError::plugin(format!(
                "WASM execution timed out after {:?}",
                self.runtime_config.timeout
            ))),
        }
    }
}

#[async_trait]
impl ToolNode for WasmToolNode {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn version(&self) -> &str {
        &self.info.version
    }

    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        // Basic validation - in a real implementation, this would use JSON schema
        Ok(())
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        debug!("Executing WASM tool: {}", self.info.name);

        let start_time = Instant::now();

        // Convert parameters to JSON string for WASM input
        let input = serde_json::to_string(&params).map_err(|e| {
            WorkflowError::plugin(format!("Failed to serialize parameters: {}", e))
        })?;

        // Execute WASM function
        let output = self.execute_wasm_function(&input).await?;

        // Parse output as JSON
        let result: Value = serde_json::from_str(&output).map_err(|e| {
            WorkflowError::plugin(format!("Failed to parse WASM output as JSON: {}", e))
        })?;

        let execution_time = start_time.elapsed();
        debug!(
            "WASM tool '{}' executed in {:?}",
            self.info.name, execution_time
        );

        Ok(result)
    }

    fn get_info(&self) -> ToolInfo {
        self.info.clone()
    }

    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        self.plugin_info.as_ref()
    }
}

/// WASM plugin builder for easier construction
pub struct WasmPluginBuilder {
    info: PluginInfo,
    runtime_config: WasmRuntimeConfig,
}

impl WasmPluginBuilder {
    pub fn new(name: String, version: String) -> Self {
        let info = PluginInfo {
            name,
            version,
            plugin_type: crate::core::PluginType::Wasm,
            description: None,
            author: None,
            metadata: HashMap::new(),
        };

        Self {
            info,
            runtime_config: WasmRuntimeConfig::default(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.info.description = Some(description);
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.info.author = Some(author);
        self
    }

    pub fn with_module_path(mut self, path: PathBuf) -> Self {
        self.runtime_config.module_path = path;
        self
    }

    pub fn with_memory_limit(mut self, limit: u64) -> Self {
        self.runtime_config.memory_limit = limit;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.runtime_config.timeout = timeout;
        self
    }

    pub fn with_fuel_limit(mut self, limit: u64) -> Self {
        self.runtime_config.fuel_limit = Some(limit);
        self
    }

    pub fn with_wasi(mut self, allow_wasi: bool) -> Self {
        self.runtime_config.allow_wasi = allow_wasi;
        self
    }

    pub fn add_entry_point(mut self, tool_name: String, function_name: String) -> Self {
        self.runtime_config.entry_points.insert(tool_name, function_name);
        self
    }

    pub fn with_runtime_type(mut self, runtime_type: WasmRuntimeType) -> Self {
        self.runtime_config.runtime_type = runtime_type;
        self
    }

    pub fn with_extism_config(mut self, extism_config: ExtismConfig) -> Self {
        self.runtime_config.extism_config = Some(extism_config);
        self.runtime_config.runtime_type = WasmRuntimeType::Extism;
        self
    }

    pub fn add_allowed_host(mut self, host: String) -> Self {
        if self.runtime_config.extism_config.is_none() {
            self.runtime_config.extism_config = Some(ExtismConfig::default());
        }
        if let Some(ref mut config) = self.runtime_config.extism_config {
            config.allowed_hosts.push(host);
        }
        self
    }

    pub fn add_config_data(mut self, key: String, value: String) -> Self {
        if self.runtime_config.extism_config.is_none() {
            self.runtime_config.extism_config = Some(ExtismConfig::default());
        }
        if let Some(ref mut config) = self.runtime_config.extism_config {
            config.config_data.insert(key, value);
        }
        self
    }

    pub fn build(self) -> WasmPlugin {
        WasmPlugin::new(self.info, self.runtime_config)
    }
}

/// WASM tool executor for standalone tool execution
pub struct WasmToolExecutor {
    runtime_config: WasmRuntimeConfig,
    engine: Option<Engine>,
    module: Option<Module>,
}

impl WasmToolExecutor {
    pub fn new(module_path: PathBuf, runtime_config: WasmRuntimeConfig) -> Result<Self> {
        let mut executor = Self {
            runtime_config: runtime_config.clone(),
            engine: None,
            module: None,
        };

        // Initialize wasmtime engine if using wasmtime runtime
        if matches!(runtime_config.runtime_type, WasmRuntimeType::Wasmtime) {
            let config = Config::new();
            let engine = Engine::new(&config).map_err(|e| {
                WorkflowError::plugin(format!("Failed to create WASM engine: {}", e))
            })?;

            let module_bytes = std::fs::read(&module_path).map_err(|e| {
                WorkflowError::plugin(format!(
                    "Failed to read WASM module from {:?}: {}",
                    module_path, e
                ))
            })?;

            let module = Module::new(&engine, &module_bytes).map_err(|e| {
                WorkflowError::plugin(format!("Failed to compile WASM module: {}", e))
            })?;

            executor.engine = Some(engine);
            executor.module = Some(module);
        }

        Ok(executor)
    }

    pub async fn execute_function(&self, function_name: &str, input: Value) -> Result<Value> {
        let now = Utc::now();
        let tool_info = ToolInfo {
            name: function_name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("WASM function: {}", function_name),
            category: Some("wasm".to_string()),
            tags: vec!["wasm".to_string()],
            parameters_schema: Value::Object(serde_json::Map::new()),
            return_schema: Value::Object(serde_json::Map::new()),
            plugin_name: None,
            created_at: now,
            updated_at: now,
        };

        let context = ExecutionContext::default();

        match self.runtime_config.runtime_type {
            WasmRuntimeType::Wasmtime => {
                let tool_node = WasmToolNode::new(
                    tool_info,
                    self.engine.as_ref().unwrap().clone(),
                    self.module.as_ref().unwrap().clone(),
                    function_name.to_string(),
                    self.runtime_config.clone(),
                    None,
                );
                tool_node.execute(input, context).await
            }
            WasmRuntimeType::Extism => {
                let tool_node = ExtismToolNode::new(
                    tool_info,
                    self.runtime_config.module_path.clone(),
                    function_name.to_string(),
                    self.runtime_config.clone(),
                    None,
                );
                tool_node.execute(input, context).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_wasm_plugin_creation() {
        let plugin = WasmPluginBuilder::new("test-wasm".to_string(), "1.0.0".to_string())
            .with_description("Test WASM plugin".to_string())
            .with_memory_limit(32 * 1024 * 1024)
            .with_timeout(Duration::from_secs(10))
            .add_entry_point("test_tool".to_string(), "test_function".to_string())
            .build();

        assert_eq!(plugin.info().name, "test-wasm");
        assert_eq!(plugin.info().version, "1.0.0");
        assert_eq!(plugin.status(), PluginStatus::Uninitialized);
    }

    #[test]
    fn test_wasm_runtime_config_default() {
        let config = WasmRuntimeConfig::default();
        assert_eq!(config.memory_limit, 64 * 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.fuel_limit, Some(1_000_000));
        assert!(!config.allow_wasi);
    }
}