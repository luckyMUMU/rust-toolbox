# 故障排除指南

> **解决 rust-tool-v2 常见问题的全面指南**  
> *最后更新日期：2026-01-14*

---

## 🔍 快速故障排除

### 问题未列出？
1. **检查 [主索引](../INDEX.md)** 以获取相关主题
2. **在 [项目 Readme](../../README.md)** 中搜索关键字
3. **查阅 [开发指南](../dev/DEVELOPMENT_GUIDE.md)** 以了解调试方法
4. **运行 `cargo check`** 以识别编译问题

---

## 🚨 各类别常见问题

### 1. 安装与设置问题

#### Cargo 构建失败
**症状：**
```
error: could not compile `rust-tool-v2`
error: failed to compile
```

**解决方案：**
```bash
# 1. 检查 Rust 版本
rustc --version  # 应为 1.70+ (2021 Edition)

# 2. 更新 Rust
rustup update

# 3. 清理并重新构建
cargo clean
cargo check
cargo build

# 4. 检查缺失的系统依赖项
# Windows 上：确保安装了 Visual Studio C++ 生成工具
# Linux 上：确保安装了 build-essential, pkg-config
# macOS 上：确保安装了 Xcode 命令行工具
```

**如果仍然失败：**
```bash
# 使用详细构建模式查看详细错误
cargo build -vv
```
