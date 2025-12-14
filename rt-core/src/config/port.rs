use async_trait::async_trait;
use std::time::Duration;
use serde::{Serialize, de::DeserializeOwned}; // 添加 DeserializeOwned 导入
use crate::{error::Result, config::domain::{ConfigItem, ConfigSource}};

// ====================== 输入端口 ======================

/// 配置管理端口，定义外部系统调用配置管理模块的接口
#[async_trait]
pub trait ConfigManagerPort: Send + Sync {
    /// 获取配置项，支持类型安全的配置访问
    /// 
    /// # 参数
    /// * `key` - 配置项键名
    /// 
    /// # 返回值
    /// * `Ok(Some(T))` - 成功获取配置项并转换为指定类型
    /// * `Ok(None)` - 配置项不存在
    /// * `Err(CoreError)` - 获取配置项失败
    async fn get_config<T: DeserializeOwned + Serialize + Send>(&self, key: &str) -> Result<Option<T>>;
    
    /// 设置配置项
    /// 
    /// # 参数
    /// * `key` - 配置项键名
    /// * `value` - 配置项值，支持多种类型
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功设置配置项
    /// * `Err(CoreError)` - 设置配置项失败
    async fn set_config<T: Serialize + Sync>(&self, key: &str, value: &T) -> Result<()>;
    
    /// 手动重载配置
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功重载配置
    /// * `Err(CoreError)` - 重载配置失败
    async fn reload_config(&self) -> Result<()>;
    
    /// 启用配置文件监控，支持配置热重载
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功启用配置监控
    /// * `Err(CoreError)` - 启用配置监控失败
    async fn watch_config(&self) -> Result<()>;
}

// ====================== 输出端口 ======================

/// 配置源端口，定义配置管理模块从外部获取配置的接口
#[async_trait]
pub trait ConfigSourcePort: Send + Sync {
    /// 从配置源加载配置
    /// 
    /// # 返回值
    /// * `Ok(HashMap<String, String>)` - 成功加载配置，返回配置键值对
    /// * `Err(CoreError)` - 加载配置失败
    async fn load_config(&self) -> Result<std::collections::HashMap<String, String>>;
    
    /// 获取配置源类型
    /// 
    /// # 返回值
    /// 配置源类型枚举
    fn get_source_type(&self) -> ConfigSource;
    
    /// 获取配置源优先级，值越大优先级越高
    /// 
    /// # 返回值
    /// 配置源优先级（0-255）
    fn get_priority(&self) -> u8;
}

/// 配置缓存端口，定义配置缓存的接口
#[async_trait]
pub trait ConfigCachePort: Send + Sync {
    /// 从缓存中获取配置项
    /// 
    /// # 参数
    /// * `key` - 配置项键名
    /// 
    /// # 返回值
    /// * `Ok(Some(Vec<u8>))` - 成功获取缓存的配置项
    /// * `Ok(None)` - 缓存中不存在该配置项
    /// * `Err(CoreError)` - 获取缓存失败
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// 设置缓存配置项
    /// 
    /// # 参数
    /// * `key` - 配置项键名
    /// * `value` - 配置项值（二进制数据）
    /// * `ttl` - 缓存过期时间，None表示永不过期
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功设置缓存
    /// * `Err(CoreError)` - 设置缓存失败
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    
    /// 使缓存中的配置项失效
    /// 
    /// # 参数
    /// * `key` - 配置项键名
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功使缓存失效
    /// * `Err(CoreError)` - 使缓存失效失败
    async fn invalidate(&self, key: &str) -> Result<()>;
}

/// 配置仓库端口，定义配置持久化的接口
#[async_trait]
pub trait ConfigRepositoryPort: Send + Sync {
    /// 保存配置项列表
    /// 
    /// # 参数
    /// * `items` - 配置项列表
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功保存配置项
    /// * `Err(CoreError)` - 保存配置项失败
    async fn save_config(&self, items: Vec<ConfigItem>) -> Result<()>;
    
    /// 获取单个配置项
    /// 
    /// # 参数
    /// * `key` - 配置项键名
    /// 
    /// # 返回值
    /// * `Ok(Some(ConfigItem))` - 成功获取配置项
    /// * `Ok(None)` - 配置项不存在
    /// * `Err(CoreError)` - 获取配置项失败
    async fn get_config(&self, key: &str) -> Result<Option<ConfigItem>>;
    
    /// 获取所有配置项
    /// 
    /// # 返回值
    /// * `Ok(Vec<ConfigItem>)` - 成功获取所有配置项
    /// * `Err(CoreError)` - 获取配置项失败
    async fn get_all_config(&self) -> Result<Vec<ConfigItem>>;
    
    /// 更新单个配置项
    /// 
    /// # 参数
    /// * `item` - 配置项
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功更新配置项
    /// * `Err(CoreError)` - 更新配置项失败
    async fn update_config(&self, item: ConfigItem) -> Result<()>;
}