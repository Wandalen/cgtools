# Command Tests

### Scope

- **Purpose:** Integration and per-command test specifications for every `shader_chunks` command.
- **Responsibility:** One per-command file, each cross-referencing real test functions.
- **In Scope:** The 6 commands in [`../../../../docs/cli/command/`](../../../../docs/cli/command/readme.md), partitioned across the 4 command groups.
- **Out of Scope:** Parameter-level edge cases (→ [`../param/`](../param/readme.md)); group-interaction corner cases (→ [`../param_group/`](../param_group/readme.md)); cross-command group invariants and workflow compositions (→ [`../command_group/`](../command_group/readme.md)).

---

### Overview Table

| # | File | Covers | Group | Status |
|---|------|--------|-------|--------|
| 1 | [cmd_001_list.md](cmd_001_list.md) | `.list` | [Query](../command_group/01_query.md) | ✅ |
| 2 | [cmd_002_get.md](cmd_002_get.md) | `.get` | [Query](../command_group/01_query.md) | ✅ |
| 3 | [cmd_003_tags.md](cmd_003_tags.md) | `.tags` | [Query](../command_group/01_query.md) | ✅ |
| 4 | [cmd_004_tree.md](cmd_004_tree.md) | `.tree` | [Graph](../command_group/02_graph.md) | ✅ |
| 5 | [cmd_005_compose.md](cmd_005_compose.md) | `.compose` | [Compose](../command_group/03_compose.md) | ✅ |
| 6 | [cmd_006_tunables.md](cmd_006_tunables.md) | `.tunables` | [Parameters](../command_group/04_parameters.md) | ✅ |

**Total:** 6 per-command files

Cross-command workflow compositions (formerly a separate category file)
now live with the group whose semantics they exercise:
[`../command_group/01_query.md`](../command_group/01_query.md) WF-1/WF-2
and [`../command_group/03_compose.md`](../command_group/03_compose.md) WF-1.

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root |
| [`../../../../docs/cli/command/readme.md`](../../../../docs/cli/command/readme.md) | Command documentation source |
