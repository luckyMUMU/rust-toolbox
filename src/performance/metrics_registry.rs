//! 统一指标注册表
//!
//! 提供统一的指标收集和 Prometheus 格式导出

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use dashmap::DashMap;
use parking_lot::RwLock;

/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// 计数器（只增不减）
    Counter,
    /// 仪表盘（可增可减）
    Gauge,
    /// 直方图
    Histogram,
    /// 摘要
    Summary,
}

/// 指标标签
pub type Labels = HashMap<String, String>;

/// 计数器
#[derive(Debug)]
pub struct Counter {
    name: String,
    help: String,
    value: AtomicU64,
    labels: Labels,
}

impl Counter {
    /// 创建新的计数器
    pub fn new(name: &str, help: &str) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            value: AtomicU64::new(0),
            labels: Labels::new(),
        }
    }

    /// 使用标签创建计数器
    pub fn with_labels(name: &str, help: &str, labels: Labels) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            value: AtomicU64::new(0),
            labels,
        }
    }

    /// 增加计数
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// 增加指定值
    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    /// 获取当前值
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// 重置计数器
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }

    /// 导出为 Prometheus 格式
    pub fn to_prometheus(&self) -> String {
        let labels_str = self.format_labels();
        format!(
            "# HELP {} {}\n# TYPE {} counter\n{}{}\n",
            self.name, self.help, self.name, self.name, labels_str
        )
    }

    fn format_labels(&self) -> String {
        if self.labels.is_empty() {
            format!(" {}", self.value.load(Ordering::Relaxed))
        } else {
            let labels: Vec<String> = self.labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            format!("{{{}}} {}", labels.join(","), self.value.load(Ordering::Relaxed))
        }
    }
}

/// 仪表盘
#[derive(Debug)]
pub struct Gauge {
    name: String,
    help: String,
    value: RwLock<f64>,
    labels: Labels,
}

impl Gauge {
    /// 创建新的仪表盘
    pub fn new(name: &str, help: &str) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            value: RwLock::new(0.0),
            labels: Labels::new(),
        }
    }

    /// 使用标签创建仪表盘
    pub fn with_labels(name: &str, help: &str, labels: Labels) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            value: RwLock::new(0.0),
            labels,
        }
    }

    /// 设置值
    pub fn set(&self, value: f64) {
        *self.value.write() = value;
    }

    /// 增加值
    pub fn inc(&self) {
        let mut v = self.value.write();
        *v += 1.0;
    }

    /// 减少值
    pub fn dec(&self) {
        let mut v = self.value.write();
        *v -= 1.0;
    }

    /// 增加指定值
    pub fn add(&self, delta: f64) {
        let mut v = self.value.write();
        *v += delta;
    }

    /// 减少指定值
    pub fn sub(&self, delta: f64) {
        let mut v = self.value.write();
        *v -= delta;
    }

    /// 获取当前值
    pub fn get(&self) -> f64 {
        *self.value.read()
    }

    /// 导出为 Prometheus 格式
    pub fn to_prometheus(&self) -> String {
        let labels_str = self.format_labels();
        format!(
            "# HELP {} {}\n# TYPE {} gauge\n{}{}\n",
            self.name, self.help, self.name, self.name, labels_str
        )
    }

    fn format_labels(&self) -> String {
        if self.labels.is_empty() {
            format!(" {}", self.get())
        } else {
            let labels: Vec<String> = self.labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            format!("{{{}}} {}", labels.join(","), self.get())
        }
    }
}

/// 直方图桶
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// 直方图
#[derive(Debug)]
pub struct Histogram {
    name: String,
    help: String,
    buckets: RwLock<Vec<HistogramBucket>>,
    sum: RwLock<f64>,
    count: AtomicU64,
    labels: Labels,
}

impl Histogram {
    /// 创建新的直方图（使用默认桶）
    pub fn new(name: &str, help: &str) -> Self {
        Self::with_buckets(name, help, vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ])
    }

    /// 使用自定义桶创建直方图
    pub fn with_buckets(name: &str, help: &str, bounds: Vec<f64>) -> Self {
        let buckets = bounds
            .into_iter()
            .map(|upper_bound| HistogramBucket {
                upper_bound,
                count: 0,
            })
            .collect();
        
        Self {
            name: name.to_string(),
            help: help.to_string(),
            buckets: RwLock::new(buckets),
            sum: RwLock::new(0.0),
            count: AtomicU64::new(0),
            labels: Labels::new(),
        }
    }

    /// 观察一个值
    pub fn observe(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        
        {
            let mut sum = self.sum.write();
            *sum += value;
        }
        
        {
            let mut buckets = self.buckets.write();
            for bucket in buckets.iter_mut() {
                if value <= bucket.upper_bound {
                    bucket.count += 1;
                }
            }
        }
    }

    /// 记录执行时间
    pub fn time<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed().as_secs_f64();
        self.observe(duration);
        result
    }

    /// 异步记录执行时间
    pub async fn time_async<F, R, Fut>(&self, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed().as_secs_f64();
        self.observe(duration);
        result
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> (f64, u64, Vec<HistogramBucket>) {
        let sum = *self.sum.read();
        let count = self.count.load(Ordering::Relaxed);
        let buckets = self.buckets.read().clone();
        (sum, count, buckets)
    }

    /// 导出为 Prometheus 格式
    pub fn to_prometheus(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# HELP {} {}\n", self.name, self.help));
        output.push_str(&format!("# TYPE {} histogram\n", self.name));
        
        let (sum, count, buckets) = self.get_stats();
        
        let mut cumulative = 0u64;
        for bucket in &buckets {
            cumulative += bucket.count;
            let le = bucket.upper_bound;
            output.push_str(&format!(
                "{}_bucket{{le=\"{}\"}} {}\n",
                self.name, le, cumulative
            ));
        }
        
        output.push_str(&format!("{}_bucket{{le=\"+Inf\"}} {}\n", self.name, count));
        output.push_str(&format!("{}_sum {}\n", self.name, sum));
        output.push_str(&format!("{}_count {}\n", self.name, count));
        
        output
    }
}

/// 指标注册表
pub struct MetricsRegistry {
    counters: DashMap<String, Arc<Counter>>,
    gauges: DashMap<String, Arc<Gauge>>,
    histograms: DashMap<String, Arc<Histogram>>,
    namespace: String,
}

impl MetricsRegistry {
    /// 创建新的指标注册表
    pub fn new() -> Self {
        Self {
            counters: DashMap::new(),
            gauges: DashMap::new(),
            histograms: DashMap::new(),
            namespace: "workflow_toolkit".to_string(),
        }
    }

    /// 使用命名空间创建注册表
    pub fn with_namespace(namespace: &str) -> Self {
        Self {
            counters: DashMap::new(),
            gauges: DashMap::new(),
            histograms: DashMap::new(),
            namespace: namespace.to_string(),
        }
    }

    /// 注册计数器
    pub fn register_counter(&self, name: &str, help: &str) -> Arc<Counter> {
        let full_name = format!("{}_{}", self.namespace, name);
        let counter = Arc::new(Counter::new(&full_name, help));
        self.counters.insert(full_name, Arc::clone(&counter));
        counter
    }

    /// 注册带标签的计数器
    pub fn register_counter_with_labels(
        &self,
        name: &str,
        help: &str,
        labels: Labels,
    ) -> Arc<Counter> {
        let full_name = format!("{}_{}", self.namespace, name);
        let counter = Arc::new(Counter::with_labels(&full_name, help, labels));
        self.counters.insert(full_name, Arc::clone(&counter));
        counter
    }

    /// 注册仪表盘
    pub fn register_gauge(&self, name: &str, help: &str) -> Arc<Gauge> {
        let full_name = format!("{}_{}", self.namespace, name);
        let gauge = Arc::new(Gauge::new(&full_name, help));
        self.gauges.insert(full_name, Arc::clone(&gauge));
        gauge
    }

    /// 注册带标签的仪表盘
    pub fn register_gauge_with_labels(
        &self,
        name: &str,
        help: &str,
        labels: Labels,
    ) -> Arc<Gauge> {
        let full_name = format!("{}_{}", self.namespace, name);
        let gauge = Arc::new(Gauge::with_labels(&full_name, help, labels));
        self.gauges.insert(full_name, Arc::clone(&gauge));
        gauge
    }

    /// 注册直方图
    pub fn register_histogram(&self, name: &str, help: &str) -> Arc<Histogram> {
        let full_name = format!("{}_{}", self.namespace, name);
        let histogram = Arc::new(Histogram::new(&full_name, help));
        self.histograms.insert(full_name, Arc::clone(&histogram));
        histogram
    }

    /// 注册带自定义桶的直方图
    pub fn register_histogram_with_buckets(
        &self,
        name: &str,
        help: &str,
        buckets: Vec<f64>,
    ) -> Arc<Histogram> {
        let full_name = format!("{}_{}", self.namespace, name);
        let histogram = Arc::new(Histogram::with_buckets(&full_name, help, buckets));
        self.histograms.insert(full_name, Arc::clone(&histogram));
        histogram
    }

    /// 获取计数器
    pub fn get_counter(&self, name: &str) -> Option<Arc<Counter>> {
        let full_name = format!("{}_{}", self.namespace, name);
        self.counters.get(&full_name).map(|c| Arc::clone(c.value()))
    }

    /// 获取仪表盘
    pub fn get_gauge(&self, name: &str) -> Option<Arc<Gauge>> {
        let full_name = format!("{}_{}", self.namespace, name);
        self.gauges.get(&full_name).map(|g| Arc::clone(g.value()))
    }

    /// 获取直方图
    pub fn get_histogram(&self, name: &str) -> Option<Arc<Histogram>> {
        let full_name = format!("{}_{}", self.namespace, name);
        self.histograms.get(&full_name).map(|h| Arc::clone(h.value()))
    }

    /// 导出所有指标为 Prometheus 格式
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        for entry in self.counters.iter() {
            output.push_str(&entry.value().to_prometheus());
        }
        
        for entry in self.gauges.iter() {
            output.push_str(&entry.value().to_prometheus());
        }
        
        for entry in self.histograms.iter() {
            output.push_str(&entry.value().to_prometheus());
        }
        
        output
    }

    /// 获取指标数量
    pub fn metric_count(&self) -> usize {
        self.counters.len() + self.gauges.len() + self.histograms.len()
    }

    /// 清空所有指标
    pub fn clear(&self) {
        self.counters.clear();
        self.gauges.clear();
        self.histograms.clear();
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局指标注册表
static GLOBAL_REGISTRY: std::sync::OnceLock<Arc<MetricsRegistry>> = std::sync::OnceLock::new();

/// 获取全局指标注册表
pub fn global_registry() -> Arc<MetricsRegistry> {
    GLOBAL_REGISTRY
        .get_or_init(|| Arc::new(MetricsRegistry::new()))
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter", "Test counter");
        
        counter.inc();
        counter.inc();
        counter.add(5);
        
        assert_eq!(counter.get(), 7);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge", "Test gauge");
        
        gauge.set(10.0);
        assert_eq!(gauge.get(), 10.0);
        
        gauge.inc();
        assert_eq!(gauge.get(), 11.0);
        
        gauge.dec();
        assert_eq!(gauge.get(), 10.0);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new("test_histogram", "Test histogram");
        
        histogram.observe(0.1);
        histogram.observe(0.5);
        histogram.observe(1.0);
        
        let (sum, count, _) = histogram.get_stats();
        assert_eq!(count, 3);
        assert!((sum - 1.6).abs() < 0.001);
    }

    #[test]
    fn test_metrics_registry() {
        let registry = MetricsRegistry::new();
        
        let counter = registry.register_counter("requests", "Total requests");
        counter.inc();
        
        let gauge = registry.register_gauge("active_connections", "Active connections");
        gauge.set(5.0);
        
        let histogram = registry.register_histogram("request_duration", "Request duration");
        histogram.observe(0.1);
        
        assert_eq!(registry.metric_count(), 3);
    }

    #[test]
    fn test_prometheus_export() {
        let registry = MetricsRegistry::new();
        
        let counter = registry.register_counter("requests", "Total requests");
        counter.inc();
        
        let output = registry.export_prometheus();
        
        assert!(output.contains("workflow_toolkit_requests"));
        assert!(output.contains("Total requests"));
    }
}
