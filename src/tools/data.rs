use crate::core::{ExecutionContext, ToolInfo};
use crate::error::Result;
use crate::tools::ToolNode;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool for storing data in the global execution context
pub struct DataCacheTool;

#[async_trait]
impl ToolNode for DataCacheTool {
    fn name(&self) -> &str {
        "data-cache"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("key").is_none() {
            return Err(crate::error::WorkflowError::InvalidParameters(
                "Missing 'key' parameter".to_string(),
            ));
        }
        if params.get("value").is_none() && params.get("operation").and_then(|v| v.as_str()) != Some("get") {
            return Err(crate::error::WorkflowError::InvalidParameters(
                "Missing 'value' parameter for set operation".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        let key = params["key"].as_str().unwrap().to_string();
        let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("set");

        match operation {
            "set" => {
                let value = params["value"].clone();
                // Return a special structure that the engine will recognize to update context
                Ok(json!({
                    "status": "success",
                    "key": key,
                    "value": value,
                    "__context_update": {
                        key: value
                    }
                }))
            }
            "get" => {
                // Try to get from global variables
                let value = context.global_variables.get(&key).cloned().unwrap_or(Value::Null);
                Ok(json!({
                    "status": "success",
                    "key": key,
                    "value": value
                }))
            }
            _ => Err(crate::error::WorkflowError::InvalidParameters(
                format!("Unknown operation: {}", operation),
            )),
        }
    }

    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: "Stores and retrieves data from global execution context".to_string(),
            category: Some("system".to_string()),
            tags: vec!["data".to_string(), "cache".to_string(), "context".to_string()],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string" },
                    "value": {},
                    "operation": { "type": "string", "enum": ["set", "get"], "default": "set" }
                },
                "required": ["key"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string" },
                    "value": {}
                }
            }),
            plugin_name: None,
            dependencies: vec![],
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn get_plugin_info(&self) -> Option<&crate::core::PluginInfo> {
        None
    }
}

/// Tool for transforming data using templates or mapping
pub struct DataTransformTool;

#[async_trait]
impl ToolNode for DataTransformTool {
    fn name(&self) -> &str {
        "data-transform"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("template").is_none() && params.get("mapping").is_none() {
            return Err(crate::error::WorkflowError::InvalidParameters(
                "Missing 'template' or 'mapping' parameter".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let input = params.get("input").unwrap_or(&Value::Null);

        // Filtering support
        if let Some(filter) = params.get("filter").and_then(|v| v.as_object()) {
            if let Some(array) = input.as_array() {
                let key = filter.get("key").and_then(|v| v.as_str()).unwrap_or("");
                let value = filter.get("value"); // Value to match
                
                let filtered: Vec<Value> = array.iter().filter(|item| {
                    if let Some(item_val) = item.get(key) {
                        if let Some(match_val) = value {
                            item_val == match_val
                        } else {
                            true // If value not specified, check existence
                        }
                    } else {
                        false
                    }
                }).cloned().collect();
                
                return Ok(Value::Array(filtered));
            }
        }

        if let Some(mapping) = params.get("mapping").and_then(|v| v.as_object()) {
            // Check if input is array - if so, map each item
            if let Some(array) = input.as_array() {
                let mapped: Vec<Value> = array.iter().map(|item| {
                    let mut result = serde_json::Map::new();
                    for (target_key, source_path) in mapping {
                        if let Some(source_path_str) = source_path.as_str() {
                             let value = if source_path_str == "." {
                                item.clone()
                            } else {
                                item.get(source_path_str).cloned().unwrap_or(Value::Null)
                            };
                            result.insert(target_key.clone(), value);
                        }
                    }
                    Value::Object(result)
                }).collect();
                return Ok(Value::Array(mapped));
            }

            // Simple key mapping for single object
            let mut result = serde_json::Map::new();
            for (target_key, source_path) in mapping {
                if let Some(source_path_str) = source_path.as_str() {
                    // Very simple pointer resolution (e.g., "data.user.id")
                    // For full support we'd use a JSON pointer library
                    // Here we implement a basic one or just use the input if path is "."
                    let value = if source_path_str == "." {
                        input.clone()
                    } else {
                        // TODO: Implement proper JSON pointer or use a crate
                        // For now, just top-level lookup
                        input.get(source_path_str).cloned().unwrap_or(Value::Null)
                    };
                    result.insert(target_key.clone(), value);
                }
            }
            return Ok(Value::Object(result));
        }
        
        // TODO: Template support would go here (using Handlebars/Tera)
        
        Ok(input.clone())
    }

    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: "Transforms data structure or format".to_string(),
            category: Some("system".to_string()),
            tags: vec!["data".to_string(), "transform".to_string(), "mapping".to_string()],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "input": {},
                    "mapping": { "type": "object", "description": "Key-Value mapping where value is source path" },
                    "filter": { 
                        "type": "object",
                        "properties": {
                            "key": { "type": "string" },
                            "value": {}
                        }
                    },
                    "template": { "type": "string" }
                }
            }),
            return_schema: json!({ "type": "object" }),
            plugin_name: None,
            dependencies: vec![],
            version_requirements: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn get_plugin_info(&self) -> Option<&crate::core::PluginInfo> {
        None
    }
}
