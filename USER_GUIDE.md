# Rust Toolbox User Guide

## 1. Introduction

Rust Toolbox (rt-box) is a powerful modular tool integration platform built in Rust that provides unified management and orchestration of various tools through a workflow engine. The platform features a plugin-based architecture supporting dynamic loading and extension, with both CLI and GUI interfaces.

### 1.1 Core Features

- **Unified Tool Management**: Centralized tool discovery, execution, and management
- **Plugin Architecture**: Dynamic loading of external tools as plugins (executable files and WebAssembly)
- **Workflow Engine**: DAG-based workflow orchestration for automated task processing
- **Multi-language Support**: Full internationalization (i18n) for English and Chinese
- **Dual Interface**: Both command-line (rt-cli) and graphical (rt-gui) interfaces
- **Model Context Protocol (MCP)**: Standardized context management and tool calling protocol
- **Persistence Layer**: Unified data storage, caching, and configuration management

### 1.2 Related Documentation

- [Design Document](DESIGN.md): Overall project design, technology stack and core principles
- [Architecture Design](ARCHITECTURE_DESIGN.md): Detailed architecture design, core components and deployment architecture
- [Plugin Development Guide](PLUGIN_GUIDE.md): Plugin development standards and guidelines
- [AI Work Protocol](AI_WORK_PROTOCOL.md): AI-assisted development workflow standards
- [Changelog](CHANGELOG.md): Project change history

## 2. Core Concepts

- **Tool**: An atomic unit that performs a single task, implementing the Tool trait
- **Plugin**: External executable that provides tools through a standardized protocol
- **Workflow**: A sequence of tools executed in a defined order with data flow
- **MCP Context**: Context object containing execution state, history and environment information
- **Persistence Layer**: Unified data storage, caching, and configuration management system

## 3. Tool Library

### 3.1 Built-in Tools

#### 3.1.1 File Operations

##### 📂 Move Folder (`file.move_folder`)
Move or rename a specified folder with collision handling.

**Behavior:**
1. **Rename/Move**: If `destination` doesn't exist, the source folder will be renamed or moved to that path
2. **Move Into**: If `destination` is an existing directory, the source folder will be moved **inside** that directory
3. **Overwrite Protection**: Prevents accidental overwrites unless explicitly enabled

**Input Schema:**
```json
{
  "source": "path/to/source_folder",
  "destination": "path/to/target_folder",
  "overwrite": false
}
```

**Output Schema:**
```json
{
  "success": true,
  "moved_files": 0
}
}
```

#### 3.1.2 Text Processing

##### 🔤 AC Automaton (`text.ac_automaton`)
Aho-Corasick pattern matching with multi-pattern support for efficient string searching.

**Behavior:**
1. **Pattern Management**: Add, remove, and list pattern strings
2. **Multi-text Matching**: Match patterns across multiple text inputs
3. **Parallel Processing**: Optional parallel matching for improved performance
4. **Case Sensitivity**: Configurable case-sensitive or case-insensitive matching

**Input Schema:**
```json
{
  "action": "match",
  "patterns": ["pattern1", "pattern2"],
  "texts": ["text1", "text2"],
  "confirm": true,
  "ignore_case": false,
  "parallel": false
}
```

**Output Schema:**
```json
{
  "success": true,
  "message": "Operation completed successfully"
  "results": [                          // Match results (for match action)
    {
      "pattern": "pattern1",
      "start": 0,
      "end": 8
    }
  ],
  "patterns": ["pattern1", "pattern2"], // Current patterns (for list action)
  "elapsed_ms": 123                     // Operation duration in milliseconds
}
```

##### 🈳 Chinese Converter (`text.convert_chinese`)
Convert between Traditional and Simplified Chinese with multiple regional variants.

**Behavior:**
1. **Bidirectional Conversion**: Convert between Traditional and Simplified Chinese
2. **Regional Variants**: Support for Taiwan, Hong Kong, and other regional variants
3. **Phrase Conversion**: Handles both character and phrase-level conversions

**Input Schema:**
```json
{
  "text": "你好世界",
  "mode": "s2t"
}
```

**Output Schema:**
```json
{
  "converted": "你好世界"
}
```

### 3.2 Plugin Tools

#### 3.2.1 Text Processing Plugins

##### 🔤 Chinese to Pinyin (`text.pinyin`)
Convert Chinese text to Pinyin with optional tone marks.

**Behavior:**
1. **Pinyin Conversion**: Convert Chinese characters to corresponding Pinyin
2. **Tone Control**: Optional tone marks for pronunciation guidance
3. **Mixed Text**: Support for mixed Chinese-English text, converting only Chinese parts

**Input Schema:**
```json
{
  "text": "你好世界",
  "tone": true
}
```

**Output Schema:**
```json
{
  "pinyin": "nǐ hǎo shì jiè"
}
```

#### 3.2.2 Media Operations

##### 📹 YouTube Downloader (`media.ytdlp`)
Download videos and audio content from YouTube and other supported websites.

**Behavior:**
1. **Single Video**: Download individual videos
2. **Playlist Support**: Download entire playlists
3. **Format Selection**: Multiple format options for different quality/size needs
4. **Subtitle Download**: Download subtitles including auto-generated ones
5. **Custom Output**: Configurable output directory and filename templates

**Input Schema:**
```json
{
  "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
  "format": "best",
  "playlist": false,
  "subtitles": false,
  "output_dir": ".",
  "filename_template": "%(title)s.%(ext)s"
}
```

**Output Schema:**
```json
{
  "success": true,
  "files": [
    {
      "path": "./Rick Astley - Never Gonna Give You Up (Official Music Video).mp4",
      "size": 123456789
    }
  ],
  "message": "Successfully downloaded 1 file"
}
```

#### 3.2.3 System Utilities (Czkawka Integration)

The Czkawka plugin provides multiple file management tools:

##### 🔍 Duplicate Files (`file.duplicates`)
Find duplicate files in specified directories based on content comparison.

**Input Schema:**
```json
{
  "directories": ["/path/to/search"],
  "min_size": 1024,
  "excluded_directories": [],
  "allowed_extensions": []
}
```

##### 🖼️ Similar Images (`file.similar_images`)
Find visually similar images using perceptual hashing.

**Input Schema:**
```json
{
  "directories": ["/path/to/images"],
  "threshold": 10,
  "hash_size": 8
}
```

##### 📁 Empty Directories (`file.empty_directories`)
Find and optionally remove empty directories.

**Input Schema:**
```json
{
  "directories": ["/path/to/search"]
}
```

##### 🗑️ Temporary Files (`file.temporary_files`)
Find temporary files that can be safely removed.

**Input Schema:**
```json
{
  "directories": ["/path/to/search"]
}
```

##### 🔗 Broken Symbolic Links (`file.broken_symlinks`)
Find symbolic links that point to non-existent targets.

**Input Schema:**
```json
{
  "directories": ["/path/to/search"]
}
```

## 4. Usage Guide

### 4.1 Command Line Interface (CLI) - `rt-cli`

#### 4.1.1 Basic Tool Operations

##### List All Available Tools
```bash
cargo run --bin rt-cli -- list
```

This command displays all available tools including:
- Built-in tools from rt-tools
- Plugin tools from the plugins directory
- Tool descriptions and capabilities

##### Run Individual Tools
Use the `--input` parameter to pass JSON input directly to tools.

**Example 1: Move Folder Tool**
```bash
cargo run --bin rt-cli -- run file.move_folder --input '{"source": "./tmp/source", "destination": "./tmp/target"}'
```

**Example 2: Chinese to Pinyin Conversion**
```bash
cargo run --bin rt-cli -- run text.pinyin --input '{"text": "你好世界", "tone": true}'
```

**Example 3: AC Automaton Pattern Matching**
```bash
# Add patterns
cargo run --bin rt-cli -- run text.ac_automaton --input '{"action": "add", "patterns": ["hello", "world"]}'

# Match patterns in text
cargo run --bin rt-cli -- run text.ac_automaton --input '{"action": "match", "texts": ["hello world", "goodbye world"]}'
```

**Example 4: Chinese Text Conversion**
```bash
cargo run --bin rt-cli -- run text.convert_chinese --input '{"text": "你好世界", "mode": "s2t"}'
```

#### 4.1.2 Workflow Management

##### Run Workflow
```bash
cargo run --bin rt-cli -- workflow run ./my_workflow.json
```

##### Check Workflow Status
```bash
cargo run --bin rt-cli -- workflow status <instance_id>
```

##### Pause Workflow
```bash
cargo run --bin rt-cli -- workflow pause <instance_id>
```

##### Stop Workflow
```bash
cargo run --bin rt-cli -- workflow stop <instance_id>
```

#### 4.1.3 MCP Server Management

##### Start MCP Server
```bash
cargo run --bin rt-cli -- mcp-server start --address 127.0.0.1 --port 8000
```

**Configuration Options:**
- `--address`: Server listening address (default: 127.0.0.1)
- `--port`: Server listening port (default: 8000)
- `--max-request-size`: Maximum request size (default: 10MB)
- `--enable-websocket`: Enable WebSocket support (default: true)

### 4.2 Workflow Definition

Workflows are defined in JSON format, containing nodes, edges, and metadata. Each node represents a tool invocation, and edges define dependencies between nodes.

#### Workflow Definition Example
```json
{
  "id": "text_processing_workflow",
  "name": "Text Processing Workflow",
  "description": "A sample workflow demonstrating tool chaining for text processing",
  "nodes": [
    {
      "id": "convert_chinese",
      "tool_name": "text.convert_chinese",
      "label": "Convert to Traditional Chinese",
      "input_mappings": {},
      "static_inputs": {
        "text": "你好世界",
        "mode": "s2t"
      }
    },
    {
      "id": "convert_pinyin",
      "tool_name": "text.pinyin",
      "label": "Convert to Pinyin",
      "input_mappings": {
        "text": "{{ convert_chinese.output.converted }}"
      },
      "static_inputs": {
        "tone": true
      }
    },
    {
      "id": "pattern_match",
      "tool_name": "text.ac_automaton",
      "label": "Pattern Matching",
      "input_mappings": {
        "texts": ["{{ convert_pinyin.output.pinyin }}"]
      },
      "static_inputs": {
        "action": "match",
        "patterns": ["nǐ", "hǎo"]
      }
    }
  ],
  "edges": [
    {
      "from": "convert_chinese",
      "to": "convert_pinyin"
    },
    {
      "from": "convert_pinyin",
      "to": "pattern_match"
    }
  ]
}
```

#### Workflow Definition Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique workflow identifier |
| `name` | String | Workflow display name |
| `description` | String | Workflow description |
| `nodes` | Array | List of workflow nodes |
| `edges` | Array | List of workflow edges defining dependencies |

#### Node Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique node identifier |
| `tool_name` | String | Tool name to invoke (e.g., `text.pinyin`) |
| `label` | String | Node display label (optional) |
| `input_mappings` | Object | Input field mappings, key is input field name, value is expression (e.g., `{{ task1.output.pinyin }}`) |
| `static_inputs` | Object | Static input parameters passed directly to the tool |

#### Edge Fields

| Field | Type | Description |
|-------|------|-------------|
| `from` | String | Source node ID |
| `to` | String | Target node ID |

### 4.3 Data Flow and Expressions

The workflow engine supports data passing between nodes through expressions. Expressions use the format `{{ node_id.output.field_path }}`, where:
- `node_id` is the source node ID
- `field_path` is the field path in the source node's output JSON

#### Example: Using Expressions for Data Flow

```json
{
  "nodes": [
    {
      "id": "find_duplicates",
      "tool_name": "file.duplicates",
      "static_inputs": {
        "directories": ["/home/user/documents"],
        "min_size": 1024
      }
    },
    {
      "id": "convert_results",
      "tool_name": "text.convert_chinese",
      "input_mappings": {
        "text": "{{ find_duplicates.output.summary }}"
      },
      "static_inputs": {
        "mode": "s2t"
      }
    },
    {
      "id": "generate_pinyin",
      "tool_name": "text.pinyin",
      "input_mappings": {
        "text": "{{ convert_results.output.converted }}"
      },
      "static_inputs": {
        "tone": false
      }
    }
  ],
  "edges": [
    {
      "from": "find_duplicates",
      "to": "convert_results"
    },
    {
      "from": "convert_results",
      "to": "generate_pinyin"
    }
  ]
}
```

In this example:
1. `find_duplicates` node calls the duplicate files tool to scan directories
2. `convert_results` node uses expression `{{ find_duplicates.output.summary }}` to extract the summary from the duplicate scan
3. `generate_pinyin` node converts the Chinese text to Pinyin for further processing

### 4.4 Workflow Best Practices

1. **Atomicity**: Each node should perform a single task for easier debugging and reuse
2. **Clear Naming**: Use descriptive names for nodes and workflows
3. **Error Handling**: Consider adding error handling nodes for potential failure scenarios
4. **Modularity**: Break complex workflows into simpler, manageable components
5. **Testing**: Validate workflows in a test environment before production use
6. **Documentation**: Document complex workflows with clear descriptions and comments

### 4.5 Graphical User Interface (GUI) - `rt-gui`

#### Starting the GUI
```bash
cargo run --bin rt-gui
```

#### Interface Operations

##### Tool Execution Panel
1. **Tool Selection**: Click on tools in the left sidebar (e.g., `file.move_folder`)
2. **Input Configuration**: 
   - Enter parameters in the "Input (JSON)" text area:
     ```json
     {
       "source": "/path/to/source",
       "destination": "/path/to/destination",
       "overwrite": true
     }
     ```
3. **Execution**: Click the "Run" button to execute the tool
4. **Results**: View execution results or error messages in the bottom panel

##### Workflow Designer
1. **Create Workflow**: Open the workflow designer from the main interface
2. **Add Nodes**: Drag tools from the toolbox to the canvas
3. **Configure Nodes**: Click nodes to configure input parameters and mappings in the properties panel
4. **Connect Nodes**: Draw connections between nodes to define dependencies
5. **Save Workflow**: Save the workflow as a JSON file
6. **Execute Workflow**: Run the workflow and monitor execution
7. **Monitor Progress**: View real-time execution status and logs

##### Workflow Monitoring
- **Real-time Status**: Display current workflow state (running, completed, failed)
- **Node Status**: Show individual node states (pending, running, success, failed)
- **Execution Logs**: Display detailed execution logs and progress information
- **Result Inspection**: Click nodes to view input/output data and execution details

### 4.6 Workflow Examples

#### Example 1: File Cleanup and Organization

```json
{
  "id": "file_cleanup_workflow",
  "name": "File Cleanup and Organization",
  "description": "Find duplicate files and organize them by moving to appropriate directories",
  "nodes": [
    {
      "id": "find_duplicates",
      "tool_name": "file.duplicates",
      "label": "Find Duplicate Files",
      "static_inputs": {
        "directories": ["/home/user/downloads"],
        "min_size": 1024
      }
    },
    {
      "id": "find_empty_dirs",
      "tool_name": "file.empty_directories",
      "label": "Find Empty Directories",
      "static_inputs": {
        "directories": ["/home/user/downloads"]
      }
    },
    {
      "id": "move_duplicates",
      "tool_name": "file.move_folder",
      "label": "Move Duplicate Files",
      "input_mappings": {
        "source": "{{ find_duplicates.output.duplicate_groups[0].files[1].path }}"
      },
      "static_inputs": {
        "destination": "/home/user/duplicates",
        "overwrite": false
      }
    }
  ],
  "edges": [
    {
      "from": "find_duplicates",
      "to": "move_duplicates"
    }
  ]
}
```

#### Example 2: Multi-language Text Processing

```json
{
  "id": "multilingual_text_processing",
  "name": "Multi-language Text Processing",
  "description": "Process Chinese text through conversion and pattern matching",
  "nodes": [
    {
      "id": "convert_to_traditional",
      "tool_name": "text.convert_chinese",
      "label": "Convert to Traditional Chinese",
      "static_inputs": {
        "text": "你好世界，欢迎使用工具箱",
        "mode": "s2t"
      }
    },
    {
      "id": "generate_pinyin",
      "tool_name": "text.pinyin",
      "label": "Generate Pinyin",
      "input_mappings": {
        "text": "{{ convert_to_traditional.output.converted }}"
      },
      "static_inputs": {
        "tone": true
      }
    },
    {
      "id": "setup_patterns",
      "tool_name": "text.ac_automaton",
      "label": "Setup Pattern Matching",
      "static_inputs": {
        "action": "add",
        "patterns": ["你好", "世界", "工具"]
      }
    },
    {
      "id": "match_patterns",
      "tool_name": "text.ac_automaton",
      "label": "Match Patterns",
      "input_mappings": {
        "texts": ["{{ convert_to_traditional.output.converted }}"]
      },
      "static_inputs": {
        "action": "match"
      }
    }
  ],
  "edges": [
    {
      "from": "convert_to_traditional",
      "to": "generate_pinyin"
    },
    {
      "from": "setup_patterns",
      "to": "match_patterns"
    },
    {
      "from": "convert_to_traditional",
      "to": "match_patterns"
    }
  ]
}
```

#### Example 3: Media Download and Processing

```json
{
  "id": "media_download_workflow",
  "name": "Media Download and Processing",
  "description": "Download YouTube videos and organize output files",
  "nodes": [
    {
      "id": "download_video",
      "tool_name": "media.ytdlp",
      "label": "Download Video",
      "static_inputs": {
        "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "output_dir": "./downloads",
        "format": "best",
        "subtitles": true
      }
    },
    {
      "id": "organize_files",
      "tool_name": "file.move_folder",
      "label": "Organize Downloaded Files",
      "input_mappings": {
        "source": "{{ download_video.output.files[0].path }}"
      },
      "static_inputs": {
        "destination": "./media/videos",
        "overwrite": false
      }
    }
  ],
  "edges": [
    {
      "from": "download_video",
      "to": "organize_files"
    }
  ]
}
```

### 4.7 Workflow Execution Process

1. **Definition Parsing**: Engine parses workflow JSON definition and validates completeness and correctness
2. **Dependency Graph Construction**: Build directed acyclic graph (DAG) based on edge definitions
3. **State Initialization**: Create workflow instance and initialize node states
4. **Node Execution**: Execute nodes according to dependency order:
   - Identify all nodes with no dependencies and execute in parallel
   - Update states and trigger dependent node execution upon completion
   - Repeat until all nodes complete or a node fails
5. **State Updates**: Update workflow and node states throughout execution
6. **Result Generation**: Collect outputs from all nodes and generate final results

### 4.8 Workflow States

| State | Description |
|-------|-------------|
| `Pending` | Workflow created but not yet started |
| `Running` | Workflow currently executing |
| `Paused` | Workflow paused, can be resumed via command |
| `Completed` | Workflow successfully completed |
| `Failed` | Workflow execution failed with error details |

### 4.9 Node States

| State | Description |
|-------|-------------|
| `Pending` | Node ready for execution but dependencies not yet completed |
| `Running` | Node currently executing |
| `Completed` | Node executed successfully |
| `Failed` | Node execution failed with error details |
| `Skipped` | Node execution was skipped |

## 5. Model Context Protocol (MCP) Support

Rust Toolbox implements the Model Context Protocol (MCP), enabling external systems to call tools and workflows through a standardized interface.

### 5.1 MCP Core Concepts

- **MCP Context**: Context object containing execution state, history, and environment information
- **MCP Request**: Standardized tool invocation format
- **MCP Response**: Standardized execution result format
- **MCP Server**: Provides REST API and WebSocket endpoints for external integration

### 5.2 Starting the MCP Server

#### Command Line
```bash
cargo run --bin rt-cli -- mcp-server start --address 127.0.0.1 --port 8000
```

#### Configuration Options
- `--address`: Server listening address (default: 127.0.0.1)
- `--port`: Server listening port (default: 8000)
- `--max-request-size`: Maximum request size (default: 10MB)
- `--enable-websocket`: Enable WebSocket support (default: true)

### 5.3 REST API Endpoints

#### Health Check
```
GET /health
```
**Response Example:**
```json
{ "status": "ok", "service": "mcp-server" }
```

#### List All Tools
```
GET /tools
```
**Response Example:**
```json
[
  {
    "name": "text.pinyin",
    "display_name": "Chinese to Pinyin",
    "description": "Convert Chinese text to Pinyin",
    "mcp_supported": true,
    "type": "plugin"
  },
  {
    "name": "text.ac_automaton",
    "display_name": "AC Automaton",
    "description": "Aho-Corasick pattern matching",
    "mcp_supported": false,
    "type": "core"
  }
]
```

#### List MCP-Supported Tools
```
GET /tools/mcp
```
**Response Example:**
```json
[
  {
    "name": "text.pinyin",
    "display_name": "Chinese to Pinyin",
    "description": "Convert Chinese text to Pinyin",
    "mcp_supported": true,
    "type": "plugin"
  }
]
```

#### Direct Tool Call
```
POST /tools/{tool_name}/call
Content-Type: application/json
```
**Request Example:**
```json
{
  "text": "你好世界",
  "tone": true
}
```
**Response Example:**
```json
{
  "success": true,
  "data": { "pinyin": "nǐ hǎo shì jiè" }
}
```

#### MCP Protocol Call
```
POST /mcp/call
Content-Type: application/json
```
**Request Example:**
```json
{
  "id": "req-12345",
  "component_type": "Tool",
  "component_name": "text.pinyin",
  "method": "run",
  "params": {
    "text": "你好世界",
    "tone": true
  },
  "context": {
    "id": "ctx-12345",
    "parent_id": null,
    "model_state": {},
    "execution_history": [],
    "environment_info": {},
    "metadata": {}
  },
  "service_context": {
    "caller_id": "test-caller",
    "caller_type": "User",
    "permission_level": "Standard",
    "extra": {}
  }
}
```
**Response Example:**
```json
{
  "id": "resp-67890",
  "request_id": "req-12345",
  "status": "Success",
  "data": { "pinyin": "nǐ hǎo shì jiè" },
  "error": null,
  "context": {
    "id": "ctx-12345",
    "parent_id": null,
    "model_state": {},
    "execution_history": [
      {
        "id": "exec-54321",
        "timestamp": "2025-12-16T08:00:00Z",
        "component_type": "Tool",
        "component_name": "text.pinyin",
        "method": "run",
        "input": { "text": "你好世界", "tone": true },
        "output": { "pinyin": "nǐ hǎo shì jiè" },
        "status": "Success",
        "error": null,
        "duration_ms": 123
      }
    ],
    "environment_info": {},
    "metadata": {}
  },
  "duration_ms": 123
}
```

### 5.4 WebSocket Support

#### WebSocket Connection
```
ws://localhost:8000/ws/mcp
```

#### Message Format
- **Request Messages**: Same format as `/mcp/call` endpoint requests
- **Response Messages**: Same format as `/mcp/call` endpoint responses

#### WebSocket Example (JavaScript)
```javascript
// Connect to WebSocket
const socket = new WebSocket('ws://localhost:8000/ws/mcp');

socket.onopen = () => {
  console.log('WebSocket connected');
  
  // Send MCP request
  const request = {
    "id": "req-12345",
    "component_type": "Tool",
    "component_name": "text.convert_chinese",
    "method": "run",
    "params": {
      "text": "你好世界",
      "mode": "s2t"
    },
    "context": {
      "id": "ctx-12345",
      "parent_id": null,
      "model_state": {},
      "execution_history": [],
      "environment_info": {},
      "metadata": {}
    },
    "service_context": {
      "caller_id": "js-client",
      "caller_type": "User",
      "permission_level": "Standard",
      "extra": {}
    }
  };
  
  socket.send(JSON.stringify(request));
};

socket.onmessage = (event) => {
  const response = JSON.parse(event.data);
  console.log('WebSocket response:', response);
};

socket.onerror = (error) => {
  console.error('WebSocket error:', error);
};

socket.onclose = () => {
  console.log('WebSocket connection closed');
};
```

### 5.5 MCP Client Examples

#### Using curl
```bash
curl -X POST http://localhost:8000/mcp/call \
  -H "Content-Type: application/json" \
  -d '{
    "id": "req-123",
    "component_type": "Tool",
    "component_name": "text.ac_automaton",
    "method": "run",
    "params": {
      "action": "match",
      "patterns": ["hello", "world"],
      "texts": ["hello world", "goodbye world"]
    },
    "context": {
      "id": "ctx-123",
      "parent_id": null,
      "model_state": {},
      "execution_history": [],
      "environment_info": {},
      "metadata": {}
    },
    "service_context": {
      "caller_id": "curl-client",
      "caller_type": "User",
      "permission_level": "Standard",
      "extra": {}
    }
  }'
```

#### Using Python
```python
import requests
import json

url = "http://localhost:8000/mcp/call"
headers = {"Content-Type": "application/json"}

# Example: Chinese text conversion
request_data = {
    "id": "req-456",
    "component_type": "Tool",
    "component_name": "text.convert_chinese",
    "method": "run",
    "params": {
        "text": "你好世界",
        "mode": "s2t"
    },
    "context": {
        "id": "ctx-456",
        "parent_id": None,
        "model_state": {},
        "execution_history": [],
        "environment_info": {},
        "metadata": {}
    },
    "service_context": {
        "caller_id": "python-client",
        "caller_type": "User",
        "permission_level": "Standard",
        "extra": {}
    }
}

response = requests.post(url, headers=headers, data=json.dumps(request_data))
result = response.json()
print(f"Converted text: {result['data']['converted']}")
```

#### Using Node.js
```javascript
const axios = require('axios');

async function callMcpTool() {
  const url = 'http://localhost:8000/mcp/call';
  
  const requestData = {
    id: 'req-789',
    component_type: 'Tool',
    component_name: 'file.duplicates',
    method: 'run',
    params: {
      directories: ['/home/user/documents'],
      min_size: 1024
    },
    context: {
      id: 'ctx-789',
      parent_id: null,
      model_state: {},
      execution_history: [],
      environment_info: {},
      metadata: {}
    },
    service_context: {
      caller_id: 'nodejs-client',
      caller_type: 'User',
      permission_level: 'Standard',
      extra: {}
    }
  };
  
  try {
    const response = await axios.post(url, requestData, {
      headers: { 'Content-Type': 'application/json' }
    });
    
    console.log('Duplicate files found:', response.data.data);
  } catch (error) {
    console.error('Error calling MCP tool:', error.response?.data || error.message);
  }
}

callMcpTool();
```

## 6. Plugin Management

Rust Toolbox supports extending functionality through external plugins with both executable and WebAssembly formats.

### 6.1 Plugin Installation

#### Installing Executable Plugins
1. Obtain the plugin executable file (e.g., `rt-plugin-custom.exe`)
2. Create a `plugins` folder in the same directory as `rt-cli` or `rt-gui`
3. Place the plugin executable in the `plugins` folder
4. Restart `rt-cli` or `rt-gui` - tools will automatically scan and load plugins prefixed with `rt-plugin-`

#### Installing WebAssembly Plugins
1. Obtain the plugin WASM file (e.g., `rt-plugin-custom.wasm`)
2. Place the WASM file in the `plugins` folder
3. Ensure the plugin follows the WASI interface specification
4. Restart the application to load the new plugin

### 6.2 Plugin Discovery and Loading

The plugin system automatically discovers plugins by:
- Scanning the `plugins/` directory for files prefixed with `rt-plugin-`
- Supporting both executable (.exe, no extension) and WebAssembly (.wasm) formats
- Loading plugin metadata through the `spec` command
- Registering tools provided by each plugin

### 6.3 Plugin Verification

#### List All Tools
```bash
cargo run --bin rt-cli -- list
```

This command shows all available tools including:
- Tool name and display name
- Tool description
- Tool type (core, plugin)
- MCP support status

#### Test Plugin Functionality
```bash
# Test a plugin tool
cargo run --bin rt-cli -- run text.pinyin --input '{"text": "测试", "tone": true}'
```

### 6.4 Multi-Tool Plugins

Some plugins provide multiple tools (like the Czkawka plugin):
- `file.duplicates` - Find duplicate files
- `file.similar_images` - Find similar images
- `file.empty_directories` - Find empty directories
- `file.temporary_files` - Find temporary files
- `file.broken_symlinks` - Find broken symbolic links

### 6.5 Plugin Configuration

#### Plugin Metadata
Each plugin exposes metadata through the `spec` command:
```bash
./plugins/rt-plugin-pinyin.exe spec --locale en
```

#### Plugin Capabilities
- **Input/Output Schemas**: JSON Schema definitions for validation
- **Internationalization**: Multi-language support (English, Chinese)
- **MCP Support**: Optional Model Context Protocol integration
- **Error Handling**: Standardized error reporting

## 7. Troubleshooting

### 7.1 Common Issues

#### Tool Not Found
**Problem**: Tool not appearing in the list or "Tool not found" error
**Solutions**:
- Verify plugin files are in the `plugins/` directory
- Check plugin file naming (must start with `rt-plugin-`)
- Ensure plugin executable has proper permissions
- Restart the application after adding plugins

#### Plugin Loading Errors
**Problem**: Plugin fails to load or crashes
**Solutions**:
- Check plugin compatibility with current rt-box version
- Verify plugin dependencies are installed
- Review error logs for specific error messages
- Test plugin independently using `plugin spec` command

#### MCP Server Connection Issues
**Problem**: Cannot connect to MCP server or API calls fail
**Solutions**:
- Verify server is running: `curl http://localhost:8000/health`
- Check firewall settings and port availability
- Ensure correct server address and port configuration
- Review server logs for error details

#### Workflow Execution Failures
**Problem**: Workflow fails to execute or nodes fail
**Solutions**:
- Validate workflow JSON syntax and structure
- Check tool input parameters match expected schemas
- Verify all referenced tools are available
- Review node dependencies and execution order
- Check for circular dependencies in workflow graph

#### Performance Issues
**Problem**: Slow tool execution or high memory usage
**Solutions**:
- Enable parallel execution for independent workflow nodes
- Use appropriate batch sizes for bulk operations
- Monitor system resources during execution
- Consider breaking large workflows into smaller components

### 7.2 Debugging Tools

#### Verbose Logging
Enable detailed logging for troubleshooting:
```bash
RUST_LOG=debug cargo run --bin rt-cli -- list
```

#### Tool Schema Validation
Verify tool input against schema:
```bash
cargo run --bin rt-cli -- run text.pinyin --input '{"invalid": "input"}' --validate
```

#### Plugin Testing
Test plugin independently:
```bash
./plugins/rt-plugin-pinyin.exe spec
./plugins/rt-plugin-pinyin.exe run < input.json
```

### 7.3 Error Codes and Messages

#### Common Error Types
- `InvalidInput`: Input parameters don't match tool schema
- `ToolFailure`: Tool execution failed with specific error
- `PluginError`: Plugin communication or execution error
- `ConfigError`: Configuration file or parameter error
- `NetworkError`: MCP server or network communication error

#### Error Resolution Steps
1. Read the complete error message and error code
2. Check input parameters against tool documentation
3. Verify tool and plugin availability
4. Review system logs for additional context
5. Consult plugin-specific documentation

### 7.4 Getting Help

#### Documentation Resources
- [Plugin Development Guide](PLUGIN_GUIDE.md) - Plugin development standards
- [Architecture Design](ARCHITECTURE_DESIGN.md) - System architecture details
- [Design Document](DESIGN.md) - Overall design principles

#### Community Support
- Check existing issues in the project repository
- Review plugin-specific documentation and examples
- Test with minimal reproduction cases
- Provide detailed error logs and system information when reporting issues
