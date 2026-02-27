//! 领域事件模块
//!
//! 提供领域事件的定义和事件总线实现

pub mod bus;
pub mod events;

pub use bus::{
    DomainEventBus, EventBusConfig, EventBusStats, EventStore, EventSubscriber, InMemoryEventStore,
    SubscriptionId,
};
pub use events::{DomainEvent, DomainEventTrait, EventId, EventMetadata};
