//! Python plugin implementation for executing Python-based tools

use crate::core::{ExecutionContext, PluginInfo, PluginType, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus};
use crate::tools::types::Tool;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command as AsyncCommand;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{debug, info};

/// Python runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonRuntimeConfig {
    /// Path to Python executable (defaults to "python3")
    pub python_executable: String,
    /// Virtual environment path
    pub virtual_env_path: Option<PathBuf>,
    /// Requirements file path
    pub requirements_file: Option<PathBuf>,
    /// Additional Python paths
    pub python_paths: Vec<PathBuf>,
    /// Environment variables
    pub environment_variables: HashMap<String, String>,
    /// Working directory
    pub working_directory: Option<PathBuf>,
}

impl Default for PythonRuntimeConfig {
    fn default() -> Self {
        Self {
            python_executable: "python3".to_string(),
            virtual_env_path: None,
            requirements_file: None,
            python_paths: Vec::new(),
            environment_variables: HashMap::new(),
            working_directory: None,
        }
    }
}

/// Python virtual environment manager
#[derive(Debug)]
pub struct PythonEnvironment {
    config: PythonRuntimeConfig,
    venv_path: Option<PathBuf>,
    python_executable: PathBuf,
    is_initialized: bool,
}

impl PythonEnvironment {
    /// Create a new Python environment
    pub fn new(config: PythonRuntimeConfig) -> Self {
        let python_executable = if let Some(venv_path) = &config.virtual_env_path {
            if cfg!(windows) {
                venv_path.join("Scripts").join("python.exe")
            } else {
                venv_path.join("bin").join("python")
            }
        } else {
            PathBuf::from(&config.python_executable)
        };

        Self {
            venv_path: config.virtual_env_path.clone(),
            config,
            python_executable,
            is_initialized: false,
        }
    }

    /// Initialize the Python environment
    pub async fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        info!("Initializing Python environment");

        // Create virtual environment if specified
        if let Some(venv_path) = &self.venv_path {
            self.create_virtual_environment(venv_path).await?;
        }

        // Install requirements if specified
        if let Some(requirements_file) = &self.config.requirements_file {
            self.install_requirements(requirements_file).await?;
        }

        self.is_initialized = true;
        info!("Python environment initialized successfully");
        Ok(())
    }

    /// Create a virtual environment
    async fn create_virtual_environment(&self, venv_path: &Path) -> Result<()> {
        if venv_path.exists() {
            debug!("Virtual environment already exists at: {:?}", venv_path);
            return Ok(());
        }

        info!("Creating virtual environment at: {:?}", venv_path);

        let output = AsyncCommand::new(&self.config.python_executable)
            .args(["-m", "venv"])
            .arg(venv_path)
            .output()
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!("Failed to create virtual environment: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkflowError::plugin(format!(
                "Failed to create virtual environment: {}",
                stderr
            )));
        }

        info!("Virtual environment created successfully");
        Ok(())
    }

    /// Install requirements from requirements.txt
    async fn install_requirements(&self, requirements_file: &Path) -> Result<()> {
        if !requirements_file.exists() {
            return Err(WorkflowError::plugin(format!(
                "Requirements file not found: {:?}",
                requirements_file
            )));
        }

        info!("Installing requirements from: {:?}", requirements_file);

        let output = AsyncCommand::new(&self.python_executable)
            .args(["-m", "pip", "install", "-r"])
            .arg(requirements_file)
            .output()
            .await
            .map_err(|e| WorkflowError::plugin(format!("Failed to install requirements: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkflowError::plugin(format!(
                "Failed to install requirements: {}",
                stderr
            )));
        }

        info!("Requirements installed successfully");
        Ok(())
    }

    /// Execute a Python script with given parameters
    pub async fn execute_script(
        &self,
        script_path: &Path,
        params: Value,
        context: ExecutionContext,
        timeout_duration: Option<Duration>,
    ) -> Result<Value> {
        if !self.is_initialized {
            return Err(WorkflowError::plugin(
                "Python environment not initialized".to_string(),
            ));
        }

        // Resolve script path relative to working directory if needed
        let (resolved_script_path, working_dir) = if script_path.is_absolute() {
            (
                script_path.to_path_buf(),
                self.config.working_directory.clone(),
            )
        } else if let Some(working_dir) = &self.config.working_directory {
            // If we have a working directory, use the script path as-is (relative to working dir)
            (script_path.to_path_buf(), Some(working_dir.clone()))
        } else {
            // No working directory, resolve relative to current directory
            (
                script_path.to_path_buf(),
                script_path.parent().map(|p| p.to_path_buf()),
            )
        };

        // Check if the script exists (resolve the full path for checking)
        let full_script_path = if let Some(ref wd) = working_dir {
            wd.join(&resolved_script_path)
        } else {
            resolved_script_path.clone()
        };

        if !full_script_path.exists() {
            return Err(WorkflowError::plugin(format!(
                "Python script not found: {:?} (resolved from {:?})",
                full_script_path, script_path
            )));
        }

        debug!("Executing Python script: {:?}", full_script_path);
        debug!("Script path for command: {:?}", resolved_script_path);
        debug!("Working directory: {:?}", working_dir);

        // Prepare the execution environment
        let mut cmd = AsyncCommand::new(&self.python_executable);
        cmd.arg(&resolved_script_path);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Set working directory (but don't change to script directory if working_directory is set)
        if let Some(wd) = working_dir {
            cmd.current_dir(wd);
        }

        // Set environment variables
        for (key, value) in &self.config.environment_variables {
            cmd.env(key, value);
        }

        // Add Python paths to PYTHONPATH
        if !self.config.python_paths.is_empty() {
            let python_path = self
                .config
                .python_paths
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" });
            cmd.env("PYTHONPATH", python_path);
        }

        // Set additional environment variables for better error reporting
        cmd.env("PYTHONUNBUFFERED", "1");
        cmd.env("PYTHONDONTWRITEBYTECODE", "1");

        // Prepare input data with enhanced context
        let enhanced_context = serde_json::json!({
            "execution_id": context.execution_id,
            "workflow_id": context.workflow_id,
            "user_id": context.user_id,
            "session_id": context.session_id,
            "started_at": context.started_at,
            "global_variables": context.global_variables,
            "script_path": full_script_path.to_string_lossy(),
            "python_executable": self.python_executable.to_string_lossy(),
        });

        let input_data = serde_json::json!({
            "params": params,
            "context": enhanced_context
        });

        let input_json = serde_json::to_string(&input_data)
            .map_err(|e| WorkflowError::plugin(format!("Failed to serialize input data: {}", e)))?;

        // Execute the command with timeout and enhanced error handling
        let execution_future = async {
            let mut child = cmd.spawn().map_err(|e| {
                WorkflowError::plugin(format!(
                    "Failed to spawn Python process for script {:?}: {}",
                    full_script_path, e
                ))
            })?;

            // Write input data to stdin
            if let Some(stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let mut stdin = stdin;
                stdin.write_all(input_json.as_bytes()).await.map_err(|e| {
                    WorkflowError::plugin(format!(
                        "Failed to write input data to Python process: {}",
                        e
                    ))
                })?;
                stdin.shutdown().await.map_err(|e| {
                    WorkflowError::plugin(format!("Failed to close Python process stdin: {}", e))
                })?;
            }

            // Wait for the process to complete
            let output = child.wait_with_output().await.map_err(|e| {
                WorkflowError::plugin(format!(
                    "Failed to wait for Python process completion: {}",
                    e
                ))
            })?;

            Ok::<_, WorkflowError>(output)
        };

        let output = if let Some(timeout_duration) = timeout_duration {
            timeout(timeout_duration, execution_future)
                .await
                .map_err(|_| {
                    WorkflowError::plugin(format!(
                        "Python script execution timed out after {:?}: {:?}",
                        timeout_duration, full_script_path
                    ))
                })?
        } else {
            execution_future.await
        }?;

        // Enhanced error handling and result parsing
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Log stderr for debugging (even if process succeeded)
        if !stderr.is_empty() {
            debug!("Python script stderr output: {}", stderr);
        }

        // Check if the process succeeded
        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);

            // Try to parse stdout as JSON error first
            if let Ok(error_json) = serde_json::from_str::<Value>(&stdout) {
                if let Some(error_msg) = error_json.get("error").and_then(|e| e.as_str()) {
                    return Err(WorkflowError::plugin(format!(
                        "Python script execution failed (exit code {}): {}",
                        exit_code, error_msg
                    )));
                }
            }

            // Fallback to stderr or generic error
            let error_message = if !stderr.is_empty() {
                stderr.to_string()
            } else if !stdout.is_empty() {
                stdout.to_string()
            } else {
                format!("Python script failed with exit code {}", exit_code)
            };

            return Err(WorkflowError::plugin(format!(
                "Python script execution failed: {}",
                error_message
            )));
        }

        // Parse the output as JSON
        if stdout.trim().is_empty() {
            return Err(WorkflowError::plugin(
                "Python script produced no output".to_string(),
            ));
        }

        let result: Value = serde_json::from_str(&stdout).map_err(|e| {
            WorkflowError::plugin(format!(
                "Failed to parse Python script output as JSON: {}. Raw output: {}",
                e, stdout
            ))
        })?;

        // Check if the result indicates an error
        if let Some(success) = result.get("success").and_then(|s| s.as_bool()) {
            if !success {
                if let Some(error_msg) = result.get("error").and_then(|e| e.as_str()) {
                    return Err(WorkflowError::plugin(format!(
                        "Python script reported error: {}",
                        error_msg
                    )));
                }
            }
        }

        debug!(
            "Python script executed successfully: {:?}",
            full_script_path
        );
        Ok(result)
    }

    /// Check if the environment is initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Get the Python executable path
    pub fn python_executable(&self) -> &Path {
        &self.python_executable
    }

    /// Validate that a Python script is compatible with the plugin system
    pub async fn validate_script(&self, script_path: &Path) -> Result<()> {
        if !script_path.exists() {
            return Err(WorkflowError::plugin(format!(
                "Python script not found: {:?}",
                script_path
            )));
        }

        if !script_path.is_file() {
            return Err(WorkflowError::plugin(format!(
                "Path is not a file: {:?}",
                script_path
            )));
        }

        // Check if the script has a .py extension
        if script_path.extension().and_then(|ext| ext.to_str()) != Some("py") {
            return Err(WorkflowError::plugin(format!(
                "Script must have .py extension: {:?}",
                script_path
            )));
        }

        // Try to compile the script to check for syntax errors
        let output = AsyncCommand::new(&self.python_executable)
            .args(["-m", "py_compile"])
            .arg(script_path)
            .output()
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!("Failed to validate Python script: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkflowError::plugin(format!(
                "Python script has syntax errors: {}",
                stderr
            )));
        }

        debug!("Python script validation passed: {:?}", script_path);
        Ok(())
    }

    /// Get information about the Python environment
    pub async fn get_environment_info(&self) -> Result<Value> {
        let output = AsyncCommand::new(&self.python_executable)
            .args(["-c", "import sys, json; print(json.dumps({'version': sys.version, 'executable': sys.executable, 'path': sys.path[:5]}))"])
            .output()
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!("Failed to get Python environment info: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkflowError::plugin(format!(
                "Failed to get Python environment info: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let info: Value = serde_json::from_str(&stdout).map_err(|e| {
            WorkflowError::plugin(format!("Failed to parse Python environment info: {}", e))
        })?;

        Ok(info)
    }
}

/// Python plugin implementation
pub struct PythonPlugin {
    info: PluginInfo,
    status: PluginStatus,
    config: Option<PluginConfig>,
    #[allow(dead_code)]
    runtime_config: PythonRuntimeConfig,
    environment: Arc<Mutex<PythonEnvironment>>,
    tools: Vec<Tool>,
}

impl PythonPlugin {
    /// Create a new Python plugin
    pub fn new(info: PluginInfo, runtime_config: PythonRuntimeConfig) -> Self {
        let environment = Arc::new(Mutex::new(PythonEnvironment::new(runtime_config.clone())));

        Self {
            info,
            status: PluginStatus::Uninitialized,
            config: None,
            runtime_config,
            environment,
            tools: Vec::new(),
        }
    }

    /// Create a Python plugin from a requirements.txt file
    pub fn from_requirements(
        info: PluginInfo,
        requirements_file: &Path,
        virtual_env_path: Option<PathBuf>,
    ) -> Result<Self> {
        if !requirements_file.exists() {
            return Err(WorkflowError::plugin(format!(
                "Requirements file not found: {:?}",
                requirements_file
            )));
        }

        let runtime_config = PythonRuntimeConfig {
            requirements_file: Some(requirements_file.to_path_buf()),
            virtual_env_path,
            ..Default::default()
        };

        Ok(Self::new(info, runtime_config))
    }

    /// Add a tool to the plugin
    pub async fn add_tool(
        &mut self,
        tool_info: ToolInfo,
        script_path: PathBuf,
        timeout: Option<Duration>,
    ) -> Result<()> {
        use crate::tools::types::{
            PythonTool, ResourceRequirements, ToolId, ToolKind, ToolMetadata,
        };

        let environment = self.environment.lock().await;
        let python_path = environment.python_executable().to_path_buf();
        drop(environment);

        let python_tool = PythonTool {
            id: ToolId::new(),
            metadata: Arc::new(ToolMetadata {
                info: tool_info.clone(),
                kind: ToolKind::Python,
                input_schema: None,
                output_schema: None,
                examples: Vec::new(),
                resource_requirements: ResourceRequirements::default(),
                version: tool_info.version.clone(),
            }),
            script_path,
            python_path,
            timeout_secs: timeout.map(|d| d.as_secs()).unwrap_or(300),
            middleware_stack: None,
        };

        self.tools.push(Tool::Python(Arc::new(python_tool)));
        Ok(())
    }

    /// Add a tool using PythonTool directly
    pub async fn add_basic_tool(
        &mut self,
        tool_info: ToolInfo,
        script_path: PathBuf,
        timeout: Option<Duration>,
    ) -> Result<()> {
        use crate::tools::types::{
            PythonTool, ResourceRequirements, ToolId, ToolKind, ToolMetadata,
        };

        let environment = self.environment.lock().await;
        let python_path = environment.python_executable().to_path_buf();
        drop(environment);

        let python_tool = PythonTool {
            id: ToolId::new(),
            metadata: Arc::new(ToolMetadata {
                info: tool_info.clone(),
                kind: ToolKind::Python,
                input_schema: None,
                output_schema: None,
                examples: Vec::new(),
                resource_requirements: ResourceRequirements::default(),
                version: tool_info.version.clone(),
            }),
            script_path,
            python_path,
            timeout_secs: timeout.map(|d| d.as_secs()).unwrap_or(300),
            middleware_stack: None,
        };

        self.tools.push(Tool::Python(Arc::new(python_tool)));
        Ok(())
    }

    /// Get the Python environment
    pub fn environment(&self) -> Arc<Mutex<PythonEnvironment>> {
        self.environment.clone()
    }
}

impl Plugin for PythonPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        info!("Initializing Python plugin: {}", self.info.name);
        self.status = PluginStatus::Initializing;

        // Store the config for later async initialization
        self.config = Some(config);
        self.status = PluginStatus::Ready;
        info!("Python plugin initialized successfully: {}", self.info.name);
        Ok(())
    }

    fn get_tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Python plugin: {}", self.info.name);
        self.status = PluginStatus::ShuttingDown;

        // Clear tools
        self.tools.clear();

        self.status = PluginStatus::Shutdown;
        info!("Python plugin shut down successfully: {}", self.info.name);
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
    }

    fn status(&self) -> PluginStatus {
        self.status
    }
}

impl PythonPlugin {
    /// Async initialization method that should be called after initialize()
    pub async fn initialize_async(&mut self) -> Result<()> {
        if self.status != PluginStatus::Ready {
            return Err(WorkflowError::plugin(
                "Plugin must be initialized before async initialization".to_string(),
            ));
        }

        // Initialize the Python environment
        let mut environment = self.environment.lock().await;
        environment.initialize().await?;

        info!(
            "Python plugin async initialization completed: {}",
            self.info.name
        );
        Ok(())
    }
}

/// Builder for Python plugins
pub struct PythonPluginBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    runtime_config: PythonRuntimeConfig,
    tools: Vec<(ToolInfo, PathBuf, Option<Duration>)>,
}

impl PythonPluginBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            runtime_config: PythonRuntimeConfig::default(),
            tools: Vec::new(),
        }
    }

    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn python_executable<S: Into<String>>(mut self, executable: S) -> Self {
        self.runtime_config.python_executable = executable.into();
        self
    }

    pub fn virtual_env_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.virtual_env_path = Some(path.into());
        self
    }

    pub fn requirements_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.requirements_file = Some(path.into());
        self
    }

    pub fn working_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.working_directory = Some(path.into());
        self
    }

    pub fn add_python_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.python_paths.push(path.into());
        self
    }

    pub fn add_environment_variable<K: Into<String>, V: Into<String>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.runtime_config
            .environment_variables
            .insert(key.into(), value.into());
        self
    }

    pub fn add_tool(
        mut self,
        tool_info: ToolInfo,
        script_path: PathBuf,
        timeout: Option<Duration>,
    ) -> Self {
        self.tools.push((tool_info, script_path, timeout));
        self
    }

    pub async fn build(self) -> Result<PythonPlugin> {
        let name = self
            .name
            .ok_or_else(|| WorkflowError::ValidationError("Plugin name is required".to_string()))?;
        let version = self.version.ok_or_else(|| {
            WorkflowError::ValidationError("Plugin version is required".to_string())
        })?;
        let description = self
            .description
            .unwrap_or_else(|| format!("Python plugin: {}", name));

        let _now = Utc::now();
        let plugin_info = PluginInfo {
            name: name.clone(),
            version,
            plugin_type: PluginType::Python,
            description: Some(description),
            author: None,
            homepage: None,
            metadata: HashMap::new(),
        };

        let mut plugin = PythonPlugin::new(plugin_info, self.runtime_config);

        // Add tools
        for (tool_info, script_path, timeout) in self.tools {
            plugin
                .add_basic_tool(tool_info, script_path, timeout)
                .await?;
        }

        Ok(plugin)
    }
}

impl Default for PythonPluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}
