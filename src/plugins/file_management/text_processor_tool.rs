//! Text Processor Tool implementation
//!
//! This module provides a workflow tool for text processing operations including
//! normalization, Chinese text processing, and pinyin conversion.

use super::error::FileManagementResult;
use super::utils::{
    ChineseTextType, MixedTextResult, PinyinResult, PinyinStyle, TextNormalizationConfig,
    TextProcessor,
};
use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::tools::ToolNode;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};

/// Text processing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextOperation {
    NormalizeCase,
    NormalizeWhitespace,
    RemoveSpaces,
    RemovePunctuation,
    NormalizeUnicode,
    FilterCharacters,
    PreserveAlphanumericOnly,
    ConvertTraditional,
    GeneratePinyin,
    CreateCombinations,
    SegmentText,
    ProcessMixed,
    ComprehensiveNormalization,
}

/// Chinese processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChineseProcessingConfig {
    pub pinyin_style: PinyinStyle,
    pub generate_combinations: bool,
    pub include_tones: bool,
    pub convert_traditional: bool,
}

impl Default for ChineseProcessingConfig {
    fn default() -> Self {
        Self {
            pinyin_style: PinyinStyle::Normal,
            generate_combinations: true,
            include_tones: false,
            convert_traditional: true,
        }
    }
}

/// Parameters for text processor tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProcessorParams {
    pub text: String,
    pub operations: Vec<TextOperation>,
    pub normalization_config: Option<TextNormalizationConfig>,
    pub chinese_processing: Option<ChineseProcessingConfig>,
    pub output_format: Option<TextOutputFormat>,
    pub experimental_mode: Option<bool>,
}

/// Output format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextOutputFormat {
    Simple,     // Just the processed text
    Detailed,   // Include metadata and variants
    Structured, // Full structured result
}

impl Default for TextOutputFormat {
    fn default() -> Self {
        TextOutputFormat::Simple
    }
}

/// Text processor tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProcessorResult {
    pub original: String,
    pub processed: String,
    pub operations_applied: Vec<String>,
    pub pinyin_result: Option<PinyinResult>,
    pub mixed_text_result: Option<MixedTextResult>,
    pub segments: Option<Vec<String>>,
    pub chinese_type: Option<ChineseTextType>,
    pub metadata: HashMap<String, Value>,
    pub processing_time_ms: u64,
    pub experimental_mode: bool,
}

/// Text Processor Tool implementation
pub struct TextProcessorTool {
    processor: TextProcessor,
    #[allow(dead_code)]
    config: TextNormalizationConfig,
    plugin_info: Option<PluginInfo>,
}

impl TextProcessorTool {
    /// Create a new text processor tool
    pub fn new(enable_chinese: bool, config: Option<TextNormalizationConfig>) -> Self {
        let config = config.unwrap_or_default();
        let processor = TextProcessor::with_config(enable_chinese, config.clone());

        Self {
            processor,
            config,
            plugin_info: None,
        }
    }

    /// Create a new text processor tool with plugin info
    pub fn with_plugin_info(
        enable_chinese: bool,
        config: Option<TextNormalizationConfig>,
        plugin_info: PluginInfo,
    ) -> Self {
        let config = config.unwrap_or_default();
        let processor = TextProcessor::with_config(enable_chinese, config.clone());

        Self {
            processor,
            config,
            plugin_info: Some(plugin_info),
        }
    }

    /// Execute a single text operation
    fn execute_operation(&self, text: &str, operation: &TextOperation) -> String {
        match operation {
            TextOperation::NormalizeCase => self.processor.normalize_case(text),
            TextOperation::NormalizeWhitespace => self.processor.normalize_whitespace(text),
            TextOperation::RemoveSpaces => self.processor.remove_spaces(text),
            TextOperation::RemovePunctuation => self.processor.remove_punctuation(text),
            TextOperation::NormalizeUnicode => self.processor.normalize_unicode(text),
            TextOperation::FilterCharacters => self.processor.filter_characters(text),
            TextOperation::PreserveAlphanumericOnly => {
                self.processor.preserve_alphanumeric_only(text)
            }
            TextOperation::ConvertTraditional => self.processor.convert_traditional(text),
            TextOperation::ComprehensiveNormalization => self.processor.normalize_text(text),
            _ => text.to_string(), // Other operations handled separately
        }
    }

    /// Process text with all specified operations
    fn process_text(
        &self,
        params: &TextProcessorParams,
    ) -> FileManagementResult<TextProcessorResult> {
        let start_time = std::time::Instant::now();
        let experimental_mode = params.experimental_mode.unwrap_or(false);

        let mut result = TextProcessorResult {
            original: params.text.clone(),
            processed: params.text.clone(),
            operations_applied: Vec::new(),
            pinyin_result: None,
            mixed_text_result: None,
            segments: None,
            chinese_type: None,
            metadata: HashMap::new(),
            processing_time_ms: 0,
            experimental_mode,
        };

        if experimental_mode {
            debug!(
                "Running text processor in experimental mode for {} operations",
                params.operations.len()
            );
            // In experimental mode, log what would be done but still perform the operations
            // since text processing is non-destructive
            result.metadata.insert(
                "experimental_mode_note".to_string(),
                Value::String(
                    "Text processing operations are non-destructive and safe to execute"
                        .to_string(),
                ),
            );
        }

        // Apply text operations in sequence
        for operation in &params.operations {
            match operation {
                TextOperation::GeneratePinyin => {
                    if let Some(ref chinese_config) = params.chinese_processing {
                        let pinyin_result = self.processor.generate_comprehensive_pinyin(
                            &result.processed,
                            chinese_config.pinyin_style.clone(),
                        );
                        result.pinyin_result = Some(pinyin_result);
                        result.operations_applied.push("GeneratePinyin".to_string());
                    }
                }
                TextOperation::CreateCombinations => {
                    if let Some(ref _pinyin_result) = result.pinyin_result {
                        // Combinations are already included in pinyin_result
                        result
                            .operations_applied
                            .push("CreateCombinations".to_string());
                    }
                }
                TextOperation::SegmentText => {
                    let segments = self.processor.segment_text(&result.processed);
                    result.segments = Some(segments);
                    result.operations_applied.push("SegmentText".to_string());
                }
                TextOperation::ProcessMixed => {
                    let mixed_result = self.processor.process_mixed_text(&result.processed);
                    result.chinese_type = Some(mixed_result.chinese_type.clone());
                    result.mixed_text_result = Some(mixed_result);
                    result.operations_applied.push("ProcessMixed".to_string());
                }
                _ => {
                    result.processed = self.execute_operation(&result.processed, operation);
                    result.operations_applied.push(format!("{:?}", operation));
                }
            }
        }

        // Add metadata
        result.metadata.insert(
            "has_chinese".to_string(),
            Value::Bool(self.processor.contains_chinese(&params.text)),
        );
        result.metadata.insert(
            "original_length".to_string(),
            Value::Number(params.text.len().into()),
        );
        result.metadata.insert(
            "processed_length".to_string(),
            Value::Number(result.processed.len().into()),
        );

        result.processing_time_ms = start_time.elapsed().as_millis() as u64;

        debug!(
            "Text processing completed in {}ms",
            result.processing_time_ms
        );
        Ok(result)
    }

    /// Format result based on output format
    fn format_result(&self, result: TextProcessorResult, format: &TextOutputFormat) -> Value {
        match format {
            TextOutputFormat::Simple => {
                json!({
                    "processed": result.processed
                })
            }
            TextOutputFormat::Detailed => {
                json!({
                    "original": result.original,
                    "processed": result.processed,
                    "operations_applied": result.operations_applied,
                    "pinyin_variants": result.pinyin_result.as_ref().map(|p| &p.pinyin_variants),
                    "segments": result.segments,
                    "chinese_type": result.chinese_type,
                    "processing_time_ms": result.processing_time_ms
                })
            }
            TextOutputFormat::Structured => {
                serde_json::to_value(result).unwrap_or_else(|_| json!({}))
            }
        }
    }
}

#[async_trait]
impl ToolNode for TextProcessorTool {
    fn name(&self) -> &str {
        "text-processor"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        info!("Executing text processor tool");

        // Parse parameters
        let params: TextProcessorParams = serde_json::from_value(params)
            .map_err(|e| WorkflowError::ValidationError(format!("Invalid parameters: {}", e)))?;

        // Check if we're in experimental mode
        let experimental_mode = params.experimental_mode.unwrap_or(false);
        if experimental_mode {
            info!("Running text processor tool in experimental mode");
        }

        // Validate parameters
        if params.text.is_empty() {
            return Err(WorkflowError::ValidationError(
                "Text cannot be empty".to_string(),
            ));
        }

        if params.operations.is_empty() {
            return Err(WorkflowError::ValidationError(
                "At least one operation must be specified".to_string(),
            ));
        }

        // Process text
        let result = self
            .process_text(&params)
            .map_err(|e| WorkflowError::tool(format!("Text processing failed: {}", e)))?;

        // Format result
        let output_format = params.output_format.unwrap_or_default();
        let formatted_result = self.format_result(result, &output_format);

        if experimental_mode {
            info!("Text processor tool experimental mode completed successfully");
        } else {
            info!("Text processor tool completed successfully");
        }

        Ok(formatted_result)
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        let parsed_params: TextProcessorParams =
            serde_json::from_value(params.clone()).map_err(|e| {
                WorkflowError::ValidationError(format!("Parameter validation failed: {}", e))
            })?;

        // Additional validation beyond JSON schema
        if parsed_params.text.trim().is_empty() {
            return Err(WorkflowError::ValidationError(
                "Text cannot be empty".to_string(),
            ));
        }

        if parsed_params.operations.is_empty() {
            return Err(WorkflowError::ValidationError(
                "At least one operation must be specified".to_string(),
            ));
        }

        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        let now = Utc::now();
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description:
                "Text processing tool with normalization, Chinese processing, and pinyin conversion"
                    .to_string(),
            category: Some("text-processing".to_string()),
            tags: vec![
                "text".to_string(),
                "chinese".to_string(),
                "pinyin".to_string(),
                "normalization".to_string(),
            ],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to process",
                        "minLength": 1
                    },
                    "operations": {
                        "type": "array",
                        "description": "List of text processing operations to apply",
                        "items": {
                            "type": "string",
                            "enum": [
                                "NormalizeCase",
                                "NormalizeWhitespace",
                                "RemoveSpaces",
                                "RemovePunctuation",
                                "NormalizeUnicode",
                                "FilterCharacters",
                                "PreserveAlphanumericOnly",
                                "ConvertTraditional",
                                "GeneratePinyin",
                                "CreateCombinations",
                                "SegmentText",
                                "ProcessMixed",
                                "ComprehensiveNormalization"
                            ]
                        },
                        "minItems": 1
                    },
                    "normalization_config": {
                        "type": "object",
                        "description": "Text normalization configuration",
                        "properties": {
                            "normalize_case": {"type": "boolean", "default": true},
                            "normalize_whitespace": {"type": "boolean", "default": true},
                            "remove_punctuation": {"type": "boolean", "default": false},
                            "normalize_unicode": {"type": "boolean", "default": true},
                            "filter_characters": {
                                "type": "array",
                                "items": {"type": "string", "maxLength": 1}
                            },
                            "preserve_alphanumeric_only": {"type": "boolean", "default": false}
                        }
                    },
                    "chinese_processing": {
                        "type": "object",
                        "description": "Chinese text processing configuration",
                        "properties": {
                            "pinyin_style": {
                                "type": "string",
                                "enum": ["Normal", "WithTone", "WithoutTone", "FirstLetter", "Numeric"],
                                "default": "Normal"
                            },
                            "generate_combinations": {"type": "boolean", "default": true},
                            "include_tones": {"type": "boolean", "default": false},
                            "convert_traditional": {"type": "boolean", "default": true}
                        }
                    },
                    "output_format": {
                        "type": "string",
                        "enum": ["Simple", "Detailed", "Structured"],
                        "default": "Simple"
                    },
                    "experimental_mode": {
                        "type": "boolean",
                        "default": false,
                        "description": "Run in experimental mode (simulation only)"
                    }
                },
                "required": ["text", "operations"]
            }),
            return_schema: json!({
                "type": "object",
                "description": "Text processing result (format depends on output_format parameter)",
                "properties": {
                    "processed": {"type": "string"},
                    "original": {"type": "string"},
                    "operations_applied": {"type": "array", "items": {"type": "string"}},
                    "pinyin_result": {"type": "object"},
                    "mixed_text_result": {"type": "object"},
                    "segments": {"type": "array", "items": {"type": "string"}},
                    "chinese_type": {"type": "string"},
                    "metadata": {"type": "object"},
                    "processing_time_ms": {"type": "number"},
                    "experimental_mode": {"type": "boolean", "description": "Whether the operation was run in experimental mode"}
                }
            }),
            plugin_name: self.plugin_info.as_ref().map(|p| p.name.clone()),
            dependencies: vec![],
            version_requirements: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        self.plugin_info.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ExecutionContext;

    #[tokio::test]
    async fn test_text_processor_tool_basic() {
        let tool = TextProcessorTool::new(true, None);

        let params = json!({
            "text": "Hello World",
            "operations": ["NormalizeCase", "NormalizeWhitespace"],
            "output_format": "Simple"
        });

        let context = ExecutionContext::new();
        let result = tool.execute(params, context).await.unwrap();

        let processed = result.get("processed").unwrap().as_str().unwrap();
        assert_eq!(processed, "hello world");
    }

    #[tokio::test]
    async fn test_text_processor_tool_chinese() {
        let tool = TextProcessorTool::new(true, None);

        let params = json!({
            "text": "你好世界",
            "operations": ["GeneratePinyin", "ProcessMixed"],
            "chinese_processing": {
                "pinyin_style": "Normal",
                "generate_combinations": true,
                "include_tones": false,
                "convert_traditional": true
            },
            "output_format": "Detailed"
        });

        let context = ExecutionContext::new();
        let result = tool.execute(params, context).await.unwrap();

        assert!(result.get("pinyin_variants").is_some());
        assert!(result.get("chinese_type").is_some());
    }

    #[tokio::test]
    async fn test_text_processor_tool_comprehensive() {
        let tool = TextProcessorTool::new(true, None);

        let params = json!({
            "text": "  Hello, 世界!  ",
            "operations": ["ComprehensiveNormalization", "ProcessMixed", "SegmentText"],
            "output_format": "Structured"
        });

        let context = ExecutionContext::new();
        let result = tool.execute(params, context).await.unwrap();

        assert!(result.get("processed").is_some());
        assert!(result.get("segments").is_some());
        assert!(result.get("mixed_text_result").is_some());
        assert!(result.get("processing_time_ms").is_some());
    }

    #[test]
    fn test_parameter_validation() {
        let tool = TextProcessorTool::new(true, None);

        // Valid parameters
        let valid_params = json!({
            "text": "test",
            "operations": ["NormalizeCase"]
        });
        assert!(tool.validate_parameters(&valid_params).is_ok());

        // Invalid parameters - empty text
        let invalid_params = json!({
            "text": "",
            "operations": ["NormalizeCase"]
        });
        assert!(tool.validate_parameters(&invalid_params).is_err());

        // Invalid parameters - no operations
        let invalid_params = json!({
            "text": "test",
            "operations": []
        });
        assert!(tool.validate_parameters(&invalid_params).is_err());
    }

    #[test]
    fn test_tool_schema() {
        let tool = TextProcessorTool::new(true, None);
        let info = tool.get_info();

        assert_eq!(info.name, "text-processor");
        assert_eq!(info.version, "1.0.0");
        assert!(info.description.contains("Text processing"));
        assert!(info.parameters_schema.get("properties").is_some());
        assert!(info.return_schema.get("properties").is_some());
    }
}
