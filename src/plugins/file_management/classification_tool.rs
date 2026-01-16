//! Classification Tool implementation
//!
//! This module provides intelligent folder classification using AC automaton
//! and text processing with scoring algorithms and decision making.

use super::ac_automaton::{AhoCorasickMatcher, AutomatonConfig, PatternMatch};
use super::error::{FileManagementError, FileManagementResult};
use super::human_decision_tool::HumanDecisionResult;
use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::tools::ToolNode;
// use super::rule_config::RuleConfigLoader;
use super::utils::{
    HumanDecisionContext, HumanDecisionType, TextNormalizationConfig, TextProcessor,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

fn default_score_weight() -> f64 {
    1.0
}

fn default_true() -> bool {
    true
}

fn default_min_confidence() -> f64 {
    0.1
}

fn default_ambiguity_threshold() -> f64 {
    0.8
}

/// Classification rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    pub category: String,
    pub keywords: Vec<String>,
    pub combinations: Option<Vec<Vec<String>>>,
    #[serde(default = "default_score_weight")]
    pub score_weight: f64,
    pub required_matches: Option<usize>,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default = "default_true")]
    pub use_pinyin: bool,
}

impl ClassificationRule {
    pub fn new<S: Into<String>>(category: S, keywords: Vec<String>) -> Self {
        Self {
            category: category.into(),
            keywords,
            combinations: None,
            score_weight: 1.0,
            required_matches: None,
            case_sensitive: false,
            use_pinyin: true,
        }
    }

    pub fn with_combinations(mut self, combinations: Vec<Vec<String>>) -> Self {
        self.combinations = Some(combinations);
        self
    }

    pub fn with_score_weight(mut self, weight: f64) -> Self {
        self.score_weight = weight;
        self
    }

    pub fn with_required_matches(mut self, required: usize) -> Self {
        self.required_matches = Some(required);
        self
    }

    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    pub fn use_pinyin(mut self, use_pinyin: bool) -> Self {
        self.use_pinyin = use_pinyin;
        self
    }
}

/// Classification rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRules {
    pub rules: Vec<ClassificationRule>,
    pub default_category: Option<String>,
    #[serde(default = "default_min_confidence")]
    pub min_confidence_threshold: f64,
    #[serde(default = "default_ambiguity_threshold")]
    pub ambiguity_threshold: f64,
}

impl Default for ClassificationRules {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            default_category: Some("未分类".to_string()),
            min_confidence_threshold: 0.1,
            ambiguity_threshold: 0.8, // If top two scores are within 80% of each other, it's ambiguous
        }
    }
}

/// Classification candidate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationCandidate {
    pub category: String,
    pub score: f64,
    pub confidence: f64,
    pub matched_keywords: Vec<String>,
    pub match_details: Vec<PatternMatch>,
}

impl ClassificationCandidate {
    pub fn new(category: String, score: f64, confidence: f64) -> Self {
        Self {
            category,
            score,
            confidence,
            matched_keywords: Vec::new(),
            match_details: Vec::new(),
        }
    }

    pub fn with_matches(mut self, matches: Vec<PatternMatch>) -> Self {
        self.matched_keywords = matches.iter().map(|m| m.pattern.clone()).collect();
        self.match_details = matches;
        self
    }
}

/// Classification engine core
pub struct ClassificationEngine {
    text_processor: TextProcessor,
    #[allow(dead_code)]
    normalization_config: TextNormalizationConfig,
    enable_chinese: bool,
}

impl ClassificationEngine {
    /// Create a new classification engine
    pub fn new(enable_chinese: bool) -> Self {
        let normalization_config = TextNormalizationConfig::default();
        let text_processor =
            TextProcessor::with_config(enable_chinese, normalization_config.clone());

        Self {
            text_processor,
            normalization_config,
            enable_chinese,
        }
    }

    /// Create a new classification engine with custom config
    pub fn with_config(enable_chinese: bool, config: TextNormalizationConfig) -> Self {
        let text_processor = TextProcessor::with_config(enable_chinese, config.clone());

        Self {
            text_processor,
            normalization_config: config,
            enable_chinese,
        }
    }

    /// Build AC automaton from classification rules
    pub fn build_automaton(
        &self,
        rules: &ClassificationRules,
    ) -> FileManagementResult<AhoCorasickMatcher> {
        let config = AutomatonConfig {
            case_sensitive: false, // We'll handle case sensitivity in preprocessing
            find_overlapping: true,
            max_patterns: 20_000,
            max_pattern_length: 1000,
        };

        let mut automaton = AhoCorasickMatcher::with_config(config);
        let mut seen_keywords = std::collections::HashSet::new();

        // Helper to add a keyword to the automaton
        let mut add_keyword_to_automaton = |keyword: &str, rule: &ClassificationRule, _weight_multiplier: f64| -> FileManagementResult<()> {
            let processed_keyword = self.preprocess_keyword(keyword, rule);
            
            if !seen_keywords.contains(&processed_keyword) {
                // Use "keyword" as generic category since we map back to rules later
                automaton
                    .add_pattern(&processed_keyword, "keyword", 1.0)
                    .map_err(|e| {
                        FileManagementError::classification(format!(
                            "Failed to add pattern '{}': {}",
                            processed_keyword, e
                        ))
                    })?;
                seen_keywords.insert(processed_keyword);
            }
            Ok(())
        };

        for rule in &rules.rules {
            // Add original keywords
            for keyword in &rule.keywords {
                add_keyword_to_automaton(keyword, rule, 1.0)?;
            }

            // Add combination keywords
            if let Some(combinations) = &rule.combinations {
                for combo in combinations {
                    for keyword in combo {
                        add_keyword_to_automaton(keyword, rule, 1.0)?;
                    }
                }
            }

            // Add pinyin variants if enabled
            if rule.use_pinyin && self.enable_chinese {
                let mut keywords_to_process = rule.keywords.clone();
                if let Some(combinations) = &rule.combinations {
                    for combo in combinations {
                        keywords_to_process.extend(combo.clone());
                    }
                }

                for keyword in keywords_to_process {
                    let pinyin_variants = self.text_processor.generate_pinyin_variants(&keyword);
                    for variant in pinyin_variants {
                        if variant != keyword {
                            add_keyword_to_automaton(&variant, rule, 0.8)?;
                        }
                    }
                }
            }
        }

        automaton.build().map_err(|e| {
            FileManagementError::classification(format!("Failed to build automaton: {}", e))
        })?;

        debug!(
            "Built classification automaton with {} patterns",
            automaton.pattern_count()
        );
        Ok(automaton)
    }

    /// Preprocess keyword according to rule settings
    fn preprocess_keyword(&self, keyword: &str, rule: &ClassificationRule) -> String {
        let mut processed = keyword.to_string();

        if !rule.case_sensitive {
            processed = self.text_processor.normalize_case(&processed);
        }

        // Apply basic normalization
        processed = self.text_processor.normalize_whitespace(&processed);
        processed = self.text_processor.normalize_unicode(&processed);

        processed
    }

    /// Preprocess folder name for classification
    pub fn preprocess_folder_name(&self, folder_name: &str) -> String {
        // Apply comprehensive text normalization
        let mut processed = self.text_processor.normalize_text(folder_name);

        // Handle Chinese text if enabled
        if self.enable_chinese && self.text_processor.contains_chinese(&processed) {
            let mixed_result = self.text_processor.process_mixed_text(&processed);

            // Create a combined text with both original and processed Chinese
            let mut combined = processed.clone();
            if !mixed_result.simplified_chinese.is_empty()
                && mixed_result.simplified_chinese != mixed_result.chinese_chars
            {
                combined.push(' ');
                combined.push_str(&mixed_result.simplified_chinese);
            }

            processed = combined;
        }

        debug!(
            "Preprocessed folder name: '{}' -> '{}'",
            folder_name, processed
        );
        processed
    }

    /// Classify a folder name using the automaton
    pub fn classify_folder(
        &self,
        folder_name: &str,
        automaton: &AhoCorasickMatcher,
        rules: &ClassificationRules,
    ) -> FileManagementResult<ClassificationResult> {
        let start_time = std::time::Instant::now();

        // Preprocess the folder name
        let processed_name = self.preprocess_folder_name(folder_name);

        // Find all matches
        let matches = automaton.find_matches(&processed_name).map_err(|e| {
            FileManagementError::classification(format!("Pattern matching failed: {}", e))
        })?;

        // Calculate scores by category
        let candidates = self.calculate_category_scores(&matches, rules)?;

        // Determine classification result
        let result = self.determine_classification_result(
            folder_name,
            candidates,
            rules,
            start_time.elapsed().as_millis() as u64,
        )?;

        debug!(
            "Classified '{}' as {:?} in {}ms",
            folder_name, result.status, result.processing_time_ms
        );

        Ok(result)
    }

    /// Calculate scores for each category based on matches
    fn calculate_category_scores(
        &self,
        matches: &[PatternMatch],
        rules: &ClassificationRules,
    ) -> FileManagementResult<Vec<ClassificationCandidate>> {
        // Create a map of matched strings to their details for fast lookup
        let mut matches_by_keyword: std::collections::HashMap<String, Vec<PatternMatch>> = std::collections::HashMap::new();
        for m in matches {
            matches_by_keyword.entry(m.pattern.clone())
                .or_default()
                .push(m.clone());
        }
            
        let mut candidates = Vec::new();
        let mut total_score = 0.0;

        for rule in &rules.rules {
            let mut rule_score = 0.0;
            let mut rule_matched_keywords = Vec::new();
            let mut rule_match_details = Vec::new();
            
            // Check simple keywords
            for keyword in &rule.keywords {
                let processed = self.preprocess_keyword(keyword, rule);
                if let Some(details) = matches_by_keyword.get(&processed) {
                    rule_score += rule.score_weight;
                    rule_matched_keywords.push(keyword.clone());
                    rule_match_details.extend(details.clone());
                } else if rule.use_pinyin && self.enable_chinese {
                     // Check pinyin variants
                     let pinyin_variants = self.text_processor.generate_pinyin_variants(keyword);
                     for variant in pinyin_variants {
                         let processed_variant = self.preprocess_keyword(&variant, rule);
                         if let Some(details) = matches_by_keyword.get(&processed_variant) {
                             rule_score += rule.score_weight; 
                             rule_matched_keywords.push(format!("{} (pinyin)", keyword));
                             rule_match_details.extend(details.clone());
                             break; 
                         }
                     }
                }
            }
            
            // Check combinations
            if let Some(combinations) = &rule.combinations {
                for combo in combinations {
                    let mut all_match = true;
                    let mut combo_matches = Vec::new();
                    let mut combo_details = Vec::new();
                    
                    for k in combo {
                        let processed = self.preprocess_keyword(k, rule);
                        let mut k_matched = false;
                        
                        if let Some(details) = matches_by_keyword.get(&processed) {
                            k_matched = true;
                            combo_details.extend(details.clone());
                        } else if rule.use_pinyin && self.enable_chinese {
                             let pinyin_variants = self.text_processor.generate_pinyin_variants(k);
                             for variant in pinyin_variants {
                                 let processed_variant = self.preprocess_keyword(&variant, rule);
                                 if let Some(details) = matches_by_keyword.get(&processed_variant) {
                                     k_matched = true;
                                     combo_details.extend(details.clone());
                                     break;
                                 }
                             }
                        }
                        
                        if k_matched {
                            combo_matches.push(k.clone());
                        } else {
                            all_match = false;
                            break; 
                        }
                    }
                    
                    if all_match {
                        rule_score += rule.score_weight;
                        rule_matched_keywords.extend(combo_matches);
                        rule_match_details.extend(combo_details);
                    }
                }
            }
            
            if rule_score > 0.0 {
                // Check required matches
                if let Some(required) = rule.required_matches {
                    if rule_matched_keywords.len() < required {
                        continue;
                    }
                }
                
                total_score += rule_score;
                
                let mut candidate = ClassificationCandidate::new(
                    rule.category.clone(),
                    rule_score,
                    0.0
                );
                candidate.matched_keywords = rule_matched_keywords;
                candidate.match_details = rule_match_details;
                candidates.push(candidate);
            }
        }
        
        // Calculate confidence
        for candidate in &mut candidates {
            if total_score > 0.0 {
                candidate.confidence = candidate.score / total_score;
            }
        }

        // Sort by score (descending)
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(candidates)
    }

    /// Determine the final classification result
    fn determine_classification_result(
        &self,
        folder_name: &str,
        mut candidates: Vec<ClassificationCandidate>,
        rules: &ClassificationRules,
        processing_time_ms: u64,
    ) -> FileManagementResult<ClassificationResult> {
        let status = if candidates.is_empty() {
            ClassificationStatus::Unclassified
        } else if candidates.len() == 1 {
            let candidate = &candidates[0];
            if candidate.confidence >= rules.min_confidence_threshold {
                ClassificationStatus::Classified
            } else {
                ClassificationStatus::Unclassified
            }
        } else {
            // Check for ambiguity
            let top_score = candidates[0].score;
            let second_score = candidates[1].score;

            if second_score / top_score >= rules.ambiguity_threshold {
                ClassificationStatus::Ambiguous
            } else if candidates[0].confidence >= rules.min_confidence_threshold {
                ClassificationStatus::Classified
            } else {
                ClassificationStatus::Unclassified
            }
        };

        let (category, score) = match status {
            ClassificationStatus::Classified => {
                (Some(candidates[0].category.clone()), candidates[0].score)
            }
            ClassificationStatus::Unclassified => (rules.default_category.clone(), 0.0),
            ClassificationStatus::Ambiguous => {
                // Keep top candidate but mark as ambiguous
                (Some(candidates[0].category.clone()), candidates[0].score)
            }
            ClassificationStatus::Error => (None, 0.0),
            ClassificationStatus::Pending => {
                (Some(candidates[0].category.clone()), candidates[0].score)
            }
        };

        // Limit candidates to top 5 for readability
        candidates.truncate(5);

        Ok(ClassificationResult {
            status,
            category,
            candidates,
            score,
            folder_name: folder_name.to_string(),
            processing_time_ms,
            metadata: HashMap::new(),
        })
    }

    /// Check if classification result needs human decision
    pub fn needs_human_decision(&self, result: &ClassificationResult) -> bool {
        matches!(result.status, ClassificationStatus::Ambiguous)
            || (matches!(result.status, ClassificationStatus::Unclassified)
                && result.candidates.len() > 1)
    }

    /// Create human decision context for ambiguous classification
    pub fn create_human_decision_context(
        &self,
        result: &ClassificationResult,
    ) -> HumanDecisionContext {
        let title = format!("Folder Classification: '{}'", result.folder_name);
        let description = match result.status {
            ClassificationStatus::Ambiguous => {
                format!("Multiple categories have similar scores for folder '{}'. Please choose the most appropriate category.", result.folder_name)
            }
            ClassificationStatus::Unclassified => {
                format!("No clear category found for folder '{}'. Please select a category or create a new one.", result.folder_name)
            }
            _ => {
                format!(
                    "Please confirm the classification for folder '{}'.",
                    result.folder_name
                )
            }
        };

        let mut context =
            HumanDecisionContext::new(HumanDecisionType::Classification, title, description);

        // Add options for each candidate
        for (_i, candidate) in result.candidates.iter().enumerate() {
            let option_description = format!(
                "Score: {:.2}, Confidence: {:.1}%, Keywords: {}",
                candidate.score,
                candidate.confidence * 100.0,
                candidate.matched_keywords.join(", ")
            );

            context = context.add_option(
                &candidate.category,
                &candidate.category,
                Some(&option_description),
            );
        }

        // Add "Other" option
        context = context.add_option(
            "other",
            "Other (specify custom category)",
            Some("Choose this to specify a different category"),
        );

        // Add "Skip" option
        context = context.add_option(
            "skip",
            "Skip classification",
            Some("Leave this folder unclassified for now"),
        );

        context.with_timeout(300) // 5 minutes timeout
    }
}

/// Classification status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClassificationStatus {
    Classified,
    Unclassified,
    Ambiguous,
    Error,
    Pending,
}

/// Classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub status: ClassificationStatus,
    pub category: Option<String>,
    pub candidates: Vec<ClassificationCandidate>,
    pub score: f64,
    pub folder_name: String,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, Value>,
}

/// Parameters for classification tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationParams {
    pub folder_path: String,
    pub classification_rules: Value, // JSON rules or file path
    pub enable_user_interaction: bool,
    pub experimental_mode: bool,
    pub output_format: Option<ClassificationOutputFormat>,
}

/// Output format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassificationOutputFormat {
    Simple,   // Just category and confidence
    Detailed, // Include candidates and metadata
    Full,     // Complete result with all details
}

impl Default for ClassificationOutputFormat {
    fn default() -> Self {
        ClassificationOutputFormat::Detailed
    }
}

/// Classification Tool implementation
pub struct ClassificationTool {
    engine: ClassificationEngine,
    plugin_info: Option<PluginInfo>,
}

impl ClassificationTool {
    /// Create a new classification tool
    pub fn new(enable_chinese: bool) -> Self {
        let engine = ClassificationEngine::new(enable_chinese);

        Self {
            engine,
            plugin_info: None,
        }
    }

    /// Create a new classification tool with plugin info
    pub fn with_plugin_info(enable_chinese: bool, plugin_info: PluginInfo) -> Self {
        let engine = ClassificationEngine::new(enable_chinese);

        Self {
            engine,
            plugin_info: Some(plugin_info),
        }
    }

    /// Load classification rules from JSON value or file path
    fn load_classification_rules(
        &self,
        rules_value: &Value,
    ) -> FileManagementResult<ClassificationRules> {
        let mut value_to_parse = if let Some(file_path) = rules_value.as_str() {
            // Load from file
            let content = std::fs::read_to_string(file_path).map_err(|e| {
                FileManagementError::io(format!("Failed to read rules file: {}", file_path), e)
            })?;

            serde_json::from_str(&content).map_err(|e| {
                FileManagementError::validation(format!("Invalid JSON in rules file: {}", e))
            })?
        } else {
            rules_value.clone()
        };

        // Apply transformations (legacy fields and nested keywords)
        if let Some(obj) = value_to_parse.as_object_mut() {
            // Legacy "categories" -> "rules"
            if obj.contains_key("categories") && !obj.contains_key("rules") {
                if let Some(categories) = obj.remove("categories") {
                    obj.insert("rules".to_string(), categories);
                }
            }

            // Transform nested keywords into combinations
            if let Some(rules) = obj.get_mut("rules").and_then(|r| r.as_array_mut()) {
                for rule in rules {
                    if let Some(rule_obj) = rule.as_object_mut() {
                        // Legacy "name" -> "category"
                        if rule_obj.contains_key("name") && !rule_obj.contains_key("category") {
                            if let Some(name) = rule_obj.remove("name") {
                                rule_obj.insert("category".to_string(), name);
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
                                    rule_obj.insert("combinations".to_string(), json!(combinations));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse as ClassificationRules
        serde_json::from_value(value_to_parse).map_err(|e| {
            FileManagementError::validation(format!("Invalid classification rules: {}", e))
        })
    }

    /// Invoke human decision tool for ambiguous classification
    async fn invoke_human_decision(
        &self,
        decision_params: Value,
        _context: &ExecutionContext,
    ) -> Result<Value> {
        // For now, use the fallback implementation since we don't have direct access to the tool registry
        // In a full implementation, this would use the workflow engine to invoke the human decision tool
        warn!(
            "Using fallback human decision implementation - full tool registry integration needed"
        );
        self.fallback_human_decision(decision_params).await
    }

    /// Fallback human decision implementation when the tool is not available
    async fn fallback_human_decision(&self, decision_params: Value) -> Result<Value> {
        // Extract experimental mode flag
        let experimental_mode = decision_params
            .get("experimental_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if experimental_mode {
            // In experimental mode, auto-select the first recommended option or first option
            let options = decision_params
                .get("options")
                .and_then(|v| v.as_array())
                .ok_or_else(|| WorkflowError::tool("No options available for decision"))?;

            let selected_option = options
                .iter()
                .find(|opt| {
                    opt.get("recommended")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                })
                .or_else(|| options.first())
                .and_then(|opt| opt.get("id").and_then(|v| v.as_str()))
                .ok_or_else(|| WorkflowError::tool("No valid option found"))?;

            info!(
                "Experimental mode: auto-selected option '{}'",
                selected_option
            );

            return Ok(json!({
                "selected_option": selected_option,
                "decision_time_ms": 0,
                "was_timeout": false,
                "user_input": "auto-selected in experimental mode",
                "experimental_mode": true
            }));
        }

        // In non-experimental mode, we can't make a decision without user interaction
        Err(WorkflowError::tool(
            "Human decision tool not available and not in experimental mode",
        ))
    }

    /// Apply human decision to classification result
    fn apply_human_decision(
        &self,
        original_result: &ClassificationResult,
        human_decision: &HumanDecisionResult,
    ) -> FileManagementResult<ClassificationResult> {
        let mut updated_result = original_result.clone();

        match human_decision.selected_option.as_str() {
            "skip" => {
                // User chose to skip classification
                updated_result.status = ClassificationStatus::Unclassified;
                updated_result.category = None;
                updated_result.score = 0.0;
                info!(
                    "User chose to skip classification for '{}'",
                    updated_result.folder_name
                );
            }
            "other" => {
                // User chose "other" - this would typically require additional input
                // For now, we'll mark it as unclassified and let the user handle it
                updated_result.status = ClassificationStatus::Unclassified;
                updated_result.category = Some("other".to_string());
                updated_result.score = 0.0;
                info!(
                    "User chose 'other' category for '{}'",
                    updated_result.folder_name
                );
            }
            selected_category => {
                // User selected a specific category
                if let Some(candidate) = updated_result
                    .candidates
                    .iter()
                    .find(|c| c.category == selected_category)
                {
                    // Update with the selected candidate
                    updated_result.status = ClassificationStatus::Classified;
                    updated_result.category = Some(candidate.category.clone());
                    updated_result.score = candidate.score;
                    info!(
                        "User selected category '{}' for '{}'",
                        selected_category, updated_result.folder_name
                    );
                } else {
                    // User selected a category not in the candidates (custom category)
                    updated_result.status = ClassificationStatus::Classified;
                    updated_result.category = Some(selected_category.to_string());
                    updated_result.score = 1.0; // Give it a default score
                    info!(
                        "User selected custom category '{}' for '{}'",
                        selected_category, updated_result.folder_name
                    );
                }
            }
        }

        Ok(updated_result)
    }

    /// Format result based on output format
    fn format_result(
        &self,
        result: ClassificationResult,
        format: &ClassificationOutputFormat,
    ) -> Value {
        match format {
            ClassificationOutputFormat::Simple => {
                json!({
                    "category": result.category,
                    "confidence": if result.candidates.is_empty() { 0.0 } else { result.candidates[0].confidence },
                    "status": result.status
                })
            }
            ClassificationOutputFormat::Detailed => {
                json!({
                    "status": result.status,
                    "category": result.category,
                    "score": result.score,
                    "candidates": result.candidates.iter().take(3).collect::<Vec<_>>(),
                    "folder_name": result.folder_name,
                    "processing_time_ms": result.processing_time_ms
                })
            }
            ClassificationOutputFormat::Full => {
                serde_json::to_value(result).unwrap_or_else(|_| json!({}))
            }
        }
    }
}

#[async_trait]
impl ToolNode for ClassificationTool {
    fn name(&self) -> &str {
        "folder-classifier"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        info!("Executing folder classification tool");

        // Parse parameters
        let params: ClassificationParams = serde_json::from_value(params)
            .map_err(|e| WorkflowError::ValidationError(format!("Invalid parameters: {}", e)))?;

        // Validate folder path
        let folder_path = Path::new(&params.folder_path);
        if !folder_path.exists() {
            return Err(WorkflowError::ValidationError(format!(
                "Folder path does not exist: {}",
                params.folder_path
            )));
        }

        let folder_name = folder_path
            .file_name()
            .ok_or_else(|| WorkflowError::ValidationError("Invalid folder path".to_string()))?
            .to_string_lossy()
            .to_string();

        // Load classification rules
        let rules = self
            .load_classification_rules(&params.classification_rules)
            .map_err(|e| WorkflowError::tool(format!("Failed to load rules: {}", e)))?;

        // Build automaton
        let automaton = self
            .engine
            .build_automaton(&rules)
            .map_err(|e| WorkflowError::tool(format!("Failed to build automaton: {}", e)))?;

        // Classify folder
        let mut result = self
            .engine
            .classify_folder(&folder_name, &automaton, &rules)
            .map_err(|e| WorkflowError::tool(format!("Classification failed: {}", e)))?;

        // Handle experimental mode
        if params.experimental_mode {
            result
                .metadata
                .insert("experimental_mode".to_string(), Value::Bool(true));
            info!("Classification completed in experimental mode");
        }

        // Handle human interaction if needed
        if params.enable_user_interaction && self.engine.needs_human_decision(&result) {
            info!("Ambiguous classification detected, invoking human decision");

            let decision_context = self.engine.create_human_decision_context(&result);

            // Create human decision parameters
            let human_decision_params = json!({
                "decision_type": "Classification",
                "context": {
                    "title": decision_context.title,
                    "description": decision_context.description,
                    "folder_name": result.folder_name,
                    "metadata": decision_context.metadata
                },
                "options": decision_context.options.iter().map(|opt| json!({
                    "id": opt.id,
                    "label": opt.label,
                    "description": opt.description,
                    "recommended": opt.recommended,
                    "score": opt.metadata.get("score")
                })).collect::<Vec<_>>(),
                "timeout_seconds": decision_context.timeout_seconds,
                "default_choice": decision_context.options.iter().position(|opt| opt.recommended),
                "experimental_mode": params.experimental_mode
            });

            // Invoke human decision tool
            match self
                .invoke_human_decision(human_decision_params, &context)
                .await
            {
                Ok(decision_result) => {
                    // Parse the human decision result
                    if let Ok(human_result) =
                        serde_json::from_value::<HumanDecisionResult>(decision_result)
                    {
                        // Update classification result based on human decision
                        result = self.apply_human_decision(&result, &human_result)?;

                        result
                            .metadata
                            .insert("human_decision_made".to_string(), Value::Bool(true));
                        result.metadata.insert(
                            "decision_time_ms".to_string(),
                            Value::Number(serde_json::Number::from(human_result.decision_time_ms)),
                        );
                        result.metadata.insert(
                            "selected_option".to_string(),
                            Value::String(human_result.selected_option),
                        );
                    } else {
                        warn!("Failed to parse human decision result");
                        result.metadata.insert(
                            "human_decision_error".to_string(),
                            Value::String("Failed to parse decision result".to_string()),
                        );
                        result.status = ClassificationStatus::Error;
                    }
                }
                Err(e) => {
                    warn!("Human decision failed: {}", e);
                    result.metadata.insert(
                        "human_decision_error".to_string(),
                        Value::String(e.to_string()),
                    );
                    result.status = ClassificationStatus::Pending;
                }
            }
        }

        // Format result
        let output_format = params.output_format.unwrap_or_default();
        let formatted_result = self.format_result(result, &output_format);

        info!("Folder classification completed successfully");
        Ok(formatted_result)
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        let parsed_params: ClassificationParams =
            serde_json::from_value(params.clone()).map_err(|e| {
                WorkflowError::ValidationError(format!("Parameter validation failed: {}", e))
            })?;

        // Validate folder path
        if parsed_params.folder_path.trim().is_empty() {
            return Err(WorkflowError::ValidationError(
                "Folder path cannot be empty".to_string(),
            ));
        }

        // Validate classification rules
        if parsed_params.classification_rules.is_null() {
            return Err(WorkflowError::ValidationError(
                "Classification rules are required".to_string(),
            ));
        }

        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        let now = Utc::now();
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description:
                "Intelligent folder classification using configurable rules and AC automaton"
                    .to_string(),
            category: Some("classification".to_string()),
            tags: vec![
                "classification".to_string(),
                "folders".to_string(),
                "automation".to_string(),
                "chinese".to_string(),
                "pinyin".to_string(),
            ],
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "folder_path": {
                        "type": "string",
                        "description": "Path to the folder to classify",
                        "minLength": 1
                    },
                    "classification_rules": {
                        "description": "Classification rules (JSON object or file path to JSON file)",
                        "oneOf": [
                            {"type": "string", "description": "Path to JSON rules file"},
                            {
                                "type": "object",
                                "properties": {
                                    "rules": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "properties": {
                                                "category": {"type": "string"},
                                                "keywords": {"type": "array", "items": {"type": "string"}},
                                                "score_weight": {"type": "number", "default": 1.0},
                                                "required_matches": {"type": "integer", "minimum": 1},
                                                "case_sensitive": {"type": "boolean", "default": false},
                                                "use_pinyin": {"type": "boolean", "default": true}
                                            },
                                            "required": ["category", "keywords"]
                                        }
                                    },
                                    "default_category": {"type": "string"},
                                    "min_confidence_threshold": {"type": "number", "default": 0.1},
                                    "ambiguity_threshold": {"type": "number", "default": 0.8}
                                },
                                "required": ["rules"]
                            }
                        ]
                    },
                    "enable_user_interaction": {
                        "type": "boolean",
                        "default": false,
                        "description": "Enable human decision for ambiguous cases"
                    },
                    "experimental_mode": {
                        "type": "boolean",
                        "default": false,
                        "description": "Run in experimental mode (simulation only)"
                    },
                    "output_format": {
                        "type": "string",
                        "enum": ["Simple", "Detailed", "Full"],
                        "default": "Detailed",
                        "description": "Output format detail level"
                    }
                },
                "required": ["folder_path", "classification_rules"]
            }),
            return_schema: json!({
                "type": "object",
                "description": "Classification result (format depends on output_format parameter)",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["Classified", "Unclassified", "Ambiguous", "Error", "Pending"]
                    },
                    "category": {"type": "string"},
                    "score": {"type": "number"},
                    "candidates": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "category": {"type": "string"},
                                "score": {"type": "number"},
                                "confidence": {"type": "number"},
                                "matched_keywords": {"type": "array", "items": {"type": "string"}}
                            }
                        }
                    },
                    "folder_name": {"type": "string"},
                    "processing_time_ms": {"type": "number"},
                    "metadata": {"type": "object"}
                }
            }),
            plugin_name: self.plugin_info.as_ref().map(|p| p.name.clone()),
            dependencies: vec![],
            version_requirements: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    fn get_plugin_info(&self) -> Option<&PluginInfo> {
        self.plugin_info.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_classification_rule_creation() {
        let rule = ClassificationRule::new("documents", vec!["doc".to_string(), "pdf".to_string()])
            .with_score_weight(1.5)
            .with_required_matches(1)
            .case_sensitive(false)
            .use_pinyin(true);

        assert_eq!(rule.category, "documents");
        assert_eq!(rule.keywords.len(), 2);
        assert_eq!(rule.score_weight, 1.5);
        assert_eq!(rule.required_matches, Some(1));
        assert!(!rule.case_sensitive);
        assert!(rule.use_pinyin);
    }

    #[test]
    fn test_classification_engine_creation() {
        let engine = ClassificationEngine::new(true);
        assert!(engine.enable_chinese);

        let config = TextNormalizationConfig {
            normalize_case: false,
            ..Default::default()
        };
        let engine_with_config = ClassificationEngine::with_config(false, config);
        assert!(!engine_with_config.enable_chinese);
    }

    #[test]
    fn test_folder_name_preprocessing() {
        let engine = ClassificationEngine::new(true);

        let processed = engine.preprocess_folder_name("  Hello World  ");
        assert_eq!(processed, "hello world");

        let processed_chinese = engine.preprocess_folder_name("文档 Documents");
        assert!(processed_chinese.contains("文档"));
        assert!(processed_chinese.contains("documents"));
    }

    #[test]
    fn test_automaton_building() {
        let engine = ClassificationEngine::new(true);

        let rules = ClassificationRules {
            rules: vec![
                ClassificationRule::new("documents", vec!["doc".to_string(), "pdf".to_string()]),
                ClassificationRule::new("images", vec!["jpg".to_string(), "png".to_string()]),
            ],
            ..Default::default()
        };

        let automaton = engine.build_automaton(&rules).unwrap();
        assert!(automaton.pattern_count() >= 4); // At least the 4 keywords
        assert!(automaton.is_built());
    }

    #[test]
    fn test_classification_candidates() {
        let candidate = ClassificationCandidate::new("test".to_string(), 1.5, 0.8);
        assert_eq!(candidate.category, "test");
        assert_eq!(candidate.score, 1.5);
        assert_eq!(candidate.confidence, 0.8);
        assert!(candidate.matched_keywords.is_empty());
    }

    #[tokio::test]
    async fn test_classification_tool_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_folder = temp_dir.path().join("test_documents");
        std::fs::create_dir(&test_folder).unwrap();

        let tool = ClassificationTool::new(true);

        let rules = json!({
            "rules": [
                {
                    "category": "documents",
                    "keywords": ["doc", "document", "文档"],
                    "score_weight": 1.0,
                    "case_sensitive": false,
                    "use_pinyin": true
                }
            ],
            "default_category": "other",
            "min_confidence_threshold": 0.1,
            "ambiguity_threshold": 0.8
        });

        let params = json!({
            "folder_path": test_folder.to_string_lossy(),
            "classification_rules": rules,
            "enable_user_interaction": false,
            "experimental_mode": false,
            "output_format": "Detailed"
        });

        let context = ExecutionContext::new();
        let result = tool.execute(params, context).await.unwrap();

        assert!(result.get("status").is_some());
        assert!(result.get("folder_name").is_some());
    }

    #[test]
    fn test_parameter_validation() {
        let tool = ClassificationTool::new(true);

        // Valid parameters
        let valid_params = json!({
            "folder_path": "/some/path",
            "classification_rules": {
                "rules": [
                    {
                        "category": "documents",
                        "keywords": ["doc", "pdf"]
                    }
                ]
            },
            "enable_user_interaction": false,
            "experimental_mode": false
        });

        assert!(tool.validate_parameters(&valid_params).is_ok());

        // Invalid parameters - empty folder path
        let invalid_params = json!({
            "folder_path": "",
            "classification_rules": {
                "rules": [
                    {
                        "category": "documents",
                        "keywords": ["doc", "pdf"]
                    }
                ]
            },
            "enable_user_interaction": false,
            "experimental_mode": false
        });
        assert!(tool.validate_parameters(&invalid_params).is_err());

        // Invalid parameters - null rules
        let invalid_params = json!({
            "folder_path": "/some/path",
            "classification_rules": null,
            "enable_user_interaction": false,
            "experimental_mode": false
        });
        assert!(tool.validate_parameters(&invalid_params).is_err());
    }

    #[test]
    fn test_human_decision_context_creation() {
        let engine = ClassificationEngine::new(true);

        let result = ClassificationResult {
            status: ClassificationStatus::Ambiguous,
            category: Some("documents".to_string()),
            candidates: vec![
                ClassificationCandidate::new("documents".to_string(), 1.5, 0.6),
                ClassificationCandidate::new("images".to_string(), 1.2, 0.4),
            ],
            score: 1.5,
            folder_name: "test_folder".to_string(),
            processing_time_ms: 100,
            metadata: HashMap::new(),
        };

        let context = engine.create_human_decision_context(&result);
        assert_eq!(context.decision_type, HumanDecisionType::Classification);
        assert!(context.title.contains("test_folder"));
        assert_eq!(context.options.len(), 4); // 2 candidates + other + skip
    }

    #[tokio::test]
    async fn test_classification_with_human_decision_experimental_mode() {
        let temp_dir = TempDir::new().unwrap();
        let test_folder = temp_dir.path().join("ambiguous_folder");
        std::fs::create_dir(&test_folder).unwrap();

        let tool = ClassificationTool::new(true);

        // Create rules that will result in ambiguous classification
        let rules = json!({
            "rules": [
                {
                    "category": "documents",
                    "keywords": ["doc", "document"],
                    "score_weight": 1.0,
                    "case_sensitive": false,
                    "use_pinyin": false
                },
                {
                    "category": "media",
                    "keywords": ["media", "video"],
                    "score_weight": 0.9,
                    "case_sensitive": false,
                    "use_pinyin": false
                }
            ],
            "default_category": "other",
            "min_confidence_threshold": 0.1,
            "ambiguity_threshold": 0.9  // High threshold to trigger ambiguity
        });

        let params = json!({
            "folder_path": test_folder.to_string_lossy(),
            "classification_rules": rules,
            "enable_user_interaction": true,
            "experimental_mode": true,
            "output_format": "Full"
        });

        let context = ExecutionContext::new();
        let result = tool.execute(params, context).await.unwrap();

        // In experimental mode with human decision, it should auto-select
        assert!(result.get("status").is_some());
        assert!(result.get("metadata").is_some());

        let metadata = result.get("metadata").unwrap();
        if metadata
            .get("human_decision_made")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            assert!(metadata.get("selected_option").is_some());
        }
    }

    #[test]
    fn test_apply_human_decision() {
        let tool = ClassificationTool::new(true);

        let original_result = ClassificationResult {
            status: ClassificationStatus::Ambiguous,
            category: Some("documents".to_string()),
            candidates: vec![
                ClassificationCandidate::new("documents".to_string(), 1.5, 0.6),
                ClassificationCandidate::new("images".to_string(), 1.2, 0.4),
            ],
            score: 1.5,
            folder_name: "test_folder".to_string(),
            processing_time_ms: 100,
            metadata: HashMap::new(),
        };

        // Test selecting a candidate
        let human_decision = HumanDecisionResult {
            selected_option: "images".to_string(),
            decision_time_ms: 5000,
            was_timeout: false,
            user_input: Some("User selected images".to_string()),
            experimental_mode: false,
        };

        let updated_result = tool
            .apply_human_decision(&original_result, &human_decision)
            .unwrap();
        assert_eq!(updated_result.status, ClassificationStatus::Classified);
        assert_eq!(updated_result.category, Some("images".to_string()));
        assert_eq!(updated_result.score, 1.2);

        // Test skip option
        let skip_decision = HumanDecisionResult {
            selected_option: "skip".to_string(),
            decision_time_ms: 1000,
            was_timeout: false,
            user_input: Some("User chose to skip".to_string()),
            experimental_mode: false,
        };

        let skipped_result = tool
            .apply_human_decision(&original_result, &skip_decision)
            .unwrap();
        assert_eq!(skipped_result.status, ClassificationStatus::Unclassified);
        assert_eq!(skipped_result.category, None);
        assert_eq!(skipped_result.score, 0.0);

        // Test custom category
        let custom_decision = HumanDecisionResult {
            selected_option: "custom_category".to_string(),
            decision_time_ms: 3000,
            was_timeout: false,
            user_input: Some("User entered custom category".to_string()),
            experimental_mode: false,
        };

        let custom_result = tool
            .apply_human_decision(&original_result, &custom_decision)
            .unwrap();
        assert_eq!(custom_result.status, ClassificationStatus::Classified);
        assert_eq!(custom_result.category, Some("custom_category".to_string()));
        assert_eq!(custom_result.score, 1.0);
    }

    #[test]
    fn test_needs_human_decision() {
        let engine = ClassificationEngine::new(true);

        // Test ambiguous result
        let ambiguous_result = ClassificationResult {
            status: ClassificationStatus::Ambiguous,
            category: Some("documents".to_string()),
            candidates: vec![
                ClassificationCandidate::new("documents".to_string(), 1.5, 0.6),
                ClassificationCandidate::new("images".to_string(), 1.2, 0.4),
            ],
            score: 1.5,
            folder_name: "test_folder".to_string(),
            processing_time_ms: 100,
            metadata: HashMap::new(),
        };

        assert!(engine.needs_human_decision(&ambiguous_result));

        // Test unclassified with multiple candidates
        let unclassified_result = ClassificationResult {
            status: ClassificationStatus::Unclassified,
            category: None,
            candidates: vec![
                ClassificationCandidate::new("documents".to_string(), 0.05, 0.3),
                ClassificationCandidate::new("images".to_string(), 0.03, 0.2),
            ],
            score: 0.0,
            folder_name: "test_folder".to_string(),
            processing_time_ms: 100,
            metadata: HashMap::new(),
        };

        assert!(engine.needs_human_decision(&unclassified_result));

        // Test classified result (should not need human decision)
        let classified_result = ClassificationResult {
            status: ClassificationStatus::Classified,
            category: Some("documents".to_string()),
            candidates: vec![ClassificationCandidate::new(
                "documents".to_string(),
                1.5,
                0.8,
            )],
            score: 1.5,
            folder_name: "test_folder".to_string(),
            processing_time_ms: 100,
            metadata: HashMap::new(),
        };

        assert!(!engine.needs_human_decision(&classified_result));
    }

    #[test]
    fn test_tool_schema() {
        let tool = ClassificationTool::new(true);
        let info = tool.get_info();

        assert_eq!(info.name, "folder-classifier");
        assert_eq!(info.version, "1.0.0");
        assert!(info.description.contains("classification"));
        assert!(info.parameters_schema.get("properties").is_some());
        assert!(info.return_schema.get("properties").is_some());
    }
}
