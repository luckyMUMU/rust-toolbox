# Czkawka Multi-Tool Plugin

## Overview

The Czkawka Multi-Tool Plugin (`rt-plugin-czkawka`) is a comprehensive file system utility plugin for Rust Toolbox that integrates multiple Czkawka tools into a single plugin. It provides powerful file system analysis and cleanup capabilities through five specialized tools for finding duplicate files, similar images, empty directories, temporary files, and broken symbolic links.

## Features

### Multi-Tool Architecture
- **Single Plugin, Multiple Tools**: Provides 5 distinct tools in one plugin
- **Unified Interface**: Consistent input/output patterns across all tools
- **Efficient Resource Usage**: Shared infrastructure and dependencies
- **Comprehensive Coverage**: Complete file system analysis suite

### Available Tools

#### 1. Duplicate Files (`file.duplicates`)
Find duplicate files based on content hash comparison
- **Hash-based Detection**: Uses cryptographic hashes for accurate comparison
- **Multiple Hash Algorithms**: Support for different hash methods
- **Size Filtering**: Filter by minimum file size
- **Performance Optimized**: Efficient scanning of large directory trees

#### 2. Similar Images (`file.similar_images`)
Detect visually similar images using perceptual hashing
- **Perceptual Comparison**: Finds images that look similar to human eyes
- **Configurable Threshold**: Adjustable similarity sensitivity
- **Multiple Formats**: Supports common image formats (JPEG, PNG, GIF, etc.)
- **Rotation/Scale Invariant**: Detects similar images regardless of transformations

#### 3. Empty Directories (`file.empty_directories`)
Locate empty directories in the file system
- **Recursive Scanning**: Deep directory tree analysis
- **True Empty Detection**: Identifies directories with no files or subdirectories
- **Hidden File Awareness**: Considers hidden files in emptiness determination
- **Safe Cleanup**: Provides information for safe directory removal

#### 4. Temporary Files (`file.temporary_files`)
Find temporary and cache files that can be safely removed
- **Pattern Recognition**: Identifies common temporary file patterns
- **Extension-based Detection**: Recognizes temporary file extensions
- **Age-based Filtering**: Filter by file modification time
- **System-aware**: Understands OS-specific temporary file locations

#### 5. Broken Symbolic Links (`file.broken_symlinks`)
Detect symbolic links that point to non-existent targets
- **Link Validation**: Checks if symbolic link targets exist
- **Cross-Platform Support**: Works on Unix-like systems with symlink support
- **Recursive Analysis**: Scans directory trees for broken links
- **Safe Identification**: Provides information for link cleanup or repair

## Installation

### Prerequisites
- **Rust Toolbox**: Requires rt-cli or rt-gui host application
- **Czkawka**: The plugin uses Czkawka's core functionality
- **Platform Support**: Windows, Linux, and macOS

### Build Instructions

1. **Clone Repository**:
   ```bash
   git clone https://github.com/your-repo/rust-tool.git
   cd rust-tool
   ```

2. **Build Plugin**:
   ```bash
   cargo build --release --package rt-plugin-czkawka
   ```

3. **Deploy Plugin**:
   ```bash
   # Copy to plugins directory
   cp target/release/rt-plugin-czkawka.exe plugins/
   # Or on Unix systems:
   cp target/release/rt-plugin-czkawka plugins/
   ```

### Verification
```bash
# List available tools (should show all 5 Czkawka tools)
rt-cli list

# Get plugin specification
rt-cli plugin spec --name file.duplicates
```

## Usage

### Command Line Interface (CLI)

#### Duplicate Files
```bash
# Find duplicates in specific directories
echo '{
  "directories": ["/home/user/Documents", "/home/user/Downloads"],
  "min_size": 1024,
  "hash_type": "blake3"
}' | rt-cli run file.duplicates
```

#### Similar Images
```bash
# Find similar images with custom threshold
echo '{
  "directories": ["/home/user/Pictures"],
  "threshold": 10,
  "hash_size": 8
}' | rt-cli run file.similar_images
```

#### Empty Directories
```bash
# Find empty directories
echo '{
  "directories": ["/home/user/Projects"]
}' | rt-cli run file.empty_directories
```

#### Temporary Files
```bash
# Find temporary files older than 7 days
echo '{
  "directories": ["/tmp", "/home/user/.cache"]
}' | rt-cli run file.temporary_files
```

#### Broken Symbolic Links
```bash
# Find broken symlinks
echo '{
  "directories": ["/home/user", "/opt"]
}' | rt-cli run file.broken_symlinks
```

### Graphical Interface (GUI)

1. Launch `rt-gui`
2. Navigate to the File Operations section
3. Select the desired Czkawka tool
4. Configure parameters using the form interface
5. Click "Run" to execute the analysis
6. Review results in the output panel

## Input/Output Schemas

### Common Input Fields
All tools share these common input parameters:

```json
{
  "directories": ["/path/to/scan"],
  "excluded_directories": [],
  "excluded_items": [],
  "recursive": true,
  "follow_symlinks": false
}
```

### Tool-Specific Parameters

#### Duplicate Files
```json
{
  "min_size": 1024,
  "hash_type": "blake3",
  "check_method": "hash"
}
```

#### Similar Images
```json
{
  "threshold": 5,
  "hash_size": 8,
  "image_filter": "all"
}
```

### Common Output Format
All tools return results in a consistent format:

```json
{
  "success": true,
  "tool_name": "file.duplicates",
  "results": [
    {
      "path": "/path/to/file",
      "size": 1024,               // Size in bytes (if applicable)
      "modified": "2023-12-01T10:00:00Z", // Modification time
      "additional_info": {}       // Tool-specific additional data
    }
  ],
  "summary": {                    // Summary statistics
    "total_found": 42,            // Number of items found
    "total_size": 1048576,        // Total size of found items
    "scan_time_ms": 1500          // Scan duration in milliseconds
  }
}
```

## Configuration

### Performance Tuning
- **Thread Count**: Adjust based on CPU cores and I/O capacity
- **Memory Usage**: Configure based on available system memory
- **Scan Depth**: Limit recursion depth for very deep directory trees
- **Batch Size**: Optimize file processing batch sizes

### Filtering Options
- **File Size Limits**: Set minimum/maximum file sizes
- **Date Ranges**: Filter by file modification dates
- **File Types**: Include/exclude specific file extensions
- **Path Patterns**: Use regex patterns for path filtering

## Internationalization

### Supported Languages
- **English (`en`)**: Primary development language
- **Chinese (`zh-CN`)**: Simplified Chinese translations

### Localized Elements
- **Tool Names**: Localized display names for each tool
- **Descriptions**: Tool purpose and functionality descriptions
- **Field Labels**: Input/output field titles and descriptions
- **User Guides**: Comprehensive usage instructions
- **Error Messages**: Localized error descriptions

### Localization Files
```
rt-plugin-czkawka/
└── locales/
    ├── tool.en.json     # English translations
    └── tool.zh.json     # Chinese translations
```

## Performance Characteristics

### Scanning Performance
- **Duplicate Files**: O(n log n) where n is number of files
- **Similar Images**: O(n²) for image comparison, optimized with hashing
- **Empty Directories**: O(d) where d is number of directories
- **Temporary Files**: O(n) linear scan with pattern matching
- **Broken Symlinks**: O(l) where l is number of symbolic links

### Memory Usage
- **Efficient Processing**: Streaming analysis for large file sets
- **Configurable Limits**: Adjustable memory usage based on system capacity
- **Garbage Collection**: Automatic cleanup of temporary data structures

### I/O Optimization
- **Parallel Scanning**: Multi-threaded directory traversal
- **Efficient Hashing**: Optimized hash computation for file comparison
- **Minimal Disk Access**: Smart caching and batching strategies

## Error Handling

### Common Error Conditions
- **Permission Denied**: Insufficient access to scan directories
- **Path Not Found**: Specified directories don't exist
- **Disk Space**: Insufficient space for temporary files
- **Memory Limits**: Out of memory for large file sets

### Error Recovery
- **Graceful Degradation**: Continue scanning accessible directories
- **Partial Results**: Return results for successfully scanned areas
- **Detailed Logging**: Comprehensive error reporting and logging
- **User Guidance**: Clear error messages with suggested solutions

## Testing

### Unit Tests
```bash
# Run plugin-specific tests
cargo test --package rt-plugin-czkawka
```

### Integration Tests
```bash
# Test with sample data
cargo test --package rt-plugin-czkawka --test integration
```

### Performance Tests
```bash
# Benchmark with large datasets
cargo bench --package rt-plugin-czkawka
```

## Development

### Project Structure
```
rt-plugin-czkawka/
├── Cargo.toml              # Dependencies and metadata
├── README.md               # This documentation
├── src/
│   ├── main.rs            # Plugin entry point and CLI handling
│   ├── i18n.rs            # Internationalization support
│   └── tools/             # Individual tool implementations
│       ├── mod.rs         # Tool module exports
│       ├── czkawka_adapter.rs  # Czkawka integration layer
│       ├── duplicates.rs  # Duplicate files tool
│       ├── similar_images.rs   # Similar images tool
│       ├── empty_dirs.rs  # Empty directories tool
│       ├── temp_files.rs  # Temporary files tool
│       ├── broken_symlinks.rs  # Broken symlinks tool
│       └── tests.rs       # Tool unit tests
└── locales/               # Internationalization resources
    ├── tool.en.json       # English translations
    └── tool.zh.json       # Chinese translations
```

### Adding New Tools
1. **Create Tool Module**: Add new tool implementation in `src/tools/`
2. **Update Metadata**: Add tool metadata to `ALL_TOOLS` constant
3. **Add Schema Functions**: Implement input/output schema functions
4. **Update CLI Handler**: Add tool execution case in main.rs
5. **Add Localization**: Update localization files with new tool strings
6. **Write Tests**: Add comprehensive unit and integration tests

### Contributing Guidelines
- **Code Style**: Follow Rust standard formatting with `cargo fmt`
- **Documentation**: Maintain comprehensive rustdoc comments
- **Testing**: Ensure all new features have corresponding tests
- **Localization**: Update all supported locales for new features
- **Performance**: Profile and optimize performance-critical code paths

## Troubleshooting

### Common Issues

#### Plugin Not Found
- **Cause**: Plugin not in correct directory or not executable
- **Solution**: Verify plugin is in `plugins/` directory with execute permissions

#### Permission Errors
- **Cause**: Insufficient permissions to scan directories
- **Solution**: Run with appropriate permissions or exclude restricted directories

#### Memory Issues
- **Cause**: Large file sets exceeding available memory
- **Solution**: Reduce scan scope or increase system memory

#### Slow Performance
- **Cause**: Scanning very large directory trees or slow storage
- **Solution**: Use exclusion patterns, limit recursion depth, or scan smaller areas

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug rt-cli run file.duplicates < input.json
```

### Performance Profiling
```bash
# Profile plugin execution
cargo build --release --package rt-plugin-czkawka
perf record target/release/rt-plugin-czkawka run < input.json
```

## License

This plugin is licensed under the same terms as the Rust Toolbox project. See the main project LICENSE file for details.

## Contributing

Contributions are welcome! Please see the main project's contributing guidelines and ensure all changes include:

- Comprehensive tests
- Updated documentation
- Localization updates
- Performance considerations

## Support

For issues, questions, or contributions:

1. **GitHub Issues**: Report bugs and request features
2. **Documentation**: Check the main project documentation
3. **Community**: Join project discussions and forums
4. **Email**: Contact maintainers for urgent issues

## Changelog

### v0.1.0 (Current)
- Initial multi-tool plugin implementation
- Support for 5 Czkawka tools
- Comprehensive internationalization
- Full CLI and GUI integration
- Performance optimizations
- Extensive documentation and testing

### Future Releases
- Additional Czkawka tool integrations
- Enhanced performance optimizations
- Extended configuration options
- Advanced filtering capabilities
- Improved error handling and recovery