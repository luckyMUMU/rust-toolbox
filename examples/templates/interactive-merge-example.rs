use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use workflow_toolkit::config::Config;
use workflow_toolkit::core::{ExecutionContext, WorkflowDefinition};
use workflow_toolkit::plugins::file_management::FileManagementPlugin;
use workflow_toolkit::plugins::manager::PluginManager;
use workflow_toolkit::tools::registry::ToolRegistry;
use workflow_toolkit::workflow::engine::WorkflowEngine;

/// Interactive Merge Workflow Example
/// 
/// This example demonstrates how to use the interactive merge workflow template
/// to intelligently merge folders with the same name across multiple directories.
/// 
/// Features demonstrated:
/// - Multiple merge strategies (SmallerToLarger, LargerToSmaller, UserDecision)
/// - Human decision-making for ambiguous scenarios
/// - Experimental mode with confirmation steps
/// - Duplicate file conflict resolution
/// - Comprehensive reporting and verification
/// 
/// Requirements validated:
/// - 8.2: Workflow templates for folder merging
/// - 11.5: Human decision-making modes
/// - 12.5: Experimental mode with confirmation steps

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🔄 Interactive Folder Merge Workflow Example");
    println!("============================================\n");
    
    // Setup configuration
    let config = Config::default();
    
    // Initialize plugin manager and register file management plugin
    let mut plugin_manager = PluginManager::new(config.clone());
    let file_management_plugin = FileManagementPlugin::new()?;
    plugin_manager.register_plugin("file-management", Box::new(file_management_plugin))?;
    
    // Initialize tool registry
    let mut tool_registry = ToolRegistry::new();
    plugin_manager.register_tools(&mut tool_registry)?;
    
    // Initialize workflow engine
    let workflow_engine = WorkflowEngine::new(config.clone(), tool_registry);
    
    // Run different merge scenarios
    run_experimental_merge_scenario(&workflow_engine).await?;
    run_production_merge_scenario(&workflow_engine).await?;
    run_custom_strategy_scenario(&workflow_engine).await?;
    
    println!("\n✅ All merge workflow scenarios completed successfully!");
    
    Ok(())
}

/// Scenario 1: Experimental Mode with User Decisions
/// 
/// This scenario demonstrates:
/// - Running in experimental mode to preview operations
/// - User decision-making for merge strategies
/// - Conflict resolution for duplicate files
/// - Comprehensive review before execution
async fn run_experimental_merge_scenario(engine: &WorkflowEngine) -> Result<()> {
    println!("📋 Scenario 1: Experimental Mode with User Decisions");
    println!("---------------------------------------------------");
    
    // Load the interactive merge workflow template
    let workflow_def = WorkflowDefinition::from_file("examples/templates/interactive-merge-workflow.yaml")?;
    
    // Setup parameters for experimental mode
    let parameters = json!({
        "source_directories": [
            "/tmp/test_merge/source1",
            "/tmp/test_merge/source2", 
            "/tmp/test_merge/source3"
        ],
        "target_directory": "/tmp/test_merge/merged_output",
        "merge_strategy": "UserDecision",
        "experimental_mode": true,
        "enable_user_interaction": true,
        "decision_timeout": 300,
        "duplicate_handling": "UserDecision",
        "enable_size_analysis": true,
        "backup_before_merge": true
    });
    
    // Create execution context
    let context = ExecutionContext::new()
        .with_parameters(parameters)
        .with_metadata("scenario", json!("experimental_with_decisions"));
    
    println!("🔍 Setting up test directories with mergeable folders...");
    setup_test_merge_directories().await?;
    
    println!("🚀 Starting experimental merge workflow...");
    println!("   - Multiple source directories will be scanned");
    println!("   - Common folder names will be identified");
    println!("   - You'll be asked to choose merge strategies");
    println!("   - Duplicate file conflicts will be presented for resolution");
    println!("   - Final confirmation will be requested before execution");
    
    // Execute the workflow
    let result = engine.execute_workflow(&workflow_def, context).await?;
    
    println!("📊 Experimental merge workflow results:");
    println!("   Status: {}", result.status);
    println!("   Duration: {:?}", result.execution_time);
    println!("   Nodes executed: {}", result.node_results.len());
    
    if let Some(merge_report) = result.outputs.get("generate_merge_report") {
        println!("   📋 Merge report generated: {}", merge_report);
    }
    
    println!("✅ Experimental scenario completed\n");
    
    Ok(())
}

/// Scenario 2: Production Mode with Automatic Strategy
/// 
/// This scenario demonstrates:
/// - Running in production mode (no experimental confirmation)
/// - Automatic merge strategy application
/// - Streamlined execution for batch processing
async fn run_production_merge_scenario(engine: &WorkflowEngine) -> Result<()> {
    println!("⚡ Scenario 2: Production Mode with Automatic Strategy");
    println!("----------------------------------------------------");
    
    let workflow_def = WorkflowDefinition::from_file("examples/templates/interactive-merge-workflow.yaml")?;
    
    // Setup parameters for production mode
    let parameters = json!({
        "source_directories": [
            "/tmp/test_merge_prod/downloads",
            "/tmp/test_merge_prod/documents",
            "/tmp/test_merge_prod/backup"
        ],
        "target_directory": "/tmp/test_merge_prod/organized",
        "merge_strategy": "SmallerToLarger",
        "experimental_mode": false,
        "enable_user_interaction": false,
        "duplicate_handling": "Rename",
        "minimum_folder_size": 1024,  // 1KB minimum
        "enable_size_analysis": true,
        "backup_before_merge": false
    });
    
    let context = ExecutionContext::new()
        .with_parameters(parameters)
        .with_metadata("scenario", json!("production_automatic"));
    
    println!("🔧 Setting up production test directories...");
    setup_production_test_directories().await?;
    
    println!("🚀 Starting production merge workflow...");
    println!("   - Automatic 'SmallerToLarger' merge strategy");
    println!("   - Duplicate files will be renamed automatically");
    println!("   - No user interaction required");
    println!("   - Direct execution without experimental mode");
    
    let result = engine.execute_workflow(&workflow_def, context).await?;
    
    println!("📊 Production merge workflow results:");
    println!("   Status: {}", result.status);
    println!("   Duration: {:?}", result.execution_time);
    
    if let Some(cleanup_result) = result.outputs.get("cleanup_empty_directories") {
        println!("   🧹 Cleanup completed: {}", cleanup_result);
    }
    
    println!("✅ Production scenario completed\n");
    
    Ok(())
}

/// Scenario 3: Custom Strategy with Target Directory
/// 
/// This scenario demonstrates:
/// - Custom merge strategy targeting a specific directory
/// - Advanced conflict resolution options
/// - Performance optimization settings
async fn run_custom_strategy_scenario(engine: &WorkflowEngine) -> Result<()> {
    println!("🎯 Scenario 3: Custom Strategy with Target Directory");
    println!("---------------------------------------------------");
    
    let workflow_def = WorkflowDefinition::from_file("examples/templates/interactive-merge-workflow.yaml")?;
    
    // Setup parameters for custom strategy
    let parameters = json!({
        "source_directories": [
            "/tmp/test_merge_custom/photos_2023",
            "/tmp/test_merge_custom/photos_2024",
            "/tmp/test_merge_custom/camera_backup"
        ],
        "target_directory": "/tmp/test_merge_custom/consolidated_photos",
        "merge_strategy": "TargetDirectory",
        "experimental_mode": true,
        "enable_user_interaction": true,
        "duplicate_handling": "UserDecision",
        "max_merge_depth": 2,
        "enable_size_analysis": true,
        "backup_before_merge": true
    });
    
    let context = ExecutionContext::new()
        .with_parameters(parameters)
        .with_metadata("scenario", json!("custom_target_directory"));
    
    println!("📁 Setting up custom test directories with photo folders...");
    setup_custom_test_directories().await?;
    
    println!("🚀 Starting custom merge workflow...");
    println!("   - All folders will be merged to target directory");
    println!("   - Limited merge depth for performance");
    println!("   - User decisions for duplicate photo handling");
    println!("   - Backup will be created before merge");
    
    let result = engine.execute_workflow(&workflow_def, context).await?;
    
    println!("📊 Custom merge workflow results:");
    println!("   Status: {}", result.status);
    println!("   Duration: {:?}", result.execution_time);
    
    if let Some(backup_result) = result.outputs.get("execute_backup") {
        println!("   💾 Backup created: {}", backup_result);
    }
    
    println!("✅ Custom scenario completed\n");
    
    Ok(())
}

/// Setup test directories for experimental merge scenario
async fn setup_test_merge_directories() -> Result<()> {
    use std::fs;
    use std::io::Write;
    
    // Create source directories with common folder names
    let base_path = PathBuf::from("/tmp/test_merge");
    fs::create_dir_all(&base_path)?;
    
    // Source 1: Documents folder structure
    let source1 = base_path.join("source1");
    fs::create_dir_all(source1.join("Documents/Reports"))?;
    fs::create_dir_all(source1.join("Photos/Vacation"))?;
    fs::create_dir_all(source1.join("Music/Classical"))?;
    
    // Create some test files
    let mut file1 = fs::File::create(source1.join("Documents/report1.txt"))?;
    file1.write_all(b"Test report from source 1")?;
    
    let mut file2 = fs::File::create(source1.join("Photos/photo1.jpg"))?;
    file2.write_all(b"Fake photo data from source 1")?;
    
    // Source 2: Overlapping folder structure
    let source2 = base_path.join("source2");
    fs::create_dir_all(source2.join("Documents/Presentations"))?;
    fs::create_dir_all(source2.join("Photos/Family"))?;
    fs::create_dir_all(source2.join("Videos/Movies"))?;
    
    let mut file3 = fs::File::create(source2.join("Documents/report2.txt"))?;
    file3.write_all(b"Test report from source 2")?;
    
    let mut file4 = fs::File::create(source2.join("Photos/photo2.jpg"))?;
    file4.write_all(b"Fake photo data from source 2")?;
    
    // Source 3: More overlapping folders
    let source3 = base_path.join("source3");
    fs::create_dir_all(source3.join("Documents/Archive"))?;
    fs::create_dir_all(source3.join("Music/Rock"))?;
    fs::create_dir_all(source3.join("Videos/Tutorials"))?;
    
    let mut file5 = fs::File::create(source3.join("Documents/report1.txt"))?; // Duplicate name
    file5.write_all(b"Different report with same name from source 3")?;
    
    // Create target directory
    fs::create_dir_all(base_path.join("merged_output"))?;
    
    println!("   ✅ Test directories created with mergeable folders:");
    println!("      - Documents (in all 3 sources)");
    println!("      - Photos (in sources 1 & 2)");
    println!("      - Music (in sources 1 & 3)");
    println!("      - Videos (in sources 2 & 3)");
    println!("      - Duplicate file: report1.txt (in sources 1 & 3)");
    
    Ok(())
}

/// Setup test directories for production merge scenario
async fn setup_production_test_directories() -> Result<()> {
    use std::fs;
    use std::io::Write;
    
    let base_path = PathBuf::from("/tmp/test_merge_prod");
    fs::create_dir_all(&base_path)?;
    
    // Downloads directory
    let downloads = base_path.join("downloads");
    fs::create_dir_all(downloads.join("Software/Tools"))?;
    fs::create_dir_all(downloads.join("Images/Screenshots"))?;
    
    let mut file1 = fs::File::create(downloads.join("Software/app1.zip"))?;
    file1.write_all(b"Fake software package")?;
    
    // Documents directory
    let documents = base_path.join("documents");
    fs::create_dir_all(documents.join("Software/Manuals"))?;
    fs::create_dir_all(documents.join("Projects/Code"))?;
    
    let mut file2 = fs::File::create(documents.join("Software/manual.pdf"))?;
    file2.write_all(b"Software manual content")?;
    
    // Backup directory
    let backup = base_path.join("backup");
    fs::create_dir_all(backup.join("Images/Backup"))?;
    fs::create_dir_all(backup.join("Projects/Archive"))?;
    
    let mut file3 = fs::File::create(backup.join("Images/backup_image.png"))?;
    file3.write_all(b"Backup image data")?;
    
    // Create organized target
    fs::create_dir_all(base_path.join("organized"))?;
    
    println!("   ✅ Production directories created:");
    println!("      - Software folders (downloads & documents)");
    println!("      - Images folders (downloads & backup)");
    println!("      - Projects folders (documents & backup)");
    
    Ok(())
}

/// Setup test directories for custom merge scenario
async fn setup_custom_test_directories() -> Result<()> {
    use std::fs;
    use std::io::Write;
    
    let base_path = PathBuf::from("/tmp/test_merge_custom");
    fs::create_dir_all(&base_path)?;
    
    // Photos 2023
    let photos_2023 = base_path.join("photos_2023");
    fs::create_dir_all(photos_2023.join("Vacation/Summer"))?;
    fs::create_dir_all(photos_2023.join("Family/Birthdays"))?;
    
    let mut file1 = fs::File::create(photos_2023.join("Vacation/beach.jpg"))?;
    file1.write_all(b"Beach photo from 2023")?;
    
    // Photos 2024
    let photos_2024 = base_path.join("photos_2024");
    fs::create_dir_all(photos_2024.join("Vacation/Winter"))?;
    fs::create_dir_all(photos_2024.join("Family/Holidays"))?;
    
    let mut file2 = fs::File::create(photos_2024.join("Vacation/ski.jpg"))?;
    file2.write_all(b"Ski photo from 2024")?;
    
    // Camera backup
    let camera_backup = base_path.join("camera_backup");
    fs::create_dir_all(camera_backup.join("Family/Events"))?;
    fs::create_dir_all(camera_backup.join("Nature/Wildlife"))?;
    
    let mut file3 = fs::File::create(camera_backup.join("Family/wedding.jpg"))?;
    file3.write_all(b"Wedding photo from camera")?;
    
    // Create consolidated target
    fs::create_dir_all(base_path.join("consolidated_photos"))?;
    
    println!("   ✅ Custom directories created:");
    println!("      - Vacation folders (2023 & 2024)");
    println!("      - Family folders (all sources)");
    println!("      - Nature folder (camera backup)");
    
    Ok(())
}