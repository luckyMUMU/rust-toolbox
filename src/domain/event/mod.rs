//! 领域事件模块
//!
//! 提供领域事件的定义和事件总线实现

pub mod events;
pub mod bus;

pub use events::{DomainEvent, DomainEventTrait, EventId, EventMetadata};
pub use bus::{DomainEventBus, EventSubscriber, SubscriptionId};
