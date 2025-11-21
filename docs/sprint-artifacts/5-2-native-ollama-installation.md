# Story 5.2: Native Ollama Installation

Status: done

## Story

As a user,
I want to install Ollama natively via the official installer,
So that I get optimal performance.

## Acceptance Criteria

1. **AC1:** Install on Linux via `curl -fsSL https://ollama.com/install.sh | sh`
2. **AC2:** Install on macOS via `brew install ollama` (with curl fallback)
3. **AC3:** Request sudo with explanation when needed
4. **AC4:** Verify installation succeeded
5. **AC5:** Start Ollama service after install
6. **AC6:** Report success or failure with next steps
7. **AC7:** Timeout after 5 minutes
8. **AC8:** `cargo build` succeeds without errors
9. **AC9:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Implement native installation (AC: 1-7)
  - [x] Add install_native() function
  - [x] Add start_ollama() function
  - [x] Add wait_for_ollama() function
  - [x] Handle OS-specific commands
  - [x] Add InstallResult struct

- [x] Task 2: Verify build (AC: 8, 9)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`

## References

- [Source: docs/sprint-artifacts/tech-spec-epic-5.md]
- [Source: docs/epics.md#Story-5.2]

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
