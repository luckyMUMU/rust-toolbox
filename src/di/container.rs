//! DI容器实现

/// 依赖注入容器（占位实现）
pub struct DiContainer;

impl DiContainer {
    /// 创建容器
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiContainer {
    fn default() -> Self {
        Self::new()
    }
}
