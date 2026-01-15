use workflow_toolkit::core::ExecutionContext;
use workflow_toolkit::plugins::file_management::ClassificationTool;
use workflow_toolkit::tools::ToolNode;
use serde_json::json;
use std::fs;
use std::path::Path;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let source_dir = Path::new("D:\\Download\\pic");
    let rules_path = "d:\\Code\\AI\\rust-tool-v2\\scripts\\classfy copy.json";
    
    info!("Starting experimental classification on {:?}", source_dir);
    info!("Using rules from: {}", rules_path);

    // Read rules content directly
    let rules_content = fs::read_to_string(rules_path)?;
    let rules_json: serde_json::Value = serde_json::from_str(&rules_content)?;

    let tool = ClassificationTool::new(true); // Enable Chinese support
    let context = ExecutionContext::new();

    // Iterate over subdirectories
    let mut entries = fs::read_dir(source_dir)?;
    
    while let Some(entry) = entries.next() {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let folder_name = path.file_name().unwrap().to_string_lossy().to_string();

            let params = json!({
                "folder_path": path.to_string_lossy(),
                "classification_rules": rules_json, // Pass raw JSON, tool handles transformation
                "enable_user_interaction": false,
                "experimental_mode": true,
                "output_format": "Detailed"
            });

            match tool.execute(params, context.clone()).await {
                Ok(result) => {
                    if let Some(status_val) = result.get("status") {
                         if let Some(status) = status_val.as_str() {
                            if status == "Classified" {
                                let category = result.get("category").and_then(|c| c.as_str()).unwrap_or("unknown");
                                info!("[Experimental] Would move '{}' to category '{}'", folder_name, category);
                            } else {
                                info!("[Experimental] Folder '{}' status: {}", folder_name, status);
                            }
                         }
                    }
                },
                Err(e) => {
                    eprintln!("Error processing {}: {}", folder_name, e);
                }
            }
        }
    }

    Ok(())
}
