//! DI容器实现

use shaku::{Container, ContainerBuilder};

/// 依赖注入容器
pub struct DiContainer {
    inner: Container<dyn super::module::AppModule>,
}

impl DiContainer {
    /// 创建容器构建器
    pub fn builder() -> ContainerBuilder<dyn super::module::AppModule> {
        ContainerBuilder::new()
    }
}
