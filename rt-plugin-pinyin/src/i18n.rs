use rt_core::{Locale, plugin::LocalizedString};
use rt_tools::utils::i18n::ToolI18n;
use std::collections::HashMap;

pub fn get_i18n() -> ToolI18n {
    ToolI18n::new(
        include_str!("../locales/tool.en.json"),
        include_str!("../locales/tool.zh.json"),
    )
}

pub fn display_name(locale: Locale) -> String {
    get_i18n().display_name(locale).to_string()
}

pub fn description(locale: Locale) -> String {
    get_i18n().description(locale).to_string()
}

pub fn user_guide(locale: Locale) -> String {
    get_i18n().user_guide(locale).to_string()
}

pub fn get_input_field_map() -> HashMap<String, LocalizedString> {
    let i18n = get_i18n();
    let en_fields = &i18n.get(Locale::En).input_schema;
    let zh_fields = &i18n.get(Locale::Zh).input_schema;
    
    let mut map = HashMap::new();
    for (key, field) in en_fields {
        let zh_title = zh_fields.get(key).map(|f| f.title.clone());
        map.insert(key.clone(), LocalizedString {
            en: field.title.clone(),
            zh: zh_title,
        });
    }
    map
}

pub fn get_output_field_map() -> HashMap<String, LocalizedString> {
    let i18n = get_i18n();
    let en_fields = &i18n.get(Locale::En).output_schema;
    let zh_fields = &i18n.get(Locale::Zh).output_schema;
    
    let mut map = HashMap::new();
    for (key, field) in en_fields {
        let zh_title = zh_fields.get(key).map(|f| f.title.clone());
        map.insert(key.clone(), LocalizedString {
            en: field.title.clone(),
            zh: zh_title,
        });
    }
    map
}
