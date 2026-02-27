//! Granular Classification Workflow Tools
//!
//! This module implements specific single-function tools for the 13-step
//! folder classification workflow.

use crate::core::ToolInfo;
use crate::plugins::file_management::utils::utils::{TextNormalizationConfig, TextProcessor};
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

// Helper to create ToolInfo
fn create_tool_info(name: &str, description: &str) -> ToolInfo {
    ToolInfo {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: description.to_string(),
        category: Some("classification".to_string()),
        tags: vec!["file-management".to_string(), "classification".to_string()],
        parameters_schema: Value::Null,
        return_schema: Value::Null,
        dependencies: Vec::new(),
        plugin_name: Some("file-management".to_string()),
        version_requirements: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// --- 1. Rule Reading & Validation ---

pub struct RuleLoaderTool;

// --- 2. Rule Preprocessing ---

pub struct RulePreprocessorTool {
    text_processor: TextProcessor,
}

impl RulePreprocessorTool {
    pub fn new(enable_chinese: bool) -> Self {
        Self {
            text_processor: TextProcessor::with_config(
                enable_chinese,
                TextNormalizationConfig::default(),
            ),
        }
    }
}

// --- 3. AC Automaton Construction ---

pub struct AutomatonBuilderTool;

// --- 4. Source Directory Scanning ---

pub struct DirectoryScannerTool;

// --- 5. Folder Name Preprocessing ---

pub struct FolderNamePreprocessorTool {
    text_processor: TextProcessor,
}

impl FolderNamePreprocessorTool {
    pub fn new(enable_chinese: bool) -> Self {
        Self {
            text_processor: TextProcessor::with_config(
                enable_chinese,
                TextNormalizationConfig::default(),
            ),
        }
    }
}

// --- 6. Parallel Matching Execution ---

pub struct ParallelMatcherTool;

// --- 7. Score Calculation ---

pub struct ScoreCalculatorTool;

// --- 8. Ambiguity Detection ---

pub struct AmbiguityDetectorTool;

// --- 9. Result Merging ---

pub struct ResultMergerTool;

// --- 10. Experimental Mode Check ---

pub struct ExperimentalCheckTool;

// --- 13. Report Generation ---

pub struct ReportGeneratorTool;
