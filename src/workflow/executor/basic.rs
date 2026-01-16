//! Basic executor implementation.
//!
//! The simplest executor that directly calls the component's execute method
//! without any additional logic.

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::workflow::component::{Component, ComponentOutput};
use crate::workflow::context::DataContext;
use crate::workflow::executor::Executor;
use async_trait::async_trait;

/// Basic executor that directly executes components.
///
/// This is the base executor in any chain. It simply delegates
/// to the component's `execute` method.
pub struct BasicExecutor;

#[async_trait]
impl Executor for BasicExecutor {
    async fn execute(
        &self,
        component: &dyn Component,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput> {
        tracing::debug!(
            component_id = component.id(),
            component_type = %component.component_type(),
            "Executing component"
        );

        component.execute(context, execution_ctx).await
    }

    fn name(&self) -> &str {
        "BasicExecutor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::component::{ComponentStatus, ComponentType};
    use uuid::Uuid;

    struct MockComponent {
        id: String,
        should_fail: bool,
    }

    #[async_trait]
    impl Component for MockComponent {
        fn id(&self) -> &str {
            &self.id
        }

        fn component_type(&self) -> ComponentType {
            ComponentType::Tool
        }

        async fn execute(
            &self,
            _context: &mut DataContext,
            _execution_ctx: &ExecutionContext,
        ) -> Result<ComponentOutput> {
            if self.should_fail {
                Ok(ComponentOutput::failure("Mock failure"))
            } else {
                Ok(ComponentOutput::success())
            }
        }
    }

    #[tokio::test]
    async fn test_basic_executor_success() {
        let executor = BasicExecutor;
        let component = MockComponent {
            id: "test".to_string(),
            should_fail: false,
        };
        let mut context = DataContext::new();
        let exec_ctx = ExecutionContext {
            workflow_id: Uuid::new_v4(),
            execution_id: "exec-1".to_string(),
            node_id: "test".to_string(),
            parameters: serde_json::Value::Null,
            environment: std::collections::HashMap::new(),
        };

        let result = executor.execute(&component, &mut context, &exec_ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap().status.is_success());
    }

    #[tokio::test]
    async fn test_basic_executor_failure() {
        let executor = BasicExecutor;
        let component = MockComponent {
            id: "test".to_string(),
            should_fail: true,
        };
        let mut context = DataContext::new();
        let exec_ctx = ExecutionContext {
            workflow_id: Uuid::new_v4(),
            execution_id: "exec-1".to_string(),
            node_id: "test".to_string(),
            parameters: serde_json::Value::Null,
            environment: std::collections::HashMap::new(),
        };

        let result = executor.execute(&component, &mut context, &exec_ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap().status.is_failure());
    }
}
