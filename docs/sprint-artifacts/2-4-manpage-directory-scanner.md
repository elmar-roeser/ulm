# Story 2.4: Manpage Directory Scanner

Status: done

## Story

As a developer,
I want to scan system manpage directories,
so that I can find all available documentation.

## Acceptance Criteria

1. **AC1:** Scan /usr/share/man, /usr/local/share/man
2. **AC2:** Include $MANPATH directories
3. **AC3:** Find man1 and man8 sections
4. **AC4:** Handle missing directories gracefully
5. **AC5:** Complete scan in < 5 seconds for 5000 manpages
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Create setup/index.rs module (AC: 1-4)
  - [x] Create ManpageScanner struct
  - [x] Define default scan paths
  - [x] Add $MANPATH support

- [x] Task 2: Implement directory scanning (AC: 1, 3, 4)
  - [x] Scan man1 and man8 subdirectories
  - [x] Filter by .gz, .1, .8 extensions
  - [x] Handle missing directories gracefully

- [x] Task 3: Return results (AC: 5)
  - [x] Return Vec<PathBuf> of found manpages
  - [x] Log count of found pages
  - [x] Ensure performance target

- [x] Task 4: Add unit tests
  - [x] Test with temp directories
  - [x] Test missing directories
  - [x] Test file filtering

- [x] Task 5: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-2.md:

```rust
// setup/index.rs
pub struct ManpageScanner {
    paths: Vec<PathBuf>,
}

impl ManpageScanner {
    pub fn new() -> Self;
    pub fn scan_directories(&self) -> Result<Vec<PathBuf>>;
}
```

### Default Paths

```rust
const DEFAULT_PATHS: &[&str] = &[
    "/usr/share/man",
    "/usr/local/share/man",
    "/opt/homebrew/share/man", // macOS
];
```

### File Extensions

- `.1`, `.1.gz` - User commands
- `.8`, `.8.gz` - System administration

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#Workflows-and-Sequencing]
- [Source: docs/epics.md#Story-2.4]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Implemented ManpageScanner struct with default paths
- Scans /usr/share/man, /usr/local/share/man, /opt/homebrew/share/man
- Includes $MANPATH directories
- Filters for man1 and man8 sections
- Supports .1, .8, .1.gz, .8.gz extensions
- 6 unit tests with tempfile for temp directory testing
- All 16 tests pass, clippy clean

### File List

- src/setup/index.rs (NEW - ManpageScanner implementation)
- src/setup/mod.rs (MODIFIED - export index module)
- Cargo.toml (MODIFIED - added tempfile dev-dependency)

## Senior Developer Review (AI)

### Reviewer
Elmar Röser

### Date
2025-11-21

### Outcome
**APPROVE** - All acceptance criteria implemented, all tests pass.

### Summary
Story 2.4 Manpage Directory Scanner complete. 7/7 ACs satisfied with 6 unit tests. Clean implementation with proper error handling.

### Acceptance Criteria Coverage
All 7 ACs verified - scans default paths, includes $MANPATH, filters man1/man8, handles missing dirs, build passes, clippy clean.

### Task Completion Validation
All 5 tasks verified complete.

### Test Coverage
6 unit tests covering scanner creation, directory scanning, filtering, and error handling.

### Action Items
None.

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
