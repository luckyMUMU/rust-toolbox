//! Native plugin implementation using dynamic library loading

use crate::core::{ExecutionContext, PluginInfo};
use crate::error::{Result, WorkflowError};
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus, SecurityPolicy};
use crate::tools::types::{NativeToolBuilder, Tool, ToolInput, ToolOutput};
use libloading::{Library, Symbol};
use serde_json::Value;
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
    tools: Vec<Tool>,
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

        // SAFETY: Library loading is inherently unsafe as it involves loading
        // arbitrary code from disk. We mitigate risks by:
        // 1. Verifying the library file exists before loading
        // 2. Verifying required symbols exist after loading
        // 3. The library is expected to follow our plugin API contract
        // 4. Security policy validation is performed during initialization
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
            // SAFETY: We are only checking if the symbol exists, not calling it.
            // The library lifetime is tied to the NativePlugin struct, and we
            // verify all symbols before any actual function calls.
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

        // SAFETY: We have verified the symbol exists in verify_symbols().
        // The init function is expected to return a valid plugin handle or null.
        // We check for null return value before using the handle.
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

        // SAFETY: The init function is called with a valid C string pointer.
        // The config_cstr lifetime extends until this function returns.
        // We verify the returned handle is non-null before storing it.
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

        // SAFETY: We have verified the symbol exists and the plugin handle is valid.
        // The get_tools function returns a linked list of tool descriptors.
        let get_tools_fn: Symbol<PluginGetToolsFn> = unsafe {
            library.get(b"plugin_get_tools").map_err(|e| {
                WorkflowError::plugin(format!("Failed to get plugin_get_tools symbol: {}", e))
            })?
        };

        // SAFETY: plugin_handle was validated during initialization.
        // The returned pointer may be null if no tools are available.
        let tools_ptr = unsafe { get_tools_fn(plugin_handle) };

        if tools_ptr.is_null() {
            warn!("Plugin returned no tools");
            return Ok(());
        }

        // Parse the tool descriptors
        let mut current_tool = tools_ptr;
        while !current_tool.is_null() {
            // SAFETY: We verify current_tool is non-null before dereferencing.
            // The tool descriptor is expected to be a valid C struct.
            let tool_descriptor = unsafe { &*current_tool };

            // SAFETY: CStr::from_ptr reads until null terminator.
            // We validate the UTF-8 conversion before using the string.
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
                // SAFETY: We check for null before calling CStr::from_ptr.
                // The schema string is expected to be valid UTF-8 JSON.
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
                // SAFETY: We check for null before calling CStr::from_ptr.
                // The schema string is expected to be valid UTF-8 JSON.
                let schema_str = unsafe {
                    CStr::from_ptr(tool_descriptor.return_schema)
                        .to_str()
                        .map_err(|e| {
                            WorkflowError::plugin(format!("Invalid return schema: {}", e))
                        })?
                };
                serde_json::from_str(schema_str).unwrap_or(Value::Null)
            };

            // 捕获变量用于闭包
            let tool_name = name.clone();
            let library_path = self.library_path.clone();
            // 将 plugin_handle 转换为 usize 以便在线程间传递
            let plugin_handle_usize = plugin_handle as usize;

            // 使用 NativeToolBuilder 创建 Tool::Native
            let native_tool = NativeToolBuilder::new()
                .name(&name)
                .version(&version)
                .description(&description)
                .category("native")
                .tag("native")
                .tag("plugin")
                .executor(move |input: ToolInput, _ctx: ExecutionContext| {
                    let tool_name = tool_name.clone();
                    let library_path = library_path.clone();
                    let plugin_handle_usize = plugin_handle_usize;
                    async move {
                        // 使用 spawn_blocking 来执行非线程安全的库加载
                        let result = tokio::task::spawn_blocking(move || {
                            // SAFETY: Library loading is performed in a blocking task
                            // to avoid blocking the async runtime. The library path
                            // has been validated during plugin initialization.
                            let library = unsafe {
                                Library::new(&library_path).map_err(|e| {
                                    WorkflowError::tool(format!("加载库失败: {}", e))
                                })?
                            };

                            // SAFETY: We have verified this symbol exists during plugin loading.
                            // The symbol is obtained from a valid library instance.
                            let execute_fn: Symbol<PluginExecuteToolFn> = unsafe {
                                library.get(b"plugin_execute_tool").map_err(|e| {
                                    WorkflowError::tool(format!("获取执行函数失败: {}", e))
                                })?
                            };

                            // 序列化参数和上下文
                            let params_json =
                                serde_json::to_string(&input.params).map_err(|e| {
                                    WorkflowError::tool(format!("序列化参数失败: {}", e))
                                })?;

                            let tool_name_cstr = CString::new(tool_name.clone()).map_err(|e| {
                                WorkflowError::tool(format!("创建C字符串失败: {}", e))
                            })?;

                            let params_cstr = CString::new(params_json).map_err(|e| {
                                WorkflowError::tool(format!("创建参数C字符串失败: {}", e))
                            })?;

                            let context_cstr = CString::new("{}").map_err(|e| {
                                WorkflowError::tool(format!("创建上下文C字符串失败: {}", e))
                            })?;

                            // 将 usize 转回指针
                            // SAFETY: The plugin_handle was originally a valid pointer from
                            // plugin_init. Converting back from usize is safe as long as the
                            // plugin is still loaded and initialized.
                            let plugin_handle = plugin_handle_usize as *mut c_void;

                            // SAFETY: All C string pointers are valid and the plugin handle
                            // was validated during initialization. We check for null result.
                            let result_ptr = unsafe {
                                execute_fn(
                                    plugin_handle,
                                    tool_name_cstr.as_ptr(),
                                    params_cstr.as_ptr(),
                                    context_cstr.as_ptr(),
                                )
                            };

                            if result_ptr.is_null() {
                                return Err(WorkflowError::tool("工具执行返回空结果".to_string()));
                            }

                            // SAFETY: We verified result_ptr is non-null. CStr::from_ptr
                            // reads until null terminator and we validate UTF-8 conversion.
                            let result_str = unsafe {
                                CStr::from_ptr(result_ptr).to_str().map_err(|e| {
                                    WorkflowError::tool(format!("无效的结果字符串: {}", e))
                                })?
                            };

                            let result: Value = serde_json::from_str(result_str).map_err(|e| {
                                WorkflowError::tool(format!("反序列化结果失败: {}", e))
                            })?;

                            Ok::<_, WorkflowError>(result)
                        })
                        .await
                        .map_err(|e| WorkflowError::tool(format!("任务执行失败: {}", e)))?;

                        match result {
                            Ok(value) => Ok(ToolOutput::success(value)),
                            Err(e) => Err(e),
                        }
                    }
                })
                .build()
                .map_err(|e| WorkflowError::plugin(format!("创建工具失败: {}", e)))?;

            self.tools.push(Tool::Native(Arc::new(native_tool)));

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
            // SAFETY: We have verified the symbol exists and the plugin handle is valid.
            // This is the final cleanup call before the plugin is unloaded.
            let shutdown_fn: Symbol<PluginShutdownFn> = unsafe {
                library.get(b"plugin_shutdown").map_err(|e| {
                    WorkflowError::plugin(format!("Failed to get plugin_shutdown symbol: {}", e))
                })?
            };

            // SAFETY: plugin_handle was validated during initialization.
            // After this call, the handle is no longer valid.
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

    fn get_tools(&self) -> Vec<Tool> {
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
