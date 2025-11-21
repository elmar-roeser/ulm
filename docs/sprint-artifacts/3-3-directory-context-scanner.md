# Story 3.3: Directory Context Scanner

Status: done

## Story

As a developer,
I want to detect the project type and relevant files in the current directory,
so that the LLM can provide context-aware command suggestions.

## Acceptance Criteria

1. **AC1:** Detect Cargo.toml → Rust
2. **AC2:** Detect package.json → Node
3. **AC3:** Detect requirements.txt/pyproject.toml → Python
4. **AC4:** Detect go.mod → Go
5. **AC5:** Detect .git → Git repository
6. **AC6:** Scan only top-level (no recursion)
7. **AC7:** `cargo build` succeeds without errors
8. **AC8:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create context module structure (AC: 7)
  - [ ] Create query/context.rs
  - [ ] Add ProjectType enum
  - [ ] Add DirectoryContext struct

- [ ] Task 2: Implement project type detection (AC: 1, 2, 3, 4, 5)
  - [ ] Check for marker files
  - [ ] Map to ProjectType enum
  - [ ] Priority: Cargo.toml > package.json > go.mod > pyproject.toml > requirements.txt

- [ ] Task 3: Implement directory scanner (AC: 6)
  - [ ] Read only top-level directory
  - [ ] Collect marker file names
  - [ ] Return DirectoryContext

- [ ] Task 4: Verify build (AC: 7, 8)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// query/context.rs
#[derive(Debug, Clone)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    CMake,
    Git,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DirectoryContext {
    pub project_type: Option<ProjectType>,
    pub marker_files: Vec<String>,
    pub cwd: PathBuf,
}

pub fn scan_directory_context() -> Result<DirectoryContext>;
```

### Marker Files

| Project Type | Marker Files |
|--------------|--------------|
| Rust | Cargo.toml |
| Node | package.json |
| Python | pyproject.toml, requirements.txt |
| Go | go.mod |
| CMake | CMakeLists.txt |
| Git | .git |

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Story-3.3]
- [Source: docs/epics.md#Story-3.3]

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
