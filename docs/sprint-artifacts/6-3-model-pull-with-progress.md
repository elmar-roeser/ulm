# Story 6.3: Model Pull with Progress

Status: done

## Story

As a user,
I want to download my selected model with progress feedback,
so that I know the download status.

## Acceptance Criteria

1. **AC1:** Call Ollama `/api/pull` endpoint with streaming enabled

2. **AC2:** Display download progress with percentage and speed

3. **AC3:** Show layer-by-layer progress during download

4. **AC4:** If model already installed, skip download and confirm ready

5. **AC5:** Handle network errors gracefully with informative messages

6. **AC6:** Timeout after 30 minutes for large models

7. **AC7:** `cargo build` succeeds without errors

8. **AC8:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Implement streaming pull request (AC: 1, 6)
  - [x] Create pull_model_with_progress function
  - [x] Send POST to /api/pull with stream: true
  - [x] Set 30 minute timeout

- [x] Task 2: Parse streaming response (AC: 2, 3)
  - [x] Define PullProgress struct
  - [x] Parse JSON lines from stream
  - [x] Extract status, digest, total, completed

- [x] Task 3: Display progress bar (AC: 2, 3)
  - [x] Use indicatif crate for progress bar
  - [x] Show percentage
  - [x] Update for each layer

- [x] Task 4: Handle special cases (AC: 4, 5)
  - [x] Handle network errors with context
  - [x] Parse error responses from Ollama

- [x] Task 5: Unit tests
  - [x] Test PullProgress parsing
  - [x] Test progress calculation

- [x] Task 6: Verify build (AC: 7, 8)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

- Use reqwest streaming response
- Parse JSON lines (newline-delimited JSON)
- Use indicatif for progress display
- Use `.context()` from anyhow for all errors

### Project Structure

Files to modify:
- `src/setup/models.rs` - Add pull_model_with_progress function

### Data Structures

```rust
/// Pull progress update from Ollama
#[derive(Debug, Clone, Deserialize)]
pub struct PullProgress {
    pub status: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}
```

### Ollama API

**Request:**
```json
POST /api/pull
{
  "name": "llama3.2:3b",
  "stream": true
}
```

**Response (streaming JSON lines):**
```json
{"status": "pulling manifest"}
{"status": "downloading", "digest": "sha256:...", "total": 2000000000, "completed": 500000000}
{"status": "verifying sha256 digest"}
{"status": "success"}
```

### Progress Display Example

```
Downloading llama3.2:3b...
[████████████████░░░░░░░░░░░░░░] 53% (1.2 GB / 2.3 GB) 15.3 MB/s
```

### Learnings from Previous Story

**From Story 6-2-display-model-selection-ui (Status: done)**

- **Functions Available**: `get_system_ram_gb()`, `display_model_selection()`, `get_available_models()`
- **Struct Available**: `RecommendedModel` with installed field
- **Pattern**: indicatif crate already in Cargo.toml for progress bars

[Source: docs/sprint-artifacts/6-2-display-model-selection-ui.md#Dev-Agent-Record]

### References

- [Source: docs/epics.md#Story-6.3]
- [Source: docs/sprint-artifacts/tech-spec-epic-6.md]
- [Source: docs/architecture.md#APIs-and-Interfaces]

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/6-3-model-pull-with-progress.context.xml

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

- Fixed clippy unnecessary_cast warning (u64 to u64)
- Fixed clippy assigning_clones warning (use clone_from)
- Removed unused BufRead import

### Completion Notes List

- Implemented `PullProgress` struct for JSON parsing
- Implemented `pull_model_with_progress()` with indicatif progress bar
- Parses streaming JSON lines from Ollama /api/pull
- Shows layer-by-layer progress with digest info
- 30 minute timeout for large models
- Added 3 new unit tests
- All 116 tests passing (2 ignored - display-dependent)

### File List

- MODIFIED: src/setup/models.rs (added ~140 lines for pull function)
- MODIFIED: src/setup/mod.rs (added exports)

### Completion Notes
**Completed:** 2025-11-21
**Definition of Done:** All acceptance criteria met, code reviewed, tests passing

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
| 2025-11-21 | 2.0 | Implementation complete - all ACs satisfied |
