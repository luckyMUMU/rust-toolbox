//! 增强版分类工具
//!
//! 基于 Python 参考实现 `folder_classifier_v5_improved2.py` 的功能增强
//! 提供多级分类、批量处理、进度追踪和智能合并建议

use crate::core::PluginInfo;
use crate::plugins::file_management::core::error::{FileManagementError, FileManagementResult};
use crate::plugins::file_management::classification::{
    ClassificationCandidate, ClassificationEngine, ClassificationResult, ClassificationRules,
    ClassificationStatus,
};
use crate::tools::algo::ac_automaton::AhoCorasickMatcher;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 多级分类类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalCategory {
    /// 类别ID
    pub id: String,
    /// 类别名称
    pub name: String,
    /// 父类别ID
    pub parent_id: Option<String>,
    /// 子类别列表
    pub children: Vec<HierarchicalCategory>,
    /// 类别权重
    pub weight: f64,
    /// 类别关键词
    pub keywords: Vec<String>,
    /// 类别描述
    pub description: Option<String>,
}

impl HierarchicalCategory {
    /// 创建新的分类类别
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            parent_id: None,
            children: Vec::new(),
            weight: 1.0,
            keywords: Vec::new(),
            description: None,
        }
    }

    /// 设置父类别
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// 添加子类别
    pub fn add_child(&mut self, child: HierarchicalCategory) {
        self.children.push(child);
    }

    /// 设置权重
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// 设置关键词
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// 获取完整路径（从根到当前类别）
    pub fn get_full_path(&self, categories: &HashMap<String, &HierarchicalCategory>) -> String {
        let mut path = vec![self.name.clone()];
        let mut current_id = self.parent_id.as_ref();

        while let Some(pid) = current_id {
            if let Some(parent) = categories.get(pid) {
                path.push(parent.name.clone());
                current_id = parent.parent_id.as_ref();
            } else {
                break;
            }
        }

        path.reverse();
        path.join(" > ")
    }

    /// 检查是否为叶子节点
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// 获取所有后代类别
    pub fn get_all_descendants(&self) -> Vec<&HierarchicalCategory> {
        let mut descendants = Vec::new();
        for child in &self.children {
            descendants.push(child);
            descendants.extend(child.get_all_descendants());
        }
        descendants
    }
}

/// 分类进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationProgress {
    /// 总项目数
    pub total_items: usize,
    /// 已处理项目数
    pub processed_items: usize,
    /// 成功分类数
    pub successful_count: usize,
    /// 失败分类数
    pub failed_count: usize,
    /// 模糊分类数（需要人工确认）
    pub ambiguous_count: usize,
    /// 当前处理的项目
    pub current_item: Option<String>,
    /// 开始时间
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 预估剩余时间（秒）
    pub estimated_remaining_secs: Option<u64>,
    /// 处理速度（项目/秒）
    pub items_per_second: Option<f64>,
}

impl ClassificationProgress {
    pub fn new(total_items: usize) -> Self {
        Self {
            total_items,
            processed_items: 0,
            successful_count: 0,
            failed_count: 0,
            ambiguous_count: 0,
            current_item: None,
            started_at: chrono::Utc::now(),
            estimated_remaining_secs: None,
            items_per_second: None,
        }
    }

    pub fn update(&mut self, result: &ClassificationResult) {
        self.processed_items += 1;
        match result.status {
            ClassificationStatus::Classified => self.successful_count += 1,
            ClassificationStatus::Unclassified => self.failed_count += 1,
            ClassificationStatus::Ambiguous => self.ambiguous_count += 1,
            _ => {}
        }

        // 计算处理速度
        let elapsed = (chrono::Utc::now() - self.started_at).num_seconds();
        if elapsed > 0 {
            self.items_per_second = Some(self.processed_items as f64 / elapsed as f64);
            
            // 预估剩余时间
            if let Some(speed) = self.items_per_second {
                if speed > 0.0 {
                    let remaining = self.total_items - self.processed_items;
                    self.estimated_remaining_secs = Some((remaining as f64 / speed) as u64);
                }
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.processed_items >= self.total_items
    }

    pub fn get_progress_percentage(&self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (self.processed_items as f64 / self.total_items as f64) * 100.0
        }
    }
}

/// 分类结果缓存
#[derive(Debug, Clone)]
pub struct ClassificationCache {
    /// 缓存存储
    cache: Arc<RwLock<HashMap<String, CachedClassification>>>,
    /// 最大缓存大小
    max_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedClassification {
    result: ClassificationResult,
    cached_at: chrono::DateTime<chrono::Utc>,
    hit_count: usize,
}

impl ClassificationCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    pub async fn get(&self, key: &str) -> Option<ClassificationResult> {
        let mut cache = self.cache.write().await;
        if let Some(cached) = cache.get_mut(key) {
            cached.hit_count += 1;
            return Some(cached.result.clone());
        }
        None
    }

    pub async fn put(&self, key: String, result: ClassificationResult) {
        let mut cache = self.cache.write().await;
        
        // 如果缓存已满，移除最少使用的项
        if cache.len() >= self.max_size {
            let mut min_hit_count = usize::MAX;
            let mut min_key = None;
            
            for (k, v) in cache.iter() {
                if v.hit_count < min_hit_count {
                    min_hit_count = v.hit_count;
                    min_key = Some(k.clone());
                }
            }
            
            if let Some(key) = min_key {
                cache.remove(&key);
            }
        }

        cache.insert(key, CachedClassification {
            result,
            cached_at: chrono::Utc::now(),
            hit_count: 0,
        });
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

/// 智能合并建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeSuggestion {
    /// 源文件夹路径
    pub source_path: PathBuf,
    /// 建议的目标类别
    pub suggested_category: String,
    /// 置信度
    pub confidence: f64,
    /// 匹配的关键词
    pub matched_keywords: Vec<String>,
    /// 建议原因
    pub reason: String,
    /// 是否需要人工确认
    pub needs_confirmation: bool,
}

/// 批量分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchClassificationResult {
    /// 总项目数
    pub total_items: usize,
    /// 成功分类数
    pub successful: usize,
    /// 失败分类数
    pub failed: usize,
    /// 模糊分类数
    pub ambiguous: usize,
    /// 分类结果列表
    pub results: Vec<(String, ClassificationResult)>,
    /// 合并建议
    pub merge_suggestions: Vec<MergeSuggestion>,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 分类统计
    pub category_stats: HashMap<String, usize>,
}

/// 增强版分类工具
pub struct EnhancedClassificationTool {
    /// 分类引擎
    engine: ClassificationEngine,
    /// 分类缓存
    cache: ClassificationCache,
    /// 层级类别映射
    hierarchical_categories: HashMap<String, HierarchicalCategory>,
    /// 插件信息
    plugin_info: Option<PluginInfo>,
    /// 是否启用缓存
    enable_cache: bool,
    /// 模糊阈值
    ambiguity_threshold: f64,
}

impl EnhancedClassificationTool {
    /// 创建新的增强版分类工具
    pub fn new(enable_chinese: bool) -> Self {
        Self {
            engine: ClassificationEngine::new(enable_chinese),
            cache: ClassificationCache::new(1000),
            hierarchical_categories: HashMap::new(),
            plugin_info: None,
            enable_cache: true,
            ambiguity_threshold: 0.8,
        }
    }

    /// 设置插件信息
    pub fn with_plugin_info(mut self, plugin_info: PluginInfo) -> Self {
        self.plugin_info = Some(plugin_info);
        self
    }

    /// 设置是否启用缓存
    pub fn with_cache(mut self, enable: bool, max_size: usize) -> Self {
        self.enable_cache = enable;
        self.cache = ClassificationCache::new(max_size);
        self
    }

    /// 设置模糊阈值
    pub fn with_ambiguity_threshold(mut self, threshold: f64) -> Self {
        self.ambiguity_threshold = threshold;
        self
    }

    /// 添加层级类别
    pub fn add_hierarchical_category(&mut self, category: HierarchicalCategory) {
        self.hierarchical_categories.insert(category.id.clone(), category);
    }

    /// 批量分类文件夹
    pub async fn classify_batch(
        &self,
        folders: &[String],
        automaton: &AhoCorasickMatcher,
        rules: &ClassificationRules,
    ) -> FileManagementResult<BatchClassificationResult> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        let mut merge_suggestions = Vec::new();
        let mut category_stats: HashMap<String, usize> = HashMap::new();
        let mut successful = 0;
        let mut failed = 0;
        let mut ambiguous = 0;

        for folder_path in folders {
            // 检查缓存
            let cache_key = if self.enable_cache {
                let path = std::path::Path::new(folder_path);
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase())
            } else {
                None
            };

            let result = if let Some(ref key) = cache_key {
                if let Some(cached) = self.cache.get(key).await {
                    debug!("使用缓存结果: {}", folder_path);
                    cached
                } else {
                    self.engine.classify_folder(folder_path, automaton, rules)?
                }
            } else {
                self.engine.classify_folder(folder_path, automaton, rules)?
            };

            // 更新统计
            match result.status {
                ClassificationStatus::Classified => successful += 1,
                ClassificationStatus::Unclassified => failed += 1,
                ClassificationStatus::Ambiguous => ambiguous += 1,
                _ => {}
            }

            // 更新类别统计
            if let Some(ref category) = result.category {
                *category_stats.entry(category.clone()).or_insert(0) += 1;
            }

            // 生成合并建议
            if result.status == ClassificationStatus::Classified {
                if let Some(ref category) = result.category {
                    if let Some(first_candidate) = result.candidates.first() {
                        merge_suggestions.push(MergeSuggestion {
                            source_path: PathBuf::from(folder_path),
                            suggested_category: category.clone(),
                            confidence: first_candidate.confidence,
                            matched_keywords: first_candidate.matched_keywords.clone(),
                            reason: format!(
                                "匹配关键词: {}",
                                first_candidate.matched_keywords.join(", ")
                            ),
                            needs_confirmation: first_candidate.confidence < self.ambiguity_threshold,
                        });
                    }
                }
            }

            // 缓存结果
            if let Some(key) = cache_key {
                self.cache.put(key, result.clone()).await;
            }

            results.push((folder_path.to_string(), result));
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "批量分类完成: {} 成功, {} 失败, {} 模糊, 耗时 {} ms",
            successful, failed, ambiguous, processing_time_ms
        );

        Ok(BatchClassificationResult {
            total_items: folders.len(),
            successful,
            failed,
            ambiguous,
            results,
            merge_suggestions,
            processing_time_ms,
            category_stats,
        })
    }

    /// 获取分类进度
    pub fn create_progress_tracker(&self, total_items: usize) -> ClassificationProgress {
        ClassificationProgress::new(total_items)
    }

    /// 根据分类结果生成合并计划
    pub fn generate_merge_plan(
        &self,
        batch_result: &BatchClassificationResult,
        target_base_path: &str,
    ) -> FileManagementResult<Vec<MergeSuggestion>> {
        let mut valid_suggestions = Vec::new();

        for suggestion in &batch_result.merge_suggestions {
            // 验证目标路径
            let target_path = std::path::Path::new(target_base_path)
                .join(&suggestion.suggested_category);

            if !target_path.exists() {
                debug!("目标类别目录不存在，将创建: {}", target_path.display());
            }

            valid_suggestions.push(suggestion.clone());
        }

        Ok(valid_suggestions)
    }

    /// 获取层级类别信息
    pub fn get_hierarchical_category(&self, id: &str) -> Option<&HierarchicalCategory> {
        self.hierarchical_categories.get(id)
    }

    /// 获取所有叶子类别
    pub fn get_leaf_categories(&self) -> Vec<&HierarchicalCategory> {
        self.hierarchical_categories
            .values()
            .filter(|c| c.is_leaf())
            .collect()
    }

    /// 执行分类
    pub async fn execute(&self, params: Value) -> FileManagementResult<Value> {
        let folders: Vec<String> = params
            .get("folders")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| FileManagementError::validation("缺少 folders 参数"))?;

        let rules_value = params
            .get("classification_rules")
            .ok_or_else(|| FileManagementError::validation("缺少 classification_rules 参数"))?;

        let rules: ClassificationRules = serde_json::from_value(rules_value.clone())
            .map_err(|e| FileManagementError::validation(format!("无效的分类规则: {}", e)))?;

        let automaton = self.engine.build_automaton(&rules)?;
        let batch_result = self.classify_batch(&folders, &automaton, &rules).await?;

        Ok(json!({
            "status": "completed",
            "total_items": batch_result.total_items,
            "successful": batch_result.successful,
            "failed": batch_result.failed,
            "ambiguous": batch_result.ambiguous,
            "processing_time_ms": batch_result.processing_time_ms,
            "category_stats": batch_result.category_stats,
            "merge_suggestions": batch_result.merge_suggestions,
            "results": batch_result.results.iter().map(|(path, result)| {
                json!({
                    "path": path,
                    "status": result.status,
                    "category": result.category,
                    "score": result.score,
                    "candidates": result.candidates.iter().take(3).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        }))
    }

    /// 获取工具信息
    pub fn get_info(&self) -> PluginInfo {
        self.plugin_info.clone().unwrap_or_else(|| PluginInfo {
            name: "enhanced-folder-classifier".to_string(),
            version: "2.0.0".to_string(),
            description: Some("增强版文件夹分类工具，支持多级分类、批量处理和智能合并建议".to_string()),
            author: Some("system".to_string()),
            homepage: None,
            plugin_type: crate::core::PluginType::Native,
            metadata: std::collections::HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchical_category() {
        let mut parent = HierarchicalCategory::new("docs", "文档");
        parent = parent.with_keywords(vec!["doc".to_string(), "document".to_string()]);

        let child = HierarchicalCategory::new("pdf", "PDF文档")
            .with_parent("docs")
            .with_keywords(vec!["pdf".to_string()]);

        assert_eq!(parent.id, "docs");
        assert!(parent.parent_id.is_none());
        assert!(child.parent_id.is_some());
    }

    #[test]
    fn test_classification_progress() {
        let mut progress = ClassificationProgress::new(10);
        assert_eq!(progress.total_items, 10);
        assert_eq!(progress.processed_items, 0);
        assert!(!progress.is_complete());

        let result = ClassificationResult {
            status: ClassificationStatus::Classified,
            category: Some("test".to_string()),
            candidates: vec![],
            score: 1.0,
            folder_name: "test_folder".to_string(),
            processing_time_ms: 10,
            metadata: HashMap::new(),
        };

        progress.update(&result);
        assert_eq!(progress.processed_items, 1);
        assert_eq!(progress.successful_count, 1);
    }

    #[tokio::test]
    async fn test_classification_cache() {
        let cache = ClassificationCache::new(10);

        let result = ClassificationResult {
            status: ClassificationStatus::Classified,
            category: Some("test".to_string()),
            candidates: vec![],
            score: 1.0,
            folder_name: "test_folder".to_string(),
            processing_time_ms: 10,
            metadata: HashMap::new(),
        };

        cache.put("test_key".to_string(), result.clone()).await;
        assert_eq!(cache.len().await, 1);

        let cached = cache.get("test_key").await;
        assert!(cached.is_some());
    }
}
