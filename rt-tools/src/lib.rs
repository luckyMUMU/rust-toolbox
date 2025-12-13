pub mod file;
pub mod text;

pub use file::move_folder::MoveFolder;
pub use text::convert_chinese::ConvertChinese;

pub fn get_all_tools() -> Vec<Box<dyn rt_core::Tool>> {
    vec![
        Box::new(MoveFolder),
        Box::new(ConvertChinese),
    ]
}
