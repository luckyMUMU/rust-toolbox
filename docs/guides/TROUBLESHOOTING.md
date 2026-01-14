# Troubleshooting Guide

> **Comprehensive guide to solving common issues with rust-tool-v2**  
> *Last Updated: 2026-01-14*

---

## 🔍 Quick Troubleshooting

### Issue Not Listed?
1. **Check [Main Index](../INDEX.md)** for related topics
2. **Search [Project Readme](../../README.md)** for keywords
3. **Review [Development Guide](../dev/DEVELOPMENT_GUIDE.md)** for debugging
4. **Run `cargo check`** to identify compilation issues

---

## 🚨 Common Issues by Category

### 1. Installation & Setup Issues

#### Cargo Build Fails
**Symptoms:**
```
error: could not compile `rust-tool-v2`
error: failed to compile
```

**Solutions:**
```bash
# 1. Check Rust version
rustc --version  # Should be 1.70+ (2021 Edition)

# 2. Update Rust
rustup update

# 3. Clean and rebuild
cargo clean
cargo check
cargo build

# 4. Check for missing system dependencies
# On Windows: Ensure Visual Studio C++ build tools
# On Linux: Ensure build-essential, pkg-config
# On macOS: Ensure Xcode command line tools
```

**If still failing:**
```bash
# Verbose build for detailed errors
cargo build --verbose

# Check specific dependency
cargo tree | grep <problem-crate>

# Update dependencies
cargo update
```

#### Missing Dependencies
**Error:** `package 'xxx' not found`

**Solution:**
```bash
# Check Cargo.toml for required features
cargo build --all-features

# Install specific feature
cargo build --features lancedb

# Check feature flags
cargo metadata --format-version 1 | grep features
```

#### Compilation Warnings (Not Errors)
**Symptoms:** Many warnings during build

**Solution:**
```bash
# Fix automatically where possible
cargo fix --allow-dirty

# Then run clippy for best practices
cargo clippy -- -D warnings

# Format code
cargo fmt
```

---

### 2. Runtime Issues

#### CLI Commands Not Working
**Symptoms:** `cargo run -- --help` works but commands fail

**Solutions:**
```bash
# 1. Check command syntax
cargo run -- workflow execute --help

# 2. Verify workflow file exists and is valid YAML
cargo run -- workflow execute examples/hello-world.yaml

# 3. Check file permissions
ls -la examples/hello-world.yaml

# 4. Validate YAML structure
# Use online YAML validator or:
python -c "import yaml; yaml.safe_load(open('examples/hello-world.yaml'))"
```

#### TUI Not Displaying Correctly
**Symptoms:** Garbled display, missing colors, unresponsive

**Solutions:**
```bash
# 1. Check terminal compatibility
echo $TERM  # Should be xterm-256color or similar

# 2. Try basic TUI
cargo run -- tui --basic

# 3. Disable colors if needed
cargo run -- tui --no-color

# 4. Check terminal emulator
# Works best with: Windows Terminal, iTerm2, Alacritty, Kitty
# May have issues with: cmd.exe, basic xterm
```

#### Performance Issues
**Symptoms:** Slow execution, high memory usage

**Solutions:**
```bash
# 1. Check system resources
top  # Linux/macOS
Task Manager  # Windows

# 2. Run with performance profile
cargo run --release -- workflow execute examples/large-workflow.yaml

# 3. Monitor specific operations
RUST_LOG=debug cargo run -- workflow execute examples/hello-world.yaml

# 4. Check for memory leaks
# Use cargo instruments (macOS) or valgrind (Linux)
```

---

### 3. File Management Issues

#### Classification Not Working
**Symptoms:** Files not categorized correctly

**Solutions:**
```bash
# 1. Check file patterns
cargo run -- file-classifier --help

# 2. Verify source directory structure
tree /path/to/source

# 3. Check destination directory permissions
ls -ld /path/to/destination

# 4. Test with small batch first
cargo run -- file-classifier --source ./test --dest ./output --dry-run
```

#### File Merging Issues
**Symptoms:** Duplicate files, data loss, merge conflicts

**Solutions:**
```bash
# 1. Always backup first
cp -r source/ source_backup/

# 2. Use dry-run mode
cargo run -- folder-merger --source ./folder1 --dest ./folder2 --dry-run

# 3. Check for duplicates
cargo run -- file-classifier --source ./merged --check-duplicates

# 4. Review merge plan before executing
# The tool shows what will be merged
```

#### Human Decision Mode Not Working
**Symptoms:** Interactive prompts not appearing

**Solutions:**
```bash
# 1. Ensure stdin is available
# Not supported in background processes or pipes

# 2. Check terminal is interactive
tty  # Should show /dev/tty or similar

# 3. Try with explicit interactive flag
cargo run -- human-decision --interactive

# 4. Use batch mode if needed
cargo run -- batch-processor --auto
```

---

### 4. MCP Server Issues

#### Server Won't Start
**Symptoms:** MCP server connection refused

**Solutions:**
```bash
# 1. Check if server is running
cargo run -- mcp-server --help

# 2. Verify port availability
netstat -an | grep 8080  # or your configured port

# 3. Check configuration
cat ~/.config/rust-tool-v2/mcp.toml

# 4. Start with debug logging
RUST_LOG=debug cargo run -- mcp-server
```

#### MCP Client Connection Issues
**Symptoms:** Client can't connect to server

**Solutions:**
```bash
# 1. Verify server is running
ps aux | grep mcp-server

# 2. Check firewall
# Windows: Check Windows Defender Firewall
# Linux: Check iptables/ufw
# macOS: Check System Preferences → Security

# 3. Test connection manually
curl http://localhost:8080/health  # or your configured port

# 4. Check client configuration
# Ensure client points to correct host/port
```

---

### 5. Performance & Optimization Issues

#### Slow Startup
**Symptoms:** Commands take >2s to start

**Solutions:**
```bash
# 1. Use release build
cargo run --release -- <command>

# 2. Pre-compile and use binary directly
cargo build --release
./target/release/rust-tool-v2 <command>

# 3. Check for antivirus interference
# Add build directory to antivirus exclusions

# 4. Use cargo run with optimizations
cargo run --release -- <command>
```

#### High Memory Usage
**Symptoms:** Memory grows continuously

**Solutions:**
```bash
# 1. Check memory usage
/usr/bin/time -v cargo run -- <command>  # Linux
/usr/bin/time -l cargo run -- <command>  # macOS

# 2. Process in smaller batches
cargo run -- batch-processor --batch-size 100

# 3. Use streaming mode if available
cargo run -- workflow execute --streaming

# 4. Monitor with specific tools
# Linux: valgrind --tool=massif
# macOS: Instruments (Memory Profiler)
# Windows: Visual Studio Diagnostic Tools
```

#### Slow File Operations
**Symptoms:** File classification/moving is slow

**Solutions:**
```bash
# 1. Check disk I/O
iostat -x 1  # Linux
# Use Task Manager → Performance → Disk (Windows)

# 2. Use SSD instead of HDD if possible

# 3. Process in parallel batches
cargo run -- batch-processor --parallel 4

# 4. Check for antivirus scanning
# Add source/destination to antivirus exclusions
```

---

### 6. Feature-Specific Issues

#### LanceDB Integration Issues
**Symptoms:** Database errors, connection failures

**Solutions:**
```bash
# 1. Ensure feature is enabled
cargo build --features lancedb

# 2. Check LanceDB version compatibility
cargo tree | grep lancedb

# 3. Verify database directory permissions
ls -ld ~/.local/share/rust-tool-v2/lancedb

# 4. Reset database if corrupted
rm -rf ~/.local/share/rust-tool-v2/lancedb
cargo run -- lancedb init
```

#### Plugin Loading Issues
**Symptoms:** Plugins not found or failing to load

**Solutions:**
```bash
# 1. Check plugin directory
ls -la ~/.config/rust-tool-v2/plugins/

# 2. Verify plugin format
# Should be .wasm files (if WASM enabled) or native libraries

# 3. Check plugin permissions
chmod +x ~/.config/rust-tool-v2/plugins/*

# 4. List loaded plugins
cargo run -- plugin list
```

#### Workflow Execution Issues
**Symptoms:** Workflows fail or produce unexpected results

**Solutions:**
```bash
# 1. Validate workflow YAML
cargo run -- workflow validate examples/hello-world.yaml

# 2. Check workflow dependencies
cargo run -- workflow graph examples/hello-world.yaml

# 3. Run with verbose logging
RUST_LOG=debug cargo run -- workflow execute examples/hello-world.yaml

# 4. Test with simple workflow first
cargo run -- workflow execute examples/hello-world.yaml
```

---

## 🔧 Diagnostic Commands

### System Information
```bash
# Rust version
rustc --version
cargo --version

# Project info
cargo metadata --format-version 1 | jq '.packages[0].version'

# Features enabled
cargo metadata --format-version 1 | jq '.packages[0].features'

# Dependencies
cargo tree
```

### Build Diagnostics
```bash
# Check compilation
cargo check

# Build with info
cargo build --verbose

# Check for updates
cargo outdated

# Audit dependencies
cargo audit
```

### Runtime Diagnostics
```bash
# Enable all logging
RUST_LOG=debug cargo run -- <command>

# Trace execution
RUST_LOG=trace cargo run -- <command> 2>&1 | tee debug.log

# Performance profiling
cargo run --release -- <command> 2> perf.log
```

### File System Checks
```bash
# Check disk space
df -h

# Check directory permissions
ls -la /path/to/directory

# Check file types
file /path/to/file

# Check file sizes
du -sh /path/to/directory
```

---

## 🎯 Error Messages & Solutions

### "No such file or directory"
**Cause:** File path incorrect or file doesn't exist

**Fix:**
```bash
# Verify file exists
ls -la /path/to/file

# Use absolute paths if needed
cargo run -- workflow execute /absolute/path/to/workflow.yaml
```

### "Permission denied"
**Cause:** Insufficient file permissions

**Fix:**
```bash
# Check permissions
ls -la /path/to/directory

# Fix permissions (Linux/macOS)
chmod +rw /path/to/file
chmod +rx /path/to/directory

# Fix ownership
sudo chown -R $USER:$USER /path/to/directory
```

### "Address already in use"
**Cause:** Port conflict for MCP server

**Fix:**
```bash
# Find process using port
lsof -i :8080  # Linux/macOS
netstat -ano | findstr :8080  # Windows

# Kill process or use different port
cargo run -- mcp-server --port 8081
```

### "Connection refused"
**Cause:** Server not running or wrong address

**Fix:**
```bash
# Check if server is running
ps aux | grep mcp-server

# Start server first
cargo run -- mcp-server &

# Verify connection
curl http://localhost:8080/health
```

### "Out of memory"
**Cause:** Insufficient RAM for large operations

**Fix:**
```bash
# Process in smaller batches
cargo run -- batch-processor --batch-size 50

# Use streaming mode
cargo run -- workflow execute --streaming

# Add swap space (Linux)
sudo fallocate -l 2G /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### "YAML parse error"
**Cause:** Invalid YAML syntax in workflow files

**Fix:**
```bash
# Validate YAML
python -c "import yaml; yaml.safe_load(open('file.yaml'))"

# Use online validator
# https://yamlvalidator.com/

# Check for tabs vs spaces
# YAML requires spaces, not tabs
```

---

## 📊 Performance Issues

### Slow Classification
**Symptoms:** File classification takes too long

**Solutions:**
```bash
# 1. Use optimized build
cargo run --release -- file-classifier --source ./large-folder

# 2. Process in parallel
cargo run -- batch-processor --parallel 8 --batch-size 1000

# 3. Exclude large directories
cargo run -- file-classifier --exclude "node_modules,target,.git"

# 4. Use file patterns
cargo run -- file-classifier --pattern "*.txt,*.pdf,*.docx"
```

### High CPU Usage
**Symptoms:** CPU at 100% for extended periods

**Solutions:**
```bash
# 1. Limit parallelism
cargo run -- batch-processor --parallel 2

# 2. Check for infinite loops
# Add logging to identify problematic code

# 3. Use release mode
cargo run --release -- <command>

# 4. Monitor with system tools
top  # Linux/macOS
Task Manager  # Windows
```

### Disk I/O Bottlenecks
**Symptoms:** Slow file operations, disk at 100%

**Solutions:**
```bash
# 1. Use SSD if possible

# 2. Process smaller batches
cargo run -- batch-processor --batch-size 100

# 3. Check disk health
# Linux: smartctl -a /dev/sda
# Windows: Check disk properties → Tools → Check

# 4. Add to antivirus exclusions
# Add source/destination directories to antivirus whitelist
```

---

## 🛠️ Advanced Debugging

### Enable All Logging
```bash
# Set environment variable
export RUST_LOG=debug

# Or inline
RUST_LOG=debug cargo run -- <command>

# For maximum detail
RUST_LOG=trace cargo run -- <command> 2>&1 | tee debug.log
```

### Generate Debug Report
```bash
# Create diagnostic script
cat > diagnose.sh << 'EOF'
#!/bin/bash
echo "=== System Info ===" > diagnostic_report.txt
echo "Rust: $(rustc --version)" >> diagnostic_report.txt
echo "Cargo: $(cargo --version)" >> diagnostic_report.txt
echo "OS: $(uname -a)" >> diagnostic_report.txt
echo "" >> diagnostic_report.txt

echo "=== Build Status ===" >> diagnostic_report.txt
cargo check >> diagnostic_report.txt 2>&1
echo "" >> diagnostic_report.txt

echo "=== Dependencies ===" >> diagnostic_report.txt
cargo tree >> diagnostic_report.txt 2>&1
echo "" >> diagnostic_report.txt

echo "=== Test Results ===" >> diagnostic_report.txt
cargo test -- --nocapture >> diagnostic_report.txt 2>&1

echo "Report generated: diagnostic_report.txt"
EOF

chmod +x diagnose.sh
./diagnose.sh
```

### Profile Specific Operations
```bash
# Time a specific command
time cargo run -- workflow execute examples/hello-world.yaml

# Profile with instruments (macOS)
cargo build --release
instruments -t "Time Profiler" ./target/release/rust-tool-v2 workflow execute examples/hello-world.yaml

# Profile with perf (Linux)
cargo build --release
perf record ./target/release/rust-tool-v2 workflow execute examples/hello-world.yaml
perf report
```

---

## 📞 Getting More Help

### When to Consult Oracle
- Architecture decisions after 2+ failed fixes
- Complex debugging scenarios
- Performance optimization needs
- Security concerns

### When to Search Codebase
- Use `grep` or `ast-grep` to find similar error patterns
- Check `src/error.rs` for error types
- Review `src/workflow/` for workflow execution issues
- Check `src/tools/` for tool-specific problems

### When to Check External Resources
- Rust documentation: https://doc.rust-lang.org/
- Tokio documentation: https://tokio.rs/
- Clap documentation: https://clap.rs/
- Ratatui documentation: https://ratatui.rs/

---

## ✅ Verification Checklist

After fixing any issue, verify with:

```bash
# 1. Compilation check
cargo check

# 2. Build succeeds
cargo build

# 3. Tests pass
cargo test

# 4. Basic functionality works
cargo run -- --help
cargo run -- workflow execute examples/hello-world.yaml

# 5. No warnings
cargo clippy -- -D warnings

# 6. Code formatted
cargo fmt -- --check
```

---

## 🎯 Quick Reference

### Essential Commands
```bash
# Check system
rustc --version && cargo --version

# Build and test
cargo check && cargo test

# Run with logging
RUST_LOG=info cargo run -- <command>

# Release mode
cargo run --release -- <command>

# Help for any command
cargo run -- <command> --help
```

### Common Fixes
```bash
# Clean rebuild
cargo clean && cargo check && cargo build

# Update dependencies
cargo update

# Fix formatting
cargo fmt

# Fix clippy warnings
cargo clippy --fix --allow-dirty

# Run all checks
cargo fmt && cargo clippy -- -D warnings && cargo test
```

---

**← Back to [INDEX.md](INDEX.md)** | **Top** ↑