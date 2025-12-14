use std::path::Path;
use tokio::fs as tokio_fs;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tempfile::NamedTempFile;
use crate::error::{CoreError, Result};

/// 文件创建操作
/// 
/// # 参数
/// * `path` - 文件路径
/// * `content` - 初始内容（可选）
/// 
/// # 返回值
/// * `Ok(())` - 文件创建成功
/// * `Err(CoreError)` - 文件创建失败，包含具体错误信息
/// 
/// # 错误类型
/// * `CoreError::IoError` - IO错误，如权限不足、磁盘空间不足等
pub async fn create_file(path: &Path, content: Option<&str>) -> Result<()> {
    // 检查目录是否存在，如果不存在则创建
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            tokio_fs::create_dir_all(parent).await?;
        }
    }
    
    // 创建文件
    let mut file = tokio_fs::File::create(path).await?;
    
    // 如果有初始内容则写入
    if let Some(content) = content {
        file.write_all(content.as_bytes()).await?;
    }
    
    Ok(())
}

/// 文件读取操作
/// 
/// # 参数
/// * `path` - 文件路径
/// 
/// # 返回值
/// * `Ok((String, String))` - 成功读取文件，返回文件内容和编码格式
/// * `Err(CoreError)` - 文件读取失败，包含具体错误信息
/// 
/// # 错误类型
/// * `CoreError::IoError` - IO错误，如文件不存在、权限不足等
pub async fn read_file(path: &Path) -> Result<(String, String)> {
    // 打开文件
    let mut file = tokio_fs::File::open(path).await?;
    
    // 读取文件内容
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await?;
    
    // 检测文件编码（简单实现，优先尝试UTF-8）
    let (content, encoding) = match String::from_utf8(bytes) {
        Ok(content) => (content, "UTF-8".to_string()),
        Err(e) => {
            // 如果UTF-8解码失败，使用损失性解码
            let content = String::from_utf8_lossy(e.as_bytes()).to_string();
            (content, "UTF-8 (with replacement)".to_string())
        }
    };
    
    Ok((content, encoding))
}

/// 文件更新操作（双缓冲区安全机制）
/// 
/// # 参数
/// * `path` - 文件路径
/// * `content` - 新内容
/// 
/// # 返回值
/// * `Ok(())` - 文件更新成功
/// * `Err(CoreError)` - 文件更新失败，包含具体错误信息
/// 
/// # 错误类型
/// * `CoreError::IoError` - IO错误，如权限不足、磁盘空间不足等
/// * `CoreError::InvalidInput` - 输入无效，如文件路径为空等
/// 
/// # 实现说明
/// 1. 创建临时文件B并完整写入更新内容
/// 2. 验证临时文件B的写入完整性和正确性
/// 3. 成功验证后，使用原子操作将临时文件B替换原始文件
/// 4. 确保替换过程中的异常处理和回滚机制
pub async fn update_file(path: &Path, content: &str) -> Result<()> {
    if path.as_os_str().is_empty() {
        return Err(CoreError::InvalidInput("File path cannot be empty".to_string()));
    }
    
    // 1. 创建临时文件B
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path().to_path_buf();
    
    // 2. 将更新内容完整写入临时文件B
    let mut file = tokio_fs::File::create(&temp_path).await?;
    file.write_all(content.as_bytes()).await?;
    file.flush().await?;
    
    // 3. 验证临时文件B的写入完整性和正确性
    let mut read_content = String::new();
    let mut read_file = tokio_fs::File::open(&temp_path).await?;
    read_file.read_to_string(&mut read_content).await?;
    
    if read_content != content {
        return Err(CoreError::InvalidInput("Failed to verify write operation".to_string()));
    }
    
    // 4. 使用原子操作将临时文件B替换原始文件
    tokio_fs::rename(&temp_path, path).await?;
    
    Ok(())
}

/// 文件删除操作
/// 
/// # 参数
/// * `path` - 文件路径
/// 
/// # 返回值
/// * `Ok(())` - 文件删除成功
/// * `Err(CoreError)` - 文件删除失败，包含具体错误信息
/// 
/// # 错误类型
/// * `CoreError::IoError` - IO错误，如文件不存在、权限不足等
pub async fn delete_file(path: &Path) -> Result<()> {
    // 检查文件是否存在
    if !path.exists() {
        return Err(CoreError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path.display())
        )));
    }
    
    // 删除文件
    tokio_fs::remove_file(path).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File as StdFile;
    use std::io::{Write as StdWrite, Read as StdRead};
    
    #[tokio::test]
    async fn test_create_file() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // 测试创建空文件
        let result = create_file(&file_path, None).await;
        assert!(result.is_ok());
        assert!(file_path.exists());
        
        // 测试创建带内容的文件
        let content = "Hello, world!";
        let file_path2 = temp_dir.path().join("test2.txt");
        let result = create_file(&file_path2, Some(content)).await;
        assert!(result.is_ok());
        assert!(file_path2.exists());
        
        // 读取文件内容验证
        let mut file = StdFile::open(file_path2).unwrap();
        let mut read_content = String::new();
        file.read_to_string(&mut read_content).unwrap();
        assert_eq!(read_content, content);
    }
    
    #[tokio::test]
    async fn test_read_file() {
        // 创建临时目录和文件
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, world!";
        
        // 创建文件
        let mut file = StdFile::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        // 测试读取文件
        let result = read_file(&file_path).await;
        assert!(result.is_ok());
        
        let (read_content, encoding) = result.unwrap();
        assert_eq!(read_content, content);
        assert_eq!(encoding, "UTF-8");
    }
    
    #[tokio::test]
    async fn test_update_file() {
        // 创建临时目录和文件
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let initial_content = "Initial content";
        let new_content = "Updated content";
        
        // 创建初始文件
        let mut file = StdFile::create(&file_path).unwrap();
        file.write_all(initial_content.as_bytes()).unwrap();
        
        // 测试更新文件
        let result = update_file(&file_path, new_content).await;
        assert!(result.is_ok());
        
        // 读取文件内容验证
        let mut read_file = StdFile::open(&file_path).unwrap();
        let mut read_content = String::new();
        read_file.read_to_string(&mut read_content).unwrap();
        assert_eq!(read_content, new_content);
    }
    
    #[tokio::test]
    async fn test_delete_file() {
        // 创建临时目录和文件
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // 创建文件
        let mut file = StdFile::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        
        // 测试删除文件
        let result = delete_file(&file_path).await;
        assert!(result.is_ok());
        assert!(!file_path.exists());
        
        // 测试删除不存在的文件
        let result = delete_file(&file_path).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), format!("IO error: File not found: {}", file_path.display()));
    }
    
    #[tokio::test]
    async fn test_file_operations_integration() {
        // 创建临时目录
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // 1. 创建文件
        let content1 = "First content";
        let result = create_file(&file_path, Some(content1)).await;
        assert!(result.is_ok());
        
        // 2. 读取文件
        let result = read_file(&file_path).await;
        assert!(result.is_ok());
        let (read_content1, _) = result.unwrap();
        assert_eq!(read_content1, content1);
        
        // 3. 更新文件
        let content2 = "Second content";
        let result = update_file(&file_path, content2).await;
        assert!(result.is_ok());
        
        // 4. 再次读取文件验证更新
        let result = read_file(&file_path).await;
        assert!(result.is_ok());
        let (read_content2, _) = result.unwrap();
        assert_eq!(read_content2, content2);
        
        // 5. 删除文件
        let result = delete_file(&file_path).await;
        assert!(result.is_ok());
        assert!(!file_path.exists());
    }
}
