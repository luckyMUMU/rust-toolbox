//! Execution Monitor Widget

/// Execution Monitor Widget implementation
pub struct ExecutionMonitorWidget {
    pub name: String,
}

impl ExecutionMonitorWidget {
    /// Create a new execution monitor widget
    pub fn new() -> Self {
        Self {
            name: "ExecutionMonitor".to_string(),
        }
    }
}

impl Default for ExecutionMonitorWidget {
    fn default() -> Self {
        Self::new()
    }
}
