use serde::{Deserialize, Serialize}; use serde_json::Value; use super::Tool; use super::czkawka_adapter::CzkawkaAdapter; use schemars::{JsonSchema};

/// 损坏的符号链接查找工具的输入结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Input {
    /// 要扫描的目录列表
    directories: Vec<String>,
}

/// 损坏的符号链接查找工具的输出结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Output {
    /// 损坏的符号链接列表
    broken_symlinks: Vec<String>,
}

/// 损坏的符号链接查找工具实现
pub struct BrokenSymlinksTool;

impl Tool for BrokenSymlinksTool {
    /// 获取工具名称
    fn name(&self) -> &'static str {
        "file.broken_symlinks"
    }
    
    /// 运行损坏的符号链接查找工具
    async fn run(&self, input: Value) -> anyhow::Result<Value> {
        let input: Input = serde_json::from_value(input)?;
        
        let broken_symlinks = CzkawkaAdapter::find_broken_symlinks(&input.directories);
        
        let output = Output {
            broken_symlinks,
        };
        
        Ok(serde_json::to_value(output)?)    
    }
}
