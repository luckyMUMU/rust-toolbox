use super::*;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_move_folder_basic() {
    let tool = MoveFolder::new();
    let tmp = tempdir().unwrap();
    let src_path = tmp.path().join("source");
    let dst_path = tmp.path().join("dest");

    fs::create_dir(&src_path).unwrap();
    fs::write(src_path.join("file.txt"), "hello").unwrap();

    let input = json!({
        "source": src_path.to_str().unwrap(),
        "destination": dst_path.to_str().unwrap(),
        "overwrite": false
    });

    let result = tool.run(input).await.unwrap();
    let output: MoveFolderOutput = serde_json::from_value(result).unwrap();

    assert!(output.success);
    assert!(!src_path.exists());
    assert!(dst_path.exists());
    // Since dst_path did NOT exist initially, it was renamed TO dst_path.
    // So file.txt is directly under dst_path.
    assert!(dst_path.join("file.txt").exists());
}

#[tokio::test]
async fn test_move_folder_into_existing() {
    let tool = MoveFolder::new();
    let tmp = tempdir().unwrap();
    let src_path = tmp.path().join("source");
    let dst_root = tmp.path().join("dst_root");

    fs::create_dir(&src_path).unwrap();
    fs::write(src_path.join("file.txt"), "hello").unwrap();
    fs::create_dir(&dst_root).unwrap();

    let input = json!({
        "source": src_path.to_str().unwrap(),
        "destination": dst_root.to_str().unwrap(),
        "overwrite": false
    });

    let result = tool.run(input).await.unwrap();
    let output: MoveFolderOutput = serde_json::from_value(result).unwrap();

    assert!(output.success);
    assert!(!src_path.exists());
    assert!(dst_root.join("source").exists());
    assert!(dst_root.join("source").join("file.txt").exists());
}

#[tokio::test]
async fn test_move_folder_overwrite() {
    let tool = MoveFolder::new();
    let tmp = tempdir().unwrap();
    let src_path = tmp.path().join("source");
    let dst_path = tmp.path().join("dest");

    fs::create_dir(&src_path).unwrap();
    fs::write(src_path.join("file.txt"), "new content").unwrap();
    
    fs::create_dir(&dst_path).unwrap();
    fs::write(dst_path.join("old.txt"), "old content").unwrap();

    // Test overwrite: false
    fs::create_dir(dst_path.join("source")).unwrap(); // Create the actual target to trigger overwrite error
    let input_no_overwrite = json!({
        "source": src_path.to_str().unwrap(),
        "destination": dst_path.to_str().unwrap(),
        "overwrite": false
    });
    let result = tool.run(input_no_overwrite).await;
    assert!(result.is_err());

    // Test overwrite: true
    let input_overwrite = json!({
        "source": src_path.to_str().unwrap(),
        "destination": dst_path.to_str().unwrap(),
        "overwrite": true
    });
    let result = tool.run(input_overwrite).await.unwrap();
    let output: MoveFolderOutput = serde_json::from_value(result).unwrap();

    assert!(output.success);
    assert!(dst_path.exists());
    // Since dst_path existed as a directory, source was moved INTO it.
    // So it should be dst_path/source/file.txt
    assert!(dst_path.join("source").join("file.txt").exists());
    assert!(!dst_path.join("source").join("old.txt").exists());
}

#[tokio::test]
async fn test_move_folder_errors() {
    let tool = MoveFolder::new();
    let tmp = tempdir().unwrap();
    
    // Source does not exist
    let input = json!({
        "source": tmp.path().join("non_existent").to_str().unwrap(),
        "destination": tmp.path().join("dest").to_str().unwrap()
    });
    let result = tool.run(input).await;
    assert!(result.is_err());
}
