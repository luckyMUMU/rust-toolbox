//! 工具组合执行模块
//!
//! 提供链式、并行、条件等组合执行策略和结果合并机制

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::tools::types::{Tool, ToolInput};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, warn};

/// 组合执行策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionStrategy {
    /// 链式执行：前一个工具的输出作为下一个工具的输入
    Chain {
        /// 是否传递完整输出
        pass_full_output: bool,
        /// 输出字段映射
        field_mapping: HashMap<String, String>,
    },
    /// 并行执行：同时执行多个工具
    Parallel {
        /// 最大并发数
        max_concurrency: usize,
        /// 失败策略
        failure_strategy: FailureStrategy,
    },
    /// 条件执行：根据条件选择执行路径
    Conditional {
        /// 条件表达式
        condition: String,
        /// 默认分支
        default_branch: Option<String>,
    },
    /// 循环执行：重复执行直到条件满足
    Loop {
        /// 最大迭代次数
        max_iterations: u32,
        /// 退出条件
        exit_condition: String,
        /// 延迟时间（毫秒）
        delay_ms: u64,
    },
    /// 重试执行：失败时重试
    Retry {
        /// 最大重试次数
        max_retries: u32,
        /// 重试延迟（毫秒）
        retry_delay_ms: u64,
        /// 指数退避因子
        backoff_factor: f64,
    },
}

impl Default for CompositionStrategy {
    fn default() -> Self {
        Self::Chain {
            pass_full_output: true,
            field_mapping: HashMap::new(),
        }
    }
}

/// 失败处理策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FailureStrategy {
    /// 任一失败即停止
    FailFast,
    /// 继续执行，收集所有结果
    ContinueOnFailure,
    /// 忽略失败，只返回成功结果
    IgnoreFailures,
}

impl Default for FailureStrategy {
    fn default() -> Self {
        Self::FailFast
    }
}

/// 结果合并策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultMergeStrategy {
    /// 合并为对象，每个工具结果作为独立字段
    MergeObject {
        /// 字段名映射：工具名 -> 结果字段名
        field_names: HashMap<String, String>,
    },
    /// 合并为数组
    MergeArray {
        /// 是否包含工具名
        include_tool_names: bool,
    },
    /// 取第一个成功结果
    FirstSuccess,
    /// 取最后一个结果
    LastResult,
    /// 自定义合并函数名
    Custom {
        function_name: String,
    },
}

impl Default for ResultMergeStrategy {
    fn default() -> Self {
        Self::MergeObject {
            field_names: HashMap::new(),
        }
    }
}

/// 组合执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionConfig {
    /// 执行策略
    pub strategy: CompositionStrategy,
    /// 结果合并策略
    pub merge_strategy: ResultMergeStrategy,
    /// 总超时时间
    pub timeout: Duration,
    /// 是否启用详细日志
    pub verbose_logging: bool,
}

impl Default for CompositionConfig {
    fn default() -> Self {
        Self {
            strategy: CompositionStrategy::default(),
            merge_strategy: ResultMergeStrategy::default(),
            timeout: Duration::from_secs(300),
            verbose_logging: false,
        }
    }
}

/// 组合执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionResult {
    /// 最终输出
    pub output: Value,
    /// 各工具执行结果
    pub tool_results: HashMap<String, ToolExecutionRecord>,
    /// 总执行时间
    pub total_duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如有）
    pub error: Option<String>,
}

/// 工具执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRecord {
    /// 工具名称
    pub tool_name: String,
    /// 执行顺序
    pub order: u32,
    /// 输入参数
    pub input: Value,
    /// 输出结果
    pub output: Option<Value>,
    /// 执行时间
    pub duration: Duration,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 重试次数
    pub retry_count: u32,
}

/// 组合执行器
pub struct CompositionExecutor {
    config: CompositionConfig,
}

impl CompositionExecutor {
    /// 创建新的组合执行器
    pub fn new(config: CompositionConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建执行器
    pub fn default_executor() -> Self {
        Self::new(CompositionConfig::default())
    }

    /// 执行链式组合
    pub async fn execute_chain(
        &self,
        tools: Vec<(String, Arc<Tool>)>,
        initial_input: ToolInput,
        context: ExecutionContext,
    ) -> Result<CompositionResult> {
        let start_time = Instant::now();
        let mut tool_results: HashMap<String, ToolExecutionRecord> = HashMap::new();
        let mut current_input = initial_input;
        let mut order = 0u32;

        let (pass_full_output, field_mapping) = match &self.config.strategy {
            CompositionStrategy::Chain { pass_full_output, field_mapping } => {
                (*pass_full_output, field_mapping.clone())
            }
            _ => (true, HashMap::new()),
        };

        for (tool_name, tool) in tools {
            order += 1;
            let tool_start = Instant::now();
            
            if self.config.verbose_logging {
                debug!("执行链式工具 [{}]: {}", order, tool_name);
            }

            let input_snapshot = current_input.params.clone();
            
            match tool.execute(current_input.clone(), context.clone()).await {
                Ok(output) => {
                    let duration = tool_start.elapsed();
                    
                    tool_results.insert(tool_name.clone(), ToolExecutionRecord {
                        tool_name: tool_name.clone(),
                        order,
                        input: input_snapshot,
                        output: Some(output.result.clone()),
                        duration,
                        success: output.success,
                        error: if !output.success { Some("工具执行失败".to_string()) } else { None },
                        retry_count: 0,
                    });

                    if output.success {
                        current_input = if pass_full_output {
                            ToolInput::new(output.result)
                        } else {
                            self.apply_field_mapping(&output.result, &field_mapping)
                        };
                    } else {
                        return Ok(CompositionResult {
                            output: output.result,
                            tool_results,
                            total_duration: start_time.elapsed(),
                            success: false,
                            error: Some(format!("工具 {} 执行失败", tool_name)),
                        });
                    }
                }
                Err(e) => {
                    let duration = tool_start.elapsed();
                    tool_results.insert(tool_name.clone(), ToolExecutionRecord {
                        tool_name: tool_name.clone(),
                        order,
                        input: input_snapshot,
                        output: None,
                        duration,
                        success: false,
                        error: Some(e.to_string()),
                        retry_count: 0,
                    });

                    return Ok(CompositionResult {
                        output: Value::Null,
                        tool_results,
                        total_duration: start_time.elapsed(),
                        success: false,
                        error: Some(format!("工具 {} 执行错误: {}", tool_name, e)),
                    });
                }
            }
        }

        let final_output = current_input.params;
        Ok(CompositionResult {
            output: final_output,
            tool_results,
            total_duration: start_time.elapsed(),
            success: true,
            error: None,
        })
    }

    /// 执行并行组合
    pub async fn execute_parallel(
        &self,
        tools: Vec<(String, Arc<Tool>)>,
        input: ToolInput,
        context: ExecutionContext,
    ) -> Result<CompositionResult> {
        let start_time = Instant::now();
        let (max_concurrency, failure_strategy) = match &self.config.strategy {
            CompositionStrategy::Parallel { max_concurrency, failure_strategy } => {
                (*max_concurrency, *failure_strategy)
            }
            _ => (4, FailureStrategy::FailFast),
        };

        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let tool_results = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let mut handles = Vec::new();

        for (idx, (tool_name, tool)) in tools.into_iter().enumerate() {
            let semaphore = Arc::clone(&semaphore);
            let tool_results = Arc::clone(&tool_results);
            let input = input.clone();
            let context = context.clone();
            let order = (idx + 1) as u32;

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let tool_start = Instant::now();
                let input_snapshot = input.params.clone();

                let result = tool.execute(input, context).await;
                let duration = tool_start.elapsed();

                let record = match result {
                    Ok(output) => ToolExecutionRecord {
                        tool_name: tool_name.clone(),
                        order,
                        input: input_snapshot,
                        output: Some(output.result.clone()),
                        duration,
                        success: output.success,
                        error: if !output.success { Some("工具执行失败".to_string()) } else { None },
                        retry_count: 0,
                    },
                    Err(e) => ToolExecutionRecord {
                        tool_name: tool_name.clone(),
                        order,
                        input: input_snapshot,
                        output: None,
                        duration,
                        success: false,
                        error: Some(e.to_string()),
                        retry_count: 0,
                    },
                };

                (tool_name, record)
            });

            handles.push(handle);
        }

        let mut success_count = 0;
        let mut failure_count = 0;

        for handle in handles {
            if let Ok((tool_name, record)) = handle.await {
                if record.success {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
                tool_results.lock().await.insert(tool_name, record);
            }
        }

        let results = tool_results.lock().await.clone();
        let success = match failure_strategy {
            FailureStrategy::FailFast => failure_count == 0,
            FailureStrategy::ContinueOnFailure => success_count > 0,
            FailureStrategy::IgnoreFailures => true,
        };

        let output = self.merge_results(&results)?;

        Ok(CompositionResult {
            output,
            tool_results: results,
            total_duration: start_time.elapsed(),
            success,
            error: if !success { Some("部分工具执行失败".to_string()) } else { None },
        })
    }

    /// 执行条件组合
    pub async fn execute_conditional(
        &self,
        branches: HashMap<String, Arc<Tool>>,
        condition_evaluator: impl Fn(&str, &Value) -> bool,
        input: ToolInput,
        context: ExecutionContext,
    ) -> Result<CompositionResult> {
        let start_time = Instant::now();
        let mut tool_results: HashMap<String, ToolExecutionRecord> = HashMap::new();

        let (condition_expr, default_branch) = match &self.config.strategy {
            CompositionStrategy::Conditional { condition, default_branch } => {
                (condition.clone(), default_branch.clone())
            }
            _ => (String::new(), None),
        };

        let selected_branch = if condition_evaluator(&condition_expr, &input.params) {
            "then"
        } else {
            default_branch.as_deref().unwrap_or("else")
        };

        if let Some(tool) = branches.get(selected_branch) {
            let tool_start = Instant::now();
            let input_snapshot = input.params.clone();
            let tool_name = format!("{}_branch", selected_branch);

            match tool.execute(input, context).await {
                Ok(output) => {
                    tool_results.insert(tool_name.clone(), ToolExecutionRecord {
                        tool_name,
                        order: 1,
                        input: input_snapshot,
                        output: Some(output.result.clone()),
                        duration: tool_start.elapsed(),
                        success: output.success,
                        error: None,
                        retry_count: 0,
                    });

                    Ok(CompositionResult {
                        output: output.result,
                        tool_results,
                        total_duration: start_time.elapsed(),
                        success: output.success,
                        error: None,
                    })
                }
                Err(e) => {
                    tool_results.insert(tool_name.clone(), ToolExecutionRecord {
                        tool_name,
                        order: 1,
                        input: input_snapshot,
                        output: None,
                        duration: tool_start.elapsed(),
                        success: false,
                        error: Some(e.to_string()),
                        retry_count: 0,
                    });

                    Ok(CompositionResult {
                        output: Value::Null,
                        tool_results,
                        total_duration: start_time.elapsed(),
                        success: false,
                        error: Some(format!("条件分支执行失败: {}", e)),
                    })
                }
            }
        } else {
            Err(WorkflowError::validation(format!("未找到条件分支: {}", selected_branch)))
        }
    }

    /// 执行重试组合
    pub async fn execute_retry(
        &self,
        tool: Arc<Tool>,
        tool_name: &str,
        input: ToolInput,
        context: ExecutionContext,
    ) -> Result<CompositionResult> {
        let start_time = Instant::now();
        let mut tool_results: HashMap<String, ToolExecutionRecord> = HashMap::new();

        let (max_retries, retry_delay_ms, backoff_factor) = match &self.config.strategy {
            CompositionStrategy::Retry { max_retries, retry_delay_ms, backoff_factor } => {
                (*max_retries, *retry_delay_ms, *backoff_factor)
            }
            _ => (3, 1000, 2.0),
        };

        let mut current_delay = retry_delay_ms;
        let mut retry_count = 0u32;
        let input_snapshot = input.params.clone();

        loop {
            let tool_start = Instant::now();

            match tool.execute(input.clone(), context.clone()).await {
                Ok(output) if output.success => {
                    tool_results.insert(tool_name.to_string(), ToolExecutionRecord {
                        tool_name: tool_name.to_string(),
                        order: 1,
                        input: input_snapshot,
                        output: Some(output.result.clone()),
                        duration: tool_start.elapsed(),
                        success: true,
                        error: None,
                        retry_count,
                    });

                    return Ok(CompositionResult {
                        output: output.result,
                        tool_results,
                        total_duration: start_time.elapsed(),
                        success: true,
                        error: None,
                    });
                }
                Ok(output) => {
                    warn!("工具 {} 执行失败，重试 {}/{}", tool_name, retry_count, max_retries);
                    if retry_count >= max_retries {
                        tool_results.insert(tool_name.to_string(), ToolExecutionRecord {
                            tool_name: tool_name.to_string(),
                            order: 1,
                            input: input_snapshot,
                            output: Some(output.result),
                            duration: tool_start.elapsed(),
                            success: false,
                            error: Some("达到最大重试次数".to_string()),
                            retry_count,
                        });

                        return Ok(CompositionResult {
                            output: Value::Null,
                            tool_results,
                            total_duration: start_time.elapsed(),
                            success: false,
                            error: Some("达到最大重试次数".to_string()),
                        });
                    }
                }
                Err(e) => {
                    warn!("工具 {} 执行错误: {}，重试 {}/{}", tool_name, e, retry_count, max_retries);
                    if retry_count >= max_retries {
                        tool_results.insert(tool_name.to_string(), ToolExecutionRecord {
                            tool_name: tool_name.to_string(),
                            order: 1,
                            input: input_snapshot,
                            output: None,
                            duration: tool_start.elapsed(),
                            success: false,
                            error: Some(e.to_string()),
                            retry_count,
                        });

                        return Ok(CompositionResult {
                            output: Value::Null,
                            tool_results,
                            total_duration: start_time.elapsed(),
                            success: false,
                            error: Some(format!("达到最大重试次数: {}", e)),
                        });
                    }
                }
            }

            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(current_delay)).await;
            current_delay = (current_delay as f64 * backoff_factor) as u64;
        }
    }

    /// 应用字段映射
    fn apply_field_mapping(&self, output: &Value, mapping: &HashMap<String, String>) -> ToolInput {
        if mapping.is_empty() {
            return ToolInput::new(output.clone());
        }

        let mut new_params = serde_json::Map::new();
        
        if let Some(obj) = output.as_object() {
            for (key, value) in obj {
                let new_key = mapping.get(key).cloned().unwrap_or_else(|| key.clone());
                new_params.insert(new_key, value.clone());
            }
        }

        ToolInput::new(Value::Object(new_params))
    }

    /// 合并结果
    fn merge_results(&self, results: &HashMap<String, ToolExecutionRecord>) -> Result<Value> {
        match &self.config.merge_strategy {
            ResultMergeStrategy::MergeObject { field_names } => {
                let mut merged = serde_json::Map::new();
                for (tool_name, record) in results {
                    if let Some(output) = &record.output {
                        let field_name = field_names.get(tool_name).cloned().unwrap_or_else(|| tool_name.clone());
                        merged.insert(field_name, output.clone());
                    }
                }
                Ok(Value::Object(merged))
            }
            ResultMergeStrategy::MergeArray { include_tool_names } => {
                let mut arr = Vec::new();
                for (tool_name, record) in results {
                    if let Some(output) = &record.output {
                        if *include_tool_names {
                            arr.push(serde_json::json!({
                                "tool": tool_name,
                                "result": output
                            }));
                        } else {
                            arr.push(output.clone());
                        }
                    }
                }
                Ok(Value::Array(arr))
            }
            ResultMergeStrategy::FirstSuccess => {
                let mut sorted: Vec<_> = results.iter().collect();
                sorted.sort_by_key(|(_, r)| r.order);
                
                for (_, record) in sorted {
                    if record.success {
                        if let Some(output) = &record.output {
                            return Ok(output.clone());
                        }
                    }
                }
                Ok(Value::Null)
            }
            ResultMergeStrategy::LastResult => {
                let mut sorted: Vec<_> = results.iter().collect();
                sorted.sort_by_key(|(_, r)| r.order);
                
                if let Some((_, record)) = sorted.last() {
                    if let Some(output) = &record.output {
                        return Ok(output.clone());
                    }
                }
                Ok(Value::Null)
            }
            ResultMergeStrategy::Custom { function_name } => {
                Err(WorkflowError::validation(format!(
                    "自定义合并函数 '{}' 尚未实现",
                    function_name
                )))
            }
        }
    }

    /// 获取配置
    pub fn config(&self) -> &CompositionConfig {
        &self.config
    }
}

/// 组合执行器构建器
pub struct CompositionExecutorBuilder {
    config: CompositionConfig,
}

impl CompositionExecutorBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: CompositionConfig::default(),
        }
    }

    /// 设置执行策略
    pub fn strategy(mut self, strategy: CompositionStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    /// 设置结果合并策略
    pub fn merge_strategy(mut self, strategy: ResultMergeStrategy) -> Self {
        self.config.merge_strategy = strategy;
        self
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// 启用详细日志
    pub fn verbose(mut self, enable: bool) -> Self {
        self.config.verbose_logging = enable;
        self
    }

    /// 构建执行器
    pub fn build(self) -> CompositionExecutor {
        CompositionExecutor::new(self.config)
    }
}

impl Default for CompositionExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_config_default() {
        let config = CompositionConfig::default();
        assert!(matches!(config.strategy, CompositionStrategy::Chain { .. }));
        assert!(matches!(config.merge_strategy, ResultMergeStrategy::MergeObject { .. }));
    }

    #[test]
    fn test_composition_executor_builder() {
        let executor = CompositionExecutorBuilder::new()
            .strategy(CompositionStrategy::Parallel {
                max_concurrency: 8,
                failure_strategy: FailureStrategy::ContinueOnFailure,
            })
            .merge_strategy(ResultMergeStrategy::MergeArray { include_tool_names: true })
            .timeout(Duration::from_secs(60))
            .verbose(true)
            .build();

        assert!(matches!(executor.config().strategy, CompositionStrategy::Parallel { .. }));
    }

    #[test]
    fn test_failure_strategy_default() {
        let strategy = FailureStrategy::default();
        assert!(matches!(strategy, FailureStrategy::FailFast));
    }

    #[test]
    fn test_result_merge_strategy_default() {
        let strategy = ResultMergeStrategy::default();
        assert!(matches!(strategy, ResultMergeStrategy::MergeObject { .. }));
    }
}
