# Story 1.2: Module Structure Setup

Status: ready-for-dev

## Story

As a developer,
I want the project organized into logical modules,
so that code is maintainable and follows architecture design.

## Acceptance Criteria

1. **AC1:** The following directories and mod.rs files exist:
   ```
   src/
   ├── main.rs
   ├── lib.rs
   ├── cli.rs
   ├── error.rs
   ├── db.rs
   ├── setup/mod.rs
   ├── query/mod.rs
   ├── llm/mod.rs
   ├── tui/mod.rs
   └── exec/mod.rs
   ```
2. **AC2:** Each mod.rs properly declares submodules (empty stubs for now)
3. **AC3:** lib.rs re-exports public API for testing
4. **AC4:** main.rs imports from lib and sets up basic structure
5. **AC5:** `cargo build` succeeds without errors
6. **AC6:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create top-level modules (AC: 1, 5)
  - [ ] Create src/lib.rs with module declarations
  - [ ] Create src/cli.rs with placeholder struct
  - [ ] Create src/error.rs with placeholder
  - [ ] Create src/db.rs with placeholder

- [ ] Task 2: Create directory modules (AC: 1, 2)
  - [ ] Create src/setup/mod.rs with submodule stubs
  - [ ] Create src/query/mod.rs with submodule stubs
  - [ ] Create src/llm/mod.rs with submodule stubs
  - [ ] Create src/tui/mod.rs with submodule stubs
  - [ ] Create src/exec/mod.rs with submodule stubs

- [ ] Task 3: Configure lib.rs exports (AC: 3)
  - [ ] Re-export all modules as pub
  - [ ] Export anyhow::Result type alias

- [ ] Task 4: Update main.rs (AC: 4)
  - [ ] Import from ulm crate
  - [ ] Set up basic entry point structure

- [ ] Task 5: Verify build (AC: 5, 6)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`
  - [ ] Fix any documentation or lint issues

## Dev Notes

### Architecture Patterns

From architecture.md Project Structure section:

**Module Responsibilities:**
| Module | Responsibility |
|--------|---------------|
| main.rs | Entry point, CLI dispatch, runtime setup |
| lib.rs | Public API re-exports for testing |
| cli.rs | Clap argument definitions |
| error.rs | Custom error types (if needed) |
| db.rs | LanceDB operations |
| setup/mod.rs | Setup orchestration |
| query/mod.rs | Query orchestration |
| llm/mod.rs | LLM orchestration |
| tui/mod.rs | TUI orchestration |
| exec/mod.rs | Command execution |

**Code Organization Rules:**
- One struct per file when > 100 lines
- Related functions grouped in impl blocks
- Tests at bottom of file in `#[cfg(test)]` module

### Submodule Structure (for future stories)

```
setup/
├── mod.rs
├── ollama.rs      # Story 2.1-2.3
└── index.rs       # Story 2.4-2.7

query/
├── mod.rs
├── search.rs      # Story 3.1-3.2
└── context.rs     # Story 3.3-3.4

llm/
├── mod.rs
├── ollama.rs      # Story 2.1
├── prompt.rs      # Story 3.5
└── response.rs    # Story 3.6

tui/
├── mod.rs
├── render.rs      # Story 4.1
└── input.rs       # Story 4.2

exec/
├── mod.rs
├── shell.rs       # Story 4.3
└── clipboard.rs   # Story 4.4
```

### Testing Standards

- All modules should compile with placeholder implementations
- Use `pub(crate)` for internal APIs
- Strict linting: no `unwrap()` or `expect()`

### Learnings from Previous Story

**From Story 1-1-project-initialization (Status: done)**

- **Dependencies Ready**: All 13 dependencies configured and building
- **Linting Configured**: Strict rules in place including `missing_docs = "warn"`
- **Technical Note**: `unused_crate_dependencies = "allow"` - dependencies won't error until used
- **Version Note**: Using LanceDB 0.22 (not 0.4.x from original spec)

[Source: docs/sprint-artifacts/1-1-project-initialization.md#Dev-Agent-Record]

### References

- [Source: docs/architecture.md#Project-Structure]
- [Source: docs/sprint-artifacts/tech-spec-epic-1.md#Services-and-Modules]
- [Source: docs/sprint-artifacts/tech-spec-epic-1.md#Data-Models-and-Contracts]
- [Source: docs/epics.md#Story-1.2]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Will be filled by dev agent -->

### Debug Log References

### Completion Notes List

### File List
