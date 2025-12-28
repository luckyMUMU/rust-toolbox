# WASM Tools Examples

This directory contains example WASM modules for demonstrating the workflow toolkit's WASM plugin capabilities.

## Files

### simple_calculator.wat
A WebAssembly Text format file demonstrating a basic calculator module compatible with Wasmtime.

To compile to WASM:
```bash
wat2wasm simple_calculator.wat
```

### text_processor.js
A JavaScript source file that can be compiled to WASM for use with Extism.

To compile to WASM using Javy (JavaScript to WASM compiler):
```bash
# Install Javy
curl -L https://github.com/bytecodealliance/javy/releases/latest/download/javy-x86_64-linux.gz | gunzip > javy
chmod +x javy

# Compile JavaScript to WASM
./javy compile text_processor.js -o text_processor.wasm
```

Alternatively, you can use other JavaScript-to-WASM tools or write the module in other languages that compile to WASM.

## Runtime Support

### Wasmtime Runtime
- Uses the `simple_calculator.wat` example
- Provides low-level WASM execution
- Suitable for performance-critical applications
- Requires manual memory management

### Extism Runtime
- Uses the `text_processor.js` example (compiled to WASM)
- Provides high-level multi-language support
- Built-in security and sandboxing
- Automatic memory management
- Support for network access and configuration

## Usage in Workflow Toolkit

Both runtime types are supported through the WASM plugin system:

```rust
// Wasmtime example
let wasmtime_plugin = WasmPluginBuilder::new("calculator".to_string(), "1.0.0".to_string())
    .with_runtime_type(WasmRuntimeType::Wasmtime)
    .with_module_path(PathBuf::from("examples/wasm_tools/simple_calculator.wasm"))
    .add_entry_point("calculate".to_string(), "calculate".to_string())
    .build();

// Extism example
let extism_plugin = WasmPluginBuilder::new("processor".to_string(), "1.0.0".to_string())
    .with_runtime_type(WasmRuntimeType::Extism)
    .with_module_path(PathBuf::from("examples/wasm_tools/text_processor.wasm"))
    .add_entry_point("process_text".to_string(), "process_text".to_string())
    .add_config_data("language".to_string(), "en".to_string())
    .build();
```

## Multi-Language Support

The Extism runtime supports WASM modules compiled from various languages:

- **JavaScript/TypeScript**: Using Javy or similar tools
- **Rust**: Using `wasm32-wasi` target
- **Go**: Using TinyGo with WASM target
- **Python**: Using Pyodide or similar tools
- **C/C++**: Using Emscripten
- **AssemblyScript**: Direct compilation to WASM

This enables cross-language data processing and transformation within the workflow toolkit.