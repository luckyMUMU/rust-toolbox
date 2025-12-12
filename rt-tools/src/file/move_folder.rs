use async_trait::async_trait;
use rt_core::{Tool, CoreError, Result, Locale};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use schemars::JsonSchema;

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

pub struct MoveFolder;

#[async_trait]
impl Tool for MoveFolder {
    fn name(&self) -> &str {
        "file.move_folder"
    }

    fn display_name(&self, locale: Locale) -> String {
        match locale {
            Locale::En => "Move Folder".to_string(),
            Locale::Zh => "移动文件夹".to_string(),
        }
    }

    fn description(&self, locale: Locale) -> String {
        match locale {
            Locale::En => "Move or rename a folder".to_string(),
            Locale::Zh => "移动或重命名文件夹".to_string(),
        }
    }

    fn input_schema(&self, locale: Locale) -> Value {
        let mut schema = serde_json::to_value(schemars::schema_for!(MoveFolderInput)).unwrap();
        
        if locale == Locale::Zh {
            if let Some(props) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
                if let Some(source) = props.get_mut("source") {
                    source["title"] = serde_json::json!("源路径");
                }
                if let Some(destination) = props.get_mut("destination") {
                    destination["title"] = serde_json::json!("目标路径");
                }
                if let Some(overwrite) = props.get_mut("overwrite") {
                    overwrite["title"] = serde_json::json!("覆盖现有");
                }
            }
        }
        
        schema
    }

    fn output_schema(&self, locale: Locale) -> Value {
        let mut schema = serde_json::to_value(schemars::schema_for!(MoveFolderOutput)).unwrap();

        if locale == Locale::Zh {
             if let Some(props) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
                if let Some(success) = props.get_mut("success") {
                    success["title"] = serde_json::json!("是否成功");
                }
                if let Some(moved_files) = props.get_mut("moved_files") {
                     moved_files["title"] = serde_json::json!("移动文件数");
                }
            }
        }
        schema
    }

    fn user_guide(&self, locale: Locale) -> String {
        match locale {
            Locale::En => r#"# Move Folder

Move or rename a folder from one location to another.

## Inputs
- **source**: Path to the folder to move.
- **destination**: Path to the new location.
- **overwrite**: If true, overwrite the destination if it exists.

## Notes
- If overwrite is true and destination exists, the destination will be deleted before moving.
- Cross-device moves may fail if simple rename is not supported (depends on OS).
"#.to_string(),
            Locale::Zh => r#"# 移动文件夹 (Move Folder)

将文件夹移动或重命名到新位置。

## 输入参数
- **source**: 源文件夹路径。
- **destination**: 目标路径。
- **overwrite**: 如果为 true，则在目标存在时覆盖。

## 注意事项
- 如果 overwrite 为 true 且目标存在，移动前将删除目标路径内容。
- 跨磁盘移动可能会因为系统不支持简单重命名而失败（取决于操作系统）。
"#.to_string(),
        }
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
        
        // Ensure parent of target exists (if target was direct destination, it might not exist)
        if let Some(parent) = target_path.parent() {
             if !parent.exists() {
                 tokio::fs::create_dir_all(parent).await
                    .map_err(|e| CoreError::ToolFailure(format!("Failed to create parent directory: {}", e)))?;
             }
        }

        match tokio::fs::rename(src, &target_path).await {
            Ok(_) => {
                let output = MoveFolderOutput {
                    success: true,
                    moved_files: 1, 
                };
                Ok(serde_json::to_value(output).map_err(|e| CoreError::ToolFailure(e.to_string()))?)
            },
            Err(e) => {
                Err(CoreError::ToolFailure(format!("Failed to move folder: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::fs;

    #[tokio::test]
    async fn test_move_folder_success() {
        let src = "./tmp/test_src_schema";
        let dst = "./tmp/test_dst_schema";

        if fs::try_exists(src).await.unwrap() { fs::remove_dir_all(src).await.unwrap(); }
        if fs::try_exists(dst).await.unwrap() { fs::remove_dir_all(dst).await.unwrap(); }
        fs::create_dir_all(src).await.unwrap();
        fs::write(format!("{}/file.txt", src), "content").await.unwrap();

        let tool = MoveFolder;
        let input = json!({
            "source": src,
            "destination": dst,
            "overwrite": false
        });

        let result = tool.run(input).await;
        assert!(result.is_ok());

        assert!(!fs::try_exists(src).await.unwrap());
        assert!(fs::try_exists(dst).await.unwrap());
        assert!(fs::try_exists(format!("{}/file.txt", dst)).await.unwrap());

        // Test Schema
        let schema = tool.input_schema(Locale::En);
        assert!(schema.get("properties").is_some());

        fs::remove_dir_all(dst).await.unwrap();
    }

    #[tokio::test]
    async fn test_move_into_existing_folder() {
        let src = "./tmp/test_src_move_into";
        let dst = "./tmp/test_dst_parent";
        let expected_path = "./tmp/test_dst_parent/test_src_move_into";

        // clean up
        if fs::try_exists(src).await.unwrap() { fs::remove_dir_all(src).await.unwrap(); }
        if fs::try_exists(dst).await.unwrap() { fs::remove_dir_all(dst).await.unwrap(); }

        // Setup: Create src and dst
        fs::create_dir_all(src).await.unwrap();
        fs::write(format!("{}/file.txt", src), "content").await.unwrap();
        fs::create_dir_all(dst).await.unwrap();

        let tool = MoveFolder;
        let input = json!({
            "source": src,
            "destination": dst,
            "overwrite": true
        });

        let result = tool.run(input).await;
        assert!(result.is_ok());

        assert!(!fs::try_exists(src).await.unwrap());
        // dst should still exist (it's the parent)
        assert!(fs::try_exists(dst).await.unwrap());
        // moved folder should be inside
        assert!(fs::try_exists(expected_path).await.unwrap());
        assert!(fs::try_exists(format!("{}/file.txt", expected_path)).await.unwrap());

        fs::remove_dir_all(dst).await.unwrap();
    }
}
