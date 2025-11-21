# Story 2.7: LanceDB Storage

Status: done

## Story

As a developer,
I want to store embeddings in LanceDB,
so that I can perform fast vector search.

## Acceptance Criteria

1. **AC1:** Create database at ~/.local/share/ulm/index.lance
2. **AC2:** Schema includes: tool_name, section, description, vector
3. **AC3:** Use directories crate for XDG paths
4. **AC4:** Create parent directories if needed
5. **AC5:** Overwrite existing index on re-run
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [x] Task 1: Add LanceDB dependencies (AC: 3)
  - [x] Add lancedb = "0.22" to Cargo.toml
  - [x] Add arrow-array = "56" (matching lancedb version)
  - [x] Add futures = "0.3" for stream handling
  - [x] directories = "5" already present

- [x] Task 2: Create database module (AC: 1, 3, 4)
  - [x] Define get_database_path() using directories
  - [x] Create parent directories if needed
  - [x] Connect to LanceDB at XDG data path

- [x] Task 3: Implement storage functions (AC: 2, 5)
  - [x] Define create_index(entries: Vec<ManpageEntry>)
  - [x] Create table "manpages" with schema
  - [x] Overwrite existing table on re-run

- [x] Task 4: Add helper functions (AC: 1)
  - [x] Define index_exists() check
  - [x] Define search() for vector queries
  - [x] Define count_entries() for statistics

- [x] Task 5: Verify build (AC: 6, 7)
  - [x] Run `cargo build`
  - [x] Run `cargo clippy -- -D warnings`
  - [x] Fix any issues

## Dev Notes

### Architecture Patterns

From tech-spec-epic-2.md:

```rust
// db.rs
use lancedb::connect;
use arrow_array::{StringArray, Float32Array, RecordBatch};
use directories::ProjectDirs;

pub async fn get_database_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "ulm")
        .ok_or_else(|| anyhow!("Could not determine data directory"))?;
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("index.lance"))
}

pub async fn create_index(entries: Vec<ManpageEntry>) -> Result<()>;
pub async fn index_exists() -> Result<bool>;
```

### LanceDB Schema

| Column | Type | Description |
|--------|------|-------------|
| tool_name | String | Tool name (e.g., "ffmpeg") |
| section | String | Man section (e.g., "1") |
| description | String | NAME + DESCRIPTION text |
| vector | Vec<f32> | 768+ dimensional embedding |

### XDG Base Directory

- Linux: ~/.local/share/ulm/index.lance
- macOS: ~/Library/Application Support/ulm/index.lance

### Learnings from Previous Story

**From Story 2-6-embedding-generation (Status: done)**

- **New Type Created**: `ManpageEntry` struct with `tool_name`, `section`, `description`, `vector` fields - use this directly as input to storage
- **EmbeddingGenerator**: Returns `Vec<ManpageEntry>` ready for database storage
- **Vector Dimensions**: Embeddings are 768+ dimensional Vec<f32>

[Source: docs/sprint-artifacts/2-6-embedding-generation.md#Dev-Agent-Record]

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#Data-Models-and-Contracts]
- [Source: docs/sprint-artifacts/tech-spec-epic-2.md#Acceptance-Criteria]
- [Source: docs/epics.md#Story-2.7]

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude Sonnet 4.5

### Debug Log References

### Completion Notes List

- Implemented complete LanceDB storage module in db.rs
- Uses arrow-array 56 to match lancedb 0.22 transitive dependency
- Added futures crate for TryStreamExt async stream handling
- get_database_path() uses directories crate for XDG-compliant paths
- create_index() creates/overwrites "manpages" table with schema
- Schema: tool_name, section, description, vector (FixedSizeList)
- index_exists() checks if database and table exist
- search() performs vector similarity search with configurable limit
- count_entries() returns total indexed manpages
- 5 new unit tests for db module, all 25 tests pass
- Clippy clean with allow for intentional i32 cast

### File List

- src/db.rs (MODIFIED - complete LanceDB implementation)
- Cargo.toml (MODIFIED - added arrow-array, futures)

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
