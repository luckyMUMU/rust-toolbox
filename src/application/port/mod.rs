//! 应用层端口 - 定义应用层对外暴露的接口

/// 工作流编排器端口
pub trait WorkflowOrchestrator: Send + Sync {
    /// 执行工作流
    fn execute(&self, workflow_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 暂停工作流
    fn pause(&self, workflow_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 恢复工作流
    fn resume(&self, workflow_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// 停止工作流
    fn stop(&self, workflow_id: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// 用例执行器端口
pub trait UseCaseExecutor: Send + Sync {
    /// 执行用例
    fn execute(
        &self,
        use_case: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}
