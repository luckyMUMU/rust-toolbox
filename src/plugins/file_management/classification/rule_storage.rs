//! 分类规则存储服务
//!
//! 提供分类规则的持久化存储能力，支持：
//! - 从 `classfy.json` 格式导入规则
//! - 规则版本管理和历史追踪
//! - 规则更新和导出

use crate::error::{Result, WorkflowError};
use crate::plugins::file_management::classification::{ClassificationRule, ClassificationRules};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// 规则版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersion {
    /// 版本号
    pub version: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 创建者
    pub created_by: Option<String>,
    /// 变更描述
    pub change_description: Option<String>,
    /// 规则数量
    pub rule_count: usize,
}

/// 规则存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStorageConfig {
    /// 存储目录
    pub storage_dir: PathBuf,
    /// 当前规则文件名
    pub current_rules_file: String,
    /// 历史版本目录
    pub history_dir: String,
    /// 最大历史版本数
    pub max_history_versions: usize,
    /// 自动保存间隔（秒）
    pub auto_save_interval_secs: Option<u64>,
}

impl Default for RuleStorageConfig {
    fn default() -> Self {
        Self {
            storage_dir: PathBuf::from("./data/rules"),
            current_rules_file: "current_rules.json".to_string(),
            history_dir: "history".to_string(),
            max_history_versions: 10,
            auto_save_interval_secs: None,
        }
    }
}

/// 分类规则存储服务
pub struct ClassificationRuleStorage {
    /// 配置
    config: RuleStorageConfig,
    /// 当前规则
    current_rules: Arc<RwLock<Option<ClassificationRules>>>,
    /// 版本历史
    version_history: Arc<RwLock<Vec<RuleVersion>>>,
}

impl ClassificationRuleStorage {
    /// 创建新的规则存储服务
    pub fn new(config: RuleStorageConfig) -> Self {
        Self {
            config,
            current_rules: Arc::new(RwLock::new(None)),
            version_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 从 classfy.json 格式导入规则
    pub async fn import_from_classfy_json<P: AsRef<Path>>(&self, path: P) -> Result<ClassificationRules> {
        let path = path.as_ref();
        info!("从 classfy.json 导入规则: {}", path.display());

        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            WorkflowError::execution(format!("读取规则文件失败: {}", path.display()))
        })?;

        let classfy_config: ClassfyJsonConfig = serde_json::from_str(&content).map_err(|e| {
            WorkflowError::validation(format!("解析 classfy.json 格式失败: {}", e))
        })?;

        // 转换为 ClassificationRules
        let rules = self.convert_classfy_to_rules(&classfy_config)?;

        // 保存为当前规则
        let mut current = self.current_rules.write().await;
        *current = Some(rules.clone());

        info!("成功导入 {} 条分类规则", rules.rules.len());
        Ok(rules)
    }

    /// 将 ClassfyJsonConfig 转换为 ClassificationRules
    fn convert_classfy_to_rules(&self, classfy: &ClassfyJsonConfig) -> Result<ClassificationRules> {
        let mut rules = Vec::new();

        for category in &classfy.categories {
            let mut rule = ClassificationRule::new(&category.name, category.keywords.clone());

            if let Some(weight) = category.score_weight {
                rule = rule.with_score_weight(weight);
            }

            if let Some(required) = category.required_matches {
                rule = rule.with_required_matches(required);
            }

            if let Some(case_sensitive) = category.case_sensitive {
                rule = rule.case_sensitive(case_sensitive);
            }

            if let Some(use_pinyin) = category.use_pinyin {
                rule = rule.use_pinyin(use_pinyin);
            }

            // 处理组合关键词
            if let Some(combinations) = &category.combinations {
                rule.combinations = Some(combinations.clone());
            }

            rules.push(rule);
        }

        Ok(ClassificationRules {
            rules,
            default_category: Some(classfy.default_category.clone()),
            min_confidence_threshold: classfy.min_confidence_threshold.unwrap_or(0.3),
            ambiguity_threshold: classfy.ambiguity_threshold.unwrap_or(0.1),
        })
    }

    /// 导出为 classfy.json 格式
    pub async fn export_to_classfy_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let current = self.current_rules.read().await;

        let rules = current.as_ref().ok_or_else(|| {
            WorkflowError::validation("没有当前规则可导出")
        })?;

        let classfy = self.convert_rules_to_classfy(rules);

        let content = serde_json::to_string_pretty(&classfy).map_err(|e| {
            WorkflowError::validation(format!("序列化规则失败: {}", e))
        })?;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                WorkflowError::execution(format!("创建目录失败: {}", parent.display()))
            })?;
        }

        tokio::fs::write(path, content).await.map_err(|e| {
            WorkflowError::execution(format!("写入规则文件失败: {}", path.display()))
        })?;

        info!("成功导出规则到: {}", path.display());
        Ok(())
    }

    /// 将 ClassificationRules 转换为 ClassfyJsonConfig
    fn convert_rules_to_classfy(&self, rules: &ClassificationRules) -> ClassfyJsonConfig {
        let categories: Vec<ClassfyCategory> = rules
            .rules
            .iter()
            .map(|rule| ClassfyCategory {
                name: rule.category.clone(),
                keywords: rule.keywords.clone(),
                score_weight: Some(rule.score_weight),
                required_matches: rule.required_matches,
                case_sensitive: Some(rule.case_sensitive),
                use_pinyin: Some(rule.use_pinyin),
                combinations: rule.combinations.clone(),
            })
            .collect();

        ClassfyJsonConfig {
            categories,
            default_category: rules.default_category.clone().unwrap_or_else(|| "other".to_string()),
            min_confidence_threshold: Some(rules.min_confidence_threshold),
            ambiguity_threshold: Some(rules.ambiguity_threshold),
        }
    }

    /// 获取当前规则
    pub async fn get_current_rules(&self) -> Option<ClassificationRules> {
        self.current_rules.read().await.clone()
    }

    /// 更新规则
    pub async fn update_rules(&self, rules: ClassificationRules, description: Option<String>) -> Result<()> {
        // 创建版本记录
        let version = RuleVersion {
            version: chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string(),
            created_at: chrono::Utc::now(),
            created_by: None,
            change_description: description,
            rule_count: rules.rules.len(),
        };

        // 更新当前规则
        let mut current = self.current_rules.write().await;
        *current = Some(rules);

        // 添加版本历史
        let mut history = self.version_history.write().await;
        history.push(version);

        // 限制历史版本数量
        if history.len() > self.config.max_history_versions {
            history.remove(0);
        }

        Ok(())
    }

    /// 添加单个规则
    pub async fn add_rule(&self, rule: ClassificationRule) -> Result<()> {
        let mut current = self.current_rules.write().await;

        if let Some(ref mut rules) = *current {
            // 检查是否已存在同名规则
            if rules.rules.iter().any(|r| r.category == rule.category) {
                return Err(WorkflowError::validation(format!(
                    "规则类别 '{}' 已存在",
                    rule.category
                )));
            }
            rules.rules.push(rule);
        } else {
            *current = Some(ClassificationRules {
                rules: vec![rule],
                default_category: Some("other".to_string()),
                min_confidence_threshold: 0.3,
                ambiguity_threshold: 0.1,
            });
        }

        Ok(())
    }

    /// 更新单个规则
    pub async fn update_rule(&self, category: &str, rule: ClassificationRule) -> Result<()> {
        let mut current = self.current_rules.write().await;

        if let Some(ref mut rules) = *current {
            if let Some(existing) = rules.rules.iter_mut().find(|r| r.category == category) {
                *existing = rule;
            } else {
                return Err(WorkflowError::validation(format!(
                    "规则类别 '{}' 不存在",
                    category
                )));
            }
        } else {
            return Err(WorkflowError::validation("没有当前规则"));
        }

        Ok(())
    }

    /// 删除规则
    pub async fn remove_rule(&self, category: &str) -> Result<()> {
        let mut current = self.current_rules.write().await;

        if let Some(ref mut rules) = *current {
            let initial_len = rules.rules.len();
            rules.rules.retain(|r| r.category != category);

            if rules.rules.len() == initial_len {
                return Err(WorkflowError::validation(format!(
                    "规则类别 '{}' 不存在",
                    category
                )));
            }
        } else {
            return Err(WorkflowError::validation("没有当前规则"));
        }

        Ok(())
    }

    /// 获取版本历史
    pub async fn get_version_history(&self) -> Vec<RuleVersion> {
        self.version_history.read().await.clone()
    }

    /// 保存当前规则到文件
    pub async fn save_current_rules(&self) -> Result<()> {
        let path = self.config.storage_dir.join(&self.config.current_rules_file);
        self.export_to_classfy_json(&path).await
    }

    /// 从文件加载规则
    pub async fn load_rules(&self) -> Result<ClassificationRules> {
        let path = self.config.storage_dir.join(&self.config.current_rules_file);

        if !path.exists() {
            return Err(WorkflowError::validation(format!(
                "规则文件不存在: {}",
                path.display()
            )));
        }

        self.import_from_classfy_json(&path).await
    }
}

/// classfy.json 配置格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassfyJsonConfig {
    /// 分类类别列表
    pub categories: Vec<ClassfyCategory>,
    /// 默认分类
    #[serde(default = "default_category")]
    pub default_category: String,
    /// 最小置信度阈值
    #[serde(default = "default_min_confidence")]
    pub min_confidence_threshold: Option<f64>,
    /// 模糊阈值
    #[serde(default = "default_ambiguity_threshold")]
    pub ambiguity_threshold: Option<f64>,
}

fn default_category() -> String {
    "other".to_string()
}

fn default_min_confidence() -> Option<f64> {
    Some(0.3)
}

fn default_ambiguity_threshold() -> Option<f64> {
    Some(0.1)
}

/// classfy.json 中的分类类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassfyCategory {
    /// 类别名称
    pub name: String,
    /// 关键词列表
    pub keywords: Vec<String>,
    /// 分数权重
    #[serde(default = "default_score_weight")]
    pub score_weight: Option<f64>,
    /// 需要匹配的关键词数量
    pub required_matches: Option<usize>,
    /// 是否区分大小写
    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: Option<bool>,
    /// 是否使用拼音匹配
    #[serde(default = "default_use_pinyin")]
    pub use_pinyin: Option<bool>,
    /// 组合关键词
    pub combinations: Option<Vec<Vec<String>>>,
}

fn default_score_weight() -> Option<f64> {
    Some(1.0)
}

fn default_case_sensitive() -> Option<bool> {
    Some(false)
}

fn default_use_pinyin() -> Option<bool> {
    Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_classfy_json_deserialization() {
        let json = r#"{
            "categories": [
                {
                    "name": "documents",
                    "keywords": ["doc", "pdf", "document"],
                    "score_weight": 1.5,
                    "case_sensitive": false,
                    "use_pinyin": true
                }
            ],
            "default_category": "other",
            "min_confidence_threshold": 0.3,
            "ambiguity_threshold": 0.1
        }"#;

        let config: ClassfyJsonConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.categories.len(), 1);
        assert_eq!(config.categories[0].name, "documents");
        assert_eq!(config.categories[0].keywords.len(), 3);
    }

    #[test]
    fn test_classfy_json_serialization() {
        let config = ClassfyJsonConfig {
            categories: vec![ClassfyCategory {
                name: "images".to_string(),
                keywords: vec!["img".to_string(), "pic".to_string()],
                score_weight: Some(1.2),
                required_matches: None,
                case_sensitive: Some(false),
                use_pinyin: Some(true),
                combinations: None,
            }],
            default_category: "other".to_string(),
            min_confidence_threshold: Some(0.3),
            ambiguity_threshold: Some(0.1),
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("images"));
        assert!(json.contains("img"));
    }

    #[tokio::test]
    async fn test_rule_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RuleStorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let storage = ClassificationRuleStorage::new(config);
        assert!(storage.get_current_rules().await.is_none());
    }

    #[tokio::test]
    async fn test_add_rule() {
        let temp_dir = TempDir::new().unwrap();
        let config = RuleStorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let storage = ClassificationRuleStorage::new(config);
        let rule = ClassificationRule::new("test", vec!["keyword1".to_string()]);

        storage.add_rule(rule).await.unwrap();
        let rules = storage.get_current_rules().await;
        assert!(rules.is_some());
        assert_eq!(rules.unwrap().rules.len(), 1);
    }

    #[tokio::test]
    async fn test_update_rule() {
        let temp_dir = TempDir::new().unwrap();
        let config = RuleStorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let storage = ClassificationRuleStorage::new(config);
        let rule = ClassificationRule::new("test", vec!["keyword1".to_string()]);
        storage.add_rule(rule).await.unwrap();

        let updated_rule = ClassificationRule::new("test", vec!["keyword1".to_string(), "keyword2".to_string()]);
        storage.update_rule("test", updated_rule).await.unwrap();

        let rules = storage.get_current_rules().await.unwrap();
        assert_eq!(rules.rules[0].keywords.len(), 2);
    }

    #[tokio::test]
    async fn test_remove_rule() {
        let temp_dir = TempDir::new().unwrap();
        let config = RuleStorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let storage = ClassificationRuleStorage::new(config);
        let rule = ClassificationRule::new("test", vec!["keyword1".to_string()]);
        storage.add_rule(rule).await.unwrap();

        storage.remove_rule("test").await.unwrap();
        let rules = storage.get_current_rules().await.unwrap();
        assert_eq!(rules.rules.len(), 0);
    }
}
