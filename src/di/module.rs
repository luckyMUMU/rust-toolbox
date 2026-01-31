//! DI模块定义

use shaku::Module;

/// 应用模块trait
pub trait AppModule: Module {
    /// 模块初始化
    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
}
