//! Tool registration framework for file management plugin

use crate::core::{PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::tools::{ToolNode, BasicTool, BasicToolBuilder, ToolExecutor};
use super::error::{FileManagementError, FileManagementResult};
use super::plugin::FileManagementConfig;
use serde_json::{json, Value};
use std::collections::HashMap;
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

        let tool_info = ToolInfo {
            name: "text-processor".to_string(),
            version: "1.0.0".to_string(),
            description: "Text processing including Chinese and pinyin conversion".to_string(),
            category: Some("text-processing".to_string()),
            tags: vec!["text".to_string(), "chinese".to_string(), "pinyin".to_string()],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to process"
                    },
                    "operations": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["NormalizeCase", "RemoveSpaces", "ConvertTraditional", "GeneratePinyin", "CreateCombinations"]
                        },
                        "description": "List of operations to perform"
                    },
                    "chinese_processing": {
                        "type": "object",
                        "properties": {
                            "pinyin_style": {
                                "type": "string",
                                "enum": ["Normal", "WithTone", "WithoutTone", "FirstLetter"],
                                "default": "Normal"
                            },
                            "generate_combinations": {
                                "type": "boolean",
                                "default": false
                            },
                            "include_tones": {
                                "type": "boolean",
                                "default": false
                            }
                        }
                    }
                },
                "required": ["text", "operations"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "original": {"type": "string"},
                    "processed": {"type": "string"},
                    "pinyin_variants": {"type": "array", "items": {"type": "string"}},
                    "combinations": {"type": "array"},
                    "metadata": {"type": "object"}
                }
            }),
            plugin_name: Some(self.plugin_info.name.clone()),
            dependencies: Vec::new(),
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Create placeholder executor (will be implemented in later tasks)
        let executor = Arc::new(PlaceholderExecutor::new("text-processor"));

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
                    "categories_found": {"type": "array", "items": {"type": "string"}}
                }
            }),
            plugin_name: Some(self.plugin_info.name.clone()),
            dependencies: Vec::new(),
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let executor = Arc::new(PlaceholderExecutor::new("ac-matcher"));

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

        let tool_info = ToolInfo {
            name: "folder-classifier".to_string(),
            version: "1.0.0".to_string(),
            description: "Intelligent folder classification using configurable rules".to_string(),
            category: Some("classification".to_string()),
            tags: vec!["classification".to_string(), "folder".to_string(), "ai".to_string()],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "folder_path": {
                        "type": "string",
                        "description": "Path to folder to classify"
                    },
                    "classification_rules": {
                        "description": "Classification rules (JSON object or file path)"
                    },
                    "enable_user_interaction": {
                        "type": "boolean",
                        "default": false,
                        "description": "Enable human decision for ambiguous cases"
                    },
                    "experimental_mode": {
                        "type": "boolean",
                        "default": false,
                        "description": "Run in experimental mode (no actual changes)"
                    }
                },
                "required": ["folder_path", "classification_rules"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string", "enum": ["classified", "unclassified", "pending", "error"]},
                    "category": {"type": "string"},
                    "candidates": {"type": "array"},
                    "score": {"type": "number"},
                    "folder_name": {"type": "string"},
                    "processing_time_ms": {"type": "number"}
                }
            }),
            plugin_name: Some(self.plugin_info.name.clone()),
            dependencies: vec!["text-processor".to_string(), "ac-matcher".to_string()],
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let executor = Arc::new(PlaceholderExecutor::new("folder-classifier"));

        let tool = BasicTool::builder()
            .name(&tool_info.name)
            .version(&tool_info.version)
            .description(&tool_info.description)
            .category(tool_info.category.clone().unwrap_or_default())
            .tags(tool_info.tags.clone())
            .parameters_schema(tool_info.parameters_schema.clone())
            .return_schema(tool_info.return_schema.clone())
            .plugin_info(self.plugin_info.clone())
            .dependencies(tool_info.dependencies.clone())
            .executor(executor)
            .build()?;

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
}