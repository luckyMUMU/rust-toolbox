# 用户指南 (User Guide)

## 1. 简介
Rust Toolbox (简称 `rt-box`) 是一个强大的工具流编排平台。

## 2. 核心概念
- **工具 (Tool)**: 执行单一任务的原子单元。
- **工作流 (Workflow)**: 串联执行的工具序列。

## 3. 工具库 (Tool Library)

### 3.1 文件操作 (File Operations)

#### 📂 移动文件夹 (`file.move_folder`)
移动或重命名指定的文件夹。

**行为说明 (Behavior):**
1. **重命名/移动**: 如果 `destination` 不存在，源文件夹将被重命名或移动到该路径。
2. **移动到内部**: 如果 `destination` 是一个已存在的目录，源文件夹将被移动到该目录**内部**。

**输入参数 (Input):**
```json
{
  "source": "path/to/source_folder",      // 源路径 (必填)
  "destination": "path/to/target_folder", // 目标路径 (必填)
  "overwrite": false                      // 是否覆盖 (可选, 默认 false)。
                                          // 如果为 true 且目标路径(计算后)已存在，将先删除目标再移动。
}
```

**输出 (Output):**
```json
{
  "success": true,
  "moved_files": 0 // 移动的文件/项目数量
}
```

## 4. 使用方式 (Usage)

### 4.1 命令行 (CLI) - `rt-cli`

#### 列出所有工具
```powershell
cargo run --bin rt-cli -- list
```

#### 运行工具
```powershell
cargo run --bin rt-cli -- run file.move_folder --input '{"source": "./tmp/a", "destination": "./tmp/b"}'
```
*注意：在 PowerShell 中输入 JSON 字符串时，建议使用单引号包裹，避免转义问题。*

### 4.2 图形界面 (GUI) - `rt-gui`

#### 启动界面
```powershell
cargo run --bin rt-gui
```

#### 界面操作
1. **左侧列表**: 点击选择要使用的工具（如 `file.move_folder`）。
2. **中间面板**: 
   - 在 "Input (JSON)" 文本框中输入参数。例如：
     ```json
     {
       "source": "D:/tmp/test_src",
       "destination": "D:/tmp/test_dst",
       "overwrite": true
     }
     ```
3. **运行**: 点击 "Run" 按钮。
4. **查看结果**: 底部面板将显示工具执行结果或错误信息。
