//! 依赖提供者 trait
//!
//! 提供延迟创建服务实例的能力

use std::sync::Arc;

/// 依赖提供者 trait
///
/// 用于延迟创建服务实例，支持带依赖的创建
pub trait Provider<T>: Send + Sync {
    /// 提供依赖实例
    fn provide(&self) -> Arc<T>;
}

/// 函数提供者
///
/// 使用闭包创建服务实例
pub struct FnProvider<T, F>
where
    T: Send + Sync + 'static,
    F: Fn() -> Arc<T> + Send + Sync,
{
    factory: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T, F> FnProvider<T, F>
where
    T: Send + Sync + 'static,
    F: Fn() -> Arc<T> + Send + Sync,
{
    /// 创建新的函数提供者
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, F> Provider<T> for FnProvider<T, F>
where
    T: Send + Sync + 'static,
    F: Fn() -> Arc<T> + Send + Sync,
{
    fn provide(&self) -> Arc<T> {
        (self.factory)()
    }
}

/// 单例提供者
///
/// 返回预先创建的单例实例
pub struct SingletonProvider<T: Send + Sync + 'static> {
    instance: Arc<T>,
}

impl<T: Send + Sync + 'static> SingletonProvider<T> {
    /// 创建新的单例提供者
    pub fn new(instance: Arc<T>) -> Self {
        Self { instance }
    }
}

impl<T: Send + Sync + 'static> Provider<T> for SingletonProvider<T> {
    fn provide(&self) -> Arc<T> {
        self.instance.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService {
        value: i32,
    }

    #[test]
    fn test_fn_provider() {
        let provider = FnProvider::new(|| Arc::new(TestService { value: 42 }));

        let instance = provider.provide();
        assert_eq!(instance.value, 42);
    }

    #[test]
    fn test_singleton_provider() {
        let instance = Arc::new(TestService { value: 100 });
        let provider = SingletonProvider::new(instance);

        let first = provider.provide();
        let second = provider.provide();

        assert!(Arc::ptr_eq(&first, &second));
    }
}
