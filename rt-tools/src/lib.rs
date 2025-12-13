pub mod file;

pub use file::move_folder::MoveFolder;

pub fn get_all_tools() -> Vec<Box<dyn rt_core::Tool>> {
    vec![
        Box::new(MoveFolder),
    ]
}
