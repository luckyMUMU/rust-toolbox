# Chinese Converter Tool Design Document

## Overview

The Chinese Converter tool (`text.convert_chinese`) provides high-quality conversion between Simplified and Traditional Chinese text, including regional variants for Taiwan and Hong Kong. It uses the `ferrous-opencc` library, a pure Rust implementation of OpenCC (Open Chinese Convert) for accurate and efficient text conversion.

## Features

### Core Functionality
- **Bidirectional Conversion**: Convert between Simplified and Traditional Chinese
- **Regional Variants**: Support for Taiwan and Hong Kong specific variants
- **Phrase-Level Accuracy**: Uses OpenCC dictionaries for contextual conversion
- **High Performance**: Pure Rust implementation without C++ dependencies
- **Unicode Support**: Full Unicode text processing capabilities

### Supported Conversion Modes
1. **s2t**: Simplified Chinese to Traditional Chinese
2. **t2s**: Traditional Chinese to Simplified Chinese
3. **s2tw**: Simplified Chinese to Taiwan Traditional Chinese
4. **tw2s**: Taiwan Traditional Chinese to Simplified Chinese
5. **s2hk**: Simplified Chinese to Hong Kong Traditional Chinese
6. **hk2s**: Hong Kong Traditional Chinese to Simplified Chinese
7. **s2twp**: Simplified Chinese to Taiwan Traditional Chinese with phrases
8. **tw2sp**: Taiwan Traditional Chinese to Simplified Chinese with phrases

## Architecture

### Core Components

#### OpenCC Integration
- **Dictionary Loading**: Loads OpenCC conversion dictionaries
- **Configuration Management**: Maps string modes to OpenCC configurations
- **Converter Instances**: Manages OpenCC converter lifecycle

#### Text Processing
- **Input Validation**: Ensures text is valid Unicode
- **Conversion Engine**: Performs character and phrase-level conversion
- **Output Formatting**: Returns converted text with proper encoding

#### Error Handling
- **Mode Validation**: Validates conversion mode parameters
- **Dictionary Errors**: Handles missing or corrupted dictionaries
- **Conversion Failures**: Manages text processing errors

## Input Schema

```json
{
  "type": "object",
  "properties": {
    "text": {
      "type": "string",
      "title": "Input Text",
      "description": "Chinese text to convert"
    },
    "mode": {
      "type": "string",
      "enum": ["s2t", "t2s", "s2tw", "tw2s", "s2hk", "hk2s", "s2twp", "tw2sp"],
      "title": "Conversion Mode",
      "description": "Type of conversion to perform",
      "x-enum-labels": {
        "s2t": "Simplified to Traditional",
        "t2s": "Traditional to Simplified", 
        "s2tw": "Simplified to Taiwan Traditional",
        "tw2s": "Taiwan Traditional to Simplified",
        "s2hk": "Simplified to Hong Kong Traditional",
        "hk2s": "Hong Kong Traditional to Simplified",
        "s2twp": "Simplified to Taiwan Traditional (with phrases)",
        "tw2sp": "Taiwan Traditional to Simplified (with phrases)"
      }
    }
  },
  "required": ["text", "mode"]
}
```

### Field Descriptions
- **text**: Unicode string containing Chinese text to convert
- **mode**: Conversion direction and regional variant specification

## Output Schema

```json
{
  "type": "object",
  "properties": {
    "converted": {
      "type": "string",
      "title": "Converted Text",
      "description": "Text after Chinese conversion"
    }
  },
  "required": ["converted"]
}
```

### Result Fields
- **converted**: The converted Chinese text in the target variant

## Operation Logic

### 1. Input Validation
- Parse input JSON into `ConvertChineseInput` structure
- Validate that text is non-empty and contains valid Unicode
- Verify that mode is one of the supported conversion types

### 2. Configuration Mapping
- Map string mode to `ferrous_opencc::config::BuiltinConfig` enum
- Load appropriate OpenCC dictionary configuration
- Initialize converter instance with selected configuration

### 3. Text Conversion
- Process input text through OpenCC converter
- Apply character-level and phrase-level conversion rules
- Handle mixed Chinese/non-Chinese text appropriately

### 4. Result Generation
- Package converted text into output structure
- Serialize result as JSON response
- Preserve original formatting and non-Chinese characters

## Usage Examples

### Simplified to Traditional
```json
{
  "text": "简体中文转换",
  "mode": "s2t"
}
```
Result: `{"converted": "簡體中文轉換"}`

### Traditional to Simplified
```json
{
  "text": "繁體中文轉換",
  "mode": "t2s"
}
```
Result: `{"converted": "繁体中文转换"}`

### Taiwan Variant Conversion
```json
{
  "text": "计算机软件",
  "mode": "s2tw"
}
```
Result: `{"converted": "電腦軟體"}`

### Mixed Text Handling
```json
{
  "text": "Hello 世界！",
  "mode": "s2t"
}
```
Result: `{"converted": "Hello 世界！"}`

## Performance Characteristics

### Time Complexity
- **Dictionary Loading**: O(1) amortized (cached after first use)
- **Text Conversion**: O(n) where n is text length
- **Memory Usage**: O(d + n) where d is dictionary size and n is text length

### Optimization Features
- **Dictionary Caching**: OpenCC dictionaries loaded once per mode
- **Efficient Processing**: Direct Unicode string processing
- **Minimal Allocations**: Reuses converter instances where possible

## Error Handling

### Input Validation Errors
- **Empty Text**: Text parameter cannot be empty
- **Invalid Mode**: Mode must be one of the supported conversion types
- **Malformed JSON**: Input must be valid JSON structure

### Runtime Errors
- **Dictionary Load Failure**: OpenCC dictionary files missing or corrupted
- **Conversion Failure**: Internal OpenCC processing errors
- **Memory Allocation**: Out-of-memory conditions for large texts

### Error Recovery
- **Graceful Degradation**: Returns original text if conversion fails
- **Detailed Error Messages**: Specific error descriptions for debugging
- **State Preservation**: Converter state remains consistent after errors

## Internationalization

### Supported Locales
- **English (`en`)**: Primary development language
- **Chinese (`zh-CN`)**: Simplified Chinese translations

### Localized Elements
- **Display Name**: Tool name in user interfaces
- **Description**: Tool purpose and capabilities
- **User Guide**: Comprehensive usage instructions
- **Field Titles**: Input/output field labels
- **Mode Labels**: Conversion mode descriptions
- **Error Messages**: Localized error descriptions

## Testing Strategy

### Unit Tests
- **Mode Validation**: Test all supported conversion modes
- **Text Processing**: Verify conversion accuracy with known examples
- **Error Conditions**: Test invalid inputs and error handling
- **Unicode Support**: Test with various Unicode characters and encodings

### Integration Tests
- **Tool Registration**: Verify proper integration with rt-core
- **Schema Validation**: Test input/output schema compliance
- **Localization**: Ensure all locales load correctly
- **Performance**: Benchmark conversion speed with large texts

### Accuracy Tests
- **Dictionary Validation**: Verify OpenCC dictionary integrity
- **Conversion Quality**: Test with standard Chinese text samples
- **Regional Variants**: Validate Taiwan and Hong Kong specific conversions
- **Mixed Content**: Test handling of Chinese/non-Chinese mixed text

## Dependencies

### Core Dependencies
- **ferrous-opencc**: Pure Rust OpenCC implementation
- **rt-core**: Core tool trait and error types
- **serde**: Serialization for input/output handling
- **schemars**: JSON schema generation

### OpenCC Dictionaries
- **Built-in Dictionaries**: Embedded OpenCC conversion tables
- **No External Files**: All dictionaries compiled into binary
- **Version Compatibility**: Uses stable OpenCC dictionary format

## Compatibility

### Platform Support
- **Windows**: Full support with native compilation
- **Linux**: Full support with native compilation
- **macOS**: Full support with native compilation

### Text Encoding
- **UTF-8**: Primary encoding for all text processing
- **Unicode**: Full Unicode character set support
- **Cross-Platform**: Consistent behavior across operating systems

## Future Enhancements

### Planned Features
- **Custom Dictionaries**: Support for user-provided conversion dictionaries
- **Batch Processing**: Convert multiple texts in single operation
- **Conversion Statistics**: Detailed metrics on conversion operations
- **Format Preservation**: Maintain text formatting during conversion

### API Extensions
- **Streaming Conversion**: Handle very large texts in chunks
- **Confidence Scoring**: Provide conversion confidence metrics
- **Alternative Suggestions**: Multiple conversion options for ambiguous text
- **Integration Hooks**: Callbacks for custom conversion logic

## Quality Assurance

### Conversion Accuracy
- **OpenCC Standard**: Uses industry-standard OpenCC dictionaries
- **Phrase Context**: Considers phrase-level context for accurate conversion
- **Regional Accuracy**: Proper handling of Taiwan and Hong Kong variants
- **Continuous Testing**: Regular validation against known conversion pairs

### Performance Monitoring
- **Benchmark Suite**: Regular performance testing with various text sizes
- **Memory Profiling**: Monitor memory usage patterns
- **Conversion Speed**: Track conversion throughput metrics
- **Resource Usage**: Monitor CPU and memory consumption
