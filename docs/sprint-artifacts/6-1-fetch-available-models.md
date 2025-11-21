# Story 6.1: Fetch Available Models

Status: done

## Story

As a developer,
I want to retrieve the list of available Ollama models,
so that I can show users their options.

## Acceptance Criteria

1. **AC1:** Query Ollama `/api/tags` for installed models

2. **AC2:** System has hardcoded list of recommended models with metadata:
   - llama3.2:3b (~4GB RAM)
   - mistral:7b (~6GB RAM)
   - llama3.1:8b (~8GB RAM)
   - phi3:mini (~3GB RAM)

3. **AC3:** Return `Vec<ModelInfo>` with name, RAM, speed/quality ratings

4. **AC4:** Mark models as `installed: true` if present in Ollama tags

5. **AC5:** Handle Ollama connection errors gracefully

6. **AC6:** `cargo build` succeeds without errors

7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Create setup/models.rs module (AC: 2, 3)
  - [x] Define `RecommendedModel` struct with all fields
  - [x] Create `get_default_models()` function with recommended models
  - [x] Export from setup/mod.rs

- [x] Task 2: Implement get_available_models function (AC: 1, 4)
  - [x] Add async function signature
  - [x] Query Ollama `/api/tags` endpoint via `OllamaClient::list_models()`
  - [x] Parse response for installed model names
  - [x] Merge with recommended models, set installed flag

- [x] Task 3: Error handling (AC: 5)
  - [x] Handle connection refused via `.context()`
  - [x] Handle timeout (OllamaClient already has 5s timeout)
  - [x] Return Result with context

- [x] Task 4: Unit tests
  - [x] Test RecommendedModel serialization
  - [x] Test default models content
  - [x] Test model equality

- [x] Task 5: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

- Use existing `OllamaClient` from `llm/ollama.rs`
- Async function with `Result<Vec<ModelInfo>>`
- Match error handling patterns with `.context()`

### Project Structure

Files to create/modify:
- `src/setup/models.rs` - NEW: Model management functions
- `src/setup/mod.rs` - Export models module

### Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub ram_gb: f32,
    pub speed_rating: u8,
    pub quality_rating: u8,
    pub installed: bool,
}
```

### Ollama API

**Request:** GET /api/tags

**Response:**
```json
{
  "models": [
    {
      "name": "llama3.2:3b",
      "model": "llama3.2:3b",
      "modified_at": "...",
      "size": 2000000000
    }
  ]
}
```

### References

- [Source: docs/epics.md#Story-6.1]
- [Source: docs/sprint-artifacts/tech-spec-epic-6.md]
- [Source: docs/architecture.md#APIs-and-Interfaces]

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/6-1-fetch-available-models.context.xml

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

- Used existing `OllamaClient::list_models()` from `src/llm/ollama.rs`
- Named struct `RecommendedModel` to distinguish from existing `ModelInfo` in ollama.rs
- Added model name matching with `:latest` suffix handling

### Completion Notes List

- Implemented `RecommendedModel` struct with name, ram_gb, speed_rating, quality_rating, installed fields
- Created `get_default_models()` function returning 4 recommended models
- Implemented `get_available_models()` async function that merges Ollama installed models with recommended list
- Added 4 unit tests for serialization, deserialization, default models content, and equality
- All 110 tests passing (2 ignored - display-dependent)

### File List

- CREATED: src/setup/models.rs (~180 lines)
- MODIFIED: src/setup/mod.rs (added models module export)

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
| 2025-11-21 | 2.0 | Implementation complete - all ACs satisfied |
