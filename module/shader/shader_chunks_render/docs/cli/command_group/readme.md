# Command Groups

### Scope

- **Purpose:** Documents every command group this crate declares — the
  same grouping the top-level help screen renders.
- **Responsibility:** One dedicated file per group — pattern, coherence
  test, invariants, membership.
- **In Scope:** The 1 group partitioning this crate's 1 command —
  `Render`.
- **Out of Scope:** Per-command syntax and parameters (→ [`../command/`](../command/readme.md));
  the other 5 groups of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Group | Members | Status |
|---|------|-------|---------|--------|
| 1 | [01_render.md](01_render.md) | Render | `.render` | ✅ |

**Total:** 1 command group (of 7 across the `shader_chunks` family)

**Partition note:** this crate's sole command belongs to this sole
group. The full 7-group, 9-command partition (spanning all 6 leaf CLIs)
is stated in [the family index](../../../../shader_chunks/docs/cli/readme.md).
The help screen (`shader_chunks_cli_core/src/lib.rs`, `help_print`)
renders all 7 groups together; a drift between the family index's table
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
| [../../../tests/render_cli_test.rs](../../../tests/render_cli_test.rs) | `subprocess_help_lists_the_render_group` asserts this group's rendering in the standalone binary |
| [../../../../shader_chunks/tests/cli_subprocess_test.rs](../../../../shader_chunks/tests/cli_subprocess_test.rs) | `top_level_help_groups_commands_by_responsibility` asserts `Preview` < `Render` group order in the aggregated binary |
