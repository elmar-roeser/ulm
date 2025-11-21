# Epic 4: Interactive Experience - Technical Specification

## Overview

Epic 4 implements the Terminal User Interface (TUI) for displaying command suggestions and handling user interactions. Users can navigate suggestions, execute commands, copy to clipboard, or edit before execution.

---

## Data Models

```rust
// tui/mod.rs
/// User action selected in the TUI.
#[derive(Debug, Clone)]
pub enum UserAction {
    /// Execute the command directly.
    Execute(String),
    /// Copy command to clipboard.
    Copy(String),
    /// Edit command before executing.
    Edit(String),
    /// Abort without action.
    Abort,
}

/// TUI application state.
pub struct App {
    /// Command suggestions to display.
    pub suggestions: Vec<CommandSuggestion>,
    /// Currently selected index.
    pub selected: usize,
    /// Status message (e.g., "Copied!").
    pub status_message: Option<String>,
    /// Whether in edit mode.
    pub editing: bool,
    /// Edit buffer for command.
    pub edit_buffer: String,
    /// Cursor position in edit buffer.
    pub cursor_position: usize,
}
```

### APIs and Interfaces

```rust
// tui/mod.rs
pub fn run_tui(suggestions: Vec<CommandSuggestion>) -> Result<UserAction>;

// tui/render.rs
pub fn render(frame: &mut Frame, app: &App);

// tui/input.rs
pub fn handle_event(app: &mut App, event: Event) -> Option<UserAction>;

// exec/shell.rs
pub fn execute_command(command: &str) -> Result<i32>;

// exec/clipboard.rs
pub fn copy_to_clipboard(text: &str) -> Result<()>;
```

---

## Non-Functional Requirements

### Performance

| Metric | Target | Strategy |
|--------|--------|----------|
| Input response | < 50ms | Immediate event handling |
| Render update | < 16ms | Efficient diff rendering |
| Command execution | Immediate | Direct shell spawn |

### User Experience

- Clear visual hierarchy
- Risk level color coding (green/yellow/red)
- Keyboard shortcuts shown in footer
- Smooth navigation with wrap-around

---

## Dependencies

New Cargo.toml dependencies:
```toml
ratatui = "0.29"
crossterm = "0.28"
arboard = "3"
```

---

## Acceptance Criteria (Authoritative)

### Story 4.1: TUI Renderer
1. AC4.1.1: Display suggestions with index, title, command
2. AC4.1.2: Show explanation for selected item
3. AC4.1.3: Highlight selected item visually
4. AC4.1.4: Color code risk levels (green/yellow/red)
5. AC4.1.5: Show hotkeys in footer

### Story 4.2: Event Loop & Navigation
1. AC4.2.1: Navigate with Up/Down arrows
2. AC4.2.2: Wrap around at list boundaries
3. AC4.2.3: Update display on selection change
4. AC4.2.4: Respond in < 50ms
5. AC4.2.5: Ignore unrecognized keys

### Story 4.3: Execute Action
1. AC4.3.1: Execute on Enter or 'A' key
2. AC4.3.2: Close TUI before execution
3. AC4.3.3: Inherit stdin/stdout/stderr
4. AC4.3.4: Exit with command's exit code

### Story 4.4: Copy Action
1. AC4.4.1: Copy to clipboard on 'K' key
2. AC4.4.2: Show "Copied!" feedback
3. AC4.4.3: Stay in TUI after copy
4. AC4.4.4: Handle clipboard errors gracefully

### Story 4.5: Edit Action
1. AC4.5.1: Enter edit mode on 'B' key
2. AC4.5.2: Show editable command line
3. AC4.5.3: Support basic editing (arrows, delete, insert)
4. AC4.5.4: Execute edited command on Enter
5. AC4.5.5: Cancel edit on Escape

### Story 4.6: Abort Action
1. AC4.6.1: Abort on Escape key
2. AC4.6.2: Restore terminal
3. AC4.6.3: Exit with code 0

### Story 4.7: Error Display
1. AC4.7.1: Clear error messages to stderr
2. AC4.7.2: Actionable guidance included
3. AC4.7.3: Exit code 1 for errors

### Story 4.8: TUI Orchestration
1. AC4.8.1: Orchestrate complete TUI flow
2. AC4.8.2: Handle Ctrl-C gracefully
3. AC4.8.3: Clean up terminal on panic
4. AC4.8.4: Auto-select single suggestion

---

## Implementation Notes

### Key Bindings

| Key | Action |
|-----|--------|
| ↑/k | Move up |
| ↓/j | Move down |
| Enter/A | Execute |
| K | Copy |
| B | Edit |
| Esc/q | Abort |

### Risk Level Colors

- Safe: Green (#00ff00)
- Moderate: Yellow (#ffff00)
- Destructive: Red (#ff0000)

### Terminal Restoration

Use `scopeguard` or manual cleanup to ensure terminal is restored even on panic.
