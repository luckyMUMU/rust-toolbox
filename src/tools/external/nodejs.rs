//! Node.js 工具执行器
//!
//! 提供 Node.js 脚本的执行能力

use super::{ExecutionResult, ExternalExecutor, ExternalExecutorConfig};
use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use serde_json::Value;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tracing::{debug, error, info};

/// Node.js 执行器
pub struct NodeJsExecutor {
    config: ExternalExecutorConfig,
    node_path: String,
}

impl NodeJsExecutor {
    /// 创建新的 Node.js 执行器
    pub fn new(config: ExternalExecutorConfig) -> Self {
        Self {
            config,
            node_path: "node".to_string(),
        }
    }

    /// 使用指定 Node.js 路径创建执行器
    pub fn with_node_path(config: ExternalExecutorConfig, node_path: String) -> Self {
        Self {
            config,
            node_path,
        }
    }

    /// 构建执行命令
    fn build_command(&self, script: &str, args: &[String]) -> Command {
        let mut cmd = Command::new(&self.node_path);
        
        cmd.arg(script);
        for arg in args {
            cmd.arg(arg);
        }
        
        if let Some(ref wd) = self.config.working_directory {
            cmd.current_dir(wd);
        }
        
        for (key, value) in &self.config.environment {
            cmd.env(key, value);
        }
        
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        cmd
    }
}

#[async_trait::async_trait]
impl ExternalExecutor for NodeJsExecutor {
    async fn execute(
        &self,
        script: &str,
        args: &[String],
        input: Option<Value>,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        let start = Instant::now();
        
        info!(
            workflow_id = ?ctx.workflow_id(),
            script = %script,
            "执行 Node.js 脚本"
        );
        
        let mut cmd = self.build_command(script, args);
        
        let mut child = cmd.spawn().map_err(|e| {
            error!("启动 Node.js 进程失败: {}", e);
            WorkflowError::execution(format!("启动 Node.js 进程失败: {}", e))
        })?;
        
        if let Some(input_value) = input {
            let input_str = serde_json::to_string(&input_value)
                .map_err(|e| WorkflowError::execution(format!("序列化输入失败: {}", e)))?;
            
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input_str.as_bytes()).await.map_err(|e| {
                    WorkflowError::execution(format!("写入输入失败: {}", e))
                })?;
            }
        }
        
        let timeout_duration = self.config.timeout;
        let result = tokio::time::timeout(
            timeout_duration,
            async {
                let mut stdout = String::new();
                let mut stderr = String::new();
                
                if let Some(mut stdout_handle) = child.stdout.take() {
                    stdout_handle.read_to_string(&mut stdout).await.map_err(|e| {
                        WorkflowError::execution(format!("读取标准输出失败: {}", e))
                    })?;
                }
                
                if let Some(mut stderr_handle) = child.stderr.take() {
                    stderr_handle.read_to_string(&mut stderr).await.map_err(|e| {
                        WorkflowError::execution(format!("读取标准错误失败: {}", e))
                    })?;
                }
                
                let status = child.wait().await.map_err(|e| {
                    WorkflowError::execution(format!("等待进程结束失败: {}", e))
                })?;
                
                Ok::<_, WorkflowError>((stdout, stderr, status.code().unwrap_or(-1)))
            }
        ).await;
        
        let duration = start.elapsed();
        
        match result {
            Ok(Ok((stdout, stderr, exit_code))) => {
                let success = exit_code == 0;
                
                debug!(
                    script = %script,
                    exit_code = exit_code,
                    duration_ms = duration.as_millis(),
                    "Node.js 脚本执行完成"
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
                    "Node.js 脚本执行超时（{}秒）",
                    timeout_duration.as_secs()
                )))
            }
        }
    }

    fn name(&self) -> &str {
        "nodejs"
    }

    async fn is_available(&self) -> bool {
        let output = Command::new(&self.node_path)
            .arg("--version")
            .output()
            .await;
        
        output.is_ok()
    }

    async fn version(&self) -> Option<String> {
        let output = Command::new(&self.node_path)
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
    async fn test_nodejs_executor_creation() {
        let executor = NodeJsExecutor::new(ExternalExecutorConfig::default());
        assert_eq!(executor.name(), "nodejs");
    }

    #[tokio::test]
    async fn test_nodejs_version() {
        let executor = NodeJsExecutor::new(ExternalExecutorConfig::default());
        let version = executor.version().await;
        if executor.is_available().await {
            assert!(version.is_some());
        }
    }
}
