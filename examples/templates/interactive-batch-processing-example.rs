use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio;
use workflow_toolkit::core::{WorkflowDefinition, WorkflowParameters};
use workflow_toolkit::workflow::engine::WorkflowEngine;

/// Interactive Batch Processing Example
/// 
/// This example demonstrates the interactive batch processing workflow template,
/// showing how to perform generic batch file operations with human oversight
/// and decision points.
/// 
/// The example covers:
/// 1. Batch file moving with conflict resolution
/// 2. Batch file copying with user decisions
/// 3. Custom batch operations with experimental mode
/// 4. Error handling and recovery scenarios

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🔄 Interactive Batch Processing Workflow Examples");
    println!("================================================\n");
    
    // Run different batch processing scenarios
    run_batch_move_example().await?;
    run_batch_copy_with_decisions().await?;
    run_custom_batch_operations().await?;
    run_error_recovery_example().await?;
    
    println!("✅ All batch processing examples completed successfully!");
    Ok(())
}

/// Example 1: Batch File Moving with Conflict Resolution
async fn run_batch_move_example() -> Result<()> {
    println!("📁 Example 1: Batch File Moving with Conflict Resolution");
    println!("--------------------------------------------------------");
    
    let workflow_path = "examples/templates/interactive-batch-processing-workflow.yaml";
    let workflow_def = WorkflowDefinition::from_file(workflow_path).await?;
    
    // Configure parameters for batch move operation
    let mut parameters = WorkflowParameters::new();
    parameters.insert("source_directory".to_string(), json!("/tmp/source_files"));
    parameters.insert("target_directory".to_string(), json!("/tmp/organized_files"));
    parameters.insert("operation_type".to_string(), json!("move"));
    parameters.insert("operation_config".to_string(), json!({
        "preserve_structure": true,
        "create_target_dirs": true,
        "verify_moves": true
    }));
    parameters.insert("experimental_mode".to_string(), json!(true));
    parameters.insert("enable_user_interaction".to_string(), json!(true));
    parameters.insert("batch_size".to_string(), json!(25));
    parameters.insert("max_concurrent_batches".to_string(), json!(2));
    parameters.insert("conflict_resolution".to_string(), json!("UserDecision"));
    parameters.insert("progress_reporting".to_string(), json!(true));
    parameters.insert("create_backup".to_string(), json!(false));
    parameters.insert("filter_criteria".to_string(), json!({
        "include_hidden": false,
        "min_size_bytes": 1024,  // Skip files smaller than 1KB
        "file_extensions": [".txt", ".pdf", ".doc", ".jpg", ".png"],
        "exclude_patterns": ["*.tmp", "*.log", ".DS_Store"]
    }));
    
    // Create workflow engine and execute
    let engine = WorkflowEngine::new();
    
    println!("🚀 Starting batch move operation in experimental mode...");
    let result = engine.execute_workflow(workflow_def, parameters).await?;
    
    // Display results
    if let Some(final_summary) = result.get("final_status_summary") {
        println!("📊 Batch Move Results:");
        println!("  Total items processed: {}", 
                final_summary.get("total_processed").unwrap_or(&json!(0)));
        println!("  Successful operations: {}", 
                final_summary.get("successful_operations").unwrap_or(&json!(0)));
        println!("  Failed operations: {}", 
                final_summary.get("failed_operations").unwrap_or(&json!(0)));
        println!("  Processing time: {}ms", 
                final_summary.get("processing_time_ms").unwrap_or(&json!(0)));
    }
    
    println!("✅ Batch move example completed\n");
    Ok(())
}

/// Example 2: Batch File Copying with User Decisions
async fn run_batch_copy_with_decisions() -> Result<()> {
    println!("📋 Example 2: Batch File Copying with User Decisions");
    println!("----------------------------------------------------");
    
    let workflow_path = "examples/templates/interactive-batch-processing-workflow.yaml";
    let workflow_def = WorkflowDefinition::from_file(workflow_path).await?;
    
    // Configure parameters for batch copy operation with user decisions
    let mut parameters = WorkflowParameters::new();
    parameters.insert("source_directory".to_string(), json!("/tmp/documents"));
    parameters.insert("target_directory".to_string(), json!("/tmp/backup_documents"));
    parameters.insert("operation_type".to_string(), json!("copy"));
    parameters.insert("operation_config".to_string(), json!({
        "preserve_timestamps": true,
        "preserve_permissions": true,
        "verify_checksums": true,
        "copy_mode": "incremental"  // Only copy new/changed files
    }));
    parameters.insert("experimental_mode".to_string(), json!(false));  // Production mode
    parameters.insert("enable_user_interaction".to_string(), json!(true));
    parameters.insert("decision_timeout".to_string(), json!(180));  // 3 minutes per decision
    parameters.insert("batch_size".to_string(), json!(15));
    parameters.insert("max_concurrent_batches".to_string(), json!(3));
    parameters.insert("conflict_resolution".to_string(), json!("UserDecision"));
    parameters.insert("progress_reporting".to_string(), json!(true));
    parameters.insert("create_backup".to_string(), json!(true));  // Backup before operations
    parameters.insert("filter_criteria".to_string(), json!({
        "include_hidden": false,
        "min_size_bytes": 0,
        "file_extensions": [".pdf", ".docx", ".xlsx", ".pptx", ".txt"],
        "exclude_patterns": ["~$*", "*.tmp"]
    }));
    
    // Create workflow engine and execute
    let engine = WorkflowEngine::new();
    
    println!("🚀 Starting batch copy operation with user decisions...");
    let result = engine.execute_workflow(workflow_def, parameters).await?;
    
    // Display detailed results
    if let Some(batch_report) = result.get("generate_batch_report") {
        println!("📊 Batch Copy Results:");
        if let Some(statistics) = batch_report.get("statistics") {
            println!("  Files copied: {}", 
                    statistics.get("files_copied").unwrap_or(&json!(0)));
            println!("  Bytes copied: {} MB", 
                    statistics.get("bytes_copied").unwrap_or(&json!(0)).as_u64().unwrap_or(0) / 1024 / 1024);
            println!("  Conflicts resolved: {}", 
                    statistics.get("conflicts_resolved").unwrap_or(&json!(0)));
            println!("  User decisions made: {}", 
                    statistics.get("user_decisions").unwrap_or(&json!(0)));
        }
    }
    
    println!("✅ Batch copy example completed\n");
    Ok(())
}

/// Example 3: Custom Batch Operations with Experimental Mode
async fn run_custom_batch_operations() -> Result<()> {
    println!("🔧 Example 3: Custom Batch Operations with Experimental Mode");
    println!("------------------------------------------------------------");
    
    let workflow_path = "examples/templates/interactive-batch-processing-workflow.yaml";
    let workflow_def = WorkflowDefinition::from_file(workflow_path).await?;
    
    // Configure parameters for custom batch operations
    let mut parameters = WorkflowParameters::new();
    parameters.insert("source_directory".to_string(), json!("/tmp/media_files"));
    parameters.insert("target_directory".to_string(), json!("/tmp/processed_media"));
    parameters.insert("operation_type".to_string(), json!("custom"));
    parameters.insert("operation_config".to_string(), json!({
        "custom_tool": "media-processor",
        "custom_params": {
            "resize_images": true,
            "target_resolution": "1920x1080",
            "compress_videos": true,
            "output_format": "mp4",
            "quality": "high"
        },
        "parallel_processing": true,
        "gpu_acceleration": false
    }));
    parameters.insert("experimental_mode".to_string(), json!(true));  // Test mode first
    parameters.insert("enable_user_interaction".to_string(), json!(true));
    parameters.insert("decision_timeout".to_string(), json!(300));
    parameters.insert("batch_size".to_string(), json!(10));  // Smaller batches for media processing
    parameters.insert("max_concurrent_batches".to_string(), json!(2));  // CPU intensive
    parameters.insert("conflict_resolution".to_string(), json!("Rename"));
    parameters.insert("progress_reporting".to_string(), json!(true));
    parameters.insert("create_backup".to_string(), json!(true));
    parameters.insert("filter_criteria".to_string(), json!({
        "include_hidden": false,
        "min_size_bytes": 10240,  // Skip files smaller than 10KB
        "file_extensions": [".jpg", ".jpeg", ".png", ".mp4", ".avi", ".mov"],
        "exclude_patterns": ["*.thumbnail", "*_processed.*"]
    }));
    
    // Create workflow engine and execute
    let engine = WorkflowEngine::new();
    
    println!("🚀 Starting custom batch operations in experimental mode...");
    let result = engine.execute_workflow(workflow_def, parameters).await?;
    
    // Display experimental results
    if let Some(confirm_execution) = result.get("confirm_batch_execution") {
        println!("🧪 Experimental Mode Results:");
        println!("  Selected option: {}", 
                confirm_execution.get("selected_option").unwrap_or(&json!("unknown")));
        println!("  Decision time: {}ms", 
                confirm_execution.get("decision_time_ms").unwrap_or(&json!(0)));
    }
    
    if let Some(review_plan) = result.get("review_final_plan") {
        println!("📋 Processing Plan Preview:");
        if let Some(plan_details) = review_plan.get("plan_details") {
            println!("  Total batches: {}", 
                    plan_details.get("batch_count").unwrap_or(&json!(0)));
            println!("  Estimated duration: {}s", 
                    plan_details.get("estimated_duration_seconds").unwrap_or(&json!(0)));
            println!("  Space required: {} MB", 
                    plan_details.get("space_required_mb").unwrap_or(&json!(0)));
        }
    }
    
    println!("✅ Custom batch operations example completed\n");
    Ok(())
}

/// Example 4: Error Recovery and Failure Handling
async fn run_error_recovery_example() -> Result<()> {
    println!("🚨 Example 4: Error Recovery and Failure Handling");
    println!("--------------------------------------------------");
    
    let workflow_path = "examples/templates/interactive-batch-processing-workflow.yaml";
    let workflow_def = WorkflowDefinition::from_file(workflow_path).await?;
    
    // Configure parameters to simulate error scenarios
    let mut parameters = WorkflowParameters::new();
    parameters.insert("source_directory".to_string(), json!("/tmp/problematic_files"));
    parameters.insert("target_directory".to_string(), json!("/tmp/recovered_files"));
    parameters.insert("operation_type".to_string(), json!("move"));
    parameters.insert("operation_config".to_string(), json!({
        "simulate_errors": true,  // Special config to simulate errors
        "error_rate": 0.2,        // 20% of operations will fail
        "error_types": ["permission_denied", "disk_full", "file_locked"],
        "enable_recovery": true
    }));
    parameters.insert("experimental_mode".to_string(), json!(false));
    parameters.insert("enable_user_interaction".to_string(), json!(true));
    parameters.insert("decision_timeout".to_string(), json!(120));  // Shorter timeout for errors
    parameters.insert("batch_size".to_string(), json!(20));
    parameters.insert("max_concurrent_batches".to_string(), json!(1));  // Sequential for error testing
    parameters.insert("conflict_resolution".to_string(), json!("Skip"));
    parameters.insert("progress_reporting".to_string(), json!(true));
    parameters.insert("create_backup".to_string(), json!(true));
    parameters.insert("filter_criteria".to_string(), json!({
        "include_hidden": true,
        "min_size_bytes": 0,
        "file_extensions": [],  // All file types
        "exclude_patterns": []
    }));
    
    // Create workflow engine and execute
    let engine = WorkflowEngine::new();
    
    println!("🚀 Starting batch operations with error simulation...");
    let result = engine.execute_workflow(workflow_def, parameters).await?;
    
    // Display error handling results
    if let Some(failure_handling) = result.get("execute_failure_handling") {
        println!("🔧 Error Recovery Results:");
        println!("  Recovery strategy: {}", 
                failure_handling.get("strategy_used").unwrap_or(&json!("unknown")));
        println!("  Operations retried: {}", 
                failure_handling.get("operations_retried").unwrap_or(&json!(0)));
        println!("  Operations rolled back: {}", 
                failure_handling.get("operations_rolled_back").unwrap_or(&json!(0)));
        println!("  Final success rate: {}%", 
                failure_handling.get("final_success_rate").unwrap_or(&json!(0)));
    }
    
    if let Some(verification) = result.get("verify_batch_results") {
        println!("✅ Verification Results:");
        println!("  Integrity checks passed: {}", 
                verification.get("integrity_passed").unwrap_or(&json!(false)));
        println!("  Completeness verified: {}", 
                verification.get("completeness_verified").unwrap_or(&json!(false)));
        println!("  Rollback capability: {}", 
                verification.get("rollback_available").unwrap_or(&json!(false)));
    }
    
    println!("✅ Error recovery example completed\n");
    Ok(())
}

/// Helper function to create test directories and files
#[allow(dead_code)]
async fn setup_test_environment() -> Result<()> {
    use std::fs;
    use std::io::Write;
    
    // Create test directories
    let test_dirs = [
        "/tmp/source_files",
        "/tmp/organized_files", 
        "/tmp/documents",
        "/tmp/backup_documents",
        "/tmp/media_files",
        "/tmp/processed_media",
        "/tmp/problematic_files",
        "/tmp/recovered_files"
    ];
    
    for dir in &test_dirs {
        fs::create_dir_all(dir)?;
    }
    
    // Create sample files for testing
    let sample_files = [
        ("/tmp/source_files/document1.txt", "Sample document content"),
        ("/tmp/source_files/image1.jpg", "Fake JPEG data"),
        ("/tmp/source_files/presentation.pdf", "Fake PDF data"),
        ("/tmp/documents/report.docx", "Sample report content"),
        ("/tmp/documents/spreadsheet.xlsx", "Sample spreadsheet data"),
        ("/tmp/media_files/video1.mp4", "Fake video data"),
        ("/tmp/media_files/photo1.png", "Fake PNG data"),
        ("/tmp/problematic_files/locked_file.txt", "File that will be locked"),
    ];
    
    for (path, content) in &sample_files {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
    }
    
    println!("🔧 Test environment setup completed");
    Ok(())
}

/// Helper function to cleanup test environment
#[allow(dead_code)]
async fn cleanup_test_environment() -> Result<()> {
    use std::fs;
    
    let test_dirs = [
        "/tmp/source_files",
        "/tmp/organized_files", 
        "/tmp/documents",
        "/tmp/backup_documents",
        "/tmp/media_files",
        "/tmp/processed_media",
        "/tmp/problematic_files",
        "/tmp/recovered_files"
    ];
    
    for dir in &test_dirs {
        if let Err(e) = fs::remove_dir_all(dir) {
            eprintln!("Warning: Could not remove {}: {}", dir, e);
        }
    }
    
    println!("🧹 Test environment cleanup completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_batch_processing_workflow_validation() {
        let workflow_path = "examples/templates/interactive-batch-processing-workflow.yaml";
        let result = WorkflowDefinition::from_file(workflow_path).await;
        assert!(result.is_ok(), "Workflow definition should be valid");
        
        let workflow_def = result.unwrap();
        assert_eq!(workflow_def.name, "interactive-batch-processing");
        assert_eq!(workflow_def.version, "1.0.0");
        assert!(!workflow_def.nodes.is_empty(), "Workflow should have nodes");
        assert!(!workflow_def.edges.is_empty(), "Workflow should have edges");
    }
    
    #[tokio::test]
    async fn test_parameter_validation() {
        let workflow_path = "examples/templates/interactive-batch-processing-workflow.yaml";
        let workflow_def = WorkflowDefinition::from_file(workflow_path).await.unwrap();
        
        // Test required parameters
        let required_params = ["source_directory", "target_directory"];
        for param in &required_params {
            assert!(
                workflow_def.parameters.contains_key(*param),
                "Required parameter {} should be defined", param
            );
        }
        
        // Test parameter types
        let operation_type_param = workflow_def.parameters.get("operation_type").unwrap();
        assert_eq!(operation_type_param.param_type, "string");
        
        let batch_size_param = workflow_def.parameters.get("batch_size").unwrap();
        assert_eq!(batch_size_param.param_type, "integer");
        
        let experimental_mode_param = workflow_def.parameters.get("experimental_mode").unwrap();
        assert_eq!(experimental_mode_param.param_type, "boolean");
    }
    
    #[tokio::test]
    async fn test_workflow_node_connectivity() {
        let workflow_path = "examples/templates/interactive-batch-processing-workflow.yaml";
        let workflow_def = WorkflowDefinition::from_file(workflow_path).await.unwrap();
        
        // Verify all nodes are connected
        let node_ids: std::collections::HashSet<String> = workflow_def.nodes
            .iter()
            .map(|node| node.id.clone())
            .collect();
        
        for edge in &workflow_def.edges {
            assert!(
                node_ids.contains(&edge.from),
                "Edge source node {} should exist", edge.from
            );
            assert!(
                node_ids.contains(&edge.to),
                "Edge target node {} should exist", edge.to
            );
        }
    }
    
    #[tokio::test]
    async fn test_decision_points_configuration() {
        let workflow_path = "examples/templates/interactive-batch-processing-workflow.yaml";
        let workflow_def = WorkflowDefinition::from_file(workflow_path).await.unwrap();
        
        // Find human decision nodes
        let decision_nodes: Vec<_> = workflow_def.nodes
            .iter()
            .filter(|node| {
                node.tool_name.as_ref().map_or(false, |name| name == "human-decision")
            })
            .collect();
        
        assert!(!decision_nodes.is_empty(), "Workflow should have human decision points");
        
        // Verify decision nodes have proper configuration
        for node in decision_nodes {
            let params = &node.parameters;
            assert!(params.contains_key("decision_type"), "Decision node should have decision_type");
            assert!(params.contains_key("context"), "Decision node should have context");
            assert!(params.contains_key("options"), "Decision node should have options");
        }
    }
}