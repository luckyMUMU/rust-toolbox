//! 外部工具执行器
//!
//! 提供外部工具的统一执行接口，支持：
//! - Python 脚本执行
//! - Node.js 脚本执行
//! - Docker 容器执行

pub mod python;
pub mod nodejs;
pub mod docker;

pub use python::PythonExecutor;
pub use nodejs::NodeJsExecutor;
pub use docker::{DockerExecutor, DockerExecutorConfig, DockerMount};

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use serde_json::Value;
use std::time::Duration;

/// 外部工具执行器配置
#[derive(Debug, Clone)]
pub struct ExternalExecutorConfig {
    /// 执行超时
    pub timeout: Duration,
    /// 工作目录
    pub working_directory: Option<std::path::PathBuf>,
    /// 环境变量
    pub environment: std::collections::HashMap<String, String>,
    /// 最大输出大小（字节）
    pub max_output_size: usize,
    /// 是否捕获标准错误
    pub capture_stderr: bool,
}

impl Default for ExternalExecutorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300),
            working_directory: None,
            environment: std::collections::HashMap::new(),
            max_output_size: 10 * 1024 * 1024,
            capture_stderr: true,
        }
    }
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
    /// 执行时间
    pub duration: Duration,
    /// 是否成功
    pub success: bool,
}

impl ExecutionResult {
    /// 创建成功结果
    pub fn success(stdout: String, duration: Duration) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            exit_code: 0,
            duration,
            success: true,
        }
    }

    /// 创建失败结果
    pub fn failure(stderr: String, exit_code: i32, duration: Duration) -> Self {
        Self {
            stdout: String::new(),
            stderr,
            exit_code,
            duration,
            success: false,
        }
    }

    /// 解析 JSON 输出
    pub fn parse_json(&self) -> Result<Value> {
        serde_json::from_str(&self.stdout)
            .map_err(|e| WorkflowError::execution(format!("JSON 解析失败: {}", e)))
    }
}

/// 外部工具执行器 trait
#[async_trait::async_trait]
pub trait ExternalExecutor: Send + Sync {
    /// 执行工具
    async fn execute(
        &self,
        command: &str,
        args: &[String],
        input: Option<Value>,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionResult>;

    /// 获取执行器名称
    fn name(&self) -> &str;

    /// 检查执行器是否可用
    async fn is_available(&self) -> bool;

    /// 获取版本信息
    async fn version(&self) -> Option<String>;
}

/// 执行器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutorType {
    Python,
    NodeJs,
    Docker,
}

/// 执行器工厂
pub struct ExecutorFactory {
    config: ExternalExecutorConfig,
}

impl ExecutorFactory {
    /// 创建执行器工厂
    pub fn new(config: ExternalExecutorConfig) -> Self {
        Self { config }
    }

    /// 创建指定类型的执行器
    pub fn create(&self, executor_type: ExecutorType) -> Box<dyn ExternalExecutor> {
        match executor_type {
            ExecutorType::Python => Box::new(PythonExecutor::new(self.config.clone())),
            ExecutorType::NodeJs => Box::new(NodeJsExecutor::new(self.config.clone())),
            ExecutorType::Docker => Box::new(DockerExecutor::new(self.config.clone())),
        }
    }
}

impl Default for ExecutorFactory {
    fn default() -> Self {
        Self::new(ExternalExecutorConfig::default())
    }
}
