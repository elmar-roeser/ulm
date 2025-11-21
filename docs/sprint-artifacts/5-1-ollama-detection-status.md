# Story 5.1: Ollama Detection & Status

Status: done

## Story

As a user,
I want setup to detect my current Ollama installation status,
So that it can guide me appropriately.

## Acceptance Criteria

1. **AC1:** Detect Ollama running (API accessible at localhost:11434) ✓
2. **AC2:** Detect Ollama installed but not running (`which ollama`) ✓
3. **AC3:** Detect Ollama not installed ✓
4. **AC4:** Detect Docker availability (`which docker`) ✓
5. **AC5:** Report clear status message to user ✓
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Create install module (AC: 1-5)
  - [x] Define OllamaStatus enum
  - [x] Define SystemCapabilities struct
  - [x] Implement detect_system()
  - [x] Check API endpoint
  - [x] Check which commands

- [ ] Task 2: Verify build (AC: 6, 7)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## References

- [Source: docs/sprint-artifacts/tech-spec-epic-5.md]
- [Source: docs/epics.md#Story-5.1]

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
