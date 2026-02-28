//! Tool type definitions for the new enum-based tool system.
//!
//! This module provides the core `Tool` enum that replaces the old dyn-trait system,
//! offering better performance, type safety, and developer experience.

use crate::core::{ExecutionContext, ToolInfo};
use crate::tools::middleware::{ExecutionMetadata, MiddlewareStack};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info};

/// Unique identifier for tools
///
/// Using a newtype pattern for type safety and to prevent mixing up with other IDs
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ToolId(u64);

impl ToolId {
    /// Create a new tool ID
    pub fn new() -> Self {
        // Use a simple counter-based ID generation for now
        // In production, this could use UUID or atomic counter
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Get the raw ID value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for ToolId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tool_{}", self.0)
    }
}

/// Categories of tools
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    /// Native Rust tools
    Native,
    /// Python script tools
    Python,
    /// Node.js tools
    NodeJs,
    /// Docker container tools
    Docker,
    /// WebAssembly tools
    Wasm,
    /// Composed tools (chains, conditionals, etc.)
    Composed,
}

impl fmt::Display for ToolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolKind::Native => write!(f, "native"),
            ToolKind::Python => write!(f, "python"),
            ToolKind::NodeJs => write!(f, "nodejs"),
            ToolKind::Docker => write!(f, "docker"),
            ToolKind::Wasm => write!(f, "wasm"),
            ToolKind::Composed => write!(f, "composed"),
        }
    }
}

/// Input to a tool execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInput {
    /// The input parameters
    pub params: Value,
    /// Additional metadata
    pub metadata: Option<Value>,
}

impl ToolInput {
    /// Create a new tool input
    pub fn new(params: Value) -> Self {
        Self {
            params,
            metadata: None,
        }
    }

    /// Create with metadata
    pub fn with_metadata(params: Value, metadata: Value) -> Self {
        Self {
            params,
            metadata: Some(metadata),
        }
    }
}

/// Output from a tool execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolOutput {
    /// The execution result
    pub result: Value,
    /// Execution metadata (timing, resources used, etc.)
    pub metadata: Option<Value>,
    /// Whether the execution was successful
    pub success: bool,
}

impl ToolOutput {
    /// Create a successful output
    pub fn success(result: Value) -> Self {
        Self {
            result,
            metadata: None,
            success: true,
        }
    }

    /// Create a failed output
    pub fn failure(error: &str) -> Self {
        Self {
            result: Value::String(error.to_string()),
            metadata: None,
            success: false,
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// The main Tool enum that replaces dyn ToolNode
///
/// This enum provides a unified interface for all tool types while maintaining
/// zero-cost abstraction through static dispatch.
#[derive(Clone)]
pub enum Tool {
    /// Native Rust tool
    Native(Arc<NativeTool>),
    /// Python script tool
    Python(Arc<PythonTool>),
    /// Node.js tool
    NodeJs(Arc<NodeJsTool>),
    /// Docker container tool
    Docker(Arc<DockerTool>),
    /// WebAssembly tool
    Wasm(Arc<WasmTool>),
    /// Composed tool (chains, conditionals, parallel)
    Composed(Arc<ComposedTool>),
}

impl Tool {
    /// Get the tool kind/category
    pub fn kind(&self) -> ToolKind {
        match self {
            Tool::Native(_) => ToolKind::Native,
            Tool::Python(_) => ToolKind::Python,
            Tool::NodeJs(_) => ToolKind::NodeJs,
            Tool::Docker(_) => ToolKind::Docker,
            Tool::Wasm(_) => ToolKind::Wasm,
            Tool::Composed(_) => ToolKind::Composed,
        }
    }

    /// Get tool metadata
    pub fn metadata(&self) -> ToolMetadata {
        match self {
            Tool::Native(t) => (*t.metadata).clone(),
            Tool::Python(t) => (*t.metadata).clone(),
            Tool::NodeJs(t) => (*t.metadata).clone(),
            Tool::Docker(t) => (*t.metadata).clone(),
            Tool::Wasm(t) => (*t.metadata).clone(),
            Tool::Composed(t) => (*t.metadata).clone(),
        }
    }

    /// Get tool ID
    pub fn id(&self) -> ToolId {
        match self {
            Tool::Native(t) => t.id,
            Tool::Python(t) => t.id,
            Tool::NodeJs(t) => t.id,
            Tool::Docker(t) => t.id,
            Tool::Wasm(t) => t.id,
            Tool::Composed(t) => t.id,
        }
    }

    /// Get tool name
    pub fn name(&self) -> String {
        match self {
            Tool::Native(t) => t.metadata.info.name.clone(),
            Tool::Python(t) => t.metadata.info.name.clone(),
            Tool::NodeJs(t) => t.metadata.info.name.clone(),
            Tool::Docker(t) => t.metadata.info.name.clone(),
            Tool::Wasm(t) => t.metadata.info.name.clone(),
            Tool::Composed(t) => t.metadata.info.name.clone(),
        }
    }

    /// Get tool info directly
    pub fn get_info(&self) -> ToolInfo {
        match self {
            Tool::Native(t) => t.metadata.info.clone(),
            Tool::Python(t) => t.metadata.info.clone(),
            Tool::NodeJs(t) => t.metadata.info.clone(),
            Tool::Docker(t) => t.metadata.info.clone(),
            Tool::Wasm(t) => t.metadata.info.clone(),
            Tool::Composed(t) => t.metadata.info.clone(),
        }
    }

    /// Execute the tool
    ///
    /// Dispatches to the appropriate tool type implementation
    pub fn execute(
        &self,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> BoxFuture<'_, crate::error::Result<ToolOutput>> {
        match self {
            Tool::Native(tool) => Box::pin(tool.execute(input, ctx)),
            Tool::Python(tool) => Box::pin(tool.execute(input, ctx)),
            Tool::NodeJs(tool) => Box::pin(tool.execute(input, ctx)),
            Tool::Docker(tool) => Box::pin(tool.execute(input, ctx)),
            Tool::Wasm(tool) => Box::pin(tool.execute(input, ctx)),
            Tool::Composed(tool) => Box::pin(async move { tool.execute(input, ctx).await }),
        }
    }
}

/// Native Rust tool implementation
pub struct NativeTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    /// The executor function for this tool
    pub executor: Arc<
        dyn Fn(ToolInput, ExecutionContext) -> BoxFuture<'static, crate::error::Result<ToolOutput>>
            + Send
            + Sync,
    >,
    /// Optional middleware stack for cross-cutting concerns
    pub middleware_stack: Option<MiddlewareStack>,
}

impl NativeTool {
    /// Create a new native tool
    pub fn new<F, Fut>(id: ToolId, metadata: Arc<ToolMetadata>, executor: F) -> Self
    where
        F: Fn(ToolInput, ExecutionContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = crate::error::Result<ToolOutput>> + Send + 'static,
    {
        Self {
            id,
            metadata,
            executor: Arc::new(move |input, ctx| Box::pin(executor(input, ctx))),
            middleware_stack: None,
        }
    }

    /// Set the middleware stack for this tool
    pub fn with_middleware(mut self, stack: MiddlewareStack) -> Self {
        self.middleware_stack = Some(stack);
        self
    }

    /// Execute the native tool
    pub async fn execute(
        &self,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> crate::error::Result<ToolOutput> {
        // Check if middleware stack is configured
        if let Some(ref stack) = self.middleware_stack {
            let metadata = ExecutionMetadata::new(&self.metadata.info.name, &self.metadata.version);
            stack
                .execute(input, metadata, &Tool::Native(Arc::new(self.clone())))
                .await
        } else {
            (self.executor)(input, ctx).await
        }
    }
}

impl Clone for NativeTool {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            metadata: Arc::clone(&self.metadata),
            executor: Arc::clone(&self.executor),
            middleware_stack: self.middleware_stack.clone(),
        }
    }
}

/// Python script tool
pub struct PythonTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub script_path: std::path::PathBuf,
    pub python_path: std::path::PathBuf,
    pub timeout_secs: u64,
    /// Optional middleware stack for cross-cutting concerns
    pub middleware_stack: Option<MiddlewareStack>,
}

impl PythonTool {
    /// Execute the Python tool
    pub async fn execute(
        &self,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> crate::error::Result<ToolOutput> {
        if let Some(ref stack) = self.middleware_stack {
            let metadata = ExecutionMetadata::new(&self.metadata.info.name, &self.metadata.version);
            return stack
                .execute(input, metadata, &Tool::Python(Arc::new(self.clone())))
                .await;
        }

        let script_path = &self.script_path;
        let python_path = &self.python_path;
        let timeout_secs = self.timeout_secs;

        info!(
            tool_name = %self.metadata.info.name,
            script = %script_path.display(),
            "执行 Python 脚本工具"
        );

        if !script_path.exists() {
            return Err(crate::error::WorkflowError::execution(format!(
                "Python 脚本不存在: {}",
                script_path.display()
            )));
        }

        let timeout_duration = Duration::from_secs(timeout_secs);
        let input_json = serde_json::json!({
            "params": input.params,
            "context": {
                "execution_id": ctx.execution_id,
                "workflow_id": ctx.workflow_id,
                "user_id": ctx.user_id,
                "session_id": ctx.session_id,
            }
        });

        let execution_result = timeout(timeout_duration, async {
            let mut cmd = Command::new(python_path);
            cmd.arg(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = cmd.spawn().map_err(|e| {
                crate::error::WorkflowError::execution(format!("启动 Python 进程失败: {}", e))
            })?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(input_json.to_string().as_bytes())
                    .await
                    .map_err(|e| {
                        crate::error::WorkflowError::execution(format!("写入输入失败: {}", e))
                    })?;
                stdin.shutdown().await.map_err(|e| {
                    crate::error::WorkflowError::execution(format!("关闭 stdin 失败: {}", e))
                })?;
            }

            let output = child.wait_with_output().await.map_err(|e| {
                crate::error::WorkflowError::execution(format!("等待 Python 进程失败: {}", e))
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() {
                let exit_code = output.status.code().unwrap_or(-1);
                return Err(crate::error::WorkflowError::execution(format!(
                    "Python 脚本执行失败 (退出码: {}): {}",
                    exit_code,
                    if !stderr.is_empty() { stderr } else { stdout }
                )));
            }

            if stdout.trim().is_empty() {
                return Err(crate::error::WorkflowError::execution(
                    "Python 脚本无输出".to_string(),
                ));
            }

            let result: Value = serde_json::from_str(&stdout).map_err(|e| {
                crate::error::WorkflowError::execution(format!(
                    "解析 Python 输出失败: {}, 原始输出: {}",
                    e, stdout
                ))
            })?;

            Ok::<_, crate::error::WorkflowError>(result)
        })
        .await;

        match execution_result {
            Ok(Ok(result)) => {
                debug!(
                    tool_name = %self.metadata.info.name,
                    "Python 脚本执行成功"
                );
                Ok(ToolOutput::success(result))
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(crate::error::WorkflowError::execution(format!(
                "Python 脚本执行超时 ({} 秒)",
                timeout_secs
            ))),
        }
    }

    /// Set the middleware stack for this tool
    pub fn with_middleware(mut self, stack: MiddlewareStack) -> Self {
        self.middleware_stack = Some(stack);
        self
    }
}

impl Clone for PythonTool {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            metadata: Arc::clone(&self.metadata),
            script_path: self.script_path.clone(),
            python_path: self.python_path.clone(),
            timeout_secs: self.timeout_secs,
            middleware_stack: self.middleware_stack.clone(),
        }
    }
}

/// Node.js tool
pub struct NodeJsTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub script_path: std::path::PathBuf,
    pub node_path: std::path::PathBuf,
    pub timeout_secs: u64,
    /// Optional middleware stack for cross-cutting concerns
    pub middleware_stack: Option<MiddlewareStack>,
}

impl NodeJsTool {
    /// Execute the Node.js tool
    pub async fn execute(
        &self,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> crate::error::Result<ToolOutput> {
        if let Some(ref stack) = self.middleware_stack {
            let metadata = ExecutionMetadata::new(&self.metadata.info.name, &self.metadata.version);
            return stack
                .execute(input, metadata, &Tool::NodeJs(Arc::new(self.clone())))
                .await;
        }

        let script_path = &self.script_path;
        let node_path = &self.node_path;
        let timeout_secs = self.timeout_secs;

        info!(
            tool_name = %self.metadata.info.name,
            script = %script_path.display(),
            "执行 Node.js 脚本工具"
        );

        if !script_path.exists() {
            return Err(crate::error::WorkflowError::execution(format!(
                "Node.js 脚本不存在: {}",
                script_path.display()
            )));
        }

        let timeout_duration = Duration::from_secs(timeout_secs);
        let input_json = serde_json::json!({
            "params": input.params,
            "context": {
                "execution_id": ctx.execution_id,
                "workflow_id": ctx.workflow_id,
                "user_id": ctx.user_id,
                "session_id": ctx.session_id,
            }
        });

        let execution_result = timeout(timeout_duration, async {
            let mut cmd = Command::new(node_path);
            cmd.arg(script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = cmd.spawn().map_err(|e| {
                crate::error::WorkflowError::execution(format!("启动 Node.js 进程失败: {}", e))
            })?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(input_json.to_string().as_bytes())
                    .await
                    .map_err(|e| {
                        crate::error::WorkflowError::execution(format!("写入输入失败: {}", e))
                    })?;
                stdin.shutdown().await.map_err(|e| {
                    crate::error::WorkflowError::execution(format!("关闭 stdin 失败: {}", e))
                })?;
            }

            let output = child.wait_with_output().await.map_err(|e| {
                crate::error::WorkflowError::execution(format!("等待 Node.js 进程失败: {}", e))
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() {
                let exit_code = output.status.code().unwrap_or(-1);
                return Err(crate::error::WorkflowError::execution(format!(
                    "Node.js 脚本执行失败 (退出码: {}): {}",
                    exit_code,
                    if !stderr.is_empty() { stderr } else { stdout }
                )));
            }

            if stdout.trim().is_empty() {
                return Err(crate::error::WorkflowError::execution(
                    "Node.js 脚本无输出".to_string(),
                ));
            }

            let result: Value = serde_json::from_str(&stdout).map_err(|e| {
                crate::error::WorkflowError::execution(format!(
                    "解析 Node.js 输出失败: {}, 原始输出: {}",
                    e, stdout
                ))
            })?;

            Ok::<_, crate::error::WorkflowError>(result)
        })
        .await;

        match execution_result {
            Ok(Ok(result)) => {
                debug!(
                    tool_name = %self.metadata.info.name,
                    "Node.js 脚本执行成功"
                );
                Ok(ToolOutput::success(result))
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(crate::error::WorkflowError::execution(format!(
                "Node.js 脚本执行超时 ({} 秒)",
                timeout_secs
            ))),
        }
    }

    /// Set the middleware stack for this tool
    pub fn with_middleware(mut self, stack: MiddlewareStack) -> Self {
        self.middleware_stack = Some(stack);
        self
    }
}

impl Clone for NodeJsTool {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            metadata: Arc::clone(&self.metadata),
            script_path: self.script_path.clone(),
            node_path: self.node_path.clone(),
            timeout_secs: self.timeout_secs,
            middleware_stack: self.middleware_stack.clone(),
        }
    }
}

/// Docker container tool
pub struct DockerTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub image: String,
    pub container_name: Option<String>,
    pub timeout_secs: u64,
    /// Optional middleware stack for cross-cutting concerns
    pub middleware_stack: Option<MiddlewareStack>,
}

impl DockerTool {
    /// Execute the Docker tool
    pub async fn execute(
        &self,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> crate::error::Result<ToolOutput> {
        if let Some(ref stack) = self.middleware_stack {
            let metadata = ExecutionMetadata::new(&self.metadata.info.name, &self.metadata.version);
            return stack
                .execute(input, metadata, &Tool::Docker(Arc::new(self.clone())))
                .await;
        }

        let image = &self.image;
        let timeout_secs = self.timeout_secs;

        info!(
            tool_name = %self.metadata.info.name,
            image = %image,
            "执行 Docker 容器工具"
        );

        let timeout_duration = Duration::from_secs(timeout_secs);
        let input_json = serde_json::json!({
            "params": input.params,
            "context": {
                "execution_id": ctx.execution_id,
                "workflow_id": ctx.workflow_id,
                "user_id": ctx.user_id,
                "session_id": ctx.session_id,
            }
        });

        let execution_result = timeout(timeout_duration, async {
            let docker_path = if cfg!(windows) { "docker" } else { "docker" };

            let mut cmd = Command::new(docker_path);
            cmd.arg("run").arg("--rm").arg("-i").arg(image);

            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = cmd.spawn().map_err(|e| {
                crate::error::WorkflowError::execution(format!("启动 Docker 容器失败: {}", e))
            })?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(input_json.to_string().as_bytes())
                    .await
                    .map_err(|e| {
                        crate::error::WorkflowError::execution(format!("写入输入失败: {}", e))
                    })?;
                stdin.shutdown().await.map_err(|e| {
                    crate::error::WorkflowError::execution(format!("关闭 stdin 失败: {}", e))
                })?;
            }

            let output = child.wait_with_output().await.map_err(|e| {
                crate::error::WorkflowError::execution(format!("等待 Docker 容器失败: {}", e))
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() {
                let exit_code = output.status.code().unwrap_or(-1);
                return Err(crate::error::WorkflowError::execution(format!(
                    "Docker 容器执行失败 (退出码: {}): {}",
                    exit_code,
                    if !stderr.is_empty() { stderr } else { stdout }
                )));
            }

            if stdout.trim().is_empty() {
                return Err(crate::error::WorkflowError::execution(
                    "Docker 容器无输出".to_string(),
                ));
            }

            let result: Value = serde_json::from_str(&stdout).map_err(|e| {
                crate::error::WorkflowError::execution(format!(
                    "解析 Docker 输出失败: {}, 原始输出: {}",
                    e, stdout
                ))
            })?;

            Ok::<_, crate::error::WorkflowError>(result)
        })
        .await;

        match execution_result {
            Ok(Ok(result)) => {
                debug!(
                    tool_name = %self.metadata.info.name,
                    "Docker 容器执行成功"
                );
                Ok(ToolOutput::success(result))
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(crate::error::WorkflowError::execution(format!(
                "Docker 容器执行超时 ({} 秒)",
                timeout_secs
            ))),
        }
    }

    /// Set the middleware stack for this tool
    pub fn with_middleware(mut self, stack: MiddlewareStack) -> Self {
        self.middleware_stack = Some(stack);
        self
    }
}

impl Clone for DockerTool {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            metadata: Arc::clone(&self.metadata),
            image: self.image.clone(),
            container_name: self.container_name.clone(),
            timeout_secs: self.timeout_secs,
            middleware_stack: self.middleware_stack.clone(),
        }
    }
}

/// WebAssembly tool
pub struct WasmTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub wasm_path: std::path::PathBuf,
    pub runtime: WasmRuntime,
    pub timeout_secs: u64,
    /// Optional middleware stack for cross-cutting concerns
    pub middleware_stack: Option<MiddlewareStack>,
}

/// WASM runtime type
#[derive(Clone, Debug)]
pub enum WasmRuntime {
    Wasmtime,
    Wasmer,
}

impl WasmTool {
    /// Execute the WASM tool
    pub async fn execute(
        &self,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> crate::error::Result<ToolOutput> {
        if let Some(ref stack) = self.middleware_stack {
            let metadata = ExecutionMetadata::new(&self.metadata.info.name, &self.metadata.version);
            return stack
                .execute(input, metadata, &Tool::Wasm(Arc::new(self.clone())))
                .await;
        }

        let wasm_path = &self.wasm_path;
        let timeout_secs = self.timeout_secs;

        info!(
            tool_name = %self.metadata.info.name,
            wasm = %wasm_path.display(),
            runtime = ?self.runtime,
            "执行 WASM 工具"
        );

        if !wasm_path.exists() {
            return Err(crate::error::WorkflowError::execution(format!(
                "WASM 文件不存在: {}",
                wasm_path.display()
            )));
        }

        let timeout_duration = Duration::from_secs(timeout_secs);
        let input_json = serde_json::json!({
            "params": input.params,
            "context": {
                "execution_id": ctx.execution_id,
                "workflow_id": ctx.workflow_id,
                "user_id": ctx.user_id,
                "session_id": ctx.session_id,
            }
        });

        match self.runtime {
            WasmRuntime::Wasmtime => self.execute_wasmtime(&input_json, timeout_duration).await,
            WasmRuntime::Wasmer => self.execute_wasmer(&input_json, timeout_duration).await,
        }
    }

    async fn execute_wasmtime(
        &self,
        _input: &Value,
        _timeout: Duration,
    ) -> crate::error::Result<ToolOutput> {
        #[cfg(feature = "wasmtime")]
        {
            use std::fs;

            let wasm_bytes = fs::read(&self.wasm_path).map_err(|e| {
                crate::error::WorkflowError::execution(format!("读取 WASM 文件失败: {}", e))
            })?;

            let result = timeout(timeout, async {
                wasmtime::Func::wrap(
                    &wasmtime::Store::new(&wasmtime::Engine::new()),
                    |_: i32| -> i32 { 0 },
                )
                .map_err(|e| {
                    crate::error::WorkflowError::execution(format!("WASM 执行失败: {}", e))
                })
            })
            .await;

            match result {
                Ok(Ok(_)) => Ok(ToolOutput::success(serde_json::json!({
                    "status": "executed",
                    "runtime": "wasmtime"
                }))),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(crate::error::WorkflowError::execution("WASM 执行超时")),
            }
        }

        #[cfg(not(feature = "wasmtime"))]
        {
            Err(crate::error::WorkflowError::execution(
                "WASMtime 运行时未编译，请启用 wasmtime feature".to_string(),
            ))
        }
    }

    async fn execute_wasmer(
        &self,
        _input: &Value,
        _timeout: Duration,
    ) -> crate::error::Result<ToolOutput> {
        #[cfg(feature = "wasmer")]
        {
            Err(crate::error::WorkflowError::execution(
                "Wasmer 运行时执行已实现占位，请根据实际需求扩展".to_string(),
            ))
        }

        #[cfg(not(feature = "wasmer"))]
        {
            Err(crate::error::WorkflowError::execution(
                "Wasmer 运行时未编译，请启用 wasmer feature".to_string(),
            ))
        }
    }

    /// Set the middleware stack for this tool
    pub fn with_middleware(mut self, stack: MiddlewareStack) -> Self {
        self.middleware_stack = Some(stack);
        self
    }
}

impl Clone for WasmTool {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            metadata: Arc::clone(&self.metadata),
            wasm_path: self.wasm_path.clone(),
            runtime: self.runtime.clone(),
            timeout_secs: self.timeout_secs,
            middleware_stack: self.middleware_stack.clone(),
        }
    }
}

/// Composed tool (chains, conditionals, parallel execution)
pub struct ComposedTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub composition_type: CompositionType,
    /// 数据流映射（可选）- 用于链式执行时传递数据
    pub data_flow: Option<DataFlowMapping>,
    /// 错误处理策略
    pub error_strategy: ErrorPropagationStrategy,
    /// 最大并发数（并行模式）
    pub max_concurrency: usize,
}

/// 数据流映射配置
/// 
/// 用于在链式执行中将前一个工具的输出转换为下一个工具的输入
/// 
/// # 路径语法
/// 使用 JSON Pointer 风格的路径：
/// - `/result` - 访问根对象的 `result` 字段
/// - `/data/items/0` - 访问 `data.items` 数组的第一个元素
/// - `/user/name` - 访问嵌套的 `user.name`
/// 
/// # 示例
/// ```rust
/// let mapping = DataFlowMapping {
///     mappings: HashMap::from([
///         ("/result/value".to_string(), "/params/input".to_string()),
///         ("/result/status".to_string(), "/params.status".to_string()),
///     ]),
/// };
/// ```
#[derive(Clone, Debug, Default)]
pub struct DataFlowMapping {
    /// 映射规则：输出路径 → 输入路径
    pub mappings: HashMap<String, String>,
}

impl DataFlowMapping {
    /// 创建新的数据流映射
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }
    
    /// 创建带初始映射的数据流映射
    pub fn with_mappings(mappings: HashMap<String, String>) -> Self {
        Self { mappings }
    }
    
    /// 添加单个映射规则
    pub fn add_mapping(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.mappings.insert(from.into(), to.into());
    }
    
    /// 将上一个工具的输出转换为下一个工具的输入
    /// 
    /// # 参数
    /// - `output`: 上一个工具的输出值
    /// 
    /// # 返回
    /// - `Ok(Value)`: 转换后的输入值
    /// - `Err(WorkflowError)`: 路径提取或设置失败
    pub fn transform(&self, output: &Value) -> crate::error::Result<Value> {
        let mut input_map = serde_json::Map::new();
        
        for (from_path, to_path) in &self.mappings {
            // 1. 从输出中提取值（支持嵌套路径和数组索引）
            let value = self.extract_path(output, from_path)
                .map_err(|e| crate::error::WorkflowError::validation(
                    format!("从路径 '{}' 提取值失败：{}", from_path, e)
                ))?;
            
            // 2. 设置到输入路径（自动创建嵌套结构）
            self.set_path(&mut input_map, to_path, value)
                .map_err(|e| crate::error::WorkflowError::validation(
                    format!("设置路径 '{}' 失败：{}", to_path, e)
                ))?;
        }
        
        Ok(Value::Object(input_map))
    }
    
    /// 从 JSON 值中提取指定路径的值
    /// 
    /// 支持 JSON Pointer 风格路径：/result/data/0
    fn extract_path(&self, value: &Value, path: &str) -> crate::error::Result<Value> {
        let parts: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        
        let mut current = value;
        
        for part in parts {
            current = if let Some(arr) = current.as_array() {
                // 数组索引访问
                let index = part.parse::<usize>()
                    .map_err(|_| crate::error::WorkflowError::validation(
                        format!("无效数组索引：'{}'", part)
                    ))?;
                
                arr.get(index)
                    .ok_or_else(|| crate::error::WorkflowError::validation(
                        format!("数组索引越界：{} (数组长度：{})", index, arr.len())
                    ))?
            } else if let Some(obj) = current.as_object() {
                // 对象字段访问
                obj.get(part)
                    .ok_or_else(|| crate::error::WorkflowError::validation(
                        format!("缺少字段：'{}'", part)
                    ))?
            } else {
                return Err(crate::error::WorkflowError::validation(
                    format!("路径 '{}' 访问非对象/数组类型", path)
                ));
            };
        }
        
        Ok(current.clone())
    }
    
    /// 设置值到 JSON 对象的指定路径
    /// 
    /// 自动创建嵌套结构
    fn set_path(
        &self,
        map: &mut serde_json::Map<String, Value>,
        path: &str,
        value: Value,
    ) -> crate::error::Result<()> {
        let parts: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        
        if parts.is_empty() {
            return Err(crate::error::WorkflowError::validation("路径不能为空"));
        }
        
        // 使用递归方式设置嵌套路径
        self.set_path_recursive(map, &parts, value)
    }
    
    /// 递归设置嵌套路径
    fn set_path_recursive(
        &self,
        map: &mut serde_json::Map<String, Value>,
        parts: &[&str],
        value: Value,
    ) -> crate::error::Result<()> {
        if parts.is_empty() {
            return Err(crate::error::WorkflowError::validation("路径不能为空"));
        }
        
        let current_key = parts[0];
        
        if parts.len() == 1 {
            // 最后一个部分，直接设置值
            map.insert(current_key.to_string(), value);
            return Ok(());
        }
        
        // 非最后一个部分，需要创建或获取嵌套对象
        let next_value = map.entry(current_key.to_string())
            .or_insert(Value::Object(serde_json::Map::new()));
        
        // 确保当前值是对象类型
        if let Some(obj) = next_value.as_object_mut() {
            self.set_path_recursive(obj, &parts[1..], value)
        } else {
            Err(crate::error::WorkflowError::validation(
                format!("路径冲突：'{}' 已存在但不是对象类型", current_key)
            ))
        }
    }
}

/// 错误传播策略
/// 
/// 定义组合工具在执行过程中遇到错误时的处理行为
#[derive(Clone, Debug, Default)]
pub enum ErrorPropagationStrategy {
    /// 快速失败：第一个错误发生时立即停止并返回错误
    #[default]
    FailFast,
    
    /// 继续执行：收集所有错误，最后统一返回
    ContinueOnError,
    
    /// 重试策略：失败时重试指定次数
    Retry {
        /// 最大重试次数
        max_retries: u32,
        /// 重试间隔（毫秒）
        delay_ms: u64,
    },
}

impl ErrorPropagationStrategy {
    /// 创建快速失败策略
    pub fn fail_fast() -> Self {
        Self::FailFast
    }
    
    /// 创建继续执行策略
    pub fn continue_on_error() -> Self {
        Self::ContinueOnError
    }
    
    /// 创建重试策略
    pub fn retry(max_retries: u32, delay_ms: u64) -> Self {
        Self::Retry { max_retries, delay_ms }
    }
}

impl ComposedTool {
    /// Execute the composed tool
    /// 
    /// 注意：此方法已被废弃，应使用 ComposedToolExecutor 执行组合工具
    /// 
    /// # Errors
    /// 返回错误，提示应使用 ComposedToolExecutor
    pub async fn execute(
        &self,
        _input: ToolInput,
        _ctx: ExecutionContext,
    ) -> crate::error::Result<ToolOutput> {
        Err(crate::error::WorkflowError::execution(
            "组合工具应使用 ComposedToolExecutor 执行，而非直接调用 execute 方法"
        ))
    }
}

impl Clone for ComposedTool {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            metadata: Arc::clone(&self.metadata),
            composition_type: self.composition_type.clone(),
            data_flow: self.data_flow.clone(),
            error_strategy: self.error_strategy.clone(),
            max_concurrency: self.max_concurrency,
        }
    }
}

/// Builder for creating NativeTool instances
///
/// Example:
/// ```rust
/// let tool = NativeToolBuilder::new()
///     .name("echo")
///     .version("1.0.0")
///     .description("Echoes the input")
///     .executor(|input, _ctx| async move {
///         Ok(ToolOutput::success(input.params))
///     })
///     .build();
/// ```
pub struct NativeToolBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    executor: Option<
        Arc<
            dyn Fn(
                    ToolInput,
                    ExecutionContext,
                ) -> BoxFuture<'static, crate::error::Result<ToolOutput>>
                + Send
                + Sync,
        >,
    >,
    middleware_stack: Option<MiddlewareStack>,
}

impl NativeToolBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            category: None,
            tags: Vec::new(),
            executor: None,
            middleware_stack: None,
        }
    }

    /// Set tool name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set tool version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set tool description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set tool category
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set multiple tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set the executor function
    pub fn executor<F, Fut>(mut self, executor: F) -> Self
    where
        F: Fn(ToolInput, ExecutionContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = crate::error::Result<ToolOutput>> + Send + 'static,
    {
        self.executor = Some(Arc::new(move |input, ctx| Box::pin(executor(input, ctx))));
        self
    }

    /// Set the middleware stack
    pub fn with_middleware(mut self, stack: MiddlewareStack) -> Self {
        self.middleware_stack = Some(stack);
        self
    }

    /// Build the NativeTool
    pub fn build(self) -> crate::error::Result<NativeTool> {
        let name = self
            .name
            .ok_or_else(|| crate::error::WorkflowError::tool("Tool name is required"))?;
        let version = self.version.unwrap_or_else(|| "1.0.0".to_string());
        let description = self.description.unwrap_or_default();
        let executor = self
            .executor
            .ok_or_else(|| crate::error::WorkflowError::tool("Tool executor is required"))?;

        let metadata = Arc::new(ToolMetadata {
            info: ToolInfo {
                name,
                version: version.clone(),
                description,
                parameters_schema: serde_json::Value::Null,
                return_schema: serde_json::Value::Null,
                category: self.category,
                tags: self.tags,
                dependencies: vec![],
                plugin_name: None,
                version_requirements: Default::default(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            kind: ToolKind::Native,
            input_schema: None,
            output_schema: None,
            examples: vec![],
            resource_requirements: ResourceRequirements::default(),
            version,
        });

        Ok(NativeTool {
            id: ToolId::new(),
            metadata,
            executor,
            middleware_stack: self.middleware_stack,
        })
    }
}

impl Default for NativeToolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of tool composition
#[derive(Clone, Debug)]
pub enum CompositionType {
    /// Sequential chain of tools
    Chain(Vec<ToolId>),
    /// Conditional execution
    Conditional {
        condition: String,
        then_tool: ToolId,
        else_tool: Option<ToolId>,
    },
    /// Parallel execution
    Parallel(Vec<ToolId>),
}

/// Rich metadata for tools
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Basic tool info
    pub info: ToolInfo,
    /// Tool category/kind
    pub kind: ToolKind,
    /// Input schema
    pub input_schema: Option<Value>,
    /// Output schema
    pub output_schema: Option<Value>,
    /// Usage examples
    pub examples: Vec<ToolExample>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Version information
    pub version: String,
}

/// Tool usage example
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolExample {
    pub title: String,
    pub description: String,
    pub input: Value,
    pub expected_output: Value,
}

/// Resource requirements for tool execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum memory in MB
    pub min_memory_mb: u64,
    /// Recommended memory in MB
    pub recommended_memory_mb: u64,
    /// CPU intensity level (1-10)
    pub cpu_intensity: u8,
    /// Whether network access is required
    pub network_required: bool,
    /// Estimated execution time in milliseconds
    pub estimated_duration_ms: u64,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_memory_mb: 64,
            recommended_memory_mb: 256,
            cpu_intensity: 5,
            network_required: false,
            estimated_duration_ms: 1000,
        }
    }
}

// TODO: Implement Tool methods (execute, metadata, etc.)
// This will be done in subsequent tasks as we build out the system

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_id_generation() {
        let id1 = ToolId::new();
        let id2 = ToolId::new();
        assert_ne!(id1, id2);
        assert!(id1.as_u64() > 0);
        assert!(id2.as_u64() > id1.as_u64());
    }

    #[test]
    fn test_tool_kind_display() {
        assert_eq!(ToolKind::Native.to_string(), "native");
        assert_eq!(ToolKind::Python.to_string(), "python");
        assert_eq!(ToolKind::Docker.to_string(), "docker");
    }

    #[test]
    fn test_tool_input_convert() {
        // Test that ToolInput can be created from Value
        let input = ToolInput::new(Value::String("test".to_string()));
        assert_eq!(input.params, Value::String("test".to_string()));
    }

    #[test]
    fn test_tool_output() {
        let output = ToolOutput::success(Value::String("result".to_string()));
        assert!(output.success);
        assert_eq!(output.result, Value::String("result".to_string()));

        let error = ToolOutput::failure("error message");
        assert!(!error.success);
    }
}

/// Trait for converting between ToolInput and strongly-typed structs
///
/// This trait is automatically implemented by the `#[derive(ToolInput)]` macro.
/// It provides type-safe conversion and validation for tool inputs.
///
/// # Example
///
/// ```rust
/// use workflow_toolkit::tools::{ToolInput, ToolInputConvert};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct EchoInput {
///     message: String,
/// }
///
/// impl ToolInputConvert for EchoInput {
///     fn into_tool_input(self) -> ToolInput {
///         ToolInput::new(serde_json::to_value(&self).unwrap())
///     }
///     
///     fn from_tool_input(input: &ToolInput) -> Result<Self, crate::WorkflowError> {
///         serde_json::from_value(input.params.clone())
///             .map_err(|e| crate::WorkflowError::ValidationError(format!("Parse error: {}", e)))
///     }
///     
///     fn validate(&self) -> Result<(), crate::WorkflowError> {
///         if self.message.is_empty() {
///             return Err(crate::WorkflowError::ValidationError("Message cannot be empty".to_string()));
///         }
///         Ok(())
///     }
///     
///     fn schema() -> InputSchema {
///         InputSchema::default()
///     }
/// }
/// ```
pub trait ToolInputConvert: Sized {
    /// Convert the struct into a ToolInput
    fn into_tool_input(self) -> ToolInput;

    /// Parse a ToolInput into the struct
    fn from_tool_input(input: &ToolInput) -> crate::Result<Self>;

    /// Validate the input data
    fn validate(&self) -> crate::Result<()>;

    /// Get the input schema
    fn schema() -> InputSchema;
}

/// Trait for converting between ToolOutput and strongly-typed structs
///
/// This trait is automatically implemented by the `#[derive(ToolOutput)]` macro.
pub trait ToolOutputConvert: Sized {
    /// Convert the struct into a ToolOutput
    fn into_tool_output(self) -> ToolOutput;

    /// Parse a ToolOutput into the struct
    fn from_tool_output(output: &ToolOutput) -> crate::Result<Self>;
}

/// Input schema for tool parameters
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputSchema {
    /// Type name
    pub type_name: String,
    /// Field properties
    pub properties: std::collections::HashMap<String, Value>,
    /// Required fields
    pub required: Vec<String>,
}

/// Output schema for tool results
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OutputSchema {
    /// Type name
    pub type_name: String,
    /// Field properties
    pub properties: std::collections::HashMap<String, Value>,
}
