//! 工具注册表端口
//!
//! ⚠️ **重要提示**: 本模块已迁移到新系统
//!
//! 旧的 trait-based 工具系统已被新的 enum-based 系统取代。
//! 新的工具类型定义在 `crate::tools::types::Tool`。
//!
//! 本文件保留作为占位符，避免破坏模块结构。
//! 所有工具相关的核心类型现在直接从 `crate::tools` 导出。

pub use crate::tools::registry::ToolRegistry;
pub use crate::tools::types::{Tool, ToolId, ToolInput, ToolKind, ToolMetadata, ToolOutput};
