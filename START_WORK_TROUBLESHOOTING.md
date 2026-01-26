# start-work 命令故障排除指南

## 🔍 问题描述
`/start-work` 命令执行失败，显示"发送命令失败"。

## ✅ 已验证的配置

### 配置文件状态
- ✅ `.opencode/oh-my-opencode.json` - 正确配置
- ✅ `C:/Users/mumu/.config/opencode/opencode.json` - 正确配置
- ✅ `.sisyphus/boulder.json` - 格式正确
- ✅ `.sisyphus/plans/rust-toolkit-optimization.md` - 计划文件存在

### 插件状态
- ✅ oh-my-opencode 插件已安装
- ✅ start-work hook 实现在主文件中
- ✅ OpenCode 版本: 1.1.35
- ✅ Rust 项目编译通过

### 配置验证
- ✅ `disabled_hooks`: 空数组（所有 hook 启用）
- ✅ `disabled_commands`: 空数组（所有命令启用）
- ✅ 代理配置正确

## 🎯 可能的原因

### 1. OpenCode 未重新加载插件
**症状**: 配置文件已修改，但 OpenCode 仍在使用旧配置

**解决方案**: 
- 完全关闭 OpenCode
- 重新启动 OpenCode
- 确保插件正确加载

### 2. 命令处理流程问题
**症状**: `/start-work` 命令被解析，但后续处理失败

**诊断步骤**:
```bash
# 在 OpenCode 中尝试以下命令：
/help           # 查看帮助
/plugins        # 查看插件列表
/slashcommand   # 查看命令列表
/plan "test"    # 测试 Prometheus
```

### 3. 模板变量替换失败
**症状**: 命令模板中的 `$SESSION_ID`, `$TIMESTAMP`, `$ARGUMENTS` 替换失败

**可能原因**:
- OpenCode 版本兼容性问题
- 模板语法错误
- 变量未定义

## 🔧 故障排除步骤

### 步骤 1: 基础测试
在 OpenCode 中依次尝试：
```
/help
```
如果成功 → 继续下一步
如果失败 → OpenCode 本身有问题

### 步骤 2: 插件测试
```
/plugins
```
检查 oh-my-opencode 是否在列表中

### 步骤 3: 命令测试
```
/slashcommand
```
检查 start-work 是否在命令列表中

### 步骤 4: Prometheus 测试
```
/plan "test"
```
如果成功 → Prometheus 正常工作
如果失败 → Prometheus 有问题

### 步骤 5: 再次尝试 start-work
```
/start-work
```

## 📊 诊断信息收集

如果仍然失败，请收集以下信息：

### 1. 错误消息
完整复制 OpenCode 显示的错误消息

### 2. 控制台输出
检查 OpenCode 控制台是否有错误信息

### 3. 配置验证
运行诊断脚本：
```bash
./diagnose-start-work.sh
```

### 4. 版本信息
```bash
opencode --version
```

## 🎯 备选方案

### 方案 A: 使用 Prometheus 重新创建计划
```
/plan "优化 Rust 工具库代码，修复 unwrap 使用，拆分超大文件"
```

然后：
```
/start-work
```

### 方案 B: 手动执行计划
1. 阅读 `.sisyphus/plans/rust-toolkit-optimization.md`
2. 手动执行 TODO 任务
3. 当问题修复后，再使用 `/start-work`

### 方案 C: 创建自定义命令
创建 `.opencode/command/start-work.md` 文件（见详细调试文档）

## 📞 寻求帮助

如果以上步骤都无法解决问题：

1. **检查 OpenCode 文档**: https://opencode.ai/docs
2. **查看 GitHub Issues**: 搜索类似问题
3. **社区支持**: 在 OpenCode 社区寻求帮助
4. **提供完整信息**: 包括配置文件、错误消息、诊断结果

## 📝 快速参考

### 关键文件位置
- 项目配置: `.opencode/oh-my-opencode.json`
- 用户配置: `C:/Users/mumu/.config/opencode/opencode.json`
- Boulder 状态: `.sisyphus/boulder.json`
- 计划文件: `.sisyphus/plans/rust-toolkit-optimization.md`

### 诊断命令
```bash
# 运行诊断脚本
./diagnose-start-work.sh

# 检查配置
cat .opencode/oh-my-opencode.json
cat C:/Users/mumu/.config/opencode/opencode.json
cat .sisyphus/boulder.json
```

### OpenCode 中的测试命令
```
/help
/plugins
/slashcommand
/plan "test"
/start-work
```

---

**最后更新**: 2026-01-25 18:40
**状态**: 配置已修复，待测试
