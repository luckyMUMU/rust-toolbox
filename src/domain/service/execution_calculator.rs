//! 执行状态计算领域服务
//!
//! 计算工作流和节点的执行状态

use crate::core::ExecutionStatus;
use crate::workflow::execution::NodeExecutionState;
use crate::workflow::state::ExecutionStats;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 执行状态计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStateResult {
    /// 工作流整体状态
    pub workflow_status: ExecutionStatus,
    /// 完成百分比
    pub completion_percentage: f64,
    /// 预计剩余时间
    pub estimated_remaining: Option<Duration>,
    /// 节点状态统计
    pub node_stats: NodeStateStats,
    /// 执行进度详情
    pub progress_details: ProgressDetails,
    /// 是否可恢复
    pub is_recoverable: bool,
    /// 阻塞原因
    pub blocked_reason: Option<String>,
}

/// 节点状态统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStateStats {
    /// 总节点数
    pub total: usize,
    /// 已完成
    pub completed: usize,
    /// 运行中
    pub running: usize,
    /// 等待中
    pub pending: usize,
    /// 失败
    pub failed: usize,
    /// 跳过
    pub skipped: usize,
}

impl NodeStateStats {
    /// 从执行统计创建
    pub fn from_stats(stats: &ExecutionStats) -> Self {
        Self {
            total: stats.total_nodes,
            completed: stats.completed_nodes,
            running: stats.running_nodes,
            pending: stats.pending_nodes,
            failed: stats.failed_nodes,
            skipped: 0,
        }
    }

    /// 计算完成百分比
    pub fn completion_percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }
}

/// 执行进度详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressDetails {
    /// 已执行节点列表
    pub executed_nodes: Vec<String>,
    /// 当前执行节点
    pub current_nodes: Vec<String>,
    /// 等待执行节点
    pub waiting_nodes: Vec<String>,
    /// 失败节点
    pub failed_nodes: Vec<String>,
    /// 执行时间线
    pub timeline: Vec<ExecutionTimelineEntry>,
}

/// 执行时间线条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimelineEntry {
    /// 节点 ID
    pub node_id: String,
    /// 状态变更
    pub status: ExecutionStatus,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 持续时间
    pub duration: Option<Duration>,
}

/// 执行状态计算器
pub struct ExecutionStateCalculator {
    /// 平均节点执行时间（用于估算）
    average_node_duration: Duration,
    /// 历史执行数据
    historical_durations: Vec<Duration>,
}

impl ExecutionStateCalculator {
    /// 创建新的计算器
    pub fn new() -> Self {
        Self {
            average_node_duration: Duration::from_millis(100),
            historical_durations: Vec::new(),
        }
    }

    /// 计算工作流执行状态
    pub fn calculate_state(
        &self,
        node_states: &HashMap<String, NodeExecutionState>,
    ) -> ExecutionStateResult {
        let stats = self.calculate_stats(node_states);
        let workflow_status = self.determine_workflow_status(node_states, &stats);
        let completion_percentage = stats.completion_percentage();
        let estimated_remaining = self.estimate_remaining_time(&stats);
        let progress_details = self.build_progress_details(node_states);
        let is_recoverable = self.is_recoverable(&workflow_status, &stats);
        let blocked_reason = self.find_blocked_reason(node_states);

        ExecutionStateResult {
            workflow_status,
            completion_percentage,
            estimated_remaining,
            node_stats: stats,
            progress_details,
            is_recoverable,
            blocked_reason,
        }
    }

    /// 计算节点统计
    fn calculate_stats(&self, node_states: &HashMap<String, NodeExecutionState>) -> NodeStateStats {
        let mut stats = NodeStateStats {
            total: node_states.len(),
            completed: 0,
            running: 0,
            pending: 0,
            failed: 0,
            skipped: 0,
        };

        for state in node_states.values() {
            match state.status {
                ExecutionStatus::Completed => stats.completed += 1,
                ExecutionStatus::Running => stats.running += 1,
                ExecutionStatus::Pending => stats.pending += 1,
                ExecutionStatus::Failed => stats.failed += 1,
                ExecutionStatus::Paused => stats.pending += 1,
                ExecutionStatus::Cancelled => stats.skipped += 1,
                ExecutionStatus::Timeout => stats.failed += 1,
            }
        }

        stats
    }

    /// 确定工作流状态
    fn determine_workflow_status(
        &self,
        node_states: &HashMap<String, NodeExecutionState>,
        stats: &NodeStateStats,
    ) -> ExecutionStatus {
        if stats.failed > 0 {
            if stats.completed + stats.failed == stats.total {
                return ExecutionStatus::Failed;
            }
            return ExecutionStatus::Running;
        }

        if stats.running > 0 {
            return ExecutionStatus::Running;
        }

        if stats.completed == stats.total {
            return ExecutionStatus::Completed;
        }

        if stats.pending == stats.total {
            return ExecutionStatus::Pending;
        }

        ExecutionStatus::Running
    }

    /// 估算剩余时间
    fn estimate_remaining_time(&self, stats: &NodeStateStats) -> Option<Duration> {
        if stats.pending == 0 && stats.running == 0 {
            return None;
        }

        let remaining_nodes = stats.pending + stats.running;
        let estimated = self.average_node_duration * remaining_nodes as u32;

        Some(estimated)
    }

    /// 构建进度详情
    fn build_progress_details(
        &self,
        node_states: &HashMap<String, NodeExecutionState>,
    ) -> ProgressDetails {
        let mut executed_nodes = Vec::new();
        let mut current_nodes = Vec::new();
        let mut waiting_nodes = Vec::new();
        let mut failed_nodes = Vec::new();
        let mut timeline = Vec::new();

        for (node_id, state) in node_states {
            match state.status {
                ExecutionStatus::Completed => executed_nodes.push(node_id.clone()),
                ExecutionStatus::Running => current_nodes.push(node_id.clone()),
                ExecutionStatus::Pending => waiting_nodes.push(node_id.clone()),
                ExecutionStatus::Failed => {
                    failed_nodes.push(node_id.clone());
                    executed_nodes.push(node_id.clone());
                }
                _ => {}
            }

            if let Some(started_at) = state.started_at {
                let duration = state
                    .completed_at
                    .map(|c| (c - started_at).to_std().unwrap_or(Duration::ZERO));

                timeline.push(ExecutionTimelineEntry {
                    node_id: node_id.clone(),
                    status: state.status,
                    timestamp: started_at,
                    duration,
                });
            }
        }

        timeline.sort_by_key(|e| e.timestamp);

        ProgressDetails {
            executed_nodes,
            current_nodes,
            waiting_nodes,
            failed_nodes,
            timeline,
        }
    }

    /// 检查是否可恢复
    fn is_recoverable(&self, status: &ExecutionStatus, stats: &NodeStateStats) -> bool {
        matches!(
            status,
            ExecutionStatus::Failed | ExecutionStatus::Paused | ExecutionStatus::Timeout
        ) && stats.completed < stats.total
    }

    /// 查找阻塞原因
    fn find_blocked_reason(
        &self,
        node_states: &HashMap<String, NodeExecutionState>,
    ) -> Option<String> {
        for (node_id, state) in node_states {
            if state.status == ExecutionStatus::Failed {
                if let Some(error) = &state.error {
                    return Some(format!("节点 {} 失败: {}", node_id, error));
                }
            }
        }

        for (node_id, state) in node_states {
            if state.status == ExecutionStatus::Pending {
                return Some(format!("等待节点 {} 执行", node_id));
            }
        }

        None
    }

    /// 更新平均执行时间
    pub fn record_duration(&mut self, duration: Duration) {
        self.historical_durations.push(duration);

        if self.historical_durations.len() > 100 {
            self.historical_durations.remove(0);
        }

        let total: Duration = self.historical_durations.iter().sum();
        self.average_node_duration = total / self.historical_durations.len() as u32;
    }

    /// 计算执行健康度
    pub fn calculate_health_score(&self, node_states: &HashMap<String, NodeExecutionState>) -> f64 {
        let stats = self.calculate_stats(node_states);

        if stats.total == 0 {
            return 100.0;
        }

        let success_rate = stats.completed as f64 / stats.total as f64;
        let failure_penalty = stats.failed as f64 * 10.0 / stats.total as f64;
        let timeout_penalty = node_states
            .values()
            .filter(|s| s.status == ExecutionStatus::Timeout)
            .count() as f64
            * 5.0
            / stats.total as f64;

        let score = success_rate * 100.0 - failure_penalty - timeout_penalty;
        score.max(0.0).min(100.0)
    }

    /// 获取瓶颈节点
    pub fn find_bottleneck_nodes(
        &self,
        node_states: &HashMap<String, NodeExecutionState>,
    ) -> Vec<String> {
        let mut bottlenecks = Vec::new();

        let avg_duration = self.calculate_average_duration(node_states);

        for (node_id, state) in node_states {
            if let (Some(started), Some(completed)) = (state.started_at, state.completed_at) {
                let duration = (completed - started).to_std().unwrap_or(Duration::ZERO);

                if duration > avg_duration * 2 {
                    bottlenecks.push(node_id.clone());
                }
            }
        }

        bottlenecks
    }

    /// 计算平均执行时间
    fn calculate_average_duration(
        &self,
        node_states: &HashMap<String, NodeExecutionState>,
    ) -> Duration {
        let mut total = Duration::ZERO;
        let mut count = 0;

        for state in node_states.values() {
            if let (Some(started), Some(completed)) = (state.started_at, state.completed_at) {
                let duration = (completed - started).to_std().unwrap_or(Duration::ZERO);
                total += duration;
                count += 1;
            }
        }

        if count > 0 {
            total / count
        } else {
            self.average_node_duration
        }
    }

    /// 获取平均节点执行时间
    pub fn average_node_duration(&self) -> Duration {
        self.average_node_duration
    }
}

impl Default for ExecutionStateCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_creation() {
        let calculator = ExecutionStateCalculator::new();
        assert!(calculator.average_node_duration() > Duration::ZERO);
    }

    #[test]
    fn test_calculate_empty_states() {
        let calculator = ExecutionStateCalculator::new();
        let states = HashMap::new();
        let result = calculator.calculate_state(&states);

        assert_eq!(result.workflow_status, ExecutionStatus::Completed);
        assert_eq!(result.completion_percentage, 100.0);
    }

    #[test]
    fn test_node_state_stats() {
        let stats = NodeStateStats {
            total: 10,
            completed: 5,
            running: 2,
            pending: 3,
            failed: 0,
            skipped: 0,
        };

        assert_eq!(stats.completion_percentage(), 50.0);
    }

    #[test]
    fn test_health_score() {
        let calculator = ExecutionStateCalculator::new();
        let mut states = HashMap::new();

        states.insert(
            "node1".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Completed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                result: None,
                error: None,
                retry_count: 0,
            },
        );
        states.insert(
            "node2".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Failed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                result: None,
                error: Some("error".to_string()),
                retry_count: 1,
            },
        );

        let score = calculator.calculate_health_score(&states);
        assert!(score < 100.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_record_duration() {
        let mut calculator = ExecutionStateCalculator::new();
        calculator.record_duration(Duration::from_millis(200));
        calculator.record_duration(Duration::from_millis(300));

        assert!(calculator.average_node_duration() >= Duration::from_millis(100));
    }
}
