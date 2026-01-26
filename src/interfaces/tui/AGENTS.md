# src/interfaces/tui/ - Terminal User Interface

## OVERVIEW
Ratatui-based TUI with event-driven architecture, action system, and reactive widgets. 25 files with 3,275+ lines in layout.rs alone.

## EVENT LOOP
**Event System**:
- **Events**: Key presses, mouse, resize, tick
- **EventLoop**: Manages event flow
- **EventResult**: Action results with state updates

**Action Processing**:
- **Actions**: User commands (Quit, Execute, Navigate, etc.)
- **ActionProcessor**: Translates events to actions
- **ActionResult**: Success/failure with updates

## ACTION SYSTEM
**Action Enum**:
- `Quit`, `Execute`, `Pause`, `Resume`
- `NavigateUp`, `NavigateDown`, `Select`
- `Refresh`, `Help`, `ToggleView`

**State Management**:
- Current view (Workflows, Tools, Logs, etc.)
- Selected items
- Execution status

## WIDGETS
**Path**: `src/interfaces/tui/widgets/` (8 widgets, 3,553+ lines in plugin_manager.rs)

### ExecutionMonitorWidget
- Real-time workflow execution tracking
- Progress bars for active nodes
- Node status indicators (running, completed, failed)
- Error display with context
- Execution timeline visualization

### LogViewerWidget (1,595 lines)
- Multi-level filtering (info, warn, error, critical)
- Search functionality with regex support
- Auto-scroll with pause capability
- Color-coded severity levels
- Export logs to file
- Real-time log streaming

### PluginManagerWidget (3,553 lines)
- Plugin listing and status
- Load/unload operations
- Configuration editing
- Resource monitoring
- Plugin dependency visualization
- Error recovery UI

### SyncStatusWidget
- Real-time sync indicators
- Conflict resolution UI
- Last sync timestamp
- Connection status
- Sync progress tracking

### SystemStatusWidget (2,687 lines)
- CPU/memory/disk usage charts
- Network status and metrics
- Load averages
- Health score calculation
- Alert management
- Maintenance recommendations
- Performance optimization suggestions

### ToolManagerWidget
- Tool listing with versions
- Execute tools interactively
- Parameter input forms
- Result display
- Tool dependency visualization
- Performance metrics

### WorkflowListWidget
- List available workflows
- Filter and search
- Execute with parameters
- Status tracking
- Execution history
- Workflow templates

## LAYOUTS
**File**: `layout.rs` (3,275 lines)  
**Purpose**: Layout management and responsive design

**Main Layout**: Split screen
- **Left**: Navigation sidebar (20% width)
- **Center**: Active widget (70% width)
- **Bottom**: Status bar (10% height)

**Responsive Design**:
- Minimum size enforcement
- Dynamic resizing
- Virtual scrolling for large datasets
- Progressive disclosure

**Layout Components**:
- `MainLayout`: Overall application layout
- `WidgetLayout`: Individual widget layouts
- `NavigationLayout`: Sidebar navigation
- `StatusLayout`: Status bar layout

## THEMING
**File**: `theme.rs` (1,385 lines)  
**Purpose**: Color schemes and styling

**Theme System**:
- Multiple color schemes (dark, light, auto)
- Custom color palettes
- Style definitions for all UI elements
- Responsive theme switching

**Color Palette**:
```rust
pub struct ColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub text: Color,
    pub text_dim: Color,
}
```

**Style Map**:
- Widget borders
- Text styles (bold, italic, underline)
- Status indicators
- Progress bars

## STATE MANAGEMENT
**File**: `state.rs`  
**Purpose**: Shared application state

**SharedAppState**:
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

**State Change Events**:
- Workflows updated
- Execution status changed
- Tools updated
- Plugins updated
- System status changed
- New log entry
- Connection status changed
- Metrics updated

## EVENT HANDLING
**File**: `event.rs`  
**Purpose**: Event processing and handling

**Event Types**:
- `Key(KeyEvent)`: Keyboard input
- `Mouse(MouseEvent)`: Mouse input
- `Resize(u16, u16)`: Terminal resize
- `Tick`: Periodic update
- `Quit`: Application quit
- `FocusGained/Lost`: Terminal focus
- `Paste(String)`: Text paste
- `Custom(String, Value)`: Custom events

**Event Processing**:
```rust
impl EventLoop {
    pub async fn run(&mut self) -> Result<Option<Action>, WidgetError> {
        let event = self.event_rx.recv().await?;
        let action = self.action_processor.process(event).await?;
        Ok(action)
    }
}
```

## NAVIGATION
**File**: `navigation.rs`  
**Purpose**: Navigation stack and modal management

**Navigation Stack**:
- Tracks view history
- Supports back navigation
- Modal dialog management
- View state preservation

**Navigation Types**:
- `UserInitiated`: User action
- `Programmatic`: Automatic navigation
- `BackNavigation`: Go back
- `ModalOpened`: Modal dialog
- `ErrorRecovery`: Error handling

**Modal System**:
- Message dialogs
- Confirmation dialogs
- Input dialogs
- Error dialogs
- Progress dialogs

## ERROR HANDLING
**File**: `error.rs` (1,284 lines)  
**Purpose**: TUI-specific error types and recovery

**TUI Error Types**:
- `RenderError`: Rendering failures
- `InputError`: Input processing errors
- `DataError`: Data synchronization issues
- `ConfigError`: Configuration problems
- `TerminalError`: Terminal operation errors
- `WidgetError`: Widget-specific errors
- `LayoutError`: Layout calculation errors
- `ThemeError`: Theme application errors
- `NetworkError`: Connection issues
- `PermissionError`: Access denied

**Error Recovery Strategies**:
- `Retry`: Retry with exponential backoff
- `UseDefault`: Use default values
- `Skip`: Skip operation
- `RestartComponent`: Restart widget
- `ShowDialog`: Show error dialog
- `Degrade`: Graceful degradation
- `Manual`: Manual intervention required

**Error Display**:
- Error display widget
- Error history tracking
- Recovery suggestions
- Error reporting

## MONITORING
**File**: `monitoring.rs`  
**Purpose**: Real-time system monitoring

**Monitoring Features**:
- CPU usage tracking
- Memory usage tracking
- Disk usage tracking
- Network monitoring
- Load average calculation
- Health score calculation
- Alert management

**Alert System**:
- Configurable thresholds
- Severity levels (Info, Warning, Critical)
- Alert history
- Alert notifications

**Performance Metrics**:
- Execution metrics
- System metrics
- Custom metrics
- Time series data

## UNDO SYSTEM
**File**: `undo.rs`  
**Purpose**: Undo/redo functionality

**Operation Types**:
- WorkflowExecution
- WorkflowCreation
- WorkflowModification
- WorkflowDeletion
- ConfigurationChange
- ThemeChange
- LayoutChange
- PluginInstallation
- ToolExecution
- FileOperation

**Undo Manager**:
- Operation history
- Undo/redo stack
- Max history size
- Operation state tracking

## SYNC SYSTEM
**File**: `sync.rs` (1,715 lines)  
**Purpose**: Synchronization with backend

**Sync Features**:
- Real-time synchronization
- Conflict resolution
- Connection status tracking
- Offline mode support
- Sync history

**Backends**:
- `MockSyncBackend`: Testing and development
- `MemoryCacheBackend`: In-memory caching
- `HttpSyncBackend`: Production HTTP backend

**Sync Operations**:
- Data synchronization
- Conflict detection
- Resolution strategies
- Retry logic

## USAGE
```bash
cargo run -- tui
```

**Global Key Bindings**:
- `↑/↓`: Navigate list
- `Enter`: Select/Execute
- `Esc`: Go back/Cancel
- `q`: Quit
- `r`: Refresh
- `h`: Help
- `Tab`: Switch view
- `/`: Search
- `Ctrl+C`: Interrupt

**Widget-Specific Bindings**:
- **LogViewer**: `f` (filter), `n/N` (search), `e` (export), `a` (auto-scroll)
- **SystemStatus**: `m` (maintenance), `c` (clear alerts), `d` (cleanup)
- **WorkflowList**: `x` (execute), `s` (status), `p` (pause/resume)
- **ToolManager**: `i` (info), `e` (execute), `v` (validate)
- **PluginManager**: `l` (load), `u` (unload), `r` (reload), `c` (configure)

## ARCHITECTURE
```
Event → EventLoop → ActionProcessor → Action → State Update → Render
```

**Component Flow**:
1. **Event Generation**: Keyboard, mouse, resize, tick
2. **Event Loop**: Continuous event processing
3. **Action Processing**: Convert events to actions
4. **State Update**: Modify shared state
5. **Widget Update**: Widgets react to state changes
6. **Rendering**: Draw to terminal
7. **Repeat**: Continue loop

## PERFORMANCE OPTIMIZATION
**File**: `performance.rs`  
**Purpose**: TUI performance optimization

**Optimization Techniques**:
- **Virtual Scrolling**: Render only visible items
- **Update Throttling**: Limit update frequency (10-60 FPS)
- **Lazy Loading**: Load data on demand
- **Memory Management**: Efficient resource usage
- **Batch Updates**: Group state changes

**Performance Metrics**:
- Frame rate (FPS)
- Memory usage
- Render time
- Event processing time

## TESTING
**Files**: `tests/tui_standalone_unit_tests.rs` (1,697 lines)  
**Purpose**: Comprehensive TUI testing

**Test Coverage**:
- Widget rendering
- Event handling
- State management
- Navigation
- Error recovery
- Performance benchmarks

**Test Types**:
- Unit tests for individual widgets
- Integration tests for widget composition
- Property-based tests for event processing
- Performance tests for rendering

## CONFIGURATION
**TUI Configuration**:
```toml
[tui]
theme = "dark"  # dark, light, auto
refresh_rate = 60  # Hz
enable_mouse = true
enable_virtualization = true
max_log_entries = 10000
```

**Theme Configuration**:
- Color schemes
- Style definitions
- Border styles
- Status colors

## ARCHITECTURE DECISIONS

### Why Event-Driven?
1. **Decoupling**: Widgets don't need to know about each other
2. **Flexibility**: Easy to add new event types
3. **Testability**: Events can be simulated
4. **Extensibility**: Custom events for custom widgets

### Why Async Rendering?
1. **Non-blocking**: UI remains responsive
2. **Real-time updates**: Widgets can update independently
3. **Performance**: Parallel rendering
4. **Scalability**: Handles large datasets

### Why Virtual Scrolling?
1. **Memory Efficiency**: Only render visible items
2. **Performance**: Handle thousands of items
3. **Responsiveness**: Smooth scrolling
4. **Scalability**: Works with any dataset size

## BEST PRACTICES

### Widget Design
1. **Single Responsibility**: One widget, one purpose
2. **Stateless Rendering**: Render based on state, not internal state
3. **Error Boundaries**: Don't crash on errors
4. **Help Text**: Document key bindings
5. **Theme Support**: Respect global theme

### Performance
1. **Throttle Updates**: 10-60 FPS depending on widget
2. **Lazy Load**: Load data on demand
3. **Cache Results**: Reuse expensive computations
4. **Release Resources**: Clean up on destruction

### User Experience
1. **Provide Feedback**: Show loading states
2. **Handle Errors Gracefully**: Show error dialogs
3. **Keyboard Navigation**: Full keyboard support
4. **Help System**: Press 'h' for help
5. **Responsive Design**: Adapt to terminal size

## SUBDIRECTORIES

### widgets/
**Path**: `src/interfaces/tui/widgets/`  
**Purpose**: 8 specialized widgets

**Files**:
- `execution_monitor.rs`: Real-time execution tracking
- `log_viewer.rs`: Audit log viewer (1,595 lines)
- `plugin_manager.rs`: Plugin management (3,553 lines)
- `sync_status.rs`: Synchronization status
- `system_status.rs`: System health monitoring (2,687 lines)
- `tool_manager.rs`: Tool registry management
- `workflow_list.rs`: Workflow browsing and execution

**Common Pattern**:
```rust
pub struct WidgetName {
    context: WidgetContext,
    capabilities: WidgetCapabilities,
    // Widget-specific state
}

impl WidgetName {
    pub async fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) -> Result<(), WidgetError>;
    pub async fn handle_event(&mut self, event: Event) -> Result<Option<Action>, WidgetError>;
    pub async fn update(&mut self) -> Result<(), WidgetError>;
    pub fn help_text(&self) -> Vec<(&str, &str)>;
}
```

## SEE ALSO

- [Interfaces AGENTS.md](../AGENTS.md) - Interface layer overview
- [Widgets AGENTS.md](widgets/AGENTS.md) - Widget components
- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [CLI AGENTS.md](../cli/AGENTS.md) - Command-line interface

<!-- AUTO-GENERATED-AGENT-MAP:START -->
## 🗺️ Agent Map & Directory Structure

> **Auto-generated** on 2026-01-17 20:44:16

- **[widgets/](widgets/AGENTS.md)**: Specialized widgets for the TUI interface, each handling a specific view or functionality.

<!-- AUTO-GENERATED-AGENT-MAP:END -->
