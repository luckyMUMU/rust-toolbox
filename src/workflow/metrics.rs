//! 指标收集实现
//!
//! 提供工作流执行的指标收集和报告功能

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// 指标收集器接口
#[async_trait::async_trait]
pub trait MetricsCollector: Send + Sync {
    /// 记录工作流执行时间
    fn record_workflow_duration(&self, workflow_id: &str, duration: Duration);

    /// 记录工作流执行结果
    fn record_workflow_result(&self, workflow_id: &str, success: bool);

    /// 记录节点执行时间
    fn record_node_duration(&self, node_type: &str, duration: Duration);

    /// 记录队列深度
    fn record_queue_depth(&self, depth: usize);

    /// 记录并发执行数
    fn record_concurrent_executions(&self, count: usize);

    /// 记录熔断器状态变化
    fn record_circuit_breaker_state(&self, name: &str, from: &str, to: &str);

    /// 获取指标快照
    async fn get_snapshot(&self) -> MetricsSnapshot;
}

/// 指标快照
#[derive(Clone, Debug, Default)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub workflow_durations: HashMap<String, Vec<Duration>>,
    pub workflow_success_count: u64,
    pub workflow_failure_count: u64,
    pub node_durations: HashMap<String, Vec<Duration>>,
    pub current_queue_depth: usize,
    pub current_concurrent_executions: usize,
    pub circuit_breaker_state_changes: Vec<CircuitBreakerStateChange>,
}

/// 熔断器状态变化记录
#[derive(Clone, Debug)]
pub struct CircuitBreakerStateChange {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub from: String,
    pub to: String,
}

/// 内存指标收集器
pub struct InMemoryMetricsCollector {
    workflow_durations: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    workflow_success_count: AtomicU64,
    workflow_failure_count: AtomicU64,
    node_durations: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    current_queue_depth: AtomicU64,
    current_concurrent_executions: AtomicU64,
    circuit_breaker_changes: Arc<RwLock<Vec<CircuitBreakerStateChange>>>,
}

impl InMemoryMetricsCollector {
    pub fn new() -> Self {
        Self {
            workflow_durations: Arc::new(RwLock::new(HashMap::new())),
            workflow_success_count: AtomicU64::new(0),
            workflow_failure_count: AtomicU64::new(0),
            node_durations: Arc::new(RwLock::new(HashMap::new())),
            current_queue_depth: AtomicU64::new(0),
            current_concurrent_executions: AtomicU64::new(0),
            circuit_breaker_changes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 获取工作流执行统计
    pub async fn get_workflow_stats(&self, workflow_id: &str) -> Option<WorkflowStats> {
        let durations = self.workflow_durations.read().await;
        durations.get(workflow_id).map(|d| {
            let total: Duration = d.iter().sum();
            let count = d.len() as u64;
            WorkflowStats {
                total_executions: count,
                average_duration: if count > 0 { total / count as u32 } else { Duration::from_secs(0) },
                min_duration: d.iter().min().copied().unwrap_or(Duration::from_secs(0)),
                max_duration: d.iter().max().copied().unwrap_or(Duration::from_secs(0)),
            }
        })
    }

    /// 获取成功率
    pub fn get_success_rate(&self) -> f64 {
        let success = self.workflow_success_count.load(Ordering::Relaxed);
        let failure = self.workflow_failure_count.load(Ordering::Relaxed);
        let total = success + failure;

        if total == 0 {
            0.0
        } else {
            success as f64 / total as f64
        }
    }
}

impl Default for InMemoryMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MetricsCollector for InMemoryMetricsCollector {
    fn record_workflow_duration(&self, workflow_id: &str, duration: Duration) {
        trace!(
            "Recording workflow duration for '{}': {:?}",
            workflow_id,
            duration
        );

        let durations = self.workflow_durations.clone();
        let workflow_id = workflow_id.to_string();

        tokio::spawn(async move {
            let mut map = durations.write().await;
            map.entry(workflow_id)
                .or_insert_with(Vec::new)
                .push(duration);
        });
    }

    fn record_workflow_result(&self, _workflow_id: &str, success: bool) {
        if success {
            self.workflow_success_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.workflow_failure_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn record_node_duration(&self, node_type: &str, duration: Duration) {
        trace!("Recording node duration for '{}': {:?}", node_type, duration);

        let durations = self.node_durations.clone();
        let node_type = node_type.to_string();

        tokio::spawn(async move {
            let mut map = durations.write().await;
            map.entry(node_type)
                .or_insert_with(Vec::new)
                .push(duration);
        });
    }

    fn record_queue_depth(&self, depth: usize) {
        self.current_queue_depth
            .store(depth as u64, Ordering::Relaxed);
    }

    fn record_concurrent_executions(&self, count: usize) {
        self.current_concurrent_executions
            .store(count as u64, Ordering::Relaxed);
    }

    fn record_circuit_breaker_state(&self, name: &str, from: &str, to: &str) {
        debug!(
            "Recording circuit breaker state change: {} from {} to {}",
            name, from, to
        );

        let changes = self.circuit_breaker_changes.clone();
        let change = CircuitBreakerStateChange {
            timestamp: Utc::now(),
            name: name.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        };

        tokio::spawn(async move {
            let mut list = changes.write().await;
            list.push(change);
        });
    }

    async fn get_snapshot(&self) -> MetricsSnapshot {
        let workflow_durations = self.workflow_durations.read().await.clone();
        let node_durations = self.node_durations.read().await.clone();
        let circuit_breaker_changes = self.circuit_breaker_changes.read().await.clone();

        MetricsSnapshot {
            timestamp: Utc::now(),
            workflow_durations,
            workflow_success_count: self.workflow_success_count.load(Ordering::Relaxed),
            workflow_failure_count: self.workflow_failure_count.load(Ordering::Relaxed),
            node_durations,
            current_queue_depth: self.current_queue_depth.load(Ordering::Relaxed) as usize,
            current_concurrent_executions: self
                .current_concurrent_executions
                .load(Ordering::Relaxed) as usize,
            circuit_breaker_state_changes: circuit_breaker_changes,
        }
    }
}

/// 工作流执行统计
#[derive(Clone, Debug)]
pub struct WorkflowStats {
    pub total_executions: u64,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

/// Prometheus 指标收集器（预留接口）
pub struct PrometheusMetricsCollector {
    // 这里可以集成 prometheus crate
    // 例如：workflow_duration: HistogramVec,
}

impl PrometheusMetricsCollector {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PrometheusMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MetricsCollector for PrometheusMetricsCollector {
    fn record_workflow_duration(&self, workflow_id: &str, duration: Duration) {
        // 集成 prometheus 的 histogram 记录
        trace!(
            "[Prometheus] Recording workflow duration for '{}': {:?}",
            workflow_id,
            duration
        );
    }

    fn record_workflow_result(&self, workflow_id: &str, success: bool) {
        trace!(
            "[Prometheus] Recording workflow result for '{}': success={}",
            workflow_id,
            success
        );
    }

    fn record_node_duration(&self, node_type: &str, duration: Duration) {
        trace!(
            "[Prometheus] Recording node duration for '{}': {:?}",
            node_type,
            duration
        );
    }

    fn record_queue_depth(&self, depth: usize) {
        trace!("[Prometheus] Recording queue depth: {}", depth);
    }

    fn record_concurrent_executions(&self, count: usize) {
        trace!("[Prometheus] Recording concurrent executions: {}", count);
    }

    fn record_circuit_breaker_state(&self, name: &str, from: &str, to: &str) {
        trace!(
            "[Prometheus] Recording circuit breaker state change: {} from {} to {}",
            name,
            from,
            to
        );
    }

    async fn get_snapshot(&self) -> MetricsSnapshot {
        // Prometheus 通常通过 /metrics 端点暴露，这里返回空快照
        MetricsSnapshot::default()
    }
}

/// 组合指标收集器
pub struct CompositeMetricsCollector {
    collectors: Vec<Box<dyn MetricsCollector>>,
}

impl CompositeMetricsCollector {
    pub fn new() -> Self {
        Self {
            collectors: Vec::new(),
        }
    }

    pub fn add_collector(&mut self, collector: Box<dyn MetricsCollector>) {
        self.collectors.push(collector);
    }
}

impl Default for CompositeMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MetricsCollector for CompositeMetricsCollector {
    fn record_workflow_duration(&self, workflow_id: &str, duration: Duration) {
        for collector in &self.collectors {
            collector.record_workflow_duration(workflow_id, duration);
        }
    }

    fn record_workflow_result(&self, workflow_id: &str, success: bool) {
        for collector in &self.collectors {
            collector.record_workflow_result(workflow_id, success);
        }
    }

    fn record_node_duration(&self, node_type: &str, duration: Duration) {
        for collector in &self.collectors {
            collector.record_node_duration(node_type, duration);
        }
    }

    fn record_queue_depth(&self, depth: usize) {
        for collector in &self.collectors {
            collector.record_queue_depth(depth);
        }
    }

    fn record_concurrent_executions(&self, count: usize) {
        for collector in &self.collectors {
            collector.record_concurrent_executions(count);
        }
    }

    fn record_circuit_breaker_state(&self, name: &str, from: &str, to: &str) {
        for collector in &self.collectors {
            collector.record_circuit_breaker_state(name, from, to);
        }
    }

    async fn get_snapshot(&self) -> MetricsSnapshot {
        // 返回第一个收集器的快照，如果没有则返回默认
        if let Some(first) = self.collectors.first() {
            first.get_snapshot().await
        } else {
            MetricsSnapshot::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_metrics_collector() {
        let collector = InMemoryMetricsCollector::new();

        // 记录一些指标
        collector.record_workflow_duration("wf1", Duration::from_secs(1));
        collector.record_workflow_duration("wf1", Duration::from_secs(2));
        collector.record_workflow_result("wf1", true);
        collector.record_workflow_result("wf1", false);

        // 等待异步任务完成
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 获取统计
        let stats = collector.get_workflow_stats("wf1").await;
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.total_executions, 2);

        // 检查成功率
        let success_rate = collector.get_success_rate();
        assert!((success_rate - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_metrics_snapshot() {
        let collector = InMemoryMetricsCollector::new();

        collector.record_workflow_duration("wf1", Duration::from_secs(1));
        collector.record_node_duration("tool", Duration::from_millis(100));
        collector.record_queue_depth(5);
        collector.record_concurrent_executions(3);

        // 等待异步任务完成
        tokio::time::sleep(Duration::from_millis(100)).await;

        let snapshot = collector.get_snapshot().await;
        assert_eq!(snapshot.current_queue_depth, 5);
        assert_eq!(snapshot.current_concurrent_executions, 3);
    }

    #[tokio::test]
    async fn test_composite_collector() {
        let mut composite = CompositeMetricsCollector::new();
        composite.add_collector(Box::new(InMemoryMetricsCollector::new()));
        composite.add_collector(Box::new(PrometheusMetricsCollector::new()));

        // 记录指标
        composite.record_workflow_duration("wf1", Duration::from_secs(1));
        composite.record_workflow_result("wf1", true);

        // 应该正常执行不 panic
    }
}
