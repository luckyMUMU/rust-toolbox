//! Domain layer - business logic and domain models

pub mod event;
pub mod model;
pub mod port;

pub use port::*;
pub use event::{
    DomainEvent, DomainEventBus, DomainEventTrait, EventBusConfig, EventBusStats,
    EventId, EventMetadata, EventStore, EventSubscriber, InMemoryEventStore, SubscriptionId,
};
