// Real-World Scenario Example: Digital Asset Management for Creative Agency
// This example demonstrates a comprehensive file management workflow for a creative agency
// that needs to organize client projects, media assets, and deliverables with human oversight

use std::collections::HashMap;
use std::path::PathBuf;
use serde_json::json;
use workflow_toolkit::core::WorkflowDefinition;
use workflow_toolkit::workflow::engine::WorkflowEngine;
use workflow_toolkit::plugins::file_management::FileManagementPlugin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Creative Agency Digital Asset Management Example");
    println!("==================================================");
    
    // Initialize the workflow engine with file management plugin
    let mut engine = WorkflowEngine::new().await?;
    let file_management_plugin = FileManagementPlugin::new()?;
    engine.register_plugin(Box::new(file_management_plugin)).await?;
    
    // Run different scenarios
    run_client_project_organization(&engine).await?;
    run_media_asset_consolidation(&engine).await?;
    run_deliverable_preparation(&engine).await?;
    
    Ok(())
}

/// Scenario 1: Organize incoming client project files
async fn run_client_project_organization(engine: &WorkflowEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📁 Scenario 1: Client Project Organization");
    println!("==========================================");
    
    // Create a workflow for organizing client project files
    let workflow = WorkflowDefinition {
        name: "client_project_organization".to_string(),
        description: "Organize incoming client files by project and asset type".to_string(),
        parameters: json!({
            "source_directory": "${AGENCY_INTAKE_DIR}/new_projects",
            "output_directory": "${AGENCY_PROJECTS_DIR}/organized",
            "classification_rules": {
                "version": "1.0",
                "categories": {
                    "brand_assets": {
                        "description": "Brand guidelines, logos, and identity materials",
                        "keywords": [
                            {"pattern": "brand", "weight": 1.0},
                            {"pattern": "logo", "weight": 1.0},
                            {"pattern": "identity", "weight": 0.9},
                            {"pattern": "guideline", "weight": 0.9},
                            {"pattern": "style", "weight": 0.8}
                        ],
                        "target_directory": "BrandAssets"
                    },
                    "photography": {
                        "description": "Photos, product shots, and photography assets",
                        "keywords": [
                            {"pattern": "photo", "weight": 1.0},
                            {"pattern": "image", "weight": 0.9},
                            {"pattern": "shot", "weight": 0.8},
                            {"pattern": "photography", "weight": 1.0},
                            {"pattern": "product", "weight": 0.7}
                        ],
                        "target_directory": "Photography"
                    },
                    "video_content": {
                        "description": "Video files, animations, and motion graphics",
                        "keywords": [
                            {"pattern": "video", "weight": 1.0},
                            {"pattern": "animation", "weight": 0.9},
                            {"pattern": "motion", "weight": 0.8},
                            {"pattern": "footage", "weight": 0.8},
                            {"pattern": "clip", "weight": 0.7}
                        ],
                        "target_directory": "VideoContent"
                    },
                    "design_files": {
                        "description": "Design source files and creative assets",
                        "keywords": [
                            {"pattern": "design", "weight": 1.0},
                            {"pattern": "creative", "weight": 0.9},
                            {"pattern": "artwork", "weight": 0.9},
                            {"pattern": "layout", "weight": 0.8},
                            {"pattern": "mockup", "weight": 0.8}
                        ],
                        "target_directory": "DesignFiles"
                    },
                    "client_content": {
                        "description": "Content provided by client",
                        "keywords": [
                            {"pattern": "client", "weight": 1.0},
                            {"pattern": "provided", "weight": 0.8},
                            {"pattern": "content", "weight": 0.7},
                            {"pattern": "copy", "weight": 0.7},
                            {"pattern": "text", "weight": 0.6}
                        ],
                        "target_directory": "ClientContent"
                    }
                },
                "settings": {
                    "minimum_score_threshold": 0.6,
                    "ambiguous_threshold": 0.25,
                    "enable_fuzzy_matching": true
                }
            },
            "experimental_mode": true,
            "enable_user_interaction": true,
            "batch_size": 8,
            "decision_timeout": 240,
            "create_backups": true,
            "human_decisions": {
                "confidence_threshold": 0.8,
                "enable_batch_decisions": true,
                "show_file_previews": true,
                "include_metadata": true
            }
        }),
        steps: vec![
            // Step 1: Analyze project structure and detect client information
            json!({
                "name": "analyze_project_structure",
                "tool": "folder-classifier",
                "params": {
                    "folder_path": "{{ source_directory }}",
                    "classification_rules": "{{ classification_rules }}",
                    "analysis_only": true,
                    "detect_client_info": true,
                    "extract_project_metadata": true
                }
            }),
            
            // Step 2: Human decision for client and project identification
            json!({
                "name": "identify_client_and_project",
                "tool": "human-decision",
                "params": {
                    "decision_type": "Custom",
                    "context": {
                        "title": "Client and Project Identification",
                        "description": "Please identify the client and project name for proper organization",
                        "metadata": {
                            "detected_folders": "{{ analyze_project_structure.folder_count }}",
                            "suggested_client": "{{ analyze_project_structure.suggested_client }}",
                            "suggested_project": "{{ analyze_project_structure.suggested_project }}"
                        }
                    },
                    "options": [
                        {
                            "id": "use_detected",
                            "label": "Use detected client/project names",
                            "description": "Use the automatically detected names"
                        },
                        {
                            "id": "specify_custom",
                            "label": "Specify custom names",
                            "description": "Enter custom client and project names"
                        },
                        {
                            "id": "skip_organization",
                            "label": "Skip client organization",
                            "description": "Organize by asset type only"
                        }
                    ],
                    "timeout_seconds": "{{ decision_timeout }}",
                    "allow_custom_input": true
                }
            }),
            
            // Step 3: Classify assets with client/project context
            json!({
                "name": "classify_assets_with_context",
                "tool": "folder-classifier",
                "params": {
                    "folder_path": "{{ source_directory }}",
                    "classification_rules": "{{ classification_rules }}",
                    "client_name": "{{ identify_client_and_project.client_name }}",
                    "project_name": "{{ identify_client_and_project.project_name }}",
                    "experimental_mode": "{{ experimental_mode }}",
                    "enable_user_interaction": "{{ enable_user_interaction }}"
                }
            }),
            
            // Step 4: Human review of asset classifications
            json!({
                "name": "review_asset_classifications",
                "tool": "human-decision",
                "condition": "{{ classify_assets_with_context.ambiguous_count > 0 }}",
                "params": {
                    "decision_type": "Classification",
                    "context": {
                        "title": "Asset Classification Review",
                        "description": "Review and confirm asset classifications for the project",
                        "metadata": {
                            "client": "{{ identify_client_and_project.client_name }}",
                            "project": "{{ identify_client_and_project.project_name }}",
                            "total_assets": "{{ classify_assets_with_context.total_processed }}",
                            "auto_classified": "{{ classify_assets_with_context.auto_classified_count }}",
                            "needs_review": "{{ classify_assets_with_context.ambiguous_count }}"
                        }
                    },
                    "items": "{{ classify_assets_with_context.ambiguous_results }}",
                    "timeout_seconds": "{{ decision_timeout }}",
                    "enable_batch_decisions": "{{ human_decisions.enable_batch_decisions }}",
                    "show_previews": "{{ human_decisions.show_file_previews }}"
                }
            }),
            
            // Step 5: Execute organization with proper folder structure
            json!({
                "name": "execute_project_organization",
                "tool": "file-mover",
                "params": {
                    "operations": "{{ classify_assets_with_context.operations + review_asset_classifications.operations }}",
                    "target_structure": "{{ identify_client_and_project.client_name }}/{{ identify_client_and_project.project_name }}/{{ asset_category }}",
                    "conflict_resolution": "Rename",
                    "experimental_mode": "{{ experimental_mode }}",
                    "create_directories": true,
                    "preserve_timestamps": true
                }
            }),
            
            // Step 6: Generate project organization report
            json!({
                "name": "generate_project_report",
                "tool": "report-generator",
                "params": {
                    "client_info": "{{ identify_client_and_project }}",
                    "classification_results": "{{ classify_assets_with_context.summary }}",
                    "organization_results": "{{ execute_project_organization.results }}",
                    "include_asset_inventory": true,
                    "generate_client_summary": true
                }
            })
        ],
        ..Default::default()
    };
    
    println!("🔄 Executing client project organization workflow...");
    
    // Set environment variables for the example
    std::env::set_var("AGENCY_INTAKE_DIR", "/tmp/agency_example/intake");
    std::env::set_var("AGENCY_PROJECTS_DIR", "/tmp/agency_example/projects");
    
    // Create example directory structure
    create_example_project_structure().await?;
    
    // Execute the workflow
    match engine.execute_workflow(&workflow).await {
        Ok(result) => {
            println!("✅ Client project organization completed successfully!");
            println!("📊 Results: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("❌ Workflow execution failed: {}", e);
        }
    }
    
    Ok(())
}

/// Scenario 2: Consolidate media assets from multiple sources
async fn run_media_asset_consolidation(engine: &WorkflowEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎬 Scenario 2: Media Asset Consolidation");
    println!("=======================================");
    
    let workflow = WorkflowDefinition {
        name: "media_asset_consolidation".to_string(),
        description: "Consolidate media assets from multiple sources with duplicate detection".to_string(),
        parameters: json!({
            "source_directories": [
                "${AGENCY_MEDIA_DIR}/raw_footage",
                "${AGENCY_MEDIA_DIR}/stock_photos",
                "${AGENCY_MEDIA_DIR}/client_assets",
                "${AGENCY_MEDIA_DIR}/temp_downloads"
            ],
            "target_directory": "${AGENCY_MEDIA_DIR}/consolidated",
            "backup_directory": "${AGENCY_BACKUP_DIR}/media_consolidation",
            "merge_strategy": "UserDecision",
            "duplicate_handling": "UserDecision",
            "experimental_mode": true
        }),
        steps: vec![
            // Step 1: Analyze media assets across all sources
            json!({
                "name": "analyze_media_assets",
                "tool": "folder-merger",
                "params": {
                    "source_directories": "{{ source_directories }}",
                    "analysis_only": true,
                    "enable_size_analysis": true,
                    "detect_duplicates": true,
                    "analyze_media_metadata": true,
                    "minimum_file_size": 10240  // 10KB minimum
                }
            }),
            
            // Step 2: Create backup of original assets
            json!({
                "name": "create_media_backup",
                "tool": "file-mover",
                "params": {
                    "operations": "{{ analyze_media_assets.backup_operations }}",
                    "target_directory": "{{ backup_directory }}",
                    "operation_type": "Copy",
                    "preserve_structure": true,
                    "verify_checksums": true
                }
            }),
            
            // Step 3: Human decision for duplicate resolution strategy
            json!({
                "name": "resolve_duplicate_strategy",
                "tool": "human-decision",
                "condition": "{{ analyze_media_assets.duplicate_count > 0 }}",
                "params": {
                    "decision_type": "Custom",
                    "context": {
                        "title": "Media Duplicate Resolution Strategy",
                        "description": "Multiple versions of media files were found. Choose resolution strategy.",
                        "metadata": {
                            "total_duplicates": "{{ analyze_media_assets.duplicate_count }}",
                            "potential_space_savings": "{{ analyze_media_assets.space_savings }}",
                            "duplicate_types": "{{ analyze_media_assets.duplicate_analysis }}"
                        }
                    },
                    "options": [
                        {
                            "id": "keep_highest_quality",
                            "label": "Keep highest quality version",
                            "description": "Automatically keep the version with highest resolution/quality",
                            "recommended": true
                        },
                        {
                            "id": "keep_largest_file",
                            "label": "Keep largest file size",
                            "description": "Keep the version with the largest file size"
                        },
                        {
                            "id": "keep_most_recent",
                            "label": "Keep most recent version",
                            "description": "Keep the version with the most recent modification date"
                        },
                        {
                            "id": "review_individually",
                            "label": "Review each duplicate individually",
                            "description": "Make decisions for each duplicate set manually"
                        }
                    ],
                    "timeout_seconds": 300
                }
            }),
            
            // Step 4: Individual duplicate review (if requested)
            json!({
                "name": "review_individual_duplicates",
                "tool": "human-decision",
                "condition": "{{ resolve_duplicate_strategy.selected_option == 'review_individually' }}",
                "params": {
                    "decision_type": "FileConflict",
                    "context": {
                        "title": "Individual Duplicate Review",
                        "description": "Review each set of duplicate media files"
                    },
                    "conflicts": "{{ analyze_media_assets.duplicate_sets }}",
                    "batch_resolution": false,
                    "show_file_previews": true,
                    "include_metadata": true,
                    "timeout_seconds": 180
                }
            }),
            
            // Step 5: Execute media consolidation
            json!({
                "name": "execute_media_consolidation",
                "tool": "folder-merger",
                "params": {
                    "source_directories": "{{ source_directories }}",
                    "target_directory": "{{ target_directory }}",
                    "duplicate_strategy": "{{ resolve_duplicate_strategy.selected_option }}",
                    "individual_resolutions": "{{ review_individual_duplicates.resolutions }}",
                    "organize_by_type": true,
                    "preserve_metadata": true,
                    "experimental_mode": "{{ experimental_mode }}"
                }
            }),
            
            // Step 6: Verify consolidation results
            json!({
                "name": "verify_consolidation_results",
                "tool": "verification-checker",
                "params": {
                    "consolidation_results": "{{ execute_media_consolidation.results }}",
                    "verify_file_integrity": true,
                    "check_metadata_preservation": true,
                    "validate_organization": true
                }
            }),
            
            // Step 7: Generate media asset inventory
            json!({
                "name": "generate_media_inventory",
                "tool": "inventory-generator",
                "params": {
                    "consolidated_assets": "{{ execute_media_consolidation.results }}",
                    "include_thumbnails": true,
                    "extract_metadata": true,
                    "categorize_by_type": true,
                    "generate_usage_report": true
                }
            })
        ],
        ..Default::default()
    };
    
    println!("🔄 Executing media asset consolidation workflow...");
    
    // Set environment variables
    std::env::set_var("AGENCY_MEDIA_DIR", "/tmp/agency_example/media");
    std::env::set_var("AGENCY_BACKUP_DIR", "/tmp/agency_example/backups");
    
    // Create example media structure
    create_example_media_structure().await?;
    
    // Execute the workflow
    match engine.execute_workflow(&workflow).await {
        Ok(result) => {
            println!("✅ Media asset consolidation completed successfully!");
            println!("📊 Results: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("❌ Workflow execution failed: {}", e);
        }
    }
    
    Ok(())
}

/// Scenario 3: Prepare deliverables for client presentation
async fn run_deliverable_preparation(engine: &WorkflowEngine) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📦 Scenario 3: Client Deliverable Preparation");
    println!("=============================================");
    
    let workflow = WorkflowDefinition {
        name: "deliverable_preparation".to_string(),
        description: "Prepare and package client deliverables with quality checks".to_string(),
        parameters: json!({
            "project_directory": "${AGENCY_PROJECTS_DIR}/ClientA/BrandRefresh2024",
            "deliverable_directory": "${AGENCY_DELIVERABLES_DIR}/ClientA_BrandRefresh_Final",
            "deliverable_types": [
                "final_designs",
                "brand_guidelines",
                "digital_assets",
                "print_ready_files",
                "web_assets"
            ],
            "quality_checks": true,
            "client_review_mode": true,
            "experimental_mode": true
        }),
        steps: vec![
            // Step 1: Analyze project assets for deliverable preparation
            json!({
                "name": "analyze_project_assets",
                "tool": "project-analyzer",
                "params": {
                    "project_directory": "{{ project_directory }}",
                    "deliverable_types": "{{ deliverable_types }}",
                    "check_completeness": true,
                    "validate_file_formats": true,
                    "assess_quality": "{{ quality_checks }}"
                }
            }),
            
            // Step 2: Human review of deliverable completeness
            json!({
                "name": "review_deliverable_completeness",
                "tool": "human-decision",
                "params": {
                    "decision_type": "Custom",
                    "context": {
                        "title": "Deliverable Completeness Review",
                        "description": "Review project assets and confirm deliverable completeness",
                        "metadata": {
                            "project_name": "{{ project_directory | basename }}",
                            "total_assets": "{{ analyze_project_assets.total_assets }}",
                            "deliverable_coverage": "{{ analyze_project_assets.coverage_analysis }}",
                            "missing_items": "{{ analyze_project_assets.missing_items }}",
                            "quality_issues": "{{ analyze_project_assets.quality_issues }}"
                        }
                    },
                    "options": [
                        {
                            "id": "proceed_with_current",
                            "label": "Proceed with current assets",
                            "description": "Use available assets for deliverable package"
                        },
                        {
                            "id": "add_missing_items",
                            "label": "Add missing items first",
                            "description": "Identify and add missing deliverable items"
                        },
                        {
                            "id": "fix_quality_issues",
                            "label": "Fix quality issues first",
                            "description": "Address quality issues before packaging"
                        },
                        {
                            "id": "postpone_delivery",
                            "label": "Postpone deliverable preparation",
                            "description": "Project not ready for client delivery"
                        }
                    ],
                    "timeout_seconds": 600  // 10 minutes for thorough review
                }
            }),
            
            // Step 3: Quality assurance checks
            json!({
                "name": "perform_quality_checks",
                "tool": "quality-checker",
                "condition": "{{ review_deliverable_completeness.selected_option != 'postpone_delivery' }}",
                "params": {
                    "project_assets": "{{ analyze_project_assets.assets }}",
                    "check_file_integrity": true,
                    "validate_color_profiles": true,
                    "check_resolution_requirements": true,
                    "verify_font_embedding": true,
                    "validate_print_specifications": true
                }
            }),
            
            // Step 4: Human review of quality check results
            json!({
                "name": "review_quality_results",
                "tool": "human-decision",
                "condition": "{{ perform_quality_checks.issues_found > 0 }}",
                "params": {
                    "decision_type": "Custom",
                    "context": {
                        "title": "Quality Check Results Review",
                        "description": "Quality issues were found that may affect client deliverables",
                        "metadata": {
                            "total_issues": "{{ perform_quality_checks.issues_found }}",
                            "critical_issues": "{{ perform_quality_checks.critical_issues }}",
                            "warning_issues": "{{ perform_quality_checks.warning_issues }}",
                            "issue_summary": "{{ perform_quality_checks.issue_summary }}"
                        }
                    },
                    "options": [
                        {
                            "id": "fix_critical_only",
                            "label": "Fix critical issues only",
                            "description": "Address only critical issues that would prevent delivery"
                        },
                        {
                            "id": "fix_all_issues",
                            "label": "Fix all identified issues",
                            "description": "Address all quality issues before delivery"
                        },
                        {
                            "id": "deliver_with_notes",
                            "label": "Deliver with quality notes",
                            "description": "Include quality notes with deliverables"
                        },
                        {
                            "id": "postpone_for_fixes",
                            "label": "Postpone delivery for fixes",
                            "description": "Delay delivery to address quality issues"
                        }
                    ],
                    "timeout_seconds": 300
                }
            }),
            
            // Step 5: Organize deliverables by type and format
            json!({
                "name": "organize_deliverables",
                "tool": "deliverable-organizer",
                "condition": "{{ review_quality_results.selected_option != 'postpone_for_fixes' }}",
                "params": {
                    "project_assets": "{{ analyze_project_assets.approved_assets }}",
                    "deliverable_directory": "{{ deliverable_directory }}",
                    "deliverable_types": "{{ deliverable_types }}",
                    "organization_structure": {
                        "final_designs": "01_Final_Designs",
                        "brand_guidelines": "02_Brand_Guidelines", 
                        "digital_assets": "03_Digital_Assets",
                        "print_ready_files": "04_Print_Ready",
                        "web_assets": "05_Web_Assets"
                    },
                    "include_previews": true,
                    "generate_thumbnails": true
                }
            }),
            
            // Step 6: Create client presentation package
            json!({
                "name": "create_presentation_package",
                "tool": "package-creator",
                "params": {
                    "organized_deliverables": "{{ organize_deliverables.results }}",
                    "package_directory": "{{ deliverable_directory }}",
                    "include_readme": true,
                    "include_usage_guide": true,
                    "create_preview_pdf": true,
                    "generate_asset_list": true,
                    "client_branding": true
                }
            }),
            
            // Step 7: Final review and approval
            json!({
                "name": "final_deliverable_review",
                "tool": "human-decision",
                "params": {
                    "decision_type": "Custom",
                    "context": {
                        "title": "Final Deliverable Package Review",
                        "description": "Review the complete deliverable package before client delivery",
                        "metadata": {
                            "package_size": "{{ create_presentation_package.package_size }}",
                            "total_files": "{{ create_presentation_package.file_count }}",
                            "deliverable_types": "{{ create_presentation_package.included_types }}",
                            "quality_status": "{{ perform_quality_checks.final_status }}",
                            "package_location": "{{ deliverable_directory }}"
                        }
                    },
                    "options": [
                        {
                            "id": "approve_for_delivery",
                            "label": "Approve for client delivery",
                            "description": "Package is ready for client delivery",
                            "recommended": true
                        },
                        {
                            "id": "request_changes",
                            "label": "Request changes",
                            "description": "Make additional changes before delivery"
                        },
                        {
                            "id": "add_supplementary",
                            "label": "Add supplementary materials",
                            "description": "Include additional materials or documentation"
                        }
                    ],
                    "timeout_seconds": 600,
                    "allow_custom_notes": true
                }
            }),
            
            // Step 8: Generate delivery report and documentation
            json!({
                "name": "generate_delivery_documentation",
                "tool": "documentation-generator",
                "condition": "{{ final_deliverable_review.selected_option == 'approve_for_delivery' }}",
                "params": {
                    "deliverable_package": "{{ create_presentation_package.package_info }}",
                    "project_summary": "{{ analyze_project_assets.project_summary }}",
                    "quality_report": "{{ perform_quality_checks.report }}",
                    "delivery_notes": "{{ final_deliverable_review.custom_notes }}",
                    "include_handoff_checklist": true,
                    "generate_client_email": true
                }
            })
        ],
        ..Default::default()
    };
    
    println!("🔄 Executing deliverable preparation workflow...");
    
    // Set environment variables
    std::env::set_var("AGENCY_DELIVERABLES_DIR", "/tmp/agency_example/deliverables");
    
    // Create example project structure
    create_example_deliverable_structure().await?;
    
    // Execute the workflow
    match engine.execute_workflow(&workflow).await {
        Ok(result) => {
            println!("✅ Deliverable preparation completed successfully!");
            println!("📊 Results: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("❌ Workflow execution failed: {}", e);
        }
    }
    
    Ok(())
}

/// Create example project directory structure for testing
async fn create_example_project_structure() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::fs;
    
    let base_dir = PathBuf::from("/tmp/agency_example");
    
    // Create directory structure
    let dirs = vec![
        "intake/new_projects/ClientA_BrandRefresh",
        "intake/new_projects/ClientB_WebsiteRedesign", 
        "projects/organized",
        "media/raw_footage",
        "media/stock_photos",
        "media/client_assets",
        "media/temp_downloads",
        "media/consolidated",
        "deliverables",
        "backups"
    ];
    
    for dir in dirs {
        let path = base_dir.join(dir);
        fs::create_dir_all(&path).await?;
        println!("📁 Created directory: {}", path.display());
    }
    
    // Create some example files
    let example_files = vec![
        ("intake/new_projects/ClientA_BrandRefresh/brand_guidelines.pdf", "Brand guidelines content"),
        ("intake/new_projects/ClientA_BrandRefresh/logo_files.zip", "Logo files archive"),
        ("intake/new_projects/ClientA_BrandRefresh/product_photos.zip", "Product photography"),
        ("intake/new_projects/ClientA_BrandRefresh/video_content.mov", "Video content file"),
        ("intake/new_projects/ClientB_WebsiteRedesign/design_mockups.psd", "Design mockups"),
        ("intake/new_projects/ClientB_WebsiteRedesign/client_content.docx", "Client provided content"),
    ];
    
    for (file_path, content) in example_files {
        let path = base_dir.join(file_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&path, content).await?;
        println!("📄 Created file: {}", path.display());
    }
    
    Ok(())
}

/// Create example media directory structure
async fn create_example_media_structure() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::fs;
    
    let base_dir = PathBuf::from("/tmp/agency_example/media");
    
    // Create example media files with potential duplicates
    let media_files = vec![
        ("raw_footage/project_a_interview.mp4", "Interview footage"),
        ("raw_footage/project_a_broll.mp4", "B-roll footage"),
        ("stock_photos/business_meeting.jpg", "Stock photo"),
        ("stock_photos/office_space.jpg", "Office stock photo"),
        ("client_assets/logo_v1.png", "Client logo version 1"),
        ("client_assets/logo_v2.png", "Client logo version 2"),
        ("temp_downloads/logo_v1.png", "Duplicate logo"), // Duplicate
        ("temp_downloads/stock_photo_business.jpg", "Another business photo"),
    ];
    
    for (file_path, content) in media_files {
        let path = base_dir.join(file_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&path, content).await?;
        println!("🎬 Created media file: {}", path.display());
    }
    
    Ok(())
}

/// Create example deliverable project structure
async fn create_example_deliverable_structure() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::fs;
    
    let project_dir = PathBuf::from("/tmp/agency_example/projects/ClientA/BrandRefresh2024");
    fs::create_dir_all(&project_dir).await?;
    
    // Create example project files
    let project_files = vec![
        ("final_logo.ai", "Adobe Illustrator logo file"),
        ("brand_guidelines.pdf", "Complete brand guidelines"),
        ("color_palette.ase", "Adobe color swatches"),
        ("typography_guide.pdf", "Typography specifications"),
        ("business_card_design.ai", "Business card design"),
        ("letterhead_design.ai", "Letterhead design"),
        ("web_assets/logo_web.png", "Web optimized logo"),
        ("web_assets/favicon.ico", "Website favicon"),
        ("print_assets/logo_print.eps", "Print ready logo"),
        ("print_assets/business_card_print.pdf", "Print ready business card"),
    ];
    
    for (file_path, content) in project_files {
        let path = project_dir.join(file_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&path, content).await?;
        println!("📋 Created project file: {}", path.display());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_example_structures() {
        create_example_project_structure().await.unwrap();
        create_example_media_structure().await.unwrap();
        create_example_deliverable_structure().await.unwrap();
        
        // Verify structures were created
        assert!(PathBuf::from("/tmp/agency_example/intake").exists());
        assert!(PathBuf::from("/tmp/agency_example/media").exists());
        assert!(PathBuf::from("/tmp/agency_example/projects").exists());
    }
}