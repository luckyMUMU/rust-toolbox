//! 基础设施层 - 技术实现

pub mod config;
pub mod external;
pub mod persistence;
pub mod plugin;

pub use persistence::*;
pub use plugin::*;
