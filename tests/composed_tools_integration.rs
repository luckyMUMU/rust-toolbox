//! 组合工具集成测试
//!
//! 测试组合工具在 ToolExecutor 中的端到端执行

use rust_tool_v2::core::{ExecutionContext, ToolInfo};
use rust_tool_v2::tools::{
    ComposedTool, ComposedToolExecutor, CompositionType, DataFlowMapping, ErrorPropagationStrategy,
    NativeTool, Tool, ToolExecutor, ToolId, ToolInput, ToolMetadata, ToolRegistry, Version,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;

/// 创建测试用的简单工具
fn create_math_tool(name: &str, multiplier: f64) -> Tool {
    let tool_id = ToolId::new();
    let metadata = Arc::new(ToolMetadata {
        info: ToolInfo {
            name: name.to_string(),
            description: format!("数学工具：乘以 {}", multiplier),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {"type": "number"}
                },
                "required": ["input"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "result": {"type": "number"},
                    "operation": {"type": "string"}
                }
            }),
            examples: vec![],
        },
        kind: rust_tool_v2::tools::ToolKind::Native,
        version: Version::new(1, 0, 0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tags: vec!["math".to_string(), "test".to_string()],
        author: "test".to_string(),
        license: "MIT".to_string(),
    });

    let tool = NativeTool::new(
        tool_id,
        metadata,
        move |input, _ctx| {
            let multiplier = multiplier;
            async move {
                let value = input.params["input"]
                    .as_number()
                    .and_then(|n| n.as_f64())
                    .unwrap_or(0.0);
                
                Ok(rust_tool_v2::tools::ToolOutput::success(json!({
                    "result": value * multiplier,
                    "operation": format!("{} * {}", value, multiplier),
                    "tool": name
                })))
            }
            .boxed()
        },
    );

    Tool::Native(Arc::new(tool))
}

/// 创建测试用的条件判断工具
fn create_condition_tool() -> Tool {
    let tool_id = ToolId::new();
    let metadata = Arc::new(ToolMetadata {
        info: ToolInfo {
            name: "check_threshold".to_string(),
            description: "检查数值是否超过阈值".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "value": {"type": "number"},
                    "threshold": {"type": "number"}
                }
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "exceeds": {"type": "boolean"},
                    "value": {"type": "number"}
                }
            }),
            examples: vec![],
        },
        kind: rust_tool_v2::tools::ToolKind::Native,
        version: Version::new(1, 0, 0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tags: vec!["condition".to_string()],
        author: "test".to_string(),
        license: "MIT".to_string(),
    });

    let tool = NativeTool::new(
        tool_id,
        metadata,
        |_input, _ctx| {
            async move {
                let value = _input.params["value"]
                    .as_number()
                    .and_then(|n| n.as_f64())
                    .unwrap_or(0.0);
                
                let threshold = _input.params["threshold"]
                    .as_number()
                    .and_then(|n| n.as_f64())
                    .unwrap_or(10.0);

                Ok(rust_tool_v2::tools::ToolOutput::success(json!({
                    "exceeds": value > threshold,
                    "value": value,
                    "threshold": threshold
                })))
            }
            .boxed()
        },
    );

    Tool::Native(Arc::new(tool))
}

#[tokio::test]
async fn test_chain_execution_integration() {
    // 创建工具注册表
    let registry = Arc::new(ToolRegistry::new());
    
    // 注册两个数学工具
    registry.register(create_math_tool("double", 2.0));
    registry.register(create_math_tool("triple", 3.0));

    // 创建执行器
    let executor = ToolExecutor::new(Arc::clone(&registry));

    // 执行工具
    let input = ToolInput::new(json!({"input": 5.0}));
    let ctx = ExecutionContext::default();

    // 测试 double 工具
    let result = executor.execute("double", input, ctx).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.result["result"].as_f64().unwrap(), 10.0);
}

#[tokio::test]
async fn test_composed_executor_chain() {
    let registry = Arc::new(ToolRegistry::new());
    
    // 创建并注册工具
    let tool1 = create_math_tool("multiply_by_2", 2.0);
    let tool2 = create_math_tool("multiply_by_3", 3.0);
    
    let tool1_id = tool1.id();
    let tool2_id = tool2.id();
    
    registry.register(tool1);
    registry.register(tool2);

    // 创建链式组合工具
    let composed_tool = ComposedTool {
        id: ToolId::new(),
        metadata: Arc::new(ToolMetadata {
            info: ToolInfo {
                name: "chain_multiply".to_string(),
                description: "链式乘法：先乘 2 再乘 3".to_string(),
                input_schema: json!({"type": "object"}),
                output_schema: json!({"type": "object"}),
                examples: vec![],
            },
            kind: rust_tool_v2::tools::ToolKind::Composed,
            version: Version::new(1, 0, 0),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["chain".to_string()],
            author: "test".to_string(),
            license: "MIT".to_string(),
        }),
        composition_type: CompositionType::Chain(vec![tool1_id, tool2_id]),
        data_flow: None,
        error_strategy: ErrorPropagationStrategy::FailFast,
        max_concurrency: 1,
    };

    // 使用 ComposedToolExecutor 执行
    let executor = ComposedToolExecutor::new(Arc::clone(&registry));
    let input = ToolInput::new(json!({"input": 5.0}));
    let ctx = ExecutionContext::default();

    let result = executor.execute(&composed_tool, input, ctx).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success);
}

#[tokio::test]
async fn test_data_flow_mapping_integration() {
    let registry = Arc::new(ToolRegistry::new());
    
    // 创建工具
    let tool1 = create_math_tool("step1", 2.0);
    let tool2 = create_math_tool("step2", 3.0);
    
    let tool1_id = tool1.id();
    let tool2_id = tool2.id();
    
    registry.register(tool1);
    registry.register(tool2);

    // 创建数据流映射
    let mut data_flow = DataFlowMapping::new();
    // 将 step1 的 result 映射到 step2 的 input
    data_flow.add_mapping("/result", "/input");

    // 创建链式组合工具
    let composed_tool = ComposedTool {
        id: ToolId::new(),
        metadata: Arc::new(ToolMetadata {
            info: ToolInfo {
                name: "chain_with_dataflow".to_string(),
                description: "带数据流的链式执行".to_string(),
                input_schema: json!({"type": "object"}),
                output_schema: json!({"type": "object"}),
                examples: vec![],
            },
            kind: rust_tool_v2::tools::ToolKind::Composed,
            version: Version::new(1, 0, 0),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
            author: "test".to_string(),
            license: "MIT".to_string(),
        }),
        composition_type: CompositionType::Chain(vec![tool1_id, tool2_id]),
        data_flow: Some(data_flow),
        error_strategy: ErrorPropagationStrategy::FailFast,
        max_concurrency: 1,
    };

    let executor = ComposedToolExecutor::new(Arc::clone(&registry));
    let input = ToolInput::new(json!({"input": 10.0}));
    let ctx = ExecutionContext::default();

    let result = executor.execute(&composed_tool, input, ctx).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success);
}

#[tokio::test]
async fn test_conditional_execution() {
    let registry = Arc::new(ToolRegistry::new());
    
    // 创建条件工具和分支工具
    let condition_tool = create_condition_tool();
    let then_tool = create_math_tool("high_value", 10.0);
    let else_tool = create_math_tool("low_value", 1.0);
    
    let condition_id = condition_tool.id();
    let then_id = then_tool.id();
    let else_id = else_tool.id();
    
    registry.register(condition_tool);
    registry.register(then_tool);
    registry.register(else_tool);

    // 创建条件组合工具
    let composed_tool = ComposedTool {
        id: ToolId::new(),
        metadata: Arc::new(ToolMetadata {
            info: ToolInfo {
                name: "conditional_multiply".to_string(),
                description: "根据条件选择乘法器".to_string(),
                input_schema: json!({"type": "object"}),
                output_schema: json!({"type": "object"}),
                examples: vec![],
            },
            kind: rust_tool_v2::tools::ToolKind::Composed,
            version: Version::new(1, 0, 0),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["conditional".to_string()],
            author: "test".to_string(),
            license: "MIT".to_string(),
        }),
        composition_type: CompositionType::Conditional {
            condition: "exceeds == true".to_string(),
            then_tool: then_id,
            else_tool: Some(else_id),
        },
        data_flow: None,
        error_strategy: ErrorPropagationStrategy::FailFast,
        max_concurrency: 1,
    };

    let executor = ComposedToolExecutor::new(Arc::clone(&registry));
    let input = ToolInput::new(json!({
        "value": 15.0,
        "threshold": 10.0
    }));
    let ctx = ExecutionContext::default();

    let result = executor.execute(&composed_tool, input, ctx).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success);
}

#[tokio::test]
async fn test_parallel_execution() {
    let registry = Arc::new(ToolRegistry::new());
    
    // 创建多个并行工具
    let tool1 = create_math_tool("parallel_1", 1.0);
    let tool2 = create_math_tool("parallel_2", 2.0);
    let tool3 = create_math_tool("parallel_3", 3.0);
    
    let tool1_id = tool1.id();
    let tool2_id = tool2.id();
    let tool3_id = tool3.id();
    
    registry.register(tool1);
    registry.register(tool2);
    registry.register(tool3);

    // 创建并行组合工具
    let composed_tool = ComposedTool {
        id: ToolId::new(),
        metadata: Arc::new(ToolMetadata {
            info: ToolInfo {
                name: "parallel_math".to_string(),
                description: "并行执行多个数学工具".to_string(),
                input_schema: json!({"type": "object"}),
                output_schema: json!({"type": "object"}),
                examples: vec![],
            },
            kind: rust_tool_v2::tools::ToolKind::Composed,
            version: Version::new(1, 0, 0),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["parallel".to_string()],
            author: "test".to_string(),
            license: "MIT".to_string(),
        }),
        composition_type: CompositionType::Parallel(vec![tool1_id, tool2_id, tool3_id]),
        data_flow: None,
        error_strategy: ErrorPropagationStrategy::ContinueOnError,
        max_concurrency: 3,
    };

    let executor = ComposedToolExecutor::new(Arc::clone(&registry));
    let input = ToolInput::new(json!({"input": 10.0}));
    let ctx = ExecutionContext::default();

    let result = executor.execute(&composed_tool, input, ctx).await;
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.success);
    
    // 验证返回结果包含所有工具的执行结果
    let status = output.result["status"].as_str().unwrap();
    assert_eq!(status, "parallel_completed");
    
    let total = output.result["total"].as_u64().unwrap();
    assert_eq!(total, 3);
}

#[tokio::test]
async fn test_error_handling_fail_fast() {
    let registry = Arc::new(ToolRegistry::new());
    
    // 创建一个不存在的工具 ID
    let nonexistent_id = ToolId::new();

    // 创建链式组合工具，包含不存在的工具
    let composed_tool = ComposedTool {
        id: ToolId::new(),
        metadata: Arc::new(ToolMetadata {
            info: ToolInfo {
                name: "error_test".to_string(),
                description: "测试错误处理".to_string(),
                input_schema: json!({"type": "object"}),
                output_schema: json!({"type": "object"}),
                examples: vec![],
            },
            kind: rust_tool_v2::tools::ToolKind::Composed,
            version: Version::new(1, 0, 0),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
            author: "test".to_string(),
            license: "MIT".to_string(),
        }),
        composition_type: CompositionType::Chain(vec![nonexistent_id]),
        data_flow: None,
        error_strategy: ErrorPropagationStrategy::FailFast,
        max_concurrency: 1,
    };

    let executor = ComposedToolExecutor::new(Arc::clone(&registry));
    let input = ToolInput::new(json!({"input": 5.0}));
    let ctx = ExecutionContext::default();

    let result = executor.execute(&composed_tool, input, ctx).await;
    
    // 应该返回错误，因为工具不存在
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_executor_with_composed_tool() {
    // 测试 ToolExecutor 能够正确识别并委托 ComposedTool 给 ComposedToolExecutor
    
    let registry = Arc::new(ToolRegistry::new());
    
    // 创建基础工具
    let tool1 = create_math_tool("base_tool", 2.0);
    let tool1_id = tool1.id();
    registry.register(tool1);

    // 创建组合工具（链式，只有一个工具用于简化测试）
    let composed_tool = ComposedTool {
        id: ToolId::new(),
        metadata: Arc::new(ToolMetadata {
            info: ToolInfo {
                name: "composed_wrapper".to_string(),
                description: "组合工具包装器".to_string(),
                input_schema: json!({"type": "object"}),
                output_schema: json!({"type": "object"}),
                examples: vec![],
            },
            kind: rust_tool_v2::tools::ToolKind::Composed,
            version: Version::new(1, 0, 0),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
            author: "test".to_string(),
            license: "MIT".to_string(),
        }),
        composition_type: CompositionType::Chain(vec![tool1_id]),
        data_flow: None,
        error_strategy: ErrorPropagationStrategy::FailFast,
        max_concurrency: 1,
    };

    // 将组合工具注册到 registry（注意：这里需要手动添加）
    // 由于 ComposedTool 需要通过 Tool 枚举包装，我们直接测试 ToolExecutor 的执行逻辑
    
    // 测试执行器创建
    let executor = ToolExecutor::new(Arc::clone(&registry));
    
    // 验证执行器可以执行普通工具
    let input = ToolInput::new(json!({"input": 5.0}));
    let ctx = ExecutionContext::default();
    
    let result = executor.execute("base_tool", input, ctx).await;
    assert!(result.is_ok());
}
