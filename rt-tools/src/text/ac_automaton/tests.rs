use super::*;

#[tokio::test]
async fn test_ac_automaton_basic() {
    // 创建AC自动机工具
    let tool = AcAutomatonTool::new();
    
    // 添加模式串
    let input = json!({
        "action": "add",
        "patterns": ["hello", "world", "test"]
    });
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to add patterns: {}", output.message);
    
    // 列出模式串
    let input = json!(
        {
            "action": "list",
            "patterns": []
        }
    );
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to list patterns: {}", output.message);
    let patterns = output.patterns.unwrap();
    assert_eq!(patterns.len(), 3);
    assert!(patterns.contains(&"hello".to_string()));
    assert!(patterns.contains(&"world".to_string()));
    assert!(patterns.contains(&"test".to_string()));
    
    // 测试匹配
    let input = json!(
        {
            "action": "match",
            "patterns": [],
            "texts": ["hello world", "test hello", "no match"]
        }
    );
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to match: {}", output.message);
    let results = output.results.unwrap();
    assert_eq!(results.len(), 4); // "hello world" 匹配 "hello" 和 "world"，"test hello" 匹配 "test" 和 "hello"
    
    // 测试删除模式串
    let input = json!(
        {
            "action": "remove",
            "patterns": ["test"],
            "confirm": true
        }
    );
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to remove pattern: {}", output.message);
    
    // 再次列出模式串，确认已删除
    let input = json!(
        {
            "action": "list",
            "patterns": []
        }
    );
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to list patterns: {}", output.message);
    let patterns = output.patterns.unwrap();
    assert_eq!(patterns.len(), 2);
    assert!(!patterns.contains(&"test".to_string()));
}

#[tokio::test]
async fn test_ac_automaton_ignore_case() {
    // 创建AC自动机工具
    let tool = AcAutomatonTool::new();
    
    // 添加模式串，忽略大小写
    let input = json!(
        {
            "action": "add",
            "patterns": ["hello"],
            "ignore_case": true
        }
    );
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to add patterns: {}", output.message);
    
    // 测试匹配，忽略大小写
    let input = json!(
        {
            "action": "match",
            "patterns": [],
            "texts": ["Hello", "HELLO", "hello", "hElLo"]
        }
    );
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to match: {}", output.message);
    let results = output.results.unwrap();
    assert_eq!(results.len(), 4);
}

#[tokio::test]
async fn test_ac_automaton_parallel() {
    // 创建AC自动机工具
    let tool = AcAutomatonTool::new();
    
    // 添加模式串
    let input = json!(
        {
            "action": "add",
            "patterns": ["test", "parallel", "match"]
        }
    );
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to add patterns: {}", output.message);
    
    // 测试并行匹配
    let input = json!(
        {
            "action": "match",
            "patterns": [],
            "texts": ["test 1", "parallel 2", "match 3", "test parallel match"],
            "parallel": true
        }
    );
    let result = tool.run(input).await.unwrap();
    let output: AcAutomatonOutput = serde_json::from_value(result).unwrap();
    assert!(output.success, "Failed to match in parallel: {}", output.message);
    let results = output.results.unwrap();
    assert_eq!(results.len(), 6); // "test 1" → 1个, "parallel 2" → 1个, "match 3" → 1个, "test parallel match" → 3个, 总共6个
}