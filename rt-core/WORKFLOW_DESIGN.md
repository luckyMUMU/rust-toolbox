# Workflow Engine Design Document

## 1. System Architecture Design

A lightweight workflow engine based on Tokio async runtime, designed to implement tool orchestration and automated execution with comprehensive support for multi-tool plugins and Model Context Protocol (MCP).

### 1.1 Architecture Diagram

```mermaid
graph TD
    subgraph Frontend [Frontend Layer]
        CLI[rt-cli]
        GUI[rt-gui]
        API[REST API]
        WS[WebSocket]
    end

    subgraph Core [rt-core Workflow Engine]
        Engine[Workflow Engine]
        State[Execution State Manager]
        Scheduler[Task Scheduler (Tokio)]
        Context[Data Context Manager]
        McpMgr[MCP Context Manager]
    end

    subgraph Tools [Tool Layer]
        Registry[Tool Registry]
        Native[Native Tools]
        ProcessPlugin[Process Plugins]
        WasmPlugin[WASM Plugins]
        MultiTool[Multi-Tool Plugins]
    end

    subgraph MCP [MCP Integration]
        McpContext[MCP Context]
        McpWorkflow[MCP Workflow]
        McpNode[MCP Nodes]
    end

    CLI -->|Command| Engine
    GUI -->|Event| Engine
    API -->|HTTP| Engine
    WS -->|WebSocket| Engine
    
    Engine -->|Schedule| Scheduler
    Engine -->|Query| Registry
    Engine -->|MCP Context| McpMgr
    
    Scheduler -->|Execute| Native
    Scheduler -->|Execute| ProcessPlugin
    Scheduler -->|Execute| WasmPlugin
    Scheduler -->|Execute| MultiTool
    
    Native -->|Result| Context
    ProcessPlugin -->|Result| Context
    WasmPlugin -->|Result| Context
    MultiTool -->|Result| Context
    
    Context -->|Data Flow| Scheduler
    McpMgr -->|Context| McpContext
    McpContext -->|Workflow| McpWorkflow
    McpWorkflow -->|Nodes| McpNode
    
    State -->|Monitor| GUI
    State -->|Status| API
```

### 1.2 Core Modules
1. **Workflow Engine**: Responsible for parsing workflow definitions and managing lifecycle (start, pause, stop)
2. **Task Scheduler**: Tokio-based concurrent scheduler for dependency analysis and task dispatch
3. **Data Context Manager**: Handles data passing between nodes with JSON Path variable substitution
4. **Execution State Manager**: Maintains runtime state and provides monitoring interfaces
5. **MCP Context Manager**: Manages Model Context Protocol contexts for enhanced tool interaction
6. **Multi-Tool Plugin Support**: Handles plugins that provide multiple tools through array-based metadata

## 2. Key Data Structures

### 2.1 Workflow Definition
Core workflow structures supporting both traditional and MCP-enhanced workflows:

```rust
/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

/// Workflow node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub tool_name: String, // Reference to registered tool (e.g., "file.move_folder", "file.duplicates")
    pub label: Option<String>,
    pub input_mappings: HashMap<String, String>, // Input field -> expression (e.g., "{{ node1.output.path }}")
    pub static_inputs: Value, // Static configuration values
}

/// Node connection (dependency relationship)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub from: String, // Node ID
    pub to: String,   // Node ID
}
```

### 2.2 Runtime State
Enhanced runtime state with MCP context support:

```rust
/// Execution instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String, // UUID
    pub def: WorkflowDefinition,
    pub status: WorkflowStatus,
    pub node_states: HashMap<String, NodeExecutionState>,
    pub context: HashMap<String, Value>, // Store all node outputs
    pub mcp_context: Option<McpContext>, // MCP context for enhanced tool interaction
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionState {
    pub status: NodeStatus,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}
```

### 2.3 MCP Workflow Extensions
Enhanced workflow structures for MCP support:

```rust
/// MCP Workflow extending existing WorkflowDefinition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWorkflow {
    /// Base workflow definition
    pub base: WorkflowDefinition,
    
    /// MCP workflow configuration
    pub mcp_config: McpWorkflowConfig,
    
    /// MCP node list
    pub mcp_nodes: Vec<McpNode>,
    
    /// Context initialization configuration
    pub context_initialization: ContextInitialization,
    
    /// Global context update rules
    pub global_context_updates: Vec<GlobalContextUpdate>,
}

/// MCP Node extending existing WorkflowNode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNode {
    /// Base workflow node
    pub base: WorkflowNode,
    
    /// MCP node configuration
    pub mcp_config: McpNodeConfig,
    
    /// Context mapping rules
    pub context_mappings: Vec<ContextMapping>,
    
    /// Node output context update rules
    pub output_context_updates: Vec<ContextUpdateRule>,
}
```

## 3. Interface Specifications

### 3.1 Engine Interface
Enhanced workflow engine interface with MCP support:

```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// Validate workflow definition
    async fn validate(&self, def: &WorkflowDefinition) -> Result<()>;

    /// Start workflow execution
    async fn start_workflow(&self, def: WorkflowDefinition) -> Result<String>;

    /// Get workflow status
    async fn get_status(&self, instance_id: &str) -> Result<WorkflowInstance>;

    /// Pause workflow execution
    async fn pause_workflow(&self, instance_id: &str) -> Result<()>;
    
    /// Stop workflow execution
    async fn stop_workflow(&self, instance_id: &str) -> Result<()>;
    
    // Future: Get execution logs
    // async fn get_logs(&self, instance_id: &str) -> Result<Vec<LogEntry>>;
}
```

### 3.2 Data Passing Specification
Enhanced data passing with MCP context support:

- **Reference Syntax**: `{{ node_id.output.json_path }}`
- **Resolution Logic**:
  1. Before node execution, engine resolves `input_mappings`
  2. Look up corresponding `node_id` output from `context`
  3. Extract value using JSON Path
  4. Inject extracted value into tool `input`
  5. For MCP-enabled tools, pass enhanced context

### 3.3 Tool Integration
The workflow engine integrates with tools through the unified Tool trait:

```rust
// Tool execution with MCP context support
if tool.mcp_supported() && mcp_context_opt.is_some() {
    // Use MCP context execution
    let mcp_request = McpRequest::new_tool_call(
        node.tool_name.clone(),
        input.clone(),
        mcp_context,
        service_context
    );
    let mcp_response = tool.run_with_context(mcp_request).await;
    // Handle response and update context
} else {
    // Standard execution mode
    let output = tool.run(input.clone()).await;
}
```

## 4. Multi-Tool Plugin Development Guide

### 4.1 Plugin Integration Principles
The workflow engine interacts with tools through the `rt-core::Tool` trait. Plugins only need to comply with the standard plugin protocol (JSON Input/Output) to be called by the workflow engine, **requiring no additional modifications**.

### 4.2 MCP Support Integration
The workflow engine supports Model Context Protocol (MCP), allowing plugins and tools to interact through standardized protocols. Plugins only need to implement the `rt-core::Tool` trait and follow standard protocols to seamlessly integrate into workflows.

### 4.3 Multi-Tool Plugin Support

#### 4.3.1 Overview
The plugin system supports single plugins providing multiple tools through outputting tool metadata lists via the `spec` command. This enables better organization of related functionality and reduces plugin management overhead.

#### 4.3.2 Output Formats
Plugin `spec` command supports two output formats:
- **Single Tool** (backward compatible): Single JSON object
- **Multiple Tools**: JSON array containing multiple tool metadata objects

#### 4.3.3 Tool Metadata Array Format
```json
[
  {
    "name": "file.duplicates",
    "display_name": { "en": "Duplicate Files", "zh": "重复文件" },
    "description": { "en": "Find duplicate files", "zh": "查找重复文件" },
    "user_guide": { "en": "Find and manage duplicate files", "zh": "查找和管理重复文件" },
    "input_schema": {
      "type": "object",
      "properties": {
        "directory": { "type": "string", "description": "Directory to scan" },
        "min_size": { "type": "integer", "description": "Minimum file size" }
      }
    },
    "output_schema": {
      "type": "object",
      "properties": {
        "duplicates": { "type": "array", "items": { "type": "string" } },
        "total_size": { "type": "integer" }
      }
    },
    "mcp_supported": true,
    "mcp_capabilities": {
      "context_aware": true,
      "batch_processing": true
    },
    "requires_full_context": false,
    "context_validation_rules": {}
  },
  {
    "name": "file.similar_images",
    "display_name": { "en": "Similar Images", "zh": "相似图片" },
    "description": { "en": "Find similar images", "zh": "查找相似图片" },
    "user_guide": { "en": "Find visually similar images", "zh": "查找视觉上相似的图片" },
    "input_schema": {
      "type": "object",
      "properties": {
        "directory": { "type": "string", "description": "Directory to scan" },
        "similarity_threshold": { "type": "number", "description": "Similarity threshold" }
      }
    },
    "output_schema": {
      "type": "object",
      "properties": {
        "similar_groups": { "type": "array" },
        "total_groups": { "type": "integer" }
      }
    },
    "mcp_supported": true,
    "mcp_capabilities": {
      "context_aware": true,
      "image_processing": true
    },
    "requires_full_context": false,
    "context_validation_rules": {}
  }
]
```

### 4.4 Plugin Loading and Registration
The workflow engine handles multi-tool plugins through the enhanced plugin manager:

```rust
// Plugin loading process
match ProcessPlugin::extract_all_metadata(&path).await {
    Ok(metadata_list) => {
        // Create plugin instance for each tool metadata
        for metadata in metadata_list {
            let plugin = ProcessPlugin::from_metadata(metadata, path.clone());
            info!("Loaded plugin: {}, MCP supported: {}", 
                  plugin.name(), plugin.mcp_supported());
            plugins.push(Box::new(plugin));
        }
    },
    Err(e) => {
        warn!("Failed to load Process plugin {:?}: {}", path, e);
    }
}
```

### 4.5 Workflow Integration Examples

#### 4.5.1 Single Tool Reference
```yaml
# Workflow definition using single tool
nodes:
  - id: "scan_duplicates"
    tool_name: "file.duplicates"
    static_inputs:
      directory: "/home/user/documents"
      min_size: 1024
    input_mappings: {}
```

#### 4.5.2 Multi-Tool Workflow
```yaml
# Workflow using multiple tools from same plugin
nodes:
  - id: "find_duplicates"
    tool_name: "file.duplicates"
    static_inputs:
      directory: "/home/user/photos"
      min_size: 10240
    
  - id: "find_similar"
    tool_name: "file.similar_images"
    static_inputs:
      similarity_threshold: 0.8
    input_mappings:
      directory: "{{ find_duplicates.output.directory }}"

edges:
  - from: "find_duplicates"
    to: "find_similar"
```

#### 4.5.3 Data Flow Example
Multi-tool plugin output can be referenced by subsequent nodes:

```json
// Output from file.duplicates
{
  "duplicates": [
    ["/path/file1.jpg", "/path/file1_copy.jpg"],
    ["/path/file2.png", "/path/file2_backup.png"]
  ],
  "total_size": 2048576,
  "directory": "/home/user/photos"
}

// Referenced in next node: {{ find_duplicates.output.directory }}
```

### 4.6 Best Practices for Multi-Tool Plugins

#### 4.6.1 Tool Organization
1. **Functional Grouping**: Group related tools in the same plugin (e.g., all file operations)
2. **Consistent Naming**: Use `category.tool` format (e.g., `file.duplicates`, `file.similar_images`)
3. **Shared Resources**: Leverage shared code and resources across tools in the same plugin
4. **Version Consistency**: Maintain consistent versioning across all tools in a plugin

#### 4.6.2 Output Standardization
1. **Consistent Structure**: Maintain consistent output structure across related tools
2. **JSON Path Friendly**: Design outputs to be easily referenced via JSON Path
3. **Error Handling**: Standardize error reporting across all tools
4. **Metadata Inclusion**: Include relevant metadata in outputs for workflow context

#### 4.6.3 MCP Integration
1. **Context Awareness**: Design tools to leverage MCP context when available
2. **Context Updates**: Update MCP context with relevant information for downstream tools
3. **Capability Declaration**: Clearly declare MCP capabilities in metadata
4. **Validation Rules**: Define appropriate context validation rules

### 4.7 Plugin Development Workflow

#### 4.7.1 Development Steps
1. **Design Tool Interface**: Define input/output schemas for each tool
2. **Implement Tool Logic**: Implement the core functionality for each tool
3. **Create Metadata**: Generate comprehensive metadata for each tool
4. **Test Integration**: Test tools individually and in workflow contexts
5. **Document Usage**: Provide clear documentation and examples

#### 4.7.2 Testing Strategy
1. **Unit Testing**: Test each tool individually
2. **Integration Testing**: Test tools within workflow contexts
3. **MCP Testing**: Test MCP context handling if supported
4. **Performance Testing**: Ensure acceptable performance for workflow execution

### 4.8 Migration from Single-Tool Plugins

#### 4.8.1 Backward Compatibility
- Single-tool plugins continue to work without modification
- Existing workflows using single-tool plugins remain functional
- Gradual migration path available for consolidating related tools

#### 4.8.2 Migration Benefits
- **Reduced Plugin Count**: Fewer plugin files to manage
- **Shared Resources**: Better resource utilization across related tools
- **Consistent Updates**: Easier to maintain and update related functionality
- **Better Organization**: Logical grouping of related tools

## 5. User Interface Design

### 5.1 CLI Interface
Enhanced CLI with multi-tool plugin support:

```bash
# List available tools (including multi-tool plugins)
$ rt-cli list
Available Tools:
  Native Tools:
    - file.move_folder: Move/rename folders with collision handling
    - text.ac_automaton: Aho-Corasick pattern matching
    - text.convert_chinese: Traditional/Simplified Chinese conversion
  
  Plugin Tools:
    - file.duplicates: Find duplicate files (rt-plugin-czkawka)
    - file.similar_images: Find similar images (rt-plugin-czkawka)
    - file.empty_dirs: Find empty directories (rt-plugin-czkawka)
    - text.pinyin: Chinese to Pinyin conversion (rt-plugin-pinyin)
    - media.ytdlp: Video/audio downloading (rt-plugin-ytdlp)

# Run workflow with multi-tool plugins
$ rt-cli workflow run ./cleanup_workflow.json
[INFO] Workflow 'File Cleanup & Organization' started (ID: 550e8400...)
[INFO] Step 1: 'Find Duplicates' (file.duplicates) ... RUNNING
[INFO] Step 1: 'Find Duplicates' (file.duplicates) ... DONE (2.3s)
[INFO] Step 2: 'Find Similar Images' (file.similar_images) ... RUNNING
[INFO] Step 2: 'Find Similar Images' (file.similar_images) ... DONE (5.7s)
[INFO] Step 3: 'Clean Empty Dirs' (file.empty_dirs) ... RUNNING
[INFO] Step 3: 'Clean Empty Dirs' (file.empty_dirs) ... DONE (0.8s)
[INFO] Workflow Completed Successfully.

# Check workflow status with detailed node information
$ rt-cli workflow status 550e8400...
Workflow: File Cleanup & Organization
Status: Completed
Duration: 8.9s
MCP Context: Enabled

Nodes:
  - Find Duplicates (file.duplicates): Success
    Output: 15 duplicate groups found, 2.1GB total size
    MCP Context: Updated with file analysis results
  
  - Find Similar Images (file.similar_images): Success  
    Output: 8 similar image groups found
    MCP Context: Enhanced with image similarity data
    
  - Clean Empty Dirs (file.empty_dirs): Success
    Output: 23 empty directories removed
    MCP Context: Final cleanup summary
```

### 5.2 GUI Interface (rt-gui)
**Enhanced multi-tab design with multi-tool plugin support**:

#### **Tab 1: Workflow Designer (Canvas)**
- **Left Panel**: Enhanced Toolbox
  - **Native Tools Section**: Built-in tools organized by category
  - **Plugin Tools Section**: Plugin tools grouped by plugin with expansion
    - `rt-plugin-czkawka` (3 tools)
      - `file.duplicates`: Find duplicate files
      - `file.similar_images`: Find similar images  
      - `file.empty_dirs`: Find empty directories
    - `rt-plugin-pinyin` (1 tool)
      - `text.pinyin`: Chinese to Pinyin conversion
    - `rt-plugin-ytdlp` (1 tool)
      - `media.ytdlp`: Video/audio downloading
  - **Search/Filter**: Quick tool discovery across all sources
  - **MCP Indicators**: Visual indicators for MCP-supported tools

- **Center Panel**: Infinite Canvas
  - **Drag & Drop**: Create nodes from toolbox
  - **Connection Lines**: Define dependencies between nodes
  - **Node Visualization**: Enhanced node display showing tool source (native/plugin)
  - **MCP Context Flow**: Visual representation of MCP context flow

- **Right Panel**: Properties Panel
  - **Node Configuration**: Configure `static_inputs` and `input_mappings`
  - **MCP Settings**: Configure MCP-specific options for supported tools
  - **Schema Validation**: Real-time input validation against tool schemas
  - **Context Mapping**: Visual MCP context mapping interface

- **Top Toolbar**: Enhanced Controls
  - **Run/Pause/Stop**: Workflow execution controls
  - **Save/Load**: Workflow persistence
  - **Validate**: Pre-execution validation
  - **MCP Toggle**: Enable/disable MCP context for workflow

#### **Tab 2: Execution Monitor**
- **Left Panel**: Enhanced History
  - **Workflow List**: Historical executions with filtering
  - **Plugin Usage Stats**: Statistics on plugin tool usage
  - **MCP Context History**: MCP context evolution tracking

- **Center Panel**: Real-time DAG Visualization
  - **Node Status Colors**:
    - Gray: Pending
    - Blue: Running (with progress animation)
    - Green: Success
    - Red: Failed
    - Orange: MCP Context Processing
  - **Plugin Tool Indicators**: Visual distinction for plugin vs native tools
  - **Context Flow Lines**: MCP context propagation visualization
  - **Performance Metrics**: Execution time and resource usage per node

- **Bottom Panel**: Enhanced Log Console
  - **Multi-level Logging**: Debug, Info, Warn, Error levels
  - **Plugin Output**: Dedicated plugin execution logs
  - **MCP Context Logs**: Context creation, updates, and propagation
  - **Performance Metrics**: Real-time performance monitoring

- **Node Details Sidebar**: Enhanced Information Panel
  - **Input/Output JSON**: Formatted JSON with syntax highlighting
  - **MCP Context**: Current and historical context state
  - **Plugin Information**: Plugin metadata and version info
  - **Execution Metrics**: Detailed timing and resource usage
  - **Error Details**: Comprehensive error information with stack traces

### 5.3 Extensibility Design

#### 5.3.1 Architecture Extensibility
- **API Future-Proofing**: `WorkflowEngine` trait designed for async operation, enabling future gRPC client replacement for distributed execution
- **Storage Flexibility**: Current in-memory storage can be extended to SQLite/Redis persistence
- **Plugin Architecture**: Modular plugin system supports easy addition of new plugin types
- **MCP Evolution**: Designed to accommodate future MCP standard enhancements

#### 5.3.2 Multi-Tool Plugin Extensibility
- **Dynamic Discovery**: Automatic detection and loading of new multi-tool plugins
- **Hot Reloading**: Support for plugin updates without system restart
- **Plugin Versioning**: Version management for multi-tool plugins
- **Dependency Management**: Handle dependencies between tools within plugins

#### 5.3.3 UI Extensibility
- **Theme System**: Customizable themes and layouts
- **Plugin UI Extensions**: Allow plugins to provide custom UI components
- **Workflow Templates**: Pre-built workflow templates for common use cases
- **Export/Import**: Workflow sharing and collaboration features
## 6. MCP Context Management

### 6.1 Context Lifecycle
The workflow engine manages MCP contexts throughout the workflow execution:

```rust
// Context initialization
let mcp_context = Some(McpContext::new());

// Context updates during execution
if let Some(mcp_context) = updated_mcp_context {
    w.mcp_context = Some(mcp_context);
}
```

### 6.2 Context Propagation Strategies
- **Full Propagation**: Complete context passed to each node (default)
- **Incremental Propagation**: Only context changes passed between nodes
- **Selective Propagation**: Specific context elements based on node requirements
- **Rule-Based Propagation**: Custom rules determine context sharing

### 6.3 Context Validation
- **Schema Validation**: Validate context against tool-specific schemas
- **Type Checking**: Ensure context data types match expectations
- **Access Control**: Control which tools can access specific context elements
- **Sanitization**: Clean and normalize context data

## 7. Performance Optimization

### 7.1 Concurrent Execution
The workflow engine supports concurrent node execution:

```rust
// Concurrent node execution
let mut handles = vec![];
for node in runnable_nodes {
    let handle = tokio::spawn(async move {
        // Execute node asynchronously
    });
    handles.push(handle);
}
```

### 7.2 Resource Management
- **Memory Optimization**: Efficient context and state management
- **CPU Utilization**: Optimal task scheduling and parallel execution
- **I/O Optimization**: Async I/O operations for plugin communication
- **Cache Management**: Intelligent caching of plugin metadata and results

### 7.3 Scalability Considerations
- **Horizontal Scaling**: Support for distributed workflow execution
- **Load Balancing**: Distribute workflow execution across multiple nodes
- **Resource Limits**: Configurable limits for memory and CPU usage
- **Monitoring**: Real-time performance monitoring and alerting

## 8. Error Handling and Recovery

### 8.1 Error Types
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed(String),
}
```

### 8.2 Error Recovery Strategies
- **Retry Logic**: Configurable retry attempts for failed nodes
- **Fallback Nodes**: Alternative execution paths for critical failures
- **Partial Success**: Continue execution with non-critical node failures
- **Rollback Support**: Undo operations for failed workflows

### 8.3 Error Reporting
- **Detailed Logging**: Comprehensive error logging with context
- **User Notifications**: Clear error messages for end users
- **Debug Information**: Technical details for developers
- **Error Aggregation**: Collect and analyze error patterns

## 9. Security and Permissions

### 9.1 Plugin Security
- **Sandboxing**: Isolate plugin execution environments
- **Permission Model**: Control plugin access to system resources
- **Input Validation**: Validate all plugin inputs and outputs
- **Resource Limits**: Prevent resource exhaustion attacks

### 9.2 Workflow Security
- **Access Control**: Control who can create and execute workflows
- **Data Protection**: Protect sensitive data in workflow contexts
- **Audit Logging**: Track all workflow operations for security analysis
- **Encryption**: Encrypt sensitive workflow data at rest and in transit

### 9.3 MCP Security
- **Context Isolation**: Prevent unauthorized context access
- **Data Sanitization**: Clean context data to prevent injection attacks
- **Permission Validation**: Verify tool permissions for context access
- **Secure Communication**: Encrypted communication for MCP operations

## 10. Testing Strategy

### 10.1 Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_execution() {
        // Test basic workflow execution
    }
    
    #[tokio::test]
    async fn test_multi_tool_plugin_integration() {
        // Test multi-tool plugin integration
    }
    
    #[tokio::test]
    async fn test_mcp_context_propagation() {
        // Test MCP context propagation
    }
}
```

### 10.2 Integration Testing
- **End-to-End Workflows**: Test complete workflow scenarios
- **Plugin Integration**: Test plugin loading and execution
- **MCP Integration**: Test MCP context handling
- **Performance Testing**: Benchmark workflow execution performance

### 10.3 Property-Based Testing
- **Workflow Invariants**: Test workflow execution properties
- **Context Consistency**: Verify MCP context consistency
- **Plugin Behavior**: Test plugin behavior across various inputs
- **Error Handling**: Test error handling and recovery mechanisms

## 11. Monitoring and Observability

### 11.1 Metrics Collection
- **Execution Metrics**: Track workflow and node execution times
- **Resource Usage**: Monitor CPU, memory, and I/O usage
- **Plugin Performance**: Track plugin execution performance
- **Error Rates**: Monitor error rates and patterns

### 11.2 Logging Integration
- **Structured Logging**: Integration with rt-core logging system
- **Workflow Tracing**: Detailed tracing of workflow execution
- **Plugin Logging**: Capture plugin execution logs
- **MCP Context Logging**: Track context creation and updates

### 11.3 Health Monitoring
- **System Health**: Monitor overall system health
- **Plugin Health**: Track plugin availability and performance
- **Workflow Health**: Monitor workflow execution success rates
- **Resource Health**: Track resource utilization and limits

## 12. Configuration Management

### 12.1 Workflow Configuration
```yaml
# Example workflow configuration
workflow:
  id: "file-cleanup-workflow"
  name: "File Cleanup and Organization"
  description: "Automated file cleanup using multiple tools"
  
  mcp:
    enabled: true
    context_propagation: "full"
    validation_rules:
      strict_typing: true
      
  nodes:
    - id: "find_duplicates"
      tool_name: "file.duplicates"
      mcp_config:
        requires_context: true
        context_mappings:
          - source: "workflow.input.directory"
            target: "scan.directory"
      static_inputs:
        min_size: 1024
        
    - id: "find_similar"
      tool_name: "file.similar_images"
      mcp_config:
        requires_context: true
      input_mappings:
        directory: "{{ find_duplicates.output.directory }}"
        
  edges:
    - from: "find_duplicates"
      to: "find_similar"
```

### 12.2 Plugin Configuration
- **Plugin Discovery**: Configure plugin search paths
- **Plugin Loading**: Control plugin loading behavior
- **Resource Limits**: Set resource limits for plugin execution
- **Security Settings**: Configure plugin security policies

### 12.3 MCP Configuration
- **Context Settings**: Configure MCP context behavior
- **Validation Rules**: Set context validation requirements
- **Propagation Policies**: Define context propagation strategies
- **Security Policies**: Configure MCP security settings

## 13. Future Enhancements

### 13.1 Advanced Features
- **Visual Workflow Editor**: Drag-and-drop workflow creation
- **Workflow Templates**: Pre-built workflow templates
- **Conditional Execution**: Support for conditional node execution
- **Loop Support**: Support for iterative workflow execution
- **Sub-workflows**: Support for nested workflow execution

### 13.2 Distributed Execution
- **Cluster Support**: Execute workflows across multiple machines
- **Load Balancing**: Distribute workflow execution load
- **Fault Tolerance**: Handle node failures in distributed environments
- **State Synchronization**: Synchronize workflow state across nodes

### 13.3 Advanced MCP Features
- **Context Versioning**: Version control for MCP contexts
- **Context Branching**: Support for parallel context branches
- **Context Merging**: Merge contexts from parallel execution paths
- **Context Analytics**: Analyze context usage patterns

### 13.4 Integration Enhancements
- **External Systems**: Integration with external workflow systems
- **API Gateway**: RESTful API for workflow management
- **Event-Driven Execution**: Trigger workflows based on events
- **Webhook Support**: HTTP webhook integration for external triggers

## 14. Migration and Compatibility

### 14.1 Version Compatibility
- **Backward Compatibility**: Support for older workflow definitions
- **Migration Tools**: Automated migration for workflow updates
- **Plugin Compatibility**: Maintain compatibility with existing plugins
- **API Versioning**: Version control for workflow engine APIs

### 14.2 Data Migration
- **Workflow Migration**: Migrate existing workflows to new formats
- **Context Migration**: Migrate MCP contexts between versions
- **Plugin Migration**: Support plugin updates and migrations
- **Configuration Migration**: Migrate configuration settings

## 15. Documentation and Examples

### 15.1 Workflow Examples
Complete examples demonstrating various workflow patterns:

#### File Processing Workflow
```json
{
  "id": "file-processing",
  "name": "File Processing Pipeline",
  "description": "Process files using multiple tools",
  "nodes": [
    {
      "id": "scan_duplicates",
      "tool_name": "file.duplicates",
      "static_inputs": {
        "directory": "/home/user/documents",
        "min_size": 1024
      }
    },
    {
      "id": "find_similar_images",
      "tool_name": "file.similar_images",
      "input_mappings": {
        "directory": "{{ scan_duplicates.output.directory }}"
      },
      "static_inputs": {
        "similarity_threshold": 0.8
      }
    }
  ],
  "edges": [
    {
      "from": "scan_duplicates",
      "to": "find_similar_images"
    }
  ]
}
```

#### Text Processing Workflow
```json
{
  "id": "text-processing",
  "name": "Text Processing Pipeline",
  "description": "Process text using multiple tools",
  "nodes": [
    {
      "id": "convert_chinese",
      "tool_name": "text.convert_chinese",
      "static_inputs": {
        "input_text": "繁體中文文本",
        "target": "simplified"
      }
    },
    {
      "id": "generate_pinyin",
      "tool_name": "text.pinyin",
      "input_mappings": {
        "text": "{{ convert_chinese.output.converted_text }}"
      }
    }
  ],
  "edges": [
    {
      "from": "convert_chinese",
      "to": "generate_pinyin"
    }
  ]
}
```

### 15.2 Best Practices Guide
- **Workflow Design**: Guidelines for effective workflow design
- **Performance Optimization**: Tips for optimizing workflow performance
- **Error Handling**: Best practices for error handling and recovery
- **Security**: Security considerations for workflow development

### 15.3 Troubleshooting Guide
- **Common Issues**: Solutions for common workflow problems
- **Debug Techniques**: Techniques for debugging workflow issues
- **Performance Issues**: Diagnosing and fixing performance problems
- **Plugin Issues**: Troubleshooting plugin-related problems

## 16. Conclusion

The rt-core workflow engine provides a robust, scalable, and extensible foundation for tool orchestration and automation. With comprehensive support for multi-tool plugins, MCP integration, and advanced workflow features, it enables users to create sophisticated automation workflows while maintaining simplicity and reliability.

The engine's modular architecture allows for easy extension and customization, while its comprehensive error handling and monitoring capabilities ensure reliable operation in production environments. The multi-tool plugin support significantly reduces plugin management overhead while providing better organization of related functionality.

Future enhancements will continue to expand the engine's capabilities while maintaining backward compatibility and ease of use, making it a powerful platform for automation and tool integration in the rt-box ecosystem.