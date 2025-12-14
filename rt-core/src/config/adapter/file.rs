use async_trait::async_trait;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use crate::{error::{CoreError, Result}, config::{domain::ConfigSource, port::ConfigSourcePort}};

/// 配置文件格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// JSON格式
    Json,
    /// YAML格式
    Yaml,
    /// Properties格式
    Properties,
}

/// 文件配置适配器，从配置文件加载配置
pub struct FileConfigAdapter {
    /// 配置文件路径
    path: PathBuf,
    
    /// 配置文件格式
    format: FileFormat,
    
    /// 配置源优先级
    priority: u8,
}

impl FileConfigAdapter {
    /// 创建新的文件配置适配器
    /// 
    /// # 参数
    /// * `path` - 配置文件路径
    /// * `format` - 配置文件格式
    /// * `priority` - 配置源优先级
    /// 
    /// # 返回值
    /// * `Ok(FileConfigAdapter)` - 成功创建文件配置适配器
    pub fn new(path: PathBuf, format: FileFormat, priority: u8) -> Result<Self> {
        // 检查文件是否存在
        if !path.exists() {
            return Err(CoreError::ConfigError(format!("Config file not found: {}", path.to_str().unwrap_or("unknown path"))));
        }
        
        Ok(Self {
            path,
            format,
            priority,
        })
    }
    
    /// 从JSON文件加载配置
    /// 
    /// # 参数
    /// * `content` - 文件内容
    /// 
    /// # 返回值
    /// * `Ok(HashMap<String, String>)` - 成功加载配置，返回配置键值对
    /// * `Err(CoreError)` - 加载配置失败
    fn load_json(&self, content: &str) -> Result<std::collections::HashMap<String, String>> {
        let value: serde_json::Value = serde_json::from_str(content)?;
        self.flatten_json("", &value)
    }
    
    /// 从YAML文件加载配置
    /// 
    /// # 参数
    /// * `content` - 文件内容
    /// 
    /// # 返回值
    /// * `Ok(HashMap<String, String>)` - 成功加载配置，返回配置键值对
    /// * `Err(CoreError)` - 加载配置失败
    fn load_yaml(&self, content: &str) -> Result<std::collections::HashMap<String, String>> {
        let value: serde_yaml::Value = serde_yaml::from_str(content)?;
        self.flatten_yaml("", &value)
    }
    
    /// 从Properties文件加载配置
    /// 
    /// # 参数
    /// * `content` - 文件内容
    /// 
    /// # 返回值
    /// * `Ok(HashMap<String, String>)` - 成功加载配置，返回配置键值对
    /// * `Err(CoreError)` - 加载配置失败
    fn load_properties(&self, content: &str) -> Result<std::collections::HashMap<String, String>> {
        let mut config_map = std::collections::HashMap::new();
        
        for line in content.lines() {
            // 跳过空行和注释
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // 解析键值对
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                config_map.insert(key.to_string(), value.to_string());
            }
        }
        
        Ok(config_map)
    }
    
    /// 扁平化JSON值，将嵌套的JSON转换为扁平的键值对
    /// 
    /// # 参数
    /// * `prefix` - 前缀键名
    /// * `value` - JSON值
    /// 
    /// # 返回值
    /// * `Ok(HashMap<String, String>)` - 成功扁平化JSON，返回扁平的键值对
    /// * `Err(CoreError)` - 扁平化JSON失败
    fn flatten_json(&self, prefix: &str, value: &serde_json::Value) -> Result<std::collections::HashMap<String, String>> {
        let mut config_map = std::collections::HashMap::new();
        
        match value {
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    
                    let mut nested_map = self.flatten_json(&new_prefix, val)?;
                    config_map.extend(nested_map);
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_prefix = if prefix.is_empty() {
                        format!("[{}]", i)
                    } else {
                        format!("{}.[{}]", prefix, i)
                    };
                    
                    let mut nested_map = self.flatten_json(&new_prefix, val)?;
                    config_map.extend(nested_map);
                }
            }
            _ => {
                // 基本类型，直接转换为字符串
                let str_val = serde_json::to_string(value)?;
                config_map.insert(prefix.to_string(), str_val);
            }
        }
        
        Ok(config_map)
    }
    
    /// 扁平化YAML值，将嵌套的YAML转换为扁平的键值对
    /// 
    /// # 参数
    /// * `prefix` - 前缀键名
    /// * `value` - YAML值
    /// 
    /// # 返回值
    /// * `Ok(HashMap<String, String>)` - 成功扁平化YAML，返回扁平的键值对
    /// * `Err(CoreError)` - 扁平化YAML失败
    fn flatten_yaml(&self, prefix: &str, value: &serde_yaml::Value) -> Result<std::collections::HashMap<String, String>> {
        let mut config_map = std::collections::HashMap::new();
        
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (key, val) in map {
                    let key_str = key.as_str().ok_or_else(|| 
                        CoreError::ConfigError("Invalid YAML key: not a string".to_string())
                    )?;
                    
                    let new_prefix = if prefix.is_empty() {
                        key_str.to_string()
                    } else {
                        format!("{}.{}", prefix, key_str)
                    };
                    
                    let mut nested_map = self.flatten_yaml(&new_prefix, val)?;
                    config_map.extend(nested_map);
                }
            }
            serde_yaml::Value::Sequence(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_prefix = if prefix.is_empty() {
                        format!("[{}]", i)
                    } else {
                        format!("{}.[{}]", prefix, i)
                    };
                    
                    let mut nested_map = self.flatten_yaml(&new_prefix, val)?;
                    config_map.extend(nested_map);
                }
            }
            _ => {
                // 基本类型，直接转换为字符串
                let str_val = serde_yaml::to_string(value)?;
                config_map.insert(prefix.to_string(), str_val);
            }
        }
        
        Ok(config_map)
    }
}

#[async_trait]
impl ConfigSourcePort for FileConfigAdapter {
    async fn load_config(&self) -> Result<std::collections::HashMap<String, String>> {
        // 读取文件内容
        let mut file = File::open(&self.path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        // 根据文件格式加载配置
        match self.format {
            FileFormat::Json => self.load_json(&content),
            FileFormat::Yaml => self.load_yaml(&content),
            FileFormat::Properties => self.load_properties(&content),
        }
    }
    
    fn get_source_type(&self) -> ConfigSource {
        ConfigSource::File
    }
    
    fn get_priority(&self) -> u8 {
        self.priority
    }
}