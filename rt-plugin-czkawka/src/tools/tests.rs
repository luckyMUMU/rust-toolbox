use super::*;
use crate::tools::duplicates::DuplicateFilesTool;
use crate::tools::similar_images::SimilarImagesTool;
use crate::tools::empty_dirs::EmptyDirectoriesTool;
use crate::tools::temp_files::TemporaryFilesTool;
use crate::tools::broken_symlinks::BrokenSymlinksTool;
use serde_json::json;

/// 测试重复文件查找工具
#[tokio::test]
async fn test_duplicate_files_tool() {
    let tool = DuplicateFilesTool;
    let input = json!({
        "directories": ["./test_data"],
        "min_size": 0
    });
    
    // 这里我们只测试工具能够正常执行，不验证实际结果
    // 因为实际结果依赖于测试数据
    let result = tool.run(input).await;
    assert!(result.is_ok());
}

/// 测试相似图片查找工具
#[tokio::test]
async fn test_similar_images_tool() {
    let tool = SimilarImagesTool;
    let input = json!(
        {
        "directories": ["./test_data"],
        "threshold": 90
    });
    
    let result = tool.run(input).await;
    assert!(result.is_ok());
}

/// 测试空目录查找工具
#[tokio::test]
async fn test_empty_directories_tool() {
    let tool = EmptyDirectoriesTool;
    let input = json!(
        {
        "directories": ["./test_data"]
    });
    
    let result = tool.run(input).await;
    assert!(result.is_ok());
}

/// 测试临时文件查找工具
#[tokio::test]
async fn test_temporary_files_tool() {
    let tool = TemporaryFilesTool;
    let input = json!(
        {
        "directories": ["./test_data"]
    });
    
    let result = tool.run(input).await;
    assert!(result.is_ok());
}

/// 测试损坏的符号链接查找工具
#[tokio::test]
async fn test_broken_symlinks_tool() {
    let tool = BrokenSymlinksTool;
    let input = json!(
        {
        "directories": ["./test_data"]
    });
    
    let result = tool.run(input).await;
    assert!(result.is_ok());
}


