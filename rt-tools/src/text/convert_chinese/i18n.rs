use rt_core::Locale;

pub fn display_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Chinese Converter",
        Locale::Zh => "简繁转换",
    }
}

pub fn description(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Convert between Simplified and Traditional Chinese",
        Locale::Zh => "简体繁体中文互转",
    }
}

pub fn user_guide(locale: Locale) -> &'static str {
    match locale {
        Locale::En => r#"# Chinese Converter

Convert text between Simplified and Traditional Chinese variants.

## Inputs
- **text**: Text to convert.
- **mode**: Conversion mode. Options:
  - `s2t`: Simplified to Traditional
  - `t2s`: Traditional to Simplified
  - `s2tw`: Simplified to Traditional (Taiwan)
  - `tw2s`: Traditional (Taiwan) to Simplified
  - `s2hk`: Simplified to Traditional (Hong Kong)
  - `hk2s`: Traditional (Hong Kong) to Simplified
  - `s2twp`: Simplified to Traditional (Taiwan with phrases)
  - `tw2sp`: Traditional (Taiwan) to Simplified (with phrases)

## Output
- **converted**: Converted text.
"#,
        Locale::Zh => r#"# 简繁转换 (Chinese Converter)

在简体中文和繁体中文之间转换文本。

## 输入参数
- **text**: 要转换的文本。
- **mode**: 转换模式。选项：
  - `s2t`: 简体到繁体
  - `t2s`: 繁体到简体
  - `s2tw`: 简体到台湾繁体
  - `tw2s`: 台湾繁体到简体
  - `s2hk`: 简体到香港繁体
  - `hk2s`: 香港繁体到简体
  - `s2twp`: 简体到台湾繁体（含惯用语）
  - `tw2sp`: 台湾繁体到简体（含惯用语）

## 输出
- **converted**: 转换后的文本。
"#,
    }
}

pub fn input_title(field: &str, locale: Locale) -> Option<&'static str> {
    match (locale, field) {
        (Locale::Zh, "text") => Some("文本"),
        (Locale::Zh, "mode") => Some("模式"),
        _ => None
    }
}

pub fn output_title(field: &str, locale: Locale) -> Option<&'static str> {
    match (locale, field) {
        (Locale::Zh, "converted") => Some("转换结果"),
        _ => None
    }
}

pub fn mode_title<'a>(mode: &'a str, locale: Locale) -> &'a str {
    match (locale, mode) {
        // English
        (Locale::En, "s2t") => "Simplified to Traditional",
        (Locale::En, "t2s") => "Traditional to Simplified",
        (Locale::En, "s2tw") => "Simplified to Traditional (Taiwan)",
        (Locale::En, "tw2s") => "Traditional (Taiwan) to Simplified",
        (Locale::En, "s2hk") => "Simplified to Traditional (Hong Kong)",
        (Locale::En, "hk2s") => "Traditional (Hong Kong) to Simplified",
        (Locale::En, "s2twp") => "Simplified to Traditional (Taiwan, with phrases)",
        (Locale::En, "tw2sp") => "Traditional (Taiwan) to Simplified (with phrases)",
        
        // Chinese
        (Locale::Zh, "s2t") => "简体到繁体",
        (Locale::Zh, "t2s") => "繁体到简体",
        (Locale::Zh, "s2tw") => "简体到台湾繁体",
        (Locale::Zh, "tw2s") => "台湾繁体到简体",
        (Locale::Zh, "s2hk") => "简体到香港繁体",
        (Locale::Zh, "hk2s") => "香港繁体到简体",
        (Locale::Zh, "s2twp") => "简体到台湾繁体（含惯用语）",
        (Locale::Zh, "tw2sp") => "台湾繁体到简体（含惯用语）",
        
        // Fallback
        _ => mode,
    }
}
