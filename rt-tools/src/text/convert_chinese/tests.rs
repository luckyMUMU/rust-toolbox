use super::*;
use serde_json::json;

#[tokio::test]
async fn test_convert_chinese_s2t() {
    let tool = ConvertChinese::new();
    let input = json!({
        "text": "简体中文",
        "mode": "s2t"
    });
    
    let result = tool.run(input).await.unwrap();
    let output: ConvertChineseOutput = serde_json::from_value(result).unwrap();
    println!("S2T Result: Input='简体中文', Result='{}'", output.converted);
    assert_eq!(output.converted, "簡體中文");
}

#[tokio::test]
async fn test_convert_chinese_t2s() {
    let tool = ConvertChinese::new();
    let input = json!({
        "text": "繁體中文",
        "mode": "t2s"
    });
    
    let result = tool.run(input).await.unwrap();
    let output: ConvertChineseOutput = serde_json::from_value(result).unwrap();
    println!("T2S Result: Input='繁體中文', Result='{}'", output.converted);
    assert_eq!(output.converted, "繁体中文");
}

#[tokio::test]
async fn test_convert_chinese_to_tw_p() {
    let tool = ConvertChinese::new();
    let input = json!({
        "text": "软件",
        "mode": "s2twp"
    });
    
    let result = tool.run(input).await.unwrap();
    let output: ConvertChineseOutput = serde_json::from_value(result).unwrap();
    println!("S2TWP Result: Input='软件', Result='{}'", output.converted);
    assert_eq!(output.converted, "軟體");
}

#[tokio::test]
async fn test_convert_chinese_invalid_mode() {
    let tool = ConvertChinese::new();
    let input = json!({
        "text": "test",
        "mode": "invalid"
    });
    
    let result = tool.run(input).await;
    assert!(result.is_err());
}
