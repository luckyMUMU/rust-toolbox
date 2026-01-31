//! 依赖提供者trait

/// 依赖提供者
pub trait Provider<T> {
    /// 提供依赖实例
    fn provide(&self) -> T;
}
