//! Interactive Classification Workflow Example
//! 
//! This example demonstrates how to use the interactive folder classification
//! workflow template with human decision support and experimental mode.
//! 
//! **Validates: Requirements 8.1, 11.5, 12.5**

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use workflow_toolkit::core::WorkflowConfig;
use workflow_toolkit::workflow::{WorkflowDefinition, WorkflowEngine};
use workflow_toolkit::plugins::file_management::FileManagementPlugin;
use workflow_toolkit::plugins::manager::PluginManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 Interactive Folder Classification Workflow Example");
    println!("====================================================");
    
    // Setup plugin manager and register file management plugin
    let mut plugin_manager = PluginManager::new();
    let file_mgmt_plugin = FileManagementPlugin::new()?;
    plugin_manager.register_plugin("file-management", Box::new(file_mgmt_plugin))?;
    
    // Load the workflow template
    let template_path = "examples/templates/interactive-classification-workflow.yaml";
    let workflow_def = load_workflow_template(template_path).await?;
    
    // Example 1: Run in experimental mode with user interaction
    println!("\n📋 Example 1: Experimental Mode with Human Decisions");
    println!("---------------------------------------------------");
    
    let experimental_params = create_experimental_parameters()?;
    run_workflow_example(&workflow_def, experimental_params, "Experimental Mode").await?;
    
    // Example 2: Run in production mode (auto-execute)
    println!("\n🔧 Example 2: Production Mode (Auto-Execute)");
    println!("--------------------------------------------");
    
    let production_params = create_production_parameters()?;
    run_workflow_example(&workflow_def, production_params, "Production Mode").await?;
    
    // Example 3: Custom classification rules
    println!("\n🎯 Example 3: Custom Classification Rules");
    println!("----------------------------------------");
    
    let custom_params = create_custom_rules_parameters()?;
    run_workflow_example(&workflow_def, custom_params, "Custom Rules").await?;
    
    println!("\n✅ All examples completed successfully!");
    Ok(())
}

/// Load workflow template from YAML file
async fn load_workflow_template(template_path: &str) -> Result<WorkflowDefinition> {
    let template_content = tokio::fs::read_to_string(template_path).await?;
    let workflow_def: WorkflowDefinition = serde_yaml::from_str(&template_content)?;
    
    println!("📄 Loaded workflow template: {}", workflow_def.name);
    println!("   Version: {}", workflow_def.version);
    println!("   Description: {}", workflow_def.description.as_deref().unwrap_or("N/A"));
    println!("   Nodes: {}", workflow_def.nodes.len());
    
    Ok(workflow_def)
}

/// Create parameters for experimental mode execution
fn create_experimental_parameters() -> Result<HashMap<String, Value>> {
    let mut params = HashMap::new();
    
    // Basic directory parameters
    params.insert("source_directory".to_string(), 
                 json!("/tmp/test_folders"));
    params.insert("output_directory".to_string(), 
                 json!("/tmp/organized_folders"));
    
    // Load classification rules from example file
    params.insert("classification_rules".to_string(), 
                 json!("examples/templates/classification-rules-example.json"));
    
    // Experimental mode settings
    params.insert("experimental_mode".to_string(), json!(true));
    params.insert("enable_user_interaction".to_string(), json!(true));
    params.insert("decision_timeout".to_string(), json!(300));
    params.insert("batch_size".to_string(), json!(5));
    params.insert("confidence_threshold".to_string(), json!(0.7));
    
    Ok(params)
}

/// Create parameters for production mode execution
fn create_production_parameters() -> Result<HashMap<String, Value>> {
    let mut params = HashMap::new();
    
    // Basic directory parameters
    params.insert("source_directory".to_string(), 
                 json!("/tmp/production_folders"));
    params.insert("output_directory".to_string(), 
                 json!("/tmp/production_organized"));
    
    // Use inline classification rules for production
    let production_rules = json!({
        "categories": {
            "documents": {
                "keywords": [
                    {"pattern": "doc", "weight": 1.0},
                    {"pattern": "pdf", "weight": 1.0},
                    {"pattern": "text", "weight": 0.8}
                ],
                "target_directory": "Documents"
            },
            "media": {
                "keywords": [
                    {"pattern": "photo", "weight": 1.0},
                    {"pattern": "video", "weight": 1.0},
                    {"pattern": "image", "weight": 1.0}
                ],
                "target_directory": "Media"
            }
        },
        "settings": {
            "minimum_score_threshold": 0.8,
            "case_sensitive": false
        }
    });
    
    params.insert("classification_rules".to_string(), production_rules);
    
    // Production mode settings (no user interaction)
    params.insert("experimental_mode".to_string(), json!(false));
    params.insert("enable_user_interaction".to_string(), json!(false));
    params.insert("batch_size".to_string(), json!(20));
    params.insert("confidence_threshold".to_string(), json!(0.8));
    
    Ok(params)
}

/// Create parameters with custom classification rules
fn create_custom_rules_parameters() -> Result<HashMap<String, Value>> {
    let mut params = HashMap::new();
    
    // Basic directory parameters
    params.insert("source_directory".to_string(), 
                 json!("/tmp/custom_folders"));
    params.insert("output_directory".to_string(), 
                 json!("/tmp/custom_organized"));
    
    // Custom classification rules with Chinese support
    let custom_rules = json!({
        "categories": {
            "工作文档": {
                "description": "Work documents in Chinese",
                "keywords": [
                    {"pattern": "工作", "weight": 1.0},
                    {"pattern": "文档", "weight": 1.0},
                    {"pattern": "报告", "weight": 0.9},
                    {"pattern": "work", "weight": 0.8},
                    {"pattern": "document", "weight": 0.8}
                ],
                "target_directory": "WorkDocuments"
            },
            "个人项目": {
                "description": "Personal projects",
                "keywords": [
                    {"pattern": "项目", "weight": 1.0},
                    {"pattern": "个人", "weight": 0.9},
                    {"pattern": "project", "weight": 0.8},
                    {"pattern": "personal", "weight": 0.8}
                ],
                "target_directory": "PersonalProjects"
            },
            "学习资料": {
                "description": "Learning materials",
                "keywords": [
                    {"pattern": "学习", "weight": 1.0},
                    {"pattern": "资料", "weight": 1.0},
                    {"pattern": "教程", "weight": 0.9},
                    {"pattern": "study", "weight": 0.8},
                    {"pattern": "tutorial", "weight": 0.8}
                ],
                "target_directory": "StudyMaterials"
            }
        },
        "settings": {
            "minimum_score_threshold": 0.6,
            "enable_chinese_processing": true,
            "case_sensitive": false
        },
        "text_processing": {
            "enable_pinyin_conversion": true,
            "normalize_traditional_chinese": true
        }
    });
    
    params.insert("classification_rules".to_string(), custom_rules);
    
    // Custom settings with moderate interaction
    params.insert("experimental_mode".to_string(), json!(true));
    params.insert("enable_user_interaction".to_string(), json!(true));
    params.insert("decision_timeout".to_string(), json!(180));
    params.insert("batch_size".to_string(), json!(8));
    params.insert("confidence_threshold".to_string(), json!(0.6));
    
    Ok(params)
}

/// Run a workflow example with given parameters
async fn run_workflow_example(
    workflow_def: &WorkflowDefinition,
    parameters: HashMap<String, Value>,
    example_name: &str,
) -> Result<()> {
    println!("\n🔄 Running {}", example_name);
    
    // Create workflow engine
    let config = WorkflowConfig::default();
    let engine = WorkflowEngine::new(config)?;
    
    // Create execution context with parameters
    let mut context = engine.create_execution_context()?;
    for (key, value) in parameters {
        context.set_parameter(key, value)?;
    }
    
    // Validate workflow before execution
    println!("   ✓ Validating workflow definition...");
    workflow_def.validate()?;
    
    // Show workflow summary
    print_workflow_summary(workflow_def, &context)?;
    
    // Execute workflow (in a real scenario)
    println!("   🚀 Starting workflow execution...");
    
    // Note: In a real implementation, you would execute the workflow here
    // let execution_result = engine.execute(workflow_def.clone(), context).await?;
    
    // For this example, we'll simulate the execution
    simulate_workflow_execution(workflow_def, example_name).await?;
    
    println!("   ✅ {} completed successfully", example_name);
    Ok(())
}

/// Print a summary of the workflow configuration
fn print_workflow_summary(
    workflow_def: &WorkflowDefinition,
    context: &workflow_toolkit::workflow::ExecutionContext,
) -> Result<()> {
    println!("   📊 Workflow Summary:");
    println!("      • Source Directory: {}", 
             context.get_parameter("source_directory")
                   .unwrap_or(&json!("N/A")));
    println!("      • Output Directory: {}", 
             context.get_parameter("output_directory")
                   .unwrap_or(&json!("N/A")));
    println!("      • Experimental Mode: {}", 
             context.get_parameter("experimental_mode")
                   .unwrap_or(&json!(false)));
    println!("      • User Interaction: {}", 
             context.get_parameter("enable_user_interaction")
                   .unwrap_or(&json!(false)));
    println!("      • Batch Size: {}", 
             context.get_parameter("batch_size")
                   .unwrap_or(&json!(10)));
    println!("      • Total Nodes: {}", workflow_def.nodes.len());
    
    Ok(())
}

/// Simulate workflow execution for demonstration
async fn simulate_workflow_execution(
    workflow_def: &WorkflowDefinition,
    example_name: &str,
) -> Result<()> {
    println!("   📝 Simulating workflow steps:");
    
    for (i, node) in workflow_def.nodes.iter().enumerate() {
        println!("      {}. {} ({})", 
                i + 1, 
                node.id, 
                node.tool_name.as_deref().unwrap_or("condition"));
        
        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Simulate different behaviors based on example type
        match example_name {
            "Experimental Mode" => {
                if node.id == "confirm_execution" {
                    println!("         💭 [Simulated] User selected: Execute operations");
                }
                if node.id == "execute_moves" {
                    println!("         📁 [Simulated] Would move 15 folders");
                }
            }
            "Production Mode" => {
                if node.id == "execute_moves" {
                    println!("         📁 [Simulated] Moved 23 folders automatically");
                }
            }
            "Custom Rules" => {
                if node.id == "classify_batch" {
                    println!("         🇨🇳 [Simulated] Processing Chinese folder names");
                }
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// Create test directories for demonstration (helper function)
#[allow(dead_code)]
async fn create_test_directories() -> Result<()> {
    let test_dirs = vec![
        "/tmp/test_folders/Document Files",
        "/tmp/test_folders/Photo Collection",
        "/tmp/test_folders/Work Project Alpha",
        "/tmp/test_folders/Old Backup Data",
        "/tmp/test_folders/Software Tools",
    ];
    
    for dir in test_dirs {
        tokio::fs::create_dir_all(dir).await?;
        println!("Created test directory: {}", dir);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_template_loading() -> Result<()> {
        let template_path = "examples/templates/interactive-classification-workflow.yaml";
        let workflow_def = load_workflow_template(template_path).await?;
        
        assert_eq!(workflow_def.name, "interactive-folder-classification");
        assert_eq!(workflow_def.version, "1.0.0");
        assert!(!workflow_def.nodes.is_empty());
        assert!(!workflow_def.edges.is_empty());
        
        Ok(())
    }
    
    #[test]
    fn test_parameter_creation() -> Result<()> {
        let experimental_params = create_experimental_parameters()?;
        assert!(experimental_params.contains_key("source_directory"));
        assert!(experimental_params.contains_key("experimental_mode"));
        
        let production_params = create_production_parameters()?;
        assert_eq!(production_params.get("experimental_mode"), Some(&json!(false)));
        
        Ok(())
    }
    
    #[test]
    fn test_custom_rules_parameters() -> Result<()> {
        let custom_params = create_custom_rules_parameters()?;
        
        // Verify Chinese processing is enabled
        let rules = custom_params.get("classification_rules").unwrap();
        let settings = rules.get("settings").unwrap();
        assert_eq!(settings.get("enable_chinese_processing"), Some(&json!(true)));
        
        Ok(())
    }
}