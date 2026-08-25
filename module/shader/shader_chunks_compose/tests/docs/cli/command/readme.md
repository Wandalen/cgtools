# Command Tests

### Scope

- **Purpose:** Integration and per-command test specifications for this
  crate's 1 command.
- **Responsibility:** One per-command file, each cross-referencing real
  test functions.
- **In Scope:** The 1 command in [`../../../docs/cli/command/`](../../../docs/cli/command/readme.md).
- **Out of Scope:** Parameter-level edge cases, owned by
  `shader_chunks_query` (→
  [`../../../../../shader_chunks_query/tests/docs/cli/param/readme.md`](../../../../../shader_chunks_query/tests/docs/cli/param/readme.md));
  cross-command group invariants and workflow compositions (→
  [`../command_group/`](../command_group/readme.md)).

---

### Overview Table

| # | File | Covers | Group | Status |
|---|------|--------|-------|--------|
| 1 | [cmd_001_compose.md](cmd_001_compose.md) | `.compose` | [Compose](../command_group/01_compose.md) | ✅ |

**Total:** 1 per-command file (of 9 across the `shader_chunks` family)

Cross-command workflow compositions live with the group whose semantics
they exercise: [`../command_group/01_compose.md`](../command_group/01_compose.md)
WF-1.

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/command/readme.md`](../../../docs/cli/command/readme.md) | Command documentation source |
