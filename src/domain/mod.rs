//! Domain layer - business logic and domain models

pub mod event;
pub mod model;
pub mod port;
pub mod service;

pub use port::*;
pub use event::{
    DomainEvent, DomainEventBus, DomainEventTrait, EventBusConfig, EventBusStats,
    EventId, EventMetadata, EventStore, EventSubscriber, InMemoryEventStore, SubscriptionId,
};
pub use service::{
    DomainWorkflowValidator, ExecutionStateCalculator, ExecutionStateResult,
    NodeStateStats, ProgressDetails, ValidationError, ValidationResult, ValidationRule,
    ValidationWarning,
};
