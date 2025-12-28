//! Property-based tests for retry strategy execution correctness

use crate::core::{ExecutionContext, ExecutionStatus, RetryPolicy, RetryStrategy};
use crate::error::{Result, WorkflowError};
use crate::storage::{StateManager, SimpleMemoryCache, FileStorage};
use crate::tools::ToolRegistry;
use crate::workflow::{
    WorkflowDefinition, WorkflowExecution, WorkflowNode, NodeType,
    NodeExecutionState,
};
use crate::workflow::engine::DefaultWorkflowEngine;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Mock tool registry that can simulate failures for testing retry logic
struct MockRetryToolRegistry {
    /// Map of tool name to failure count - tool will fail this many times before succeeding
    failure_counts: Arc<Mutex<std::collections::HashMap<String, u32>>>,
    /// Map of tool name to current attempt count
    attempt_counts: Arc<Mutex<std::collections::HashMap<String, u32>>>,
}

impl MockRetryToolRegistry {
    fn new() -> Self {
        Self {
            failure_counts: Arc::new(Mutex::new(std::collections::HashMap::new())),
            attempt_counts: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Set a tool to fail for the specified number of attempts before succeeding
    fn set_tool_failure_count(&self, tool_name: &str, failure_count: u32) {
        let mut failures = self.failure_counts.lock().unwrap();
        failures.insert(tool_name.to_string(), failure_count);
        
        let mut attempts = self.attempt_counts.lock().unwrap();
        attempts.insert(tool_name.to_string(), 0);
    }

    /// Get the number of attempts made for a tool
    fn get_attempt_count(&self, tool_name: &str) -> u32 {
        let attempts = self.attempt_counts.lock().unwrap();
        attempts.get(tool_name).copied().unwrap_or(0)
    }
}

#[async_trait]
impl ToolRegistry for MockRetryToolRegistry {
    fn register_tool(&mut self, _tool: Arc<dyn crate::tools::ToolNode>) -> Result<()> {
        Ok(())
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn crate::tools::ToolNode>> {
        None
    }

    fn list_tools(&self) -> Vec<crate::core::ToolInfo> {
        Vec::new()
    }

    async fn execute_tool(&self, name: &str, _params: Value, _context: ExecutionContext) -> Result<Value> {
        // Increment attempt count
        {
            let mut attempts = self.attempt_counts.lock().unwrap();
            let current_attempts = attempts.get(name).copied().unwrap_or(0);
            attempts.insert(name.to_string(), current_attempts + 1);
        }

        // Check if we should fail
        let should_fail = {
            let failures = self.failure_counts.lock().unwrap();
            let attempts = self.attempt_counts.lock().unwrap();
            
            if let (Some(&failure_count), Some(&attempt_count)) = (failures.get(name), attempts.get(name)) {
                attempt_count <= failure_count
            } else {
                false
            }
        };

        if should_fail {
            Err(WorkflowError::tool(format!("Tool {} failed on attempt", name)).into())
        } else {
            Ok(Value::String(format!("Tool {} succeeded", name)))
        }
    }

    fn validate_tool_params(&self, _name: &str, _params: &Value) -> Result<()> {
        Ok(())
    }

    fn has_tool(&self, _name: &str) -> bool {
        true // Mock registry has all tools
    }

    fn unregister_tool(&mut self, _name: &str) -> Result<()> {
        Ok(())
    }

    fn tool_count(&self) -> usize {
        1 // Mock registry always has one tool
    }

    fn clear(&mut self) {
        // Clear attempt counts for mock
        self.attempt_counts.lock().unwrap().clear();
        self.failure_counts.lock().unwrap().clear();
    }
}

/// Helper function to create a test workflow engine with mock tool registry
fn create_test_engine_with_mock_tools() -> (DefaultWorkflowEngine, Arc<MockRetryToolRegistry>) {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(FileStorage::new(temp_dir.path().join("storage")).unwrap());
    let cache = Arc::new(SimpleMemoryCache::new());
    let state_manager = Arc::new(StateManager::new(storage, cache));
    let mock_registry = Arc::new(MockRetryToolRegistry::new());
    let tool_registry = mock_registry.clone() as Arc<dyn ToolRegistry>;
    
    let engine = DefaultWorkflowEngine::new(state_manager, tool_registry, 10);
    (engine, mock_registry)
}

/// Helper function to create a workflow with a single node that has retry policy
fn create_workflow_with_retry(node_id: &str, retry_policy: RetryPolicy) -> WorkflowDefinition {
    let mut workflow = WorkflowDefinition::new("retry_test_workflow", "1.0.0");
    
    let node = WorkflowNode::new(node_id, NodeType::Tool)
        .with_retry_policy(retry_policy);
    
    workflow.add_node(node).unwrap();
    workflow
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit test for basic retry functionality
    #[tokio::test]
    async fn test_retry_policy_basic_functionality() {
        // **Feature: workflow-toolkit, Property 10: Retry strategy execution correctness**
        // *For any* failed task, retry count and intervals should conform to configured retry policy
        // **Validates: Requirements 5.2, 8.1**
        
        let (engine, mock_registry) = create_test_engine_with_mock_tools();
        
        // Set up a tool that fails twice then succeeds
        mock_registry.set_tool_failure_count("test_node", 2);
        
        // Create retry policy with 3 max attempts
        let retry_policy = RetryPolicy {
            strategy: RetryStrategy::FixedInterval,
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Some(Duration::from_secs(1)),
            backoff_multiplier: 2.0,
        };
        
        let workflow = create_workflow_with_retry("test_node", retry_policy);
        let workflow_id = workflow.generate_id();
        
        // Create execution state
        let mut execution = WorkflowExecution {
            id: workflow_id,
            workflow_name: workflow.name.clone(),
            status: ExecutionStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            current_node: Some("test_node".to_string()),
            node_states: std::collections::HashMap::new(),
            global_context: Value::Null,
        };
        
        // Initialize node state
        execution.node_states.insert(
            "test_node".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Pending,
                started_at: None,
                completed_at: None,
                result: None,
                error: None,
                retry_count: 0,
            },
        );
        
        let execution_arc = Arc::new(RwLock::new(execution));
        let context = ExecutionContext::new().with_workflow_id(workflow_id);
        
        // Get the retry policy from the workflow node
        let node_retry_policy = workflow.get_node("test_node")
            .and_then(|node| node.retry_policy.as_ref());
        
        // Execute node with retry
        let result: Result<Value> = engine.execute_node_with_retry(
            "test_node",
            execution_arc.clone(),
            context,
            node_retry_policy,
        ).await;
        
        // Should succeed after retries
        assert!(result.is_ok(), "Node should succeed after retries: {:?}", result);
        
        // Check that exactly 3 attempts were made (2 failures + 1 success)
        let attempt_count = mock_registry.get_attempt_count("test_node");
        assert_eq!(attempt_count, 3, "Should have made exactly 3 attempts");
        
        // Check that retry count was updated in node state
        let execution = execution_arc.read().await;
        let node_state = execution.node_states.get("test_node").unwrap();
        assert_eq!(node_state.retry_count, 3, "Node state should reflect 3 retry attempts");
    }

    /// Unit test for retry exhaustion
    #[tokio::test]
    async fn test_retry_exhaustion() {
        // **Feature: workflow-toolkit, Property 10: Retry strategy execution correctness**
        // *For any* task that fails more than max_attempts, execution should fail
        // **Validates: Requirements 5.2, 8.1**
        
        let (engine, mock_registry) = create_test_engine_with_mock_tools();
        
        // Set up a tool that always fails
        mock_registry.set_tool_failure_count("failing_node", 10); // More failures than max attempts
        
        // Create retry policy with 2 max attempts
        let retry_policy = RetryPolicy {
            strategy: RetryStrategy::FixedInterval,
            max_attempts: 2,
            base_delay: Duration::from_millis(10),
            max_delay: Some(Duration::from_secs(1)),
            backoff_multiplier: 2.0,
        };
        
        let workflow = create_workflow_with_retry("failing_node", retry_policy);
        let workflow_id = workflow.generate_id();
        
        // Create execution state
        let mut execution = WorkflowExecution {
            id: workflow_id,
            workflow_name: workflow.name.clone(),
            status: ExecutionStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            current_node: Some("failing_node".to_string()),
            node_states: std::collections::HashMap::new(),
            global_context: Value::Null,
        };
        
        // Initialize node state
        execution.node_states.insert(
            "failing_node".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Pending,
                started_at: None,
                completed_at: None,
                result: None,
                error: None,
                retry_count: 0,
            },
        );
        
        let execution_arc = Arc::new(RwLock::new(execution));
        let context = ExecutionContext::new().with_workflow_id(workflow_id);
        
        // Get the retry policy from the workflow node
        let node_retry_policy = workflow.get_node("failing_node")
            .and_then(|node| node.retry_policy.as_ref());
        
        // Execute node with retry
        let result: Result<Value> = engine.execute_node_with_retry(
            "failing_node",
            execution_arc.clone(),
            context,
            node_retry_policy,
        ).await;
        
        // Should fail after exhausting retries
        assert!(result.is_err(), "Node should fail after exhausting retries");
        
        // Check that exactly max_attempts were made
        let attempt_count = mock_registry.get_attempt_count("failing_node");
        assert_eq!(attempt_count, 2, "Should have made exactly max_attempts (2) attempts");
        
        // Check that retry count was updated in node state
        let execution = execution_arc.read().await;
        let node_state = execution.node_states.get("failing_node").unwrap();
        assert_eq!(node_state.retry_count, 2, "Node state should reflect max retry attempts");
    }

    /// Unit test for delay calculation
    #[tokio::test]
    async fn test_retry_delay_calculation() {
        // **Feature: workflow-toolkit, Property 10: Retry strategy execution correctness**
        // *For any* retry strategy, delay calculation should follow the specified algorithm
        // **Validates: Requirements 5.2, 8.1**
        
        let (engine, _) = create_test_engine_with_mock_tools();
        
        // Test fixed interval
        let fixed_policy = RetryPolicy {
            strategy: RetryStrategy::FixedInterval,
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Some(Duration::from_secs(10)),
            backoff_multiplier: 2.0,
        };
        
        let delay1 = engine.calculate_retry_delay(&fixed_policy, 1);
        let delay2 = engine.calculate_retry_delay(&fixed_policy, 2);
        let delay3 = engine.calculate_retry_delay(&fixed_policy, 3);
        
        assert_eq!(delay1, Duration::from_millis(100), "Fixed interval should be constant");
        assert_eq!(delay2, Duration::from_millis(100), "Fixed interval should be constant");
        assert_eq!(delay3, Duration::from_millis(100), "Fixed interval should be constant");
        
        // Test exponential backoff
        let exponential_policy = RetryPolicy {
            strategy: RetryStrategy::ExponentialBackoff,
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Some(Duration::from_secs(10)),
            backoff_multiplier: 2.0,
        };
        
        let exp_delay1 = engine.calculate_retry_delay(&exponential_policy, 1);
        let exp_delay2 = engine.calculate_retry_delay(&exponential_policy, 2);
        let exp_delay3 = engine.calculate_retry_delay(&exponential_policy, 3);
        
        assert_eq!(exp_delay1, Duration::from_millis(100), "First attempt should use base delay");
        assert_eq!(exp_delay2, Duration::from_millis(200), "Second attempt should double");
        assert_eq!(exp_delay3, Duration::from_millis(400), "Third attempt should quadruple");
        
        // Test linear backoff
        let linear_policy = RetryPolicy {
            strategy: RetryStrategy::LinearBackoff,
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Some(Duration::from_secs(10)),
            backoff_multiplier: 2.0,
        };
        
        let lin_delay1 = engine.calculate_retry_delay(&linear_policy, 1);
        let lin_delay2 = engine.calculate_retry_delay(&linear_policy, 2);
        let lin_delay3 = engine.calculate_retry_delay(&linear_policy, 3);
        
        assert_eq!(lin_delay1, Duration::from_millis(100), "First attempt should use base delay");
        assert_eq!(lin_delay2, Duration::from_millis(200), "Second attempt should be 2x base");
        assert_eq!(lin_delay3, Duration::from_millis(300), "Third attempt should be 3x base");
    }

    /// Simplified property-based test using basic iteration
    #[tokio::test]
    async fn test_retry_strategy_execution_correctness_property() {
        // **Feature: workflow-toolkit, Property 10: Retry strategy execution correctness**
        // *For any* retry policy and failure pattern, retry behavior should conform to policy
        // **Validates: Requirements 5.2, 8.1**

        let test_cases = vec![
            // (max_attempts, tool_failure_count, should_succeed)
            (3, 0, true),  // No failures, should succeed immediately
            (3, 1, true),  // 1 failure, should succeed on 2nd attempt
            (3, 2, true),  // 2 failures, should succeed on 3rd attempt
            (3, 3, false), // 3 failures, should fail after max attempts
            (3, 5, false), // More failures than max attempts, should fail
            (1, 0, true),  // Single attempt, no failure
            (1, 1, false), // Single attempt, with failure
        ];

        for (max_attempts, tool_failure_count, should_succeed) in test_cases {
            let (engine, mock_registry) = create_test_engine_with_mock_tools();
            
            // Set up tool failure pattern
            mock_registry.set_tool_failure_count("property_test_node", tool_failure_count);
            
            let retry_policy = RetryPolicy {
                strategy: RetryStrategy::FixedInterval,
                max_attempts,
                base_delay: Duration::from_millis(10),
                max_delay: Some(Duration::from_secs(1)),
                backoff_multiplier: 2.0,
            };
            
            let workflow = create_workflow_with_retry("property_test_node", retry_policy.clone());
            let workflow_id = workflow.generate_id();
            
            // Create execution state
            let mut execution = WorkflowExecution {
                id: workflow_id,
                workflow_name: workflow.name.clone(),
                status: ExecutionStatus::Running,
                started_at: Utc::now(),
                completed_at: None,
                current_node: Some("property_test_node".to_string()),
                node_states: std::collections::HashMap::new(),
                global_context: Value::Null,
            };
            
            // Initialize node state
            execution.node_states.insert(
                "property_test_node".to_string(),
                NodeExecutionState {
                    status: ExecutionStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    result: None,
                    error: None,
                    retry_count: 0,
                },
            );
            
            let execution_arc = Arc::new(RwLock::new(execution));
            let context = ExecutionContext::new().with_workflow_id(workflow_id);
            
            // Get the retry policy from the workflow node
            let node_retry_policy = workflow.get_node("property_test_node")
                .and_then(|node| node.retry_policy.as_ref());
            
            // Execute node with retry
            let result: Result<Value> = engine.execute_node_with_retry(
                "property_test_node",
                execution_arc.clone(),
                context,
                node_retry_policy,
            ).await;
            
            let attempt_count = mock_registry.get_attempt_count("property_test_node");
            
            // Property 1: Attempt count should never exceed max_attempts
            assert!(
                attempt_count <= max_attempts,
                "Attempt count ({}) should not exceed max_attempts ({}) for case: max_attempts={}, failures={}",
                attempt_count,
                max_attempts,
                max_attempts,
                tool_failure_count
            );
            
            // Property 2: Result should match expected outcome
            if should_succeed {
                assert!(
                    result.is_ok(),
                    "Execution should succeed for case: max_attempts={}, failures={}",
                    max_attempts,
                    tool_failure_count
                );
                
                // Should make exactly (failure_count + 1) attempts
                let expected_attempts = std::cmp::min(tool_failure_count + 1, max_attempts);
                assert_eq!(
                    attempt_count,
                    expected_attempts,
                    "Should make exactly {} attempts for case: max_attempts={}, failures={}",
                    expected_attempts,
                    max_attempts,
                    tool_failure_count
                );
            } else {
                assert!(
                    result.is_err(),
                    "Execution should fail for case: max_attempts={}, failures={}",
                    max_attempts,
                    tool_failure_count
                );
                
                // Should make exactly max_attempts
                assert_eq!(
                    attempt_count,
                    max_attempts,
                    "Should make exactly max_attempts ({}) when execution fails for case: max_attempts={}, failures={}",
                    max_attempts,
                    max_attempts,
                    tool_failure_count
                );
            }
            
            // Property 3: Retry count in node state should match actual attempts
            let execution = execution_arc.read().await;
            let node_state = execution.node_states.get("property_test_node").unwrap();
            assert_eq!(
                node_state.retry_count,
                attempt_count,
                "Node state retry count should match actual attempts for case: max_attempts={}, failures={}",
                max_attempts,
                tool_failure_count
            );
        }
    }
}