use async_trait::async_trait;
use std::env;
use crate::{error::Result, config::{domain::ConfigSource, port::ConfigSourcePort}}; // 删除未使用的 CoreError 导入

/// 环境变量配置适配器，从环境变量加载配置
pub struct EnvConfigAdapter {
    /// 环境变量前缀，用于过滤环境变量
    prefix: String,
    
    /// 配置源优先级
    priority: u8,
}

impl EnvConfigAdapter {
    /// 创建新的环境变量配置适配器
    /// 
    /// # 参数
    /// * `prefix` - 环境变量前缀，用于过滤环境变量
    /// * `priority` - 配置源优先级
    /// 
    /// # 返回值
    /// * `Ok(EnvConfigAdapter)` - 成功创建环境变量配置适配器
    pub fn new(prefix: String, priority: u8) -> Result<Self> {
        Ok(Self {
            prefix,
            priority,
        })
    }
}

#[async_trait]
impl ConfigSourcePort for EnvConfigAdapter {
    async fn load_config(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut config_map = std::collections::HashMap::new();
        
        // 遍历所有环境变量
        for (key, value) in env::vars() {
            // 过滤出带有指定前缀的环境变量
            if key.starts_with(&self.prefix) {
                // 移除前缀并转换为小写
                let config_key = key
                    .trim_start_matches(&self.prefix)
                    .to_lowercase()
                    .replace('_', "."); // 将下划线转换为点，符合配置键命名规范
                
                // 将环境变量值转换为JSON字符串
                let json_value = serde_json::to_string(&value)?;
                
                config_map.insert(config_key, json_value);
            }
        }
        
        Ok(config_map)
    }
    
    fn get_source_type(&self) -> ConfigSource {
        ConfigSource::Environment
    }
    
    fn get_priority(&self) -> u8 {
        self.priority
    }
}