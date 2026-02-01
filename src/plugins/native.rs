//! Native plugin implementation using dynamic library loading

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus, SecurityPolicy};
use crate::tools::{BasicTool, ToolExecutor, ToolNode};
use async_trait::async_trait;
use libloading::{Library, Symbol};
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Native plugin implementation
pub struct NativePlugin {
    info: PluginInfo,
    library_path: PathBuf,
    library: Option<Library>,
    tools: Vec<Arc<dyn ToolNode>>,
    status: PluginStatus,
    config: Option<PluginConfig>,
    plugin_handle: Option<*mut c_void>,
}

// Native plugin API function signatures
#[allow(dead_code)]
type PluginInfoFn = unsafe extern "C" fn() -> *const c_char;
type PluginInitFn = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type PluginGetToolsFn = unsafe extern "C" fn(*mut c_void) -> *const ToolDescriptor;
type PluginExecuteToolFn =
    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, *const c_char) -> *const c_char;
type PluginShutdownFn = unsafe extern "C" fn(*mut c_void);

/// Tool descriptor from native plugin
#[repr(C)]
pub struct ToolDescriptor {
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub parameters_schema: *const c_char,
    pub return_schema: *const c_char,
    pub next: *const ToolDescriptor,
}

impl NativePlugin {
    /// Create a new native plugin
    pub fn new(info: PluginInfo, library_path: PathBuf) -> Self {
        Self {
            info,
            library_path,
            library: None,
            tools: Vec::new(),
            status: PluginStatus::Uninitialized,
            config: None,
            plugin_handle: None,
        }
    }

    /// Load the dynamic library
    fn load_library(&mut self) -> Result<()> {
        debug!("Loading native library: {:?}", self.library_path);

        // Check if library file exists
        if !self.library_path.exists() {
            return Err(WorkflowError::plugin(format!(
                "Library file not found: {:?}",
                self.library_path
            )));
        }

        // Load the library
        let library = unsafe {
            Library::new(&self.library_path).map_err(|e| {
                WorkflowError::plugin(format!(
                    "Failed to load library {:?}: {}",
                    self.library_path, e
                ))
            })?
        };

        // Verify required symbols exist
        self.verify_symbols(&library)?;

        self.library = Some(library);
        debug!(
            "Successfully loaded native library: {:?}",
            self.library_path
        );
        Ok(())
    }

    /// Verify that all required symbols exist in the library
    fn verify_symbols(&self, library: &Library) -> Result<()> {
        let required_symbols = [
            "plugin_info",
            "plugin_init",
            "plugin_get_tools",
            "plugin_execute_tool",
            "plugin_shutdown",
        ];

        for symbol_name in &required_symbols {
            unsafe {
                let _: Symbol<unsafe extern "C" fn()> =
                    library.get(symbol_name.as_bytes()).map_err(|e| {
                        WorkflowError::plugin(format!(
                            "Required symbol '{}' not found in library {:?}: {}",
                            symbol_name, self.library_path, e
                        ))
                    })?;
            }
        }

        debug!(
            "All required symbols found in library: {:?}",
            self.library_path
        );
        Ok(())
    }

    /// Initialize the native plugin
    fn initialize_native_plugin(&mut self, config: &PluginConfig) -> Result<()> {
        let library = self
            .library
            .as_ref()
            .ok_or_else(|| WorkflowError::plugin("Library not loaded".to_string()))?;

        // Get the init function
        let init_fn: Symbol<PluginInitFn> = unsafe {
            library.get(b"plugin_init").map_err(|e| {
                WorkflowError::plugin(format!("Failed to get plugin_init symbol: {}", e))
            })?
        };

        // Serialize config to JSON
        let config_json = serde_json::to_string(config).map_err(|e| {
            WorkflowError::plugin(format!("Failed to serialize plugin config: {}", e))
        })?;

        let config_cstr = CString::new(config_json).map_err(|e| {
            WorkflowError::plugin(format!("Failed to create C string from config: {}", e))
        })?;

        // Call the init function
        let plugin_handle = unsafe { init_fn(config_cstr.as_ptr()) };

        if plugin_handle.is_null() {
            return Err(WorkflowError::plugin(
                "Plugin initialization returned null handle".to_string(),
            ));
        }

        self.plugin_handle = Some(plugin_handle);
        debug!("Native plugin initialized successfully");
        Ok(())
    }

    /// Load tools from the native plugin
    fn load_tools(&mut self) -> Result<()> {
        let library = self
            .library
            .as_ref()
            .ok_or_else(|| WorkflowError::plugin("Library not loaded".to_string()))?;

        let plugin_handle = self
            .plugin_handle
            .ok_or_else(|| WorkflowError::plugin("Plugin not initialized".to_string()))?;

        // Get the get_tools function
        let get_tools_fn: Symbol<PluginGetToolsFn> = unsafe {
            library.get(b"plugin_get_tools").map_err(|e| {
                WorkflowError::plugin(format!("Failed to get plugin_get_tools symbol: {}", e))
            })?
        };

        // Call the get_tools function
        let tools_ptr = unsafe { get_tools_fn(plugin_handle) };

        if tools_ptr.is_null() {
            warn!("Plugin returned no tools");
            return Ok(());
        }

        // Parse the tool descriptors
        let mut current_tool = tools_ptr;
        while !current_tool.is_null() {
            let tool_descriptor = unsafe { &*current_tool };

            // Convert C strings to Rust strings
            let name = unsafe {
                CStr::from_ptr(tool_descriptor.name)
                    .to_str()
                    .map_err(|e| WorkflowError::plugin(format!("Invalid tool name: {}", e)))?
                    .to_string()
            };

            let version = unsafe {
                CStr::from_ptr(tool_descriptor.version)
                    .to_str()
                    .map_err(|e| WorkflowError::plugin(format!("Invalid tool version: {}", e)))?
                    .to_string()
            };

            let description = unsafe {
                CStr::from_ptr(tool_descriptor.description)
                    .to_str()
                    .map_err(|e| WorkflowError::plugin(format!("Invalid tool description: {}", e)))?
                    .to_string()
            };

            let parameters_schema: Value = if tool_descriptor.parameters_schema.is_null() {
                Value::Null
            } else {
                let schema_str = unsafe {
                    CStr::from_ptr(tool_descriptor.parameters_schema)
                        .to_str()
                        .map_err(|e| {
                            WorkflowError::plugin(format!("Invalid parameters schema: {}", e))
                        })?
                };
                serde_json::from_str(schema_str).unwrap_or(Value::Null)
            };

            let return_schema: Value = if tool_descriptor.return_schema.is_null() {
                Value::Null
            } else {
                let schema_str = unsafe {
                    CStr::from_ptr(tool_descriptor.return_schema)
                        .to_str()
                        .map_err(|e| {
                            WorkflowError::plugin(format!("Invalid return schema: {}", e))
                        })?
                };
                serde_json::from_str(schema_str).unwrap_or(Value::Null)
            };

            // Create tool info
            let tool_info = ToolInfo {
                name: name.clone(),
                version,
                description,
                category: Some("native".to_string()),
                tags: vec!["native".to_string(), "plugin".to_string()],
                parameters_schema,
                return_schema,
                plugin_name: Some(self.info.name.clone()),
                dependencies: Vec::new(),
                version_requirements: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // Create native tool executor
            let executor = Arc::new(NativeToolExecutor::new(
                name.clone(),
                self.library_path.clone(),
                plugin_handle,
            ));

            // Create the tool
            let tool = BasicTool::from_executor(tool_info, executor, Some(self.info.clone()))?;
            self.tools.push(Arc::new(tool));

            debug!("Loaded native tool: {}", name);

            // Move to next tool
            current_tool = tool_descriptor.next;
        }

        info!("Loaded {} tools from native plugin", self.tools.len());
        Ok(())
    }

    /// Shutdown the native plugin
    fn shutdown_native_plugin(&mut self) -> Result<()> {
        if let (Some(library), Some(plugin_handle)) = (&self.library, self.plugin_handle) {
            // Get the shutdown function
            let shutdown_fn: Symbol<PluginShutdownFn> = unsafe {
                library.get(b"plugin_shutdown").map_err(|e| {
                    WorkflowError::plugin(format!("Failed to get plugin_shutdown symbol: {}", e))
                })?
            };

            // Call the shutdown function
            unsafe { shutdown_fn(plugin_handle) };

            self.plugin_handle = None;
            debug!("Native plugin shutdown completed");
        }

        // Clear tools
        self.tools.clear();

        // Drop the library (this will unload it)
        self.library = None;

        Ok(())
    }

    /// Validate security policy for native plugins
    fn validate_security_policy(&self, policy: &SecurityPolicy) -> Result<()> {
        if !policy.sandbox_enabled {
            warn!(
                "Native plugin '{}' has sandbox disabled - this may be unsafe",
                self.info.name
            );
        }

        if policy.allow_network_access {
            warn!(
                "Native plugin '{}' has network access enabled",
                self.info.name
            );
        }

        if policy.allow_file_system_access && policy.allowed_paths.is_empty() {
            return Err(WorkflowError::plugin(format!(
                "Native plugin '{}' has file system access enabled but no allowed paths specified",
                self.info.name
            )));
        }

        Ok(())
    }
}

impl Plugin for NativePlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        if self.status != PluginStatus::Uninitialized {
            return Err(WorkflowError::plugin(format!(
                "Plugin '{}' is already initialized",
                self.info.name
            )));
        }

        self.status = PluginStatus::Initializing;

        // Validate security policy
        self.validate_security_policy(&config.security_policy)?;

        // Load the dynamic library
        self.load_library().inspect_err(|_e| {
            self.status = PluginStatus::Error;
        })?;

        // Initialize the native plugin
        self.initialize_native_plugin(&config).inspect_err(|_e| {
            self.status = PluginStatus::Error;
        })?;

        // Load tools from the plugin
        self.load_tools().inspect_err(|_e| {
            self.status = PluginStatus::Error;
        })?;

        self.config = Some(config);
        self.status = PluginStatus::Ready;

        info!(
            "Native plugin '{}' initialized successfully",
            self.info.name
        );
        Ok(())
    }

    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        self.tools.clone()
    }

    fn shutdown(&mut self) -> Result<()> {
        if matches!(
            self.status,
            PluginStatus::Shutdown | PluginStatus::Uninitialized
        ) {
            return Ok(());
        }

        self.status = PluginStatus::ShuttingDown;

        self.shutdown_native_plugin().inspect_err(|_e| {
            self.status = PluginStatus::Error;
        })?;

        self.status = PluginStatus::Shutdown;
        info!("Native plugin '{}' shutdown successfully", self.info.name);
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
    }

    fn status(&self) -> PluginStatus {
        self.status
    }
}

unsafe impl Send for NativePlugin {}
unsafe impl Sync for NativePlugin {}

/// Native tool executor that calls into the dynamic library
pub struct NativeToolExecutor {
    tool_name: String,
    library_path: PathBuf,
    plugin_handle: *mut c_void,
}

impl NativeToolExecutor {
    pub fn new(tool_name: String, library_path: PathBuf, plugin_handle: *mut c_void) -> Self {
        Self {
            tool_name,
            library_path,
            plugin_handle,
        }
    }
}

#[async_trait]
impl ToolExecutor for NativeToolExecutor {
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        // Load the library (we need to do this each time since we don't store the library reference)
        let library = unsafe {
            Library::new(&self.library_path).map_err(|e| {
                WorkflowError::tool(format!("Failed to load library for tool execution: {}", e))
            })?
        };

        // Get the execute function
        let execute_fn: Symbol<PluginExecuteToolFn> = unsafe {
            library.get(b"plugin_execute_tool").map_err(|e| {
                WorkflowError::tool(format!("Failed to get plugin_execute_tool symbol: {}", e))
            })?
        };

        // Serialize parameters and context
        let params_json = serde_json::to_string(&params)
            .map_err(|e| WorkflowError::tool(format!("Failed to serialize parameters: {}", e)))?;

        let context_json = serde_json::to_string(&context)
            .map_err(|e| WorkflowError::tool(format!("Failed to serialize context: {}", e)))?;

        let tool_name_cstr = CString::new(self.tool_name.clone()).map_err(|e| {
            WorkflowError::tool(format!("Failed to create C string from tool name: {}", e))
        })?;

        let params_cstr = CString::new(params_json).map_err(|e| {
            WorkflowError::tool(format!("Failed to create C string from parameters: {}", e))
        })?;

        let context_cstr = CString::new(context_json).map_err(|e| {
            WorkflowError::tool(format!("Failed to create C string from context: {}", e))
        })?;

        // Call the execute function
        let result_ptr = unsafe {
            execute_fn(
                self.plugin_handle,
                tool_name_cstr.as_ptr(),
                params_cstr.as_ptr(),
                context_cstr.as_ptr(),
            )
        };

        if result_ptr.is_null() {
            return Err(WorkflowError::tool(
                "Tool execution returned null result".to_string(),
            ));
        }

        // Convert result back to Rust
        let result_str = unsafe {
            CStr::from_ptr(result_ptr)
                .to_str()
                .map_err(|e| WorkflowError::tool(format!("Invalid result string: {}", e)))?
        };

        let result: Value = serde_json::from_str(result_str)
            .map_err(|e| WorkflowError::tool(format!("Failed to deserialize result: {}", e)))?;

        Ok(result)
    }
}

unsafe impl Send for NativeToolExecutor {}
unsafe impl Sync for NativeToolExecutor {}

/// Builder for native plugins
pub struct NativePluginBuilder {
    info: Option<PluginInfo>,
    library_path: Option<PathBuf>,
}

impl NativePluginBuilder {
    pub fn new() -> Self {
        Self {
            info: None,
            library_path: None,
        }
    }

    pub fn info(mut self, info: PluginInfo) -> Self {
        self.info = Some(info);
        self
    }

    pub fn library_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.library_path = Some(path.into());
        self
    }

    pub fn build(self) -> Result<NativePlugin> {
        let info = self
            .info
            .ok_or_else(|| WorkflowError::ValidationError("Plugin info is required".to_string()))?;

        let library_path = self.library_path.ok_or_else(|| {
            WorkflowError::ValidationError("Library path is required".to_string())
        })?;

        Ok(NativePlugin::new(info, library_path))
    }
}

impl Default for NativePluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}
