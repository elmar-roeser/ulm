# Story 2.8: Setup Orchestration & Progress

Status: done

## Story

As a user,
I want `ulm setup` to orchestrate the complete setup process,
so that I can get started with one command.

## Acceptance Criteria

1. **AC1:** `ulm setup` runs all steps in order
2. **AC2:** Display progress for each step
3. **AC3:** Report final count: "Indexed N manpages"
4. **AC4:** `ulm update` refreshes index (skip Ollama check)
5. **AC5:** Optional shell alias installation
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Create setup orchestrator (AC: 1, 2)
  - [x] Implement run_setup() in setup/mod.rs
  - [x] Step 1: Check Ollama connection
  - [x] Step 2: Verify/pull model
  - [x] Step 3: Scan manpage directories
  - [x] Step 4: Extract descriptions
  - [x] Step 5: Generate embeddings
  - [x] Step 6: Store in LanceDB

- [x] Task 2: Add progress reporting (AC: 2, 3)
  - [x] Display step progress messages
  - [x] Show counts for each phase
  - [x] Report final indexed count

- [x] Task 3: Implement update command (AC: 4)
  - [x] Skip Ollama connection check
  - [x] Re-run indexing steps
  - [x] Overwrite existing index

- [x] Task 4: Wire up CLI commands (AC: 1, 4)
  - [x] Connect setup subcommand to orchestrator
  - [x] Connect update subcommand to orchestrator
  - [x] Add tokio runtime for async support

- [x] Task 5: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-2.md:

```rust
// setup/mod.rs
pub async fn run_setup() -> Result<()> {
    // 1. Check Ollama
    let checker = OllamaChecker::new()?;
    checker.check_connection().await?;

    // 2. Verify model
    let model = checker.check_model().await?;

    // 3. Scan manpages
    let scanner = ManpageScanner::new();
    let paths = scanner.scan_directories()?;

    // 4. Extract content
    let contents: Vec<ManpageContent> = ...;

    // 5. Generate embeddings
    let generator = EmbeddingGenerator::new()?;
    let entries = generator.generate_embeddings(contents).await?;

    // 6. Store in LanceDB
    db::create_index(entries).await?;

    println!("✓ Indexed {} manpages", count);
    Ok(())
}

pub async fn run_update() -> Result<()> {
    // Skip steps 1-2, run 3-6
}
```

### Progress Display

```
✓ Ollama detected at localhost:11434
✓ Model 'llama3' available
  Scanning manpage directories...
✓ Found 4,523 manpages
  Extracting descriptions... 100/4523
  Generating embeddings... 100/4523
  Storing in database...
✓ Indexed 4,523 manpages
```

### Learnings from Previous Stories

**From Story 2-7-lancedb-storage (Status: done)**

- **db::create_index()**: Takes Vec<ManpageEntry>, creates/overwrites table
- **db::index_exists()**: Check if index already exists
- **db::count_entries()**: Get total indexed count
- Uses XDG paths via directories crate

[Source: docs/sprint-artifacts/2-7-lancedb-storage.md#Dev-Agent-Record]

**From Story 2-6-embedding-generation (Status: done)**

- **EmbeddingGenerator::generate_embeddings()**: Takes Vec<ManpageContent>, returns Vec<ManpageEntry>
- Progress display every 10 items
- Retry logic with 3 attempts

[Source: docs/sprint-artifacts/2-6-embedding-generation.md#Dev-Agent-Record]

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#Workflows-and-Sequencing]
- [Source: docs/epics.md#Story-2.8]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Implemented run_setup() and run_update() orchestration functions
- run_setup() checks Ollama, verifies model, then indexes
- run_update() skips Ollama checks, just re-indexes
- Shared run_indexing() function for common steps
- Progress display with step counts and final summary
- Fixed UTF-8 truncation bug in manpage content parsing
- Updated main.rs with tokio runtime for async support
- Updated CLI tests with timeouts for setup/update commands
- All 35 tests pass (25 unit + 10 CLI), clippy clean

### File List

- src/setup/mod.rs (MODIFIED - added run_setup, run_update, run_indexing)
- src/setup/index.rs (MODIFIED - fixed UTF-8 truncation bug)
- src/main.rs (MODIFIED - added tokio runtime, wired up setup/update)
- tests/cli_test.rs (MODIFIED - updated tests with timeouts)

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
