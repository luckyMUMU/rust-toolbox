//! Granular Classification Workflow Tools
//!
//! This module implements specific single-function tools for the 13-step
//! folder classification workflow.

use crate::tools::algo::ac_automaton::{AhoCorasickMatcher, AutomatonConfig};
use super::classification_tool::ClassificationRules;
use super::utils::{TextNormalizationConfig, TextProcessor};
use crate::core::{ExecutionContext, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::tools::ToolNode;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

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

#[async_trait]
impl ToolNode for RuleLoaderTool {
    fn name(&self) -> &str {
        "rule-loader"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("rules").is_none() {
            return Err(WorkflowError::validation("rules parameter required"));
        }
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("rule-loader", "Loads and validates classification rules")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let rules_input = params
            .get("rules")
            .ok_or_else(|| WorkflowError::validation("rules parameter required"))?;

        // 1. Load initial Value
        let mut rules_value = if let Some(path) = rules_input.as_str() {
            info!("Attempting to read rules from path: {}", path);
            let content = std::fs::read_to_string(path).map_err(|e| {
                WorkflowError::tool(format!("Failed to read rules file '{}': {}", path, e))
            })?;
            serde_json::from_str::<Value>(&content)
                .map_err(|e| WorkflowError::validation(format!("Invalid rules JSON: {}", e)))?
        } else {
            rules_input.clone()
        };

        // 2. Apply transformations (Legacy Support)
        if let Some(obj) = rules_value.as_object_mut() {
            // Legacy "categories" -> "rules"
            if obj.contains_key("categories") && !obj.contains_key("rules") {
                if let Some(categories) = obj.remove("categories") {
                    obj.insert("rules".to_string(), categories);
                }
            }

            // Transform nested keywords into combinations and name -> category
            if let Some(rules) = obj.get_mut("rules").and_then(|r| r.as_array_mut()) {
                for rule in rules {
                    if let Some(rule_obj) = rule.as_object_mut() {
                        // Legacy "name" -> "category"
                        if rule_obj.contains_key("name") && !rule_obj.contains_key("category") {
                            if let Some(name) = rule_obj.remove("name") {
                                rule_obj.insert("category".to_string(), name);
                            }
                        }

                        // Legacy "priority" -> "score_weight"
                        // Python logic: priority_weight = (1 + priority * 0.1)
                        if let Some(priority) = rule_obj.get("priority").and_then(|v| v.as_i64()) {
                            if !rule_obj.contains_key("score_weight") {
                                let weight = 1.0 + (priority as f64) * 0.1;
                                rule_obj.insert("score_weight".to_string(), json!(weight));
                            }
                        }

                        // Process keywords
                        if let Some(keywords_val) = rule_obj.get_mut("keywords") {
                            if let Some(keywords_arr) = keywords_val.as_array() {
                                let mut simple_keywords = Vec::new();
                                let mut combinations = Vec::new();

                                for item in keywords_arr {
                                    if let Some(s) = item.as_str() {
                                        simple_keywords.push(Value::String(s.to_string()));
                                    } else if let Some(arr) = item.as_array() {
                                        let mut combo = Vec::new();
                                        for sub_item in arr {
                                            if let Some(s) = sub_item.as_str() {
                                                combo.push(s.to_string());
                                            }
                                        }
                                        if !combo.is_empty() {
                                            combinations.push(combo);
                                        }
                                    }
                                }

                                *keywords_val = Value::Array(simple_keywords);

                                if !combinations.is_empty() {
                                    rule_obj
                                        .insert("combinations".to_string(), json!(combinations));
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Deserialize to struct
        let rules: ClassificationRules = serde_json::from_value(rules_value).map_err(|e| {
            WorkflowError::validation(format!("Invalid classification rules structure: {}", e))
        })?;

        // Basic validation
        if rules.rules.is_empty() {
            return Err(WorkflowError::validation("Rules cannot be empty"));
        }

        Ok(serde_json::to_value(rules)?)
    }
}

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

#[async_trait]
impl ToolNode for RulePreprocessorTool {
    fn name(&self) -> &str {
        "rule-preprocessor"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info(
            "rule-preprocessor",
            "Preprocesses rules (pinyin, lowercase)",
        )
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let rules: ClassificationRules =
            serde_json::from_value(params.get("rules").unwrap_or(&json!({})).clone())
                .map_err(|e| WorkflowError::validation(format!("Invalid rules: {}", e)))?;

        let mut processed_patterns = Vec::new();
        let mut pattern_id_counter = 0;

        for rule in rules.rules {
            let mut keywords = rule.keywords.clone();

            if let Some(combinations) = &rule.combinations {
                for combo in combinations {
                    keywords.extend(combo.clone());
                }
            }

            for keyword in keywords {
                // Lowercase
                let mut variants = vec![keyword.to_lowercase()];

                // Chinese variants
                if self.text_processor.contains_chinese(&keyword) {
                    let pinyin = self
                        .text_processor
                        .generate_comprehensive_pinyin(&keyword, super::utils::PinyinStyle::Normal);
                    variants.extend(pinyin.pinyin_variants);

                    let mixed = self.text_processor.process_mixed_text(&keyword);
                    if mixed.simplified_chinese != keyword {
                        variants.push(mixed.simplified_chinese);
                    }
                    // Note: Traditional Chinese conversion not supported in current TextProcessor (only to Simplified)
                }

                for variant in variants {
                    processed_patterns.push(json!({
                        "pattern": variant,
                        "category": rule.category,
                        "score": rule.score_weight,
                        "id": pattern_id_counter,
                        "is_combination": false
                    }));
                    pattern_id_counter += 1;
                }
            }
        }

        Ok(json!({ "patterns": processed_patterns }))
    }
}

// --- 3. AC Automaton Construction ---

pub struct AutomatonBuilderTool;

#[async_trait]
impl ToolNode for AutomatonBuilderTool {
    fn name(&self) -> &str {
        "ac-automaton-builder"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("patterns").is_none() {
            return Err(WorkflowError::validation("patterns parameter required"));
        }
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("ac-automaton-builder", "Builds AC Automaton configuration")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let patterns = params
            .get("patterns")
            .and_then(|p| p.as_array())
            .ok_or_else(|| WorkflowError::validation("patterns array required"))?;

        let config = AutomatonConfig {
            case_sensitive: false,
            find_overlapping: true,
            max_patterns: 20000,
            max_pattern_length: 1000,
        };

        Ok(json!({
            "config": config,
            "patterns": patterns
        }))
    }
}

// --- 4. Source Directory Scanning ---

pub struct DirectoryScannerTool;

#[async_trait]
impl ToolNode for DirectoryScannerTool {
    fn name(&self) -> &str {
        "directory-scanner"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("directory").is_none() {
            return Err(WorkflowError::validation("directory parameter required"));
        }
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("directory-scanner", "Scans directory for folders")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let path_str = params
            .get("directory")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkflowError::validation("directory parameter required"))?;
        let path = Path::new(path_str);

        if !path.exists() || !path.is_dir() {
            return Err(WorkflowError::validation("Invalid directory path"));
        }

        let mut folders = Vec::new();
        for entry in
            std::fs::read_dir(path).map_err(|e| WorkflowError::tool(format!("IO error: {}", e)))?
        {
            let entry = entry.map_err(|e| WorkflowError::tool(format!("IO error: {}", e)))?;
            let ty = entry
                .file_type()
                .map_err(|e| WorkflowError::tool(format!("IO error: {}", e)))?;
            if ty.is_dir() {
                folders.push(entry.path().to_string_lossy().to_string());
            }
        }

        Ok(json!({ "folders": folders }))
    }
}

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

#[async_trait]
impl ToolNode for FolderNamePreprocessorTool {
    fn name(&self) -> &str {
        "folder-name-preprocessor"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("folders").is_none() {
            return Err(WorkflowError::validation("folders parameter required"));
        }
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("folder-name-preprocessor", "Preprocesses folder names")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let folders = params
            .get("folders")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WorkflowError::validation("folders array required"))?;

        let processed: Vec<Value> = folders
            .iter()
            .filter_map(|v| v.as_str())
            .map(|path| {
                let path_obj = Path::new(path);
                let name = path_obj
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let normalized = self.text_processor.normalize_text(&name);

                json!({
                    "original_path": path,
                    "original_name": name,
                    "processed_name": normalized
                })
            })
            .collect();

        Ok(json!({ "processed_folders": processed }))
    }
}

// --- 6. Parallel Matching Execution ---

pub struct ParallelMatcherTool;

#[async_trait]
impl ToolNode for ParallelMatcherTool {
    fn name(&self) -> &str {
        "parallel-matcher"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("automaton_config").is_none() {
            return Err(WorkflowError::validation("automaton_config required"));
        }
        if params.get("folders").is_none() {
            return Err(WorkflowError::validation("folders required"));
        }
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("parallel-matcher", "Executes parallel AC matching")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let config_val = params
            .get("automaton_config")
            .ok_or_else(|| WorkflowError::validation("automaton_config required"))?;
        let patterns_val = config_val
            .get("patterns")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WorkflowError::validation("patterns required"))?;
        let config_obj: AutomatonConfig =
            serde_json::from_value(config_val.get("config").unwrap_or(&json!({})).clone())
                .unwrap_or_default();

        let folders = params
            .get("folders")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WorkflowError::validation("folders required"))?;

        // Rebuild automaton
        let mut matcher = AhoCorasickMatcher::with_config(config_obj);
        for p in patterns_val {
            let pattern = p.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let category = p
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let score = p.get("score").and_then(|v| v.as_f64()).unwrap_or(1.0);
            if !pattern.is_empty() {
                let _ = matcher.add_pattern(pattern, category, score);
            }
        }
        matcher
            .build()
            .map_err(|e| WorkflowError::tool(e.to_string()))?;
        let matcher = Arc::new(matcher);

        let mut results = Vec::new();
        for folder in folders {
            let processed_name = folder
                .get("processed_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let original_path = folder
                .get("original_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let matches = matcher
                .find_overlapping_matches(processed_name)
                .unwrap_or_default();

            results.push(json!({
                "folder_path": original_path,
                "matches": matches
            }));
        }

        Ok(json!({ "match_results": results }))
    }
}

// --- 7. Score Calculation ---

pub struct ScoreCalculatorTool;

#[async_trait]
impl ToolNode for ScoreCalculatorTool {
    fn name(&self) -> &str {
        "score-calculator"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("match_results").is_none() {
            return Err(WorkflowError::validation("match_results required"));
        }
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("score-calculator", "Calculates classification scores")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let match_results = params
            .get("match_results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WorkflowError::validation("match_results required"))?;

        let mut scored_results = Vec::new();

        for res in match_results {
            let folder_path = res
                .get("folder_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let matches = res.get("matches").and_then(|v| v.as_array()).unwrap();

            let mut category_scores: HashMap<String, f64> = HashMap::new();

            for m in matches {
                let category = m
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let score = m.get("score").and_then(|v| v.as_f64()).unwrap_or(1.0);

                *category_scores.entry(category).or_insert(0.0) += score;
            }

            let mut candidates: Vec<Value> = category_scores
                .into_iter()
                .map(|(cat, score)| json!({ "category": cat, "score": score }))
                .collect();

            candidates.sort_by(|a, b| {
                let s_a = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let s_b = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                s_b.partial_cmp(&s_a).unwrap_or(std::cmp::Ordering::Equal)
            });

            scored_results.push(json!({
                "folder_path": folder_path,
                "candidates": candidates
            }));
        }

        Ok(json!({ "scored_results": scored_results }))
    }
}

// --- 8. Ambiguity Detection ---

pub struct AmbiguityDetectorTool;

#[async_trait]
impl ToolNode for AmbiguityDetectorTool {
    fn name(&self) -> &str {
        "ambiguity-detector"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        if params.get("scored_results").is_none() {
            return Err(WorkflowError::validation("scored_results required"));
        }
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("ambiguity-detector", "Detects classification ambiguity")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let scored_results = params
            .get("scored_results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WorkflowError::validation("scored_results required"))?;
        let threshold = params
            .get("confidence_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.8);

        let mut classified = Vec::new();
        let mut ambiguous = Vec::new();
        let mut unclassified = Vec::new();

        for item in scored_results {
            let candidates = item.get("candidates").and_then(|v| v.as_array()).unwrap();

            let status = if candidates.is_empty() {
                "Unclassified"
            } else if candidates.len() == 1 {
                "Classified"
            } else {
                let top_score = candidates[0]
                    .get("score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let second_score = candidates[1]
                    .get("score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                if second_score / top_score < threshold {
                    "Classified"
                } else {
                    "Ambiguous"
                }
            };

            let result_item = json!({
                "folder_path": item["folder_path"],
                "candidates": candidates,
                "status": status,
                "top_category": candidates.first().and_then(|c| c.get("category"))
            });

            match status {
                "Classified" => classified.push(result_item),
                "Ambiguous" => ambiguous.push(result_item),
                _ => unclassified.push(result_item),
            }
        }

        Ok(json!({
            "classified": classified,
            "ambiguous": ambiguous,
            "unclassified": unclassified
        }))
    }
}

// --- 9. Result Merging ---

pub struct ResultMergerTool;

#[async_trait]
impl ToolNode for ResultMergerTool {
    fn name(&self) -> &str {
        "result-merger"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("result-merger", "Merges classification results")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let classified = params
            .get("classified")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .clone();
        let manual_results = params
            .get("manual_results")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .clone();

        let mut final_operations = Vec::new();

        // Add auto-classified
        for item in classified {
            if let Some(category) = item.get("top_category").and_then(|v| v.as_str()) {
                final_operations.push(json!({
                    "source": item["folder_path"],
                    "destination": format!("{}/{}", category, Path::new(item["folder_path"].as_str().unwrap_or("")).file_name().unwrap_or_default().to_string_lossy()),
                    "destination_category": category,
                    "type": "Auto"
                }));
            }
        }

        // Add manual results
        for item in manual_results {
            if let Some(category) = item.get("selected_category").and_then(|v| v.as_str()) {
                final_operations.push(json!({
                    "source": item["folder_path"],
                    "destination": format!("{}/{}", category, Path::new(item["folder_path"].as_str().unwrap_or("")).file_name().unwrap_or_default().to_string_lossy()),
                    "destination_category": category,
                    "type": "Manual"
                }));
            }
        }

        Ok(json!({ "merge_operations": final_operations }))
    }
}

// --- 10. Experimental Mode Check ---

pub struct ExperimentalCheckTool;

#[async_trait]
impl ToolNode for ExperimentalCheckTool {
    fn name(&self) -> &str {
        "experimental-check"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("experimental-check", "Checks experimental mode status")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let mode = params
            .get("experimental_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        info!("Experimental Check: Mode is {}", mode);
        Ok(json!({ "is_experimental": mode }))
    }
}

// --- 13. Report Generation ---

pub struct ReportGeneratorTool;

#[async_trait]
impl ToolNode for ReportGeneratorTool {
    fn name(&self) -> &str {
        "report-generator"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        create_tool_info("report-generator", "Generates execution report")
    }

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let empty = Vec::new();
        let operations = params
            .get("operations")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty);
        let stats = json!({
            "total_processed": operations.len(),
            "timestamp": Utc::now().to_rfc3339(),
            "details": operations
        });

        Ok(json!({ "report": stats }))
    }
}
