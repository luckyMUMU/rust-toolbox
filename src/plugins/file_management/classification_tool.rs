//! Classification Tool implementation
//! 
//! This module provides intelligent folder classification using AC automaton
//! and text processing with scoring algorithms and decision making.

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::tools::ToolNode;
use super::ac_automaton::{AhoCorasickMatcher, AutomatonConfig, PatternMatch};
use super::error::{FileManagementError, FileManagementResult};
// use super::rule_config::RuleConfigLoader;
use super::utils::{
    TextProcessor, TextNormalizationConfig, 
    HumanDecisionContext, HumanDecisionType,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Classification rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    pub category: String,
    pub keywords: Vec<String>,
    pub score_weight: f64,
    pub required_matches: Option<usize>,
    pub case_sensitive: bool,
    pub use_pinyin: bool,
}

impl ClassificationRule {
    pub fn new<S: Into<String>>(category: S, keywords: Vec<String>) -> Self {
        Self {
            category: category.into(),
            keywords,
            score_weight: 1.0,
            required_matches: None,
            case_sensitive: false,
            use_pinyin: true,
        }
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
    pub min_confidence_threshold: f64,
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
        self.matched_keywords = matches.iter()
            .map(|m| m.pattern.clone())
            .collect();
        self.match_details = matches;
        self
    }
}

/// Classification engine core
pub struct ClassificationEngine {
    text_processor: TextProcessor,
    normalization_config: TextNormalizationConfig,
    enable_chinese: bool,
}

impl ClassificationEngine {
    /// Create a new classification engine
    pub fn new(enable_chinese: bool) -> Self {
        let normalization_config = TextNormalizationConfig::default();
        let text_processor = TextProcessor::with_config(enable_chinese, normalization_config.clone());
        
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
    pub fn build_automaton(&self, rules: &ClassificationRules) -> FileManagementResult<AhoCorasickMatcher> {
        let config = AutomatonConfig {
            case_sensitive: false, // We'll handle case sensitivity in preprocessing
            find_overlapping: true,
            max_patterns: 10_000,
            max_pattern_length: 1000,
        };

        let mut automaton = AhoCorasickMatcher::with_config(config);
        let mut _pattern_id = 0;

        for rule in &rules.rules {
            // Add original keywords
            for keyword in &rule.keywords {
                let processed_keyword = self.preprocess_keyword(keyword, rule);
                automaton.add_pattern(&processed_keyword, &rule.category, rule.score_weight)
                    .map_err(|e| FileManagementError::classification(
                        format!("Failed to add pattern '{}': {}", processed_keyword, e)
                    ))?;
                _pattern_id += 1;
            }

            // Add pinyin variants if enabled
            if rule.use_pinyin && self.enable_chinese {
                for keyword in &rule.keywords {
                    let pinyin_variants = self.text_processor.generate_pinyin_variants(keyword);
                    for variant in pinyin_variants {
                        if variant != *keyword {
                            let processed_variant = self.preprocess_keyword(&variant, rule);
                            if let Err(e) = automaton.add_pattern(&processed_variant, &rule.category, rule.score_weight * 0.8) {
                                // Pinyin variants get slightly lower weight
                                debug!("Failed to add pinyin variant '{}': {}", processed_variant, e);
                                // Continue with other variants even if one fails
                            } else {
                                _pattern_id += 1;
                            }
                        }
                    }
                }
            }
        }

        automaton.build()
            .map_err(|e| FileManagementError::classification(
                format!("Failed to build automaton: {}", e)
            ))?;

        debug!("Built classification automaton with {} patterns", automaton.pattern_count());
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
            if !mixed_result.simplified_chinese.is_empty() && mixed_result.simplified_chinese != mixed_result.chinese_chars {
                combined.push(' ');
                combined.push_str(&mixed_result.simplified_chinese);
            }
            
            processed = combined;
        }

        debug!("Preprocessed folder name: '{}' -> '{}'", folder_name, processed);
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
        let matches = automaton.find_matches(&processed_name)
            .map_err(|e| FileManagementError::classification(
                format!("Pattern matching failed: {}", e)
            ))?;

        // Calculate scores by category
        let candidates = self.calculate_category_scores(&matches, rules)?;
        
        // Determine classification result
        let result = self.determine_classification_result(
            folder_name,
            candidates,
            rules,
            start_time.elapsed().as_millis() as u64,
        )?;

        debug!("Classified '{}' as {:?} in {}ms", 
               folder_name, result.status, result.processing_time_ms);
        
        Ok(result)
    }

    /// Calculate scores for each category based on matches
    fn calculate_category_scores(
        &self,
        matches: &[PatternMatch],
        rules: &ClassificationRules,
    ) -> FileManagementResult<Vec<ClassificationCandidate>> {
        let mut category_scores: HashMap<String, (f64, Vec<PatternMatch>)> = HashMap::new();
        
        // Group matches by category and calculate scores
        for pattern_match in matches {
            let entry = category_scores.entry(pattern_match.category.clone())
                .or_insert((0.0, Vec::new()));
            
            entry.0 += pattern_match.score;
            entry.1.push(pattern_match.clone());
        }

        // Convert to candidates and calculate confidence
        let mut candidates = Vec::new();
        let total_score: f64 = category_scores.values().map(|(score, _)| *score).sum();
        
        for (category, (score, matches)) in category_scores {
            let confidence = if total_score > 0.0 { score / total_score } else { 0.0 };
            
            // Check if required matches are met
            if let Some(rule) = rules.rules.iter().find(|r| r.category == category) {
                if let Some(required) = rule.required_matches {
                    if matches.len() < required {
                        debug!("Category '{}' doesn't meet required matches: {} < {}", 
                               category, matches.len(), required);
                        continue;
                    }
                }
            }
            
            let candidate = ClassificationCandidate::new(category, score, confidence)
                .with_matches(matches);
            candidates.push(candidate);
        }

        // Sort by score (descending)
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
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
            ClassificationStatus::Unclassified => {
                (rules.default_category.clone(), 0.0)
            }
            ClassificationStatus::Ambiguous => {
                // Keep top candidate but mark as ambiguous
                (Some(candidates[0].category.clone()), candidates[0].score)
            }
            ClassificationStatus::Error => {
                (None, 0.0)
            }
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
        matches!(result.status, ClassificationStatus::Ambiguous) ||
        (matches!(result.status, ClassificationStatus::Unclassified) && result.candidates.len() > 1)
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
                format!("Please confirm the classification for folder '{}'.", result.folder_name)
            }
        };

        let mut context = HumanDecisionContext::new(
            HumanDecisionType::Classification,
            title,
            description,
        );

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
    Simple,      // Just category and confidence
    Detailed,    // Include candidates and metadata
    Full,        // Complete result with all details
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
    fn load_classification_rules(&self, rules_value: &Value) -> FileManagementResult<ClassificationRules> {
        if let Some(file_path) = rules_value.as_str() {
            // Load from file
            let content = std::fs::read_to_string(file_path)
                .map_err(|e| FileManagementError::io(
                    format!("Failed to read rules file: {}", file_path), e
                ))?;
            
            serde_json::from_str(&content)
                .map_err(|e| FileManagementError::validation(
                    format!("Invalid JSON in rules file: {}", e)
                ))
        } else {
            // Parse as JSON object
            serde_json::from_value(rules_value.clone())
                .map_err(|e| FileManagementError::validation(
                    format!("Invalid classification rules: {}", e)
                ))
        }
    }

    /// Format result based on output format
    fn format_result(&self, result: ClassificationResult, format: &ClassificationOutputFormat) -> Value {
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

    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        info!("Executing folder classification tool");

        // Parse parameters
        let params: ClassificationParams = serde_json::from_value(params)
            .map_err(|e| WorkflowError::ValidationError(format!("Invalid parameters: {}", e)))?;

        // Validate folder path
        let folder_path = Path::new(&params.folder_path);
        if !folder_path.exists() {
            return Err(WorkflowError::ValidationError(
                format!("Folder path does not exist: {}", params.folder_path)
            ));
        }

        let folder_name = folder_path.file_name()
            .ok_or_else(|| WorkflowError::ValidationError("Invalid folder path".to_string()))?
            .to_string_lossy()
            .to_string();

        // Load classification rules
        let rules = self.load_classification_rules(&params.classification_rules)
            .map_err(|e| WorkflowError::tool(format!("Failed to load rules: {}", e)))?;

        // Build automaton
        let automaton = self.engine.build_automaton(&rules)
            .map_err(|e| WorkflowError::tool(format!("Failed to build automaton: {}", e)))?;

        // Classify folder
        let mut result = self.engine.classify_folder(&folder_name, &automaton, &rules)
            .map_err(|e| WorkflowError::tool(format!("Classification failed: {}", e)))?;

        // Handle experimental mode
        if params.experimental_mode {
            result.metadata.insert("experimental_mode".to_string(), Value::Bool(true));
            info!("Classification completed in experimental mode");
        }

        // Handle human interaction if needed
        if params.enable_user_interaction && self.engine.needs_human_decision(&result) {
            let decision_context = self.engine.create_human_decision_context(&result);
            result.metadata.insert("human_decision_required".to_string(), Value::Bool(true));
            result.metadata.insert("decision_context".to_string(), 
                serde_json::to_value(decision_context).unwrap_or(Value::Null));
            result.status = ClassificationStatus::Pending;
        }

        // Format result
        let output_format = params.output_format.unwrap_or_default();
        let formatted_result = self.format_result(result, &output_format);

        info!("Folder classification completed successfully");
        Ok(formatted_result)
    }

    fn validate_parameters(&self, params: &Value) -> Result<()> {
        let parsed_params: ClassificationParams = serde_json::from_value(params.clone())
            .map_err(|e| WorkflowError::ValidationError(format!("Parameter validation failed: {}", e)))?;

        // Validate folder path
        if parsed_params.folder_path.trim().is_empty() {
            return Err(WorkflowError::ValidationError("Folder path cannot be empty".to_string()));
        }

        // Validate classification rules
        if parsed_params.classification_rules.is_null() {
            return Err(WorkflowError::ValidationError("Classification rules are required".to_string()));
        }

        Ok(())
    }

    fn get_info(&self) -> ToolInfo {
        let now = Utc::now();
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: "Intelligent folder classification using configurable rules and AC automaton".to_string(),
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