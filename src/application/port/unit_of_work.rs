//! Unit of Work 模式实现
//!
//! 提供事务管理和工作单元模式

use crate::domain::event::{DomainEvent, DomainEventBus};
use crate::error::{Result, WorkflowError};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Unit of Work trait
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// 提交事务
    async fn commit(&mut self) -> Result<()>;

    /// 回滚事务
    async fn rollback(&mut self) -> Result<()>;

    /// 添加待发布事件
    fn add_event(&mut self, event: DomainEvent);

    /// 获取待发布事件
    fn get_events(&self) -> &VecDeque<DomainEvent>;

    /// 清空待发布事件
    fn clear_events(&mut self);

    /// 是否有变更
    fn has_changes(&self) -> bool;
}

/// Unit of Work 上下文
pub struct UnitOfWorkContext {
    /// 事件总线
    event_bus: Arc<DomainEventBus>,
    /// 待发布事件队列
    pending_events: VecDeque<DomainEvent>,
    /// 是否已提交
    committed: bool,
    /// 是否已回滚
    rolled_back: bool,
    /// 事务 ID
    transaction_id: String,
}

impl UnitOfWorkContext {
    /// 创建新的 Unit of Work 上下文
    pub fn new(event_bus: Arc<DomainEventBus>) -> Self {
        Self {
            event_bus,
            pending_events: VecDeque::new(),
            committed: false,
            rolled_back: false,
            transaction_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// 获取事务 ID
    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }
}

#[async_trait]
impl UnitOfWork for UnitOfWorkContext {
    async fn commit(&mut self) -> Result<()> {
        if self.committed {
            return Err(WorkflowError::execution("事务已提交，不能重复提交"));
        }

        if self.rolled_back {
            return Err(WorkflowError::execution("事务已回滚，不能提交"));
        }

        info!(
            transaction_id = %self.transaction_id,
            event_count = self.pending_events.len(),
            "提交事务"
        );

        while let Some(event) = self.pending_events.pop_front() {
            if let Err(e) = self.event_bus.publish(event).await {
                error!(
                    transaction_id = %self.transaction_id,
                    error = %e,
                    "发布事件失败"
                );
            }
        }

        self.committed = true;

        debug!(transaction_id = %self.transaction_id, "事务提交完成");
        Ok(())
    }

    async fn rollback(&mut self) -> Result<()> {
        if self.committed {
            return Err(WorkflowError::execution("事务已提交，不能回滚"));
        }

        if self.rolled_back {
            return Err(WorkflowError::execution("事务已回滚，不能重复回滚"));
        }

        info!(
            transaction_id = %self.transaction_id,
            event_count = self.pending_events.len(),
            "回滚事务"
        );

        self.pending_events.clear();
        self.rolled_back = true;

        debug!(transaction_id = %self.transaction_id, "事务回滚完成");
        Ok(())
    }

    fn add_event(&mut self, event: DomainEvent) {
        self.pending_events.push_back(event);
    }

    fn get_events(&self) -> &VecDeque<DomainEvent> {
        &self.pending_events
    }

    fn clear_events(&mut self) {
        self.pending_events.clear();
    }

    fn has_changes(&self) -> bool {
        !self.pending_events.is_empty()
    }
}

/// Unit of Work 工厂
pub struct UnitOfWorkFactory {
    event_bus: Arc<DomainEventBus>,
}

impl UnitOfWorkFactory {
    /// 创建新的工厂
    pub fn new(event_bus: Arc<DomainEventBus>) -> Self {
        Self { event_bus }
    }

    /// 创建新的 Unit of Work
    pub fn create(&self) -> UnitOfWorkContext {
        UnitOfWorkContext::new(Arc::clone(&self.event_bus))
    }
}

/// Unit of Work 管理器
pub struct UnitOfWorkManager {
    factory: UnitOfWorkFactory,
}

impl UnitOfWorkManager {
    /// 创建新的管理器
    pub fn new(event_bus: Arc<DomainEventBus>) -> Self {
        Self {
            factory: UnitOfWorkFactory::new(event_bus),
        }
    }

    /// 执行事务
    pub async fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(
                &mut UnitOfWorkContext,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>
            + Send,
        T: Send,
    {
        let mut uow = self.factory.create();

        debug!(transaction_id = %uow.transaction_id(), "开始事务");

        let result = operation(&mut uow).await;

        match result {
            Ok(value) => {
                uow.commit().await?;
                Ok(value)
            }
            Err(e) => {
                if let Err(rollback_err) = uow.rollback().await {
                    error!(error = %rollback_err, "回滚失败");
                }
                Err(e)
            }
        }
    }

    /// 执行事务（带重试）
    pub async fn execute_with_retry<F, T>(&self, max_retries: u32, operation: F) -> Result<T>
    where
        F: Fn(
                &mut UnitOfWorkContext,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>
            + Send
            + Sync,
        T: Send,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            attempts += 1;

            let mut uow = self.factory.create();

            debug!(
                transaction_id = %uow.transaction_id(),
                attempt = attempts,
                max_retries = max_retries,
                "开始事务（带重试）"
            );

            match operation(&mut uow).await {
                Ok(value) => {
                    uow.commit().await?;
                    return Ok(value);
                }
                Err(e) => {
                    if let Err(rollback_err) = uow.rollback().await {
                        error!(error = %rollback_err, "回滚失败");
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| WorkflowError::execution("事务执行失败")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::event::events::WorkflowCreatedEvent;
    use crate::domain::event::{EventBusConfig, EventMetadata};

    fn create_test_event() -> DomainEvent {
        DomainEvent::WorkflowCreated(WorkflowCreatedEvent {
            metadata: EventMetadata::new("Workflow", "wf-123", "WorkflowCreated"),
            workflow_id: "wf-123".to_string(),
            workflow_name: "test-workflow".to_string(),
            workflow_version: "1.0.0".to_string(),
        })
    }

    #[tokio::test]
    async fn test_unit_of_work_creation() {
        let event_bus = Arc::new(DomainEventBus::with_defaults());
        let mut uow = UnitOfWorkContext::new(event_bus);

        assert!(!uow.has_changes());
        assert!(uow.get_events().is_empty());
    }

    #[tokio::test]
    async fn test_add_event() {
        let event_bus = Arc::new(DomainEventBus::with_defaults());
        let mut uow = UnitOfWorkContext::new(event_bus);

        uow.add_event(create_test_event());

        assert!(uow.has_changes());
        assert_eq!(uow.get_events().len(), 1);
    }

    #[tokio::test]
    async fn test_commit() {
        let event_bus = Arc::new(DomainEventBus::with_defaults());
        let mut uow = UnitOfWorkContext::new(Arc::clone(&event_bus));

        uow.add_event(create_test_event());
        uow.commit().await.unwrap();

        assert!(uow.get_events().is_empty());
    }

    #[tokio::test]
    async fn test_rollback() {
        let event_bus = Arc::new(DomainEventBus::with_defaults());
        let mut uow = UnitOfWorkContext::new(event_bus);

        uow.add_event(create_test_event());
        uow.rollback().await.unwrap();

        assert!(!uow.has_changes());
        assert!(uow.get_events().is_empty());
    }

    #[tokio::test]
    async fn test_unit_of_work_manager() {
        let event_bus = Arc::new(DomainEventBus::with_defaults());
        let manager = UnitOfWorkManager::new(event_bus);

        let result = manager
            .execute(|uow| {
                Box::pin(async move {
                    uow.add_event(create_test_event());
                    Ok(42)
                })
            })
            .await;

        assert_eq!(result.unwrap(), 42);
    }
}
