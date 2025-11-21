# Story 3.7: Query Orchestration

Status: done

## Story

As a developer,
I want to orchestrate the full query pipeline,
so that users can get command suggestions from natural language queries.

## Acceptance Criteria

1. **AC1:** Orchestrate full query pipeline
2. **AC2:** Return Vec<CommandSuggestion>
3. **AC3:** Total latency < 5 seconds
4. **AC4:** Handle "no matching tools" error
5. **AC5:** Include explanations of WHY
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create orchestration function (AC: 1, 2)
  - [ ] Add process_query function to query/mod.rs
  - [ ] Call search_tools
  - [ ] Load manpage content
  - [ ] Scan directory context
  - [ ] Build prompt
  - [ ] Call Ollama generate
  - [ ] Parse response

- [ ] Task 2: Error handling (AC: 4, 5)
  - [ ] Handle no matching tools
  - [ ] Handle Ollama errors
  - [ ] Clear error messages

- [ ] Task 3: Verify build (AC: 6, 7)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// query/mod.rs
pub async fn process_query(query: &str) -> Result<Vec<CommandSuggestion>>;
```

### Pipeline Flow

1. Generate query embedding (Ollama)
2. Vector search (LanceDB) → top 3 tools
3. Load manpage content (man command)
4. Scan directory context
5. Build prompt
6. Call Ollama generate
7. Parse JSON response

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Story-3.7]
- [Source: docs/epics.md#Story-3.7]

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
