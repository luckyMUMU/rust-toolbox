use serde::{Deserialize, Serialize}; use serde_json::Value; use super::Tool; use super::czkawka_adapter::CzkawkaAdapter; use schemars::{JsonSchema};

/// 重复文件查找工具的输入结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Input {
    /// 要扫描的目录列表
    directories: Vec<String>,
    /// 最小文件大小（字节）
    #[serde(default = "default_min_size")]
    min_size: u64,
}

fn default_min_size() -> u64 {
    0
}

/// 重复文件查找工具的输出结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Output {
    /// 重复文件组列表
    duplicate_groups: Vec<Vec<String>>,
}

/// 重复文件查找工具实现
pub struct DuplicateFilesTool;

impl Tool for DuplicateFilesTool {
    /// 获取工具名称
    fn name(&self) -> &'static str {
        "file.duplicates"
    }
    
    /// 运行重复文件查找工具
    async fn run(&self, input: Value) -> anyhow::Result<Value> {
        let input: Input = serde_json::from_value(input)?;
        
        let duplicate_groups = CzkawkaAdapter::find_duplicate_files(&input.directories, Some(input.min_size));
        
        let output = Output {
            duplicate_groups,
        };
        
        Ok(serde_json::to_value(output)?)    
    }
}
