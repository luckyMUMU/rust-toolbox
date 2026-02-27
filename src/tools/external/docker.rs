//! Docker 工具执行器
//!
//! 提供 Docker 容器的执行能力

use super::{ExecutionResult, ExternalExecutor, ExternalExecutorConfig};
use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use serde_json::Value;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tracing::{debug, error, info};

/// Docker 执行器配置
#[derive(Debug, Clone)]
pub struct DockerExecutorConfig {
    /// 基础配置
    pub base: ExternalExecutorConfig,
    /// 镜像名称
    pub image: String,
    /// 容器名称前缀
    pub container_prefix: String,
    /// 是否自动删除容器
    pub auto_remove: bool,
    /// 挂载点
    pub mounts: Vec<DockerMount>,
    /// 环境变量
    pub env_vars: Vec<String>,
    /// 网络模式
    pub network: Option<String>,
    /// 内存限制
    pub memory_limit: Option<String>,
    /// CPU 限制
    pub cpu_limit: Option<String>,
}

impl Default for DockerExecutorConfig {
    fn default() -> Self {
        Self {
            base: ExternalExecutorConfig::default(),
            image: "alpine:latest".to_string(),
            container_prefix: "workflow-toolkit-".to_string(),
            auto_remove: true,
            mounts: Vec::new(),
            env_vars: Vec::new(),
            network: None,
            memory_limit: None,
            cpu_limit: None,
        }
    }
}

/// Docker 挂载配置
#[derive(Debug, Clone)]
pub struct DockerMount {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

impl DockerMount {
    pub fn new(host_path: &str, container_path: &str, read_only: bool) -> Self {
        Self {
            host_path: host_path.to_string(),
            container_path: container_path.to_string(),
            read_only,
        }
    }

    fn to_docker_arg(&self) -> String {
        let ro = if self.read_only { ":ro" } else { "" };
        format!("-v{}:{}{}", self.host_path, self.container_path, ro)
    }
}

/// Docker 执行器
pub struct DockerExecutor {
    config: ExternalExecutorConfig,
    docker_config: DockerExecutorConfig,
}

impl DockerExecutor {
    /// 创建新的 Docker 执行器
    pub fn new(config: ExternalExecutorConfig) -> Self {
        Self {
            config: config.clone(),
            docker_config: DockerExecutorConfig {
                base: config,
                ..Default::default()
            },
        }
    }

    /// 使用 Docker 配置创建执行器
    pub fn with_docker_config(
        config: ExternalExecutorConfig,
        docker_config: DockerExecutorConfig,
    ) -> Self {
        Self {
            config,
            docker_config,
        }
    }

    /// 构建执行命令
    fn build_command(&self, command: &str, args: &[String]) -> Command {
        let mut cmd = Command::new("docker");

        cmd.arg("run");

        if self.docker_config.auto_remove {
            cmd.arg("--rm");
        }

        let container_name = format!(
            "{}{}",
            self.docker_config.container_prefix,
            uuid::Uuid::new_v4()
        );
        cmd.arg("--name").arg(&container_name);

        for mount in &self.docker_config.mounts {
            cmd.arg(mount.to_docker_arg());
        }

        for env in &self.docker_config.env_vars {
            cmd.arg("-e").arg(env);
        }

        if let Some(ref network) = self.docker_config.network {
            cmd.arg("--network").arg(network);
        }

        if let Some(ref memory) = self.docker_config.memory_limit {
            cmd.arg("--memory").arg(memory);
        }

        if let Some(ref cpu) = self.docker_config.cpu_limit {
            cmd.arg("--cpus").arg(cpu);
        }

        cmd.arg(&self.docker_config.image);
        cmd.arg(command);

        for arg in args {
            cmd.arg(arg);
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        cmd
    }
}

#[async_trait::async_trait]
impl ExternalExecutor for DockerExecutor {
    async fn execute(
        &self,
        command: &str,
        args: &[String],
        input: Option<Value>,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        let start = Instant::now();

        info!(
            workflow_id = ?ctx.workflow_id,
            image = %self.docker_config.image,
            command = %command,
            "执行 Docker 容器"
        );

        let mut cmd = self.build_command(command, args);

        let mut child = cmd.spawn().map_err(|e| {
            error!("启动 Docker 容器失败: {}", e);
            WorkflowError::execution(format!("启动 Docker 容器失败: {}", e))
        })?;

        if let Some(input_value) = input {
            let input_str = serde_json::to_string(&input_value)
                .map_err(|e| WorkflowError::execution(format!("序列化输入失败: {}", e)))?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(input_str.as_bytes())
                    .await
                    .map_err(|e| WorkflowError::execution(format!("写入输入失败: {}", e)))?;
            }
        }

        let timeout_duration = self.config.timeout;
        let result = tokio::time::timeout(timeout_duration, async {
            let mut stdout = String::new();
            let mut stderr = String::new();

            if let Some(mut stdout_handle) = child.stdout.take() {
                stdout_handle
                    .read_to_string(&mut stdout)
                    .await
                    .map_err(|e| WorkflowError::execution(format!("读取标准输出失败: {}", e)))?;
            }

            if let Some(mut stderr_handle) = child.stderr.take() {
                stderr_handle
                    .read_to_string(&mut stderr)
                    .await
                    .map_err(|e| WorkflowError::execution(format!("读取标准错误失败: {}", e)))?;
            }

            let status = child
                .wait()
                .await
                .map_err(|e| WorkflowError::execution(format!("等待容器结束失败: {}", e)))?;

            Ok::<_, WorkflowError>((stdout, stderr, status.code().unwrap_or(-1)))
        })
        .await;

        let duration = start.elapsed();

        match result {
            Ok(Ok((stdout, stderr, exit_code))) => {
                let success = exit_code == 0;

                debug!(
                    image = %self.docker_config.image,
                    exit_code = exit_code,
                    duration_ms = duration.as_millis(),
                    "Docker 容器执行完成"
                );

                Ok(ExecutionResult {
                    stdout,
                    stderr,
                    exit_code,
                    duration,
                    success,
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                let _ = child.kill().await;
                Err(WorkflowError::execution(format!(
                    "Docker 容器执行超时（{}秒）",
                    timeout_duration.as_secs()
                )))
            }
        }
    }

    fn name(&self) -> &str {
        "docker"
    }

    async fn is_available(&self) -> bool {
        let output = Command::new("docker").arg("--version").output().await;

        output.is_ok()
    }

    async fn version(&self) -> Option<String> {
        let output = Command::new("docker")
            .arg("--version")
            .output()
            .await
            .ok()?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        Some(version_str.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_executor_creation() {
        let executor = DockerExecutor::new(ExternalExecutorConfig::default());
        assert_eq!(executor.name(), "docker");
    }

    #[test]
    fn test_docker_mount() {
        let mount = DockerMount::new("/host/path", "/container/path", true);
        assert_eq!(mount.to_docker_arg(), "-v/host/path:/container/path:ro");
    }

    #[test]
    fn test_docker_executor_config_default() {
        let config = DockerExecutorConfig::default();
        assert_eq!(config.image, "alpine:latest");
        assert!(config.auto_remove);
    }
}
