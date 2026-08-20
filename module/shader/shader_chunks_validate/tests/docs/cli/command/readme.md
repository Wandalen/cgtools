# Command Tests

### Scope

- **Purpose:** Integration and per-command test specifications for this
  crate's 1 command.
- **Responsibility:** One per-command file, cross-referencing real test
  functions.
- **In Scope:** The 1 command in [`../../../docs/cli/command/`](../../../docs/cli/command/readme.md).
- **Out of Scope:** Engine-level check behavior (owned by
  `shader_chunks_validate_core`'s own tests, cited from
  [`../command_group/`](../command_group/readme.md) per the
  `_core`-split precedent); cross-command group invariants and workflow
  compositions (→ [`../command_group/`](../command_group/readme.md)).

---

### Overview Table

| # | File | Covers | Group | Status |
|---|------|--------|-------|--------|
| 1 | [cmd_001_validate.md](cmd_001_validate.md) | `.validate` | [Validate](../command_group/01_validate.md) | ✅ |

**Total:** 1 per-command file (of 9 across the `shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/command/readme.md`](../../../docs/cli/command/readme.md) | Command documentation source |
