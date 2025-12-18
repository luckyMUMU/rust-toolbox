# Pinyin Converter Plugin

## Overview

The Pinyin Converter Plugin (`rt-plugin-pinyin`) is a specialized text processing plugin for Rust Toolbox that converts Chinese text to Pinyin romanization. It supports both toned and toneless Pinyin output and handles mixed Chinese-English text seamlessly.

## Features

### Core Functionality
- **Chinese to Pinyin Conversion**: Convert Chinese characters to Pinyin romanization
- **Tone Support**: Optional tone marks (diacritics) in output
- **Mixed Text Handling**: Preserves non-Chinese text while converting Chinese characters
- **Unicode Support**: Full Unicode Chinese character support
- **High Accuracy**: Uses comprehensive Chinese character database

### Conversion Options
- **With Tones**: Includes tone marks (nǐ hǎo shì jiè)
- **Without Tones**: Plain ASCII output (ni hao shi jie)
- **Flexible Input**: Handles pure Chinese, mixed Chinese-English, and punctuation

## Installation

### Prerequisites
- **Rust Toolbox**: Requires rt-cli or rt-gui host application
- **Platform Support**: Windows, Linux, and macOS

### Build Instructions

1. **Clone Repository**:
   ```bash
   git clone https://github.com/your-repo/rust-tool.git
   cd rust-tool
   ```

2. **Build Plugin**:
   ```bash
   cargo build --release --package rt-plugin-pinyin
   ```

3. **Deploy Plugin**:
   ```bash
   # Copy to plugins directory
   cp target/release/rt-plugin-pinyin.exe plugins/  # Windows
   cp target/release/rt-plugin-pinyin plugins/      # Unix systems
   ```

### Verification
```bash
# Verify plugin is available
rt-cli list | grep pinyin

# Get plugin specification
rt-cli plugin spec --name text.pinyin
```

## Usage

### Command Line Interface (CLI)

#### Basic Conversion with Tones
```bash
echo '{"text": "你好世界", "tone": true}' | rt-cli run text.pinyin
# Output: {"pinyin": "nǐ hǎo shì jiè"}
```

#### Conversion without Tones
```bash
echo '{"text": "你好世界", "tone": false}' | rt-cli run text.pinyin
# Output: {"pinyin": "ni hao shi jie"}
```

#### Mixed Text Conversion
```bash
echo '{"text": "Hello世界！How are you？", "tone": true}' | rt-cli run text.pinyin
# Output: {"pinyin": "Hello shì jiè ！How are you？"}
```

#### Complex Text Example
```bash
echo '{"text": "北京大学计算机科学与技术专业", "tone": false}' | rt-cli run text.pinyin
# Output: {"pinyin": "bei jing da xue ji suan ji ke xue yu ji shu zhuan ye"}
```

### Graphical Interface (GUI)

1. **Launch GUI**: Start `rt-gui` application
2. **Navigate to Text Tools**: Find "Pinyin Converter" in the text processing section
3. **Configure Parameters**:
   - Enter Chinese text in the "Text" field
   - Toggle "Include Tones" checkbox as desired
4. **Execute**: Click "Run" to perform conversion
5. **View Results**: Converted Pinyin appears in the output panel

### Batch Processing
```bash
# Process multiple texts (using shell scripting)
for text in "你好" "世界" "中国"; do
  echo "{\"text\": \"$text\", \"tone\": true}" | rt-cli run text.pinyin
done
```

## Input/Output Schema

### Input Schema
```json
{
  "type": "object",
  "properties": {
    "text": {
      "type": "string",
      "title": "Input Text",
      "description": "Chinese text to convert to Pinyin"
    },
    "tone": {
      "type": "boolean",
      "default": true,
      "title": "Include Tones",
      "description": "Whether to include tone marks in the output"
    }
  },
  "required": ["text"]
}
```

#### Field Descriptions
- **text**: Unicode string containing Chinese text (required)
- **tone**: Boolean flag for tone mark inclusion (optional, defaults to true)

### Output Schema
```json
{
  "type": "object",
  "properties": {
    "pinyin": {
      "type": "string",
      "title": "Pinyin Result",
      "description": "Converted Pinyin text"
    }
  },
  "required": ["pinyin"]
}
```

#### Result Fields
- **pinyin**: The converted Pinyin text with or without tone marks

## Technical Details

### Character Processing
- **Unicode Support**: Handles all Unicode Chinese characters
- **Traditional/Simplified**: Supports both Traditional and Simplified Chinese
- **Punctuation Preservation**: Maintains original punctuation and spacing
- **Non-Chinese Text**: Passes through non-Chinese characters unchanged

### Tone Mark System
- **Tone 1**: High level (ā, ē, ī, ō, ū, ǖ)
- **Tone 2**: Rising (á, é, í, ó, ú, ǘ)
- **Tone 3**: Falling-rising (ǎ, ě, ǐ, ǒ, ǔ, ǚ)
- **Tone 4**: Falling (à, è, ì, ò, ù, ǜ)
- **Neutral Tone**: No mark (a, e, i, o, u, ü)

### Performance Characteristics
- **Time Complexity**: O(n) where n is text length
- **Memory Usage**: Minimal memory footprint
- **Processing Speed**: Fast conversion suitable for real-time use
- **Accuracy**: High accuracy based on comprehensive character database

## Configuration

### Default Settings
- **Tone Marks**: Enabled by default
- **Encoding**: UTF-8 input and output
- **Separator**: Space-separated Pinyin syllables
- **Case**: Lowercase output

### Customization Options
While the current version has fixed settings, future versions may support:
- Custom syllable separators
- Case conversion options
- Alternative romanization systems
- Batch processing modes

## Internationalization

### Supported Languages
- **English (`en`)**: Primary development language
- **Chinese (`zh-CN`)**: Simplified Chinese translations

### Localized Elements
- **Tool Name**: "Pinyin Converter" / "拼音转换器"
- **Description**: Tool purpose and functionality
- **Field Labels**: Input/output field titles
- **User Guide**: Comprehensive usage instructions
- **Error Messages**: Localized error descriptions

### Localization Files
```
rt-plugin-pinyin/
└── locales/
    ├── tool.en.json     # English translations
    └── tool.zh.json     # Chinese translations
```

## Error Handling

### Common Error Conditions
- **Empty Input**: Text parameter cannot be empty
- **Invalid JSON**: Input must be valid JSON format
- **Encoding Issues**: Text must be valid UTF-8

### Error Recovery
- **Graceful Degradation**: Returns original text if conversion fails
- **Partial Processing**: Converts recognizable characters, preserves others
- **Detailed Messages**: Clear error descriptions for troubleshooting

## Development

### Project Structure
```
rt-plugin-pinyin/
├── Cargo.toml              # Dependencies and metadata
├── README.md               # This documentation
├── DESIGN.md               # Technical design document
├── input.json              # Sample input for testing
├── src/
│   ├── main.rs            # Plugin entry point and CLI handling
│   └── i18n.rs            # Internationalization support
└── locales/               # Internationalization resources
    ├── tool.en.json       # English translations
    └── tool.zh.json       # Chinese translations
```

### Dependencies
- **pinyin**: Core Pinyin conversion library
- **serde**: JSON serialization/deserialization
- **clap**: Command-line argument parsing
- **rt-core**: Rust Toolbox core types and traits

### Building from Source
```bash
# Development build
cargo build --package rt-plugin-pinyin

# Release build with optimizations
cargo build --release --package rt-plugin-pinyin

# Run tests
cargo test --package rt-plugin-pinyin
```

## Testing

### Unit Tests
```bash
# Run plugin-specific tests
cargo test --package rt-plugin-pinyin

# Run with output
cargo test --package rt-plugin-pinyin -- --nocapture
```

### Integration Tests
```bash
# Test plugin integration with rt-cli
echo '{"text": "测试", "tone": true}' | cargo run --bin rt-cli -- run text.pinyin
```

### Test Cases
- **Basic Conversion**: Simple Chinese characters
- **Tone Variations**: All four tones plus neutral tone
- **Mixed Content**: Chinese text with English and punctuation
- **Edge Cases**: Empty strings, special characters, numbers
- **Unicode**: Various Unicode Chinese character ranges

## Performance Benchmarks

### Typical Performance
- **Short Text** (< 100 chars): < 1ms
- **Medium Text** (< 1000 chars): < 10ms
- **Long Text** (< 10000 chars): < 100ms
- **Memory Usage**: < 1MB for typical operations

### Optimization Features
- **Efficient Lookup**: Fast character-to-Pinyin mapping
- **Minimal Allocations**: Optimized string processing
- **Unicode Handling**: Efficient Unicode character processing

## Use Cases

### Educational Applications
- **Language Learning**: Help students learn Chinese pronunciation
- **Dictionary Tools**: Provide Pinyin for Chinese entries
- **Reading Aids**: Assist with Chinese text comprehension

### Text Processing
- **Search Indexing**: Create searchable Pinyin indexes
- **Data Normalization**: Convert Chinese text to ASCII-compatible format
- **Input Methods**: Support Pinyin-based input systems

### Integration Examples
- **Document Processing**: Add Pinyin annotations to Chinese documents
- **Web Applications**: Provide Pinyin tooltips for Chinese text
- **Mobile Apps**: Enable Pinyin search for Chinese content

## Troubleshooting

### Common Issues

#### Plugin Not Recognized
- **Cause**: Plugin not in correct directory
- **Solution**: Ensure plugin is in `plugins/` directory with execute permissions

#### Conversion Errors
- **Cause**: Invalid input format or encoding issues
- **Solution**: Verify input is valid UTF-8 JSON

#### Missing Tone Marks
- **Cause**: Terminal or display doesn't support Unicode
- **Solution**: Use `"tone": false` for ASCII-only output

### Debug Information
```bash
# Enable debug logging
RUST_LOG=debug rt-cli run text.pinyin < input.json

# Verbose plugin information
rt-cli plugin spec --name text.pinyin --verbose
```

## License

This plugin is licensed under the same terms as the Rust Toolbox project. See the main project LICENSE file for details.

## Contributing

Contributions are welcome! Please ensure all changes include:

- **Tests**: Comprehensive test coverage for new features
- **Documentation**: Updated documentation for changes
- **Localization**: Updates to all supported language files
- **Performance**: Consider performance impact of changes

## Support

For issues, questions, or contributions:

1. **GitHub Issues**: Report bugs and request features
2. **Documentation**: Check the main project documentation
3. **Community**: Join project discussions
4. **Testing**: Provide sample inputs that cause issues

## Changelog

### v0.1.0 (Current)
- Initial Pinyin conversion implementation
- Support for toned and toneless output
- Mixed text handling
- Full internationalization support
- CLI and GUI integration
- Comprehensive documentation

### Future Enhancements
- **Alternative Systems**: Support for other romanization systems
- **Batch Processing**: Enhanced batch conversion capabilities
- **Custom Dictionaries**: User-provided pronunciation dictionaries
- **Advanced Options**: More conversion customization options
