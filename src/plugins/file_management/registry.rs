//! Tool registration framework for file management plugin

use super::ac_automaton::{AhoCorasickMatcher, AutomatonConfig, Pattern};
use super::plugin::FileManagementConfig;
use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::tools::{BasicTool, ToolExecutor, ToolNode};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

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

        // Register granular classification flow tools
        if let Ok(mut flow_tools) = self.register_classification_flow_tools() {
            tools.append(&mut flow_tools);
        }

        if let Ok(tool) = self.register_batch_processor_tool() {
            tools.push(tool);
        }

        // 4. Human interaction tools
        if let Ok(tool) = self.register_human_decision_tool() {
            tools.push(tool);
        }

        // 5. Result review and confirmation tools
        if let Ok(tool) = self.register_result_review_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_batch_confirmation_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_result_confirmation_tool() {
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
        self.registered_tools
            .insert("text-processor".to_string(), tool_arc.clone());

        Ok(tool_arc)
    }

    /// Register all classification flow tools (granular steps)
    fn register_classification_flow_tools(&mut self) -> Result<Vec<Arc<dyn ToolNode>>> {
        let mut tools: Vec<Arc<dyn ToolNode>> = Vec::new();

        // 1. Rule Loader
        let tool = Arc::new(super::classification_flow::RuleLoaderTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 2. Rule Preprocessor
        let tool = Arc::new(super::classification_flow::RulePreprocessorTool::new(self.config.enable_chinese_processing));
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 3. Automaton Builder
        let tool = Arc::new(super::classification_flow::AutomatonBuilderTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 4. Directory Scanner
        let tool = Arc::new(super::classification_flow::DirectoryScannerTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 5. Folder Name Preprocessor
        let tool = Arc::new(super::classification_flow::FolderNamePreprocessorTool::new(self.config.enable_chinese_processing));
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 6. Parallel Matcher
        let tool = Arc::new(super::classification_flow::ParallelMatcherTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 7. Score Calculator
        let tool = Arc::new(super::classification_flow::ScoreCalculatorTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 8. Ambiguity Detector
        let tool = Arc::new(super::classification_flow::AmbiguityDetectorTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 9. Result Merger
        let tool = Arc::new(super::classification_flow::ResultMergerTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 10. Experimental Check
        let tool = Arc::new(super::classification_flow::ExperimentalCheckTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        // 13. Report Generator
        let tool = Arc::new(super::classification_flow::ReportGeneratorTool);
        self.registered_tools.insert(tool.name().to_string(), tool.clone());
        tools.push(tool);

        Ok(tools)
    }

    /// Register the AC matcher tool
    fn register_ac_matcher_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering AC matcher tool");

        let tool_info = ToolInfo {
            name: "ac-matcher".to_string(),
            version: "1.0.0".to_string(),
            description: "Aho-Corasick multi-pattern string matching".to_string(),
            category: Some("pattern-matching".to_string()),
            tags: vec![
                "pattern".to_string(),
                "matching".to_string(),
                "aho-corasick".to_string(),
            ],
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
                    "find_overlapping": {"type": "boolean", "default": false},
                    "experimental_mode": {"type": "boolean", "default": false, "description": "Run in experimental mode (simulation only)"}
                },
                "required": ["text", "patterns"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "matches": {"type": "array"},
                    "total_matches": {"type": "number"},
                    "categories_found": {"type": "array", "items": {"type": "string"}},
                    "statistics": {"type": "object"},
                    "experimental_mode": {"type": "boolean", "description": "Whether the operation was run in experimental mode"}
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
        self.registered_tools
            .insert("ac-matcher".to_string(), tool_arc.clone());

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
        self.registered_tools
            .insert("folder-classifier".to_string(), tool_arc.clone());

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
                                "operation_type": {"type": "string", "enum": ["Move", "Copy", "Link", "HardLink"], "default": "Move"}
                            },
                            "required": ["source", "destination"]
                        }
                    },
                    "conflict_resolution": {"type": "string", "enum": ["Skip", "Overwrite", "Rename", "Fail", "Ask", "Merge", "KeepBoth", "KeepNewer", "KeepLarger"], "default": "Rename"},
                    "check_disk_space": {"type": "boolean", "default": true},
                    "create_directories": {"type": "boolean", "default": true},
                    "experimental_mode": {"type": "boolean", "default": false}
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

        let executor = Arc::new(FileMoverExecutor::new(self.config.clone()));

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
        self.registered_tools
            .insert("file-mover".to_string(), tool_arc.clone());

        Ok(tool_arc)
    }

    /// Register the folder merger tool
    fn register_folder_merger_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering folder merger tool");

        let tool_info = ToolInfo {
            name: "folder-merger".to_string(),
            version: "1.0.0".to_string(),
            description: "Intelligent folder merging with duplicate handling".to_string(),
            category: Some("file-operations".to_string()),
            tags: vec![
                "folder".to_string(),
                "merge".to_string(),
                "duplicate".to_string(),
            ],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "source_directories": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of source directory paths to analyze for merging"
                    },
                    "merge_strategy": {
                        "type": "string",
                        "enum": ["SizeBased", "DateBased", "Manual", "Intelligent"],
                        "default": "SizeBased",
                        "description": "Strategy for determining merge direction"
                    },
                    "duplicate_handling": {
                        "type": "string",
                        "enum": ["Skip", "Rename", "KeepNewer", "KeepLarger", "Merge"],
                        "default": "Rename",
                        "description": "How to handle duplicate files during merge"
                    },
                    "max_recursion_depth": {
                        "type": "number",
                        "default": 10,
                        "description": "Maximum recursion depth for directory traversal"
                    },
                    "min_confidence_threshold": {
                        "type": "number",
                        "default": 0.7,
                        "minimum": 0.0,
                        "maximum": 1.0,
                        "description": "Minimum confidence score for automatic merge decisions"
                    },
                    "experimental_mode": {
                        "type": "boolean",
                        "default": false,
                        "description": "Run in experimental mode (dry run) without making actual changes"
                    }
                },
                "required": ["source_directories"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "comparison_result": {
                        "type": "object",
                        "properties": {
                            "common_folders": {"type": "array"},
                            "unique_folders": {"type": "array"},
                            "total_folders_analyzed": {"type": "number"},
                            "total_size_bytes": {"type": "number"},
                            "merge_recommendations": {"type": "array"}
                        }
                    },
                    "merge_result": {
                        "type": "object",
                        "properties": {
                            "total_operations": {"type": "number"},
                            "successful_operations": {"type": "number"},
                            "failed_operations": {"type": "number"},
                            "total_bytes_moved": {"type": "number"},
                            "duration_ms": {"type": "number"},
                            "folders_merged": {"type": "number"}
                        }
                    },
                    "experimental_mode": {"type": "boolean"}
                }
            }),
            plugin_name: Some(self.plugin_info.name.clone()),
            dependencies: Vec::new(),
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let executor = Arc::new(FolderMergerExecutor::new(self.config.clone()));

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
        self.registered_tools
            .insert("folder-merger".to_string(), tool_arc.clone());

        Ok(tool_arc)
    }

    /// Register the batch processor tool
    fn register_batch_processor_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering batch processor tool");

        let batch_tool = super::batch_processor_tool::BatchProcessorTool::new(
            self.config.clone(),
            self.plugin_info.clone(),
        );

        let tool = batch_tool.create_tool()?;
        let tool_arc = Arc::new(tool);
        self.registered_tools
            .insert("batch-processor".to_string(), tool_arc.clone());

        Ok(tool_arc)
    }

    /// Register the human decision tool
    fn register_human_decision_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering human decision tool");

        let tool = super::human_decision_tool::create_human_decision_tool(
            self.config.clone(),
            self.plugin_info.clone(),
        )?;

        let tool_arc = Arc::new(tool);
        self.registered_tools
            .insert("human-decision".to_string(), tool_arc.clone());

        Ok(tool_arc)
    }

    /// Register the result review tool
    fn register_result_review_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering result review tool");

        let tool = super::result_review_tool::ResultReviewTool::with_default_config();
        let tool_arc = Arc::new(tool);
        self.registered_tools
            .insert("result-reviewer".to_string(), tool_arc.clone());

        Ok(tool_arc)
    }

    /// Register the batch confirmation tool
    fn register_batch_confirmation_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering batch confirmation tool");

        let tool = super::batch_confirmation_tool::BatchConfirmationTool::with_default_config();
        let tool_arc = Arc::new(tool);
        self.registered_tools
            .insert("batch-confirmer".to_string(), tool_arc.clone());

        Ok(tool_arc)
    }

    /// Register the comprehensive result confirmation tool
    fn register_result_confirmation_tool(&mut self) -> Result<Arc<dyn ToolNode>> {
        debug!("Registering comprehensive result confirmation tool");

        let tool = super::result_confirmation_tool::ResultConfirmationTool::with_default_config();
        let tool_arc = Arc::new(tool);
        self.registered_tools
            .insert("result-confirmer".to_string(), tool_arc.clone());

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
                .ok_or_else(|| {
                    WorkflowError::validation(format!(
                        "patterns[{}].pattern must be a string",
                        index
                    ))
                })?;

            let category = pattern_obj
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    WorkflowError::validation(format!(
                        "patterns[{}].category must be a string",
                        index
                    ))
                })?;

            let score = pattern_obj
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);

            patterns.push(Pattern::new(pattern_str, category, score, index));
        }

        Ok(patterns)
    }

    /// Build automaton from patterns
    fn build_automaton(
        &self,
        patterns: Vec<Pattern>,
        case_sensitive: bool,
        find_overlapping: bool,
    ) -> Result<AhoCorasickMatcher> {
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
                .map_err(|e| {
                    WorkflowError::tool(format!(
                        "Failed to add pattern '{}': {}",
                        pattern.pattern, e
                    ))
                })?;
        }

        // Build the automaton
        matcher
            .build()
            .map_err(|e| WorkflowError::tool(format!("Failed to build automaton: {}", e)))?;

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
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        debug!("Executing AC matcher tool with parameters: {}", params);

        // Check if we're in experimental mode
        let experimental_mode = params
            .get("experimental_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if experimental_mode {
            info!("Running AC matcher tool in experimental mode");
        }

        // Extract parameters
        let text = params.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
            WorkflowError::validation("text parameter is required and must be a string")
        })?;

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
                },
                "experimental_mode": experimental_mode
            }));
        }

        debug!("Building automaton with {} patterns", patterns.len());

        // In experimental mode, log what would be done
        if experimental_mode {
            debug!(
                "Experimental mode: Would build automaton with {} patterns for text of length {}",
                patterns.len(),
                text.chars().count()
            );
            debug!("Experimental mode: Would search for patterns with case_sensitive={}, find_overlapping={}", 
                   case_sensitive, find_overlapping);
        }

        // Build automaton
        let matcher = self.build_automaton(patterns, case_sensitive, find_overlapping)?;

        // Find matches
        let matches = if find_overlapping {
            matcher.find_overlapping_matches(text)
        } else {
            matcher.find_matches(text)
        }
        .map_err(|e| WorkflowError::tool(format!("Failed to find matches: {}", e)))?;

        debug!(
            "Found {} matches in text of length {}",
            matches.len(),
            text.chars().count()
        );

        // Get statistics
        let statistics = matcher
            .get_match_statistics(text)
            .map_err(|e| WorkflowError::tool(format!("Failed to get match statistics: {}", e)))?;

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
            },
            "experimental_mode": experimental_mode
        });

        if experimental_mode {
            info!(
                "AC matcher experimental mode completed: {} matches would be found",
                statistics.total_matches
            );
        } else {
            info!(
                "AC matcher completed successfully: {} matches found",
                statistics.total_matches
            );
        }

        Ok(response)
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Validate text parameter
        if !params.get("text").and_then(|v| v.as_str()).is_some() {
            return Err(WorkflowError::validation(
                "text parameter is required and must be a string",
            ));
        }

        // Validate patterns parameter
        let patterns_value = params
            .get("patterns")
            .ok_or_else(|| WorkflowError::validation("patterns parameter is required"))?;

        let patterns_array = patterns_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("patterns must be an array"))?;

        if patterns_array.is_empty() {
            return Err(WorkflowError::validation("patterns array cannot be empty"));
        }

        // Validate each pattern
        for (index, pattern_obj) in patterns_array.iter().enumerate() {
            if !pattern_obj.is_object() {
                return Err(WorkflowError::validation(format!(
                    "patterns[{}] must be an object",
                    index
                )));
            }

            // Check required fields
            if !pattern_obj
                .get("pattern")
                .and_then(|v| v.as_str())
                .is_some()
            {
                return Err(WorkflowError::validation(format!(
                    "patterns[{}].pattern is required and must be a string",
                    index
                )));
            }

            if !pattern_obj
                .get("category")
                .and_then(|v| v.as_str())
                .is_some()
            {
                return Err(WorkflowError::validation(format!(
                    "patterns[{}].category is required and must be a string",
                    index
                )));
            }

            // Validate optional score field
            if let Some(score_value) = pattern_obj.get("score") {
                if !score_value.is_number() {
                    return Err(WorkflowError::validation(format!(
                        "patterns[{}].score must be a number",
                        index
                    )));
                }
            }
        }

        // Validate optional boolean parameters
        if let Some(case_sensitive) = params.get("case_sensitive") {
            if !case_sensitive.is_boolean() {
                return Err(WorkflowError::validation(
                    "case_sensitive must be a boolean",
                ));
            }
        }

        if let Some(find_overlapping) = params.get("find_overlapping") {
            if !find_overlapping.is_boolean() {
                return Err(WorkflowError::validation(
                    "find_overlapping must be a boolean",
                ));
            }
        }

        if let Some(experimental_mode) = params.get("experimental_mode") {
            if !experimental_mode.is_boolean() {
                return Err(WorkflowError::validation(
                    "experimental_mode must be a boolean",
                ));
            }
        }

        Ok(())
    }
}

/// File mover executor that implements actual file operations
pub struct FileMoverExecutor {
    config: FileManagementConfig,
}

impl FileMoverExecutor {
    pub fn new(config: FileManagementConfig) -> Self {
        Self { config }
    }

    /// Parse file operations from parameters
    fn parse_operations(
        &self,
        operations_value: &Value,
    ) -> Result<Vec<(String, String, super::utils::FileOperationType)>> {
        let operations_array = operations_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("operations must be an array"))?;

        let mut operations = Vec::new();

        for (index, op_obj) in operations_array.iter().enumerate() {
            let op_obj = op_obj.as_object().ok_or_else(|| {
                WorkflowError::validation(format!("operations[{}] must be an object", index))
            })?;

            let source = op_obj
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    WorkflowError::validation(format!(
                        "operations[{}].source is required and must be a string",
                        index
                    ))
                })?;

            let destination = op_obj
                .get("destination")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    WorkflowError::validation(format!(
                        "operations[{}].destination is required and must be a string",
                        index
                    ))
                })?;

            let operation_type_str = op_obj
                .get("operation_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Move");

            let operation_type = match operation_type_str {
                "Move" => super::utils::FileOperationType::Move,
                "Copy" => super::utils::FileOperationType::Copy,
                "Link" => super::utils::FileOperationType::Link,
                "HardLink" => super::utils::FileOperationType::HardLink,
                _ => {
                    return Err(WorkflowError::validation(format!(
                        "operations[{}].operation_type must be one of: Move, Copy, Link, HardLink",
                        index
                    )))
                }
            };

            operations.push((source.to_string(), destination.to_string(), operation_type));
        }

        Ok(operations)
    }

    /// Parse conflict resolution strategy
    fn parse_conflict_resolution(&self, params: &Value) -> super::utils::ConflictResolution {
        let conflict_str = params
            .get("conflict_resolution")
            .and_then(|v| v.as_str())
            .unwrap_or("Rename");

        match conflict_str {
            "Skip" => super::utils::ConflictResolution::Skip,
            "Overwrite" => super::utils::ConflictResolution::Overwrite,
            "Rename" => super::utils::ConflictResolution::Rename,
            "Fail" => super::utils::ConflictResolution::Fail,
            "Ask" => super::utils::ConflictResolution::Ask,
            "Merge" => super::utils::ConflictResolution::Merge,
            "KeepBoth" => super::utils::ConflictResolution::KeepBoth,
            "KeepNewer" => super::utils::ConflictResolution::KeepNewer,
            "KeepLarger" => super::utils::ConflictResolution::KeepLarger,
            _ => super::utils::ConflictResolution::Rename, // Default fallback
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for FileMoverExecutor {
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        debug!("Executing file mover tool with parameters: {}", params);

        // Parse operations
        let operations_value = params
            .get("operations")
            .ok_or_else(|| WorkflowError::validation("operations parameter is required"))?;

        let operations = self.parse_operations(operations_value)?;

        if operations.is_empty() {
            return Ok(json!({
                "operations_completed": 0,
                "operations_failed": 0,
                "operations_skipped": 0,
                "total_bytes_moved": 0,
                "duration_ms": 0,
                "errors": [],
                "preflight_check": {
                    "total_operations": 0,
                    "validation_errors": [],
                    "total_estimated_bytes": 0,
                    "is_valid": true
                }
            }));
        }

        // Parse configuration
        let conflict_resolution = self.parse_conflict_resolution(&params);
        let check_disk_space = params
            .get("check_disk_space")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let create_directories = params
            .get("create_directories")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Check if we're in experimental mode (check for experimental_mode parameter)
        let experimental_mode = params
            .get("experimental_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Create file operation manager
        let file_manager = super::utils::FileOperationManager::with_config(
            self.config.temp_directory.clone(),
            experimental_mode,
            conflict_resolution,
            create_directories,
            check_disk_space,
        );

        // Perform preflight check
        let operations_for_preflight: Vec<(PathBuf, PathBuf, super::utils::FileOperationType)> =
            operations
                .iter()
                .map(|(source, destination, op_type)| {
                    (
                        PathBuf::from(source),
                        PathBuf::from(destination),
                        op_type.clone(),
                    )
                })
                .collect();

        let preflight_result = file_manager.preflight_check(&operations_for_preflight)?;

        // If preflight check fails, return early with errors
        if !preflight_result.is_valid {
            return Ok(json!({
                "operations_completed": 0,
                "operations_failed": operations.len(),
                "operations_skipped": 0,
                "total_bytes_moved": 0,
                "duration_ms": 0,
                "errors": preflight_result.validation_errors,
                "preflight_check": preflight_result,
                "experimental_mode": experimental_mode
            }));
        }

        let start_time = std::time::Instant::now();
        let mut operations_completed = 0;
        let mut operations_failed = 0;
        let operations_skipped = 0;
        let mut total_bytes_moved = 0;
        let mut errors = Vec::new();

        // Execute operations
        for (source, destination, operation_type) in operations {
            debug!(
                "Executing {:?} operation: {} -> {}",
                operation_type, source, destination
            );

            let result = match operation_type {
                super::utils::FileOperationType::Move => {
                    file_manager.move_file(&source, &destination).await
                }
                super::utils::FileOperationType::Copy => {
                    file_manager.copy_file(&source, &destination).await
                }
                super::utils::FileOperationType::Link => {
                    file_manager.link_file(&source, &destination).await
                }
                super::utils::FileOperationType::HardLink => {
                    file_manager.hard_link_file(&source, &destination).await
                }
            };

            match result {
                Ok(op_result) => {
                    operations_completed += 1;
                    total_bytes_moved += op_result.bytes_moved;
                    debug!(
                        "Operation completed successfully: {} bytes moved",
                        op_result.bytes_moved
                    );
                }
                Err(e) => {
                    operations_failed += 1;
                    let error_info = json!({
                        "source": source,
                        "destination": destination,
                        "operation_type": format!("{:?}", operation_type),
                        "error": e.to_string(),
                        "error_category": e.category()
                    });
                    errors.push(error_info);
                    warn!("Operation failed: {} -> {}: {}", source, destination, e);
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        let response = json!({
            "operations_completed": operations_completed,
            "operations_failed": operations_failed,
            "operations_skipped": operations_skipped,
            "total_bytes_moved": total_bytes_moved,
            "duration_ms": duration_ms,
            "errors": errors,
            "experimental_mode": experimental_mode,
            "preflight_check": preflight_result
        });

        info!(
            "File mover completed: {} completed, {} failed, {} bytes moved in {}ms",
            operations_completed, operations_failed, total_bytes_moved, duration_ms
        );

        Ok(response)
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Validate operations parameter
        let operations_value = params
            .get("operations")
            .ok_or_else(|| WorkflowError::validation("operations parameter is required"))?;

        let operations_array = operations_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("operations must be an array"))?;

        if operations_array.is_empty() {
            return Err(WorkflowError::validation(
                "operations array cannot be empty",
            ));
        }

        // Validate each operation
        for (index, op_obj) in operations_array.iter().enumerate() {
            let op_obj = op_obj.as_object().ok_or_else(|| {
                WorkflowError::validation(format!("operations[{}] must be an object", index))
            })?;

            // Check required fields
            if !op_obj.get("source").and_then(|v| v.as_str()).is_some() {
                return Err(WorkflowError::validation(format!(
                    "operations[{}].source is required and must be a string",
                    index
                )));
            }

            if !op_obj.get("destination").and_then(|v| v.as_str()).is_some() {
                return Err(WorkflowError::validation(format!(
                    "operations[{}].destination is required and must be a string",
                    index
                )));
            }

            // Validate optional operation_type field
            if let Some(op_type) = op_obj.get("operation_type") {
                if let Some(op_type_str) = op_type.as_str() {
                    if !matches!(op_type_str, "Move" | "Copy" | "Link" | "HardLink") {
                        return Err(WorkflowError::validation(format!(
                            "operations[{}].operation_type must be one of: Move, Copy, Link, HardLink", 
                            index
                        )));
                    }
                } else {
                    return Err(WorkflowError::validation(format!(
                        "operations[{}].operation_type must be a string",
                        index
                    )));
                }
            }
        }

        // Validate optional parameters
        if let Some(conflict_resolution) = params.get("conflict_resolution") {
            if let Some(conflict_str) = conflict_resolution.as_str() {
                if !matches!(
                    conflict_str,
                    "Skip"
                        | "Overwrite"
                        | "Rename"
                        | "Fail"
                        | "Ask"
                        | "Merge"
                        | "KeepBoth"
                        | "KeepNewer"
                        | "KeepLarger"
                ) {
                    return Err(WorkflowError::validation(
                        "conflict_resolution must be one of: Skip, Overwrite, Rename, Fail, Ask, Merge, KeepBoth, KeepNewer, KeepLarger"
                    ));
                }
            } else {
                return Err(WorkflowError::validation(
                    "conflict_resolution must be a string",
                ));
            }
        }

        if let Some(check_disk_space) = params.get("check_disk_space") {
            if !check_disk_space.is_boolean() {
                return Err(WorkflowError::validation(
                    "check_disk_space must be a boolean",
                ));
            }
        }

        if let Some(create_directories) = params.get("create_directories") {
            if !create_directories.is_boolean() {
                return Err(WorkflowError::validation(
                    "create_directories must be a boolean",
                ));
            }
        }

        Ok(())
    }
}

/// Folder merger executor that implements actual folder merging operations
pub struct FolderMergerExecutor {
    config: FileManagementConfig,
}

impl FolderMergerExecutor {
    pub fn new(config: FileManagementConfig) -> Self {
        Self { config }
    }

    /// Parse merge strategy from parameters
    fn parse_merge_strategy(&self, params: &Value) -> super::utils::MergeStrategy {
        let strategy_str = params
            .get("merge_strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("SizeBased");

        match strategy_str {
            "SizeBased" => super::utils::MergeStrategy::SizeBased,
            "DateBased" => super::utils::MergeStrategy::DateBased,
            "Manual" => super::utils::MergeStrategy::Manual,
            "Intelligent" => super::utils::MergeStrategy::Intelligent,
            _ => super::utils::MergeStrategy::SizeBased, // Default fallback
        }
    }

    /// Parse duplicate handling strategy from parameters
    fn parse_duplicate_handling(&self, params: &Value) -> super::utils::DuplicateHandling {
        let handling_str = params
            .get("duplicate_handling")
            .and_then(|v| v.as_str())
            .unwrap_or("Rename");

        match handling_str {
            "Skip" => super::utils::DuplicateHandling::Skip,
            "Rename" => super::utils::DuplicateHandling::Rename,
            "KeepNewer" => super::utils::DuplicateHandling::KeepNewer,
            "KeepLarger" => super::utils::DuplicateHandling::KeepLarger,
            "Merge" => super::utils::DuplicateHandling::Merge,
            _ => super::utils::DuplicateHandling::Rename, // Default fallback
        }
    }

    /// Parse source directories from parameters
    fn parse_source_directories(&self, params: &Value) -> Result<Vec<String>> {
        let directories_value = params
            .get("source_directories")
            .ok_or_else(|| WorkflowError::validation("source_directories parameter is required"))?;

        let directories_array = directories_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("source_directories must be an array"))?;

        if directories_array.is_empty() {
            return Err(WorkflowError::validation(
                "source_directories array cannot be empty",
            ));
        }

        let mut directories = Vec::new();
        for (index, dir_value) in directories_array.iter().enumerate() {
            let dir_str = dir_value.as_str().ok_or_else(|| {
                WorkflowError::validation(format!("source_directories[{}] must be a string", index))
            })?;
            directories.push(dir_str.to_string());
        }

        Ok(directories)
    }

    /// Create folder merger configuration from parameters
    fn create_merger_config(
        &self,
        params: &Value,
        experimental_mode: bool,
    ) -> super::utils::FolderMergerConfig {
        let merge_strategy = self.parse_merge_strategy(params);
        let duplicate_handling = self.parse_duplicate_handling(params);

        let max_recursion_depth = params
            .get("max_recursion_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let min_confidence_threshold = params
            .get("min_confidence_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        super::utils::FolderMergerConfig {
            merge_strategy,
            duplicate_handling,
            max_recursion_depth,
            min_confidence_threshold,
            enable_size_based_decisions: true,
            enable_date_based_decisions: true,
            dry_run: experimental_mode,
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for FolderMergerExecutor {
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        debug!("Executing folder merger tool with parameters: {}", params);

        // Parse source directories
        let source_directories = self.parse_source_directories(&params)?;

        // Check if we're in experimental mode
        let experimental_mode = params
            .get("experimental_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Create merger configuration
        let merger_config = self.create_merger_config(&params, experimental_mode);
        let folder_merger = super::utils::FolderMerger::with_config(merger_config);

        // Perform folder comparison
        debug!(
            "Comparing folders across {} source directories",
            source_directories.len()
        );
        let comparison_result = folder_merger
            .compare_folders(&source_directories)
            .map_err(|e| WorkflowError::tool(format!("Failed to compare folders: {}", e)))?;

        debug!(
            "Folder comparison complete: {} common folders, {} unique folders",
            comparison_result.common_folders.len(),
            comparison_result.unique_folders.len()
        );

        // If no common folders found, return comparison result only
        if comparison_result.common_folders.is_empty() {
            return Ok(json!({
                "comparison_result": comparison_result,
                "merge_result": null,
                "experimental_mode": experimental_mode,
                "message": "No common folders found for merging"
            }));
        }

        // Create file operation manager for merge operations
        let file_operation_manager = super::utils::FileOperationManager::with_config(
            self.config.temp_directory.clone(),
            experimental_mode,                       // dry_run mode
            super::utils::ConflictResolution::Merge, // Use merge resolution for folder operations
            true,                                    // create_directories
            true,                                    // check_disk_space
        );

        // Execute merge operations
        debug!(
            "Executing merge operations for {} common folders",
            comparison_result.common_folders.len()
        );
        let merge_result = folder_merger
            .execute_merge_operations(&comparison_result, &file_operation_manager)
            .await
            .map_err(|e| {
                WorkflowError::tool(format!("Failed to execute merge operations: {}", e))
            })?;

        info!(
            "Folder merger completed: {} folders merged, {} operations performed, {} bytes moved in {}ms",
            merge_result.folders_merged,
            merge_result.total_operations,
            merge_result.total_bytes_moved,
            merge_result.duration_ms
        );

        // Prepare response
        let response = json!({
            "comparison_result": comparison_result,
            "merge_result": merge_result,
            "experimental_mode": experimental_mode
        });

        Ok(response)
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Validate source_directories parameter
        let directories_value = params
            .get("source_directories")
            .ok_or_else(|| WorkflowError::validation("source_directories parameter is required"))?;

        let directories_array = directories_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("source_directories must be an array"))?;

        if directories_array.is_empty() {
            return Err(WorkflowError::validation(
                "source_directories array cannot be empty",
            ));
        }

        // Validate each directory path
        for (index, dir_value) in directories_array.iter().enumerate() {
            if !dir_value.is_string() {
                return Err(WorkflowError::validation(format!(
                    "source_directories[{}] must be a string",
                    index
                )));
            }
        }

        // Validate optional merge_strategy parameter
        if let Some(strategy) = params.get("merge_strategy") {
            if let Some(strategy_str) = strategy.as_str() {
                if !matches!(
                    strategy_str,
                    "SizeBased" | "DateBased" | "Manual" | "Intelligent"
                ) {
                    return Err(WorkflowError::validation(
                        "merge_strategy must be one of: SizeBased, DateBased, Manual, Intelligent",
                    ));
                }
            } else {
                return Err(WorkflowError::validation("merge_strategy must be a string"));
            }
        }

        // Validate optional duplicate_handling parameter
        if let Some(handling) = params.get("duplicate_handling") {
            if let Some(handling_str) = handling.as_str() {
                if !matches!(
                    handling_str,
                    "Skip" | "Rename" | "KeepNewer" | "KeepLarger" | "Merge"
                ) {
                    return Err(WorkflowError::validation(
                        "duplicate_handling must be one of: Skip, Rename, KeepNewer, KeepLarger, Merge"
                    ));
                }
            } else {
                return Err(WorkflowError::validation(
                    "duplicate_handling must be a string",
                ));
            }
        }

        // Validate optional numeric parameters
        if let Some(depth) = params.get("max_recursion_depth") {
            if !depth.is_number() {
                return Err(WorkflowError::validation(
                    "max_recursion_depth must be a number",
                ));
            }
            if let Some(depth_val) = depth.as_u64() {
                if depth_val == 0 || depth_val > 100 {
                    return Err(WorkflowError::validation(
                        "max_recursion_depth must be between 1 and 100",
                    ));
                }
            }
        }

        if let Some(threshold) = params.get("min_confidence_threshold") {
            if let Some(threshold_val) = threshold.as_f64() {
                if threshold_val < 0.0 || threshold_val > 1.0 {
                    return Err(WorkflowError::validation(
                        "min_confidence_threshold must be between 0.0 and 1.0",
                    ));
                }
            } else {
                return Err(WorkflowError::validation(
                    "min_confidence_threshold must be a number",
                ));
            }
        }

        // Validate optional boolean parameters
        if let Some(experimental) = params.get("experimental_mode") {
            if !experimental.is_boolean() {
                return Err(WorkflowError::validation(
                    "experimental_mode must be a boolean",
                ));
            }
        }

        Ok(())
    }
}

/// Placeholder executor for tools that will be implemented in later tasks
#[allow(dead_code)]
struct PlaceholderExecutor {
    tool_name: String,
}

impl PlaceholderExecutor {
    #[allow(dead_code)]
    fn new<S: Into<String>>(tool_name: S) -> Self {
        Self {
            tool_name: tool_name.into(),
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for PlaceholderExecutor {
    async fn execute(
        &self,
        params: Value,
        _context: crate::core::ExecutionContext,
    ) -> Result<Value> {
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
