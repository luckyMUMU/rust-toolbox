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
    use crate::tools::{DataFlowMapping, NativeTool, ToolMetadata, Version};
    use crate::core::ToolInfo;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

    /// 创建测试用的简单工具
    fn create_test_tool(name: &str) -> (ToolId, Tool) {
        let tool_id = ToolId::new();
        let metadata = Arc::new(ToolMetadata {
            info: ToolInfo {
                name: name.to_string(),
                description: format!("测试工具 {}", name),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "number"}
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "number"}
                    }
                }),
                examples: vec![],
            },
            kind: crate::tools::ToolKind::Native,
            version: Version::new(1, 0, 0),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["test".to_string()],
            author: "test".to_string(),
            license: "MIT".to_string(),
        });

        let tool = NativeTool::new(
            tool_id,
            metadata,
            move |input, _ctx| {
                async move {
                    let value = input.params["input"].as_number().and_then(|n| n.as_f64()).unwrap_or(0.0);
                    Ok(ToolOutput::success(json!({
                        "result": value * 2.0,
                        "tool": name
                    })))
                }
                .boxed()
            },
        );

        (tool_id, Tool::Native(Arc::new(tool)))
    }

    #[test]
    fn test_executor_creation() {
        let registry = Arc::new(ToolRegistry::new());
        let executor = ComposedToolExecutor::new(Arc::clone(&registry));

        // 验证执行器创建成功
        assert!(Arc::ptr_eq(&executor.registry, &registry));
    }

    #[tokio::test]
    async fn test_chain_execution() {
        let registry = Arc::new(ToolRegistry::new());
        
        // 创建两个测试工具
        let (tool1_id, tool1) = create_test_tool("tool1");
        let (tool2_id, tool2) = create_test_tool("tool2");
        
        registry.register(tool1);
        registry.register(tool2);

        // 创建链式组合工具
        let composed_tool = ComposedTool {
            id: ToolId::new(),
            metadata: Arc::new(ToolMetadata {
                info: ToolInfo {
                    name: "chain_test".to_string(),
                    description: "测试链式执行".to_string(),
                    input_schema: json!({"type": "object"}),
                    output_schema: json!({"type": "object"}),
                    examples: vec![],
                },
                kind: crate::tools::ToolKind::Composed,
                version: Version::new(1, 0, 0),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                tags: vec![],
                author: "test".to_string(),
                license: "MIT".to_string(),
            }),
            composition_type: CompositionType::Chain(vec![tool1_id, tool2_id]),
            data_flow: None, // 简单测试，不使用数据流映射
            error_strategy: ErrorPropagationStrategy::FailFast,
            max_concurrency: 1,
        };

        let executor = ComposedToolExecutor::new(Arc::clone(&registry));
        let input = ToolInput::new(json!({"input": 5.0}));
        let ctx = ExecutionContext::default();

        let result = executor.execute(&composed_tool, input, ctx).await;
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
    }

    #[test]
    fn test_data_flow_mapping() {
        let mut mapping = DataFlowMapping::new();
        mapping.add_mapping("/result/value", "/params/input");
        mapping.add_mapping("/result/status", "/params/status");

        let output = json!({
            "result": {
                "value": 42,
                "status": "success"
            }
        });

        let input = mapping.transform(&output).unwrap();

        assert_eq!(input["params"]["input"], 42);
        assert_eq!(input["params"]["status"], "success");
    }

    #[test]
    fn test_data_flow_mapping_nested() {
        let mut mapping = DataFlowMapping::new();
        mapping.add_mapping("/data/user/name", "/user/name");

        let output = json!({
            "data": {
                "user": {
                    "name": "Alice",
                    "age": 30
                }
            }
        });

        let input = mapping.transform(&output).unwrap();

        assert_eq!(input["user"]["name"], "Alice");
    }

    #[test]
    fn test_data_flow_mapping_empty() {
        let mapping = DataFlowMapping::new();
        let output = json!({"result": 42});

        let input = mapping.transform(&output).unwrap();

        assert!(input.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_error_strategy_fail_fast() {
        let strategy = ErrorPropagationStrategy::FailFast;
        
        match strategy {
            ErrorPropagationStrategy::FailFast => {
                // 预期分支
                assert!(true);
            }
            _ => panic!("错误的错误策略"),
        }
    }

    #[test]
    fn test_error_strategy_continue_on_error() {
        let strategy = ErrorPropagationStrategy::ContinueOnError;
        
        match strategy {
            ErrorPropagationStrategy::ContinueOnError => {
                // 预期分支
                assert!(true);
            }
            _ => panic!("错误的错误策略"),
        }
    }

    #[test]
    fn test_error_strategy_retry() {
        let strategy = ErrorPropagationStrategy::Retry {
            max_retries: 3,
            delay_ms: 100,
        };
        
        match strategy {
            ErrorPropagationStrategy::Retry { max_retries, delay_ms } => {
                assert_eq!(max_retries, 3);
                assert_eq!(delay_ms, 100);
            }
            _ => panic!("错误的错误策略"),
        }
    }

    #[test]
    fn test_composition_type_chain() {
        let tool_ids = vec![ToolId::new(), ToolId::new()];
        let composition_type = CompositionType::Chain(tool_ids.clone());
        
        match composition_type {
            CompositionType::Chain(ids) => {
                assert_eq!(ids.len(), 2);
            }
            _ => panic!("期望 Chain 类型"),
        }
    }

    #[test]
    fn test_composition_type_conditional() {
        let then_tool = ToolId::new();
        let else_tool = Some(ToolId::new());
        let composition_type = CompositionType::Conditional {
            condition: "x > 10".to_string(),
            then_tool,
            else_tool,
        };
        
        match composition_type {
            CompositionType::Conditional { condition, .. } => {
                assert_eq!(condition, "x > 10");
            }
            _ => panic!("期望 Conditional 类型"),
        }
    }

    #[test]
    fn test_composition_type_parallel() {
        let tool_ids = vec![ToolId::new(), ToolId::new(), ToolId::new()];
        let composition_type = CompositionType::Parallel(tool_ids.clone());
        
        match composition_type {
            CompositionType::Parallel(ids) => {
                assert_eq!(ids.len(), 3);
            }
            _ => panic!("期望 Parallel 类型"),
        }
    }
}
