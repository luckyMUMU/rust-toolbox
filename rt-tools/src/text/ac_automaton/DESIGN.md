# AC Automaton Tool Design Document

## Overview

The AC Automaton tool (`text.ac_automaton`) implements the Aho-Corasick algorithm for efficient multi-pattern string matching. This tool allows users to search for multiple patterns simultaneously in one or more texts, making it highly efficient for large-scale text processing tasks.

## Features

### Core Functionality
- **Multi-pattern Matching**: Search for multiple patterns in a single pass through the text
- **Pattern Management**: Add, remove, and list patterns in the automaton
- **Case Sensitivity Control**: Support both case-sensitive and case-insensitive matching
- **Parallel Processing**: Optional parallel matching for multiple texts
- **Performance Metrics**: Execution time tracking for performance analysis

### Supported Operations
1. **Add Patterns** (`add`): Add one or more patterns to the automaton
2. **Remove Patterns** (`remove`): Remove patterns with confirmation requirement
3. **List Patterns** (`list`): Display all currently loaded patterns
4. **Match Texts** (`match`): Perform pattern matching on input texts
5. **Save/Load** (`save`/`load`): Persistence operations (planned for future implementation)

## Architecture

### Core Components

#### AcAutomatonImpl
Internal implementation managing:
- **Aho-Corasick Automaton**: The underlying pattern matching engine
- **Pattern Storage**: Thread-safe storage of active patterns
- **Builder Configuration**: Automaton configuration (case sensitivity, etc.)

#### Pattern Management
- **Validation**: Ensures patterns are non-empty and unique
- **Deduplication**: Prevents duplicate pattern registration
- **Atomic Operations**: Thread-safe pattern addition and removal

#### Matching Engine
- **Single Text Matching**: Efficient matching for individual texts
- **Parallel Processing**: Async parallel matching for multiple texts
- **Result Aggregation**: Combines results from multiple matching operations

## Input Schema

```json
{
  "action": "match",
  "patterns": ["pattern1"],
  "texts": ["text to search"],
  "confirm": false,
  "ignore_case": false,
  "parallel": false
}
```

### Field Descriptions
- **action**: Operation to perform (`add`, `remove`, `list`, `match`, `save`, `load`)
- **patterns**: Array of pattern strings to add/remove or use for matching
- **texts**: Array of text strings to search (only used with `match` action)
- **confirm**: Boolean flag required for `remove` operations to prevent accidental deletion
- **ignore_case**: Enable case-insensitive matching (rebuilds automaton)
- **parallel**: Use parallel processing for multiple text matching

## Output Schema

```json
{
  "success": true,
  "message": "操作成功",
  "results": [
    {
      "pattern": "pattern1",   // Matched pattern
      "start": 0,              // Start position in text
      "end": 8                 // End position in text
    }
  ],
  "patterns": ["pattern1"],    // Current pattern list (for list action)
  "elapsed_ms": 15            // Execution time in milliseconds
}
```

### Result Fields
- **success**: Boolean indicating operation success
- **message**: Localized status message
- **results**: Array of match results with pattern, start, and end positions
- **patterns**: Current list of loaded patterns
- **elapsed_ms**: Performance timing information

## Usage Examples

### Adding Patterns
```json
{
  "action": "add",
  "patterns": ["hello", "world", "rust"]
}
```

### Pattern Matching
```json
{
  "action": "match",
  "patterns": ["hello", "world"],
  "texts": ["hello world", "rust programming"],
  "ignore_case": false,
  "parallel": true
}
```

### Removing Patterns
```json
{
  "action": "remove",
  "patterns": ["hello"],
  "confirm": true
}
```

### Listing Patterns
```json
{
  "action": "list",
  "patterns": []
}
```

## Performance Characteristics

### Time Complexity
- **Pattern Addition**: O(m) where m is the total length of all patterns
- **Automaton Construction**: O(m) for building the failure function
- **Text Matching**: O(n + z) where n is text length and z is number of matches
- **Parallel Matching**: Near-linear speedup with number of CPU cores

### Memory Usage
- **Pattern Storage**: O(m) for pattern strings
- **Automaton Structure**: O(m) for state machine
- **Match Results**: O(z) for storing match positions

### Optimization Features
- **Lazy Automaton Rebuild**: Only rebuilds when patterns change
- **Async Parallel Processing**: Non-blocking parallel text processing
- **Efficient Memory Layout**: Minimal memory overhead per pattern

## Error Handling

### Input Validation Errors
- **Empty Pattern**: Patterns cannot be empty strings
- **Duplicate Pattern**: Attempting to add existing patterns
- **Missing Confirmation**: Remove operations require explicit confirmation
- **Invalid Action**: Unsupported operation types

### Runtime Errors
- **Automaton Build Failure**: Issues constructing the Aho-Corasick automaton
- **Parallel Execution Failure**: Errors in async task execution
- **Memory Allocation**: Out-of-memory conditions for large pattern sets

### Error Recovery
- **Graceful Degradation**: Failed operations don't affect existing patterns
- **State Preservation**: Automaton state remains consistent after errors
- **Detailed Error Messages**: Localized error descriptions for debugging

## Internationalization

### Supported Locales
- **English (`en`)**: Primary development language
- **Chinese (`zh-CN`)**: Simplified Chinese translations

### Localized Elements
- **Display Name**: Tool name in user interfaces
- **Description**: Tool purpose and capabilities
- **User Guide**: Comprehensive usage instructions
- **Field Titles**: Input/output field labels
- **Action Labels**: Operation type descriptions
- **Error Messages**: Localized error descriptions

## Testing Strategy

### Unit Tests
- **Pattern Management**: Add, remove, list operations
- **Matching Accuracy**: Verify correct pattern detection
- **Case Sensitivity**: Test both case-sensitive and insensitive modes
- **Error Conditions**: Validate proper error handling
- **Performance**: Benchmark execution times

### Integration Tests
- **Tool Registration**: Verify proper integration with rt-core
- **Schema Validation**: Test input/output schema compliance
- **Localization**: Ensure all locales load correctly
- **Async Operations**: Test parallel processing functionality

### Performance Tests
- **Large Pattern Sets**: Test with thousands of patterns
- **Long Texts**: Validate performance on large documents
- **Parallel Scaling**: Measure speedup with multiple texts
- **Memory Usage**: Monitor memory consumption patterns

## Future Enhancements

### Planned Features
- **Pattern Persistence**: Save/load pattern sets to/from files
- **Regular Expression Support**: Extend beyond literal string patterns
- **Streaming Processing**: Handle very large texts in chunks
- **Advanced Statistics**: Detailed matching statistics and analytics

### API Extensions
- **Batch Operations**: Bulk pattern management operations
- **Configuration Profiles**: Named pattern sets for different use cases
- **Export Formats**: Multiple output formats (CSV, XML, etc.)
- **Integration Hooks**: Callbacks for real-time processing

## Dependencies

### Core Dependencies
- **aho-corasick**: Efficient Aho-Corasick implementation
- **tokio**: Async runtime for parallel processing
- **serde**: Serialization for input/output handling
- **schemars**: JSON schema generation

### Development Dependencies
- **rt-core**: Core tool trait and error types
- **futures**: Future utilities for async operations
- **serde_json**: JSON processing and schema handling

## Compatibility

### Platform Support
- **Windows**: Full support with native compilation
- **Linux**: Full support with native compilation
- **macOS**: Full support with native compilation

### Rust Version
- **Minimum**: Rust 2021 Edition
- **Recommended**: Latest stable Rust version
- **Features**: Uses async/await, const generics, and other modern features