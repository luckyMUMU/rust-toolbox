//! Strongly Typed Tool Examples
//!
//! This module demonstrates how to use the new strongly-typed tool system
//! with the `#[derive(ToolInput)]` and `#[derive(ToolOutput)]` macros.
//!
//! # Examples
//!
//! ## Echo Tool
//!
//! ```rust
//! use workflow_toolkit::tools::{ToolInput, ToolInputConvert, NativeToolBuilder};
//! use workflow_toolkit::macros::ToolInput;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(ToolInput, Serialize, Deserialize, Debug)]
//! pub struct EchoInput {
//!     #[tool_input(description = "Message to echo", required = true)]
//!     pub message: String,
//!     
//!     #[tool_input(description = "Number of times to echo", default = 1)]
//!     pub count: u32,
//! }
//!
//! #[derive(ToolOutput, Serialize, Deserialize, Debug)]
//! pub struct EchoOutput {
//!     pub echoed_messages: Vec<String>,
//!     pub total_chars: usize,
//! }
//!
//! // Create the tool
//! let tool = NativeToolBuilder::new()
//!     .name("echo")
//!     .version("1.0.0")
//!     .executor(|input: ToolInput, _ctx| async move {
//!         let echo_input = EchoInput::from_tool_input(&input)?;
//!         echo_input.validate()?;
//!         
//!         let mut messages = Vec::new();
//!         for i in 0..echo_input.count {
//!             messages.push(format!("{}: {}", i + 1, echo_input.message));
//!         }
//!         
//!         let output = EchoOutput {
//!             echoed_messages: messages,
//!             total_chars: echo_input.message.len() * echo_input.count as usize,
//!         };
//!         
//!         Ok(output.into_tool_output())
//!     })
//!     .build()?;
//! ```

use workflow_toolkit::tools::{
    ToolInput, ToolInputConvert, ToolOutput, ToolOutputConvert, 
    NativeToolBuilder, ToolRegistry, MiddlewareStack, LoggingMiddleware
};
use workflow_toolkit::macros::{ToolInput, ToolOutput};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Example 1: Echo Tool with strongly typed input/output
/// 
/// This tool echoes a message multiple times.
#[derive(ToolInput, Serialize, Deserialize, Debug, Clone)]
pub struct EchoInput {
    #[tool_input(description = "Message to echo", required = true)]
    pub message: String,
    
    #[tool_input(description = "Number of times to echo the message", default = 1)]
    pub count: u32,
    
    #[tool_input(description = "Prefix to add before each echo")]
    pub prefix: Option<String>,
}

#[derive(ToolOutput, Serialize, Deserialize, Debug, Clone)]
pub struct EchoOutput {
    pub echoed_messages: Vec<String>,
    pub total_chars: usize,
    pub total_messages: u32,
}

/// Create an echo tool with strongly typed parameters
pub fn create_echo_tool() -> workflow_toolkit::Result<workflow_toolkit::tools::Tool> {
    let tool = NativeToolBuilder::new()
        .name("echo")
        .version("1.0.0")
        .description("Echoes a message multiple times")
        .category("utility")
        .tags(vec!["echo", "repeat", "string"])
        .executor(|input: ToolInput, _ctx| async move {
            // Convert ToolInput to strongly-typed struct
            let echo_input = EchoInput::from_tool_input(&input)
                .map_err(|e| workflow_toolkit::WorkflowError::ValidationError(
                    format!("Invalid input: {}", e)
                ))?;
            
            // Validate input
            echo_input.validate()?;
            
            // Process
            let mut messages = Vec::new();
            for i in 0..echo_input.count {
                let msg = if let Some(ref prefix) = echo_input.prefix {
                    format!("{} {}: {}", prefix, i + 1, echo_input.message)
                } else {
                    format!("{}: {}", i + 1, echo_input.message)
                };
                messages.push(msg);
            }
            
            // Create strongly-typed output
            let output = EchoOutput {
                total_messages: echo_input.count,
                total_chars: messages.iter().map(|m| m.len()).sum(),
                echoed_messages: messages,
            };
            
            Ok(output.into_tool_output())
        })
        .build()?;
    
    Ok(workflow_toolkit::tools::Tool::Native(Arc::new(tool)))
}

/// Example 2: Calculator Tool with validation
/// 
/// This tool performs basic arithmetic operations.
#[derive(ToolInput, Serialize, Deserialize, Debug, Clone)]
pub struct CalculatorInput {
    #[tool_input(description = "First operand", required = true)]
    pub a: f64,
    
    #[tool_input(description = "Second operand", required = true)]
    pub b: f64,
    
    #[tool_input(description = "Operation to perform (add, subtract, multiply, divide)", required = true)]
    pub operation: String,
}

#[derive(ToolOutput, Serialize, Deserialize, Debug, Clone)]
pub struct CalculatorOutput {
    pub result: f64,
    pub operation: String,
    pub expression: String,
}

impl ToolInputConvert for CalculatorInput {
    fn into_tool_input(self) -> ToolInput {
        ToolInput::new(serde_json::to_value(&self).unwrap_or_default())
    }
    
    fn from_tool_input(input: &ToolInput) -> Result<Self, workflow_toolkit::WorkflowError> {
        serde_json::from_value(input.params.clone())
            .map_err(|e| workflow_toolkit::WorkflowError::ValidationError(
                format!("Failed to parse calculator input: {}", e)
            ))
    }
    
    fn validate(&self) -> Result<(), workflow_toolkit::WorkflowError> {
        // Validate operation
        let valid_ops = ["add", "subtract", "multiply", "divide"];
        if !valid_ops.contains(&self.operation.as_str()) {
            return Err(workflow_toolkit::WorkflowError::ValidationError(
                format!("Invalid operation: {}. Must be one of: {:?}", self.operation, valid_ops)
            ));
        }
        
        // Validate division by zero
        if self.operation == "divide" && self.b == 0.0 {
            return Err(workflow_toolkit::WorkflowError::ValidationError(
                "Cannot divide by zero".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn schema() -> workflow_toolkit::tools::InputSchema {
        let mut schema = workflow_toolkit::tools::InputSchema::default();
        schema.type_name = "CalculatorInput".to_string();
        schema
    }
}

/// Create a calculator tool
pub fn create_calculator_tool() -> workflow_toolkit::Result<workflow_toolkit::tools::Tool> {
    let tool = NativeToolBuilder::new()
        .name("calculator")
        .version("1.0.0")
        .description("Performs basic arithmetic operations")
        .category("math")
        .tags(vec!["math", "calculator", "arithmetic"])
        .executor(|input: ToolInput, _ctx| async move {
            let calc_input = CalculatorInput::from_tool_input(&input)?;
            calc_input.validate()?;
            
            let result = match calc_input.operation.as_str() {
                "add" => calc_input.a + calc_input.b,
                "subtract" => calc_input.a - calc_input.b,
                "multiply" => calc_input.a * calc_input.b,
                "divide" => calc_input.a / calc_input.b,
                _ => return Err(workflow_toolkit::WorkflowError::ValidationError(
                    "Invalid operation".to_string()
                )),
            };
            
            let output = CalculatorOutput {
                result,
                operation: calc_input.operation.clone(),
                expression: format!("{} {} {} = {}", 
                    calc_input.a, 
                    match calc_input.operation.as_str() {
                        "add" => "+",
                        "subtract" => "-",
                        "multiply" => "*",
                        "divide" => "/",
                        _ => "?",
                    },
                    calc_input.b,
                    result
                ),
            };
            
            Ok(output.into_tool_output())
        })
        .build()?;
    
    Ok(workflow_toolkit::tools::Tool::Native(Arc::new(tool)))
}

/// Example 3: File Info Tool with optional parameters
/// 
/// This tool retrieves information about a file.
#[derive(ToolInput, Serialize, Deserialize, Debug, Clone)]
pub struct FileInfoInput {
    #[tool_input(description = "Path to the file", required = true)]
    pub path: String,
    
    #[tool_input(description = "Whether to calculate file hash")]
    pub calculate_hash: Option<bool>,
    
    #[tool_input(description = "Hash algorithm (md5, sha256)")]
    pub hash_algorithm: Option<String>,
}

#[derive(ToolOutput, Serialize, Deserialize, Debug, Clone)]
pub struct FileInfoOutput {
    pub path: String,
    pub exists: bool,
    pub size_bytes: Option<u64>,
    pub modified_time: Option<String>,
    pub hash: Option<String>,
    pub hash_algorithm: Option<String>,
}

/// Create a file info tool
pub fn create_file_info_tool() -> workflow_toolkit::Result<workflow_toolkit::tools::Tool> {
    let tool = NativeToolBuilder::new()
        .name("file_info")
        .version("1.0.0")
        .description("Retrieves information about a file")
        .category("filesystem")
        .tags(vec!["file", "filesystem", "info"])
        .executor(|input: ToolInput, _ctx| async move {
            use tokio::fs;
            
            let file_input = FileInfoInput::from_tool_input(&input)?;
            
            let path = std::path::Path::new(&file_input.path);
            let exists = path.exists();
            
            let (size_bytes, modified_time) = if exists {
                let metadata = fs::metadata(&file_input.path).await.ok();
                (
                    metadata.as_ref().map(|m| m.len()),
                    metadata.and_then(|m| {
                        m.modified().ok().map(|t| {
                            chrono::DateTime::<chrono::Local>::from(t)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                        })
                    })
                )
            } else {
                (None, None)
            };
            
            // Calculate hash if requested
            let hash = if exists && file_input.calculate_hash.unwrap_or(false) {
                let algorithm = file_input.hash_algorithm.as_deref().unwrap_or("md5");
                if algorithm == "md5" {
                    // Simplified - in real implementation, calculate actual MD5
                    Some(format!("md5:{:x}", file_input.path.len() * 12345))
                } else {
                    Some(format!("sha256:{:x}", file_input.path.len() * 67890))
                }
            } else {
                None
            };
            
            let output = FileInfoOutput {
                path: file_input.path,
                exists,
                size_bytes,
                modified_time,
                hash,
                hash_algorithm: file_input.hash_algorithm,
            };
            
            Ok(output.into_tool_output())
        })
        .build()?;
    
    Ok(workflow_toolkit::tools::Tool::Native(Arc::new(tool)))
}

/// Example 4: Using middleware with strongly typed tools
/// 
/// This example shows how to add middleware to a strongly typed tool.
pub fn create_logged_echo_tool() -> workflow_toolkit::Result<workflow_toolkit::tools::Tool> {
    let tool = NativeToolBuilder::new()
        .name("logged_echo")
        .version("1.0.0")
        .description("Echo tool with logging middleware")
        .category("utility")
        .executor(|input: ToolInput, _ctx| async move {
            let echo_input = EchoInput::from_tool_input(&input)?;
            echo_input.validate()?;
            
            let output = EchoOutput {
                echoed_messages: vec![echo_input.message.clone()],
                total_chars: echo_input.message.len(),
                total_messages: 1,
            };
            
            Ok(output.into_tool_output())
        })
        .build()?;
    
    // Add middleware
    let mut stack = MiddlewareStack::new();
    stack.add(Arc::new(LoggingMiddleware::new()));
    
    let tool = tool.with_middleware(stack);
    
    Ok(workflow_toolkit::tools::Tool::Native(Arc::new(tool)))
}

/// Example 5: Complete workflow with multiple strongly typed tools
/// 
/// This example demonstrates registering and using multiple tools.
pub async fn demonstrate_strongly_typed_tools() -> workflow_toolkit::Result<()> {
    // Create registry
    let registry = ToolRegistry::new();
    
    // Register tools
    let echo_tool = create_echo_tool()?;
    let calc_tool = create_calculator_tool()?;
    let file_tool = create_file_info_tool()?;
    
    registry.register(echo_tool);
    registry.register(calc_tool);
    registry.register(file_tool);
    
    // Use echo tool with strongly typed input
    let echo_input = EchoInput {
        message: "Hello, World!".to_string(),
        count: 3,
        prefix: Some("Echo".to_string()),
    };
    
    let echo_result = registry.execute("echo", echo_input.into_tool_input()).await?;
    let echo_output = EchoOutput::from_tool_output(&echo_result)?;
    
    println!("Echo result: {:?}", echo_output);
    
    // Use calculator tool
    let calc_input = CalculatorInput {
        a: 10.0,
        b: 5.0,
        operation: "multiply".to_string(),
    };
    
    let calc_result = registry.execute("calculator", calc_input.into_tool_input()).await?;
    let calc_output = CalculatorOutput::from_tool_output(&calc_result)?;
    
    println!("Calculator result: {} = {}", calc_output.expression, calc_output.result);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_echo_input_validation() {
        let valid_input = EchoInput {
            message: "Hello".to_string(),
            count: 1,
            prefix: None,
        };
        assert!(valid_input.validate().is_ok());
        
        // Empty message should fail validation
        let invalid_input = EchoInput {
            message: "".to_string(),
            count: 1,
            prefix: None,
        };
        assert!(invalid_input.validate().is_err());
    }
    
    #[test]
    fn test_calculator_input_validation() {
        let valid_input = CalculatorInput {
            a: 10.0,
            b: 5.0,
            operation: "add".to_string(),
        };
        assert!(valid_input.validate().is_ok());
        
        // Invalid operation
        let invalid_input = CalculatorInput {
            a: 10.0,
            b: 5.0,
            operation: "invalid".to_string(),
        };
        assert!(invalid_input.validate().is_err());
        
        // Division by zero
        let div_by_zero = CalculatorInput {
            a: 10.0,
            b: 0.0,
            operation: "divide".to_string(),
        };
        assert!(div_by_zero.validate().is_err());
    }
    
    #[test]
    fn test_tool_input_convert() {
        let echo_input = EchoInput {
            message: "Test".to_string(),
            count: 2,
            prefix: None,
        };
        
        // Convert to ToolInput
        let tool_input = echo_input.clone().into_tool_input();
        
        // Convert back
        let recovered = EchoInput::from_tool_input(&tool_input).unwrap();
        
        assert_eq!(echo_input.message, recovered.message);
        assert_eq!(echo_input.count, recovered.count);
    }
}
