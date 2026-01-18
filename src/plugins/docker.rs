//! Docker plugin implementation for executing Docker-based tools

use crate::core::{ExecutionContext, PluginInfo, PluginType, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus};
use crate::tools::{BasicTool, ToolExecutor, ToolNode};
use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
    WaitContainerOptions,
};
use bollard::image::{BuildImageOptions, CreateImageOptions};
use bollard::models::{HostConfig, Mount, MountTypeEnum, PortBinding};
use bollard::Docker;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Docker runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerRuntimeConfig {
    /// Docker daemon connection URL (defaults to local socket)
    pub docker_url: Option<String>,
    /// Default image to use for containers
    pub default_image: Option<String>,
    /// Default working directory inside containers
    pub default_working_dir: Option<String>,
    /// Default environment variables for containers
    pub default_environment: HashMap<String, String>,
    /// Default volume mounts
    pub default_mounts: Vec<DockerMount>,
    /// Default port mappings
    pub default_ports: HashMap<String, String>,
    /// Container resource limits
    pub resource_limits: DockerResourceLimits,
    /// Network configuration
    pub network_config: DockerNetworkConfig,
    /// Auto-remove containers after execution
    pub auto_remove: bool,
    /// Container execution timeout
    pub execution_timeout: Option<Duration>,
}

impl Default for DockerRuntimeConfig {
    fn default() -> Self {
        Self {
            docker_url: None, // Use default Docker connection
            default_image: None,
            default_working_dir: Some("/workspace".to_string()),
            default_environment: HashMap::new(),
            default_mounts: Vec::new(),
            default_ports: HashMap::new(),
            resource_limits: DockerResourceLimits::default(),
            network_config: DockerNetworkConfig::default(),
            auto_remove: true,
            execution_timeout: Some(Duration::from_secs(300)), // 5 minutes default
        }
    }
}

/// Docker mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerMount {
    /// Source path on host
    pub source: PathBuf,
    /// Target path in container
    pub target: String,
    /// Mount type (bind, volume, tmpfs)
    pub mount_type: DockerMountType,
    /// Read-only mount
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockerMountType {
    Bind,
    Volume,
    Tmpfs,
}

impl From<DockerMountType> for MountTypeEnum {
    fn from(mount_type: DockerMountType) -> Self {
        match mount_type {
            DockerMountType::Bind => MountTypeEnum::BIND,
            DockerMountType::Volume => MountTypeEnum::VOLUME,
            DockerMountType::Tmpfs => MountTypeEnum::TMPFS,
        }
    }
}

/// Docker resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerResourceLimits {
    /// Memory limit in bytes
    pub memory: Option<i64>,
    /// Memory swap limit in bytes
    pub memory_swap: Option<i64>,
    /// CPU limit (number of CPUs * 1e9)
    pub nano_cpus: Option<i64>,
    /// CPU shares (relative weight)
    pub cpu_shares: Option<i64>,
    /// Maximum number of PIDs
    pub pids_limit: Option<i64>,
}

impl Default for DockerResourceLimits {
    fn default() -> Self {
        Self {
            memory: Some(1024 * 1024 * 1024),          // 1GB
            memory_swap: Some(2 * 1024 * 1024 * 1024), // 2GB
            nano_cpus: Some(1_000_000_000),            // 1 CPU
            cpu_shares: None,
            pids_limit: Some(1024),
        }
    }
}

/// Docker network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerNetworkConfig {
    /// Network mode (bridge, host, none, container:<name>)
    pub network_mode: String,
    /// Custom networks to connect to
    pub networks: Vec<String>,
    /// DNS servers
    pub dns: Vec<String>,
    /// DNS search domains
    pub dns_search: Vec<String>,
}

impl Default for DockerNetworkConfig {
    fn default() -> Self {
        Self {
            network_mode: "bridge".to_string(),
            networks: Vec::new(),
            dns: Vec::new(),
            dns_search: Vec::new(),
        }
    }
}

/// Docker container configuration for a specific tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerToolConfig {
    /// Docker image to use
    pub image: String,
    /// Command to run in the container
    pub command: Option<Vec<String>>,
    /// Entry point override
    pub entrypoint: Option<Vec<String>>,
    /// Working directory in container
    pub working_dir: Option<String>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Volume mounts
    pub mounts: Vec<DockerMount>,
    /// Port mappings (host_port:container_port)
    pub ports: HashMap<String, String>,
    /// Resource limits override
    pub resource_limits: Option<DockerResourceLimits>,
    /// Network configuration override
    pub network_config: Option<DockerNetworkConfig>,
    /// Container labels
    pub labels: HashMap<String, String>,
    /// User to run as (user:group)
    pub user: Option<String>,
    /// Privileged mode
    pub privileged: bool,
}

impl DockerToolConfig {
    pub fn new(image: String) -> Self {
        Self {
            image,
            command: None,
            entrypoint: None,
            working_dir: None,
            environment: HashMap::new(),
            mounts: Vec::new(),
            ports: HashMap::new(),
            resource_limits: None,
            network_config: None,
            labels: HashMap::new(),
            user: None,
            privileged: false,
        }
    }
}

/// Docker environment manager
#[derive(Debug)]
pub struct DockerEnvironment {
    config: DockerRuntimeConfig,
    docker: Docker,
    is_initialized: bool,
}

impl DockerEnvironment {
    /// Create a new Docker environment
    pub fn new(config: DockerRuntimeConfig) -> Result<Self> {
        let docker = if let Some(url) = &config.docker_url {
            Docker::connect_with_http(url, 120, bollard::API_DEFAULT_VERSION).map_err(|e| {
                WorkflowError::plugin(format!(
                    "Failed to connect to Docker daemon at {}: {}",
                    url, e
                ))
            })?
        } else {
            Docker::connect_with_local_defaults().map_err(|e| {
                WorkflowError::plugin(format!("Failed to connect to Docker daemon: {}", e))
            })?
        };

        Ok(Self {
            config,
            docker,
            is_initialized: false,
        })
    }

    /// Initialize the Docker environment
    pub async fn initialize(&mut self) -> Result<()> {
        if self.is_initialized {
            return Ok(());
        }

        info!("Initializing Docker environment");

        // Verify Docker daemon connection
        self.verify_docker_connection().await?;

        self.is_initialized = true;
        info!("Docker environment initialized successfully");
        Ok(())
    }

    /// Verify Docker daemon connection
    async fn verify_docker_connection(&self) -> Result<()> {
        debug!("Verifying Docker daemon connection");

        let version = self.docker.version().await.map_err(|e| {
            WorkflowError::plugin(format!(
                "Failed to connect to Docker daemon: {}. Please ensure Docker is running and accessible.",
                e
            ))
        })?;

        info!(
            "Docker daemon version: {}",
            version.version.unwrap_or_else(|| "unknown".to_string())
        );
        info!(
            "Docker API version: {}",
            version.api_version.unwrap_or_else(|| "unknown".to_string())
        );

        Ok(())
    }

    /// Execute a Docker container with given configuration and parameters
    pub async fn execute_container(
        &self,
        tool_config: &DockerToolConfig,
        params: Value,
        context: ExecutionContext,
        timeout_duration: Option<Duration>,
    ) -> Result<Value> {
        if !self.is_initialized {
            return Err(WorkflowError::plugin(
                "Docker environment not initialized".to_string(),
            ));
        }

        let container_name = format!(
            "workflow-tool-{}-{}",
            context.execution_id,
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        debug!("Creating Docker container: {}", container_name);

        // Create container configuration
        let container_config =
            self.create_container_config(tool_config, &params, &context, &container_name)?;

        // Create the container
        let container_response = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: container_name.clone(),
                    platform: None,
                }),
                container_config,
            )
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!("Failed to create Docker container: {}", e))
            })?;

        let container_id = container_response.id;
        debug!("Created Docker container with ID: {}", container_id);

        // Start the container
        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!("Failed to start Docker container: {}", e))
            })?;

        debug!("Started Docker container: {}", container_id);

        // Wait for container completion with timeout
        let execution_result =
            if let Some(timeout_duration) = timeout_duration.or(self.config.execution_timeout) {
                tokio::time::timeout(timeout_duration, self.wait_for_container(&container_id))
                    .await
                    .map_err(|_| {
                        WorkflowError::plugin(format!(
                            "Docker container execution timed out after {:?}: {}",
                            timeout_duration, container_id
                        ))
                    })?
            } else {
                self.wait_for_container(&container_id).await
            };

        // Get container logs
        let logs = self.get_container_logs(&container_id).await?;

        // Clean up container if auto-remove is enabled
        if self.config.auto_remove {
            if let Err(e) = self.remove_container(&container_id).await {
                warn!("Failed to remove container {}: {}", container_id, e);
            }
        }

        // Process execution result
        match execution_result {
            Ok(exit_code) => {
                if exit_code == 0 {
                    // Parse output from logs
                    self.parse_container_output(&logs)
                } else {
                    Err(WorkflowError::plugin(format!(
                        "Docker container exited with code {}: {}",
                        exit_code, logs.stderr
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Create container configuration
    fn create_container_config(
        &self,
        tool_config: &DockerToolConfig,
        params: &Value,
        context: &ExecutionContext,
        container_name: &str,
    ) -> Result<Config<String>> {
        // Prepare environment variables
        let mut env_vars = Vec::new();

        // Add default environment variables
        for (key, value) in &self.config.default_environment {
            env_vars.push(format!("{}={}", key, value));
        }

        // Add tool-specific environment variables
        for (key, value) in &tool_config.environment {
            env_vars.push(format!("{}={}", key, value));
        }

        // Add context information as environment variables
        env_vars.push(format!("WORKFLOW_EXECUTION_ID={}", context.execution_id));
        env_vars.push(format!(
            "WORKFLOW_ID={}",
            context
                .workflow_id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_default()
        ));
        env_vars.push(format!(
            "WORKFLOW_USER_ID={}",
            context.user_id.as_ref().unwrap_or(&String::new())
        ));
        env_vars.push(format!(
            "WORKFLOW_SESSION_ID={}",
            context.session_id.as_ref().unwrap_or(&String::new())
        ));

        // Serialize parameters as JSON and pass as environment variable
        let params_json = serde_json::to_string(params)
            .map_err(|e| WorkflowError::plugin(format!("Failed to serialize parameters: {}", e)))?;
        env_vars.push(format!("WORKFLOW_PARAMS={}", params_json));

        // Prepare mounts
        let mut mounts = Vec::new();

        // Add default mounts
        for mount in &self.config.default_mounts {
            mounts.push(Mount {
                target: Some(mount.target.clone()),
                source: Some(mount.source.to_string_lossy().to_string()),
                typ: Some(mount.mount_type.clone().into()),
                read_only: Some(mount.read_only),
                consistency: None,
                bind_options: None,
                volume_options: None,
                tmpfs_options: None,
            });
        }

        // Add tool-specific mounts
        for mount in &tool_config.mounts {
            mounts.push(Mount {
                target: Some(mount.target.clone()),
                source: Some(mount.source.to_string_lossy().to_string()),
                typ: Some(mount.mount_type.clone().into()),
                read_only: Some(mount.read_only),
                consistency: None,
                bind_options: None,
                volume_options: None,
                tmpfs_options: None,
            });
        }

        // Prepare port bindings
        let mut port_bindings = HashMap::new();

        // Add default port mappings
        for (container_port, host_port) in &self.config.default_ports {
            port_bindings.insert(
                container_port.clone(),
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port.clone()),
                }]),
            );
        }

        // Add tool-specific port mappings
        for (container_port, host_port) in &tool_config.ports {
            port_bindings.insert(
                container_port.clone(),
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port.clone()),
                }]),
            );
        }

        // Get resource limits (tool-specific overrides default)
        let resource_limits = tool_config
            .resource_limits
            .as_ref()
            .unwrap_or(&self.config.resource_limits);

        // Create host configuration
        let host_config = HostConfig {
            mounts: Some(mounts),
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            memory: resource_limits.memory,
            memory_swap: resource_limits.memory_swap,
            nano_cpus: resource_limits.nano_cpus,
            cpu_shares: resource_limits.cpu_shares,
            pids_limit: resource_limits.pids_limit,
            network_mode: tool_config
                .network_config
                .as_ref()
                .map(|nc| nc.network_mode.clone())
                .or_else(|| Some(self.config.network_config.network_mode.clone())),
            dns: tool_config
                .network_config
                .as_ref()
                .and_then(|nc| {
                    if nc.dns.is_empty() {
                        None
                    } else {
                        Some(nc.dns.clone())
                    }
                })
                .or_else(|| {
                    if self.config.network_config.dns.is_empty() {
                        None
                    } else {
                        Some(self.config.network_config.dns.clone())
                    }
                }),
            dns_search: tool_config
                .network_config
                .as_ref()
                .and_then(|nc| {
                    if nc.dns_search.is_empty() {
                        None
                    } else {
                        Some(nc.dns_search.clone())
                    }
                })
                .or_else(|| {
                    if self.config.network_config.dns_search.is_empty() {
                        None
                    } else {
                        Some(self.config.network_config.dns_search.clone())
                    }
                }),
            privileged: Some(tool_config.privileged),
            auto_remove: Some(self.config.auto_remove),
            ..Default::default()
        };

        // Prepare labels
        let mut labels = HashMap::new();
        labels.insert("workflow-toolkit.managed".to_string(), "true".to_string());
        labels.insert(
            "workflow-toolkit.execution-id".to_string(),
            context.execution_id.clone(),
        );
        labels.insert(
            "workflow-toolkit.container-name".to_string(),
            container_name.to_string(),
        );

        // Add tool-specific labels
        for (key, value) in &tool_config.labels {
            labels.insert(key.clone(), value.clone());
        }

        // Create container configuration
        let config = Config {
            image: Some(tool_config.image.clone()),
            cmd: tool_config.command.clone(),
            entrypoint: tool_config.entrypoint.clone(),
            working_dir: tool_config
                .working_dir
                .clone()
                .or_else(|| self.config.default_working_dir.clone()),
            env: Some(env_vars),
            user: tool_config.user.clone(),
            labels: Some(labels),
            host_config: Some(host_config),
            ..Default::default()
        };

        Ok(config)
    }

    /// Wait for container to complete and return exit code
    async fn wait_for_container(&self, container_id: &str) -> Result<i64> {
        let mut stream = self.docker.wait_container(
            container_id,
            Some(WaitContainerOptions {
                condition: "not-running",
            }),
        );

        while let Some(result) = stream.next().await {
            match result {
                Ok(wait_response) => {
                    let status_code = wait_response.status_code;
                    debug!(
                        "Container {} exited with code: {}",
                        container_id, status_code
                    );
                    return Ok(status_code);
                }
                Err(e) => {
                    return Err(WorkflowError::plugin(format!(
                        "Error waiting for container {}: {}",
                        container_id, e
                    )));
                }
            }
        }

        Err(WorkflowError::plugin(format!(
            "Container {} wait stream ended without exit code",
            container_id
        )))
    }

    /// Get container logs
    async fn get_container_logs(&self, container_id: &str) -> Result<ContainerLogs> {
        use bollard::container::LogsOptions;

        let mut stdout_stream = self.docker.logs(
            container_id,
            Some(LogsOptions::<String> {
                stdout: true,
                stderr: false,
                follow: false,
                ..Default::default()
            }),
        );

        let mut stderr_stream = self.docker.logs(
            container_id,
            Some(LogsOptions::<String> {
                stdout: false,
                stderr: true,
                follow: false,
                ..Default::default()
            }),
        );

        let mut stdout = String::new();
        let mut stderr = String::new();

        // Collect stdout
        while let Some(log_result) = stdout_stream.next().await {
            match log_result {
                Ok(log_output) => {
                    stdout.push_str(&log_output.to_string());
                }
                Err(e) => {
                    warn!(
                        "Error reading stdout from container {}: {}",
                        container_id, e
                    );
                    break;
                }
            }
        }

        // Collect stderr
        while let Some(log_result) = stderr_stream.next().await {
            match log_result {
                Ok(log_output) => {
                    stderr.push_str(&log_output.to_string());
                }
                Err(e) => {
                    warn!(
                        "Error reading stderr from container {}: {}",
                        container_id, e
                    );
                    break;
                }
            }
        }

        Ok(ContainerLogs { stdout, stderr })
    }

    /// Parse container output as JSON result
    fn parse_container_output(&self, logs: &ContainerLogs) -> Result<Value> {
        // Try to parse stdout as JSON
        if !logs.stdout.trim().is_empty() {
            match serde_json::from_str::<Value>(&logs.stdout) {
                Ok(result) => {
                    // Check if the result indicates an error
                    if let Some(success) = result.get("success").and_then(|s| s.as_bool()) {
                        if !success {
                            if let Some(error_msg) = result.get("error").and_then(|e| e.as_str()) {
                                return Err(WorkflowError::plugin(format!(
                                    "Docker container reported error: {}",
                                    error_msg
                                )));
                            }
                        }
                    }
                    return Ok(result);
                }
                Err(e) => {
                    debug!("Failed to parse stdout as JSON: {}", e);
                    // If JSON parsing fails, return the raw output
                    return Ok(serde_json::json!({
                        "success": true,
                        "result": logs.stdout,
                        "raw_output": true
                    }));
                }
            }
        }

        // If no stdout or parsing failed, check stderr
        if !logs.stderr.trim().is_empty() {
            return Err(WorkflowError::plugin(format!(
                "Docker container produced error output: {}",
                logs.stderr
            )));
        }

        // No output at all
        Err(WorkflowError::plugin(
            "Docker container produced no output".to_string(),
        ))
    }

    /// Remove a container
    async fn remove_container(&self, container_id: &str) -> Result<()> {
        self.docker
            .remove_container(
                container_id,
                Some(RemoveContainerOptions {
                    force: true,
                    v: true,
                    ..Default::default()
                }),
            )
            .await
            .map_err(|e| {
                WorkflowError::plugin(format!(
                    "Failed to remove container {}: {}",
                    container_id, e
                ))
            })?;

        debug!("Removed Docker container: {}", container_id);
        Ok(())
    }

    /// Build a Docker image from a Dockerfile
    pub async fn build_image(&self, dockerfile_path: &Path, image_tag: &str) -> Result<String> {
        if !dockerfile_path.exists() {
            return Err(WorkflowError::plugin(format!(
                "Dockerfile not found: {:?}",
                dockerfile_path
            )));
        }

        let dockerfile_dir = dockerfile_path
            .parent()
            .ok_or_else(|| WorkflowError::plugin("Invalid Dockerfile path".to_string()))?;

        info!("Building Docker image from: {:?}", dockerfile_path);

        // Create a tar archive of the build context
        let build_context = self.create_build_context(dockerfile_dir).await?;

        let build_options = BuildImageOptions {
            dockerfile: "Dockerfile",
            t: image_tag,
            rm: true,
            forcerm: true,
            pull: true,
            ..Default::default()
        };

        let mut stream = self
            .docker
            .build_image(build_options, None, Some(build_context.into()));

        while let Some(result) = stream.next().await {
            match result {
                Ok(build_info) => {
                    if let Some(stream) = build_info.stream {
                        debug!("Docker build: {}", stream.trim());
                    }
                    if let Some(error) = build_info.error {
                        return Err(WorkflowError::plugin(format!(
                            "Docker build failed: {}",
                            error
                        )));
                    }
                }
                Err(e) => {
                    return Err(WorkflowError::plugin(format!("Docker build error: {}", e)));
                }
            }
        }

        info!("Successfully built Docker image: {}", image_tag);
        Ok(image_tag.to_string())
    }

    /// Create build context tar archive
    async fn create_build_context(&self, context_dir: &Path) -> Result<Vec<u8>> {
        let mut tar_data = Vec::new();
        {
            let mut tar = tar::Builder::new(&mut tar_data);
            tar.append_dir_all(".", context_dir).map_err(|e| {
                WorkflowError::plugin(format!("Failed to create build context: {}", e))
            })?;
            tar.finish().map_err(|e| {
                WorkflowError::plugin(format!("Failed to finalize build context: {}", e))
            })?;
        }
        Ok(tar_data)
    }

    /// Pull a Docker image
    pub async fn pull_image(&self, image: &str) -> Result<()> {
        info!("Pulling Docker image: {}", image);

        let mut stream = self.docker.create_image(
            Some(CreateImageOptions {
                from_image: image,
                ..Default::default()
            }),
            None,
            None,
        );

        while let Some(result) = stream.next().await {
            match result {
                Ok(create_info) => {
                    if let Some(status) = create_info.status {
                        debug!("Docker pull: {}", status);
                    }
                    if let Some(error) = create_info.error {
                        return Err(WorkflowError::plugin(format!(
                            "Docker pull failed: {}",
                            error
                        )));
                    }
                }
                Err(e) => {
                    return Err(WorkflowError::plugin(format!("Docker pull error: {}", e)));
                }
            }
        }

        info!("Successfully pulled Docker image: {}", image);
        Ok(())
    }

    /// Check if the environment is initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Get Docker client reference
    pub fn docker(&self) -> &Docker {
        &self.docker
    }

    /// Get environment information
    pub async fn get_environment_info(&self) -> Result<Value> {
        let version =
            self.docker.version().await.map_err(|e| {
                WorkflowError::plugin(format!("Failed to get Docker version: {}", e))
            })?;

        let info = self
            .docker
            .info()
            .await
            .map_err(|e| WorkflowError::plugin(format!("Failed to get Docker info: {}", e)))?;

        Ok(serde_json::json!({
            "version": version,
            "info": {
                "containers": info.containers,
                "images": info.images,
                "server_version": info.server_version,
                "operating_system": info.operating_system,
                "architecture": info.architecture,
                "memory_total": info.mem_total,
                "cpus": info.ncpu,
            }
        }))
    }
}

/// Container logs structure
#[derive(Debug, Clone)]
pub struct ContainerLogs {
    pub stdout: String,
    pub stderr: String,
}

/// Docker tool node implementation
pub struct DockerToolNode {
    info: ToolInfo,
    plugin_info: PluginInfo,
    tool_config: DockerToolConfig,
    environment: Arc<Mutex<DockerEnvironment>>,
    timeout: Option<Duration>,
}

impl DockerToolNode {
    /// Create a new Docker tool node
    pub fn new(
        info: ToolInfo,
        plugin_info: PluginInfo,
        tool_config: DockerToolConfig,
        environment: Arc<Mutex<DockerEnvironment>>,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            info,
            plugin_info,
            tool_config,
            environment,
            timeout,
        }
    }
}

#[async_trait]
impl ToolNode for DockerToolNode {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn version(&self) -> &str {
        &self.info.version
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Basic validation - ensure params is an object
        if !params.is_object() && !params.is_null() {
            return Err(WorkflowError::ValidationError(
                "Parameters must be a JSON object or null".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        self.validate_parameters(&params)?;

        let environment = self.environment.lock().await;

        if !environment.is_initialized() {
            return Err(WorkflowError::plugin(
                "Docker environment not initialized".to_string(),
            ));
        }

        environment
            .execute_container(&self.tool_config, params, context, self.timeout)
            .await
    }

    fn get_info(&self) -> ToolInfo {
        self.info.clone()
    }

    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        Some(&self.plugin_info)
    }
}

/// Docker tool executor for use with BasicTool
pub struct DockerToolExecutor {
    tool_config: DockerToolConfig,
    environment: Arc<Mutex<DockerEnvironment>>,
    timeout: Option<Duration>,
}

impl DockerToolExecutor {
    pub fn new(
        tool_config: DockerToolConfig,
        environment: Arc<Mutex<DockerEnvironment>>,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            tool_config,
            environment,
            timeout,
        }
    }
}

#[async_trait]
impl ToolExecutor for DockerToolExecutor {
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        let environment = self.environment.lock().await;

        if !environment.is_initialized() {
            return Err(WorkflowError::plugin(
                "Docker environment not initialized".to_string(),
            ));
        }

        environment
            .execute_container(&self.tool_config, params, context, self.timeout)
            .await
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Basic validation - ensure params is an object
        if !params.is_object() && !params.is_null() {
            return Err(WorkflowError::ValidationError(
                "Parameters must be a JSON object or null".to_string(),
            ));
        }
        Ok(())
    }
}

/// Docker plugin implementation
pub struct DockerPlugin {
    info: PluginInfo,
    status: PluginStatus,
    config: Option<PluginConfig>,
    #[allow(dead_code)]
    runtime_config: DockerRuntimeConfig,
    environment: Arc<Mutex<DockerEnvironment>>,
    tools: Vec<Arc<dyn ToolNode>>,
}

impl DockerPlugin {
    /// Create a new Docker plugin
    pub fn new(info: PluginInfo, runtime_config: DockerRuntimeConfig) -> Result<Self> {
        let environment = Arc::new(Mutex::new(DockerEnvironment::new(runtime_config.clone())?));

        Ok(Self {
            info,
            status: PluginStatus::Uninitialized,
            config: None,
            runtime_config,
            environment,
            tools: Vec::new(),
        })
    }

    /// Create a Docker plugin from a Dockerfile
    pub async fn from_dockerfile(
        info: PluginInfo,
        dockerfile_path: &Path,
        image_tag: Option<String>,
    ) -> Result<Self> {
        if !dockerfile_path.exists() {
            return Err(WorkflowError::plugin(format!(
                "Dockerfile not found: {:?}",
                dockerfile_path
            )));
        }

        let image_tag =
            image_tag.unwrap_or_else(|| format!("workflow-toolkit/{}", info.name.to_lowercase()));

        let runtime_config = DockerRuntimeConfig {
            default_image: Some(image_tag),
            ..Default::default()
        };

        Self::new(info, runtime_config)
    }

    /// Add a tool to the plugin
    pub async fn add_tool(
        &mut self,
        tool_info: ToolInfo,
        tool_config: DockerToolConfig,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let tool = Arc::new(DockerToolNode::new(
            tool_info,
            self.info.clone(),
            tool_config,
            self.environment.clone(),
            timeout,
        ));

        self.tools.push(tool);
        Ok(())
    }

    /// Add a tool using BasicTool with DockerToolExecutor
    pub async fn add_basic_tool(
        &mut self,
        tool_info: ToolInfo,
        tool_config: DockerToolConfig,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let executor = Arc::new(DockerToolExecutor::new(
            tool_config,
            self.environment.clone(),
            timeout,
        ));

        let tool = Arc::new(BasicTool::new(
            tool_info,
            executor,
            Some(self.info.clone()),
        )?);

        self.tools.push(tool);
        Ok(())
    }

    /// Get the Docker environment
    pub fn environment(&self) -> Arc<Mutex<DockerEnvironment>> {
        self.environment.clone()
    }

    /// Build Docker image from Dockerfile
    pub async fn build_image(&self, dockerfile_path: &Path, image_tag: &str) -> Result<String> {
        let environment = self.environment.lock().await;
        environment.build_image(dockerfile_path, image_tag).await
    }

    /// Pull Docker image
    pub async fn pull_image(&self, image: &str) -> Result<()> {
        let environment = self.environment.lock().await;
        environment.pull_image(image).await
    }
}

impl Plugin for DockerPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        info!("Initializing Docker plugin: {}", self.info.name);
        self.status = PluginStatus::Initializing;

        // Store the config for later async initialization
        self.config = Some(config);
        self.status = PluginStatus::Ready;
        info!("Docker plugin initialized successfully: {}", self.info.name);
        Ok(())
    }

    fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        self.tools.clone()
    }

    fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Docker plugin: {}", self.info.name);
        self.status = PluginStatus::ShuttingDown;

        // Clear tools
        self.tools.clear();

        self.status = PluginStatus::Shutdown;
        info!("Docker plugin shut down successfully: {}", self.info.name);
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
    }

    fn status(&self) -> PluginStatus {
        self.status
    }
}

impl DockerPlugin {
    /// Async initialization method that should be called after initialize()
    pub async fn initialize_async(&mut self) -> Result<()> {
        if self.status != PluginStatus::Ready {
            return Err(WorkflowError::plugin(
                "Plugin must be initialized before async initialization".to_string(),
            ));
        }

        // Initialize the Docker environment
        let mut environment = self.environment.lock().await;
        environment.initialize().await?;

        info!(
            "Docker plugin async initialization completed: {}",
            self.info.name
        );
        Ok(())
    }
}

/// Builder for Docker plugins
pub struct DockerPluginBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    runtime_config: DockerRuntimeConfig,
    tools: Vec<(ToolInfo, DockerToolConfig, Option<Duration>)>,
}

impl DockerPluginBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            runtime_config: DockerRuntimeConfig::default(),
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

    pub fn docker_url<S: Into<String>>(mut self, url: S) -> Self {
        self.runtime_config.docker_url = Some(url.into());
        self
    }

    pub fn default_image<S: Into<String>>(mut self, image: S) -> Self {
        self.runtime_config.default_image = Some(image.into());
        self
    }

    pub fn default_working_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.runtime_config.default_working_dir = Some(dir.into());
        self
    }

    pub fn add_default_environment_variable<K: Into<String>, V: Into<String>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.runtime_config
            .default_environment
            .insert(key.into(), value.into());
        self
    }

    pub fn add_default_mount(
        mut self,
        source: PathBuf,
        target: String,
        mount_type: DockerMountType,
        read_only: bool,
    ) -> Self {
        self.runtime_config.default_mounts.push(DockerMount {
            source,
            target,
            mount_type,
            read_only,
        });
        self
    }

    pub fn add_default_port_mapping<S1: Into<String>, S2: Into<String>>(
        mut self,
        container_port: S1,
        host_port: S2,
    ) -> Self {
        self.runtime_config
            .default_ports
            .insert(container_port.into(), host_port.into());
        self
    }

    pub fn resource_limits(mut self, limits: DockerResourceLimits) -> Self {
        self.runtime_config.resource_limits = limits;
        self
    }

    pub fn network_config(mut self, config: DockerNetworkConfig) -> Self {
        self.runtime_config.network_config = config;
        self
    }

    pub fn auto_remove(mut self, auto_remove: bool) -> Self {
        self.runtime_config.auto_remove = auto_remove;
        self
    }

    pub fn execution_timeout(mut self, timeout: Duration) -> Self {
        self.runtime_config.execution_timeout = Some(timeout);
        self
    }

    pub fn add_tool(
        mut self,
        tool_info: ToolInfo,
        tool_config: DockerToolConfig,
        timeout: Option<Duration>,
    ) -> Self {
        self.tools.push((tool_info, tool_config, timeout));
        self
    }

    pub async fn build(self) -> Result<DockerPlugin> {
        let name = self
            .name
            .ok_or_else(|| WorkflowError::ValidationError("Plugin name is required".to_string()))?;
        let version = self.version.ok_or_else(|| {
            WorkflowError::ValidationError("Plugin version is required".to_string())
        })?;
        let description = self
            .description
            .unwrap_or_else(|| format!("Docker plugin: {}", name));

        let plugin_info = PluginInfo {
            name: name.clone(),
            version,
            plugin_type: PluginType::Docker,
            description: Some(description),
            author: None,
            homepage: None,
            metadata: HashMap::new(),
        };

        let mut plugin = DockerPlugin::new(plugin_info, self.runtime_config)?;

        // Add tools
        for (tool_info, tool_config, timeout) in self.tools {
            plugin
                .add_basic_tool(tool_info, tool_config, timeout)
                .await?;
        }

        Ok(plugin)
    }
}

impl Default for DockerPluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}
