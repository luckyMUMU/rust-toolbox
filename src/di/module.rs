//! DI模块定义
//!
//! 提供模块化的服务注册机制

use super::container::DiContainer;

/// 应用模块 trait
///
/// 模块用于组织和配置相关服务的注册
pub trait AppModule: Send + Sync {
    /// 模块名称
    fn name(&self) -> &str;

    /// 配置服务注册
    ///
    /// 在此方法中注册模块提供的服务
    fn configure(&self, container: &DiContainer);

    /// 模块初始化
    ///
    /// 在所有模块配置完成后调用，用于执行初始化逻辑
    fn initialize(&self, _container: &DiContainer) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// 模块依赖
    ///
    /// 返回此模块依赖的其他模块名称列表
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
}

/// 模块注册器
///
/// 管理多个模块的注册和初始化
pub struct ModuleRegistrar {
    modules: Vec<Box<dyn AppModule>>,
    container: DiContainer,
}

impl ModuleRegistrar {
    /// 创建新的模块注册器
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            container: DiContainer::new(),
        }
    }

    /// 使用现有容器创建注册器
    pub fn with_container(container: DiContainer) -> Self {
        Self {
            modules: Vec::new(),
            container,
        }
    }

    /// 注册模块
    pub fn register<M: AppModule + 'static>(&mut self, module: M) {
        self.modules.push(Box::new(module));
    }

    /// 配置所有模块
    ///
    /// 按依赖顺序配置模块
    pub fn configure_all(&self) -> Result<(), ModuleError> {
        // 检查依赖是否满足
        self.validate_dependencies()?;

        // 按拓扑顺序配置模块
        let ordered = self.topological_sort()?;
        
        for module_name in ordered {
            if let Some(module) = self.modules.iter().find(|m| m.name() == module_name) {
                module.configure(&self.container);
            }
        }

        Ok(())
    }

    /// 初始化所有模块
    pub fn initialize_all(&self) -> Result<(), ModuleError> {
        for module in &self.modules {
            module.initialize(&self.container)
                .map_err(|e| ModuleError::InitializationFailed {
                    module_name: module.name().to_string(),
                    source: e,
                })?;
        }
        Ok(())
    }

    /// 获取容器引用
    pub fn container(&self) -> &DiContainer {
        &self.container
    }

    /// 消费注册器，返回容器
    pub fn into_container(self) -> DiContainer {
        self.container
    }

    /// 验证模块依赖
    fn validate_dependencies(&self) -> Result<(), ModuleError> {
        let module_names: std::collections::HashSet<&str> = 
            self.modules.iter().map(|m| m.name()).collect();

        for module in &self.modules {
            for dep in module.dependencies() {
                if !module_names.contains(dep) {
                    return Err(ModuleError::MissingDependency {
                        module_name: module.name().to_string(),
                        dependency: dep.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// 拓扑排序模块
    fn topological_sort(&self) -> Result<Vec<&str>, ModuleError> {
        use std::collections::{HashMap, VecDeque};
        
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut module_map: HashMap<&str, &dyn AppModule> = HashMap::new();

        // 初始化
        for module in &self.modules {
            let name = module.name();
            in_degree.insert(name, 0);
            graph.insert(name, Vec::new());
            module_map.insert(name, module.as_ref());
        }

        // 构建图
        for module in &self.modules {
            let name = module.name();
            for dep in module.dependencies() {
                graph.get_mut(dep).unwrap().push(name);
                *in_degree.get_mut(name).unwrap() += 1;
            }
        }

        // Kahn 算法
        let mut queue: VecDeque<&str> = VecDeque::new();
        for (name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(name);
            }
        }

        let mut result = Vec::new();
        while let Some(name) = queue.pop_front() {
            result.push(name);
            if let Some(neighbors) = graph.get(name) {
                for &neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // 检查循环依赖
        if result.len() != self.modules.len() {
            return Err(ModuleError::CircularDependency);
        }

        Ok(result)
    }
}

impl Default for ModuleRegistrar {
    fn default() -> Self {
        Self::new()
    }
}

/// 模块错误
#[derive(Debug)]
pub enum ModuleError {
    /// 缺少依赖
    MissingDependency { module_name: String, dependency: String },
    /// 循环依赖
    CircularDependency,
    /// 初始化失败
    InitializationFailed { module_name: String, source: Box<dyn std::error::Error + Send + Sync> },
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::MissingDependency { module_name, dependency } => {
                write!(f, "模块 {} 缺少依赖 {}", module_name, dependency)
            }
            ModuleError::CircularDependency => {
                write!(f, "检测到循环依赖")
            }
            ModuleError::InitializationFailed { module_name, source } => {
                write!(f, "模块 {} 初始化失败: {}", module_name, source)
            }
        }
    }
}

impl std::error::Error for ModuleError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    trait TestService: Send + Sync {
        fn value(&self) -> i32;
    }

    struct TestServiceImpl {
        value: i32,
    }

    impl TestServiceImpl {
        fn new(value: i32) -> Self {
            Self { value }
        }
    }

    impl TestService for TestServiceImpl {
        fn value(&self) -> i32 {
            self.value
        }
    }

    struct TestModule {
        name: String,
        value: i32,
    }

    impl TestModule {
        fn new(name: &str, value: i32) -> Self {
            Self { name: name.to_string(), value }
        }
    }

    impl AppModule for TestModule {
        fn name(&self) -> &str {
            &self.name
        }

        fn configure(&self, container: &DiContainer) {
            let value = self.value;
            container.register_factory::<dyn TestService, _>(move || {
                Arc::new(TestServiceImpl::new(value))
            });
        }
    }

    #[test]
    fn test_module_registration() {
        let mut registrar = ModuleRegistrar::new();
        registrar.register(TestModule::new("test", 42));
        
        assert_eq!(registrar.modules.len(), 1);
    }

    #[test]
    fn test_configure_all() {
        let mut registrar = ModuleRegistrar::new();
        registrar.register(TestModule::new("test", 42));
        
        registrar.configure_all().unwrap();
        
        let service = registrar.container().resolve::<dyn TestService>();
        assert!(service.is_some());
        assert_eq!(service.unwrap().value(), 42);
    }

    #[test]
    fn test_missing_dependency() {
        struct DependentModule;
        
        impl AppModule for DependentModule {
            fn name(&self) -> &str { "dependent" }
            fn configure(&self, _container: &DiContainer) {}
            fn dependencies(&self) -> Vec<&str> { vec!["missing"] }
        }

        let mut registrar = ModuleRegistrar::new();
        registrar.register(DependentModule);
        
        let result = registrar.configure_all();
        assert!(matches!(result, Err(ModuleError::MissingDependency { .. })));
    }

    #[test]
    fn test_dependency_order() {
        struct CoreModule;
        impl AppModule for CoreModule {
            fn name(&self) -> &str { "core" }
            fn configure(&self, _container: &DiContainer) {}
        }

        struct AppModule;
        impl AppModule for AppModule {
            fn name(&self) -> &str { "app" }
            fn configure(&self, _container: &DiContainer) {}
            fn dependencies(&self) -> Vec<&str> { vec!["core"] }
        }

        let mut registrar = ModuleRegistrar::new();
        registrar.register(AppModule);
        registrar.register(CoreModule);
        
        let result = registrar.configure_all();
        assert!(result.is_ok());
    }
}
