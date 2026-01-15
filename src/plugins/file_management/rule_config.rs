//! Rule configuration support for classification

use super::classification_tool::{ClassificationRule, ClassificationRules};
use super::error::{FileManagementError, FileManagementResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

/// Rule configuration loader and validator
pub struct RuleConfigLoader {
    /// Cache for loaded rule files to avoid repeated parsing
    rule_cache: HashMap<String, ClassificationRules>,
}

impl RuleConfigLoader {
    /// Create a new rule configuration loader
    pub fn new() -> Self {
        Self {
            rule_cache: HashMap::new(),
        }
    }

    /// Load classification rules from various sources
    pub fn load_rules(
        &mut self,
        rules_source: &Value,
    ) -> FileManagementResult<ClassificationRules> {
        match rules_source {
            Value::String(file_path) => self.load_rules_from_file(file_path),
            Value::Object(_) => self.load_rules_from_json(rules_source),
            _ => Err(FileManagementError::validation(
                "Rules source must be either a file path (string) or a JSON object",
            )),
        }
    }

    /// Load rules from a JSON file
    pub fn load_rules_from_file(
        &mut self,
        file_path: &str,
    ) -> FileManagementResult<ClassificationRules> {
        // Check cache first
        if let Some(cached_rules) = self.rule_cache.get(file_path) {
            debug!("Using cached rules from file: {}", file_path);
            return Ok(cached_rules.clone());
        }

        // Validate file path
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(FileManagementError::not_found(path));
        }

        if !path.is_file() {
            return Err(FileManagementError::invalid_path(
                path,
                "Path is not a file",
            ));
        }

        // Read and parse file
        let content = std::fs::read_to_string(path).map_err(|e| {
            FileManagementError::io(format!("Failed to read rules file: {}", file_path), e)
        })?;

        let rules_value: Value = serde_json::from_str(&content).map_err(|e| {
            FileManagementError::validation(format!(
                "Invalid JSON in rules file '{}': {}",
                file_path, e
            ))
        })?;

        let rules = self.load_rules_from_json(&rules_value)?;

        // Cache the loaded rules
        self.rule_cache.insert(file_path.to_string(), rules.clone());

        info!(
            "Loaded {} classification rules from file: {}",
            rules.rules.len(),
            file_path
        );
        Ok(rules)
    }

    /// Load rules from a JSON value
    pub fn load_rules_from_json(
        &self,
        rules_value: &Value,
    ) -> FileManagementResult<ClassificationRules> {
        let mut processed_value = rules_value.clone();
        
        // Handle legacy "categories" field mapping to "rules"
        if let Some(obj) = processed_value.as_object_mut() {
            if obj.contains_key("categories") && !obj.contains_key("rules") {
                if let Some(categories) = obj.remove("categories") {
                    obj.insert("rules".to_string(), categories);
                }
            }
            
            // Transform nested keywords into combinations
            if let Some(rules) = obj.get_mut("rules").and_then(|r| r.as_array_mut()) {
                for rule in rules {
                    if let Some(rule_obj) = rule.as_object_mut() {
                        // Handle legacy "name" -> "category" mapping
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
                                        // This is a combination (nested array)
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
                                
                                // Update rule object
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

        // Parse the JSON into ClassificationRules
        let mut rules: ClassificationRules =
            serde_json::from_value(processed_value).map_err(|e| {
                FileManagementError::validation(format!(
                    "Invalid classification rules format: {}",
                    e
                ))
            })?;

        // Validate and process the rules
        self.validate_and_process_rules(&mut rules)?;

        debug!(
            "Loaded {} classification rules from JSON",
            rules.rules.len()
        );
        Ok(rules)
    }

    /// Validate and process classification rules
    fn validate_and_process_rules(
        &self,
        rules: &mut ClassificationRules,
    ) -> FileManagementResult<()> {
        if rules.rules.is_empty() {
            return Err(FileManagementError::validation(
                "At least one classification rule is required",
            ));
        }

        // Validate thresholds
        if rules.min_confidence_threshold < 0.0 || rules.min_confidence_threshold > 1.0 {
            return Err(FileManagementError::validation(
                "min_confidence_threshold must be between 0.0 and 1.0",
            ));
        }

        if rules.ambiguity_threshold < 0.0 || rules.ambiguity_threshold > 1.0 {
            return Err(FileManagementError::validation(
                "ambiguity_threshold must be between 0.0 and 1.0",
            ));
        }

        // Validate and process each rule
        for (index, rule) in rules.rules.iter_mut().enumerate() {
            self.validate_and_process_rule(rule, index)?;
        }

        Ok(())
    }

    /// Validate and process a single classification rule
    fn validate_and_process_rule(
        &self,
        rule: &mut ClassificationRule,
        index: usize,
    ) -> FileManagementResult<()> {
        // Validate category name
        if rule.category.trim().is_empty() {
            return Err(FileManagementError::validation(format!(
                "Rule {}: category cannot be empty",
                index
            )));
        }

        // Validate keywords
        if rule.keywords.is_empty() {
            return Err(FileManagementError::validation(format!(
                "Rule {}: at least one keyword is required",
                index
            )));
        }

        // Process and validate keywords
        let mut processed_keywords = Vec::new();
        for (keyword_index, keyword) in rule.keywords.iter().enumerate() {
            let trimmed = keyword.trim();
            if trimmed.is_empty() {
                return Err(FileManagementError::validation(format!(
                    "Rule {}, keyword {}: keyword cannot be empty",
                    index, keyword_index
                )));
            }

            processed_keywords.push(trimmed.to_string());
        }

        // Remove duplicates while preserving order
        let mut unique_keywords = Vec::new();
        for keyword in processed_keywords {
            if !unique_keywords.contains(&keyword) {
                unique_keywords.push(keyword);
            }
        }

        rule.keywords = unique_keywords;

        // Validate score weight
        if rule.score_weight <= 0.0 {
            return Err(FileManagementError::validation(format!(
                "Rule {}: score_weight must be positive",
                index
            )));
        }

        Ok(())
    }

    /// Clear the rule cache
    pub fn clear_cache(&mut self) {
        self.rule_cache.clear();
        debug!("Rule cache cleared");
    }
}

impl Default for RuleConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Complex keyword combination support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordCombination {
    /// Keywords that must all be present (AND logic)
    pub required_all: Vec<String>,
    /// Keywords where at least one must be present (OR logic)
    pub required_any: Vec<String>,
    /// Keywords that must not be present (NOT logic)
    pub excluded: Vec<String>,
    /// Score multiplier for this combination
    pub score_multiplier: f64,
}

impl KeywordCombination {
    /// Create a new keyword combination
    pub fn new() -> Self {
        Self {
            required_all: Vec::new(),
            required_any: Vec::new(),
            excluded: Vec::new(),
            score_multiplier: 1.0,
        }
    }
}

impl Default for KeywordCombination {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced classification rule with complex keyword combinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedClassificationRule {
    /// Base rule information
    pub base_rule: ClassificationRule,
    /// Complex keyword combinations
    pub combinations: Vec<KeywordCombination>,
    /// Rule priority (higher values take precedence)
    pub priority: i32,
}

impl EnhancedClassificationRule {
    /// Create a new enhanced rule from a base rule
    pub fn from_base(base_rule: ClassificationRule) -> Self {
        Self {
            base_rule,
            combinations: Vec::new(),
            priority: 0,
        }
    }
}
