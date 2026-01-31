# 故障排除指南

> **rust-tool-v2常见问题综合解决方案**  
> *最后更新：2026-01-14*

---

## 🔍 快速故障排除

### 问题未列出？
1. **查看[主索引](../INDEX.md)** 了解相关主题
2. **搜索[项目自述](../../README.md)** 查找关键词
3. **参考[开发指南](../dev/DEVELOPMENT_GUIDE.md)** 进行调试
4. **运行 `cargo check`** 识别编译问题

---

## 🚨 按类别分类的常见问题

### 1. 安装与设置问题

#### Cargo构建失败
**症状：**
```
error: could not compile `rust-tool-v2`
error: failed to compile
```

**解决方案：**
```bash
# 1. 检查Rust版本
rustc --version  # 应为1.70+ (2021 Edition)

# 2. 更新Rust
rustup update

# 3. 清理并重新构建
cargo clean
cargo check
cargo build

# 4. 检查缺失的系统依赖
# Windows: 确保安装Visual Studio C++构建工具
# Linux: 确保安装build-essential, pkg-config
# macOS: 确保安装Xcode命令行工具
```

**如果仍然失败：**
```bash
# 详细构建以查看错误
cargo build --verbose

# 检查特定依赖
cargo tree | grep <problem-crate>

# 更新依赖
cargo update
```

#### 缺少依赖
**错误：** `package 'xxx' not found`

**解决方案：**
```bash
# 检查Cargo.toml中所需的功能
cargo build --all-features

# 安装特定功能
cargo build --features lancedb

# 检查功能标志
cargo metadata --format-version 1 | grep features
```

#### 编译警告（非错误）
**症状：** 构建过程中出现许多警告

**解决方案：**
```bash
# 自动修复（尽可能）
cargo fix --allow-dirty

# 然后运行clippy检查最佳实践
cargo clippy -- -D warnings

# 格式化代码
cargo fmt
```

---

### 2. 运行时问题

#### CLI命令不工作
**症状：** `cargo run -- --help` 可以工作但命令失败

**解决方案：**
```bash
# 1. 检查命令语法
cargo run -- workflow execute --help

# 2. 验证工作流文件存在且是有效的YAML
cargo run -- workflow execute workflows/basic/hello-world.yaml

# 3. 检查文件权限
ls -la workflows/basic/hello-world.yaml

# 4. 验证YAML结构
# 使用在线YAML验证器或：
python -c "import yaml; yaml.safe_load(open('workflows/basic/hello-world.yaml'))"
```

#### TUI显示不正确
**症状：** 显示混乱、缺少颜色、无响应

**解决方案：**
```bash
# 1. 检查终端兼容性
echo $TERM  # 应为xterm-256color或类似

# 2. 尝试基础TUI
cargo run -- tui --basic

# 3. 如需禁用颜色
cargo run -- tui --no-color

# 4. 检查终端模拟器
# 最佳支持：Windows Terminal, iTerm2, Alacritty, Kitty
# 可能有问题：cmd.exe, 基础xterm
```

#### 性能问题
**症状：** 执行缓慢、内存使用高

**解决方案：**
```bash
# 1. 检查系统资源
top  # Linux/macOS
任务管理器  # Windows

# 2. 使用性能分析构建运行
cargo run --release -- workflow execute examples/large-workflow.yaml

# 3. 监控特定操作
RUST_LOG=debug cargo run -- workflow execute workflows/basic/hello-world.yaml

# 4. 检查内存泄漏
# 使用cargo instruments (macOS) 或 valgrind (Linux)
```

---

### 3. 文件管理问题

#### 分类不工作
**症状：** 文件未正确分类

**解决方案：**
```bash
# 1. 检查文件模式
cargo run -- file-classifier --help

# 2. 验证源目录结构
tree /path/to/source

# 3. 检查目标目录权限
ls -ld /path/to/destination

# 4. 先用小批量测试
cargo run -- file-classifier --source ./test --dest ./output --dry-run
```

#### 文件合并问题
**症状：** 重复文件、数据丢失、合并冲突

**解决方案：**
```bash
# 1. 始终先备份
cp -r source/ source_backup/

# 2. 使用试运行模式
cargo run -- folder-merger --source ./folder1 --dest ./folder2 --dry-run

# 3. 检查重复项
cargo run -- file-classifier --source ./merged --check-duplicates

# 4. 执行前查看合并计划
# 工具会显示将要合并的内容
```

#### 人工决策模式不工作
**症状：** 交互式提示未出现

**解决方案：**
```bash
# 1. 确保stdin可用
# 后台进程或管道中不支持

# 2. 检查终端是交互式的
tty  # 应显示/dev/tty或类似

# 3. 尝试显式交互标志
cargo run -- human-decision --interactive

# 4. 如需使用批处理模式
cargo run -- batch-processor --auto
```

---

### 4. MCP服务器问题

#### 服务器无法启动
**症状：** MCP服务器连接被拒绝

**解决方案：**
```bash
# 1. 检查服务器是否运行
cargo run -- mcp-server --help

# 2. 验证端口可用性
netstat -an | grep 8080  # 或您配置的端口

# 3. 检查配置
cat ~/.config/rust-tool-v2/mcp.toml

# 4. 使用调试日志启动
RUST_LOG=debug cargo run -- mcp-server
```

#### MCP客户端连接问题
**症状：** 客户端无法连接服务器

**解决方案：**
```bash
# 1. 验证服务器正在运行
ps aux | grep mcp-server

# 2. 检查防火墙
# Windows: 检查Windows Defender防火墙
# Linux: 检查iptables/ufw
# macOS: 检查系统偏好设置 → 安全性

# 3. 手动测试连接
curl http://localhost:8080/health  # 或您配置的端口

# 4. 检查客户端配置
# 确保客户端指向正确的主机/端口
```

---

### 5. 性能与优化问题

#### 启动缓慢
**症状：** 命令启动需要>2秒

**解决方案：**
```bash
# 1. 使用release构建
cargo run --release -- <command>

# 2. 预编译并直接使用二进制文件
cargo build --release
./target/release/rust-tool-v2 <command>

# 3. 检查杀毒软件干扰
# 将构建目录添加到杀毒软件排除项

# 4. 使用cargo run并优化
cargo run --release -- <command>
```

#### 内存使用高
**症状：** 内存持续增长

**解决方案：**
```bash
# 1. 检查内存使用
/usr/bin/time -v cargo run -- <command>  # Linux
/usr/bin/time -l cargo run -- <command>  # macOS

# 2. 使用更小的批次处理
cargo run -- batch-processor --batch-size 100

# 3. 使用流式模式（如果可用）
cargo run -- workflow execute --streaming

# 4. 使用特定工具监控
# Linux: valgrind --tool=massif
# macOS: Instruments (内存分析器)
```

#### 文件操作缓慢
**症状：** 文件分类/移动缓慢

**解决方案：**
```bash
# 1. 检查磁盘I/O
iostat -x 1  # Linux
# 使用任务管理器 → 性能 → 磁盘 (Windows)

# 2. 如可能使用SSD

# 3. 并行批次处理
cargo run -- batch-processor --parallel 4

# 4. 检查杀毒软件扫描
# 将源/目标添加到杀毒软件排除项
```

---

### 6. 功能特定问题

#### LanceDB集成问题
**症状：** 数据库错误、连接失败

**解决方案：**
```bash
# 1. 确保功能已启用
cargo build --features lancedb

# 2. 检查LanceDB版本兼容性
cargo tree | grep lancedb

# 3. 验证数据库目录权限
ls -ld ~/.local/share/rust-tool-v2/lancedb

# 4. 如损坏重置数据库
rm -rf ~/.local/share/rust-tool-v2/lancedb
cargo run -- lancedb init
```

#### 插件加载问题
**症状：** 插件未找到或加载失败

**解决方案：**
```bash
# 1. 检查插件目录
ls -la ~/.config/rust-tool-v2/plugins/

# 2. 验证插件格式
# 应为.wasm文件（如果启用WASM）或原生库

# 3. 检查插件权限
chmod +x ~/.config/rust-tool-v2/plugins/*

# 4. 列出已加载插件
cargo run -- plugin list
```

#### 工作流执行问题
**症状：** 工作流失败或产生意外结果

**解决方案：**
```bash
# 1. 验证工作流YAML
cargo run -- workflow validate workflows/basic/hello-world.yaml

# 2. 检查工作流依赖
cargo run -- workflow graph workflows/basic/hello-world.yaml

# 3. 使用详细日志运行
RUST_LOG=debug cargo run -- workflow execute workflows/basic/hello-world.yaml

# 4. 先用简单工作流测试
cargo run -- workflow execute workflows/basic/hello-world.yaml
```

---

## 🔧 诊断命令

### 系统信息
```bash
# Rust版本
rustc --version

# 项目信息
cargo metadata --format-version 1 | jq '.packages[0].version'

# 启用的功能
cargo metadata --format-version 1 | jq '.packages[0].features'

# 依赖
cargo tree
```

### 构建诊断
```bash
# 检查编译
cargo check

# 带信息构建
cargo build --verbose

# 检查更新
cargo outdated

# 审计依赖
cargo audit
```

### 运行时诊断
```bash
# 启用所有日志
RUST_LOG=debug cargo run -- <command>

# 跟踪执行
RUST_LOG=trace cargo run -- <command> 2>&1 | tee debug.log

# 性能分析
cargo run --release -- <command> 2> perf.log
```

### 文件系统检查
```bash
# 检查磁盘空间
df -h

# 检查目录权限
ls -la /path/to/directory

# 检查文件类型
file /path/to/file

# 检查文件大小
du -sh /path/to/directory
```

---

## 🎯 错误消息与解决方案

### "No such file or directory"
**原因：** 文件路径不正确或文件不存在

**修复：**
```bash
# 验证文件存在
ls -la /path/to/file

# 如需使用绝对路径
cargo run -- workflow execute /absolute/path/to/workflow.yaml
```

### "Permission denied"
**原因：** 文件权限不足

**修复：**
```bash
# 检查权限
ls -la /path/to/directory

# 修复权限 (Linux/macOS)
chmod +rw /path/to/file
chmod +rx /path/to/directory

# 修复所有权
sudo chown -R $USER:$USER /path/to/directory
```

### "Address already in use"
**原因：** MCP服务器端口冲突

**修复：**
```bash
# 查找使用端口的进程
lsof -i :8080  # Linux/macOS
netstat -ano | findstr :8080  # Windows

# 终止进程或使用不同端口
cargo run -- mcp-server --port 8081
```

### "Connection refused"
**原因：** 服务器未运行或地址错误

**修复：**
```bash
# 检查服务器是否运行
ps aux | grep mcp-server

# 先启动服务器
cargo run -- mcp-server &

# 验证连接
curl http://localhost:8080/health
```

### "Out of memory"
**原因：** 大操作内存不足

**修复：**
```bash
# 使用更小批次处理
cargo run -- batch-processor --batch-size 50

# 使用流式模式
cargo run -- workflow execute --streaming

# 添加交换空间 (Linux)
sudo fallocate -l 2G /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### "YAML parse error"
**原因：** 工作流文件中YAML语法无效

**修复：**
```bash
# 验证YAML
python -c "import yaml; yaml.safe_load(open('file.yaml'))"

# 使用在线验证器
# https://yamlvalidator.com/

# 检查制表符vs空格
# YAML需要空格，不是制表符
```

---

## 📊 性能问题

### 分类缓慢
**症状：** 文件分类耗时过长

**解决方案：**
```bash
# 1. 使用优化构建
cargo run --release -- file-classifier --source ./large-folder

# 2. 并行处理
cargo run -- batch-processor --parallel 8 --batch-size 1000

# 3. 排除大目录
cargo run -- file-classifier --exclude "node_modules,target,.git"

# 4. 使用文件模式
cargo run -- file-classifier --pattern "*.txt,*.pdf,*.docx"
```

### CPU使用率高
**症状：** CPU长时间100%

**解决方案：**
```bash
# 1. 限制并行度
cargo run -- batch-processor --parallel 2

# 2. 检查无限循环
# 添加日志识别问题代码

# 3. 使用release模式
cargo run --release -- <command>

# 4. 使用系统工具监控
top  # Linux/macOS
任务管理器  # Windows
```

### 磁盘I/O瓶颈
**症状：** 文件操作缓慢，磁盘100%

**解决方案：**
```bash
# 1. 如可能使用SSD

# 2. 处理更小批次
cargo run -- batch-processor --batch-size 100

# 3. 检查磁盘健康
# Linux: smartctl -a /dev/sda
# Windows: 检查磁盘属性 → 工具 → 检查

# 4. 添加到杀毒软件排除项
# 将源/目标目录添加到杀毒软件白名单
```

---

## 🛠️ 高级调试

### 启用所有日志
```bash
# 设置环境变量
export RUST_LOG=debug

# 或内联
RUST_LOG=debug cargo run -- <command>

# 获取最大详情
RUST_LOG=trace cargo run -- <command> 2>&1 | tee debug.log
```

### 生成调试报告
```bash
# 创建诊断脚本
cat > diagnose.sh << 'EOF'
#!/bin/bash
echo "=== 系统信息 ===" > diagnostic_report.txt
echo "Rust: $(rustc --version)" >> diagnostic_report.txt
echo "Cargo: $(cargo --version)" >> diagnostic_report.txt
echo "OS: $(uname -a)" >> diagnostic_report.txt
echo "" >> diagnostic_report.txt

echo "=== 构建状态 ===" >> diagnostic_report.txt
cargo check >> diagnostic_report.txt 2>&1
echo "" >> diagnostic_report.txt

echo "=== 依赖 ===" >> diagnostic_report.txt
cargo tree >> diagnostic_report.txt 2>&1
echo "" >> diagnostic_report.txt

echo "=== 测试结果 ===" >> diagnostic_report.txt
cargo test -- --nocapture >> diagnostic_report.txt 2>&1

echo "报告已生成: diagnostic_report.txt"
EOF

chmod +x diagnose.sh
./diagnose.sh
```

### 分析特定操作
```bash
# 计时特定命令
time cargo run -- workflow execute workflows/basic/hello-world.yaml

# 使用instruments分析 (macOS)
cargo build --release
instruments -t "Time Profiler" ./target/release/rust-tool-v2 workflow execute workflows/basic/hello-world.yaml

# 使用perf分析 (Linux)
cargo build --release
perf record ./target/release/rust-tool-v2 workflow execute workflows/basic/hello-world.yaml
perf report
```

---

## 📞 获取更多帮助

### 何时咨询Oracle
- 2+次修复失败后的架构决策
- 复杂调试场景
- 性能优化需求
- 安全问题

### 何时搜索代码库
- 使用 `grep` 或 `ast-grep` 查找类似错误模式
- 检查 `src/error.rs` 了解错误类型
- 查看 `src/workflow/` 解决工作流执行问题
- 检查 `src/tools/` 解决工具特定问题

### 何时查看外部资源
- Rust文档: https://doc.rust-lang.org/
- Tokio文档: https://tokio.rs/
- Clap文档: https://clap.rs/
- Ratatui文档: https://ratatui.rs/

---

## ✅ 验证清单

修复任何问题后，使用以下命令验证：

```bash
# 1. 编译检查
cargo check

# 2. 构建成功
cargo build

# 3. 测试通过
cargo test

# 4. 基本功能正常
cargo run -- --help
cargo run -- workflow execute workflows/basic/hello-world.yaml

# 5. 无警告
cargo clippy -- -D warnings

# 6. 代码已格式化
cargo fmt -- --check
```

---

## 🎯 快速参考

### 基本命令
```bash
# 检查系统
rustc --version && cargo --version

# 构建和测试
cargo check && cargo test

# 带日志运行
RUST_LOG=info cargo run -- <command>

# Release模式
cargo run --release -- <command>

# 任何命令的帮助
cargo run -- <command> --help
```

### 常见修复
```bash
# 清理重建
cargo clean && cargo check && cargo build

# 更新依赖
cargo update

# 修复格式
cargo fmt

# 修复clippy警告
cargo clippy --fix --allow-dirty

# 运行所有检查
cargo fmt && cargo clippy -- -D warnings && cargo test
```

---

**← 返回 [INDEX.md](INDEX.md)** | **顶部** ↑
