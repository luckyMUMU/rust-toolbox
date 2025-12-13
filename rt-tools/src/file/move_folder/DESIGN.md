# Design: Move Folder (file.move_folder)

## 1. 模块概述 (Module Overview)
`file.move_folder` 提供文件或文件夹的移动功能，支持重命名和覆盖保护。

## 2. 核心职责 (Core Responsibilities)
- 安全地移动文件系统对象。
- 处理目标路径冲突（覆盖与否）。
- 提供操作结果统计（如移动了多少个文件）。

## 3. 详细设计 (Detailed Design)

### 3.1 接口定义 (Tool Trait)
- **Name**: `file.move_folder`
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "source": { "type": "string", "title": "源路径" },
      "destination": { "type": "string", "title": "目标路径" },
      "overwrite": { "type": "boolean", "default": false, "title": "覆盖" }
    },
    "required": ["source", "destination"]
  }
  ```
- **Output Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "success": { "type": "boolean", "title": "成功" },
      "moved_files": { "type": "integer", "title": "移动文件数" }
    }
  }
  ```

### 3.2 逻辑流程
1. **Validation**:
   - 检查 `source` 是否存在且为目录/文件。
   - 检查 `destination` 是否已存在。
2. **Path Resolution**:
   - 如果 `destination` 是已存在的目录，则目标路径为 `destination/source_name`。
   - 否则，目标路径为 `destination`（视为重命名移动）。
3. **Conflict Handling**:
   - 如果目标路径已存在：
     - `overwrite=false`: 返回错误。
     - `overwrite=true`: 删除目标路径及其内容。
4. **Execution**:
   - 调用 `std::fs::rename` (原子操作) 或 `copy` + `remove` (跨分区移动)。
   - 这里的实现目前主要依赖 `std::fs::rename`，如果是跨挂载点移动可能会失败，未来需优化为 Copy-Delete 策略。

### 3.3 错误处理
- **InvalidInput**: 源不存在、目标已存在且不可覆盖。
- **ToolFailure**: IO 错误（权限不足、磁盘满等）。

## 4. 依赖 (Dependencies)
- `std::fs`: 标准文件系统操作。

## 5. 插件开发规范参考

有关插件开发的通用规范和多语言支持细节，请参阅 [PLUGIN_GUIDE.md](../../../../../PLUGIN_GUIDE.md)。
