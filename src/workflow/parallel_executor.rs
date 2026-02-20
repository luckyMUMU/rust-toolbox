//! 并行执行优化模块
//!
//! 使用 JoinSet 替代 join_all，提供更好的任务管理和取消支持

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::tools::types::{Tool, ToolInput, ToolOutput};
use crate::workflow::component::{ComponentOutput, ComponentRegistry, ComponentStatus};
use crate::workflow::context::DataContext;
use crate::workflow::executor::BoxedExecutor;
use crate::workflow::state::ExecutionTracker;
use futures::future::{self, Either};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::{AbortHandle, JoinSet};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// 并行执行配置
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// 最大并发数
    pub max_concurrency: usize,
    /// 单任务超时时间
    pub task_timeout: Duration,
    /// 是否启用任务取消
    pub enable_cancellation: bool,
    /// 失败时是否取消其他任务
    pub cancel_on_failure: bool,
    /// 是否收集所有结果（即使有失败）
    pub collect_all_results: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 4,
            task_timeout: Duration::from_secs(300),
            enable_cancellation: true,
            cancel_on_failure: false,
            collect_all_results: true,
        }
    }
}

/// 并行任务执行结果
#[derive(Debug)]
pub struct ParallelResult<T> {
    /// 成功的结果
    pub successes: Vec<(String, T)>,
    /// 失败的结果
    pub failures: Vec<(String, WorkflowError)>,
    /// 总执行时间
    pub total_duration: Duration,
    /// 是否全部成功
    pub all_succeeded: bool,
    /// 被取消的任务数
    pub cancelled_count: usize,
}

impl<T> ParallelResult<T> {
    /// 创建新的并行结果
    pub fn new() -> Self {
        Self {
            successes: Vec::new(),
            failures: Vec::new(),
            total_duration: Duration::ZERO,
            all_succeeded: true,
            cancelled_count: 0,
        }
    }

    /// 添加成功结果
    pub fn add_success(&mut self, task_id: String, result: T) {
        self.successes.push((task_id, result));
    }

    /// 添加失败结果
    pub fn add_failure(&mut self, task_id: String, error: WorkflowError) {
        self.failures.push((task_id, error));
        self.all_succeeded = false;
    }

    /// 增加取消计数
    pub fn add_cancelled(&mut self) {
        self.cancelled_count += 1;
        self.all_succeeded = false;
    }

    /// 获取成功数量
    pub fn success_count(&self) -> usize {
        self.successes.len()
    }

    /// 获取失败数量
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// 是否有任何成功
    pub fn has_successes(&self) -> bool {
        !self.successes.is_empty()
    }
}

impl<T> Default for ParallelResult<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// 任务执行器 trait
pub trait TaskExecutor<T>: Send + Sync {
    /// 执行任务
    fn execute(
        &self,
        task_id: &str,
        context: &mut DataContext,
        exec_context: &ExecutionContext,
    ) -> impl std::future::Future<Output = Result<T>> + Send;
}

/// 组件任务执行器
pub struct ComponentTaskExecutor {
    executor: Arc<BoxedExecutor>,
    registry: Arc<ComponentRegistry>,
    workflow_id: Uuid,
}

impl ComponentTaskExecutor {
    /// 创建新的组件任务执行器
    pub fn new(
        executor: Arc<BoxedExecutor>,
        registry: Arc<ComponentRegistry>,
        workflow_id: Uuid,
    ) -> Self {
        Self {
            executor,
            registry,
            workflow_id,
        }
    }
}

impl TaskExecutor<ComponentOutput> for ComponentTaskExecutor {
    async fn execute(
        &self,
        task_id: &str,
        context: &mut DataContext,
        exec_context: &ExecutionContext,
    ) -> Result<ComponentOutput> {
        let component = self.registry.get(task_id)?;
        self.executor
            .execute(component.as_ref(), context, exec_context)
            .await
    }
}

/// 工具任务执行器
pub struct ToolTaskExecutor {
    tools: HashMap<String, Arc<Tool>>,
}

impl ToolTaskExecutor {
    /// 创建新的工具任务执行器
    pub fn new(tools: HashMap<String, Arc<Tool>>) -> Self {
        Self { tools }
    }

    /// 添加工具
    pub fn add_tool(&mut self, name: String, tool: Arc<Tool>) {
        self.tools.insert(name, tool);
    }
}

impl TaskExecutor<ToolOutput> for ToolTaskExecutor {
    async fn execute(
        &self,
        task_id: &str,
        context: &mut DataContext,
        exec_context: &ExecutionContext,
    ) -> Result<ToolOutput> {
        let tool = self
            .tools
            .get(task_id)
            .ok_or_else(|| WorkflowError::validation(format!("工具未找到: {}", task_id)))?;

        let input = ToolInput::new(context.get_local_slots().clone());
        tool.execute(input, exec_context.clone()).await
    }
}

/// JoinSet 并行执行器
pub struct JoinSetExecutor<T> {
    config: ParallelConfig,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + 'static> JoinSetExecutor<T> {
    /// 创建新的 JoinSet 执行器
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 使用默认配置创建执行器
    pub fn default_executor() -> Self {
        Self::new(ParallelConfig::default())
    }

    /// 并行执行多个任务
    pub async fn execute_parallel<E>(
        &self,
        tasks: Vec<String>,
        executor: Arc<E>,
        context: &mut DataContext,
        workflow_id: Uuid,
    ) -> ParallelResult<T>
    where
        E: TaskExecutor<T> + ?Sized,
    {
        let start_time = Instant::now();
        let mut result = ParallelResult::new();
        let mut join_set: JoinSet<(String, Result<T>)> = JoinSet::new();
        let mut task_handles: HashMap<String, AbortHandle> = HashMap::new();

        for task_id in tasks {
            let task_id_clone = task_id.clone();
            let executor_clone = Arc::clone(&executor);
            let mut task_context = context.enter_scope(&task_id);
            let exec_context = ExecutionContext::new().with_workflow_id(workflow_id);
            let timeout = self.config.task_timeout;

            let handle = join_set.spawn(async move {
                let result = tokio::time::timeout(
                    timeout,
                    executor_clone.execute(&task_id_clone, &mut task_context, &exec_context),
                )
                .await;

                match result {
                    Ok(Ok(output)) => (task_id_clone, Ok(output)),
                    Ok(Err(e)) => (task_id_clone, Err(e)),
                    Err(_) => (
                        task_id_clone,
                        Err(WorkflowError::execution(format!(
                            "任务超时 ({:?})",
                            timeout
                        ))),
                    ),
                }
            });

            task_handles.insert(task_id, handle);

            if join_set.len() >= self.config.max_concurrency {
                if let Some((task_id, task_result)) = join_set.join_next().await {
                    task_handles.remove(&task_id);
                    match task_result {
                        Ok((id, Ok(output))) => {
                            result.add_success(id, output);
                        }
                        Ok((id, Err(e))) => {
                            result.add_failure(id, e);
                            if self.config.cancel_on_failure {
                                self.cancel_remaining_tasks(&task_handles);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("任务加入失败: {}", e);
                            result.add_cancelled();
                        }
                    }
                }
            }
        }

        while let Some(task_result) = join_set.join_next().await {
            match task_result {
                Ok((task_id, Ok(output))) => {
                    result.add_success(task_id, output);
                }
                Ok((task_id, Err(e))) => {
                    result.add_failure(task_id, e);
                }
                Err(e) => {
                    warn!("任务加入失败: {}", e);
                    result.add_cancelled();
                }
            }
        }

        result.total_duration = start_time.elapsed();
        result
    }

    /// 取消剩余任务
    fn cancel_remaining_tasks(&self, handles: &HashMap<String, AbortHandle>) {
        if !self.config.enable_cancellation {
            return;
        }

        info!("取消 {} 个剩余任务", handles.len());
        for (task_id, handle) in handles {
            handle.abort();
            debug!("已取消任务: {}", task_id);
        }
    }

    /// 获取配置
    pub fn config(&self) -> &ParallelConfig {
        &self.config
    }
}

/// 工作流节点并行执行器
pub struct WorkflowParallelExecutor {
    config: ParallelConfig,
    join_set_executor: JoinSetExecutor<ComponentOutput>,
}

impl WorkflowParallelExecutor {
    /// 创建新的工作流并行执行器
    pub fn new(config: ParallelConfig) -> Self {
        let join_set_executor = JoinSetExecutor::new(config.clone());
        Self {
            config,
            join_set_executor,
        }
    }

    /// 使用默认配置创建执行器
    pub fn default_executor() -> Self {
        Self::new(ParallelConfig::default())
    }

    /// 执行节点并行
    pub async fn execute_nodes(
        &self,
        node_ids: Vec<String>,
        executor: Arc<BoxedExecutor>,
        registry: Arc<ComponentRegistry>,
        context: &mut DataContext,
        workflow_id: Uuid,
        tracker: Arc<ExecutionTracker>,
    ) -> ParallelResult<ComponentOutput> {
        let start_time = Instant::now();

        for node_id in &node_ids {
            tracker.mark_node_started(node_id).await;
        }

        let task_executor = Arc::new(ComponentTaskExecutor::new(executor, registry, workflow_id));
        let mut result = self
            .join_set_executor
            .execute_parallel(node_ids.clone(), task_executor, context, workflow_id)
            .await;

        for (task_id, output) in &result.successes {
            match output.status {
                ComponentStatus::Success => {
                    tracker.mark_node_completed(task_id, output.result.clone());
                }
                ComponentStatus::Failure(msg) => {
                    tracker.mark_node_failed(task_id, msg);
                    result.add_failure(task_id.clone(), WorkflowError::execution(msg.clone()));
                }
                ComponentStatus::Skip => {
                    tracker.mark_node_skipped(task_id);
                }
                ComponentStatus::Break | ComponentStatus::Continue => {
                    tracker.mark_node_completed(task_id, output.result.clone());
                }
            }
        }

        for (task_id, error) in &result.failures {
            tracker.mark_node_failed(task_id, error.to_string());
        }

        result.total_duration = start_time.elapsed();
        result
    }

    /// 带取消支持的执行
    pub async fn execute_with_cancellation(
        &self,
        node_ids: Vec<String>,
        executor: Arc<BoxedExecutor>,
        registry: Arc<ComponentRegistry>,
        context: &mut DataContext,
        workflow_id: Uuid,
        tracker: Arc<ExecutionTracker>,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> ParallelResult<ComponentOutput> {
        let start_time = Instant::now();
        let mut result = ParallelResult::new();

        for node_id in &node_ids {
            tracker.mark_node_started(node_id).await;
        }

        let mut join_set: JoinSet<(String, Result<ComponentOutput>)> = JoinSet::new();

        for node_id in node_ids {
            if cancellation_token.is_cancelled() {
                result.add_cancelled();
                continue;
            }

            let node_id_clone = node_id.clone();
            let executor_clone = Arc::clone(&executor);
            let registry_clone = Arc::clone(&registry);
            let mut node_context = context.enter_scope(&node_id);
            let exec_context = ExecutionContext::new().with_workflow_id(workflow_id);
            let token = cancellation_token.clone();
            let timeout = self.config.task_timeout;

            join_set.spawn(async move {
                tokio::select! {
                    result = tokio::time::timeout(
                        timeout,
                        async {
                            let component = registry_clone.get(&node_id_clone)?;
                            executor_clone.execute(component.as_ref(), &mut node_context, &exec_context).await
                        }
                    ) => {
                        match result {
                            Ok(Ok(output)) => (node_id_clone, Ok(output)),
                            Ok(Err(e)) => (node_id_clone, Err(e)),
                            Err(_) => (node_id_clone, Err(WorkflowError::execution("任务超时"))),
                        }
                    }
                    _ = token.cancelled() => {
                        (node_id_clone, Err(WorkflowError::ExecutionCancelled))
                    }
                }
            });
        }

        while let Some(task_result) = join_set.join_next().await {
            match task_result {
                Ok((task_id, Ok(output))) => {
                    match output.status {
                        ComponentStatus::Success => {
                            tracker.mark_node_completed(&task_id, output.result.clone());
                            result.add_success(task_id, output);
                        }
                        ComponentStatus::Failure(msg) => {
                            tracker.mark_node_failed(&task_id, &msg);
                            result.add_failure(task_id, WorkflowError::execution(msg));
                        }
                        ComponentStatus::Skip => {
                            tracker.mark_node_skipped(&task_id);
                            result.add_success(task_id, output);
                        }
                        ComponentStatus::Break | ComponentStatus::Continue => {
                            tracker.mark_node_completed(&task_id, output.result.clone());
                            result.add_success(task_id, output);
                        }
                    }
                }
                Ok((task_id, Err(e))) => {
                    if matches!(e, WorkflowError::ExecutionCancelled) {
                        result.add_cancelled();
                    } else {
                        tracker.mark_node_failed(&task_id, e.to_string());
                        result.add_failure(task_id, e);
                    }
                }
                Err(e) => {
                    warn!("任务加入失败: {}", e);
                    result.add_cancelled();
                }
            }
        }

        result.total_duration = start_time.elapsed();
        result
    }

    /// 获取配置
    pub fn config(&self) -> &ParallelConfig {
        &self.config
    }
}

/// 并行执行器构建器
pub struct ParallelExecutorBuilder {
    config: ParallelConfig,
}

impl ParallelExecutorBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: ParallelConfig::default(),
        }
    }

    /// 设置最大并发数
    pub fn max_concurrency(mut self, max: usize) -> Self {
        self.config.max_concurrency = max;
        self
    }

    /// 设置任务超时
    pub fn task_timeout(mut self, timeout: Duration) -> Self {
        self.config.task_timeout = timeout;
        self
    }

    /// 启用任务取消
    pub fn enable_cancellation(mut self, enable: bool) -> Self {
        self.config.enable_cancellation = enable;
        self
    }

    /// 设置失败时取消
    pub fn cancel_on_failure(mut self, cancel: bool) -> Self {
        self.config.cancel_on_failure = cancel;
        self
    }

    /// 设置收集所有结果
    pub fn collect_all_results(mut self, collect: bool) -> Self {
        self.config.collect_all_results = collect;
        self
    }

    /// 构建工作流并行执行器
    pub fn build_workflow_executor(self) -> WorkflowParallelExecutor {
        WorkflowParallelExecutor::new(self.config)
    }

    /// 构建 JoinSet 执行器
    pub fn build_join_set_executor<T: Send + 'static>(self) -> JoinSetExecutor<T> {
        JoinSetExecutor::new(self.config)
    }
}

impl Default for ParallelExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert_eq!(config.max_concurrency, 4);
        assert!(config.enable_cancellation);
    }

    #[test]
    fn test_parallel_result() {
        let mut result: ParallelResult<String> = ParallelResult::new();
        assert!(result.all_succeeded);
        assert_eq!(result.success_count(), 0);

        result.add_success("task1".to_string(), "output1".to_string());
        assert_eq!(result.success_count(), 1);

        result.add_failure("task2".to_string(), WorkflowError::execution("error"));
        assert!(!result.all_succeeded);
        assert_eq!(result.failure_count(), 1);
    }

    #[test]
    fn test_parallel_executor_builder() {
        let executor = ParallelExecutorBuilder::new()
            .max_concurrency(8)
            .task_timeout(Duration::from_secs(60))
            .cancel_on_failure(true)
            .build_workflow_executor();

        assert_eq!(executor.config().max_concurrency, 8);
        assert!(executor.config().cancel_on_failure);
    }

    #[tokio::test]
    async fn test_join_set_executor_creation() {
        let executor: JoinSetExecutor<String> = JoinSetExecutor::default_executor();
        assert_eq!(executor.config().max_concurrency, 4);
    }
}
