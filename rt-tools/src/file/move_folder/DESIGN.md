# Move Folder Tool Design Document

## Overview

The Move Folder tool (`file.move_folder`) provides safe and reliable file and directory moving functionality with collision handling and overwrite protection. It supports both renaming operations and moving items into existing directories.

## Features

### Core Functionality
- **Safe File/Directory Moving**: Atomic operations where possible
- **Collision Detection**: Identifies destination conflicts before execution
- **Overwrite Protection**: Configurable overwrite behavior with explicit confirmation
- **Cross-Platform Support**: Works on Windows, Linux, and macOS
- **Operation Statistics**: Reports number of files/items moved

### Supported Operations
- **File Moving**: Move individual files to new locations
- **Directory Moving**: Move entire directory trees
- **Renaming**: Rename files and directories in place
- **Into Directory**: Move items into existing directories

## Architecture

### Core Components

#### Path Validation
- **Source Validation**: Ensures source path exists and is accessible
- **Destination Analysis**: Determines if destination is a directory or new path
- **Permission Checking**: Validates read/write permissions

#### Conflict Resolution
- **Collision Detection**: Identifies when destination already exists
- **Overwrite Logic**: Handles overwrite confirmation and execution
- **Atomic Operations**: Uses filesystem rename when possible

#### Operation Execution
- **Rename Strategy**: Primary method using `std::fs::rename`
- **Copy-Delete Fallback**: For cross-filesystem moves (future enhancement)
- **Progress Tracking**: Counts moved files and directories

## Input Schema

```json
{
  "type": "object",
  "properties": {
    "source": {
      "type": "string",
      "title": "Source Path",
      "description": "Path to the file or directory to move"
    },
    "destination": {
      "type": "string", 
      "title": "Destination Path",
      "description": "Target path or directory for the move operation"
    },
    "overwrite": {
      "type": "boolean",
      "default": false,
      "title": "Allow Overwrite",
      "description": "Whether to overwrite existing files at destination"
    }
  },
  "required": ["source", "destination"]
}
```

### Field Descriptions
- **source**: Absolute or relative path to the item to move
- **destination**: Target location (can be directory or new path)
- **overwrite**: Boolean flag controlling overwrite behavior

## Output Schema

```json
{
  "type": "object",
  "properties": {
    "success": {
      "type": "boolean",
      "title": "Operation Success",
      "description": "Whether the move operation completed successfully"
    },
    "moved_files": {
      "type": "integer",
      "title": "Files Moved",
      "description": "Number of files and directories moved"
    }
  },
  "required": ["success", "moved_files"]
}
```

### Result Fields
- **success**: Boolean indicating operation success
- **moved_files**: Count of items successfully moved

## Operation Logic

### 1. Input Validation
- Verify source path exists and is readable
- Check destination path validity
- Validate user permissions for both paths

### 2. Path Resolution
- **Directory Destination**: If destination exists and is a directory, move source into it
- **New Path**: If destination doesn't exist, treat as rename operation
- **File Destination**: If destination exists and is a file, handle collision

### 3. Conflict Handling
- **No Overwrite**: Return error if destination exists and overwrite is false
- **With Overwrite**: Remove existing destination before moving
- **Safety Checks**: Prevent moving directory into itself

### 4. Execution
- **Primary Method**: Use `std::fs::rename` for atomic operation
- **Error Recovery**: Provide detailed error messages for failures
- **Statistics**: Count and report moved items

## Usage Examples

### Basic File Move
```json
{
  "source": "/home/user/document.txt",
  "destination": "/home/user/backup/document.txt"
}
```

### Move Into Directory
```json
{
  "source": "/home/user/project",
  "destination": "/home/user/archive/"
}
```

### Move With Overwrite
```json
{
  "source": "/tmp/data.csv",
  "destination": "/home/user/data.csv",
  "overwrite": true
}
```

## Error Handling

### Input Validation Errors
- **Source Not Found**: Source path does not exist
- **Permission Denied**: Insufficient permissions for source or destination
- **Invalid Path**: Malformed or invalid path strings
- **Self-Move**: Attempting to move directory into itself

### Runtime Errors
- **Destination Exists**: Target exists and overwrite is disabled
- **Filesystem Full**: Insufficient space for move operation
- **Cross-Device Move**: Moving across filesystem boundaries (not yet supported)
- **IO Errors**: Hardware or system-level failures

### Error Recovery
- **Atomic Operations**: Failed moves don't leave partial results
- **State Preservation**: Original files remain intact on failure
- **Detailed Messages**: Specific error descriptions for troubleshooting

## Performance Characteristics

### Time Complexity
- **Same Filesystem**: O(1) for rename operations
- **Cross Filesystem**: O(n) where n is total file size (future)
- **Directory Trees**: O(m) where m is number of items

### Memory Usage
- **Minimal Footprint**: Uses filesystem operations directly
- **No Buffering**: Doesn't load file contents into memory
- **Efficient Counting**: Tracks moved items without storing paths

## Internationalization

### Supported Locales
- **English (`en`)**: Primary development language
- **Chinese (`zh-CN`)**: Simplified Chinese translations

### Localized Elements
- **Field Titles**: Input/output field labels
- **Error Messages**: Localized error descriptions
- **User Guide**: Comprehensive usage instructions
- **Tool Description**: Purpose and capabilities

## Testing Strategy

### Unit Tests
- **Path Validation**: Test various path formats and edge cases
- **Collision Handling**: Verify overwrite logic works correctly
- **Error Conditions**: Test all error scenarios
- **Cross-Platform**: Ensure consistent behavior across operating systems

### Integration Tests
- **Tool Registration**: Verify proper integration with rt-core
- **Schema Validation**: Test input/output schema compliance
- **Filesystem Operations**: Test with real files and directories
- **Permission Handling**: Test various permission scenarios

## Future Enhancements

### Planned Features
- **Cross-Filesystem Support**: Copy-delete strategy for cross-device moves
- **Progress Reporting**: Real-time progress for large operations
- **Batch Operations**: Move multiple items in single operation
- **Symbolic Link Handling**: Proper handling of symlinks and hardlinks

### API Extensions
- **Preserve Metadata**: Option to preserve timestamps and permissions
- **Dry Run Mode**: Preview operations without executing
- **Conflict Resolution**: Interactive conflict resolution options
- **Backup Creation**: Automatic backup before overwrite

## Dependencies

### Core Dependencies
- **std::fs**: Standard filesystem operations
- **rt-core**: Core tool trait and error types
- **serde**: Serialization for input/output handling
- **schemars**: JSON schema generation

### Platform Dependencies
- **Windows**: Uses Windows API for optimal performance
- **Unix**: Uses POSIX filesystem operations
- **Cross-Platform**: Consistent behavior across platforms

## Compatibility

### Platform Support
- **Windows**: Full support with native paths
- **Linux**: Full support with Unix paths
- **macOS**: Full support with Unix paths

### Filesystem Support
- **NTFS**: Full support on Windows
- **ext4/XFS**: Full support on Linux
- **APFS/HFS+**: Full support on macOS
- **Network Drives**: Limited support (depends on network filesystem)
