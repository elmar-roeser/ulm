# Story 4.1: TUI Renderer

Status: done

## Story

As a user,
I want to see command suggestions displayed clearly,
So that I can understand my options.

## Acceptance Criteria

1. **AC1:** Display suggestions with index, title, command
2. **AC2:** Show explanation for selected item
3. **AC3:** Highlight selected item visually
4. **AC4:** Color code risk levels (green/yellow/red)
5. **AC5:** Show hotkeys in footer
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create TUI module structure (AC: 6)
  - [ ] Create src/tui/mod.rs
  - [ ] Create src/tui/render.rs
  - [ ] Define App state struct

- [ ] Task 2: Implement renderer (AC: 1, 2, 3, 4, 5)
  - [ ] Render suggestion list
  - [ ] Highlight selected item
  - [ ] Show explanation panel
  - [ ] Color by risk level
  - [ ] Render footer with keys

- [ ] Task 3: Verify build (AC: 6, 7)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// tui/mod.rs
pub struct App {
    pub suggestions: Vec<CommandSuggestion>,
    pub selected: usize,
    pub status_message: Option<String>,
}

// tui/render.rs
pub fn render(frame: &mut Frame, app: &App);
```

### Layout

```
┌─ ulm - Command Suggestions ───────────────┐
│ [1] Find large files                       │
│     find . -size +100M                     │
│                                            │
│ [2] List by size ◄ SELECTED                │
│     ls -lhS                                │
│                                            │
│ [3] Disk usage                             │
│     du -sh *                               │
├────────────────────────────────────────────┤
│ Explanation:                               │
│ Lists files sorted by size in human-       │
│ readable format                            │
├────────────────────────────────────────────┤
│ ↑↓ Navigate  A Execute  K Copy  B Edit  Esc│
└────────────────────────────────────────────┘
```

### Risk Level Colors

- Safe: Green
- Moderate: Yellow
- Destructive: Red

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-4.md#Story-4.1]
- [Source: docs/epics.md#Story-4.1]

## Dev Agent Record

### Context Reference

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
