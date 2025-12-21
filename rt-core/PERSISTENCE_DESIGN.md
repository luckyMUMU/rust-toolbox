# 模块名称：持久化模块

## 1. 目标 (Goal)
- **核心功能**：为工具包提供统一的数据存储、缓存、配置管理和文件操作服务。采用分层设计，结合内存缓存 (`moka`) 和嵌入式 KV 数据库 (`sled`)，支持数据压缩 (`zstd`) 和高效序列化 (`bincode`)，同时提供安全可靠的本地文件操作。
- **非目标**：不处理分布式存储（未来增强），不提供加密功能（未来增强）。

## 2. 核心数据结构 (Data Structures)
- **PersistenceManager**：统一的持久化管理器，包含缓存、存储、配置和文件操作功能。
- **Storage**：抽象存储接口，定义了数据的基本操作。
- **SledBackend**：基于 sled 的存储实现，支持数据压缩和高效序列化。
- **CacheLayer**：基于 moka 的缓存实现，支持 TTL/TTI 配置。
- **FileOperations**：安全的文件操作实现，支持原子操作和自动清理。

## 3. 算法与逻辑设计 (Algorithm & Logic)

### 核心流程

#### 数据存储流程
1. **序列化**：使用 `bincode` 将数据序列化为二进制格式。
2. **压缩**：使用 `zstd` 压缩序列化后的数据，减少存储占用。
3. **存储更新**：将压缩数据写入 `sled` 数据库。
4. **缓存更新**：将压缩数据插入 `moka` 缓存，设置 TTL/TTI。

#### 数据读取流程
1. **缓存检查**：从 `moka` 缓存中查找数据。
   - 如果命中，直接解压缩并反序列化返回。
   - 如果未命中，从 `sled` 数据库读取。
2. **存储读取**：从 `sled` 数据库读取压缩数据。
3. **解压缩**：使用 `zstd` 解压缩数据。
4. **反序列化**：使用 `bincode` 反序列化为原始数据。
5. **缓存回填**：将数据插入 `moka` 缓存，设置 TTL/TTI。
6. **返回结果**：返回反序列化后的数据。

### 架构设计

```mermaid
graph TD
    subgraph Client [工具 / 插件]
        API[PersistenceManager]
    end

    subgraph Core [rt-core::persistence]
        API -->|Get/Set| Cache[Moka 缓存层]
        Cache -->|Miss/Evict| Storage[存储层]
        
        Storage -->|序列化| Bincode
        Bincode -->|压缩| Zstd
        Zstd -->|写入| Sled[Sled 数据库]
        
        API -->|配置| ConfigMgr[配置管理器 (confy)]
        API -->|临时文件| TempMgr[临时文件管理器]
        API -->|文件操作| FileOps[文件操作]
        
        FileOps -->|原子写入| TempFile[临时文件]
        FileOps -->|安全替换| AtomicOps[原子操作]
        FileOps -->|编码检测| EncodingMgr[编码管理器]
    end
```

## 4. 接口契约 (Interface)

### 4.1 存储接口

#### Storage 特性
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    /// 根据键获取值
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// 设置键值对
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    
    /// 删除键
    async fn remove(&self, key: &str) -> Result<()>;
    
    /// 刷新挂起的写入
    async fn flush(&self) -> Result<()>;
}
```

### 4.2 PersistenceManager 接口

#### 数据操作
```rust
impl PersistenceManager {
    /// 获取 KV 数据（优先查缓存，未命中则查数据库）
    pub async fn get_data<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    
    /// 保存 KV 数据（同时更新缓存和数据库）
    pub async fn set_data<T: Serialize + ?Sized>(&self, key: &str, value: &T) -> Result<()>;
    
    /// 加载配置（应用级或工具级）
    pub fn load_config<T: Serialize + DeserializeOwned + Default>(&self, app_name: &str, config_name: &str) -> Result<T>;
    
    /// 保存配置
    pub fn save_config<T: Serialize>(&self, app_name: &str, config_name: &str, config: &T) -> Result<()>;
    
    /// 创建临时目录（自动清理）
    pub async fn create_temp_dir(&self) -> Result<TempDir>;
    
    /// 创建本地文件，带有可选的初始内容
    pub async fn create_file(&self, path: &std::path::Path, content: Option<&str>) -> Result<()>;
    
    /// 读取本地文件，带有编码检测
    pub async fn read_file(&self, path: &std::path::Path) -> Result<(String, String)>;
    
    /// 使用双缓冲区安全机制更新本地文件
    pub async fn update_file(&self, path: &std::path::Path, content: &str) -> Result<()>;
    
    /// 删除本地文件，带有存在性检查
    pub async fn delete_file(&self, path: &std::path::Path) -> Result<()>;
}
```

### 4.3 错误条件
- 源路径不存在
- 目标存在且覆盖为 false
- 权限被拒绝
- 无效路径格式
- 序列化/反序列化失败
- 压缩/解压缩失败

## 5. 变更记录 (Status)
> 格式：[状态] | 变更描述 | 日期

### 当前变更
- `[已完成]`：更新文档结构，统一命名规范，添加变更记录 | 2025-12-21

### 历史记录
- `[已完成]`：初始设计文档创建 | 2025-12-20

## 附加信息

### 技术栈
- **配置管理**：`confy`（简化的 TOML/YAML 配置文件读写）
- **内存缓存**：`moka`（高性能并发缓存，支持 TTL/TTI）
- **嵌入式数据库**：`sled`（纯 Rust 现代 KV 数据库）
- **序列化**：`bincode`（二进制序列化，紧凑且快速）
- **压缩**：`zstd`（Zstandard 压缩算法，高压缩比）
- **临时文件**：`tempfile`（安全的临时文件/目录创建）
- **异步 I/O**：`tokio::fs`（异步文件系统操作）
- **文件操作**：自定义实现，带有原子操作和安全保证

### 核心组件
1. **PersistenceManager**：统一入口点，公开缓存、存储、配置和文件操作逻辑
2. **CacheLayer**：基于 `moka` 的异步缓存，用于处理热点数据
3. **StorageBackend**：抽象存储接口，默认实现为 `SledBackend`
4. **ConfigManager**：封装 `confy`，提供类型安全的配置读写
5. **FileOperations**：提供安全可靠的本地文件操作

### 性能优化

#### 缓存策略
- **多级缓存**：带有可配置 TTL/TTI 的内存缓存
- **缓存预热**：为频繁访问的数据主动填充缓存
- **内存管理**：基于大小和时间限制的自动缓存驱逐
- **缓存压缩**：在缓存中存储压缩数据以减少内存使用

#### I/O 优化
- **异步操作**：所有文件操作都是完全异步的
- **批处理操作**：支持批量读写操作
- **流式处理**：支持大型文件的流式处理
- **连接池**：高效的数据库连接管理

### 安全考虑

#### 文件系统安全
- **路径遍历保护**：防止目录遍历攻击
- **权限验证**：在操作前验证文件系统权限
- **安全临时文件**：使用安全的临时文件创建
- **原子操作**：防止竞争条件和部分写入

#### 数据安全
- **输入验证**：对所有操作进行全面的输入验证
- **错误信息**：谨慎处理错误消息，防止信息泄露
- **访问控制**：与服务层权限系统集成
- **数据清理**：适当的数据清理，用于日志和错误报告

### 测试与验证

#### 单元测试
持久化模块包含全面的单元测试，涵盖：
```rust
#[cfg(test)]
mod tests {
    // 文件操作测试
    #[tokio::test]
    async fn test_create_file() { /* ... */ }
    
    #[tokio::test]
    async fn test_read_file() { /* ... */ }
    
    #[tokio::test]
    async fn test_update_file() { /* ... */ }
    
    #[tokio::test]
    async fn test_delete_file() { /* ... */ }
    
    // 持久化管理器测试
    #[tokio::test]
    async fn test_persistence_manager() { /* ... */ }
}
```

#### 集成测试
- **跨平台测试**：在 Windows、Linux 和 macOS 上验证操作
- **并发访问**：测试并发读写操作
- **错误场景**：测试错误处理和恢复机制
- **性能测试**：在各种负载下进行基准测试

### 迁移与兼容性

#### 数据迁移
- **版本检测**：自动检测数据格式版本
- **迁移脚本**：数据格式版本之间的自动迁移
- **向后兼容**：支持读取旧数据格式
- **回滚支持**：失败迁移的安全回滚机制

#### API 兼容性
- **语义版本控制**：API 变更遵循语义版本控制
- **弃用警告**：API 变更的明确弃用警告
- **迁移指南**：主要版本变更的综合迁移指南

### 监控与可观察性

#### 指标收集
- **操作指标**：跟踪文件操作成功/失败率
- **性能指标**：监控操作延迟和吞吐量
- **缓存指标**：跟踪缓存命中率和内存使用情况
- **存储指标**：监控存储使用情况和增长

#### 日志集成
- **结构化日志**：与 rt-core 日志系统集成
- **操作跟踪**：文件操作的详细跟踪
- **错误日志**：带有上下文的全面错误日志
- **性能日志**：性能计时和统计

### 未来增强

#### 计划功能
- **分布式存储**：支持分布式存储后端
- **加密**：敏感数据的内置加密
- **备份和恢复**：自动化备份和恢复功能
- **复制**：高可用性的数据复制

#### 性能改进
- **高级缓存**：更复杂的缓存策略
- **并行操作**：增强的并行处理能力
- **内存优化**：进一步的内存使用优化
- **网络优化**：分布式场景的优化网络操作

### 结论

持久化模块为 rt-box 系统提供了强大、安全、高效的数据存储和文件操作基础。其分层架构、全面的错误处理和广泛的测试确保了数据完整性和系统可靠性，同时通过缓存和压缩优化提供了出色的性能。

该模块的设计允许轻松扩展和定制，同时保持向后兼容性，并为未来增强提供清晰的迁移路径。