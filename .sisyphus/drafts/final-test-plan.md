# start-work 命令最终测试方案

## ✅ 诊断结果总结

### 配置文件状态
- ✅ 项目级插件配置: `.opencode/oh-my-opencode.json` - 正确
- ✅ 用户级 OpenCode 配置: `C:/Users/mumu/.config/opencode/opencode.json` - 正确
- ✅ Boulder 状态文件: `.sisyphus/boulder.json` - 正确
- ✅ 计划文件: `.sisyphus/plans/rust-toolkit-optimization.md` - 存在

### 插件状态
- ✅ oh-my-opencode 插件: 已安装
- ✅ start-work hook: 实现在主文件中
- ✅ OpenCode 版本: 1.1.35
- ✅ Rust 项目: 编译通过

### 配置验证
- ✅ `disabled_hooks`: 空数组（所有 hook 启用）
- ✅ `disabled_commands`: 空数组（所有命令启用）
- ✅ 代理配置: sisyphus, prometheus, atlas 都配置为 `xiaomi/mimo-v2-flash`

## 🎯 可能的问题根源

基于 OpenCode 的工作原理，`/start-work` 命令失败的最可能原因是：

### 1. 命令处理流程中断
```
用户输入 "/start-work"
    ↓
auto-slash-command hook 解析
    ↓
executeSlashCommand 查找命令
    ↓
findCommand2 查找 "start-work" 命令
    ↓
❌ 可能在这里失败：命令未找到或模板替换失败
```

### 2. 模板变量替换问题
`start-work` 命令模板需要替换：
- `$SESSION_ID` → 当前会话 ID
- `$TIMESTAMP` → 当前时间戳
- `$ARGUMENTS` → 用户参数（可能为空）

如果变量替换失败，模板可能无法生成 `<session-context>` 标签。

### 3. start-work hook 触发条件
```javascript
const isStartWorkCommand = promptText.includes("<session-context>");
```

如果 `<session-context>` 标签没有正确生成，hook 就不会触发。

## 🔧 解决方案

### 方案 1: 测试其他命令（推荐）

在 OpenCode 中依次尝试：

1. **测试帮助命令**:
```
/help
```

2. **测试插件列表**:
```
/plugins
```

3. **测试命令列表**:
```
/slashcommand
```

4. **测试 Prometheus**:
```
/plan "test"
```

5. **再次尝试 start-work**:
```
/start-work
```

### 方案 2: 手动创建命令文件

创建项目级命令文件：

```bash
mkdir -p .opencode/command
```

创建 `.opencode/command/start-work.md`:

```markdown
---
description: "Start Sisyphus work session from Prometheus plan"
argument-hint: "[plan-name]"
model: "xiaomi/mimo-v2-flash"
agent: "atlas"
---

<command-instruction>
You are starting a Sisyphus work session.

## WHAT TO DO

1. **Find available plans**: Search for Prometheus-generated plan files at `.sisyphus/plans/`

2. **Check for active boulder state**: Read `.sisyphus/boulder.json` if it exists

3. **Decision logic**:
   - If `.sisyphus/boulder.json` exists AND plan is NOT complete (has unchecked boxes):
     - **APPEND** current session to session_ids
     - Continue work on existing plan
   - If no active plan OR plan is complete:
     - List available plan files
     - If ONE plan: auto-select it
     - If MULTIPLE plans: show list with timestamps, ask user to select

4. **Create/Update boulder.json**:
   ```json
   {
     "active_plan": "/absolute/path/to/plan.md",
     "started_at": "ISO_TIMESTAMP",
     "session_ids": ["session_id_1", "session_id_2"],
     "plan_name": "plan-name"
   }
   ```

5. **Read the plan file** and start executing tasks according to Orchestrator Sisyphus workflow
</command-instruction>

<session-context>
Session ID: $SESSION_ID
Timestamp: $TIMESTAMP
</session-context>

<user-request>
$ARGUMENTS
</user-request>
```

### 方案 3: 检查 OpenCode 日志

在 OpenCode 中查看：
1. 控制台输出
2. 错误消息
3. 插件加载日志

### 方案 4: 重置并重启

1. **备份当前状态**:
```bash
cp .sisyphus/boulder.json .sisyphus/boulder.json.backup
```

2. **删除 boulder.json**:
```bash
rm .sisyphus/boulder.json
```

3. **重启 OpenCode**:
   - 完全关闭 OpenCode
   - 重新启动

4. **重新尝试**:
```
/start-work
```

## 📊 预期行为

### 成功的情况
```
/start-work
```

应该输出：
```
## Auto-Selected Plan

**Plan**: rust-toolkit-optimization
**Path**: .sisyphus/plans/rust-toolkit-optimization.md
**Progress**: 0/16 tasks
**Session ID**: [当前会话 ID]
**Started**: [当前时间]

boulder.json has been created. Read the plan and begin execution.
```

### 失败的情况
如果仍然失败，记录：
1. 完整的错误消息
2. OpenCode 控制台输出
3. 任何相关的日志信息

## 🎯 备选方案

如果 `/start-work` 始终失败：

### 方案 A: 使用 Prometheus 重新创建计划
```
/plan "优化 Rust 工具库代码，修复 unwrap 使用，拆分超大文件"
```

然后：
```
/start-work
```

### 方案 B: 手动执行计划
1. 直接阅读 `.sisyphus/plans/rust-toolkit-optimization.md`
2. 手动执行 TODO 任务
3. 使用 `/start-work` 当问题修复后

### 方案 C: 检查 OpenCode 社区
- 查看 OpenCode 文档: https://opencode.ai/docs
- 检查 GitHub issues
- 寻求社区帮助

## 📝 下一步行动

1. **立即执行**: 在 OpenCode 中尝试 `/help` 和 `/plan "test"`
2. **记录结果**: 记录命令输出和任何错误信息
3. **报告问题**: 如果仍然失败，提供完整的错误信息
4. **寻求帮助**: 考虑在 OpenCode 社区寻求支持

---

**创建时间**: 2026-01-25 18:35
**状态**: 待测试
