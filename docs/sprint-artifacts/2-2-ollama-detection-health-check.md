# Story 2.2: Ollama Detection & Health Check

Status: done

## Story

As a user,
I want setup to detect if Ollama is running,
so that I know if I need to install or start it.

## Acceptance Criteria

1. **AC1:** Health check pings localhost:11434/api/tags
2. **AC2:** Success displays "✓ Ollama detected at localhost:11434"
3. **AC3:** Failure displays clear install instructions
4. **AC4:** Timeout is 5 seconds for detection
5. **AC5:** `cargo build` succeeds without errors
6. **AC6:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Create setup/ollama.rs module (AC: 1-4)
  - [x] Create OllamaChecker struct
  - [x] Implement check_connection() method
  - [x] Use OllamaClient from Story 2.1

- [x] Task 2: Implement detection logic (AC: 1, 4)
  - [x] Call health_check() from OllamaClient
  - [x] Handle connection errors
  - [x] 5 second timeout

- [x] Task 3: User feedback (AC: 2, 3)
  - [x] Success message with checkmark
  - [x] Failure message with install instructions
  - [x] Include Docker option hint

- [x] Task 4: Verify build (AC: 5, 6)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-2.md:

```rust
// setup/ollama.rs
pub struct OllamaChecker {
    client: OllamaClient,
}

impl OllamaChecker {
    pub fn new() -> Result<Self>;
    pub async fn check_connection(&self) -> Result<()>;
}
```

### User Messages

```rust
// Success
"✓ Ollama detected at localhost:11434"

// Failure
"✗ Ollama not found at localhost:11434

Please install Ollama:
  • Download from https://ollama.ai
  • Or run via Docker: docker run -d -p 11434:11434 ollama/ollama

Then start with: ollama serve"
```

### Learnings from Previous Story

**From Story 2.1 (Status: done)**

- OllamaClient.health_check() already implemented
- Returns Result<bool> for connection status
- User-friendly error messages in place

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#Workflows-and-Sequencing]
- [Source: docs/epics.md#Story-2.2]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Implemented OllamaChecker struct wrapping OllamaClient
- check_connection() method with success/failure messages
- User-friendly install instructions with download URL and Docker option
- 2 unit tests for checker creation
- All tests pass (10 total), clippy clean

### File List

- src/setup/ollama.rs (NEW - Ollama detection and health check)
- src/setup/mod.rs (MODIFIED - export ollama module)

## Senior Developer Review (AI)

### Reviewer
Elmar Röser

### Date
2025-11-21

### Outcome
**APPROVE** - All acceptance criteria implemented, all tests pass.

### Summary
Story 2.2 Ollama Detection & Health Check complete. 6/6 ACs satisfied with 2 unit tests. Clean implementation using OllamaClient from Story 2.1.

### Acceptance Criteria Coverage
All 6 ACs verified - health check pings /api/tags, success/failure messages displayed, 5s timeout, build passes, clippy clean.

### Task Completion Validation
All 4 tasks verified complete.

### Test Coverage
2 unit tests for OllamaChecker creation.

### Action Items
None.

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
