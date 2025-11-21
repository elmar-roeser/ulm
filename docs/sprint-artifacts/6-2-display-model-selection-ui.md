# Story 6.2: Display Model Selection UI

Status: done

## Story

As a user,
I want to see available models with RAM requirements,
so that I can choose one suitable for my system.

## Acceptance Criteria

1. **AC1:** Display table of models with:
   - Model name
   - RAM requirement
   - Speed rating (1-5)
   - Quality rating (1-5)
   - [Installed] marker if present

2. **AC2:** Detect system RAM using `sysinfo` crate

3. **AC3:** Highlight recommended model based on detected RAM

4. **AC4:** User can select model with number key (1-4)

5. **AC5:** Return selected model index to caller

6. **AC6:** `cargo build` succeeds without errors

7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Add sysinfo dependency (AC: 2)
  - [x] Add sysinfo to Cargo.toml
  - [x] Create get_system_ram_gb() function

- [x] Task 2: Implement display_model_selection function (AC: 1, 3, 5)
  - [x] Format model table with columns
  - [x] Add [Installed] marker for installed models
  - [x] Highlight recommended model based on RAM
  - [x] Return selected index

- [x] Task 3: Implement user input handling (AC: 4)
  - [x] Read user input (1-4)
  - [x] Validate input range
  - [x] Handle invalid input with retry

- [x] Task 4: Unit tests
  - [x] Test format_stars function
  - [x] Test RAM-based recommendation logic
  - [x] Test get_system_ram_gb returns valid value

- [x] Task 5: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

- Simple terminal table (no TUI/crossterm needed)
- Sync function using stdin for input
- Use `.context()` from anyhow for all errors

### Project Structure

Files to modify:
- `Cargo.toml` - Add sysinfo dependency
- `src/setup/models.rs` - Add display_model_selection and get_system_ram_gb

### Data Structures

Use existing `RecommendedModel` struct from Story 6.1:
```rust
pub struct RecommendedModel {
    pub name: String,
    pub ram_gb: f32,
    pub speed_rating: u8,
    pub quality_rating: u8,
    pub installed: bool,
}
```

### Function Signature

```rust
/// Displays model selection table and returns selected index.
pub fn display_model_selection(models: &[RecommendedModel], system_ram_gb: f32) -> Result<usize>;

/// Gets system RAM in GB.
pub fn get_system_ram_gb() -> f32;
```

### Table Format Example

```
Available Models:

 #  Model         RAM    Speed  Quality  Status
 1  llama3.2:3b   4 GB   ★★★★★  ★★★☆☆   [Installed]
 2  mistral:7b    6 GB   ★★★★☆  ★★★★☆
 3  llama3.1:8b   8 GB   ★★★☆☆  ★★★★★   [Recommended]
 4  phi3:mini     3 GB   ★★★★★  ★★☆☆☆

Your system has 16 GB RAM. Recommended: llama3.1:8b

Select model (1-4):
```

### Recommendation Logic

- If system RAM >= 8 GB: recommend llama3.1:8b (quality)
- If system RAM >= 6 GB: recommend mistral:7b (balance)
- If system RAM >= 4 GB: recommend llama3.2:3b (speed)
- Otherwise: recommend phi3:mini (minimal)

### Learnings from Previous Story

**From Story 6-1-fetch-available-models (Status: done)**

- **New Struct Created**: `RecommendedModel` at `src/setup/models.rs` - use this for model data
- **Function Available**: `get_available_models(&OllamaClient)` returns `Vec<RecommendedModel>` with installed status
- **Pattern**: Use `get_default_models()` to get hardcoded list with metadata

[Source: docs/sprint-artifacts/6-1-fetch-available-models.md#Dev-Agent-Record]

### References

- [Source: docs/epics.md#Story-6.2]
- [Source: docs/sprint-artifacts/tech-spec-epic-6.md]
- [Source: docs/architecture.md#Project-Structure]

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/6-2-display-model-selection-ui.context.xml

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

- Added sysinfo crate v0.31 for RAM detection
- Fixed clippy cast_precision_loss warning with #[allow] attribute
- Fixed clippy print_literal warning in table header

### Completion Notes List

- Implemented `get_system_ram_gb()` using sysinfo crate
- Implemented `display_model_selection()` with formatted table and star ratings
- Added `format_stars()` helper for visual rating display
- Added `get_recommended_model_index()` for RAM-based recommendations
- Added 3 new unit tests (format_stars, recommendation logic, RAM detection)
- All 113 tests passing (2 ignored - display-dependent)

### File List

- MODIFIED: Cargo.toml (added sysinfo dependency)
- MODIFIED: src/setup/models.rs (added ~120 lines for display and RAM functions)
- MODIFIED: src/setup/mod.rs (added exports)

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
| 2025-11-21 | 2.0 | Implementation complete - all ACs satisfied |
