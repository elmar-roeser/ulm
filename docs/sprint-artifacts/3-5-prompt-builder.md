# Story 3.5: Prompt Builder

Status: done

## Story

As a developer,
I want to build a complete prompt for the LLM,
so that it can generate contextually relevant command suggestions.

## Acceptance Criteria

1. **AC1:** Include system instructions
2. **AC2:** Include manpage content
3. **AC3:** Include directory context
4. **AC4:** Include user query
5. **AC5:** Request JSON output format
6. **AC6:** Total prompt < 12000 tokens
7. **AC7:** `cargo build` succeeds without errors
8. **AC8:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create prompt module (AC: 1, 2, 3, 4, 5)
  - [ ] Create llm/prompt.rs
  - [ ] Define system instructions
  - [ ] Add build_prompt function
  - [ ] Include JSON format request

- [ ] Task 2: Add token estimation (AC: 6)
  - [ ] Estimate token count
  - [ ] Truncate content if needed

- [ ] Task 3: Verify build (AC: 7, 8)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// llm/prompt.rs
pub fn build_prompt(
    query: &str,
    manpage_content: &str,
    context: &DirectoryContext,
) -> String;
```

### Prompt Structure

```
System: You are a command-line expert assistant...
[JSON format instructions]

Context:
{directory context}

Manpage Content:
{truncated manpage}

User Query: {query}
```

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Story-3.5]
- [Source: docs/epics.md#Story-3.5]

## Dev Agent Record

### Context Reference

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Version | Description |
|------|---------|-------------|
| 2025-11-21 | 1.0 | Initial story creation |
