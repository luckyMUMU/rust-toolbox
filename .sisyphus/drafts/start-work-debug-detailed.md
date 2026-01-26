# start-work 命令失败详细调试

## 🔍 命令处理流程

### 1. 用户输入 `/start-work`
```
用户输入 → auto-slash-command hook 解析 → executeSlashCommand → findCommand2
```

### 2. 命令查找逻辑
```javascript
// 查找顺序：
// 1. 内置命令 (BUILTIN_COMMAND_DEFINITIONS)
// 2. 项目命令 (.opencode/command/)
// 3. 全局命令 (~/.config/opencode/command/)
// 4. 用户命令 (~/.claude/commands/)
// 5. 技能命令
```

### 3. start-work 命令定义
```javascript
"start-work": {
  description: "(builtin) Start Sisyphus work session from Prometheus plan",
  agent: "atlas",
  template: `<command-instruction>
${START_WORK_TEMPLATE}
</command-instruction>

<session-context>
Session ID: $SESSION_ID
Timestamp: $TIMESTAMP
</session-context>

<user-request>
$ARGUMENTS
</user-request>`,
  argumentHint: "[plan-name]"
}
```

### 4. start-work hook 触发条件
```javascript
// start-work hook 的触发条件
const isStartWorkCommand = promptText.includes("<session-context>");

// 这意味着：
// 1. auto-slash-command 先处理 `/start-work`
// 2. 替换为包含 <session-context> 的模板
// 3. start-work hook 检测到 <session-context> 标签
// 4. 执行实际的逻辑
```

## 🎯 问题分析

### 可能的问题点

1. **auto-slash-command hook 未启用**
   - 检查 `disabled_hooks` 是否包含 `auto-slash-command`

2. **命令查找失败**
   - `findCommand2` 找不到 `start-work` 命令
   - 可能是内置命令未正确加载

3. **模板替换失败**
   - `executeSlashCommand` 执行失败
   - 变量替换（`$SESSION_ID`, `$TIMESTAMP`, `$ARGUMENTS`）失败

4. **hook 触发条件不匹配**
   - `start-work` hook 需要 `<session-context>` 标签
   - 但模板可能没有正确生成这个标签

## 🔧 详细修复方案

### 方案 1: 确保所有必要的 hook 启用

更新 `.opencode/oh-my-opencode.json`：

```json
{
  "disabled_hooks": [],
  "disabled_commands": [],
  "agents": {
    "sisyphus": {
      "model": "xiaomi/mimo-v2-flash",
      "variant": "default",
      "category": "unspecified-high",
      "skills": ["git-master"]
    },
    "prometheus": {
      "model": "xiaomi/mimo-v2-flash",
      "variant": "default",
      "category": "unspecified-high",
      "skills": ["git-master"]
    },
    "atlas": {
      "model": "xiaomi/mimo-v2-flash",
      "variant": "default",
      "category": "unspecified-high",
      "skills": ["git-master"]
    }
  }
}
```

### 方案 2: 检查 OpenCode 版本兼容性

```bash
# 检查 OpenCode 版本
opencode --version

# 检查 oh-my-opencode 插件版本
cat C:/Users/mumu/.config/opencode/node_modules/oh-my-opencode/package.json | grep version
```

### 方案 3: 手动触发命令

如果自动命令处理失败，尝试手动创建命令：

1. **创建命令文件**:
```bash
mkdir -p .opencode/command
```

2. **创建 start-work.md**:
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

### 方案 4: 检查 OpenCode 配置目录

```bash
# 检查配置目录结构
ls -la C:/Users/mumu/.config/opencode/

# 检查是否有 command 目录
ls -la C:/Users/mumu/.config/opencode/command/

# 检查项目级配置
ls -la .opencode/
```

## 🧪 测试步骤

### 步骤 1: 验证配置
```bash
# 检查所有配置文件
cat .opencode/oh-my-opencode.json
cat C:/Users/mumu/.config/opencode/opencode.json
cat .sisyphus/boulder.json
```

### 步骤 2: 重启 OpenCode
1. 完全关闭 OpenCode
2. 重新启动 OpenCode
3. 确保插件正确加载

### 步骤 3: 测试命令
在 OpenCode 中尝试：
```
/start-work
```

### 步骤 4: 检查错误信息
如果仍然失败，记录：
1. 完整的错误消息
2. OpenCode 控制台输出
3. 任何相关的日志信息

## 📊 诊断命令

在 OpenCode 中尝试以下命令来诊断：

```
/help
```

```
/plugins
```

```
/slashcommand
```

```
/plan "test"
```

## 🎯 备选方案

如果 `/start-work` 始终失败，可以：

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
- 查看 OpenCode 文档
- 检查 GitHub issues
- 寻求社区帮助

---

**调试时间**: 2026-01-25 18:30
**下一步**: 执行诊断命令并记录结果
