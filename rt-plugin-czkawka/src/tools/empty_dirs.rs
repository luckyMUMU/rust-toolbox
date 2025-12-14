use serde::{Deserialize, Serialize}; use serde_json::Value; use super::Tool; use super::czkawka_adapter::CzkawkaAdapter; use schemars::{JsonSchema};

/// 空目录查找工具的输入结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Input {
    /// 要扫描的目录列表
    directories: Vec<String>,
}

/// 空目录查找工具的输出结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Output {
    /// 空目录列表
    empty_directories: Vec<String>,
}

/// 空目录查找工具实现
pub struct EmptyDirectoriesTool;

impl Tool for EmptyDirectoriesTool {
    /// 获取工具名称
    fn name(&self) -> &'static str {
        "file.empty_directories"
    }
    
    /// 运行空目录查找工具
    async fn run(&self, input: Value) -> anyhow::Result<Value> {
        let input: Input = serde_json::from_value(input)?;
        
        let empty_directories = CzkawkaAdapter::find_empty_directories(&input.directories);
        
        let output = Output {
            empty_directories,
        };
        
        Ok(serde_json::to_value(output)?)    
    }
}
