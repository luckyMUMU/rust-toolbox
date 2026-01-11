# src/interfaces/tui/widgets/ - TUI Widget Components

## OVERVIEW
Specialized widgets for the TUI interface, each handling a specific view or functionality.

## WIDGETS

### ExecutionMonitor
**File**: `execution_monitor.rs`  
**Purpose**: Real-time workflow execution tracking  
**Features**:
- Progress bars for active nodes
- Node status indicators (running, completed, failed)
- Error display with context
- Real-time updates via event system

### LogViewer
**File**: `log_viewer.rs`  
**Purpose**: Audit log viewing and filtering  
**Features**:
- Multi-level filtering (info, warn, error, critical)
- Search functionality
- Auto-scroll with pause capability
- Color-coded severity levels

### PluginManager
**File**: `plugin_manager.rs`  
**Purpose**: Plugin lifecycle management UI  
**Features**:
- Plugin listing and status
- Load/unload operations
- Configuration editing
- Resource monitoring

### SyncStatus
**File**: `sync_status.rs`  
**Purpose**: Synchronization state display  
**Features**:
- Real-time sync indicators
- Conflict resolution UI
- Last sync timestamp
- Connection status

### SystemStatus
**File**: `system_status.rs`  
**Purpose**: System health monitoring  
**Features**:
- CPU/memory/disk usage charts
- Network status
- Load averages
- Health score calculation

### ToolManager
**File**: `tool_manager.rs`  
**Purpose**: Tool registry management  
**Features**:
- Tool listing with versions
- Execute tools interactively
- Parameter input forms
- Result display

### WorkflowList
**File**: `workflow_list.rs`  
**Purpose**: Workflow browsing and execution  
**Features**:
- List available workflows
- Filter and search
- Execute with parameters
- Status tracking

## WIDGET ARCHITECTURE

All widgets follow the same pattern:
```rust
pub struct WidgetName {
    state: WidgetState,
    // ... fields
}

impl WidgetName {
    pub fn render(&self, frame: &mut Frame, area: Rect);
    pub fn handle_event(&mut self, event: Event) -> ActionResult;
    pub fn update(&mut self, state: &AppState);
}
```

## INTEGRATION

Widgets are integrated into the main TUI layout:
- **Left sidebar**: Navigation (WorkflowList, ToolManager)
- **Center**: Active widget (ExecutionMonitor, LogViewer, etc.)
- **Bottom**: Status bar (SystemStatus, SyncStatus)

## THEMING

All widgets respect the global theme configuration:
- Colors from `theme.rs`
- Styles for active/inactive states
- Responsive layout constraints

## USAGE

Widgets are instantiated in `app.rs` and managed by the main application loop:
```rust
let mut workflow_list = WorkflowList::new();
let mut execution_monitor = ExecutionMonitor::new();
// ... other widgets

loop {
    // Event handling
    match event {
        Event::Key(key) => {
            let action = action_processor.process(key);
            match action {
                Action::Select => workflow_list.select(),
                Action::Execute => execution_monitor.start(),
                // ...
            }
        }
        Event::Tick => {
            execution_monitor.update(&app_state);
        }
    }
    
    // Rendering
    terminal.draw(|frame| {
        let layout = Layout::default().split(frame.size());
        workflow_list.render(frame, layout[0]);
        execution_monitor.render(frame, layout[1]);
        // ...
    })?;
}
```
