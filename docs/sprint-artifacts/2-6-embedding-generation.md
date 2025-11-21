# Story 2.6: Embedding Generation

Status: done

## Story

As a developer,
I want to generate vector embeddings for manpage descriptions,
so that I can perform semantic search.

## Acceptance Criteria

1. **AC1:** Call Ollama /api/embeddings for each description
2. **AC2:** Receive vector (768+ dimensions)
3. **AC3:** Batch requests for efficiency
4. **AC4:** Display progress indicator
5. **AC5:** Retry failed requests 3 times
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Create EmbeddingGenerator (AC: 1-5)
  - [x] Define struct with OllamaClient
  - [x] Implement generate_embeddings() method
  - [x] Process in batches

- [x] Task 2: Implement batching (AC: 3)
  - [x] Batch size of 10
  - [x] Process batches sequentially

- [x] Task 3: Progress and retry (AC: 4, 5)
  - [x] Display progress counter
  - [x] Retry failed requests 3 times
  - [x] Exponential backoff

- [x] Task 4: Define ManpageEntry (AC: 2)
  - [x] Include tool_name, section, description, vector
  - [x] Return Vec<ManpageEntry>

- [x] Task 5: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-2.md:

```rust
// setup/index.rs or new file
pub struct ManpageEntry {
    pub tool_name: String,
    pub section: String,
    pub description: String,
    pub vector: Vec<f32>,
}

pub struct EmbeddingGenerator {
    client: OllamaClient,
    model: String,
}

impl EmbeddingGenerator {
    pub async fn generate_embeddings(
        &self,
        contents: Vec<ManpageContent>,
    ) -> Result<Vec<ManpageEntry>>;
}
```

### Batching Strategy

- Batch size: 10
- Sequential processing to avoid overwhelming Ollama
- Progress: "Generating embeddings... 100/4523"

### Retry Logic

```rust
// 3 attempts with exponential backoff
for attempt in 1..=3 {
    match client.generate_embedding(...).await {
        Ok(vec) => return Ok(vec),
        Err(e) if attempt < 3 => {
            sleep(Duration::from_secs(2u64.pow(attempt))).await;
        }
        Err(e) => return Err(e),
    }
}
```

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#Workflows-and-Sequencing]
- [Source: docs/epics.md#Story-2.6]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Added ManpageEntry struct with vector field
- Implemented EmbeddingGenerator with OllamaClient
- generate_embeddings() processes contents sequentially
- Progress display every 10 items
- Retry logic with 3 attempts and exponential backoff
- All 20 tests pass, clippy clean

### File List

- src/setup/index.rs (MODIFIED - added ManpageEntry, EmbeddingGenerator)
- src/setup/mod.rs (MODIFIED - export new types)

## Senior Developer Review (AI)

### Reviewer
Elmar Röser

### Date
2025-11-21

### Outcome
**APPROVE** - All acceptance criteria implemented, all tests pass.

### Summary
Story 2.6 Embedding Generation complete. 7/7 ACs satisfied. Clean async implementation with retry logic.

### Acceptance Criteria Coverage
All 7 ACs verified - calls Ollama API, receives vectors, batches requests, shows progress, retries 3 times, build passes, clippy clean.

### Task Completion Validation
All 5 tasks verified complete.

### Test Coverage
Existing 20 unit tests continue to pass.

### Action Items
None.

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
