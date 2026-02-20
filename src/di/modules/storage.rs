//! 存储模块
//!
//! 注册状态管理器、存储后端等服务

use crate::di::{AppModule, DiContainer};
use crate::storage::{FileStorage, SimpleMemoryCache, StateManager};
use std::sync::Arc;
use std::path::PathBuf;

/// 存储模块
///
/// 负责注册状态管理器、存储后端、缓存等服务
pub struct StorageModule {
    storage_path: PathBuf,
    cache_capacity: usize,
}

impl StorageModule {
    /// 创建新的存储模块
    pub fn new() -> Self {
        Self {
            storage_path: PathBuf::from("./data/storage"),
            cache_capacity: 1000,
        }
    }

    /// 设置存储路径
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = path;
        self
    }

    /// 设置缓存容量
    pub fn with_cache_capacity(mut self, capacity: usize) -> Self {
        self.cache_capacity = capacity;
        self
    }
}

impl Default for StorageModule {
    fn default() -> Self {
        Self::new()
    }
}

impl AppModule for StorageModule {
    fn name(&self) -> &str {
        "storage"
    }

    fn configure(&self, container: &DiContainer) {
        let storage_path = self.storage_path.clone();
        let cache_capacity = self.cache_capacity;
        
        container.register_factory::<dyn StateManagerService, _>(move || {
            let storage = Arc::new(
                FileStorage::new(&storage_path)
                    .expect("Failed to create file storage")
            );
            let cache = Arc::new(SimpleMemoryCache::new());
            let state_manager = Arc::new(StateManager::new(storage, cache));
            
            Arc::new(StateManagerServiceImpl::new(state_manager))
        });
        
        container.register_factory::<dyn CacheService, _>(move || {
            Arc::new(CacheServiceImpl::new(cache_capacity))
        });
        
        container.register_factory::<dyn BackupService, _>(|| {
            Arc::new(BackupServiceImpl::new())
        });
    }
}

/// 状态管理器服务 trait
pub trait StateManagerService: Send + Sync {
    /// 转换为 Any 类型用于向下转型
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 状态管理器服务实现
struct StateManagerServiceImpl {
    state_manager: Arc<StateManager>,
}

impl StateManagerServiceImpl {
    fn new(state_manager: Arc<StateManager>) -> Self {
        Self { state_manager }
    }
}

impl StateManagerService for StateManagerServiceImpl {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 缓存服务 trait
pub trait CacheService: Send + Sync {
    /// 获取缓存容量
    fn capacity(&self) -> usize;
    
    /// 清空缓存
    fn clear(&self);
}

/// 缓存服务实现
struct CacheServiceImpl {
    capacity: usize,
}

impl CacheServiceImpl {
    fn new(capacity: usize) -> Self {
        Self { capacity }
    }
}

impl CacheService for CacheServiceImpl {
    fn capacity(&self) -> usize {
        self.capacity
    }
    
    fn clear(&self) {
    }
}

/// 备份服务 trait
pub trait BackupService: Send + Sync {
    /// 获取备份状态
    fn status(&self) -> BackupStatus;
}

/// 备份状态
#[derive(Debug, Clone)]
pub struct BackupStatus {
    pub last_backup: Option<String>,
    pub backup_count: usize,
}

/// 备份服务实现
struct BackupServiceImpl {
    backup_count: usize,
}

impl BackupServiceImpl {
    fn new() -> Self {
        Self { backup_count: 0 }
    }
}

impl BackupService for BackupServiceImpl {
    fn status(&self) -> BackupStatus {
        BackupStatus {
            last_backup: None,
            backup_count: self.backup_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_module_creation() {
        let module = StorageModule::new();
        assert_eq!(module.name(), "storage");
    }

    #[test]
    fn test_storage_module_dependencies() {
        let module = StorageModule::new();
        let deps = module.dependencies();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_storage_module_with_config() {
        let module = StorageModule::new()
            .with_storage_path(PathBuf::from("/tmp/storage"))
            .with_cache_capacity(2000);
        
        assert_eq!(module.storage_path, PathBuf::from("/tmp/storage"));
        assert_eq!(module.cache_capacity, 2000);
    }
}
