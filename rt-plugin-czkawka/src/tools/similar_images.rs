use serde::{Deserialize, Serialize}; use serde_json::Value; use super::Tool; use super::czkawka_adapter::CzkawkaAdapter; use schemars::{JsonSchema};

/// 相似图片查找工具的输入结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Input {
    /// 要扫描的目录列表
    directories: Vec<String>,
    /// 相似度阈值（0-100）
    #[serde(default = "default_threshold")]
    threshold: u32,
}

fn default_threshold() -> u32 {
    90
}

/// 相似图片查找工具的输出结构
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Output {
    /// 相似图片组列表，每个组包含相似度分数
    similar_groups: Vec<(Vec<String>, u32)>,
}

/// 相似图片查找工具实现
pub struct SimilarImagesTool;

impl Tool for SimilarImagesTool {
    /// 获取工具名称
    fn name(&self) -> &'static str {
        "file.similar_images"
    }
    
    /// 运行相似图片查找工具
    async fn run(&self, input: Value) -> anyhow::Result<Value> {
        let input: Input = serde_json::from_value(input)?;
        
        let similar_groups = CzkawkaAdapter::find_similar_images(&input.directories, Some(input.threshold));
        
        let output = Output {
            similar_groups,
        };
        
        Ok(serde_json::to_value(output)?)    
    }
}
