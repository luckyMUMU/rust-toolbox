# Persistence Module Design Document

## 1. Module Overview
The persistence module (`rt-core::persistence`) provides unified data storage, caching, configuration management, and file operation services for the toolbox. The module adopts a layered design, combining in-memory caching (`moka`) and embedded KV database (`sled`), with support for data compression (`zstd`) and efficient serialization (`bincode`), while providing secure and reliable local file operations.

## 2. Technology Stack
- **Configuration Management**: `confy` (simplified TOML/YAML configuration file read/write)
- **In-Memory Caching**: `moka` (high-performance concurrent cache with TTL/TTI support)
- **Embedded Database**: `sled` (pure Rust modern KV database)
- **Serialization**: `bincode` (binary serialization, compact and fast)
- **Compression**: `zstd` (Zstandard compression algorithm, high compression ratio)
- **Temporary Files**: `tempfile` (secure temporary file/directory creation)
- **Async I/O**: `tokio::fs` (async file system operations)
- **File Operations**: Custom implementation with atomic operations and safety guarantees

## 3. Architecture Design

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
        API -->|File Ops| FileOps[File Operations]
        
        FileOps -->|Atomic Write| TempFile[Temporary Files]
        FileOps -->|Safe Replace| AtomicOps[Atomic Operations]
        FileOps -->|Encoding Detection| EncodingMgr[Encoding Manager]
    end
```

### 3.1 Core Components
1. **PersistenceManager**: Unified entry point exposing caching, storage, configuration, and file operation logic
2. **CacheLayer**: Async cache based on `moka` for handling hot data
3. **StorageBackend**: Abstract storage interface with default `SledBackend` implementation
4. **ConfigManager**: Wraps `confy` providing type-safe configuration read/write
5. **FileOperations**: Provides secure and reliable local file operations including create, read, update, and delete
6. **MCP Support**: Persistence module supports Model Context Protocol (MCP) for standardized tool and plugin interaction
7. **Atomic File Operations**: Implements double-buffering safety mechanism for file updates
8. **Encoding Detection**: Automatic encoding detection and handling for file operations

## 4. Interface Design

### 4.1 Storage Trait
Abstract storage backend interface for pluggable storage implementations:

```rust
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Set value for key
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    
    /// Remove key
    async fn remove(&self, key: &str) -> Result<()>;
    
    /// Flush pending writes
    async fn flush(&self) -> Result<()>;
}
```

### 4.2 PersistenceManager API
Unified persistence management interface with comprehensive file operations:

```rust
pub struct PersistenceManager {
    storage: Arc<dyn Storage>,
    cache: CacheLayer,
}

impl PersistenceManager {
    /// Get KV data (cache first, fallback to DB)
    pub async fn get_data<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    
    /// Save KV data (update both cache and DB)
    pub async fn set_data<T: Serialize + ?Sized>(&self, key: &str, value: &T) -> Result<()>;
    
    /// Load configuration (application or tool level)
    pub fn load_config<T: Serialize + DeserializeOwned + Default>(&self, app_name: &str, config_name: &str) -> Result<T>;
    
    /// Save configuration
    pub fn save_config<T: Serialize>(&self, app_name: &str, config_name: &str, config: &T) -> Result<()>;
    
    /// Create temporary directory (auto-cleanup)
    pub async fn create_temp_dir(&self) -> Result<TempDir>;
    
    /// Create local file with optional initial content
    pub async fn create_file(&self, path: &std::path::Path, content: Option<&str>) -> Result<()>;
    
    /// Read local file with encoding detection
    pub async fn read_file(&self, path: &std::path::Path) -> Result<(String, String)>;
    
    /// Update local file using double-buffer safety mechanism
    pub async fn update_file(&self, path: &std::path::Path, content: &str) -> Result<()>;
    
    /// Delete local file with existence checking
    pub async fn delete_file(&self, path: &std::path::Path) -> Result<()>;
}
```

### 4.3 File Operations Module
Dedicated file operations with safety guarantees:

```rust
/// File creation with directory auto-creation
pub async fn create_file(path: &Path, content: Option<&str>) -> Result<()>;

/// File reading with encoding detection
pub async fn read_file(path: &Path) -> Result<(String, String)>;

/// Atomic file update using temporary file and rename
pub async fn update_file(path: &Path, content: &str) -> Result<()>;

/// Safe file deletion with existence checking
pub async fn delete_file(path: &Path) -> Result<()>;
```

## 5. Data Flow

### 5.1 Write Operations (`set_data`)
1. **Serialization**: `T` -> `bincode` -> `Vec<u8>`
2. **Compression**: `zstd::encode(value)` (level 3 default)
3. **Storage Update**: `sled.insert(key, compressed_value)`
4. **Cache Update**: `moka.insert(key, compressed_value)`

### 5.2 Read Operations (`get_data`)
1. **Cache Check**: `moka.get(key)` -> if hit, decompress and deserialize
2. **Storage Fallback**: `sled.get(key)` if cache miss
3. **Decompression**: `zstd::decode(compressed_value)`
4. **Cache Backfill**: `moka.insert(key, compressed_value)`
5. **Deserialization**: `bincode` -> `T`

### 5.3 File Operations Flow

#### 5.3.1 File Creation (`create_file`)
1. **Directory Check**: Ensure parent directory exists, create if needed
2. **File Creation**: `tokio::fs::File::create(path)`
3. **Content Writing**: Write initial content if provided
4. **Error Handling**: Comprehensive error reporting

#### 5.3.2 File Reading (`read_file`)
1. **File Opening**: `tokio::fs::File::open(path)`
2. **Content Reading**: Read all bytes asynchronously
3. **Encoding Detection**: Attempt UTF-8 decoding, fallback to lossy conversion
4. **Result Return**: Content and detected encoding

#### 5.3.3 Atomic File Update (`update_file`)
1. **Validation**: Check path validity
2. **Temporary File**: Create `NamedTempFile` for atomic operation
3. **Content Writing**: Write new content to temporary file
4. **Integrity Check**: Verify written content matches input
5. **Atomic Replace**: `tokio::fs::rename(temp_path, target_path)`
6. **Error Recovery**: Automatic cleanup on failure

#### 5.3.4 File Deletion (`delete_file`)
1. **Existence Check**: Verify file exists before deletion
2. **Deletion**: `tokio::fs::remove_file(path)`
3. **Error Reporting**: Detailed error information

## 6. Storage Paths and Configuration

### 6.1 Default Storage Locations
- **Windows**: `%APPDATA%\rt-box\data`
- **Linux**: `~/.local/share/rt-box/data`
- **macOS**: `~/Library/Application Support/rt-box/data`
- **Configuration**: System standard configuration paths (handled by `confy`)

### 6.2 Configuration Management
- **Application Config**: `confy::load(app_name, config_name)`
- **Tool-specific Config**: Separate configuration namespaces per tool
- **Format Support**: TOML and YAML configuration files
- **Type Safety**: Strongly typed configuration with serde derive macros

### 6.3 Temporary File Management
- **Auto-cleanup**: Temporary directories automatically cleaned up
- **Secure Creation**: Uses `tempfile` crate for secure temporary file creation
- **Cross-platform**: Works consistently across all supported platforms

## 7. Error Handling and Safety

### 7.1 Error Types
Uses `rt_core::CoreError` variants for comprehensive error handling:

```rust
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Tool execution failed: {0}")]
    ToolFailure(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("YAML serialization error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
```

### 7.2 File Operation Safety
- **Atomic Updates**: Double-buffering mechanism prevents data corruption
- **Integrity Verification**: Content verification before atomic replacement
- **Error Recovery**: Automatic cleanup of temporary files on failure
- **Path Validation**: Comprehensive path validation and sanitization
- **Encoding Handling**: Robust encoding detection and conversion

### 7.3 Data Integrity
- **Compression Verification**: Verify compression/decompression operations
- **Serialization Safety**: Type-safe serialization with error handling
- **Cache Consistency**: Ensure cache and storage consistency
- **Transaction Safety**: Atomic operations for data consistency

## 8. Performance Optimizations

### 8.1 Caching Strategy
- **Multi-level Caching**: In-memory cache with configurable TTL/TTI
- **Cache Warming**: Proactive cache population for frequently accessed data
- **Memory Management**: Automatic cache eviction based on size and time limits
- **Compression in Cache**: Store compressed data in cache to reduce memory usage

### 8.2 I/O Optimizations
- **Async Operations**: All file operations are fully asynchronous
- **Batch Operations**: Support for batch read/write operations
- **Streaming**: Large file handling with streaming support
- **Connection Pooling**: Efficient database connection management

### 8.3 Compression Benefits
- **Storage Efficiency**: Zstd compression reduces storage requirements
- **Network Efficiency**: Compressed data transfer for distributed scenarios
- **Memory Efficiency**: Compressed cache entries reduce memory footprint
- **Performance**: Fast compression/decompression with minimal CPU overhead

## 9. Security Considerations

### 9.1 File System Security
- **Path Traversal Protection**: Prevent directory traversal attacks
- **Permission Validation**: Verify file system permissions before operations
- **Secure Temporary Files**: Use secure temporary file creation
- **Atomic Operations**: Prevent race conditions and partial writes

### 9.2 Data Security
- **Input Validation**: Comprehensive input validation for all operations
- **Error Information**: Careful error message handling to prevent information leakage
- **Access Control**: Integration with service layer permission system
- **Data Sanitization**: Proper data sanitization for logging and error reporting

## 10. Testing and Validation

### 10.1 Unit Testing
The persistence module includes comprehensive unit tests covering:

```rust
#[cfg(test)]
mod tests {
    // File operation tests
    #[tokio::test]
    async fn test_create_file() { /* ... */ }
    
    #[tokio::test]
    async fn test_read_file() { /* ... */ }
    
    #[tokio::test]
    async fn test_update_file() { /* ... */ }
    
    #[tokio::test]
    async fn test_delete_file() { /* ... */ }
    
    #[tokio::test]
    async fn test_file_operations_integration() { /* ... */ }
    
    // Persistence manager tests
    #[tokio::test]
    async fn test_persistence_manager() { /* ... */ }
}
```

### 10.2 Integration Testing
- **Cross-platform Testing**: Verify operations across Windows, Linux, and macOS
- **Concurrent Access**: Test concurrent read/write operations
- **Error Scenarios**: Test error handling and recovery mechanisms
- **Performance Testing**: Benchmark operations under various loads

### 10.3 Property-based Testing
- **File Operation Properties**: Verify file operation invariants
- **Data Consistency**: Ensure cache and storage consistency
- **Encoding Roundtrip**: Verify encoding detection and conversion accuracy

## 11. Usage Examples

### 11.1 Basic File Operations
```rust
use rt_core::PersistenceManager;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("./data");
    let manager = PersistenceManager::new(db_path)?;
    
    // Create a file
    let file_path = PathBuf::from("./test.txt");
    manager.create_file(&file_path, Some("Hello, World!")).await?;
    
    // Read the file
    let (content, encoding) = manager.read_file(&file_path).await?;
    println!("Content: {}, Encoding: {}", content, encoding);
    
    // Update the file
    manager.update_file(&file_path, "Updated content").await?;
    
    // Delete the file
    manager.delete_file(&file_path).await?;
    
    Ok(())
}
```

### 11.2 Data Storage and Retrieval
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct UserData {
    id: u32,
    name: String,
    settings: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("./data");
    let manager = PersistenceManager::new(db_path)?;
    
    let user_data = UserData {
        id: 1,
        name: "Alice".to_string(),
        settings: vec!["theme:dark".to_string(), "lang:en".to_string()],
    };
    
    // Store data
    manager.set_data("user:1", &user_data).await?;
    
    // Retrieve data
    let retrieved: Option<UserData> = manager.get_data("user:1").await?;
    println!("Retrieved: {:?}", retrieved);
    
    Ok(())
}
```

### 11.3 Configuration Management
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    theme: String,
    language: String,
    auto_save: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("./data");
    let manager = PersistenceManager::new(db_path)?;
    
    // Load configuration
    let mut config: AppConfig = manager.load_config("rt-box", "app")?;
    
    // Modify configuration
    config.theme = "dark".to_string();
    config.auto_save = true;
    
    // Save configuration
    manager.save_config("rt-box", "app", &config)?;
    
    Ok(())
}
```

## 12. Migration and Compatibility

### 12.1 Data Migration
- **Version Detection**: Automatic detection of data format versions
- **Migration Scripts**: Automated migration between data format versions
- **Backward Compatibility**: Support for reading older data formats
- **Rollback Support**: Safe rollback mechanisms for failed migrations

### 12.2 API Compatibility
- **Semantic Versioning**: Follow semantic versioning for API changes
- **Deprecation Warnings**: Clear deprecation warnings for API changes
- **Migration Guides**: Comprehensive migration guides for major version changes

## 13. Monitoring and Observability

### 13.1 Metrics Collection
- **Operation Metrics**: Track file operation success/failure rates
- **Performance Metrics**: Monitor operation latency and throughput
- **Cache Metrics**: Track cache hit rates and memory usage
- **Storage Metrics**: Monitor storage usage and growth

### 13.2 Logging Integration
- **Structured Logging**: Integration with rt-core logging system
- **Operation Tracing**: Detailed tracing of file operations
- **Error Logging**: Comprehensive error logging with context
- **Performance Logging**: Performance metrics logging

## 14. Future Enhancements

### 14.1 Planned Features
- **Distributed Storage**: Support for distributed storage backends
- **Encryption**: Built-in encryption for sensitive data
- **Backup and Restore**: Automated backup and restore functionality
- **Replication**: Data replication for high availability

### 14.2 Performance Improvements
- **Advanced Caching**: More sophisticated caching strategies
- **Parallel Operations**: Enhanced parallel processing capabilities
- **Memory Optimization**: Further memory usage optimizations
- **Network Optimization**: Optimized network operations for distributed scenarios

## 15. Conclusion

The persistence module provides a robust, secure, and efficient foundation for data storage and file operations in the rt-box system. With its layered architecture, comprehensive error handling, and extensive testing, it ensures data integrity and system reliability while providing excellent performance through caching and compression optimizations.

The module's design allows for easy extension and customization while maintaining backward compatibility and providing clear migration paths for future enhancements.