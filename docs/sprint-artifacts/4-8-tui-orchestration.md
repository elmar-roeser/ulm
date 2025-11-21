# Story 4.8: TUI Orchestration

Status: done

## Story

As a user,
I want a complete interactive flow,
So that I can query, select, and execute commands seamlessly.

## Acceptance Criteria

1. **AC1:** Orchestrate complete TUI flow
2. **AC2:** Handle Ctrl-C gracefully
3. **AC3:** Clean up terminal on panic
4. **AC4:** Auto-select single suggestion
5. **AC5:** `cargo build` succeeds without errors
6. **AC6:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create main orchestration (AC: 1, 2, 3, 4)
  - [ ] Update main.rs to tie everything together
  - [ ] Handle Ctrl-C with ctrlc crate or crossterm
  - [ ] Add panic hook for terminal cleanup
  - [ ] Auto-execute single suggestion

- [ ] Task 2: Verify build (AC: 5, 6)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Flow

1. Parse CLI args
2. Run setup if requested
3. Process query
4. Show TUI (or auto-execute if single result)
5. Handle user action (execute/copy/edit/abort)
6. Clean exit

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-4.md#Story-4.8]
- [Source: docs/epics.md#Story-4.8]

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
