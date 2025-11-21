# Story 3.1: Vector Search

Status: done

## Story

As a developer,
I want to perform semantic search against the manpage index,
so that I can find relevant tools for user queries.

## Acceptance Criteria

1. **AC1:** Generate embedding for query via Ollama
2. **AC2:** Search LanceDB for nearest neighbors
3. **AC3:** Return top 3 matches with scores
4. **AC4:** Search completes in < 100ms (excluding embedding)
5. **AC5:** Handle empty index gracefully
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Create query module structure (AC: 1-3)
  - [x] Create src/query/mod.rs
  - [x] Create src/query/search.rs
  - [x] Update lib.rs to export query module

- [x] Task 2: Implement search_tools function (AC: 1-3)
  - [x] Generate query embedding via OllamaClient
  - [x] Call db::search with embedding
  - [x] Map results to SearchMatch struct

- [x] Task 3: Define SearchMatch struct (AC: 3)
  - [x] tool_name, section, description, score
  - [x] Derive Debug, Clone

- [x] Task 4: Handle edge cases (AC: 4, 5)
  - [x] Return empty vec for empty index
  - [x] Return error if index doesn't exist

- [x] Task 5: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-3.md:

```rust
// query/search.rs
pub struct SearchMatch {
    pub tool_name: String,
    pub section: String,
    pub description: String,
    pub score: f32,
}

pub async fn search_tools(query: &str, limit: usize) -> Result<Vec<SearchMatch>>;
```

### Flow

1. Call OllamaClient::generate_embedding() with query
2. Call db::search() with embedding vector
3. Map db::SearchResult to SearchMatch
4. Return Vec<SearchMatch>

### Learnings from Previous Stories

**From Epic 2:**

- **db::search()**: Takes query vector and limit, returns Vec<SearchResult>
- **OllamaClient::generate_embedding()**: Returns Vec<f32>
- **DEFAULT_MODEL**: "llama3" constant in llm module

[Source: docs/sprint-artifacts/tech-spec-epic-2.md]

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Acceptance-Criteria]
- [Source: docs/epics.md#Story-3.1]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Created query module structure (mod.rs, search.rs)
- Implemented search_tools() async function
- Generates query embedding via OllamaClient
- Performs vector search via db::search()
- Maps db::SearchResult to SearchMatch struct
- Checks if index exists before searching
- Logs search results with tracing
- 2 new unit tests for SearchMatch
- All 37 tests pass, clippy clean

### File List

- src/query/mod.rs (NEW)
- src/query/search.rs (NEW)

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
