//! Human Decision Tool Implementation
//!
//! This module provides interactive decision-making capabilities for ambiguous scenarios
//! in file management workflows. It supports timeouts, default choices, and experimental mode.

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::plugins::file_management::plugin::FileManagementConfig;
use crate::plugins::file_management::utils::utils::{HumanDecisionContext, HumanDecisionOption, HumanDecisionType};
use crate::error::{Result, WorkflowError};
use crate::tools::types::{Tool, NativeToolBuilder, ToolInput, ToolOutput};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::warn;

/// Parameters for human decision tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDecisionParams {
    pub decision_type: String,
    pub context: DecisionContext,
    pub options: Option<Vec<DecisionOption>>,
    pub items: Option<Vec<Value>>,
    pub timeout_seconds: Option<u64>,
    pub default_choice: Option<usize>,
    #[serde(default)]
    pub experimental_mode: bool,
}

/// Decision context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub title: String,
    pub description: String,
    pub folder_name: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Decision option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub score: Option<f64>,
    pub recommended: bool,
}

/// Result of human decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDecisionResult {
    pub selected_option: String,
    pub decision_time_ms: u64,
    pub was_timeout: bool,
    pub user_input: Option<String>,
    pub experimental_mode: bool,
}

/// Human Decision Tool executor
pub struct HumanDecisionExecutor {
    #[allow(dead_code)]
    config: FileManagementConfig,
}

impl HumanDecisionExecutor {
    pub fn new(config: FileManagementConfig) -> Self {
        Self { config }
    }

    /// Parse decision type from string
    fn parse_decision_type(&self, decision_type_str: &str) -> Result<HumanDecisionType> {
        match decision_type_str {
            "Classification" => Ok(HumanDecisionType::Classification),
            "FileConflict" => Ok(HumanDecisionType::FileConflict),
            "MergeStrategy" => Ok(HumanDecisionType::MergeStrategy),
            "Custom" => Ok(HumanDecisionType::Custom("Custom".to_string())),
            other => Ok(HumanDecisionType::Custom(other.to_string())),
        }
    }

    /// Convert parameters to HumanDecisionContext
    fn create_decision_context(
        &self,
        params: &HumanDecisionParams,
    ) -> Result<HumanDecisionContext> {
        let decision_type = self.parse_decision_type(&params.decision_type)?;

        let mut context = HumanDecisionContext::new(
            decision_type,
            &params.context.title,
            &params.context.description,
        );

        // Add options
        if let Some(options) = &params.options {
            for option in options {
                let mut decision_option = HumanDecisionOption {
                    id: option.id.clone(),
                    label: option.label.clone(),
                    description: option.description.clone(),
                    recommended: option.recommended,
                    metadata: HashMap::new(),
                };

                // Add score to metadata if present
                if let Some(score) = option.score {
                    decision_option
                        .metadata
                        .insert("score".to_string(), json!(score));
                }

                context.options.push(decision_option);
            }
        }

        // Set timeout if specified
        if let Some(timeout) = params.timeout_seconds {
            context.timeout_seconds = Some(timeout);
        }

        // Add folder name to metadata if present
        if let Some(folder_name) = &params.context.folder_name {
            context
                .metadata
                .insert("folder_name".to_string(), json!(folder_name));
        }

        // Add all context metadata
        for (key, value) in &params.context.metadata {
            context.metadata.insert(key.clone(), value.clone());
        }

        Ok(context)
    }

    /// Present decision to user and get input
    async fn present_decision_to_user(
        &self,
        context: &HumanDecisionContext,
        default_choice: Option<usize>,
    ) -> Result<HumanDecisionResult> {
        let start_time = Instant::now();

        // Display decision context
        println!("\n{}", "=".repeat(60));
        println!("🤔 {}", context.title);
        println!("{}", "=".repeat(60));
        println!("{}", context.description);

        // Display folder name if present
        if let Some(folder_name) = context.metadata.get("folder_name") {
            if let Some(name) = folder_name.as_str() {
                println!("📁 Folder: '{}'", name);
            }
        }

        // Display additional metadata
        for (key, value) in &context.metadata {
            if key != "folder_name" {
                println!("ℹ️  {}: {}", key, value);
            }
        }

        println!();

        // Display options
        println!("Available options:");
        for (index, option) in context.options.iter().enumerate() {
            let number = index + 1;
            let recommended = if option.recommended {
                " ⭐ (recommended)"
            } else {
                ""
            };

            // Get score from metadata if present
            let score = option
                .metadata
                .get("score")
                .and_then(|v| v.as_f64())
                .map(|s| format!(" (score: {:.2})", s))
                .unwrap_or_default();

            println!("  {}. {}{}{}", number, option.label, score, recommended);

            if let Some(desc) = &option.description {
                println!("     💡 {}", desc);
            }
        }

        // Show default choice if present
        if let Some(default_idx) = default_choice {
            if default_idx < context.options.len() {
                println!(
                    "\n⏰ Default choice: {} ({})",
                    default_idx + 1,
                    context.options[default_idx].label
                );
            }
        }

        // Show timeout if present
        if let Some(timeout_secs) = context.timeout_seconds {
            println!("⏱️  Timeout: {} seconds", timeout_secs);
        }

        println!("\nEnter your choice (1-{}):", context.options.len());
        print!("> ");
        io::stdout()
            .flush()
            .map_err(|e| WorkflowError::tool(format!("Failed to flush stdout: {}", e)))?;

        // Get user input with optional timeout
        let selected_option = if let Some(timeout_secs) = context.timeout_seconds {
            self.get_user_input_with_timeout(timeout_secs, &context.options, default_choice)
                .await?
        } else {
            self.get_user_input(&context.options).await?
        };

        let decision_time = start_time.elapsed().as_millis() as u64;

        Ok(HumanDecisionResult {
            selected_option: selected_option.id,
            decision_time_ms: decision_time,
            was_timeout: false,
            user_input: Some(selected_option.label.clone()),
            experimental_mode: false,
        })
    }

    /// Get user input without timeout
    async fn get_user_input(&self, options: &[HumanDecisionOption]) -> Result<HumanDecisionOption> {
        loop {
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();

                    // Try to parse as number
                    if let Ok(choice) = input.parse::<usize>() {
                        if choice >= 1 && choice <= options.len() {
                            return Ok(options[choice - 1].clone());
                        }
                    }

                    // Try to match by option ID or label
                    for option in options {
                        if option.id.eq_ignore_ascii_case(input)
                            || option.label.eq_ignore_ascii_case(input)
                        {
                            return Ok(option.clone());
                        }
                    }

                    println!("❌ Invalid choice. Please enter a number between 1 and {} or an option name.", options.len());
                    print!("> ");
                    io::stdout().flush().map_err(|e| {
                        WorkflowError::tool(format!("Failed to flush stdout: {}", e))
                    })?;
                }
                Err(e) => {
                    return Err(WorkflowError::tool(format!(
                        "Failed to read user input: {}",
                        e
                    )));
                }
            }
        }
    }

    /// Get user input with timeout
    async fn get_user_input_with_timeout(
        &self,
        timeout_seconds: u64,
        options: &[HumanDecisionOption],
        default_choice: Option<usize>,
    ) -> Result<HumanDecisionOption> {
        let timeout_duration = Duration::from_secs(timeout_seconds);

        // Use tokio::time::timeout to handle the timeout
        match timeout(timeout_duration, self.get_user_input(options)).await {
            Ok(result) => result,
            Err(_) => {
                // Timeout occurred
                println!("\n⏰ Timeout reached!");

                if let Some(default_idx) = default_choice {
                    if default_idx < options.len() {
                        println!("Using default choice: {}", options[default_idx].label);
                        return Ok(options[default_idx].clone());
                    }
                }

                // If no default choice, use the first recommended option or first option
                let selected_option = options
                    .iter()
                    .find(|opt| opt.recommended)
                    .or_else(|| options.first())
                    .ok_or_else(|| {
                        WorkflowError::tool("No options available for timeout fallback")
                    })?;

                println!("Using fallback choice: {}", selected_option.label);
                Ok(selected_option.clone())
            }
        }
    }

    /// Simulate decision in experimental mode
    fn simulate_decision(&self, context: &HumanDecisionContext) -> Result<HumanDecisionResult> {
        // In experimental mode, automatically select the recommended option or first option
        let selected_option = context
            .options
            .iter()
            .find(|opt| opt.recommended)
            .or_else(|| context.options.first())
            .ok_or_else(|| WorkflowError::tool("No options available for decision"))?;

        println!(
            "🧪 [Experimental Mode] Auto-selected: {}",
            selected_option.label
        );

        if let Some(desc) = &selected_option.description {
            println!("   💡 {}", desc);
        }

        Ok(HumanDecisionResult {
            selected_option: selected_option.id.clone(),
            decision_time_ms: 0,
            was_timeout: false,
            user_input: Some("auto-selected in experimental mode".to_string()),
            experimental_mode: true,
        })
    }

    /// Validate parameters
    fn validate_params(&self, params: &HumanDecisionParams) -> Result<()> {
        // Validate decision type
        if params.decision_type.is_empty() {
            return Err(WorkflowError::validation("decision_type cannot be empty"));
        }

        // Validate context
        if params.context.title.is_empty() {
            return Err(WorkflowError::validation("context.title cannot be empty"));
        }

        if params.context.description.is_empty() {
            return Err(WorkflowError::validation(
                "context.description cannot be empty",
            ));
        }

        // Validate options
        if params.options.is_none() && params.items.is_none() {
            return Err(WorkflowError::validation(
                "Either options or items must be provided",
            ));
        }

        if let Some(options) = &params.options {
            if options.is_empty() {
                return Err(WorkflowError::validation(
                    "options cannot be empty if provided",
                ));
            }

            for (index, option) in options.iter().enumerate() {
                if option.id.is_empty() {
                    return Err(WorkflowError::validation(format!(
                        "options[{}].id cannot be empty",
                        index
                    )));
                }

                if option.label.is_empty() {
                    return Err(WorkflowError::validation(format!(
                        "options[{}].label cannot be empty",
                        index
                    )));
                }
            }
        }

        // Validate default choice if present
        if let Some(default_choice) = params.default_choice {
            let options_len = params.options.as_ref().map(|o| o.len()).unwrap_or(0);
            if options_len > 0 && default_choice >= options_len {
                return Err(WorkflowError::validation(format!(
                    "default_choice ({}) is out of range (0-{})",
                    default_choice,
                    options_len - 1
                )));
            }
        }

        // Validate timeout if present
        if let Some(timeout) = params.timeout_seconds {
            if timeout == 0 {
                return Err(WorkflowError::validation(
                    "timeout_seconds must be greater than 0",
                ));
            }
            if timeout > 3600 {
                warn!("Timeout of {} seconds is very long (> 1 hour)", timeout);
            }
        }

        Ok(())
    }
}

/// Create a human decision tool with the given configuration
pub fn create_human_decision_tool(
    config: FileManagementConfig,
    plugin_info: PluginInfo,
) -> Result<Tool> {
    let tool_info = ToolInfo {
        name: "human-decision".to_string(),
        version: "1.0.0".to_string(),
        description: "Human decision-making for ambiguous scenarios".to_string(),
        category: Some("human-interaction".to_string()),
        tags: vec![
            "human".to_string(),
            "decision".to_string(),
            "interactive".to_string(),
        ],
        parameters_schema: json!({
            "type": "object",
            "properties": {
                "decision_type": {
                    "type": "string",
                    "enum": ["Classification", "FileConflict", "MergeStrategy", "Custom"],
                    "description": "Type of decision being made"
                },
                "context": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string", "description": "Title of the decision"},
                        "description": {"type": "string", "description": "Detailed description of the decision"},
                        "folder_name": {"type": "string", "description": "Optional folder name context"},
                        "metadata": {"type": "object", "description": "Additional context metadata"}
                    },
                    "required": ["title", "description"]
                },
                "options": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string", "description": "Unique identifier for the option"},
                            "label": {"type": "string", "description": "Display label for the option"},
                            "description": {"type": "string", "description": "Optional detailed description"},
                            "score": {"type": "number", "description": "Optional confidence score"},
                            "recommended": {"type": "boolean", "default": false, "description": "Whether this option is recommended"}
                        },
                        "required": ["id", "label"]
                    },
                    "minItems": 1
                },
                "timeout_seconds": {
                    "type": "number",
                    "minimum": 1,
                    "maximum": 3600,
                    "description": "Optional timeout in seconds"
                },
                "default_choice": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Index of default choice (0-based)"
                },
                "experimental_mode": {
                    "type": "boolean",
                    "default": false,
                    "description": "Run in experimental mode (auto-select without user interaction)"
                }
            },
            "required": ["decision_type", "context", "options"]
        }),
        return_schema: json!({
            "type": "object",
            "properties": {
                "selected_option": {"type": "string", "description": "ID of the selected option"},
                "decision_time_ms": {"type": "number", "description": "Time taken to make decision in milliseconds"},
                "was_timeout": {"type": "boolean", "description": "Whether the decision was made due to timeout"},
                "user_input": {"type": "string", "description": "The actual user input or selection method"},
                "experimental_mode": {"type": "boolean", "description": "Whether the decision was made in experimental mode"}
            },
            "required": ["selected_option", "decision_time_ms", "was_timeout", "experimental_mode"]
        }),
        plugin_name: Some(plugin_info.name.clone()),
        dependencies: Vec::new(),
        version_requirements: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 使用 NativeToolBuilder 创建 Tool::Native
    let native_tool = NativeToolBuilder::new()
        .name(&tool_info.name)
        .version(&tool_info.version)
        .description(&tool_info.description)
        .category(tool_info.category.unwrap_or_default())
        .tags(tool_info.tags)
        .executor(move |input: ToolInput, _ctx: ExecutionContext| {
            let executor = Arc::new(HumanDecisionExecutor::new(config.clone()));
            async move {
                // 解析输入参数
                let params: HumanDecisionParams = serde_json::from_value(input.params)
                    .map_err(|e| WorkflowError::validation(format!("参数解析失败: {}", e)))?;

                // 验证参数
                executor.validate_params(&params)?;

                // 创建决策上下文
                let decision_context = executor.create_decision_context(&params)?;

                // 获取默认选择
                let default_choice = params.default_choice;

                // 执行决策
                let result = if params.experimental_mode {
                    executor.simulate_decision(&decision_context)?
                } else {
                    executor.present_decision_to_user(&decision_context, default_choice).await?
                };

                // 返回结果
                let output = serde_json::to_value(result)
                    .map_err(|e| WorkflowError::tool(format!("结果序列化失败: {}", e)))?;

                Ok(ToolOutput::success(output))
            }
        })
        .build()
        .map_err(|e| WorkflowError::tool(format!("创建人工决策工具失败: {}", e)))?;

    Ok(Tool::Native(Arc::new(native_tool)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ExecutionContext, PluginType};
    use tempfile::TempDir;

    fn create_test_config() -> FileManagementConfig {
        let temp_dir = TempDir::new().unwrap();
        FileManagementConfig {
            temp_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        }
    }

    fn create_test_plugin_info() -> PluginInfo {
        PluginInfo {
            name: "file-management".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Native,
            description: Some("Test plugin".to_string()),
            author: Some("Test".to_string()),
            homepage: Some("https://example.com".to_string()),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_create_human_decision_tool() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();

        let tool = create_human_decision_tool(config, plugin_info).unwrap();

        assert_eq!(tool.name(), "human-decision");
        assert_eq!(tool.version(), "1.0.0");
    }

    #[test]
    fn test_parameter_validation() {
        let config = create_test_config();
        let executor = HumanDecisionExecutor::new(config);

        // Valid parameters
        let valid_params = json!({
            "decision_type": "Classification",
            "context": {
                "title": "Test Decision",
                "description": "Test description",
                "metadata": {}
            },
            "options": [
                {
                    "id": "option1",
                    "label": "Option 1",
                    "recommended": true
                }
            ]
        });

        assert!(executor.validate_parameters(&valid_params).is_ok());

        // Invalid parameters - empty options
        let invalid_params = json!({
            "decision_type": "Classification",
            "context": {
                "title": "Test Decision",
                "description": "Test description",
                "metadata": {}
            },
            "options": []
        });

        assert!(executor.validate_parameters(&invalid_params).is_err());
    }

    #[tokio::test]
    async fn test_experimental_mode_execution() {
        let config = create_test_config();
        let executor = HumanDecisionExecutor::new(config);

        let params = json!({
            "decision_type": "Classification",
            "context": {
                "title": "Test Decision",
                "description": "Test description",
                "folder_name": "test_folder",
                "metadata": {}
            },
            "options": [
                {
                    "id": "option1",
                    "label": "Option 1",
                    "recommended": true
                },
                {
                    "id": "option2",
                    "label": "Option 2",
                    "recommended": false
                }
            ],
            "experimental_mode": true
        });

        let context = ExecutionContext::new();

        let result = executor.execute(params, context).await.unwrap();

        let decision_result: HumanDecisionResult = serde_json::from_value(result).unwrap();

        assert_eq!(decision_result.selected_option, "option1");
        assert!(decision_result.experimental_mode);
        assert!(!decision_result.was_timeout);
    }

    #[test]
    fn test_decision_type_parsing() {
        let config = create_test_config();
        let executor = HumanDecisionExecutor::new(config);

        assert!(matches!(
            executor.parse_decision_type("Classification").unwrap(),
            HumanDecisionType::Classification
        ));

        assert!(matches!(
            executor.parse_decision_type("FileConflict").unwrap(),
            HumanDecisionType::FileConflict
        ));

        assert!(matches!(
            executor.parse_decision_type("CustomType").unwrap(),
            HumanDecisionType::Custom(_)
        ));
    }
}
