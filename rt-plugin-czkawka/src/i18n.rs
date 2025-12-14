use rt_core::locale::Locale; use rt_core::plugin::{LocalizedString}; use std::collections::HashMap;

/// 获取工具用户指南的本地化字符串
///
/// # 参数
/// * `tool_name` - 工具名称。
/// * `locale` - 目标语言环境。
///
/// # 返回值
/// 返回工具的本地化用户指南。
pub fn get_tool_user_guide(tool_name: &str, locale: Locale) -> String {
    match locale {
        Locale::En => match tool_name {
            "file.duplicates" => "# Duplicate Files Tool\n\nThis tool finds duplicate files by comparing their content.\n\n## Usage\n- Enter the directories to scan\n- Set the minimum file size (optional)\n- Click 'Run' to start scanning\n\n## Output\nThe tool will return a list of duplicate file groups.".to_string(),
            "file.similar_images" => "# Similar Images Tool\n\nThis tool finds visually similar images based on their content.\n\n## Usage\n- Enter the directories to scan\n- Adjust the similarity threshold (0-100)\n- Click 'Run' to start scanning\n\n## Output\nThe tool will return a list of similar image groups.".to_string(),
            "file.empty_directories" => "# Empty Directories Tool\n\nThis tool finds directories that contain no files or only empty subdirectories.\n\n## Usage\n- Enter the directories to scan\n- Click 'Run' to start scanning\n\n## Output\nThe tool will return a list of empty directories.".to_string(),
            "file.temporary_files" => "# Temporary Files Tool\n\nThis tool finds temporary files based on common patterns and extensions.\n\n## Usage\n- Enter the directories to scan\n- Click 'Run' to start scanning\n\n## Output\nThe tool will return a list of temporary files.".to_string(),
            "file.broken_symlinks" => "# Broken Symbolic Links Tool\n\nThis tool finds symbolic links that point to non-existent targets.\n\n## Usage\n- Enter the directories to scan\n- Click 'Run' to start scanning\n\n## Output\nThe tool will return a list of broken symbolic links.".to_string(),
            _ => "".to_string(),
        },
        Locale::Zh => match tool_name {
            "file.duplicates" => "# 重复文件工具\n\n该工具通过比较文件内容查找重复文件。\n\n## 使用方法\n- 输入要扫描的目录\n- 设置最小文件大小（可选）\n- 点击'运行'开始扫描\n\n## 输出\n工具将返回重复文件组列表。".to_string(),
            "file.similar_images" => "# 相似图片工具\n\n该工具基于图片内容查找视觉相似的图片。\n\n## 使用方法\n- 输入要扫描的目录\n- 调整相似度阈值（0-100）\n- 点击'运行'开始扫描\n\n## 输出\n工具将返回相似图片组列表。".to_string(),
            "file.empty_directories" => "# 空目录工具\n\n该工具查找不包含文件或仅包含空子目录的目录。\n\n## 使用方法\n- 输入要扫描的目录\n- 点击'运行'开始扫描\n\n## 输出\n工具将返回空目录列表。".to_string(),
            "file.temporary_files" => "# 临时文件工具\n\n该工具基于常见模式和扩展名查找临时文件。\n\n## 使用方法\n- 输入要扫描的目录\n- 点击'运行'开始扫描\n\n## 输出\n工具将返回临时文件列表。".to_string(),
            "file.broken_symlinks" => "# 损坏的符号链接工具\n\n该工具查找指向不存在目标的符号链接。\n\n## 使用方法\n- 输入要扫描的目录\n- 点击'运行'开始扫描\n\n## 输出\n工具将返回损坏的符号链接列表。".to_string(),
            _ => "".to_string(),
        },
    }
}

/// 获取输入字段的本地化标题映射
///
/// # 返回值
/// 返回输入字段的本地化标题映射。
pub fn get_input_field_map() -> HashMap<String, LocalizedString> {
    let mut map = HashMap::new();
    
    // Common fields
    map.insert("directories".to_string(), LocalizedString {
        en: "Directories".to_string(),
        zh: Some("目录".to_string()),
    });
    
    map.insert("min_size".to_string(), LocalizedString {
        en: "Minimum File Size (bytes)".to_string(),
        zh: Some("最小文件大小（字节）".to_string()),
    });
    
    map.insert("threshold".to_string(), LocalizedString {
        en: "Similarity Threshold (0-100)".to_string(),
        zh: Some("相似度阈值（0-100）".to_string()),
    });
    
    // Output fields
    map.insert("duplicate_groups".to_string(), LocalizedString {
        en: "Duplicate Groups".to_string(),
        zh: Some("重复文件组".to_string()),
    });
    
    map.insert("similar_groups".to_string(), LocalizedString {
        en: "Similar Groups".to_string(),
        zh: Some("相似文件组".to_string()),
    });
    
    map.insert("empty_directories".to_string(), LocalizedString {
        en: "Empty Directories".to_string(),
        zh: Some("空目录".to_string()),
    });
    
    map.insert("temporary_files".to_string(), LocalizedString {
        en: "Temporary Files".to_string(),
        zh: Some("临时文件".to_string()),
    });
    
    map.insert("broken_symlinks".to_string(), LocalizedString {
        en: "Broken Symbolic Links".to_string(),
        zh: Some("损坏的符号链接".to_string()),
    });
    
    map.insert("files".to_string(), LocalizedString {
        en: "Files".to_string(),
        zh: Some("文件".to_string()),
    });
    
    map.insert("group_id".to_string(), LocalizedString {
        en: "Group ID".to_string(),
        zh: Some("组ID".to_string()),
    });
    
    map
}

/// 获取输出字段的本地化标题映射
///
/// # 返回值
/// 返回输出字段的本地化标题映射。
pub fn get_output_field_map() -> HashMap<String, LocalizedString> {
    get_input_field_map()
}
