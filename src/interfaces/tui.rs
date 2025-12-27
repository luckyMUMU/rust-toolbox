//! TUI interface implementation

use crate::error::Result;

/// TUI interface trait
pub trait TuiInterface: Send + Sync {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
}

/// Basic TUI interface implementation (stub)
pub struct BasicTuiInterface {
    // Implementation will be added later
}

impl BasicTuiInterface {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BasicTuiInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiInterface for BasicTuiInterface {
    fn start(&self) -> Result<()> {
        // Stub implementation
        Ok(())
    }
    
    fn stop(&self) -> Result<()> {
        // Stub implementation
        Ok(())
    }
}