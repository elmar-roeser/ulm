# Story 2.1: Ollama API Client

Status: done

## Story

As a developer,
I want a client to communicate with Ollama API,
so that I can generate embeddings and LLM responses.

## Acceptance Criteria

1. **AC1:** `OllamaClient` can POST to /api/embeddings
2. **AC2:** `OllamaClient` can POST to /api/generate
3. **AC3:** Requests are serialized as JSON using serde
4. **AC4:** Responses are deserialized into typed structs
5. **AC5:** Connection errors return descriptive messages
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Create OllamaClient struct (AC: 1-5)
  - [x] Define client struct with reqwest::Client and base_url
  - [x] Implement new() constructor
  - [x] Add configurable timeout

- [x] Task 2: Implement embedding endpoint (AC: 1, 3, 4)
  - [x] Define EmbeddingRequest struct
  - [x] Define EmbeddingResponse struct
  - [x] Implement generate_embedding() async method

- [x] Task 3: Implement generate endpoint (AC: 2, 3, 4)
  - [x] Define GenerateRequest struct
  - [x] Define GenerateResponse struct
  - [x] Implement generate() async method

- [x] Task 4: Implement health check (AC: 1, 5)
  - [x] Define TagsResponse struct
  - [x] Implement health_check() method
  - [x] Implement list_models() method

- [x] Task 5: Error handling (AC: 5)
  - [x] Add context to all errors
  - [x] User-friendly error messages

- [x] Task 6: Add unit tests
  - [x] Test request serialization
  - [x] Test response deserialization
  - [x] Test error handling

- [x] Task 7: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From architecture.md and tech-spec-epic-2.md:

```rust
// llm/ollama.rs
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub response: String,
}
```

### API Endpoints

- `POST /api/embeddings` - Generate vector embeddings
- `POST /api/generate` - Generate LLM response
- `GET /api/tags` - List available models (health check)

### Timeouts

- Embeddings: 30 seconds
- Generate: 60 seconds
- Health check: 5 seconds

### Error Messages

```rust
// User-facing errors
"Cannot connect to Ollama at {url}. Start with: ollama serve"
"Ollama request timed out after {seconds}s"

// Developer errors with context
.context("Failed to connect to Ollama API")?
.context("Failed to parse embedding response")?
```

### References

- [Source: docs/architecture.md#API-Contracts]
- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#APIs-and-Interfaces]
- [Source: docs/epics.md#Story-2.1]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Implemented OllamaClient with reqwest async HTTP client
- Supports /api/embeddings, /api/generate, /api/tags endpoints
- Request/Response structs with serde serialization
- Configurable timeouts (5s health, 30s embed, 60s generate)
- User-friendly error messages with context
- 8 unit tests for serialization/deserialization
- All tests pass, clippy clean

### File List

- src/llm/ollama.rs (NEW - Ollama API client implementation)
- src/llm/mod.rs (MODIFIED - export ollama module)

## Senior Developer Review (AI)

### Reviewer
Elmar Röser

### Date
2025-11-21

### Outcome
**APPROVE** - All acceptance criteria implemented, all tests pass.

### Summary
Story 2.1 Ollama API Client complete. 7/7 ACs satisfied with 8 unit tests. Clean async implementation with proper error handling.

### Acceptance Criteria Coverage
All 7 ACs verified - OllamaClient supports embeddings and generate endpoints, uses serde JSON, typed structs, descriptive errors, build passes, clippy clean.

### Task Completion Validation
All 7 tasks verified complete.

### Test Coverage
8 unit tests covering serialization, deserialization, and client creation.

### Action Items
None.

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
