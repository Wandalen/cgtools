# Command Group Tests

### Scope

- **Purpose:** Cross-command invariant test specifications for every command group `shader_chunks` declares.
- **Responsibility:** One file per command group, verifying the group's Semantic Coherence Test holds for every current member command.
- **In Scope:** The 4 command groups in [`../../../../docs/cli/command_group/`](../../../../docs/cli/command_group/readme.md).
- **Out of Scope:** Per-command behavior (→ [`../command/`](../command/readme.md)).

---

### Overview Table

| # | File | Command Group | Status |
|---|------|----------------|--------|
| 1 | [01_query.md](01_query.md) | Query | ✅ |
| 2 | [02_graph.md](02_graph.md) | Graph | ✅ |
| 3 | [03_compose.md](03_compose.md) | Compose | ✅ |
| 4 | [04_parameters.md](04_parameters.md) | Parameters | ✅ |

**Total:** 4 command group test specs

The rendered-grouping invariant shared by all 3 files —
`cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility`
— asserts the help screen's group order and membership matches the
documented partition.
