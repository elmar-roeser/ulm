# Story 3.2: Manpage Content Loading

Status: done

## Story

As a developer,
I want to load full manpage content for matched tools,
so that I can provide complete context to the LLM.

## Acceptance Criteria

1. **AC1:** Run `man -P cat <tool>` for matched tools
2. **AC2:** Capture full output as string
3. **AC3:** Clean escape sequences and formatting
4. **AC4:** Truncate to ~8000 chars for LLM context
5. **AC5:** Handle missing manpages with error
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Add load_manpage_content function (AC: 1, 2)
  - [ ] Implement in query/search.rs
  - [ ] Shell out to `man -P cat <tool>`
  - [ ] Capture stdout as String

- [ ] Task 2: Clean content (AC: 3)
  - [ ] Strip ANSI escape codes
  - [ ] Normalize whitespace

- [ ] Task 3: Truncate for LLM (AC: 4)
  - [ ] Limit to ~8000 chars
  - [ ] Handle UTF-8 boundaries
  - [ ] Prioritize important sections

- [ ] Task 4: Error handling (AC: 5)
  - [ ] Return error for missing manpage
  - [ ] Clear error message

- [ ] Task 5: Verify build (AC: 6, 7)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// query/search.rs
pub fn load_manpage_content(tool_name: &str) -> Result<String>;
```

### Shell Command

```bash
man -P cat <tool>
```

### Content Cleaning

- Strip ANSI: `\x1b\[[0-9;]*m`
- Normalize multiple spaces/newlines

### Learnings from Previous Stories

**From Story 2.5 (Manpage Content Extraction):**

- `man -P cat` outputs raw text without pager
- UTF-8 validation needed
- Handle command failures gracefully

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Story-3.2]
- [Source: docs/epics.md#Story-3.2]

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
