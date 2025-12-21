use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize}; // 添加 DeserializeOwned 导入，删除未使用的 Deserialize
use std::sync::{Arc, Mutex};
use chrono::Utc;
use crate::{error::Result, config::{domain::{ConfigItem, ConfigSource}, port::{ConfigSourcePort, ConfigCachePort, ConfigRepositoryPort, ConfigManagerPort}}}; 

/// 配置服务，实现配置管理的核心业务逻辑
pub struct ConfigService {
    /// 配置源列表，按优先级排序
    sources: Mutex<Vec<Arc<Box<dyn ConfigSourcePort>>>>,
    
    /// 配置仓库，用于持久化配置
    repository: Option<Arc<dyn ConfigRepositoryPort>>,
    
    /// 配置缓存，用于加速配置访问
    cache: Option<Arc<dyn ConfigCachePort>>,
    
    /// 已加载的配置项，存储在内存中
    config_items: Mutex<std::collections::HashMap<String, ConfigItem>>,
}

impl ConfigService {
    /// 创建新的配置服务实例
    /// 
    /// # 返回值
    /// * `Ok(ConfigService)` - 成功创建配置服务
    pub fn new() -> Result<Self> {
        Ok(Self {
            sources: Mutex::new(Vec::new()),
            repository: None,
            cache: None,
            config_items: Mutex::new(std::collections::HashMap::new()),
        })
    }
    
    /// 设置配置仓库
    /// 
    /// # 参数
    /// * `repository` - 配置仓库实例
    pub fn set_repository(&mut self, repository: Arc<dyn ConfigRepositoryPort>) {
        self.repository = Some(repository);
    }
    
    /// 设置配置缓存
    /// 
    /// # 参数
    /// * `cache` - 配置缓存实例
    pub fn set_cache(&mut self, cache: Arc<dyn ConfigCachePort>) {
        self.cache = Some(cache);
    }
    
    /// 添加配置源
    /// 
    /// # 参数
    /// * `source` - 配置源实例
    pub fn add_source(&self, source: Box<dyn ConfigSourcePort>) {
        let mut sources = self.sources.lock().unwrap();
        sources.push(Arc::new(source));
        
        // 按优先级排序，优先级高的放在前面
        sources.sort_by_key(|source| std::cmp::Reverse(source.get_priority()));
    }
    
    /// 从所有配置源加载配置
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功加载配置
    /// * `Err(CoreError)` - 加载配置失败
    pub async fn load_config(&self) -> Result<()> {
        // 1. 克隆所有配置源，避免在异步调用中持有锁
        let sources: Vec<Arc<Box<dyn ConfigSourcePort>>> = {
            let sources_lock = self.sources.lock().unwrap();
            // 使用 Vec<Arc<Box<dyn ConfigSourcePort>>> 来存储配置源的引用
            sources_lock.iter().cloned().collect()
        };
        
        // 2. 从每个配置源加载配置，保存到临时变量
        let mut all_configs: Vec<(std::collections::HashMap<String, String>, ConfigSource, u8)> = Vec::new();
        for source in sources {
            // 显式指定 source 的类型
            let source: &Arc<Box<dyn ConfigSourcePort>> = &source;
            
            // 获取配置源类型和优先级
            let source_type: ConfigSource = source.get_source_type();
            let source_priority: u8 = source.get_priority();
            
            // 显式指定 load_config 的返回类型
            let config_map: std::collections::HashMap<String, String> = source.load_config().await?;
            all_configs.push((config_map, source_type, source_priority));
        }
        
        // 3. 更新内存配置
        let mut all_keys_to_invalidate: Vec<String> = Vec::new();
        let items = {
            let mut config_items = self.config_items.lock().unwrap();
            
            // 清空现有配置
            config_items.clear();
            
            // 合并所有配置源的配置
            for (config_map, source_type, source_priority) in all_configs {
                for (key, value) in config_map {
                    // 直接插入，因为配置源已经按优先级排序
                    let config_item = ConfigItem {
                        key: key.clone(),
                        value: value.clone(),
                        source: source_type,
                        priority: source_priority,
                        updated_at: Utc::now(),
                    };
                    config_items.insert(key.clone(), config_item);
                    
                    // 记录需要清除缓存的键（保存为 String）
                    all_keys_to_invalidate.push(key.clone());
                }
            }
            
            // 返回所有配置项，用于持久化
            config_items.values().cloned().collect::<Vec<ConfigItem>>()
        };
        
        // 4. 清除所有缓存
        if let Some(cache) = &self.cache {
            for key in all_keys_to_invalidate {
                cache.invalidate(&key).await?;
            }
        }
        
        // 5. 持久化配置
        if let Some(repository) = &self.repository {
            repository.save_config(items).await?;
        }
        
        Ok(())
    }
    
    /// 获取配置项
    /// 
    /// # 参数
    /// * `key` - 配置项键名
    /// 
    /// # 返回值
    /// * `Ok(Some<T>)` - 成功获取配置项并转换为指定类型
    /// * `Ok(None)` - 配置项不存在
    /// * `Err(CoreError)` - 获取配置项失败
    pub async fn get_config<T: DeserializeOwned + Serialize + Send>(&self, key: &str) -> Result<Option<T>> {
        // 先从缓存获取
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(key).await? {
                // 缓存命中，反序列化并返回
                let value: T = serde_json::from_slice(&cached)?;
                return Ok(Some(value));
            }
        }
        
        // 缓存未命中，从内存获取
        let json_value = {
            let config_items = self.config_items.lock().unwrap();
            config_items.get(key).map(|item| item.value.clone())
        };
        
        if let Some(json_str) = json_value {
            // 反序列化配置值
            let value: T = serde_json::from_str(&json_str)?;
            
            // 写入缓存
            if let Some(cache) = &self.cache {
                let json_str = serde_json::to_string(&value)?;
                cache.set(key, json_str.as_bytes(), None).await?;
            }
            
            return Ok(Some(value));
        }
        
        // 配置项不存在
        Ok(None)
    }
    
    /// 设置配置项
    /// 
    /// # 参数
    /// * `key` - 配置项键名
    /// * `value` - 配置项值
    /// * `source` - 配置来源
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功设置配置项
    /// * `Err(CoreError)` - 设置配置项失败
    pub async fn set_config<T: Serialize + Sync>(&self, key: &str, value: &T, source: ConfigSource) -> Result<()> {
        // 序列化配置值
        let json_str = serde_json::to_string(value)?;
        
        // 更新内存配置
        let config_item = {
            let mut config_items = self.config_items.lock().unwrap();
            let config_item = ConfigItem {
                key: key.to_string(),
                value: json_str.clone(),
                source,
                priority: 0, // 默认优先级为 0
                updated_at: Utc::now(),
            };
            
            config_items.insert(key.to_string(), config_item.clone());
            config_item
        };
        
        // 复制 key，以便在异步调用中使用
        let key_clone = key.to_string();
        
        // 清除缓存
        if let Some(cache) = &self.cache {
            cache.invalidate(&key_clone).await?;
        }
        
        // 持久化配置
        if let Some(repository) = &self.repository {
            repository.update_config(config_item).await?;
        }
        
        Ok(())
    }
    
    /// 手动重载配置
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功重载配置
    /// * `Err(CoreError)` - 重载配置失败
    pub async fn reload_config(&self) -> Result<()> {
        self.load_config().await
    }
}

/// 配置管理器，实现 ConfigManagerPort 接口
pub struct ConfigManager {
    /// 配置服务实例
    service: Arc<ConfigService>,
}

impl ConfigManager {
    /// 创建新的配置管理器实例
    /// 
    /// # 参数
    /// * `service` - 配置服务实例
    /// 
    /// # 返回值
    /// 配置管理器实例
    pub fn new(service: Arc<ConfigService>) -> Self {
        Self {
            service,
        }
    }
}

#[async_trait]
impl ConfigManagerPort for ConfigManager {
    async fn get_config<T: DeserializeOwned + Serialize + Send>(&self, key: &str) -> Result<Option<T>> {
        self.service.get_config(key).await
    }
    
    async fn set_config<T: Serialize + Sync>(&self, key: &str, value: &T) -> Result<()> {
        self.service.set_config(key, value, ConfigSource::Default).await
    }
    
    async fn reload_config(&self) -> Result<()> {
        self.service.reload_config().await
    }
    
    async fn watch_config(&self) -> Result<()> {
        // 实现配置文件监控，支持热重载
        // 这里暂时返回Ok，实际实现需要使用notify库监控文件变化
        Ok(())
    }
}