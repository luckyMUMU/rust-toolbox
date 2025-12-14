use async_trait::async_trait;
use std::time::Duration;
use crate::{error::Result, config::port::ConfigCachePort}; // 删除未使用的 CoreError 导入
use crate::persistence::PersistenceManager;

/// 基于PersistenceManager的配置缓存适配器
pub struct CacheAdapter {
    /// 持久化管理器实例，用于实现缓存功能
    persistence_manager: std::sync::Arc<PersistenceManager>,
}

impl CacheAdapter {
    /// 创建新的缓存适配器
    /// 
    /// # 参数
    /// * `persistence_manager` - 持久化管理器实例
    /// 
    /// # 返回值
    /// * `Ok(CacheAdapter)` - 成功创建缓存适配器
    pub fn new(persistence_manager: std::sync::Arc<PersistenceManager>) -> Result<Self> {
        Ok(Self {
            persistence_manager,
        })
    }
}

#[async_trait]
impl ConfigCachePort for CacheAdapter {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // 使用PersistenceManager的get_data方法获取缓存数据
        // 注意：这里需要将key转换为缓存键格式，添加前缀避免冲突
        let cache_key = format!("config_cache:{}", key);
        
        // 尝试从缓存获取数据
        match self.persistence_manager.get_data::<Vec<u8>>(&cache_key).await? {
            Some(data) => Ok(Some(data)),
            None => Ok(None),
        }
    }
    
    async fn set(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> Result<()> {
        // 使用PersistenceManager的set_data方法设置缓存数据
        // 注意：这里需要将key转换为缓存键格式，添加前缀避免冲突
        let cache_key = format!("config_cache:{}", key);
        
        // 将数据保存到缓存
        self.persistence_manager.set_data(&cache_key, value).await?;
        
        Ok(())
    }
    
    async fn invalidate(&self, key: &str) -> Result<()> {
        // 注意：当前PersistenceManager没有提供删除数据的方法，所以这里暂时不实现
        // 可以考虑在PersistenceManager中添加delete_data方法来支持缓存失效
        Ok(())
    }
}