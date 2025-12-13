# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Persistence Module**: Introduced `rt-core::persistence` for unified data storage.
  - Implemented `PersistenceManager` with `sled` (DB) and `moka` (Cache).
  - Added support for data compression (`zstd`) and serialization (`bincode`).
  - Added configuration management via `confy`.
  - Added temporary file management via `tempfile`.
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

### Changed
- Updated `rt-core` dependencies to include `sled`, `moka`, `bincode`, `zstd`, `confy`.
- Updated `DESIGN.md` to reflect new architecture components.
- Updated `USER_GUIDE.md` with workflow execution instructions and new tool documentation.
- Updated `README.md` to include new tools and plugin information.

### Fixed
- Fixed CLI unused import warnings.
- Fixed input mapping resolution logic in workflow engine.
