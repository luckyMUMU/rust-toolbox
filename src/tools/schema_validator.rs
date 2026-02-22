//! 工具 Schema 验证器
//!
//! 提供工具输入输出的 JSON Schema 验证功能

use crate::error::{Result, WorkflowError};
use crate::tools::types::{InputSchema, OutputSchema, ToolInput, ToolOutput};
use jsonschema::{JSONSchema, ValidationError as JsonValidationError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

/// Schema 验证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidatorConfig {
    /// 是否启用严格模式
    pub strict_mode: bool,
    /// 是否允许额外属性
    pub allow_additional_properties: bool,
    /// 最大验证深度
    pub max_depth: u32,
    /// 是否收集所有错误
    pub collect_all_errors: bool,
}

impl Default for SchemaValidatorConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            allow_additional_properties: true,
            max_depth: 32,
            collect_all_errors: false,
        }
    }
}

/// Schema 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationError {
    /// 错误路径
    pub path: String,
    /// 错误消息
    pub message: String,
    /// 错误类型
    pub error_type: SchemaErrorType,
    /// 实际值
    pub actual_value: Option<Value>,
    /// 期望值
    pub expected_value: Option<Value>,
}

impl std::fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Schema 验证错误 [{}]: {} - {}", self.path, self.error_type, self.message)
    }
}

impl std::error::Error for SchemaValidationError {}

/// Schema 错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaErrorType {
    /// 类型不匹配
    TypeMismatch,
    /// 必填字段缺失
    RequiredMissing,
    /// 值超出范围
    ValueOutOfRange,
    /// 模式不匹配
    PatternMismatch,
    /// 长度超出限制
    LengthExceeded,
    /// 枚举值不匹配
    EnumMismatch,
    /// 额外属性
    AdditionalProperty,
    /// 依赖缺失
    DependencyMissing,
    /// 格式错误
    FormatError,
    /// 其他错误
    Other,
}

impl std::fmt::Display for SchemaErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaErrorType::TypeMismatch => write!(f, "类型不匹配"),
            SchemaErrorType::RequiredMissing => write!(f, "必填字段缺失"),
            SchemaErrorType::ValueOutOfRange => write!(f, "值超出范围"),
            SchemaErrorType::PatternMismatch => write!(f, "模式不匹配"),
            SchemaErrorType::LengthExceeded => write!(f, "长度超出限制"),
            SchemaErrorType::EnumMismatch => write!(f, "枚举值不匹配"),
            SchemaErrorType::AdditionalProperty => write!(f, "额外属性"),
            SchemaErrorType::DependencyMissing => write!(f, "依赖缺失"),
            SchemaErrorType::FormatError => write!(f, "格式错误"),
            SchemaErrorType::Other => write!(f, "其他错误"),
        }
    }
}

/// Schema 验证结果
#[derive(Debug, Clone)]
pub struct SchemaValidationResult {
    /// 是否验证通过
    pub is_valid: bool,
    /// 错误列表
    pub errors: Vec<SchemaValidationError>,
    /// 警告列表
    pub warnings: Vec<String>,
}

impl SchemaValidationResult {
    /// 创建成功的验证结果
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// 创建失败的验证结果
    pub fn invalid(errors: Vec<SchemaValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// 添加警告
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// 合并两个验证结果
    pub fn merge(mut self, other: SchemaValidationResult) -> Self {
        self.is_valid = self.is_valid && other.is_valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self
    }
}

/// 编译后的 Schema 缓存
#[derive(Debug)]
struct CompiledSchema {
    schema: JSONSchema,
    raw_schema: Value,
}

/// Schema 验证器
pub struct SchemaValidator {
    config: SchemaValidatorConfig,
    compiled_schemas: HashMap<String, Arc<CompiledSchema>>,
}

impl SchemaValidator {
    /// 创建新的 Schema 验证器
    pub fn new(config: SchemaValidatorConfig) -> Self {
        Self {
            config,
            compiled_schemas: HashMap::new(),
        }
    }

    /// 使用默认配置创建验证器
    pub fn default_validator() -> Self {
        Self::new(SchemaValidatorConfig::default())
    }

    /// 获取配置
    pub fn config(&self) -> &SchemaValidatorConfig {
        &self.config
    }

    /// 将 InputSchema 转换为 JSON Schema Value
    fn input_schema_to_value(schema: &InputSchema) -> Value {
        let properties = schema.properties.clone();
        
        let mut schema_value = serde_json::json!({
            "type": "object",
            "properties": properties,
        });

        if !schema.required.is_empty() {
            schema_value["required"] = Value::Array(
                schema.required.iter().map(|s| Value::String(s.clone())).collect()
            );
        }

        if !schema.type_name.is_empty() {
            schema_value["title"] = Value::String(schema.type_name.clone());
        }

        schema_value
    }

    /// 将 OutputSchema 转换为 JSON Schema Value
    fn output_schema_to_value(schema: &OutputSchema) -> Value {
        let properties = schema.properties.clone();
        
        let mut schema_value = serde_json::json!({
            "type": "object",
            "properties": properties,
        });

        if !schema.type_name.is_empty() {
            schema_value["title"] = Value::String(schema.type_name.clone());
        }

        schema_value
    }

    /// 编译并缓存 Schema
    fn compile_schema(&mut self, name: &str, schema_value: Value) -> Result<Arc<CompiledSchema>> {
        if let Some(cached) = self.compiled_schemas.get(name) {
            return Ok(Arc::clone(cached));
        }

        let compiled = JSONSchema::compile(&schema_value).map_err(|e| {
            WorkflowError::validation(format!("Schema 编译失败 '{}': {}", name, e))
        })?;

        let compiled_schema = Arc::new(CompiledSchema {
            schema: compiled,
            raw_schema: schema_value,
        });

        self.compiled_schemas.insert(name.to_string(), Arc::clone(&compiled_schema));
        debug!("Schema '{}' 编译成功", name);
        Ok(compiled_schema)
    }

    /// 验证工具输入
    pub fn validate_input(
        &mut self,
        tool_name: &str,
        schema: &InputSchema,
        input: &ToolInput,
    ) -> Result<SchemaValidationResult> {
        let schema_name = format!("{}_input", tool_name);
        let schema_value = Self::input_schema_to_value(schema);
        
        let compiled = self.compile_schema(&schema_name, schema_value)?;
        self.validate_value(&compiled, &input.params)
    }

    /// 验证工具输出
    pub fn validate_output(
        &mut self,
        tool_name: &str,
        schema: &OutputSchema,
        output: &ToolOutput,
    ) -> Result<SchemaValidationResult> {
        let schema_name = format!("{}_output", tool_name);
        let schema_value = Self::output_schema_to_value(schema);
        
        let compiled = self.compile_schema(&schema_name, schema_value)?;
        self.validate_value(&compiled, &output.result)
    }

    /// 验证任意 JSON 值
    pub fn validate_value(
        &self,
        compiled: &CompiledSchema,
        value: &Value,
    ) -> Result<SchemaValidationResult> {
        let result = compiled.schema.validate(value);
        
        match result {
            Ok(_) => {
                debug!("Schema 验证通过");
                Ok(SchemaValidationResult::valid())
            }
            Err(errors) => {
                let schema_errors: Vec<SchemaValidationError> = errors
                    .map(|e| self.convert_validation_error(&e))
                    .collect();
                
                if self.config.collect_all_errors {
                    Ok(SchemaValidationResult::invalid(schema_errors))
                } else if let Some(first_error) = schema_errors.into_iter().next() {
                    Ok(SchemaValidationResult::invalid(vec![first_error]))
                } else {
                    Ok(SchemaValidationResult::invalid(vec![SchemaValidationError {
                        path: "/".to_string(),
                        message: "未知验证错误".to_string(),
                        error_type: SchemaErrorType::Other,
                        actual_value: None,
                        expected_value: None,
                    }]))
                }
            }
        }
    }

    /// 转换 jsonschema 验证错误
    fn convert_validation_error(&self, error: &JsonValidationError) -> SchemaValidationError {
        let path = error.instance_path.to_string();
        let message = error.to_string();
        
        let error_type = match error {
            _ => SchemaErrorType::Other,
        };

        SchemaValidationError {
            path: if path.is_empty() { "/".to_string() } else { path },
            message,
            error_type,
            actual_value: Some(error.instance.clone().into_owned()),
            expected_value: None,
        }
    }

    /// 验证 JSON 值是否符合指定 Schema
    pub fn validate_against_schema(
        &mut self,
        schema_name: &str,
        schema: &Value,
        value: &Value,
    ) -> Result<SchemaValidationResult> {
        let compiled = self.compile_schema(schema_name, schema.clone())?;
        self.validate_value(&compiled, value)
    }

    /// 清除 Schema 缓存
    pub fn clear_cache(&mut self) {
        self.compiled_schemas.clear();
        debug!("Schema 缓存已清除");
    }

    /// 获取缓存的 Schema 数量
    pub fn cached_schema_count(&self) -> usize {
        self.compiled_schemas.len()
    }
}

/// Schema 验证器构建器
pub struct SchemaValidatorBuilder {
    config: SchemaValidatorConfig,
}

impl SchemaValidatorBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: SchemaValidatorConfig::default(),
        }
    }

    /// 启用严格模式
    pub fn strict_mode(mut self, enable: bool) -> Self {
        self.config.strict_mode = enable;
        self
    }

    /// 设置是否允许额外属性
    pub fn allow_additional_properties(mut self, allow: bool) -> Self {
        self.config.allow_additional_properties = allow;
        self
    }

    /// 设置最大验证深度
    pub fn max_depth(mut self, depth: u32) -> Self {
        self.config.max_depth = depth;
        self
    }

    /// 设置是否收集所有错误
    pub fn collect_all_errors(mut self, collect: bool) -> Self {
        self.config.collect_all_errors = collect;
        self
    }

    /// 构建验证器
    pub fn build(self) -> SchemaValidator {
        SchemaValidator::new(self.config)
    }
}

impl Default for SchemaValidatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具 Schema 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// 工具名称
    pub name: String,
    /// 输入 Schema
    pub input_schema: InputSchema,
    /// 输出 Schema
    pub output_schema: OutputSchema,
}

impl ToolSchema {
    /// 创建新的工具 Schema
    pub fn new(name: String, input_schema: InputSchema, output_schema: OutputSchema) -> Self {
        Self {
            name,
            input_schema,
            output_schema,
        }
    }

    /// 从 JSON Schema 创建
    pub fn from_json(name: String, input: Value, output: Value) -> Self {
        let input_schema = InputSchema {
            type_name: input.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            properties: input.get("properties")
                .and_then(|v| v.as_object())
                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
            required: input.get("required")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        };

        let output_schema = OutputSchema {
            type_name: output.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            properties: output.get("properties")
                .and_then(|v| v.as_object())
                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
        };

        Self {
            name,
            input_schema,
            output_schema,
        }
    }
}

/// Schema 注册表
pub struct SchemaRegistry {
    validator: SchemaValidator,
    schemas: HashMap<String, ToolSchema>,
}

impl SchemaRegistry {
    /// 创建新的 Schema 注册表
    pub fn new() -> Self {
        Self {
            validator: SchemaValidator::default_validator(),
            schemas: HashMap::new(),
        }
    }

    /// 使用指定配置创建注册表
    pub fn with_config(config: SchemaValidatorConfig) -> Self {
        Self {
            validator: SchemaValidator::new(config),
            schemas: HashMap::new(),
        }
    }

    /// 注册工具 Schema
    pub fn register(&mut self, schema: ToolSchema) {
        debug!("注册工具 Schema: {}", schema.name);
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// 获取工具 Schema
    pub fn get(&self, name: &str) -> Option<&ToolSchema> {
        self.schemas.get(name)
    }

    /// 验证工具输入
    pub fn validate_input(&mut self, tool_name: &str, input: &ToolInput) -> Result<SchemaValidationResult> {
        let schema = self.schemas.get(tool_name).ok_or_else(|| {
            WorkflowError::validation(format!("工具 Schema 未找到: {}", tool_name))
        })?;
        
        self.validator.validate_input(tool_name, &schema.input_schema, input)
    }

    /// 验证工具输出
    pub fn validate_output(&mut self, tool_name: &str, output: &ToolOutput) -> Result<SchemaValidationResult> {
        let schema = self.schemas.get(tool_name).ok_or_else(|| {
            WorkflowError::validation(format!("工具 Schema 未找到: {}", tool_name))
        })?;
        
        self.validator.validate_output(tool_name, &schema.output_schema, output)
    }

    /// 注销工具 Schema
    pub fn unregister(&mut self, name: &str) -> bool {
        self.schemas.remove(name).is_some()
    }

    /// 获取已注册的工具数量
    pub fn len(&self) -> usize {
        self.schemas.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.schemas.is_empty()
    }

    /// 列出所有已注册的工具名称
    pub fn tool_names(&self) -> Vec<&String> {
        self.schemas.keys().collect()
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_input_schema() -> InputSchema {
        InputSchema {
            type_name: "TestInput".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("name".to_string(), json!({"type": "string"}));
                props.insert("count".to_string(), json!({"type": "integer", "minimum": 0}));
                props
            },
            required: vec!["name".to_string()],
        }
    }

    fn create_test_output_schema() -> OutputSchema {
        OutputSchema {
            type_name: "TestOutput".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("result".to_string(), json!({"type": "string"}));
                props
            },
        }
    }

    #[test]
    fn test_schema_validator_creation() {
        let validator = SchemaValidator::default_validator();
        assert!(!validator.config.strict_mode);
        assert!(validator.config.allow_additional_properties);
    }

    #[test]
    fn test_validate_valid_input() {
        let mut validator = SchemaValidator::default_validator();
        let schema = create_test_input_schema();
        let input = ToolInput::new(json!({"name": "test", "count": 5}));

        let result = validator.validate_input("test_tool", &schema, &input).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_missing_required_field() {
        let mut validator = SchemaValidator::default_validator();
        let schema = create_test_input_schema();
        let input = ToolInput::new(json!({"count": 5}));

        let result = validator.validate_input("test_tool", &schema, &input).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e.error_type, SchemaErrorType::RequiredMissing)));
    }

    #[test]
    fn test_validate_type_mismatch() {
        let mut validator = SchemaValidator::default_validator();
        let schema = create_test_input_schema();
        let input = ToolInput::new(json!({"name": "test", "count": "not_a_number"}));

        let result = validator.validate_input("test_tool", &schema, &input).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e.error_type, SchemaErrorType::TypeMismatch)));
    }

    #[test]
    fn test_schema_registry() {
        let mut registry = SchemaRegistry::new();
        
        let schema = ToolSchema::new(
            "test_tool".to_string(),
            create_test_input_schema(),
            create_test_output_schema(),
        );
        
        registry.register(schema);
        assert_eq!(registry.len(), 1);
        assert!(registry.get("test_tool").is_some());
    }

    #[test]
    fn test_schema_validation_result() {
        let valid_result = SchemaValidationResult::valid();
        assert!(valid_result.is_valid);
        assert!(valid_result.errors.is_empty());

        let invalid_result = SchemaValidationResult::invalid(vec![SchemaValidationError {
            path: "/test".to_string(),
            message: "测试错误".to_string(),
            error_type: SchemaErrorType::TypeMismatch,
            actual_value: None,
            expected_value: None,
        }]);
        assert!(!invalid_result.is_valid);
        assert_eq!(invalid_result.errors.len(), 1);
    }

    #[test]
    fn test_validator_builder() {
        let validator = SchemaValidatorBuilder::new()
            .strict_mode(true)
            .collect_all_errors(true)
            .build();

        assert!(validator.config.strict_mode);
        assert!(validator.config.collect_all_errors);
    }
}
