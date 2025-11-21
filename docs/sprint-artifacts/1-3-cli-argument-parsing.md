# Story 1.3: CLI Argument Parsing

Status: done

## Story

As a user,
I want to run ulm with different commands,
so that I can setup, query, or update the system.

## Acceptance Criteria

1. **AC1:** `ulm --help` displays usage information
2. **AC2:** `ulm --version` displays version from Cargo.toml
3. **AC3:** `ulm setup` is recognized as valid subcommand
4. **AC4:** `ulm update` is recognized as valid subcommand
5. **AC5:** `ulm "query text"` captures query string correctly
6. **AC6:** Invalid commands show helpful error messages
7. **AC7:** `cargo build` succeeds without errors
8. **AC8:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Define CLI argument structures (AC: 1-6)
  - [x] Create Args struct with clap derive in cli.rs
  - [x] Define Commands enum with Setup and Update variants
  - [x] Add query as trailing variadic argument
  - [x] Configure --help and --version from Cargo.toml

- [x] Task 2: Implement argument parsing in main.rs (AC: 1-6)
  - [x] Parse Args using clap
  - [x] Match on command/query and dispatch (stub handlers)
  - [x] Handle missing query gracefully

- [x] Task 3: Add integration tests (AC: 1-6)
  - [x] Test --help output
  - [x] Test --version output
  - [x] Test setup subcommand recognition
  - [x] Test update subcommand recognition
  - [x] Test query string capture
  - [x] Test invalid command error messages

- [x] Task 4: Verify build (AC: 7, 8)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-1.md Data Models and Contracts:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ulm")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Natural language query
    #[arg(trailing_var_arg = true)]
    pub query: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize ulm with Ollama and manpage index
    Setup,
    /// Refresh the manpage index
    Update,
}
```

### CLI Behavior

- Default command is query if string provided (no subcommand)
- `ulm "find large files"` → query = ["find", "large", "files"]
- `ulm setup` → command = Some(Commands::Setup)
- `ulm update` → command = Some(Commands::Update)

### Testing Standards

- Use `assert_cmd` crate for CLI testing
- Test actual binary execution
- Verify exit codes and output

### Learnings from Previous Story

**From Story 1-2-module-structure-setup (Status: done)**

- **cli.rs exists**: Placeholder struct ready to be replaced with clap Args
- **main.rs structure**: Already imports ulm::Result, returns Result<()>
- **Documentation required**: All public items need doc comments (missing_docs = "warn")

[Source: docs/sprint-artifacts/1-2-module-structure-setup.md#Dev-Agent-Record]

### References

- [Source: docs/architecture.md#CLI-Parsing]
- [Source: docs/sprint-artifacts/tech-spec-epic-1.md#Data-Models-and-Contracts]
- [Source: docs/epics.md#Story-1.3]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Implemented Args struct with clap Parser derive
- Commands enum with Setup and Update variants
- Query captured as trailing variadic argument (Vec<String>)
- Added helper methods: parse_args(), has_query(), query_string()
- main.rs dispatches based on command/query
- 10 integration tests covering all ACs
- Added assert_cmd and predicates to dev-dependencies
- Fixed missing_const_for_fn lint for has_query()

### File List

- src/cli.rs (MODIFIED - replaced placeholder with clap implementation)
- src/main.rs (MODIFIED - added CLI parsing and dispatch)
- tests/cli_test.rs (NEW - 10 integration tests)
- Cargo.toml (MODIFIED - added assert_cmd, predicates)

## Senior Developer Review (AI)

### Reviewer
Elmar Röser

### Date
2025-11-21

### Outcome
**APPROVE** - All acceptance criteria implemented, all tests pass.

### Summary
Story 1.3 CLI Argument Parsing complete. 8/8 ACs satisfied with 10 integration tests. Clean clap implementation following architecture spec.

### Acceptance Criteria Coverage
All 8 ACs verified through integration tests.

### Task Completion Validation
All 4 tasks verified complete.

### Test Coverage
10 integration tests covering all CLI functionality.

### Action Items
None.

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial implementation with 10 tests |
| 2025-11-21 | 1.0 | Senior Developer Review - APPROVED |
