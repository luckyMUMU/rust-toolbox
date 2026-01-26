//! TUI Widget implementations
//!
//! This module contains concrete widget implementations for the TUI interface.
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.

pub mod execution_monitor;
pub mod log_viewer;
pub mod plugin_manager;
pub mod sync_status;
pub mod system_status;
pub mod tool_manager;
pub mod workflow_list;

// Re-export widget implementations
pub use execution_monitor::ExecutionMonitorWidget;
pub use log_viewer::LogViewerWidget;
pub use plugin_manager::PluginManagerWidget;
pub use sync_status::SyncStatusWidget;
pub use system_status::SystemStatusWidget;
pub use tool_manager::ToolManagerWidget;
pub use workflow_list::WorkflowListWidget;
