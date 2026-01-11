//! Parameter template expansion for tool configurations

use crate::error::{Result, WorkflowError};
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Template variable context for parameter expansion
pub struct TemplateContext {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Box<dyn TemplateFn>>,
}

impl std::fmt::Debug for TemplateContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateContext")
            .field("variables", &self.variables)
            .field("functions", &format!("{} functions", self.functions.len()))
            .finish()
    }
}

impl Clone for TemplateContext {
    fn clone(&self) -> Self {
        let mut new_context = Self::new();
        new_context.variables = self.variables.clone();
        // Note: Functions are not cloned as they contain trait objects
        // The new context will have the default built-in functions
        new_context
    }
}

impl TemplateContext {
    /// Create a new empty template context
    pub fn new() -> Self {
        let mut context = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        };

        // Add built-in functions
        context.add_builtin_functions();
        context
    }

    /// Set a variable in the context
    pub fn set_variable<K: Into<String>>(&mut self, key: K, value: Value) {
        self.variables.insert(key.into(), value);
    }

    /// Get a variable from the context
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }

    /// Set multiple variables at once
    pub fn set_variables(&mut self, variables: HashMap<String, Value>) {
        self.variables.extend(variables);
    }

    /// Add a custom template function
    pub fn add_function<K: Into<String>>(&mut self, name: K, func: Box<dyn TemplateFn>) {
        self.functions.insert(name.into(), func);
    }

    /// Add built-in template functions
    fn add_builtin_functions(&mut self) {
        // env(var_name) - get environment variable
        self.functions
            .insert("env".to_string(), Box::new(EnvFunction));

        // default(value, default_value) - use default if value is null/empty
        self.functions
            .insert("default".to_string(), Box::new(DefaultFunction));

        // upper(text) - convert to uppercase
        self.functions
            .insert("upper".to_string(), Box::new(UpperFunction));

        // lower(text) - convert to lowercase
        self.functions
            .insert("lower".to_string(), Box::new(LowerFunction));

        // join(array, separator) - join array elements
        self.functions
            .insert("join".to_string(), Box::new(JoinFunction));

        // split(text, separator) - split text into array
        self.functions
            .insert("split".to_string(), Box::new(SplitFunction));

        // length(value) - get length of string or array
        self.functions
            .insert("length".to_string(), Box::new(LengthFunction));

        // format(template, ...args) - format string with arguments
        self.functions
            .insert("format".to_string(), Box::new(FormatFunction));
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for template functions
pub trait TemplateFn: Send + Sync {
    /// Execute the function with given arguments
    fn call(&self, args: Vec<Value>) -> Result<Value>;

    /// Get function name for error reporting
    fn name(&self) -> &str;

    /// Get expected argument count (None for variable arguments)
    fn expected_args(&self) -> Option<usize> {
        None
    }
}

/// Template expansion engine
pub struct TemplateEngine {
    variable_pattern: Regex,
    function_pattern: Regex,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Result<Self> {
        let variable_pattern = Regex::new(r"\$\{([^}]+)\}").map_err(|e| {
            WorkflowError::ValidationError(format!("Invalid variable regex: {}", e))
        })?;

        let function_pattern = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)\s*\(\s*([^)]*)\s*\)")
            .map_err(|e| {
                WorkflowError::ValidationError(format!("Invalid function regex: {}", e))
            })?;

        Ok(Self {
            variable_pattern,
            function_pattern,
        })
    }

    /// Expand templates in a JSON value
    pub fn expand(&self, value: &Value, context: &TemplateContext) -> Result<Value> {
        match value {
            Value::String(s) => self.expand_string(s, context),
            Value::Array(arr) => {
                let expanded: Result<Vec<Value>> =
                    arr.iter().map(|v| self.expand(v, context)).collect();
                Ok(Value::Array(expanded?))
            }
            Value::Object(obj) => {
                let mut expanded = Map::new();
                for (key, val) in obj {
                    let expanded_key =
                        if let Value::String(expanded_key) = self.expand_string(key, context)? {
                            expanded_key
                        } else {
                            key.clone()
                        };
                    expanded.insert(expanded_key, self.expand(val, context)?);
                }
                Ok(Value::Object(expanded))
            }
            _ => Ok(value.clone()),
        }
    }

    /// Expand templates in a string
    pub fn expand_string(&self, template: &str, context: &TemplateContext) -> Result<Value> {
        debug!("Expanding template: {}", template);

        let mut result = template.to_string();
        let mut changed = true;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10; // Prevent infinite loops

        while changed && iterations < MAX_ITERATIONS {
            changed = false;
            iterations += 1;

            // Replace variables first
            result = self
                .variable_pattern
                .replace_all(&result, |caps: &regex::Captures| {
                    let var_expr = &caps[1];
                    match self.evaluate_expression(var_expr, context) {
                        Ok(value) => {
                            changed = true;
                            self.value_to_string(&value)
                        }
                        Err(e) => {
                            warn!("Failed to expand variable '{}': {}", var_expr, e);
                            caps[0].to_string() // Return original if expansion fails
                        }
                    }
                })
                .to_string();
        }

        if iterations >= MAX_ITERATIONS {
            warn!(
                "Template expansion reached maximum iterations for: {}",
                template
            );
        }

        // Try to parse as JSON if it looks like JSON
        if (result.starts_with('{') && result.ends_with('}'))
            || (result.starts_with('[') && result.ends_with(']'))
        {
            match serde_json::from_str(&result) {
                Ok(json_value) => Ok(json_value),
                Err(_) => Ok(Value::String(result)),
            }
        } else {
            // Try to parse as number or boolean
            if let Ok(num) = result.parse::<i64>() {
                Ok(Value::Number(num.into()))
            } else if let Ok(num) = result.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(num) {
                    Ok(Value::Number(num))
                } else {
                    Ok(Value::String(result))
                }
            } else if let Ok(bool_val) = result.parse::<bool>() {
                Ok(Value::Bool(bool_val))
            } else {
                Ok(Value::String(result))
            }
        }
    }

    /// Evaluate a template expression (variable or function call)
    fn evaluate_expression(&self, expr: &str, context: &TemplateContext) -> Result<Value> {
        let expr = expr.trim();

        // Check if it's a function call
        if let Some(caps) = self.function_pattern.captures(expr) {
            let func_name = &caps[1];
            let args_str = &caps[2];

            if let Some(func) = context.functions.get(func_name) {
                let args = self.parse_function_args(args_str, context)?;
                return func.call(args);
            } else {
                return Err(WorkflowError::ValidationError(format!(
                    "Unknown function: {}",
                    func_name
                )));
            }
        }

        // Check if it's a simple variable reference
        if let Some(value) = context.get_variable(expr) {
            return Ok(value.clone());
        }

        // Check for nested property access (e.g., "object.property")
        if expr.contains('.') {
            return self.evaluate_nested_property(expr, context);
        }

        Err(WorkflowError::ValidationError(format!(
            "Unknown variable or expression: {}",
            expr
        )))
    }

    /// Parse function arguments
    fn parse_function_args(&self, args_str: &str, context: &TemplateContext) -> Result<Vec<Value>> {
        if args_str.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';
        let mut paren_depth = 0;

        for ch in args_str.chars() {
            match ch {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                    current_arg.push(ch);
                }
                ch if in_quotes && ch == quote_char => {
                    in_quotes = false;
                    current_arg.push(ch);
                }
                '(' if !in_quotes => {
                    paren_depth += 1;
                    current_arg.push(ch);
                }
                ')' if !in_quotes => {
                    paren_depth -= 1;
                    current_arg.push(ch);
                }
                ',' if !in_quotes && paren_depth == 0 => {
                    args.push(self.parse_argument_value(&current_arg, context)?);
                    current_arg.clear();
                }
                _ => {
                    current_arg.push(ch);
                }
            }
        }

        if !current_arg.trim().is_empty() {
            args.push(self.parse_argument_value(&current_arg, context)?);
        }

        Ok(args)
    }

    /// Parse a single argument value
    fn parse_argument_value(&self, arg: &str, context: &TemplateContext) -> Result<Value> {
        let arg = arg.trim();

        // String literal
        if (arg.starts_with('"') && arg.ends_with('"'))
            || (arg.starts_with('\'') && arg.ends_with('\''))
        {
            let content = &arg[1..arg.len() - 1];
            return Ok(Value::String(content.to_string()));
        }

        // Number literal
        if let Ok(num) = arg.parse::<i64>() {
            return Ok(Value::Number(num.into()));
        }

        if let Ok(num) = arg.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(num) {
                return Ok(Value::Number(num));
            }
        }

        // Boolean literal
        if let Ok(bool_val) = arg.parse::<bool>() {
            return Ok(Value::Bool(bool_val));
        }

        // Variable or expression
        self.evaluate_expression(arg, context)
    }

    /// Evaluate nested property access
    fn evaluate_nested_property(&self, expr: &str, context: &TemplateContext) -> Result<Value> {
        let parts: Vec<&str> = expr.split('.').collect();
        if parts.is_empty() {
            return Err(WorkflowError::ValidationError(
                "Empty property path".to_string(),
            ));
        }

        let mut current = context
            .get_variable(parts[0])
            .ok_or_else(|| {
                WorkflowError::ValidationError(format!("Unknown variable: {}", parts[0]))
            })?
            .clone();

        for part in &parts[1..] {
            match &current {
                Value::Object(obj) => {
                    current = obj
                        .get(*part)
                        .ok_or_else(|| {
                            WorkflowError::ValidationError(format!("Property '{}' not found", part))
                        })?
                        .clone();
                }
                Value::Array(arr) => {
                    let index = part.parse::<usize>().map_err(|_| {
                        WorkflowError::ValidationError(format!("Invalid array index: {}", part))
                    })?;
                    current = arr
                        .get(index)
                        .ok_or_else(|| {
                            WorkflowError::ValidationError(format!(
                                "Array index {} out of bounds",
                                index
                            ))
                        })?
                        .clone();
                }
                _ => {
                    return Err(WorkflowError::ValidationError(format!(
                        "Cannot access property '{}' on non-object/array",
                        part
                    )));
                }
            }
        }

        Ok(current)
    }

    /// Convert a JSON value to string for template substitution
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => value.to_string(),
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default template engine")
    }
}

// Built-in template functions

/// Environment variable function
struct EnvFunction;

impl TemplateFn for EnvFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(WorkflowError::ValidationError(
                "env() function expects exactly 1 argument".to_string(),
            ));
        }

        let var_name = args[0].as_str().ok_or_else(|| {
            WorkflowError::ValidationError("env() function expects string argument".to_string())
        })?;

        match std::env::var(var_name) {
            Ok(value) => Ok(Value::String(value)),
            Err(_) => Ok(Value::Null),
        }
    }

    fn name(&self) -> &str {
        "env"
    }

    fn expected_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// Default value function
struct DefaultFunction;

impl TemplateFn for DefaultFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(WorkflowError::ValidationError(
                "default() function expects exactly 2 arguments".to_string(),
            ));
        }

        let value = &args[0];
        let default_value = &args[1];

        if value.is_null() || (value.is_string() && value.as_str().unwrap().is_empty()) {
            Ok(default_value.clone())
        } else {
            Ok(value.clone())
        }
    }

    fn name(&self) -> &str {
        "default"
    }

    fn expected_args(&self) -> Option<usize> {
        Some(2)
    }
}

/// Uppercase function
struct UpperFunction;

impl TemplateFn for UpperFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(WorkflowError::ValidationError(
                "upper() function expects exactly 1 argument".to_string(),
            ));
        }

        let text = args[0].as_str().ok_or_else(|| {
            WorkflowError::ValidationError("upper() function expects string argument".to_string())
        })?;

        Ok(Value::String(text.to_uppercase()))
    }

    fn name(&self) -> &str {
        "upper"
    }

    fn expected_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// Lowercase function
struct LowerFunction;

impl TemplateFn for LowerFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(WorkflowError::ValidationError(
                "lower() function expects exactly 1 argument".to_string(),
            ));
        }

        let text = args[0].as_str().ok_or_else(|| {
            WorkflowError::ValidationError("lower() function expects string argument".to_string())
        })?;

        Ok(Value::String(text.to_lowercase()))
    }

    fn name(&self) -> &str {
        "lower"
    }

    fn expected_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// Join array function
struct JoinFunction;

impl TemplateFn for JoinFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(WorkflowError::ValidationError(
                "join() function expects exactly 2 arguments".to_string(),
            ));
        }

        let array = args[0].as_array().ok_or_else(|| {
            WorkflowError::ValidationError(
                "join() function expects array as first argument".to_string(),
            )
        })?;

        let separator = args[1].as_str().ok_or_else(|| {
            WorkflowError::ValidationError(
                "join() function expects string as second argument".to_string(),
            )
        })?;

        let strings: Vec<String> = array
            .iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => v.to_string(),
            })
            .collect();

        Ok(Value::String(strings.join(separator)))
    }

    fn name(&self) -> &str {
        "join"
    }

    fn expected_args(&self) -> Option<usize> {
        Some(2)
    }
}

/// Split string function
struct SplitFunction;

impl TemplateFn for SplitFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(WorkflowError::ValidationError(
                "split() function expects exactly 2 arguments".to_string(),
            ));
        }

        let text = args[0].as_str().ok_or_else(|| {
            WorkflowError::ValidationError(
                "split() function expects string as first argument".to_string(),
            )
        })?;

        let separator = args[1].as_str().ok_or_else(|| {
            WorkflowError::ValidationError(
                "split() function expects string as second argument".to_string(),
            )
        })?;

        let parts: Vec<Value> = text
            .split(separator)
            .map(|s| Value::String(s.to_string()))
            .collect();

        Ok(Value::Array(parts))
    }

    fn name(&self) -> &str {
        "split"
    }

    fn expected_args(&self) -> Option<usize> {
        Some(2)
    }
}

/// Length function
struct LengthFunction;

impl TemplateFn for LengthFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.len() != 1 {
            return Err(WorkflowError::ValidationError(
                "length() function expects exactly 1 argument".to_string(),
            ));
        }

        let length = match &args[0] {
            Value::String(s) => s.len(),
            Value::Array(arr) => arr.len(),
            Value::Object(obj) => obj.len(),
            _ => {
                return Err(WorkflowError::ValidationError(
                    "length() function expects string, array, or object".to_string(),
                ))
            }
        };

        Ok(Value::Number((length as i64).into()))
    }

    fn name(&self) -> &str {
        "length"
    }

    fn expected_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// Format string function
struct FormatFunction;

impl TemplateFn for FormatFunction {
    fn call(&self, args: Vec<Value>) -> Result<Value> {
        if args.is_empty() {
            return Err(WorkflowError::ValidationError(
                "format() function expects at least 1 argument".to_string(),
            ));
        }

        let template = args[0].as_str().ok_or_else(|| {
            WorkflowError::ValidationError(
                "format() function expects string template as first argument".to_string(),
            )
        })?;

        let mut result = template.to_string();

        // Replace {0}, {1}, etc. with corresponding arguments
        for (i, arg) in args.iter().skip(1).enumerate() {
            let placeholder = format!("{{{}}}", i);
            let replacement = match arg {
                Value::String(s) => s.clone(),
                _ => arg.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        Ok(Value::String(result))
    }

    fn name(&self) -> &str {
        "format"
    }
}

/// Parameter template configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParameterTemplate {
    pub name: String,
    pub description: Option<String>,
    pub template: Value,
    pub required_variables: Vec<String>,
    pub default_values: HashMap<String, Value>,
}

impl ParameterTemplate {
    /// Create a new parameter template
    pub fn new(name: String, template: Value) -> Self {
        Self {
            name,
            description: None,
            template,
            required_variables: Vec::new(),
            default_values: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add required variable
    pub fn with_required_variable(mut self, variable: String) -> Self {
        self.required_variables.push(variable);
        self
    }

    /// Add default value
    pub fn with_default_value(mut self, key: String, value: Value) -> Self {
        self.default_values.insert(key, value);
        self
    }

    /// Expand the template with given context
    pub fn expand(&self, context: &mut TemplateContext, engine: &TemplateEngine) -> Result<Value> {
        // Add default values to context
        for (key, value) in &self.default_values {
            if context.get_variable(key).is_none() {
                context.set_variable(key.clone(), value.clone());
            }
        }

        // Check required variables
        for var in &self.required_variables {
            if context.get_variable(var).is_none() {
                return Err(WorkflowError::ValidationError(format!(
                    "Required template variable '{}' not provided",
                    var
                )));
            }
        }

        engine.expand(&self.template, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_template_engine_basic() {
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::new();
        context.set_variable("name", json!("world"));

        let result = engine.expand_string("Hello ${name}!", &context).unwrap();
        assert_eq!(result, json!("Hello world!"));
    }

    #[test]
    fn test_template_functions() {
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::new();
        context.set_variable("text", json!("hello"));

        let result = engine.expand_string("${upper(text)}", &context).unwrap();
        assert_eq!(result, json!("HELLO"));

        let result = engine
            .expand_string("${default(missing, 'default_value')}", &context)
            .unwrap();
        assert_eq!(result, json!("default_value"));
    }

    #[test]
    fn test_nested_properties() {
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::new();
        context.set_variable(
            "config",
            json!({
                "database": {
                    "host": "localhost",
                    "port": 5432
                }
            }),
        );

        let result = engine
            .expand_string("${config.database.host}:${config.database.port}", &context)
            .unwrap();
        assert_eq!(result, json!("localhost:5432"));
    }

    #[test]
    fn test_parameter_template() {
        let template = ParameterTemplate::new(
            "database_config".to_string(),
            json!({
                "host": "${host}",
                "port": "${port}",
                "database": "${database}",
                "url": "postgresql://${host}:${port}/${database}"
            }),
        )
        .with_required_variable("host".to_string())
        .with_default_value("port".to_string(), json!(5432))
        .with_default_value("database".to_string(), json!("myapp"));

        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::new();
        context.set_variable("host", json!("localhost"));

        let result = template.expand(&mut context, &engine).unwrap();
        let expected = json!({
            "host": "localhost",
            "port": 5432,
            "database": "myapp",
            "url": "postgresql://localhost:5432/myapp"
        });

        assert_eq!(result, expected);
    }
}
