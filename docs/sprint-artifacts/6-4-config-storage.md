# Story 6.4: Config Storage

Status: done

## Story

As a user,
I want my model choice saved,
so that ulm uses it for all future queries.

## Acceptance Criteria

1. **AC1:** Save model name to config file after selection

2. **AC2:** Config stored at `~/.config/ulm/config.toml`

3. **AC3:** Use toml crate for serialization

4. **AC4:** Load existing config on startup if present

5. **AC5:** Use configured model name when querying Ollama

6. **AC6:** Create config directory if it doesn't exist

7. **AC7:** Config file has user-only permissions (0600)

8. **AC8:** `cargo build` succeeds without errors

9. **AC9:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Implement config module (AC: 1, 2, 3, 6)
  - [x] Create `src/setup/config.rs`
  - [x] Define Config struct with model_name and ollama_url
  - [x] Implement `get_config_path()` using directories crate
  - [x] Create config directory if missing

- [x] Task 2: Implement save_config (AC: 1, 7)
  - [x] Serialize Config to TOML
  - [x] Write to config file
  - [x] Set file permissions to 0600

- [x] Task 3: Implement load_config (AC: 4, 5)
  - [x] Read config file if exists
  - [x] Deserialize from TOML
  - [x] Return default config if file missing

- [x] Task 4: Integrate with setup flow (AC: 5)
  - [x] Save config after model selection
  - [x] Export functions from setup/mod.rs

- [x] Task 5: Unit tests
  - [x] Test Config serialization/deserialization
  - [x] Test get_config_path returns XDG-compliant path
  - [x] Test default config values
  - [x] Test save/load roundtrip

- [x] Task 6: Verify build (AC: 8, 9)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

- Use `directories` crate for XDG-compliant paths (already in Cargo.toml)
- Add `toml` crate for config serialization
- Use `.context()` from anyhow for all errors
- Simple config struct, no complex nesting

### Project Structure

Files to create:
- `src/setup/config.rs` - Config management module

Files to modify:
- `src/setup/mod.rs` - Add config exports
- `Cargo.toml` - Add toml dependency

### Data Structures

```rust
/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model_name: String,
    pub ollama_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_name: "llama3.2:3b".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
        }
    }
}
```

### Public Functions

```rust
/// Get XDG-compliant config file path
pub fn get_config_path() -> Result<PathBuf>;

/// Load config from file, return default if not found
pub fn load_config() -> Result<Config>;

/// Save config to file with proper permissions
pub fn save_config(config: &Config) -> Result<()>;
```

### Config File Example

```toml
# ~/.config/ulm/config.toml
model_name = "llama3.2:3b"
ollama_url = "http://localhost:11434"
```

### Learnings from Previous Story

**From Story 6-3-model-pull-with-progress (Status: done)**

- **Functions Available**: `pull_model_with_progress()`, `PullProgress` struct
- **Pattern**: indicatif progress bars already implemented
- **Integration Point**: After `pull_model_with_progress()` succeeds, save config
- **Existing Exports**: `src/setup/mod.rs` already exports model-related functions

[Source: docs/sprint-artifacts/6-3-model-pull-with-progress.md#Dev-Agent-Record]

### References

- [Source: docs/epics.md#Story-6.4]
- [Source: docs/sprint-artifacts/tech-spec-epic-6.md#Data-Models-and-Contracts]
- [Source: docs/sprint-artifacts/tech-spec-epic-6.md#APIs-and-Interfaces]

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/6-4-config-storage.context.xml

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

- No issues encountered

### Completion Notes List

- Implemented `Config` struct with model_name and ollama_url fields
- Implemented `get_config_path()` using directories crate for XDG compliance
- Implemented `load_config()` returning default if file missing
- Implemented `save_config()` with TOML serialization and 0600 permissions
- Added 7 unit tests for config functionality
- All 123 tests passing (2 ignored - display-dependent)

### File List

- NEW: src/setup/config.rs (~180 lines)
- MODIFIED: src/setup/mod.rs (added config module and exports)
- MODIFIED: Cargo.toml (added toml = "0.8")

### Completion Notes
**Completed:** 2025-11-21
**Definition of Done:** All acceptance criteria met, code reviewed, tests passing

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
| 2025-11-21 | 2.0 | Implementation complete - all ACs satisfied |
