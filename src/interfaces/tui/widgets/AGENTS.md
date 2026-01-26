# src/interfaces/tui/widgets/ - TUI Widget Components

## OVERVIEW
Specialized widgets for the TUI interface, each handling a specific view or functionality. 8 widgets with 3,553+ total lines.

## WIDGETS

### ExecutionMonitorWidget
**File**: `execution_monitor.rs`  
**Purpose**: Real-time workflow execution tracking  
**Features**:
- Progress bars for active nodes
- Node status indicators (running, completed, failed)
- Error display with context
- Real-time updates via event system
- Execution timeline visualization

### LogViewerWidget
**File**: `log_viewer.rs` (1,595 lines)  
**Purpose**: Audit log viewing and filtering  
**Features**:
- Multi-level filtering (info, warn, error, critical)
- Search functionality with regex support
- Auto-scroll with pause capability
- Color-coded severity levels
- Export logs to file
- Real-time log streaming

### PluginManagerWidget
**File**: `plugin_manager.rs` (3,553 lines)  
**Purpose**: Plugin lifecycle management UI  
**Features**:
- Plugin listing and status
- Load/unload operations
- Configuration editing
- Resource monitoring
- Plugin dependency visualization
- Error recovery UI

### SyncStatusWidget
**File**: `sync_status.rs`  
**Purpose**: Synchronization state display  
**Features**:
- Real-time sync indicators
- Conflict resolution UI
- Last sync timestamp
- Connection status
- Sync progress tracking

### SystemStatusWidget
**File**: `system_status.rs` (2,687 lines)  
**Purpose**: System health monitoring  
**Features**:
- CPU/memory/disk usage charts
- Network status and metrics
- Load averages
- Health score calculation
- Alert management
- Maintenance recommendations
- Performance optimization suggestions

### ToolManagerWidget
**File**: `tool_manager.rs`  
**Purpose**: Tool registry management  
**Features**:
- Tool listing with versions
- Execute tools interactively
- Parameter input forms
- Result display
- Tool dependency visualization
- Performance metrics

### WorkflowListWidget
**File**: `workflow_list.rs`  
**Purpose**: Workflow browsing and execution  
**Features**:
- List available workflows
- Filter and search
- Execute with parameters
- Status tracking
- Execution history
- Workflow templates

## WIDGET ARCHITECTURE

### Common Pattern
All widgets follow the same architecture:
```rust
pub struct WidgetName {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    size_constraints: SizeConstraints,
    // Widget-specific state
}

impl WidgetName {
    pub fn new(context: WidgetContext) -> Self { /* ... */ }
    
    pub async fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
    ) -> Result<(), WidgetError> { /* ... */ }
    
    pub async fn handle_event(
        &mut self,
        event: Event,
    ) -> Result<Option<Action>, WidgetError> { /* ... */ }
    
    pub async fn update(&mut self) -> Result<(), WidgetError> { /* ... */ }
    
    pub fn help_text(&self) -> Vec<(&str, &str)> { /* ... */ }
}
```

### Widget Context
**File**: `context.rs` (in parent directory)
```rust
pub struct WidgetContext {
    state: SharedAppState,
    event_sender: mpsc::UnboundedSender<Event>,
    action_sender: mpsc::UnboundedSender<Action>,
    capabilities: WidgetCapabilities,
}
```

### Widget Capabilities
```rust
pub struct WidgetCapabilities {
    pub can_navigate: bool,
    pub can_edit: bool,
    pub can_execute: bool,
    pub can_refresh: bool,
    pub can_export: bool,
}
```

## INTEGRATION

### Main TUI Layout
```
┌─────────────────────────────────────────────────────────┐
│                    Navigation Sidebar                    │
│  ┌─────────────────────────────────────────────────┐  │
│  │ WorkflowListWidget                             │  │
│  │ ToolManagerWidget                              │  │
│  │ PluginManagerWidget                            │  │
│  └─────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                    Active Widget                        │
│  ┌─────────────────────────────────────────────────┐  │
│  │ ExecutionMonitorWidget                         │  │
│  │ LogViewerWidget                                │  │
│  │ SystemStatusWidget                             │  │
│  └─────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                    Status Bar                           │
│  ┌─────────────────────────────────────────────────┐  │
│  │ SyncStatusWidget                               │  │
│  │ SystemStatus (compact)                         │  │
│  └─────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Widget Lifecycle
```rust
1. Initialization
   - Create widget with context
   - Load initial state
   - Set up event subscriptions

2. Event Loop
   - Receive events (key, mouse, resize, tick)
   - Handle events (return Action or None)
   - Update internal state
   - Trigger state changes

3. Rendering
   - Clear previous render
   - Calculate layout
   - Render widgets
   - Apply theme

4. Updates
   - Periodic updates (tick events)
   - State change notifications
   - Refresh data from shared state
```

## THEMING

### Theme Integration
All widgets respect the global theme from `theme.rs`:
```rust
pub struct Theme {
    pub colors: ColorPalette,
    pub styles: StyleMap,
    pub borders: BorderStyle,
}
```

### Widget-Specific Styles
- **Active widgets**: Highlighted border
- **Inactive widgets**: Dimmed appearance
- **Error states**: Red/orange colors
- **Success states**: Green colors
- **Warning states**: Yellow colors

### Responsive Design
- **Minimum size**: Widgets enforce minimum dimensions
- **Flexible layout**: Adapts to terminal size
- **Virtual scrolling**: Handles large datasets
- **Progressive disclosure**: Show/hide details

## EVENT HANDLING

### Event Types
**File**: `event.rs`
```rust
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
    Quit,
    FocusGained,
    FocusLost,
    Paste(String),
    Custom(String, serde_json::Value),
}
```

### Action System
**File**: `action.rs`
```rust
pub enum Action {
    // Navigation
    NavigateUp,
    NavigateDown,
    Select,
    GoBack,
    
    // Execution
    Execute,
    Pause,
    Resume,
    Stop,
    
    // UI
    Refresh,
    Help,
    ToggleView,
    
    // Custom
    Custom(String, serde_json::Value),
}
```

### Event Processing
```rust
impl WidgetName {
    async fn handle_event(&mut self, event: Event) -> Result<Option<Action>, WidgetError> {
        match event {
            Event::Key(key) => self.handle_key_event(key).await,
            Event::Tick => self.update().await.map(|_| None),
            _ => Ok(None),
        }
    }
}
```

## STATE MANAGEMENT

### Shared Application State
**File**: `state.rs`
```rust
pub struct SharedAppState {
    workflows: Arc<RwLock<Vec<WorkflowInfo>>>,
    executions: Arc<RwLock<Vec<ExecutionInfo>>>,
    tools: Arc<RwLock<Vec<ToolInfo>>>,
    plugins: Arc<RwLock<Vec<PluginInfo>>>,
    system_status: Arc<RwLock<SystemStatus>>,
    logs: Arc<RwLock<VecDeque<LogEntry>>>,
    connection_status: Arc<RwLock<ConnectionStatus>>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
}
```

### State Change Events
```rust
pub enum StateChangeEvent {
    WorkflowsUpdated,
    WorkflowStatusChanged { name: String, status: WorkflowStatus },
    ExecutionsUpdated,
    ExecutionStatusChanged { id: String, status: ExecutionStatus },
    ToolsUpdated,
    PluginsUpdated,
    SystemStatusUpdated,
    LogEntryAdded { entry: LogEntry },
    ConnectionStatusChanged { status: ConnectionStatus },
    MetricsUpdated,
}
```

### Widget Subscriptions
Widgets subscribe to relevant state changes:
```rust
impl WorkflowListWidget {
    fn subscribe_to_state_changes(&mut self) {
        self.state.subscribe(StateChangeEventType::Workflows);
        self.state.subscribe(StateChangeEventType::Executions);
    }
}
```

## RENDERING

### Rendering Pattern
```rust
impl WidgetName {
    async fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
    ) -> Result<(), WidgetError> {
        // 1. Calculate layout
        let layout = self.calculate_layout(area);
        
        // 2. Render main content
        self.render_content(frame, layout.main, theme)?;
        
        // 3. Render overlays (if any)
        if self.show_help {
            self.render_help_overlay(frame, area, theme)?;
        }
        
        // 4. Render status indicators
        self.render_status(frame, layout.status, theme)?;
        
        Ok(())
    }
}
```

### Layout Calculation
```rust
impl WidgetName {
    fn calculate_layout(&self, area: Rect) -> WidgetLayout {
        WidgetLayout {
            main: Rect::new(area.x, area.y, area.width, area.height - 2),
            status: Rect::new(area.x, area.y + area.height - 2, area.width, 2),
            help: Rect::new(area.x + 5, area.y + 5, area.width - 10, area.height - 10),
        }
    }
}
```

## KEY BINDINGS

### Global Bindings
- `↑/↓`: Navigate list
- `Enter`: Select/Execute
- `Esc`: Go back/Cancel
- `q`: Quit
- `r`: Refresh
- `h`: Help
- `Tab`: Switch view
- `/`: Search
- `Ctrl+C`: Interrupt

### Widget-Specific Bindings

**LogViewerWidget**:
- `f`: Filter by level
- `n`: Next search result
- `N`: Previous search result
- `e`: Export logs
- `a`: Toggle auto-scroll

**SystemStatusWidget**:
- `m`: Maintenance mode
- `c`: Clear alerts
- `d`: Disk cleanup
- `p`: Performance optimization

**WorkflowListWidget**:
- `x`: Execute workflow
- `s`: Show status
- `p`: Pause/resume
- `t`: Show template

**ToolManagerWidget**:
- `i`: Tool information
- `e`: Execute tool
- `v`: Validate parameters

**PluginManagerWidget**:
- `l`: Load plugin
- `u`: Unload plugin
- `r`: Reload plugin
- `c`: Configure plugin

## ERROR HANDLING

### WidgetError
**File**: `error.rs`
```rust
pub enum WidgetError {
    RenderError { message: String },
    InputError { message: String },
    DataError { message: String },
    ConfigError { message: String },
    TerminalError { message: String },
    WidgetError { widget_id: String, message: String },
    LayoutError { message: String },
    ThemeError { message: String },
    NetworkError { message: String },
    PermissionError { message: String },
}
```

### Error Recovery
```rust
impl WidgetName {
    async fn handle_error(&mut self, error: WidgetError) -> Result<(), WidgetError> {
        match error {
            WidgetError::RenderError { message } => {
                self.error_manager.report_error(error.clone());
                self.show_error_dialog(message);
                Ok(())
            }
            _ => Err(error),
        }
    }
}
```

## PERFORMANCE OPTIMIZATION

### Virtual Scrolling
For large datasets (e.g., logs with 10,000+ entries):
```rust
pub struct VirtualScroller {
    visible_range: Range<usize>,
    item_height: u16,
    total_items: usize,
}

impl VirtualScroller {
    fn render_visible_items(&self, items: &[LogEntry], area: Rect) {
        let start = self.visible_range.start;
        let end = min(self.visible_range.end, items.len());
        
        for (i, item) in items[start..end].iter().enumerate() {
            let y = area.y + (i as u16 * self.item_height);
            // Render only visible items
        }
    }
}
```

### Update Throttling
```rust
impl WidgetName {
    fn should_update(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.last_update) > Duration::from_millis(100) // 10 FPS
    }
}
```

### Memory Management
- **Lazy loading**: Load data on demand
- **Cache results**: Reuse computed values
- **Release resources**: Clean up on widget destruction
- **Batch updates**: Group state changes

## TESTING

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_widget_render() {
        let mut widget = WorkflowListWidget::new();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
        widget.render(&mut buffer);
        // Verify rendering
    }
    
    #[test]
    fn test_event_handling() {
        let mut widget = WorkflowListWidget::new();
        let event = Event::Key(KeyEvent::new(KeyCode::Enter));
        let action = widget.handle_event(event);
        assert_eq!(action, Some(Action::Select));
    }
}
```

### Integration Tests
**File**: `tests/tui_standalone_unit_tests.rs` (1,697 lines)

## USAGE EXAMPLES

### Widget Instantiation
```rust
use workflow_toolkit::interfaces::tui::widgets::{
    WorkflowListWidget, ExecutionMonitorWidget, LogViewerWidget,
    SystemStatusWidget, ToolManagerWidget, PluginManagerWidget,
};

let mut workflow_list = WorkflowListWidget::new(context.clone());
let mut execution_monitor = ExecutionMonitorWidget::new(context.clone());
let mut log_viewer = LogViewerWidget::new(context.clone());
let mut system_status = SystemStatusWidget::new(context.clone());
let mut tool_manager = ToolManagerWidget::new(context.clone());
let mut plugin_manager = PluginManagerWidget::new(context.clone());
```

### Widget Integration
```rust
// In main TUI loop
loop {
    // Event handling
    let event = event_rx.recv().await?;
    let action = match current_view {
        View::WorkflowList => workflow_list.handle_event(event).await?,
        View::ExecutionMonitor => execution_monitor.handle_event(event).await?,
        View::LogViewer => log_viewer.handle_event(event).await?,
        // ...
    };
    
    // State updates
    if let Some(action) = action {
        process_action(action).await;
    }
    
    // Rendering
    terminal.draw(|frame| {
        match current_view {
            View::WorkflowList => workflow_list.render(frame, area, &theme),
            View::ExecutionMonitor => execution_monitor.render(frame, area, &theme),
            View::LogViewer => log_viewer.render(frame, area, &theme),
            // ...
        }
    })?;
}
```

## WIDGET-SPECIFIC DETAILS

### WorkflowListWidget
- **Data source**: SharedAppState.workflows
- **Refresh interval**: 2 seconds
- **Max items displayed**: 50 (virtual scrolling for more)
- **Filtering**: By name, status, tags
- **Sorting**: By name, last execution, success rate

### ExecutionMonitorWidget
- **Data source**: SharedAppState.executions
- **Refresh interval**: 500ms (real-time)
- **Progress tracking**: Per-node progress bars
- **Error display**: Detailed error context with stack traces
- **Timeline**: Visual execution timeline

### LogViewerWidget
- **Data source**: SharedAppState.logs
- **Refresh interval**: 1 second (streaming)
- **Max entries**: 10,000 (virtual scrolling)
- **Filtering**: By level, timestamp, component
- **Search**: Regex support, case-insensitive
- **Export**: JSON, CSV, plain text

### SystemStatusWidget
- **Data source**: SharedAppState.system_status
- **Refresh interval**: 5 seconds
- **Metrics**: CPU, memory, disk, network
- **Alerts**: Configurable thresholds
- **Maintenance**: Automated recommendations

### ToolManagerWidget
- **Data source**: SharedAppState.tools
- **Refresh interval**: 10 seconds
- **Execution**: Interactive parameter input
- **Validation**: Real-time parameter validation
- **Performance**: Execution time tracking

### PluginManagerWidget
- **Data source**: SharedAppState.plugins
- **Refresh interval**: 30 seconds
- **Lifecycle**: Load/unload/reload
- **Configuration**: Edit plugin configs
- **Monitoring**: Resource usage per plugin

### SyncStatusWidget
- **Data source**: SharedAppState.connection_status
- **Refresh interval**: 1 second
- **Status**: Connected/Disconnected/Degraded
- **Conflicts**: Display sync conflicts
- **Resolution**: Interactive conflict resolution

## ARCHITECTURE DECISIONS

### Why Separate Widgets?
1. **Single Responsibility**: Each widget handles one view
2. **Reusability**: Widgets can be composed in different layouts
3. **Testability**: Individual widgets can be tested in isolation
4. **Maintainability**: Changes to one widget don't affect others

### Why Async Rendering?
1. **Non-blocking**: UI remains responsive during data fetch
2. **Real-time updates**: Widgets can update independently
3. **Performance**: Parallel rendering of multiple widgets
4. **Scalability**: Handles large datasets without freezing

### Why Event-Driven?
1. **Decoupling**: Widgets don't need to know about each other
2. **Flexibility**: Easy to add new event types
3. **Testability**: Events can be simulated for testing
4. **Extensibility**: Custom events for custom widgets

## BEST PRACTICES

### Widget Design
1. **Keep widgets small**: Focus on one responsibility
2. **Use virtual scrolling**: For large datasets
3. **Implement error boundaries**: Don't crash on errors
4. **Provide help text**: Document key bindings
5. **Respect theme**: Use theme colors and styles

### Performance
1. **Throttle updates**: Don't update more than 60 FPS
2. **Lazy load data**: Fetch only what's needed
3. **Cache computations**: Reuse expensive calculations
4. **Release resources**: Clean up on widget destruction

### User Experience
1. **Provide feedback**: Show loading states
2. **Handle errors gracefully**: Show error dialogs
3. **Support keyboard navigation**: Full keyboard support
4. **Include help**: Press 'h' for help
5. **Responsive design**: Adapt to terminal size

## SEE ALSO

- [TUI AGENTS.md](../AGENTS.md) - Terminal UI overview
- [Event System](../event.rs) - Event handling
- [State Management](../state.rs) - Shared state
- [Theme System](../theme.rs) - Styling
- [Layout System](../layout.rs) - Widget positioning
