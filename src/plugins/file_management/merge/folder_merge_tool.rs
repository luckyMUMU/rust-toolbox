//! 文件夹合并工具
//!
//! 基于参考实现 `mergeClassifierSimple.py` 和 `merge_set_folders.py` 的功能
//! 提供智能文件夹合并、冲突检测和批量处理能力

use crate::core::PluginInfo;
use crate::plugins::file_management::core::error::{FileManagementError, FileManagementResult};
use crate::plugins::file_management::utils::utils::{
    FileOperationManager, FolderMergerConfig, MergeStrategy,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Set文件夹信息 - 对应 Python 实现中的 SetFolderInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFolderInfo {
    /// 文件夹名称
    pub name: String,
    /// 完整路径
    pub path: PathBuf,
    /// 分类类别
    pub category: Option<String>,
    /// 文件数量
    pub file_count: usize,
    /// 总大小(字节)
    pub total_size: u64,
    /// 子文件夹数量
    pub subfolder_count: usize,
    /// 最后修改时间
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    /// 是否可写
    pub is_writable: bool,
    /// 分类置信度
    pub confidence: Option<f64>,
    /// 匹配的关键词
    pub matched_keywords: Vec<String>,
    /// 来源位置索引
    pub source_index: usize,
}

impl SetFolderInfo {
    /// 从路径创建 SetFolderInfo
    pub fn from_path(path: &std::path::Path, source_index: usize) -> FileManagementResult<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let metadata = std::fs::metadata(path).map_err(|e| {
            FileManagementError::io(format!("获取元数据失败: {}", path.display()), e)
        })?;

        let last_modified = metadata.modified().ok().and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|duration| {
                    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                })
        });

        let is_writable = Self::check_writable(path);

        let (file_count, subfolder_count, total_size) = Self::count_contents(path)?;

        Ok(Self {
            name,
            path: path.to_path_buf(),
            category: None,
            file_count,
            total_size,
            subfolder_count,
            last_modified,
            is_writable,
            confidence: None,
            matched_keywords: Vec::new(),
            source_index,
        })
    }

    fn check_writable(path: &std::path::Path) -> bool {
        let test_file = path.join(format!(".test_write_{}", uuid::Uuid::new_v4().simple()));
        match std::fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                true
            }
            Err(_) => false,
        }
    }

    fn count_contents(path: &std::path::Path) -> FileManagementResult<(usize, usize, u64)> {
        let mut file_count = 0;
        let mut subfolder_count = 0;
        let mut total_size = 0;

        let mut dir_stack = vec![path.to_path_buf()];
        const MAX_DEPTH: usize = 256;

        while let Some(current_path) = dir_stack.pop() {
            if dir_stack.len() > MAX_DEPTH {
                warn!("目录层级过深，跳过: {}", current_path.display());
                continue;
            }

            let entries = match std::fs::read_dir(&current_path) {
                Ok(e) => e,
                Err(e) => {
                    warn!("读取目录失败: {} - {}", current_path.display(), e);
                    continue;
                }
            };

            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        warn!("读取目录条目失败: {} - {}", current_path.display(), e);
                        continue;
                    }
                };

                let entry_path = entry.path();
                if entry_path.is_dir() {
                    subfolder_count += 1;
                    dir_stack.push(entry_path);
                } else {
                    file_count += 1;
                    total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }

        Ok((file_count, subfolder_count, total_size))
    }

    /// 设置分类信息
    pub fn with_classification(mut self, category: String, confidence: f64, keywords: Vec<String>) -> Self {
        self.category = Some(category);
        self.confidence = Some(confidence);
        self.matched_keywords = keywords;
        self
    }
}

/// 合并操作类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeOperationType {
    /// 移动到目标位置
    Move,
    /// 复制到目标位置
    Copy,
    /// 创建符号链接
    Symlink,
    /// 创建硬链接
    Hardlink,
    /// 跳过
    Skip,
}

/// 合并冲突类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeConflictType {
    /// 目标已存在
    TargetExists,
    /// 名称冲突
    NameConflict,
    /// 权限不足
    PermissionDenied,
    /// 空间不足
    InsufficientSpace,
    /// 分类冲突
    CategoryConflict,
    /// 内容冲突
    ContentConflict,
}

/// 合并冲突信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    /// 冲突类型
    pub conflict_type: MergeConflictType,
    /// 源路径
    pub source_path: PathBuf,
    /// 目标路径
    pub target_path: PathBuf,
    /// 冲突描述
    pub description: String,
    /// 建议的解决方案
    pub suggested_resolution: MergeOperationType,
    /// 冲突严重程度 (0-1)
    pub severity: f64,
}

/// 合并计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergePlan {
    /// 计划ID
    pub plan_id: String,
    /// 源文件夹列表
    pub source_folders: Vec<SetFolderInfo>,
    /// 目标位置
    pub target_location: PathBuf,
    /// 合并操作类型
    pub operation_type: MergeOperationType,
    /// 检测到的冲突
    pub conflicts: Vec<MergeConflict>,
    /// 预估的总大小
    pub estimated_total_size: u64,
    /// 预估的操作数量
    pub estimated_operations: usize,
    /// 预估节省的空间
    pub estimated_space_saved: u64,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 计划状态
    pub status: MergePlanStatus,
}

impl MergePlan {
    /// 创建新的合并计划
    pub fn new(source_folders: Vec<SetFolderInfo>, target_location: PathBuf) -> Self {
        let estimated_total_size: u64 = source_folders.iter().map(|f| f.total_size).sum();
        let estimated_operations: usize = source_folders.iter().map(|f| f.file_count).sum();

        Self {
            plan_id: uuid::Uuid::new_v4().to_string(),
            source_folders,
            target_location,
            operation_type: MergeOperationType::Move,
            conflicts: Vec::new(),
            estimated_total_size,
            estimated_operations,
            estimated_space_saved: 0,
            created_at: chrono::Utc::now(),
            status: MergePlanStatus::Pending,
        }
    }

    /// 添加冲突
    pub fn add_conflict(&mut self, conflict: MergeConflict) {
        self.conflicts.push(conflict);
    }

    /// 检查是否有严重冲突
    pub fn has_critical_conflicts(&self) -> bool {
        self.conflicts.iter().any(|c| c.severity > 0.7)
    }

    /// 获取冲突总数
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }
}

/// 合并计划状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergePlanStatus {
    /// 待审核
    Pending,
    /// 审核中
    Reviewing,
    /// 已批准
    Approved,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 失败
    Failed,
}

/// 合并执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeExecutionResult {
    /// 计划ID
    pub plan_id: String,
    /// 执行状态
    pub status: MergePlanStatus,
    /// 成功操作数
    pub successful_operations: usize,
    /// 失败操作数
    pub failed_operations: usize,
    /// 跳过操作数
    pub skipped_operations: usize,
    /// 总字节数
    pub total_bytes_transferred: u64,
    /// 执行时间(毫秒)
    pub execution_time_ms: u64,
    /// 错误信息
    pub errors: Vec<String>,
    /// 详细结果
    pub details: Vec<MergeOperationResult>,
}

/// 单个合并操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOperationResult {
    /// 源路径
    pub source_path: PathBuf,
    /// 目标路径
    pub target_path: PathBuf,
    /// 操作类型
    pub operation_type: MergeOperationType,
    /// 是否成功
    pub success: bool,
    /// 传输字节数
    pub bytes_transferred: u64,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间(毫秒)
    pub duration_ms: u64,
}

/// 文件夹合并工具参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMergeParams {
    /// 源目录列表
    pub source_directories: Vec<String>,
    /// 目标目录
    pub target_directory: String,
    /// 合并策略
    pub merge_strategy: MergeStrategy,
    /// 冲突解决策略
    pub conflict_resolution: ConflictResolution,
    /// 是否预览模式(不实际执行)
    pub dry_run: bool,
    /// 是否启用分类
    pub enable_classification: bool,
    /// 分类规则(可选)
    pub classification_rules: Option<Value>,
    /// 进度回调间隔(毫秒)
    pub progress_interval_ms: Option<u64>,
    /// 最大并发操作数
    pub max_concurrent_operations: Option<usize>,
}

impl FolderMergeParams {
    pub fn from_json(params: Value) -> FileManagementResult<Self> {
        serde_json::from_value(params)
            .map_err(|e| FileManagementError::validation(format!("解析合并参数失败: {}", e)))
    }
}

/// 冲突解决策略
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// 跳过冲突项
    Skip,
    /// 覆盖目标
    Overwrite,
    /// 重命名源文件
    RenameSource,
    /// 重命名目标文件
    RenameTarget,
    /// 保留较新的文件
    KeepNewer,
    /// 保留较大的文件
    KeepLarger,
    /// 合并内容(目录)
    Merge,
    /// 需要人工决策
    AskUser,
}

/// 文件夹合并工具
pub struct FolderMergeTool {
    /// 插件信息
    plugin_info: Option<PluginInfo>,
    /// 合并器配置
    merger_config: FolderMergerConfig,
    /// 是否启用中文支持
    enable_chinese: bool,
}

impl FolderMergeTool {
    /// 创建新的文件夹合并工具
    pub fn new(enable_chinese: bool) -> Self {
        Self {
            plugin_info: None,
            merger_config: FolderMergerConfig::default(),
            enable_chinese,
        }
    }

    /// 创建带配置的文件夹合并工具
    pub fn with_config(enable_chinese: bool, config: FolderMergerConfig) -> Self {
        Self {
            plugin_info: None,
            merger_config: config,
            enable_chinese,
        }
    }

    /// 设置插件信息
    pub fn with_plugin_info(mut self, plugin_info: PluginInfo) -> Self {
        self.plugin_info = Some(plugin_info);
        self
    }

    /// 扫描源目录获取 SetFolderInfo 列表
    fn scan_source_directories(
        &self,
        source_directories: &[String],
    ) -> FileManagementResult<Vec<SetFolderInfo>> {
        let mut folders = Vec::new();

        for (source_index, source_dir) in source_directories.iter().enumerate() {
            let source_path = std::path::Path::new(source_dir);
            if !source_path.exists() {
                warn!("源目录不存在: {}", source_dir);
                continue;
            }

            if !source_path.is_dir() {
                warn!("源路径不是目录: {}", source_dir);
                continue;
            }

            let entries = std::fs::read_dir(source_path).map_err(|e| {
                FileManagementError::io(format!("读取源目录失败: {}", source_dir), e)
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    FileManagementError::io(format!("读取目录条目失败: {}", source_dir), e)
                })?;

                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let folder_info = SetFolderInfo::from_path(&entry_path, source_index)?;
                    folders.push(folder_info);
                }
            }
        }

        debug!("扫描到 {} 个文件夹", folders.len());
        Ok(folders)
    }

    /// 分析文件夹并创建合并计划
    fn create_merge_plan(
        &self,
        source_folders: Vec<SetFolderInfo>,
        target_directory: &str,
        params: &FolderMergeParams,
    ) -> FileManagementResult<MergePlan> {
        let target_path = std::path::Path::new(target_directory);
        let mut plan = MergePlan::new(source_folders, target_path.to_path_buf());

        // 检测冲突
        self.detect_conflicts(&mut plan, target_path, params)?;

        // 计算预估节省的空间
        plan.estimated_space_saved = self.calculate_space_savings(&plan);

        info!(
            "创建合并计划: {} 个源文件夹, {} 个冲突, 预估节省 {} 字节",
            plan.source_folders.len(),
            plan.conflicts.len(),
            plan.estimated_space_saved
        );

        Ok(plan)
    }

    /// 检测合并冲突
    fn detect_conflicts(
        &self,
        plan: &mut MergePlan,
        target_path: &std::path::Path,
        _params: &FolderMergeParams,
    ) -> FileManagementResult<()> {
        let mut conflicts = Vec::new();

        for folder in &plan.source_folders {
            let target_folder = target_path.join(&folder.name);

            if target_folder.exists() {
                conflicts.push(MergeConflict {
                    conflict_type: MergeConflictType::TargetExists,
                    source_path: folder.path.clone(),
                    target_path: target_folder.clone(),
                    description: format!(
                        "目标位置已存在同名文件夹: {}",
                        target_folder.display()
                    ),
                    suggested_resolution: MergeOperationType::Skip,
                    severity: 0.5,
                });
            }

            // 检查写入权限
            if !folder.is_writable {
                conflicts.push(MergeConflict {
                    conflict_type: MergeConflictType::PermissionDenied,
                    source_path: folder.path.clone(),
                    target_path: target_folder.clone(),
                    description: format!("源文件夹不可写: {}", folder.path.display()),
                    suggested_resolution: MergeOperationType::Skip,
                    severity: 0.8,
                });
            }
        }

        // 检查名称冲突
        let mut name_count: HashMap<String, usize> = HashMap::new();
        for folder in &plan.source_folders {
            *name_count.entry(folder.name.clone()).or_insert(0) += 1;
        }

        for (name, count) in name_count {
            if count > 1 {
                for folder in plan.source_folders.iter().filter(|f| f.name == name) {
                    conflicts.push(MergeConflict {
                        conflict_type: MergeConflictType::NameConflict,
                        source_path: folder.path.clone(),
                        target_path: target_path.join(&name),
                        description: format!(
                            "存在 {} 个同名文件夹 '{}'",
                            count, name
                        ),
                        suggested_resolution: MergeOperationType::Skip,
                        severity: 0.6,
                    });
                }
            }
        }

        // 添加所有冲突到计划
        for conflict in conflicts {
            plan.add_conflict(conflict);
        }

        Ok(())
    }

    /// 计算预估节省的空间
    fn calculate_space_savings(&self, plan: &MergePlan) -> u64 {
        let mut unique_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut duplicate_size: u64 = 0;

        for folder in &plan.source_folders {
            if !unique_names.insert(folder.name.clone()) {
                duplicate_size += folder.total_size;
            }
        }

        duplicate_size
    }

    /// 执行合并计划
    async fn execute_merge_plan(
        &self,
        plan: &MergePlan,
        params: &FolderMergeParams,
    ) -> FileManagementResult<MergeExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut result = MergeExecutionResult {
            plan_id: plan.plan_id.clone(),
            status: MergePlanStatus::Executing,
            successful_operations: 0,
            failed_operations: 0,
            skipped_operations: 0,
            total_bytes_transferred: 0,
            execution_time_ms: 0,
            errors: Vec::new(),
            details: Vec::new(),
        };

        if params.dry_run {
            result.status = MergePlanStatus::Completed;
            result.execution_time_ms = start_time.elapsed().as_millis() as u64;
            info!("预览模式: 跳过实际执行");
            return Ok(result);
        }

        // 创建目标目录
        if !plan.target_location.exists() {
            std::fs::create_dir_all(&plan.target_location).map_err(|e| {
                FileManagementError::io(
                    format!("创建目标目录失败: {}", plan.target_location.display()),
                    e,
                )
            })?;
        }

        // 创建文件操作管理器
        let temp_dir = std::env::temp_dir();
        let file_op_manager = FileOperationManager::new(temp_dir, false);

        // 执行合并操作
        for folder in &plan.source_folders {
            let target_folder = plan.target_location.join(&folder.name);

            // 检查是否有冲突需要跳过
            let has_conflict = plan.conflicts.iter().any(|c| {
                c.source_path == folder.path && c.suggested_resolution == MergeOperationType::Skip
            });

            if has_conflict {
                result.skipped_operations += 1;
                result.details.push(MergeOperationResult {
                    source_path: folder.path.clone(),
                    target_path: target_folder.clone(),
                    operation_type: MergeOperationType::Skip,
                    success: true,
                    bytes_transferred: 0,
                    error: Some("跳过冲突项".to_string()),
                    duration_ms: 0,
                });
                continue;
            }

            let op_start = std::time::Instant::now();
            let op_result = match plan.operation_type {
                MergeOperationType::Move => {
                    file_op_manager.move_file(&folder.path, &target_folder).await
                }
                MergeOperationType::Copy => {
                    file_op_manager.copy_file(&folder.path, &target_folder).await
                }
                _ => {
                    result.skipped_operations += 1;
                    continue;
                }
            };

            match op_result {
                Ok(op_result) => {
                    result.successful_operations += 1;
                    result.total_bytes_transferred += op_result.bytes_moved;
                    result.details.push(MergeOperationResult {
                        source_path: folder.path.clone(),
                        target_path: target_folder.clone(),
                        operation_type: plan.operation_type.clone(),
                        success: true,
                        bytes_transferred: op_result.bytes_moved,
                        error: None,
                        duration_ms: op_start.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    result.failed_operations += 1;
                    result.errors.push(e.to_string());
                    result.details.push(MergeOperationResult {
                        source_path: folder.path.clone(),
                        target_path: target_folder.clone(),
                        operation_type: plan.operation_type.clone(),
                        success: false,
                        bytes_transferred: 0,
                        error: Some(e.to_string()),
                        duration_ms: op_start.elapsed().as_millis() as u64,
                    });
                }
            }
        }

        result.execution_time_ms = start_time.elapsed().as_millis() as u64;
        result.status = if result.failed_operations > 0 {
            MergePlanStatus::Failed
        } else {
            MergePlanStatus::Completed
        };

        info!(
            "合并执行完成: {} 成功, {} 失败, {} 跳过, 耗时 {} ms",
            result.successful_operations,
            result.failed_operations,
            result.skipped_operations,
            result.execution_time_ms
        );

        Ok(result)
    }

    /// 执行工具
    pub async fn execute(&self, params: Value) -> FileManagementResult<Value> {
        let merge_params = FolderMergeParams::from_json(params)?;

        info!(
            "开始文件夹合并: {} 个源目录 -> {}",
            merge_params.source_directories.len(),
            merge_params.target_directory
        );

        // 扫描源目录
        let source_folders = self.scan_source_directories(&merge_params.source_directories)?;

        if source_folders.is_empty() {
            return Ok(json!({
                "status": "completed",
                "message": "没有找到需要合并的文件夹",
                "source_folders": [],
                "merge_plan": null,
                "execution_result": null
            }));
        }

        // 创建合并计划
        let merge_plan = self.create_merge_plan(
            source_folders,
            &merge_params.target_directory,
            &merge_params,
        )?;

        // 执行合并
        let execution_result = self.execute_merge_plan(&merge_plan, &merge_params).await?;

        Ok(json!({
            "status": execution_result.status,
            "plan_id": merge_plan.plan_id,
            "source_folders": merge_plan.source_folders,
            "target_directory": merge_params.target_directory,
            "conflicts": merge_plan.conflicts,
            "estimated_space_saved": merge_plan.estimated_space_saved,
            "execution_result": {
                "successful_operations": execution_result.successful_operations,
                "failed_operations": execution_result.failed_operations,
                "skipped_operations": execution_result.skipped_operations,
                "total_bytes_transferred": execution_result.total_bytes_transferred,
                "execution_time_ms": execution_result.execution_time_ms,
                "errors": execution_result.errors,
                "details": execution_result.details
            }
        }))
    }

    /// 获取工具信息
    pub fn get_info(&self) -> PluginInfo {
        self.plugin_info.clone().unwrap_or_else(|| PluginInfo {
            name: "folder-merge".to_string(),
            version: "1.0.0".to_string(),
            description: Some("智能文件夹合并工具，支持冲突检测和批量处理".to_string()),
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
    use tempfile::TempDir;

    #[test]
    fn test_set_folder_info_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_folder = temp_dir.path().join("test_folder");
        std::fs::create_dir(&test_folder).unwrap();
        std::fs::write(test_folder.join("test.txt"), b"hello world").unwrap();

        let folder_info = SetFolderInfo::from_path(&test_folder, 0).unwrap();

        assert_eq!(folder_info.name, "test_folder");
        assert_eq!(folder_info.file_count, 1);
        assert!(folder_info.total_size > 0);
        assert_eq!(folder_info.source_index, 0);
    }

    #[test]
    fn test_merge_plan_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_folder = temp_dir.path().join("test_folder");
        std::fs::create_dir(&test_folder).unwrap();

        let folder_info = SetFolderInfo::from_path(&test_folder, 0).unwrap();
        let plan = MergePlan::new(vec![folder_info], temp_dir.path().to_path_buf());

        assert!(!plan.plan_id.is_empty());
        assert_eq!(plan.status, MergePlanStatus::Pending);
        assert_eq!(plan.source_folders.len(), 1);
    }

    #[test]
    fn test_merge_plan_conflicts() {
        let temp_dir = TempDir::new().unwrap();
        let test_folder = temp_dir.path().join("test_folder");
        std::fs::create_dir(&test_folder).unwrap();

        let folder_info = SetFolderInfo::from_path(&test_folder, 0).unwrap();
        let mut plan = MergePlan::new(vec![folder_info], temp_dir.path().to_path_buf());

        plan.add_conflict(MergeConflict {
            conflict_type: MergeConflictType::TargetExists,
            source_path: temp_dir.path().to_path_buf(),
            target_path: temp_dir.path().to_path_buf(),
            description: "Test conflict".to_string(),
            suggested_resolution: MergeOperationType::Skip,
            severity: 0.5,
        });

        assert_eq!(plan.conflict_count(), 1);
        assert!(!plan.has_critical_conflicts());
    }

    #[tokio::test]
    async fn test_folder_merge_tool_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");
        std::fs::create_dir(&source_dir).unwrap();

        let test_folder = source_dir.join("test_folder");
        std::fs::create_dir(&test_folder).unwrap();
        std::fs::write(test_folder.join("test.txt"), b"hello").unwrap();

        let tool = FolderMergeTool::new(true);

        let params = json!({
            "source_directories": [source_dir.to_string_lossy().to_string()],
            "target_directory": target_dir.to_string_lossy().to_string(),
            "merge_strategy": "SizeBased",
            "conflict_resolution": "Skip",
            "dry_run": true,
            "enable_classification": false
        });

        let result = tool.execute(params).await.unwrap();
        assert!(result.get("status").is_some());
        assert!(result.get("source_folders").is_some());
    }
}
