use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::core::ToolInfo;
use crate::tools::ToolNode;
use crate::plugins::file_management::classification_tool::{
    ClassificationEngine, ClassificationOutputFormat, ClassificationRules,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

/// Pure classification tool without side effects or human interaction
pub struct FolderClassifierTool {
    engine: ClassificationEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderClassifierParams {
    pub folder_path: String,
    pub classification_rules: Value,
    pub experimental_mode: bool,
    pub output_format: Option<ClassificationOutputFormat>,
}

impl FolderClassifierTool {
    pub fn new(enable_chinese: bool) -> Self {
        Self {
            engine: ClassificationEngine::new(enable_chinese),
        }
    }

    fn load_classification_rules(&self, rules_value: &Value) -> Result<ClassificationRules> {
        // Simple wrapper around serde_json::from_value for now, 
        // assuming rules are already resolved or inline.
        // For full support, we should reuse the loading logic from classification_tool.rs
        // but that method is private. We'll duplicate strict JSON parsing here.
        serde_json::from_value(rules_value.clone())
            .map_err(|e| WorkflowError::ValidationError(format!("Invalid rules: {}", e)))
    }
}

#[async_trait]
impl ToolNode for FolderClassifierTool {
    fn name(&self) -> &str {
        "folder-classifier-pure"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        let _parsed: FolderClassifierParams = serde_json::from_value(params.clone())
            .map_err(|e| WorkflowError::ValidationError(format!("Invalid parameters: {}", e)))?;
        Ok(())
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let params: FolderClassifierParams = serde_json::from_value(params)?;
        
        let folder_path = Path::new(&params.folder_path);
        let folder_name = folder_path
            .file_name()
            .ok_or_else(|| WorkflowError::ValidationError("Invalid folder path".to_string()))?
            .to_string_lossy();

        let rules = self.load_classification_rules(&params.classification_rules)?;
        
        // We need to build the automaton. 
        // ClassificationEngine::build_automaton returns FileManagementResult.
        // We need to map it to WorkflowError.
        let automaton = self.engine.build_automaton(&rules)
            .map_err(|e| WorkflowError::tool(e.to_string()))?;
            
        let mut result = self.engine.classify_folder(&folder_name, &automaton, &rules)
            .map_err(|e| WorkflowError::tool(e.to_string()))?;

        if params.experimental_mode {
            result.metadata.insert("experimental_mode".to_string(), Value::Bool(true));
        }

        // Format output
        let format = params.output_format.unwrap_or(ClassificationOutputFormat::Detailed);
        
        // Re-implement format_result since it's private in the other file
        let output = match format {
            ClassificationOutputFormat::Simple => {
                json!({
                    "category": result.category,
                    "confidence": if result.candidates.is_empty() { 0.0 } else { result.candidates[0].confidence },
                    "status": result.status,
                    "folder_path": params.folder_path
                })
            }
            ClassificationOutputFormat::Detailed => {
                json!({
                    "status": result.status,
                    "category": result.category,
                    "score": result.score,
                    "candidates": result.candidates.iter().take(3).collect::<Vec<_>>(),
                    "folder_name": result.folder_name,
                    "folder_path": params.folder_path,
                    "processing_time_ms": result.processing_time_ms
                })
            }
            ClassificationOutputFormat::Full => {
                let mut full_result = serde_json::to_value(result)?;
                if let Some(obj) = full_result.as_object_mut() {
                    obj.insert("folder_path".to_string(), Value::String(params.folder_path));
                }
                full_result
            }
        };

        Ok(output)
    }

    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: "Classifies a folder based on rules (pure function)".to_string(),
            category: Some("classification".to_string()),
            tags: vec!["classify".to_string(), "folder".to_string(), "rules".to_string()],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "folder_path": { "type": "string" },
                    "classification_rules": { "type": "object" },
                    "experimental_mode": { "type": "boolean", "default": false },
                    "output_format": { "type": "string", "enum": ["Simple", "Detailed", "Full"] }
                },
                "required": ["folder_path", "classification_rules"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "status": { "type": "string" },
                    "category": { "type": "string" },
                    "score": { "type": "number" }
                }
            }),
            plugin_name: None,
            dependencies: vec![],
            version_requirements: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn get_plugin_info(&self) -> Option<&crate::core::PluginInfo> {
        None
    }
}
