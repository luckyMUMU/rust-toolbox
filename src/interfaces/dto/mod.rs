//! 数据传输对象 - 适配层与应用层之间的数据传递

/// 工作流执行请求DTO
#[derive(Debug, Clone)]
pub struct ExecuteWorkflowRequest {
    pub workflow_id: String,
    pub parameters: serde_json::Value,
}

/// 工作流执行响应DTO
#[derive(Debug, Clone)]
pub struct ExecuteWorkflowResponse {
    pub success: bool,
    pub message: String,
    pub result: Option<serde_json::Value>,
}
