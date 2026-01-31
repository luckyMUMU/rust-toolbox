//! Dependency injection container using shaku

pub mod container;
pub mod module;
pub mod provider;

pub use container::DiContainer;
pub use module::AppModule;
pub use provider::Provider;
