//! Performance profiling and analysis

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use dashmap::DashMap;

/// Profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    /// Enable profiling
    pub enabled: bool,
    
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    
    /// Maximum number of samples to keep
    pub max_samples: usize,
    
    /// Enable CPU profiling
    pub cpu_profiling: bool,
    
    /// Enable memory profiling
    pub memory_profiling: bool,
    
    /// Enable I/O profiling
    pub io_profiling: bool,
    
    /// Profile output directory
    pub output_directory: String,
    
    /// Automatic profile generation interval
    pub auto_profile_interval: Option<Duration>,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for performance
            sampling_rate: 0.01, // 1% sampling
            max_samples: 10000,
            cpu_profiling: true,
            memory_profiling: true,
            io_profiling: false,
            output_directory: "./profiles".to_string(),
            auto_profile_interval: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

/// Profiler for performance analysis
pub struct Profiler {
    config: Arc<RwLock<ProfilingConfig>>,
    sessions: Arc<DashMap<String, ProfileSession>>,
    samples: Arc<DashMap<String, Vec<ProfileSample>>>,
    call_graph: Arc<RwLock<CallGraph>>,
}

impl Profiler {
    /// Create a new profiler
    pub fn new(config: ProfilingConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            sessions: Arc::new(DashMap::new()),
            samples: Arc::new(DashMap::new()),
            call_graph: Arc::new(RwLock::new(CallGraph::new())),
        }
    }
    
    /// Check if profiling is enabled
    pub async fn is_enabled(&self) -> bool {
        self.config.read().await.enabled
    }
    
    /// Start a profiling session
    pub async fn start_session(&self, component: &str) -> ProfileSession {
        let config = self.config.read().await;
        
        let session = ProfileSession {
            id: uuid::Uuid::new_v4().to_string(),
            component: component.to_string(),
            start_time: Instant::now(),
            samples: Vec::new(),
            call_stack: Vec::new(),
            enabled: config.enabled && self.should_sample(config.sampling_rate).await,
        };
        
        if session.enabled {
            self.sessions.insert(session.id.clone(), session.clone());
        }
        
        session
    }
    
    /// Finish a profiling session
    pub async fn finish_session(&self, session: ProfileSession) -> crate::Result<ProfileData> {
        if !session.enabled {
            return Ok(ProfileData::empty());
        }
        
        let duration = session.start_time.elapsed();
        
        // Remove session from active sessions
        self.sessions.remove(&session.id);
        
        // Store samples
        if !session.samples.is_empty() {
            let mut component_samples = self.samples.entry(session.component.clone()).or_insert_with(Vec::new);
            component_samples.extend(session.samples.clone());
            
            // Limit sample count
            let config = self.config.read().await;
            if component_samples.len() > config.max_samples {
                let len = component_samples.len();
                component_samples.drain(0..len - config.max_samples);
            }
        }
        
        // Update call graph
        if !session.call_stack.is_empty() {
            let mut call_graph = self.call_graph.write().await;
            call_graph.add_call_sequence(&session.call_stack);
        }
        
        Ok(ProfileData {
            session_id: session.id.clone(),
            component: session.component.clone(),
            duration,
            samples: session.samples.clone(),
            call_stack: session.call_stack.clone(),
            memory_usage: self.get_memory_profile(&session).await,
            cpu_usage: self.get_cpu_profile(&session).await,
        })
    }
    
    /// Record a function call
    pub async fn record_call(&self, session: &mut ProfileSession, function_name: &str) {
        if !session.enabled {
            return;
        }
        
        let thread_id = std::thread::current().id();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        thread_id.hash(&mut hasher);
        
        let call_info = CallInfo {
            function_name: function_name.to_string(),
            start_time_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            thread_id_hash: hasher.finish(),
        };
        
        session.call_stack.push(call_info);
    }
    
    /// Record function return
    pub async fn record_return(&self, session: &mut ProfileSession) {
        if !session.enabled || session.call_stack.is_empty() {
            return;
        }
        
        if let Some(call_info) = session.call_stack.pop() {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            
            let duration = Duration::from_millis(current_time.saturating_sub(call_info.start_time_millis));
            
            let sample = ProfileSample {
                timestamp_millis: call_info.start_time_millis,
                function_name: call_info.function_name,
                duration,
                thread_id_hash: call_info.thread_id_hash,
                memory_delta: 0, // Would be calculated in real implementation
                cpu_usage: 0.0,  // Would be calculated in real implementation
            };
            
            session.samples.push(sample);
        }
    }
    
    /// Get profiling report for a component
    pub async fn get_report(&self, component: &str) -> Option<ProfilingReport> {
        let samples = self.samples.get(component)?;
        
        if samples.is_empty() {
            return None;
        }
        
        let total_samples = samples.len();
        let total_duration: Duration = samples.iter().map(|s| s.duration).sum();
        let average_duration = total_duration / total_samples as u32;
        
        // Calculate function statistics
        let mut function_stats: HashMap<String, FunctionStats> = HashMap::new();
        
        for sample in samples.iter() {
            let stats = function_stats.entry(sample.function_name.clone()).or_insert_with(FunctionStats::default);
            stats.call_count += 1;
            stats.total_duration += sample.duration;
            stats.min_duration = stats.min_duration.min(sample.duration);
            stats.max_duration = stats.max_duration.max(sample.duration);
        }
        
        // Calculate averages
        for stats in function_stats.values_mut() {
            stats.average_duration = stats.total_duration / stats.call_count as u32;
        }
        
        // Find hotspots (functions taking most time)
        let mut hotspots: Vec<_> = function_stats.iter()
            .map(|(name, stats)| (name.clone(), stats.total_duration))
            .collect();
        hotspots.sort_by(|a, b| b.1.cmp(&a.1));
        hotspots.truncate(10); // Top 10 hotspots
        
        Some(ProfilingReport {
            component: component.to_string(),
            total_samples,
            total_duration,
            average_duration,
            function_stats,
            hotspots: hotspots.into_iter().map(|(name, duration)| Hotspot { name, duration }).collect(),
            call_graph_summary: self.get_call_graph_summary().await,
        })
    }
    
    /// Generate flame graph data
    pub async fn generate_flame_graph(&self, component: &str) -> Option<FlameGraphData> {
        let samples = self.samples.get(component)?;
        
        if samples.is_empty() {
            return None;
        }
        
        let mut flame_graph = FlameGraphData::new();
        
        // Build flame graph from call samples
        for sample in samples.iter() {
            flame_graph.add_sample(&sample.function_name, sample.duration);
        }
        
        Some(flame_graph)
    }
    
    /// Export profiling data
    pub async fn export_data(&self, format: ExportFormat) -> crate::Result<String> {
        match format {
            ExportFormat::Json => self.export_json().await,
            ExportFormat::FlameGraph => self.export_flame_graph().await,
            ExportFormat::CallGraph => self.export_call_graph().await,
        }
    }
    
    async fn should_sample(&self, sampling_rate: f64) -> bool {
        rand::random::<f64>() < sampling_rate
    }
    
    async fn get_memory_profile(&self, _session: &ProfileSession) -> MemoryProfile {
        // Mock memory profile - in real implementation, would track actual memory usage
        MemoryProfile {
            peak_usage: 1024 * 1024, // 1MB
            average_usage: 512 * 1024, // 512KB
            allocations: 100,
            deallocations: 95,
        }
    }
    
    async fn get_cpu_profile(&self, _session: &ProfileSession) -> CpuProfile {
        // Mock CPU profile - in real implementation, would track actual CPU usage
        CpuProfile {
            user_time: Duration::from_millis(100),
            system_time: Duration::from_millis(20),
            idle_time: Duration::from_millis(880),
        }
    }
    
    async fn get_call_graph_summary(&self) -> CallGraphSummary {
        let call_graph = self.call_graph.read().await;
        
        CallGraphSummary {
            total_nodes: call_graph.nodes.len(),
            total_edges: call_graph.edges.len(),
            max_depth: call_graph.max_depth,
            most_called_functions: call_graph.get_most_called_functions(5),
        }
    }
    
    async fn export_json(&self) -> crate::Result<String> {
        let mut export_data = serde_json::Map::new();
        
        // Export all component reports
        for component_entry in self.samples.iter() {
            let component = component_entry.key();
            if let Some(report) = self.get_report(component).await {
                export_data.insert(component.clone(), serde_json::to_value(report)?);
            }
        }
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }
    
    async fn export_flame_graph(&self) -> crate::Result<String> {
        let mut flame_graph_data = String::new();
        
        for component_entry in self.samples.iter() {
            let component = component_entry.key();
            if let Some(flame_graph) = self.generate_flame_graph(component).await {
                flame_graph_data.push_str(&format!("# Component: {}\n", component));
                flame_graph_data.push_str(&flame_graph.to_string());
                flame_graph_data.push('\n');
            }
        }
        
        Ok(flame_graph_data)
    }
    
    async fn export_call_graph(&self) -> crate::Result<String> {
        let call_graph = self.call_graph.read().await;
        Ok(call_graph.to_dot_format())
    }
}

/// Profile session
#[derive(Debug, Clone)]
pub struct ProfileSession {
    pub id: String,
    pub component: String,
    pub start_time: Instant,
    pub samples: Vec<ProfileSample>,
    pub call_stack: Vec<CallInfo>,
    pub enabled: bool,
}

/// Profile data result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub session_id: String,
    pub component: String,
    pub duration: Duration,
    pub samples: Vec<ProfileSample>,
    pub call_stack: Vec<CallInfo>,
    pub memory_usage: MemoryProfile,
    pub cpu_usage: CpuProfile,
}

impl ProfileData {
    pub fn empty() -> Self {
        Self {
            session_id: String::new(),
            component: String::new(),
            duration: Duration::ZERO,
            samples: Vec::new(),
            call_stack: Vec::new(),
            memory_usage: MemoryProfile::default(),
            cpu_usage: CpuProfile::default(),
        }
    }
}

/// Profile sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSample {
    pub timestamp_millis: u64, // Unix timestamp in milliseconds
    pub function_name: String,
    pub duration: Duration,
    pub thread_id_hash: u64, // Hash of thread ID for serialization
    pub memory_delta: i64,
    pub cpu_usage: f64,
}

/// Call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInfo {
    pub function_name: String,
    pub start_time_millis: u64, // Unix timestamp in milliseconds
    pub thread_id_hash: u64, // Hash of thread ID for serialization
}

/// Memory profile
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryProfile {
    pub peak_usage: usize,
    pub average_usage: usize,
    pub allocations: u64,
    pub deallocations: u64,
}

/// CPU profile
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuProfile {
    pub user_time: Duration,
    pub system_time: Duration,
    pub idle_time: Duration,
}

/// Function statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionStats {
    pub call_count: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

/// Hotspot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub name: String,
    pub duration: Duration,
}

/// Profiling report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    pub component: String,
    pub total_samples: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub function_stats: HashMap<String, FunctionStats>,
    pub hotspots: Vec<Hotspot>,
    pub call_graph_summary: CallGraphSummary,
}

/// Call graph for tracking function relationships
#[derive(Debug, Clone)]
pub struct CallGraph {
    pub nodes: HashMap<String, CallGraphNode>,
    pub edges: Vec<CallGraphEdge>,
    pub max_depth: usize,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            max_depth: 0,
        }
    }
    
    pub fn add_call_sequence(&mut self, call_stack: &[CallInfo]) {
        self.max_depth = self.max_depth.max(call_stack.len());
        
        for (i, call) in call_stack.iter().enumerate() {
            // Add or update node
            let node = self.nodes.entry(call.function_name.clone()).or_insert_with(|| {
                CallGraphNode {
                    name: call.function_name.clone(),
                    call_count: 0,
                    total_duration: Duration::ZERO,
                }
            });
            node.call_count += 1;
            
            // Add edge to next function in stack
            if i + 1 < call_stack.len() {
                let caller = &call.function_name;
                let callee = &call_stack[i + 1].function_name;
                
                if let Some(edge) = self.edges.iter_mut().find(|e| e.from == *caller && e.to == *callee) {
                    edge.call_count += 1;
                } else {
                    self.edges.push(CallGraphEdge {
                        from: caller.clone(),
                        to: callee.clone(),
                        call_count: 1,
                    });
                }
            }
        }
    }
    
    pub fn get_most_called_functions(&self, limit: usize) -> Vec<(String, u64)> {
        let mut functions: Vec<_> = self.nodes.iter()
            .map(|(name, node)| (name.clone(), node.call_count))
            .collect();
        
        functions.sort_by(|a, b| b.1.cmp(&a.1));
        functions.truncate(limit);
        functions
    }
    
    pub fn to_dot_format(&self) -> String {
        let mut dot = String::from("digraph CallGraph {\n");
        
        // Add nodes
        for (name, node) in &self.nodes {
            dot.push_str(&format!("  \"{}\" [label=\"{}\\n({} calls)\"];\n", name, name, node.call_count));
        }
        
        // Add edges
        for edge in &self.edges {
            dot.push_str(&format!("  \"{}\" -> \"{}\" [label=\"{}\"];\n", edge.from, edge.to, edge.call_count));
        }
        
        dot.push_str("}\n");
        dot
    }
}

#[derive(Debug, Clone)]
pub struct CallGraphNode {
    pub name: String,
    pub call_count: u64,
    pub total_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct CallGraphEdge {
    pub from: String,
    pub to: String,
    pub call_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub max_depth: usize,
    pub most_called_functions: Vec<(String, u64)>,
}

/// Flame graph data
#[derive(Debug, Clone)]
pub struct FlameGraphData {
    pub stacks: HashMap<String, Duration>,
}

impl FlameGraphData {
    pub fn new() -> Self {
        Self {
            stacks: HashMap::new(),
        }
    }
    
    pub fn add_sample(&mut self, function_name: &str, duration: Duration) {
        *self.stacks.entry(function_name.to_string()).or_insert(Duration::ZERO) += duration;
    }
    
    pub fn to_string(&self) -> String {
        let mut output = String::new();
        
        for (stack, duration) in &self.stacks {
            output.push_str(&format!("{} {}\n", stack, duration.as_micros()));
        }
        
        output
    }
}

/// Export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    FlameGraph,
    CallGraph,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_profiler_creation() {
        let config = ProfilingConfig::default();
        let profiler = Profiler::new(config);
        
        assert!(!profiler.is_enabled().await);
    }
    
    #[tokio::test]
    async fn test_profiling_session() {
        let mut config = ProfilingConfig::default();
        config.enabled = true;
        config.sampling_rate = 1.0; // Always sample for testing
        
        let profiler = Profiler::new(config);
        
        let mut session = profiler.start_session("test_component").await;
        assert!(session.enabled);
        
        // Simulate function calls
        profiler.record_call(&mut session, "test_function").await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        profiler.record_return(&mut session).await;
        
        let profile_data = profiler.finish_session(session).await.unwrap();
        
        assert_eq!(profile_data.component, "test_component");
        assert!(!profile_data.samples.is_empty());
    }
    
    #[tokio::test]
    async fn test_profiling_report() {
        let mut config = ProfilingConfig::default();
        config.enabled = true;
        config.sampling_rate = 1.0;
        
        let profiler = Profiler::new(config);
        
        // Generate some profile data
        let mut session = profiler.start_session("test_component").await;
        profiler.record_call(&mut session, "function_a").await;
        tokio::time::sleep(Duration::from_millis(5)).await;
        profiler.record_return(&mut session).await;
        
        profiler.record_call(&mut session, "function_b").await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        profiler.record_return(&mut session).await;
        
        profiler.finish_session(session).await.unwrap();
        
        // Get report
        let report = profiler.get_report("test_component").await.unwrap();
        
        assert_eq!(report.component, "test_component");
        assert_eq!(report.total_samples, 2);
        assert!(report.function_stats.contains_key("function_a"));
        assert!(report.function_stats.contains_key("function_b"));
    }
    
    #[tokio::test]
    async fn test_call_graph() {
        let mut call_graph = CallGraph::new();
        
        let call_stack = vec![
            CallInfo {
                function_name: "main".to_string(),
                start_time_millis: chrono::Utc::now().timestamp_millis() as u64,
                thread_id_hash: 12345,
            },
            CallInfo {
                function_name: "process_data".to_string(),
                start_time_millis: chrono::Utc::now().timestamp_millis() as u64,
                thread_id_hash: 12345,
            },
            CallInfo {
                function_name: "validate_input".to_string(),
                start_time_millis: chrono::Utc::now().timestamp_millis() as u64,
                thread_id_hash: 12345,
            },
        ];
        
        call_graph.add_call_sequence(&call_stack);
        
        assert_eq!(call_graph.nodes.len(), 3);
        assert_eq!(call_graph.edges.len(), 2);
        assert_eq!(call_graph.max_depth, 3);
        
        let most_called = call_graph.get_most_called_functions(5);
        assert_eq!(most_called.len(), 3);
    }
    
    #[tokio::test]
    async fn test_flame_graph() {
        let mut flame_graph = FlameGraphData::new();
        
        flame_graph.add_sample("main", Duration::from_millis(100));
        flame_graph.add_sample("process_data", Duration::from_millis(50));
        flame_graph.add_sample("main", Duration::from_millis(20));
        
        assert_eq!(flame_graph.stacks.len(), 2);
        assert_eq!(flame_graph.stacks["main"], Duration::from_millis(120));
        assert_eq!(flame_graph.stacks["process_data"], Duration::from_millis(50));
    }
}