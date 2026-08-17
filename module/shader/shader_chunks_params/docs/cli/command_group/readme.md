# Command Groups

### Scope

- **Purpose:** Documents every command group this crate declares — the
  same grouping the top-level help screen renders.
- **Responsibility:** One dedicated file per group — pattern, coherence
  test, invariants, membership.
- **In Scope:** The 1 group this crate contributes — `Parameters`.
- **Out of Scope:** Per-command syntax and parameters (→ [`../command/`](../command/readme.md));
  the other 5 groups of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Group | Members | Status |
|---|------|-------|---------|--------|
| 1 | [01_parameters.md](01_parameters.md) | Parameters | `.tunables` | ✅ |

**Total:** 1 command group (of 6 across the `shader_chunks` family)

**Partition note:** this crate's sole command, `.tunables`, is the only
member of the Parameters group — a deliberate single-member group (data
source, not command count, is the boundary; see
[`01_parameters.md`](01_parameters.md#why-not-merge-into-query)). The
full 6-group, 8-command partition (spanning all 5 leaf CLIs) is stated
in [the family index](../../../../shader_chunks/docs/cli/readme.md).
The help screen (`shader_chunks_cli_core/src/lib.rs`, `help_print`)
renders all 6 groups together; a drift between the family index's table
and the help output is a documentation bug.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command/readme.md](../command/readme.md) | Member command definition |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command_group/readme.md](../../../tests/docs/cli/command_group/readme.md) | Group-level test specifications |
| [../../../../shader_chunks/tests/cli_subprocess_test.rs](../../../../shader_chunks/tests/cli_subprocess_test.rs) | `top_level_help_groups_commands_by_responsibility` asserts the rendered grouping in the aggregated binary |
