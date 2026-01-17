# Python 到 Rust 迁移指南

> **将 Python 脚本迁移到 rust-tool-v2 的完整指南**  
> *最后更新日期：2026-01-14*

---

## 🎯 迁移概述

### 为什么要迁移？
| 指标 | Python | Rust | 提升 |
|--------|--------|------|-------------|
| **启动时间** | 2.1s | 0.1s | **快 21 倍** |
| **分类速度** | 12.3s | 2.5s | **快 5 倍** |
| **文件操作** | 8.7s | 2.1s | **快 4 倍** |
| **内存占用** | 180MB | 45MB | **节省 4 倍** |
| **二进制大小** | 2.5MB + Python | 8MB 单文件 | **自包含** |

### 迁移复杂度
- **简单脚本**：1-2 小时
- **中等复杂度**：4-8 小时
- **复杂系统**：1-3 天

---

## 📋 迁移前检查清单

### 1. 分析你的 Python 脚本
```bash
# 检查脚本大小和复杂度
wc -l your_script.py
grep -c "def " your_script.py
grep -c "import " your_script.py

# 检查依赖项
pip freeze | grep -f <(grep "import " your_script.py | awk '{print $2}' | sed 's/from //;s/;//')

# 性能分析
python -m cProfile -o profile.stats your_script.py
python -m pstats profile.stats
```

### 2. 确定所需功能
```bash
# 检查常见模式
grep -E "(threading|multiprocessing|asyncio)" your_script.py  # 并发
grep -E "(json|yaml|toml)" your_script.py                    # 序列化
grep -E "(re|glob|fnmatch)" your_script.py                   # 模式匹配
grep -E "(os\.|shutil|pathlib)" your_script.py               # 文件操作
grep -E "(socket|http|requests)" your_script.py              # 网络
```
