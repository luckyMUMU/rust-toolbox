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

### Changed
- Updated `rt-core` dependencies to include `sled`, `moka`, `bincode`, `zstd`, `confy`.
- Updated `DESIGN.md` to reflect new architecture components.
- Updated `USER_GUIDE.md` with workflow execution instructions.

### Fixed
- Fixed CLI unused import warnings.
- Fixed input mapping resolution logic in workflow engine.
