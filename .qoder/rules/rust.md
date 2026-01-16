---
trigger: always_on
---

# Rust Development Guidelines

You are an expert Rust developer helping with a comprehensive workflow toolkit project. Follow these guidelines when working with this codebase.

## Project Tech Stack

### Core Technologies
- **Language**: Rust 1.70+ (2021 Edition)
- **Async Runtime**: tokio (full async/await support)
- **CLI Framework**: clap with derive features
- **TUI Framework**: ratatui + crossterm
- **Graph Processing**: petgraph (for DAG workflows)
- **Serialization**: serde + serde_json
- **Error Handling**: thiserror + anyhow
- **Caching**: moka with async support
- **Logging**: tracing + tracing-subscriber
- **Testing**: proptest + criterion (for benchmarks)

### Key Dependencies
tokio = { version = "1.35", features = ["full"] }
clap = { version = "4.4", features = ["derive"] }
ratatui = "0.25"
crossterm = "0.27"
serde = { version = "1.0", features = ["derive"] }
petgraph = "0.6"
moka = { version = "0.12", features = ["future"] }
thiserror = "1.0"
tracing = "0.1"

## Code Standards

### Type Safety
// Use strong types
struct WorkflowId(String);
struct Timeout(Duration);

// Use enums for states
enum WorkflowState {
    Created,
    Running,
    Paused,
    Completed,
    Failed(String),
}

### Error Handling
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Workflow not found: {0}")]
    NotFound(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

### Async Programming
use tokio::task;
use tokio::sync::{mpsc, RwLock};

async fn process_workflow(workflow: Workflow) -> Result<()> {
    let result = execute_steps(&workflow).await?;
    save_result(result).await?;
    Ok(())
}

### Memory Management
use std::sync::Arc;

// Use Arc for shared ownership
type SharedData = Arc<RwLock<Data>>;

// Prefer borrowing
fn process_data(data: &Data) -> Result<()> {
    // Process without taking ownership
}

## Project Architecture

### Module Structure
src/
├── cli/              # CLI commands
├── tui/              # Terminal UI
│   ├── widgets/      # Widget system
│   ├── monitoring/   # System monitoring
│   └── maintenance/  # Maintenance tools
├── server/           # MCP server
├── workflow/         # Workflow engine
├── plugin/           # Plugin system
├── file_management/  # File operations
├── system/           # System monitoring
├── storage/          # Persistence
└── config/           # Configuration

## Common Patterns

### Configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub tui: TuiConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(Environment::with_prefix("WORKFLOW_TOOLKIT"))
            .build()?
            .try_deserialize()
    }
}

### TUI Widgets
use ratatui::{Frame, layout::Rect, widgets::Widget};

pub struct CustomWidget {
    data: Vec<String>,
    selected: usize,
}

impl CustomWidget {
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        // Rendering logic
    }
    
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        // Event handling
    }
}

### Plugin System
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn initialize(&mut self) -> Result<()>;
    async fn execute(&self, params: PluginParams) -> Result<PluginResult>;
    async fn cleanup(&mut self) -> Result<()>;
}

## Testing

### Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_execution() {
        let result = execute_workflow("test").await;
        assert!(result.is_ok());
    }
}

### Integration Tests
#[tokio::test]
async fn test_full_workflow() {
    let executor = WorkflowExecutor::new(TestConfig::default());
    let workflow = create_test_workflow();
    let result = executor.execute(workflow).await;
    assert!(result.is_ok());
}

### Performance Tests
use criterion::{black_box, criterion_group, Criterion};

fn benchmark_execution(c: &mut Criterion) {
    c.bench_function("execute_workflow", |b| {
        b.iter(|| execute_workflow(black_box("test")))
    });
}

## Best Practices

1. **Use Builder Pattern** for complex initialization
2. **Implement Display and Debug** for better errors
3. **Use Type Aliases** for complex generics
4. **Prefer Composition** over inheritance (traits)
5. **Document Panics** with `# Panics`
6. **Handle All Errors** - avoid unwrap() in production
7. **Enable Clippy** and fix warnings
8. **Profile Performance** with cargo-flamegraph
9. **Pin Dependencies** in Cargo.toml
10. **Write Comprehensive Tests**

## Code Review Checklist

- [ ] Public APIs documented
- [ ] Comprehensive error handling
- [ ] Tests cover critical paths
- [ ] No clippy warnings
- [ ] Performance considered
- [ ] Proper async cancellation
- [ ] Thread safety verified
- [ ] Resources cleaned up properly
- [ ] Follows project conventions
