//! Plugin system for extending functionality

pub mod manager;
pub mod types;

pub use manager::PluginManager;
pub use types::{Plugin, PluginType};