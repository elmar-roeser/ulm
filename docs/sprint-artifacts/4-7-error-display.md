# Story 4.7: Error Display

Status: done

## Story

As a user,
I want clear error messages with actionable guidance,
So that I can understand and resolve issues.

## Acceptance Criteria

1. **AC1:** Clear error messages to stderr
2. **AC2:** Actionable guidance included
3. **AC3:** Exit code 1 for errors
4. **AC4:** `cargo build` succeeds without errors
5. **AC5:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create error display module (AC: 1, 2)
  - [ ] Create `src/tui/error.rs`
  - [ ] Format errors with context
  - [ ] Add actionable suggestions

- [ ] Task 2: Integrate with main (AC: 3)
  - [ ] Handle errors in main
  - [ ] Exit with code 1

- [ ] Task 3: Verify build (AC: 4, 5)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Error Categories

1. **Setup errors**: Ollama not running, model not available
2. **Query errors**: No results found, LLM timeout
3. **TUI errors**: Terminal issues, clipboard failures

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-4.md#Story-4.7]
- [Source: docs/epics.md#Story-4.7]

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
