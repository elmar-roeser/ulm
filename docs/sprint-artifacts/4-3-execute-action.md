# Story 4.3: Execute Action

Status: done

## Story

As a user,
I want to execute the selected command with Enter or 'A',
So that I can run it immediately.

## Acceptance Criteria

1. **AC1:** Execute on Enter or 'A' key
2. **AC2:** Close TUI before execution
3. **AC3:** Inherit stdin/stdout/stderr
4. **AC4:** Exit with command's exit code
5. **AC5:** `cargo build` succeeds without errors
6. **AC6:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create exec module (AC: 3, 4)
  - [ ] Create src/exec/mod.rs
  - [ ] Create src/exec/shell.rs
  - [ ] Implement execute_command function

- [ ] Task 2: Verify build (AC: 5, 6)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// exec/shell.rs
pub fn execute_command(command: &str) -> Result<i32>;
```

### Execution Strategy

- Use std::process::Command
- Spawn with shell (sh -c)
- Inherit stdin/stdout/stderr
- Return exit code

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-4.md#Story-4.3]
- [Source: docs/epics.md#Story-4.3]

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
