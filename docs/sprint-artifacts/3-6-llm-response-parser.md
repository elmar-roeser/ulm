# Story 3.6: LLM Response Parser

Status: done

## Story

As a developer,
I want to parse the LLM's JSON response into structured data,
so that I can display command suggestions to the user.

## Acceptance Criteria

1. **AC1:** Deserialize JSON to Vec<CommandSuggestion>
2. **AC2:** Each suggestion has command, title, explanation, risk_level
3. **AC3:** Handle malformed JSON with error
4. **AC4:** Validate command is not empty
5. **AC5:** Default risk_level to Safe if missing
6. **AC6:** `cargo build` succeeds without errors
7. **AC7:** `cargo clippy -- -D warnings` passes

## Tasks / Subtasks

- [ ] Task 1: Create response module (AC: 1, 2)
  - [ ] Create llm/response.rs
  - [ ] Define CommandSuggestion struct
  - [ ] Define RiskLevel enum

- [ ] Task 2: Implement parser (AC: 3, 4, 5)
  - [ ] Add parse_suggestions function
  - [ ] Handle JSON errors
  - [ ] Validate fields
  - [ ] Default risk level

- [ ] Task 3: Verify build (AC: 6, 7)
  - [ ] Run `cargo build`
  - [ ] Run `cargo clippy -- -D warnings`

## Dev Notes

### Architecture Patterns

```rust
// llm/response.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command: String,
    pub title: String,
    pub explanation: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Moderate,
    Destructive,
}

pub fn parse_suggestions(response: &str) -> Result<Vec<CommandSuggestion>>;
```

### JSON Format

```json
{
  "suggestions": [
    {
      "command": "find . -size +100M",
      "title": "Find large files",
      "explanation": "Finds files larger than 100MB",
      "risk_level": "safe"
    }
  ]
}
```

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Story-3.6]
- [Source: docs/epics.md#Story-3.6]

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
