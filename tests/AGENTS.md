# tests/ - Test Suite

## OVERVIEW
Integration tests, property-based tests, and comprehensive test fixtures. 12 files with 1,697+ lines in tui_standalone_unit_tests.rs.

## TEST TYPES

### Integration Tests
**integration_tests.rs**: End-to-end workflows
- Full workflow execution lifecycle
- Tool registry integration
- State persistence verification
- Error recovery scenarios
- Performance benchmarks

**file_management_integration_tests.rs**: File operations (1,431 lines)
- Classification workflows with AI-powered categorization
- Batch processing with progress tracking
- Human decision integration
- Result validation and verification
- Error recovery scenarios
- Performance benchmarks

### Property-Based Tests
**template_property_tests.rs**: Template engine
- Random parameter generation
- Expansion correctness
- Edge case handling
- Fuzz testing for security

**version_property_tests.rs**: Version resolution
- Dependency conflicts
- Version matching
- Resolution strategies
- Edge cases (circular dependencies, incompatible versions)

**tui_property_tests.rs**: TUI interactions
- Event processing
- State transitions
- Widget rendering
- Keyboard navigation

### Unit Tests
**tui_standalone_unit_tests.rs**: TUI component tests (1,697 lines)
- Widget rendering tests
- Event handling tests
- State management tests
- Navigation tests
- Error recovery tests
- Performance benchmarks

## TEST PATTERNS

### Fixtures
```rust
struct IntegrationTestFixture {
    temp_dir: TempDir,
    config: Arc<ConfigManager>,
    state_manager: Arc<StateManager>,
    tool_registry: Arc<BasicToolRegistry>,
    workflow_engine: Arc<DefaultWorkflowEngine>,
    plugin_manager: Arc<PluginManager>,
}

impl IntegrationTestFixture {
    async fn new() -> Self {
        // Create isolated test environment
        // Initialize all components
        // Return fixture
    }
}
```

### Async Tests
```rust
#[tokio::test]
async fn test_workflow_execution() {
    let fixture = IntegrationTestFixture::new().await;
    let result = fixture.workflow_engine.execute_workflow(workflow).await;
    assert!(result.is_ok());
}
```

### Property Tests
```rust
#[test]
fn test_template_expansion() {
    proptest!(|(params: HashMap<String, String>)| {
        // Test expansion logic
        let expanded = template_engine.expand(&params);
        assert!(expanded.is_ok());
    });
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_performance_batch_processing() {
    let fixture = IntegrationTestFixture::new().await;
    let start = Instant::now();
    
    // Execute large batch
    let result = fixture.batch_processor.process(large_batch).await;
    
    let duration = start.elapsed();
    assert!(result.is_ok());
    assert!(duration < Duration::from_secs(30));
}
```

## TEST COVERAGE

### Unit Tests
- **In-module**: `#[cfg(test)]` in source files
- **Coverage**: ~70% of codebase
- **Focus**: Individual functions and methods

### Integration Tests
- **Directory**: `tests/`
- **Coverage**: End-to-end scenarios
- **Focus**: Component interaction and system behavior

### Property-Based Tests
- **Framework**: Proptest
- **Coverage**: Critical algorithms
- **Focus**: Edge cases and random inputs

### E2E Tests
- **Coverage**: Full system workflows
- **Focus**: User scenarios and real-world usage

## RUNNING TESTS

### All Tests
```bash
cargo test
```

### Integration Tests Only
```bash
cargo test --test integration_tests
cargo test --test file_management_integration_tests
```

### Property-Based Tests
```bash
cargo test property_tests
cargo test template_property_tests
cargo test version_property_tests
cargo test tui_property_tests
```

### Unit Tests
```bash
cargo test --lib
cargo test workflow::tests
cargo test tools::tests
cargo test plugins::tests
```

### TUI Tests
```bash
cargo test tui_standalone_unit_tests
cargo test tui_integration_tests
cargo test tui_performance_tests
```

### With Output
```bash
cargo test -- --nocapture
```

### Single Thread (Debugging)
```bash
cargo test -- --test-threads=1
```

### Specific Test
```bash
cargo test test_workflow_execution
cargo test test_classification_accuracy
cargo test test_batch_processing
```

### Performance Benchmarks
```bash
cargo test --release performance_benchmark
```

## TEST STRUCTURE

### Test Organization
```
tests/
├── integration_tests.rs                    # General integration tests
├── file_management_integration_tests.rs    # File management tests (1,431 lines)
├── template_property_tests.rs              # Template engine tests
├── version_property_tests.rs               # Version resolution tests
├── tui_property_tests.rs                   # TUI interaction tests
├── tui_standalone_unit_tests.rs            # TUI unit tests (1,697 lines)
├── tui_integration_tests.rs                # TUI integration tests
├── tui_performance_benchmark_tests.rs      # TUI performance tests
└── fixtures/                               # Test fixtures and data
```

### Test Data
```
tests/fixtures/
├── workflows/                              # Test workflow definitions
├── classification_rules/                   # Test classification rules
├── batch_definitions/                      # Test batch definitions
├── plugin_examples/                        # Test plugin examples
└── expected_results/                       # Expected test outputs
```

## TEST SCENARIOS

### Workflow Execution
1. **Basic Execution**: Single workflow with simple steps
2. **Parallel Execution**: Multiple concurrent workflows
3. **Error Recovery**: Workflow with failures and recovery
4. **State Persistence**: Workflow state across restarts
5. **Checkpoint Recovery**: Resume from checkpoint
6. **Timeout Handling**: Workflow timeout scenarios

### File Management
1. **Classification**: AI-powered folder categorization
2. **Batch Processing**: Large batch operations
3. **Human Decision**: Interactive decision scenarios
4. **Merge Operations**: Folder merging with conflicts
5. **Text Processing**: Chinese text processing
6. **Error Recovery**: File operation failures

### TUI Components
1. **Widget Rendering**: Visual rendering tests
2. **Event Handling**: Keyboard and mouse events
3. **Navigation**: View transitions and history
4. **State Management**: Shared state updates
5. **Error Display**: Error dialog rendering
6. **Performance**: Rendering performance under load

### Tool System
1. **Registration**: Tool registration and lookup
2. **Execution**: Tool execution with parameters
3. **Validation**: Parameter validation
4. **Versioning**: Dependency resolution
5. **Templates**: Parameter template expansion
6. **Composition**: Tool composition patterns

### Plugin System
1. **Loading**: Plugin loading and initialization
2. **Lifecycle**: Plugin start/stop/reload
3. **Sandboxing**: Security and isolation
4. **Resource Management**: Memory and file handles
5. **Error Handling**: Plugin failure scenarios
6. **Integration**: Plugin workflow integration

## PERFORMANCE BENCHMARKS

### Workflow Performance
- **Sequential Execution**: Baseline performance
- **Parallel Execution**: Scalability testing
- **Checkpoint Overhead**: State persistence cost
- **Memory Usage**: Resource consumption

### File Management Performance
- **Classification Speed**: Pattern matching performance
- **Batch Processing**: Throughput under load
- **Memory Efficiency**: Large dataset handling
- **I/O Performance**: File operation speed

### TUI Performance
- **Render Time**: Frame rate under load
- **Memory Usage**: Widget memory consumption
- **Event Processing**: Input responsiveness
- **Virtual Scrolling**: Large list performance

## TEST FIXTURES

### IntegrationTestFixture
```rust
pub struct IntegrationTestFixture {
    pub temp_dir: TempDir,
    pub config: Arc<ConfigManager>,
    pub state_manager: Arc<StateManager>,
    pub tool_registry: Arc<BasicToolRegistry>,
    pub workflow_engine: Arc<DefaultWorkflowEngine>,
    pub plugin_manager: Arc<PluginManager>,
    pub performance_manager: Arc<PerformanceManager>,
}

impl IntegrationTestFixture {
    pub async fn new() -> Self {
        // Create temporary directory
        let temp_dir = tempfile::tempdir().unwrap();
        
        // Initialize configuration
        let config = Arc::new(ConfigManager::new());
        
        // Initialize storage
        let storage = Arc::new(FileStorage::new(temp_dir.path().join("storage")).unwrap());
        let cache = Arc::new(LocalMemoryCache::new(CacheConfig::default()));
        let state_manager = Arc::new(StateManager::new(storage, cache));
        
        // Initialize tool registry
        let tool_registry = Arc::new(BasicToolRegistry::new());
        
        // Initialize workflow engine
        let workflow_engine = Arc::new(
            DefaultWorkflowEngine::new(
                config.clone(),
                tool_registry.clone(),
                state_manager.clone(),
            ).unwrap()
        );
        
        // Initialize plugin manager
        let plugin_manager = Arc::new(PluginManager::new());
        
        // Initialize performance manager
        let performance_manager = Arc::new(PerformanceManager::new(PerformanceConfig::default()));
        
        Self {
            temp_dir,
            config,
            state_manager,
            tool_registry,
            workflow_engine,
            plugin_manager,
            performance_manager,
        }
    }
}
```

### FileManagementTestFixture
```rust
pub struct FileManagementTestFixture {
    pub temp_dir: TempDir,
    pub plugin: Arc<FileManagementPlugin>,
    pub rules: ClassificationRules,
    pub test_data: PathBuf,
}

impl FileManagementTestFixture {
    pub async fn new() -> Self {
        // Create test data directory
        let temp_dir = tempfile::tempdir().unwrap();
        let test_data = temp_dir.path().join("test_data");
        std::fs::create_dir_all(&test_data).unwrap();
        
        // Initialize file management plugin
        let config = FileManagementConfig {
            directory: test_data.clone(),
            rules_file: None,
            experimental_mode: true,
            enable_human_interaction: false,
            decision_timeout: Duration::from_secs(30),
            batch_size: 10,
            max_concurrent: 4,
        };
        
        let plugin = Arc::new(
            FileManagementPlugin::builder()
                .config(config)
                .build()
                .unwrap()
        );
        
        // Create test classification rules
        let rules = ClassificationRules::default();
        
        Self {
            temp_dir,
            plugin,
            rules,
            test_data,
        }
    }
}
```

## ERROR HANDLING IN TESTS

### Test Failures
- **Assertion Failures**: Clear error messages
- **Timeout Failures**: Configurable timeouts
- **Resource Leaks**: Detect and report leaks
- **State Corruption**: Verify state integrity

### Test Isolation
- **Temporary Directories**: Each test gets isolated directory
- **Database Isolation**: Separate databases per test
- **Port Isolation**: Unique ports for network tests
- **Process Isolation**: Separate processes when needed

### Cleanup
- **Automatic Cleanup**: Temp directories cleaned on drop
- **Resource Release**: Explicit cleanup in fixtures
- **Leak Detection**: Detect unclosed resources
- **State Verification**: Verify clean state after tests

## TEST DATA GENERATION

### Workflow Generation
```rust
pub fn generate_test_workflow(name: &str, steps: usize) -> WorkflowDefinition {
    // Generate workflow with specified number of steps
}
```

### Classification Rules Generation
```rust
pub fn generate_test_rules(categories: usize, keywords_per_category: usize) -> ClassificationRules {
    // Generate random classification rules
}
```

### Batch Data Generation
```rust
pub fn generate_test_batch(size: usize) -> BatchDefinition {
    // Generate batch with specified number of items
}
```

## TEST REPORTING

### Test Output
- **Verbose**: Detailed test execution logs
- **Summary**: Test pass/fail counts
- **Performance**: Benchmark results
- **Coverage**: Code coverage reports (with grcov)

### Test Artifacts
- **Logs**: Test execution logs
- **Screenshots**: TUI test screenshots
- **Traces**: Execution traces for debugging
- **Metrics**: Performance metrics

## CONTINUOUS INTEGRATION

### CI Pipeline
1. **Unit Tests**: Fast feedback on code changes
2. **Integration Tests**: Verify component interaction
3. **Property Tests**: Catch edge cases
4. **Performance Tests**: Ensure no regressions
5. **Coverage Check**: Maintain test coverage

### Test Requirements
- **Minimum Coverage**: 70% code coverage
- **No Regressions**: All tests must pass
- **Performance**: No significant performance regressions
- **Documentation**: Tests must be documented

## BEST PRACTICES

### Test Design
1. **Isolation**: Tests must not depend on each other
2. **Deterministic**: Tests must be reproducible
3. **Fast**: Unit tests < 100ms, integration < 5s
4. **Clear**: Test names describe what they test
5. **Complete**: Cover success and failure cases

### Test Data
1. **Realistic**: Use realistic test data
2. **Isolated**: Each test gets fresh data
3. **Cleanup**: Clean up test data after execution
4. **Variety**: Test with different data sizes

### Test Maintenance
1. **Regular Review**: Review and update tests regularly
2. **Remove Flaky Tests**: Fix or remove unreliable tests
3. **Update Documentation**: Keep test docs current
4. **Track Coverage**: Monitor and improve coverage

## DEBUGGING TESTS

### Test Failures
```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run with logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name
```

### Test Profiling
```bash
# Profile test execution
cargo test --release -- --profile-time=10

# Memory profiling
valgrind --tool=memcheck cargo test test_name
```

## SEE ALSO

- [Root AGENTS.md](../AGENTS.md) - Project overview
- [Workflow AGENTS.md](../src/workflow/AGENTS.md) - Workflow testing
- [File Management AGENTS.md](../src/plugins/file_management/AGENTS.md) - File management testing
- [TUI AGENTS.md](../src/interfaces/tui/AGENTS.md) - TUI testing
