# Command Group Tests

### Scope

- **Purpose:** Cross-command invariant test specifications for this
  crate's 2 command groups.
- **Responsibility:** One file per command group, verifying the group's
  Semantic Coherence Test holds for every current member command.
- **In Scope:** The 2 command groups in [`../../../docs/cli/command_group/`](../../../docs/cli/command_group/readme.md).
- **Out of Scope:** Per-command behavior (→ [`../command/`](../command/readme.md)).

---

### Overview Table

| # | File | Command Group | Status |
|---|------|----------------|--------|
| 1 | [01_query.md](01_query.md) | Query | ✅ |
| 2 | [02_graph.md](02_graph.md) | Graph | ✅ |

**Total:** 2 command group test specs (of 7 across the `shader_chunks` family)

The rendered-grouping invariant —
[`../../../../../shader_chunks/tests/cli_subprocess_test.rs`](../../../../../shader_chunks/tests/cli_subprocess_test.rs)`::top_level_help_groups_commands_by_responsibility`
— asserts the help screen's group order and membership matches the
documented partition across all 7 groups.

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/command_group/readme.md`](../../../docs/cli/command_group/readme.md) | Command group documentation source |
