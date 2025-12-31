//! Tool registration framework for file management plugin

use crate::core::{PluginInfo, ToolInfo, ExecutionContext};
use crate::error::{Result, WorkflowError};
use crate::tools::{ToolNode, BasicTool, BasicToolBuilder, ToolExecutor};
use super::error::{FileManagementError, FileManagementResult};
use super::plugin::FileManagementConfig;
use super::ac_automaton::{AhoCorasickMatcher, AutomatonConfig, Pattern};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn, error};

/// Registry for file management tools
pub struct FileManagementToolRegistry {
    config: FileManagementConfig,
    plugin_info: PluginInfo,
    registered_tools: HashMap<String, Arc<dyn ToolNode>>,
}

impl FileManagementToolRegistry {
    /// Create a new tool registry
    pub fn new(config: FileManagementConfig, plugin_info: PluginInfo) -> Self {
        Self {
            config,
            plugin_info,
            registered_tools: HashMap::new(),
        }
    }

    /// Register all file management tools
    pub fn register_all_tools(&mut self) -> Result<Vec<Arc<dyn ToolNode>>> {
        info!("Registering all file management tools");

        let mut tools = Vec::new();

        // Register tools in order of dependencies
        // Tools will be implemented in subsequent tasks

        // 1. Core utility tools (no dependencies)
        if let Ok(tool) = self.register_text_processor_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_ac_matcher_tool() {
            tools.push(tool);
        }

        // 2. File operation tools (depend on utilities)
        if let Ok(tool) = self.register_file_mover_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_folder_merger_tool() {
            tools.push(tool);
        }

        // 3. Higher-level tools (depend on file operations)
        if let Ok(tool) = self.register_classification_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_batch_processor_tool() {
            tools.push(tool);
        }

        // 4. Human interaction tools
        if let Ok(tool) = self.register_human_decision_tool() {
            tools.push(tool);
        }

        info!("Registered {} file management tools", tools.len());
        Ok(tools)
    }

    /// Register the text processor tool
    fn register_text_processor_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering text processor tool");

        let tool = super::text_processor_tool::TextProcessorTool::new(
            self.config.enable_chinese_processing,
            None, // Use default normalization config
        );

        let tool_arc = Arc::new(tool);
        self.registered_tools.insert("text-processor".to_string(), tool_arc.clone());
        
        Ok(tool_arc)
    }

    /// Register the AC matcher tool
    fn register_ac_matcher_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering AC matcher tool");

        let tool_info = ToolInfo {
            name: "ac-matcher".to_string(),
            version: "1.0.0".to_string(),
            description: "Aho-Corasick multi-pattern string matching".to_string(),
            category: Some("pattern-matching".to_string()),
            tags: vec!["pattern".to_string(), "matching".to_string(), "aho-corasick".to_string()],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to search in"
                    },
                    "patterns": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "pattern": {"type": "string"},
                                "category": {"type": "string"},
                                "score": {"type": "number", "default": 1.0}
                            },
                            "required": ["pattern", "category"]
                        }
                    },
                    "case_sensitive": {"type": "boolean", "default": false},
                    "find_overlapping": {"type": "boolean", "default": false}
                },
                "required": ["text", "patterns"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "matches": {"type": "array"},
                    "total_matches": {"type": "number"},
                    "categories_found": {"type": "array", "items": {"type": "string"}},
                    "statistics": {"type": "object"}
                }
            }),
            plugin_name: Some(self.plugin_info.name.clone()),
            dependencies: Vec::new(),
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let executor = Arc::new(AcMatcherExecutor::new());

        let tool = BasicTool::builder()
            .name(&tool_info.name)
            .version(&tool_info.version)
            .description(&tool_info.description)
            .category(tool_info.category.clone().unwrap_or_default())
            .tags(tool_info.tags.clone())
            .parameters_schema(tool_info.parameters_schema.clone())
            .return_schema(tool_info.return_schema.clone())
            .plugin_info(self.plugin_info.clone())
            .executor(executor)
            .build()?;

        let tool_arc = Arc::new(tool);
        self.registered_tools.insert("ac-matcher".to_string(), tool_arc.clone());
        
        Ok(tool_arc)
    }

    /// Register the classification tool
    fn register_classification_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering classification tool");

        let tool = super::classification_tool::ClassificationTool::with_plugin_info(
            self.config.enable_chinese_processing,
            self.plugin_info.clone(),
        );

        let tool_arc = Arc::new(tool);
        self.registered_tools.insert("folder-classifier".to_string(), tool_arc.clone());
        
        Ok(tool_arc)
    }

    /// Register the file mover tool
    fn register_file_mover_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering file mover tool");

        let tool_info = ToolInfo {
            name: "file-mover".to_string(),
            version: "1.0.0".to_string(),
            description: "Safe file and folder operations with conflict resolution".to_string(),
            category: Some("file-operations".to_string()),
            tags: vec!["file".to_string(), "move".to_string(), "copy".to_string()],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "operations": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "source": {"type": "string"},
                                "destination": {"type": "string"},
                                "operation_type": {"type": "string", "enum": ["Move", "Copy", "Link"], "default": "Move"}
                            },
                            "required": ["source", "destination"]
                        }
                    },
                    "conflict_resolution": {"type": "string", "enum": ["Skip", "Overwrite", "Rename", "Fail"], "default": "Rename"},
                    "check_disk_space": {"type": "boolean", "default": true},
                    "create_directories": {"type": "boolean", "default": true}
                },
                "required": ["operations"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "operations_completed": {"type": "number"},
                    "operations_failed": {"type": "number"},
                    "operations_skipped": {"type": "number"},
                    "total_bytes_moved": {"type": "number"},
                    "duration_ms": {"type": "number"},
                    "errors": {"type": "array"}
                }
            }),
            plugin_name: Some(self.plugin_info.name.clone()),
            dependencies: Vec::new(),
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let executor = Arc::new(PlaceholderExecutor::new("file-mover"));

        let tool = BasicTool::builder()
            .name(&tool_info.name)
            .version(&tool_info.version)
            .description(&tool_info.description)
            .category(tool_info.category.clone().unwrap_or_default())
            .tags(tool_info.tags.clone())
            .parameters_schema(tool_info.parameters_schema.clone())
            .return_schema(tool_info.return_schema.clone())
            .plugin_info(self.plugin_info.clone())
            .executor(executor)
            .build()?;

        let tool_arc = Arc::new(tool);
        self.registered_tools.insert("file-mover".to_string(), tool_arc.clone());
        
        Ok(tool_arc)
    }

    /// Register the folder merger tool
    fn register_folder_merger_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering folder merger tool");

        let executor = Arc::new(PlaceholderExecutor::new("folder-merger"));

        let tool = BasicTool::builder()
            .name("folder-merger")
            .version("1.0.0")
            .description("Intelligent folder merging with duplicate handling")
            .category("file-operations")
            .tags(vec!["folder", "merge", "duplicate"])
            .parameters_schema(json!({
                "type": "object",
                "properties": {
                    "source_directories": {"type": "array", "items": {"type": "string"}},
                    "merge_strategy": {"type": "string", "enum": ["SizeBased", "DateBased", "Manual"], "default": "SizeBased"},
                    "handle_duplicates": {"type": "string", "enum": ["Skip", "Rename", "Merge"], "default": "Rename"},
                    "experimental_mode": {"type": "boolean", "default": false}
                },
                "required": ["source_directories"]
            }))
            .plugin_info(self.plugin_info.clone())
            .executor(executor)
            .build()?;

        let tool_arc = Arc::new(tool);
        self.registered_tools.insert("folder-merger".to_string(), tool_arc.clone());
        
        Ok(tool_arc)
    }

    /// Register the batch processor tool
    fn register_batch_processor_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering batch processor tool");

        let executor = Arc::new(PlaceholderExecutor::new("batch-processor"));

        let tool = BasicTool::builder()
            .name("batch-processor")
            .version("1.0.0")
            .description("Generic batch processing for any tool")
            .category("batch-processing")
            .tags(vec!["batch", "parallel", "processing"])
            .parameters_schema(json!({
                "type": "object",
                "properties": {
                    "tool_name": {"type": "string", "description": "Name of tool to run in batch"},
                    "batch_items": {"type": "array", "description": "Array of parameter objects for each batch item"},
                    "max_concurrency": {"type": "number", "default": 4},
                    "continue_on_error": {"type": "boolean", "default": true}
                },
                "required": ["tool_name", "batch_items"]
            }))
            .plugin_info(self.plugin_info.clone())
            .executor(executor)
            .build()?;

        let tool_arc = Arc::new(tool);
        self.registered_tools.insert("batch-processor".to_string(), tool_arc.clone());
        
        Ok(tool_arc)
    }

    /// Register the human decision tool
    fn register_human_decision_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering human decision tool");

        let executor = Arc::new(PlaceholderExecutor::new("human-decision"));

        let tool = BasicTool::builder()
            .name("human-decision")
            .version("1.0.0")
            .description("Human decision-making for ambiguous scenarios")
            .category("human-interaction")
            .tags(vec!["human", "decision", "interactive"])
            .parameters_schema(json!({
                "type": "object",
                "properties": {
                    "decision_type": {"type": "string", "enum": ["Classification", "FileConflict", "MergeStrategy", "Custom"]},
                    "context": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string"},
                            "description": {"type": "string"},
                            "folder_name": {"type": "string"},
                            "metadata": {"type": "object"}
                        },
                        "required": ["title", "description"]
                    },
                    "options": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string"},
                                "label": {"type": "string"},
                                "description": {"type": "string"},
                                "score": {"type": "number"},
                                "recommended": {"type": "boolean", "default": false}
                            },
                            "required": ["id", "label"]
                        }
                    },
                    "timeout_seconds": {"type": "number"},
                    "default_choice": {"type": "number"}
                },
                "required": ["decision_type", "context", "options"]
            }))
            .plugin_info(self.plugin_info.clone())
            .executor(executor)
            .build()?;

        let tool_arc = Arc::new(tool);
        self.registered_tools.insert("human-decision".to_string(), tool_arc.clone());
        
        Ok(tool_arc)
    }

    /// Get a registered tool by name
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolNode>> {
        self.registered_tools.get(name).cloned()
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<Arc<dyn ToolNode>> {
        self.registered_tools.values().cloned().collect()
    }

    /// Get tool count
    pub fn tool_count(&self) -> usize {
        self.registered_tools.len()
    }
}

/// AC Matcher executor that uses the Aho-Corasick automaton
struct AcMatcherExecutor;

impl AcMatcherExecutor {
    fn new() -> Self {
        Self
    }

    /// Parse patterns from JSON input
    fn parse_patterns(&self, patterns_value: &Value) -> Result<Vec<Pattern>> {
        let patterns_array = patterns_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("patterns must be an array"))?;

        let mut patterns = Vec::new();
        for (index, pattern_obj) in patterns_array.iter().enumerate() {
            let pattern_str = pattern_obj
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| WorkflowError::validation(format!("patterns[{}].pattern must be a string", index)))?;

            let category = pattern_obj
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or_else(|| WorkflowError::validation(format!("patterns[{}].category must be a string", index)))?;

            let score = pattern_obj
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);

            patterns.push(Pattern::new(pattern_str, category, score, index));
        }

        Ok(patterns)
    }

    /// Build automaton from patterns
    fn build_automaton(&self, patterns: Vec<Pattern>, case_sensitive: bool, find_overlapping: bool) -> Result<AhoCorasickMatcher> {
        let config = AutomatonConfig {
            case_sensitive,
            find_overlapping,
            max_patterns: 10_000,
            max_pattern_length: 1000,
        };

        let mut matcher = AhoCorasickMatcher::with_config(config);

        // Add patterns to the automaton
        for pattern in patterns {
            matcher
                .add_pattern(&pattern.pattern, &pattern.category, pattern.score)
                .map_err(|e| WorkflowError::tool(format!("Failed to add pattern '{}': {}", pattern.pattern, e)))?;
        }

        // Build the automaton
        matcher.build().map_err(|e| WorkflowError::tool(format!("Failed to build automaton: {}", e)))?;

        Ok(matcher)
    }

    /// Convert pattern matches to JSON
    fn matches_to_json(&self, matches: Vec<super::ac_automaton::PatternMatch>) -> Value {
        let match_objects: Vec<Value> = matches
            .iter()
            .map(|m| {
                json!({
                    "pattern": m.pattern,
                    "category": m.category,
                    "score": m.score,
                    "pattern_id": m.pattern_id,
                    "start_pos": m.start_pos,
                    "end_pos": m.end_pos,
                    "match_length": m.match_len()
                })
            })
            .collect();

        json!(match_objects)
    }

    /// Get unique categories from matches
    fn get_categories_found(&self, matches: &[super::ac_automaton::PatternMatch]) -> Vec<String> {
        let mut categories: Vec<String> = matches
            .iter()
            .map(|m| m.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        categories.sort();
        categories
    }
}

#[async_trait::async_trait]
impl ToolExecutor for AcMatcherExecutor {
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        debug!("Executing AC matcher tool with parameters: {}", params);

        // Extract parameters
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkflowError::validation("text parameter is required and must be a string"))?;

        let patterns_value = params
            .get("patterns")
            .ok_or_else(|| WorkflowError::validation("patterns parameter is required"))?;

        let case_sensitive = params
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let find_overlapping = params
            .get("find_overlapping")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Parse patterns
        let patterns = self.parse_patterns(patterns_value)?;
        
        if patterns.is_empty() {
            return Ok(json!({
                "matches": [],
                "total_matches": 0,
                "categories_found": [],
                "statistics": {
                    "total_matches": 0,
                    "unique_patterns": 0,
                    "categories_found": [],
                    "text_length": text.chars().count(),
                    "coverage_ratio": 0.0
                }
            }));
        }

        debug!("Building automaton with {} patterns", patterns.len());

        // Build automaton
        let matcher = self.build_automaton(patterns, case_sensitive, find_overlapping)?;

        // Find matches
        let matches = if find_overlapping {
            matcher.find_overlapping_matches(text)
        } else {
            matcher.find_matches(text)
        }.map_err(|e| WorkflowError::tool(format!("Failed to find matches: {}", e)))?;

        debug!("Found {} matches in text of length {}", matches.len(), text.chars().count());

        // Get statistics
        let statistics = matcher.get_match_statistics(text).map_err(|e| WorkflowError::tool(format!("Failed to get match statistics: {}", e)))?;

        // Prepare response
        let categories_found = self.get_categories_found(&matches);
        let matches_json = self.matches_to_json(matches);

        let response = json!({
            "matches": matches_json,
            "total_matches": statistics.total_matches,
            "categories_found": categories_found,
            "statistics": {
                "total_matches": statistics.total_matches,
                "unique_patterns": statistics.unique_patterns,
                "categories_found": statistics.categories_found,
                "category_counts": statistics.category_counts,
                "total_score": statistics.total_score,
                "average_score": statistics.average_score,
                "max_score": statistics.max_score,
                "min_score": statistics.min_score,
                "text_length": statistics.text_length,
                "coverage_ratio": statistics.coverage_ratio
            }
        });

        info!("AC matcher completed successfully: {} matches found", statistics.total_matches);
        Ok(response)
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Validate text parameter
        if !params.get("text").and_then(|v| v.as_str()).is_some() {
            return Err(WorkflowError::validation("text parameter is required and must be a string"));
        }

        // Validate patterns parameter
        let patterns_value = params.get("patterns").ok_or_else(|| WorkflowError::validation("patterns parameter is required"))?;

        let patterns_array = patterns_value.as_array().ok_or_else(|| WorkflowError::validation("patterns must be an array"))?;

        if patterns_array.is_empty() {
            return Err(WorkflowError::validation("patterns array cannot be empty"));
        }

        // Validate each pattern
        for (index, pattern_obj) in patterns_array.iter().enumerate() {
            if !pattern_obj.is_object() {
                return Err(WorkflowError::validation(format!("patterns[{}] must be an object", index)));
            }

            // Check required fields
            if !pattern_obj.get("pattern").and_then(|v| v.as_str()).is_some() {
                return Err(WorkflowError::validation(format!("patterns[{}].pattern is required and must be a string", index)));
            }

            if !pattern_obj.get("category").and_then(|v| v.as_str()).is_some() {
                return Err(WorkflowError::validation(format!("patterns[{}].category is required and must be a string", index)));
            }

            // Validate optional score field
            if let Some(score_value) = pattern_obj.get("score") {
                if !score_value.is_number() {
                    return Err(WorkflowError::validation(format!("patterns[{}].score must be a number", index)));
                }
            }
        }

        // Validate optional boolean parameters
        if let Some(case_sensitive) = params.get("case_sensitive") {
            if !case_sensitive.is_boolean() {
                return Err(WorkflowError::validation("case_sensitive must be a boolean"));
            }
        }

        if let Some(find_overlapping) = params.get("find_overlapping") {
            if !find_overlapping.is_boolean() {
                return Err(WorkflowError::validation("find_overlapping must be a boolean"));
            }
        }

        Ok(())
    }
}

/// Placeholder executor for tools that will be implemented in later tasks
struct PlaceholderExecutor {
    tool_name: String,
}

impl PlaceholderExecutor {
    fn new<S: Into<String>>(tool_name: S) -> Self {
        Self {
            tool_name: tool_name.into(),
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for PlaceholderExecutor {
    async fn execute(&self, params: Value, _context: crate::core::ExecutionContext) -> Result<Value> {
        warn!("Placeholder executor called for tool: {}", self.tool_name);
        
        // Return a placeholder response indicating the tool is not yet implemented
        Ok(json!({
            "status": "not_implemented",
            "message": format!("Tool '{}' is not yet implemented", self.tool_name),
            "tool_name": self.tool_name,
            "received_params": params
        }))
    }

    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        // Placeholder validation - always passes
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PluginType;
    use tempfile::TempDir;

    fn create_test_plugin_info() -> PluginInfo {
        PluginInfo {
            name: "file-management".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Native,
            description: Some("Test plugin".to_string()),
            author: Some("Test".to_string()),
            metadata: HashMap::new(),
        }
    }

    fn create_test_config() -> FileManagementConfig {
        let temp_dir = TempDir::new().unwrap();
        FileManagementConfig {
            temp_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        }
    }

    #[test]
    fn test_registry_creation() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();
        let registry = FileManagementToolRegistry::new(config, plugin_info);
        
        assert_eq!(registry.tool_count(), 0);
    }

    #[test]
    fn test_tool_registration() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();
        let mut registry = FileManagementToolRegistry::new(config, plugin_info);
        
        let result = registry.register_all_tools();
        assert!(result.is_ok());
        
        let tools = result.unwrap();
        assert!(tools.len() > 0);
        assert_eq!(registry.tool_count(), tools.len());
    }

    #[test]
    fn test_individual_tool_registration() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();
        let mut registry = FileManagementToolRegistry::new(config, plugin_info);
        
        // Test text processor registration
        let result = registry.register_text_processor_tool();
        assert!(result.is_ok());
        
        let tool = result.unwrap();
        assert_eq!(tool.name(), "text-processor");
        assert_eq!(tool.version(), "1.0.0");
        
        // Test tool retrieval
        let retrieved_tool = registry.get_tool("text-processor");
        assert!(retrieved_tool.is_some());
    }

    #[tokio::test]
    async fn test_placeholder_executor() {
        let executor = PlaceholderExecutor::new("test-tool");
        let context = crate::core::ExecutionContext::new();
        let params = json!({"test": "value"});
        
        let result = executor.execute(params.clone(), context).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["status"], "not_implemented");
        assert_eq!(response["tool_name"], "test-tool");
        assert_eq!(response["received_params"], params);
    }

    #[tokio::test]
    async fn test_ac_matcher_executor() {
        let executor = AcMatcherExecutor::new();
        let context = crate::core::ExecutionContext::new();
        
        let params = json!({
            "text": "hello world test hello",
            "patterns": [
                {"pattern": "hello", "category": "greeting", "score": 1.0},
                {"pattern": "world", "category": "noun", "score": 0.8},
                {"pattern": "test", "category": "action", "score": 1.2}
            ],
            "case_sensitive": false,
            "find_overlapping": false
        });
        
        let result = executor.execute(params, context).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["total_matches"], 4); // hello appears twice
        assert!(response["matches"].is_array());
        assert!(response["categories_found"].is_array());
        assert!(response["statistics"].is_object());
        
        let categories = response["categories_found"].as_array().unwrap();
        assert!(categories.contains(&json!("greeting")));
        assert!(categories.contains(&json!("noun")));
        assert!(categories.contains(&json!("action")));
    }

    #[tokio::test]
    async fn test_ac_matcher_executor_overlapping() {
        let executor = AcMatcherExecutor::new();
        let context = crate::core::ExecutionContext::new();
        
        let params = json!({
            "text": "abcde",
            "patterns": [
                {"pattern": "abc", "category": "pattern1"},
                {"pattern": "bcd", "category": "pattern2"},
                {"pattern": "cde", "category": "pattern3"}
            ],
            "find_overlapping": true
        });
        
        let result = executor.execute(params, context).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["total_matches"], 3); // All three overlapping patterns
    }

    #[tokio::test]
    async fn test_ac_matcher_executor_validation() {
        let executor = AcMatcherExecutor::new();
        
        // Test missing text parameter
        let params = json!({
            "patterns": [{"pattern": "test", "category": "test"}]
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_err());
        
        // Test missing patterns parameter
        let params = json!({
            "text": "test"
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_err());
        
        // Test empty patterns array
        let params = json!({
            "text": "test",
            "patterns": []
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_err());
        
        // Test invalid pattern object
        let params = json!({
            "text": "test",
            "patterns": [{"pattern": "test"}] // missing category
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_err());
        
        // Test valid parameters
        let params = json!({
            "text": "test",
            "patterns": [{"pattern": "test", "category": "test"}]
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ac_matcher_executor_unicode() {
        let executor = AcMatcherExecutor::new();
        let context = crate::core::ExecutionContext::new();
        
        let params = json!({
            "text": "hello测试world",
            "patterns": [
                {"pattern": "hello", "category": "english"},
                {"pattern": "测试", "category": "chinese"},
                {"pattern": "world", "category": "english"}
            ]
        });
        
        let result = executor.execute(params, context).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["total_matches"], 3);
        
        let categories = response["categories_found"].as_array().unwrap();
        assert!(categories.contains(&json!("english")));
        assert!(categories.contains(&json!("chinese")));
    }

    #[tokio::test]
    async fn test_ac_matcher_executor_empty_patterns() {
        let executor = AcMatcherExecutor::new();
        let context = crate::core::ExecutionContext::new();
        
        // This should be caught by validation, but test the execution path too
        let params = json!({
            "text": "test text",
            "patterns": []
        });
        
        // Validation should fail
        let validation_result = executor.validate_parameters(&params);
        assert!(validation_result.is_err());
    }
}