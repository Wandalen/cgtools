# Command Groups

### Scope

- **Purpose:** Documents every command group this crate declares — the
  same grouping the top-level help screen renders.
- **Responsibility:** One dedicated file per group — pattern, coherence
  test, invariants, membership.
- **In Scope:** The 2 groups partitioning this crate's 4 commands —
  `Query`, `Graph`.
- **Out of Scope:** Per-command syntax and parameters (→ [`../command/`](../command/readme.md));
  the other 4 groups of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Group | Members | Status |
|---|------|-------|---------|--------|
| 1 | [01_query.md](01_query.md) | Query | `.list`, `.get`, `.tags` | ✅ |
| 2 | [02_graph.md](02_graph.md) | Graph | `.tree` | ✅ |

**Total:** 2 command groups (of 6 across the `shader_chunks` family)

**Partition note:** every one of this crate's 4 commands belongs to
exactly one of these 2 groups — no command is outside the partition,
none is in two groups. The full 6-group, 8-command partition (spanning
all 5 leaf CLIs) is stated in
[the family index](../../../../shader_chunks/docs/cli/readme.md). The
help screen (`shader_chunks_cli_core/src/lib.rs`, `help_print`) renders
all 6 groups together; a drift between the family index's table and the
help output is a documentation bug.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command/readme.md](../command/readme.md) | Member command definitions |
| [../param_group/readme.md](../param_group/readme.md) | Parameter groups shared by the Query group's engine |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command_group/readme.md](../../../tests/docs/cli/command_group/readme.md) | Group-level test specifications |
| [../../../../shader_chunks/tests/cli_subprocess_test.rs](../../../../shader_chunks/tests/cli_subprocess_test.rs) | `top_level_help_groups_commands_by_responsibility` asserts the rendered grouping in the aggregated binary |
