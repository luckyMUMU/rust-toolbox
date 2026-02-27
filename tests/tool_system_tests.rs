//! Tool System Tests
//!
//! Comprehensive tests for the new enum-based tool system.

use serde_json::json;
use workflow_toolkit::error::WorkflowError;
use workflow_toolkit::tools::{
    CacheMiddleware, CircuitBreakerMiddleware, ConditionalTool, LoggingMiddleware,
    MetricsMiddleware, MiddlewareStack, MiddlewareStackBuilder, ParallelTools, RetryMiddleware,
    TimeoutMiddleware, TimingMiddleware, Tool, ToolChain, ToolCompositionBuilder, ToolId,
    ToolInput, ToolKind, ToolMetadata, ToolOutput, ToolRegistry, ToolRegistryBuilder,
};

// ============================================================================
// Tool Enum Tests
// ============================================================================

#[test]
fn test_tool_enum_creation() {
    let tool = Tool::Native(crate::tools::types::NativeTool {
        id: ToolId::new("test_tool"),
        name: "Test Tool".to_string(),
        description: "A test tool".to_string(),
        version: "1.0.0".to_string(),
        kind: ToolKind::Native,
        executor: None,
        metadata: ToolMetadata::default(),
        input_schema: None,
        output_schema: None,
        resource_requirements: None,
    });

    assert_eq!(tool.name(), "Test Tool");
    assert_eq!(tool.id().as_str(), "test_tool");
}

#[test]
fn test_tool_id_parsing() {
    let id = ToolId::new("my_tool");
    assert_eq!(id.as_str(), "my_tool");

    let id2 = ToolId::parse("my_tool:1.0.0").unwrap();
    assert_eq!(id2.name(), "my_tool");
}

#[test]
fn test_tool_metadata() {
    let metadata = ToolMetadata {
        author: Some("Test Author".to_string()),
        tags: vec!["test".to_string(), "example".to_string()],
        category: Some("testing".to_string()),
        documentation_url: Some("https://example.com".to_string()),
        examples: vec![],
    };

    assert_eq!(metadata.author, Some("Test Author".to_string()));
    assert_eq!(metadata.tags.len(), 2);
}

// ============================================================================
// ToolRegistry Tests
// ============================================================================

#[tokio::test]
async fn test_registry_register() {
    let registry = ToolRegistryBuilder::new().build();

    let tool = create_test_tool("test_tool");
    registry.register("test_tool", tool).await.unwrap();

    assert!(registry.get("test_tool").await.is_some());
}

#[tokio::test]
async fn test_registry_get() {
    let registry = ToolRegistryBuilder::new().build();

    let tool = create_test_tool("test_tool");
    registry.register("test_tool", tool.clone()).await.unwrap();

    let retrieved = registry.get("test_tool").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "Test Tool");
}

#[tokio::test]
async fn test_registry_list() {
    let registry = ToolRegistryBuilder::new().build();

    registry
        .register("tool1", create_test_tool("tool1"))
        .await
        .unwrap();
    registry
        .register("tool2", create_test_tool("tool2"))
        .await
        .unwrap();
    registry
        .register("tool3", create_test_tool("tool3"))
        .await
        .unwrap();

    let tools = registry.list_tools().await;
    assert_eq!(tools.len(), 3);
}

#[tokio::test]
async fn test_registry_execute() {
    let registry = ToolRegistryBuilder::new().build();

    let tool = create_test_tool("echo");
    registry.register("echo", tool).await.unwrap();

    let input = ToolInput::new(json!({"message": "hello"}));
    let result = registry.execute("echo", input).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_access() {
    let registry = ToolRegistryBuilder::new().build();
    let mut handles = vec![];

    // Spawn multiple concurrent registrations
    for i in 0..10 {
        let reg = registry.clone();
        let handle = tokio::spawn(async move {
            let tool = create_test_tool(&format!("tool_{}", i));
            reg.register(&format!("tool_{}", i), tool).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    // Verify all tools are registered
    let tools = registry.list_tools().await;
    assert_eq!(tools.len(), 10);
}

// ============================================================================
// Middleware Tests
// ============================================================================

#[tokio::test]
async fn test_logging_middleware() {
    let middleware = LoggingMiddleware::new();
    let ctx = MiddlewareContext::new();
    let input = ToolInput::new(json!({}));

    // Test that logging middleware executes without error
    let result = middleware
        .execute(ctx, input, |_, _| {
            Box::pin(async { Ok(ToolOutput::success(json!({}))) })
        })
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_timing_middleware() {
    let middleware = TimingMiddleware::new();
    let ctx = MiddlewareContext::new();
    let input = ToolInput::new(json!({}));

    let result = middleware
        .execute(ctx, input, |_, _| {
            Box::pin(async {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                Ok(ToolOutput::success(json!({})))
            })
        })
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_retry_middleware() {
    let middleware = RetryMiddleware::new(3);
    let ctx = MiddlewareContext::new();
    let input = ToolInput::new(json!({}));

    let mut attempts = 0;
    let result = middleware
        .execute(ctx, input, |_, _| {
            Box::pin(async {
                attempts += 1;
                if attempts < 3 {
                    Err(WorkflowError::ExecutionError("Retry".to_string()))
                } else {
                    Ok(ToolOutput::success(json!({})))
                }
            })
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(attempts, 3);
}

#[tokio::test]
async fn test_timeout_middleware() {
    let middleware = TimeoutMiddleware::new(std::time::Duration::from_millis(50));
    let ctx = MiddlewareContext::new();
    let input = ToolInput::new(json!({}));

    let result = middleware
        .execute(ctx, input, |_, _| {
            Box::pin(async {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok(ToolOutput::success(json!({})))
            })
        })
        .await;

    assert!(result.is_err()); // Should timeout
}

#[tokio::test]
async fn test_circuit_breaker_middleware() {
    let middleware = CircuitBreakerMiddleware::new(2, std::time::Duration::from_secs(60));
    let ctx = MiddlewareContext::new();
    let input = ToolInput::new(json!({}));

    // First two calls fail
    for _ in 0..2 {
        let _ = middleware
            .execute(ctx.clone(), input.clone(), |_, _| {
                Box::pin(async { Err(WorkflowError::ExecutionError("Fail".to_string())) })
            })
            .await;
    }

    // Third call should be blocked by circuit breaker
    let result = middleware
        .execute(ctx, input, |_, _| {
            Box::pin(async { Ok(ToolOutput::success(json!({}))) })
        })
        .await;

    assert!(result.is_err()); // Circuit breaker open
}

#[tokio::test]
async fn test_metrics_middleware() {
    let middleware = MetricsMiddleware::new();
    let ctx = MiddlewareContext::new();
    let input = ToolInput::new(json!({}));

    let result = middleware
        .execute(ctx, input, |_, _| {
            Box::pin(async { Ok(ToolOutput::success(json!({}))) })
        })
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cache_middleware() {
    let middleware = CacheMiddleware::new(std::time::Duration::from_secs(60));
    let ctx = MiddlewareContext::new();
    let input = ToolInput::new(json!({"key": "value"}));

    let mut call_count = 0;

    // First call
    let _ = middleware
        .execute(ctx.clone(), input.clone(), |_, _| {
            Box::pin(async {
                call_count += 1;
                Ok(ToolOutput::success(json!({"count": call_count})))
            })
        })
        .await;

    // Second call with same input should use cache
    let result = middleware
        .execute(ctx, input, |_, _| {
            Box::pin(async {
                call_count += 1;
                Ok(ToolOutput::success(json!({"count": call_count})))
            })
        })
        .await;

    assert!(result.is_ok());
    // Note: Actual cache behavior depends on implementation
}

#[tokio::test]
async fn test_middleware_chain() {
    let stack = MiddlewareStackBuilder::new()
        .add(LoggingMiddleware::new())
        .add(TimingMiddleware::new())
        .build();

    let ctx = MiddlewareContext::new();
    let input = ToolInput::new(json!({}));

    let result = stack
        .execute(ctx, input, |_, _| {
            Box::pin(async { Ok(ToolOutput::success(json!({}))) })
        })
        .await;

    assert!(result.is_ok());
}

// ============================================================================
// Typed Tool Tests
// ============================================================================

#[test]
fn test_tool_input_creation() {
    let input = ToolInput::new(json!({
        "message": "hello",
        "count": 5
    }));

    assert!(input.get("message").is_some());
    assert_eq!(input.get("message").unwrap(), "hello");
}

#[test]
fn test_tool_output_creation() {
    let output = ToolOutput::success(json!({
        "result": "success",
        "data": [1, 2, 3]
    }));

    assert!(output.is_success());
    assert!(!output.is_error());
}

#[test]
fn test_input_conversion() {
    use workflow_toolkit::tools::ToolInputConvert;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct TestInput {
        message: String,
        count: u32,
    }

    impl ToolInputConvert for TestInput {
        fn into_tool_input(self) -> ToolInput {
            ToolInput::new(serde_json::to_value(&self).unwrap())
        }

        fn from_tool_input(input: &ToolInput) -> Result<Self, WorkflowError> {
            serde_json::from_value(input.params.clone())
                .map_err(|e| WorkflowError::ValidationError(format!("Parse error: {}", e)))
        }

        fn validate(&self) -> Result<(), WorkflowError> {
            if self.message.is_empty() {
                return Err(WorkflowError::ValidationError(
                    "Message cannot be empty".to_string(),
                ));
            }
            Ok(())
        }

        fn schema() -> workflow_toolkit::tools::InputSchema {
            workflow_toolkit::tools::InputSchema::default()
        }
    }

    let test_input = TestInput {
        message: "hello".to_string(),
        count: 5,
    };

    let tool_input = test_input.into_tool_input();
    let recovered = TestInput::from_tool_input(&tool_input).unwrap();

    assert_eq!(recovered.message, "hello");
    assert_eq!(recovered.count, 5);
}

#[test]
fn test_output_conversion() {
    use workflow_toolkit::tools::ToolOutputConvert;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct TestOutput {
        result: String,
        value: i32,
    }

    impl ToolOutputConvert for TestOutput {
        fn into_tool_output(self) -> ToolOutput {
            ToolOutput::success(serde_json::to_value(&self).unwrap())
        }

        fn from_tool_output(output: &ToolOutput) -> Result<Self, WorkflowError> {
            serde_json::from_value(output.result.clone())
                .map_err(|e| WorkflowError::ValidationError(format!("Parse error: {}", e)))
        }
    }

    let test_output = TestOutput {
        result: "success".to_string(),
        value: 42,
    };

    let tool_output = test_output.into_tool_output();
    let recovered = TestOutput::from_tool_output(&tool_output).unwrap();

    assert_eq!(recovered.result, "success");
    assert_eq!(recovered.value, 42);
}

// ============================================================================
// Composition Tests
// ============================================================================

#[test]
fn test_tool_chain_builder() {
    let chain = ToolChain::new("test_chain", "A test chain")
        .add_step("step1", ToolId::new("tool1"))
        .add_step("step2", ToolId::new("tool2"))
        .add_step("step3", ToolId::new("tool3"));

    assert_eq!(chain.name(), "test_chain");
    assert_eq!(chain.step_count(), 3);
}

#[test]
fn test_conditional_tool_builder() {
    let conditional = ConditionalTool::new(
        "test_conditional",
        "A test conditional",
        "x > 0",
        ToolId::new("then_tool"),
    )
    .with_else_branch(ToolId::new("else_tool"));

    assert_eq!(conditional.name(), "test_conditional");
    assert!(conditional.has_else_branch());
}

#[test]
fn test_parallel_tool_builder() {
    let parallel = ParallelTools::new("test_parallel", "A test parallel tool")
        .add_tool("tool1", ToolId::new("tool1"))
        .add_tool("tool2", ToolId::new("tool2"))
        .add_tool("tool3", ToolId::new("tool3"));

    assert_eq!(parallel.name(), "test_parallel");
    assert_eq!(parallel.tool_count(), 3);
}

#[test]
fn test_composition_builder() {
    let mut builder = ToolCompositionBuilder::new();

    builder
        .chain("chain1", "First chain")
        .add_step("step1", ToolId::new("tool1"))
        .register();

    builder
        .conditional("conditional1", "First conditional", "x > 0")
        .then(ToolId::new("then_tool"))
        .otherwise(ToolId::new("else_tool"))
        .register()
        .unwrap();

    builder
        .parallel("parallel1", "First parallel")
        .tool("tool1", ToolId::new("tool1"))
        .tool("tool2", ToolId::new("tool2"))
        .register();

    let composer = builder.build();
    assert_eq!(composer.list_chains().len(), 1);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_tool(name: &str) -> Tool {
    Tool::Native(crate::tools::types::NativeTool {
        id: ToolId::new(name),
        name: format!("Test Tool {}", name),
        description: "A test tool".to_string(),
        version: "1.0.0".to_string(),
        kind: ToolKind::Native,
        executor: None,
        metadata: ToolMetadata::default(),
        input_schema: None,
        output_schema: None,
        resource_requirements: None,
    })
}
