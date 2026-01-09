# src/interfaces/tui/ - Terminal User Interface

## OVERVIEW
Ratatui-based TUI with event-driven architecture, action system, and reactive widgets.

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
**WorkflowList**: Displays available workflows
- Navigation with arrow keys
- Selection and execution
- Status indicators

**ExecutionMonitor**: Real-time execution tracking
- Progress bars
- Node status
- Error display

**LogViewer**: Audit log viewer
- Filtering by level
- Search functionality
- Auto-scroll

## LAYOUTS
**Main Layout**: Split screen
- **Left**: Navigation sidebar
- **Center**: Active view
- **Bottom**: Status bar

**Detail Views**:
- Workflow details
- Tool parameters
- Execution results

## USAGE
```bash
cargo run -- tui
```

**Key Bindings**:
- `↑/↓`: Navigate
- `Enter`: Select/Execute
- `q`: Quit
- `r`: Refresh
- `h`: Help

## ARCHITECTURE
```
Event → EventLoop → ActionProcessor → Action → State Update → Render
```

## THEMING
- Custom color schemes
- Style configuration
- Responsive design
