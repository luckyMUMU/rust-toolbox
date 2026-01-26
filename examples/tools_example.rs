//! Example demonstrating the tools system

use serde_json::{json, Value};
use std::sync::Arc;
use workflow_toolkit::{
    core::ExecutionContext,
    error::Result,
    tools::{AsyncFunctionExecutor, BasicTool, BasicToolRegistry, ToolRegistry},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a simple calculator tool
    let calculator_executor = Arc::new(AsyncFunctionExecutor::new(|params, _context| Box::pin(async move {
        let operation = params["operation"].as_str().unwrap_or("add");
        let a = params["a"].as_f64().unwrap_or(0.0);
        let b = params["b"].as_f64().unwrap_or(0.0);

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b != 0.0 {
                    a / b
                } else {
                    return Err(workflow_toolkit::error::WorkflowError::ValidationError(
                        "Division by zero".to_string(),
                    ));
                }
            }
            _ => {
                return Err(workflow_toolkit::error::WorkflowError::ValidationError(
                    format!("Unknown operation: {}", operation),
                ))
            }
        };

        Ok(json!({ "result": result }))
    }));

    let calculator_tool = BasicTool::builder()
        .name("calculator")
        .version("1.0.0")
        .description("A simple calculator tool")
        .category("math")
        .tags(vec!["calculator".to_string(), "math".to_string(), "arithmetic".to_string()])
        .parameters_schema(json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "a": { "type": "number" },
                "b": { "type": "number" }
            },
            "required": ["operation", "a", "b"]
        }))
        .return_schema(json!({
            "type": "object",
            "properties": {
                "result": { "type": "number" }
            }
        }))
        .executor(calculator_executor)
        .build()?;

    // Create a string manipulation tool
    let string_executor = Arc::new(AsyncFunctionExecutor::new(|params, _context| Box::pin(async move {
        let operation = params["operation"].as_str().unwrap_or("uppercase");
        let text = params["text"].as_str().unwrap_or("");

        let result = match operation {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "reverse" => text.chars().rev().collect(),
            "length" => return Ok(json!({ "length": text.len() })),
            _ => {
                return Err(workflow_toolkit::error::WorkflowError::ValidationError(
                    format!("Unknown operation: {}", operation),
                ))
            }
        };

        Ok(json!({ "result": result }))
    }));

    let string_tool = BasicTool::builder()
        .name("string_manipulator")
        .version("1.0.0")
        .description("A string manipulation tool")
        .category("text")
        .tags(vec!["string".to_string(), "text".to_string(), "manipulation".to_string()])
        .parameters_schema(json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["uppercase", "lowercase", "reverse", "length"]
                },
                "text": { "type": "string" }
            },
            "required": ["operation", "text"]
        }))
        .executor(string_executor)
        .build()?;

    // Create a tool registry and register the tools
    let mut registry = BasicToolRegistry::new();
    registry.register_tool(Arc::new(calculator_tool))?;
    registry.register_tool(Arc::new(string_tool))?;

    println!("Tool Registry Example");
    println!("=====================");

    // List all tools
    println!("\nRegistered tools:");
    for tool_info in registry.list_tools() {
        println!(
            "- {} v{}: {}",
            tool_info.name, tool_info.version, tool_info.description
        );
        if let Some(category) = &tool_info.category {
            println!("  Category: {}", category);
        }
        if !tool_info.tags.is_empty() {
            println!("  Tags: {}", tool_info.tags.join(", "));
        }
    }

    // Create execution context
    let context = ExecutionContext::new();

    // Test calculator tool
    println!("\n--- Calculator Tool Tests ---");

    let add_params = json!({
        "operation": "add",
        "a": 10.0,
        "b": 5.0
    });

    match registry
        .execute_tool("calculator", add_params, context.clone())
        .await
    {
        Ok(result) => println!("10 + 5 = {}", result["result"]),
        Err(e) => println!("Error: {}", e),
    }

    let divide_params = json!({
        "operation": "divide",
        "a": 20.0,
        "b": 4.0
    });

    match registry
        .execute_tool("calculator", divide_params, context.clone())
        .await
    {
        Ok(result) => println!("20 / 4 = {}", result["result"]),
        Err(e) => println!("Error: {}", e),
    }

    // Test division by zero
    let divide_zero_params = json!({
        "operation": "divide",
        "a": 10.0,
        "b": 0.0
    });

    match registry
        .execute_tool("calculator", divide_zero_params, context.clone())
        .await
    {
        Ok(result) => println!("10 / 0 = {}", result["result"]),
        Err(e) => println!("Division by zero error: {}", e),
    }

    // Test string manipulation tool
    println!("\n--- String Manipulation Tool Tests ---");

    let uppercase_params = json!({
        "operation": "uppercase",
        "text": "hello world"
    });

    match registry
        .execute_tool("string_manipulator", uppercase_params, context.clone())
        .await
    {
        Ok(result) => println!("Uppercase: {}", result["result"]),
        Err(e) => println!("Error: {}", e),
    }

    let reverse_params = json!({
        "operation": "reverse",
        "text": "hello"
    });

    match registry
        .execute_tool("string_manipulator", reverse_params, context.clone())
        .await
    {
        Ok(result) => println!("Reverse: {}", result["result"]),
        Err(e) => println!("Error: {}", e),
    }

    let length_params = json!({
        "operation": "length",
        "text": "hello world"
    });

    match registry
        .execute_tool("string_manipulator", length_params, context.clone())
        .await
    {
        Ok(result) => println!("Length: {}", result["length"]),
        Err(e) => println!("Error: {}", e),
    }

    // Test parameter validation
    println!("\n--- Parameter Validation Tests ---");

    let invalid_params = json!({
        "operation": "invalid_op",
        "a": 10.0,
        "b": 5.0
    });

    match registry.validate_tool_params("calculator", &invalid_params) {
        Ok(_) => println!("Validation passed (unexpected)"),
        Err(e) => println!("Validation failed as expected: {}", e),
    }

    // Test tool discovery
    println!("\n--- Tool Discovery Tests ---");

    let math_tools = registry.get_tools_by_category("math");
    println!("Math tools: {}", math_tools.len());

    let text_tools = registry.get_tools_by_category("text");
    println!("Text tools: {}", text_tools.len());

    let calculator_tools = registry.find_tools_by_pattern("calc");
    println!("Tools matching 'calc': {:?}", calculator_tools);

    println!("\nExample completed successfully!");
    Ok(())
}
