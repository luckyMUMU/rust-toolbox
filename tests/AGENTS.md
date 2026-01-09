# tests/ - Test Suite

## OVERVIEW
Integration tests, property-based tests, and comprehensive test fixtures.

## TEST TYPES

### Integration Tests
**integration_tests.rs**: End-to-end workflows
- Full workflow execution lifecycle
- Tool registry integration
- State persistence verification
- Error recovery scenarios

**file_management_integration_tests.rs**: File operations
- Classification workflows
- Batch processing
- Human decision integration
- Result validation

### Property-Based Tests
**template_property_tests.rs**: Template engine
- Random parameter generation
- Expansion correctness
- Edge case handling

**version_property_tests.rs**: Version resolution
- Dependency conflicts
- Version matching
- Resolution strategies

**tui_property_tests.rs**: TUI interactions
- Event processing
- State transitions
- Widget rendering

## TEST PATTERNS

### Fixtures
```rust
struct IntegrationTestFixture {
    temp_dir: TempDir,
    config: Arc<ConfigManager>,
    state_manager: Arc<StateManager>,
    tool_registry: Arc<BasicToolRegistry>,
    workflow_engine: Arc<DefaultWorkflowEngine>,
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
    });
}
```

## RUNNING TESTS
```bash
# All tests
cargo test

# Integration only
cargo test --test integration_tests
cargo test --test file_management_integration_tests

# Property-based
cargo test property_tests

# Specific module
cargo test workflow::tests

# With output
cargo test -- --nocapture

# Single thread (for debugging)
cargo test -- --test-threads=1
```

## TEST COVERAGE
- **Unit**: In-module `#[cfg(test)]`
- **Integration**: `tests/` directory
- **Property**: Randomized input testing
- **E2E**: Full system workflows

## BEST PRACTICES
1. Use `tempfile::TempDir` for isolation
2. Clean up resources in fixtures
3. Test async code with `#[tokio::test]`
4. Use descriptive test names
5. Include error cases
6. Verify state after execution
