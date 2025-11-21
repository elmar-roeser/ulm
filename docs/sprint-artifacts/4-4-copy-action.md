# Story 4.4: Copy Action

Status: done

## Story

As a user,
I want to copy the command with 'K',
So that I can paste it elsewhere.

## Acceptance Criteria

1. **AC1:** Copy to clipboard on 'K' key
2. **AC2:** Show "Copied!" feedback
3. **AC3:** Stay in TUI after copy
4. **AC4:** Handle clipboard errors gracefully
5. **AC5:** `cargo build` succeeds without errors
6. **AC6:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create clipboard module (AC: 1, 4)
  - [ ] Create src/exec/clipboard.rs
  - [ ] Use arboard crate
  - [ ] Handle errors

- [ ] Task 2: Verify build (AC: 5, 6)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// exec/clipboard.rs
pub fn copy_to_clipboard(text: &str) -> Result<()>;
```

### arboard Usage

```rust
use arboard::Clipboard;
let mut clipboard = Clipboard::new()?;
clipboard.set_text(text)?;
```

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-4.md#Story-4.4]
- [Source: docs/epics.md#Story-4.4]

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
