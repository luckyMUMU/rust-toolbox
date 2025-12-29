//! Example demonstrating backup and recovery functionality

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use workflow_toolkit::{
    core::{ExecutionStatus, WorkflowId},
    storage::{
        BackupConfig, BackupManager, FileStorage, RetentionPolicy, SimpleMemoryCache, StateManager,
    },
    workflow::{ExecutionRecord, WorkflowExecution, WorkflowState},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Workflow Toolkit - Backup and Recovery Example");
    println!("================================================");

    // Setup storage and state manager with backup functionality
    let storage_path = PathBuf::from("./tmp/backup_example_storage");
    let backup_path = PathBuf::from("./tmp/backup_example_backups");
    
    // Clean up previous runs
    if storage_path.exists() {
        std::fs::remove_dir_all(&storage_path)?;
    }
    if backup_path.exists() {
        std::fs::remove_dir_all(&backup_path)?;
    }
    
    let storage = Arc::new(FileStorage::new(&storage_path)?);
    let cache = Arc::new(SimpleMemoryCache::new());
    
    let backup_config = BackupConfig {
        backup_directory: backup_path,
        compression_level: 0, // No compression for this example
        include_execution_history: true,
        max_backup_count: Some(5),
        retention_policy: RetentionPolicy {
            max_age: chrono::Duration::days(7),
            max_count: Some(10),
        },
        enable_incremental: true,
        verify_backup: true,
    };
    
    let state_manager = StateManager::with_backup(storage, cache, backup_config)?;
    
    println!("✅ State manager with backup functionality initialized");
    
    // Create some sample workflow states and execution records
    println!("\n📝 Creating sample workflow data...");
    
    let workflow_id1 = Uuid::new_v4();
    let workflow_id2 = Uuid::new_v4();
    
    // Create workflow states
    let workflow_state1 = create_sample_workflow_state(workflow_id1, "data-processing-pipeline");
    let workflow_state2 = create_sample_workflow_state(workflow_id2, "notification-workflow");
    
    // Save workflow states
    state_manager.save_workflow_state(workflow_id1, workflow_state1.clone()).await?;
    state_manager.save_workflow_state(workflow_id2, workflow_state2.clone()).await?;
    
    // Create execution records
    let execution_record1 = create_sample_execution_record(workflow_id1, "exec-001");
    let execution_record2 = create_sample_execution_record(workflow_id2, "exec-002");
    
    state_manager.save_execution_record(execution_record1).await?;
    state_manager.save_execution_record(execution_record2).await?;
    
    println!("✅ Sample data created and saved");
    
    // Create a full backup
    println!("\n💾 Creating full backup...");
    let backup_metadata = state_manager.create_backup().await?;
    println!("✅ Full backup created:");
    println!("   - Backup ID: {}", backup_metadata.backup_id);
    println!("   - Created at: {}", backup_metadata.created_at);
    println!("   - Workflow count: {}", backup_metadata.workflow_count);
    println!("   - Execution record count: {}", backup_metadata.execution_record_count);
    println!("   - Size: {} bytes", backup_metadata.size_bytes);
    
    // Verify the backup
    println!("\n🔍 Verifying backup integrity...");
    let verification = state_manager.verify_backup(&backup_metadata.backup_id).await?;
    if verification.is_valid {
        println!("✅ Backup verification passed");
        println!("   - Checksum match: {}", verification.checksum_match);
        println!("   - Data integrity: {}", verification.data_integrity);
    } else {
        println!("❌ Backup verification failed:");
        for error in &verification.errors {
            println!("   - {}", error);
        }
    }
    
    // Modify some data and create an incremental backup
    println!("\n📝 Modifying data for incremental backup...");
    let workflow_id3 = Uuid::new_v4();
    let workflow_state3 = create_sample_workflow_state(workflow_id3, "new-workflow");
    state_manager.save_workflow_state(workflow_id3, workflow_state3).await?;
    
    // Wait a moment to ensure different timestamps
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("\n💾 Creating incremental backup...");
    let incremental_backup = state_manager.create_incremental_backup(&backup_metadata.backup_id).await?;
    println!("✅ Incremental backup created:");
    println!("   - Backup ID: {}", incremental_backup.backup_id);
    println!("   - Previous backup: {:?}", incremental_backup.previous_backup_id);
    println!("   - Workflow count: {}", incremental_backup.workflow_count);
    
    // List all backups
    println!("\n📋 Listing all backups...");
    let backups = state_manager.list_backups().await?;
    for (i, backup) in backups.iter().enumerate() {
        println!("{}. Backup ID: {} ({})", 
                 i + 1, 
                 backup.backup_id, 
                 match backup.backup_type {
                     workflow_toolkit::storage::BackupType::Full => "Full",
                     workflow_toolkit::storage::BackupType::Incremental => "Incremental",
                 });
        println!("   Created: {}", backup.created_at);
        println!("   Size: {} bytes", backup.size_bytes);
    }
    
    // Get backup statistics
    println!("\n📊 Backup statistics:");
    let stats = state_manager.get_backup_statistics().await?;
    println!("   - Total backups: {}", stats.total_backups);
    println!("   - Full backups: {}", stats.full_backups);
    println!("   - Incremental backups: {}", stats.incremental_backups);
    println!("   - Total size: {} bytes", stats.total_size_bytes);
    if let Some(oldest) = stats.oldest_backup {
        println!("   - Oldest backup: {}", oldest);
    }
    if let Some(newest) = stats.newest_backup {
        println!("   - Newest backup: {}", newest);
    }
    
    // Simulate data loss by clearing storage
    println!("\n💥 Simulating data loss (clearing storage)...");
    state_manager.clear_cache().await?;
    
    // Verify data is gone
    let lost_state = state_manager.load_workflow_state(workflow_id1).await?;
    if lost_state.is_none() {
        println!("✅ Data successfully cleared (simulating loss)");
    }
    
    // Restore from backup
    println!("\n🔄 Restoring from incremental backup...");
    let restore_result = state_manager.restore_from_backup(&incremental_backup.backup_id).await?;
    println!("✅ Restore completed:");
    println!("   - Restored workflows: {}", restore_result.restored_workflows);
    println!("   - Restored execution records: {}", restore_result.restored_execution_records);
    println!("   - Restored system data keys: {}", restore_result.restored_system_data_keys);
    
    if !restore_result.errors.is_empty() {
        println!("   - Errors during restore:");
        for error in &restore_result.errors {
            println!("     * {}", error);
        }
    }
    
    // Verify restored data
    println!("\n🔍 Verifying restored data...");
    let restored_state1 = state_manager.load_workflow_state(workflow_id1).await?;
    let restored_state2 = state_manager.load_workflow_state(workflow_id2).await?;
    let restored_state3 = state_manager.load_workflow_state(workflow_id3).await?;
    
    if restored_state1.is_some() && restored_state2.is_some() && restored_state3.is_some() {
        println!("✅ All workflow states successfully restored");
    } else {
        println!("❌ Some workflow states were not restored properly");
    }
    
    // Verify execution history
    let history1 = state_manager.get_execution_history(workflow_id1).await?;
    let history2 = state_manager.get_execution_history(workflow_id2).await?;
    
    println!("✅ Execution history restored:");
    println!("   - Workflow 1 history entries: {}", history1.len());
    println!("   - Workflow 2 history entries: {}", history2.len());
    
    // Clean up example backups
    println!("\n🧹 Cleaning up example backups...");
    for backup in &backups {
        state_manager.delete_backup(&backup.backup_id).await?;
    }
    println!("✅ Cleanup completed");
    
    println!("\n🎉 Backup and recovery example completed successfully!");
    
    Ok(())
}

fn create_sample_workflow_state(workflow_id: WorkflowId, workflow_name: &str) -> WorkflowState {
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;
    use workflow_toolkit::workflow::{Checkpoint, NodeExecutionState, WorkflowExecution};
    
    let mut node_states = HashMap::new();
    node_states.insert(
        "node1".to_string(),
        NodeExecutionState {
            status: ExecutionStatus::Completed,
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
            result: Some(json!({"output": "success"})),
            error: None,
            retry_count: 0,
        },
    );
    
    let execution = WorkflowExecution {
        id: workflow_id,
        workflow_name: workflow_name.to_string(),
        status: ExecutionStatus::Completed,
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
        current_node: None,
        node_states,
        global_context: json!({"environment": "test"}),
    };
    
    let checkpoint = Checkpoint {
        id: "checkpoint1".to_string(),
        timestamp: Utc::now(),
        node_id: "node1".to_string(),
        state_snapshot: json!({"checkpoint_data": "test"}),
    };
    
    let mut metadata = HashMap::new();
    metadata.insert("created_by".to_string(), json!("backup_example"));
    metadata.insert("version".to_string(), json!("1.0"));
    
    WorkflowState {
        execution,
        checkpoints: vec![checkpoint],
        metadata,
    }
}

fn create_sample_execution_record(workflow_id: WorkflowId, execution_id: &str) -> ExecutionRecord {
    use chrono::Utc;
    use serde_json::json;
    
    ExecutionRecord {
        workflow_id,
        execution_id: execution_id.to_string(),
        timestamp: Utc::now(),
        status: ExecutionStatus::Completed,
        result: Some(json!({"status": "success", "message": "Workflow completed successfully"})),
        error: None,
        duration: Some(chrono::Duration::seconds(30)),
    }
}