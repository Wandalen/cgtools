# Command Tests

### Scope

- **Purpose:** Integration and per-command test specifications for this
  crate's 4 commands.
- **Responsibility:** One per-command file, each cross-referencing real
  test functions.
- **In Scope:** The 4 commands in [`../../../docs/cli/command/`](../../../docs/cli/command/readme.md).
- **Out of Scope:** Parameter-level edge cases (→ [`../param/`](../param/readme.md));
  group-interaction corner cases (→ [`../param_group/`](../param_group/readme.md));
  cross-command group invariants and workflow compositions (→
  [`../command_group/`](../command_group/readme.md)).

---

### Overview Table

| # | File | Covers | Group | Status |
|---|------|--------|-------|--------|
| 1 | [cmd_001_list.md](cmd_001_list.md) | `.list` | [Query](../command_group/01_query.md) | ✅ |
| 2 | [cmd_002_get.md](cmd_002_get.md) | `.get` | [Query](../command_group/01_query.md) | ✅ |
| 3 | [cmd_003_tags.md](cmd_003_tags.md) | `.tags` | [Query](../command_group/01_query.md) | ✅ |
| 4 | [cmd_004_tree.md](cmd_004_tree.md) | `.tree` | [Graph](../command_group/02_graph.md) | ✅ |

**Total:** 4 per-command files (of 9 across the `shader_chunks` family)

Cross-command workflow compositions live with the group whose semantics
they exercise: [`../command_group/01_query.md`](../command_group/01_query.md)
WF-1/WF-2.

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/command/readme.md`](../../../docs/cli/command/readme.md) | Command documentation source |
