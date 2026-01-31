//! DI模块定义

/// 应用模块trait（占位）
pub trait AppModule {
    /// 模块初始化
    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
}
