# Command Tests

### Scope

- **Purpose:** Integration and per-command test specifications for this
  crate's 1 command.
- **Responsibility:** One per-command file, cross-referencing real test
  functions.
- **In Scope:** The 1 command in [`../../../docs/cli/command/`](../../../docs/cli/command/readme.md).
- **Out of Scope:** Parameter-level edge cases (→ [`../param/`](../param/readme.md));
  cross-command group invariants and workflow compositions (→
  [`../command_group/`](../command_group/readme.md)).

---

### Overview Table

| # | File | Covers | Group | Status |
|---|------|--------|-------|--------|
| 1 | [cmd_001_render.md](cmd_001_render.md) | `.render` | [Render](../command_group/01_render.md) | ✅ |

**Total:** 1 per-command file (of 8 across the `shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/command/readme.md`](../../../docs/cli/command/readme.md) | Command documentation source |
