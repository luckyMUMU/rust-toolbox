use async_trait::async_trait;
use rt_core::{Tool, CoreError, Result, Locale};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use schemars::JsonSchema;
use crate::i18n_utils::ToolI18n;

#[derive(Debug, Deserialize, JsonSchema)]
struct MoveFolderInput {
    source: String,
    destination: String,
    #[serde(default)]
    overwrite: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
struct MoveFolderOutput {
    success: bool,
    moved_files: usize,
}

pub struct MoveFolder {
    i18n: ToolI18n,
}

impl MoveFolder {
    pub fn new() -> Self {
        Self {
            i18n: ToolI18n::new(
                include_str!("locales/tool.en.json"),
                include_str!("locales/tool.zh-CN.json"),
            ),
        }
    }
}

#[async_trait]
impl Tool for MoveFolder {
    fn name(&self) -> &str {
        "file.move_folder"
    }

    fn display_name(&self, locale: Locale) -> String {
        self.i18n.display_name(locale).to_string()
    }

    fn description(&self, locale: Locale) -> String {
        self.i18n.description(locale).to_string()
    }

    fn input_schema(&self, locale: Locale) -> Value {
        let mut schema = serde_json::to_value(schemars::schema_for!(MoveFolderInput)).unwrap();
        
        if let Some(props) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
            for (key, val) in props.iter_mut() {
                if let Some(title) = self.i18n.input_title(key, locale) {
                    val["title"] = serde_json::json!(title);
                }
            }
        }
        
        schema
    }

    fn output_schema(&self, locale: Locale) -> Value {
        let mut schema = serde_json::to_value(schemars::schema_for!(MoveFolderOutput)).unwrap();

        if let Some(props) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
            for (key, val) in props.iter_mut() {
                if let Some(title) = self.i18n.output_title(key, locale) {
                    val["title"] = serde_json::json!(title);
                }
            }
        }
        schema
    }

    fn user_guide(&self, locale: Locale) -> String {
        self.i18n.user_guide(locale).to_string()
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let args: MoveFolderInput = serde_json::from_value(input)
            .map_err(|e| CoreError::InvalidInput(format!("Failed to parse input: {}", e)))?;

        let src = Path::new(&args.source);
        let dst_root = Path::new(&args.destination);

        if !src.exists() {
            return Err(CoreError::InvalidInput(format!("Source path does not exist: {}", args.source)));
        }

        if !src.is_dir() {
            return Err(CoreError::InvalidInput(format!("Source path is not a directory: {}", args.source)));
        }

        // Calculate actual target path
        // If dst_root exists and is a directory, move INTO it.
        // Otherwise, rename src TO dst_root.
        let target_path = if dst_root.exists() && dst_root.is_dir() {
             let file_name = src.file_name().ok_or_else(|| CoreError::InvalidInput("Source path ends with ..".to_string()))?;
             dst_root.join(file_name)
        } else {
             dst_root.to_path_buf()
        };

        // Check if target path exists
        if target_path.exists() {
            if !args.overwrite {
                return Err(CoreError::InvalidInput(format!("Target path exists and overwrite is false: {:?}", target_path)));
            }
            // Remove existing target before moving
            if target_path.is_dir() {
                tokio::fs::remove_dir_all(&target_path).await
                    .map_err(|e| CoreError::ToolFailure(format!("Failed to remove existing target directory: {}", e)))?;
            } else {
                tokio::fs::remove_file(&target_path).await
                    .map_err(|e| CoreError::ToolFailure(format!("Failed to remove existing target file: {}", e)))?;
            }
        }

        // Perform move (rename)
        tokio::fs::rename(src, &target_path).await
            .map_err(|e| CoreError::ToolFailure(format!("Failed to move folder: {}", e)))?;

        // TODO: Count moved files (recursive) - for now just return 0 or implement a counter
        // Implementing a simple counter is non-trivial async recursively without walkdir or similar.
        // For this task, we focus on i18n.
        
        let output = MoveFolderOutput {
            success: true,
            moved_files: 0, // Placeholder
        };

        Ok(serde_json::to_value(output).unwrap())
    }
}
