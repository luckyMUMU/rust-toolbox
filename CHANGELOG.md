# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **MCP Support**: Added Model Context Protocol (MCP) support to the core architecture.
  - Added `McpTool` trait extension for MCP-enabled tools.
  - Updated `Tool` trait with MCP-related methods.
  - Added MCP API and MCP Server components to the architecture.
  - Updated documentation to include MCP support across all modules.
- **Persistence Module**: Introduced `rt-core::persistence` for unified data storage.
  - Implemented `PersistenceManager` with `sled` (DB) and `moka` (Cache).
  - Added support for data compression (`zstd`) and serialization (`bincode`).
  - Added configuration management via `confy`.
  - Added temporary file management via `tempfile`.
  - Added local file operations with safety mechanisms:
    - `create_file`: Create files with error handling and permission checks.
    - `read_file`: Read files with encoding detection.
    - `update_file`: Update files with dual-buffer safety mechanism.
    - `delete_file`: Delete files with existence checks.
- **Workflow Engine**: Added `rt-core::workflow` for DAG-based task orchestration.
  - Implemented `InMemoryWorkflowEngine` with Tokio async runtime.
  - Added `workflow run` command to `rt-cli`.
- **Text Operations**: Added `text.pinyin` tool for Chinese to Pinyin conversion.
  - Supports both toned and non-toned Pinyin output.
  - Handles mixed Chinese-English text.
- **Media Operations**: Added `media.ytdlp` plugin for YouTube and video downloading.
  - Supports single video and playlist downloads.
  - Supports multiple format options.
  - Supports subtitle downloads (including auto-generated subtitles).
  - Customizable output directory and filename.
- **Configuration Management Module**: Added `rt-core::config` for unified configuration management.
  - Implemented multi-source configuration loading (file, environment variables).
  - Added configuration caching and hot reload support.
  - Supports configuration item priority management.
  - Implemented type-safe configuration access.
- **Logging Module**: Added `rt-core::logger` for comprehensive logging functionality.
  - Implemented level-based logging (DEBUG, INFO, WARN, ERROR).
  - Supports multiple output targets (console, file).
  - Added structured logging support.
  - Implemented log rotation and memory log storage.
- **Service Layer**: Added `rt-core::service` for standardized service invocation.
  - Implemented ServiceManager for unified service management.
  - Added permission-based access control.
  - Supports tool and plugin unified invocation.
  - Implemented standardized error handling and response formatting.

### Changed
- Updated `rt-core` dependencies to include `sled`, `moka`, `bincode`, `zstd`, `confy`.
- Updated `DESIGN.md` to reflect new architecture components.
- Updated `USER_GUIDE.md` with workflow execution instructions and new tool documentation.
- Updated `README.md` to include new tools and plugin information.

### Fixed
- Fixed CLI unused import warnings.
- Fixed input mapping resolution logic in workflow engine.
