//! EL (Expression Language) expression engine for workflow orchestration.
//!
//! This module provides a lightweight expression language for workflow definitions,
//! supporting variable interpolation (${context.xxx}), condition expressions (> < == && ||),
//! and function calls.
//!
//! # Examples
//!
//! ```rust
//! use workflow_toolkit::workflow::el::{ExpressionEngine, ExpressionContext};
//!
//! let engine = ExpressionEngine::new();
//! let mut context = ExpressionContext::new();
//! context.set("user.name", "Alice");
//! context.set("user.age", 30);
//!
//! // Variable interpolation
//! let result = engine.evaluate("${user.name}", &context).unwrap();
//! assert_eq!(result, "Alice".into());
//!
//! // Condition expression
//! let result = engine.evaluate_condition("${user.age} >= 18", &context).unwrap();
//! assert!(result);
//! ```

use crate::error::{Result, WorkflowError};
use serde_json::Value;
use std::collections::HashMap;

/// Expression context for variable storage and lookup
#[derive(Debug, Clone, Default)]
pub struct ExpressionContext {
    variables: HashMap<String, Value>,
}

impl ExpressionContext {
    /// Create a new empty expression context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Set a variable with dot notation support (e.g., "user.name")
    pub fn set(&mut self, key: &str, value: impl Into<Value>) {
        let value = value.into();
        let parts: Vec<&str> = key.split('.').collect();

        if parts.len() == 1 {
            self.variables.insert(key.to_string(), value);
        } else {
            // Handle nested paths like "user.name"
            self.set_nested(&parts, value);
        }
    }

    /// Get a variable with dot notation support
    pub fn get(&self, key: &str) -> Option<&Value> {
        let parts: Vec<&str> = key.split('.').collect();
        self.get_nested(&parts)
    }

    fn set_nested(&mut self, parts: &[&str], value: Value) {
        if parts.is_empty() {
            return;
        }

        let first = parts[0];
        if parts.len() == 1 {
            self.variables.insert(first.to_string(), value);
            return;
        }

        // Get or create the parent object
        let parent = self
            .variables
            .entry(first.to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(map) = parent {
            let mut current = map;
            for i in 1..parts.len() - 1 {
                let key = parts[i];
                if !current.contains_key(key) {
                    current.insert(key.to_string(), Value::Object(serde_json::Map::new()));
                }
                if let Some(Value::Object(next)) = current.get_mut(key) {
                    current = next;
                } else {
                    return;
                }
            }
            current.insert(parts[parts.len() - 1].to_string(), value);
        }
    }

    fn get_nested(&self, parts: &[&str]) -> Option<&Value> {
        if parts.is_empty() {
            return None;
        }

        let mut current = self.variables.get(parts[0])?;

        for part in &parts[1..] {
            current = current.get(part)?;
        }

        Some(current)
    }

    /// Merge another context into this one
    pub fn merge(&mut self, other: &ExpressionContext) {
        for (key, value) in &other.variables {
            self.variables.insert(key.clone(), value.clone());
        }
    }

    /// Convert from ExecutionContext
    pub fn from_execution_context(context: &crate::core::ExecutionContext) -> Self {
        let mut expr_context = Self::new();
        for (key, value) in &context.global_variables {
            expr_context.set(key, value.clone());
        }
        for (key, value) in &context.step_results {
            expr_context.set(key, value.clone());
        }
        expr_context
    }
}

/// Expression engine for evaluating EL expressions
#[derive(Debug, Clone, Default)]
pub struct ExpressionEngine {
    // Could add custom functions registry here in the future
}

impl ExpressionEngine {
    /// Create a new expression engine
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate an expression and return the result as JSON Value
    pub fn evaluate(&self, expression: &str, context: &ExpressionContext) -> Result<Value> {
        let trimmed = expression.trim();

        // Handle ${...} interpolation syntax
        if trimmed.starts_with("${") && trimmed.ends_with("}") {
            let inner = &trimmed[2..trimmed.len() - 1];
            return self.evaluate_expression(inner, context);
        }

        // Handle #{...} expression syntax (for conditions)
        if trimmed.starts_with("#{") && trimmed.ends_with("}") {
            let inner = &trimmed[2..trimmed.len() - 1];
            let result = self.evaluate_condition(inner, context)?;
            return Ok(Value::Bool(result));
        }

        // Plain string - try to parse as expression
        self.evaluate_expression(trimmed, context)
    }

    /// Evaluate a condition expression and return boolean result
    pub fn evaluate_condition(&self, condition: &str, context: &ExpressionContext) -> Result<bool> {
        let trimmed = condition.trim();

        // Remove surrounding #{...} if present
        let expr = if trimmed.starts_with("#{") && trimmed.ends_with("}") {
            &trimmed[2..trimmed.len() - 1]
        } else {
            trimmed
        };

        // Parse and evaluate the condition
        self.parse_and_evaluate_condition_bool(expr, context)
    }

    /// Interpolate expressions within a string
    pub fn interpolate(&self, template: &str, context: &ExpressionContext) -> Result<String> {
        let mut result = String::with_capacity(template.len());
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // consume '{'
                let mut expr = String::new();
                let mut brace_count = 1;

                while let Some(c) = chars.next() {
                    if c == '{' {
                        brace_count += 1;
                    } else if c == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                    expr.push(c);
                }

                let value = self.evaluate(&format!("${{{}}}", expr), context)?;
                result.push_str(&value_to_string(&value));
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    fn evaluate_expression(&self, expr: &str, context: &ExpressionContext) -> Result<Value> {
        let trimmed = expr.trim();

        // Try to parse as literal first
        if let Some(literal) = self.parse_literal(trimmed) {
            return Ok(literal);
        }

        // Try to get variable from context
        if let Some(value) = context.get(trimmed) {
            return Ok(value.clone());
        }

        // Try to parse as arithmetic expression
        if let Ok(result) = self.parse_and_evaluate_arithmetic(trimmed, context) {
            return Ok(result);
        }

        // Try to parse as conditional expression (but avoid recursion on simple expressions)
        // Check if it looks like a comparison or logical expression
        let is_comparison = ["==", "!=", "<", ">", "<=", ">="].iter().any(|op| trimmed.contains(op));
        let is_logical = trimmed.contains("&&") || trimmed.contains("||");
        
        if (is_comparison || is_logical) && trimmed.len() > 3 {
            // Use a simple check to avoid infinite recursion
            // Only try condition evaluation if we haven't already
            if !trimmed.starts_with("${") {
                if let Ok(result) = self.parse_and_evaluate_condition_bool(trimmed, context) {
                    return Ok(Value::Bool(result));
                }
            }
        }

        Err(WorkflowError::workflow_execution(format!(
            "Unable to evaluate expression: {}",
            expr
        )))
    }

    fn parse_literal(&self, s: &str) -> Option<Value> {
        let trimmed = s.trim();

        // Boolean literals
        match trimmed {
            "true" => return Some(Value::Bool(true)),
            "false" => return Some(Value::Bool(false)),
            "null" => return Some(Value::Null),
            _ => {}
        }

        // Number literals
        if let Ok(n) = trimmed.parse::<i64>() {
            return Some(Value::Number(n.into()));
        }
        if let Ok(n) = trimmed.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(n) {
                return Some(Value::Number(num));
            }
        }

        // String literals (quoted)
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            let content = &trimmed[1..trimmed.len() - 1];
            return Some(Value::String(content.to_string()));
        }

        None
    }

    fn parse_and_evaluate_arithmetic(&self, expr: &str, context: &ExpressionContext) -> Result<Value> {
        // Simple arithmetic parser for expressions like "a + b", "a - b", "a * b", "a / b"
        let tokens = self.tokenize(expr)?;
        self.evaluate_arithmetic_tokens(&tokens, context)
    }


    fn parse_and_evaluate_condition_bool(&self, expr: &str, context: &ExpressionContext) -> Result<bool> {
        // Handle logical operators: &&, ||
        if let Some(pos) = self.find_logical_operator(expr, "&&") {
            let left = &expr[..pos];
            let right = &expr[pos + 2..];
            return Ok(self.parse_and_evaluate_condition_bool(left, context)?
                && self.parse_and_evaluate_condition_bool(right, context)?);
        }

        if let Some(pos) = self.find_logical_operator(expr, "||") {
            let left = &expr[..pos];
            let right = &expr[pos + 2..];
            return Ok(self.parse_and_evaluate_condition_bool(left, context)?
                || self.parse_and_evaluate_condition_bool(right, context)?);
        }

        // Handle comparison operators: ==, !=, <, >, <=, >=
        if let Some((op, pos)) = self.find_comparison_operator(expr) {
            let left = expr[..pos].trim();
            let right = expr[pos + op.len()..].trim();

            let left_val = self.evaluate_expression(left, context)?;
            let right_val = self.evaluate_expression(right, context)?;

            return self.compare_values(&left_val, &right_val, &op);
        }

        // Single value - evaluate and check truthiness
        let value = self.evaluate_expression(expr, context)?;
        Ok(self.is_truthy(&value))
    }

    fn find_logical_operator(&self, expr: &str, op: &str) -> Option<usize> {
        let mut depth = 0;
        let mut chars = expr.chars().enumerate().peekable();

        while let Some((i, ch)) = chars.next() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ if depth == 0 => {
                    if expr[i..].starts_with(op) {
                        // Check that it's not part of a larger operator
                        if i + op.len() < expr.len() {
                            let next_char = expr.chars().nth(i + op.len()).unwrap();
                            if next_char.is_alphanumeric() || next_char == '_' {
                                continue;
                            }
                        }
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_comparison_operator(&self, expr: &str) -> Option<(String, usize)> {
        let operators = ["==", "!=", "<=", ">=", "<", ">"];
        let mut depth = 0;
        let mut chars = expr.chars().enumerate().peekable();

        while let Some((i, ch)) = chars.next() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ if depth == 0 => {
                    for op in &operators {
                        if expr[i..].starts_with(op) {
                            return Some((op.to_string(), i));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn compare_values(&self, left: &Value, right: &Value, op: &str) -> Result<bool> {
        match op {
            "==" => Ok(left == right),
            "!=" => Ok(left != right),
            "<" => self.compare_numeric(left, right, |a, b| a < b),
            ">" => self.compare_numeric(left, right, |a, b| a > b),
            "<=" => self.compare_numeric(left, right, |a, b| a <= b),
            ">=" => self.compare_numeric(left, right, |a, b| a >= b),
            _ => Err(WorkflowError::workflow_execution(format!(
                "Unknown comparison operator: {}",
                op
            ))),
        }
    }

    fn compare_numeric<F>(&self, left: &Value, right: &Value, compare: F) -> Result<bool>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let left_num = value_to_f64(left)?;
        let right_num = value_to_f64(right)?;
        Ok(compare(left_num, right_num))
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
        }
    }

    fn tokenize(&self, expr: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = expr.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => continue,
                '+' => tokens.push(Token::Plus),
                '-' => tokens.push(Token::Minus),
                '*' => tokens.push(Token::Multiply),
                '/' => tokens.push(Token::Divide),
                '(' => tokens.push(Token::LeftParen),
                ')' => tokens.push(Token::RightParen),
                '0'..='9' | '.' => {
                    let mut num = String::new();
                    num.push(ch);
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == '.' {
                            num.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    ident.push(ch);
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' || c == '.' {
                            ident.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Identifier(ident));
                }
                '"' | '\'' => {
                    let quote = ch;
                    let mut string = String::new();
                    while let Some(c) = chars.next() {
                        if c == quote {
                            break;
                        }
                        string.push(c);
                    }
                    tokens.push(Token::String(string));
                }
                _ => {
                    return Err(WorkflowError::workflow_execution(format!(
                        "Unexpected character: {}",
                        ch
                    )))
                }
            }
        }

        Ok(tokens)
    }

    fn evaluate_arithmetic_tokens(&self, tokens: &[Token], context: &ExpressionContext) -> Result<Value> {
        // Simple recursive descent parser for arithmetic expressions
        // This is a basic implementation - could be enhanced with proper precedence
        if tokens.is_empty() {
            return Ok(Value::Null);
        }

        // For now, handle simple binary operations
        if tokens.len() == 1 {
            match &tokens[0] {
                Token::Number(n) => {
                    if let Ok(i) = n.parse::<i64>() {
                        return Ok(Value::Number(i.into()));
                    }
                    if let Ok(f) = n.parse::<f64>() {
                        if let Some(num) = serde_json::Number::from_f64(f) {
                            return Ok(Value::Number(num));
                        }
                    }
                }
                Token::Identifier(id) => {
                    if let Some(value) = context.get(id) {
                        return Ok(value.clone());
                    }
                }
                Token::String(s) => return Ok(Value::String(s.clone())),
                _ => {}
            }
        }

        // Handle binary operations with proper precedence
        self.evaluate_binary_expression(tokens, context)
    }

    fn evaluate_binary_expression(&self, tokens: &[Token], context: &ExpressionContext) -> Result<Value> {
        // Look for + or - first (lower precedence)
        for (i, token) in tokens.iter().enumerate() {
            if matches!(token, Token::Plus | Token::Minus) {
                let left = self.evaluate_arithmetic_tokens(&tokens[..i], context)?;
                let right = self.evaluate_arithmetic_tokens(&tokens[i + 1..], context)?;

                let left_num = value_to_f64(&left)?;
                let right_num = value_to_f64(&right)?;

                let result = match token {
                    Token::Plus => left_num + right_num,
                    Token::Minus => left_num - right_num,
                    _ => unreachable!(),
                };

                if let Some(num) = serde_json::Number::from_f64(result) {
                    return Ok(Value::Number(num));
                }
            }
        }

        // Then look for * or / (higher precedence)
        for (i, token) in tokens.iter().enumerate() {
            if matches!(token, Token::Multiply | Token::Divide) {
                let left = self.evaluate_arithmetic_tokens(&tokens[..i], context)?;
                let right = self.evaluate_arithmetic_tokens(&tokens[i + 1..], context)?;

                let left_num = value_to_f64(&left)?;
                let right_num = value_to_f64(&right)?;

                let result = match token {
                    Token::Multiply => left_num * right_num,
                    Token::Divide => {
                        if right_num == 0.0 {
                            return Err(WorkflowError::workflow_execution(
                                "Division by zero".to_string(),
                            ));
                        }
                        left_num / right_num
                    }
                    _ => unreachable!(),
                };

                if let Some(num) = serde_json::Number::from_f64(result) {
                    return Ok(Value::Number(num));
                }
            }
        }

        // Handle parentheses
        if let Some(Token::LeftParen) = tokens.first() {
            // Find matching right paren
            let mut depth = 1;
            let mut end = 1;
            for (i, token) in tokens[1..].iter().enumerate() {
                match token {
                    Token::LeftParen => depth += 1,
                    Token::RightParen => {
                        depth -= 1;
                        if depth == 0 {
                            end = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            return self.evaluate_arithmetic_tokens(&tokens[1..=end], context);
        }

        Err(WorkflowError::workflow_execution(
            "Unable to evaluate arithmetic expression".to_string(),
        ))
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(String),
    Identifier(String),
    String(String),
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => serde_json::to_string(arr).unwrap_or_default(),
        Value::Object(obj) => serde_json::to_string(obj).unwrap_or_default(),
    }
}

fn value_to_f64(value: &Value) -> Result<f64> {
    match value {
        Value::Number(n) => n.as_f64().ok_or_else(|| {
            WorkflowError::workflow_execution("Cannot convert number to f64".to_string())
        }),
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            WorkflowError::workflow_execution(format!("Cannot parse '{}' as number", s))
        }),
        _ => Err(WorkflowError::workflow_execution(format!(
            "Cannot convert {:?} to number",
            value
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_basic() {
        let mut context = ExpressionContext::new();
        context.set("name", "Alice");
        context.set("age", 30);

        assert_eq!(context.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(context.get("age"), Some(&Value::Number(30.into())));
    }

    #[test]
    fn test_context_nested() {
        let mut context = ExpressionContext::new();
        context.set("user.name", "Alice");
        context.set("user.age", 30);

        assert_eq!(context.get("user.name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(context.get("user.age"), Some(&Value::Number(30.into())));
    }

    #[test]
    fn test_evaluate_variable() {
        let engine = ExpressionEngine::new();
        let mut context = ExpressionContext::new();
        context.set("name", "Alice");

        let result = engine.evaluate("${name}", &context).unwrap();
        assert_eq!(result, Value::String("Alice".to_string()));
    }

    #[test]
    fn test_evaluate_nested_variable() {
        let engine = ExpressionEngine::new();
        let mut context = ExpressionContext::new();
        context.set("user.name", "Alice");

        let result = engine.evaluate("${user.name}", &context).unwrap();
        assert_eq!(result, Value::String("Alice".to_string()));
    }

    #[test]
    fn test_evaluate_literal() {
        let engine = ExpressionEngine::new();
        let context = ExpressionContext::new();

        assert_eq!(engine.evaluate("42", &context).unwrap(), Value::Number(42.into()));
        assert_eq!(engine.evaluate("true", &context).unwrap(), Value::Bool(true));
        assert_eq!(engine.evaluate("\"hello\"", &context).unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_evaluate_condition_comparison() {
        let engine = ExpressionEngine::new();
        let mut context = ExpressionContext::new();
        context.set("age", 30);

        assert!(engine.evaluate_condition("${age} > 18", &context).unwrap());
        assert!(engine.evaluate_condition("${age} >= 30", &context).unwrap());
        assert!(!engine.evaluate_condition("${age} < 18", &context).unwrap());
        assert!(engine.evaluate_condition("${age} == 30", &context).unwrap());
        assert!(!engine.evaluate_condition("${age} != 30", &context).unwrap());
    }

    #[test]
    fn test_evaluate_condition_logical() {
        let engine = ExpressionEngine::new();
        let mut context = ExpressionContext::new();
        context.set("age", 30);
        context.set("name", "Alice");

        assert!(engine.evaluate_condition("${age} > 18 && ${age} < 65", &context).unwrap());
        assert!(engine.evaluate_condition("${age} > 100 || ${age} < 65", &context).unwrap());
        assert!(!engine.evaluate_condition("${age} > 100 || ${age} < 18", &context).unwrap());
    }

    #[test]
    fn test_interpolate() {
        let engine = ExpressionEngine::new();
        let mut context = ExpressionContext::new();
        context.set("name", "Alice");
        context.set("age", 30);

        let result = engine.interpolate("Hello, ${name}! You are ${age} years old.", &context).unwrap();
        assert_eq!(result, "Hello, Alice! You are 30 years old.");
    }
}
