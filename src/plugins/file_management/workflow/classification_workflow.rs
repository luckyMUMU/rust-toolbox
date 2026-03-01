//! 分类工作流定义
//!
//! 提供类似 `run-classify.bat` 的完整工作流编排能力
//! 包含三个主要步骤：
//! 1. Set 文件夹合并预处理
//! 2. 文件夹分类
//! 3. 多目录合并

use crate::error::{Result, WorkflowError};
use crate::plugins::file_management::classification::{
    ClassificationEngine, ClassificationRuleStorage, ClassificationRules, RuleStorageConfig,
};
use crate::plugins::file_management::merge::FolderMergeTool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// 分类工作流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationWorkflowConfig {
    /// 目标目录
    pub target_directory: PathBuf,
    /// 分类规则配置文件路径
    pub rules_config_path: Option<PathBuf>,
    /// 输出目录
    pub output_directory: PathBuf,
    /// 合并目录列表
    pub merge_directories: Vec<PathBuf>,
    /// 并行工作线程数
    pub parallel_workers: usize,
    /// IO 工作线程数
    pub io_workers: usize,
    /// 是否启用实验模式（只预览不执行）
    pub experimental_mode: bool,
    /// 是否禁用用户交互
    pub no_interaction: bool,
    /// 输出模式
    pub output_mode: OutputMode,
    /// 日志文件路径
    pub log_file: Option<PathBuf>,
}

impl Default for ClassificationWorkflowConfig {
    fn default() -> Self {
        Self {
            target_directory: PathBuf::from("."),
            rules_config_path: None,
            output_directory: PathBuf::from("./output"),
            merge_directories: Vec::new(),
            parallel_workers: 4,
            io_workers: 6,
            experimental_mode: false,
            no_interaction: false,
            output_mode: OutputMode::Console,
            log_file: None,
        }
    }
}

/// 输出模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputMode {
    /// 仅控制台输出
    Console,
    /// 仅日志文件输出
    Log,
    /// 同时输出到控制台和日志
    Both,
}

/// 工作流步骤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 跳过
    Skipped,
}

/// 工作流步骤结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 步骤名称
    pub step_name: String,
    /// 执行状态
    pub status: StepStatus,
    /// 开始时间
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// 结束时间
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 执行结果数据
    pub result_data: Option<Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
}

impl StepResult {
    pub fn new(step_name: &str) -> Self {
        Self {
            step_name: step_name.to_string(),
            status: StepStatus::Running,
            started_at: chrono::Utc::now(),
            completed_at: None,
            result_data: None,
            error: None,
            duration_ms: 0,
        }
    }

    pub fn complete(mut self, result_data: Option<Value>) -> Self {
        self.status = StepStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
        self.duration_ms = (self.completed_at.unwrap() - self.started_at)
            .num_milliseconds()
            .max(0) as u64;
        self.result_data = result_data;
        self
    }

    pub fn fail(mut self, error: String) -> Self {
        self.status = StepStatus::Failed;
        self.completed_at = Some(chrono::Utc::now());
        self.duration_ms = (self.completed_at.unwrap() - self.started_at)
            .num_milliseconds()
            .max(0) as u64;
        self.error = Some(error);
        self
    }

    pub fn skip(mut self, reason: &str) -> Self {
        self.status = StepStatus::Skipped;
        self.completed_at = Some(chrono::Utc::now());
        self.duration_ms = 0;
        self.result_data = Some(json!({ "reason": reason }));
        self
    }
}

/// 工作流执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult {
    /// 工作流ID
    pub workflow_id: String,
    /// 配置摘要
    pub config_summary: Value,
    /// 各步骤结果
    pub step_results: Vec<StepResult>,
    /// 总执行时间（毫秒）
    pub total_duration_ms: u64,
    /// 是否成功
    pub success: bool,
    /// 失败步骤（如果有）
    pub failed_step: Option<String>,
}

/// 分类工作流执行器
pub struct ClassificationWorkflow {
    /// 工作流配置
    config: ClassificationWorkflowConfig,
    /// 规则存储服务
    rule_storage: Arc<ClassificationRuleStorage>,
    /// 工作流ID
    workflow_id: String,
    /// 步骤结果
    step_results: Arc<RwLock<Vec<StepResult>>>,
}

impl ClassificationWorkflow {
    /// 创建新的分类工作流
    pub fn new(config: ClassificationWorkflowConfig) -> Self {
        let storage_config = RuleStorageConfig {
            storage_dir: config.output_directory.join("rules"),
            ..Default::default()
        };

        Self {
            config,
            rule_storage: Arc::new(ClassificationRuleStorage::new(storage_config)),
            workflow_id: uuid::Uuid::new_v4().to_string(),
            step_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 使用自定义规则存储创建工作流
    pub fn with_rule_storage(
        config: ClassificationWorkflowConfig,
        rule_storage: Arc<ClassificationRuleStorage>,
    ) -> Self {
        Self {
            config,
            rule_storage,
            workflow_id: uuid::Uuid::new_v4().to_string(),
            step_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 执行完整工作流
    pub async fn execute(&self) -> Result<WorkflowExecutionResult> {
        let start_time = std::time::Instant::now();
        info!("开始执行分类工作流: {}", self.workflow_id);

        let mut results: Vec<StepResult> = Vec::new();

        // 步骤1: Set 文件夹合并预处理
        let step1_result = self.execute_set_folder_merge().await;
        results.push(step1_result.clone());

        if step1_result.status == StepStatus::Failed {
            return Ok(self.create_result(results.clone(), start_time, false));
        }

        // 步骤2: 文件夹分类
        let step2_result = self.execute_classification().await;
        results.push(step2_result.clone());

        if step2_result.status == StepStatus::Failed {
            return Ok(self.create_result(results.clone(), start_time, false));
        }

        // 步骤3: 多目录合并
        let step3_result = self.execute_multi_folder_merge().await;
        results.push(step3_result.clone());

        // 保存结果到内部状态
        let mut step_results = self.step_results.write().await;
        *step_results = results.clone();

        Ok(self.create_result(results, start_time, step3_result.status == StepStatus::Completed))
    }

    /// 步骤1: Set 文件夹合并预处理
    async fn execute_set_folder_merge(&self) -> StepResult {
        let mut result = StepResult::new("Set文件夹合并预处理");

        info!("执行 Set 文件夹合并预处理: {}", self.config.target_directory.display());

        // 检查目标目录是否存在
        if !self.config.target_directory.exists() {
            return result.skip("目标目录不存在");
        }

        // 创建合并工具
        let merge_tool = FolderMergeTool::new(true);

        // 构建参数
        let params = json!({
            "source_directories": [self.config.target_directory.to_string_lossy()],
            "target_directory": self.config.target_directory.to_string_lossy(),
            "merge_strategy": "SizeBased",
            "conflict_resolution": "Skip",
            "dry_run": self.config.experimental_mode,
            "enable_classification": false
        });

        match merge_tool.execute(params).await {
            Ok(output) => result.complete(Some(output)),
            Err(e) => result.fail(e.to_string()),
        }
    }

    /// 步骤2: 文件夹分类
    async fn execute_classification(&self) -> StepResult {
        let mut result = StepResult::new("文件夹分类");

        info!("执行文件夹分类: {}", self.config.target_directory.display());

        // 加载分类规则
        let rules = if let Some(ref rules_path) = self.config.rules_config_path {
            match self.rule_storage.import_from_classfy_json(rules_path).await {
                Ok(r) => r,
                Err(e) => return result.fail(format!("加载分类规则失败: {}", e)),
            }
        } else {
            // 使用默认规则
            ClassificationRules::default()
        };

        // 扫描目标目录
        let folders = match self.scan_target_directory().await {
            Ok(f) => f,
            Err(e) => return result.fail(format!("扫描目标目录失败: {}", e)),
        };

        if folders.is_empty() {
            return result.skip("没有找到需要分类的文件夹");
        }

        // 创建分类引擎
        let engine = ClassificationEngine::new(true);

        // 构建 AC 自动机
        let automaton = match engine.build_automaton(&rules) {
            Ok(a) => a,
            Err(e) => return result.fail(format!("构建分类自动机失败: {}", e)),
        };

        // 执行分类
        let mut classified_count = 0;
        let mut unclassified_count = 0;
        let mut ambiguous_count = 0;
        let mut classification_results = Vec::new();

        for folder_path in &folders {
            let path = std::path::Path::new(folder_path);
            let folder_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            match engine.classify_folder(folder_name, &automaton, &rules) {
                Ok(classify_result) => {
                    let status = classify_result.status;
                    match status {
                        crate::plugins::file_management::classification::ClassificationStatus::Classified => {
                            classified_count += 1;
                        }
                        crate::plugins::file_management::classification::ClassificationStatus::Ambiguous => {
                            ambiguous_count += 1;
                        }
                        _ => {
                            unclassified_count += 1;
                        }
                    }

                    classification_results.push(json!({
                        "folder": folder_path,
                        "folder_name": folder_name,
                        "category": classify_result.category,
                        "status": status,
                        "score": classify_result.score,
                        "confidence": classify_result.candidates.first()
                            .map(|c| c.confidence).unwrap_or(0.0),
                        "matched_keywords": classify_result.candidates.first()
                            .map(|c| c.matched_keywords.clone()).unwrap_or_default()
                    }));
                }
                Err(e) => {
                    unclassified_count += 1;
                    classification_results.push(json!({
                        "folder": folder_path,
                        "folder_name": folder_name,
                        "category": null,
                        "status": "error",
                        "error": e.to_string()
                    }));
                }
            }
        }

        // 分类结果
        let output = json!({
            "total_folders": folders.len(),
            "classified_count": classified_count,
            "unclassified_count": unclassified_count,
            "ambiguous_count": ambiguous_count,
            "rules_applied": rules.rules.len(),
            "output_directory": self.config.output_directory.to_string_lossy(),
            "experimental_mode": self.config.experimental_mode,
            "results": classification_results
        });

        result.complete(Some(output))
    }

    /// 扫描目标目录
    async fn scan_target_directory(&self) -> Result<Vec<String>> {
        let mut folders = Vec::new();

        let mut entries = tokio::fs::read_dir(&self.config.target_directory)
            .await
            .map_err(|e| {
                WorkflowError::execution(format!(
                    "读取目标目录失败: {}",
                    self.config.target_directory.display()
                ))
            })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            WorkflowError::execution("读取目录条目失败")
        })? {
            let path = entry.path();
            if path.is_dir() {
                folders.push(path.to_string_lossy().to_string());
            }
        }

        Ok(folders)
    }

    /// 步骤3: 多目录合并
    async fn execute_multi_folder_merge(&self) -> StepResult {
        let mut result = StepResult::new("多目录合并");

        if self.config.merge_directories.len() < 2 {
            return result.skip("需要至少两个合并目录");
        }

        info!(
            "执行多目录合并: {} 个目录",
            self.config.merge_directories.len()
        );

        // 创建合并工具
        let merge_tool = FolderMergeTool::new(true);

        // 构建参数
        let source_dirs: Vec<String> = self
            .config
            .merge_directories
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let params = json!({
            "source_directories": source_dirs,
            "target_directory": self.config.output_directory.to_string_lossy(),
            "merge_strategy": "SizeBased",
            "conflict_resolution": "Skip",
            "dry_run": self.config.experimental_mode,
            "enable_classification": true
        });

        match merge_tool.execute(params).await {
            Ok(output) => result.complete(Some(output)),
            Err(e) => result.fail(e.to_string()),
        }
    }

    /// 创建工作流结果
    fn create_result(
        &self,
        step_results: Vec<StepResult>,
        start_time: std::time::Instant,
        success: bool,
    ) -> WorkflowExecutionResult {
        let failed_step = step_results
            .iter()
            .find(|r| r.status == StepStatus::Failed)
            .map(|r| r.step_name.clone());

        WorkflowExecutionResult {
            workflow_id: self.workflow_id.clone(),
            config_summary: json!({
                "target_directory": self.config.target_directory.to_string_lossy(),
                "output_directory": self.config.output_directory.to_string_lossy(),
                "merge_directories": self.config.merge_directories.iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>(),
                "experimental_mode": self.config.experimental_mode,
                "parallel_workers": self.config.parallel_workers,
            }),
            step_results,
            total_duration_ms: start_time.elapsed().as_millis() as u64,
            success,
            failed_step,
        }
    }

    /// 获取工作流ID
    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }

    /// 获取配置
    pub fn config(&self) -> &ClassificationWorkflowConfig {
        &self.config
    }
}

/// 工作流构建器
pub struct ClassificationWorkflowBuilder {
    config: ClassificationWorkflowConfig,
    rule_storage: Option<Arc<ClassificationRuleStorage>>,
}

impl ClassificationWorkflowBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: ClassificationWorkflowConfig::default(),
            rule_storage: None,
        }
    }

    /// 设置目标目录
    pub fn target_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.target_directory = path.into();
        self
    }

    /// 设置规则配置文件路径
    pub fn rules_config_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.rules_config_path = Some(path.into());
        self
    }

    /// 设置输出目录
    pub fn output_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.output_directory = path.into();
        self
    }

    /// 添加合并目录
    pub fn add_merge_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.merge_directories.push(path.into());
        self
    }

    /// 设置合并目录列表
    pub fn merge_directories(mut self, dirs: Vec<PathBuf>) -> Self {
        self.config.merge_directories = dirs;
        self
    }

    /// 设置并行工作线程数
    pub fn parallel_workers(mut self, count: usize) -> Self {
        self.config.parallel_workers = count;
        self
    }

    /// 设置 IO 工作线程数
    pub fn io_workers(mut self, count: usize) -> Self {
        self.config.io_workers = count;
        self
    }

    /// 设置实验模式
    pub fn experimental_mode(mut self, enabled: bool) -> Self {
        self.config.experimental_mode = enabled;
        self
    }

    /// 设置禁用用户交互
    pub fn no_interaction(mut self, enabled: bool) -> Self {
        self.config.no_interaction = enabled;
        self
    }

    /// 设置输出模式
    pub fn output_mode(mut self, mode: OutputMode) -> Self {
        self.config.output_mode = mode;
        self
    }

    /// 设置日志文件路径
    pub fn log_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.log_file = Some(path.into());
        self
    }

    /// 设置规则存储服务
    pub fn rule_storage(mut self, storage: Arc<ClassificationRuleStorage>) -> Self {
        self.rule_storage = Some(storage);
        self
    }

    /// 构建工作流
    pub fn build(self) -> ClassificationWorkflow {
        if let Some(storage) = self.rule_storage {
            ClassificationWorkflow::with_rule_storage(self.config, storage)
        } else {
            ClassificationWorkflow::new(self.config)
        }
    }
}

impl Default for ClassificationWorkflowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workflow_builder() {
        let temp_dir = TempDir::new().unwrap();

        let workflow = ClassificationWorkflowBuilder::new()
            .target_directory(temp_dir.path())
            .output_directory(temp_dir.path().join("output"))
            .parallel_workers(8)
            .experimental_mode(true)
            .build();

        assert_eq!(workflow.config().parallel_workers, 8);
        assert!(workflow.config().experimental_mode);
    }

    #[test]
    fn test_step_result() {
        let result = StepResult::new("test_step");
        assert_eq!(result.step_name, "test_step");
        assert_eq!(result.status, StepStatus::Running);

        let completed = result.complete(Some(json!({"key": "value"})));
        assert_eq!(completed.status, StepStatus::Completed);
        assert!(completed.result_data.is_some());
    }

    #[test]
    fn test_step_result_failure() {
        let result = StepResult::new("test_step");
        let failed = result.fail("Something went wrong".to_string());

        assert_eq!(failed.status, StepStatus::Failed);
        assert_eq!(failed.error, Some("Something went wrong".to_string()));
    }

    #[tokio::test]
    async fn test_workflow_execution_skip() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("non_existent");

        let workflow = ClassificationWorkflowBuilder::new()
            .target_directory(non_existent)
            .output_directory(temp_dir.path().join("output"))
            .build();

        let result = workflow.execute().await.unwrap();

        // 第一个步骤应该被跳过，因为目标目录不存在
        assert!(result.step_results[0].status == StepStatus::Skipped);
    }
}
