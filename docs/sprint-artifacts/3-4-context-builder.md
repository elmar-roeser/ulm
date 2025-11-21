# Story 3.4: Context Builder

Status: done

## Story

As a developer,
I want to format the directory context for LLM prompt inclusion,
so that the LLM can understand the project environment.

## Acceptance Criteria

1. **AC1:** Create DirectoryContext with format_for_prompt method
2. **AC2:** Format context for prompt inclusion
3. **AC3:** Handle no project type gracefully
4. **AC4:** Limit marker list to 20 items
5. **AC5:** `cargo build` succeeds without errors
6. **AC6:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Add format_for_prompt method (AC: 1, 2)
  - [ ] Format project type
  - [ ] Format marker files
  - [ ] Format current directory

- [ ] Task 2: Handle edge cases (AC: 3, 4)
  - [ ] Handle None project type
  - [ ] Limit markers to 20 items

- [ ] Task 3: Verify build (AC: 5, 6)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// query/context.rs
impl DirectoryContext {
    pub fn format_for_prompt(&self) -> String;
}
```

### Format Example

```
Working Directory: /home/user/project
Project Type: Rust
Marker Files: Cargo.toml, .git, README.md
```

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Story-3.4]
- [Source: docs/epics.md#Story-3.4]

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
