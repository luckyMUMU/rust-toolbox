use rt_core::Locale;

pub fn display_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Move Folder",
        Locale::Zh => "移动文件夹",
    }
}

pub fn description(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Move or rename a folder",
        Locale::Zh => "移动或重命名文件夹",
    }
}

pub fn user_guide(locale: Locale) -> &'static str {
    match locale {
        Locale::En => r#"# Move Folder

Move or rename a folder.

## Behavior
1. **Rename/Move**: If `destination` does not exist, the source folder is renamed/moved to that path.
2. **Move Into**: If `destination` is an existing directory, the source folder is moved *into* that directory.

## Inputs
- **source**: Path to the folder to move.
- **destination**: Path to the new location.
- **overwrite**: If true, overwrite the destination if it exists.

## Notes
- If `overwrite` is true and the calculated target path exists, it will be deleted before moving.
- Cross-device moves may fail if simple rename is not supported (depends on OS).
"#,
        Locale::Zh => r#"# 移动文件夹 (Move Folder)

移动或重命名指定的文件夹。

## 行为说明
1. **重命名/移动**: 如果 `destination` 不存在，源文件夹将被重命名或移动到该路径。
2. **移动到内部**: 如果 `destination` 是一个已存在的目录，源文件夹将被移动到该目录**内部**。

## 输入参数
- **source**: 源文件夹路径。
- **destination**: 目标路径。
- **overwrite**: 是否覆盖。如果为 true 且目标路径(计算后)已存在，将先删除目标再移动。

## 注意事项
- 跨磁盘移动可能会因为系统不支持简单重命名而失败（取决于操作系统）。
"#,
    }
}

pub fn input_title(field: &str, locale: Locale) -> Option<&'static str> {
    match (locale, field) {
        (Locale::Zh, "source") => Some("源路径"),
        (Locale::Zh, "destination") => Some("目标路径"),
        (Locale::Zh, "overwrite") => Some("覆盖现有"),
        (Locale::Zh, "success") => Some("是否成功"), // Although this is output, keeping generic map
        (Locale::Zh, "moved_files") => Some("移动文件数"),
        _ => None
    }
}

pub fn output_title(field: &str, locale: Locale) -> Option<&'static str> {
    match (locale, field) {
        (Locale::Zh, "success") => Some("是否成功"),
        (Locale::Zh, "moved_files") => Some("移动文件数"),
         _ => None
    }
}
