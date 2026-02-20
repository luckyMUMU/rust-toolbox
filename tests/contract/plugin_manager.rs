//! 插件管理器契约测试
//!
//! 定义插件管理器必须满足的契约

use std::sync::Arc;

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub status: PluginStatus,
}

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Uninitialized,
    Ready,
    Running,
    Error,
    Shutdown,
}

/// 插件配置
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub enabled: bool,
}

/// 插件管理器契约 trait
///
/// 所有插件管理器实现必须满足此契约
pub trait PluginManagerContract: Send + Sync {
    /// 错误类型
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// 加载插件
    fn load_plugin(
        &self,
        config: PluginConfig,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
    
    /// 卸载插件
    fn unload_plugin(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
    
    /// 获取插件信息
    fn get_plugin(&self, name: &str) -> Option<PluginInfo>;
    
    /// 列出所有插件
    fn list_plugins(&self) -> Vec<PluginInfo>;
    
    /// 检查插件是否已加载
    fn is_loaded(&self, name: &str) -> bool;
    
    /// 获取插件数量
    fn plugin_count(&self) -> usize;
    
    /// 关闭所有插件
    fn shutdown_all(
        &self,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

/// 插件管理器契约测试
pub struct PluginManagerContractTests;

impl PluginManagerContractTests {
    /// 测试：加载插件后应能找到
    pub async fn test_load_and_is_loaded<M: PluginManagerContract>(
        manager: &M,
        config: PluginConfig,
    ) -> Result<(), String> {
        let plugin_name = config.name.clone();
        
        manager
            .load_plugin(config)
            .await
            .map_err(|e| format!("加载失败: {}", e))?;
        
        if !manager.is_loaded(&plugin_name) {
            return Err("加载后插件应存在".to_string());
        }
        
        Ok(())
    }
    
    /// 测试：卸载插件后不应能找到
    pub async fn test_unload<M: PluginManagerContract>(
        manager: &M,
        config: PluginConfig,
    ) -> Result<(), String> {
        let plugin_name = config.name.clone();
        
        manager
            .load_plugin(config)
            .await
            .map_err(|e| format!("加载失败: {}", e))?;
        
        manager
            .unload_plugin(&plugin_name)
            .await
            .map_err(|e| format!("卸载失败: {}", e))?;
        
        if manager.is_loaded(&plugin_name) {
            return Err("卸载后插件不应存在".to_string());
        }
        
        Ok(())
    }
    
    /// 测试：获取不存在的插件应返回 None
    pub fn test_get_nonexistent<M: PluginManagerContract>(
        manager: &M,
    ) -> Result<(), String> {
        if manager.get_plugin("nonexistent_plugin").is_some() {
            return Err("获取不存在的插件应返回 None".to_string());
        }
        
        Ok(())
    }
    
    /// 测试：列出插件应包含已加载的插件
    pub async fn test_list_plugins<M: PluginManagerContract>(
        manager: &M,
        config: PluginConfig,
    ) -> Result<(), String> {
        let plugin_name = config.name.clone();
        
        manager
            .load_plugin(config)
            .await
            .map_err(|e| format!("加载失败: {}", e))?;
        
        let plugins = manager.list_plugins();
        
        if !plugins.iter().any(|p| p.name == plugin_name) {
            return Err("插件列表应包含已加载的插件".to_string());
        }
        
        Ok(())
    }
    
    /// 测试：插件计数应正确
    pub async fn test_plugin_count<M: PluginManagerContract>(
        manager: &M,
        config: PluginConfig,
    ) -> Result<(), String> {
        let initial_count = manager.plugin_count();
        
        manager
            .load_plugin(config.clone())
            .await
            .map_err(|e| format!("加载失败: {}", e))?;
        
        if manager.plugin_count() != initial_count + 1 {
            return Err("加载后插件计数应增加".to_string());
        }
        
        manager
            .unload_plugin(&config.name)
            .await
            .map_err(|e| format!("卸载失败: {}", e))?;
        
        if manager.plugin_count() != initial_count {
            return Err("卸载后插件计数应恢复".to_string());
        }
        
        Ok(())
    }
    
    /// 测试：关闭所有插件
    pub async fn test_shutdown_all<M: PluginManagerContract>(
        manager: &M,
        config: PluginConfig,
    ) -> Result<(), String> {
        manager
            .load_plugin(config.clone())
            .await
            .map_err(|e| format!("加载失败: {}", e))?;
        
        manager
            .shutdown_all()
            .await
            .map_err(|e| format!("关闭失败: {}", e))?;
        
        if manager.plugin_count() != 0 {
            return Err("关闭后插件计数应为 0".to_string());
        }
        
        Ok(())
    }
    
    /// 运行所有契约测试
    pub async fn run_all<M: PluginManagerContract>(
        manager: &M,
        config: PluginConfig,
    ) -> Vec<(String, Result<(), String>)> {
        vec![
            ("get_nonexistent".to_string(), Ok(Self::test_get_nonexistent(manager))),
            ("load_and_is_loaded".to_string(), Self::test_load_and_is_loaded(manager, config.clone()).await),
            ("list_plugins".to_string(), Self::test_list_plugins(manager, config.clone()).await),
            ("plugin_count".to_string(), Self::test_plugin_count(manager, config.clone()).await),
            ("unload".to_string(), Self::test_unload(manager, config.clone()).await),
            ("shutdown_all".to_string(), Self::test_shutdown_all(manager, config).await),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;
    
    struct MockPluginManager {
        plugins: RwLock<HashMap<String, PluginInfo>>,
    }
    
    impl MockPluginManager {
        fn new() -> Self {
            Self {
                plugins: RwLock::new(HashMap::new()),
            }
        }
    }
    
    #[async_trait::async_trait]
    impl PluginManagerContract for MockPluginManager {
        type Error = std::io::Error;
        
        async fn load_plugin(&self, config: PluginConfig) -> Result<(), Self::Error> {
            self.plugins.write().unwrap().insert(
                config.name.clone(),
                PluginInfo {
                    name: config.name,
                    version: "1.0.0".to_string(),
                    status: PluginStatus::Ready,
                },
            );
            Ok(())
        }
        
        async fn unload_plugin(&self, name: &str) -> Result<(), Self::Error> {
            self.plugins.write().unwrap().remove(name);
            Ok(())
        }
        
        fn get_plugin(&self, name: &str) -> Option<PluginInfo> {
            self.plugins.read().unwrap().get(name).cloned()
        }
        
        fn list_plugins(&self) -> Vec<PluginInfo> {
            self.plugins.read().unwrap().values().cloned().collect()
        }
        
        fn is_loaded(&self, name: &str) -> bool {
            self.plugins.read().unwrap().contains_key(name)
        }
        
        fn plugin_count(&self) -> usize {
            self.plugins.read().unwrap().len()
        }
        
        async fn shutdown_all(&self) -> Result<(), Self::Error> {
            self.plugins.write().unwrap().clear();
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_contract_load_and_is_loaded() {
        let manager = MockPluginManager::new();
        let config = PluginConfig {
            name: "test_plugin".to_string(),
            enabled: true,
        };
        let result = PluginManagerContractTests::test_load_and_is_loaded(&manager, config).await;
        assert!(result.is_ok());
    }
}
