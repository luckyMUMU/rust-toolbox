use crate::core::{ExecutionContext, ToolInfo};
use crate::error::Result;
use crate::tools::ToolNode;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use walkdir::WalkDir;

pub struct DirectoryScannerTool;

#[async_trait]
impl ToolNode for DirectoryScannerTool {
    fn name(&self) -> &str {
        "directory-scanner"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("path").is_none() {
            return Err(crate::error::WorkflowError::InvalidParameters(
                "Missing 'path' parameter".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let path_str = params["path"].as_str().unwrap();
        let recursive = params.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        let scan_type = params.get("scan_type").and_then(|v| v.as_str()).unwrap_or("files");
        
        // max_depth 1 means current directory only? WalkDir depth 0 is root, 1 is children.
        // If not recursive, we want depth 1.
        let max_depth = if recursive { usize::MAX } else { 1 };
        
        let mut items = Vec::new();
        
        // WalkDir follows symlinks by default? No.
        let walker = WalkDir::new(path_str).max_depth(max_depth);
        
        for entry in walker {
            match entry {
                Ok(entry) => {
                    // Skip the root directory itself
                    if entry.path().to_string_lossy() == path_str.to_string() {
                        continue;
                    }
                    
                    let is_file = entry.file_type().is_file();
                    let is_dir = entry.file_type().is_dir();
                    
                    let should_include = match scan_type {
                        "files" => is_file,
                        "directories" => is_dir,
                        "both" => is_file || is_dir,
                        _ => is_file,
                    };
                    
                    if should_include {
                        let path = entry.path();
                        let item_info = json!({
                            "path": path.to_string_lossy(),
                            "name": path.file_name().unwrap_or_default().to_string_lossy(),
                            "type": if is_dir { "directory" } else { "file" },
                            "extension": path.extension().map(|e: &std::ffi::OsStr| e.to_string_lossy()).unwrap_or_default(),
                            "size": entry.metadata().map(|m: std::fs::Metadata| m.len()).unwrap_or(0),
                            "created": entry.metadata().ok().and_then(|m: std::fs::Metadata| m.created().ok()).map(|t: std::time::SystemTime| format!("{:?}", t)),
                            "modified": entry.metadata().ok().and_then(|m: std::fs::Metadata| m.modified().ok()).map(|t: std::time::SystemTime| format!("{:?}", t)),
                        });
                        items.push(item_info);
                    }
                }
                Err(e) => tracing::warn!("Error scanning directory entry: {}", e),
            }
        }
        
        Ok(json!({
            "items": items,
            "count": items.len(),
            "scanned_path": path_str
        }))
    }

    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: "Scans a directory for files or subdirectories".to_string(),
            category: Some("file-system".to_string()),
            tags: vec!["scan".to_string(), "directory".to_string(), "files".to_string()],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to scan"
                    },
                    "recursive": {
                        "type": "boolean",
                        "default": false,
                        "description": "Whether to scan recursively"
                    },
                    "scan_type": {
                        "type": "string",
                        "enum": ["files", "directories", "both"],
                        "default": "files",
                        "description": "Type of items to scan"
                    }
                },
                "required": ["path"]
            }),
            return_schema: json!({
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "name": { "type": "string" },
                                "type": { "type": "string" },
                                "extension": { "type": "string" },
                                "size": { "type": "number" }
                            }
                        }
                    },
                    "count": { "type": "number" },
                    "scanned_path": { "type": "string" }
                }
            }),
            plugin_name: None, // Can be injected if needed
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
