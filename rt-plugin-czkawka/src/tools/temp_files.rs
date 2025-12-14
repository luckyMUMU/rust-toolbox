use serde::{Deserialize, Serialize}; use serde_json::Value; use super::Tool; use super::czkawka_adapter::CzkawkaAdapter; use schemars::{JsonSchema};

/// 临时文件查找工具的输入结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Input {
    /// 要扫描的目录列表
    directories: Vec<String>,
}

/// 临时文件查找工具的输出结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Output {
    /// 临时文件列表
    temporary_files: Vec<String>,
}

/// 临时文件查找工具实现
pub struct TemporaryFilesTool;

impl Tool for TemporaryFilesTool {
    /// 获取工具名称
    fn name(&self) -> &'static str {
        "file.temporary_files"
    }
    
    /// 运行临时文件查找工具
    async fn run(&self, input: Value) -> anyhow::Result<Value> {
        let input: Input = serde_json::from_value(input)?;
        
        let temporary_files = CzkawkaAdapter::find_temporary_files(&input.directories);
        
        let output = Output {
            temporary_files,
        };
        
        Ok(serde_json::to_value(output)?)    
    }
}
