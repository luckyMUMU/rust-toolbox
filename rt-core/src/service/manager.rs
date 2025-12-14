use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serde_json::Value;
use crate::{error::{Result, CoreError}, service::{ServicePort, ServiceRequest, ServiceResponse, ResponseStatus, ServiceContext, CallerType, PermissionLevel}};
use uuid::Uuid;

/// 服务管理器，负责管理所有服务实例并提供统一的服务调用入口
pub struct ServiceManager {
    /// 服务注册表，存储所有已注册的服务实例
    services: RwLock<HashMap<String, Arc<dyn ServicePort>>>,
    
    /// 默认权限策略
    default_permission_policy: PermissionPolicy,
}

/// 权限策略
pub struct PermissionPolicy {
    /// 调用者类型对应的默认权限级别
    pub default_permissions: HashMap<CallerType, PermissionLevel>,
    
    /// 服务方法对应的最小权限要求
    pub method_permissions: HashMap<(String, String), PermissionLevel>,
}

impl PermissionPolicy {
    /// 创建默认权限策略
    pub fn default() -> Self {
        let mut default_permissions = HashMap::new();
        default_permissions.insert(CallerType::System, PermissionLevel::Admin);
        default_permissions.insert(CallerType::CoreTool, PermissionLevel::Standard);
        default_permissions.insert(CallerType::Plugin, PermissionLevel::ReadOnly);
        default_permissions.insert(CallerType::User, PermissionLevel::Standard);
        
        let mut method_permissions = HashMap::new();
        // 配置服务权限
        method_permissions.insert(("config".to_string(), "set_config".to_string()), PermissionLevel::Admin);
        method_permissions.insert(("config".to_string(), "reload_config".to_string()), PermissionLevel::Admin);
        
        Self {
            default_permissions,
            method_permissions,
        }
    }
    
    /// 检查权限
    pub fn check_permission(&self, context: &ServiceContext, service_name: &str, method: &str) -> bool {
        // 获取该服务方法的最小权限要求
        let min_permission = self.method_permissions.get(&(service_name.to_string(), method.to_string()))
            .unwrap_or_else(|| self.default_permissions.get(&context.caller_type)
                .unwrap_or(&PermissionLevel::ReadOnly));
        
        // 检查调用者权限是否满足要求
        &context.permission_level >= min_permission
    }
}

impl ServiceManager {
    /// 创建新的服务管理器实例
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            default_permission_policy: PermissionPolicy::default(),
        }
    }
    
    /// 注册服务实例
    pub async fn register_service(&self, service: Arc<dyn ServicePort>) {
        let mut services = self.services.write().await;
        services.insert(service.name().to_string(), service);
    }
    
    /// 注销服务实例
    pub async fn unregister_service(&self, service_name: &str) {
        let mut services = self.services.write().await;
        services.remove(service_name);
    }
    
    /// 获取服务实例
    pub async fn get_service(&self, service_name: &str) -> Option<Arc<dyn ServicePort>> {
        let services = self.services.read().await;
        services.get(service_name).cloned()
    }
    
    /// 列出所有已注册的服务
    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }
    
    /// 调用服务方法
    pub async fn call_service(
        &self,
        request: ServiceRequest
    ) -> Result<ServiceResponse> {
        let trace_id = Uuid::new_v4().to_string();
        
        // 1. 检查权限
        if !self.default_permission_policy.check_permission(
            &request.context,
            &request.service_name,
            &request.method
        ) {
            return Ok(ServiceResponse {
                status: ResponseStatus::Unauthorized,
                data: None,
                error: Some("Permission denied".to_string()),
                trace_id,
            });
        }
        
        // 2. 获取服务实例
        let service = self.get_service(&request.service_name).await
            .ok_or_else(|| CoreError::ConfigError(format!("Service not found: {}", request.service_name)))?;
        
        // 3. 调用服务方法
        match service.call(&request.method, &request.params, &request.context).await {
            Ok(mut response) => {
                // 添加跟踪ID
                response.trace_id = trace_id;
                Ok(response)
            },
            Err(e) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Failure,
                    data: None,
                    error: Some(e.to_string()),
                    trace_id,
                })
            }
        }
    }
    
    /// 设置权限策略
    pub fn set_permission_policy(&mut self, policy: PermissionPolicy) {
        self.default_permission_policy = policy;
    }
}

/// 服务工厂，用于创建各种服务实例
pub struct ServiceFactory {
    /// 配置服务实例
    config_service: Option<Arc<dyn ServicePort>>,
    
    /// 日志服务实例
    log_service: Option<Arc<dyn ServicePort>>,
    
    /// 工具服务实例
    tool_service: Option<Arc<dyn ServicePort>>,
}

impl ServiceFactory {
    /// 创建新的服务工厂实例
    pub fn new() -> Self {
        Self {
            config_service: None,
            log_service: None,
            tool_service: None,
        }
    }
    
    /// 注册配置服务
    pub fn register_config_service(&mut self, service: Arc<dyn ServicePort>) {
        self.config_service = Some(service);
    }
    
    /// 注册日志服务
    pub fn register_log_service(&mut self, service: Arc<dyn ServicePort>) {
        self.log_service = Some(service);
    }
    
    /// 注册工具服务
    pub fn register_tool_service(&mut self, service: Arc<dyn ServicePort>) {
        self.tool_service = Some(service);
    }
    
    /// 创建服务管理器实例
    pub async fn create_service_manager(&self) -> Result<ServiceManager> {
        let manager = ServiceManager::new();
        
        // 注册所有已配置的服务
        if let Some(config_service) = &self.config_service {
            manager.register_service(config_service.clone()).await;
        }
        
        if let Some(log_service) = &self.log_service {
            manager.register_service(log_service.clone()).await;
        }
        
        if let Some(tool_service) = &self.tool_service {
            manager.register_service(tool_service.clone()).await;
        }
        
        Ok(manager)
    }
}
