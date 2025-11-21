# Story 4.5: Edit Action

Status: done

## Story

As a user,
I want to edit the command before executing with 'B',
So that I can modify paths or parameters.

## Acceptance Criteria

1. **AC1:** Enter edit mode on 'B' key
2. **AC2:** Show editable command line
3. **AC3:** Support basic editing (arrows, delete, insert)
4. **AC4:** Execute edited command on Enter
5. **AC5:** Cancel edit on Escape
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Add edit functionality (AC: 1, 2, 3, 4, 5)
  - [ ] Use simple line input
  - [ ] Pre-fill with command
  - [ ] Handle Enter to execute
  - [ ] Handle Escape to cancel

- [ ] Task 2: Verify build (AC: 6, 7)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Implementation

For simplicity, we'll use crossterm's line reading capabilities
rather than rustyline to avoid complexity.

The edit action will:
1. Exit raw mode temporarily
2. Print the command for editing
3. Read a new line
4. Return the edited command

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-4.md#Story-4.5]
- [Source: docs/epics.md#Story-4.5]

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
