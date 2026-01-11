//! TUI Performance Benchmark Tests
//! 
//! This module implements comprehensive performance benchmark tests for the TUI system,
//! covering rendering performance, memory usage, and response time testing as specified
//! in task 20.3 and requirements 12.1, 12.2, 12.3.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::time::timeout;
use workflow_toolkit::interfaces::tui::{
    SharedAppState, Theme,
    widgets::{
        WorkflowListWidget, ExecutionMonitorWidget, LogViewerWidget,
        ToolManagerWidget, PluginManagerWidget, SystemStatusWidget,
        workflow_list::{WorkflowInfo, WorkflowStatus},
        log_viewer::LogEntry,
    },
    action::LogLevel,
    state::{SystemStatus, SystemHealth, NetworkStatus},
    performance::{RenderingPerformanceManager, RenderingConfig, FrameRateMonitor},
    memory::{TuiMemoryManager, TuiMemoryConfig},
    monitoring::{TuiPerformanceMonitor, MonitoringConfig},
    Widget,
};
use workflow_toolkit::error::Result;
use workflow_toolkit::performance::{PerformanceManager, PerformanceConfig};
use ratatui::{
    backend::TestBackend,
    Terminal,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use chrono::Utc;

/// Performance benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations for each benchmark
    pub iterations: usize,
    
    /// Warmup iterations before actual benchmarking
    pub warmup_iterations: usize,
    
    /// Target frame rate for rendering tests
    pub target_fps: u32,
    
    /// Maximum acceptable memory usage (bytes)
    pub max_memory_usage: usize,
    
    /// Maximum acceptable response time (milliseconds)
    pub max_response_time: u64,
    
    /// Enable detailed profiling
    pub enable_profiling: bool,
    
    /// Test data sizes for scalability testing
    pub test_data_sizes: Vec<usize>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            target_fps: 60,
            max_memory_usage: 50 * 1024 * 1024, // 50MB as per requirement 12.3
            max_response_time: 100, // 100ms for responsive UI
            enable_profiling: true,
            test_data_sizes: vec![10, 100, 1000, 5000, 10000],
        }
    }
}

/// Benchmark results for a specific test
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub median_duration: Duration,
    pub percentile_95: Duration,
    pub percentile_99: Duration,
    pub thro