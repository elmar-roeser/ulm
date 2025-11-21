# Story 2.5: Manpage Content Extraction

Status: done

## Story

As a developer,
I want to extract NAME and DESCRIPTION from manpages,
so that I can create searchable descriptions.

## Acceptance Criteria

1. **AC1:** Run `man -P cat <tool>` for each manpage
2. **AC2:** Parse NAME section
3. **AC3:** Parse DESCRIPTION (first paragraph)
4. **AC4:** Handle malformed manpages gracefully
5. **AC5:** Validate UTF-8 output
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Add content extraction to ManpageScanner (AC: 1-5)
  - [x] Define ManpageContent struct
  - [x] Implement extract_content() method
  - [x] Shell out to `man -P cat`

- [x] Task 2: Parse manpage output (AC: 2, 3)
  - [x] Extract tool name from path
  - [x] Parse NAME section
  - [x] Parse first paragraph of DESCRIPTION

- [x] Task 3: Error handling (AC: 4, 5)
  - [x] Handle missing manpages
  - [x] Handle malformed content
  - [x] Validate UTF-8

- [x] Task 4: Add unit tests
  - [x] Test content extraction
  - [x] Test parsing logic

- [x] Task 5: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-2.md:

```rust
// setup/index.rs
pub struct ManpageContent {
    pub tool_name: String,
    pub section: String,
    pub description: String,
}

impl ManpageScanner {
    pub fn extract_content(&self, path: &Path) -> Result<ManpageContent>;
}
```

### Shell Command

```bash
man -P cat <tool>
```

The `-P cat` option uses `cat` as the pager, outputting raw text.

### Parsing Strategy

1. Extract tool name from filename (e.g., "ls.1.gz" → "ls")
2. Find NAME section and extract first line
3. Find DESCRIPTION and extract first paragraph (~500 chars)

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#Workflows-and-Sequencing]
- [Source: docs/epics.md#Story-2.5]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Added ManpageContent struct with tool_name, section, description
- Implemented extract_content() using `man -P cat`
- Parse NAME and DESCRIPTION sections
- Truncates to 500 chars for embedding efficiency
- Handles malformed manpages gracefully
- UTF-8 validation on output
- 4 new unit tests for parsing functions
- All 20 tests pass, clippy clean

### File List

- src/setup/index.rs (MODIFIED - added ManpageContent and extraction methods)
- src/setup/mod.rs (MODIFIED - export ManpageContent)

## Senior Developer Review (AI)

### Reviewer
Elmar Röser

### Date
2025-11-21

### Outcome
**APPROVE** - All acceptance criteria implemented, all tests pass.

### Summary
Story 2.5 Manpage Content Extraction complete. 7/7 ACs satisfied with 4 new unit tests. Robust parsing with graceful error handling.

### Acceptance Criteria Coverage
All 7 ACs verified - runs man -P cat, parses NAME/DESCRIPTION, handles errors, validates UTF-8, build passes, clippy clean.

### Task Completion Validation
All 5 tasks verified complete.

### Test Coverage
4 new unit tests for filename parsing, section extraction, content parsing.

### Action Items
None.

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
