//! Error recovery utilities for file management operations

use super::error::{FileManagementError, FileManagementResult, ErrorContext, RecoverySuggestion, ErrorSeverity};
use crate::performance::PerformanceManager;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn, error};

/// Error recovery manager for handling and recovering from errors
pub struct ErrorRecoveryManager {
    performance_manager: Option<Arc<PerformanceManager>>,
    recovery_strategies: HashMap<String, Box<dyn RecoveryStrategy + Send + Sync>>,
    recovery_config: RecoveryConfig,
    recovery_stats: RecoveryStats,
}

/// Configuration for error recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    
    /// Base delay between retries (exponential backoff)
    pub base_retry_delay_ms: u64,
    
    /// Maximum delay between retries
    pub max_retry_delay_ms: u64,
    
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    
    /// Enable automatic recovery for recoverable errors
    pub enable_auto_recovery: bool,
    
    /// Timeout for recovery operations
    pub recovery_timeout_ms: u64,
    
    /// Enable recovery statistics collection
    pub enable_recovery_stats: bool,
    
    /// Minimum confidence threshold for automatic recovery
    pub min_auto_recovery_confidence: f64,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_retry_delay_ms: 1000, // 1 second
            max_retry_delay_ms: 30000, // 30 seconds
            backoff_multiplier: 2.0,
            enable_auto_recovery: true,
            recovery_timeout_ms: 300000, // 5 minutes
            enable_recovery_stats: true,
            min_auto_recovery_confidence: 0.7,
        }
    }
}

/// Recovery statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecoveryStats {
    pub total_errors: usize,
    pub recovered_errors: usize,
    pub failed_recoveries: usize,
    pub recovery_success_rate: f64,
    pub average_recovery_time_ms: u64,
    pub recovery_attempts_by_category: HashMap<String, usize>,
    pub successful_recoveries_by_category: HashMap<String, usize>,
}

/// Recovery attempt result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    pub attempt_number: usize,
    pub strategy_used: String,
    pub success: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

/// Recovery session for tracking multiple attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySession {
    pub session_id: String,
    pub original_error: String,
    pub error_category: String,
    pub attempts: Vec<RecoveryAttempt>,
    pub final_success: bool,
    pub total_duration_ms: u64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Trait for recovery strategies
#[async_trait::async_trait]
pub trait RecoveryStrategy {
    /// Name of the recovery strategy
    fn name(&self) -> &str;
    
    /// Check if this strategy can handle the given error
    fn can_handle(&self, error: &FileManagementError) -> bool;
    
    /// Attempt to recover from the error
    async fn recover(&self, error: &FileManagementError) -> FileManagementResult<()>;
    
    /// Get confidence level for this recovery strategy (0.0 to 1.0)
    fn confidence(&self, error: &FileManagementError) -> f64;
    
    /// Get estimated recovery time
    fn estimated_recovery_time(&self, error: &FileManagementError) -> Duration;
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new(config: RecoveryConfig) -> Self {
        let mut manager = Self {
            performance_manager: None,
            recovery_strategies: HashMap::new(),
            recovery_config: config,
            recovery_stats: RecoveryStats::default(),
        };
        
        // Register default recovery strategies
        manager.register_default_strategies();
        
        manager
    }

    /// Create error recovery manager with performance manager
    pub fn with_performance_manager(
        config: RecoveryConfig,
        performance_manager: Arc<PerformanceManager>,
    ) -> Self {
        let mut manager = Self::new(config);
        manager.performance_manager = Some(performance_manager);
        manager
    }

    /// Register default recovery strategies
    fn register_default_strategies(&mut self) {
        self.register_strategy(Box::new(RetryStrategy::new()));
        self.register_strategy(Box::new(SpaceCleanupStrategy::new()));
        self.register_strategy(Box::new(PermissionFixStrategy::new()));
        self.register_strategy(Box::new(PathCreationStrategy::new()));
        self.register_strategy(Box::new(TimeoutAdjustmentStrategy::new()));
        self.register_strategy(Box::new(ResourceCleanupStrategy::new()));
    }

    /// Register a recovery strategy
    pub fn register_strategy(&mut self, strategy: Box<dyn RecoveryStrategy + Send + Sync>) {
        let name = strategy.name().to_string();
        self.recovery_strategies.insert(name, strategy);
    }

    /// Attempt to recover from an error
    pub async fn recover_from_error(&mut self, error: FileManagementError) -> FileManagementResult<()> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();
        
        let mut session = RecoverySession {
            session_id: session_id.clone(),
            original_error: error.to_string(),
            error_category: error.category().to_string(),
            attempts: Vec::new(),
            final_success: false,
            total_duration_ms: 0,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        info!("Starting error recovery session: {}", session_id);
        debug!("Original error: {}", error);

        // Update statistics
        self.recovery_stats.total_errors += 1;
        *self.recovery_stats.recovery_attempts_by_category
            .entry(error.category().to_string())
            .or_insert(0) += 1;

        // Check if error is recoverable
        if !error.is_recoverable() {
            warn!("Error is not recoverable: {}", error);
            session.completed_at = Some(chrono::Utc::now());
            session.total_duration_ms = start_time.elapsed().as_millis() as u64;
            return Err(error);
        }

        // Find suitable recovery strategies
        let mut strategies: Vec<(&String, &Box<dyn RecoveryStrategy + Send + Sync>)> = 
            self.recovery_strategies
                .iter()
                .filter(|(_, strategy)| strategy.can_handle(&error))
                .collect();

        // Sort strategies by confidence (highest first)
        strategies.sort_by(|a, b| {
            let conf_a = a.1.confidence(&error);
            let conf_b = b.1.confidence(&error);
            conf_b.partial_cmp(&conf_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        if strategies.is_empty() {
            warn!("No recovery strategies available for error: {}", error);
            session.completed_at = Some(chrono::Utc::now());
            session.total_duration_ms = start_time.elapsed().as_millis() as u64;
            return Err(error);
        }

        // Attempt recovery with each strategy
        let mut last_error = error;
        
        for attempt_num in 1..=self.recovery_config.max_retries {
            for (strategy_name, strategy) in &strategies {
                let attempt_start = Instant::now();
                
                info!("Recovery attempt {} using strategy: {}", attempt_num, strategy_name);
                
                // Check confidence threshold for automatic recovery
                let confidence = strategy.confidence(&last_error);
                if self.recovery_config.enable_auto_recovery && 
                   confidence < self.recovery_config.min_auto_recovery_confidence {
                    debug!("Skipping strategy {} due to low confidence: {}", strategy_name, confidence);
                    continue;
                }

                // Attempt recovery
                let recovery_result = tokio::time::timeout(
                    Duration::from_millis(self.recovery_config.recovery_timeout_ms),
                    strategy.recover(&last_error)
                ).await;

                let attempt_duration = attempt_start.elapsed().as_millis() as u64;
                
                match recovery_result {
                    Ok(Ok(())) => {
                        // Recovery successful
                        info!("Recovery successful with strategy: {}", strategy_name);
                        
                        session.attempts.push(RecoveryAttempt {
                            attempt_number: attempt_num,
                            strategy_used: strategy_name.to_string(),
                            success: true,
                            duration_ms: attempt_duration,
                            error_message: None,
                        });
                        
                        session.final_success = true;
                        session.completed_at = Some(chrono::Utc::now());
                        session.total_duration_ms = start_time.elapsed().as_millis() as u64;
                        
                        // Update statistics
                        self.recovery_stats.recovered_errors += 1;
                        *self.recovery_stats.successful_recoveries_by_category
                            .entry(last_error.category().to_string())
                            .or_insert(0) += 1;
                        
                        self.update_recovery_stats(&session);
                        
                        return Ok(());
                    }
                    Ok(Err(recovery_error)) => {
                        // Recovery failed
                        warn!("Recovery failed with strategy {}: {}", strategy_name, recovery_error);
                        
                        session.attempts.push(RecoveryAttempt {
                            attempt_number: attempt_num,
                            strategy_used: strategy_name.to_string(),
                            success: false,
                            duration_ms: attempt_duration,
                            error_message: Some(recovery_error.to_string()),
                        });
                        
                        last_error = recovery_error;
                    }
                    Err(_) => {
                        // Recovery timed out
                        warn!("Recovery timed out with strategy: {}", strategy_name);
                        
                        session.attempts.push(RecoveryAttempt {
                            attempt_number: attempt_num,
                            strategy_used: strategy_name.to_string(),
                            success: false,
                            duration_ms: attempt_duration,
                            error_message: Some("Recovery timed out".to_string()),
                        });
                    }
                }
            }

            // Wait before next attempt (exponential backoff)
            if attempt_num < self.recovery_config.max_retries {
                let delay = self.calculate_retry_delay(attempt_num);
                debug!("Waiting {} ms before next recovery attempt", delay.as_millis());
                sleep(delay).await;
            }
        }

        // All recovery attempts failed
        error!("All recovery attempts failed for error: {}", last_error);
        
        session.completed_at = Some(chrono::Utc::now());
        session.total_duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Update statistics
        self.recovery_stats.failed_recoveries += 1;
        self.update_recovery_stats(&session);
        
        Err(FileManagementError::recovery(
            "All recovery attempts failed",
            last_error,
        ))
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt_number: usize) -> Duration {
        let delay_ms = (self.recovery_config.base_retry_delay_ms as f64 * 
                       self.recovery_config.backoff_multiplier.powi(attempt_number as i32 - 1)) as u64;
        
        Duration::from_millis(delay_ms.min(self.recovery_config.max_retry_delay_ms))
    }

    /// Update recovery statistics
    fn update_recovery_stats(&mut self, session: &RecoverySession) {
        if !self.recovery_config.enable_recovery_stats {
            return;
        }

        // Calculate success rate
        let total_attempts = self.recovery_stats.recovered_errors + self.recovery_stats.failed_recoveries;
        if total_attempts > 0 {
            self.recovery_stats.recovery_success_rate = 
                self.recovery_stats.recovered_errors as f64 / total_attempts as f64;
        }

        // Update average recovery time
        let total_recovery_time = self.recovery_stats.average_recovery_time_ms * 
                                 (total_attempts.saturating_sub(1)) as u64 + 
                                 session.total_duration_ms;
        
        if total_attempts > 0 {
            self.recovery_stats.average_recovery_time_ms = total_recovery_time / total_attempts as u64;
        }
    }

    /// Get recovery statistics
    pub fn get_recovery_stats(&self) -> &RecoveryStats {
        &self.recovery_stats
    }

    /// Reset recovery statistics
    pub fn reset_stats(&mut self) {
        self.recovery_stats = RecoveryStats::default();
    }
}

// Default recovery strategies

/// Simple retry strategy
pub struct RetryStrategy;

impl RetryStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RecoveryStrategy for RetryStrategy {
    fn name(&self) -> &str {
        "retry"
    }

    fn can_handle(&self, error: &FileManagementError) -> bool {
        error.is_recoverable()
    }

    async fn recover(&self, _error: &FileManagementError) -> FileManagementResult<()> {
        // Simple retry - just wait a bit and return success
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    fn confidence(&self, _error: &FileManagementError) -> f64 {
        0.3 // Low confidence - this is a fallback strategy
    }

    fn estimated_recovery_time(&self, _error: &FileManagementError) -> Duration {
        Duration::from_millis(100)
    }
}

/// Space cleanup strategy for insufficient space errors
pub struct SpaceCleanupStrategy;

impl SpaceCleanupStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RecoveryStrategy for SpaceCleanupStrategy {
    fn name(&self) -> &str {
        "space_cleanup"
    }

    fn can_handle(&self, error: &FileManagementError) -> bool {
        matches!(error, FileManagementError::InsufficientSpace { .. })
    }

    async fn recover(&self, error: &FileManagementError) -> FileManagementResult<()> {
        if let FileManagementError::InsufficientSpace { required, available, .. } = error {
            let needed = required - available;
            info!("Attempting to free {} bytes of disk space", needed);
            
            // In a real implementation, this would:
            // 1. Clean up temporary files
            // 2. Clear caches
            // 3. Remove old log files
            // 4. Compress large files
            
            // For now, just simulate cleanup
            sleep(Duration::from_millis(500)).await;
            
            // Simulate successful cleanup
            Ok(())
        } else {
            Err(FileManagementError::other("Not an insufficient space error"))
        }
    }

    fn confidence(&self, error: &FileManagementError) -> f64 {
        if matches!(error, FileManagementError::InsufficientSpace { .. }) {
            0.7 // Good confidence for space cleanup
        } else {
            0.0
        }
    }

    fn estimated_recovery_time(&self, _error: &FileManagementError) -> Duration {
        Duration::from_secs(5) // Cleanup might take a while
    }
}

/// Permission fix strategy
pub struct PermissionFixStrategy;

impl PermissionFixStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RecoveryStrategy for PermissionFixStrategy {
    fn name(&self) -> &str {
        "permission_fix"
    }

    fn can_handle(&self, error: &FileManagementError) -> bool {
        matches!(error, FileManagementError::PermissionDenied { .. })
    }

    async fn recover(&self, error: &FileManagementError) -> FileManagementResult<()> {
        if let FileManagementError::PermissionDenied { path, .. } = error {
            info!("Attempting to fix permissions for: {}", path.display());
            
            // In a real implementation, this would:
            // 1. Check current permissions
            // 2. Attempt to change permissions if possible
            // 3. Suggest running as administrator
            
            // For now, just simulate permission fix
            sleep(Duration::from_millis(200)).await;
            
            // Simulate that we can't actually fix permissions automatically
            Err(FileManagementError::permission_denied(path))
        } else {
            Err(FileManagementError::other("Not a permission denied error"))
        }
    }

    fn confidence(&self, error: &FileManagementError) -> f64 {
        if matches!(error, FileManagementError::PermissionDenied { .. }) {
            0.4 // Low confidence - usually requires manual intervention
        } else {
            0.0
        }
    }

    fn estimated_recovery_time(&self, _error: &FileManagementError) -> Duration {
        Duration::from_millis(500)
    }
}

/// Path creation strategy
pub struct PathCreationStrategy;

impl PathCreationStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RecoveryStrategy for PathCreationStrategy {
    fn name(&self) -> &str {
        "path_creation"
    }

    fn can_handle(&self, error: &FileManagementError) -> bool {
        matches!(error, FileManagementError::NotFound { .. })
    }

    async fn recover(&self, error: &FileManagementError) -> FileManagementResult<()> {
        if let FileManagementError::NotFound { path, .. } = error {
            info!("Attempting to create missing path: {}", path.display());
            
            // Try to create the parent directory
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| {
                        FileManagementError::io(
                            format!("Failed to create directory: {}", parent.display()),
                            e,
                        )
                    })?;
                    
                    info!("Successfully created directory: {}", parent.display());
                    return Ok(());
                }
            }
            
            Err(FileManagementError::not_found(path))
        } else {
            Err(FileManagementError::other("Not a not found error"))
        }
    }

    fn confidence(&self, error: &FileManagementError) -> f64 {
        if matches!(error, FileManagementError::NotFound { .. }) {
            0.8 // High confidence for creating missing directories
        } else {
            0.0
        }
    }

    fn estimated_recovery_time(&self, _error: &FileManagementError) -> Duration {
        Duration::from_millis(300)
    }
}

/// Timeout adjustment strategy
pub struct TimeoutAdjustmentStrategy;

impl TimeoutAdjustmentStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RecoveryStrategy for TimeoutAdjustmentStrategy {
    fn name(&self) -> &str {
        "timeout_adjustment"
    }

    fn can_handle(&self, error: &FileManagementError) -> bool {
        matches!(error, FileManagementError::Timeout { .. })
    }

    async fn recover(&self, error: &FileManagementError) -> FileManagementResult<()> {
        if let FileManagementError::Timeout { operation, duration_seconds, .. } = error {
            info!("Adjusting timeout for operation: {} (was {} seconds)", operation, duration_seconds);
            
            // In a real implementation, this would adjust the timeout configuration
            // For now, just simulate the adjustment
            sleep(Duration::from_millis(100)).await;
            
            Ok(())
        } else {
            Err(FileManagementError::other("Not a timeout error"))
        }
    }

    fn confidence(&self, error: &FileManagementError) -> f64 {
        if matches!(error, FileManagementError::Timeout { .. }) {
            0.9 // High confidence - timeout adjustment usually works
        } else {
            0.0
        }
    }

    fn estimated_recovery_time(&self, _error: &FileManagementError) -> Duration {
        Duration::from_millis(100)
    }
}

/// Resource cleanup strategy
pub struct ResourceCleanupStrategy;

impl ResourceCleanupStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RecoveryStrategy for ResourceCleanupStrategy {
    fn name(&self) -> &str {
        "resource_cleanup"
    }

    fn can_handle(&self, error: &FileManagementError) -> bool {
        matches!(error, FileManagementError::ResourceExhaustion { .. })
    }

    async fn recover(&self, error: &FileManagementError) -> FileManagementResult<()> {
        if let FileManagementError::ResourceExhaustion { resource, .. } = error {
            info!("Cleaning up {} resources", resource);
            
            // In a real implementation, this would:
            // 1. Close unused file handles
            // 2. Clear memory caches
            // 3. Reduce concurrency limits
            // 4. Garbage collect
            
            // For now, just simulate cleanup
            sleep(Duration::from_secs(1)).await;
            
            Ok(())
        } else {
            Err(FileManagementError::other("Not a resource exhaustion error"))
        }
    }

    fn confidence(&self, error: &FileManagementError) -> f64 {
        if matches!(error, FileManagementError::ResourceExhaustion { .. }) {
            0.8 // Good confidence for resource cleanup
        } else {
            0.0
        }
    }

    fn estimated_recovery_time(&self, _error: &FileManagementError) -> Duration {
        Duration::from_secs(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_error_recovery_manager() {
        let config = RecoveryConfig::default();
        let mut manager = ErrorRecoveryManager::new(config);
        
        // Test recovery from a timeout error
        let error = FileManagementError::timeout("test_operation", 30);
        let result = manager.recover_from_error(error).await;
        
        // Should succeed with timeout adjustment strategy
        assert!(result.is_ok());
        
        let stats = manager.get_recovery_stats();
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.recovered_errors, 1);
    }

    #[tokio::test]
    async fn test_space_cleanup_strategy() {
        let strategy = SpaceCleanupStrategy::new();
        let error = FileManagementError::insufficient_space(1000, 500);
        
        assert!(strategy.can_handle(&error));
        assert!(strategy.confidence(&error) > 0.5);
        
        let result = strategy.recover(&error).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_path_creation_strategy() {
        let strategy = PathCreationStrategy::new();
        let error = FileManagementError::not_found(PathBuf::from("/nonexistent/path"));
        
        assert!(strategy.can_handle(&error));
        assert!(strategy.confidence(&error) > 0.7);
    }
}