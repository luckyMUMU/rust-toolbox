# start-work 命令调试分析

## 🔍 当前状态检查

### 1. 配置文件验证
```bash
# 检查 boulder.json
cat .sisyphus/boulder.json

# 检查插件配置
cat .opencode/oh-my-opencode.json

# 检查计划文件
ls -la .sisyphus/plans/
```

### 2. OpenCode 配置检查
```bash
# 检查用户级配置
cat C:/Users/mumu/.config/opencode/opencode.json

# 检查插件目录
ls -la C:/Users/mumu/.config/opencode/node_modules/oh-my-opencode/dist/hooks/start-work/
```

### 3. 可能的问题点
1. OpenCode 没有重新加载插件配置
2. start-work hook 的触发条件不匹配
3. OpenCode 的命令解析机制问题
4. 缺少必要的环境变量或上下文

## 🔧 调试步骤

### 步骤 1: 检查 OpenCode 是否识别命令
尝试在 OpenCode 中输入：
```
/help
```
查看是否显示可用命令列表。

### 步骤 2: 检查插件状态
尝试在 OpenCode 中输入：
```
/plugins
```
查看 oh-my-opencode 插件是否加载。

### 步骤 3: 尝试其他命令
尝试在 OpenCode 中输入：
```
/plan "test"
```
查看 Prometheus 是否正常工作。

### 步骤 4: 检查会话上下文
尝试在 OpenCode 中输入：
```
/session-info
```
查看当前会话信息。

## 🎯 可能的解决方案

### 方案 1: 清理并重启
1. 删除 `boulder.json`
2. 重启 OpenCode
3. 重新运行 `/start-work`

### 方案 2: 手动触发
1. 先运行 `/plan "优化代码"` 创建新计划
2. 再运行 `/start-work` 执行

### 方案 3: 检查 OpenCode 版本
1. 确认 OpenCode 版本支持 start-work 命令
2. 检查 oh-my-opencode 插件版本

## 📝 记录错误信息

请在 OpenCode 中尝试 `/start-work` 并记录：
1. 完整的错误消息
2. 错误代码（如果有）
3. 任何相关的日志输出

---

**调试时间**: 2026-01-25 18:20
