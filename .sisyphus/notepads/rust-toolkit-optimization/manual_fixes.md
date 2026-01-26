# 手动修复记录

## 修复策略
1. **使用 `?` 操作符**: 对于可恢复错误
2. **使用 `expect()`**: 对于不可恢复错误，提供详细错误信息
3. **模式匹配**: 对于 Option 类型
4. **unwrap_or_else**: 对于有默认值的情况

## 已修复的文件

### src/config.rs

#### 第 287-288 行 - update_config()
```rust
// 修复前
let mut config = self.config.write().unwrap();
let mut sources = self.sources.write().unwrap();

// 修复后
let mut config = self.config.write()
    .expect("Config RwLock poisoned during update");
let mut sources = self.sources.write()
    .expect("Sources RwLock poisoned during update");
```

#### 第 486 行 - get_sources()
```rust
// 修复前
self.sources.read().unwrap().clone()

// 修复后
self.sources.read()
    .expect("Sources RwLock poisoned")
    .clone()
```

#### 第 537-538 行 - reload_config_file()
```rust
// 修复前
let mut config = self.config.write().unwrap();
let mut sources = self.sources.write().unwrap();

// 修复后
let mut config = self.config.write()
    .expect("Config RwLock poisoned during reload");
let mut sources = self.sources.write()
    .expect("Sources RwLock poisoned during reload");
```

#### 第 799-842 行 - 测试代码
```rust
// 修复前
let temp_dir = TempDir::new().unwrap();
std::fs::write(&config_file, initial_config).unwrap();
let manager = Config::load_from_path_with_priority(&config_file).unwrap();
manager.start_hot_reload().await.unwrap();
std::fs::write(&config_file, updated_config).unwrap();
manager.reload_from_file().await.unwrap();

// 修复后
let temp_dir = TempDir::new()
    .expect("Failed to create temp directory for test");
std::fs::write(&config_file, initial_config)
    .expect("Failed to write test config file");
let manager = Config::load_from_path_with_priority(&config_file)
    .expect("Failed to load config from path");
manager.start_hot_reload().await
    .expect("Failed to start hot reload");
std::fs::write(&config_file, updated_config)
    .expect("Failed to write updated config");
manager.reload_from_file().await
    .expect("Failed to reload config from file");
```

#### 第 859, 884 行 - 测试代码
```rust
// 修复前
manager.update_config(new_config, source.clone()).unwrap();
manager.update_config(new_config, source).unwrap();

// 修复后
manager.update_config(new_config, source.clone())
    .expect("Failed to update config");
manager.update_config(new_config, source)
    .expect("Failed to update config");
```

## 统计
- **已修复**: 11 处
- **剩余**: 859 处
- **预计总工作量**: 2-3 天

## 下一步
继续修复其他高优先级文件：
- src/core/version.rs (7 处)
- src/workflow/engine.rs (20 处)
- src/plugins/file_management/ (30 处)
