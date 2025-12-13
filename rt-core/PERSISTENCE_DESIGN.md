# Persistence Module Design Document

## 1. 模块概述 (Module Overview)
持久化模块 (`rt-core::persistence`) 旨在为工具箱提供统一的数据存储、缓存和配置管理服务。
该模块采用分层设计，结合了内存缓存 (`moka`) 和嵌入式 KV 数据库 (`sled`)，并支持数据压缩 (`zstd`) 和高效序列化 (`bincode`)。

## 2. 技术选型 (Tech Stack)
- **配置管理**: `confy` (简化 TOML/YAML 配置文件的读写)
- **内存缓存**: `moka` (高性能并发缓存，支持 TTL/TTI)
- **嵌入式数据库**: `sled` (纯 Rust 实现的现代 KV 数据库)
- **序列化**: `bincode` (二进制序列化，紧凑且快)
- **压缩**: `zstd` (Zstandard 压缩算法，高压缩比)
- **临时文件**: `tempfile` (安全创建临时文件/目录)
- **异步 IO**: `tokio::fs`

## 3. 架构设计 (Architecture)

```mermaid
graph TD
    subgraph Client [Tool / Plugin]
        API[PersistenceManager]
    end

    subgraph Core [rt-core::persistence]
        API -->|Get/Set| Cache[Moka Cache Layer]
        Cache -->|Miss/Evict| Storage[Storage Layer]
        
        Storage -->|Serialize| Bincode
        Bincode -->|Compress| Zstd
        Zstd -->|Write| Sled[Sled DB]
        
        API -->|Config| ConfigMgr[Config Manager (confy)]
        API -->|Temp| TempMgr[Temp File Manager]
    end
```

### 3.1 核心组件
1.  **PersistenceManager**: 对外暴露的统一入口，封装缓存、存储和配置逻辑。
2.  **CacheLayer**: 基于 `moka` 的异步缓存，处理热点数据。
3.  **StorageBackend**: 抽象存储接口，默认实现为 `SledBackend`。
4.  **ConfigManager**: 封装 `confy`，提供类型安全的配置读写。

## 4. 接口设计 (Interface Design)

### 4.1 Storage Trait
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn remove(&self, key: &str) -> Result<()>;
    async fn flush(&self) -> Result<()>;
}
```

### 4.2 PersistenceManager API
```rust
pub struct PersistenceManager {
    // ... fields
}

impl PersistenceManager {
    /// 获取 KV 数据 (优先查缓存，未命中查 DB)
    pub async fn get_data<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    
    /// 保存 KV 数据 (同时更新缓存和 DB)
    pub async fn set_data<T: Serialize>(&self, key: &str, value: &T) -> Result<()>;
    
    /// 加载配置 (应用级或工具级)
    pub fn load_config<T: Serialize + DeserializeOwned + Default>(&self, app_name: &str) -> Result<T>;
    
    /// 保存配置
    pub fn save_config<T: Serialize>(&self, app_name: &str, config: &T) -> Result<()>;
    
    /// 创建临时目录 (自动清理)
    pub async fn create_temp_dir(&self) -> Result<TempDir>;
}
```

## 5. 数据流 (Data Flow)

### 5.1 写操作 (`set_data`)
1.  序列化: `T` -> `bincode` -> `Vec<u8>`
2.  更新缓存: `moka.insert(key, value)`
3.  压缩: `zstd::encode(value)`
4.  写入存储: `sled.insert(key, compressed_value)`

### 5.2 读操作 (`get_data`)
1.  查缓存: `moka.get(key)` -> 命中则反序列化返回。
2.  未命中: `sled.get(key)`
3.  解压: `zstd::decode(compressed_value)`
4.  写入缓存 (回填): `moka.insert(key, decompressed_value)`
5.  反序列化: `bincode` -> `T`

## 6. 存储路径
- **Windows**: `%APPDATA%\rt-box\data`
- **Linux**: `~/.local/share/rt-box/data`
- **Config**: 使用系统标准配置路径 (由 `confy` 处理)。

## 7. 错误处理
使用 `rt_core::CoreError` 的变体：
- `StorageError(String)`: IO 或 DB 错误。
- `SerializationError(String)`: 序列化/反序列化失败。
