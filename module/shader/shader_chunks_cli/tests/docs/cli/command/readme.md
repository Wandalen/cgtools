# Command Tests

### Scope

- **Purpose:** Integration and per-command test specifications for every `shader_chunks_cli` command.
- **Responsibility:** One category integration file plus one per-command file, each cross-referencing real test functions.
- **In Scope:** The 5 commands in [`../../../../docs/cli/command/`](../../../../docs/cli/command/readme.md), all belonging to the single `chunk` category.
- **Out of Scope:** Parameter-level edge cases (→ [`../param/`](../param/readme.md)); cross-command group invariants (→ [`../command_group/`](../command_group/readme.md)).

---

### Overview Table

| # | File | Kind | Covers | Status |
|---|------|------|--------|--------|
| — | [001_chunk.md](001_chunk.md) | Category integration | Cross-command workflows within `chunk` | ✅ |
| 1 | [cmd_001_list.md](cmd_001_list.md) | Per-command | `.list` | ✅ |
| 2 | [cmd_002_get.md](cmd_002_get.md) | Per-command | `.get` | ✅ |
| 3 | [cmd_003_tags.md](cmd_003_tags.md) | Per-command | `.tags` | ✅ |
| 4 | [cmd_004_tree.md](cmd_004_tree.md) | Per-command | `.tree` | ✅ |
| 5 | [cmd_005_compose.md](cmd_005_compose.md) | Per-command | `.compose` | ✅ |

**Total:** 1 category file + 5 per-command files

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root |
| [`../../../../docs/cli/command/readme.md`](../../../../docs/cli/command/readme.md) | Command documentation source |
