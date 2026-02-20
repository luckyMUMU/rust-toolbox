//! 领域事件总线
//!
//! 提供事件的发布订阅机制

use super::events::{DomainEvent, EventId};
use crate::error::{Result, WorkflowError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// 订阅 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 事件订阅者 trait
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// 处理事件
    async fn handle(&self, event: &DomainEvent) -> Result<()>;
    
    /// 获取订阅者名称
    fn name(&self) -> &str;
    
    /// 感兴趣的事件类型（空表示订阅所有事件）
    fn event_types(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

/// 事件总线配置
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// 事件通道容量
    pub channel_capacity: usize,
    /// 是否持久化事件
    pub persist_events: bool,
    /// 最大持久化事件数
    pub max_persisted_events: usize,
    /// 是否启用异步处理
    pub async_processing: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 1000,
            persist_events: true,
            max_persisted_events: 10000,
            async_processing: true,
        }
    }
}

/// 事件总线统计
#[derive(Debug, Clone, Default)]
pub struct EventBusStats {
    /// 发布的事件总数
    pub total_published: u64,
    /// 成功处理的事件数
    pub total_processed: u64,
    /// 处理失败的事件数
    pub total_failed: u64,
    /// 当前订阅者数量
    pub subscriber_count: usize,
}

/// 领域事件总线
pub struct DomainEventBus {
    /// 配置
    config: EventBusConfig,
    /// 事件广播器
    broadcaster: broadcast::Sender<DomainEvent>,
    /// 订阅者映射
    subscribers: RwLock<HashMap<SubscriptionId, Arc<dyn EventSubscriber>>>,
    /// 订阅 ID 计数器
    subscription_counter: Mutex<u64>,
    /// 事件存储
    event_store: RwLock<Vec<DomainEvent>>,
    /// 统计信息
    stats: RwLock<EventBusStats>,
}

impl DomainEventBus {
    /// 创建新的事件总线
    pub fn new(config: EventBusConfig) -> Self {
        let (broadcaster, _) = broadcast::channel(config.channel_capacity);
        
        Self {
            config,
            broadcaster,
            subscribers: RwLock::new(HashMap::new()),
            subscription_counter: Mutex::new(0),
            event_store: RwLock::new(Vec::new()),
            stats: RwLock::new(EventBusStats::default()),
        }
    }

    /// 使用默认配置创建事件总线
    pub fn with_defaults() -> Self {
        Self::new(EventBusConfig::default())
    }

    /// 发布事件
    pub async fn publish(&self, event: DomainEvent) -> Result<()> {
        let event_type = event.event_type_name();
        let event_id = event.metadata().event_id;
        
        info!(
            event_id = %event_id,
            event_type = %event_type,
            "发布领域事件"
        );
        
        if self.config.persist_events {
            self.persist_event(&event).await;
        }
        
        {
            let mut stats = self.stats.write().await;
            stats.total_published += 1;
        }
        
        let receiver_count = self.broadcaster.receiver_count();
        
        if receiver_count == 0 {
            debug!("没有订阅者，事件将被丢弃");
            return Ok(());
        }
        
        self.broadcaster
            .send(event)
            .map_err(|e| WorkflowError::execution(format!("发布事件失败: {}", e)))?;
        
        Ok(())
    }

    /// 批量发布事件
    pub async fn publish_batch(&self, events: Vec<DomainEvent>) -> Result<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }

    /// 订阅事件
    pub async fn subscribe(&self, subscriber: Arc<dyn EventSubscriber>) -> SubscriptionId {
        let mut counter = self.subscription_counter.lock().await;
        let subscription_id = SubscriptionId(*counter);
        *counter += 1;
        
        self.subscribers
            .write()
            .await
            .insert(subscription_id, subscriber.clone());
        
        let mut receiver = self.broadcaster.subscribe();
        let stats = Arc::clone(&self.stats);
        let subscriber_name = subscriber.name().to_string();
        
        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        let event_types = subscriber.event_types();
                        let should_handle = event_types.is_empty()
                            || event_types.iter().any(|t| *t == event.event_type_name());
                        
                        if should_handle {
                            if let Err(e) = subscriber.handle(&event).await {
                                error!(
                                    subscriber = %subscriber_name,
                                    error = %e,
                                    "事件处理失败"
                                );
                                
                                let mut s = stats.write().await;
                                s.total_failed += 1;
                            } else {
                                let mut s = stats.write().await;
                                s.total_processed += 1;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("事件通道已关闭");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(
                            subscriber = %subscriber_name,
                            lagged = n,
                            "事件处理落后"
                        );
                    }
                }
            }
        });
        
        {
            let mut stats = self.stats.write().await;
            stats.subscriber_count = self.subscribers.read().await.len();
        }
        
        info!(
            subscription_id = %subscription_id,
            subscriber = %subscriber.name(),
            "订阅者注册成功"
        );
        
        subscription_id
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<()> {
        let removed = self.subscribers.write().await.remove(&subscription_id);
        
        if removed.is_some() {
            info!(subscription_id = %subscription_id, "取消订阅成功");
            
            let mut stats = self.stats.write().await;
            stats.subscriber_count = self.subscribers.read().await.len();
            
            Ok(())
        } else {
            Err(WorkflowError::not_found(format!(
                "订阅 {} 不存在",
                subscription_id
            )))
        }
    }

    /// 持久化事件
    async fn persist_event(&self, event: &DomainEvent) {
        let mut store = self.event_store.write().await;
        
        if store.len() >= self.config.max_persisted_events {
            let remove_count = store.len() - self.config.max_persisted_events + 1;
            store.drain(0..remove_count);
        }
        
        store.push(event.clone());
    }

    /// 获取持久化的事件
    pub async fn get_events(&self) -> Vec<DomainEvent> {
        self.event_store.read().await.clone()
    }

    /// 按类型获取事件
    pub async fn get_events_by_type(&self, event_type: &str) -> Vec<DomainEvent> {
        self.event_store
            .read()
            .await
            .iter()
            .filter(|e| e.event_type_name() == event_type)
            .cloned()
            .collect()
    }

    /// 按聚合 ID 获取事件
    pub async fn get_events_by_aggregate(&self, aggregate_id: &str) -> Vec<DomainEvent> {
        self.event_store
            .read()
            .await
            .iter()
            .filter(|e| e.metadata().aggregate_id == aggregate_id)
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub async fn stats(&self) -> EventBusStats {
        self.stats.read().await.clone()
    }

    /// 清空事件存储
    pub async fn clear_events(&self) {
        self.event_store.write().await.clear();
        info!("事件存储已清空");
    }

    /// 获取订阅者数量
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
}

/// 事件存储接口
#[async_trait]
pub trait EventStore: Send + Sync {
    /// 保存事件
    async fn save(&self, event: &DomainEvent) -> Result<()>;
    
    /// 获取事件
    async fn get(&self, event_id: EventId) -> Result<Option<DomainEvent>>;
    
    /// 获取聚合的事件流
    async fn get_stream(&self, aggregate_id: &str) -> Result<Vec<DomainEvent>>;
    
    /// 获取所有事件
    async fn get_all(&self) -> Result<Vec<DomainEvent>>;
}

/// 内存事件存储
pub struct InMemoryEventStore {
    events: RwLock<Vec<DomainEvent>>,
}

impl InMemoryEventStore {
    /// 创建新的内存事件存储
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn save(&self, event: &DomainEvent) -> Result<()> {
        self.events.write().await.push(event.clone());
        Ok(())
    }

    async fn get(&self, event_id: EventId) -> Result<Option<DomainEvent>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .find(|e| e.metadata().event_id == event_id)
            .cloned())
    }

    async fn get_stream(&self, aggregate_id: &str) -> Result<Vec<DomainEvent>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .filter(|e| e.metadata().aggregate_id == aggregate_id)
            .cloned()
            .collect())
    }

    async fn get_all(&self) -> Result<Vec<DomainEvent>> {
        Ok(self.events.read().await.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::event::events::{WorkflowCreatedEvent, EventMetadata};

    fn create_test_event() -> DomainEvent {
        DomainEvent::WorkflowCreated(WorkflowCreatedEvent {
            metadata: EventMetadata::new("Workflow", "wf-123", "WorkflowCreated"),
            workflow_id: "wf-123".to_string(),
            workflow_name: "test-workflow".to_string(),
            workflow_version: "1.0.0".to_string(),
        })
    }

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = DomainEventBus::with_defaults();
        let stats = bus.stats().await;
        assert_eq!(stats.total_published, 0);
    }

    #[tokio::test]
    async fn test_publish_event() {
        let bus = DomainEventBus::with_defaults();
        let event = create_test_event();
        
        bus.publish(event).await.unwrap();
        
        let stats = bus.stats().await;
        assert_eq!(stats.total_published, 1);
    }

    #[tokio::test]
    async fn test_subscribe() {
        let bus = DomainEventBus::with_defaults();
        
        struct TestSubscriber;
        
        #[async_trait]
        impl EventSubscriber for TestSubscriber {
            async fn handle(&self, _event: &DomainEvent) -> Result<()> {
                Ok(())
            }
            
            fn name(&self) -> &str {
                "TestSubscriber"
            }
        }
        
        let subscriber = Arc::new(TestSubscriber);
        let subscription_id = bus.subscribe(subscriber).await;
        
        assert!(bus.subscriber_count().await >= 1);
    }

    #[tokio::test]
    async fn test_in_memory_event_store() {
        let store = InMemoryEventStore::new();
        let event = create_test_event();
        let event_id = event.metadata().event_id;
        
        store.save(&event).await.unwrap();
        
        let retrieved = store.get(event_id).await.unwrap();
        assert!(retrieved.is_some());
    }
}
