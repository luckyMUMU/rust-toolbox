# rt-tools Design Document

## 1. 模块概述 (Module Overview)
`rt-tools` 包含具体的工具实现。所有工具必须实现 `rt-core::Tool` trait。

## 2. 工具列表 (Tool List)

### 2.1 File Operations (`file`)

#### Move Folder (`file.move_folder`)
- **Name**: `file.move_folder`
- **Description**: 移动或重命名文件夹。
- **Input Schema**:
  ```json
  {
    "source": "path/to/source",
    "destination": "path/to/dest",
    "overwrite": false // Optional, default false
  }
  ```
- **Output Schema**:
  ```json
  {
    "success": true,
    "moved_files": 10 // Count of moved files/items
  }
  ```
- **Error Conditions**:
  - Source path does not exist.
  - Destination exists and overwrite is false.
  - Permission denied.

## 3. 结构 (Structure)
```
src/
├── lib.rs
├── file/
│   ├── mod.rs
│   └── move_folder.rs
└── text/ (Future)
```
