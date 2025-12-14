pub mod duplicates; pub mod similar_images; pub mod empty_dirs; pub mod temp_files; pub mod broken_symlinks; pub mod czkawka_adapter;

use serde_json::Value;

/// 通用输入结构 #[derive(Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CommonInput {
    pub tool_name: String,
    pub directories: Vec<String>,
    pub min_size: Option<u64>,
    pub threshold: Option<u32>,
}

/// 重复文件组 #[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct DuplicateGroup {
    pub group_id: u32,
    pub files: Vec<String>,
}

/// 相似图片组 #[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct SimilarGroup {
    pub group_id: u32,
    pub files: Vec<String>,
    pub similarity: u32,
}

// 移除不再使用的 get_tool_by_name 函数，因为 async fn 不能用于 dyn 对象

/// 工具 trait，定义工具的基本行为
pub trait Tool {
    /// 运行工具
    async fn run(&self, input: Value) -> anyhow::Result<Value>;
    
    /// 获取工具名称
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests;
