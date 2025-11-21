# Story 4.2: Event Loop & Navigation

Status: done

## Story

As a user,
I want to navigate suggestions with arrow keys,
So that I can select the right option.

## Acceptance Criteria

1. **AC1:** Navigate with Up/Down arrows
2. **AC2:** Wrap around at list boundaries
3. **AC3:** Update display on selection change
4. **AC4:** Respond in < 50ms
5. **AC5:** Ignore unrecognized keys
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create input module (AC: 1, 2, 5)
  - [ ] Create src/tui/input.rs
  - [ ] Handle key events
  - [ ] Support j/k and arrow keys

- [ ] Task 2: Create event loop (AC: 3, 4)
  - [ ] Implement main TUI loop
  - [ ] Poll for events
  - [ ] Re-render on changes

- [ ] Task 3: Verify build (AC: 6, 7)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// tui/input.rs
pub fn handle_event(app: &mut App, event: Event) -> Option<UserAction>;

// tui/mod.rs
pub fn run_tui(suggestions: Vec<CommandSuggestion>) -> Result<UserAction>;
```

### Key Bindings

- Up/k: Previous
- Down/j: Next
- Enter/A: Execute
- K: Copy
- B: Edit
- Esc/q: Abort

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-4.md#Story-4.2]
- [Source: docs/epics.md#Story-4.2]

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
