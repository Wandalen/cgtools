# Command Groups

### Scope

- **Purpose:** Documents every command group `shader_chunks` declares — the same grouping the top-level help screen renders.
- **Responsibility:** One dedicated file per group — pattern, coherence test, invariants, membership.
- **In Scope:** The 4 groups partitioning the 6 commands.
- **Out of Scope:** Per-command syntax and parameters (→ [`../command/`](../command/readme.md)).

---

### Overview Table

| # | File | Group | Members | Status |
|---|------|-------|---------|--------|
| 1 | [01_query.md](01_query.md) | Query | `.list`, `.get`, `.tags` | ✅ |
| 2 | [02_graph.md](02_graph.md) | Graph | `.tree` | ✅ |
| 3 | [03_compose.md](03_compose.md) | Compose | `.compose` | ✅ |
| 4 | [04_parameters.md](04_parameters.md) | Parameters | `.tunables` | ✅ |

**Total:** 4 command groups

**Complete partition:** every one of the 6 commands belongs to exactly one
group — no command is outside the partition, none is in two groups. The
help screen (`src/cli.rs`, `help_print`) renders exactly these 4 groups
with exactly this membership; a drift between this table and the help
output is a documentation bug.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../command/readme.md](../command/readme.md) | Member command definitions |
| [../param_group/readme.md](../param_group/readme.md) | Parameter groups shared by the Query group's engine |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command_group/readme.md](../../../tests/docs/cli/command_group/readme.md) | Group-level test specifications |
| [../../../tests/cli_subprocess_test.rs](../../../tests/cli_subprocess_test.rs) | `top_level_help_groups_commands_by_responsibility` asserts the rendered grouping |
