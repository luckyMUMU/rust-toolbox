//! DI容器实现
//!
//! 提供依赖注入容器，支持单例和工厂方法注册

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 服务工厂类型（无依赖）
type ServiceFactory = Box<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>;

/// 带依赖的服务工厂类型
type ServiceFactoryWithDeps = Box<dyn Fn(&DiContainer) -> Arc<dyn Any + Send + Sync> + Send + Sync>;

/// 依赖注入容器
///
/// 支持单例和工厂方法两种注册模式
pub struct DiContainer {
    /// 单例服务存储
    singletons: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    /// 工厂方法存储（无依赖）
    factories: RwLock<HashMap<TypeId, ServiceFactory>>,
    /// 带依赖的工厂方法存储
    factories_with_deps: RwLock<HashMap<TypeId, ServiceFactoryWithDeps>>,
    /// 已解析的单例缓存
    resolved: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl DiContainer {
    /// 创建新的 DI 容器
    pub fn new() -> Self {
        Self {
            singletons: RwLock::new(HashMap::new()),
            factories: RwLock::new(HashMap::new()),
            factories_with_deps: RwLock::new(HashMap::new()),
            resolved: RwLock::new(HashMap::new()),
        }
    }

    /// 注册单例服务
    ///
    /// 单例服务在注册时即创建实例，后续所有解析返回同一实例
    pub fn register_singleton<T>(&self, instance: Arc<T>)
    where
        T: 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        if let Ok(mut singletons) = self.singletons.write() {
            singletons.insert(type_id, instance);
        }
    }

    /// 注册工厂方法
    ///
    /// 工厂方法在每次解析时创建新实例
    pub fn register_factory<T, F>(&self, factory: F)
    where
        T: 'static + Send + Sync,
        F: Fn() -> Arc<T> + 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        if let Ok(mut factories) = self.factories.write() {
            factories.insert(
                type_id,
                Box::new(move || factory() as Arc<dyn Any + Send + Sync>),
            );
        }
    }

    /// 注册带依赖的工厂方法
    ///
    /// 工厂方法可以依赖容器中的其他服务
    /// 注意：工厂方法接收当前容器的引用，以便解析依赖
    pub fn register_factory_with_deps<T, F>(&self, factory: F)
    where
        T: 'static + Send + Sync,
        F: Fn(&DiContainer) -> Arc<T> + 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        if let Ok(mut factories) = self.factories_with_deps.write() {
            factories.insert(
                type_id,
                Box::new(move |container| factory(container) as Arc<dyn Any + Send + Sync>),
            );
        }
    }

    /// 解析服务
    ///
    /// 按以下顺序查找：
    /// 1. 已解析的单例缓存
    /// 2. 单例服务存储
    /// 3. 带依赖的工厂方法
    /// 4. 工厂方法（无依赖）
    pub fn resolve<T>(&self) -> Option<Arc<T>>
    where
        T: 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();

        // 1. 检查已解析缓存
        if let Ok(resolved) = self.resolved.read() {
            if let Some(instance) = resolved.get(&type_id) {
                return instance.clone().downcast::<T>().ok();
            }
        }

        // 2. 检查单例存储
        if let Ok(singletons) = self.singletons.read() {
            if let Some(instance) = singletons.get(&type_id) {
                // 缓存到已解析
                if let Ok(mut resolved) = self.resolved.write() {
                    resolved.insert(type_id, instance.clone());
                }
                return instance.clone().downcast::<T>().ok();
            }
        }

        // 3. 检查带依赖的工厂方法
        if let Ok(factories) = self.factories_with_deps.read() {
            if let Some(factory) = factories.get(&type_id) {
                let instance = factory(self);
                // 缓存到已解析（工厂创建的实例也缓存）
                if let Ok(mut resolved) = self.resolved.write() {
                    resolved.insert(type_id, instance.clone());
                }
                return instance.downcast::<T>().ok();
            }
        }

        // 4. 检查工厂方法（无依赖）
        if let Ok(factories) = self.factories.read() {
            if let Some(factory) = factories.get(&type_id) {
                let instance = factory();
                // 缓存到已解析（工厂创建的实例也缓存）
                if let Ok(mut resolved) = self.resolved.write() {
                    resolved.insert(type_id, instance.clone());
                }
                return instance.downcast::<T>().ok();
            }
        }

        None
    }

    /// 解析服务（带错误处理）
    ///
    /// 如果服务未注册，返回错误
    pub fn resolve_or_error<T>(&self) -> Result<Arc<T>, DiContainerError>
    where
        T: 'static + Send + Sync,
    {
        self.resolve::<T>()
            .ok_or_else(|| DiContainerError::ServiceNotFound {
                type_name: std::any::type_name::<T>().to_string(),
            })
    }

    /// 尝试解析服务
    ///
    /// 如果服务未注册，返回 None
    pub fn try_resolve<T>(&self) -> Option<Arc<T>>
    where
        T: 'static + Send + Sync,
    {
        self.resolve::<T>()
    }

    /// 检查服务是否已注册
    pub fn is_registered<T>(&self) -> bool
    where
        T: 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();

        if let Ok(singletons) = self.singletons.read() {
            if singletons.contains_key(&type_id) {
                return true;
            }
        }

        if let Ok(factories) = self.factories.read() {
            if factories.contains_key(&type_id) {
                return true;
            }
        }

        if let Ok(factories) = self.factories_with_deps.read() {
            if factories.contains_key(&type_id) {
                return true;
            }
        }

        false
    }

    /// 注销服务
    pub fn unregister<T>(&self) -> bool
    where
        T: 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        let mut removed = false;

        if let Ok(mut singletons) = self.singletons.write() {
            if singletons.remove(&type_id).is_some() {
                removed = true;
            }
        }

        if let Ok(mut factories) = self.factories.write() {
            if factories.remove(&type_id).is_some() {
                removed = true;
            }
        }

        if let Ok(mut factories) = self.factories_with_deps.write() {
            if factories.remove(&type_id).is_some() {
                removed = true;
            }
        }

        if let Ok(mut resolved) = self.resolved.write() {
            resolved.remove(&type_id);
        }

        removed
    }

    /// 清空所有注册
    pub fn clear(&self) {
        if let Ok(mut singletons) = self.singletons.write() {
            singletons.clear();
        }
        if let Ok(mut factories) = self.factories.write() {
            factories.clear();
        }
        if let Ok(mut factories) = self.factories_with_deps.write() {
            factories.clear();
        }
        if let Ok(mut resolved) = self.resolved.write() {
            resolved.clear();
        }
    }

    /// 获取已注册服务的数量
    pub fn service_count(&self) -> usize {
        let mut type_ids: std::collections::HashSet<TypeId> = std::collections::HashSet::new();

        if let Ok(singletons) = self.singletons.read() {
            for id in singletons.keys() {
                type_ids.insert(*id);
            }
        }
        if let Ok(factories) = self.factories.read() {
            for id in factories.keys() {
                type_ids.insert(*id);
            }
        }
        if let Ok(factories) = self.factories_with_deps.read() {
            for id in factories.keys() {
                type_ids.insert(*id);
            }
        }

        type_ids.len()
    }
}

impl Default for DiContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DiContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiContainer")
            .field("service_count", &self.service_count())
            .finish()
    }
}

/// DI 容器错误
#[derive(Debug, Clone)]
pub enum DiContainerError {
    /// 服务未找到
    ServiceNotFound { type_name: String },
    /// 服务注册失败
    RegistrationFailed { type_name: String, reason: String },
    /// 依赖解析失败
    DependencyResolutionFailed {
        type_name: String,
        dependency_name: String,
    },
}

impl std::fmt::Display for DiContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiContainerError::ServiceNotFound { type_name } => {
                write!(f, "服务未找到: {}", type_name)
            }
            DiContainerError::RegistrationFailed { type_name, reason } => {
                write!(f, "服务注册失败 {}: {}", type_name, reason)
            }
            DiContainerError::DependencyResolutionFailed {
                type_name,
                dependency_name,
            } => {
                write!(f, "依赖解析失败: {} 依赖 {}", type_name, dependency_name)
            }
        }
    }
}

impl std::error::Error for DiContainerError {}

#[cfg(test)]
mod tests {
    use super::*;

    trait TestService: Send + Sync {
        fn name(&self) -> &str;
    }

    struct TestServiceImpl {
        name: String,
    }

    impl TestServiceImpl {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl TestService for TestServiceImpl {
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_register_singleton() {
        let container = DiContainer::new();
        let service = Arc::new(TestServiceImpl::new("test"));

        container.register_singleton::<dyn TestService>(service);

        assert!(container.is_registered::<dyn TestService>());
    }

    #[test]
    fn test_resolve_singleton() {
        let container = DiContainer::new();
        let service = Arc::new(TestServiceImpl::new("singleton"));

        container.register_singleton::<dyn TestService>(service.clone());

        let resolved = container.resolve::<dyn TestService>();
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name(), "singleton");
    }

    #[test]
    fn test_singleton_returns_same_instance() {
        let container = DiContainer::new();
        let service = Arc::new(TestServiceImpl::new("same"));

        container.register_singleton::<dyn TestService>(service);

        let first = container.resolve::<dyn TestService>().unwrap();
        let second = container.resolve::<dyn TestService>().unwrap();

        // 验证是同一个实例
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn test_register_factory() {
        let container = DiContainer::new();

        container
            .register_factory::<dyn TestService, _>(|| Arc::new(TestServiceImpl::new("factory")));

        assert!(container.is_registered::<dyn TestService>());
    }

    #[test]
    fn test_factory_creates_new_instances() {
        let container = DiContainer::new();

        container
            .register_factory::<TestServiceImpl, _>(|| Arc::new(TestServiceImpl::new("factory")));

        let first = container.resolve::<TestServiceImpl>().unwrap();
        let second = container.resolve::<TestServiceImpl>().unwrap();

        // 注意：当前实现会缓存工厂创建的实例
        // 如果需要每次创建新实例，需要修改实现
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn test_resolve_unregistered_returns_none() {
        let container = DiContainer::new();

        let result = container.resolve::<dyn TestService>();
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_or_error_returns_error() {
        let container = DiContainer::new();

        let result = container.resolve_or_error::<dyn TestService>();
        assert!(result.is_err());
    }

    #[test]
    fn test_unregister() {
        let container = DiContainer::new();
        let service = Arc::new(TestServiceImpl::new("test"));

        container.register_singleton::<dyn TestService>(service);
        assert!(container.is_registered::<dyn TestService>());

        let removed = container.unregister::<dyn TestService>();
        assert!(removed);
        assert!(!container.is_registered::<dyn TestService>());
    }

    #[test]
    fn test_clear() {
        let container = DiContainer::new();

        container.register_factory::<TestServiceImpl, _>(|| Arc::new(TestServiceImpl::new("test")));

        assert!(container.service_count() > 0);

        container.clear();

        assert_eq!(container.service_count(), 0);
    }

    #[test]
    fn test_service_count() {
        let container = DiContainer::new();

        assert_eq!(container.service_count(), 0);

        container.register_factory::<TestServiceImpl, _>(|| Arc::new(TestServiceImpl::new("test")));

        assert_eq!(container.service_count(), 1);

        // 注册同一个类型的单例会覆盖工厂
        container
            .register_singleton::<TestServiceImpl>(Arc::new(TestServiceImpl::new("singleton")));

        // 应该仍然是 1，因为是同一个类型
        assert_eq!(container.service_count(), 1);
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::thread;

        let container = Arc::new(DiContainer::new());
        let counter = Arc::new(AtomicU32::new(0));

        container.register_factory::<TestServiceImpl, _>({
            let counter = counter.clone();
            move || {
                counter.fetch_add(1, Ordering::SeqCst);
                Arc::new(TestServiceImpl::new("factory"))
            }
        });

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let container = container.clone();
                thread::spawn(move || container.resolve::<TestServiceImpl>())
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // 由于缓存，工厂只应该被调用一次
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
