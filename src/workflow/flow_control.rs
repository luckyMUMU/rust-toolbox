//! 流程控制执行器
//!
//! 实现 Switch 和 Loop 节点的执行逻辑

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::workflow::context::DataContext;
use crate::workflow::el_expression::{ExpressionContext, ExpressionEngine};
use crate::workflow::flow_node::FlowNode;
use crate::workflow::state::ExecutionTracker;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// 最大循环迭代次数
const MAX_LOOP_ITERATIONS: u32 = 10000;

/// 流程控制执行器
pub struct FlowControlExecutor {
    /// 表达式引擎
    expr_engine: ExpressionEngine,
    /// 最大循环迭代次数
    max_iterations: u32,
}

impl FlowControlExecutor {
    /// 创建新的流程控制执行器
    pub fn new() -> Self {
        Self {
            expr_engine: ExpressionEngine::new(),
            max_iterations: MAX_LOOP_ITERATIONS,
        }
    }

    /// 使用自定义最大迭代次数创建执行器
    pub fn with_max_iterations(max_iterations: u32) -> Self {
        Self {
            expr_engine: ExpressionEngine::new(),
            max_iterations,
        }
    }

    /// 执行 Switch 节点
    ///
    /// 根据条件表达式选择执行分支
    pub async fn execute_switch(
        &self,
        condition: &str,
        cases: &[(String, FlowNode)],
        default: &FlowNode,
        context: &mut DataContext,
        tracker: Arc<ExecutionTracker>,
        execute_fn: impl Fn(&FlowNode, &mut DataContext, Arc<ExecutionTracker>) -> futures::future::BoxFuture<'_, Result<()>> + Send + Sync + 'static,
    ) -> Result<()> {
        info!(
            condition = %condition,
            case_count = cases.len(),
            "执行 Switch 节点"
        );

        let expr_context = self.build_expression_context(context);
        
        let condition_value = self.expr_engine.evaluate(condition, &expr_context)?;
        let condition_str = value_to_string(&condition_value);
        
        debug!(
            condition = %condition,
            evaluated_value = %condition_str,
            "Switch 条件评估结果"
        );

        let mut matched = false;
        for (case_value, case_node) in cases {
            if case_value == &condition_str || self.match_case(case_value, &condition_value) {
                info!(
                    matched_case = %case_value,
                    "Switch 匹配到分支"
                );
                execute_fn(case_node, context, tracker.clone()).await?;
                matched = true;
                break;
            }
        }

        if !matched {
            info!("Switch 未匹配任何分支，执行默认分支");
            execute_fn(default, context, tracker).await?;
        }

        Ok(())
    }

    /// 执行 Loop 节点
    ///
    /// 根据条件循环执行节点体
    pub async fn execute_loop(
        &self,
        condition: &str,
        body: &FlowNode,
        context: &mut DataContext,
        tracker: Arc<ExecutionTracker>,
        execute_fn: impl Fn(&FlowNode, &mut DataContext, Arc<ExecutionTracker>) -> futures::future::BoxFuture<'_, Result<()>> + Send + Sync + Clone + 'static,
    ) -> Result<()> {
        info!(
            condition = %condition,
            max_iterations = self.max_iterations,
            "执行 Loop 节点"
        );

        let mut iteration = 0u32;
        
        loop {
            if iteration >= self.max_iterations {
                warn!(
                    iterations = iteration,
                    max = self.max_iterations,
                    "Loop 达到最大迭代次数限制"
                );
                return Err(WorkflowError::execution(format!(
                    "循环超过最大迭代次数限制: {}",
                    self.max_iterations
                )));
            }

            let expr_context = self.build_expression_context(context);
            let should_continue = self.expr_engine.evaluate_condition(condition, &expr_context)?;

            if !should_continue {
                info!(
                    iterations = iteration,
                    "Loop 条件不满足，退出循环"
                );
                break;
            }

            debug!(
                iteration = iteration,
                condition = %condition,
                "Loop 迭代执行"
            );

            execute_fn(body, context, tracker.clone()).await?;
            
            iteration += 1;
        }

        info!(
            total_iterations = iteration,
            "Loop 执行完成"
        );

        Ok(())
    }

    /// 执行 ForEach 循环
    ///
    /// 遍历数组执行节点体
    pub async fn execute_foreach(
        &self,
        items_expr: &str,
        item_var: &str,
        index_var: Option<&str>,
        body: &FlowNode,
        context: &mut DataContext,
        tracker: Arc<ExecutionTracker>,
        execute_fn: impl Fn(&FlowNode, &mut DataContext, Arc<ExecutionTracker>) -> futures::future::BoxFuture<'_, Result<()>> + Send + Sync + Clone + 'static,
    ) -> Result<()> {
        info!(
            items_expr = %items_expr,
            item_var = %item_var,
            "执行 ForEach 循环"
        );

        let expr_context = self.build_expression_context(context);
        let items = self.expr_engine.evaluate(items_expr, &expr_context)?;

        let items_array = match items {
            Value::Array(arr) => arr,
            _ => {
                return Err(WorkflowError::execution(format!(
                    "ForEach items 表达式必须返回数组，实际返回: {:?}",
                    items
                )));
            }
        };

        info!(
            item_count = items_array.len(),
            "ForEach 开始遍历"
        );

        for (index, item) in items_array.into_iter().enumerate() {
            if index as u32 >= self.max_iterations {
                warn!(
                    index = index,
                    max = self.max_iterations,
                    "ForEach 达到最大迭代次数限制"
                );
                return Err(WorkflowError::execution(format!(
                    "循环超过最大迭代次数限制: {}",
                    self.max_iterations
                )));
            }

            context.set(item_var, item.clone());
            
            if let Some(idx_var) = index_var {
                context.set(idx_var, Value::Number(index as i64.into()));
            }

            debug!(
                index = index,
                item = ?item,
                "ForEach 迭代执行"
            );

            execute_fn(body, context, tracker.clone()).await?;
        }

        info!(
            total_iterations = items_array.len(),
            "ForEach 执行完成"
        );

        Ok(())
    }

    /// 执行 While 循环
    ///
    /// 当条件为真时持续执行
    pub async fn execute_while(
        &self,
        condition: &str,
        body: &FlowNode,
        context: &mut DataContext,
        tracker: Arc<ExecutionTracker>,
        execute_fn: impl Fn(&FlowNode, &mut DataContext, Arc<ExecutionTracker>) -> futures::future::BoxFuture<'_, Result<()>> + Send + Sync + Clone + 'static,
    ) -> Result<()> {
        self.execute_loop(condition, body, context, tracker, execute_fn).await
    }

    /// 构建表达式上下文
    fn build_expression_context(&self, data_context: &DataContext) -> ExpressionContext {
        let mut expr_context = ExpressionContext::new();
        
        for (key, value) in data_context.all_variables() {
            expr_context.set(&key, value.clone());
        }
        
        expr_context
    }

    /// 匹配 case 值
    fn match_case(&self, case_pattern: &str, value: &Value) -> bool {
        if case_pattern == "*" {
            return true;
        }
        
        let case_value = match serde_json::from_str::<Value>(case_pattern) {
            Ok(v) => v,
            Err(_) => Value::String(case_pattern.to_string()),
        };
        
        &case_value == value
    }
}

impl Default for FlowControlExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 值转字符串
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => serde_json::to_string(arr).unwrap_or_default(),
        Value::Object(obj) => serde_json::to_string(obj).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_control_executor_creation() {
        let executor = FlowControlExecutor::new();
        assert_eq!(executor.max_iterations, MAX_LOOP_ITERATIONS);
    }

    #[test]
    fn test_flow_control_executor_with_custom_iterations() {
        let executor = FlowControlExecutor::with_max_iterations(100);
        assert_eq!(executor.max_iterations, 100);
    }

    #[test]
    fn test_match_case() {
        let executor = FlowControlExecutor::new();
        
        assert!(executor.match_case("*", &Value::String("anything".to_string())));
        assert!(executor.match_case("test", &Value::String("test".to_string())));
        assert!(!executor.match_case("test", &Value::String("other".to_string())));
        assert!(executor.match_case("42", &Value::Number(42.into())));
    }
}
