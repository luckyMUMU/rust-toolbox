//! 分布式追踪模块
//!
//! 提供追踪上下文传播和 Span 记录功能

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};
use uuid::Uuid;

/// 追踪 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(pub u128);

impl TraceId {
    /// 生成新的追踪 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().as_u128())
    }

    /// 从字符串解析
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim_start_matches("0x");
        u128::from_str_radix(s, 16).ok().map(Self)
    }

    /// 转换为十六进制字符串
    pub fn to_hex(&self) -> String {
        format!("{:032x}", self.0)
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Span ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpanId(pub u64);

impl SpanId {
    /// 生成新的 Span ID
    pub fn new() -> Self {
        Self(rand::random())
    }

    /// 从字符串解析
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim_start_matches("0x");
        u64::from_str_radix(s, 16).ok().map(Self)
    }

    /// 转换为十六进制字符串
    pub fn to_hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

impl Default for SpanId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SpanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// 追踪上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingContext {
    /// 追踪 ID
    pub trace_id: TraceId,
    /// 当前 Span ID
    pub span_id: SpanId,
    /// 父 Span ID
    pub parent_span_id: Option<SpanId>,
    /// 采样标志
    pub sampled: bool,
    /// Baggage（跨服务传递的键值对）
    pub baggage: HashMap<String, String>,
    /// 追踪标志
    pub flags: u8,
}

impl TracingContext {
    /// 创建新的追踪上下文
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
            sampled: true,
            baggage: HashMap::new(),
            flags: 1,
        }
    }

    /// 从追踪 ID 创建
    pub fn with_trace_id(trace_id: TraceId) -> Self {
        Self {
            trace_id,
            span_id: SpanId::new(),
            parent_span_id: None,
            sampled: true,
            baggage: HashMap::new(),
            flags: 1,
        }
    }

    /// 创建子上下文
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: SpanId::new(),
            parent_span_id: Some(self.span_id),
            sampled: self.sampled,
            baggage: self.baggage.clone(),
            flags: self.flags,
        }
    }

    /// 设置 Baggage
    pub fn set_baggage(&mut self, key: String, value: String) {
        self.baggage.insert(key, value);
    }

    /// 获取 Baggage
    pub fn get_baggage(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }

    /// 解析 W3C Trace Context 格式
    pub fn parse_w3c(traceparent: &str, tracestate: Option<&str>) -> Option<Self> {
        let parts: Vec<&str> = traceparent.split('-').collect();
        if parts.len() != 4 {
            return None;
        }

        let version = u8::from_str_radix(parts[0], 16).ok()?;
        if version != 0 && version != 1 {
            return None;
        }

        let trace_id = TraceId::parse(parts[1])?;
        let parent_id = SpanId::parse(parts[2])?;
        let flags = u8::from_str_radix(parts[3], 16).ok()?;

        let mut baggage = HashMap::new();
        if let Some(state) = tracestate {
            for entry in state.split(',') {
                if let Some((key, value)) = entry.split_once('=') {
                    baggage.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Some(Self {
            trace_id,
            span_id: parent_id,
            parent_span_id: None,
            sampled: flags & 1 == 1,
            baggage,
            flags,
        })
    }

    /// 导出为 W3C Trace Context 格式
    pub fn to_w3c(&self) -> (String, Option<String>) {
        let traceparent = format!(
            "00-{}-{}-{:02x}",
            self.trace_id.to_hex(),
            self.span_id.to_hex(),
            self.flags
        );

        let tracestate = if self.baggage.is_empty() {
            None
        } else {
            Some(
                self.baggage
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        };

        (traceparent, tracestate)
    }

    /// 解析 B3 格式
    pub fn parse_b3(b3: &str) -> Option<Self> {
        let parts: Vec<&str> = b3.split('-').collect();

        match parts.len() {
            1 => {
                let trace_id = TraceId::parse(parts[0])?;
                Some(Self::with_trace_id(trace_id))
            }
            2 => {
                let trace_id = TraceId::parse(parts[0])?;
                let span_id = SpanId::parse(parts[1])?;
                Some(Self {
                    trace_id,
                    span_id,
                    parent_span_id: None,
                    sampled: true,
                    baggage: HashMap::new(),
                    flags: 1,
                })
            }
            3 => {
                let trace_id = TraceId::parse(parts[0])?;
                let span_id = SpanId::parse(parts[1])?;
                let sampled = parts[2] == "1" || parts[2] == "d";
                Some(Self {
                    trace_id,
                    span_id,
                    parent_span_id: None,
                    sampled,
                    baggage: HashMap::new(),
                    flags: if sampled { 1 } else { 0 },
                })
            }
            4 => {
                let trace_id = TraceId::parse(parts[0])?;
                let span_id = SpanId::parse(parts[1])?;
                let sampled = parts[2] == "1" || parts[2] == "d";
                let parent_span_id = SpanId::parse(parts[3])?;
                Some(Self {
                    trace_id,
                    span_id,
                    parent_span_id: Some(parent_span_id),
                    sampled,
                    baggage: HashMap::new(),
                    flags: if sampled { 1 } else { 0 },
                })
            }
            _ => None,
        }
    }

    /// 导出为 B3 格式
    pub fn to_b3(&self) -> String {
        match self.parent_span_id {
            Some(parent) => format!(
                "{}-{}-{}-{}",
                self.trace_id.to_hex(),
                self.span_id.to_hex(),
                if self.sampled { "1" } else { "0" },
                parent.to_hex()
            ),
            None => format!(
                "{}-{}-{}",
                self.trace_id.to_hex(),
                self.span_id.to_hex(),
                if self.sampled { "1" } else { "0" }
            ),
        }
    }
}

impl Default for TracingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Span 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanKind {
    /// 服务端
    Server,
    /// 客户端
    Client,
    /// 生产者
    Producer,
    /// 消费者
    Consumer,
    /// 内部
    Internal,
}

impl std::fmt::Display for SpanKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpanKind::Server => write!(f, "SERVER"),
            SpanKind::Client => write!(f, "CLIENT"),
            SpanKind::Producer => write!(f, "PRODUCER"),
            SpanKind::Consumer => write!(f, "CONSUMER"),
            SpanKind::Internal => write!(f, "INTERNAL"),
        }
    }
}

/// Span 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// 未设置
    Unset,
    /// 成功
    Ok,
    /// 错误
    Error,
}

/// Span 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// 事件名称
    pub name: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 属性
    pub attributes: HashMap<String, AttributeValue>,
}

impl SpanEvent {
    /// 创建新事件
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            timestamp: Utc::now(),
            attributes: HashMap::new(),
        }
    }

    /// 添加属性
    pub fn with_attribute(mut self, key: String, value: AttributeValue) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

/// 属性值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    StringArray(Vec<String>),
    IntArray(Vec<i64>),
    FloatArray(Vec<f64>),
    BoolArray(Vec<bool>),
}

/// Span 链接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    /// 关联的追踪 ID
    pub trace_id: TraceId,
    /// 关联的 Span ID
    pub span_id: SpanId,
    /// 属性
    pub attributes: HashMap<String, AttributeValue>,
}

impl SpanLink {
    /// 创建新链接
    pub fn new(trace_id: TraceId, span_id: SpanId) -> Self {
        Self {
            trace_id,
            span_id,
            attributes: HashMap::new(),
        }
    }
}

/// 追踪 Span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingSpan {
    /// 追踪 ID
    pub trace_id: TraceId,
    /// Span ID
    pub span_id: SpanId,
    /// 父 Span ID
    pub parent_span_id: Option<SpanId>,
    /// Span 名称
    pub name: String,
    /// Span 类型
    pub kind: SpanKind,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 持续时间
    pub duration: Option<Duration>,
    /// 状态
    pub status: SpanStatus,
    /// 状态描述
    pub status_message: Option<String>,
    /// 属性
    pub attributes: HashMap<String, AttributeValue>,
    /// 事件列表
    pub events: Vec<SpanEvent>,
    /// 链接列表
    pub links: Vec<SpanLink>,
}

impl TracingSpan {
    /// 创建新的 Span
    pub fn new(name: impl Into<String>, ctx: &TracingContext) -> Self {
        Self {
            trace_id: ctx.trace_id,
            span_id: SpanId::new(),
            parent_span_id: Some(ctx.span_id),
            name: name.into(),
            kind: SpanKind::Internal,
            start_time: Utc::now(),
            end_time: None,
            duration: None,
            status: SpanStatus::Unset,
            status_message: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            links: Vec::new(),
        }
    }

    /// 创建根 Span
    pub fn root(name: impl Into<String>) -> Self {
        let ctx = TracingContext::new();
        Self {
            trace_id: ctx.trace_id,
            span_id: ctx.span_id,
            parent_span_id: None,
            name: name.into(),
            kind: SpanKind::Internal,
            start_time: Utc::now(),
            end_time: None,
            duration: None,
            status: SpanStatus::Unset,
            status_message: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            links: Vec::new(),
        }
    }

    /// 设置 Span 类型
    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    /// 设置属性
    pub fn set_attribute(&mut self, key: String, value: AttributeValue) {
        self.attributes.insert(key, value);
    }

    /// 添加事件
    pub fn add_event(&mut self, event: SpanEvent) {
        self.events.push(event);
    }

    /// 添加链接
    pub fn add_link(&mut self, link: SpanLink) {
        self.links.push(link);
    }

    /// 设置成功状态
    pub fn set_ok(&mut self) {
        self.status = SpanStatus::Ok;
    }

    /// 设置错误状态
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.status = SpanStatus::Error;
        self.status_message = Some(message.into());
    }

    /// 结束 Span
    pub fn end(&mut self) {
        self.end_time = Some(Utc::now());
        self.duration = self.end_time.and_then(|end| {
            (end - self.start_time).to_std().ok()
        });
    }

    /// 获取追踪上下文
    pub fn context(&self) -> TracingContext {
        TracingContext {
            trace_id: self.trace_id,
            span_id: self.span_id,
            parent_span_id: self.parent_span_id,
            sampled: true,
            baggage: HashMap::new(),
            flags: 1,
        }
    }

    /// 是否已结束
    pub fn is_finished(&self) -> bool {
        self.end_time.is_some()
    }
}

/// 追踪导出器 trait
pub trait SpanExporter: Send + Sync {
    /// 导出 Span
    fn export(&self, spans: Vec<TracingSpan>) -> Result<(), ExportError>;
}

/// 导出错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum ExportError {
    #[error("导出失败: {0}")]
    ExportFailed(String),
    #[error("连接失败: {0}")]
    ConnectionFailed(String),
    #[error("序列化失败: {0}")]
    SerializationFailed(String),
}

/// 内存导出器（用于测试）
pub struct InMemoryExporter {
    spans: std::sync::Mutex<Vec<TracingSpan>>,
}

impl InMemoryExporter {
    /// 创建新的内存导出器
    pub fn new() -> Self {
        Self {
            spans: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// 获取所有 Span
    pub fn get_spans(&self) -> Vec<TracingSpan> {
        self.spans.lock().unwrap().clone()
    }

    /// 清空 Span
    pub fn clear(&self) {
        self.spans.lock().unwrap().clear();
    }
}

impl Default for InMemoryExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanExporter for InMemoryExporter {
    fn export(&self, spans: Vec<TracingSpan>) -> Result<(), ExportError> {
        let mut stored = self.spans.lock().unwrap();
        stored.extend(spans);
        Ok(())
    }
}

/// 追踪配置
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// 服务名称
    pub service_name: String,
    /// 采样率 (0.0 - 1.0)
    pub sampling_rate: f64,
    /// 最大属性数
    pub max_attributes: usize,
    /// 最大事件数
    pub max_events: usize,
    /// 最大链接数
    pub max_links: usize,
    /// 批量导出大小
    pub batch_size: usize,
    /// 导出超时
    pub export_timeout: Duration,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "workflow-toolkit".to_string(),
            sampling_rate: 1.0,
            max_attributes: 128,
            max_events: 128,
            max_links: 128,
            batch_size: 512,
            export_timeout: Duration::from_secs(30),
        }
    }
}

/// 追踪器
pub struct Tracer {
    config: TracingConfig,
    exporter: Arc<dyn SpanExporter>,
}

impl Tracer {
    /// 创建新的追踪器
    pub fn new(config: TracingConfig, exporter: Arc<dyn SpanExporter>) -> Self {
        Self { config, exporter }
    }

    /// 使用默认配置创建追踪器
    pub fn default_tracer(exporter: Arc<dyn SpanExporter>) -> Self {
        Self::new(TracingConfig::default(), exporter)
    }

    /// 创建根 Span
    pub fn start_span(&self, name: &str) -> TracingSpan {
        let should_sample = rand::random::<f64>() < self.config.sampling_rate;
        if should_sample {
            TracingSpan::root(name)
        } else {
            let mut span = TracingSpan::root(name);
            span.set_attribute("sampled".to_string(), AttributeValue::Bool(false));
            span
        }
    }

    /// 创建子 Span
    pub fn start_child_span(&self, name: &str, parent: &TracingSpan) -> TracingSpan {
        TracingSpan::new(name, &parent.context())
    }

    /// 导出 Span
    pub fn export(&self, span: TracingSpan) -> Result<(), ExportError> {
        self.exporter.export(vec![span])
    }

    /// 批量导出
    pub fn export_batch(&self, spans: Vec<TracingSpan>) -> Result<(), ExportError> {
        self.exporter.export(spans)
    }

    /// 获取配置
    pub fn config(&self) -> &TracingConfig {
        &self.config
    }
}

/// 追踪作用域守卫
pub struct SpanGuard {
    span: Option<TracingSpan>,
    tracer: Arc<Tracer>,
}

impl SpanGuard {
    /// 创建新的守卫
    pub fn new(span: TracingSpan, tracer: Arc<Tracer>) -> Self {
        Self {
            span: Some(span),
            tracer,
        }
    }

    /// 获取 Span 引用
    pub fn span(&self) -> Option<&TracingSpan> {
        self.span.as_ref()
    }

    /// 获取 Span 可变引用
    pub fn span_mut(&mut self) -> Option<&mut TracingSpan> {
        self.span.as_mut()
    }

    /// 设置属性
    pub fn set_attribute(&mut self, key: String, value: AttributeValue) {
        if let Some(span) = &mut self.span {
            span.set_attribute(key, value);
        }
    }

    /// 添加事件
    pub fn add_event(&mut self, name: impl Into<String>) {
        if let Some(span) = &mut self.span {
            span.add_event(SpanEvent::new(name));
        }
    }

    /// 设置错误
    pub fn set_error(&mut self, message: impl Into<String>) {
        if let Some(span) = &mut self.span {
            span.set_error(message);
        }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        if let Some(mut span) = self.span.take() {
            span.end();
            if span.status != SpanStatus::Error || span.attributes.contains_key("sampled") {
                let _ = self.tracer.export(span);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_generation() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();
        assert_ne!(id1, id2);

        let hex = id1.to_hex();
        assert_eq!(hex.len(), 32);

        let parsed = TraceId::parse(&hex).unwrap();
        assert_eq!(id1, parsed);
    }

    #[test]
    fn test_span_id_generation() {
        let id1 = SpanId::new();
        let id2 = SpanId::new();
        assert_ne!(id1, id2);

        let hex = id1.to_hex();
        assert_eq!(hex.len(), 16);

        let parsed = SpanId::parse(&hex).unwrap();
        assert_eq!(id1, parsed);
    }

    #[test]
    fn test_tracing_context() {
        let ctx = TracingContext::new();
        let child = ctx.child();

        assert_eq!(ctx.trace_id, child.trace_id);
        assert_eq!(child.parent_span_id, Some(ctx.span_id));
    }

    #[test]
    fn test_w3c_format() {
        let ctx = TracingContext::new();
        let (traceparent, tracestate) = ctx.to_w3c();

        assert!(traceparent.starts_with("00-"));

        let parsed = TracingContext::parse_w3c(&traceparent, tracestate.as_deref()).unwrap();
        assert_eq!(ctx.trace_id, parsed.trace_id);
    }

    #[test]
    fn test_span_creation() {
        let span = TracingSpan::root("test-span");
        assert_eq!(span.name, "test-span");
        assert!(!span.is_finished());
    }

    #[test]
    fn test_span_end() {
        let mut span = TracingSpan::root("test-span");
        span.end();

        assert!(span.is_finished());
        assert!(span.duration.is_some());
    }

    #[test]
    fn test_in_memory_exporter() {
        let exporter = InMemoryExporter::new();
        let span = TracingSpan::root("test");

        exporter.export(vec![span]).unwrap();

        let spans = exporter.get_spans();
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_span_guard() {
        let exporter = Arc::new(InMemoryExporter::new());
        let tracer = Arc::new(Tracer::default_tracer(exporter.clone()));

        {
            let _guard = SpanGuard::new(TracingSpan::root("test"), tracer);
        }

        let spans = exporter.get_spans();
        assert_eq!(spans.len(), 1);
        assert!(spans[0].is_finished());
    }
}
