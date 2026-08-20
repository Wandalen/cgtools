# Command Group Tests

### Scope

- **Purpose:** Cross-command invariant test specifications for this
  crate's 1 command group.
- **Responsibility:** One file per command group, verifying the group's
  Semantic Coherence Test holds for every current member command.
- **In Scope:** The 1 command group in [`../../../docs/cli/command_group/`](../../../docs/cli/command_group/readme.md).
- **Out of Scope:** Per-command behavior (→ [`../command/`](../command/readme.md)).

---

### Overview Table

| # | File | Command Group | Status |
|---|------|----------------|--------|
| 1 | [01_render.md](01_render.md) | Render | ✅ |

**Total:** 1 command group test spec (of 7 across the `shader_chunks`
family)

The rendered-grouping invariant —
[`../../../../../shader_chunks/tests/cli_subprocess_test.rs`](../../../../../shader_chunks/tests/cli_subprocess_test.rs)`::top_level_help_groups_commands_by_responsibility`
— asserts the help screen's group order and membership matches the
documented partition across all 7 groups; `../../render_cli_test.rs::subprocess_help_lists_the_render_group`
asserts the same grouping through the standalone binary.

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/command_group/readme.md`](../../../docs/cli/command_group/readme.md) | Command group documentation source |
