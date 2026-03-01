//! 组合工具执行器
//!
//! 负责协调组合工具的执行，包括链式、条件和并行执行

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::tools::{
    ComposedTool, CompositionType, ErrorPropagationStrategy, Tool, ToolId, ToolInput, ToolOutput,
    ToolRegistry,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;

/// 组合工具执行器
///
/// 负责协调组合工具的执行，包括链式、条件和并行执行
pub struct ComposedToolExecutor {
    /// 工具注册表（用于查找子工具）
    registry: Arc<ToolRegistry>,
}

impl ComposedToolExecutor {
    /// 创建新的组合工具执行器
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    /// 执行组合工具
    ///
    /// 根据 composition_type 自动分发到对应的执行方法
    pub async fn execute(
        &self,
        tool: &ComposedTool,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> Result<ToolOutput> {
        // 根据组合类型执行不同的逻辑
        match &tool.composition_type {
            CompositionType::Chain(tools) => {
                self.execute_chain(tool, input, ctx, tools).await
            }
            CompositionType::Conditional {
                condition,
                then_tool,
                else_tool,
            } => {
                self.execute_conditional(tool, input, ctx, condition, *then_tool, *else_tool)
                    .await
            }
            CompositionType::Parallel(tools) => {
                self.execute_parallel(tool, input, ctx, tools).await
            }
        }
    }

    /// 链式执行：A 的输出 → B 的输入 → C 的输入
    ///
    /// # 执行语义
    /// 1. 按顺序依次执行每个工具
    /// 2. 前一个工具的输出通过 `DataFlowMapping` 转换为下一个工具的输入
    /// 3. 任一步骤失败时，根据 `error_strategy` 处理
    /// 4. 返回所有步骤的执行结果和最终输出
    async fn execute_chain(
        &self,
        composed_tool: &ComposedTool,
        mut input: ToolInput,
        ctx: ExecutionContext,
        tools: &[ToolId],
    ) -> Result<ToolOutput> {
        let mut results = Vec::new();

        for (index, &tool_id) in tools.iter().enumerate() {
            // 获取工具
            let tool_impl = self.registry.get_by_id(tool_id).ok_or_else(|| {
                WorkflowError::tool_not_found(&format!("工具 ID: {}", tool_id))
            })?;

            // 执行工具（带重试逻辑）
            let output = self
                .execute_with_retry(&tool_impl, input.clone(), ctx.clone(), composed_tool)
                .await?;

            // 错误处理
            if !output.success {
                match &composed_tool.error_strategy {
                    ErrorPropagationStrategy::FailFast => {
                        error!("链式执行在第 {} 步失败", index + 1);
                        return Err(WorkflowError::execution(format!(
                            "链式执行在第 {} 步失败",
                            index + 1
                        )));
                    }
                    ErrorPropagationStrategy::ContinueOnError => {
                        // 记录错误，继续执行
                        error!("链式执行第 {} 步失败，但继续执行", index + 1);
                        results.push(output.clone());
                        continue;
                    }
                    ErrorPropagationStrategy::Retry { .. } => {
                        // 重试逻辑已在 execute_with_retry 中处理
                        results.push(output.clone());
                    }
                }
            } else {
                results.push(output.clone());
            }

            // 数据流转换（为下一个工具准备输入）
            // 如果不是最后一个工具，且有数据流映射
            if index < tools.len() - 1 {
                if let Some(mapping) = &composed_tool.data_flow {
                    input = ToolInput::new(mapping.transform(&output.result)?);
                }
            }
        }

        // 返回最终结果
        let final_result = results
            .last()
            .map(|r| r.result.clone())
            .unwrap_or(Value::Null);

        Ok(ToolOutput::success(serde_json::json!({
            "status": "chain_completed",
            "steps": results.len(),
            "successful_steps": results.iter().filter(|r| r.success).count(),
            "failed_steps": results.iter().filter(|r| !r.success).count(),
            "final_result": final_result,
            "all_results": results
        })))
    }

    /// 条件执行：根据条件选择分支
    ///
    /// # 执行语义
    /// 1. 使用 EL 表达式引擎评估条件表达式
    /// 2. 根据条件结果选择执行 `then_tool` 或 `else_tool`
    /// 3. 返回选中分支的执行结果
    async fn execute_conditional(
        &self,
        composed_tool: &ComposedTool,
        input: ToolInput,
        ctx: ExecutionContext,
        condition: &str,
        then_tool: ToolId,
        else_tool: Option<ToolId>,
    ) -> Result<ToolOutput> {
        use crate::workflow::el::{ExpressionContext, ExpressionEngine};

        // 创建 EL 上下文，将 input.params 作为变量源
        let mut el_context = ExpressionContext::new();

        // 将 input.params 的顶层字段添加到上下文中
        if let Some(obj) = input.params.as_object() {
            for (key, value) in obj {
                el_context.set(key, value.clone());
            }
        }

        // 评估条件
        let engine = ExpressionEngine::new();
        let condition_result = engine
            .evaluate_condition(condition, &el_context)
            .map_err(|e| {
                WorkflowError::validation(format!("条件表达式求值失败：{}", e))
            })?;

        // 选择分支
        let selected_tool = if condition_result {
            then_tool
        } else {
            else_tool.ok_or_else(|| {
                WorkflowError::validation("条件为假但未指定 else 分支")
            })?
        };

        // 执行选中的工具
        let tool_impl = self.registry.get_by_id(selected_tool).ok_or_else(|| {
            WorkflowError::tool_not_found(&format!("工具 ID: {}", selected_tool))
        })?;

        let output = self
            .execute_with_retry(&tool_impl, input.clone(), ctx.clone(), composed_tool)
            .await?;

        Ok(ToolOutput::success(serde_json::json!({
            "condition": condition,
            "condition_result": condition_result,
            "selected_branch": if selected_tool == then_tool {
                "then"
            } else {
                "else"
            },
            "success": output.success,
            "result": output.result
        })))
    }

    /// 并行执行：同时执行多个工具
    ///
    /// # 执行语义
    /// 1. 并发执行所有工具（受 `max_concurrency` 限制）
    /// 2. 所有工具共享相同的输入
    /// 3. 收集所有工具的执行结果
    /// 4. 根据错误策略处理失败
    async fn execute_parallel(
        &self,
        composed_tool: &ComposedTool,
        input: ToolInput,
        ctx: ExecutionContext,
        tools: &[ToolId],
    ) -> Result<ToolOutput> {
        use futures::stream::{self, StreamExt};
        use std::time::Instant;

        // 创建并发执行流
        let mut stream = stream::iter(tools.iter().map(|&tool_id| {
            let registry = &self.registry;
            let input = input.clone();
            let ctx = ctx.clone();

            async move {
                let tool_impl = registry.get_by_id(tool_id).ok_or_else(|| {
                    WorkflowError::tool_not_found(&format!("工具 ID: {}", tool_id))
                })?;

                let start = Instant::now();
                let output = tool_impl.execute(input, ctx).await?;
                let duration = start.elapsed();

                Ok::<_, WorkflowError>((tool_id, output, duration))
            }
        }))
        .buffer_unordered(composed_tool.max_concurrency);

        // 收集所有结果
        let mut results = Vec::new();
        let mut errors = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok((tool_id, output, duration)) => {
                    if output.success {
                        results.push(serde_json::json!({
                            "tool_id": format!("{}", tool_id),
                            "success": true,
                            "result": output.result,
                            "duration_ms": duration.as_millis() as u64
                        }));
                    } else {
                        errors.push(serde_json::json!({
                            "tool_id": format!("{}", tool_id),
                            "success": false,
                            "error": output.result
                        }));
                    }
                }
                Err(e) => {
                    errors.push(serde_json::json!({ "error": e.to_string() }));
                }
            }
        }

        // 错误处理策略
        if !errors.is_empty() {
            match &composed_tool.error_strategy {
                ErrorPropagationStrategy::FailFast => {
                    return Err(WorkflowError::execution(format!(
                        "并行执行中有 {} 个工具失败",
                        errors.len()
                    )));
                }
                ErrorPropagationStrategy::ContinueOnError
                | ErrorPropagationStrategy::Retry { .. } => {
                    // 部分成功也返回成功，包含失败信息
                }
            }
        }

        Ok(ToolOutput::success(serde_json::json!({
            "status": "parallel_completed",
            "total": tools.len(),
            "successful": results.len(),
            "failed": errors.len(),
            "results": results,
            "errors": if errors.is_empty() {
                None
            } else {
                Some(errors)
            }
        })))
    }

    /// 带重试的执行辅助方法
    async fn execute_with_retry(
        &self,
        tool: &Tool,
        input: ToolInput,
        ctx: ExecutionContext,
        composed_tool: &ComposedTool,
    ) -> Result<ToolOutput> {
        match &composed_tool.error_strategy {
            ErrorPropagationStrategy::Retry {
                max_retries,
                delay_ms,
            } => {
                let mut last_error = None;

                for attempt in 0..=*max_retries {
                    match tool.execute(input.clone(), ctx.clone()).await {
                        Ok(output) => {
                            if output.success {
                                return Ok(output);
                            }
                            // 业务逻辑失败，不重试
                            return Ok(output);
                        }
                        Err(e) => {
                            last_error = Some(e);
                            if attempt < *max_retries {
                                // 等待重试延迟
                                tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
                            }
                        }
                    }
                }

                Err(last_error.unwrap())
            }
            _ => {
                // 无重试，直接执行
                tool.execute(input, ctx).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let registry = Arc::new(ToolRegistry::new());
        let executor = ComposedToolExecutor::new(Arc::clone(&registry));
        assert!(Arc::ptr_eq(&executor.registry, &registry));
    }
}
