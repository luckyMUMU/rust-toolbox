//! 插件模块
//!
//! 注册插件管理器及相关服务

use crate::di::{AppModule, DiContainer};
use crate::plugins::manager::PluginManager;
use crate::tools::registry::ToolRegistry;
use std::sync::Arc;

/// 插件模块
///
/// 负责注册插件管理器、运行时管理器等服务
pub struct PluginModule {
    auto_load_plugins: bool,
}

impl PluginModule {
    /// 创建新的插件模块
    pub fn new() -> Self {
        Self {
            auto_load_plugins: false,
        }
    }

    /// 设置是否自动加载插件
    pub fn with_auto_load(mut self, auto_load: bool) -> Self {
        self.auto_load_plugins = auto_load;
        self
    }
}

impl Default for PluginModule {
    fn default() -> Self {
        Self::new()
    }
}

impl AppModule for PluginModule {
    fn name(&self) -> &str {
        "plugin"
    }

    fn configure(&self, container: &DiContainer) {
        let auto_load = self.auto_load_plugins;
        
        container.register_factory::<dyn PluginManagerService, _>(move || {
            let tool_registry = container
                .resolve::<dyn ToolRegistryService>();
            
            let manager = if let Some(registry) = tool_registry {
                let registry_impl = registry
                    .as_any()
                    .downcast_ref::<ToolRegistryServiceImpl>()
                    .expect("Invalid ToolRegistry implementation");
                PluginManager::with_tool_registry(registry_impl.registry.clone())
            } else {
                PluginManager::new()
            };
            
            Arc::new(PluginManagerServiceImpl::new(manager, auto_load))
        });
        
        container.register_factory::<dyn RuntimeManagerService, _>(|| {
            Arc::new(RuntimeManagerServiceImpl::new())
        });
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["tools"]
    }
}

/// 插件管理器服务 trait
pub trait PluginManagerService: Send + Sync {
    /// 获取插件数量
    fn plugin_count(&self) -> usize;
    
    /// 是否自动加载
    fn is_auto_load(&self) -> bool;
    
    /// 转换为 Any 类型用于向下转型
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 插件管理器服务实现
struct PluginManagerServiceImpl {
    manager: PluginManager,
    auto_load: bool,
}

impl PluginManagerServiceImpl {
    fn new(manager: PluginManager, auto_load: bool) -> Self {
        Self { manager, auto_load }
    }
}

impl PluginManagerService for PluginManagerServiceImpl {
    fn plugin_count(&self) -> usize {
        self.manager.plugin_count()
    }
    
    fn is_auto_load(&self) -> bool {
        self.auto_load
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 运行时管理器服务 trait
pub trait RuntimeManagerService: Send + Sync {
    /// 获取运行时数量
    fn runtime_count(&self) -> usize;
}

/// 运行时管理器服务实现
struct RuntimeManagerServiceImpl {
    count: usize,
}

impl RuntimeManagerServiceImpl {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl RuntimeManagerService for RuntimeManagerServiceImpl {
    fn runtime_count(&self) -> usize {
        self.count
    }
}

/// 工具注册表服务 trait (共享定义)
pub trait ToolRegistryService: Send + Sync {
    /// 转换为 Any 类型用于向下转型
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 工具注册表服务实现
pub struct ToolRegistryServiceImpl {
    pub registry: Arc<ToolRegistry>,
}

impl ToolRegistryServiceImpl {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }
}

impl ToolRegistryService for ToolRegistryServiceImpl {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_module_creation() {
        let module = PluginModule::new();
        assert_eq!(module.name(), "plugin");
    }

    #[test]
    fn test_plugin_module_dependencies() {
        let module = PluginModule::new();
        let deps = module.dependencies();
        assert!(deps.contains(&"tools"));
    }

    #[test]
    fn test_plugin_module_with_auto_load() {
        let module = PluginModule::new().with_auto_load(true);
        assert!(module.auto_load_plugins);
    }
}
