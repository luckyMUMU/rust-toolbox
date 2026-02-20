//! 工作流引擎契约测试
//!
//! 定义工作流引擎必须满足的契约

use std::collections::HashMap;
use serde_json::Value;

/// 工作流引擎契约 trait
///
/// 所有工作流引擎实现必须满足此契约
pub trait WorkflowEngineContract: Send + Sync {
    /// 执行工作流的错误类型
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// 执行工作流
    fn execute_workflow(
        &self,
        workflow_id: &str,
        params: HashMap<String, Value>,
    ) -> impl std::future::Future<Output = Result<WorkflowExecutionResult, Self::Error>> + Send;
    
    /// 暂停工作流
    fn pause_workflow(
        &self,
        execution_id: &str,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
    
    /// 恢复工作流
    fn resume_workflow(
        &self,
        execution_id: &str,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
    
    /// 停止工作流
    fn stop_workflow(
        &self,
        execution_id: &str,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
    
    /// 获取工作流状态
    fn get_status(
        &self,
        execution_id: &str,
    ) -> impl std::future::Future<Output = Result<WorkflowStatus, Self::Error>> + Send;
}

/// 工作流执行结果
#[derive(Debug, Clone)]
pub struct WorkflowExecutionResult {
    /// 执行 ID
    pub execution_id: String,
    /// 工作流名称
    pub workflow_name: String,
    /// 执行状态
    pub status: WorkflowStatus,
    /// 输出结果
    pub output: Option<Value>,
    /// 错误信息
    pub error: Option<String>,
}

/// 工作流状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    /// 待执行
    Pending,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 工作流引擎契约测试
pub struct WorkflowEngineContractTests;

impl WorkflowEngineContractTests {
    /// 测试：执行工作流应返回有效的执行 ID
    pub async fn test_execute_returns_execution_id<E: WorkflowEngineContract>(
        engine: &E,
    ) -> Result<(), String> {
        let params = HashMap::new();
        
        let result = engine
            .execute_workflow("test-workflow", params)
            .await
            .map_err(|e| format!("执行失败: {}", e))?;
        
        if result.execution_id.is_empty() {
            return Err("执行 ID 不应为空".to_string());
        }
        
        Ok(())
    }
    
    /// 测试：执行后状态应为运行中或已完成
    pub async fn test_execute_status_is_valid<E: WorkflowEngineContract>(
        engine: &E,
    ) -> Result<(), String> {
        let params = HashMap::new();
        
        let result = engine
            .execute_workflow("test-workflow", params)
            .await
            .map_err(|e| format!("执行失败: {}", e))?;
        
        match result.status {
            WorkflowStatus::Running | WorkflowStatus::Completed => Ok(()),
            _ => Err(format!("无效状态: {:?}", result.status)),
        }
    }
    
    /// 测试：暂停工作流应成功
    pub async fn test_pause_workflow<E: WorkflowEngineContract>(
        engine: &E,
    ) -> Result<(), String> {
        let params = HashMap::new();
        
        let result = engine
            .execute_workflow("test-workflow", params)
            .await
            .map_err(|e| format!("执行失败: {}", e))?;
        
        engine
            .pause_workflow(&result.execution_id)
            .await
            .map_err(|e| format!("暂停失败: {}", e))?;
        
        let status = engine
            .get_status(&result.execution_id)
            .await
            .map_err(|e| format!("获取状态失败: {}", e))?;
        
        if status != WorkflowStatus::Paused {
            return Err(format!("暂停后状态应为 Paused，实际为: {:?}", status));
        }
        
        Ok(())
    }
    
    /// 测试：恢复工作流应成功
    pub async fn test_resume_workflow<E: WorkflowEngineContract>(
        engine: &E,
    ) -> Result<(), String> {
        let params = HashMap::new();
        
        let result = engine
            .execute_workflow("test-workflow", params)
            .await
            .map_err(|e| format!("执行失败: {}", e))?;
        
        engine
            .pause_workflow(&result.execution_id)
            .await
            .map_err(|e| format!("暂停失败: {}", e))?;
        
        engine
            .resume_workflow(&result.execution_id)
            .await
            .map_err(|e| format!("恢复失败: {}", e))?;
        
        let status = engine
            .get_status(&result.execution_id)
            .await
            .map_err(|e| format!("获取状态失败: {}", e))?;
        
        if status == WorkflowStatus::Paused {
            return Err("恢复后状态不应为 Paused".to_string());
        }
        
        Ok(())
    }
    
    /// 测试：停止工作流应成功
    pub async fn test_stop_workflow<E: WorkflowEngineContract>(
        engine: &E,
    ) -> Result<(), String> {
        let params = HashMap::new();
        
        let result = engine
            .execute_workflow("test-workflow", params)
            .await
            .map_err(|e| format!("执行失败: {}", e))?;
        
        engine
            .stop_workflow(&result.execution_id)
            .await
            .map_err(|e| format!("停止失败: {}", e))?;
        
        let status = engine
            .get_status(&result.execution_id)
            .await
            .map_err(|e| format!("获取状态失败: {}", e))?;
        
        if status != WorkflowStatus::Cancelled {
            return Err(format!("停止后状态应为 Cancelled，实际为: {:?}", status));
        }
        
        Ok(())
    }
    
    /// 运行所有契约测试
    pub async fn run_all<E: WorkflowEngineContract>(engine: &E) -> Vec<(String, Result<(), String>)> {
        vec![
            ("execute_returns_execution_id".to_string(), Self::test_execute_returns_execution_id(engine).await),
            ("execute_status_is_valid".to_string(), Self::test_execute_status_is_valid(engine).await),
            ("pause_workflow".to_string(), Self::test_pause_workflow(engine).await),
            ("resume_workflow".to_string(), Self::test_resume_workflow(engine).await),
            ("stop_workflow".to_string(), Self::test_stop_workflow(engine).await),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockWorkflowEngine;
    
    #[async_trait::async_trait]
    impl WorkflowEngineContract for MockWorkflowEngine {
        type Error = std::io::Error;
        
        async fn execute_workflow(
            &self,
            _workflow_id: &str,
            _params: HashMap<String, Value>,
        ) -> Result<WorkflowExecutionResult, Self::Error> {
            Ok(WorkflowExecutionResult {
                execution_id: "exec-123".to_string(),
                workflow_name: "test".to_string(),
                status: WorkflowStatus::Completed,
                output: Some(Value::Null),
                error: None,
            })
        }
        
        async fn pause_workflow(&self, _execution_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        
        async fn resume_workflow(&self, _execution_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        
        async fn stop_workflow(&self, _execution_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        
        async fn get_status(&self, _execution_id: &str) -> Result<WorkflowStatus, Self::Error> {
            Ok(WorkflowStatus::Completed)
        }
    }
    
    #[tokio::test]
    async fn test_contract_execute_returns_id() {
        let engine = MockWorkflowEngine;
        let result = WorkflowEngineContractTests::test_execute_returns_execution_id(&engine).await;
        assert!(result.is_ok());
    }
}
