//! 配置验证器实现
//!
//! 提供工作流配置的验证和校验功能

use crate::core::WorkflowConfig;
use crate::error::{Result, WorkflowError};
use std::collections::HashMap;
use std::time::Duration;

/// 验证报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// 验证是否通过
    pub is_valid: bool,
    /// 错误列表
    pub errors: Vec<ValidationError>,
    /// 警告列表
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    /// 创建新的验证报告
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// 添加错误
    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
        });
        self.is_valid = false;
    }

    /// 添加警告
    pub fn add_warning(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.warnings.push(ValidationWarning {
            field: field.into(),
            message: message.into(),
        });
    }

    /// 合并另一个报告
    pub fn merge(&mut self, other: ValidationReport) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.is_valid {
            self.is_valid = false;
        }
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// 验证警告
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

/// 配置验证器
pub struct ConfigValidator;

impl ConfigValidator {
    /// 验证工作流配置
    pub fn validate(config: &WorkflowConfig) -> ValidationReport {
        let mut report = ValidationReport::new();

        // 验证超时设置
        Self::validate_timeout(config, &mut report);

        // 验证并发设置
        Self::validate_concurrency(config, &mut report);

        // 验证重试策略
        Self::validate_retry_policy(config, &mut report);

        // 验证检查点设置
        Self::validate_checkpoint(config, &mut report);

        report
    }

    /// 验证超时设置
    fn validate_timeout(config: &WorkflowConfig, report: &mut ValidationReport) {
        // 检查默认超时
        if let Some(timeout) = config.default_timeout {
            if timeout == 0 {
                report.add_error("default_timeout", "超时时间不能为0");
            } else if timeout > 3600 {
                report.add_warning("default_timeout", "超时时间超过1小时，可能影响系统性能");
            }
        }
    }

    /// 验证并发设置
    fn validate_concurrency(config: &WorkflowConfig, report: &mut ValidationReport) {
        // 检查最大并发步骤数
        if config.max_concurrent_steps == 0 {
            report.add_error("max_concurrent_steps", "并发数必须大于0");
        } else if config.max_concurrent_steps > 100 {
            report.add_warning(
                "max_concurrent_steps",
                "并发数超过100，可能导致资源耗尽",
            );
        }
    }

    /// 验证重试策略
    fn validate_retry_policy(config: &WorkflowConfig, report: &mut ValidationReport) {
        if let Some(ref retry) = config.retry_policy {
            // 检查最大重试次数
            if retry.max_attempts == 0 {
                report.add_warning("retry_policy.max_attempts", "重试次数为0，相当于禁用重试");
            } else if retry.max_attempts > 10 {
                report.add_warning(
                    "retry_policy.max_attempts",
                    "重试次数超过10，可能导致长时间等待",
                );
            }

            // 检查重试延迟
            if retry.base_delay == Duration::from_secs(0) {
                report.add_warning("retry_policy.base_delay", "重试延迟为0，可能导致频繁重试");
            } else if retry.base_delay > Duration::from_secs(60) {
                report.add_warning("retry_policy.base_delay", "重试延迟超过1分钟");
            }

            // 检查最大延迟
            if let Some(max_delay) = retry.max_delay {
                if max_delay < retry.base_delay {
                    report.add_error(
                        "retry_policy.max_delay",
                        "最大延迟不能小于基础延迟",
                    );
                }
                if max_delay > Duration::from_secs(3600) {
                    report.add_warning("retry_policy.max_delay", "最大延迟超过1小时");
                }
            }
        }
    }

    /// 验证检查点设置
    fn validate_checkpoint(config: &WorkflowConfig, report: &mut ValidationReport) {
        // 检查检查点间隔
        if let Some(interval) = config.checkpoint_interval {
            if interval == Duration::from_secs(0) {
                report.add_error("checkpoint_interval", "检查点间隔不能为0");
            } else if interval < Duration::from_secs(10) {
                report.add_warning(
                    "checkpoint_interval",
                    "检查点间隔小于10秒，可能影响性能",
                );
            } else if interval > Duration::from_secs(3600) {
                report.add_warning("checkpoint_interval", "检查点间隔超过1小时，容错能力较弱");
            }
        }
    }

    /// 验证单个值
    pub fn validate_value<T: Validatable>(value: &T, field_name: &str) -> ValidationReport {
        value.validate(field_name)
    }
}

/// 可验证 trait
pub trait Validatable {
    /// 验证自身
    fn validate(&self, field_name: &str) -> ValidationReport;
}

/// 配置加载器 trait
#[async_trait::async_trait]
pub trait ConfigLoader: Send + Sync {
    /// 加载配置
    async fn load(&self) -> Result<WorkflowConfig>;
    /// 重新加载配置
    async fn reload(&self) -> Result<WorkflowConfig>;
    /// 监听配置变化
    async fn watch(&self) -> Result<tokio::sync::mpsc::Receiver<ConfigChange>>;
}

/// 配置变化
#[derive(Debug, Clone)]
pub struct ConfigChange {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub change_type: ConfigChangeType,
    pub field: Option<String>,
}

/// 配置变化类型
#[derive(Debug, Clone)]
pub enum ConfigChangeType {
    Created,
    Modified,
    Deleted,
}

/// 文件配置加载器
pub struct FileConfigLoader {
    path: std::path::PathBuf,
}

impl FileConfigLoader {
    /// 创建新的文件配置加载器
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
        }
    }
}

#[async_trait::async_trait]
impl ConfigLoader for FileConfigLoader {
    async fn load(&self) -> Result<WorkflowConfig> {
        let content = tokio::fs::read_to_string(&self.path).await.map_err(|e| {
            WorkflowError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        let config: WorkflowConfig = serde_yaml::from_str(&content).map_err(|e| {
            WorkflowError::ConfigError(format!("Failed to parse config: {}", e))
        })?;

        // 验证配置
        let report = ConfigValidator::validate(&config);
        if !report.is_valid {
            let errors: Vec<String> = report
                .errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            return Err(WorkflowError::ConfigError(format!(
                "Config validation failed: {}",
                errors.join(", ")
            )));
        }

        Ok(config)
    }

    async fn reload(&self) -> Result<WorkflowConfig> {
        self.load().await
    }

    async fn watch(&self) -> Result<tokio::sync::mpsc::Receiver<ConfigChange>> {
        // 简化实现：返回一个空的 channel
        // 实际实现应该使用 notify crate 监听文件变化
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        let _ = tx; // 避免未使用警告
        Ok(rx)
    }
}

/// 热更新管理器
pub struct ConfigHotReloader {
    loader: Box<dyn ConfigLoader>,
    current_config: std::sync::Arc<tokio::sync::RwLock<WorkflowConfig>>,
    change_handlers: Vec<Box<dyn Fn(&ConfigChange) + Send + Sync>>,
}

impl ConfigHotReloader {
    /// 创建新的热更新管理器
    pub fn new(loader: Box<dyn ConfigLoader>, initial_config: WorkflowConfig) -> Self {
        Self {
            loader,
            current_config: std::sync::Arc::new(tokio::sync::RwLock::new(initial_config)),
            change_handlers: Vec::new(),
        }
    }

    /// 添加配置变化处理器
    pub fn on_change<F>(&mut self, handler: F)
    where
        F: Fn(&ConfigChange) + Send + Sync + 'static,
    {
        self.change_handlers.push(Box::new(handler));
    }

    /// 获取当前配置
    pub async fn current_config(&self) -> WorkflowConfig {
        self.current_config.read().await.clone()
    }

    /// 启动热更新监听
    pub async fn start(&self) -> Result<()> {
        let mut receiver = self.loader.watch().await?;

        while let Some(change) = receiver.recv().await {
            match self.loader.reload().await {
                Ok(new_config) => {
                    *self.current_config.write().await = new_config;
                    for handler in &self.change_handlers {
                        handler(&change);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to reload config: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::RetryPolicy;

    #[test]
    fn test_validate_timeout() {
        let mut config = WorkflowConfig::default();
        config.default_timeout = Some(0);

        let report = ConfigValidator::validate(&config);
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.field == "default_timeout"));
    }

    #[test]
    fn test_validate_concurrency() {
        let mut config = WorkflowConfig::default();
        config.max_concurrent_steps = 0;

        let report = ConfigValidator::validate(&config);
        assert!(!report.is_valid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.field == "max_concurrent_steps"));
    }

    #[test]
    fn test_validate_retry_policy() {
        let mut config = WorkflowConfig::default();
        config.retry_policy = Some(RetryPolicy {
            max_attempts: 0,
            base_delay: Duration::from_secs(1),
            max_delay: Some(Duration::from_secs(10)),
            retryable_errors: vec![],
        });

        let report = ConfigValidator::validate(&config);
        assert!(report.is_valid); // 警告不导致验证失败
        assert!(report
            .warnings
            .iter()
            .any(|w| w.field == "retry_policy.max_attempts"));
    }

    #[test]
    fn test_validate_checkpoint() {
        let mut config = WorkflowConfig::default();
        config.enable_checkpoint = true;
        config.checkpoint_interval = Duration::from_secs(0);

        let report = ConfigValidator::validate(&config);
        assert!(!report.is_valid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.field == "checkpoint_interval"));
    }

    #[test]
    fn test_validation_report_merge() {
        let mut report1 = ValidationReport::new();
        report1.add_error("field1", "error1");

        let mut report2 = ValidationReport::new();
        report2.add_warning("field2", "warning1");

        report1.merge(report2);

        assert_eq!(report1.errors.len(), 1);
        assert_eq!(report1.warnings.len(), 1);
    }
}
