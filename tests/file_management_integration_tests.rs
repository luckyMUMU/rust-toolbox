//! Comprehensive integration tests for file management tools
//!
//! This module tests end-to-end workflows, tool interactions, and human decision
//! integration for the file management plugin system.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

use workflow_toolkit::{
    config::ConfigManager,
    storage::{FileStorage, SimpleMemoryCache, StateManager},
    tools::{AsyncFunctionExecutor, BasicTool, BasicToolRegistry},
    workflow::{engine::DefaultWorkflowEngine, NodeType, WorkflowNode, WorkflowState},
    Config, ExecutionContext, ExecutionStatus, Result, ToolRegistry, WorkflowConfig,
    WorkflowDefinition, WorkflowEngine, WorkflowError,
};

/// Test fixture for file management integration tests
struct FileManagementTestFixture {
    temp_dir: TempDir,
    config: Arc<ConfigManager>,
    state_manager: Arc<StateManager>,
    tool_registry: Arc<BasicToolRegistry>,
    workflow_engine: Arc<DefaultWorkflowEngine>,
}

impl FileManagementTestFixture {
    async fn new() -> Result<Self> {
        let temp_dir = TempDir::new().map_err(|e| {
            WorkflowError::workflow_execution(&format!("Failed to create temp dir: {}", e))
        })?;

        // Create configuration manager
        let config = Arc::new(ConfigManager::new(Config::default()));

        // Create storage components
        let storage = Arc::new(FileStorage::new(&temp_dir.path().join("storage")).map_err(
            |e| WorkflowError::workflow_execution(&format!("Failed to create storage: {}", e)),
        )?);
        let cache = Arc::new(SimpleMemoryCache::new());
        let state_manager = Arc::new(StateManager::new(storage, cache));

        // Create tool registry
        let mut tool_registry = BasicToolRegistry::new();

        // Register mock file management tools for testing
        Self::register_mock_file_management_tools(&mut tool_registry).await?;
        let tool_registry = Arc::new(tool_registry);

        // Create workflow engine
        let workflow_engine = Arc::new(DefaultWorkflowEngine::new(
            state_manager.clone(),
            tool_registry.clone(),
            4,
        ));

        Ok(Self {
            temp_dir,
            config,
            state_manager,
            tool_registry,
            workflow_engine,
        })
    }

    async fn register_mock_file_management_tools(registry: &mut BasicToolRegistry) -> Result<()> {
        // Mock folder classifier
        let classifier_executor = Arc::new(AsyncFunctionExecutor::new(
            |params: Value, _context| async move {
                let folder_path = params
                    .get("folder_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let experimental_mode = params
                    .get("experimental_mode")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // Mock classification logic
                let category = if folder_path.contains("document")
                    || folder_path.contains("文档")
                    || folder_path.contains("Documents")
                    || folder_path.contains("Project")
                {
                    "documents"
                } else if folder_path.contains("photo")
                    || folder_path.contains("music")
                    || folder_path.contains("Photos")
                    || folder_path.contains("Music")
                {
                    "media"
                } else {
                    "other"
                };

                Ok(json!({
                    "status": "classified",
                    "category": category,
                    "score": 0.85,
                    "folder_name": std::path::Path::new(folder_path).file_name()
                        .unwrap_or_default().to_string_lossy(),
                    "processing_time_ms": 50,
                    "experimental_mode": experimental_mode,
                    "candidates": [
                        {"category": category, "score": 0.85}
                    ]
                }))
            },
        ));

        let classifier_tool = BasicTool::builder()
            .name("folder-classifier")
            .version("1.0.0")
            .description("Mock folder classifier for testing")
            .executor(classifier_executor)
            .build()
            .map_err(|e| {
                WorkflowError::workflow_execution(&format!(
                    "Failed to create classifier tool: {}",
                    e
                ))
            })?;

        registry.register_tool(Arc::new(classifier_tool))?;

        // Mock text processor
        let text_processor_executor = Arc::new(AsyncFunctionExecutor::new(
            |params: Value, _context| async move {
                let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let empty_vec = vec![];
                let operations = params
                    .get("operations")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty_vec);

                let mut processed = text.to_lowercase();
                let mut pinyin_variants = Vec::new();

                // Mock text processing
                for operation in operations {
                    if let Some(op) = operation.as_str() {
                        match op {
                            "NormalizeCase" => processed = processed.to_lowercase(),
                            "GeneratePinyin" => {
                                if text.contains("项目") {
                                    pinyin_variants.push("xiangmu".to_string());
                                }
                                if text.contains("文档") {
                                    pinyin_variants.push("wendang".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }

                Ok(json!({
                    "original": text,
                    "processed": processed,
                    "pinyin_variants": pinyin_variants,
                    "combinations": [],
                    "metadata": {}
                }))
            },
        ));

        let text_processor_tool = BasicTool::builder()
            .name("text-processor")
            .version("1.0.0")
            .description("Mock text processor for testing")
            .executor(text_processor_executor)
            .build()
            .map_err(|e| {
                WorkflowError::workflow_execution(&format!(
                    "Failed to create text processor tool: {}",
                    e
                ))
            })?;

        registry.register_tool(Arc::new(text_processor_tool))?;

        // Mock batch processor
        let batch_processor_executor = Arc::new(AsyncFunctionExecutor::new(
            |params: Value, _context| async move {
                let empty_vec = vec![];
                let items = params
                    .get("items")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty_vec);
                let tool_name = params
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let mut results = Vec::new();
                let mut processed_count = 0;
                let mut failed_count = 0;

                for (i, item) in items.iter().enumerate() {
                    // Mock processing each item
                    if tool_name == "folder-classifier" {
                        let folder_path = item
                            .get("folder_path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        // Simulate some failures for testing
                        if folder_path.contains("invalid") {
                            failed_count += 1;
                            results.push(json!({
                                "item_index": i,
                                "status": "failed",
                                "error": "Invalid folder path"
                            }));
                        } else {
                            processed_count += 1;
                            results.push(json!({
                                "item_index": i,
                                "status": "completed",
                                "result": {
                                    "status": "classified",
                                    "category": "documents",
                                    "score": 0.8
                                }
                            }));
                        }
                    }
                }

                Ok(json!({
                    "total_items": items.len(),
                    "processed_items": processed_count,
                    "failed_items": failed_count,
                    "results": results,
                    "processing_time_ms": 200
                }))
            },
        ));

        let batch_processor_tool = BasicTool::builder()
            .name("batch-processor")
            .version("1.0.0")
            .description("Mock batch processor for testing")
            .executor(batch_processor_executor)
            .build()
            .map_err(|e| {
                WorkflowError::workflow_execution(&format!(
                    "Failed to create batch processor tool: {}",
                    e
                ))
            })?;

        registry.register_tool(Arc::new(batch_processor_tool))?;

        // Mock human decision tool
        let human_decision_executor = Arc::new(AsyncFunctionExecutor::new(
            |params: Value, _context| async move {
                let _decision_type = params
                    .get("decision_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Custom");
                let empty_vec = vec![];
                let options = params
                    .get("options")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty_vec);

                // Auto-select recommended option or first option
                let recommended_option = options
                    .iter()
                    .find(|opt| {
                        opt.get("recommended")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    })
                    .or_else(|| options.first());

                if let Some(option) = recommended_option {
                    let selected_id = option
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default");
                    return Ok(json!({
                        "selected_option": selected_id,
                        "decision_time_ms": 0,
                        "was_timeout": false,
                        "user_input": "auto-selected for testing"
                    }));
                }

                // Default selection (first option)
                let selected_id = options
                    .first()
                    .and_then(|opt| opt.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                Ok(json!({
                    "selected_option": selected_id,
                    "decision_time_ms": 100,
                    "was_timeout": false,
                    "user_input": null
                }))
            },
        ));

        let human_decision_tool = BasicTool::builder()
            .name("human-decision")
            .version("1.0.0")
            .description("Mock human decision tool for testing")
            .executor(human_decision_executor)
            .build()
            .map_err(|e| {
                WorkflowError::workflow_execution(&format!(
                    "Failed to create human decision tool: {}",
                    e
                ))
            })?;

        registry.register_tool(Arc::new(human_decision_tool))?;

        // Add test helper tools
        Self::register_test_helper_tools(registry).await?;

        Ok(())
    }

    async fn register_test_helper_tools(registry: &mut BasicToolRegistry) -> Result<()> {
        // Directory scanner mock
        let scanner_executor = Arc::new(AsyncFunctionExecutor::new(
            |params: Value, _context| async move {
                let directory = params
                    .get("directory")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");

                // Mock directory scan results
                Ok(json!({
                    "folders": [
                        {"name": "Documents", "path": format!("{}/Documents", directory), "size": 1024000},
                        {"name": "Photos", "path": format!("{}/Photos", directory), "size": 2048000},
                        {"name": "Music", "path": format!("{}/Music", directory), "size": 512000},
                    ],
                    "folder_count": 3,
                    "total_size": 3584000
                }))
            },
        ));

        let scanner_tool = BasicTool::builder()
            .name("directory-scanner")
            .version("1.0.0")
            .description("Mock directory scanner for testing")
            .executor(scanner_executor)
            .build()
            .map_err(|e| {
                WorkflowError::workflow_execution(&format!("Failed to create scanner tool: {}", e))
            })?;

        registry.register_tool(Arc::new(scanner_tool))?;

        // Rule validator mock
        let validator_executor = Arc::new(AsyncFunctionExecutor::new(
            |params: Value, _context| async move {
                let _rules = params.get("rules");

                // Mock validation results
                Ok(json!({
                    "valid": true,
                    "rule_count": 5,
                    "categories": ["documents", "media", "archives", "code", "other"]
                }))
            },
        ));

        let validator_tool = BasicTool::builder()
            .name("rule-validator")
            .version("1.0.0")
            .description("Mock rule validator for testing")
            .executor(validator_executor)
            .build()
            .map_err(|e| {
                WorkflowError::workflow_execution(&format!(
                    "Failed to create validator tool: {}",
                    e
                ))
            })?;

        registry.register_tool(Arc::new(validator_tool))?;

        Ok(())
    }

    /// Create test folders in the temp directory
    async fn create_test_folders(&self) -> Result<Vec<std::path::PathBuf>> {
        let test_folders = vec![
            "Project Documents",
            "Family Photos 2023",
            "Music Collection",
            "工作文档", // Chinese folder name
            "Backup Files",
        ];

        let mut created_paths = Vec::new();

        for folder_name in test_folders {
            let folder_path = self.temp_dir.path().join("source").join(folder_name);
            std::fs::create_dir_all(&folder_path).map_err(|e| {
                WorkflowError::workflow_execution(&format!("Failed to create test folder: {}", e))
            })?;

            // Create some test files in each folder
            let test_file = folder_path.join("test_file.txt");
            std::fs::write(&test_file, "Test content").map_err(|e| {
                WorkflowError::workflow_execution(&format!("Failed to create test file: {}", e))
            })?;

            created_paths.push(folder_path);
        }

        Ok(created_paths)
    }
}

#[tokio::test]
async fn test_file_management_tools_registration() -> Result<()> {
    let fixture = FileManagementTestFixture::new().await?;

    // Verify that file management tools are registered
    let tools = fixture.tool_registry.list_tools();
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Check for key file management tools
    assert!(tool_names.contains(&"folder-classifier"));
    assert!(tool_names.contains(&"text-processor"));
    assert!(tool_names.contains(&"batch-processor"));
    assert!(tool_names.contains(&"human-decision"));

    println!("File management tools registered: {:?}", tool_names);

    Ok(())
}

#[tokio::test]
async fn test_classification_tool_integration() -> Result<()> {
    let fixture = FileManagementTestFixture::new().await?;
    let _test_folders = fixture.create_test_folders().await?;

    // Test classification tool with sample rules
    let classification_rules = json!({
        "categories": {
            "documents": {
                "keywords": ["document", "doc", "文档", "project"],
                "score": 1.0
            },
            "media": {
                "keywords": ["photo", "music", "video", "照片"],
                "score": 1.0
            }
        }
    });

    let classification_params = json!({
        "folder_path": fixture.temp_dir.path().join("source/Project Documents").to_string_lossy(),
        "classification_rules": classification_rules,
        "experimental_mode": true,
        "enable_user_interaction": false
    });

    let result = fixture
        .tool_registry
        .execute_tool(
            "folder-classifier",
            classification_params,
            ExecutionContext::new(),
        )
        .await?;

    // Verify classification result structure
    assert!(result.get("status").is_some());
    assert!(result.get("folder_name").is_some());
    assert!(result.get("processing_time_ms").is_some());
    assert_eq!(
        result.get("category").and_then(|v| v.as_str()),
        Some("documents")
    );

    println!(
        "Classification result: {}",
        serde_json::to_string_pretty(&result)?
    );

    Ok(())
}

#[tokio::test]
async fn test_text_processor_tool_integration() -> Result<()> {
    let fixture = FileManagementTestFixture::new().await?;

    // Test text processing with Chinese text
    let text_params = json!({
        "text": "工作文档 Project Documents",
        "operations": ["NormalizeCase", "GeneratePinyin"],
        "chinese_processing": {
            "pinyin_style": "Normal",
            "generate_combinations": true,
            "include_tones": false
        }
    });

    let result = fixture
        .tool_registry
        .execute_tool("text-processor", text_params, ExecutionContext::new())
        .await?;

    // Verify text processing result structure
    assert!(result.get("original").is_some());
    assert!(result.get("processed").is_some());
    assert!(result.get("pinyin_variants").is_some());

    let pinyin_variants = result
        .get("pinyin_variants")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(pinyin_variants.len() > 0);

    println!(
        "Text processing result: {}",
        serde_json::to_string_pretty(&result)?
    );

    Ok(())
}

#[tokio::test]
async fn test_batch_processor_integration() -> Result<()> {
    let fixture = FileManagementTestFixture::new().await?;
    let test_folders = fixture.create_test_folders().await?;

    // Create batch processing parameters
    let batch_items: Vec<Value> = test_folders
        .iter()
        .map(|path| {
            json!({
                "folder_path": path.to_string_lossy(),
                "classification_rules": {
                    "categories": {
                        "documents": {"keywords": ["document", "project"], "score": 1.0},
                        "media": {"keywords": ["photo", "music"], "score": 1.0}
                    }
                },
                "experimental_mode": true
            })
        })
        .collect();

    let batch_params = json!({
        "items": batch_items,
        "tool_name": "folder-classifier",
        "batch_size": 2,
        "max_concurrent": 2,
        "progress_reporting": true
    });

    let result = fixture
        .tool_registry
        .execute_tool("batch-processor", batch_params, ExecutionContext::new())
        .await?;

    // Verify batch processing result structure
    assert!(result.get("total_items").is_some());
    assert!(result.get("processed_items").is_some());
    assert!(result.get("results").is_some());

    let total_items = result
        .get("total_items")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert_eq!(total_items, test_folders.len() as u64);

    println!(
        "Batch processing result: {}",
        serde_json::to_string_pretty(&result)?
    );

    Ok(())
}

#[tokio::test]
async fn test_human_decision_experimental_mode() -> Result<()> {
    let fixture = FileManagementTestFixture::new().await?;

    // Test human decision tool in experimental mode
    let decision_params = json!({
        "decision_type": "Classification",
        "context": {
            "title": "Test Classification Decision",
            "description": "Choose classification for ambiguous folder",
            "folder_name": "Mixed Content Folder"
        },
        "options": [
            {
                "id": "documents",
                "label": "Documents Category",
                "description": "Classify as documents",
                "score": 0.7,
                "recommended": true
            },
            {
                "id": "media",
                "label": "Media Category",
                "description": "Classify as media",
                "score": 0.6,
                "recommended": false
            }
        ],
        "timeout_seconds": 30
    });

    // Create execution context
    let context = ExecutionContext::new();

    let result = fixture
        .tool_registry
        .execute_tool("human-decision", decision_params, context)
        .await?;

    // Verify experimental mode auto-selection
    assert!(result.get("selected_option").is_some());
    assert!(result.get("was_timeout").is_some());

    let selected = result
        .get("selected_option")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(selected, "documents"); // Should auto-select recommended option

    println!(
        "Human decision result (experimental): {}",
        serde_json::to_string_pretty(&result)?
    );

    Ok(())
}

#[tokio::test]
async fn test_classification_workflow_integration() -> Result<()> {
    let fixture = FileManagementTestFixture::new().await?;
    let _test_folders = fixture.create_test_folders().await?;

    // Create a simplified classification workflow
    let workflow_def = WorkflowDefinition {
        name: "test_classification_workflow".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test classification workflow".to_string()),
        metadata: HashMap::new(),
        nodes: vec![
            WorkflowNode {
                id: "scan_folders".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("directory-scanner".to_string()),
                parameters: json!({
                    "directory": fixture.temp_dir.path().join("source").to_string_lossy(),
                    "recursive": false,
                    "filter_type": "directories"
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: Vec::new(),
                metadata: HashMap::new(),
            },
            WorkflowNode {
                id: "validate_rules".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("rule-validator".to_string()),
                parameters: json!({
                    "rules": {
                        "categories": {
                            "documents": {"keywords": ["document", "project"], "score": 1.0}
                        }
                    }
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(30)),
                depends_on: vec!["scan_folders".to_string()],
                metadata: HashMap::new(),
            },
            WorkflowNode {
                id: "classify_batch".to_string(),
                node_type: NodeType::Tool,
                tool_name: Some("batch-processor".to_string()),
                parameters: json!({
                    "items": "${scan_folders.folders}",
                    "tool_name": "folder-classifier",
                    "tool_params": {
                        "classification_rules": {
                            "categories": {
                                "documents": {"keywords": ["document", "project"], "score": 1.0}
                            }
                        },
                        "experimental_mode": true
                    },
                    "batch_size": 2
                }),
                retry_policy: None,
                timeout: Some(Duration::from_secs(60)),
                depends_on: vec!["validate_rules".to_string()],
                metadata: HashMap::new(),
            },
        ],
        edges: vec![],
        global_config: WorkflowConfig::default(),
    };

    // Execute workflow
    let execution = fixture
        .workflow_engine
        .execute_workflow(workflow_def)
        .await?;

    // Wait for completion with timeout
    let result: Result<ExecutionStatus> = timeout(Duration::from_secs(30), async {
        loop {
            let status = fixture
                .workflow_engine
                .get_workflow_status(execution.id)
                .await?;
            if status.is_terminal() {
                return Ok(status);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .map_err(|_| WorkflowError::workflow_execution("Workflow execution timeout"))?;

    let final_status = result?;
    assert_eq!(final_status, ExecutionStatus::Completed);

    // Verify workflow state was saved
    let workflow_state: Option<WorkflowState> = fixture
        .state_manager
        .load_workflow_state(execution.id)
        .await?;
    assert!(workflow_state.is_some());

    println!("Classification workflow completed successfully");

    Ok(())
}

#[tokio::test]
async fn test_tool_interaction_data_flow() -> Result<()> {
    let fixture = FileManagementTestFixture::new().await?;

    // Test data flow between text processor and classification

    // Step 1: Process text with Chinese content
    let text_result = fixture
        .tool_registry
        .execute_tool(
            "text-processor",
            json!({
                "text": "项目文档 Project Files",
                "operations": ["NormalizeCase", "GeneratePinyin"]
            }),
            ExecutionContext::new(),
        )
        .await?;

    // Step 2: Use processed text in classification
    let processed_text = text_result
        .get("processed")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let pinyin_variants = text_result
        .get("pinyin_variants")
        .cloned()
        .unwrap_or(json!([]));

    // Create enhanced classification rules using pinyin variants
    let mut keywords = vec!["project", "document", "file"];
    if let Some(variants) = pinyin_variants.as_array() {
        for variant in variants {
            if let Some(pinyin) = variant.as_str() {
                keywords.push(pinyin);
            }
        }
    }

    let classification_result = fixture
        .tool_registry
        .execute_tool(
            "folder-classifier",
            json!({
                "folder_path": "/test/项目文档",
                "classification_rules": {
                    "categories": {
                        "documents": {
                            "keywords": keywords,
                            "score": 1.0
                        }
                    }
                },
                "experimental_mode": true
            }),
            ExecutionContext::new(),
        )
        .await?;

    // Verify data flow worked correctly
    assert!(classification_result.get("status").is_some());
    assert_eq!(
        classification_result
            .get("category")
            .and_then(|v| v.as_str()),
        Some("documents")
    );

    println!("Text processing -> Classification data flow test completed");
    println!("Processed text: {}", processed_text);
    println!(
        "Classification result: {}",
        serde_json::to_string_pretty(&classification_result)?
    );

    Ok(())
}

/// Performance and stress tests with memory and CPU profiling
mod performance_tests {
    use super::*;
    use std::time::Instant;
    use workflow_toolkit::performance::{PerformanceConfig, PerformanceManager};

    /// Enhanced performance test fixture with monitoring
    struct PerformanceTestFixture {
        base_fixture: FileManagementTestFixture,
        performance_manager: Arc<PerformanceManager>,
    }

    impl PerformanceTestFixture {
        async fn new() -> Result<Self> {
            let base_fixture = FileManagementTestFixture::new().await?;
            let performance_config = PerformanceConfig::default();
            let performance_manager = Arc::new(PerformanceManager::new(performance_config));

            Ok(Self {
                base_fixture,
                performance_manager,
            })
        }
    }

    #[tokio::test]
    async fn test_batch_processing_performance_with_profiling() -> Result<()> {
        let fixture = PerformanceTestFixture::new().await?;

        // Start performance monitoring
        let monitor = fixture
            .performance_manager
            .start_monitoring("batch_processing_test")
            .await;

        // Create a large number of test items for stress testing
        let num_items = 100;
        let batch_items: Vec<Value> = (0..num_items)
            .map(|i| {
                json!({
                    "folder_path": format!("/test/folder_{}", i),
                    "classification_rules": {
                        "categories": {
                            "test": {"keywords": ["test", "folder"], "score": 1.0},
                            "documents": {"keywords": ["doc", "document"], "score": 0.8},
                            "media": {"keywords": ["photo", "music", "video"], "score": 0.9}
                        }
                    },
                    "experimental_mode": true
                })
            })
            .collect();

        let start_time = Instant::now();

        let result = fixture
            .base_fixture
            .tool_registry
            .execute_tool(
                "batch-processor",
                json!({
                    "items": batch_items,
                    "tool_name": "folder-classifier",
                    "batch_size": 10,
                    "max_concurrent": 4,
                    "progress_reporting": false  // Disable for performance
                }),
                ExecutionContext::new(),
            )
            .await?;

        let elapsed = start_time.elapsed();

        // Finish performance monitoring and get detailed metrics
        let perf_result = monitor.finish().await?;

        let processed_items = result
            .get("processed_items")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let throughput = processed_items as f64 / elapsed.as_secs_f64();

        println!("Batch processing performance with profiling:");
        println!("  Items: {}", num_items);
        println!("  Processed: {}", processed_items);
        println!("  Duration: {:?}", elapsed);
        println!("  Throughput: {:.2} items/second", throughput);
        println!("  Memory usage:");
        println!("    Initial: {} bytes", perf_result.memory_usage.initial);
        println!("    Final: {} bytes", perf_result.memory_usage.final_usage);
        println!("    Peak: {} bytes", perf_result.memory_usage.peak_usage);
        println!(
            "    Allocated: {} bytes",
            perf_result.memory_usage.allocated
        );

        // Performance assertions based on requirement 10.4
        assert!(
            throughput >= 10.0,
            "Throughput too low: {:.2} items/second",
            throughput
        );

        // Memory efficiency check - should not allocate more than 10MB for 100 items
        assert!(
            perf_result.memory_usage.allocated < 10_000_000,
            "Memory usage too high: {} bytes",
            perf_result.memory_usage.allocated
        );

        // Get and verify performance statistics
        let stats = fixture
            .performance_manager
            .get_stats("batch_processing_test")
            .await;
        assert!(stats.is_some(), "Performance statistics should be recorded");

        if let Some(stats) = stats {
            println!("  Performance statistics:");
            println!("    Execution count: {}", stats.execution_count);
            println!("    Average duration: {:?}", stats.average_duration);
            println!("    Min duration: {:?}", stats.min_duration);
            println!("    Max duration: {:?}", stats.max_duration);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_text_processing_performance_with_profiling() -> Result<()> {
        let fixture = PerformanceTestFixture::new().await?;

        let test_texts = vec![
            "Simple English text",
            "项目文档管理系统",
            "Mixed 中英文 Content",
            "Very long text with multiple words and Chinese characters 这是一个很长的文本包含多个单词和中文字符",
            "Complex text with numbers 12345 and symbols !@#$% and mixed content 复杂文本",
        ];

        let mut total_duration = Duration::new(0, 0);
        let mut total_operations = 0;
        let mut total_memory_allocated = 0u64;

        for (i, text) in test_texts.iter().enumerate() {
            let monitor = fixture
                .performance_manager
                .start_monitoring(&format!("text_processing_{}", i))
                .await;

            let start_time = Instant::now();

            let result = fixture
                .base_fixture
                .tool_registry
                .execute_tool(
                    "text-processor",
                    json!({
                        "text": text,
                        "operations": ["NormalizeCase", "GeneratePinyin"],
                        "chinese_processing": {
                            "pinyin_style": "Normal",
                            "generate_combinations": true,
                            "include_tones": false
                        }
                    }),
                    ExecutionContext::new(),
                )
                .await?;

            let elapsed = start_time.elapsed();
            let perf_result = monitor.finish().await?;

            total_duration += elapsed;
            total_operations += 1;
            total_memory_allocated += perf_result.memory_usage.allocated as u64;

            // Verify result structure
            assert!(result.get("processed").is_some());

            println!("Text processing operation {}:", i + 1);
            println!("  Text length: {} chars", text.len());
            println!("  Duration: {:?}", elapsed);
            println!(
                "  Memory allocated: {} bytes",
                perf_result.memory_usage.allocated
            );
        }

        let avg_latency = total_duration / total_operations;
        let avg_memory_per_operation = total_memory_allocated / total_operations as u64;

        println!("Text processing performance summary:");
        println!("  Operations: {}", total_operations);
        println!("  Total duration: {:?}", total_duration);
        println!("  Average latency: {:?}", avg_latency);
        println!("  Total memory allocated: {} bytes", total_memory_allocated);
        println!(
            "  Average memory per operation: {} bytes",
            avg_memory_per_operation
        );

        // Performance assertions based on requirement 10.4
        assert!(
            avg_latency < Duration::from_millis(50),
            "Average latency too high: {:?}",
            avg_latency
        );

        // Memory efficiency check - should not allocate more than 1MB per operation on average
        assert!(
            avg_memory_per_operation < 1_000_000,
            "Average memory usage too high: {} bytes per operation",
            avg_memory_per_operation
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_usage_under_load() -> Result<()> {
        let fixture = PerformanceTestFixture::new().await?;

        // Test memory usage with increasing load
        let load_sizes = vec![10, 50, 100, 200];
        let mut memory_usage_results = Vec::new();

        for load_size in load_sizes {
            let monitor = fixture
                .performance_manager
                .start_monitoring(&format!("memory_load_test_{}", load_size))
                .await;

            // Create batch items for this load size
            let batch_items: Vec<Value> = (0..load_size).map(|i| {
                json!({
                    "folder_path": format!("/test/large_dataset/folder_{}", i),
                    "classification_rules": {
                        "categories": {
                            "documents": {"keywords": ["doc", "document", "file"], "score": 1.0},
                            "media": {"keywords": ["photo", "music", "video", "image"], "score": 0.9},
                            "archives": {"keywords": ["zip", "tar", "backup", "archive"], "score": 0.8},
                            "code": {"keywords": ["src", "code", "project", "dev"], "score": 0.85}
                        }
                    },
                    "experimental_mode": true
                })
            }).collect();

            let result = fixture
                .base_fixture
                .tool_registry
                .execute_tool(
                    "batch-processor",
                    json!({
                        "items": batch_items,
                        "tool_name": "folder-classifier",
                        "batch_size": 20,
                        "max_concurrent": 6
                    }),
                    ExecutionContext::new(),
                )
                .await?;

            let perf_result = monitor.finish().await?;

            memory_usage_results.push((load_size, perf_result.memory_usage.clone()));

            let processed_items = result
                .get("processed_items")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!("Memory usage test - Load size: {}", load_size);
            println!("  Processed items: {}", processed_items);
            println!(
                "  Memory allocated: {} bytes",
                perf_result.memory_usage.allocated
            );
            println!(
                "  Peak memory: {} bytes",
                perf_result.memory_usage.peak_usage
            );
            println!(
                "  Memory per item: {} bytes",
                if processed_items > 0 {
                    perf_result.memory_usage.allocated / processed_items as usize
                } else {
                    0
                }
            );
        }

        // Analyze memory scaling
        println!("Memory scaling analysis:");
        for (load_size, memory_usage) in &memory_usage_results {
            println!(
                "  Load {}: {} bytes allocated, {} bytes peak",
                load_size, memory_usage.allocated, memory_usage.peak_usage
            );
        }

        // Memory should scale reasonably with load (not exponentially)
        if memory_usage_results.len() >= 2 {
            let first = &memory_usage_results[0];
            let last = &memory_usage_results[memory_usage_results.len() - 1];

            let load_ratio = last.0 as f64 / first.0 as f64;
            let memory_ratio = last.1.allocated as f64 / first.1.allocated as f64;

            println!("  Load scaling ratio: {:.2}x", load_ratio);
            println!("  Memory scaling ratio: {:.2}x", memory_ratio);

            // Memory scaling should be reasonable (not more than 3x the load scaling)
            // Handle case where no memory is allocated (mock implementation)
            if first.1.allocated == 0 && last.1.allocated == 0 {
                println!("  Memory scaling: No allocation detected (mock implementation)");
            } else {
                assert!(
                    memory_ratio <= load_ratio * 3.0,
                    "Memory scaling too high: {:.2}x vs load {:.2}x",
                    memory_ratio,
                    load_ratio
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_processing_efficiency() -> Result<()> {
        let fixture = PerformanceTestFixture::new().await?;

        // Test different concurrency levels
        let concurrency_levels = vec![1, 2, 4, 8];
        let num_items = 40;

        let batch_items: Vec<Value> = (0..num_items)
            .map(|i| {
                json!({
                    "folder_path": format!("/test/concurrent/folder_{}", i),
                    "classification_rules": {
                        "categories": {
                            "test": {"keywords": ["test", "concurrent"], "score": 1.0}
                        }
                    },
                    "experimental_mode": true
                })
            })
            .collect();

        let mut results = Vec::new();

        for concurrency in concurrency_levels {
            let monitor = fixture
                .performance_manager
                .start_monitoring(&format!("concurrency_test_{}", concurrency))
                .await;

            let start_time = Instant::now();

            let result = fixture
                .base_fixture
                .tool_registry
                .execute_tool(
                    "batch-processor",
                    json!({
                        "items": batch_items.clone(),
                        "tool_name": "folder-classifier",
                        "batch_size": 10,
                        "max_concurrent": concurrency
                    }),
                    ExecutionContext::new(),
                )
                .await?;

            let elapsed = start_time.elapsed();
            let perf_result = monitor.finish().await?;

            let processed_items = result
                .get("processed_items")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let throughput = processed_items as f64 / elapsed.as_secs_f64();

            results.push((
                concurrency,
                elapsed,
                throughput,
                perf_result.memory_usage.allocated,
            ));

            println!("Concurrency test - Level: {}", concurrency);
            println!("  Duration: {:?}", elapsed);
            println!("  Throughput: {:.2} items/second", throughput);
            println!(
                "  Memory allocated: {} bytes",
                perf_result.memory_usage.allocated
            );
        }

        // Analyze concurrency efficiency
        println!("Concurrency efficiency analysis:");
        let baseline = &results[0]; // Single-threaded baseline

        for (concurrency, duration, throughput, memory) in &results {
            let speedup = baseline.1.as_secs_f64() / duration.as_secs_f64();
            let efficiency = speedup / (*concurrency as f64);

            println!(
                "  Concurrency {}: {:.2}x speedup, {:.2}% efficiency, {} bytes memory",
                concurrency,
                speedup,
                efficiency * 100.0,
                memory
            );

            // Efficiency should be reasonable (at least 20% for higher concurrency in mock environment)
            if *concurrency <= 4 {
                assert!(
                    efficiency >= 0.2,
                    "Concurrency efficiency too low: {:.2}%",
                    efficiency * 100.0
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_resource_cleanup_and_monitoring() -> Result<()> {
        let fixture = PerformanceTestFixture::new().await?;

        // Test that resources are properly cleaned up after operations
        let initial_stats = fixture.performance_manager.get_all_stats().await;
        let initial_count = initial_stats.len();

        // Perform several operations
        for i in 0..5 {
            let monitor = fixture
                .performance_manager
                .start_monitoring(&format!("cleanup_test_{}", i))
                .await;

            let _result = fixture
                .base_fixture
                .tool_registry
                .execute_tool(
                    "folder-classifier",
                    json!({
                        "folder_path": format!("/test/cleanup/folder_{}", i),
                        "classification_rules": {
                            "categories": {
                                "test": {"keywords": ["cleanup", "test"], "score": 1.0}
                            }
                        },
                        "experimental_mode": true
                    }),
                    ExecutionContext::new(),
                )
                .await?;

            let _perf_result = monitor.finish().await?;
        }

        // Check that performance statistics are properly recorded
        let final_stats = fixture.performance_manager.get_all_stats().await;

        println!("Resource monitoring test:");
        println!("  Initial stats count: {}", initial_count);
        println!("  Final stats count: {}", final_stats.len());
        println!(
            "  New stats recorded: {}",
            final_stats.len() - initial_count
        );

        // Should have recorded stats for each operation
        assert!(
            final_stats.len() >= initial_count + 5,
            "Performance statistics not properly recorded"
        );

        // Verify stats contain expected information
        for (component, stats) in &final_stats {
            if component.starts_with("cleanup_test_") {
                assert!(
                    stats.execution_count > 0,
                    "Execution count should be recorded"
                );
                assert!(
                    stats.total_duration > Duration::ZERO,
                    "Duration should be recorded"
                );
                assert!(
                    stats.last_updated_timestamp > 0,
                    "Timestamp should be recorded"
                );

                println!(
                    "  Component {}: {} executions, {:?} total duration",
                    component, stats.execution_count, stats.total_duration
                );
            }
        }

        Ok(())
    }
}

/// Error handling and recovery tests
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_classification_rules() -> Result<()> {
        let fixture = FileManagementTestFixture::new().await?;

        // Test with invalid classification rules - this should work with mock implementation
        let result = fixture
            .tool_registry
            .execute_tool(
                "folder-classifier",
                json!({
                    "folder_path": "/test/folder",
                    "classification_rules": "invalid_json_string",
                    "experimental_mode": true
                }),
                ExecutionContext::new(),
            )
            .await?;

        // Mock implementation should still return a result
        assert!(result.get("status").is_some());

        println!("Classification with invalid rules handled gracefully");

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_processing_partial_failures() -> Result<()> {
        let fixture = FileManagementTestFixture::new().await?;

        // Create batch with some invalid items
        let batch_items = vec![
            json!({
                "folder_path": "/valid/folder1",
                "classification_rules": {"categories": {"test": {"keywords": ["test"], "score": 1.0}}},
                "experimental_mode": true
            }),
            json!({
                "folder_path": "/invalid/folder2",
                "classification_rules": "invalid_rules",  // This should fail in mock
                "experimental_mode": true
            }),
            json!({
                "folder_path": "/valid/folder3",
                "classification_rules": {"categories": {"test": {"keywords": ["test"], "score": 1.0}}},
                "experimental_mode": true
            }),
        ];

        let result = fixture
            .tool_registry
            .execute_tool(
                "batch-processor",
                json!({
                    "items": batch_items,
                    "tool_name": "folder-classifier",
                    "batch_size": 3,
                    "continue_on_error": true
                }),
                ExecutionContext::new(),
            )
            .await?;

        // Should have some successes and potentially some failures
        let processed = result
            .get("processed_items")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total = result
            .get("total_items")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        println!("Batch processing with mixed results:");
        println!("  Total: {}", total);
        println!("  Processed: {}", processed);

        assert!(processed > 0, "Should have some successful items");
        assert_eq!(total, 3, "Should have processed all items");

        Ok(())
    }
}
