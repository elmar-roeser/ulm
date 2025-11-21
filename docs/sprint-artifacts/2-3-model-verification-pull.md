# Story 2.3: Model Verification & Pull

Status: done

## Story

As a user,
I want setup to ensure a suitable LLM model is available,
so that I can generate embeddings and responses.

## Acceptance Criteria

1. **AC1:** Query /api/tags for installed models
2. **AC2:** Display available model when found
3. **AC3:** Prompt user to pull if no model
4. **AC4:** Pull displays progress
5. **AC5:** Confirm when pull complete
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Add model checking to OllamaChecker (AC: 1, 2)
  - [x] Implement check_model() method
  - [x] Use list_models() from OllamaClient
  - [x] Check for suitable model (llama3, mistral, etc.)

- [x] Task 2: Implement model pulling (AC: 3, 4, 5)
  - [x] Add pull_model() to OllamaClient
  - [x] Prompt user for confirmation
  - [x] Display progress during pull

- [x] Task 3: User feedback (AC: 2, 3, 5)
  - [x] Success message when model found
  - [x] Prompt message for pull
  - [x] Completion message after pull

- [x] Task 4: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-2.md:

```rust
// Extend OllamaChecker
impl OllamaChecker {
    pub async fn check_model(&self, preferred: &str) -> Result<String>;
    pub async fn pull_model(&self, name: &str) -> Result<()>;
}
```

### User Messages

```rust
// Model found
"✓ Model 'llama3' available"

// No model - prompt
"No suitable model found.
Pull default model 'llama3'? [Y/n] "

// Pulling
"Pulling llama3... (this may take a few minutes)"

// Complete
"✓ Model 'llama3' pulled successfully"

// Declined
"No model available. Pull manually with: ollama pull llama3"
```

### Suitable Models

Check for any of these (in order of preference):
- llama3, llama3.2
- mistral
- gemma2
- phi3

### Learnings from Previous Stories

**From Story 2.1 & 2.2 (Status: done)**

- OllamaClient.list_models() returns Vec<ModelInfo>
- OllamaChecker wraps OllamaClient
- Use tracing for logging

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#Workflows-and-Sequencing]
- [Source: docs/epics.md#Story-2.3]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Added pull_model() to OllamaClient with 10-minute timeout
- Implemented check_model() in OllamaChecker
- Checks for preferred models: llama3, llama3.2, mistral, gemma2, phi3
- User prompt for model pull with Y/n confirmation
- Success/failure messages for all states
- All tests pass (10 total), clippy clean

### File List

- src/llm/ollama.rs (MODIFIED - added PullRequest struct and pull_model method)
- src/setup/ollama.rs (MODIFIED - added check_model, pull_model, prompt_pull_model)

## Senior Developer Review (AI)

### Reviewer
Elmar Röser

### Date
2025-11-21

### Outcome
**APPROVE** - All acceptance criteria implemented, all tests pass.

### Summary
Story 2.3 Model Verification & Pull complete. 7/7 ACs satisfied. User-friendly model detection with automatic pull prompt.

### Acceptance Criteria Coverage
All 7 ACs verified - queries /api/tags, displays model status, prompts for pull, shows progress message, confirms completion, build passes, clippy clean.

### Task Completion Validation
All 4 tasks verified complete.

### Test Coverage
Existing 10 unit tests continue to pass.

### Action Items
None.

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
