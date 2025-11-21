# Story 5.3: Docker Ollama Installation

Status: done

## Story

As a user,
I want to run Ollama in Docker,
so that I can use it without system-wide installation.

## Acceptance Criteria

1. **AC1:** When Docker is available and user chooses Docker installation, run:
   ```
   docker run -d -v ollama:/root/.ollama -p 11434:11434 --name ollama ollama/ollama
   ```

2. **AC2:** Wait for container to be healthy (health check with timeout)

3. **AC3:** Verify Ollama API is accessible at localhost:11434

4. **AC4:** Report success with container info (container ID, status)

5. **AC5:** When Docker is not available, report "Docker not found" with installation link

6. **AC6:** Offer native install as alternative when Docker unavailable

7. **AC7:** When container named "ollama" already exists, offer to:
   - Restart existing container, OR
   - Remove and recreate

8. **AC8:** Handle port conflicts (11434 already in use)

9. **AC9:** Use volume mount for persistence (`ollama:/root/.ollama`)

10. **AC10:** `cargo build` succeeds without errors

11. **AC11:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Implement Docker detection (AC: 5, 6)
  - [x] Add `is_docker_available()` function
  - [x] Check if Docker daemon is running
  - [x] Return helpful error message if Docker not found

- [x] Task 2: Implement Docker installation (AC: 1, 9)
  - [x] Add `install_docker()` function to `setup/install.rs`
  - [x] Execute `docker run` command with correct flags
  - [x] Ensure volume mount for persistence

- [x] Task 3: Handle existing container (AC: 7)
  - [x] Check if container "ollama" exists (`docker ps -a`)
  - [x] If exists: prompt user to restart or recreate
  - [x] Implement restart: `docker start ollama`
  - [x] Implement recreate: `docker rm ollama` then create new

- [x] Task 4: Health check and verification (AC: 2, 3, 4)
  - [x] Wait for container to start (max 60s timeout)
  - [x] Poll localhost:11434/api/tags for API readiness
  - [x] Get container info via `docker inspect`
  - [x] Report container ID and status on success

- [x] Task 5: Error handling (AC: 5, 8)
  - [x] Handle port 11434 already in use
  - [x] Provide clear error messages with suggestions
  - [x] Offer native install as fallback

- [x] Task 6: Integration with setup flow
  - [x] Add Docker option to `InstallMethod` enum (if not exists)
  - [x] Integrate into setup orchestration
  - [x] Update user prompts in setup workflow

- [x] Task 7: Verify build (AC: 10, 11)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

- Use `std::process::Command` for Docker CLI calls
- Follow async pattern for health check polling
- Match existing error handling with `.context()` from anyhow

### Project Structure

Files to modify/create:
- `src/setup/install.rs` - Add Docker installation functions
- `src/setup/mod.rs` - Export new functions if needed

### Testing Approach

- Unit tests for Docker command construction
- Integration test requires Docker daemon
- Mock Docker responses for CI environment

### Learnings from Previous Story

**From Story 5-2-native-ollama-installation (Status: done)**

- InstallResult struct available for returning installation outcomes
- `install_native()`, `start_ollama()`, `wait_for_ollama()` patterns established
- OS detection patterns in place
- Timeout handling implemented (5 minute pattern)

Use existing patterns from Story 5.2:
- `start_ollama()` for starting services
- `wait_for_ollama()` for health checks
- `InstallResult` for return values

[Source: docs/sprint-artifacts/5-2-native-ollama-installation.md]

### References

- [Source: docs/epics.md#Story-5.3]
- [Source: docs/architecture.md#Project-Structure]
- [Source: docs/sprint-artifacts/tech-spec-epic-5.md]

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/5-3-docker-ollama-installation.context.xml

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

- Installed protobuf via brew for lance-encoding build
- Fixed clippy warnings (unnested or-patterns, doc_markdown, match_same_arms)

### Completion Notes List

- Implemented `install_docker()` function following `install_native()` pattern
- Added `ContainerStatus` enum (Running, Stopped, NotFound)
- Implemented container detection via `docker ps` commands
- Added health check with 60s timeout using existing `wait_for_ollama()`
- Port conflict detection via stderr parsing
- 7 new unit tests added for Docker functions
- All 106 tests passing (2 ignored - display-dependent)

### File List

- MODIFIED: src/setup/install.rs (added ~270 lines for Docker installation)

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
| 2025-11-21 | 2.0 | Implementation complete - all ACs satisfied |
