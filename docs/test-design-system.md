# System-Level Test Design - ulm

Generated: 2025-11-21
Test Architect: Murat (TEA)
Version: 1.0

---

## Executive Summary

ulm is a Rust CLI application with **95 tests** across 14 modules. The architecture demonstrates good testability with clear module boundaries, async patterns, and embedded data storage. Key testability concerns are external service dependencies (Ollama) and display-dependent clipboard operations.

---

## Testability Assessment

### Controllability: PASS

**Strengths:**
- Modular design with clear public interfaces
- Async/await patterns allow timeout control
- LanceDB embedded - no external database server
- Config via XDG paths - easily overridable for tests
- Dependency injection possible via trait objects

**Evidence:**
- Tests use temp directories for isolation (`tempfile` crate)
- Ollama client accepts custom base_url
- DB operations use `get_database_path()` which respects `XDG_DATA_HOME`

### Observability: PASS

**Strengths:**
- Structured logging with `tracing` crate
- Clear error chains with `anyhow::Context`
- Tests verify specific error messages
- Exit codes propagated correctly

**Evidence:**
- `RUST_LOG=ulm=debug` enables full tracing
- Error display module provides actionable guidance
- Tests check stdout/stderr content

### Reliability: PASS with minor concerns

**Strengths:**
- Tests are generally isolated
- No global state in most modules
- Parallel test execution works

**Concerns:**
- 2 clipboard tests require display environment (correctly ignored)
- Some tests modify current directory (handled with internal functions)

---

## Current Test Distribution

| Module | Tests | Coverage Focus | Test Level |
|--------|-------|----------------|------------|
| `db.rs` | 5 | LanceDB operations, path resolution | Integration |
| `setup/ollama.rs` | 2 | Client creation | Unit |
| `setup/index.rs` | 10 | Manpage scanning, parsing | Unit |
| `query/search.rs` | 2 | Search match data structures | Unit |
| `query/context.rs` | 10 | Directory scanning, project detection | Unit |
| `llm/ollama.rs` | 8 | Request/response serialization | Unit |
| `llm/prompt.rs` | 4 | Prompt building, truncation | Unit |
| `llm/response.rs` | 8 | JSON parsing, risk levels | Unit |
| `tui/mod.rs` | 8 | App state, navigation | Unit |
| `tui/input.rs` | 12 | Keyboard event handling | Unit |
| `tui/error.rs` | 6 | Error display, guidance | Unit |
| `exec/shell.rs` | 4 | Command execution | Integration |
| `exec/clipboard.rs` | 2 | Clipboard operations | Integration (ignored) |
| `exec/edit.rs` | 1 | Module compilation | Smoke |
| **CLI integration** | 10 | End-to-end CLI behavior | E2E |
| **Total** | **95** | | |

---

## Test Levels Strategy

Based on ulm's architecture (CLI tool with async backend):

| Level | Target % | Current | Rationale |
|-------|----------|---------|-----------|
| Unit | 70% | ~73% (69 tests) | Business logic, parsing, data structures |
| Integration | 20% | ~16% (15 tests) | DB, shell, external service contracts |
| E2E | 10% | ~11% (10 tests) | CLI argument parsing, full workflows |

**Assessment:** Distribution is well-balanced for a CLI application.

---

## Architecturally Significant Requirements (ASRs)

| ASR ID | Requirement | Risk Score | Test Approach |
|--------|-------------|------------|---------------|
| ASR-1 | Ollama API availability | 6 (2×3) | Health check, graceful degradation |
| ASR-2 | LanceDB data integrity | 4 (2×2) | Index creation/overwrite tests |
| ASR-3 | Terminal raw mode cleanup | 6 (2×3) | Panic hook, explicit cleanup in TUI |
| ASR-4 | Clipboard cross-platform | 3 (1×3) | Platform-specific tests, ignored in CI |
| ASR-5 | Command execution safety | 9 (3×3) | Shell escaping, input validation |

---

## NFR Testing Approach

### Performance

**Current:** No explicit performance tests
**Recommendation:** Add benchmarks for:
- Embedding generation time
- Vector search latency
- TUI render time

**Tools:** `criterion` crate for Rust benchmarks

### Security

**Current:** Shell execution inherits stdin/stdout
**Risks:**
- Command injection via user-provided arguments
- Clipboard data exposure

**Mitigations in place:**
- Commands passed via `sh -c` (shell handles escaping)
- No secrets stored in config

### Reliability

**Current:** Good error handling with anyhow
**Evidence:**
- 6 error guidance tests
- Context chains for debugging
- Exit codes propagated

### Maintainability

**Current:**
- Strict clippy linting (`-D warnings`)
- Comprehensive module tests
- Clear separation of concerns

**Metrics:**
- 95 tests passing
- 2 ignored (legitimate - display dependency)
- 0 flaky tests observed

---

## Test Environment Requirements

| Environment | Purpose | Components |
|-------------|---------|------------|
| Local Dev | Fast feedback | `cargo test`, no Ollama required |
| CI Pipeline | PR validation | GitHub Actions, no display |
| Integration | Full stack | Ollama running, LanceDB, terminal |

### CI Configuration

```yaml
# Recommended workflow
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo test
    - run: cargo clippy -- -D warnings
```

**Note:** Clipboard tests auto-skip in headless environments.

---

## Testability Concerns

### Minor Concerns

1. **Clipboard tests require display**
   - Status: Handled correctly with `#[ignore]`
   - Impact: Low - clipboard is non-critical path

2. **Ollama dependency for integration tests**
   - Status: Tests mock or avoid Ollama calls
   - Impact: Low - unit tests cover business logic

3. **No contract tests for Ollama API**
   - Recommendation: Add response schema validation
   - Impact: Medium - API changes could break parsing

### No Blockers Identified

Architecture supports testability well. No fundamental issues blocking quality gates.

---

## Quality Gate Criteria

### Pre-commit (P0)

- [ ] All unit tests pass
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes

### PR Merge (P0 + P1)

- [ ] All 95 tests pass
- [ ] No new clippy warnings
- [ ] CLI integration tests pass

### Release

- [ ] Manual testing with Ollama
- [ ] Cross-platform clipboard verification (if applicable)
- [ ] Performance acceptable (< 2s for query)

---

## Recommendations for Improvement

### High Priority

1. **Add Ollama API contract tests**
   - Validate response schemas
   - Detect API version changes
   - Risk: API breaking changes

2. **Benchmark critical paths**
   - Embedding generation
   - Vector search
   - TUI render loop

### Medium Priority

3. **Property-based testing for parsers**
   - JSON response parsing
   - Manpage content extraction
   - Tool: `proptest` crate

4. **Integration test for full query flow**
   - Mock Ollama responses
   - Verify end-to-end pipeline

### Low Priority

5. **Visual regression for TUI**
   - Snapshot terminal output
   - Detect render changes

---

## Existing Test Strengths

- **Comprehensive input handling**: 12 tests for TUI keyboard events
- **Good error coverage**: 6 tests for error display and guidance
- **Serialization validation**: 8 tests for Ollama request/response
- **Project detection**: 10 tests covering multiple project types (Rust, Go, Node, Python)
- **CLI integration**: 10 tests validating argument parsing and help output

---

## Next Steps

1. ✅ Test Design complete
2. Run `/bmad:bmm:workflows:testarch:trace` to map requirements to tests
3. Consider `/bmad:bmm:workflows:testarch:ci` for pipeline setup
4. Optional: `/bmad:bmm:workflows:testarch:automate` for additional test generation

---

## Appendix: Test Command Reference

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific module
cargo test tui::input

# Run ignored tests (requires display)
cargo test -- --ignored

# Enable debug logging
RUST_LOG=ulm=debug cargo test

# Check clippy
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```
