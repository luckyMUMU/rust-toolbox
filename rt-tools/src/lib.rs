pub mod file;
pub mod text;
pub mod utils;

pub use file::move_folder::MoveFolder;
pub use text::convert_chinese::ConvertChinese;

use rt_core::Tool;
use std::sync::Mutex;

// 工具注册中心
lazy_static::lazy_static! {
    static ref TOOL_REGISTRY: Mutex<Vec<fn() -> Box<dyn Tool>>> = Mutex::new(Vec::new());
}

// 注册工具的宏
#[macro_export]
macro_rules! register_tool {
    ($tool:ty) => {
        lazy_static::lazy_static! {
            static ref _TOOL_REGISTRATION: () = {
                $crate::register_tool_impl(|| Box::new(<$tool>::new()));
            };
        }
    };
}

// 注册工具的实现
fn register_tool_impl(factory: fn() -> Box<dyn Tool>) {
    TOOL_REGISTRY.lock().unwrap().push(factory);
}

// 获取所有注册的工具
pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    TOOL_REGISTRY.lock().unwrap().iter().map(|factory| factory()).collect()
}
