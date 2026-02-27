//! Node.js plugin implementation for executing Node.js-based tools

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

/// Node.js runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeJsRuntimeConfig {
    /// Path to Node.js executable (defaults to "node")
    pub node_executable: String,
    /// Path to npm executable (defaults to "npm")
    pub npm_executable: String,
    /// Project directory containing package.json
    pub project_directory: Option<PathBuf>,
    /// package.json file path
    pub package_json: Option<PathBuf>,
    /// node_modules directory path
    pub node_modules_path: Option<PathBuf>,
    /// Additional Node.js module paths
    pub module_paths: Vec<PathBuf>,
    /// Environment variables
    pub environment_variables: HashMap<String, String>,
    /// Working directory
    pub working_directory: Option<PathBuf>,
    /// Auto-install dependencies from package.json
    pub auto_install_dependencies: bool,
}

impl Default for NodeJsRuntimeConfig {
    fn default() -> Self {
        Self {
            node_executable: "node".to_string(),
            npm_executable: "npm".to_string(),
            project_directory: None,
            package_json: None,
            node_modules_path: None,
            module_paths: Vec::new(),
            environment_variables: HashMap::new(),
            working_directory: None,
            auto_install_dependencies: true,
        }
    }
}

/// Package.json structure for dependency management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub scripts: Option<HashMap<String, String>>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    pub keywords: Option<Vec<String>>,
    pub author: Option<String>,
    pub license: Option<String>,
}

impl PackageJson {
    /// Load package.json from file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(WorkflowError::plugin(format!(
                "package.json not found: {:?}",
                path
            )));
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| WorkflowError::plugin(format!("Failed to read package.json: {}", e)))?;

        let package_json: PackageJson = serde_json::from_str(&content)
            .map_err(|e| WorkflowError::plugin(format!("Failed to parse package.json: {}", e)))?;

        Ok(package_json)
    }

    /// Get all dependencies (both dependencies and devDependencies)
    pub fn get_all_dependencies(&self) -> HashMap<String, String> {
        let mut all_deps = HashMap::new();

        if let Some(deps) = &self.dependencies {
            all_deps.extend(deps.clone());
        }

        if let Some(dev_deps) = &self.dev_dependencies {
            all_deps.extend(dev_deps.clone());
        }

        all_deps
    }
}

/// Node.js environment manager
#[derive(Debug)]
pub struct NodeJsEnvironment {
    config: NodeJsRuntimeConfig,
    package_json: Option<PackageJson>,
    node_executable: PathBuf,
    npm_executable: PathBuf,
    is_initialized: bool,
}

impl NodeJsEnvironment {
    /// Create a new Node.js environment
    pub fn new(config: NodeJsRuntimeConfig) -> Self {
        let node_executable = PathBuf::from(&config.node_executable);
        let npm_executable = PathBuf::from(&config.npm_executable);

        Self {
            config,
            package_json: None,
            node_executable,
            npm_executable,
            is_initialized: false,
        }
    }

    /// Initialize the Node.js environment
    pub async fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        info!("Initializing Node.js environment");

        // Verify Node.js installation
        self.verify_node_installation().await?;

        // Load package.json if specified
        if let Some(package_json_path) = &self.config.package_json {
            self.package_json = Some(PackageJson::load_from_file(package_json_path).await?);
            info!("Loaded package.json from: {:?}", package_json_path);
        } else if let Some(project_dir) = &self.config.project_directory {
            let package_json_path = project_dir.join("package.json");
            if package_json_path.exists() {
                self.package_json = Some(PackageJson::load_from_file(&package_json_path).await?);
                info!(
                    "Loaded package.json from project directory: {:?}",
                    package_json_path
                );
            }
        }

        // Install dependencies if auto-install is enabled and package.json exists
        if self.config.auto_install_dependencies && self.package_json.is_some() {
            self.install_dependencies().await?;
        }

        self.is_initialized = true;
        info!("Node.js environment initialized successfully");
        Ok(())
    }

    /// Verify Node.js installation
    async fn verify_node_installation(&self) -> Result<()> {
        debug!("Verifying Node.js installation");

        // Check Node.js version
        let node_output = AsyncCommand::new(&self.node_executable)
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!(
                    "Failed to execute Node.js ({}): {}. Please ensure Node.js is installed and in PATH.",
                    self.node_executable.display(), e
                ))
            })?;

        if !node_output.status.success() {
            let stderr = String::from_utf8_lossy(&node_output.stderr);
            return Err(WorkflowError::plugin(format!(
                "Node.js version check failed: {}",
                stderr
            )));
        }

        let node_version = String::from_utf8_lossy(&node_output.stdout)
            .trim()
            .to_string();
        info!("Node.js version: {}", node_version);

        // Only check npm if auto_install_dependencies is enabled
        if self.config.auto_install_dependencies {
            // Check npm version
            let npm_output = AsyncCommand::new(&self.npm_executable)
                .arg("--version")
                .output()
                .await
                .map_err(|e| {
                    WorkflowError::plugin(format!(
                        "Failed to execute npm ({}): {}. Please ensure npm is installed and in PATH.",
                        self.npm_executable.display(), e
                    ))
                })?;

            if !npm_output.status.success() {
                let stderr = String::from_utf8_lossy(&npm_output.stderr);
                return Err(WorkflowError::plugin(format!(
                    "npm version check failed: {}",
                    stderr
                )));
            }

            let npm_version = String::from_utf8_lossy(&npm_output.stdout)
                .trim()
                .to_string();
            info!("npm version: {}", npm_version);
        } else {
            debug!("Skipping npm verification (auto_install_dependencies is disabled)");
        }

        Ok(())
    }

    /// Install dependencies from package.json
    async fn install_dependencies(&self) -> Result<()> {
        let working_dir = self.get_working_directory()?;

        info!("Installing Node.js dependencies in: {:?}", working_dir);

        let output = AsyncCommand::new(&self.npm_executable)
            .arg("install")
            .current_dir(&working_dir)
            .output()
            .await
            .map_err(|e| WorkflowError::plugin(format!("Failed to run npm install: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(WorkflowError::plugin(format!(
                "npm install failed:\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        info!("Dependencies installed successfully");
        Ok(())
    }

    /// Get the working directory for Node.js operations
    fn get_working_directory(&self) -> Result<PathBuf> {
        if let Some(working_dir) = &self.config.working_directory {
            Ok(working_dir.clone())
        } else if let Some(project_dir) = &self.config.project_directory {
            Ok(project_dir.clone())
        } else if let Some(package_json_path) = &self.config.package_json {
            if let Some(parent) = package_json_path.parent() {
                Ok(parent.to_path_buf())
            } else {
                Ok(std::env::current_dir().map_err(|e| {
                    WorkflowError::plugin(format!("Failed to get current directory: {}", e))
                })?)
            }
        } else {
            Ok(std::env::current_dir().map_err(|e| {
                WorkflowError::plugin(format!("Failed to get current directory: {}", e))
            })?)
        }
    }

    /// Execute a Node.js script with given parameters
    pub async fn execute_script(
        &self,
        script_path: &Path,
        params: Value,
        context: ExecutionContext,
        timeout_duration: Option<Duration>,
    ) -> Result<Value> {
        if !self.is_initialized {
            return Err(WorkflowError::plugin(
                "Node.js environment not initialized".to_string(),
            ));
        }

        let working_dir = self.get_working_directory()?;

        // Resolve script path relative to working directory if needed
        let resolved_script_path = if script_path.is_absolute() {
            script_path.to_path_buf()
        } else {
            // Use the script path as-is relative to the working directory
            script_path.to_path_buf()
        };

        // Check if the script exists (resolve the full path for checking)
        let full_script_path = working_dir.join(&resolved_script_path);
        if !full_script_path.exists() {
            return Err(WorkflowError::plugin(format!(
                "Node.js script not found: {:?} (resolved from {:?})",
                full_script_path, script_path
            )));
        }

        debug!("Executing Node.js script: {:?}", full_script_path);
        debug!("Working directory: {:?}", working_dir);

        // Prepare the execution environment
        let mut cmd = AsyncCommand::new(&self.node_executable);
        cmd.arg(&resolved_script_path);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(&working_dir);

        // Set environment variables
        for (key, value) in &self.config.environment_variables {
            cmd.env(key, value);
        }

        // Add module paths to NODE_PATH
        if !self.config.module_paths.is_empty() {
            let node_path = self
                .config
                .module_paths
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" });
            cmd.env("NODE_PATH", node_path);
        }

        // Set node_modules path if specified
        if let Some(node_modules_path) = &self.config.node_modules_path {
            let current_node_path = std::env::var("NODE_PATH").unwrap_or_default();
            let new_node_path = if current_node_path.is_empty() {
                node_modules_path.to_string_lossy().to_string()
            } else {
                format!(
                    "{}{}{}",
                    node_modules_path.to_string_lossy(),
                    if cfg!(windows) { ";" } else { ":" },
                    current_node_path
                )
            };
            cmd.env("NODE_PATH", new_node_path);
        }

        // Prepare input data with enhanced context
        let enhanced_context = serde_json::json!({
            "execution_id": context.execution_id,
            "workflow_id": context.workflow_id,
            "user_id": context.user_id,
            "session_id": context.session_id,
            "started_at": context.started_at,
            "global_variables": context.global_variables,
            "script_path": full_script_path.to_string_lossy(),
            "node_executable": self.node_executable.to_string_lossy(),
            "working_directory": working_dir.to_string_lossy(),
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
                    "Failed to spawn Node.js process for script {:?}: {}",
                    full_script_path, e
                ))
            })?;

            // Write input data to stdin
            if let Some(stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let mut stdin = stdin;
                stdin.write_all(input_json.as_bytes()).await.map_err(|e| {
                    WorkflowError::plugin(format!(
                        "Failed to write input data to Node.js process: {}",
                        e
                    ))
                })?;
                stdin.shutdown().await.map_err(|e| {
                    WorkflowError::plugin(format!("Failed to close Node.js process stdin: {}", e))
                })?;
            }

            // Wait for the process to complete
            let output = child.wait_with_output().await.map_err(|e| {
                WorkflowError::plugin(format!(
                    "Failed to wait for Node.js process completion: {}",
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
                        "Node.js script execution timed out after {:?}: {:?}",
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
            debug!("Node.js script stderr output: {}", stderr);
        }

        // Check if the process succeeded
        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);

            // Try to parse stdout as JSON error first
            if let Ok(error_json) = serde_json::from_str::<Value>(&stdout) {
                if let Some(error_msg) = error_json.get("error").and_then(|e| e.as_str()) {
                    return Err(WorkflowError::plugin(format!(
                        "Node.js script execution failed (exit code {}): {}",
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
                format!("Node.js script failed with exit code {}", exit_code)
            };

            return Err(WorkflowError::plugin(format!(
                "Node.js script execution failed: {}",
                error_message
            )));
        }

        // Parse the output as JSON
        if stdout.trim().is_empty() {
            return Err(WorkflowError::plugin(
                "Node.js script produced no output".to_string(),
            ));
        }

        let result: Value = serde_json::from_str(&stdout).map_err(|e| {
            WorkflowError::plugin(format!(
                "Failed to parse Node.js script output as JSON: {}. Raw output: {}",
                e, stdout
            ))
        })?;

        // Check if the result indicates an error
        if let Some(success) = result.get("success").and_then(|s| s.as_bool()) {
            if !success {
                if let Some(error_msg) = result.get("error").and_then(|e| e.as_str()) {
                    return Err(WorkflowError::plugin(format!(
                        "Node.js script reported error: {}",
                        error_msg
                    )));
                }
            }
        }

        debug!(
            "Node.js script executed successfully: {:?}",
            full_script_path
        );
        Ok(result)
    }

    /// Check if the environment is initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Get the Node.js executable path
    pub fn node_executable(&self) -> &Path {
        &self.node_executable
    }

    /// Get the npm executable path
    pub fn npm_executable(&self) -> &Path {
        &self.npm_executable
    }

    /// Get the package.json information
    pub fn package_json(&self) -> Option<&PackageJson> {
        self.package_json.as_ref()
    }

    /// Validate that a Node.js script is compatible with the plugin system
    pub async fn validate_script(&self, script_path: &Path) -> Result<()> {
        let working_dir = self.get_working_directory()?;
        let resolved_script_path = if script_path.is_absolute() {
            script_path.to_path_buf()
        } else {
            working_dir.join(script_path)
        };

        if !resolved_script_path.exists() {
            return Err(WorkflowError::plugin(format!(
                "Node.js script not found: {:?}",
                resolved_script_path
            )));
        }

        if !resolved_script_path.is_file() {
            return Err(WorkflowError::plugin(format!(
                "Path is not a file: {:?}",
                resolved_script_path
            )));
        }

        // Check if the script has a .js or .mjs extension
        let extension = resolved_script_path
            .extension()
            .and_then(|ext| ext.to_str());
        if !matches!(extension, Some("js") | Some("mjs") | Some("cjs")) {
            return Err(WorkflowError::plugin(format!(
                "Script must have .js, .mjs, or .cjs extension: {:?}",
                resolved_script_path
            )));
        }

        // Try to check the script syntax
        let output = AsyncCommand::new(&self.node_executable)
            .args([
                "-c",
                &format!(
                    "require('fs').readFileSync('{}', 'utf8')",
                    resolved_script_path.display()
                ),
            ])
            .current_dir(&working_dir)
            .output()
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!("Failed to validate Node.js script: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkflowError::plugin(format!(
                "Node.js script has syntax errors: {}",
                stderr
            )));
        }

        debug!(
            "Node.js script validation passed: {:?}",
            resolved_script_path
        );
        Ok(())
    }

    /// Get information about the Node.js environment
    pub async fn get_environment_info(&self) -> Result<Value> {
        let working_dir = self.get_working_directory()?;

        let output = AsyncCommand::new(&self.node_executable)
            .args(["-e", "console.log(JSON.stringify({version: process.version, platform: process.platform, arch: process.arch, execPath: process.execPath}))"])
            .current_dir(&working_dir)
            .output()
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!("Failed to get Node.js environment info: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkflowError::plugin(format!(
                "Failed to get Node.js environment info: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let info: Value = serde_json::from_str(&stdout).map_err(|e| {
            WorkflowError::plugin(format!("Failed to parse Node.js environment info: {}", e))
        })?;

        Ok(info)
    }

    /// Install a specific npm package
    pub async fn install_package(&self, package_name: &str, version: Option<&str>) -> Result<()> {
        let working_dir = self.get_working_directory()?;

        let package_spec = if let Some(version) = version {
            format!("{}@{}", package_name, version)
        } else {
            package_name.to_string()
        };

        info!("Installing npm package: {}", package_spec);

        let output = AsyncCommand::new(&self.npm_executable)
            .args(["install", &package_spec])
            .current_dir(&working_dir)
            .output()
            .await
            .map_err(|e| WorkflowError::plugin(format!("Failed to install npm package: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(WorkflowError::plugin(format!(
                "npm install {} failed:\nstdout: {}\nstderr: {}",
                package_spec, stdout, stderr
            )));
        }

        info!("Package installed successfully: {}", package_spec);
        Ok(())
    }
}

/// Node.js plugin implementation
pub struct NodeJsPlugin {
    info: PluginInfo,
    status: PluginStatus,
    config: Option<PluginConfig>,
    #[allow(dead_code)]
    runtime_config: NodeJsRuntimeConfig,
    environment: Arc<Mutex<NodeJsEnvironment>>,
    tools: Vec<Tool>,
}

impl NodeJsPlugin {
    /// Create a new Node.js plugin
    pub fn new(info: PluginInfo, runtime_config: NodeJsRuntimeConfig) -> Self {
        let environment = Arc::new(Mutex::new(NodeJsEnvironment::new(runtime_config.clone())));

        Self {
            info,
            status: PluginStatus::Uninitialized,
            config: None,
            runtime_config,
            environment,
            tools: Vec::new(),
        }
    }

    /// Create a Node.js plugin from a package.json file
    pub fn from_package_json(
        info: PluginInfo,
        package_json_path: &Path,
        project_directory: Option<PathBuf>,
    ) -> Result<Self> {
        if !package_json_path.exists() {
            return Err(WorkflowError::plugin(format!(
                "package.json file not found: {:?}",
                package_json_path
            )));
        }

        let runtime_config = NodeJsRuntimeConfig {
            package_json: Some(package_json_path.to_path_buf()),
            project_directory,
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
            NodeJsTool, ResourceRequirements, ToolId, ToolKind, ToolMetadata,
        };

        let environment = self.environment.lock().await;
        let node_path = environment.node_executable().to_path_buf();
        drop(environment);

        let nodejs_tool = NodeJsTool {
            id: ToolId::new(),
            metadata: Arc::new(ToolMetadata {
                info: tool_info.clone(),
                kind: ToolKind::NodeJs,
                input_schema: None,
                output_schema: None,
                examples: Vec::new(),
                resource_requirements: ResourceRequirements::default(),
                version: tool_info.version.clone(),
            }),
            script_path,
            node_path,
            timeout_secs: timeout.map(|d| d.as_secs()).unwrap_or(300),
            middleware_stack: None,
        };

        self.tools.push(Tool::NodeJs(Arc::new(nodejs_tool)));
        Ok(())
    }

    /// Add a tool using NodeJsTool directly
    pub async fn add_basic_tool(
        &mut self,
        tool_info: ToolInfo,
        script_path: PathBuf,
        timeout: Option<Duration>,
    ) -> Result<()> {
        use crate::tools::types::{
            NodeJsTool, ResourceRequirements, ToolId, ToolKind, ToolMetadata,
        };

        let environment = self.environment.lock().await;
        let node_path = environment.node_executable().to_path_buf();
        drop(environment);

        let nodejs_tool = NodeJsTool {
            id: ToolId::new(),
            metadata: Arc::new(ToolMetadata {
                info: tool_info.clone(),
                kind: ToolKind::NodeJs,
                input_schema: None,
                output_schema: None,
                examples: Vec::new(),
                resource_requirements: ResourceRequirements::default(),
                version: tool_info.version.clone(),
            }),
            script_path,
            node_path,
            timeout_secs: timeout.map(|d| d.as_secs()).unwrap_or(300),
            middleware_stack: None,
        };

        self.tools.push(Tool::NodeJs(Arc::new(nodejs_tool)));
        Ok(())
    }

    /// Get the Node.js environment
    pub fn environment(&self) -> Arc<Mutex<NodeJsEnvironment>> {
        self.environment.clone()
    }
}

impl Plugin for NodeJsPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        info!("Initializing Node.js plugin: {}", self.info.name);
        self.status = PluginStatus::Initializing;

        // Store the config for later async initialization
        self.config = Some(config);
        self.status = PluginStatus::Ready;
        info!(
            "Node.js plugin initialized successfully: {}",
            self.info.name
        );
        Ok(())
    }

    fn get_tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Node.js plugin: {}", self.info.name);
        self.status = PluginStatus::ShuttingDown;

        // Clear tools
        self.tools.clear();

        self.status = PluginStatus::Shutdown;
        info!("Node.js plugin shut down successfully: {}", self.info.name);
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
    }

    fn status(&self) -> PluginStatus {
        self.status
    }
}

impl NodeJsPlugin {
    /// Async initialization method that should be called after initialize()
    pub async fn initialize_async(&mut self) -> Result<()> {
        if self.status != PluginStatus::Ready {
            return Err(WorkflowError::plugin(
                "Plugin must be initialized before async initialization".to_string(),
            ));
        }

        // Initialize the Node.js environment
        let mut environment = self.environment.lock().await;
        environment.initialize().await?;

        info!(
            "Node.js plugin async initialization completed: {}",
            self.info.name
        );
        Ok(())
    }
}

/// Builder for Node.js plugins
pub struct NodeJsPluginBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    runtime_config: NodeJsRuntimeConfig,
    tools: Vec<(ToolInfo, PathBuf, Option<Duration>)>,
}

impl NodeJsPluginBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            runtime_config: NodeJsRuntimeConfig::default(),
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

    pub fn node_executable<S: Into<String>>(mut self, executable: S) -> Self {
        self.runtime_config.node_executable = executable.into();
        self
    }

    pub fn npm_executable<S: Into<String>>(mut self, executable: S) -> Self {
        self.runtime_config.npm_executable = executable.into();
        self
    }

    pub fn project_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.project_directory = Some(path.into());
        self
    }

    pub fn package_json<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.package_json = Some(path.into());
        self
    }

    pub fn working_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.working_directory = Some(path.into());
        self
    }

    pub fn node_modules_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.node_modules_path = Some(path.into());
        self
    }

    pub fn add_module_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.runtime_config.module_paths.push(path.into());
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

    pub fn auto_install_dependencies(mut self, auto_install: bool) -> Self {
        self.runtime_config.auto_install_dependencies = auto_install;
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

    pub async fn build(self) -> Result<NodeJsPlugin> {
        let name = self
            .name
            .ok_or_else(|| WorkflowError::ValidationError("Plugin name is required".to_string()))?;
        let version = self.version.ok_or_else(|| {
            WorkflowError::ValidationError("Plugin version is required".to_string())
        })?;
        let description = self
            .description
            .unwrap_or_else(|| format!("Node.js plugin: {}", name));

        let _now = Utc::now();
        let plugin_info = PluginInfo {
            name: name.clone(),
            version,
            plugin_type: PluginType::NodeJs,
            description: Some(description),
            author: None,
            homepage: None,
            metadata: HashMap::new(),
        };

        let mut plugin = NodeJsPlugin::new(plugin_info, self.runtime_config);

        // Add tools
        for (tool_info, script_path, timeout) in self.tools {
            plugin
                .add_basic_tool(tool_info, script_path, timeout)
                .await?;
        }

        Ok(plugin)
    }
}

impl Default for NodeJsPluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    include!("nodejs_tests.rs");
}
